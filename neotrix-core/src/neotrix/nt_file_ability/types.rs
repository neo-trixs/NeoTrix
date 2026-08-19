//! 共享类型层 — 错误、文件大类、内容快照、表格/工作表单据、编码与结构化数据。

use std::path::PathBuf;

use office_oxide::DocumentFormat;
use serde::{Deserialize, Serialize};

use crate::core::nt_core_traits::SpecialistType;

/// 统一文件能力错误
#[derive(Debug, thiserror::Error)]
pub enum FileAbilityError {
    #[error("文件无法读取: {0}")]
    Io(#[from] std::io::Error),
    #[error("office_oxide 错误: {0}")]
    Office(#[from] office_oxide::OfficeError),
    #[error("不支持的格式: {ext}")]
    UnsupportedFormat { ext: String },
    #[error("sheet 索引越界: {index} (共 {count} 个)")]
    SheetIndexOutOfRange { index: usize, count: usize },
    #[error("图像解码失败: {0}")]
    Image(image::ImageError),
    #[error("内容为空: {path}")]
    Empty { path: PathBuf },
    #[error("结构化解析失败: {0}")]
    Parse(String),
}

/// 统一结果类型
pub(crate) type Result<T> = std::result::Result<T, FileAbilityError>;

/// 文件大类 — 融合 office_oxide (Office) 与 FileParser (通用) 的探测结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileKind {
    /// Office 6 格式 (DOCX/XLSX/PPTX/DOC/XLS/PPT)
    Office(DocumentFormat),
    /// 文本/代码 (txt/md/json/xml/csv/rs/py/...)
    Text,
    /// PDF
    Pdf,
    /// 图像 (png/jpg/webp/gif/bmp/ico)
    Image,
    /// 音频 (mp3/wav/ogg/flac/m4a)
    Audio,
    /// 视频 (mp4/avi/mkv/mov/webm)
    Video,
    /// 其他二进制
    Binary,
}

impl FileKind {
    /// 该文件类对应的 GWT 关注专家 (动态路由: 按文件类型映射 SpecialistType)
    pub fn specialist(&self) -> SpecialistType {
        match self {
            FileKind::Office(_) => SpecialistType::KnowledgeIntegrator,
            FileKind::Text => SpecialistType::CodeAnalyzer,
            FileKind::Pdf => SpecialistType::KnowledgeRetriever,
            FileKind::Image => SpecialistType::ImageGenerator,
            FileKind::Audio => SpecialistType::CreativityGenerator,
            FileKind::Video => SpecialistType::CreativityGenerator,
            FileKind::Binary => SpecialistType::PatternMatcher,
        }
    }

    /// 是否为文本可提取类
    pub(super) fn is_textual(&self) -> bool {
        matches!(self, FileKind::Office(_) | FileKind::Text | FileKind::Pdf)
    }

    /// 是否为 Office 格式
    pub(super) fn office(&self) -> bool {
        matches!(self, FileKind::Office(_))
    }
}

/// 文档内容快照 (替代原先与 core 冲突的 OmniContent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSnapshot {
    pub kind: FileKind,
    /// 纯文本提取 (office/文本/PDF 场景)
    pub text: Option<String>,
    /// Markdown (office 场景)
    pub markdown: Option<String>,
    /// 表格数据 (office xlsx / csv 场景; 支持多 sheet 全量)
    pub table: Option<Vec<TableData>>,
    /// 图像元数据
    pub image: Option<ImageMetadata>,
    /// 音频/视频元数据 (真实头解析, WAV 支持时长)
    pub media: Option<MediaMetadata>,
    /// MIME 类型
    pub mime_type: String,
    /// 文件大小 (字节)
    pub size_bytes: u64,
}

/// 音频/视频元数据 (真实协议头解析)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub duration_ms: Option<u64>,
    pub sample: String,
}

/// 图像元数据 (image crate 真实提取)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub color_channels: u8,
    /// 每通道位深 (image ColorType::bits_per_pixel 真实值)
    pub bit_depth: u16,
    /// 是否有 alpha 通道
    pub has_alpha: bool,
    /// 宽高比 (宽/高, 防除零)
    pub aspect_ratio: f64,
    /// 解码后的像素通道布局 (如 "rgb8"/"rgba8"/"l8")
    pub format: String,
    pub mime_type: String,
}

/// XLSX 单元格值类型 (结构读取时的分类标签)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SheetCellValueType {
    /// 空单元格
    Empty,
    /// 数值
    Number,
    /// 文本 (含共享字符串)
    Text,
    /// 布尔
    Boolean,
    /// 错误值 (如 `#DIV/0!`)
    Error,
    /// 日期/时间
    Date,
}

/// XLSX 单元格结构化值 (列号/行号/引用 + 类型 + 显示文本 + 原始数值 + 公式)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetCellData {
    /// 1-based 列号 (A=1, B=2, ...)
    pub col: u32,
    /// 1-based 行号
    pub row: u32,
    /// 列字母 (如 "A")
    pub col_letter: String,
    /// A1 引用 (如 "C5")
    pub reference: String,
    /// 值类型
    pub value_type: SheetCellValueType,
    /// 显示文本 (office_oxide 按样式格式化后的值)
    pub text: String,
    /// 原始数值 (Number/Date 单元格有值)
    pub number: Option<f64>,
    /// 公式文本 (如 "SUM(A1:A10)"), 无公式为 None
    pub formula: Option<String>,
}

/// XLSX 工作表单行 (按 row 升序)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetRowData {
    /// 1-based 行号
    pub index: u32,
    /// 该行的单元格 (按 col 升序)
    pub cells: Vec<SheetCellData>,
}

/// XLSX 工作表结构化数据 (保留列位置/单元格类型, 供表头映射与交叉验证)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetData {
    /// 工作表名
    pub name: String,
    /// 维度字符串 (如 "A1:G50")
    pub dimension: Option<String>,
    /// 合并单元格范围 (如 "A1:C1")
    pub merged_cells: Vec<String>,
    /// 数据行 (按行号升序)
    pub rows: Vec<SheetRowData>,
}

/// 通用表格数据 — 表头 + 行数据 (行内单元格为字符串, 保留原始显示文本)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableData {
    /// 表名 (sheet 名 / CSV 文件名 stem)
    pub name: String,
    /// 表头 (列名)
    pub headers: Vec<String>,
    /// 数据行 (每行列数 = headers.len(), 缺失格以空串填充)
    pub rows: Vec<Vec<String>>,
}

impl TableData {
    /// 从 CSV/TSV 文本构造表格 (自动检测分隔符: 逗号/制表符/分号)
    pub fn from_delimited_text(name: impl Into<String>, text: &str) -> TableData {
        let mut lines = text.lines();
        // 跳过空行
        let header = lines
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .to_string();
        let delimiter = detect_delimiter(&header);
        let headers: Vec<String> = split_delimited(&header, delimiter);
        let mut rows = Vec::new();
        for line in lines {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let mut cells = split_delimited(t, delimiter);
            // 行补齐/截断到表头列数
            if cells.len() < headers.len() {
                cells.resize(headers.len(), String::new());
            } else if cells.len() > headers.len() {
                cells.truncate(headers.len());
            }
            rows.push(cells);
        }
        TableData {
            name: name.into(),
            headers,
            rows,
        }
    }

    /// 行数
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// 列数
    pub fn col_count(&self) -> usize {
        self.headers.len()
    }

    /// 按列名取值 (第 row 行, 列名匹配 header)。不存在返回 None。
    pub fn cell(&self, row: usize, header: &str) -> Option<&str> {
        let idx = self.headers.iter().position(|h| h == header)?;
        self.rows.get(row).and_then(|r| r.get(idx)).map(|s| s.as_str())
    }
}

/// 检测表格行使用的分隔符 (逗号/制表符/分号/竖线)
fn detect_delimiter(line: &str) -> char {
    for d in [',', '\t', ';', '|'] {
        if line.contains(d) {
            return d;
        }
    }
    ','
}

/// 按分隔符拆分单元格 (支持引号包裹的字段, 引号内分隔符不切分)
fn split_delimited(line: &str, delimiter: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                // 双引号转义 ("" → ")
                if in_quotes && chars.peek() == Some(&'"') {
                    chars.next();
                    cur.push('"');
                } else {
                    in_quotes = !in_quotes;
                }
            }
            c if c == delimiter && !in_quotes => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    out.push(cur.trim().to_string());
    out
}

/// 检测到的编码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    /// UTF-8 (含 BOM)
    Utf8,
    /// GBK/GB18030 (简体中文)
    Gbk,
    /// UTF-16 (BE/LE)
    Utf16,
    /// 无法判定, 回退 UTF-8 宽容解码
    Unknown,
}

/// 编码名称 (人类可读)
impl TextEncoding {
    pub fn as_str(&self) -> &'static str {
        match self {
            TextEncoding::Utf8 => "UTF-8",
            TextEncoding::Gbk => "GBK/GB18030",
            TextEncoding::Utf16 => "UTF-16",
            TextEncoding::Unknown => "unknown",
        }
    }
}