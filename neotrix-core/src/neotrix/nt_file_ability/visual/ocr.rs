//! OCR 抽象 (Ext-2) — 图像 → 文字识别 trait 与启发式基线实现。

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::core::FileAbility;
use super::types::FileKind;

/// OCR 引擎抽象 — 图像 → 文字识别。
///
/// NeoTrix 零 unsafe 纪律下 OCR 接入点: 任何自研/第三方 OCR (tesseract 封装、
/// PaddleOCR、视觉多模态模型) 都通过此 trait 注入, 由 FileAbility 统一调度。
/// 默认实现 `RuleBasedOcr` 提供确定性启发式基线 (低置信), 后续可替换为
/// `TesseractOcr`/`VisionModelOcr` 而不改调用方。
pub trait OcrEngine: Send + Sync {
    /// 引擎名
    fn name(&self) -> &str;
    /// 识别图像文件中的文字
    fn recognize(&self, path: &Path) -> OcrResult;
}

/// OCR 结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f64,
    pub engine: String,
}

/// 启发式基线 OCR — 非真实视觉识别, 用于测试 trait 链路与占位
///
/// 说明: NeoTrix 不内置真实 OCR 模型 (R-P1 零 unsafe + 体积约束)。
/// 该实现从图像 EXIF 文本字段/文件名启发式提取, confidence 固定低值,
/// 真实场景请注入 `VisionModelOcr` (例: image → question → LLM 描述)。
/// Hosted 决策: 占位引擎保持模块可编译、链路可测、可被视觉引擎替换。
pub struct RuleBasedOcr;

impl OcrEngine for RuleBasedOcr {
    fn name(&self) -> &str {
        "rule-based"
    }

    fn recognize(&self, path: &Path) -> OcrResult {
        // 提取文件名作为低置信启发 (图像场景无真实文字时退化降级)
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        OcrResult {
            text: stem,
            confidence: 0.05,
            engine: self.name().to_string(),
        }
    }
}

impl FileAbility {
    /// 用给定 OCR 引擎识别图像文字 (动态接线: 默认 RuleBasedOcr)
    pub fn ocr(&self, engine: Option<Box<dyn OcrEngine>>) -> OcrResult {
        if self.kind != FileKind::Image {
            return OcrResult::default();
        }
        let e = engine.unwrap_or_else(|| Box::new(RuleBasedOcr));
        e.recognize(&self.path)
    }
}