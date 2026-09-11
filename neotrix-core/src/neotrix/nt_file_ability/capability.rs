//! PDF 增强能力节点 — 实现 UnifiedCapability trait
//!
//! 设计 (R-P42): 复用 pdf_icon_enhance + image_super_resolution，组合为能力节点
//! 跨域错位: 将文件能力映射到能力树节点

use std::sync::Arc;
use crate::core::nt_core_capability::*;

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
            layer: Layer::L1,
            domain: Domain::NtFileAbility,
            version: "1.0.0".to_string(),
            description: "PDF icon/image super-resolution enhancement".to_string(),
            tags: vec!["pdf".to_string(), "enhance".to_string(), "super-resolution".to_string()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics::default(),
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
                        output_path: result.output_pdf,
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

/// 创建 PDF 增强能力
pub fn create_pdf_enhance_capability() -> Arc<dyn UnifiedCapability> {
    Arc::new(PdfEnhanceCapability::new())
}

/// 注册 PDF 增强能力到注册中心
pub fn register_pdf_enhance_capability(registry: &mut CapabilityRegistry) {
    registry.register(create_pdf_enhance_capability());
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
        assert!(health.score > 0.9);
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
