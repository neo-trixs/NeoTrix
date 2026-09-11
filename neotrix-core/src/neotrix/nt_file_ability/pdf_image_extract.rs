//! PDF 图像提取模块 — 从 PDF 中提取嵌入图像
//!
//! 吸收来源: PyMuPDF (fitz) 图像提取 API + lopdf XObject 处理
//! 公理: PDF 图像是独立 XObject 对象，通过 xref 引用，提取时保持原生分辨率无损
//!
//! 设计 (R-P42): 复用 FileKind::Pdf / FileAbilityError，不平行重造
//! 使用 lopdf 直接解析 PDF 对象树，提取嵌入的图像流

use std::path::Path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{FileAbilityError, Result};

/// PDF 图像提取配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfImageExtractConfig {
    /// 最小图像尺寸 (像素)，低于此值跳过 (过滤图标/水印)
    pub min_dimension: u32,
    /// 最小文件大小 (字节)，低于此值跳过
    pub min_bytes: usize,
    /// 是否过滤单色图像
    pub filter_unicolor: bool,
    /// 输出格式
    pub output_format: PdfImageFormat,
}

impl Default for PdfImageExtractConfig {
    fn default() -> Self {
        Self {
            min_dimension: 32,
            min_bytes: 100,
            filter_unicolor: true,
            output_format: PdfImageFormat::Png,
        }
    }
}

/// PDF 图像输出格式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PdfImageFormat {
    Png,
    Jpeg,
    Webp,
}

/// 提取的 PDF 图像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfExtractedImage {
    /// 页码 (0-based)
    pub page: usize,
    /// xref 编号
    pub xref: u32,
    /// 图像宽度
    pub width: u32,
    /// 图像高度
    pub height: u32,
    /// 颜色通道数
    pub color_channels: u8,
    /// 原始格式 (jpeg/png/tiff等)
    pub original_format: String,
    /// 输出文件路径
    pub output_path: String,
    /// 文件大小 (字节)
    pub size_bytes: usize,
    /// 是否有 alpha 通道
    pub has_alpha: bool,
}

/// PDF 图像提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfImageExtractResult {
    /// 是否成功
    pub success: bool,
    /// 提取的图像列表
    pub images: Vec<PdfExtractedImage>,
    /// 总页数
    pub total_pages: usize,
    /// 处理耗时 (毫秒)
    pub processing_time_ms: u64,
    /// 错误信息
    pub error: Option<String>,
}

/// PDF 图像统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfImageStats {
    pub total_images: usize,
    pub total_pages: usize,
    pub avg_image_size: (u32, u32),
    pub formats: Vec<String>,
}

/// 从 PDF 提取所有图像
///
/// # Arguments
/// * `pdf_path` - PDF 文件路径
/// * `output_dir` - 输出目录
/// * `config` - 提取配置
///
/// # Returns
/// 提取结果，包含所有提取的图像信息
pub fn extract_pdf_images(
    pdf_path: &Path,
    output_dir: &Path,
    config: &PdfImageExtractConfig,
) -> Result<PdfImageExtractResult> {
    let start = std::time::Instant::now();
    
    // 检查文件存在
    if !pdf_path.exists() {
        return Err(FileAbilityError::Other(format!(
            "PDF 文件不存在: {}",
            pdf_path.display()
        )));
    }
    
    // 创建输出目录
    std::fs::create_dir_all(output_dir).map_err(FileAbilityError::Io)?;
    
    // 读取 PDF 文件
    let data = std::fs::read(pdf_path).map_err(FileAbilityError::Io)?;
    
    // 使用 lopdf 解析 PDF
    let doc = lopdf::Document::load_mem(&data)
        .map_err(|e| FileAbilityError::Parse(format!("PDF 解析失败: {e}")))?;
    
    let total_pages = doc.get_pages().len();
    let mut images = Vec::new();
    let mut xref_map: HashMap<u32, bool> = HashMap::new(); // 避免重复提取
    
    // 使用通用遍历器
    let iterator = XObjectImageIterator::new(&doc);
    iterator.for_each_image(|xobj_dict, page_num, xref| {
        // 避免重复提取
        if xref_map.contains_key(&xref) {
            return Ok(true); // 继续遍历
        }
        xref_map.insert(xref, true);
        
        // 提取图像信息
        if let Some(img) = extract_image_from_xobject(
            &doc,
            xobj_dict,
            page_num,
            xref,
            config,
        ) {
            if let Ok(img_data) = img {
                // 保存图像文件
                let filename = format!(
                    "page_{}_img_{}.png",
                    page_num + 1,
                    xref
                );
                let output_path = output_dir.join(&filename);
                
                if let Err(e) = std::fs::write(&output_path, &img_data.bytes) {
                    eprintln!("保存图像失败: {e}");
                    return Ok(true); // 继续遍历
                }
                
                images.push(PdfExtractedImage {
                    page: page_num,
                    xref,
                    width: img_data.width,
                    height: img_data.height,
                    color_channels: img_data.color_channels,
                    original_format: img_data.format,
                    output_path: output_path.display().to_string(),
                    size_bytes: img_data.bytes.len(),
                    has_alpha: img_data.has_alpha,
                });
            }
        }
        Ok(true) // 继续遍历
    })?;
    
    let result = PdfImageExtractResult {
        success: true,
        images: images.clone(),
        total_pages,
        processing_time_ms: start.elapsed().as_millis() as u64,
        error: None,
    };
    
    Ok(result)
}

/// XObject 图像遍历器 — 消除重复的遍历代码
struct XObjectImageIterator<'a> {
    doc: &'a lopdf::Document,
    pages: std::collections::BTreeMap<u32, lopdf::ObjectId>,
}

impl<'a> XObjectImageIterator<'a> {
    fn new(doc: &'a lopdf::Document) -> Self {
        Self {
            doc,
            pages: doc.get_pages(),
        }
    }

    /// 遍历所有页面的图像 XObject，对每个图像执行回调
    fn for_each_image<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(&lopdf::Dictionary, usize, u32) -> Result<bool>, // 返回 false 停止遍历
    {
        for (page_num, page_id) in self.pages.iter() {
            let page_num = *page_num as usize;
            
            if let Ok(page_obj) = self.doc.get_object(*page_id) {
                if let Ok(page_dict) = page_obj.as_dict() {
                    if let Ok(resources_ref) = page_dict.get(b"Resources") {
                        if let Ok(resources) = self.doc.get_object(resources_ref.as_reference().unwrap_or((0, 0))) {
                            if let Ok(res_dict) = resources.as_dict() {
                                if let Ok(xobjects) = res_dict.get(b"XObject") {
                                    if let Ok(xobj_dict) = self.doc.get_object(xobjects.as_reference().unwrap_or((0, 0))) {
                                        if let Ok(xobj) = xobj_dict.as_dict() {
                                            for (_name, xobj_ref) in xobj.iter() {
                                                if let Ok(xobj_obj) = self.doc.get_object(xobj_ref.as_reference().unwrap_or((0, 0))) {
                                                    if let Ok(xobj_dict) = xobj_obj.as_dict() {
                                                        if let Ok(subtype) = xobj_dict.get(b"Subtype") {
                                                            if subtype.as_name().unwrap_or(b"") == b"Image" {
                                                                let xref = xobj_ref.as_reference().unwrap_or((0, 0)).0;
                                                                if !callback(&xobj_dict, page_num, xref)? {
                                                                    return Ok(());
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 获取文档引用
    fn doc(&self) -> &'a lopdf::Document {
        self.doc
    }
}

/// 从 XObject 提取图像数据
fn extract_image_from_xobject(
    doc: &lopdf::Document,
    xobj_dict: &lopdf::Dictionary,
    _page: usize,
    _xref: u32,
    config: &PdfImageExtractConfig,
) -> Option<Result<ImageData>> {
    // 获取图像尺寸
    let width = xobj_dict.get(b"Width").ok()?.as_i64().ok()? as u32;
    let height = xobj_dict.get(b"Height").ok()?.as_i64().ok()? as u32;
    
    // 过滤太小的图像
    if width < config.min_dimension || height < config.min_dimension {
        return None;
    }
    
    // 获取颜色空间
    let (color_channels, has_alpha) = get_color_info(doc, xobj_dict).unwrap_or((3, false));
    
    // 获取图像流数据
    let data = xobj_dict.get(b"Filter").ok();
    let filter = data.and_then(|f| f.as_name().ok()).unwrap_or(b"DCTDecode");
    
    // 尝试从流中提取原始图像数据
    let stream_data = extract_stream_data(doc, xobj_dict)?;
    
    // 过滤太小的图像
    if stream_data.len() < config.min_bytes {
        return None;
    }
    
    // 检查是否是单色图像 (可选过滤)
    if config.filter_unicolor && is_unicolor(&stream_data, width, height, color_channels) {
        return None;
    }
    
    // 转换格式
    let (bytes, format) = match filter {
        b"DCTDecode" => {
            // JPEG 数据，直接使用
            (stream_data, "jpeg".to_string())
        }
        b"FlateDecode" => {
            // 压缩数据，解压并转换为 PNG
            match decode_flate_data(&stream_data) {
                Ok(decoded) => {
                    match convert_to_png(&decoded, width, height, color_channels) {
                        Ok(png) => (png, "png".to_string()),
                        Err(_) => return Some(Err(FileAbilityError::Parse("图像转换失败".to_string()))),
                    }
                }
                Err(e) => return Some(Err(FileAbilityError::Parse(format!("解压失败: {e}")))),
            }
        }
        b"JPXDecode" => {
            // JPEG2000，尝试直接使用或转换
            (stream_data, "jp2".to_string())
        }
        _ => {
            // 其他格式，尝试作为原始数据处理
            (stream_data, "raw".to_string())
        }
    };
    
    Some(Ok(ImageData {
        bytes,
        width,
        height,
        color_channels,
        format,
        has_alpha,
    }))
}

/// 从文档和字典中提取流数据
fn extract_stream_data(doc: &lopdf::Document, dict: &lopdf::Dictionary) -> Option<Vec<u8>> {
    // 获取流对象
    let stream_ref = dict.get(b"Stream").ok()?;
    let stream_obj = doc.get_object(stream_ref.as_reference().unwrap_or((0, 0))).ok()?;
    
    // 尝试获取流内容
    if let Ok(stream) = stream_obj.as_stream() {
        return Some(stream.content.clone());
    }
    
    None
}

/// 获取颜色信息
fn get_color_info(_doc: &lopdf::Document, dict: &lopdf::Dictionary) -> Option<(u8, bool)> {
    let color_space = dict.get(b"ColorSpace").ok()?;
    let cs_name = color_space.as_name().ok()?;
    
    let channels = match cs_name {
        b"DeviceGray" => 1,
        b"DeviceRGB" => 3,
        b"DeviceCMYK" => 4,
        b"CalGray" => 1,
        b"CalRGB" => 3,
        b"Lab" => 3,
        _ => {
            // 尝试解析数组格式的颜色空间
            if let Ok(cs_array) = color_space.as_array() {
                if let Some(first) = cs_array.first() {
                    if let Ok(name) = first.as_name() {
                        match name {
                            b"ICCBased" => {
                                // ICC 颜色空间，检查 N 值
                                if let Some(n_val) = cs_array.get(1) {
                                    if let Ok(n) = n_val.as_i64() {
                                        n as u8
                                    } else {
                                        3
                                    }
                                } else {
                                    3
                                }
                            }
                            b"Indexed" => 1,
                            b"Separation" => 1,
                            b"DeviceN" => 4,
                            _ => 3,
                        }
                    } else {
                        3
                    }
                } else {
                    3
                }
            } else {
                3
            }
        }
    };
    
    // 检查是否有 Mask (透明度)
    let has_alpha = dict.get(b"SMask").is_ok() || dict.get(b"Mask").is_ok();
    
    Some((channels, has_alpha))
}

/// 解压 FlateDecode 数据
fn decode_flate_data(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;
    
    let mut decoder = ZlibDecoder::new(data);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded)
        .map_err(|e| FileAbilityError::Parse(format!("FlateDecode 解压失败: {e}")))?;
    
    Ok(decoded)
}

/// 将原始图像数据转换为 PNG
fn convert_to_png(
    data: &[u8],
    width: u32,
    height: u32,
    channels: u8,
) -> Result<Vec<u8>> {
    use image::{ImageBuffer, Rgba, Rgb, Luma};
    
    let expected_len = (width as usize) * (height as usize) * (channels as usize);
    if data.len() < expected_len {
        return Err(FileAbilityError::Parse(format!(
            "图像数据不足: 期望 {} 字节, 实际 {} 字节",
            expected_len,
            data.len()
        )));
    }
    
    let mut output = std::io::Cursor::new(Vec::new());
    
    match channels {
        1 => {
            // 灰度图
            let img = ImageBuffer::<Luma<u8>, _>::from_raw(width, height, &data[..expected_len])
                .ok_or_else(|| FileAbilityError::Parse("创建灰度图像失败".to_string()))?;
            img.write_to(&mut output, image::ImageFormat::Png)
                .map_err(|e| FileAbilityError::Parse(format!("PNG 编码失败: {e}")))?;
        }
        3 => {
            // RGB
            let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, &data[..expected_len])
                .ok_or_else(|| FileAbilityError::Parse("创建 RGB 图像失败".to_string()))?;
            img.write_to(&mut output, image::ImageFormat::Png)
                .map_err(|e| FileAbilityError::Parse(format!("PNG 编码失败: {e}")))?;
        }
        4 => {
            // RGBA
            let img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, &data[..expected_len])
                .ok_or_else(|| FileAbilityError::Parse("创建 RGBA 图像失败".to_string()))?;
            img.write_to(&mut output, image::ImageFormat::Png)
                .map_err(|e| FileAbilityError::Parse(format!("PNG 编码失败: {e}")))?;
        }
        _ => {
            return Err(FileAbilityError::Parse(format!(
                "不支持的颜色通道数: {channels}"
            )));
        }
    }
    
    Ok(output.into_inner())
}

/// 检查是否是单色图像
fn is_unicolor(data: &[u8], width: u32, height: u32, channels: u8) -> bool {
    if channels == 0 || width == 0 || height == 0 {
        return true;
    }
    
    let pixel_count = (width as usize) * (height as usize);
    if data.len() < pixel_count * (channels as usize) {
        return true;
    }
    
    // 采样检查 (每 100 像素检查一个)
    let sample_step = 100.max(1);
    let first_pixel = &data[..channels as usize];
    
    for i in (0..pixel_count).step_by(sample_step) {
        let offset = i * channels as usize;
        if offset + channels as usize > data.len() {
            break;
        }
        let pixel = &data[offset..offset + channels as usize];
        if pixel != first_pixel {
            return false;
        }
    }
    
    true
}

/// 从 PDF 单页提取图像
pub fn extract_page_images(
    pdf_path: &Path,
    _page: usize,
    output_dir: &Path,
    config: &PdfImageExtractConfig,
) -> Result<Vec<PdfExtractedImage>> {
    let result = extract_pdf_images(pdf_path, output_dir, config)?;
    Ok(result.images.into_iter().filter(|img| img.page == page).collect())
}

/// 检测 PDF 是否包含图像
pub fn pdf_has_images(pdf_path: &Path) -> Result<bool> {
    let data = std::fs::read(pdf_path).map_err(FileAbilityError::Io)?;
    let doc = lopdf::Document::load_mem(&data)
        .map_err(|e| FileAbilityError::Parse(format!("PDF 解析失败: {e}")))?;
    
    let iterator = XObjectImageIterator::new(&doc);
    let mut found = false;
    
    iterator.for_each_image(|_xobj_dict, _page_num, _xref| {
        found = true;
        Ok(false) // 找到即停止
    })?;
    
    Ok(found)
}

/// 获取 PDF 图像统计信息
pub fn pdf_image_stats(pdf_path: &Path) -> Result<PdfImageStats> {
    let data = std::fs::read(pdf_path).map_err(FileAbilityError::Io)?;
    let doc = lopdf::Document::load_mem(&data)
        .map_err(|e| FileAbilityError::Parse(format!("PDF 解析失败: {e}")))?;
    
    let total_pages = doc.get_pages().len();
    let mut total_images = 0u64;
    let mut formats = Vec::new();
    let mut total_width = 0u64;
    let mut total_height = 0u64;
    let mut image_count = 0u64;
    
    let iterator = XObjectImageIterator::new(&doc);
    iterator.for_each_image(|xobj_dict, _page_num, _xref| {
        total_images += 1;
        
        if let Ok(width) = xobj_dict.get(b"Width") {
            if let Ok(w) = width.as_i64() {
                total_width += w as u64;
            }
        }
        if let Ok(height) = xobj_dict.get(b"Height") {
            if let Ok(h) = height.as_i64() {
                total_height += h as u64;
            }
        }
        image_count += 1;
        
        // 检查格式
        if let Ok(filter) = xobj_dict.get(b"Filter") {
            if let Ok(name) = filter.as_name() {
                let format = match name {
                    b"DCTDecode" => "jpeg",
                    b"JPXDecode" => "jp2",
                    b"CCITTFaxDecode" => "tiff",
                    _ => "other",
                };
                if !formats.contains(&format.to_string()) {
                    formats.push(format.to_string());
                }
            }
        }
        Ok(true) // 继续遍历
    })?;
    
    let avg_image_size = if image_count > 0 {
        ((total_width / image_count) as u32, (total_height / image_count) as u32)
    } else {
        (0, 0)
    };
    
    Ok(PdfImageStats {
        total_images: total_images as usize,
        total_pages,
        avg_image_size,
        formats,
    })
}

/// 内部图像数据结构
struct ImageData {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    color_channels: u8,
    format: String,
    has_alpha: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_default_config() {
        let config = PdfImageExtractConfig::default();
        assert_eq!(config.min_dimension, 32);
        assert_eq!(config.min_bytes, 100);
        assert!(config.filter_unicolor);
        assert_eq!(config.output_format, PdfImageFormat::Png);
    }
    
    #[test]
    fn test_extract_nonexistent_pdf() {
        let tmp = TempDir::new().unwrap();
        let result = extract_pdf_images(
            Path::new("/nonexistent.pdf"),
            tmp.path(),
            &PdfImageExtractConfig::default(),
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_pdf_has_images_nonexistent() {
        let result = pdf_has_images(Path::new("/nonexistent.pdf"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_pdf_image_stats_nonexistent() {
        let result = pdf_image_stats(Path::new("/nonexistent.pdf"));
        assert!(result.is_err());
    }
}
