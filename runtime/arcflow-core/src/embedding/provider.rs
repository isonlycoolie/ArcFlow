//! Embedding provider trait (Phase 1.5).

use async_trait::async_trait;

use super::error::EmbeddingError;

/// Generates dense vectors for semantic retrieval.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Stable provider id, e.g. `openai` or `stub`.
    fn id(&self) -> &str;

    /// Output vector dimension for this provider/model.
    fn dimensions(&self) -> usize;

    /// Embeds one or more text inputs in provider-defined batch sizes.
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// Maximum texts per remote batch request.
    fn max_batch_size(&self) -> usize {
        64
    }
}
