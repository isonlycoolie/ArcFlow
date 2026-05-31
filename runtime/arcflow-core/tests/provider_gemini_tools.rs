//! Gemini provider tool-loop mock HTTP tests (Phase 2-Pro).

use arcflow_core::providers::gemini::GeminiProvider;
use arcflow_core::providers::model_provider::ModelProvider;
use arcflow_core::providers::request::{
    MessageRole, ProviderMessage, ProviderRequest, ToolSchema,
};
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn gemini_complete_parses_function_call() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"/v1beta/models/.*:generateContent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{
                        "functionCall": {
                            "name": "web_search",
                            "args": {"query": "AAPL"}
                        }
                    }]
                },
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 10,
                "candidatesTokenCount": 5,
                "totalTokenCount": 15
            }
        })))
        .mount(&server)
        .await;

    let base = format!("{}/v1beta/models", server.uri());
    let provider =
        GeminiProvider::with_base_url("gemini-1.5-flash".into(), "test-key".into(), base).unwrap();
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
}
