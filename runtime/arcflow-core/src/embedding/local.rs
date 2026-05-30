//! Local (offline) embedding provider — 384-dim hash vectors (Phase 1.5).
//!
//! Air-gapped deployments use this instead of remote APIs. Vectors are derived from
//! text content via feature hashing; they are not transformer-quality but require no network.

use std::sync::Arc;

use async_trait::async_trait;

use super::error::EmbeddingError;
use super::provider::EmbeddingProvider;

const DEFAULT_MODEL: &str = "all-MiniLM-L6-v2";
const DEFAULT_DIM: usize = 384;

/// Offline embedder matching `local/all-MiniLM-L6-v2` registry id (384 dimensions).
pub struct LocalEmbeddingProvider {
    model: String,
    dimensions: usize,
}

impl LocalEmbeddingProvider {
    pub fn new(model: &str) -> Result<Self, EmbeddingError> {
        let (dimensions, normalized) = match model {
            DEFAULT_MODEL | "all-minilm-l6-v2" => (DEFAULT_DIM, DEFAULT_MODEL),
            other if other.parse::<usize>().is_ok() => {
                let dim: usize = other.parse().unwrap_or(DEFAULT_DIM);
                (dim.max(8), DEFAULT_MODEL)
            }
            other => {
                return Err(EmbeddingError::InvalidSpec {
                    reason: format!("unsupported local embedding model '{other}'"),
                });
            }
        };
        Ok(Self {
            model: normalized.to_string(),
            dimensions,
        })
    }
}

/// Feature-hash text into a fixed-size vector and L2-normalize.
pub fn local_embed(text: &str, dim: usize) -> Vec<f32> {
    let mut out = vec![0.0_f32; dim.max(1)];
    for token in text.split_whitespace() {
        let hash = fnv1a(token.as_bytes());
        let bucket = (hash as usize) % dim;
        let sign = if hash & 1 == 0 { 1.0 } else { -1.0 };
        out[bucket] += sign;
    }
    if out.iter().all(|v| *v == 0.0) {
        out[0] = 1.0;
    }
    l2_normalize(&mut out);
    out
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn l2_normalize(v: &mut [f32]) {
    l2_normalize_slice(v);
}

pub(crate) fn l2_normalize_slice(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > f32::EPSILON {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    fn id(&self) -> &str {
        "local"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Err(EmbeddingError::EmptyBatch);
        }
        if let Ok(path) = std::env::var("ARCFLOW_EMBEDDING_ONNX_PATH") {
