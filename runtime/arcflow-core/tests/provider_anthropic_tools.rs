//! Anthropic provider tool-loop mock HTTP tests (Phase 2-Pro).

use arcflow_core::providers::anthropic::AnthropicProvider;
use arcflow_core::providers::model_provider::ModelProvider;
use arcflow_core::providers::request::{
    MessageRole, ProviderMessage, ProviderRequest, ToolSchema,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn anthropic_complete_parses_tool_use_blocks() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "claude-3-5-sonnet-20241022",
            "content": [{
                "type": "tool_use",
                "id": "toolu_01",
                "name": "web_search",
                "input": {"query": "AAPL"}
            }],
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 12, "output_tokens": 8}
        })))
        .mount(&server)
        .await;

    let url = format!("{}/v1/messages", server.uri());
    let provider =
        AnthropicProvider::with_endpoint("claude-3-5-sonnet-20241022".into(), "test-key".into(), url)
            .unwrap();
    let response = provider
        .complete(ProviderRequest {
            messages: vec![ProviderMessage {
                role: MessageRole::User,
                content: "search AAPL".into(),
                tool_calls: None,
                tool_call_id: None,
            }],
            system_prompt: None,
            max_tokens: 256,
            temperature: 0.0,
            tools: vec![ToolSchema {
                name: "web_search".into(),
                description: Some("Search".into()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {"query": {"type": "string"}},
                }),
            }],
        })
        .await
        .unwrap();
    let calls = response.tool_calls.expect("tool calls");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "web_search");
    assert!(calls[0].arguments.contains("AAPL"));
}
