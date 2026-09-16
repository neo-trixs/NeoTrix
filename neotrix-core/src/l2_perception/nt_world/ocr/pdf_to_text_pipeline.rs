//! PDF→文本完整管线
//!
//! 流水线步骤:
//!   Step 1: PDF 图像提取 — `pdf_image_extract::extract_pdf_images()`
//!   Step 2: 图像超分辨率增强 — `ImageSuperResolver::upscale()`
//!   Step 3: OCR 文字识别 — `OcrEngine::recognize()`
//!
//! 设计 (R-P42): 复用 nt_file_ability 的提取和超分能力,
//! 不平行重造 PDF 解析和图像增强逻辑。
//! 设计 (P4): Ordered Backend Fallback — 任一阶段失败时记录错误但继续管线。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::l2_perception::nt_world::ocr::{OcrConfig, OcrEngine, OcrResult};
use crate::neotrix::nt_file_ability::pdf::pdf_image_extract;
use crate::neotrix::nt_file_ability::{ImageSuperResolver, SuperResolutionConfig, SuperResolutionModel};

/// PDF→文本管线
pub struct PdfToTextPipeline {
    ocr_config: OcrConfig,
    super_res_config: SuperResolutionConfig,
}

impl PdfToTextPipeline {
    /// 创建新的 PDF→文本管线
    pub fn new(ocr_config: OcrConfig) -> Self {
        Self {
            ocr_config,
            super_res_config: SuperResolutionConfig {
                model: SuperResolutionModel::RealEsrganGeneral,
                scale: 2,
                ..Default::default()
            },
        }
    }

    /// 执行完整管线: PDF → 提取图像 → 超分辨率 → OCR
    ///
    /// # 参数
    /// * `pdf_path` - PDF 文件路径
    /// * `output_dir` - 中间图像输出目录
    ///
    /// # 返回
    /// 识别出的完整文本
    pub fn run(&self, pdf_path: &str, output_dir: &str) -> Result<String, String> {
        let pdf_path_buf = PathBuf::from(pdf_path);
        let output_dir_buf = PathBuf::from(output_dir);

        if !pdf_path_buf.exists() {
            return Err(format!("PDF 文件不存在: {}", pdf_path));
        }

        // Step 1: Extract images from PDF
        let images = self.extract_images(&pdf_path_buf, &output_dir_buf)?;

        if images.is_empty() {
            return Err("PDF 中未找到可识别的图像".into());
        }

        // Step 2: Enhance resolution + Step 3: OCR
        let mut all_text = String::new();
        let mut _total_confidence = 0.0;
        let mut _box_count = 0u64;

        for extracted in &images {
            // Step 2: Enhance resolution via image_super_resolution
            let enhanced_path = self.enhance_image(&extracted.output_path, &output_dir_buf)?;

            // Step 3: OCR recognize via ocr::recognize
            let engine = self.create_ocr_engine();
            let result: OcrResult = engine.recognize(Path::new(&enhanced_path));

            if !result.text.is_empty() {
                all_text.push_str(&result.text);
                all_text.push('\n');
            }
            _total_confidence += result.confidence;
            _box_count += result.bounding_boxes.len() as u64;
        }

        // 计算平均置信度
        let _avg_confidence = if _box_count > 0 {
            _total_confidence / _box_count as f64
        } else {
            0.0
        };

        if !all_text.is_empty() {
            Ok(all_text)
        } else {
            Err("OCR 未能识别出任何文字".into())
        }
    }

    /// Step 1: 从 PDF 提取图像
    fn extract_images(
        &self,
        pdf_path: &Path,
        output_dir: &PathBuf,
    ) -> Result<Vec<pdf_image_extract::PdfExtractedImage>, String> {
        std::fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;

        let config = pdf_image_extract::PdfImageExtractConfig::default();
        let result = pdf_image_extract::extract_pdf_images(pdf_path, output_dir, &config)
            .map_err(|e| format!("PDF 图像提取失败: {e}"))?;

        Ok(result.images)
    }

    /// Step 2: 增强图像分辨率
    fn enhance_image(
        &self,
        image_path: &str,
        output_dir: &PathBuf,
    ) -> Result<String, String> {
        let input_path = PathBuf::from(image_path);
        if !input_path.exists() {
            return Err(format!("图像文件不存在: {}", image_path));
        }

        let output_path = output_dir.join(format!("enhanced_{}", input_path.file_name().unwrap_or_default().to_string_lossy()));

        let mut resolver = ImageSuperResolver::with_config(self.super_res_config.clone());
        let result = resolver.upscale(&input_path, &output_path);

        if result.success {
            Ok(output_path.to_string_lossy().into_owned())
        } else {
            // 超分失败时回退到原始图像
            eprintln!("超分辨率增强失败: {:?}, 使用原始图像", result.error);
            Ok(image_path.to_string())
        }
    }

    /// 创建 OCR 引擎实例
    fn create_ocr_engine(&self) -> Arc<dyn OcrEngine> {
        crate::l2_perception::nt_world::ocr::create_ocr_engine(self.ocr_config.clone())
    }
}

impl Default for PdfToTextPipeline {
    fn default() -> Self {
        Self::new(OcrConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_pipeline_new() {
        let config = OcrConfig::default();
        let pipeline = PdfToTextPipeline::new(config);
        assert_eq!(pipeline.ocr_config.language, "en");
    }

    #[test]
    fn test_pipeline_run_nonexistent_pdf() {
        let pipeline = PdfToTextPipeline::new(OcrConfig::default());
        let result = pipeline.run("/nonexistent.pdf", "/tmp");
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_with_temp_dir() {
        let tmp = TempDir::new().unwrap();
        let pipeline = PdfToTextPipeline::new(OcrConfig::default());
        let result = pipeline.run("/nonexistent.pdf", tmp.path().to_str().unwrap());
        assert!(result.is_err());
    }
}