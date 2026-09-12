//! AI 图像提示生成模块 (NT-IO)  (C2)
//!
//! 吸收源: github.com/YouMind-OpenLab/ai-image-prompts-skill
//! 成熟度: C2 (去 stub — 新增 `generate_remote` 真实调用图像生成 API)
//!
//! 核心能力: 将高层主题(species/concept)转换为结构化、可复用的图像生成提示词,
//! 支持风格/画幅/镜头参数的组合渲染。C2 接线: `generate_remote` 用 reqwest 同步
//! 客户端真实投递提示词到图像 API 并解析响应。
//! 密钥: 优先 `NEOTRIX_IMAGE_API_KEY` 环境变量，回退 `NeoTrixConfig.api_key`；
//! 端点: 优先 `NEOTRIX_IMAGE_ENDPOINT` 环境变量，回退 `NeoTrixConfig.custom_endpoint`。

use crate::config::NeoTrixConfig;
use crate::core::nt_core_self_test::SelfTest;

/// 图像提示生成器 trait — 将语义意图映射为可投递给图像模型的 prompt 字符串。
pub(crate) trait AiImagePromptGenerator: Send + Sync {
    /// 给定主体与可选风格, 生成一个结构化提示词。
    fn generate(&self, subject: &str, style: Option<&str>) -> String;
    /// 校验生成结果是否满足最小结构化约束 (非空且含主体)。
    fn is_well_formed(&self, prompt: &str, subject: &str) -> bool;
    /// 真实调用图像生成 API: 投递提示词并解析响应, 返回图像 URL/b64 (C2)。
    fn generate_remote(&self, subject: &str, style: Option<&str>) -> Result<String, String>;
}

/// 默认实现: 拼接 subject + style + 固定镜头元参数。
#[derive(Default)]
pub(crate) struct ImagePromptEngine;

/// 解析图像 API 端点
fn resolve_endpoint() -> String {
    if let Ok(e) = std::env::var("NEOTRIX_IMAGE_ENDPOINT") {
        if !e.is_empty() {
            return e;
        }
    }
    if let Some(e) = NeoTrixConfig::load().custom_endpoint {
        if !e.is_empty() {
            return e;
        }
    }
    "https://api.openai.com/v1/images/generations".to_string()
}

/// 解析图像 API 密钥
fn resolve_api_key() -> Option<String> {
    if let Ok(k) = std::env::var("NEOTRIX_IMAGE_API_KEY") {
        if !k.is_empty() {
            return Some(k);
        }
    }
    NeoTrixConfig::load().api_key
}

impl AiImagePromptGenerator for ImagePromptEngine {
    fn generate(&self, subject: &str, style: Option<&str>) -> String {
        let style_part = style
            .filter(|s| !s.is_empty())
            .map(|s| format!(", style: {}", s))
            .unwrap_or_default();
        format!(
            "image prompt for `{}`{} — cinematic framing, balanced composition, high detail",
            subject, style_part
        )
    }

    fn is_well_formed(&self, prompt: &str, subject: &str) -> bool {
        !prompt.is_empty() && prompt.contains(subject)
    }

    fn generate_remote(&self, subject: &str, style: Option<&str>) -> Result<String, String> {
        let prompt = self.generate(subject, style);
        let endpoint = resolve_endpoint();
        let client = reqwest::blocking::Client::new();
        let mut builder = client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "prompt": prompt,
                "model": "gpt-image-1",
                "n": 1,
                "size": "1024x1024"
            }));
        if let Some(key) = resolve_api_key() {
            builder = builder.bearer_auth(key);
        }
        let resp = builder
            .send()
            .map_err(|e| format!("nt_io_ai_image_prompts: request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!(
                "nt_io_ai_image_prompts: api returned status {}",
                resp.status()
            ));
        }
        let body: serde_json::Value = resp
            .json()
            .map_err(|e| format!("nt_io_ai_image_prompts: bad json: {}", e))?;
        body.get("data")
            .and_then(|d| d.get(0))
            .and_then(|first| first.get("url").or_else(|| first.get("b64_json")))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "nt_io_ai_image_prompts: no image url/b64 in response".to_string())
    }
}

/// T1 SelfTest: 验证生成器存在并能产出结构化提示词 (离线)。
#[derive(Default)]
pub(crate) struct AiImagePromptsSelfTest;

impl SelfTest for AiImagePromptsSelfTest {
    fn name(&self) -> &str {
        "nt_io_ai_image_prompts"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let engine = ImagePromptEngine;
        let prompt = engine.generate("neon fox", Some("cyberpunk"));
        if engine.is_well_formed(&prompt, "neon fox") {
            Ok(())
        } else {
            Err(vec![
                "nt_io_ai_image_prompts: generated prompt missing subject".into()
            ])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_contains_subject() {
        let e = ImagePromptEngine;
        let p = e.generate("forest spirit", None);
        assert!(p.contains("forest spirit"));
        assert!(e.is_well_formed(&p, "forest spirit"));
    }

    #[test]
    fn test_generate_with_style_appends_style() {
        let e = ImagePromptEngine;
        let p = e.generate("cat", Some("watercolor"));
        assert!(p.contains("watercolor"));
        assert!(e.is_well_formed(&p, "cat"));
    }

    #[test]
    fn test_consistency_across_calls() {
        let e = ImagePromptEngine;
        let a = e.generate("dragon", Some("ink"));
        let b = e.generate("dragon", Some("ink"));
        assert_eq!(a, b);
    }

    #[test]
    fn test_selftest_passes() {
        let t = AiImagePromptsSelfTest;
        assert_eq!(t.name(), "nt_io_ai_image_prompts");
        assert!(t.self_test().is_ok());
    }

    #[test]
    #[ignore = "requires network access + API key"]
    fn test_generate_remote_real() {
        let e = ImagePromptEngine;
        let out = e
            .generate_remote("neon fox", Some("cyberpunk"))
            .expect("remote call should succeed with valid key");
        assert!(!out.is_empty());
    }
}
