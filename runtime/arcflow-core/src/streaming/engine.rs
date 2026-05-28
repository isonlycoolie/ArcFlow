//! Internal streaming with bounds (Sprint 6 — not SDK-exposed).

use std::time::Duration;

use futures_util::StreamExt;

use crate::constants::{PROVIDER_MAX_STREAM_TOKENS, PROVIDER_STREAM_TIMEOUT_SECS};

use crate::providers::error::ProviderCallError;
use crate::providers::response::ProviderStream;

/// Collects a bounded provider stream into a single string.
pub struct StreamingEngine;

impl StreamingEngine {
    pub async fn collect_bounded(stream: ProviderStream) -> Result<String, ProviderCallError> {
        let deadline =
            tokio::time::Instant::now() + Duration::from_secs(PROVIDER_STREAM_TIMEOUT_SECS);
        tokio::pin!(stream);
        let mut output = String::new();
        let mut token_estimate = 0u32;
        while tokio::time::Instant::now() < deadline {
            let next = tokio::time::timeout_at(deadline, stream.next()).await;
            let chunk = match next {
                Ok(Some(Ok(c))) => c,
                Ok(Some(Err(e))) => return Err(e),
                Ok(None) => break,
                Err(_) => {
                    return Err(ProviderCallError::Timeout {
                        provider_id: "stream".into(),
                        timeout_secs: PROVIDER_STREAM_TIMEOUT_SECS,
                    });
                }
            };
            token_estimate = token_estimate.saturating_add(chunk.content.len() as u32 / 4);
            if token_estimate > PROVIDER_MAX_STREAM_TOKENS {
                break;
            }
            output.push_str(&chunk.content);
            if chunk.is_final {
                break;
            }
        }
        Ok(output)
    }
}
