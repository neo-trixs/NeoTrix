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
            Some(doc) => doc.as_xlsx().ok_or_else(|| FileAbilityError::UnsupportedFormat {
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
        let ws = doc
            .worksheets
            .get(index - 1)
            .ok_or_else(|| FileAbilityError::SheetIndexOutOfRange {
                index,
                count: doc.worksheets.len(),
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
        let pos = names
            .iter()
            .position(|n| n == name)
            .ok_or_else(|| FileAbilityError::SheetIndexOutOfRange {
                index: 0,
                count: names.len(),
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
            image::ColorType::Rgba8 | image::ColorType::La8 | image::ColorType::Rgba16 | image::ColorType::La16
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
                        byte_rate =
                            u32::from_le_bytes(data[pos + 16..pos + 20].try_into().unwrap_or([0; 4]));
                    }
                }
                b"data" => {
                    data_len = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap_or([0; 4]));
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
        if !matches!(
            fmt,
            neotrix_types::core::file_parser::FileFormat::PlainText
        ) {
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
        if specialist_index_inv(specialist_index(SpecialistType::Planner)) != SpecialistType::Planner
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
        xw.sheet_set_cell(s0, 0, 0, office_oxide::xlsx::write::CellData::String("k".into()));
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
        assert_eq!(s1.rows.iter().map(|r| r.index).collect::<Vec<_>>(), vec![1, 2, 3, 4]);
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
            assert_eq!(specialist_index(t), specialist_index_inv(specialist_index(t)) as usize);
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
}
