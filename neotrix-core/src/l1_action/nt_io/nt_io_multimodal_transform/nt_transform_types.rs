//! NT-IO multimodal_transform — 共享内核类型 (`nt_transform_types`).
//!
//! 分析器契约 + 变换配置/结果。纯数据类型, 无管线逻辑; 各模态模块依赖本模块 (DAG 根).

use std::fmt;

/// 视觉分析器契约 — 骨架仅提供占位实现，真实后端 (本地视觉模型/多模态 API)
/// 实现此 trait 后注入。
pub trait VisionAnalyzer: Send + Sync {
    /// 对单张图片产出纯文本分析。
    fn analyze(&self, image_id: usize, marker: &str) -> String;
    /// 意图感知分析 (吸收自 Anionex/agent-vision-toolkit): 把当前任务意图
    /// 传给视觉层, 使其产出贴合当前目标的观察 (而非通用描述)。默认委托
    /// [`analyze`](VisionAnalyzer::analyze), 后端可选择覆盖以利用意图。
    fn analyze_with_intent(&self, image_id: usize, marker: &str, _intent: &str) -> String {
        self.analyze(image_id, marker)
    }
    /// 后端是否可用 (不可用则原样透传图片标记，避免丢信息)。
    fn is_available(&self) -> bool {
        true
    }
}

/// 骨架占位分析器 — 用图片元数据生成占位文本，供管线联通测试。
pub struct PlaceholderAnalyzer {
    pub prefix: String,
}

impl Default for PlaceholderAnalyzer {
    fn default() -> Self {
        Self {
            prefix: "image".to_string(),
        }
    }
}

impl VisionAnalyzer for PlaceholderAnalyzer {
    fn analyze(&self, image_id: usize, marker: &str) -> String {
        format!("[{} #{image_id}: {}]", self.prefix, marker)
    }
}

/// 变换配置。
#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub enabled: bool,
    /// 目标模型列表 (空 = 全部模型均 text-only 处理)。
    pub target_models: Vec<String>,
    /// 变换后标记前缀。
    pub marker_prefix: String,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target_models: vec![
                "deepseek-v4-flash".to_string(),
                "deepseek-v4-pro".to_string(),
            ],
            marker_prefix: "IMG".to_string(),
        }
    }
}

/// 单条消息变换结果。
#[derive(Debug, Clone)]
pub struct Transformed {
    pub text: String,
    /// 检出并替换的图片数。
    pub images_replaced: usize,
    /// 未替换的图片标记 (analyzer 不可用时)。
    pub images_passthrough: usize,
}

impl fmt::Display for Transformed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (replaced={}, passthrough={})",
            self.text, self.images_replaced, self.images_passthrough
        )
    }
}
