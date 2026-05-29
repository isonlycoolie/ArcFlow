//! Voyage AI embeddings API (Phase 1.5).

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::constants::{ARCFLOW_USER_AGENT, PROVIDER_REQUEST_TIMEOUT_SECS};

use super::error::EmbeddingError;
use super::provider::EmbeddingProvider;

const VOYAGE_API_KEY_ENV: &str = "VOYAGE_API_KEY";
const VOYAGE_EMBEDDINGS_ENDPOINT: &str = "https://api.voyageai.com/v1/embeddings";
const VOYAGE_EMBEDDINGS_ENDPOINT_ENV: &str = "ARCFLOW_VOYAGE_EMBEDDINGS_ENDPOINT";

pub struct VoyageEmbeddingProvider {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
    dimensions: usize,
}

impl VoyageEmbeddingProvider {
    pub fn new(model: &str) -> Result<Self, EmbeddingError> {
        let api_key = std::env::var(VOYAGE_API_KEY_ENV).map_err(|_| EmbeddingError::NotConfigured {
            reason: format!("{VOYAGE_API_KEY_ENV} is not set"),
        })?;
        let dimensions = dimensions_for_model(model)?;
        let endpoint = std::env::var(VOYAGE_EMBEDDINGS_ENDPOINT_ENV)
            .unwrap_or_else(|_| VOYAGE_EMBEDDINGS_ENDPOINT.to_string());
        let client = Client::builder()
            .timeout(Duration::from_secs(PROVIDER_REQUEST_TIMEOUT_SECS))
            .user_agent(ARCFLOW_USER_AGENT)
            .build()
            .map_err(|e| EmbeddingError::RequestFailed {
                reason: e.to_string(),
            })?;
        Ok(Self {
            client,
            api_key,
            model: model.to_string(),
            endpoint,
            dimensions,
        })
    }
}

fn dimensions_for_model(model: &str) -> Result<usize, EmbeddingError> {
    match model {
        "voyage-3" => Ok(1024),
        "voyage-3-lite" => Ok(512),
        other => Err(EmbeddingError::InvalidSpec {
            reason: format!("unsupported Voyage embedding model '{other}'"),
        }),
    }
}

#[derive(Serialize)]
struct EmbeddingsRequest<'a> {
    model: &'a str,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbeddingsResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for VoyageEmbeddingProvider {
    fn id(&self) -> &str {
        "voyage"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Err(EmbeddingError::EmptyBatch);
        }
        let body = EmbeddingsRequest {
            model: &self.model,
            input: texts.to_vec(),
        };
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| EmbeddingError::RequestFailed {
                reason: e.to_string(),
            })?;
        let status = response.status();
        if !status.is_success() {
            let reason = response
                .text()
                .await
                .unwrap_or_else(|_| status.to_string());
            return Err(EmbeddingError::RequestFailed { reason });
        }
        let parsed: EmbeddingsResponse = response.json().await.map_err(|e| {
            EmbeddingError::ParseError {
                reason: e.to_string(),
            }
        })?;
        if parsed.data.is_empty() {
            return Err(EmbeddingError::EmptyBatch);
        }
        Ok(parsed.data.into_iter().map(|d| d.embedding).collect())
    }
}

pub fn voyage_provider(model: &str) -> Result<Arc<dyn EmbeddingProvider>, EmbeddingError> {
    Ok(Arc::new(VoyageEmbeddingProvider::new(model)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn voyage_embed_parses_response() {
        let mock = MockServer::start().await;
        std::env::set_var(VOYAGE_API_KEY_ENV, "test-key");
        std::env::set_var(VOYAGE_EMBEDDINGS_ENDPOINT_ENV, format!("{}/v1/embeddings", mock.uri()));

        Mock::given(method("POST"))
            .and(path("/v1/embeddings"))
            .and(header("authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{ "embedding": [0.1, 0.2] }]
            })))
            .mount(&mock)
            .await;

        let provider = VoyageEmbeddingProvider::new("voyage-3").unwrap();
        let vectors = provider.embed(&["hello".into()]).await.unwrap();
        assert_eq!(vectors.len(), 1);
    }
}
