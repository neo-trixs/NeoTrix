//! NT-IO multimodal_transform — 图片标记→文本管线 (`nt_transform_pipeline`).
//!
//! `MultimodalTransform`: AgentLoop 接线点的 text-only 降维阶段
//! (图片标记 → `[PREFIX_n: analysis]`), 含意图感知变换。

use super::nt_transform_types::{
    PlaceholderAnalyzer, TransformConfig, Transformed, VisionAnalyzer,
};

/// 多模态预处理阶段。
pub struct MultimodalTransform {
    pub config: TransformConfig,
    analyzer: Box<dyn VisionAnalyzer>,
}

impl MultimodalTransform {
    pub fn new(config: TransformConfig, analyzer: Box<dyn VisionAnalyzer>) -> Self {
        Self { config, analyzer }
    }

    pub fn with_placeholder(config: TransformConfig) -> Self {
        Self::new(config, Box::new(PlaceholderAnalyzer::default()))
    }

    /// 检测目标模型是否应做 text-only 变换。
    pub fn applies_to(&self, model: &str) -> bool {
        if !self.config.enabled {
            return false;
        }
        if self.config.target_models.is_empty() {
            return true;
        }
        self.config.target_models.iter().any(|m| m == model)
    }

    /// 变换单条消息文本: 图片标记 → `[PREFIX_n: analysis]`。
    pub fn transform(&self, content: &str) -> Transformed {
        self.transform_with_intent(content, "")
    }

    /// 意图感知变换 (modlens + agent-vision-toolkit 吸收):
    /// 与 [`transform`](MultimodalTransform::transform) 相同, 但把当前任务意图
    /// 传入视觉分析器, 使图片分析贴合当前目标。意图为空时退化为通用分析。
    pub fn transform_with_intent(&self, content: &str, intent: &str) -> Transformed {
        if !self.config.enabled {
            return Transformed {
                text: content.to_string(),
                images_replaced: 0,
                images_passthrough: 0,
            };
        }
        let mut replaced = 0usize;
        let mut passthrough = 0usize;
        let available = self.analyzer.is_available();
        // 骨架: 匹配 markdown 图片 `![...](...)` 与显式 `[Image: n]`。
        let mut out = String::with_capacity(content.len());
        let mut rest = content;
        loop {
            let start_md = rest.find("![");
            let start_explicit = rest.find("[Image:");
            let start = match (start_md, start_explicit) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            match start {
                None => {
                    out.push_str(rest);
                    break;
                }
                Some(idx) => {
                    out.push_str(&rest[..idx]);
                    let (marker, consumed) = if rest[idx..].starts_with("![") {
                        self.consume_markdown(&rest[idx..])
                    } else {
                        self.consume_explicit(&rest[idx..])
                    };
                    if available {
                        replaced += 1;
                        out.push_str(&format!(
                            "[{}: {}]",
                            self.config.marker_prefix,
                            self.analyzer.analyze_with_intent(replaced, &marker, intent)
                        ));
                    } else {
                        passthrough += 1;
                        out.push_str(&marker);
                    }
                    rest = &rest[idx + consumed..];
                }
            }
        }
        Transformed {
            text: out,
            images_replaced: replaced,
            images_passthrough: passthrough,
        }
    }

    fn consume_markdown(&self, s: &str) -> (String, usize) {
        // `![alt](url)` — 返回 (alt, 消耗字节数)。
        let after = &s[2..];
        let alt_end = after.find(']').unwrap_or(0);
        let alt = &after[..alt_end];
        let rest = &after[alt_end + 1..];
        let (url, consumed_url) = if rest.starts_with('(') {
            let end = rest.find(')').unwrap_or(rest.len());
            (&rest[1..end], end + 1)
        } else {
            ("", 0)
        };
        (format!("{alt} [{url}]"), 2 + alt_end + 1 + consumed_url)
    }

    fn consume_explicit(&self, s: &str) -> (String, usize) {
        // `[Image: n]`
        let after = &s[1..];
        let end = after.find(']').unwrap_or(after.len());
        (after[..end].to_string(), 1 + end + 1)
    }

    /// 变换用户输入 (AgentLoop 生产接线点)。
    pub fn transform_input(&self, user_input: &str) -> String {
        self.transform(user_input).text
    }

    /// 意图感知变换用户输入 (AgentLoop 生产接线点, agent-vision-toolkit 吸收)。
    pub fn transform_input_with_intent(&self, user_input: &str, intent: &str) -> String {
        self.transform_with_intent(user_input, intent).text
    }
}

#[cfg(test)]
mod tests {
    use super::super::nt_transform_types::{TransformConfig, VisionAnalyzer};
    use super::*;

    #[test]
    fn markdown_image_replaced_with_marker() {
        let t = MultimodalTransform::with_placeholder(TransformConfig::default());
        let r = t.transform("看这张图 ![diagram](data:image/png;base64,xxx) 分析它");
        assert_eq!(r.images_replaced, 1);
        assert!(r
            .text
            .contains("[IMG: [image #1: diagram [data:image/png;base64,xxx]]"));
        assert!(!r.text.contains("![diagram"));
    }

    #[test]
    fn explicit_marker_replaced() {
        let t = MultimodalTransform::with_placeholder(TransformConfig::default());
        let r = t.transform("[Image: 1] 请描述");
        assert_eq!(r.images_replaced, 1);
        assert!(r.text.contains("[IMG:"));
    }

    #[test]
    fn no_image_passthrough_unchanged() {
        let t = MultimodalTransform::with_placeholder(TransformConfig::default());
        let r = t.transform("纯文本，无图片");
        assert_eq!(r.images_replaced, 0);
        assert_eq!(r.text, "纯文本，无图片");
    }

    #[test]
    fn disabled_config_returns_input() {
        let mut cfg = TransformConfig::default();
        cfg.enabled = false;
        let t = MultimodalTransform::with_placeholder(cfg);
        let r = t.transform("![a](b) 保留");
        assert_eq!(r.images_replaced, 0);
        assert_eq!(r.text, "![a](b) 保留");
    }

    #[test]
    fn applies_to_target_model_only() {
        let mut cfg = TransformConfig::default();
        cfg.target_models = vec!["deepseek-v4-flash".into()];
        let t = MultimodalTransform::with_placeholder(cfg);
        assert!(t.applies_to("deepseek-v4-flash"));
        assert!(!t.applies_to("qwen2.5:7b"));
    }

    #[test]
    fn intent_forwarded_to_analyzer() {
        // 意图感知 (agent-vision-toolkit 吸收): 覆盖 analyze_with_intent 的
        // 后端应收到当前任务意图。
        struct IntentAnalyzer {
            received: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
        }
        impl VisionAnalyzer for IntentAnalyzer {
            fn analyze(&self, _id: usize, marker: &str) -> String {
                format!("base:{}", marker)
            }
            fn analyze_with_intent(&self, id: usize, marker: &str, intent: &str) -> String {
                self.received.lock().unwrap_or_else(|e| e.into_inner()).push(intent.to_string());
                format!("intent[{}]:{}:{}", id, marker, intent)
            }
        }
        let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let t = MultimodalTransform::new(
            TransformConfig::default(),
            Box::new(IntentAnalyzer { received: received.clone() }),
        );
        let r = t.transform_with_intent("![图](url) 看这个", "提取登录按钮坐标");
        assert_eq!(r.images_replaced, 1);
        let got = received.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(got[0], "提取登录按钮坐标");
        assert!(r.text.contains("intent[1]"));
    }

    #[test]
    fn intent_empty_falls_back_to_base() {
        // 默认 analyze_with_intent 委托 analyze (placeholder 不感知意图)。
        let t = MultimodalTransform::with_placeholder(TransformConfig::default());
        let r = t.transform_with_intent("![a](b)", "");
        assert_eq!(r.images_replaced, 1);
    }
}
