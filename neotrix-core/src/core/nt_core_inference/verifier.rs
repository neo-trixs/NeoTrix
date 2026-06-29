use crate::core::nt_core_inference::cascade::AsyncVerifierFn;
use crate::neotrix::nt_io_llm_provider::{LlmRequest, Message, Role};
use crate::neotrix::nt_io_provider::factory::{create_provider, ProviderConfig};

/// LLM-backed verifier for cascade execution.
///
/// Wraps the async LLM provider to verify and improve draft responses
/// when the drafter's quality is below threshold.
#[derive(Clone)]
pub struct LlmVerifier {
    system_prompt: String,
    max_tokens: u32,
    temperature: f32,
}

impl LlmVerifier {
    pub fn new(system_prompt: &str, max_tokens: u32, temperature: f32) -> Self {
        Self {
            system_prompt: system_prompt.to_string(),
            max_tokens,
            temperature,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(
            "You verify and improve the quality of a draft response. \
             Provide a corrected, more accurate version. \
             Keep the same length or shorter. Be precise and factual.",
            512,
            0.3,
        )
    }

    /// Verify and improve a draft response via the configured LLM provider.
    ///
    /// Returns (verified_response, estimated_cost, latency_ms).
    /// On provider error, falls back to the original draft with reduced confidence.
    pub async fn verify(&self, query: String, draft: String) -> (String, f64, f64) {
        let start = std::time::Instant::now();
        let config = ProviderConfig::from_env();
        let provider = create_provider(config);

        let request = LlmRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            messages: vec![
                Message {
                    role: Role::System,
                    content: self.system_prompt.clone(),
                    tool_calls: None,
                    tool_call_id: None,
                },
                Message {
                    role: Role::User,
                    content: format!("Query: {}\n\nDraft response: {}", query, draft),
                    tool_calls: None,
                    tool_call_id: None,
                },
            ],
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            tools: vec![],
            image_data: None,
        };

        match provider.complete(&request).await {
            Ok(response) => {
                let latency = start.elapsed().as_secs_f64() * 1000.0;
                let completion_tokens = response.usage.completion_tokens.max(1) as f64;
                let cost = completion_tokens * 0.000015;
                (response.content, cost, latency)
            }
            Err(e) => {
                let latency = start.elapsed().as_secs_f64() * 1000.0;
                log::warn!("[cascade:verifier] LLM verification failed: {}", e);
                (draft, 0.001, latency)
            }
        }
    }
}

/// Create an AsyncVerifierFn that delegates to LlmVerifier::with_defaults().
///
/// The returned closure is Send + Sync and can be passed to
/// CascadeEngine::process_pending_async.
pub fn create_verifier_fn() -> AsyncVerifierFn {
    let verifier = LlmVerifier::with_defaults();
    Box::new(move |query: String, draft: String| {
        let v = verifier.clone();
        Box::pin(async move { v.verify(query, draft).await })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_new() {
        let v = LlmVerifier::new("custom prompt", 256, 0.7);
        assert_eq!(v.system_prompt, "custom prompt");
        assert_eq!(v.max_tokens, 256);
        assert_eq!(v.temperature, 0.7);
    }

    #[test]
    fn test_verifier_with_defaults() {
        let v = LlmVerifier::with_defaults();
        assert!(v.system_prompt.contains("verify"));
        assert!(v.system_prompt.contains("improve"));
        assert_eq!(v.max_tokens, 512);
        assert!((v.temperature - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_create_verifier_fn() {
        let vfn = create_verifier_fn();
        let result = vfn("test query".into(), "test draft".into());
        // The future should be Send (for tokio::spawn compatibility)
        fn assert_send<T: Send>(_t: &T) {}
        assert_send(&result);
    }

    #[test]
    fn test_verifier_clone() {
        let v1 = LlmVerifier::with_defaults();
        let v2 = v1.clone();
        assert_eq!(v1.system_prompt, v2.system_prompt);
        assert_eq!(v1.max_tokens, v2.max_tokens);
        assert!((v1.temperature - v2.temperature).abs() < f32::EPSILON);
    }
}
