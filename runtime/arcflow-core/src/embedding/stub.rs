//! Deterministic stub embeddings for tests (Phase 1.5).

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;

use super::error::EmbeddingError;
use super::provider::EmbeddingProvider;

static STUB_WARNED: AtomicBool = AtomicBool::new(false);

/// Deterministic hash vector from input bytes — not semantically meaningful.
pub fn stub_embedding(seed: &[u8], dim: usize) -> Vec<f32> {
    let mut out = vec![0.0_f32; dim];
    for (i, item) in out.iter_mut().enumerate() {
        let byte = seed.get(i % seed.len().max(1)).copied().unwrap_or(0);
        *item = (byte as f32) / 255.0;
    }
    out
}

fn warn_once() {
    if !STUB_WARNED.swap(true, Ordering::Relaxed) {
        tracing::warn!(
            "Using stub embeddings — not suitable for production; set ARCFLOW_EMBEDDING_PROVIDER"
        );
    }
}

/// Explicit test-only embedding provider.
pub struct StubEmbeddingProvider {
    dim: usize,
}

impl StubEmbeddingProvider {
    pub fn new(dim: usize) -> Self {
        warn_once();
        Self { dim: dim.max(1) }
    }
}

#[async_trait]
impl EmbeddingProvider for StubEmbeddingProvider {
    fn id(&self) -> &str {
        "stub"
    }

    fn dimensions(&self) -> usize {
        self.dim
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Err(EmbeddingError::EmptyBatch);
        }
        Ok(texts
            .iter()
            .map(|t| stub_embedding(t.as_bytes(), self.dim))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_provider_warns_and_embeds() {
        let provider = StubEmbeddingProvider::new(8);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vectors = rt
            .block_on(provider.embed(&["hello".into()]))
            .expect("embed");
        assert_eq!(vectors.len(), 1);
        assert_eq!(vectors[0].len(), 8);
    }
}
