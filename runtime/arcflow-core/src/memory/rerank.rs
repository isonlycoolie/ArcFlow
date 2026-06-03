//! Rerank providers for hybrid vector retrieval (Phase 2.5).

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

use crate::constants::{ARCFLOW_USER_AGENT, PROVIDER_REQUEST_TIMEOUT_SECS};

const COHERE_RERANK_ENDPOINT: &str = "https://api.cohere.com/v1/rerank";
const COHERE_API_KEY_ENV: &str = "COHERE_API_KEY";
const COHERE_RERANK_MODEL: &str = "rerank-english-v3.0";

/// Failures from rerank providers.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RerankError {
    #[error("rerank provider not configured: {reason}")]
    NotConfigured { reason: String },
    #[error("rerank request failed: {reason}")]
    RequestFailed { reason: String },
    #[error("rerank response parse error: {reason}")]
    ParseError { reason: String },
    #[error("rerank provider returned no results")]
    EmptyResults,
}

/// One reranked document chunk.
#[derive(Clone, Debug, PartialEq)]
pub struct RankedChunk {
    pub index: usize,
    pub score: f32,
    pub text: String,
}

/// Reorders retrieval candidates by query relevance.
#[async_trait]
pub trait RerankProvider: Send + Sync {
    fn id(&self) -> &str;

    async fn rerank(
        &self,
        query: &str,
        documents: &[String],
        top_k: usize,
    ) -> Result<Vec<RankedChunk>, RerankError>;
}

/// Cohere rerank API (explicit opt-in via memory config).
pub struct CohereRerankProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl CohereRerankProvider {
    pub fn from_env() -> Result<Self, RerankError> {
        let api_key =
            std::env::var(COHERE_API_KEY_ENV).map_err(|_| RerankError::NotConfigured {
                reason: format!("{COHERE_API_KEY_ENV} is not set"),
            })?;
        let client = Client::builder()
            .timeout(Duration::from_secs(PROVIDER_REQUEST_TIMEOUT_SECS))
            .user_agent(ARCFLOW_USER_AGENT)
            .build()
            .map_err(|e| RerankError::RequestFailed {
                reason: e.to_string(),
            })?;
        Ok(Self {
            client,
            api_key,
            model: COHERE_RERANK_MODEL.into(),
        })
    }
}

#[derive(Serialize)]
struct RerankRequest<'a> {
    model: &'a str,
    query: &'a str,
    documents: Vec<String>,
    top_n: usize,
}

#[derive(Deserialize)]
struct RerankResponse {
    results: Vec<RerankResult>,
}

#[derive(Deserialize)]
struct RerankResult {
    index: usize,
    relevance_score: f32,
}

#[async_trait]
impl RerankProvider for CohereRerankProvider {
    fn id(&self) -> &str {
        "cohere"
    }

    async fn rerank(
        &self,
        query: &str,
        documents: &[String],
        top_k: usize,
    ) -> Result<Vec<RankedChunk>, RerankError> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }
        let body = RerankRequest {
            model: &self.model,
            query,
            documents: documents.to_vec(),
            top_n: top_k.min(documents.len()),
        };
        let response = self
            .client
            .post(COHERE_RERANK_ENDPOINT)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| RerankError::RequestFailed {
                reason: e.to_string(),
            })?;
        if !response.status().is_success() {
            return Err(RerankError::RequestFailed {
                reason: format!("cohere rerank HTTP {}", response.status()),
            });
        }
        let parsed: RerankResponse =
            response.json().await.map_err(|e| RerankError::ParseError {
                reason: e.to_string(),
            })?;
        if parsed.results.is_empty() {
            return Err(RerankError::EmptyResults);
        }
        Ok(parsed
            .results
            .into_iter()
            .filter_map(|r| {
                documents.get(r.index).map(|text| RankedChunk {
                    index: r.index,
                    score: r.relevance_score,
                    text: text.clone(),
                })
            })
            .collect())
    }
}

/// Lexical overlap rerank for local-only environments (no remote API).
pub struct LocalLexicalRerankProvider;

#[async_trait]
impl RerankProvider for LocalLexicalRerankProvider {
    fn id(&self) -> &str {
        "local"
    }

    async fn rerank(
        &self,
        query: &str,
        documents: &[String],
        top_k: usize,
    ) -> Result<Vec<RankedChunk>, RerankError> {
        use super::hybrid::sparse_lexical_score;

        let mut scored: Vec<(usize, f32)> = documents
            .iter()
            .enumerate()
            .map(|(idx, doc)| (idx, sparse_lexical_score(query, doc)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored
            .into_iter()
            .take(top_k)
            .filter_map(|(idx, score)| {
                documents.get(idx).map(|text| RankedChunk {
                    index: idx,
                    score,
                    text: text.clone(),
                })
            })
            .collect())
    }
}

pub fn resolve_rerank_provider(spec: &str) -> Result<Box<dyn RerankProvider>, RerankError> {
    match spec {
        "cohere" => Ok(Box::new(CohereRerankProvider::from_env()?)),
        "local" => Ok(Box::new(LocalLexicalRerankProvider)),
        other => Err(RerankError::NotConfigured {
            reason: format!("unsupported rerank provider '{other}'"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_rerank_orders_by_overlap() {
        let provider = LocalLexicalRerankProvider;
        let docs = vec!["unrelated text".into(), "rust memory vector search".into()];
        let ranked = provider
            .rerank("rust vector", &docs, 2)
            .await
            .expect("local rerank");
        assert_eq!(ranked.len(), 2);
        assert!(ranked[0].score >= ranked[1].score);
        assert_eq!(ranked[0].index, 1);
    }
}
