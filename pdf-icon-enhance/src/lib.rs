//! PDF Icon Enhance Module
//!
//! PDF 图标/图像增强工具，通过 Extract → Super-resolve → Embed-back 流程
//! 提升 PDF 中嵌入图像的清晰度。

pub mod pdf_image_extract;
pub mod image_super_resolution;
pub mod pdf_icon_enhance;

// Re-export main functionality
pub use pdf_icon_enhance::{enhance_pdf_icons, EnhanceConfig, EnhanceResult};
pub use pdf_image_extract::extract_images_from_pdf;
pub use image_super_resolution::upscale_image;
