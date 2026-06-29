use super::sense_modality::SenseModality;

/// Result of type detection — identifies what sense modality a byte stream belongs to.
#[derive(Debug, Clone)]
pub struct DetectedType {
    pub mime: String,
    pub extension: String,
    pub confidence: f64,
    pub modality: SenseModality,
}

impl DetectedType {
    pub fn unknown() -> Self {
        Self {
            mime: "application/octet-stream".into(),
            extension: "bin".into(),
            confidence: 0.1,
            modality: SenseModality::Olfactory,
        }
    }

    pub fn is_known(&self) -> bool {
        self.confidence > 0.3
    }

    pub fn text_plain() -> Self {
        Self {
            mime: "text/plain".into(),
            extension: "txt".into(),
            confidence: 0.9,
            modality: SenseModality::Mental,
        }
    }

    pub fn pdf() -> Self {
        Self {
            mime: "application/pdf".into(),
            extension: "pdf".into(),
            confidence: 0.95,
            modality: SenseModality::Visual,
        }
    }

    pub fn html() -> Self {
        Self {
            mime: "text/html".into(),
            extension: "html".into(),
            confidence: 0.9,
            modality: SenseModality::Visual,
        }
    }

    pub fn json() -> Self {
        Self {
            mime: "application/json".into(),
            extension: "json".into(),
            confidence: 0.85,
            modality: SenseModality::Mental,
        }
    }

    pub fn image() -> Self {
        Self {
            mime: "image/png".into(),
            extension: "png".into(),
            confidence: 0.95,
            modality: SenseModality::Visual,
        }
    }
}

struct MagicEntry {
    magic: &'static [u8],
    offset: usize,
    result: fn() -> DetectedType,
}

struct ExtEntry {
    ext: &'static str,
    result: fn() -> DetectedType,
}

/// Fast type detector that maps raw bytes → DetectedType via magic bytes + extension.
pub struct TypeDetector {
    magic: &'static [MagicEntry],
    ext_map: &'static [ExtEntry],
}

impl TypeDetector {
    pub fn new() -> Self {
        Self {
            magic: &MAGIC_DB,
            ext_map: &EXT_DB,
        }
    }

    pub fn detect(&self, data: &[u8], filename: Option<&str>) -> DetectedType {
        for entry in self.magic {
            if data.len() > entry.offset + entry.magic.len()
                && data[entry.offset..entry.offset + entry.magic.len()] == *entry.magic
            {
                return (entry.result)();
            }
        }

        if let Some(name) = filename {
            if let Some(dot) = name.rfind('.') {
                let ext = name[dot + 1..].to_lowercase();
                for entry in self.ext_map {
                    if entry.ext == ext {
                        return (entry.result)();
                    }
                }
            }
        }

        if std::str::from_utf8(data).is_ok() {
            return DetectedType::text_plain();
        }

        DetectedType::unknown()
    }
}

const MAGIC_DB: &[MagicEntry] = &[
    MagicEntry {
        magic: b"%PDF",
        offset: 0,
        result: DetectedType::pdf,
    },
    MagicEntry {
        magic: b"\x89PNG\r\n\x1a\n",
        offset: 0,
        result: DetectedType::image,
    },
    MagicEntry {
        magic: b"\xff\xd8\xff",
        offset: 0,
        result: || DetectedType {
            mime: "image/jpeg".into(),
            extension: "jpg".into(),
            confidence: 0.95,
            modality: SenseModality::Visual,
        },
    },
    MagicEntry {
        magic: b"GIF87a",
        offset: 0,
        result: || DetectedType {
            mime: "image/gif".into(),
            extension: "gif".into(),
            confidence: 0.95,
            modality: SenseModality::Visual,
        },
    },
    MagicEntry {
        magic: b"GIF89a",
        offset: 0,
        result: || DetectedType {
            mime: "image/gif".into(),
            extension: "gif".into(),
            confidence: 0.95,
            modality: SenseModality::Visual,
        },
    },
    MagicEntry {
        magic: b"<html",
        offset: 0,
        result: DetectedType::html,
    },
    MagicEntry {
        magic: b"<!DOCTYPE html",
        offset: 0,
        result: DetectedType::html,
    },
    MagicEntry {
        magic: b"<!DOCTYPE HTML",
        offset: 0,
        result: DetectedType::html,
    },
    MagicEntry {
        magic: b"<?xml",
        offset: 0,
        result: || DetectedType {
            mime: "text/xml".into(),
            extension: "xml".into(),
            confidence: 0.85,
            modality: SenseModality::Mental,
        },
    },
    MagicEntry {
        magic: b"{",
        offset: 0,
        result: DetectedType::json,
    },
    MagicEntry {
        magic: b"[",
        offset: 0,
        result: DetectedType::json,
    },
    MagicEntry {
        magic: b"#!",
        offset: 0,
        result: || DetectedType {
            mime: "text/x-script".into(),
            extension: "sh".into(),
            confidence: 0.8,
            modality: SenseModality::Mental,
        },
    },
];

const EXT_DB: &[ExtEntry] = &[
    ExtEntry {
        ext: "pdf",
        result: DetectedType::pdf,
    },
    ExtEntry {
        ext: "txt",
        result: DetectedType::text_plain,
    },
    ExtEntry {
        ext: "md",
        result: || DetectedType {
            mime: "text/markdown".into(),
            extension: "md".into(),
            confidence: 0.8,
            modality: SenseModality::Mental,
        },
    },
    ExtEntry {
        ext: "rs",
        result: || DetectedType {
            mime: "text/x-rust".into(),
            extension: "rs".into(),
            confidence: 0.8,
            modality: SenseModality::Mental,
        },
    },
    ExtEntry {
        ext: "py",
        result: || DetectedType {
            mime: "text/x-python".into(),
            extension: "py".into(),
            confidence: 0.8,
            modality: SenseModality::Mental,
        },
    },
    ExtEntry {
        ext: "js",
        result: || DetectedType {
            mime: "text/javascript".into(),
            extension: "js".into(),
            confidence: 0.8,
            modality: SenseModality::Mental,
        },
    },
    ExtEntry {
        ext: "ts",
        result: || DetectedType {
            mime: "text/typescript".into(),
            extension: "ts".into(),
            confidence: 0.8,
            modality: SenseModality::Mental,
        },
    },
    ExtEntry {
        ext: "json",
        result: DetectedType::json,
    },
    ExtEntry {
        ext: "html",
        result: DetectedType::html,
    },
    ExtEntry {
        ext: "htm",
        result: DetectedType::html,
    },
    ExtEntry {
        ext: "xml",
        result: || DetectedType {
            mime: "text/xml".into(),
            extension: "xml".into(),
            confidence: 0.8,
            modality: SenseModality::Mental,
        },
    },
    ExtEntry {
        ext: "csv",
        result: || DetectedType {
            mime: "text/csv".into(),
            extension: "csv".into(),
            confidence: 0.8,
            modality: SenseModality::Mental,
        },
    },
    ExtEntry {
        ext: "svg",
        result: || DetectedType {
            mime: "image/svg+xml".into(),
            extension: "svg".into(),
            confidence: 0.8,
            modality: SenseModality::Visual,
        },
    },
    ExtEntry {
        ext: "png",
        result: || DetectedType {
            mime: "image/png".into(),
            extension: "png".into(),
            confidence: 0.95,
            modality: SenseModality::Visual,
        },
    },
    ExtEntry {
        ext: "jpg",
        result: || DetectedType {
            mime: "image/jpeg".into(),
            extension: "jpg".into(),
            confidence: 0.95,
            modality: SenseModality::Visual,
        },
    },
    ExtEntry {
        ext: "jpeg",
        result: || DetectedType {
            mime: "image/jpeg".into(),
            extension: "jpeg".into(),
            confidence: 0.95,
            modality: SenseModality::Visual,
        },
    },
    ExtEntry {
        ext: "gif",
        result: || DetectedType {
            mime: "image/gif".into(),
            extension: "gif".into(),
            confidence: 0.95,
            modality: SenseModality::Visual,
        },
    },
    ExtEntry {
        ext: "zip",
        result: || DetectedType {
            mime: "application/zip".into(),
            extension: "zip".into(),
            confidence: 0.9,
            modality: SenseModality::Visual,
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pdf_by_magic() {
        let dt = TypeDetector::new();
        let result = dt.detect(b"%PDF-1.4\n...", None);
        assert_eq!(result.mime, "application/pdf");
        assert_eq!(result.modality, SenseModality::Visual);
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_detect_text_by_utf8() {
        let dt = TypeDetector::new();
        let result = dt.detect(b"hello world\nthis is text", None);
        assert_eq!(result.mime, "text/plain");
        assert_eq!(result.modality, SenseModality::Mental);
    }

    #[test]
    fn test_detect_by_extension() {
        let dt = TypeDetector::new();
        let result = dt.detect(b"blah blah", Some("main.rs"));
        assert_eq!(result.extension, "rs");
        assert_eq!(result.modality, SenseModality::Mental);
    }

    #[test]
    fn test_detect_unknown_binary() {
        let dt = TypeDetector::new();
        let data = &[0x00, 0x01, 0x02, 0xDE, 0xAD, 0xBE, 0xEF];
        let result = dt.detect(data, None);
        assert_eq!(result.modality, SenseModality::Olfactory);
        assert!(!result.is_known());
    }

    #[test]
    fn test_detect_html() {
        let dt = TypeDetector::new();
        let result = dt.detect(b"<html><body>hello</body></html>", None);
        assert_eq!(result.mime, "text/html");
        assert_eq!(result.modality, SenseModality::Visual);
    }

    #[test]
    fn test_detect_json() {
        let dt = TypeDetector::new();
        let result = dt.detect(b"{\"key\": \"value\"}", None);
        assert_eq!(result.mime, "application/json");
    }
}
