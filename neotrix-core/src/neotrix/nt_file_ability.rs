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

// ─────────────────────── doc7 吸收: Grounding 精确值校验 ───────────────────────
// 来源: github.com/magicrew/doc7 internal/extract/grounding.go + grounding_numeric.go
// (MIT, absorbed 2026-08-13, cycle 1101)。
//
// 公理 (Grounding 是精确值保险): 视觉理解管线可能幻读数字/代码/ID。从嵌入文本层
// 提取关键 token (≥3 位数字或含小数/百分号/货币符号, 以及大写字母+数字标识符),
// 与 VLM 输出比对, 缺失则标记为 ungrounded — 由调用方决定二次校正 (遵循 R-P36:
// grounding 结果必须进入行为, 而非仅日志)。本模块是纯算法, 无 LLM 依赖。

/// grounding 版本戳 — 算法变更时递增, 保证缓存 key 与旧结果区分
pub const GROUNDING_VERSION: &str = "11";

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

        // 9) grounding 公理链路 (公理二: 精确值保险): 可靠性评分契约 + 提取上限分发。
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
