//! Qdrant vector memory with pluggable embedding providers (Phase 1.5).

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use async_trait::async_trait;
use qdrant_client::qdrant::{
    CreateCollection, Distance, PointStruct, SearchPoints, UpsertPoints, Value as QdrantValue,
    VectorParams, VectorsConfig,
};
use qdrant_client::Qdrant;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::embedding::{resolve_from_env, resolve_provider, EmbeddingError, EmbeddingProvider};

use super::error::MemoryError;
use super::namespace::durable_key;
use super::provider::VectorStoreProvider;

const COLLECTION: &str = "arcflow_memory";

/// Qdrant point ids must be valid UUIDs; derive deterministically from storage key.
fn point_id_from_key(storage_key: &str) -> String {
    let digest = Sha256::digest(storage_key.as_bytes());
    let mut id_bytes = [0u8; 16];
    id_bytes.copy_from_slice(&digest[..16]);
    Uuid::from_bytes(id_bytes).to_string()
}

fn map_qdrant_client_error(err: impl std::fmt::Display) -> MemoryError {
    let reason = err.to_string();
    let lower = reason.to_lowercase();
    if lower.contains("connect")
        || lower.contains("connection refused")
        || lower.contains("transport")
        || lower.contains("unavailable")
    {
        MemoryError::InfrastructureUnavailable {
            backend: "qdrant".into(),
            suggestion: reason,
        }
    } else {
        MemoryError::OperationFailed { reason }
    }
}

fn map_embedding(err: EmbeddingError) -> MemoryError {
    MemoryError::OperationFailed {
        reason: err.to_string(),
    }
}

/// Qdrant-backed vector store.
pub struct QdrantVectorStore {
    client: Option<Qdrant>,
    dim: usize,
}

impl Default for QdrantVectorStore {
    fn default() -> Self {
        Self::new(8)
    }
}

impl QdrantVectorStore {
    /// Creates a store with vector dimension `dim`.
    pub fn new(dim: usize) -> Self {
        Self { client: None, dim }
    }

    async fn client(&mut self) -> Result<&Qdrant, MemoryError> {
        if self.client.is_none() {
            let url = env::var("ARCFLOW_QDRANT_URL").map_err(|_| {
                MemoryError::InfrastructureUnavailable {
                    backend: "qdrant".into(),
                    suggestion: "Set ARCFLOW_QDRANT_URL and start Qdrant.".into(),
                }
            })?;
            let client = Qdrant::from_url(&url)
                .skip_compatibility_check()
                .build()
                .map_err(map_qdrant_client_error)?;
            let create_result = client
                .create_collection(CreateCollection {
                    collection_name: COLLECTION.into(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                            VectorParams {
                                size: self.dim as u64,
                                distance: Distance::Cosine.into(),
                                ..Default::default()
                            },
                        )),
                    }),
                    ..Default::default()
                })
                .await;
            if let Err(ref err) = create_result {
                let msg = err.to_string().to_lowercase();
                if !msg.contains("already exists") {
                    return Err(map_qdrant_client_error(err));
                }
            }
            self.client = Some(client);
        }
        self.client.as_ref().ok_or(MemoryError::OperationFailed {
            reason: "qdrant client missing after connect".into(),
        })
    }
}

#[async_trait]
impl VectorStoreProvider for QdrantVectorStore {
    async fn upsert(
        &mut self,
        collection: &str,
        point_id: &str,
        vector: &[f32],
        payload: &[u8],
    ) -> Result<(), MemoryError> {
        let client = self.client().await?;
        let mut payload_map = HashMap::new();
        payload_map.insert(
            "payload".to_string(),
            QdrantValue::from(String::from_utf8_lossy(payload).to_string()),
        );
        let qdrant_id = point_id_from_key(point_id);
        let point = PointStruct::new(qdrant_id, vector.to_vec(), payload_map);
        client
            .upsert_points(UpsertPoints {
                collection_name: collection.into(),
                points: vec![point],
                ..Default::default()
            })
            .await
            .map_err(map_qdrant_client_error)?;
        Ok(())
    }

    async fn search(
        &mut self,
        collection: &str,
        vector: &[f32],
        limit: usize,
    ) -> Result<Vec<Vec<u8>>, MemoryError> {
        let client = self.client().await?;
        let results = client
            .search_points(SearchPoints {
                collection_name: collection.into(),
                vector: vector.to_vec(),
                limit: limit as u64,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await
            .map_err(map_qdrant_client_error)?;
        let mut out = Vec::new();
        for point in results.result {
            if let Some(payload) = point.payload.get("payload") {
                if let Some(s) = payload.as_str() {
                    out.push(s.as_bytes().to_vec());
                }
            }
        }
        Ok(out)
    }
}

/// High-level vector memory API.
pub struct VectorMemory {
    store: QdrantVectorStore,
    provider: Arc<dyn EmbeddingProvider>,
}

impl Default for VectorMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorMemory {
    /// Uses `ARCFLOW_EMBEDDING_PROVIDER`, or explicit `stub` in tests/dev mode.
    pub fn new() -> Self {
        Self::from_env().unwrap_or_else(|_| {
            Self::from_provider_spec("stub").expect("stub provider always available")
        })
    }

    /// Builds vector memory from a provider spec such as `openai/text-embedding-3-small`.
    pub fn from_provider_spec(spec: &str) -> Result<Self, EmbeddingError> {
        let provider = resolve_provider(spec)?;
        Ok(Self {
            store: QdrantVectorStore::new(provider.dimensions()),
            provider,
        })
    }

    /// Resolves the embedding provider from environment variables.
    pub fn from_env() -> Result<Self, EmbeddingError> {
        let provider = resolve_from_env()?;
        Ok(Self {
            store: QdrantVectorStore::new(provider.dimensions()),
            provider,
        })
    }

    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, MemoryError> {
        let mut vectors = self
            .provider
            .embed(&[text.to_string()])
            .await
            .map_err(map_embedding)?;
        vectors.pop().ok_or(MemoryError::OperationFailed {
            reason: "embedding provider returned no vectors".into(),
        })
    }

    /// Stores text under namespace using semantic embedding of the payload.
    pub async fn write(
        &mut self,
        namespace: &str,
        logical_key: &str,
        value: &[u8],
    ) -> Result<(), MemoryError> {
        let storage_key = durable_key(namespace, logical_key);
        let text = String::from_utf8_lossy(value).into_owned();
        let vector = self.embed_text(&text).await?;
        self.store
            .upsert(COLLECTION, &storage_key, &vector, value)
            .await
    }

    /// Retrieves nearest payload for a semantic query (`logical_key` text).
    pub async fn read(
        &mut self,
        namespace: &str,
        logical_key: &str,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        let _ = namespace;
        let vector = self.embed_text(logical_key).await?;
        let hits = self.store.search(COLLECTION, &vector, 1).await?;
        Ok(hits.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::stub_embedding;

    #[test]
    fn stub_embedding_reexported_compat() {
        let v = stub_embedding(b"hello", 4);
        assert_eq!(v.len(), 4);
    }

    #[test]
    fn vector_memory_stub_provider_dimensions() {
        let mem = VectorMemory::from_provider_spec("stub/16").expect("stub");
        assert_eq!(mem.provider.dimensions(), 16);
    }
}
