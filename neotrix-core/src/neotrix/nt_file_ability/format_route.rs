//! NT-WORLD 文档格式路由 (吸收 `firecrawl/anydoc` + `pdf-inspector`):
//! 按扩展名/内容探测文档格式并路由到对应解析器。R-P42 强化 `nt_file_ability`
//! (复用既有 Office6/PDF/图像探测路径), 不平行重造格式枚举。

use super::selftest::SelfTest;
use std::path::Path;

/// 文档格式 — 路由目标。**本仓唯一的本地格式分类真源**。
///
/// 2026-09-29 统一：此前存在三套格式枚举，各说各话 ——
/// - 本枚举（11 变体）：`format_route.rs`，**零生产调用**
/// - `book_to_skill::DocFormat`（7 变体：Pdf/Epub/Docx/Markdown/Html/Rtf/Mobi）：
///   技能管线自用，不调本模块
/// - `anydoc::Format`（12 变体）：解析后端，含内容嗅探
/// 本枚举现扩展为**全集**（anydoc 12 + 本地媒体/文本 6 + 遗留办公 3），并提供
/// 到 `anydoc::Format` 的**单向**转换（`to_anydoc`）。方向只许单向：
/// 本地枚举 → 后端枚举，反向禁止（后端变体增减不应回灌本地路由）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocFormat {
    // anydoc 直达（有对应后端解析器）
    Docx,
    Doc,
    Xlsx,
    Xls,
    Pptx,
    Ppt,
    Pdf,
    Odt,
    Ods,
    Odp,
    Rtf,
    Epub,
    Csv,
    // 本地直读（纯文本系，无需后端）
    Markdown,
    Html,
    Text,
    // 元数据模式（只取元信息，不做内容解析）
    Image,
    Audio,
    Video,
    Mobi,
    Unknown,
}

impl DocFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocFormat::Docx => "docx",
            DocFormat::Doc => "doc",
            DocFormat::Xlsx => "xlsx",
            DocFormat::Xls => "xls",
            DocFormat::Pptx => "pptx",
            DocFormat::Ppt => "ppt",
            DocFormat::Pdf => "pdf",
            DocFormat::Odt => "odt",
            DocFormat::Ods => "ods",
            DocFormat::Odp => "odp",
            DocFormat::Rtf => "rtf",
            DocFormat::Epub => "epub",
            DocFormat::Csv => "csv",
            DocFormat::Markdown => "markdown",
            DocFormat::Html => "html",
            DocFormat::Text => "text",
            DocFormat::Image => "image",
            DocFormat::Audio => "audio",
            DocFormat::Video => "video",
            DocFormat::Mobi => "mobi",
            DocFormat::Unknown => "unknown",
        }
    }

    /// 扩展名 → 格式（全集映射，含遗留办公与电子书）。
    pub fn from_ext(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "docx" | "docm" => DocFormat::Docx,
            "doc" => DocFormat::Doc,
            "xlsx" | "xlsm" | "xlsb" => DocFormat::Xlsx,
            "xls" => DocFormat::Xls,
            "pptx" | "pptm" | "ppsx" | "ppsm" => DocFormat::Pptx,
            "ppt" | "pps" | "pot" => DocFormat::Ppt,
            "pdf" => DocFormat::Pdf,
            "odt" => DocFormat::Odt,
            "ods" => DocFormat::Ods,
            "odp" => DocFormat::Odp,
            "rtf" => DocFormat::Rtf,
            "epub" => DocFormat::Epub,
            "csv" => DocFormat::Csv,
            "md" | "markdown" => DocFormat::Markdown,
            "html" | "htm" => DocFormat::Html,
            "txt" | "text" | "log" => DocFormat::Text,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "tiff" => DocFormat::Image,
            "mp3" | "wav" | "ogg" | "flac" | "m4a" => DocFormat::Audio,
            "mp4" | "webm" | "mov" | "mkv" | "avi" => DocFormat::Video,
            "mobi" | "azw" | "azw3" => DocFormat::Mobi,
            _ => DocFormat::Unknown,
        }
    }

    /// 到 anydoc 后端格式的**单向**转换。`None` = 后端无此解析器，
    /// 调用方必须走本地路径（纯文本直读 / 元数据模式），不得 panic。
    pub fn to_anydoc(self) -> Option<anydoc::Format> {
        match self {
            DocFormat::Docx => Some(anydoc::Format::Docx),
            DocFormat::Doc => Some(anydoc::Format::Doc),
            DocFormat::Xlsx => Some(anydoc::Format::Excel),
            DocFormat::Xls => Some(anydoc::Format::Excel),
            DocFormat::Pptx => Some(anydoc::Format::Pptx),
            DocFormat::Ppt => Some(anydoc::Format::Ppt),
            DocFormat::Pdf => Some(anydoc::Format::Pdf),
            DocFormat::Odt => Some(anydoc::Format::Odt),
            DocFormat::Ods => Some(anydoc::Format::Ods),
            DocFormat::Odp => Some(anydoc::Format::Odp),
            DocFormat::Rtf => Some(anydoc::Format::Rtf),
            DocFormat::Epub => Some(anydoc::Format::Epub),
            DocFormat::Csv => Some(anydoc::Format::Csv),
            _ => None,
        }
    }

    /// 纯文本系：可直接按 UTF-8 读入，无需后端解析器。
    pub fn is_text_like(self) -> bool {
        matches!(
            self,
            DocFormat::Markdown | DocFormat::Html | DocFormat::Text | DocFormat::Csv
        )
    }

    /// 媒体系：只取元信息（尺寸/时长/编码），不做内容解析。
    pub fn is_media(self) -> bool {
        matches!(
            self,
            DocFormat::Image | DocFormat::Audio | DocFormat::Video | DocFormat::Mobi
        )
    }
}

/// 探测路径对应的文档格式 (扩展名优先; 无扩展名回退 Unknown)。
pub fn classify_format(path: &str) -> DocFormat {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    DocFormat::from_ext(&ext)
}

/// NT-WORLD 文档格式路由自测 (卫生层 P0)。
pub struct FormatRouteSelfTest;

impl SelfTest for FormatRouteSelfTest {
    fn name(&self) -> &str {
        "nt_world_file_format_route"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let cases = [
            ("a.docx", DocFormat::Docx),
            ("b.pdf", DocFormat::Pdf),
            ("c.md", DocFormat::Markdown),
            ("d.png", DocFormat::Image),
            ("e.mp4", DocFormat::Video),
            ("f.xyz", DocFormat::Unknown),
        ];
        for (p, want) in cases {
            if classify_format(p) != want {
                return Err(vec![format!(
                    "{} => {:?}, want {:?}",
                    p,
                    classify_format(p),
                    want
                )]);
            }
        }
        Ok(())
    }
}

struct CoreFormatRouteBridge;
impl crate::l6_meta::healing::nt_core_self_test::SelfTest for CoreFormatRouteBridge {
    fn name(&self) -> &str { "nt_world_file_format_route" }
    fn self_test(&self) -> Result<(), Vec<String>> { FormatRouteSelfTest.self_test() }
}

/// 注册文档格式路由 SelfTest 到核心全局注册表 (T2)。
pub fn register_format_route_self_tests(registry: &mut crate::l6_meta::healing::nt_core_self_test::SelfTestRegistry) {
    registry.register(Box::new(CoreFormatRouteBridge));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_route_self_test_passes() {
        assert!(
            FormatRouteSelfTest.self_test().is_ok(),
            "format route self_test failed: {:?}",
            FormatRouteSelfTest.self_test().err()
        );
    }

    #[test]
    fn test_ext_covers_full_enum() {
        // 之前 book_to_skill 有 Epub/Mobi 而本枚举没有 —— 那正是「两套格式
        // 枚举各说各话」的证据。现在每个变体都必须有扩展名能命中它。
        for (ext, want) in [
            ("pdf", DocFormat::Pdf),
            ("epub", DocFormat::Epub),
            ("docx", DocFormat::Docx),
            ("DOCX", DocFormat::Docx),
            ("doc", DocFormat::Doc),
            ("xlsx", DocFormat::Xlsx),
            ("xls", DocFormat::Xls),
            ("pptx", DocFormat::Pptx),
            ("ppt", DocFormat::Ppt),
            ("odt", DocFormat::Odt),
            ("ods", DocFormat::Ods),
            ("odp", DocFormat::Odp),
            ("rtf", DocFormat::Rtf),
            ("csv", DocFormat::Csv),
            ("md", DocFormat::Markdown),
            ("markdown", DocFormat::Markdown),
            ("html", DocFormat::Html),
            ("txt", DocFormat::Text),
            ("png", DocFormat::Image),
            ("mp3", DocFormat::Audio),
            ("mp4", DocFormat::Video),
            ("mobi", DocFormat::Mobi),
            ("azw3", DocFormat::Mobi),
            ("zzz", DocFormat::Unknown),
        ] {
            assert_eq!(DocFormat::from_ext(ext), want, "ext={ext}");
        }
    }

    #[test]
    fn test_to_anydoc_is_total_and_one_way() {
        // 有后端的必须映射到 anydoc；纯文本/媒体/未知必须返回 None
        // （调用方据此走本地路径，不得 panic）。
        for f in [
            DocFormat::Docx,
            DocFormat::Doc,
            DocFormat::Xlsx,
            DocFormat::Xls,
            DocFormat::Pptx,
            DocFormat::Ppt,
            DocFormat::Pdf,
            DocFormat::Odt,
            DocFormat::Ods,
            DocFormat::Odp,
            DocFormat::Rtf,
            DocFormat::Epub,
            DocFormat::Csv,
        ] {
            assert!(f.to_anydoc().is_some(), "{} 缺后端映射", f.as_str());
        }
        for f in [
            DocFormat::Markdown,
            DocFormat::Html,
            DocFormat::Text,
            DocFormat::Image,
            DocFormat::Audio,
            DocFormat::Video,
            DocFormat::Mobi,
            DocFormat::Unknown,
        ] {
            assert!(f.to_anydoc().is_none(), "{} 不该有后端映射", f.as_str());
        }
    }

    #[test]
    fn test_text_like_and_media_partition() {
        for f in [DocFormat::Markdown, DocFormat::Html, DocFormat::Text, DocFormat::Csv] {
            assert!(f.is_text_like() && !f.is_media(), "{} 应是纯文本系", f.as_str());
        }
        for f in [DocFormat::Image, DocFormat::Audio, DocFormat::Video, DocFormat::Mobi] {
            assert!(f.is_media() && !f.is_text_like(), "{} 应是媒体系", f.as_str());
        }
        // 三类互斥且穷尽（后端系两类都是 false）
        for f in [DocFormat::Docx, DocFormat::Pdf, DocFormat::Epub] {
            assert!(!f.is_text_like() && !f.is_media(), "{} 应属后端系", f.as_str());
        }
    }

    #[test]
    fn test_as_str_roundtrip_for_ext_backed() {
        for f in [
            DocFormat::Docx,
            DocFormat::Xlsx,
            DocFormat::Pptx,
            DocFormat::Pdf,
            DocFormat::Odt,
            DocFormat::Rtf,
            DocFormat::Epub,
            DocFormat::Csv,
        ] {
            assert_eq!(DocFormat::from_ext(f.as_str()), f, "{} 往返失真", f.as_str());
        }
    }
}
