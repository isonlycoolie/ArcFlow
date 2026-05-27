//! Vector store provider abstraction (ADR-008).

use async_trait::async_trait;

use super::error::MemoryError;

/// Pluggable vector backend (Sprint 4: Qdrant only).
#[async_trait]
pub trait VectorStoreProvider: Send + Sync {
    /// Upserts a vector with payload bytes.
    async fn upsert(
        &mut self,
        collection: &str,
        point_id: &str,
        vector: &[f32],
        payload: &[u8],
    ) -> Result<(), MemoryError>;

    /// Nearest-neighbor search; returns payload bytes.
    async fn search(
        &mut self,
        collection: &str,
        vector: &[f32],
        limit: usize,
    ) -> Result<Vec<Vec<u8>>, MemoryError>;
}
