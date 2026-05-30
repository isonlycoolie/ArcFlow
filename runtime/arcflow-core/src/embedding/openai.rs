//! OpenAI embeddings API (Phase 1.5).

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::constants::{
    endpoint_from_env, ARCFLOW_USER_AGENT, OPENAI_API_KEY_ENV, PROVIDER_REQUEST_TIMEOUT_SECS,
};

use super::error::EmbeddingError;
use super::provider::EmbeddingProvider;

const OPENAI_EMBEDDINGS_ENDPOINT: &str = "https://api.openai.com/v1/embeddings";
const OPENAI_EMBEDDINGS_ENDPOINT_ENV: &str = "ARCFLOW_OPENAI_EMBEDDINGS_ENDPOINT";

pub struct OpenAIEmbeddingProvider {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
    dimensions: usize,
}

impl OpenAIEmbeddingProvider {
    pub fn new(model: &str) -> Result<Self, EmbeddingError> {
        let api_key = std::env::var(OPENAI_API_KEY_ENV).map_err(|_| EmbeddingError::NotConfigured {
            reason: format!("{OPENAI_API_KEY_ENV} is not set"),
        })?;
        let dimensions = dimensions_for_model(model)?;
        let endpoint = endpoint_from_env(OPENAI_EMBEDDINGS_ENDPOINT_ENV, OPENAI_EMBEDDINGS_ENDPOINT);
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
        "text-embedding-3-small" => Ok(1536),
        "text-embedding-3-large" => Ok(3072),
        other => Err(EmbeddingError::InvalidSpec {
            reason: format!("unsupported OpenAI embedding model '{other}'"),
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
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    fn id(&self) -> &str {
        "openai"
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
