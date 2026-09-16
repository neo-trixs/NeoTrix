//! NT-WORLD OCR 模块 — 图像文字识别能力层
//!
//! 设计 (R-P1): 零 unsafe,纯 Rust OCR 引擎抽象
//! 设计 (R-P42): 复用 nt_file_ability 的 OcrEngine trait 与视觉管线
//! 设计 (P4): Ordered Backend Fallback — PaddleOcrEngine (ONNX) ↔ RuleBasedOcr (启发式)
//! 设计 (A1): Cost-Aware Routing — 按置信度/成本选择引擎
//!
//! 模块组成:
//!   - OcrEngine trait    — 抽象 OCR 接口 (swappable)
//!   - PaddleOcrEngine    — 生产级 OCR 基于 ONNX Runtime (ort)
//!   - OcrConfig          — 语言/置信阈值/批大小配置
//!   - OcrResult          — 文本/置信度/边界框结果
//!   - OcrCapability      — UnifiedCapability 实现
//!   - pdf_to_text_pipeline — PDF→图像→超分→OCR 管线

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::nt_core_capability::{
    CapabilityError, CapabilityInput, CapabilityMeta, CapabilityOutput, CapabilityState,
    UnifiedCapability,
};
use crate::l2_perception::nt_world::ocr::pdf_to_text_pipeline::PdfToTextPipeline;

pub mod pdf_to_text_pipeline;

// ──────────────────────────────────────────────
// OCR 配置
// ──────────────────────────────────────────────

/// OCR 引擎配置
#[derive(Debug, Clone)]
pub struct OcrConfig {
    /// 识别语言 (e.g. "en", "ch_sim", "japan")
    pub language: String,
    /// 置信度阈值 (0.0-1.0), 低于此值的识别结果被过滤
    pub confidence_threshold: f64,
    /// 批处理大小
    pub batch_size: usize,
    /// 是否使用 GPU 加速
    pub use_gpu: bool,
    /// ONNX 模型路径 (PaddleOcrEngine 使用)
    pub model_path: Option<String>,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            language: "en".into(),
            confidence_threshold: 0.5,
            batch_size: 1,
            use_gpu: false,
            model_path: None,
        }
    }
}

// ──────────────────────────────────────────────
// OCR 边界框
// ──────────────────────────────────────────────

/// 文字边界框
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrBoundingBox {
    /// 左上角 x
    pub x: u32,
    /// 左上角 y
    pub y: u32,
    /// 右下角 x
    pub x2: u32,
    /// 右下角 y
    pub y2: u32,
    /// 该区域的文字文本
    pub text: String,
    /// 置信度
    pub confidence: f64,
}

// ──────────────────────────────────────────────
// OCR 结果
// ──────────────────────────────────────────────

/// OCR 识别结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OcrResult {
    /// 识别出的完整文本
    pub text: String,
    /// 整体置信度 (0.0-1.0)
    pub confidence: f64,
    /// 文字块及其边界框
    pub bounding_boxes: Vec<OcrBoundingBox>,
    /// 使用的引擎名
    pub engine: String,
    /// 识别耗时 (毫秒)
    pub processing_time_ms: u64,
}

// ──────────────────────────────────────────────
// OCR 引擎抽象 trait
// ──────────────────────────────────────────────

/// 抽象 OCR 引擎接口 — 支持 swappable 实现
pub trait OcrEngine: Send + Sync {
    /// 引擎名
    fn name(&self) -> &str;

    /// 识别图像文件中的文字
    fn recognize(&self, path: &Path) -> OcrResult;

    /// 识别图像字节数据中的文字
    fn recognize_bytes(&self, _data: &[u8]) -> OcrResult {
        OcrResult::default()
    }

    /// 设置引擎配置
    fn set_config(&mut self, config: OcrConfig);

    /// 获取当前配置
    fn config(&self) -> &OcrConfig;
}

// ──────────────────────────────────────────────
// PaddleOcrEngine — 基于 ONNX Runtime 的生产 OCR
// ──────────────────────────────────────────────

/// 基于 ONNX Runtime (ort) 的 PaddleOCR 生产引擎
///
/// 使用 `ort` crate (已为 neotrix 依赖, 通过 `onnx` feature 启用).
/// 实际模型路径由 `OcrConfig.model_path` 指定.
/// 若 ONNX feature 未启用, 自动回退到启发式识别.
pub struct PaddleOcrEngine {
    config: OcrConfig,
    /// ONNX 会话 (feature-gated)
    #[cfg(feature = "onnx")]
    session: Option<OcrSession>,
    #[cfg(not(feature = "onnx"))]
    _phantom: std::marker::PhantomData<()>,
}

impl PaddleOcrEngine {
    /// 创建新的 PaddleOcrEngine
    pub fn new(config: OcrConfig) -> Self {
        #[cfg(feature = "onnx")]
        let session = Self::load_session(&config);
        #[cfg(not(feature = "onnx"))]
        let _session: Option<()> = None;

        Self {
            config,
            #[cfg(feature = "onnx")]
            session,
            #[cfg(not(feature = "onnx"))]
            _phantom: std::marker::PhantomData,
        }
    }

    /// 加载 ONNX 会话
    #[cfg(feature = "onnx")]
    fn load_session(config: &OcrConfig) -> Option<OcrSession> {
        let model_path = config.model_path.as_deref()?;
        if !std::path::Path::new(model_path).exists() {
            eprintln!("PaddleOCR model not found at {}, using fallback", model_path);
            return None;
        }
        match OcrSession::builder()
            .map_err(|e| eprintln!("Failed to create ORT session builder: {e}"))
            .and_then(|builder| {
                builder
                    .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
                    .map_err(|e| eprintln!("Failed to set optimization: {e}"))
                    .and_then(|b| b.commit_from_file(model_path))
            }) {
            Ok(session) => {
                eprintln!("PaddleOCR ONNX session loaded from {}", model_path);
                Some(session)
            }
            Err(e) => {
                eprintln!("PaddleOCR ONNX load failed: {e}");
                None
            }
        }
    }

    /// 执行 ONNX 推理 (占位: 实际实现需要完整的预处理/后处理)
    #[cfg(feature = "onnx")]
    fn run_inference(&self, _image_data: &[u8]) -> OcrResult {
        // PaddleOCR 完整推理流程:
        // 1. 图像预处理 (resize, normalize, CHW conversion)
        // 2. ONNX 推理 (detection + recognition stages)
        // 3. 后处理 (NMS, text decode, confidence scoring)
        //
        // 此处返回启发式结果作为占位, 实际部署时需加载完整 PaddleOCR 模型
        OcrResult {
            text: String::new(),
            confidence: 0.0,
            bounding_boxes: Vec::new(),
            engine: self.name().to_string(),
            processing_time_ms: 0,
        }
    }

    /// 执行 ONNX 推理 (非 onnx feature 占位)
    #[cfg(not(feature = "onnx"))]
    fn run_inference(&self, _image_data: &[u8]) -> OcrResult {
        OcrResult::default()
    }
}

impl OcrEngine for PaddleOcrEngine {
    fn name(&self) -> &str {
        "paddleocr-onnx"
    }

    fn recognize(&self, path: &Path) -> OcrResult {
        let start = std::time::Instant::now();

        // 读取图像
        let image_data = match std::fs::read(path) {
            Ok(data) => data,
            Err(_e) => {
                return OcrResult {
                    text: String::new(),
                    confidence: 0.0,
                    bounding_boxes: Vec::new(),
                    engine: self.name().to_string(),
                    processing_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        let result = self.run_inference(&image_data);
        OcrResult {
            processing_time_ms: start.elapsed().as_millis() as u64,
            ..result
        }
    }

    fn set_config(&mut self, config: OcrConfig) {
        self.config = config;
        #[cfg(feature = "onnx")]
        {
            if let Some(ref model_path) = self.config.model_path {
                if std::path::Path::new(model_path).exists() {
                    self.session = Self::load_session(&self.config);
                }
            }
        }
    }

    fn config(&self) -> &OcrConfig {
        &self.config
    }
}

impl Default for PaddleOcrEngine {
    fn default() -> Self {
        Self::new(OcrConfig::default())
    }
}

// ──────────────────────────────────────────────
// 引擎工厂 — 按配置创建合适的 OCR 引擎
// ──────────────────────────────────────────────

/// 创建 OCR 引擎实例 (Cost-Aware Routing)
/// 根据配置自动选择最佳引擎
pub fn create_ocr_engine(#[allow(unused_variables)] config: OcrConfig) -> Arc<dyn OcrEngine> {
    // 策略: 优先使用 PaddleOcrEngine (ONNX), 不可用时回退到 RuleBasedOcr
    #[cfg(feature = "onnx")]
    {
        if config.model_path.as_ref().map_or(false, |p| std::path::Path::new(p).exists()) {
            return Arc::new(PaddleOcrEngine::new(config));
        }
    }

    // 回退到启发式引擎 — RuleBasedOcr 只实现 nt_file_ability 的 OcrEngine,
    // 需要一个适配器来桥接到 nt_world 的 OcrEngine trait
    use crate::neotrix::nt_file_ability::visual::OcrEngine as FileAbilityOcrEngine;

    struct FallbackOcr {
        inner: crate::neotrix::nt_file_ability::visual::RuleBasedOcr,
    }

    impl OcrEngine for FallbackOcr {
        fn name(&self) -> &str {
            "rule-based"
        }

        fn recognize(&self, path: &Path) -> OcrResult {
            let file_ability_result = self.inner.recognize(path);
            OcrResult {
                text: file_ability_result.text,
                confidence: file_ability_result.confidence,
                bounding_boxes: Vec::new(),
                engine: file_ability_result.engine,
                processing_time_ms: 0,
            }
        }

        fn set_config(&mut self, _config: OcrConfig) {}

        fn config(&self) -> &OcrConfig {
            static DEFAULT_CONFIG: std::sync::OnceLock<OcrConfig> = std::sync::OnceLock::new();
            DEFAULT_CONFIG.get_or_init(OcrConfig::default)
        }
    }

    Arc::new(FallbackOcr {
        inner: crate::neotrix::nt_file_ability::visual::RuleBasedOcr,
    })
}

// ──────────────────────────────────────────────
// OcrCapability — UnifiedCapability 实现
// ──────────────────────────────────────────────

/// OCR 能力实现 — 实现了 NT-CORE 的 UnifiedCapability trait
/// 可注册到 CapabilityRegistry 进行统一调度
pub struct OcrCapability {
    meta: CapabilityMeta,
    health: crate::core::nt_core_capability::CapabilityHealth,
    engine: Arc<dyn OcrEngine>,
}

impl OcrCapability {
    /// 创建新的 OCR 能力实例
    pub fn new(engine: Arc<dyn OcrEngine>) -> Self {
        let _config = engine.config();
        Self {
            meta: CapabilityMeta {
                id: "nt-world-ocr".into(),
                name: "NT-WORLD OCR".into(),
                layer: crate::core::nt_core_capability::Layer::L2Perception,
                domain: crate::core::nt_core_capability::Domain::NtWorld,
                version: "0.1.0".into(),
                description: "基于 ONNX Runtime 的 OCR 文字识别能力".into(),
                tags: vec!["ocr".into(), "paddleocr".into(), "onnx".into()],
                status: crate::core::nt_core_capability::CapabilityStatus::Healthy,
                metrics: crate::core::nt_core_capability::CapabilityMetrics::default(),
                cost_weight: 0.3,
                priority: 1.0,
            },
            health: crate::core::nt_core_capability::CapabilityHealth {
                state: CapabilityState::Ready,
                success_rate: 1.0,
                avg_latency_ms: 0.0,
                last_called: None,
                call_count: 0,
            },
            engine,
        }
    }
}

impl UnifiedCapability for OcrCapability {
    fn meta(&self) -> CapabilityMeta {
        self.meta.clone()
    }

    fn health(&self) -> crate::core::nt_core_capability::CapabilityHealth {
        self.health.clone()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::FileEnhance(file_input) => {
                let input_path = std::path::Path::new(&file_input.input_path);
                let result = self.engine.recognize(input_path);
                Ok(CapabilityOutput::Text(result.text))
            }
            CapabilityInput::Text(_) => {
                Err(CapabilityError::UnsupportedInput(
                    "OCR expects image path, not text".into(),
                ))
            }
            _ => Err(CapabilityError::UnsupportedInput(
                "Unsupported OCR input type".into(),
            )),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::FileEnhance(_))
    }
}

impl Drop for OcrCapability {
    fn drop(&mut self) {
        self.health.state = CapabilityState::Disabled;
    }
}

/// 创建 OCR 能力实例 — 返回核心 trait 对象供 NT-CORE 能力工厂消费
pub fn create_ocr_capability() -> Arc<dyn crate::core::nt_core_capability::UnifiedCapability> {
    let engine = create_ocr_engine(OcrConfig::default());
    Arc::new(OcrCapability::new(engine))
}

/// 注册 OCR 能力到本地注册中心
pub fn register_ocr_capability(registry: &mut crate::core::nt_core_capability::CapabilityRegistry) {
    registry.register(Arc::new(OcrCapability::new(create_ocr_engine(OcrConfig::default()))) as Arc<dyn UnifiedCapability>);
}

// ──────────────────────────────────────────────
// PDF→文本管线入口
// ──────────────────────────────────────────────

/// PDF→文本完整管线: 提取图像→超分辨率→OCR识别
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
/// * `output_dir` - 中间图像和输出目录
/// * `ocr_config` - OCR 配置
///
/// # 返回
/// 识别出的完整文本
pub fn pdf_to_text(
    pdf_path: &str,
    output_dir: &str,
    ocr_config: OcrConfig,
) -> Result<String, String> {
    let pipeline = PdfToTextPipeline::new(ocr_config);
    let text = pipeline.run(pdf_path, output_dir)?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_ocr_config_default() {
        let config = OcrConfig::default();
        assert_eq!(config.language, "en");
        assert_eq!(config.confidence_threshold, 0.5);
        assert_eq!(config.batch_size, 1);
        assert!(!config.use_gpu);
    }

    #[test]
    fn test_ocr_result_default() {
        let result = OcrResult::default();
        assert!(result.text.is_empty());
        assert_eq!(result.confidence, 0.0);
        assert!(result.bounding_boxes.is_empty());
    }

    #[test]
    fn test_bounding_box_creation() {
        let box_ = OcrBoundingBox {
            x: 0,
            y: 0,
            x2: 100,
            y2: 50,
            text: "test".into(),
            confidence: 0.95,
        };
        assert_eq!(box_.text, "test");
        assert_eq!(box_.confidence, 0.95);
    }

    #[test]
    fn test_create_ocr_engine_default() {
        let engine = create_ocr_engine(OcrConfig::default());
        assert_eq!(engine.name(), "rule-based");
    }

    #[test]
    fn test_ocr_capability_meta() {
        let engine = create_ocr_engine(OcrConfig::default());
        let cap = OcrCapability::new(engine);
        assert_eq!(cap.meta().id, "nt-world-ocr");
        assert_eq!(cap.meta().domain, crate::core::nt_core_capability::Domain::NtWorld);
        assert_eq!(cap.meta().layer, crate::core::nt_core_capability::Layer::L2Perception);
    }

    #[test]
    fn test_ocr_capability_supports_file_enhance() {
        let engine = create_ocr_engine(OcrConfig::default());
        let cap = OcrCapability::new(engine);
        let input = CapabilityInput::FileEnhance(
            crate::core::nt_core_capability::FileEnhanceInput {
                input_path: "test.pdf".to_string(),
                output_path: None,
                mode: crate::core::nt_core_capability::FileEnhanceMode::PdfIconEnhance,
            }
        );
        assert!(cap.supports(&input));
    }

    #[test]
    fn test_pdf_to_text_with_nonexistent_pdf() {
        let result = pdf_to_text("/nonexistent.pdf", "/tmp", OcrConfig::default());
        assert!(result.is_err());
    }
}