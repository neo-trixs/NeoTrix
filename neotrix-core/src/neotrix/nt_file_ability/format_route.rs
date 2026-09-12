//! NT-WORLD 文档格式路由 (吸收 `firecrawl/anydoc` + `pdf-inspector`):
//! 按扩展名/内容探测文档格式并路由到对应解析器。R-P42 强化 `nt_file_ability`
//! (复用既有 Office6/PDF/图像探测路径), 不平行重造格式枚举。

use super::selftest::{SelfTest, SelfTestRegistry};
use std::path::Path;

/// 文档格式 — 路由目标。覆盖 anydoc 支持的多格式 + 通用类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocFormat {
    Docx,
    Xlsx,
    Pptx,
    Pdf,
    Markdown,
    Html,
    Text,
    Image,
    Audio,
    Video,
    Unknown,
}

impl DocFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocFormat::Docx => "docx",
            DocFormat::Xlsx => "xlsx",
            DocFormat::Pptx => "pptx",
            DocFormat::Pdf => "pdf",
            DocFormat::Markdown => "markdown",
            DocFormat::Html => "html",
            DocFormat::Text => "text",
            DocFormat::Image => "image",
            DocFormat::Audio => "audio",
            DocFormat::Video => "video",
            DocFormat::Unknown => "unknown",
        }
    }

    /// 扩展名 → 格式 (anydoc 多格式映射)。
    pub fn from_ext(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "docx" | "doc" => DocFormat::Docx,
            "xlsx" | "xls" => DocFormat::Xlsx,
            "pptx" | "ppt" => DocFormat::Pptx,
            "pdf" => DocFormat::Pdf,
            "md" | "markdown" => DocFormat::Markdown,
            "html" | "htm" => DocFormat::Html,
            "txt" | "text" => DocFormat::Text,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => DocFormat::Image,
            "mp3" | "wav" | "ogg" | "flac" => DocFormat::Audio,
            "mp4" | "webm" | "mov" | "mkv" => DocFormat::Video,
            _ => DocFormat::Unknown,
        }
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

/// 注册文档格式路由 SelfTest 到全局注册表 (T2)。
pub fn register_format_route_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(FormatRouteSelfTest));
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
}
