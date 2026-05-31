//! OpenAI provider mock HTTP tests (Sprint 6).

use arcflow_core::providers::model_provider::ModelProvider;
use arcflow_core::providers::openai::OpenAIProvider;
use arcflow_core::providers::request::{MessageRole, ProviderMessage, ProviderRequest};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn openai_complete_uses_mock_server() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "gpt-4o",
            "choices": [{
                "message": { "content": "mocked response" },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        })))
        .mount(&server)
        .await;

    let url = format!("{}/v1/chat/completions", server.uri());
    let provider =
        OpenAIProvider::with_endpoint("gpt-4o".into(), "test-key".into(), url).unwrap();
    let response = provider
        .complete(ProviderRequest {
            messages: vec![ProviderMessage {
                role: MessageRole::User,
                content: "hello".into(),
                tool_calls: None,
                tool_call_id: None,
            }],
            system_prompt: Some("sys".into()),
            max_tokens: 100,
            temperature: 0.5,
            tools: vec![],
        })
        .await
        .unwrap();
    assert_eq!(response.content, "mocked response");
    assert_eq!(response.tokens.total_tokens, 15);
}
