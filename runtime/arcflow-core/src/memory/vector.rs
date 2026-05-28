//! Qdrant vector memory with deterministic stub embeddings in tests.

use std::collections::HashMap;
use std::env;

use async_trait::async_trait;
use qdrant_client::qdrant::{
    CreateCollection, Distance, PointStruct, SearchPoints, UpsertPoints, Value as QdrantValue,
    VectorParams, VectorsConfig,
};
use qdrant_client::Qdrant;
use sha2::{Digest, Sha256};
use uuid::Uuid;

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

/// Deterministic stub embedding from key bytes (Sprint 4 — no external API).
pub fn stub_embedding(seed: &[u8], dim: usize) -> Vec<f32> {
    let mut out = vec![0.0_f32; dim];
    for (i, item) in out.iter_mut().enumerate() {
        let byte = seed.get(i % seed.len()).copied().unwrap_or(0);
        *item = (byte as f32) / 255.0;
    }
    out
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
}

impl Default for VectorMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorMemory {
    /// Default 8-dimensional stub vectors.
    pub fn new() -> Self {
        Self {
            store: QdrantVectorStore::new(8),
        }
    }

    /// Stores text under namespace using stub embedding of the logical key.
    pub async fn write(
        &mut self,
        namespace: &str,
        logical_key: &str,
        value: &[u8],
    ) -> Result<(), MemoryError> {
        let storage_key = durable_key(namespace, logical_key);
        let vector = stub_embedding(storage_key.as_bytes(), self.store.dim);
        self.store
            .upsert(COLLECTION, &storage_key, &vector, value)
            .await
    }

    /// Retrieves nearest payload for the same logical key embedding.
    pub async fn read(
        &mut self,
        namespace: &str,
        logical_key: &str,
    ) -> Result<Option<Vec<u8>>, MemoryError> {
        let storage_key = durable_key(namespace, logical_key);
        let vector = stub_embedding(storage_key.as_bytes(), self.store.dim);
        let hits = self.store.search(COLLECTION, &vector, 1).await?;
        Ok(hits.into_iter().next())
    }
}
