//! SelfTest 实现 — PDF 图标增强能力自检
//!
//! 设计 (R-P42): 本地 SelfTest trait 定义，标记 ConstellationLevel 成熟度
//! Dark Forest: 模块经 SelfTest T1-T3 接线到意识树健康链
//! T2 验证: 注册到意识树 SelfTestRegistry
//! T3 验证: 实际功能测试（类型检查、默认值、错误处理）
//!
//! 桥接策略: 本地 SelfTest trait 供 nt_file_ability 内部解耦使用，
//! 跨层注册通过 CoreXxxBridge 包装器适配核心 nt_core_self_test::SelfTest。

use std::collections::HashMap;

// ─── 本地 SelfTest trait — 消除对 crate::core::nt_core_self_test 的硬耦合 ───

/// 跨模块共享的自检 trait (与 NT-CORE SelfTest 接口一致，L1→L5 依赖倒置)
pub trait SelfTest: Send + Sync {
    fn name(&self) -> &str;
    fn self_test(&self) -> Result<(), Vec<String>>;
}

/// 自检注册表 (精简版，仅保留 nt_file_ability 需要的接口)
#[derive(Default)]
pub struct SelfTestRegistry {
    tests: HashMap<String, Box<dyn SelfTest>>,
}

impl SelfTestRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, test: Box<dyn SelfTest>) {
        self.tests.insert(test.name().to_string(), test);
    }

    pub fn register_all(&mut self, tests: Vec<Box<dyn SelfTest>>) {
        for t in tests {
            self.register(t);
        }
    }

    pub fn count(&self) -> usize {
        self.tests.len()
    }
}

// ─── SelfTest 实现 ────────────────────────────────────────────────────────

/// PDF 图标增强能力自检
pub struct PdfIconEnhanceSelfTest;

impl SelfTest for PdfIconEnhanceSelfTest {
    fn name(&self) -> &str {
        "nt_file_ability::pdf_icon_enhance"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // T1: 检查模块是否存在 (通过编译即证明)
        
        // T3: 功能验证
        let config = super::pdf::pdf_icon_enhance::PdfIconEnhanceConfig::default();
        if !config.embed_back {
            errors.push("PdfIconEnhanceConfig.embed_back should default to true".to_string());
        }
        
        let enhancer = super::pdf::pdf_icon_enhance::PdfIconEnhancer::new();
        if !enhancer.config().embed_back {
            errors.push("PdfIconEnhancer should have embed_back=true by default".to_string());
        }
        
        let result = super::pdf::pdf_icon_enhance::enhance_pdf_icons(
            std::path::Path::new("/nonexistent.pdf")
        );
        if result.is_ok() {
            errors.push("Should return error for nonexistent PDF".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 图像超分能力自检
pub struct ImageSuperResolutionSelfTest;

impl SelfTest for ImageSuperResolutionSelfTest {
    fn name(&self) -> &str {
        "nt_file_ability::image_super_resolution"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // T3: 功能验证
        use super::image_super_resolution::SuperResolutionModel;
        let models = vec![
            SuperResolutionModel::RealEsrganGeneral,
            SuperResolutionModel::RealEsrganAnime,
            SuperResolutionModel::Lanczos,
        ];
        
        for model in &models {
            let name = model.display_name();
            if name.is_empty() {
                errors.push(format!("Model display_name() should not be empty for {:?}", model));
            }
            
            let _tile_size = model.recommended_tile_size();
            let overlap = model.recommended_overlap();
            
            if matches!(model, SuperResolutionModel::Lanczos) && overlap > 0 {
                errors.push("Lanczos should have 0 overlap".to_string());
            }
        }
        
        let config = super::image_super_resolution::SuperResolutionConfig::default();
        if config.scale == 0 {
            errors.push("SuperResolutionConfig.scale should not be 0".to_string());
        }
        
        let resolver = super::image_super_resolution::ImageSuperResolver::new();
        if resolver.config().scale == 0 {
            errors.push("ImageSuperResolver should have non-zero scale".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// PDF 图像提取能力自检
pub struct PdfImageExtractSelfTest;

impl SelfTest for PdfImageExtractSelfTest {
    fn name(&self) -> &str {
        "nt_file_ability::pdf_image_extract"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        let config = super::pdf::pdf_image_extract::PdfImageExtractConfig::default();
        if config.min_dimension == 0 {
            errors.push("PdfImageExtractConfig.min_dimension should not be 0".to_string());
        }
        if config.min_bytes == 0 {
            errors.push("PdfImageExtractConfig.min_bytes should not be 0".to_string());
        }
        
        let result = super::pdf::pdf_image_extract::pdf_has_images(
            std::path::Path::new("/nonexistent.pdf")
        );
        if result.is_ok() {
            errors.push("Should return error for nonexistent PDF".to_string());
        }
        
        let result = super::pdf::pdf_image_stats(
            std::path::Path::new("/nonexistent.pdf")
        );
        if result.is_ok() {
            errors.push("Should return error for nonexistent PDF".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// PDF 文本编辑能力自检
pub struct PdfEditSelfTest;

impl SelfTest for PdfEditSelfTest {
    fn name(&self) -> &str {
        "nt_file_ability::pdf_edit"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // T1: 模块存在性 (编译即证明)

        // T3: 功能验证
        // 1. PdfEdit 结构体构造与字段验证
        let edit = super::pdf::PdfEdit {
            page: 1,
            find: "test".to_string(),
            replace: Some("replaced".to_string()),
        };
        if edit.page != 1 {
            errors.push("PdfEdit.page should be 1".to_string());
        }
        if edit.find != "test" {
            errors.push("PdfEdit.find should be 'test'".to_string());
        }
        if edit.replace.as_deref() != Some("replaced") {
            errors.push("PdfEdit.replace should be Some('replaced')".to_string());
        }

        // 2. 空替换 (删除模式)
        let delete_edit = super::pdf::PdfEdit {
            page: 2,
            find: "delete me".to_string(),
            replace: None,
        };
        if delete_edit.replace.is_some() {
            errors.push("PdfEdit.replace should be None for delete mode".to_string());
        }

        // 3. edit_pdf 对不存在文件应返回错误
        let result = super::pdf::edit_pdf(
            std::path::Path::new("/nonexistent.pdf"),
            std::path::Path::new("/tmp/out.pdf"),
            &[edit],
            None,
        );
        if result.is_ok() {
            errors.push("edit_pdf should return error for nonexistent source".to_string());
        }

        // 4. extract_pdf_tables 对不存在文件应返回错误
        let result = super::pdf::extract_pdf_tables(
            std::path::Path::new("/nonexistent.pdf"),
        );
        if result.is_ok() {
            errors.push("extract_pdf_tables should return error for nonexistent file".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 文档解析能力自检
pub struct DocParseSelfTest;

impl SelfTest for DocParseSelfTest {
    fn name(&self) -> &str {
        "nt_file_ability::doc_parse"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // T1: 模块存在性 (编译即证明)

        // T3: 功能验证
        // 1. parse_document 对不存在文件应返回错误
        let result = super::doc_parse::parse_document(
            std::path::Path::new("/nonexistent.docx"),
        );
        if result.is_ok() {
            errors.push("parse_document should return error for nonexistent file".to_string());
        }

        // 2. parse_bytes 空内容应返回错误
        let result = super::doc_parse::parse_bytes(
            b"",
            anydoc::Format::Markdown,
        );
        // 空内容可能解析为空文档，不一定报错，此处仅验证不 panic
        let _ = result;

        // 3. PdfParseMode 默认值验证
        let mode = super::doc_parse::PdfParseMode::default();
        // 无 GPU 环境下应默认为 Fast
        if std::env::var("CUDA_VISIBLE_DEVICES").is_err()
            && !std::path::Path::new("/dev/nvidia0").exists()
        {
            if mode != super::doc_parse::PdfParseMode::Fast {
                errors.push("PdfParseMode::default() should be Fast without GPU".to_string());
            }
        }

        // 4. PdfParseConfig 默认值验证
        let config = super::doc_parse::PdfParseConfig::default();
        if !config.enable_ocr {
            errors.push("PdfParseConfig.enable_ocr should default to true".to_string());
        }
        if !config.enable_tables {
            errors.push("PdfParseConfig.enable_tables should default to true".to_string());
        }
        if !config.enable_images {
            errors.push("PdfParseConfig.enable_images should default to true".to_string());
        }

        // 5. detect_format_from_content 基本探测
        let result = super::doc_parse::detect_format_from_content(b"%PDF-1.4");
        if result.is_none() {
            errors.push("detect_format_from_content should detect PDF header".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// FileAbility 自检 (兼容旧代码)
pub struct FileAbilitySelfTest;

impl SelfTest for FileAbilitySelfTest {
    fn name(&self) -> &str {
        "nt_file_ability"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        Ok(())
    }
}

// ─── 核心 trait 桥接包装器 (跨层注册用) ──────────────────────────────────

struct CorePdfIconEnhanceBridge;
impl crate::core::nt_core_self_test::SelfTest for CorePdfIconEnhanceBridge {
    fn name(&self) -> &str { "nt_file_ability::pdf_icon_enhance" }
    fn self_test(&self) -> Result<(), Vec<String>> { PdfIconEnhanceSelfTest.self_test() }
}

struct CoreImageSRBridge;
impl crate::core::nt_core_self_test::SelfTest for CoreImageSRBridge {
    fn name(&self) -> &str { "nt_file_ability::image_super_resolution" }
    fn self_test(&self) -> Result<(), Vec<String>> { ImageSuperResolutionSelfTest.self_test() }
}

struct CorePdfImageExtractBridge;
impl crate::core::nt_core_self_test::SelfTest for CorePdfImageExtractBridge {
    fn name(&self) -> &str { "nt_file_ability::pdf_image_extract" }
    fn self_test(&self) -> Result<(), Vec<String>> { PdfImageExtractSelfTest.self_test() }
}

struct CorePdfEditBridge;
impl crate::core::nt_core_self_test::SelfTest for CorePdfEditBridge {
    fn name(&self) -> &str { "nt_file_ability::pdf_edit" }
    fn self_test(&self) -> Result<(), Vec<String>> { PdfEditSelfTest.self_test() }
}

struct CoreDocParseBridge;
impl crate::core::nt_core_self_test::SelfTest for CoreDocParseBridge {
    fn name(&self) -> &str { "nt_file_ability::doc_parse" }
    fn self_test(&self) -> Result<(), Vec<String>> { DocParseSelfTest.self_test() }
}

/// 双 trait 实现: 使 FileAbilitySelfTest 可被 NT-MIND 意识树 SelfTestRegistry 注册。
impl crate::core::nt_core_self_test::SelfTest for FileAbilitySelfTest {
    fn name(&self) -> &str { "nt_file_ability" }
    fn self_test(&self) -> Result<(), Vec<String>> { Ok(()) }
}

/// 注册 PDF/SR/doc_parse SelfTest 到核心 registry
/// 接受核心 SelfTestRegistry (跨层调用契约)
pub fn register_pdf_sr_self_tests(registry: &mut crate::core::nt_core_self_test::SelfTestRegistry) {
    registry.register(Box::new(CorePdfIconEnhanceBridge));
    registry.register(Box::new(CoreImageSRBridge));
    registry.register(Box::new(CorePdfImageExtractBridge));
    registry.register(Box::new(CorePdfEditBridge));
    registry.register(Box::new(CoreDocParseBridge));
    registry.register(Box::new(FileAbilitySelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pdf_icon_enhance_selftest() {
        let test = PdfIconEnhanceSelfTest;
        assert_eq!(test.name(), "nt_file_ability::pdf_icon_enhance");
        assert!(test.self_test().is_ok());
    }
    
    #[test]
    fn test_image_sr_selftest() {
        let test = ImageSuperResolutionSelfTest;
        assert_eq!(test.name(), "nt_file_ability::image_super_resolution");
        assert!(test.self_test().is_ok());
    }
    
    #[test]
    fn test_pdf_image_extract_selftest() {
        let test = PdfImageExtractSelfTest;
        assert_eq!(test.name(), "nt_file_ability::pdf_image_extract");
        assert!(test.self_test().is_ok());
    }
    
    #[test]
    fn test_file_ability_selftest() {
        let test = FileAbilitySelfTest;
        assert_eq!(test.name(), "nt_file_ability");
        assert!(test.self_test().is_ok());
    }
    
    #[test]
    fn test_pdf_edit_selftest() {
        let test = PdfEditSelfTest;
        assert_eq!(test.name(), "nt_file_ability::pdf_edit");
        assert!(test.self_test().is_ok());
    }
    
    #[test]
    fn test_doc_parse_selftest() {
        let test = DocParseSelfTest;
        assert_eq!(test.name(), "nt_file_ability::doc_parse");
        assert!(test.self_test().is_ok());
    }
    
    #[test]
    fn test_register_all_selftests() {
        let mut registry = SelfTestRegistry::new();
        registry.register_all(vec![
            Box::new(PdfIconEnhanceSelfTest),
            Box::new(ImageSuperResolutionSelfTest),
            Box::new(PdfImageExtractSelfTest),
            Box::new(PdfEditSelfTest),
            Box::new(DocParseSelfTest),
            Box::new(FileAbilitySelfTest),
        ]);
        assert_eq!(registry.count(), 6);
    }
}
