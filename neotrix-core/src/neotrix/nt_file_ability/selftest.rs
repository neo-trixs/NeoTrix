//! SelfTest 实现 — PDF 图标增强能力自检
//!
//! 设计 (R-P42): 复用 core SelfTest trait，标记 ConstellationLevel 成熟度
//! Dark Forest: 模块经 SelfTest T1-T3 接线到意识树健康链
//! T2 验证: 注册到意识树 SelfTestRegistry
//! T3 验证: 实际功能测试（类型检查、默认值、错误处理）

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
        // 1. 验证配置类型可以创建
        let config = super::pdf::pdf_icon_enhance::PdfIconEnhanceConfig::default();
        if !config.embed_back {
            errors.push("PdfIconEnhanceConfig.embed_back should default to true".to_string());
        }
        
        // 2. 验证增强器可以创建
        let enhancer = super::pdf::pdf_icon_enhance::PdfIconEnhancer::new();
        if !enhancer.config().embed_back {
            errors.push("PdfIconEnhancer should have embed_back=true by default".to_string());
        }
        
        // 3. 验证错误处理
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
        
        // T1: 检查模块是否存在
        
        // T3: 功能验证
        // 1. 验证 SuperResolutionModel 枚举变体
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
            
            // Lanczos 不应有 overlap
            if matches!(model, SuperResolutionModel::Lanczos) && overlap > 0 {
                errors.push("Lanczos should have 0 overlap".to_string());
            }
        }
        
        // 2. 验证配置默认值
        let config = super::image_super_resolution::SuperResolutionConfig::default();
        if config.scale == 0 {
            errors.push("SuperResolutionConfig.scale should not be 0".to_string());
        }
        
        // 3. 验证处理器可以创建
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
        
        // T1: 检查模块是否存在
        
        // T3: 功能验证
        // 1. 验证配置默认值
        let config = super::pdf::pdf_image_extract::PdfImageExtractConfig::default();
        if config.min_dimension == 0 {
            errors.push("PdfImageExtractConfig.min_dimension should not be 0".to_string());
        }
        if config.min_bytes == 0 {
            errors.push("PdfImageExtractConfig.min_bytes should not be 0".to_string());
        }
        
        // 2. 验证错误处理
        let result = super::pdf::pdf_image_extract::pdf_has_images(
            std::path::Path::new("/nonexistent.pdf")
        );
        if result.is_ok() {
            errors.push("Should return error for nonexistent PDF".to_string());
        }
        
        // pdf_image_stats 位于 pdf 子模块, 需通过 super::pdf 访问
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

/// FileAbility 自检 (兼容旧代码)
pub struct FileAbilitySelfTest;

impl SelfTest for FileAbilitySelfTest {
    fn name(&self) -> &str {
        "nt_file_ability"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        // T1: 检查模块是否存在
        // 通过编译即证明
        Ok(())
    }
}

/// 注册 PDF/SR SelfTest 到主 registry
pub fn register_pdf_sr_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(PdfIconEnhanceSelfTest));
    registry.register(Box::new(ImageSuperResolutionSelfTest));
    registry.register(Box::new(PdfImageExtractSelfTest));
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
    fn test_register_all_selftests() {
        let mut registry = SelfTestRegistry::new();
        registry.register_all(vec![
            Box::new(PdfIconEnhanceSelfTest),
            Box::new(ImageSuperResolutionSelfTest),
            Box::new(PdfImageExtractSelfTest),
            Box::new(FileAbilitySelfTest),
        ]);
        assert_eq!(registry.count(), 4);
    }
}
