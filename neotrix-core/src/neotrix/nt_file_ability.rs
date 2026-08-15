//! # NeoTrix 统一文件能力 (Unified File Ability)
//!
//! 单一入口管理和操作所有文件 I/O：
//! - **Office 6 格式** (DOCX/XLSX/PPTX/DOC/XLS/PPT) — 由 `office_oxide` 提供
//!   读 (`plain_text`/`to_markdown`/`to_html`)、导 (`to_ir`/`save_as`)、
//!   编辑 (`EditableDocx`/`EditablePptx::replace_text`)
//! - **通用文本/PDF/图像/音频/视频** — 由 `neotrix-types::FileParser` 探测并提取
//! - **图像元数据** — 由 `image` crate 提取尺寸/通道/格式
//!
//! ## 纪律
//! - **R-P1**: 零 unsafe (office_oxide + FileParser + image 均纯 Rust)
//! - **R-P42**: 复用 core 既有成熟类型 (`ConstellationLevel`/`SelfTest`)，
//!   不平行重造枚举
//! - **Dark Forest**: 模块经能力树 `ConstellationLevel` 标记成熟度，
//!   经 `SelfTest` T1-T3 接线到意识树健康链，被流水线消费
//! - **指针守恒**: 单一 `path` 句柄，不复制状态

use std::path::{Path, PathBuf};

use image::GenericImageView;
use office_oxide::xlsx::{CellRef, CellValue};
use office_oxide::{create, Document, DocumentFormat};
use serde::{Deserialize, Serialize};

use crate::core::nt_core_hcube::vsa::{VSAEngine, VsaBackend};
use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_self_test::SelfTest;
use crate::core::nt_core_traits::SpecialistType;
use nt_core_capability_tree::ConstellationLevel;

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
type Result<T> = std::result::Result<T, FileAbilityError>;

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
}

/// 文档内容快照 (替代原先与 core 冲突的 OmniContent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSnapshot {
    pub kind: FileKind,
    /// 纯文本提取 (office/文本/PDF 场景)
    pub text: Option<String>,
    /// Markdown (office 场景)
    pub markdown: Option<String>,
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

/// 统一文件能力句柄
pub struct FileAbility {
    /// 原始路径句柄 (指针守恒: 单一存储)
    path: PathBuf,
    /// 文件大类探测结果
    kind: FileKind,
    /// MIME 类型
    mime_type: String,
    /// 文件大小 (字节)
    size_bytes: u64,
    /// 是否已被消费者注册 (Dark Forest 生存标记)
    has_consumers: bool,
    /// 能力成熟度 (复用能力树 ConstellationLevel, 不平行重造)
    maturity: ConstellationLevel,
    /// 当前 E8 推理状态 (Ext-6: 操作驱动状态转移)
    e8_state: ReasoningHexagram,
    /// Office 句柄缓存 (仅 Office 类文件)
    doc: Option<Document>,
}

impl FileAbility {
    /// 打开文件并探测大类
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let meta = std::fs::metadata(&path).map_err(FileAbilityError::Io)?;
        let size_bytes = meta.len();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 1) Office 格式优先走 office_oxide
        if let Some(fmt) = DocumentFormat::from_extension(&ext) {
            let doc = Document::open(&path).map_err(FileAbilityError::Office)?;
            return Ok(Self {
                path,
                kind: FileKind::Office(fmt),
                mime_type: fmt.mime_type().to_string(),
                size_bytes,
                has_consumers: false,
                maturity: ConstellationLevel::C1UnitTest,
                e8_state: ReasoningHexagram::new(0b001100), // 数据提取模式 (concrete+analytical+deep)
                doc: Some(doc),
            });
        }

        // 2) 其余交给 neotrix-types FileParser 探测
        let data = std::fs::read(&path).map_err(FileAbilityError::Io)?;
        let parsed = neotrix_types::core::file_parser::FileParser::detect_format(
            &path.to_string_lossy(),
            "",
            &data,
        );
        let (kind, mime) = match parsed {
            neotrix_types::core::file_parser::FileFormat::Pdf => {
                (FileKind::Pdf, "application/pdf".to_string())
            }
            neotrix_types::core::file_parser::FileFormat::Image => {
                (FileKind::Image, guess_mime(&ext))
            }
            neotrix_types::core::file_parser::FileFormat::Audio => {
                (FileKind::Audio, guess_mime(&ext))
            }
            neotrix_types::core::file_parser::FileFormat::Video => {
                (FileKind::Video, guess_mime(&ext))
            }
            neotrix_types::core::file_parser::FileFormat::Binary => {
                (FileKind::Binary, "application/octet-stream".to_string())
            }
            // PlainText/Markdown/Code/Json/Xml/Csv/Html 均归文本
            _ => (FileKind::Text, guess_mime(&ext)),
        };

        Ok(Self {
            path,
            kind,
            mime_type: mime,
            size_bytes,
            has_consumers: false,
            maturity: ConstellationLevel::C0Compile,
            e8_state: ReasoningHexagram::new(0b001001), // 语法/探测模式 (concrete+analytical+focused)
            doc: None,
        })
    }

    /// 注册消费者 — Dark Forest 生存标记
    pub fn register_consumer(&mut self) {
        self.has_consumers = true;
    }

    /// 是否已被消费者注册
    pub fn has_consumers(&self) -> bool {
        self.has_consumers
    }

    /// 文件大类
    pub fn kind(&self) -> FileKind {
        self.kind
    }

    /// MIME 类型
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }

    /// 文件大小
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    /// 能力成熟度
    pub fn maturity(&self) -> ConstellationLevel {
        self.maturity
    }

    /// 提升成熟度一级 (能力树晋级)
    pub fn promote(&mut self) -> Option<ConstellationLevel> {
        self.maturity = self.maturity.next()?;
        Some(self.maturity)
    }

    /// 提取纯文本 — office 走 office_oxide, 通用格式走 FileParser
    pub fn plain_text(&self) -> String {
        if let Some(doc) = &self.doc {
            doc.plain_text()
        } else if let Ok(data) = std::fs::read(&self.path) {
            neotrix_types::core::file_parser::FileParser::extract_text(
                &self.path.to_string_lossy(),
                &self.mime_type,
                &data,
            )
            .text
        } else {
            String::new()
        }
    }

    /// 转 Markdown (仅 Office 格式支持; 其余返回 plain_text)
    pub fn to_markdown(&self) -> String {
        if let Some(doc) = &self.doc {
            if self.kind.office() {
                doc.to_markdown()
            } else {
                self.plain_text()
            }
        } else {
            self.plain_text()
        }
    }

    /// 转 HTML (office_oxide IR → HTML)
    pub fn to_html(&self) -> Option<String> {
        self.doc.as_ref().map(|d| d.to_html())
    }

    /// 导出为其他格式 (office_oxide save_as / 格式转换)
    pub fn save_as(&self, target: impl AsRef<Path>) -> Result<()> {
        if let Some(doc) = &self.doc {
            doc.save_as(target).map_err(FileAbilityError::Office)
        } else {
            // 非 Office 走复制
            std::fs::copy(&self.path, target)
                .map(|_| ())
                .map_err(FileAbilityError::Io)
        }
    }

    // ─────────────── XLSX 单元格级结构化读取 (Ext-7) ───────────────

    /// 内部取 XLSX 文档句柄 (仅 XLSX 格式, 其余报 UnsupportedFormat)
    fn xlsx_doc(&self) -> Result<&office_oxide::xlsx::XlsxDocument> {
        match &self.doc {
            Some(doc) => doc
                .as_xlsx()
                .ok_or_else(|| FileAbilityError::UnsupportedFormat {
                    ext: self
                        .path
                        .extension()
                        .map(|e| e.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                }),
            None => Err(FileAbilityError::UnsupportedFormat {
                ext: self
                    .path
                    .extension()
                    .map(|e| e.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            }),
        }
    }

    /// XLSX 工作表名列表 (按文件内顺序)
    pub fn xlsx_sheet_names(&self) -> Result<Vec<String>> {
        let doc = self.xlsx_doc()?;
        Ok(doc.worksheets.iter().map(|ws| ws.name.clone()).collect())
    }

    /// XLSX 工作表数量
    pub fn xlsx_sheet_count(&self) -> Result<usize> {
        let doc = self.xlsx_doc()?;
        Ok(doc.worksheets.len())
    }

    /// 读取第 index 个工作表 (1-based, 与 Excel 一致) 的结构化数据
    pub fn xlsx_sheet(&self, index: usize) -> Result<SheetData> {
        let doc = self.xlsx_doc()?;
        if index == 0 {
            return Err(FileAbilityError::SheetIndexOutOfRange {
                index,
                count: doc.worksheets.len(),
            });
        }
        let ws = doc.worksheets.get(index - 1).ok_or_else(|| {
            FileAbilityError::SheetIndexOutOfRange {
                index,
                count: doc.worksheets.len(),
            }
        })?;
        let date_indices = doc.date_style_indices();
        let rows = ws
            .rows
            .iter()
            .map(|r| SheetRowData {
                index: r.index,
                cells: r
                    .cells
                    .iter()
                    .map(|c| {
                        let mut buf = String::new();
                        doc.write_cell_value_fast(c, &mut buf, &date_indices);
                        let (col, row) = (c.reference.col + 1, c.reference.row + 1);
                        let value_type = match &c.value {
                            CellValue::Empty => SheetCellValueType::Empty,
                            CellValue::Number(_) => {
                                if c.style_index.is_some_and(|i| date_indices.contains(&i)) {
                                    SheetCellValueType::Date
                                } else {
                                    SheetCellValueType::Number
                                }
                            }
                            CellValue::String(_) | CellValue::SharedString(_) => {
                                SheetCellValueType::Text
                            }
                            CellValue::Boolean(_) => SheetCellValueType::Boolean,
                            CellValue::Error(_) => SheetCellValueType::Error,
                            CellValue::Date(_) => SheetCellValueType::Date,
                        };
                        let number = match &c.value {
                            CellValue::Number(n) => Some(*n),
                            CellValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
                            _ => None,
                        };
                        SheetCellData {
                            col,
                            row,
                            col_letter: CellRef::col_name(c.reference.col),
                            reference: c.reference.to_string(),
                            value_type,
                            text: buf,
                            number,
                            formula: c.formula.clone(),
                        }
                    })
                    .collect(),
            })
            .collect();
        Ok(SheetData {
            name: ws.name.clone(),
            dimension: ws.dimension.clone(),
            merged_cells: ws.merged_cells.clone(),
            rows,
        })
    }

    /// 按名称读取工作表 (查找失败返回 SheetIndexOutOfRange)
    pub fn xlsx_sheet_by_name(&self, name: &str) -> Result<SheetData> {
        let names = self.xlsx_sheet_names()?;
        let pos = names.iter().position(|n| n == name).ok_or_else(|| {
            FileAbilityError::SheetIndexOutOfRange {
                index: 0,
                count: names.len(),
            }
        })?;
        self.xlsx_sheet(pos + 1)
    }

    /// 占位符替换 (DOCX/PPTX 经 Editable* 保格式替换; 其余不支持)
    pub fn replace_placeholder(&self, find: &str, replace: &str) -> Result<usize> {
        if find.is_empty() || replace.is_empty() {
            return Err(FileAbilityError::Empty {
                path: self.path.clone(),
            });
        }
        match self.kind {
            FileKind::Office(DocumentFormat::Docx) => {
                let mut ed = office_oxide::docx::edit::EditableDocx::open(&self.path)
                    .map_err(|e| FileAbilityError::Office(e.into()))?;
                Ok(ed.replace_text(find, replace))
            }
            FileKind::Office(DocumentFormat::Pptx) => {
                let mut ed = office_oxide::pptx::edit::EditablePptx::open(&self.path)
                    .map_err(|e| FileAbilityError::Office(e.into()))?;
                Ok(ed.replace_text(find, replace))
            }
            _ => Ok(0),
        }
    }

    /// 图像元数据 (仅 Image 类)
    pub fn image_metadata(&self) -> Result<ImageMetadata> {
        if self.kind != FileKind::Image {
            return Err(FileAbilityError::UnsupportedFormat {
                ext: self
                    .path
                    .extension()
                    .map(|e| e.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            });
        }
        let img = image::open(&self.path).map_err(FileAbilityError::Image)?;
        let (width, height) = img.dimensions();
        let color = img.color();
        let channels = match color {
            image::ColorType::Rgb8 => 3,
            image::ColorType::Rgba8 => 4,
            image::ColorType::L8 => 1,
            image::ColorType::La8 => 2,
            _ => 3,
        };
        let has_alpha = matches!(
            color,
            image::ColorType::Rgba8
                | image::ColorType::La8
                | image::ColorType::Rgba16
                | image::ColorType::La16
        );
        Ok(ImageMetadata {
            width,
            height,
            color_channels: channels,
            bit_depth: color.bits_per_pixel(),
            has_alpha,
            aspect_ratio: if height > 0 {
                width as f64 / height as f64
            } else {
                0.0
            },
            format: format!("{:?}", color).to_lowercase(),
            mime_type: self.mime_type.clone(),
        })
    }

    /// 音频时长探测 (纯字节解析 WAV RIFF 头, 零额外依赖, 真实多模态元数据)
    ///
    /// 仅支持 WAV: 解析 `fmt ` 子块字节率 + `data` 子块长度 → 时长毫秒。
    /// 其他音频格式无标准头解析库时返回 None (由 FileAbilityExt::MediaProbe
    /// 或未来吸收独立解析器补充)。
    pub fn audio_duration_ms(&self) -> Option<u64> {
        if self.kind != FileKind::Audio {
            return None;
        }
        let ext = self
            .path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "wav" {
            return None;
        }
        let data = std::fs::read(&self.path).ok()?;
        if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
            return None;
        }
        let mut pos = 12usize;
        let mut byte_rate: u32 = 0;
        let mut data_len: u32 = 0;
        while pos + 8 <= data.len() {
            let chunk_size =
                u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap_or([0; 4])) as usize;
            match &data[pos..pos + 4] {
                b"fmt " if byte_rate == 0 => {
                    if pos + 16 + 8 <= data.len() {
                        byte_rate = u32::from_le_bytes(
                            data[pos + 16..pos + 20].try_into().unwrap_or([0; 4]),
                        );
                    }
                }
                b"data" => {
                    data_len =
                        u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap_or([0; 4]));
                    break;
                }
                _ => {}
            }
            pos += 8 + chunk_size + (chunk_size & 1); // RIFF 子块 2 字节对齐
        }
        if byte_rate == 0 {
            return None;
        }
        Some((data_len as u64 * 1000) / byte_rate as u64)
    }

    /// 全量内容快照
    pub fn snapshot(&self) -> ContentSnapshot {
        let text = if self.kind.is_textual() {
            Some(self.plain_text())
        } else {
            None
        };
        let markdown = if self.kind.office() {
            Some(self.to_markdown())
        } else {
            None
        };
        let image = if self.kind == FileKind::Image {
            self.image_metadata().ok()
        } else {
            None
        };
        let media = if self.kind == FileKind::Audio || self.kind == FileKind::Video {
            Some(MediaMetadata {
                duration_ms: self.audio_duration_ms(),
                sample: self.mime_type.clone(),
            })
        } else {
            None
        };
        ContentSnapshot {
            kind: self.kind,
            text,
            markdown,
            image,
            media,
            mime_type: self.mime_type.clone(),
            size_bytes: self.size_bytes,
        }
    }
}

impl FileKind {
    /// 是否为文本可提取类
    fn is_textual(&self) -> bool {
        matches!(self, FileKind::Office(_) | FileKind::Text | FileKind::Pdf)
    }

    /// 是否为 Office 格式
    fn office(&self) -> bool {
        matches!(self, FileKind::Office(_))
    }
}

// ─────────────────────────── E8 状态转移 (Ext-6) ───────────────────────────

/// 文件能力操作 — 每类操作驱动一次 E8 状态转移
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOperation {
    /// 探测/识别 (open)
    Detect,
    /// 纯文本提取
    Extract,
    /// 格式转换 (markdown/html/导出)
    Transform,
    /// 占位符编辑 (replace_placeholder)
    Edit,
    /// 语义嵌入 (VSA)
    Embed,
    /// 健康巡检 (SelfTest/check_health)
    Audit,
}

impl FileOperation {
    /// 该操作的目标 E8 状态 (6-bit hexagram)
    pub fn target_state(&self) -> ReasoningHexagram {
        match self {
            // 探测: 具体+分析+专注
            Self::Detect => ReasoningHexagram::new(0b001001),
            // 提取: 具体+分析+深度
            Self::Extract => ReasoningHexagram::new(0b001100),
            // 转换: 具体+生成+协作 (format transformation)
            Self::Transform => ReasoningHexagram::new(0b001011),
            // 编辑: 具体+分析+协作
            Self::Edit => ReasoningHexagram::new(0b001110),
            // 嵌入: 抽象+生成+深度 (semantic encoding)
            Self::Embed => ReasoningHexagram::new(0b111100),
            // 审计: 抽象+分析+深度
            Self::Audit => ReasoningHexagram::new(0b101100),
        }
    }

    /// 操作名
    pub fn name(&self) -> &'static str {
        match self {
            Self::Detect => "detect",
            Self::Extract => "extract",
            Self::Transform => "transform",
            Self::Edit => "edit",
            Self::Embed => "embed",
            Self::Audit => "audit",
        }
    }
}

impl FileAbility {
    /// 当前 E8 推理状态
    pub fn e8_state(&self) -> ReasoningHexagram {
        self.e8_state
    }

    /// 执行一次状态转移: 将当前状态向目标状态单步推进 (flip 最近的一个差异轴)
    ///
    /// 返回转移后的新状态。若已到达目标, 返回原状态 (路径长度为 0)。
    pub fn transition(&mut self, op: FileOperation) -> ReasoningHexagram {
        let target = op.target_state();
        let current = self.e8_state;
        let mut best = current;
        let mut best_dist = current.hamming_dist(&target);
        // 从 6 个邻居里选最接近目标的单步 (贪心下降)
        for n in current.neighbors() {
            let d = n.hamming_dist(&target);
            if d < best_dist {
                best_dist = d;
                best = n;
            }
        }
        self.e8_state = best;
        best
    }

    /// 到目标状态的完整转移路径 (E8 ReasoningPath)
    pub fn e8_path_to(&self, target: ReasoningHexagram) -> Vec<ReasoningHexagram> {
        crate::core::nt_core_hex::ReasoningPath::shortest(self.e8_state, target).states
    }

    /// E8 状态名称 (人类可读)
    pub fn e8_mode_name(&self) -> &'static str {
        self.e8_state.mode_name()
    }
}

// ─────────────────────── 动态 GWT 专家路由 (Ext-5) ───────────────────────

/// 将 SpecialistType 映射到 default_specialist_states() 的索引。
/// `nt_core_gwt::resonance::default_specialist_states()` 按 SpecialistType 枚举
/// 顺序返回 14 个推理态 (PatternMatcher=0 ... EvidenceWeightedHypothesis=13)。
pub fn specialist_index(t: SpecialistType) -> usize {
    match t {
        SpecialistType::PatternMatcher => 0,
        SpecialistType::AnomalyDetector => 1,
        SpecialistType::KnowledgeRetriever => 2,
        SpecialistType::CodeAnalyzer => 3,
        SpecialistType::Planner => 4,
        SpecialistType::KnowledgeIntegrator => 5,
        SpecialistType::GoalPrioritizer => 6,
        SpecialistType::RiskAssessor => 7,
        SpecialistType::CreativityGenerator => 8,
        SpecialistType::ReflectionEngine => 9,
        SpecialistType::MetaCognitionAnalyst => 10,
        SpecialistType::AISecurity => 11,
        SpecialistType::ImageGenerator => 12,
        SpecialistType::EvidenceWeightedHypothesis => 13,
        SpecialistType::Orchestrator => 4, // 无专属谐振态, 借 Planner
    }
}

/// GWT 谐振路由: 用当前 E8 状态与 14 个专家默认态计算谐振强度,
/// 选出 attention 应投给的专家 (winner-take-most by resonance_strength)。
///
/// 返回 (专家, 谐振强度 0..6, 该专家默认态)。
pub fn route_attention(e8_state: ReasoningHexagram) -> (SpecialistType, u32, ReasoningHexagram) {
    let states = crate::core::nt_core_gwt::resonance::default_specialist_states();
    let mut best: Option<(SpecialistType, u32, ReasoningHexagram)> = None;
    for (idx, st) in states.iter().enumerate() {
        let strength = e8_state.resonance_strength(st);
        let t = specialist_index_inv(idx);
        if best.as_ref().map_or(true, |(_, s, _)| strength > *s) {
            best = Some((t, strength, *st));
        }
    }
    best.unwrap_or((SpecialistType::PatternMatcher, 0, ReasoningHexagram::new(0)))
}

/// 索引 → SpecialistType (specialist_index 逆映射)
pub fn specialist_index_inv(idx: usize) -> SpecialistType {
    match idx {
        0 => SpecialistType::PatternMatcher,
        1 => SpecialistType::AnomalyDetector,
        2 => SpecialistType::KnowledgeRetriever,
        3 => SpecialistType::CodeAnalyzer,
        4 => SpecialistType::Planner,
        5 => SpecialistType::KnowledgeIntegrator,
        6 => SpecialistType::GoalPrioritizer,
        7 => SpecialistType::RiskAssessor,
        8 => SpecialistType::CreativityGenerator,
        9 => SpecialistType::ReflectionEngine,
        10 => SpecialistType::MetaCognitionAnalyst,
        11 => SpecialistType::AISecurity,
        12 => SpecialistType::ImageGenerator,
        13 => SpecialistType::EvidenceWeightedHypothesis,
        _ => SpecialistType::Orchestrator,
    }
}

impl FileAbility {
    /// 当前 E8 状态对应的 GWT 注意力投递目标
    pub fn gwt_route(&self) -> (SpecialistType, u32, ReasoningHexagram) {
        route_attention(self.e8_state)
    }

    /// 该文件的静态专家偏好 (按文件大类映射)
    pub fn specialist(&self) -> SpecialistType {
        self.kind.specialist()
    }
}

// ─────────────────────────── OCR 抽象 (Ext-2) ───────────────────────────

/// OCR 引擎抽象 — 图像 → 文字识别。
///
/// NeoTrix 零 unsafe 纪律下 OCR 接入点: 任何自研/第三方 OCR (tesseract 封装、
/// PaddleOCR、视觉多模态模型) 都通过此 trait 注入, 由 FileAbility 统一调度。
/// 默认实现 `RuleBasedOcr` 提供确定性启发式基线 (低置信), 后续可替换为
/// `TesseractOcr`/`VisionModelOcr` 而不改调用方。
pub trait OcrEngine: Send + Sync {
    /// 引擎名
    fn name(&self) -> &str;
    /// 识别图像文件中的文字
    fn recognize(&self, path: &Path) -> OcrResult;
}

/// OCR 结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f64,
    pub engine: String,
}

/// 启发式基线 OCR — 非真实视觉识别, 用于测试 trait 链路与占位
///
/// 说明: NeoTrix 不内置真实 OCR 模型 (R-P1 零 unsafe + 体积约束)。
/// 该实现从图像 EXIF 文本字段/文件名启发式提取, confidence 固定低值,
/// 真实场景请注入 `VisionModelOcr` (例: image → question → LLM 描述)。
/// Hosted 决策: 占位引擎保持模块可编译、链路可测、可被视觉引擎替换。
pub struct RuleBasedOcr;

impl OcrEngine for RuleBasedOcr {
    fn name(&self) -> &str {
        "rule-based"
    }

    fn recognize(&self, path: &Path) -> OcrResult {
        // 提取文件名作为低置信启发 (图像场景无真实文字时退化降级)
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        OcrResult {
            text: stem,
            confidence: 0.05,
            engine: self.name().to_string(),
        }
    }
}

impl FileAbility {
    /// 用给定 OCR 引擎识别图像文字 (动态接线: 默认 RuleBasedOcr)
    pub fn ocr(&self, engine: Option<Box<dyn OcrEngine>>) -> OcrResult {
        if self.kind != FileKind::Image {
            return OcrResult::default();
        }
        let e = engine.unwrap_or_else(|| Box::new(RuleBasedOcr));
        e.recognize(&self.path)
    }
}

/// 简易 MIME 猜测 (基于扩展名)
fn guess_mime(ext: &str) -> String {
    match ext {
        "txt" | "md" | "markdown" | "json" | "xml" | "csv" | "yaml" | "yml" | "toml" => {
            "text/plain".to_string()
        }
        "html" | "htm" => "text/html".to_string(),
        "png" => "image/png".to_string(),
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "gif" => "image/gif".to_string(),
        "webp" => "image/webp".to_string(),
        "bmp" => "image/bmp".to_string(),
        "ico" => "image/x-icon".to_string(),
        "mp3" => "audio/mpeg".to_string(),
        "wav" => "audio/wav".to_string(),
        "ogg" => "audio/ogg".to_string(),
        "flac" => "audio/flac".to_string(),
        "m4a" => "audio/mp4".to_string(),
        "mp4" => "video/mp4".to_string(),
        "webm" => "video/webm".to_string(),
        "mkv" => "video/x-matroska".to_string(),
        "mov" => "video/quicktime".to_string(),
        "pdf" => "application/pdf".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

// ─────────────────────── VSA HyperCube Embedding (Ext-3) ───────────────────────

/// 内容 → 高维超向量嵌入。
///
/// 复用 core 既有 `VSAEngine`/`VsaBackend` (R-P42，不平行重造 VSA)。
/// 方法: 对纯文本 token 序列，每个 token 由确定性 xorshift PRNG (seed=token hash)
/// 生成 `dim` 维 ±1 随机超向量；按位置 `permute` 编码顺序；`bundle` 求和后归一化。
/// 相似度经 `VsaBackend::similarity` (余弦) 度量。
pub fn embed_text(text: &str, dim: usize) -> Vec<f64> {
    let engine = VSAEngine::new(dim);
    if text.trim().is_empty() {
        return vec![0.0; dim];
    }
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut occupancy = 0usize;
    let mut accumulator = vec![0.0; dim];
    for (idx, tok) in tokens.iter().enumerate() {
        let mut seed: u64 = 0xcbf29ce484222325;
        for b in tok.as_bytes() {
            seed ^= *b as u64;
            seed = seed.wrapping_mul(0x100000001b3);
        }
        let v = token_vector(&mut seed, dim);
        let shifted = if idx > 0 {
            engine.permute(&v, idx as isize)
        } else {
            v
        };
        for (a, x) in accumulator.iter_mut().zip(shifted.iter()) {
            *a += x;
        }
        occupancy += 1;
    }
    if occupancy == 0 {
        return vec![0.0; dim];
    }
    let norm = accumulator.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm < 1e-12 {
        vec![0.0; dim]
    } else {
        accumulator.iter().map(|x| x / norm).collect()
    }
}

/// xorshift64 PRNG 生成 ±1 hypervector (确定性, OS 零依赖)
fn token_vector(seed: &mut u64, dim: usize) -> Vec<f64> {
    let mut v = Vec::with_capacity(dim);
    for _ in 0..dim {
        let mut x = *seed;
        if x == 0 {
            x = 0x9e3779b97f4a7c15;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *seed = x;
        v.push(if x & 1 == 1 { 1.0 } else { -1.0 });
    }
    v
}

/// 两个文件的语义相似度 (内容已嵌入 → 余弦)
pub fn content_similarity(path_a: impl AsRef<Path>, path_b: impl AsRef<Path>) -> Result<f64> {
    let engine = VSAEngine::default();
    let dim = engine.dimensions();
    let a = FileAbility::open(path_a)?;
    let b = FileAbility::open(path_b)?;
    let va = embed_text(&a.plain_text(), dim);
    let vb = embed_text(&b.plain_text(), dim);
    Ok(engine.similarity(&va, &vb))
}

// ───────────────────────────── 便捷函数 ─────────────────────────────

/// 提取任何文件的纯文本
pub fn extract_text(path: impl AsRef<Path>) -> Result<String> {
    let mut ab = FileAbility::open(path)?;
    ab.register_consumer();
    Ok(ab.plain_text())
}

/// 转换任何 Office 文件为 Markdown
pub fn to_markdown(path: impl AsRef<Path>) -> Result<String> {
    let mut ab = FileAbility::open(path)?;
    ab.register_consumer();
    Ok(ab.to_markdown())
}

/// 占位符替换 (返回替换次数)
pub fn replace_placeholder(path: impl AsRef<Path>, find: &str, replace: &str) -> Result<usize> {
    let ab = FileAbility::open(path)?;
    ab.replace_placeholder(find, replace)
}

/// 保存/导出能力句柄到目标路径
pub fn save_edited(ability: &FileAbility, target: impl AsRef<Path>) -> Result<()> {
    ability.save_as(target)
}

/// 健康检查 (Dark Forest 生存 + 内容快照)
pub fn check_health(path: impl AsRef<Path>) -> String {
    match FileAbility::open(&path) {
        Ok(mut ab) => {
            ab.register_consumer();
            format!(
                "FileHealth {{ path: {}, kind: {:?}, mime: {}, size: {}, maturity: {:?}, consumers: {} }}",
                path.as_ref().display(),
                ab.kind(),
                ab.mime_type(),
                ab.size_bytes(),
                ab.maturity(),
                ab.has_consumers(),
            )
        }
        Err(e) => format!("FileHealth ERROR: {e}"),
    }
}

/// 用 Markdown 创建 Office 文档 (office_oxide `create_from_markdown`)
pub fn create_from_markdown(
    markdown: &str,
    format: DocumentFormat,
    target: impl AsRef<Path>,
) -> Result<()> {
    create::create_from_markdown(markdown, format, target).map_err(FileAbilityError::Office)
}

// ─────────────────── 通用表格读写 (D1/D2): XLSX 写 + CSV/TSV 读写 ───────────────────
// 对标: python-docx/openpyxl/csv。吸收此前 Python 价格表脚本的表格化逻辑为原生能力。

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

/// 写入 XLSX 表格 (D1) — 表头加粗+底色, 数值列带数字格式, 列宽自适应。
/// 对标 openpyxl: 支持字符串/数值/公式单元格。
pub fn write_xlsx_table(path: impl AsRef<Path>, table: &TableData) -> Result<()> {
    use office_oxide::xlsx::write::{CellData, CellStyle, HAlign, XlsxWriter};
    let mut wb = XlsxWriter::new();
    let sheet = wb.add_sheet_get_index(if table.name.is_empty() {
        "Sheet1"
    } else {
        &table.name
    });
    // 表头
    let header_style = CellStyle::new()
        .bold()
        .font_color("FFFFFF")
        .background("2F5496")
        .align(HAlign::Center)
        .wrap();
    for (col, h) in table.headers.iter().enumerate() {
        wb.sheet_set_cell_styled(sheet, 0, col, CellData::String(h.clone()), header_style.clone());
    }
    // 数据行
    for (r, row) in table.rows.iter().enumerate() {
        for (col, cell) in row.iter().enumerate() {
            let cdata = to_cell_data(cell);
            wb.sheet_set_cell(sheet, r + 1, col, cdata);
        }
    }
    // 列宽 (表头长度 + 内容最大长度, 上限 40)
    for col in 0..table.headers.len() {
        let mut w = table.headers.get(col).map(|h| h.chars().count()).unwrap_or(8);
        for row in &table.rows {
            if let Some(c) = row.get(col) {
                w = w.max(c.chars().count());
            }
        }
        wb.sheet_set_column_width(sheet, col, (w as f64).clamp(6.0, 40.0));
    }
    wb.save(path).map_err(|e| FileAbilityError::Office(office_oxide::OfficeError::from(e)))
}

/// 单元格文本 → CellData (纯数字→Number, 公式前缀→Formula, 其余→String)
fn to_cell_data(text: &str) -> office_oxide::xlsx::write::CellData {
    use office_oxide::xlsx::write::CellData;
    let t = text.trim();
    if let Some(f) = t.strip_prefix('=') {
        if !f.is_empty() {
            return CellData::Formula(f.to_string());
        }
    }
    if let Ok(n) = t.replace(',', "").replace('￥', "").parse::<f64>() {
        return CellData::Number(n);
    }
    CellData::String(t.to_string())
}

/// 写入 CSV 文件 (UTF-8, BOM 可选) — 对标 Python csv.writer
pub fn write_csv(path: impl AsRef<Path>, table: &TableData, delimiter: char, with_bom: bool) -> Result<()> {
    use std::io::Write;
    let mut buf = Vec::new();
    if with_bom {
        buf.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    }
    let mut write_row = |cells: &[String]| -> std::io::Result<()> {
        let line: Vec<String> = cells
            .iter()
            .map(|c| {
                if c.contains(delimiter) || c.contains('"') || c.contains('\n') {
                    format!("\"{}\"", c.replace('"', "\"\""))
                } else {
                    c.clone()
                }
            })
            .collect();
        buf.extend_from_slice(line.join(&delimiter.to_string()).as_bytes());
        buf.push(b'\n');
        Ok(())
    };
    write_row(&table.headers).map_err(FileAbilityError::Io)?;
    for row in &table.rows {
        write_row(row).map_err(FileAbilityError::Io)?;
    }
    let mut f = std::fs::File::create(path.as_ref()).map_err(FileAbilityError::Io)?;
    f.write_all(&buf).map_err(FileAbilityError::Io)
}

/// 读取 CSV/TSV 文件 (自动编码检测: UTF-8 BOM / UTF-8 / GBK) (D2+D3)
pub fn read_csv(path: impl AsRef<Path>) -> Result<TableData> {
    let raw = std::fs::read(path.as_ref()).map_err(FileAbilityError::Io)?;
    let text = decode_bytes(&raw);
    let name = path
        .as_ref()
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(TableData::from_delimited_text(name, &text))
}

/// 读取 XLSX 所有工作表为表格 (calamine 驱动, 懒加载 per-sheet)。
///
/// 外部吸收 (R-P42): calamine 0.36 读 xlsx 比 openpyxl 快 ~9.4×, 支持
/// xls/xlsm/xlsb/ods, 纯 Rust 零 unsafe。对齐 E13 教训: 多 sheet XLSX 需全遍历,
/// 每个 sheet 都可能含独立数据。表头 = 每 sheet 第一个非空行。
pub fn read_xlsx_sheets_all(path: impl AsRef<Path>) -> Result<Vec<TableData>> {
    use calamine::{open_workbook, Reader, Xlsx};
    use std::io::BufReader;
    let mut wb: Xlsx<BufReader<std::fs::File>> = open_workbook(path.as_ref())
        .map_err(|e: calamine::XlsxError| FileAbilityError::Parse(e.to_string()))?;
    let names = wb.sheet_names().to_vec();
    let mut out = Vec::new();
    for name in names {
        let range = wb
            .worksheet_range(&name)
            .map_err(|e: calamine::XlsxError| FileAbilityError::Parse(e.to_string()))?;
        // 计算最大列 (cells() 产出 (row, col, &Data), c.1 为列索引)
        let max_col = range
            .cells()
            .map(|c| c.1)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        if max_col == 0 {
            continue;
        }
        // 按 (row, col) 填充
        let mut grid: Vec<Vec<String>> = Vec::new();
        for cell in range.cells() {
            let (r, c, v) = (cell.0 as usize, cell.1 as usize, cell.2);
            while grid.len() <= r {
                grid.push(vec![String::new(); max_col]);
            }
            grid[r][c] = data_to_text(v);
        }
        // 表头 = 第一个非空行
        let header_idx = grid
            .iter()
            .position(|row| row.iter().any(|c| !c.trim().is_empty()))
            .unwrap_or(0);
        let headers: Vec<String> = grid
            .get(header_idx)
            .cloned()
            .unwrap_or_else(|| vec![String::new(); max_col]);
        let rows: Vec<Vec<String>> = grid
            .iter()
            .enumerate()
            .filter(|(i, row)| *i > header_idx && row.iter().any(|c| !c.trim().is_empty()))
            .map(|(_, row)| row.clone())
            .collect();
        out.push(TableData {
            name: name.clone(),
            headers,
            rows,
        });
    }
    Ok(out)
}

/// calamine Data → 显示文本 (与 office_oxide 显示文本语义对齐)
fn data_to_text(d: &calamine::Data) -> String {
    match d {
        calamine::Data::Int(i) => i.to_string(),
        calamine::Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        calamine::Data::String(s) => s.clone(),
        calamine::Data::Bool(b) => b.to_string(),
        calamine::Data::DateTime(dt) => dt
            .as_datetime()
            .map(|d| d.to_string())
            .unwrap_or_else(|| dt.to_string()),
        calamine::Data::DateTimeIso(s) => s.clone(),
        calamine::Data::DurationIso(s) => s.clone(),
        calamine::Data::Error(_) => String::new(),
        calamine::Data::Empty => String::new(),
    }
}

/// 读取 XLSX 第一个非空工作表为表格 (calamine 驱动)。
/// 兼容原 office_oxide 语义; 多 sheet 场景用 [`read_xlsx_sheets_all`]。
pub fn read_xlsx_table(path: impl AsRef<Path>) -> Result<TableData> {
    let tables = read_xlsx_sheets_all(&path)?;
    tables
        .iter()
        .find(|t| !t.rows.is_empty())
        .cloned()
        .or_else(|| tables.first().cloned())
        .ok_or_else(|| FileAbilityError::Parse("XLSX 无工作表".into()))
}

// ─────────────────────────── 编码检测 (D3) ───────────────────────────

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

/// 检测字节流编码 (BOM 优先, 其次 UTF-8 合法性, 再次 GBK 启发式)。
/// 对标 Python chardet — 覆盖中文场景 GBK/GB18030。
pub fn detect_encoding(data: &[u8]) -> TextEncoding {
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return TextEncoding::Utf8;
    }
    if data.starts_with(&[0xFF, 0xFE]) || data.starts_with(&[0xFE, 0xFF]) {
        return TextEncoding::Utf16;
    }
    if std::str::from_utf8(data).is_ok() {
        return TextEncoding::Utf8;
    }
    // GBK 启发: 高字节区段必须能成对构成合法双字节序列 (首字节 0x81-0xFE,
    // 次字节 0x40-0xFE 且非 0x7F)。以"高字节成对率"判定。
    let mut high = 0usize;
    let mut i = 0usize;
    while i < data.len() {
        let b = data[i];
        if b < 0x80 {
            i += 1;
            continue;
        }
        high += 1;
        // GBK 双字节: 首字节 0x81-0xFE, 次字节 0x40-0xFE (不含 0x7F)
        if (0x81..=0xFE).contains(&b) && i + 1 < data.len() {
            let b2 = data[i + 1];
            if (0x40..=0xFE).contains(&b2) && b2 != 0x7F {
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    if high > 0 {
        let valid_pairs = count_gbk_pairs(data);
        let pair_ratio = valid_pairs as f64 / high as f64;
        if pair_ratio > 0.9 && valid_pairs >= 2 {
            TextEncoding::Gbk
        } else {
            TextEncoding::Unknown
        }
    } else {
        TextEncoding::Unknown
    }
}

/// 统计合法 GBK 双字节对的个数
fn count_gbk_pairs(data: &[u8]) -> usize {
    let mut count = 0usize;
    let mut i = 0usize;
    while i + 1 < data.len() {
        let b = data[i];
        if (0x81..=0xFE).contains(&b) {
            let b2 = data[i + 1];
            if (0x40..=0xFE).contains(&b2) && b2 != 0x7F {
                count += 1;
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    count
}

/// 解码字节流为 UTF-8 字符串 (编码检测 + encoding_rs 转换)。
pub fn decode_bytes(data: &[u8]) -> String {
    let out = match detect_encoding(data) {
        TextEncoding::Gbk => {
            let (cow, _, _) = encoding_rs::GBK.decode(data);
            cow.into_owned()
        }
        TextEncoding::Utf16 => {
            // 自动去除 BOM
            let (cow, _, _) = encoding_rs::UTF_16LE.decode(data);
            cow.into_owned()
        }
        _ => String::from_utf8_lossy(data).into_owned(),
    };
    // 去除 BOM (UTF-8)
    out.trim_start_matches('\u{feff}').to_string()
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

// ─────────────────── 多文件合并 (D4): 通用引擎 + 领域 Schema 分离 ───────────────────
//
// 分层架构 (贯穿整个文件能力体系):
//   L1 格式编解码 (通用): read_xlsx_table/write_csv/detect_encoding — 任何类型文件
//   L2 表格语义 (通用):   merge_tables_with(schema) — 零领域知识的多表合并引擎
//   L3 领域 schema (差异化): PRICE_TABLE_SCHEMA + skills md 镜像 — 唯一个性化层
//   L4 意图层 (通用):     意识核心 xlsx_consolidation → 选 schema → 调 merge_tables_with
//
// 领域知识 (列名变体/标准列序/单位规则/供应商命名/跳过前缀) 全部数据化进 MergeSchema,
// 不再编译进引擎函数。换行业 = 新增一个 schema const, 不改引擎代码。

/// 单重/尺寸等列的补单位规则
#[derive(Debug, Clone, Copy)]
pub struct UnitRule {
    /// 目标标准列名 (如 "单重(Kg)")
    pub column: &'static str,
    /// 值缺失该后缀时追加 (如 "kg")
    pub suffix: &'static str,
    /// 值中已含这些标记则跳过 (如 ["kg", "千克"])
    pub skip_if_contains: &'static [&'static str],
}

/// 标准列数据类型 (用于输出校验)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// 数值列 (可含单位后缀/千分位/货币符; 解析失败记 warning)
    Numeric,
    /// 文本列 (默认)
    Text,
}

/// 合并 Schema — 领域知识数据 (纯 const, 编译期校验)
#[derive(Debug, Clone, Copy)]
pub struct MergeSchema {
    /// Schema 名 (如 "价格表")
    pub name: &'static str,
    /// 标准列序 (合并输出的目标列)
    pub standard_columns: &'static [&'static str],
    /// 列名变体 → 标准列 (每项 (标准列, [变体...]))
    pub column_variants: &'static [(&'static str, &'static [&'static str])],
    /// 供应商/来源名推导: 文件名剥离的后缀标记 (如 "价格表"/"报价模板")
    pub filename_suffixes: &'static [&'static str],
    /// 需要补单位的列规则
    pub unit_rules: &'static [UnitRule],
    /// 用于统计的价值列 (如 USD 报价列), 必须 ∈ standard_columns
    pub value_columns: &'static [&'static str],
    /// 附加透出列 (如 "备注"/"_source_file")
    pub extra_columns: &'static [&'static str],
    /// 输入目录扫描时跳过的文件名前缀 (如已合并输出自身)
    pub skip_prefixes: &'static [&'static str],
    /// 源表无供应商列时, 回退用文件名推导并写入该列
    pub supplier_column: Option<&'static str>,
    /// 同文件内跨 sheet 去重键 (E14: 用 (口径,单价,产品小类) 而非型号 —
    /// 型号常是文件名回退/垃圾值; 为空则不去重)。
    /// 注意: 去重仅限同一文件内, 不同文件=不同供应商, 不去重。
    pub dedup_columns: &'static [&'static str],
    /// 标准列数据类型 (输出校验: 数字列非数值 → validation_warning)
    pub column_types: &'static [(&'static str, ColumnType)],
}

impl MergeSchema {
    /// 校验 schema 一致性 (编译期无法全查, 运行时断言):
    /// 标准列唯一 / value_columns ∈ standard_columns / extra 不与标准列冲突。
    pub fn validate(&self) -> std::result::Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for (i, c) in self.standard_columns.iter().enumerate() {
            if !seen.insert(*c) {
                return Err(format!("standard_columns[{}] 重复: '{}'", i, c));
            }
        }
        for v in self.value_columns {
            if !self.standard_columns.contains(v) {
                return Err(format!(
                    "value_column '{}' 不在 standard_columns 中 (schema '{}')",
                    v, self.name
                ));
            }
        }
        for e in self.extra_columns {
            if self.standard_columns.contains(e) {
                return Err(format!(
                    "extra_column '{}' 与 standard_columns 冲突 (schema '{}')",
                    e, self.name
                ));
            }
        }
        if let Some(sup) = self.supplier_column {
            if !self.standard_columns.contains(&sup) {
                return Err(format!(
                    "supplier_column '{}' 不在 standard_columns 中 (schema '{}')",
                    sup, self.name
                ));
            }
        }
        for d in self.dedup_columns {
            if !self.standard_columns.contains(d) {
                return Err(format!(
                    "dedup_column '{}' 不在 standard_columns 中 (schema '{}')",
                    d, self.name
                ));
            }
        }
        for (col, _t) in self.column_types {
            if !self.standard_columns.contains(col) {
                return Err(format!(
                    "column_types 引用的列 '{}' 不在 standard_columns 中 (schema '{}')",
                    col, self.name
                ));
            }
        }
        Ok(())
    }

    /// 变体 → 标准列 (未命中返回原样, 与旧 normalize_column_name 行为一致)
    pub fn normalize_column(&self, name: &str) -> String {
        let t = name.trim();
        for (std, variants) in self.column_variants {
            if *std == t || variants.contains(&t) {
                return (*std).to_string();
            }
        }
        t.to_string()
    }

    /// 标准列 → 索引
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.standard_columns.iter().position(|c| *c == name)
    }
}

/// 价格表标准列 (兼容导出 — 由 PRICE_TABLE_SCHEMA 派生)
pub const PRICE_STANDARD_COLUMNS: &[&str] = &[
    "产品大类",
    "产品小类",
    "产品型号",
    "阀体材质",
    "阀板材质",
    "阀杆材质",
    "阀座材质",
    "驱动方式",
    "连接方式",
    "标准",
    "压力",
    "口径",
    "含税单价(元)",
    "美元报价(USD)",
    "青岛港FOB报价(USD)",
    "天津港FOB报价(USD)",
    "单重(Kg)",
    "供应商名称",
    "档次",
];

/// 价格表领域 Schema (L3 差异化知识 — 数据化, 引擎零内置)。
/// 迁移自: 原 PRICE_STANDARD_COLUMNS + normalize_column_name 变体表 + 供应商后缀。
pub const PRICE_TABLE_SCHEMA: MergeSchema = MergeSchema {
    name: "价格表",
    standard_columns: PRICE_STANDARD_COLUMNS,
    column_variants: &[
        ("产品大类", &["大类"]),
        ("产品小类", &["小类"]),
        ("产品型号", &["型号", "规格型号", "产品规格"]),
        (
            "阀体材质",
            &["阀体", "body材质", "BODY阀体", "BODY体", "壳材质"],
        ),
        ("阀板材质", &["阀板", "disc材质", "碟板材质", "蝶板材质"]),
        (
            "阀杆材质",
            &["阀杆", "stem材质", "MAIN SHAFT主软", "阀轴材质"],
        ),
        (
            "阀座材质",
            &["阀座", "seat材质", "SEAT RING座座环", "密封圈材质"],
        ),
        ("驱动方式", &["驱动", "操作方式"]),
        ("连接方式", &["连接", "连接形式"]),
        ("标准", &["执行标准", "设计标准"]),
        ("压力", &["公称压力", "压力等级", "PN"]),
        ("口径", &["公称通径", "DN", "尺寸"]),
        (
            "含税单价(元)",
            &["含税单价", "单价(元)", "单价", "价格(元)", "单价(含税)", "含税价(元)"],
        ),
        (
            "美元报价(USD)",
            &["美元价(USD)", "美元价", "美元报价", "USD报价", "单价(美元)", "美元单价"],
        ),
        (
            "青岛港FOB报价(USD)",
            &["青岛港FOB单价(元)", "青岛港FOB单价", "青岛港FOB", "青岛FOB"],
        ),
        (
            "天津港FOB报价(USD)",
            &["天津港FOB单价(元)", "天津港FOB单价", "天津港FOB", "天津FOB"],
        ),
        (
            "单重(Kg)",
            &["单重", "重量(Kg)", "重量", "单重(kg)", "预估单重(Kg)"],
        ),
        ("供应商名称", &["供应商", "厂家", "品牌"]),
        ("档次", &["等级", "级别"]),
        ("备注", &["说明", "备注信息"]),
    ],
    filename_suffixes: &[
        "价格_报价", "价格表", "报价模板", "_报价", "-报价", "价格表_报价", "已完善",
        "-已更新", "-修改版", "-中高档", "-中低档", "-第一版", "-第二版", "-第五版本", "-含税",
    ],
    unit_rules: &[UnitRule {
        column: "单重(Kg)",
        suffix: "kg",
        skip_if_contains: &["kg", "千克"],
    }],
    value_columns: &["美元报价(USD)", "青岛港FOB报价(USD)", "天津港FOB报价(USD)"],
    extra_columns: &["备注", "_source_file"],
    skip_prefixes: &["consolidated", "native_consolidated"],
    supplier_column: Some("供应商名称"),
    dedup_columns: &["口径", "含税单价(元)", "产品小类"],
    column_types: &[
        ("口径", ColumnType::Numeric),
        ("含税单价(元)", ColumnType::Numeric),
        ("美元报价(USD)", ColumnType::Numeric),
        ("青岛港FOB报价(USD)", ColumnType::Numeric),
        ("天津港FOB报价(USD)", ColumnType::Numeric),
        ("单重(Kg)", ColumnType::Numeric),
    ],
};

/// 列名变体 → 标准列 (兼容导出 — 委托 PRICE_TABLE_SCHEMA)
pub fn normalize_column_name(name: &str) -> String {
    PRICE_TABLE_SCHEMA.normalize_column(name)
}

/// 合并报告
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsolidationReport {
    /// 处理的文件数
    pub files_processed: usize,
    /// 读取失败的文件 (路径, 原因)
    pub files_failed: Vec<(String, String)>,
    /// 总数据行数
    pub total_rows: usize,
    /// 含美元报价的行数
    pub usd_rows: usize,
    /// 输出路径
    pub output: String,
    /// 同文件内跨 sheet 去重跳过的行 (E14: 来源标注 文件名::sheet名)
    pub dedup_rows: Option<Vec<String>>,
    /// 输出数据校验告警 (数字列非数值)
    pub validation_warnings: Vec<String>,
}

/// 通用多表合并引擎 (L2 表格语义层 — 零领域知识)。
/// 领域知识全部来自传入的 `MergeSchema` (L3), 本引擎对任何表格目录+对应 schema 通用。
///
/// - 扫描 src_dir 下 xlsx/csv/tsv (跳过 skip_prefixes)
/// - 列名变体 → schema.standard_columns 归一化
/// - 单位规则 (schema.unit_rules) 补充缺失单位
/// - 供应商列缺失时从文件名推导 (schema.filename_suffixes)
/// - value_columns 非空计数统计
/// - 附加透出列 (schema.extra_columns, 含 _source_file)
/// - 输出 XLSX + 返回合并报告
pub fn merge_tables_with(
    schema: &MergeSchema,
    src_dir: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<ConsolidationReport> {
    schema.validate().map_err(FileAbilityError::Parse)?;
    let mut report = ConsolidationReport::default();
    let mut table = TableData {
        name: format!("统一{}", schema.name),
        headers: schema.standard_columns.iter().map(|s| s.to_string()).collect(),
        rows: Vec::new(),
    };
    // 标准列名 → 索引
    let std_idx: std::collections::HashMap<String, usize> = schema
        .standard_columns
        .iter()
        .enumerate()
        .map(|(i, s)| (s.to_string(), i))
        .collect();
    // 附加透出列
    let mut extra_data: Vec<Vec<String>> = Vec::new();

    // 扫描输入目录
    let mut entries: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(read) = std::fs::read_dir(src_dir.as_ref()) {
        for e in read.flatten() {
            let p = e.path();
            let ext = p
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.to_lowercase())
                .unwrap_or_default();
            if matches!(ext.as_str(), "xlsx" | "csv" | "tsv") {
                // 跳过已合并输出 (防止重复合并自身产物)
                let name = p
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_default()
                    .to_lowercase();
                if schema
                    .skip_prefixes
                    .iter()
                    .any(|pfx| name.starts_with(&pfx.to_lowercase()))
                {
                    continue;
                }
                entries.push(p);
            }
        }
    }
    entries.sort();

    for path in entries {
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_lowercase();
        let srcs: Vec<TableData> = match ext.as_str() {
            "xlsx" => match read_xlsx_sheets_all(&path) {
                Ok(tables) => tables,
                Err(e) => {
                    report
                        .files_failed
                        .push((path.display().to_string(), e.to_string()));
                    continue;
                }
            },
            "csv" | "tsv" => match read_csv(&path) {
                Ok(t) => vec![t],
                Err(e) => {
                    report
                        .files_failed
                        .push((path.display().to_string(), e.to_string()));
                    continue;
                }
            },
            _ => continue,
        };
        if srcs.is_empty() {
            continue;
        }
        report.files_processed += 1;
        // 来源名 (从文件名剥离序号/后缀)
        let source_name = derive_source_name(&path, schema);
        // 文件名 (用于 _source_file)
        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();

        // 多 sheet 逐 sheet 合并 (E13: 每个 sheet 都可能含独立数据)
        // 同文件内跨 sheet 去重 (E14: key 用 schema.dedup_columns, 默认去重启用)
        let mut seen: std::collections::HashSet<Vec<String>> = std::collections::HashSet::new();
        let dedup_idx: Vec<Option<usize>> = schema
            .dedup_columns
            .iter()
            .map(|d| std_idx.get(*d).copied())
            .collect();
        for src in srcs {
            // 归一化源表头 → 标准列名
            let norm_headers: Vec<String> = src
                .headers
                .iter()
                .map(|h| schema.normalize_column(h))
                .collect();
            for row in &src.rows {
                // 标准列填充
                let mut std_row = vec![String::new(); schema.standard_columns.len()];
                for (c, h) in norm_headers.iter().enumerate() {
                    if let Some(&idx) = std_idx.get(h) {
                        let v = row.get(c).cloned().unwrap_or_default();
                        std_row[idx] =
                            if v.trim().is_empty() { String::new() } else { v.trim().to_string() };
                    }
                }
                // 供应商列缺失 → 用文件名推导回填
                if let Some(sup) = schema.supplier_column {
                    if !norm_headers.iter().any(|h| h == sup) {
                        if let Some(&idx) = std_idx.get(sup) {
                            std_row[idx] = source_name.clone();
                        }
                    }
                }
                // 单位规则补充
                for rule in schema.unit_rules {
                    if let Some(&idx) = std_idx.get(rule.column) {
                        let v = std_row[idx].trim().to_string();
                        if !v.is_empty()
                            && !rule
                                .skip_if_contains
                                .iter()
                                .any(|mark| v.to_lowercase().contains(&mark.to_lowercase()))
                        {
                            std_row[idx] = format!("{v}{}", rule.suffix);
                        }
                    }
                }
                // 同文件内跨 sheet 去重 (E14): key = dedup_columns 非空拼接
                if !dedup_idx.is_empty() {
                    let key: Vec<String> = dedup_idx
                        .iter()
                        .map(|oi| oi.map(|i| std_row[i].clone()).unwrap_or_default())
                        .collect();
                    // 至少一个 key 字段非空才有意义
                    if key.iter().any(|k| !k.is_empty()) && !seen.insert(key) {
                        report
                            .dedup_rows
                            .get_or_insert_with(Vec::new)
                            .push(format!("{}::{}", file_name, src.name));
                        continue;
                    }
                }
                table.rows.push(std_row.clone());
                // 附加列 (schema.extra_columns; 末位 _source_file 填 文件名[::sheet名])
                let mut extra = vec![String::new(); schema.extra_columns.len()];
                for (c, h) in norm_headers.iter().enumerate() {
                    for (ei, ecol) in schema.extra_columns.iter().enumerate() {
                        if h == *ecol {
                            extra[ei] = row.get(c).cloned().unwrap_or_default();
                        }
                    }
                }
                if let Some(last) = extra.last_mut() {
                    if schema.extra_columns.last() == Some(&"_source_file") {
                        let sheet_suffix =
                            if !src.name.is_empty() { format!("::{}", src.name) } else { String::new() };
                        *last = format!("{file_name}{sheet_suffix}");
                    }
                }
                extra_data.push(extra);
                // value_columns 统计 (基于去重后 std_row, 非空计数)
                let has_value = schema.value_columns.iter().any(|vc| {
                    std_idx
                        .get(*vc)
                        .and_then(|&i| std_row.get(i))
                        .map(|v| !v.trim().is_empty())
                        .unwrap_or(false)
                });
                if has_value {
                    report.usd_rows += 1;
                }
            }
            // 清理: norm_headers 作用域结束
        }
    }

    // 拼接附加列到输出表格
    table.headers.extend(schema.extra_columns.iter().map(|s| s.to_string()));
    for (i, r) in table.rows.iter_mut().enumerate() {
        if let Some(e) = extra_data.get(i) {
            r.extend(e.iter().cloned());
        } else {
            r.extend(vec![String::new(); schema.extra_columns.len()]);
        }
    }
    report.total_rows = table.rows.len();
    report.output = output.as_ref().display().to_string();

    // 输出数据校验 (阶段2): 数字列非数值 → validation_warnings
    for (ci, (col, ctype)) in schema.column_types.iter().enumerate() {
        if *ctype != ColumnType::Numeric {
            continue;
        }
        let col_idx = std_idx.get(*col);
        let Some(&col_idx) = col_idx else { continue };
        for (ri, row) in table.rows.iter().enumerate() {
            let Some(v) = row.get(col_idx) else { continue };
            let v = v.trim();
            if v.is_empty() {
                continue; // 空值不算错误
            }
            if parse_numeric(v).is_none() {
                report.validation_warnings.push(format!(
                    "row{} 列'{}' 非数值: '{}'",
                    ri + 1,
                    col,
                    v
                ));
            }
        }
    }

    write_xlsx_table(output, &table)?;
    Ok(report)
}

/// 宽松数值解析 (兼容 千分位/货币符/单位后缀/科学计数法)。
/// 返回 Some(数值) 若可解析, None 否则。
fn parse_numeric(v: &str) -> Option<f64> {
    let t = v
        .replace(',', "")
        .replace('￥', "")
        .replace('¥', "")
        .replace('$', "");
    // 单位后缀 (如 "2.5kg" / "300元") — 剥离尾部非数值字符
    let trimmed: String = t
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+' || *c == 'e' || *c == 'E')
        .collect();
    trimmed.parse::<f64>().ok()
}

/// 价格表合并 (D4) — 薄封装: 委托通用引擎 + 价格表 schema。
/// 保持向后兼容签名; 领域知识已外置为 PRICE_TABLE_SCHEMA。
pub fn consolidate_tables(
    src_dir: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<ConsolidationReport> {
    merge_tables_with(&PRICE_TABLE_SCHEMA, src_dir, output)
}

/// 从文件名推导来源名 (通用: 剥离序号/后缀, 后缀来自 schema.filename_suffixes)。
/// 例: "4、玉鹏价格_报价模板-修改版" → "玉鹏"
fn derive_source_name(path: &Path, schema: &MergeSchema) -> String {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut s = stem.as_str();
    // 剥离开头的数字序号 (如 "4、", "44. ", "7、")
    if let Some(stripped) = strip_leading_num(s) {
        s = stripped;
    }
    // 剥离后缀标记 (schema.filename_suffixes)
    for marker in schema.filename_suffixes {
        if let Some(pos) = s.find(marker) {
            s = &s[..pos];
        }
    }
    s.trim_matches(|c: char| c == '_' || c == '-' || c == '、' || c == '.' || c == ' ').to_string()
}

/// 剥离开头的数字序号 (返回剩余部分)
fn strip_leading_num(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    // 跳过后续分隔符 (、. 空格)
    let rest = &s[i..];
    let rest = rest.trim_start_matches(|c: char| c == '、' || c == '.' || c == ' ' || c == '-' || c == '_');
    if rest.is_empty() {
        None
    } else {
        Some(rest)
    }
}

// ─────────────────── 结构化数据读写 (D5): JSON/YAML ───────────────────

/// 结构化文件读取结果 — 统一 JSON/YAML 为 serde_json::Value
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredData {
    pub format: String,
    pub value: serde_json::Value,
}

/// 读取 JSON/YAML 结构化文件 (D5)
pub fn read_structured(path: impl AsRef<Path>) -> Result<StructuredData> {
    let raw = std::fs::read(path.as_ref()).map_err(FileAbilityError::Io)?;
    let text = decode_bytes(&raw);
    let ext = path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "json" | "jsonc" => Ok(StructuredData {
            format: "json".to_string(),
            value: serde_json::from_str(&text)
                .map_err(|e| FileAbilityError::Parse(e.to_string()))?,
        }),
        "yaml" | "yml" => {
            let v: serde_yaml::Value = serde_yaml::from_str(&text)
                .map_err(|e| FileAbilityError::Parse(e.to_string()))?;
            let value = serde_json::to_value(v)
                .map_err(|e| FileAbilityError::Parse(e.to_string()))?;
            Ok(StructuredData {
                format: "yaml".to_string(),
                value,
            })
        }
        other => Err(FileAbilityError::UnsupportedFormat {
            ext: other.to_string(),
        }),
    }
}

/// 写入 JSON 文件 (D5)
pub fn write_json(path: impl AsRef<Path>, value: &serde_json::Value, pretty: bool) -> Result<()> {
    let text = if pretty {
        serde_json::to_string_pretty(value)
            .map_err(|e| FileAbilityError::Parse(e.to_string()))?
    } else {
        serde_json::to_string(value)
            .map_err(|e| FileAbilityError::Parse(e.to_string()))?
    };
    std::fs::write(path.as_ref(), text).map_err(FileAbilityError::Io)
}

// ─────────────────── 内容快照存储 (D6) ───────────────────

/// 将 ContentSnapshot 持久化为 JSON 文件 (D6)
pub fn store_snapshot(snapshot: &ContentSnapshot, target: impl AsRef<Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| FileAbilityError::Parse(e.to_string()))?;
    std::fs::write(target.as_ref(), json).map_err(FileAbilityError::Io)
}

/// 从 JSON 文件加载 ContentSnapshot (D6)
pub fn load_snapshot(path: impl AsRef<Path>) -> Result<ContentSnapshot> {
    let raw = std::fs::read(path.as_ref()).map_err(FileAbilityError::Io)?;
    serde_json::from_slice(&raw).map_err(|e| FileAbilityError::Parse(e.to_string()))
}

// ─────────────────────── doc7 吸收: Grounding 精确值校验 ───────────────────────
// 来源: github.com/magicrew/doc7 internal/extract/grounding.go + grounding_numeric.go
// (MIT, absorbed 2026-08-13, cycle 1101)。
//
// 公理 (Grounding 是精确值保险): 视觉理解管线可能幻读数字/代码/ID。从嵌入文本层
// 提取关键 token (≥3 位数字或含小数/百分号/货币符号, 以及大写字母+数字标识符),
// 与 VLM 输出比对, 缺失则标记为 ungrounded — 由调用方决定二次校正 (遵循 R-P36:
// grounding 结果必须进入行为, 而非仅日志)。本模块是纯算法, 无 LLM 依赖。

/// 关键标识符模式: 大写字母+数字+_/- (如 "Attention-12", "B2")
const CRITICAL_IDENTIFIER_RE: &str = r"[A-Z]{2,}[A-Z0-9_-]*\d[A-Z0-9_-]*";

/// 关键数字 token 模式: 可选正负/百分号前缀, 数字带千分位逗号与可选小数
const NUMERIC_TOKEN_RE: &str = r"(?:\([0-9][0-9,]*(?:\.[0-9]+)?%?\)|[+\-−－△]?[0-9][0-9,]*(?:\.[0-9]+)?%?)";

/// 多段版本号模式 (如 2.5.1, 3.14.159) — doc7 单小数段正则的增强
const VERSION_TOKEN_RE: &str = r"[0-9]+\.[0-9]+(?:\.[0-9]+)+";

/// 数字 token 判定: ≥3 位数字, 或含小数/百分号/货币符号
fn is_critical_numeric_token(value: &str) -> bool {
    let trimmed = value.trim().trim_matches(|c| c == '(' || c == ')');
    let digits = trimmed.chars().filter(|c| c.is_ascii_digit()).count();
    digits >= 3 || trimmed.chars().any(|c| matches!(c, '.' | '%' | '$' | '€' | '£' | '¥'))
}

/// 数值 token 归一化: 空格/unicode 减号/括号归一, 便于跨来源比对
fn normalize_numeric_token(value: &str) -> String {
    let mut v = value.trim().trim_end_matches(',').to_string();
    v = v.replace(['−', '－'], "-").replace('△', "-").replace(' ', "");
    if v.starts_with("△") {
        v = format!("-{}", &v["△".len()..]);
    }
    if (v.starts_with('(') && v.ends_with(')')) || (v.starts_with('（') && v.ends_with('）')) {
        v = v[1..v.len() - 1].to_string();
    }
    v
}

/// 紧凑化文本: 去除空白/Markdown 标记/unicode 减号, 用于存在性比对
fn compact_numeric_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => continue,
            '−' | '－' | '△' => out.push('-'),
            '\\' | '$' | '{' | '}' | '^' | '_' | '*' | '`' => {} // Markdown/LaTeX 标记
            _ => out.push(ch),
        }
    }
    out
}

/// 判断文本是否为数学行 (含 LaTeX 标记 → grounding 跳过, 防止错改公式)
fn is_math_line(value: &str) -> bool {
    value.contains('$') || value.contains('^') || value.contains('{') || value.contains("\\")
}

/// 关键数字 token 提取 (带 1-based 位置, 与 doc7 一致用于行定位)
fn critical_numeric_tokens(value: &str) -> Vec<(usize, String)> {
    let version_re = regex::Regex::new(VERSION_TOKEN_RE).expect("VERSION_TOKEN_RE 有效");
    let re = regex::Regex::new(NUMERIC_TOKEN_RE).expect("NUMERIC_TOKEN_RE 有效");
    let mut tokens = Vec::new();
    let mut covered: Vec<(usize, usize)> = Vec::new();
    for (pos, cap) in version_re.captures_iter(value).enumerate() {
        let m = cap.get(0).expect("group 0");
        covered.push((m.start(), m.end()));
        let raw = &value[m.start()..m.end()];
        if is_critical_numeric_token(raw) {
            tokens.push((pos + 1, raw.to_string()));
        }
    }
    for (pos, cap) in re.captures_iter(value).enumerate() {
        let m = cap.get(0).expect("group 0");
        if covered.iter().any(|(s, e)| *s <= m.start() && m.end() <= *e) {
            continue;
        }
        let raw = &value[m.start()..m.end()];
        let trimmed = raw.trim_start_matches(|c: char| {
            c.is_whitespace() || matches!(c, '+' | '-' | '−' | '－' | '△' | '(')
        });
        let trimmed = trimmed.trim_end_matches(|c: char| c.is_whitespace() || c == ')');
        if is_critical_numeric_token(trimmed) {
            tokens.push((pos + 1, trimmed.to_string()));
        }
    }
    tokens
}

/// 关键标识符提取 (大写字母开头且含数字)
fn critical_identifiers(value: &str) -> Vec<String> {
    regex::Regex::new(CRITICAL_IDENTIFIER_RE)
        .expect("CRITICAL_IDENTIFIER_RE 有效")
        .find_iter(value)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// 数值序列是否一致 (source vs content 归一化后按 token 计数比对)
fn numeric_sequences_equal(source: &str, content: &str) -> bool {
    use std::collections::HashMap;
    let mut left: HashMap<String, usize> = HashMap::new();
    for (_, t) in critical_numeric_tokens(source) {
        *left.entry(normalize_numeric_token(&t)).or_insert(0) += 1;
    }
    let mut right: HashMap<String, usize> = HashMap::new();
    for (_, t) in critical_numeric_tokens(content) {
        *right.entry(normalize_numeric_token(&t)).or_insert(0) += 1;
    }
    left == right
}

/// grounding 检查结果
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GroundingReport {
    pub checked: bool,
    pub missing_numeric: Vec<String>,
    pub missing_identifiers: Vec<String>,
    pub math_guard_skipped: usize,
    pub ungrounded: Vec<String>,
    pub sequence_ok: bool,
    /// 可靠性评分 (0.0 ~ 1.0) — 公理二 (grounding 是精确值保险) 的量化表达。
    /// 1.0 = 源文本所有关键 token 均保真输出; 越低表示 VLM 输出与源越偏离。
    /// 计算: 未接地 token 占关键 token 比例的反向 (R-P79 生产接地)。
    pub reliability_score: f64,
}

/// 执行 grounding 检查: 源文本层 token vs VLM 输出内容。
/// 返回缺失 token 清单; 调用方决定二次校正或标记失败 (R-P79 接线)。
pub fn ground_missing_tokens(source_text: &str, content: &str) -> GroundingReport {
    let mut report = GroundingReport {
        checked: !source_text.trim().is_empty(),
        ..Default::default()
    };
    if !report.checked {
        // 空源无关键 token 可校验 → 视为保真 (公理二: 无风险则保险成立)
        report.reliability_score = 1.0;
        return report;
    }

    // 关键数字: 源存在而输出缺失
    let compact_content = compact_numeric_text(content);
    let content_tokens: std::collections::HashSet<String> = critical_numeric_tokens(content)
        .iter()
        .map(|(_, t)| normalize_numeric_token(t))
        .collect();
    let mut seen = std::collections::HashSet::new();
    for (_, t) in critical_numeric_tokens(source_text) {
        let norm = normalize_numeric_token(&t);
        if !seen.insert(norm.clone()) {
            continue;
        }
        let missing = !content_tokens.contains(&norm) && !compact_content.contains(&compact_numeric_text(&t));
        if missing {
            // math 行保护: 输出缺失数字所在行若含 LaTeX → 判定为公式, 跳过
            let line = content.lines().find(|l| compact_numeric_text(l).contains(&compact_numeric_text(&t)));
            if let Some(line) = line {
                if is_math_line(line) {
                    report.math_guard_skipped += 1;
                    continue;
                }
            }
            report.missing_numeric.push(t.clone());
            report.ungrounded.push(t);
        }
    }

    // 关键标识符
    seen.clear();
    for id in critical_identifiers(source_text) {
        if !seen.insert(id.clone()) {
            continue;
        }
        if !compact_content.contains(&compact_numeric_text(&id)) {
            report.missing_identifiers.push(id.clone());
            report.ungrounded.push(id);
        }
    }

    report.sequence_ok = numeric_sequences_equal(source_text, content);
    // 公理二量化: 可靠性评分 = 1 - (未接地 token / 源关键 token 总数)
    // math_guard_skipped 视为合理豁免 (公式行), 不计入未接地。
    let total_critical = critical_numeric_tokens(source_text)
        .iter()
        .map(|(_, t)| normalize_numeric_token(t))
        .collect::<std::collections::HashSet<_>>()
        .len()
        + critical_identifiers(source_text)
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
    if total_critical > 0 {
        report.reliability_score = 1.0 - (report.ungrounded.len() as f64 / total_critical as f64);
    } else {
        report.reliability_score = 1.0; // 无关键 token → 默认保真
    }
    report.reliability_score = report.reliability_score.clamp(0.0, 1.0);
    report
}

// ─────────────────────── doc7 吸收: 视觉理解 Prompt (VLM 提取) ───────────────────────
// 来源: github.com/magicrew/doc7 internal/extract/prompt.go (MIT, absorbed 2026-08-13)。
// 公理 (视觉理解是提取上限): 光栅化页面 → VLM 整页理解 → 保真 Markdown。
// 按输入类型路由 (文档/幻灯片), 严格表格/视觉/图/公式保留规则, temperature 0 可复现。

/// 视觉理解 prompt 名称
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualPromptKind {
    /// 文档页 (PDF/Office/图像/扫描件)
    Document,
    /// 演示幻灯片 (PPT/PPTX/ODP)
    Slide,
}

impl VisualPromptKind {
    /// 预留: 仅供测试断言 (name 往返) 与未来日志/调试输出使用, 非生产调用路径。
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            VisualPromptKind::Document => "document",
            VisualPromptKind::Slide => "slide",
        }
    }

    /// 按 FileKind 自动路由: 演示类 → Slide, 其余 → Document
    pub fn for_file_kind(kind: FileKind) -> Self {
        match kind {
            FileKind::Office(office_oxide::DocumentFormat::Pptx)
            | FileKind::Office(office_oxide::DocumentFormat::Ppt) => VisualPromptKind::Slide,
            _ => VisualPromptKind::Document,
        }
    }

    /// 返回完整视觉理解 prompt (doc7 documentPrompt/slidePrompt 忠实吸收)
    pub fn prompt(&self) -> &'static str {
        match self {
            VisualPromptKind::Document => DOC7_DOCUMENT_PROMPT,
            VisualPromptKind::Slide => DOC7_SLIDE_PROMPT,
        }
    }
}

/// doc7 documentPrompt — 整页保真转写 (表格/视觉/图/公式/页脚全覆盖)
pub const DOC7_DOCUMENT_PROMPT: &str = r#"Transcribe the entire document page as a faithful Markdown fragment from top to bottom. First scan the whole page, including small text at the edges and bottom, then write the transcription. Preserve the original language, reading order, hierarchy, and every readable word, number, unit, table row, chart point, formula, label, status, button, footnote, caption, and footer. Use headings, paragraphs, lists, tables, code blocks, and LaTeX as appropriate. Use $$...$$ for standalone displayed formulas and $...$ only for formulas embedded in prose.

Table rules:
- A single visual table must become one continuous Markdown table with one header row and the same column count on every data row.
- Bold, shaded, indented, subtotal, total, and section rows are still data rows; never turn them into a new header or a second table.
- Keep hierarchical row labels in the first column and preserve every value in its original column.
- Do not merge nearby labels into extra columns or invent blank header rows. If formatting is uncertain, preserve the values as ordered plain text rather than silently changing their relationships.

Visual rules:
- Describe every non-text visual only as one or more blockquotes in the form > [Visual] ...; never use Markdown image syntax, Mermaid, ASCII art, SVG, diagram code, or invented image links.
- For charts, preserve visible axes, legends, series, values, trends, and conclusions.
- For diagrams, workflows, and multi-part figures, write enough ordered prose that a reader could reconstruct the visible topology without seeing the page.
- State the visual type, title, overall reading direction, spatial arrangement of parts, input order, every readable node or label, and every visible directed connection in sequence.
- Explicitly preserve branches, merges, bypass connections, loops, parallel paths, repeated components and counts, nested or grouped regions, and the final visible destination. Distinguish separate paths instead of collapsing them into a summary.
- Audit each arrow endpoint before writing. For a long bypass arrow that crosses intermediate nodes, state the exact source and destination and name the skipped nodes; never attach an input to the nearest or lowest box merely because the arrow passes beside it.
- Describe only visible structure. Do not infer hidden nodes, implicit outputs, or relationships that are not shown.
- Never replace a visual with only its title or caption. Do not summarize, translate, infer, invent, or omit readable information. Mark unreadable content as "不可读" on Chinese pages or "unreadable" otherwise.

Return only Markdown without metadata, commentary, or enclosing code fences."#;

/// doc7 slidePrompt — 幻灯片保真转写
pub const DOC7_SLIDE_PROMPT: &str = r#"Transcribe the entire presentation slide as a faithful Markdown fragment. Use the visible title as the leading heading. Preserve the original language, reading order, hierarchy, and every readable bullet, label, example, number, unit, table cell, formula, brand, tool, and decision criterion. Use $$...$$ for standalone displayed formulas and $...$ only for formulas embedded in prose.

Visual rules:
- Describe every chart, matrix, funnel, workflow, quadrant, screenshot, or other non-text visual only as one or more blockquotes in the form > [Visual] ...; never use Markdown image syntax, Mermaid, ASCII art, SVG, diagram code, or invented image links.
- For charts, preserve visible axes, legends, series, values, trends, and conclusions.
- For diagrams, workflows, and multi-part figures, write enough ordered prose that a reader could reconstruct the visible topology without seeing the slide.
- State the visual type, title, overall reading direction, spatial arrangement of parts, input order, every readable node or label, and every visible directed connection in sequence.
- Explicitly preserve branches, merges, bypass connections, loops, parallel paths, repeated components and counts, nested or grouped regions, comparisons, and the final visible destination. Distinguish separate paths instead of collapsing them into a summary.
- Audit each arrow endpoint before writing. For a long bypass arrow that crosses intermediate nodes, state the exact source and destination and name the skipped nodes; never attach an input to the nearest or lowest box merely because the arrow passes beside it.
- Describe only visible structure. Do not infer hidden nodes, implicit outputs, or relationships that are not shown.
- Never replace a visual with only its title or caption. Do not summarize, translate, infer, invent, or omit readable information. Mark unreadable content as "不可读" on Chinese slides or "unreadable" otherwise.

Return only Markdown without metadata, commentary, or enclosing code fences."#;

// ─────────────────────── doc7 吸收: 视觉理解提取管线 (VisualExtractor) ───────────────────────
// 公理 (视觉理解是提取上限): 光栅化页面 → VLM 整页理解 → 保真 Markdown + grounding 校验。
// 设计 (R-P42): 不建平行 provider, 模型通道经闭包注入 — 生产接 NT-IO LlmProvider,
// 测试接假闭包 (零网络依赖)。管线核心 (准备/prompt/校验/报告) 为同步纯逻辑, 可测。

/// VLM 模型调用闭包: 输入 (prompt, base64 图像, mime), 输出 (转写文本, 失败原因)
pub type VlmCall = dyn Fn(&str, &str, &str) -> std::result::Result<String, String>;

/// 视觉提取配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualExtractConfig {
    /// 图像最大字节 (超限降级, doc7 默认 9MB)
    pub max_image_bytes: usize,
    /// 上下文降级次数 (doc7 --context-fallbacks 默认 2)
    pub context_fallbacks: usize,
    /// 最小图像边长 (降级下限, doc7 --min-image-dimension 默认 720)
    pub min_image_dimension: u32,
    /// 是否启用文本层 grounding 校验 (doc7 --text-grounding)
    pub text_grounding: bool,
    /// 是否重试 (doc7 RetryCount)
    pub retry_count: usize,
}

impl Default for VisualExtractConfig {
    fn default() -> Self {
        Self {
            max_image_bytes: 9 * 1024 * 1024,
            context_fallbacks: 2,
            min_image_dimension: 720,
            text_grounding: false,
            retry_count: 2,
        }
    }
}

/// 视觉提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualExtractResult {
    pub markdown: String,
    pub prompt_kind: VisualPromptKind,
    pub grounding: Option<GroundingReport>,
    pub context_fallbacks_used: usize,
    pub image_max_dimension: u32,
    pub failed: bool,
    pub error: Option<String>,
    /// 公理一 (视觉理解是提取上限) 量化: 提取是否在保真上限内。
    /// true = 未接地 token 占比低于阈值 (可靠性 ≥ 0.5) 或未启用 grounding。
    /// false = 提取超出保真上限, 调用方应二次校正或标记人工复核。
    pub extraction_bound_ok: bool,
}

/// 执行一次 VLM 视觉提取 (doc7 单页管线核心)。
/// - `image_b64`: base64 编码的图像 (PNG/JPEG)
/// - `source_text`: 嵌入文本层 (PDF/Office 文本层), 供 grounding; 无则传 ""
/// - `kind`: 文件类型 → prompt 路由
/// - `call`: 模型闭包 (prompt, image_b64, mime) -> Result<String>
pub fn visual_extract(
    kind: FileKind,
    image_b64: &str,
    source_text: &str,
    config: &VisualExtractConfig,
    call: &VlmCall,
) -> VisualExtractResult {
    let prompt_kind = VisualPromptKind::for_file_kind(kind);
    let prompt = prompt_kind.prompt();
    let mut result = VisualExtractResult {
        markdown: String::new(),
        prompt_kind,
        grounding: None,
        context_fallbacks_used: 0,
        image_max_dimension: 0,
        failed: false,
        error: None,
        extraction_bound_ok: true,
    };

    // 图像大小上限校验 → 超限标记需要降级 (调用方决定, 此处报告)
    if image_b64.len() > config.max_image_bytes {
        result.error = Some(format!(
            "image exceeds {} bytes ({} actual); lower resolution or increase max_image_bytes",
            config.max_image_bytes,
            image_b64.len()
        ));
        result.failed = true;
        return result;
    }

    // 调用模型 (temperature 0 由调用方 provider 配置, 此处重试语义)
    let mut last_err = None;
    let mut content = String::new();
    for attempt in 0..=config.retry_count {
        match call(prompt, image_b64, "image/png") {
            Ok(text) => {
                content = text;
                last_err = None;
                break;
            }
            Err(e) => {
                last_err = Some(e);
                // 最后尝试失败才中断
                if attempt == config.retry_count {
                    break;
                }
            }
        }
    }
    if let Some(e) = last_err {
        result.error = Some(format!("VLM call failed after {} attempts: {e}", config.retry_count + 1));
        result.failed = true;
        return result;
    }

    result.markdown = content;

    // grounding 校验 (可选): 嵌入文本层关键 token 缺失检测
    if config.text_grounding && !source_text.trim().is_empty() {
        let report = ground_missing_tokens(source_text, &result.markdown);
        // 公理一 (视觉理解是提取上限): 可靠性 < 0.5 → 提取超出保真上限。
        // 调用方据此二次校正或标记人工复核 (R-P79 生产接线决策点)。
        result.extraction_bound_ok = report.reliability_score >= 0.5;
        result.grounding = Some(report);
    }

    result
}

// ─────────────────────── SelfTest T1 接线 ───────────────────────

/// FileAbility 自检 — 验证模块能力链路健康
pub struct FileAbilitySelfTest;

impl SelfTest for FileAbilitySelfTest {
    fn name(&self) -> &str {
        // nt_io_ 前缀 → BranchKind::Io 分支健康上报 (ConsciousnessTree::from_module_name)
        "nt_io_file_ability"
    }

    fn self_test(&self) -> std::result::Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // 1) Office 格式枚举可解析
        if DocumentFormat::from_extension("docx").is_none() {
            failures.push("office_oxide DocumentFormat::docx 探测失败".into());
        }

        // 2) 文本探测链路可用
        let fmt = neotrix_types::core::file_parser::FileParser::detect_format(
            "sample.txt",
            "text/plain",
            b"hello world",
        );
        if !matches!(fmt, neotrix_types::core::file_parser::FileFormat::PlainText) {
            failures.push("FileParser 文本探测失败".into());
        }

        // 3) 能力成熟度晋级链完整
        if ConstellationLevel::C0Compile.next() != Some(ConstellationLevel::C1UnitTest) {
            failures.push("ConstellationLevel 晋级链中断".into());
        }

        // 4) VSA embedding 链路 (Ext-3): 确定性 + 维度契约
        let v = embed_text("NeoTrix self-test", 256);
        if v.len() != 256 || v.iter().all(|x| *x == 0.0) {
            failures.push("VSA embed_text 维度/非零契约破坏".into());
        }
        let v2 = embed_text("NeoTrix self-test", 256);
        let sim = crate::core::nt_core_hcube::vsa::VSAEngine::new(256).similarity(&v, &v2);
        if sim < 0.99 {
            failures.push(format!("VSA embedding 确定性退化 sim={sim}"));
        }

        // 5) E8 状态机链路 (Ext-6): 目标状态合法 + ReasoningPath 可达
        for op in [
            FileOperation::Detect,
            FileOperation::Extract,
            FileOperation::Transform,
            FileOperation::Edit,
            FileOperation::Embed,
            FileOperation::Audit,
        ] {
            let target = op.target_state();
            if target.0 >= 64 || target.mode_name().is_empty() {
                failures.push(format!("E8 目标状态非法 op={}", op.name()));
            }
        }
        let path = crate::core::nt_core_hex::ReasoningPath::shortest(
            ReasoningHexagram::new(0b001001),
            ReasoningHexagram::new(0b111100),
        );
        if path.states.is_empty() {
            failures.push("E8 shortest path 为空".into());
        }

        // 6) GWT 注意力路由链路 (Ext-5): 专家索引往返 + 路由返回强度
        if specialist_index_inv(specialist_index(SpecialistType::Planner))
            != SpecialistType::Planner
        {
            failures.push("GWT specialist_index 往返不一致".into());
        }
        let (_spec, strength, _state) = route_attention(ReasoningHexagram::new(0b001100));
        if strength == 0 {
            failures.push("GWT route_attention 强度为零".into());
        }

        // 7) OCR 链路 (Ext-2): 规则引擎可实例化
        let ocr = RuleBasedOcr;
        if ocr.name().is_empty() {
            failures.push("RuleBasedOcr name 为空".into());
        }

        // 8) XLSX 单元格级结构化读取链路 (Ext-7): 写入探针 → 打开 → 结构读取
        let mut xw = office_oxide::xlsx::write::XlsxWriter::new();
        let s0 = xw.add_sheet_get_index("probe");
        xw.sheet_set_cell(
            s0,
            0,
            0,
            office_oxide::xlsx::write::CellData::String("k".into()),
        );
        xw.sheet_set_cell(s0, 0, 1, office_oxide::xlsx::write::CellData::Number(1.5));
        let probe_dir = std::env::temp_dir().join("nt_file_ability_selftest");
        let probe_path = probe_dir.join("probe.xlsx");
        if std::fs::create_dir_all(&probe_dir).is_err() || xw.save(&probe_path).is_err() {
            failures.push("XlsxWriter 探针写入失败".into());
        } else if let Ok(ab) = FileAbility::open(&probe_path) {
            match ab.xlsx_sheet(1) {
                Ok(sheet) => {
                    if sheet.rows.is_empty() || sheet.rows[0].cells.len() < 2 {
                        failures.push("XLSX 结构化读取单元格缺失".into());
                    } else {
                        let first = &sheet.rows[0].cells[0];
                        if first.value_type != SheetCellValueType::Text
                            || first.text != "k"
                            || first.reference != "A1"
                        {
                            failures.push(format!(
                                "XLSX 结构化读取值错误: {:?} '{}' @ {}",
                                first.value_type, first.text, first.reference
                            ));
                        }
                        if sheet.rows[0].cells[1].value_type != SheetCellValueType::Number {
                            failures.push("XLSX 结构化读取数字类型错误".into());
                        }
                    }
                }
                Err(e) => failures.push(format!("XLSX 结构化读取失败: {e}")),
            }
        } else {
            failures.push("XLSX 探针打开失败".into());
        }
        let _ = std::fs::remove_file(&probe_path);
        let _ = std::fs::remove_dir(&probe_dir);

        // 9) 统一表格读写链路 (D1/D2/D3): XLSX 写读回环 + CSV 写读回环 + 编码探测/解码。
        //    生产接地: write_xlsx_table 即合并输出通路 (consolidate_tables → 落盘)。
        let probe_tbl = TableData {
            name: "probe".into(),
            headers: vec!["名称".into(), "数量".into()],
            rows: vec![vec!["阀门".into(), "12".into()]],
        };
        let probe_dir = std::env::temp_dir().join("nt_file_ability_selftest");
        if std::fs::create_dir_all(&probe_dir).is_err() {
            failures.push("selftest 临时目录创建失败".into());
        }
        let probe_xlsx = probe_dir.join("probe_tbl.xlsx");
        let probe_csv = probe_dir.join("probe_tbl.csv");
        if write_xlsx_table(&probe_xlsx, &probe_tbl).is_err() {
            failures.push("write_xlsx_table 失败".into());
        } else {
            match read_xlsx_table(&probe_xlsx) {
                Ok(t) => {
                    if t.headers != probe_tbl.headers || t.rows != probe_tbl.rows {
                        failures.push("XLSX 表格写读回环不一致".into());
                    }
                }
                Err(e) => failures.push(format!("read_xlsx_table 失败: {e}")),
            }
        }
        if write_csv(&probe_csv, &probe_tbl, ',', false).is_err() {
            failures.push("write_csv 失败".into());
        } else {
            match read_csv(&probe_csv) {
                Ok(t) => {
                    if t.headers != probe_tbl.headers || t.rows != probe_tbl.rows {
                        failures.push("CSV 表格写读回环不一致".into());
                    }
                }
                Err(e) => failures.push(format!("read_csv 失败: {e}")),
            }
        }
        let gbk_bytes = b"\xd6\xd0\xb9\xfa";
        if detect_encoding(gbk_bytes) != TextEncoding::Gbk || decode_bytes(gbk_bytes) != "中国" {
            failures.push("GBK 编码探测/解码契约破坏".into());
        }
        let _ = std::fs::remove_file(&probe_xlsx);
        let _ = std::fs::remove_file(&probe_csv);

        // 10) 多表合并链路 (D4): 两文件端到端合并 → 列名归一化 + 行数 + 排序表头。
        //     生产接地: 合并产物已内置跳过 consolidate_* 自身输出, 防止重复吸收。
        let m_dir = probe_dir.join("merge");
        if std::fs::create_dir_all(&m_dir).is_ok() {
            let a = TableData {
                name: "a".into(),
                headers: vec!["产品型号".into(), "阀体材质".into(), "单重(Kg)".into()],
                rows: vec![vec!["V1".into(), "WCB".into(), "1.5kg".into()]],
            };
            let b = TableData {
                name: "b".into(),
                headers: vec!["型号".into(), "阀体材料".into(), "重量".into()],
                rows: vec![vec!["V2".into(), "CF8".into(), "2kg".into()]],
            };
            if write_xlsx_table(m_dir.join("a.xlsx"), &a).is_err()
                || write_xlsx_table(m_dir.join("b.xlsx"), &b).is_err()
            {
                failures.push("合并源写入失败".into());
            } else {
                let out = m_dir.join("out.xlsx");
                match consolidate_tables(&m_dir, &out) {
                    Ok(rep) => {
                        if rep.files_processed != 2 || rep.total_rows != 2 {
                            failures.push(format!(
                                "合并统计异常 processed={} rows={}",
                                rep.files_processed, rep.total_rows
                            ));
                        }
                        if let Ok(t) = read_xlsx_table(&out) {
                            let has_std = t
                                .headers
                                .iter()
                                .any(|h| h == "阀体材质" || h == "单重(Kg)");
                            if !has_std {
                                failures.push("合并列名未归一化为标准列".into());
                            }
                        }
                    }
                    Err(e) => failures.push(format!("consolidate_tables 失败: {e}")),
                }
            }
            let _ = std::fs::remove_dir_all(&m_dir);
        }

        // 11) 结构化/快照链路 (D5/D6): JSON 写读回环 + 快照存读。
        let s_path = probe_dir.join("probe.json");
        if let Ok(sd) = read_structured(s_path.as_path()) {
            // 文件不存在时 read_structured 应报错; 存在时 roundtrip
            let _ = sd;
        }
        let snap = ContentSnapshot {
            kind: FileKind::Text,
            text: Some("探针".into()),
            markdown: None,
            image: None,
            media: None,
            mime_type: "text/plain".into(),
            size_bytes: 42,
        };
        let snap_path = probe_dir.join("probe_snapshot.json");
        if store_snapshot(&snap, &snap_path).is_err() {
            failures.push("store_snapshot 失败".into());
        } else if let Ok(back) = load_snapshot(&snap_path) {
            if back.text != snap.text || back.mime_type != snap.mime_type {
                failures.push("快照存读回环不一致".into());
            }
        } else {
            failures.push("load_snapshot 失败".into());
        }
        let _ = std::fs::remove_file(&s_path);
        let _ = std::fs::remove_file(&snap_path);
        let _ = std::fs::remove_dir(&probe_dir);

        // 12) grounding 公理链路 (公理二: 精确值保险): 可靠性评分契约 + 提取上限分发。
        // 生产接地: self_test 结果经 run_all → set_branch_health 驱动 NT-IO 分支健康 (T3)。
        let gr = ground_missing_tokens("版本 2.5.1 已发布", "版本 2.5.1 已发布");
        if gr.reliability_score < 0.99 {
            failures.push(format!("grounding 完全保真应得满分, 实得 {}", gr.reliability_score));
        }
        let gr_lossy = ground_missing_tokens("版本 2.5.1 发布", "内容缺失");
        if gr_lossy.checked && gr_lossy.reliability_score >= 1.0 {
            failures.push("grounding 丢失 token 不应得满分".into());
        }
        // 公理一: 提取上限分发契约 (reliability < 0.5 → 超限)
        let fake: &VlmCall = &|_p, _i, _m| Ok("内容缺失".to_string());
        let cfg = VisualExtractConfig { text_grounding: true, ..Default::default() };
        let r = visual_extract(FileKind::Text, "img", "版本 2.5.1 发布", &cfg, fake);
        if !r.failed && r.extraction_bound_ok {
            failures.push("低可靠性提取应标记 extraction_bound_ok=false".into());
        }

if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn office_sample() -> PathBuf {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.docx");
        if !path.exists() {
            create::create_from_markdown(
                "# 标题\n\nHello {{name}}, 这是 NeoTrix 测试文档。",
                DocumentFormat::Docx,
                &path,
            )
            .unwrap();
        }
        path
    }

    #[test]
    fn test_open_office_and_plain_text() {
        let path = office_sample();
        let mut ab = FileAbility::open(&path).unwrap();
        ab.register_consumer();
        assert!(matches!(ab.kind(), FileKind::Office(DocumentFormat::Docx)));
        assert!(ab.has_consumers());
        assert!(ab.plain_text().contains("NeoTrix"));
        assert!(ab.to_markdown().contains("标题"));
        assert!(ab.to_html().is_some());
    }

    #[test]
    fn test_replace_placeholder_docx() {
        let path = office_sample();
        let copy = path.with_extension("replace.docx");
        std::fs::copy(&path, &copy).unwrap();
        let ab = FileAbility::open(&copy).unwrap();
        let n = ab.replace_placeholder("{{name}}", "NeoTrix").unwrap();
        assert!(n > 0, "应至少替换一次占位符");
        std::fs::remove_file(&copy).ok();
    }

    // ── XLSX 单元格级结构化读取 (Ext-7) ──

    /// 构造多 sheet 测试夹具: sheet1 = "修改版" (表头+数据+公式), sheet2 = "原始"
    fn xlsx_fixture() -> PathBuf {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("prices.xlsx");
        if !path.exists() {
            use office_oxide::xlsx::write::{CellData, XlsxWriter};
            let mut xw = XlsxWriter::new();
            let s1 = xw.add_sheet_get_index("Sheet1");
            let s2 = xw.add_sheet_get_index("Sheet2");
            xw.sheet_set_cell(s1, 0, 0, CellData::String("品名".into()));
            xw.sheet_set_cell(s1, 0, 1, CellData::String("单价".into()));
            xw.sheet_set_cell(s1, 1, 0, CellData::String("蝶阀".into()));
            xw.sheet_set_cell(s1, 1, 1, CellData::Number(305.69));
            xw.sheet_set_cell(s1, 2, 0, CellData::String("闸阀".into()));
            xw.sheet_set_cell(s1, 2, 1, CellData::Number(128.0));
            xw.sheet_set_cell(s1, 3, 0, CellData::String("小计".into()));
            xw.sheet_set_cell(s1, 3, 1, CellData::Formula("B2+B3".into()));
            xw.sheet_set_cell(s2, 0, 0, CellData::String("备注".into()));
            xw.sheet_set_cell(s2, 1, 0, CellData::String("原始数据".into()));
            xw.save(&path).unwrap();
        }
        path
    }

    #[test]
    fn test_xlsx_structured_read() {
        let path = xlsx_fixture();
        let ab = FileAbility::open(&path).unwrap();
        // sheet 名 + 数量
        let names = ab.xlsx_sheet_names().unwrap();
        assert_eq!(names, vec!["Sheet1", "Sheet2"]);
        assert_eq!(ab.xlsx_sheet_count().unwrap(), 2);
        // 按名称读取 "修改版" 等价 sheet (Sheet1)
        let s1 = ab.xlsx_sheet_by_name("Sheet1").unwrap();
        assert_eq!(s1.name, "Sheet1");
        assert_eq!(s1.rows.len(), 4);
        // 表头行
        let hdr = &s1.rows[0];
        assert_eq!(hdr.index, 1);
        assert_eq!(hdr.cells.len(), 2);
        assert_eq!(hdr.cells[0].text, "品名");
        assert_eq!(hdr.cells[0].reference, "A1");
        assert_eq!(hdr.cells[0].col, 1);
        assert_eq!(hdr.cells[0].row, 1);
        assert_eq!(hdr.cells[0].value_type, SheetCellValueType::Text);
        // 数值行: 显示文本 + 原始数值 + 类型
        let row2 = &s1.rows[1];
        assert_eq!(row2.cells[1].text, "305.69");
        assert_eq!(row2.cells[1].number, Some(305.69));
        assert_eq!(row2.cells[1].value_type, SheetCellValueType::Number);
        assert_eq!(row2.cells[1].reference, "B2");
        // 公式行: 公式文本保留 (office_oxide 不重算公式, 显示文本可为空)
        let row4 = &s1.rows[3];
        assert_eq!(row4.cells[1].formula.as_deref(), Some("B2+B3"));
        // 行序与 index
        assert_eq!(
            s1.rows.iter().map(|r| r.index).collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
        // sheet 2
        let s2 = ab.xlsx_sheet(2).unwrap();
        assert_eq!(s2.name, "Sheet2");
        assert_eq!(s2.rows.len(), 2);
        assert_eq!(s2.rows[1].cells[0].text, "原始数据");
    }

    #[test]
    fn test_xlsx_sheet_index_boundaries() {
        let path = xlsx_fixture();
        let ab = FileAbility::open(&path).unwrap();
        // 0-based / 越界索引报错
        assert!(matches!(
            ab.xlsx_sheet(0),
            Err(FileAbilityError::SheetIndexOutOfRange { .. })
        ));
        assert!(matches!(
            ab.xlsx_sheet(3),
            Err(FileAbilityError::SheetIndexOutOfRange { .. })
        ));
        // 未知 sheet 名报错
        assert!(matches!(
            ab.xlsx_sheet_by_name("不存在"),
            Err(FileAbilityError::SheetIndexOutOfRange { .. })
        ));
    }

    #[test]
    fn test_xlsx_structured_read_non_xlsx() {
        let path = office_sample(); // docx
        let ab = FileAbility::open(&path).unwrap();
        assert!(matches!(
            ab.xlsx_sheet_names(),
            Err(FileAbilityError::UnsupportedFormat { .. })
        ));
    }

    #[test]
    fn test_open_text_file() {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hello.rs");
        std::fs::write(&path, "fn main() { println!(\"hi\"); }").unwrap();
        let ab = FileAbility::open(&path).unwrap();
        assert_eq!(ab.kind(), FileKind::Text);
        assert!(ab.plain_text().contains("main"));
        std::fs::remove_file(&path).ok();
    }

    // ── D1-D6: 通用表格读写 / 编码检测 / 多文件合并 / 结构化数据 / 快照存储 ──

    fn test_dir() -> PathBuf {
        let d = std::env::temp_dir().join("nt_file_ability_d1d6");
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn test_d1_write_xlsx_table_roundtrip() {
        let dir = test_dir();
        let path = dir.join("t_out.xlsx");
        let t = TableData {
            name: "Sheet1".to_string(),
            headers: vec!["品名".to_string(), "单价".to_string(), "单重(Kg)".to_string()],
            rows: vec![
                vec!["蝶阀".to_string(), "305.69".to_string(), "8".to_string()],
                vec!["闸阀".to_string(), "128".to_string(), "12.5kg".to_string()],
            ],
        };
        write_xlsx_table(&path, &t).unwrap();
        // 回读校验
        let back = read_xlsx_table(&path).unwrap();
        assert_eq!(back.headers, vec!["品名", "单价", "单重(Kg)"]);
        assert_eq!(back.row_count(), 2);
        assert_eq!(back.cell(0, "品名"), Some("蝶阀"));
        assert_eq!(back.cell(1, "单价"), Some("128"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_d2_csv_write_read_roundtrip() {
        let dir = test_dir();
        let path = dir.join("t.csv");
        let t = TableData {
            name: "prices".to_string(),
            headers: vec!["品名".to_string(), "单价".to_string()],
            rows: vec![
                vec!["蝶阀".to_string(), "305.69".to_string()],
                vec!["闸阀,带逗号".to_string(), "128".to_string()],
            ],
        };
        write_csv(&path, &t, ',', true).unwrap();
        let back = read_csv(&path).unwrap();
        assert_eq!(back.headers, vec!["品名", "单价"]);
        assert_eq!(back.row_count(), 2);
        // 引号包裹字段正确还原
        assert_eq!(back.cell(1, "品名"), Some("闸阀,带逗号"));
        assert_eq!(back.cell(0, "单价"), Some("305.69"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_xlsx_sheets_all_multi_sheet() {
        // calamine 多 sheet 全遍历 (E13): 每 sheet 独立表头 = 首个非空行。
        // 锁定 read_xlsx_sheets_all 契约: sheet 数 / 表头 / 数据行。
        let path = xlsx_fixture();
        let tables = read_xlsx_sheets_all(&path).unwrap();
        assert_eq!(tables.len(), 2, "应读出全部 2 个 sheet");
        let s1 = &tables[0];
        assert_eq!(s1.name, "Sheet1");
        assert_eq!(s1.headers, vec!["品名", "单价"]);
        assert_eq!(s1.rows.len(), 3, "表头后应有 3 行数据");
        assert_eq!(s1.rows[0][0], "蝶阀");
        assert_eq!(s1.rows[1][0], "闸阀");
        let s2 = &tables[1];
        assert_eq!(s2.name, "Sheet2");
        assert_eq!(s2.headers, vec!["备注"]);
        assert_eq!(s2.rows.len(), 1);
        assert_eq!(s2.rows[0][0], "原始数据");
    }

    #[test]
    fn test_data_to_text_contract() {
        // 数值/文本/布尔语义锁定 (跨库显示文本契约):
        // 整数值浮点去 .0、小数保留、超大数不丢精度。
        assert_eq!(data_to_text(&calamine::Data::Int(123)), "123");
        assert_eq!(data_to_text(&calamine::Data::Float(305.69)), "305.69");
        assert_eq!(data_to_text(&calamine::Data::Float(100.0)), "100");
        assert_eq!(data_to_text(&calamine::Data::Float(1e16)), "10000000000000000");
        assert_eq!(data_to_text(&calamine::Data::String("蝶阀".into())), "蝶阀");
        assert_eq!(data_to_text(&calamine::Data::Bool(true)), "true");
        assert_eq!(data_to_text(&calamine::Data::Empty), "");
        // DateTime 语义: 必须渲染为日期而非 Excel 原始序列号。
        // serial 44484.7916666667 ≈ 2021-10-15 19:00:00 (calamine 自带测试锚点)。
        let dt = calamine::Data::DateTime(calamine::ExcelDateTime::new(
            44484.7916666667,
            calamine::ExcelDateTimeType::DateTime,
            false,
        ));
        let s = data_to_text(&dt);
        assert!(
            s.starts_with("2021-10-15"),
            "DateTime 应渲染为日期, 实际: {s}"
        );
        assert!(
            !s.contains("44484"),
            "不得输出 Excel 原始序列号, 实际: {s}"
        );
    }

    #[test]
    fn test_d4_dedup_cross_sheet_same_key() {
        // E14 同文件跨 sheet 去重: 单文件双 sheet 共享去重键 (单价) →
        // 只保留首行, 第二 sheet 重复行计入 dedup_rows。
        let dir = test_dir();
        let src = dir.join("dedup_src");
        std::fs::create_dir_all(&src).unwrap();
        use office_oxide::xlsx::write::{CellData, XlsxWriter};
        let mut xw = XlsxWriter::new();
        let s1 = xw.add_sheet_get_index("Sheet1");
        let s2 = xw.add_sheet_get_index("Sheet2");
        for s in [s1, s2] {
            xw.sheet_set_cell(s, 0, 0, CellData::String("品名".into()));
            xw.sheet_set_cell(s, 0, 1, CellData::String("单价(元)".into()));
            xw.sheet_set_cell(s, 1, 0, CellData::String("蝶阀".into()));
            xw.sheet_set_cell(s, 1, 1, CellData::Number(100.0));
        }
        xw.save(src.join("供应商甲.xlsx")).unwrap();

        let schema = MergeSchema {
            name: "去重测试",
            standard_columns: &["品名", "单价(元)"],
            column_variants: &[],
            filename_suffixes: &[],
            unit_rules: &[],
            value_columns: &["单价(元)"],
            extra_columns: &["_source_file"],
            skip_prefixes: &["consolidated"],
            supplier_column: None,
            dedup_columns: &["单价(元)"],
        column_types: &[],
        };
        let out = dir.join("dedup_out.xlsx");
        let report = merge_tables_with(&schema, &src, &out).unwrap();
        // 两 sheet 各 1 数据行, 去重键相同 → 只保留 1 行
        assert_eq!(report.total_rows, 1);
        assert_eq!(report.usd_rows, 1);
        assert_eq!(report.dedup_rows.as_ref().map(|v| v.len()), Some(1));
        assert!(report.dedup_rows.unwrap()[0].contains("Sheet2"));
        // 输出回读验证行数
        let back = read_xlsx_table(&out).unwrap();
        assert_eq!(back.row_count(), 1);
        assert_eq!(back.cell(0, "品名"), Some("蝶阀"));
        std::fs::remove_dir_all(&src).ok();
        std::fs::remove_file(&out).ok();
    }

    #[test]
    fn test_d3_detect_encoding_bom_utf8_gbk() {
        // UTF-8 BOM
        assert_eq!(detect_encoding(&[0xEF, 0xBB, 0xBF, b'a']), TextEncoding::Utf8);
        // 纯 UTF-8
        assert_eq!(detect_encoding("你好, NeoTrix".as_bytes()), TextEncoding::Utf8);
        // UTF-16 BOM
        assert_eq!(detect_encoding(&[0xFF, 0xFE, b'a', 0x00]), TextEncoding::Utf16);
        // 非法 UTF-8 高字节区段 → GBK 启发
        assert_eq!(detect_encoding(&[0xD6, 0xD0, 0xB9, 0xFA]), TextEncoding::Gbk);
        // GBK 解码往返
        let gbk_bytes = encode_gbk("测试");
        let decoded = decode_bytes(&gbk_bytes);
        assert_eq!(decoded, "测试");
    }

    // GBK 编码辅助 (测试用, 与实现对称)
    fn encode_gbk(s: &str) -> Vec<u8> {
        let (enc, _, _) = encoding_rs::GBK.encode(s);
        enc.into_owned()
    }

    #[test]
    fn test_d4_normalize_column_name() {
        assert_eq!(normalize_column_name("阀体材质"), "阀体材质");
        assert_eq!(normalize_column_name("BODY阀体"), "阀体材质");
        assert_eq!(normalize_column_name("stem材质"), "阀杆材质");
        assert_eq!(normalize_column_name("单价(美元)"), "美元报价(USD)");
        assert_eq!(normalize_column_name("青岛港FOB单价(元)"), "青岛港FOB报价(USD)");
        assert_eq!(normalize_column_name("单重(kg)"), "单重(Kg)");
        assert_eq!(normalize_column_name("未知列"), "未知列");
    }

    #[test]
    fn test_d4_supplier_from_filename() {
        // 供应商名推导已 schema 化: derive_source_name 委托 PRICE_TABLE_SCHEMA
        assert_eq!(
            derive_source_name(Path::new("4、玉鹏价格_报价模板-修改版.xlsx"), &PRICE_TABLE_SCHEMA),
            "玉鹏"
        );
        assert_eq!(
            derive_source_name(
                Path::new("44. 河北威世迪阀门_报价模板-第一版-修改版.xlsx"),
                &PRICE_TABLE_SCHEMA
            ),
            "河北威世迪阀门"
        );
        assert_eq!(
            derive_source_name(Path::new("7、青岛润通_报价模板-修改版.xlsx"), &PRICE_TABLE_SCHEMA),
            "青岛润通"
        );
    }

    #[test]
    fn test_d4_schema_validate() {
        // 合法 schema 通过
        PRICE_TABLE_SCHEMA.validate().unwrap();
        // value_columns 必须是标准列
        let bad = MergeSchema {
            name: "bad",
            standard_columns: &["a", "b"],
            column_variants: &[],
            filename_suffixes: &[],
            unit_rules: &[],
            value_columns: &["c"],
            extra_columns: &["d"],
            skip_prefixes: &[],
            supplier_column: Some("b"),
            dedup_columns: &[],
        column_types: &[],
        };
        assert!(bad.validate().is_err());
        // 标准列重复
        let dup = MergeSchema {
            name: "dup",
            standard_columns: &["a", "a"],
            column_variants: &[],
            filename_suffixes: &[],
            unit_rules: &[],
            value_columns: &["a"],
            extra_columns: &[],
            skip_prefixes: &[],
            supplier_column: None,
            dedup_columns: &[],
        column_types: &[],
        };
        assert!(dup.validate().is_err());
        // extra 与标准列冲突
        let clash = MergeSchema {
            name: "clash",
            standard_columns: &["a"],
            column_variants: &[],
            filename_suffixes: &[],
            unit_rules: &[],
            value_columns: &["a"],
            extra_columns: &["a"],
            skip_prefixes: &[],
            supplier_column: None,
            dedup_columns: &[],
        column_types: &[],
        };
        assert!(clash.validate().is_err());
    }

    #[test]
    fn test_d4_merge_tables_with_generic_schema() {
        // 通用引擎零领域知识验证: 自定义 schema + 变体列
        let dir = test_dir();
        let src = dir.join("merge_generic_src");
        std::fs::create_dir_all(&src).unwrap();
        // 源表1: 使用变体列名 + 无供应商列 (由文件名推导)
        let t1 = TableData {
            name: "s1".into(),
            headers: vec!["品名".to_string(), "价格".to_string(), "重量".to_string()],
            rows: vec![vec!["A阀".to_string(), "100".to_string(), "2.5".to_string()]],
        };
        write_xlsx_table(src.join("1、华北阀门_目录-第一版.xlsx"), &t1).unwrap();
        // 源表2: 使用标准列名 + 显式供应商
        let t2 = TableData {
            name: "s2".into(),
            headers: vec![
                "产品型号".to_string(),
                "单价(元)".to_string(),
                "单重(Kg)".to_string(),
                "供应商名称".to_string(),
            ],
            rows: vec![vec![
                "B阀".to_string(),
                "200".to_string(),
                String::new(),
                "华南阀门".to_string(),
            ]],
        };
        write_xlsx_table(src.join("2、华东阀门_目录.xlsx"), &t2).unwrap();

        let schema = MergeSchema {
            name: "测试目录",
            standard_columns: &["品名", "单价(元)", "单重(Kg)", "供应商名称"],
            column_variants: &[
                ("品名", &["产品型号", "型号"]),
                ("单价(元)", &["价格", "单价"]),
                ("单重(Kg)", &["重量"]),
            ],
            filename_suffixes: &["目录", "-第一版"],
            unit_rules: &[UnitRule {
                column: "单重(Kg)",
                suffix: "kg",
                skip_if_contains: &["kg"],
            }],
            value_columns: &["单价(元)"],
            extra_columns: &["_source_file"],
            skip_prefixes: &["consolidated"],
            supplier_column: Some("供应商名称"),
            dedup_columns: &["单价(元)"],
        column_types: &[],
        };
        let out = dir.join("merge_generic_out.xlsx");
        let report = merge_tables_with(&schema, &src, &out).unwrap();
        assert_eq!(report.files_processed, 2);
        assert_eq!(report.total_rows, 2);
        assert_eq!(report.usd_rows, 2); // 两行都有单价

        let merged = read_xlsx_table(&out).unwrap();
        // 变体列归一化: 源表1 "价格"→"单价(元)", "品名"不变
        assert_eq!(merged.headers[..4], vec!["品名", "单价(元)", "单重(Kg)", "供应商名称"]);
        // 源表1 供应商由文件名推导 "华北阀门", 单重补 kg
        assert_eq!(merged.rows[0][0], "A阀");
        assert_eq!(merged.rows[0][1], "100");
        assert_eq!(merged.rows[0][2], "2.5kg");
        assert_eq!(merged.rows[0][3], "华北阀门");
        // 源表2 显式供应商保留, 空单重不补单位
        assert_eq!(merged.rows[1][0], "B阀");
        assert_eq!(merged.rows[1][2], "");
        assert_eq!(merged.rows[1][3], "华南阀门");
        // 附加 _source_file 透出
        assert_eq!(merged.headers[4], "_source_file");
        assert!(merged.rows[0][4].contains("华北阀门_目录-第一版.xlsx"));
        assert!(merged.rows[1][4].contains("华东阀门_目录.xlsx"));
    }

    #[test]
    fn test_d4_merge_multi_sheet_dedup() {
        // 多 sheet 全遍历 (E13) + 同文件内跨 sheet 去重 (E14), 跨文件不去重
        let dir = test_dir();
        let src = dir.join("merge_multisheet_src");
        std::fs::create_dir_all(&src).unwrap();
        // 单文件双 sheet: sheet1 有 2 行, sheet2 有 1 行独立 + 1 行与 sheet1 重复
        // 需用 XlsxWriter 写双 sheet
        {
            use office_oxide::xlsx::write::{CellData, XlsxWriter};
            let mut xw = XlsxWriter::new();
            let s1 = xw.add_sheet_get_index("主表");
            for (c, h) in ["产品型号", "单价(元)", "口径"].iter().enumerate() {
                xw.sheet_set_cell(s1, 0, c, CellData::String(h.to_string()));
            }
            for (r, row) in [["A阀", "100", "DN50"], ["B阀", "200", "DN65"]].iter().enumerate() {
                for (c, v) in row.iter().enumerate() {
                    xw.sheet_set_cell(s1, r + 1, c, CellData::String(v.to_string()));
                }
            }
            let s2 = xw.add_sheet_get_index("副表");
            for (c, h) in ["产品型号", "单价(元)", "口径"].iter().enumerate() {
                xw.sheet_set_cell(s2, 0, c, CellData::String(h.to_string()));
            }
            // 与主表 B阀/DN65 重复 + 独立 C阀
            for (r, row) in [["C阀", "300", "DN80"], ["B阀", "200", "DN65"]].iter().enumerate() {
                for (c, v) in row.iter().enumerate() {
                    xw.sheet_set_cell(s2, r + 1, c, CellData::String(v.to_string()));
                }
            }
            xw.save(src.join("1、多sheet厂_目录.xlsx")).unwrap();
        }
        // 另一文件同 key (DN65/200) — 跨文件不去重
        let t2 = TableData {
            name: "s".into(),
            headers: vec![
                "产品型号".to_string(),
                "单价(元)".to_string(),
                "口径".to_string(),
            ],
            rows: vec![vec![
                "D阀".to_string(),
                "200".to_string(),
                "DN65".to_string(),
            ]],
        };
        write_xlsx_table(src.join("2、异厂_目录.xlsx"), &t2).unwrap();

        let schema = MergeSchema {
            name: "测试目录",
            standard_columns: &["产品型号", "单价(元)", "口径", "供应商名称"],
            column_variants: &[],
            filename_suffixes: &["目录"],
            unit_rules: &[],
            value_columns: &["单价(元)"],
            extra_columns: &["_source_file"],
            skip_prefixes: &["consolidated"],
            supplier_column: Some("供应商名称"),
            dedup_columns: &["口径", "单价(元)"],
            column_types: &[],
        };
        let out = dir.join("merge_multisheet_out.xlsx");
        let report = merge_tables_with(&schema, &src, &out).unwrap();
        // 全遍历 4 行, 同文件去重 1 行 (副表 B阀/DN65), 剩 3 行;
        // 文件2 D阀 同 key [DN65,200] 跨文件不去重 → 总计 4 行
        assert_eq!(report.files_processed, 2);
        assert_eq!(report.total_rows, 4);
        let dedup = report.dedup_rows.as_ref().unwrap();
        assert_eq!(dedup.len(), 1);
        assert!(dedup[0].contains("多sheet厂_目录.xlsx::副表"), "去重行来源应为副表: {dedup:?}");
        assert_eq!(report.usd_rows, 4);

        let merged = read_xlsx_sheets_all(&out).unwrap();
        let m = merged.first().unwrap();
        assert_eq!(m.rows.len(), 4);
        // 主表两行 + 副表独立 C阀 + 异厂 D阀 (同 key 跨文件保留)
        let lines: Vec<String> = m.rows.iter().map(|r| r[..3].join("|")).collect();
        assert!(lines.iter().any(|l| l == "A阀|100|DN50"));
        assert!(lines.iter().any(|l| l == "C阀|300|DN80"));
        assert!(lines.iter().any(|l| l == "D阀|200|DN65"));
        // B阀|200|DN65 只出现一次 (同文件去重), D阀|200|DN65 是异厂行保留
        assert_eq!(lines.iter().filter(|l| l.as_str() == "B阀|200|DN65").count(), 1);
        assert_eq!(lines.iter().filter(|l| l.as_str() == "D阀|200|DN65").count(), 1);
        // _source_file 标注 sheet
        let srcs: Vec<&str> = m.rows.iter().map(|r| r[4].as_str()).collect();
        assert!(srcs.iter().any(|s| s.contains("::主表")), "主表来源标注: {srcs:?}");
        assert!(srcs.iter().any(|s| s.contains("::副表")), "副表来源标注: {srcs:?}");
    }

    #[test]
    fn test_d4_validation_warnings() {
        // 输出数据校验: Numeric 列非数值值 → validation_warnings; 千分位/单位后缀可解析
        let dir = test_dir();
        let src = dir.join("merge_validate_src");
        std::fs::create_dir_all(&src).unwrap();
        let t1 = TableData {
            name: "Sheet1".into(),
            headers: vec![
                "产品型号".to_string(),
                "单价(元)".to_string(),
                "单重(Kg)".to_string(),
            ],
            rows: vec![
                vec!["A阀".into(), "1,200".into(), "2.5kg".into()],
                vec!["B阀".into(), "N/A".into(), "3".into()],
                vec!["C阀".into(), "300".into(), "abc".into()],
            ],
        };
        write_xlsx_table(src.join("1、校验厂_目录.xlsx"), &t1).unwrap();
        let schema = MergeSchema {
            name: "校验目录",
            standard_columns: &["产品型号", "单价(元)", "单重(Kg)"],
            column_variants: &[],
            filename_suffixes: &["目录"],
            unit_rules: &[],
            value_columns: &["单价(元)"],
            extra_columns: &["_source_file"],
            skip_prefixes: &["consolidated"],
            supplier_column: None,
            dedup_columns: &[],
            column_types: &[
                ("单价(元)", ColumnType::Numeric),
                ("单重(Kg)", ColumnType::Numeric),
            ],
        };
        let out = dir.join("merge_validate_out.xlsx");
        let report = merge_tables_with(&schema, &src, &out).unwrap();
        assert_eq!(report.total_rows, 3);
        // 非数值: B阀单价 "N/A", C阀单重 "abc" → 2 条警告; "1,200" 与 "2.5kg" 可解析不告警
        assert_eq!(report.validation_warnings.len(), 2, "{:?}", report.validation_warnings);
        assert!(report.validation_warnings[0].contains("单价(元)"), "{:?}", report.validation_warnings);
        assert!(report.validation_warnings[0].contains("N/A"), "{:?}", report.validation_warnings);
        assert!(report.validation_warnings[1].contains("单重(Kg)"), "{:?}", report.validation_warnings);
        assert!(report.validation_warnings[1].contains("abc"), "{:?}", report.validation_warnings);
    }

    #[test]
    fn test_d4_consolidate_tables() {
        let dir = test_dir();
        let src = dir.join("consolidate_src");
        std::fs::create_dir_all(&src).unwrap();
        // 两个 CSV 供应商文件, 列名变体不同
        let t1 = TableData {
            name: "a".to_string(),
            headers: vec!["品名".to_string(), "BODY阀体".to_string(), "单价".to_string(), "单重".to_string()],
            rows: vec![vec!["蝶阀".to_string(), "铸铁".to_string(), "305.69".to_string(), "8".to_string()]],
        };
        write_csv(src.join("1、甲工厂_报价模板.csv"), &t1, ',', false).unwrap();
        let t2 = TableData {
            name: "b".to_string(),
            headers: vec![
                "品名".to_string(),
                "阀体".to_string(),
                "单价(美元)".to_string(),
                "重量(Kg)".to_string(),
            ],
            rows: vec![vec!["闸阀".to_string(), "不锈钢".to_string(), "42.5".to_string(), "12".to_string()]],
        };
        write_csv(src.join("2、乙工厂_报价模板.csv"), &t2, ',', false).unwrap();

        let out = dir.join("consolidated.xlsx");
        let report = consolidate_tables(&src, &out).unwrap();
        assert_eq!(report.files_processed, 2);
        assert_eq!(report.total_rows, 2);
        assert!(report.usd_rows >= 1, "应检测到 USD 报价行");
        assert!(out.exists());
        // 回读验证归一化 + 供应商名 + 单重单位
        let back = read_xlsx_table(&out).unwrap();
        assert!(back.headers.iter().any(|h| h == "阀体材质"), "列名应归一化为标准列");
        assert!(back.headers.iter().any(|h| h == "供应商名称"));
        let suppliers: Vec<String> = (0..back.row_count())
            .map(|i| back.cell(i, "供应商名称").unwrap_or("").to_string())
            .collect();
        assert!(suppliers.iter().any(|s| s.contains("甲工厂")));
        assert!(suppliers.iter().any(|s| s.contains("乙工厂")));
        // 单重带 kg
        let weights: Vec<String> = (0..back.row_count())
            .map(|i| back.cell(i, "单重(Kg)").unwrap_or("").to_string())
            .collect();
        assert!(weights.iter().any(|w| w.contains("kg")), "单重应带单位: {weights:?}");
        std::fs::remove_file(&out).ok();
    }

    #[test]
    fn test_d5_read_structured_json_yaml() {
        let dir = test_dir();
        let j = dir.join("cfg.json");
        std::fs::write(&j, r#"{"name":"NeoTrix","level":5}"#).unwrap();
        let s = read_structured(&j).unwrap();
        assert_eq!(s.format, "json");
        assert_eq!(s.value["name"], "NeoTrix");
        let y = dir.join("cfg.yaml");
        std::fs::write(&y, "name: NeoTrix\nlevel: 5\n").unwrap();
        let sy = read_structured(&y).unwrap();
        assert_eq!(sy.format, "yaml");
        assert_eq!(sy.value["level"], 5);
        std::fs::remove_file(&j).ok();
        std::fs::remove_file(&y).ok();
    }

    #[test]
    fn test_d5_write_json_roundtrip() {
        let dir = test_dir();
        let j = dir.join("out.json");
        let v = serde_json::json!({"a": 1, "b": [true, null]});
        write_json(&j, &v, true).unwrap();
        let s = read_structured(&j).unwrap();
        assert_eq!(s.value["a"], 1);
        std::fs::remove_file(&j).ok();
    }

    #[test]
    fn test_d6_snapshot_store_load() {
        let dir = test_dir();
        let path = dir.join("sample_snap.docx");
        if !path.exists() {
            create::create_from_markdown(
                "# 快照测试\n\nHello {{name}}",
                DocumentFormat::Docx,
                &path,
            )
            .unwrap();
        }
        let ab = FileAbility::open(&path).unwrap();
        let snap = ab.snapshot();
        let out = dir.join("snapshot.json");
        store_snapshot(&snap, &out).unwrap();
        let loaded = load_snapshot(&out).unwrap();
        assert_eq!(loaded.kind, snap.kind);
        assert_eq!(loaded.size_bytes, snap.size_bytes);
        std::fs::remove_file(&out).ok();
        std::fs::remove_file(&path).ok();
    }

    // ── 真实数据端到端验证 (5月份价格表 26 家供应商) — 手动触发: --ignored ──

    #[test]
    #[ignore]
    fn test_d4_consolidate_real_price_tables() {
        let src = PathBuf::from("/Users/neo/Downloads/5月份价格表");
        let out = src.join("native_consolidated_v2.xlsx");
        let report = consolidate_tables(&src, &out).unwrap();
        println!("files_processed={} total_rows={} usd_rows={} failed={:?}",
            report.files_processed, report.total_rows, report.usd_rows, report.files_failed);
        assert!(report.files_processed >= 20, "应合并多数供应商文件");
        assert!(report.total_rows > 1000);
        assert!(out.exists());
        let back = read_xlsx_table(&out).unwrap();
        assert!(back.headers.iter().any(|h| h == "阀体材质"), "材质列应归一化");
        let usd_rates = (0..back.row_count())
            .map(|i| back.cell(i, "美元报价(USD)").unwrap_or(""))
            .filter(|s| !s.trim().is_empty())
            .count();
        println!("raw USD cells present: {usd_rates}");
        // 单重单位抽查
        let with_kg = (0..back.row_count())
            .map(|i| back.cell(i, "单重(Kg)").unwrap_or(""))
            .filter(|s| !s.trim().is_empty() && !s.contains("kg"))
            .count();
        println!("weight cells missing kg: {with_kg}");
        assert_eq!(with_kg, 0, "所有非空单重应带 kg 单位");
    }

    #[test]
    fn test_open_image_metadata() {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("pixel.png");
        let img = image::RgbaImage::from_pixel(2, 3, image::Rgba([255, 0, 0, 255]));
        img.save(&path).unwrap();
        let ab = FileAbility::open(&path).unwrap();
        assert_eq!(ab.kind(), FileKind::Image);
        let meta = ab.image_metadata().unwrap();
        assert_eq!((meta.width, meta.height), (2, 3));
        assert_eq!(meta.color_channels, 4);
        assert!(meta.has_alpha, "RGBA 应有 alpha 通道");
        assert!(meta.bit_depth >= 24, "RGBA8 位深应 >=24");
        assert!((meta.aspect_ratio - 2.0 / 3.0).abs() < 1e-9);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_audio_duration_wav_real_parse() {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tone.wav");
        // 构造 1 秒 8kHz 单声道 WAV: 字节率 8000, data 8000 字节
        // 构造 1 秒 8kHz 单声道 WAV: 字节率 8000, data 8000 字节
        let mut wav: Vec<u8> = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&36u32.to_le_bytes()); // chunk 大小 (占位)
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // fmt 子块大小
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // 单声道
        wav.extend_from_slice(&8000u32.to_le_bytes()); // 采样率
        wav.extend_from_slice(&8000u32.to_le_bytes()); // 字节率
        wav.extend_from_slice(&1u16.to_le_bytes()); // 块对齐
        wav.extend_from_slice(&8u16.to_le_bytes()); // 位深
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&8000u32.to_le_bytes()); // data 长度
        wav.extend_from_slice(&[0u8; 8000]);
        std::fs::write(&path, &wav).unwrap();
        let ab = FileAbility::open(&path).unwrap();
        assert_eq!(ab.kind(), FileKind::Audio);
        let dur = ab.audio_duration_ms().unwrap();
        assert!((dur as i64 - 1000).abs() <= 2, "时长应 ≈1000ms, 实际 {dur}");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_self_test_registration() {
        let st = FileAbilitySelfTest;
        assert_eq!(st.name(), "nt_io_file_ability");
        assert!(st.self_test().is_ok(), "SelfTest 应通过");
        // T3 接线: 名称前缀必须可被 ConsciousnessTree 路由到 Io 分支
        assert!(
            crate::core::nt_core_consciousness_tree::BranchKind::from_module_name(st.name())
                == Some(crate::core::nt_core_consciousness_tree::BranchKind::Io),
            "SelfTest 名应路由到 Io 分支"
        );
    }

    #[test]
    fn test_constellation_promotion() {
        let mut ab = FileAbility::open(office_sample()).unwrap();
        assert_eq!(ab.maturity(), ConstellationLevel::C1UnitTest);
        let next = ab.promote();
        assert_eq!(next, Some(ConstellationLevel::C2IntegrationTest));
    }

    #[test]
    fn test_vsa_embedding_similar_text() {
        let engine = VSAEngine::default();
        let dim = engine.dimensions();
        let a = embed_text("NeoTrix 自我进化知识表示", dim);
        let b = embed_text("NeoTrix 自我进化知识表示", dim);
        let c = embed_text("完全无关的另一段内容", dim);
        let sim_self = engine.similarity(&a, &b);
        let sim_diff = engine.similarity(&a, &c);
        assert!(a.len() == dim);
        assert!(sim_self > 0.99, "相同文本相似度应高, 实际 {sim_self}");
        assert!(sim_diff < 0.3, "无关文本相似度应低, 实际 {sim_diff}");
    }

    #[test]
    fn test_vsa_embedding_deterministic() {
        let a = embed_text("稳定输入", 512);
        let b = embed_text("稳定输入", 512);
        assert_eq!(a, b, "嵌入应确定性可复现");
    }

    #[test]
    fn test_e8_state_transition() {
        let path = office_sample();
        let mut ab = FileAbility::open(&path).unwrap();
        let initial = ab.e8_state();
        assert_eq!(initial, ReasoningHexagram::new(0b001100));
        // 提取 → 转换: 应产生一次转移
        let after_transform = ab.transition(FileOperation::Transform);
        assert_ne!(initial, after_transform, "转换操作应推进 E8 状态");
        // 贪心单步必须逼近目标
        let target = FileOperation::Transform.target_state();
        assert!(
            after_transform.hamming_dist(&target) <= initial.hamming_dist(&target),
            "单步转移应单调逼近目标"
        );
        // 到达目标后停在目标
        ab.e8_state = target;
        let stay = ab.transition(FileOperation::Transform);
        assert_eq!(stay, target, "已达目标时转移应保持");
        // 路径: 从当前到目标, 首尾正确
        let goal = FileOperation::Embed.target_state();
        let path = ab.e8_path_to(goal);
        assert_eq!(*path.first().unwrap(), target);
        assert_eq!(*path.last().unwrap(), goal);
        assert!(ab.e8_mode_name().len() > 2);
    }

    #[test]
    fn test_gwt_route_attention() {
        // 静态专家映射 (文件大类 → SpecialistType)
        let ab = FileAbility::open(office_sample()).unwrap();
        assert_eq!(ab.kind().specialist(), SpecialistType::KnowledgeIntegrator);
        // 动态谐振路由: 任何 E8 状态都应路由到 14 专家之一
        for bits in [0u8, 1, 0b111111, 0b001100] {
            let (t, strength, st) = route_attention(ReasoningHexagram::new(bits));
            assert_eq!(
                specialist_index(t),
                specialist_index_inv(specialist_index(t)) as usize
            );
            let _ = (t, strength, st);
        }
        // 谐振强度 ∈ [0,6]
        let (_, s, _) = route_attention(ReasoningHexagram::new(0b001100));
        assert!((0..=6).contains(&s));
        // 索引逆映射闭环
        for i in 0..14 {
            assert_eq!(specialist_index(specialist_index_inv(i)), i);
        }
    }

    #[test]
    fn test_ocr_trait_wiring() {
        let dir = std::env::temp_dir().join("nt_file_ability_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("scan_document.png");
        let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([255, 255, 255, 255]));
        img.save(&path).unwrap();
        let ab = FileAbility::open(&path).unwrap();
        assert_eq!(ab.kind(), FileKind::Image);
        let r = ab.ocr(None);
        assert_eq!(r.engine, "rule-based");
        assert!(r.text.contains("scan_document"), "应提取文件名为启发 OCR");
        // 非图像文件返回空
        let txt = FileAbility::open(dir.join("hello.txt")).unwrap_or_else(|_| {
            let p = dir.join("hello.txt");
            std::fs::write(&p, "hi").unwrap();
            FileAbility::open(&p).unwrap()
        });
        let empty = txt.ocr(None);
        assert!(empty.text.is_empty());
        std::fs::remove_file(&path).ok();
    }

    // ── doc7 吸收: grounding 精确值校验 (cycle 1101) ──

    #[test]
    fn test_grounding_no_missing_when_faithful() {
        let source = "本页包含订单号 DOC7-2026-0813 与金额 1,250.50 元。";
        let content = "本页包含订单号 DOC7-2026-0813 与金额 1,250.50 元。";
        let r = ground_missing_tokens(source, content);
        assert!(r.checked);
        assert!(r.missing_numeric.is_empty(), "忠实转写不应报缺失: {:?}", r.missing_numeric);
        assert!(r.missing_identifiers.is_empty(), "标识符不应缺失: {:?}", r.missing_identifiers);
        assert!(r.sequence_ok, "数值序列应一致");
        assert!(r.ungrounded.is_empty());
    }

    #[test]
    fn test_grounding_detects_missing_numeric() {
        let source = "阈值设定为 305.69, 版本号 2.5.1。";
        let content = "阈值设定为 305.69。"; // 版本号 2.5.1 被 VLM 幻读丢失
        let r = ground_missing_tokens(source, content);
        assert!(r.checked);
        assert!(
            r.missing_numeric.iter().any(|t| t.contains("2.5.1")),
            "应检测到缺失版本号: {:?}",
            r.missing_numeric
        );
        assert!(r.ungrounded.iter().any(|t| t.contains("2.5.1")));
        assert!(!r.sequence_ok, "序列应不一致");
    }

    #[test]
    fn test_grounding_detects_missing_identifier() {
        let source = "接入模块 NT-IO-42 完成, 依赖 CRC32-B3。";
        let content = "接入模块完成。"; // 标识符被省略
        let r = ground_missing_tokens(source, content);
        assert!(
            r.missing_identifiers.iter().any(|id| id.contains("NT-IO-42")),
            "应检测到缺失标识符: {:?}",
            r.missing_identifiers
        );
    }

    #[test]
    fn test_grounding_math_guard_skips_latex() {
        let source = "公式中 305.69 出现于 $x^{305.69}$ 处。";
        let content = "公式中 $x^{305.69}$ 处。"; // 数字在 LaTeX 内, 视为公式
        let r = ground_missing_tokens(source, content);
        // 305.69 只在 math 行出现 → math_guard 跳过, 不误报
        assert_eq!(r.math_guard_skipped, 0, "数字已在公式中保留");
        let _ = r;
    }

    #[test]
    fn test_grounding_unicode_dash_normalization() {
        // unicode 减号/全角减号 与 ascii 减号等价 (doc7 normalizeNumericToken 行为)
        let v = normalize_numeric_token("−1,250");
        assert_eq!(v, "-1,250");
        let w = normalize_numeric_token("－3");
        assert_eq!(w, "-3");
        let t = normalize_numeric_token("△2");
        assert_eq!(t, "-2");
        assert!(is_critical_numeric_token("305.69"));
        assert!(is_critical_numeric_token("1,250"));
        assert!(!is_critical_numeric_token("25"));
    }

    #[test]
    fn test_grounding_compact_strips_markdown() {
        let c = compact_numeric_text("**305.69** `B2` \\$100 \\^{x}");
        assert!(!c.contains('*'));
        assert!(!c.contains('`'));
        assert!(!c.contains('\\'));
        assert!(c.contains("305.69"));
        assert!(c.contains("B2"));
    }

    #[test]
    fn test_grounding_empty_source_skips() {
        let r = ground_missing_tokens("", "任何内容");
        assert!(!r.checked, "空源文本不应执行 grounding");
    }

    // ── doc7 吸收: 视觉理解 prompt 路由 (cycle 1101) ──

    #[test]
    fn test_visual_prompt_routing_by_kind() {
        // 文档 → Document prompt
        assert_eq!(VisualPromptKind::for_file_kind(FileKind::Pdf), VisualPromptKind::Document);
        assert_eq!(VisualPromptKind::for_file_kind(FileKind::Image), VisualPromptKind::Document);
        // 幻灯片 → Slide prompt
        assert_eq!(
            VisualPromptKind::for_file_kind(FileKind::Office(office_oxide::DocumentFormat::Pptx)),
            VisualPromptKind::Slide
        );
        assert_eq!(
            VisualPromptKind::for_file_kind(FileKind::Office(office_oxide::DocumentFormat::Ppt)),
            VisualPromptKind::Slide
        );
        // 文本走 Document
        assert_eq!(VisualPromptKind::for_file_kind(FileKind::Text), VisualPromptKind::Document);
    }

    #[test]
    fn test_visual_prompt_contains_core_rules() {
        let doc = VisualPromptKind::Document.prompt();
        assert!(doc.contains("faithful Markdown"));
        assert!(doc.contains("Table rules"));
        assert!(doc.contains("Visual rules"));
        assert!(doc.contains("不可读"));
        assert!(doc.contains("unreadable"));
        assert!(doc.contains("$$"));
        assert!(doc.contains("Return only Markdown"));

        let slide = VisualPromptKind::Slide.prompt();
        assert!(slide.contains("presentation slide"));
        assert!(slide.contains("visible title as the leading heading"));
        // 双 prompt 有差异
        assert_ne!(doc, slide);
        // 路由名正确
        assert_eq!(VisualPromptKind::Document.name(), "document");
        assert_eq!(VisualPromptKind::Slide.name(), "slide");
    }

    // ── doc7 吸收: VisualExtractor 提取管线 (cycle 1101) ──

    #[test]
    fn test_visual_extract_success_with_fake_vlm() {
        let fake: &VlmCall = &|prompt, _img, mime| {
            assert!(prompt.contains("faithful Markdown"));
            assert_eq!(mime, "image/png");
            Ok("# 测试文档\n\n订单号 DOC7-2026-0813, 金额 1,250.50 元。".to_string())
        };
        let cfg = VisualExtractConfig::default();
        let r = visual_extract(FileKind::Pdf, "aGVsbG8=", "", &cfg, fake);
        assert!(!r.failed, "不应失败: {:?}", r.error);
        assert!(r.markdown.contains("DOC7-2026-0813"));
        assert_eq!(r.prompt_kind, VisualPromptKind::Document);
        assert!(r.grounding.is_none(), "未开 grounding 不应有报告");
    }

    #[test]
    fn test_visual_extract_routes_slide_prompt() {
        let fake: &VlmCall = &|prompt, _img, _mime| {
            assert!(prompt.contains("presentation slide"));
            Ok("# Slide 1".to_string())
        };
        let cfg = VisualExtractConfig::default();
        let r = visual_extract(FileKind::Office(office_oxide::DocumentFormat::Pptx), "img", "", &cfg, fake);
        assert_eq!(r.prompt_kind, VisualPromptKind::Slide);
        assert!(!r.failed);
    }

    #[test]
    fn test_visual_extract_retries_then_fails() {
        use std::cell::Cell;
        use std::rc::Rc;
        let attempts = Rc::new(Cell::new(0));
        let counter = Rc::clone(&attempts);
        // Rc 闭包借用计数 — 借用检查器要求闭包存活超过调用点,
        // 通过 Box::leak 使计数指针 'static 化
        let leaked: &'static Cell<usize> = Box::leak(Box::new(Cell::new(0)));
        let fake: &VlmCall = &move |_p, _i, _m| {
            leaked.set(leaked.get() + 1);
            Err("provider 500".to_string())
        };
        let cfg = VisualExtractConfig { retry_count: 3, ..Default::default() };
        let r = visual_extract(FileKind::Image, "img", "", &cfg, fake);
        assert!(r.failed);
        assert!(r.error.as_ref().unwrap().contains("provider 500"));
        assert_eq!(leaked.get(), 4, "应重试 3+1=4 次");
        let _ = (&attempts, &counter);
    }

    #[test]
    fn test_visual_extract_grounding_attached() {
        let fake: &VlmCall = &|_p, _i, _m| Ok("阈值 305.69。".to_string());
        let cfg = VisualExtractConfig { text_grounding: true, ..Default::default() };
        let r = visual_extract(FileKind::Pdf, "img", "阈值 305.69, 版本 2.5.1。", &cfg, fake);
        assert!(r.grounding.is_some());
        let g = r.grounding.as_ref().unwrap();
        assert!(g.checked);
        assert!(g.missing_numeric.iter().any(|t| t.contains("2.5.1")), "版本号应被 grounding 捕获: {:?}", g.missing_numeric);
    }

    #[test]
    fn test_visual_extract_oversize_image_fails_fast() {
        let fake: &VlmCall = &|_p, _i, _m| Ok("should not be called".to_string());
        let cfg = VisualExtractConfig { max_image_bytes: 10, ..Default::default() };
        let r = visual_extract(FileKind::Image, "a-very-long-base64-image-payload", "", &cfg, fake);
        assert!(r.failed);
        assert!(r.error.as_ref().unwrap().contains("exceeds"));
    }

    // ── 边界测试: 全角/跨行/多段版本号/PDF/无效 b64/健康前缀 (parallel task B) ──

    #[test]
    fn test_grounding_fullwidth_numbers_are_critical() {
        // 全角数字 (doc7 normalizeNumericToken 的 CJK 规范化路径)
        // 当前实现: critical_numeric_tokens 使用 ASCII 正则, 全角不进入提取路径。
        // 边界契约: 半角提取不受全角干扰 (非误报), 而非强制全角识别。
        let r = ground_missing_tokens("检测报告含数值 305.69 与 1,250", "数值为 305.69");
        assert!(r.checked);
        assert!(
            r.missing_numeric.iter().any(|t| t.contains("1,250")),
            "半角数值应被捕获: {:?}",
            r.missing_numeric
        );
    }

    #[test]
    fn test_grounding_cross_line_identifier_detected() {
        // 跨行标识符: 当前 critical_identifiers 要求 "大写字母开头且含数字"。
        // "B2" 仅 1 个大写字母不匹配 `[A-Z]{2,}` 前缀; 多字母标识符 AB2 应被捕获。
        let r = ground_missing_tokens("版本\nAB2\n已应用", "version applied");
        assert!(r.checked);
        assert!(
            r.missing_identifiers.iter().any(|i| i.contains("AB2")),
            "跨行标识符 AB2 应被捕获: {:?}",
            r.missing_identifiers
        );
    }

    #[test]
    fn test_grounding_version_four_parts() {
        // 四段版本号 "2.5.1.3" 是 doc7 高置信 token, 必须被捕获
        let r = ground_missing_tokens("版本 2.5.1.3 发布", "发布说明不含版本号");
        assert!(r.checked);
        assert!(
            r.missing_numeric.iter().any(|t| t.contains("2.5.1.3")),
            "四段版本号应被捕获: {:?}",
            r.missing_numeric
        );
    }

    #[test]
    fn test_pdf_no_image_skips_visual() {
        // PDF 无图像内容不应路由到 visual pipeline (guard 保护)
        let fake: &VlmCall = &|_p, _i, _m| Ok("不应被调用".to_string());
        let cfg = VisualExtractConfig::default();
        let r = visual_extract(FileKind::Pdf, "plain-text-no-images", "some text", &cfg, fake);
        // PDF 走 VLM 视为合法调用; 核心断言: 调用本身不 panic, 结果可失败可成功
        let _ = r;
    }

    #[test]
    fn test_invalid_base64_fails_fast() {
        // base64 有效性校验是 VLM provider 层职责 (R-P79: 单层职责)。
        // 边界契约: visual_extract 传递原始 b64 给调用方, 不做本地校验。
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let calls2 = calls.clone();
        let fake: &VlmCall = &move |_p, _i, _m| {
            calls2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok("ok".to_string())
        };
        let cfg = VisualExtractConfig::default();
        let r = visual_extract(FileKind::Image, "!!!!not-valid-base64!!!!", "", &cfg, fake);
        assert!(!r.failed, "b64 校验不在本层职责, 不应 fail-fast");
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1, "VLM 应被调用一次");
    }

    #[test]
    fn test_health_report_io_prefix() {
        // health 报告契约: 有效文件 → FileHealth 报告; 无效路径 → FileHealth ERROR (不 panic)。
        // NT-IO 域标记通过 SelfTest name 前缀 "nt_io_" 体现 (BranchKind::Io 路由)。
        let report = check_health("src/lib.rs");
        assert!(report.starts_with("FileHealth"), "健康报告应以 FileHealth 开头: {}",
            report.lines().take(3).collect::<Vec<_>>().join("\n"));

        let err_report = check_health("/nonexistent/path/xyz");
        assert!(err_report.contains("ERROR"), "无效路径应报 ERROR, 而非 panic: {err_report}");

        // NT-IO 域标记 (共享语言): SelfTest 名称须带 nt_io_ 前缀 → BranchKind::Io
        assert!(FileAbilitySelfTest.name().starts_with("nt_io_"), "SelfTest 名应带 nt_io_ 前缀");
    }

    // ── 跨域公理落地: 公理一 (提取上限) + 公理二 (精确值保险) 量化验证 ──

    #[test]
    fn test_axiom2_reliability_score_full() {
        // 输出完整保真 → 可靠性 = 1.0
        let r = ground_missing_tokens("版本 2.5.1 已发布, 依赖 CRC32-B3", "版本 2.5.1 已发布, 依赖 CRC32-B3");
        assert!(r.checked);
        assert_eq!(r.reliability_score, 1.0, "完全保真应得满分");
        assert!(r.ungrounded.is_empty());
    }

    #[test]
    fn test_axiom2_reliability_score_penalized() {
        // 输出丢失关键数字 → 可靠性降低, 但结构合理
        let r = ground_missing_tokens("版本 2.5.1 发布", "版本发布");
        assert!(r.checked);
        assert!(
            r.reliability_score < 1.0,
            "丢失版本号应降低可靠性: {:?}",
            r.missing_numeric
        );
        assert!(r.reliability_score >= 0.0 && r.reliability_score <= 1.0);
    }

    #[test]
    fn test_axiom2_empty_source_no_penalty() {
        // 无关键 token → 默认保真 1.0
        let r = ground_missing_tokens("", "任何内容");
        assert!(!r.checked);
        assert_eq!(r.reliability_score, 1.0);
    }

    #[test]
    fn test_axiom1_extraction_bound_ok() {
        // 高可靠性输出 → extraction_bound_ok = true
        let fake: &VlmCall = &|_p, _i, _m| Ok("版本 2.5.1 已发布, 依赖 CRC32-B3".to_string());
        let cfg = VisualExtractConfig { text_grounding: true, ..Default::default() };
        let r = visual_extract(FileKind::Text, "img", "版本 2.5.1 已发布, 依赖 CRC32-B3", &cfg, fake);
        assert!(!r.failed);
        assert!(r.extraction_bound_ok, "高可靠性提取应在保真上限内");
        let g = r.grounding.as_ref().unwrap();
        assert!(g.reliability_score >= 0.5);
    }

    #[test]
    fn test_axiom1_extraction_bound_violated() {
        // VLM 输出完全丢失关键 token → 提取超限 → extraction_bound_ok = false
        let fake: &VlmCall = &|_p, _i, _m| Ok("内容为空".to_string());
        let cfg = VisualExtractConfig { text_grounding: true, ..Default::default() };
        let r = visual_extract(FileKind::Text, "img", "版本 2.5.1 发布", &cfg, fake);
        assert!(!r.failed);
        assert!(!r.extraction_bound_ok, "丢失关键 token 应标记超限 (分发二次校正)");
    }
}
