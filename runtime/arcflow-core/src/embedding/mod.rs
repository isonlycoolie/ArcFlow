//! Embedding providers for vector memory (Phase 1.5).

mod error;
mod local;
mod onnx;
mod openai;
mod provider;
mod stub;
mod voyage;

pub use error::EmbeddingError;
pub use provider::EmbeddingProvider;
pub use stub::{stub_embedding, StubEmbeddingProvider};

use std::sync::Arc;

const LOCAL_ONLY_ENV: &str = "ARCFLOW_EMBEDDING_LOCAL_ONLY";

fn local_only_enabled() -> bool {
    std::env::var(LOCAL_ONLY_ENV)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn reject_remote(provider: &str) -> Result<(), EmbeddingError> {
    if local_only_enabled() {
        return Err(EmbeddingError::LocalOnlyViolation {
            provider: provider.to_string(),
        });
    }
    Ok(())
}

/// Resolves a provider from `provider/model` spec or bare `stub`.
pub fn resolve_provider(spec: &str) -> Result<Arc<dyn EmbeddingProvider>, EmbeddingError> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return Err(EmbeddingError::InvalidSpec {
            reason: "embedding provider spec must be non-empty".into(),
        });
    }
    if trimmed == "stub" {
        return Ok(Arc::new(StubEmbeddingProvider::new(8)));
    }
    let Some((provider, model)) = trimmed.split_once('/') else {
        return Err(EmbeddingError::InvalidSpec {
            reason: format!("expected 'provider/model' or 'stub', got '{trimmed}'"),
        });
    };
    match provider {
        "stub" => {
            let dim = model.parse::<usize>().unwrap_or(8);
            Ok(Arc::new(StubEmbeddingProvider::new(dim)))
        }
        "openai" => {
            reject_remote("openai")?;
            openai::openai_provider(model)
        }
        "local" => local::local_provider(model),
        "voyage" => {
            reject_remote("voyage")?;
            voyage::voyage_provider(model)
        }
        other => Err(EmbeddingError::InvalidSpec {
            reason: format!("unsupported embedding provider '{other}'"),
        }),
    }
}

/// Resolves provider from `ARCFLOW_EMBEDDING_PROVIDER` or explicit `stub` in dev/tests.
pub fn resolve_from_env() -> Result<Arc<dyn EmbeddingProvider>, EmbeddingError> {
    match std::env::var("ARCFLOW_EMBEDDING_PROVIDER") {
        Ok(spec) if !spec.trim().is_empty() => resolve_provider(&spec),
        _ => {
            if cfg!(test)
                || std::env::var("ARCFLOW_DEV_MODE")
                    .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
            {
                resolve_provider("stub")
            } else {
                Err(EmbeddingError::NotConfigured {
                    reason: "set ARCFLOW_EMBEDDING_PROVIDER (e.g. openai/text-embedding-3-small)"
                        .into(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_stub_spec() {
        let p = resolve_provider("stub").expect("stub");
        assert_eq!(p.id(), "stub");
        assert_eq!(p.dimensions(), 8);
    }

    #[test]
    fn resolve_local_spec() {
        let p = resolve_provider("local/all-MiniLM-L6-v2").expect("local");
        assert_eq!(p.id(), "local");
        assert_eq!(p.dimensions(), 384);
    }

    #[test]
    fn local_only_blocks_openai() {
        std::env::set_var(LOCAL_ONLY_ENV, "true");
        let err = resolve_provider("openai/text-embedding-3-small");
        assert!(matches!(err, Err(EmbeddingError::LocalOnlyViolation { .. })));
        std::env::remove_var(LOCAL_ONLY_ENV);
    }

    #[test]
    fn local_only_blocks_voyage() {
        std::env::set_var(LOCAL_ONLY_ENV, "true");
        let err = resolve_provider("voyage/voyage-3");
        assert!(matches!(err, Err(EmbeddingError::LocalOnlyViolation { .. })));
        std::env::remove_var(LOCAL_ONLY_ENV);
    }
}
