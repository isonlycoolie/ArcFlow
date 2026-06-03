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
use crate::rcs::types::{
    MemoryChunkingConfig, MemoryConfig, MemoryRetrievalConfig, MemoryType, RerankProviderSpec,
    RetrievalModeSpec,
};

use super::chunking::{
    extract_chunk_metadata, ChunkMetadata, ChunkStrategy, RecursiveCharacterSplitter,
};
use super::error::MemoryError;
use super::hybrid::{
    sparse_lexical_score, HybridHit, HybridRetriever, DEFAULT_DENSE_WEIGHT, DEFAULT_SPARSE_WEIGHT,
};
use super::namespace::durable_key;
use super::provider::VectorStoreProvider;
use super::rerank::resolve_rerank_provider;

const COLLECTION: &str = "arcflow_memory";

fn sparse_term_indices(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

/// Maps RCS `MemoryConfig` to runtime vector settings (Phase 2.5).
pub fn vector_config_from_memory(config: &MemoryConfig) -> VectorMemoryConfig {
    let mut out = VectorMemoryConfig::default();
    if let Some(chunking) = &config.chunking {
        out.chunking = ChunkingConfig {
            chunk_size: chunking.chunk_size,
            overlap: chunking.overlap,
        };
    }
    if let Some(retrieval) = &config.retrieval {
        out.mode = match retrieval.mode {
            RetrievalModeSpec::Hybrid => RetrievalMode::Hybrid,
            RetrievalModeSpec::Dense => RetrievalMode::Dense,
        };
        out.retrieval = HybridRetrievalConfig {
            dense_weight: retrieval.dense_weight,
            sparse_weight: retrieval.sparse_weight,
        };
        out.rerank = retrieval.rerank.map(|r| match r {
            RerankProviderSpec::Cohere => "cohere".into(),
            RerankProviderSpec::Local => "local".into(),
        });
        out.top_k = retrieval.top_k;
    }
    out
}

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

    /// Dense search returning cosine score and payload text for hybrid fusion.
    pub async fn search_scored(
        &mut self,
        collection: &str,
        vector: &[f32],
        limit: usize,
    ) -> Result<Vec<(f32, String)>, MemoryError> {
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
            let score = point.score;
            if let Some(payload) = point.payload.get("payload") {
                if let Some(s) = payload.as_str() {
                    out.push((score, s.to_string()));
                }
            }
        }
        Ok(out)
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
        let meta = extract_chunk_metadata(String::from_utf8_lossy(payload).as_ref());
        if let Some(title) = meta.title {
            payload_map.insert("title".to_string(), QdrantValue::from(title));
        }
        if let Some(source) = meta.source {
            payload_map.insert("source".to_string(), QdrantValue::from(source));
        }
        if let Some(page) = meta.page {
            payload_map.insert("page".to_string(), QdrantValue::from(page as i64));
        }
        if env::var("ARCFLOW_QDRANT_HYBRID")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
        {
            let sparse_terms = sparse_term_indices(String::from_utf8_lossy(payload).as_ref());
            payload_map.insert(
                "sparse_terms".to_string(),
                QdrantValue::from(sparse_terms.join(" ")),
            );
        }
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

/// Chunking settings for document ingest.
#[derive(Clone, Debug)]
pub struct ChunkingConfig {
    pub chunk_size: usize,
    pub overlap: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            overlap: 64,
        }
    }
}

/// Hybrid dense/sparse fusion weights.
#[derive(Clone, Debug)]
pub struct HybridRetrievalConfig {
    pub dense_weight: f32,
    pub sparse_weight: f32,
}

impl Default for HybridRetrievalConfig {
    fn default() -> Self {
        Self {
            dense_weight: DEFAULT_DENSE_WEIGHT,
            sparse_weight: DEFAULT_SPARSE_WEIGHT,
        }
    }
}

/// Retrieval strategy for semantic search.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RetrievalMode {
    #[default]
    Dense,
    Hybrid,
}

/// Vector memory runtime configuration (chunking + retrieval).
#[derive(Clone, Debug)]
pub struct VectorMemoryConfig {
    pub chunking: ChunkingConfig,
    pub retrieval: HybridRetrievalConfig,
    pub mode: RetrievalMode,
    pub rerank: Option<String>,
    pub top_k: Option<u32>,
}

impl Default for VectorMemoryConfig {
    fn default() -> Self {
        Self {
            chunking: ChunkingConfig::default(),
            retrieval: HybridRetrievalConfig::default(),
            mode: RetrievalMode::Dense,
            rerank: None,
            top_k: None,
        }
    }
}

/// High-level vector memory API.
pub struct VectorMemory {
    store: QdrantVectorStore,
    provider: Arc<dyn EmbeddingProvider>,
    config: VectorMemoryConfig,
    splitter: RecursiveCharacterSplitter,
    hybrid: HybridRetriever,
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
            Self::from_provider_spec("stub").unwrap_or_else(|err| {
                eprintln!("arcflow: stub embedding provider unavailable: {err}");
                std::process::exit(1);
            })
        })
    }
    pub fn from_provider_spec(spec: &str) -> Result<Self, EmbeddingError> {
        let provider = resolve_provider(spec)?;
        Ok(Self::with_provider(provider, VectorMemoryConfig::default()))
    }

    /// Resolves the embedding provider from environment variables.
    pub fn from_env() -> Result<Self, EmbeddingError> {
        let provider = resolve_from_env()?;
        Ok(Self::with_provider(provider, VectorMemoryConfig::default()))
    }

    /// Builds vector memory with chunking and hybrid retrieval settings.
    pub fn with_config(spec: &str, config: VectorMemoryConfig) -> Result<Self, EmbeddingError> {
        let provider = resolve_provider(spec)?;
        Ok(Self::with_provider(provider, config))
    }

    fn with_provider(provider: Arc<dyn EmbeddingProvider>, config: VectorMemoryConfig) -> Self {
        let splitter =
            RecursiveCharacterSplitter::new(config.chunking.chunk_size, config.chunking.overlap);
        let hybrid = HybridRetriever::new(
            config.retrieval.dense_weight,
            config.retrieval.sparse_weight,
        );
        Self {
            store: QdrantVectorStore::new(provider.dimensions()),
            provider,
            config,
            splitter,
            hybrid,
        }
    }

    /// Active configuration (chunking + retrieval).
    pub fn config(&self) -> &VectorMemoryConfig {
        &self.config
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
        let hits = self.search(namespace, logical_key, 1).await?;
        Ok(hits.into_iter().next())
    }

    /// Chunks `text`, embeds each chunk, and upserts under `{key}#chunkN`.
    pub async fn write_document(
        &mut self,
        namespace: &str,
        key: &str,
        text: &str,
    ) -> Result<usize, MemoryError> {
        let _meta = extract_chunk_metadata(text);
        let chunks = self.splitter.split(text);
        let count = chunks.len();
        for (idx, chunk) in chunks.into_iter().enumerate() {
            let chunk_key = format!("{key}#chunk{idx}");
            self.write(namespace, &chunk_key, chunk.as_bytes()).await?;
        }
        Ok(count)
    }

    /// Semantic search with optional rerank when configured.
    pub async fn search_reranked(
        &mut self,
        namespace: &str,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<Vec<u8>>, MemoryError> {
        let prefetch = top_k.saturating_mul(4).max(top_k);
        let hits = self.search(namespace, query, prefetch).await?;
        let Some(rerank_spec) = self.config.rerank.as_deref() else {
            return Ok(hits.into_iter().take(top_k).collect());
        };
        let provider =
            resolve_rerank_provider(rerank_spec).map_err(|e| MemoryError::OperationFailed {
                reason: e.to_string(),
            })?;
        let docs: Vec<String> = hits
            .iter()
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .collect();
        let ranked = provider.rerank(query, &docs, top_k).await.map_err(|e| {
            MemoryError::OperationFailed {
                reason: e.to_string(),
            }
        })?;
        Ok(ranked.into_iter().map(|r| r.text.into_bytes()).collect())
    }

    /// Builds vector memory from agent memory config (Phase 2.5).
    pub fn from_memory_config(config: &MemoryConfig) -> Result<Self, EmbeddingError> {
        if config.memory_type != MemoryType::Vector {
            return Err(EmbeddingError::InvalidSpec {
                reason: "memory_type must be Vector".into(),
            });
        }
        let spec = config.embedding.as_deref().unwrap_or("stub/8");
        Self::with_config(spec, vector_config_from_memory(config))
    }

    /// Semantic search returning up to `top_k` payload blobs.
    pub async fn search(
        &mut self,
        namespace: &str,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<Vec<u8>>, MemoryError> {
        let _ = namespace;
        let vector = self.embed_text(query).await?;
        match self.config.mode {
            RetrievalMode::Dense => self.store.search(COLLECTION, &vector, top_k).await,
            RetrievalMode::Hybrid => {
                let candidates = self
                    .store
                    .search_scored(COLLECTION, &vector, top_k.saturating_mul(3).max(top_k))
                    .await?;
                let hits: Vec<HybridHit> = candidates
                    .iter()
                    .enumerate()
                    .map(|(idx, (dense_score, text))| HybridHit {
                        point_id: idx.to_string(),
                        dense_score: *dense_score,
                        sparse_score: sparse_lexical_score(query, text),
                    })
                    .collect();
                let ranked = self.hybrid.rank(hits, top_k);
                Ok(ranked
                    .into_iter()
                    .filter_map(|(id, _)| {
                        id.parse::<usize>().ok().and_then(|i| {
                            candidates.get(i).map(|(_, text)| text.as_bytes().to_vec())
                        })
                    })
                    .collect())
            }
        }
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
    fn vector_memory_config_defaults() {
        let cfg = VectorMemoryConfig::default();
        assert_eq!(cfg.chunking.chunk_size, 512);
        assert_eq!(cfg.mode, RetrievalMode::Dense);
    }

    #[test]
    fn vector_memory_hybrid_mode_configured() {
        let mut cfg = VectorMemoryConfig::default();
        cfg.mode = RetrievalMode::Hybrid;
        let mem = VectorMemory::with_config("stub/8", cfg).expect("stub");
        assert_eq!(mem.config().mode, RetrievalMode::Hybrid);
    }
}
