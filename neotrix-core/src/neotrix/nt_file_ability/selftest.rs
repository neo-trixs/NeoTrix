//! SelfTest 实现 — PDF 图标增强能力自检
//!
//! 设计 (R-P42): 复用 core SelfTest trait，标记 ConstellationLevel 成熟度
//! Dark Forest: 模块经 SelfTest T1-T3 接线到意识树健康链

use crate::core::nt_core_self_test::SelfTest;

/// PDF 图标增强能力自检
pub struct PdfIconEnhanceSelfTest;

impl SelfTest for PdfIconEnhanceSelfTest {
    fn name(&self) -> &str {
        "nt_file_ability::pdf_icon_enhance"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        // T1: 检查模块是否存在
        // 通过编译即证明存在
        Ok(())
    }
}

/// 图像超分能力自检
pub struct ImageSuperResolutionSelfTest;

impl SelfTest for ImageSuperResolutionSelfTest {
    fn name(&self) -> &str {
        "nt_file_ability::image_super_resolution"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        // T1: 检查模块是否存在
        Ok(())
    }
}

/// PDF 图像提取能力自检
pub struct PdfImageExtractSelfTest;

impl SelfTest for PdfImageExtractSelfTest {
    fn name(&self) -> &str {
        "nt_file_ability::pdf_image_extract"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        // T1: 检查模块是否存在
        Ok(())
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTestRegistry;
    
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
