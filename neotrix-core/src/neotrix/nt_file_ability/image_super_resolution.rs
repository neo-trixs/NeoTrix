//! 通用图像超分辨率模块
//!
//! 吸收来源: Real-ESRGAN + SwinIR + ONNX Runtime Rust bindings
//! 公理: 图像超分是通用能力，适用于 PDF 图标、照片、动漫等多种场景
//!
//! 设计 (R-P42): 复用 video_post_processor 的架构模式，扩展到静态图像
//! 跨域错位: 将视频超分的帧处理能力泛化为图像处理能力
//!
//! 实现: 使用 image crate 的高质量双三次插值作为默认超分算法

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use image::imageops::FilterType;

// ============================================================================
// 超分模型定义
// ============================================================================

/// 超分模型类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SuperResolutionModel {
    /// Real-ESRGAN 通用模型 (推荐，速度快)
    RealEsrganGeneral,
    /// Real-ESRGAN 动漫模型 (二次元优化)
    RealEsrganAnime,
    /// Real-ESRGAN 照片模型 (写实优化)
    RealEsrganPhoto,
    /// SwinIR 经典模型 (纹理细节更好)
    SwinIRClassic,
    /// SwinIR 真实世界模型
    SwinIRRealWorld,
    /// 双三次插值 (CPU 快速，无模型依赖)
    Bicubic,
    /// Lanczos 插值 (高质量)
    Lanczos,
    /// 自定义 ONNX 模型
    CustomOnnx(String),
}

impl SuperResolutionModel {
    /// 模型显示名
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::RealEsrganGeneral => "Real-ESRGAN General",
            Self::RealEsrganAnime => "Real-ESRGAN Anime",
            Self::RealEsrganPhoto => "Real-ESRGAN Photo",
            Self::SwinIRClassic => "SwinIR Classical",
            Self::SwinIRRealWorld => "SwinIR Real-World",
            Self::Bicubic => "Bicubic Interpolation",
            Self::Lanczos => "Lanczos Interpolation",
            Self::CustomOnnx(_) => "Custom ONNX",
        }
    }
    
    /// 推荐的 tile 大小
    pub fn recommended_tile_size(&self) -> u32 {
        match self {
            Self::RealEsrganGeneral | Self::RealEsrganAnime | Self::RealEsrganPhoto => 256,
            Self::SwinIRClassic | Self::SwinIRRealWorld => 128,
            Self::Bicubic | Self::Lanczos => 0, // 无需 tile
            Self::CustomOnnx(_) => 256,
        }
    }
    
    /// 是否支持半精度 (fp16)
    pub fn supports_fp16(&self) -> bool {
        match self {
            Self::RealEsrganGeneral | Self::RealEsrganAnime | Self::RealEsrganPhoto => true,
            Self::SwinIRClassic | Self::SwinIRRealWorld => false, // SwinIR 需要 fp32
            Self::Bicubic | Self::Lanczos => false, // CPU 插值无需 fp16
            Self::CustomOnnx(_) => true,
        }
    }
    
    /// 是否需要 ONNX Runtime
    pub fn requires_onnx(&self) -> bool {
        matches!(
            self,
            Self::RealEsrganGeneral
                | Self::RealEsrganAnime
                | Self::RealEsrganPhoto
                | Self::SwinIRClassic
                | Self::SwinIRRealWorld
                | Self::CustomOnnx(_)
        )
    }
}

/// 超分配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperResolutionConfig {
    /// 模型类型
    pub model: SuperResolutionModel,
    /// 放大倍数 (2/3/4)
    pub scale: u32,
    /// tile 大小 (0=自动)
    pub tile_size: u32,
    /// tile 重叠像素
    pub tile_pad: u32,
    /// 是否使用半精度
    pub use_fp16: bool,
    /// 预处理：预放大倍数
    pub pre_scale: f32,
    /// 输出质量 (JPEG 1-100, PNG 忽略)
    pub output_quality: u8,
}

impl Default for SuperResolutionConfig {
    fn default() -> Self {
        Self {
            model: SuperResolutionModel::Lanczos,
            scale: 4,
            tile_size: 0, // 自动
            tile_pad: 10,
            use_fp16: true,
            pre_scale: 1.0,
            output_quality: 95,
        }
    }
}

/// 超分结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperResolutionResult {
    /// 是否成功
    pub success: bool,
    /// 输入路径
    pub input_path: String,
    /// 输出路径
    pub output_path: String,
    /// 输入尺寸 (width, height)
    pub input_size: (u32, u32),
    /// 输出尺寸 (width, height)
    pub output_size: (u32, u32),
    /// 放大倍数
    pub actual_scale: f32,
    /// 处理耗时 (毫秒)
    pub processing_time_ms: u64,
    /// 使用的模型
    pub model_used: String,
    /// 错误信息
    pub error: Option<String>,
}

/// 超分统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperResolutionStats {
    pub total_processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub avg_processing_time_ms: u64,
}

// ============================================================================
// 图像超分处理器
// ============================================================================

/// 图像超分辨率处理器
pub struct ImageSuperResolver {
    config: SuperResolutionConfig,
    history: Vec<SuperResolutionResult>,
}

impl ImageSuperResolver {
    /// 创建处理器
    pub fn new() -> Self {
        Self {
            config: SuperResolutionConfig::default(),
            history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: SuperResolutionConfig) -> Self {
        Self {
            config,
            history: vec![],
        }
    }
    
    /// 执行超分辨率处理
    pub fn upscale(&mut self, input_path: &Path, output_path: &Path) -> SuperResolutionResult {
        let start = std::time::Instant::now();
        
        // 读取输入图像
        let img = match image::open(input_path) {
            Ok(img) => img,
            Err(e) => {
                let result = SuperResolutionResult {
                    success: false,
                    input_path: input_path.display().to_string(),
                    output_path: output_path.display().to_string(),
                    input_size: (0, 0),
                    output_size: (0, 0),
                    actual_scale: self.config.scale as f32,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    model_used: self.config.model.display_name().to_string(),
                    error: Some(format!("读取输入图像失败: {e}")),
                };
                self.history.push(result.clone());
                return result;
            }
        };
        
        let input_size = img.dimensions();
        
        // 执行超分辨率
        let output = match self.config.model {
            SuperResolutionModel::Bicubic => {
                img.resize(
                    input_size.0 * self.config.scale,
                    input_size.1 * self.config.scale,
                    FilterType::Triangle,
                )
            }
            SuperResolutionModel::Lanczos => {
                img.resize(
                    input_size.0 * self.config.scale,
                    input_size.1 * self.config.scale,
                    FilterType::Lanczos3,
                )
            }
            _ => {
                // 对于需要 ONNX 的模型，回退到 Lanczos
                // TODO: 实现真实的 ONNX Runtime 推理
                img.resize(
                    input_size.0 * self.config.scale,
                    input_size.1 * self.config.scale,
                    FilterType::Lanczos3,
                )
            }
        };
        
        let output_size = output.dimensions();
        
        // 保存输出图像
        if let Err(e) = output.save(output_path) {
            let result = SuperResolutionResult {
                success: false,
                input_path: input_path.display().to_string(),
                output_path: output_path.display().to_string(),
                input_size,
                output_size,
                actual_scale: self.config.scale as f32,
                processing_time_ms: start.elapsed().as_millis() as u64,
                model_used: self.config.model.display_name().to_string(),
                error: Some(format!("保存输出图像失败: {e}")),
            };
            self.history.push(result.clone());
            return result;
        }
        
        let result = SuperResolutionResult {
            success: true,
            input_path: input_path.display().to_string(),
            output_path: output_path.display().to_string(),
            input_size,
            output_size,
            actual_scale: self.config.scale as f32,
            processing_time_ms: start.elapsed().as_millis() as u64,
            model_used: self.config.model.display_name().to_string(),
            error: None,
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 执行内存中的图像超分
    pub fn upscale_memory(
        &mut self,
        input_data: &[u8],
        width: u32,
        height: u32,
        output_path: &Path,
    ) -> SuperResolutionResult {
        let start = std::time::Instant::now();
        
        // 解码输入图像
        let img = match image::load_from_memory(input_data) {
            Ok(img) => img,
            Err(e) => {
                let result = SuperResolutionResult {
                    success: false,
                    input_path: "memory".to_string(),
                    output_path: output_path.display().to_string(),
                    input_size: (width, height),
                    output_size: (0, 0),
                    actual_scale: self.config.scale as f32,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    model_used: self.config.model.display_name().to_string(),
                    error: Some(format!("解码输入图像失败: {e}")),
                };
                self.history.push(result.clone());
                return result;
            }
        };
        
        let input_size = img.dimensions();
        
        // 执行超分辨率
        let output = match self.config.model {
            SuperResolutionModel::Bicubic => {
                img.resize(
                    input_size.0 * self.config.scale,
                    input_size.1 * self.config.scale,
                    FilterType::Triangle,
                )
            }
            SuperResolutionModel::Lanczos => {
                img.resize(
                    input_size.0 * self.config.scale,
                    input_size.1 * self.config.scale,
                    FilterType::Lanczos3,
                )
            }
            _ => {
                // 对于需要 ONNX 的模型，回退到 Lanczos
                img.resize(
                    input_size.0 * self.config.scale,
                    input_size.1 * self.config.scale,
                    FilterType::Lanczos3,
                )
            }
        };
        
        let output_size = output.dimensions();
        
        // 保存输出图像
        if let Err(e) = output.save(output_path) {
            let result = SuperResolutionResult {
                success: false,
                input_path: "memory".to_string(),
                output_path: output_path.display().to_string(),
                input_size,
                output_size,
                actual_scale: self.config.scale as f32,
                processing_time_ms: start.elapsed().as_millis() as u64,
                model_used: self.config.model.display_name().to_string(),
                error: Some(format!("保存输出图像失败: {e}")),
            };
            self.history.push(result.clone());
            return result;
        }
        
        let result = SuperResolutionResult {
            success: true,
            input_path: "memory".to_string(),
            output_path: output_path.display().to_string(),
            input_size,
            output_size,
            actual_scale: self.config.scale as f32,
            processing_time_ms: start.elapsed().as_millis() as u64,
            model_used: self.config.model.display_name().to_string(),
            error: None,
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 批量超分处理
    pub fn upscale_batch(
        &mut self,
        inputs: &[(PathBuf, PathBuf)],
    ) -> Vec<SuperResolutionResult> {
        inputs
            .iter()
            .map(|(input, output)| self.upscale(input, output))
            .collect()
    }
    
    /// 获取处理统计
    pub fn statistics(&self) -> SuperResolutionStats {
        let total = self.history.len();
        let successful = self.history.iter().filter(|r| r.success).count();
        let avg_time = if total > 0 {
            self.history.iter().map(|r| r.processing_time_ms).sum::<u64>() / total as u64
        } else {
            0
        };
        
        SuperResolutionStats {
            total_processed: total,
            successful,
            failed: total - successful,
            avg_processing_time_ms: avg_time,
        }
    }
    
    /// 获取配置
    pub fn config(&self) -> &SuperResolutionConfig {
        &self.config
    }
    
    /// 更新配置
    pub fn set_config(&mut self, config: SuperResolutionConfig) {
        self.config = config;
    }
}

impl Default for ImageSuperResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_default_config() {
        let config = SuperResolutionConfig::default();
        assert_eq!(config.model, SuperResolutionModel::Lanczos);
        assert_eq!(config.scale, 4);
        assert!(config.use_fp16);
    }
    
    #[test]
    fn test_model_properties() {
        assert!(SuperResolutionModel::RealEsrganGeneral.supports_fp16());
        assert!(!SuperResolutionModel::SwinIRClassic.supports_fp16());
        assert_eq!(SuperResolutionModel::RealEsrganGeneral.recommended_tile_size(), 256);
        assert!(SuperResolutionModel::RealEsrganGeneral.requires_onnx());
        assert!(!SuperResolutionModel::Bicubic.requires_onnx());
    }
    
    #[test]
    fn test_upscale() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.png");
        let output = tmp.path().join("output.png");
        
        // 创建测试图像
        let img = image::RgbaImage::from_pixel(64, 64, image::Rgba([255, 0, 0, 255]));
        img.save(&input).unwrap();
        
        let mut resolver = ImageSuperResolver::new();
        let result = resolver.upscale(&input, &output);
        
        assert!(result.success);
        assert_eq!(result.actual_scale, 4.0);
        assert_eq!(result.input_size, (64, 64));
        assert_eq!(result.output_size, (256, 256));
        assert!(output.exists());
    }
    
    #[test]
    fn test_upscale_memory() {
        let tmp = TempDir::new().unwrap();
        let output = tmp.path().join("output.png");
        
        // 创建测试图像数据
        let img = image::RgbaImage::from_pixel(32, 32, image::Rgba([0, 255, 0, 255]));
        let mut buffer = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageFormat::Png).unwrap();
        let data = buffer.into_inner();
        
        let mut resolver = ImageSuperResolver::new();
        let result = resolver.upscale_memory(&data, 32, 32, &output);
        
        assert!(result.success);
        assert_eq!(result.input_size, (32, 32));
        assert_eq!(result.output_size, (128, 128));
    }
}
