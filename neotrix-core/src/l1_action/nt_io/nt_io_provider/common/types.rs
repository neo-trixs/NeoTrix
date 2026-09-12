//! LLM Provider 核心类型定义 (D3 分层修复: 类型已下沉至 core::nt_core_llm)
//!
//! 本文件保留为 re-export 兼容层 — 所有 `nt_io_provider::types::*` 路径不变,
//! 类型实体定义于 `crate::core::nt_core_llm` (NT-CORE 域)。
//!
//! 更新说明 (2026-07-01):
//! - `temperature` 改为 `Option<f32>` — 部分新模型 (如 Claude Sonnet 5) 不再支持采样参数
//! - 新增 `thinking_budget` — 支持模型自适应思考 (extended thinking)
//! - 新增 `provider_params` — 额外 provider 专用参数映射
//!
//! 2026-07-04: 新增 ProviderCategory (自我/客体分离)

pub use crate::core::nt_core_llm::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_image_b64_plain_png_gets_data_uri() {
        let req = LlmRequest::new("gpt-4o", "describe").with_image_b64("iVBORw0KGgo=");
        let uri = req.image_data.expect("image_data set");
        assert!(uri.starts_with("data:image/png;base64,iVBORw0KGgo="), "got {uri}");
    }

    #[test]
    fn test_with_image_b64_jpeg_magic_gets_jpeg_uri() {
        let req = LlmRequest::new("gpt-4o", "describe").with_image_b64("/9j/4AAQSkZJRg==");
        let uri = req.image_data.expect("image_data set");
        assert!(uri.starts_with("data:image/jpeg;base64,/9j/"), "got {uri}");
    }

    #[test]
    fn test_with_image_b64_already_data_uri_unchanged() {
        let req = LlmRequest::new("gpt-4o", "describe").with_image_b64("data:image/webp;base64,UklGRg==");
        let uri = req.image_data.expect("image_data set");
        assert_eq!(uri, "data:image/webp;base64,UklGRg==");
    }

    #[test]
    fn test_temperature_clean_removes_f32_noise() {
        let req = LlmRequest::new("openai", "hi");
        let clean = req.temperature_clean().expect("temperature set");
        assert_eq!(clean, 0.7, "f32 noise must be removed, got {clean}");
        let json = serde_json::json!(clean);
        assert_eq!(json.to_string(), "0.7", "json serialization must be clean, got {json}");
    }

    #[test]
    fn test_temperature_clean_none_returns_none() {
        let req = LlmRequest::new("gpt-4o", "describe").with_temperature(None);
        assert_eq!(req.temperature_clean(), None);
    }

    #[test]
    fn test_temperature_clean_preserves_common_values() {
        for (input, expected) in [
            (0.0f32, 0.0f64),
            (1.0f32, 1.0f64),
            (0.5f32, 0.5f64),
            (1.5f32, 1.5f64),
            (0.25f32, 0.25f64),
        ] {
            let req = LlmRequest::new("gpt-4o", "describe").with_temperature(Some(input));
            let clean = req.temperature_clean().expect("temperature set");
            assert_eq!(clean, expected, "input {input} -> {clean}, expected {expected}");
        }
    }

    #[test]
    fn test_temperature_clean_rounds_to_two_decimals() {
        let req = LlmRequest::new("gpt-4o", "describe").with_temperature(Some(0.3333f32));
        let clean = req.temperature_clean().expect("temperature set");
        assert_eq!(clean, 0.33, "got {clean}");
    }
}
