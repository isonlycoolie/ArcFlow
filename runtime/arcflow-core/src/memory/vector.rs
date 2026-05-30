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

use super::chunking::{ChunkStrategy, RecursiveCharacterSplitter};
use super::error::MemoryError;
use super::hybrid::{sparse_lexical_score, HybridHit, HybridRetriever, DEFAULT_DENSE_WEIGHT, DEFAULT_SPARSE_WEIGHT};
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
