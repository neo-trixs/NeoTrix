//! PDF 增强能力节点 — 实现 UnifiedCapability trait
//!
//! 设计 (R-P42): 复用 pdf_icon_enhance + image_super_resolution，组合为能力节点
//! 跨域错位: 将文件能力映射到能力树节点

use std::sync::Arc;

// ─── 本地能力接口类型 — 消除对 crate::l6_meta::nt_core_capability 的硬耦合 ───

/// 层级定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    L1Action,
    L2Perception,
    L3Embodiment,
    L4Emotion,
    L5Cognition,
    L6Meta,
}

/// 简写别名 (兼容 nt_core_capability 原有 L1/L2/... 命名)
impl Layer {
    pub const L1: Self = Self::L1Action;
    pub const L2: Self = Self::L2Perception;
    pub const L3: Self = Self::L3Embodiment;
    pub const L4: Self = Self::L4Emotion;
    pub const L5: Self = Self::L5Cognition;
    pub const L6: Self = Self::L6Meta;
}

/// 域定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    NtCore,
    NtMind,
    NtMemory,
    NtWorld,
    NtAct,
    NtIo,
    NtShield,
    NtPhysical,
    NtFeel,
    NtFileAbility,
}

/// 能力元数据
#[derive(Debug, Clone)]
pub struct CapabilityMeta {
    pub id: String,
    pub name: String,
    pub layer: Layer,
    pub domain: Domain,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub status: CapabilityStatus,
    pub metrics: CapabilityMetrics,
    pub cost_weight: f64,
    pub priority: f64,
}

/// 能力状态指示器
#[derive(Debug, Clone)]
#[derive(Default)]
pub enum CapabilityStatus {
    #[default]
    Healthy,
    Degraded,
    Unhealthy,
}


/// 能力指标
#[derive(Debug, Clone)]
pub struct CapabilityMetrics {
    pub total_calls: u64,
    pub total_errors: u64,
    pub avg_latency_ms: f64,
    pub last_called: Option<std::time::Instant>,
}

impl Default for CapabilityMetrics {
    fn default() -> Self {
        Self {
            total_calls: 0,
            total_errors: 0,
            avg_latency_ms: 0.0,
            last_called: None,
        }
    }
}

/// 能力状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityState {
    Ready,
    Healthy,
    Running,
    Error(String),
    Disabled,
}

/// 能力健康度
#[derive(Debug, Clone)]
pub struct CapabilityHealth {
    pub state: CapabilityState,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub last_called: Option<std::time::Instant>,
    pub call_count: u64,
}

/// 文件增强输入
#[derive(Debug, Clone)]
pub struct FileEnhanceInput {
    pub input_path: String,
    pub output_path: Option<String>,
    pub mode: FileEnhanceMode,
}

/// 文件增强模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEnhanceMode {
    PdfIconEnhance,
    ImageSuperResolution,
}

/// 统一输入
#[derive(Debug, Clone)]
pub enum CapabilityInput {
    Text(String),
    FileEnhance(FileEnhanceInput),
}

/// 文件增强输出
#[derive(Debug, Clone)]
pub struct FileEnhanceOutput {
    pub success: bool,
    pub input_path: String,
    pub output_path: Option<String>,
    pub message: String,
}

/// 统一输出
#[derive(Debug)]
pub enum CapabilityOutput {
    Text(String),
    FileEnhance(FileEnhanceOutput),
}

/// 能力错误
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    #[error("不支持的输入: {0}")]
    UnsupportedInput(String),
    #[error("执行失败: {0}")]
    ExecutionFailed(String),
}

/// 统一能力 trait (精简版，仅保留 nt_file_ability 需要的接口)
pub trait UnifiedCapability: Send + Sync {
    fn meta(&self) -> CapabilityMeta;
    fn health(&self) -> CapabilityHealth;
    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError>;
    fn supports(&self, input: &CapabilityInput) -> bool;
}

/// PDF 增强能力
pub struct PdfEnhanceCapability;

impl PdfEnhanceCapability {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PdfEnhanceCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedCapability for PdfEnhanceCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-file-pdf-enhance".to_string(),
            name: "PDF Enhancement".to_string(),
            layer: Layer::L1Action,
            domain: Domain::NtFileAbility,
            version: "1.0.0".to_string(),
            description: "PDF icon/image super-resolution enhancement".to_string(),
            tags: vec!["pdf".to_string(), "enhance".to_string(), "super-resolution".to_string()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics::default(),
            cost_weight: 0.0,
            priority: 1.0,
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::FileEnhance(file_input) => {
                let input_path = std::path::Path::new(&file_input.input_path);
                
                let config = super::PdfIconEnhanceConfig {
                    output_pdf: file_input.output_path.map(std::path::PathBuf::from),
                    ..Default::default()
                };
                
                match super::enhance_pdf_icons_with_config(input_path, config) {
                    Ok(result) => Ok(CapabilityOutput::FileEnhance(FileEnhanceOutput {
                        success: result.success,
                        input_path: result.input_pdf,
                        output_path: Some(result.output_pdf),
                        message: format!(
                            "Extracted {} images, enhanced {}",
                            result.images_extracted,
                            result.images_enhanced
                        ),
                    })),
                    Err(e) => Err(CapabilityError::ExecutionFailed(e.to_string())),
                }
            }
            _ => Err(CapabilityError::UnsupportedInput(
                "Expected FileEnhance input".to_string()
            )),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::FileEnhance(_))
    }
}

/// 创建 PDF 增强能力 — 返回核心 trait 对象供 NT-CORE 能力工厂消费
///
/// **这是 PDF 增强能力的唯一正典注册路径**（2026-09-28）。
/// 旧的「本地注册中心」路线（`CapabilityRegistry` + `register_pdf_enhance_capability`）
/// 已删除：那是 4 份同名 `CapabilityRegistry` 中的一份早期重复实现，全仓零生产
/// 消费者（唯一引用是 README 的示例片段）。
pub fn create_pdf_enhance_capability() -> Arc<dyn crate::l0_substrate::nt_core_capability_types::UnifiedCapability> {
    Arc::new(PdfEnhanceCapability::new())
}

/// 双 trait 实现: 使 PdfEnhanceCapability 可被 NT-CORE CapabilityFactory 消费
impl crate::l0_substrate::nt_core_capability_types::UnifiedCapability for PdfEnhanceCapability {
    fn meta(&self) -> crate::l0_substrate::nt_core_capability_types::CapabilityMeta {
        crate::l0_substrate::nt_core_capability_types::CapabilityMeta {
            id: "nt-file-pdf-enhance".to_string(),
            name: "PDF Enhancement".to_string(),
            layer: crate::l0_substrate::nt_core_capability_types::Layer::L1Action,
            domain: crate::l0_substrate::nt_core_capability_types::Domain::NtFileAbility,
            version: "1.0.0".to_string(),
            description: "PDF icon/image super-resolution enhancement".to_string(),
            tags: vec!["pdf".to_string(), "enhance".to_string(), "super-resolution".to_string()],
            status: crate::l0_substrate::nt_core_capability_types::CapabilityStatus::Healthy,
            metrics: crate::l0_substrate::nt_core_capability_types::CapabilityMetrics::default(),
            cost_weight: 0.0,
            priority: 1.0,
        }
    }

    fn health(&self) -> crate::l0_substrate::nt_core_capability_types::CapabilityHealth {
        crate::l0_substrate::nt_core_capability_types::CapabilityHealth {
            state: crate::l0_substrate::nt_core_capability_types::CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: crate::l0_substrate::nt_core_capability_types::CapabilityInput) -> Result<crate::l0_substrate::nt_core_capability_types::CapabilityOutput, crate::l0_substrate::nt_core_capability_types::CapabilityError> {
        match input {
            crate::l0_substrate::nt_core_capability_types::CapabilityInput::FileEnhance(file_input) => {
                let input_path = std::path::Path::new(&file_input.input_path);
                let config = super::pdf::pdf_icon_enhance::PdfIconEnhanceConfig {
                    output_pdf: file_input.output_path.map(std::path::PathBuf::from),
                    ..Default::default()
                };
                match super::enhance_pdf_icons_with_config(input_path, config) {
                    Ok(result) => Ok(crate::l0_substrate::nt_core_capability_types::CapabilityOutput::FileEnhance(
                        crate::l0_substrate::nt_core_capability_types::FileEnhanceOutput {
                            success: result.success,
                            input_path: result.input_pdf,
                        output_path: result.output_pdf,
                            message: format!("Extracted {} images, enhanced {}", result.images_extracted, result.images_enhanced),
                        },
                    )),
                    Err(e) => Err(crate::l0_substrate::nt_core_capability_types::CapabilityError::ExecutionFailed(e.to_string())),
                }
            }
            _ => Err(crate::l0_substrate::nt_core_capability_types::CapabilityError::UnsupportedInput(
                "Expected FileEnhance input".to_string(),
            )),
        }
    }

    fn supports(&self, input: &crate::l0_substrate::nt_core_capability_types::CapabilityInput) -> bool {
        matches!(input, crate::l0_substrate::nt_core_capability_types::CapabilityInput::FileEnhance(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_enhance_capability_meta() {
        let cap = PdfEnhanceCapability::new();
        let meta = cap.meta();
        assert_eq!(meta.id, "nt-file-pdf-enhance");
        assert_eq!(meta.domain, Domain::NtFileAbility);
        assert_eq!(meta.layer, Layer::L1);
    }

    #[test]
    fn test_pdf_enhance_capability_health() {
        let cap = PdfEnhanceCapability::new();
        let health = cap.health();
        assert!(health.success_rate > 0.9);
    }

    #[test]
    fn test_pdf_enhance_capability_supports() {
        let cap = PdfEnhanceCapability::new();
        
        let file_input = CapabilityInput::FileEnhance(FileEnhanceInput {
            input_path: "test.pdf".to_string(),
            output_path: None,
            mode: FileEnhanceMode::PdfIconEnhance,
        });
        assert!(cap.supports(&file_input));
        
        let text_input = CapabilityInput::Text("test".to_string());
        assert!(!cap.supports(&text_input));
    }
}
