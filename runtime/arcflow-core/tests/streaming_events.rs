//! Stream event emission during workflow runs (Phase 2.1).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use arcflow_core::providers::{
    ModelProvider, ProviderCallError, ProviderRequest, ProviderResponse,
};
use arcflow_core::providers::response::{FinishReason, ProviderStream, StreamChunk};
use arcflow_core::rcs::types::{
    AgentDefinition, ExecutionMode, StepDefinition, WorkflowDefinition,
};
use arcflow_core::streaming::{default_stream_pair, StreamEvent};
use arcflow_core::tracing::types::TokenUsage;
use arcflow_core::workflow::{ExecutionConfig, StreamConfig, WorkflowEngine};

struct MockStreamProvider {
    chunks: Vec<String>,
}

#[async_trait]
impl ModelProvider for MockStreamProvider {
    fn provider_id(&self) -> &str {
        "mock"
    }

    fn model_id(&self) -> &str {
        "mock-stream"
    }

    async fn complete(
        &self,
        _request: ProviderRequest,
    ) -> Result<ProviderResponse, ProviderCallError> {
        Ok(ProviderResponse {
            content: self.chunks.join(""),
            tokens: TokenUsage::default(),
            model_id: self.model_id().to_string(),
            finish_reason: FinishReason::Stop,
        })
    }

    async fn stream(
        &self,
        _request: ProviderRequest,
    ) -> Result<ProviderStream, ProviderCallError> {
        let chunks = self.chunks.clone();
        let len = chunks.len();
        let items: Vec<Result<StreamChunk, ProviderCallError>> = chunks
            .into_iter()
            .enumerate()
            .map(|(idx, content)| {
                Ok(StreamChunk {
                    content,
                    is_final: idx + 1 == len,
                    tokens: None,
                })
            })
            .collect();
        Ok(Box::pin(futures_util::stream::iter(items)))
    }
}

fn single_step_workflow(agent_id: Uuid, step_id: Uuid) -> (WorkflowDefinition, HashMap<Uuid, AgentDefinition>) {
    let agent = AgentDefinition {
        id: agent_id,
        name: "writer".into(),
        role: "author".into(),
        instructions: "Write.".into(),
        tools: None,
        memory_config: None,
    };
    let mut agents = HashMap::new();
    agents.insert(agent_id, agent);
    let workflow = WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "stream-test".into(),
        steps: vec![StepDefinition {
            id: step_id,
            agent_id,
            order: 1,
            fallback_step_id: None,
            hitl: None,
        }],
        retry_policy: None,
        execution_mode: ExecutionMode::Linear,
        graph: None,
    };
    (workflow, agents)
}

#[test]
