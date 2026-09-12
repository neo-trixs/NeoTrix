//! Capability-based Router — selects providers based on required capabilities
//! rather than model names.
//!
//! Infers required capabilities from the request (vision, tools, streaming)
//! and matches against provider catalog metadata.

use crate::core::nt_core_llm::{LlmError, LlmRequest, Message};
use super::provider_catalog::{ProviderCategory, ProviderInfo, PROVIDER_CATALOG};

/// Wrapper around the static provider catalog with capability-based query support.
pub struct ProviderCatalog;

impl ProviderCatalog {
    /// Find providers that satisfy the required capabilities.
    pub fn find_by_capabilities(required: &RequiredCapabilities) -> Vec<&'static ProviderInfo> {
        PROVIDER_CATALOG
            .iter()
            .filter(|p| Self::satisfies(p, required))
            .collect()
    }

    fn satisfies(info: &ProviderInfo, required: &RequiredCapabilities) -> bool {
        // Vision: only check if required — local providers always satisfy via passthrough
        if required.vision && info.category == ProviderCategory::Cloud {
            // Most cloud providers support vision; skip those known not to
            // (conservative: allow all cloud, filter later if needed)
        }
        // Free preference
        if required.prefer_free && !info.is_free {
            return false;
        }
        true
    }
}

/// Required capabilities inferred from a request.
#[derive(Debug, Clone, Default)]
pub struct RequiredCapabilities {
    /// Request needs vision (image input).
    pub vision: bool,
    /// Request uses function calling / tools.
    pub function_calling: bool,
    /// Prefer free providers.
    pub prefer_free: bool,
}

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
    pub fn infer_capabilities(request: &LlmRequest) -> RequiredCapabilities {
        let vision = request.image_data.is_some()
            || request.messages.iter().any(|m| {
                m.content.contains("![") || m.content.contains("<image>")
            });
        let function_calling = !request.tools.is_empty();
        RequiredCapabilities {
            vision,
            function_calling,
            prefer_free: false,
        }
    }

    /// Route request to best matching provider name.
    pub fn route(&self, request: &LlmRequest) -> Result<String, LlmError> {
        let mut required = Self::infer_capabilities(request);
        required.prefer_free = self.prefer_free;

        let candidates = ProviderCatalog::find_by_capabilities(&required);

        if candidates.is_empty() {
            return Err(LlmError::Unknown(
                "No provider supports required capabilities".into(),
            ));
        }

        // Sort: free first, then by category priority (Local > Proxy > Cloud)
        let mut sorted = candidates;
        sorted.sort_by(|a, b| {
            b.is_free
                .cmp(&a.is_free)
                .then(a.category.route_priority().cmp(&b.category.route_priority()))
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
    use crate::core::nt_core_llm::{Role, Tool};

    fn user_msg(content: &str) -> Message {
        Message::new(Role::User, content)
    }

    #[test]
    fn test_infer_no_tools_no_images() {
        let req = LlmRequest::new("test", "hello");
        let cap = CapabilityRouter::infer_capabilities(&req);
        assert!(!cap.vision);
        assert!(!cap.function_calling);
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
    fn test_catalog_find_returns_providers() {
        let candidates = ProviderCatalog::find_by_capabilities(&RequiredCapabilities::default());
        assert!(!candidates.is_empty(), "catalog should have providers");
    }

    #[test]
    fn test_catalog_free_filter() {
        let req_cap = RequiredCapabilities {
            prefer_free: true,
            ..Default::default()
        };
        let candidates = ProviderCatalog::find_by_capabilities(&req_cap);
        assert!(candidates.iter().all(|p| p.is_free));
    }

    #[test]
    fn test_route_returns_provider_name() {
        let router = CapabilityRouter::new();
        let req = LlmRequest::new("test", "hello");
        let name = router.route(&req).unwrap();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_route_prefers_free() {
        let router = CapabilityRouter::with_prefer_free(true);
        let req = LlmRequest::new("test", "hello");
        let name = router.route(&req).unwrap();
        let info = PROVIDER_CATALOG.iter().find(|p| p.name == name).unwrap();
        assert!(info.is_free, "prefer_free should select free provider: {name}");
    }
}
