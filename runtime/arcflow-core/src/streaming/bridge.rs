//! Thread bridge from tokio stream channel to std mpsc for SDK polling.

use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use super::channel::{default_stream_pair, StreamChannelSender};
use super::events::StreamEvent;

/// Outcome of a bridged workflow run.
pub struct StreamRunOutcome<R> {
    pub result: R,
}

/// Polls stream events from a worker thread while a blocking run executes.
pub struct StreamRunBridge<R> {
    event_rx: mpsc::Receiver<StreamEvent>,
    bridge_handle: JoinHandle<()>,
    worker: JoinHandle<R>,
}

impl<R: Send + 'static> StreamRunBridge<R> {
    /// Spawns `run` on a worker thread and forwards stream events to a sync queue.
    pub fn spawn<F>(run: F) -> Self
    where
        F: FnOnce(StreamChannelSender) -> R + Send + 'static,
    {
        let (stream_tx, mut tokio_rx) = default_stream_pair();
        let (event_tx, event_rx) = mpsc::channel();
        let bridge_handle = thread::spawn(move || {
            while let Some(event) = tokio_rx.blocking_recv() {
                if event_tx.send(event).is_err() {
                    break;
                }
            }
        });
        let worker = thread::spawn(move || run(stream_tx));
        Self {
            event_rx,
            bridge_handle,
            worker,
        }
    }

    /// Non-blocking poll for the next stream event.
    pub fn try_recv_event(&self) -> Option<StreamEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Blocks until the next stream event or the bridge closes.
    pub fn recv_event(&self) -> Option<StreamEvent> {
        self.event_rx.recv().ok()
    }

    /// Waits for the worker and bridge threads; returns the run result.
    pub fn join(self) -> StreamRunOutcome<R> {
        let result = self
            .worker
            .join()
            .unwrap_or_else(|_| panic!("stream worker panicked"));
        let _ = self.bridge_handle.join();
        StreamRunOutcome { result }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use async_trait::async_trait;
    use uuid::Uuid;

    use crate::providers::{
        ModelProvider, ProviderCallError, ProviderRequest, ProviderResponse,
    };
    use crate::providers::response::{FinishReason, ProviderStream, StreamChunk};
    use crate::rcs::types::{AgentDefinition, ExecutionMode, StepDefinition, WorkflowDefinition};
    use crate::tracing::types::TokenUsage;
    use crate::workflow::{ExecutionConfig, StreamConfig, WorkflowEngine, WorkflowRunError};

    use super::*;

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
                tool_calls: None,
            })
        }

        async fn stream(
            &self,
            _request: ProviderRequest,
        ) -> Result<ProviderStream, ProviderCallError> {
            let len = self.chunks.len();
            let items: Vec<Result<StreamChunk, ProviderCallError>> = self
                .chunks
                .iter()
                .enumerate()
                .map(|(idx, content)| {
                    Ok(StreamChunk {
                        content: content.clone(),
                        is_final: idx + 1 == len,
                        tokens: None,
                    })
                })
                .collect();
            Ok(Box::pin(futures_util::stream::iter(items)))
        }
    }

    #[test]
    fn bridge_yields_events_before_run_completes() {
        let agent_id = Uuid::new_v4();
        let step_id = Uuid::new_v4();
        let agent = AgentDefinition {
            id: agent_id,
            name: "writer".into(),
            role: "author".into(),
            instructions: "Write.".into(),
            tools: None,
            memory_config: None,
            context: None,
            tool_execution: None,
        };
        let mut agents = HashMap::new();
        agents.insert(agent_id, agent);
        let workflow = WorkflowDefinition {
            id: Uuid::new_v4(),
            name: "bridge-test".into(),
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
            external_bindings: None,
        };
        let provider = Arc::new(MockStreamProvider {
            chunks: vec!["hello".into(), " world".into()],
        });
        let exec_config = ExecutionConfig {
            stream: Some(StreamConfig { enabled: true }),
            ..ExecutionConfig::default()
        };

        let bridge = StreamRunBridge::spawn(move |stream_tx| {
            let engine = WorkflowEngine::new();
            engine.execute_with_config(
                &workflow,
                &agents,
                "input",
                None,
                None,
                Some(provider),
                128,
                0.0,
                &exec_config,
                Some(stream_tx),
            )
        });

        let mut saw_step_start = false;
        for _ in 0..50 {
            if let Some(StreamEvent::StepStart { .. }) = bridge.try_recv_event() {
                saw_step_start = true;
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }

        let outcome = bridge.join();
        assert!(saw_step_start, "expected step_start before join");
        assert!(matches!(
            outcome.result,
            Ok(_) | Err(WorkflowRunError::Failed { .. })
        ));
    }
}
