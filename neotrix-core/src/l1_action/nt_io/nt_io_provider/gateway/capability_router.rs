//! Capability-based Router — selects providers based on required capabilities
//! rather than model names.
//!
//! Infers required capabilities from the request (vision, tools, streaming)
//! and matches against provider catalog metadata via `ProviderCapabilities`.

use crate::core::nt_core_llm::{LlmError, LlmRequest};
use super::provider_catalog::{
    find_by_capabilities, ProviderCapabilities,
};

/// Capability-based router — selects providers based on required capabilities.
pub struct CapabilityRouter {
    prefer_free: bool,
}

impl CapabilityRouter {
    pub fn new() -> Self {
        Self {
            prefer_free: false,
        }
    }

    pub fn with_prefer_free(prefer_free: bool) -> Self {
        Self { prefer_free }
    }

    /// Infer required capabilities from request.
    pub fn infer_capabilities(request: &LlmRequest) -> ProviderCapabilities {
        let vision = request.image_data.is_some()
            || request.messages.iter().any(|m| {
                m.content.contains("![") || m.content.contains("<image>")
            });
        let function_calling = !request.tools.is_empty();
        ProviderCapabilities {
            text: true,
            vision,
            audio: false,
            function_calling,
            streaming: true,
            context_window: 0, // no limit
            cost_per_1k_tokens: 0.0,
        }
    }

    /// Route request to best matching provider name.
    pub fn route(&self, request: &LlmRequest) -> Result<String, LlmError> {
        let mut required = Self::infer_capabilities(request);
        if self.prefer_free {
            required.cost_per_1k_tokens = 0.0;
        }

        let candidates = find_by_capabilities(&required);

        if candidates.is_empty() {
            return Err(LlmError::Unknown(
                "No provider supports required capabilities".into(),
            ));
        }

        // Sort: free first, then by category priority (Local > Proxy > Cloud),
        // then by cost ascending, then by name for determinism.
        let mut sorted = candidates;
        sorted.sort_by(|a, b| {
            b.is_free
                .cmp(&a.is_free)
                .then(a.category.route_priority().cmp(&b.category.route_priority()))
                .then(
                    a.capabilities
                        .cost_per_1k_tokens
                        .partial_cmp(&b.capabilities.cost_per_1k_tokens)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then(a.name.cmp(&b.name))
        });

        Ok(sorted.first().unwrap().name.to_string())
    }
}

impl Default for CapabilityRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_llm::{Message, Role, Tool};

    fn user_msg(content: &str) -> Message {
        Message::new(Role::User, content)
    }

    #[test]
    fn test_infer_no_tools_no_images() {
        let req = LlmRequest::new("test", "hello");
        let cap = CapabilityRouter::infer_capabilities(&req);
        assert!(!cap.vision);
        assert!(!cap.function_calling);
        assert!(cap.text);
        assert!(cap.streaming);
    }

    #[test]
    fn test_infer_image_data_sets_vision() {
        let req = LlmRequest::new("test", "describe").with_image_b64("iVBORw0KGgo=");
        let cap = CapabilityRouter::infer_capabilities(&req);
        assert!(cap.vision);
    }

    #[test]
    fn test_infer_markdown_image_sets_vision() {
        let mut req = LlmRequest::new("test", "what is this?");
        req.messages = vec![user_msg("![photo](data:image/png;base64,abc)")];
        let cap = CapabilityRouter::infer_capabilities(&req);
        assert!(cap.vision);
    }

    #[test]
    fn test_infer_html_image_sets_vision() {
        let mut req = LlmRequest::new("test", "describe");
        req.messages = vec![user_msg("<image>scan this</image>")];
        let cap = CapabilityRouter::infer_capabilities(&req);
        assert!(cap.vision);
    }

    #[test]
    fn test_infer_tools_sets_function_calling() {
        let req = LlmRequest::new("test", "use tools").with_tools(vec![Tool {
            name: "search".into(),
            description: "search the web".into(),
            parameters: "{}".into(),
        }]);
        let cap = CapabilityRouter::infer_capabilities(&req);
        assert!(cap.function_calling);
    }

    #[test]
    fn test_route_returns_provider_name() {
        let router = CapabilityRouter::new();
        let req = LlmRequest::new("test", "hello");
        let name = router.route(&req).unwrap();
        assert!(!name.is_empty());
        assert!(
            PROVIDER_CATALOG.iter().any(|p| p.name == name),
            "route should return a known provider: {name}"
        );
    }

    #[test]
    fn test_route_prefers_free() {
        let router = CapabilityRouter::with_prefer_free(true);
        let req = LlmRequest::new("test", "hello");
        let name = router.route(&req).unwrap();
        let info = PROVIDER_CATALOG.iter().find(|p| p.name == name).unwrap();
        assert!(info.is_free, "prefer_free should select free provider: {name}");
    }

    #[test]
    fn test_route_vision_prefers_vision_capable() {
        let router = CapabilityRouter::new();
        let req = LlmRequest::new("test", "describe").with_image_b64("iVBORw0KGgo=");
        let name = router.route(&req).unwrap();
        let info = PROVIDER_CATALOG.iter().find(|p| p.name == name).unwrap();
        assert!(
            info.capabilities.vision,
            "vision request should route to vision-capable provider: {name}"
        );
    }

    #[test]
    fn test_route_tool_calling_prefers_capable() {
        let router = CapabilityRouter::new();
        let req = LlmRequest::new("test", "use tools").with_tools(vec![Tool {
            name: "search".into(),
            description: "search".into(),
            parameters: "{}".into(),
        }]);
        let name = router.route(&req).unwrap();
        let info = PROVIDER_CATALOG.iter().find(|p| p.name == name).unwrap();
        assert!(
            info.capabilities.function_calling,
            "tool request should route to function-calling-capable provider: {name}"
        );
    }
}
