//! FileAbility 核心句柄 — 探测/纯文本/Office 转换/占位符/图像/媒体/快照。

use std::path::{Path, PathBuf};

use image::GenericImageView;
use office_oxide::xlsx::{CellRef, CellValue};
use office_oxide::{Document, DocumentFormat};

use crate::core::nt_core_hex::ReasoningHexagram;
use nt_core_capability_tree::ConstellationLevel;

use super::tables::{read_csv, read_xlsx_sheets_all};
use super::types::{
    ContentSnapshot, FileAbilityError, FileKind, ImageMetadata, MediaMetadata, Result,
    SheetCellData, SheetCellValueType, SheetData, SheetRowData, TableData,
};

/// 统一文件能力句柄
pub struct FileAbility {
    /// 原始路径句柄 (指针守恒: 单一存储)
    pub(super) path: PathBuf,
    /// 文件大类探测结果
    pub(super) kind: FileKind,
    /// MIME 类型
    pub(super) mime_type: String,
    /// 文件大小 (字节)
    pub(super) size_bytes: u64,
    /// 是否已被消费者注册 (Dark Forest 生存标记)
    pub(super) has_consumers: bool,
    /// 能力成熟度 (复用能力树 ConstellationLevel, 不平行重造)
    pub(super) maturity: ConstellationLevel,
    /// 当前 E8 推理状态 (Ext-6: 操作驱动状态转移)
    pub(super) e8_state: ReasoningHexagram,
    /// Office 句柄缓存 (仅 Office 类文件)
    pub(super) doc: Option<Document>,
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
        // 表格提取 (office xlsx / csv 场景): 尝试读多 sheet 全量, 失败则 None
        let table = if self.kind.office() {
            self.read_table_snapshot()
        } else {
            None
        };
        ContentSnapshot {
            kind: self.kind,
            text,
            markdown,
            table,
            image,
            media,
            mime_type: self.mime_type.clone(),
            size_bytes: self.size_bytes,
        }
    }

    /// 从文件路径读取表格快照 (xlsx 多 sheet 全量 / csv 单表)。
    fn read_table_snapshot(&self) -> Option<Vec<TableData>> {
        let path = &self.path;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        match ext.as_str() {
            "xlsx" => read_xlsx_sheets_all(path).ok(),
            "csv" | "tsv" => read_csv(path).ok().map(|t| vec![t]),
            _ => None,
        }
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