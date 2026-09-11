//! NT-FILE-ABILITY PDF 增强统一能力接口实现
//!
//! 封装 PdfIconEnhancer 为 UnifiedCapability，支持:
//! - PDF 图标清晰度提升 (提取→超分→嵌入)
//! - 路由到 NT-IO 域能力注册中心
//!
//! 设计 (R-P42): 复用 pdf_icon_enhance 既有管线，不平行重造

use std::path::Path;
use std::sync::Arc;

use crate::core::nt_core_capability::*;

use super::pdf_icon_enhance::{PdfIconEnhancer, PdfIconEnhanceConfig};

/// PDF 增强能力实现
pub struct PdfEnhanceCapability {
    meta: CapabilityMeta,
    health: CapabilityHealth,
}

impl PdfEnhanceCapability {
    /// 创建新的 PDF 增强能力
    pub fn new() -> Self {
        Self {
            meta: CapabilityMeta {
                id: "nt-file-pdf-enhance".into(),
                name: "NT-FILE PDF Icon Enhance".into(),
                layer: Layer::L1Action,
                domain: Domain::NtIo,
                version: "0.1.0".into(),
                description: "PDF 图标清晰度提升管线: 提取→超分→嵌入".into(),
                tags: vec![
                    "pdf".into(),
                    "enhance".into(),
                    "super-resolution".into(),
                    "icon".into(),
                ],
                status: CapabilityStatus::Healthy,
                metrics: CapabilityMetrics::default(),
            },
            health: CapabilityHealth {
                state: CapabilityState::Ready,
                success_rate: 1.0,
                avg_latency_ms: 0.0,
                last_called: None,
                call_count: 0,
            },
        }
    }
}

impl Default for PdfEnhanceCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedCapability for PdfEnhanceCapability {
    fn meta(&self) -> CapabilityMeta {
        self.meta.clone()
    }

    fn health(&self) -> CapabilityHealth {
        self.health.clone()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::FileEnhance(req) => {
                if req.mode != FileEnhanceMode::PdfIconEnhance {
                    return Err(CapabilityError::UnsupportedInput(format!(
                        "不支持的增强模式: {:?}",
                        req.mode
                    )));
                }

                let input_path = Path::new(&req.input_path);
                if !input_path.exists() {
                    return Err(CapabilityError::ExecutionFailed(format!(
                        "文件不存在: {}",
                        req.input_path
                    )));
                }

                let config = PdfIconEnhanceConfig {
                    output_pdf: req.output_path.map(std::path::PathBuf::from),
                    ..PdfIconEnhanceConfig::default()
                };

                let enhancer = PdfIconEnhancer::with_config(config);
                match enhancer.enhance(input_path) {
                    Ok(result) => Ok(CapabilityOutput::FileEnhance(FileEnhanceOutput {
                        success: result.success,
                        input_path: result.input_pdf,
                        output_path: result.output_pdf,
                        message: format!(
                            "提取 {} 张图像, 成功增强 {} 张, 失败 {} 张, 总耗时 {}ms",
                            result.images_extracted,
                            result.images_enhanced,
                            result.images_failed,
                            result.total_time_ms
                        ),
                    })),
                    Err(e) => Err(CapabilityError::ExecutionFailed(format!(
                        "PDF 增强失败: {e}"
                    ))),
                }
            }
            other => Err(CapabilityError::UnsupportedInput(format!(
                "期望 FileEnhance 输入, 收到: {other:?}"
            ))),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(
            input,
            CapabilityInput::FileEnhance(FileEnhanceInput {
                mode: FileEnhanceMode::PdfIconEnhance,
                ..
            })
        )
    }
}

/// 创建 PDF 增强能力实例
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
    fn pdf_enhance_capability_meta() {
        let cap = PdfEnhanceCapability::new();
        let meta = cap.meta();
        assert_eq!(meta.id, "nt-file-pdf-enhance");
        assert_eq!(meta.layer, Layer::L1Action);
        assert_eq!(meta.domain, Domain::NtIo);
        assert!(meta.tags.contains(&"pdf".to_string()));
    }

    #[test]
    fn pdf_enhance_health() {
        let cap = PdfEnhanceCapability::new();
        let health = cap.health();
        assert_eq!(health.state, CapabilityState::Ready);
        assert_eq!(health.success_rate, 1.0);
    }

    #[test]
    fn pdf_enhance_supports() {
        let cap = PdfEnhanceCapability::new();

        assert!(cap.supports(&CapabilityInput::FileEnhance(FileEnhanceInput {
            input_path: "test.pdf".into(),
            output_path: None,
            mode: FileEnhanceMode::PdfIconEnhance,
        })));

        assert!(!cap.supports(&CapabilityInput::FileEnhance(FileEnhanceInput {
            input_path: "test.png".into(),
            output_path: None,
            mode: FileEnhanceMode::ImageSuperResolution,
        })));

        assert!(!cap.supports(&CapabilityInput::Text("hello".into())));
    }

    #[test]
    fn pdf_enhance_execute_missing_file() {
        let cap = PdfEnhanceCapability::new();
        let input = CapabilityInput::FileEnhance(FileEnhanceInput {
            input_path: "/nonexistent/file.pdf".into(),
            output_path: None,
            mode: FileEnhanceMode::PdfIconEnhance,
        });

        let result = cap.execute(input);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::ExecutionFailed(msg) => {
                assert!(msg.contains("文件不存在"));
            }
            other => panic!("期望 ExecutionFailed, 收到: {other:?}"),
        }
    }

    #[test]
    fn pdf_enhance_execute_unsupported_mode() {
        let cap = PdfEnhanceCapability::new();
        let input = CapabilityInput::FileEnhance(FileEnhanceInput {
            input_path: "test.pdf".into(),
            output_path: None,
            mode: FileEnhanceMode::ImageSuperResolution,
        });

        let result = cap.execute(input);
        assert!(result.is_err());
        match result.unwrap_err() {
            CapabilityError::UnsupportedInput(msg) => {
                assert!(msg.contains("不支持的增强模式"));
            }
            other => panic!("期望 UnsupportedInput, 收到: {other:?}"),
        }
    }

    #[test]
    fn pdf_enhance_register() {
        let mut registry = CapabilityRegistry::new();
        register_pdf_enhance_capability(&mut registry);
        let cap = registry.get("nt-file-pdf-enhance");
        assert!(cap.is_some());
        assert_eq!(cap.unwrap().meta().domain, Domain::NtIo);
    }
}
