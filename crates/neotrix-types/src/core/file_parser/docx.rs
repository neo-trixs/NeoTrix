use std::io::{Cursor, Read};
use zip::ZipArchive;

use super::{FileParser, extract_xml_text};

impl FileParser {
    pub(super) fn extract_docx_text(data: &[u8]) -> String {
        let mut all = String::new();
        let cursor = Cursor::new(data);
        let mut archive: ZipArchive<Cursor<&[u8]>> = match ZipArchive::new(cursor) {
            Ok(a) => a,
            Err(_) => return all,
        };
        let paths = ["word/document.xml", "word/header1.xml", "word/footer1.xml"];
        for path in &paths {
            if let Ok(mut file) = archive.by_name(path) {
                let mut buf = Vec::new();
                if file.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                    let text = extract_xml_text(&buf, &[b"w:t"]);
                    if !text.is_empty() {
                        all.push_str(&text);
                        all.push('\n');
                    }
                }
            }
        }
        all.trim().to_string()
    }
}
