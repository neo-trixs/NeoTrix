#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub tokens_used: u32,
    pub model: String,
}

pub trait LlmProvider: Send + Sync {
    fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, String>;
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_creation() {
        let msg = Message {
            role: "user".into(),
            content: "hello".into(),
        };
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "hello");
    }

    #[test]
    fn completion_request_fields() {
        let req = CompletionRequest {
            model: "gpt-4".into(),
            messages: vec![Message { role: "system".into(), content: "be helpful".into() }],
            max_tokens: Some(100),
        };
        assert_eq!(req.model, "gpt-4");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.max_tokens, Some(100));
    }

    #[test]
    fn completion_response_fields() {
        let resp = CompletionResponse {
            content: "answer".into(),
            tokens_used: 10,
            model: "gpt-4".into(),
        };
        assert_eq!(resp.content, "answer");
        assert_eq!(resp.tokens_used, 10);
    }

    struct MockProvider;

    impl LlmProvider for MockProvider {
        fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, String> {
            Ok(CompletionResponse {
                content: format!("mock response for {}", request.model),
                tokens_used: 5,
                model: request.model.clone(),
            })
        }
        fn name(&self) -> &str {
            "mock"
        }
    }

    #[test]
    fn mock_provider_trait() {
        let p = MockProvider;
        let req = CompletionRequest {
            model: "test".into(),
            messages: vec![],
            max_tokens: None,
        };
        let resp = p.complete(&req).unwrap();
        assert_eq!(resp.content, "mock response for test");
        assert_eq!(p.name(), "mock");
    }
}
