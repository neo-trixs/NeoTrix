use std::io::{Cursor, Read};
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use super::FileParser;

/// 通用 XML 文本提取（从指定标签名中提取文本内容）
pub(crate) fn extract_xml_text(xml: &[u8], tag_names: &[&[u8]]) -> String {
    let s = match std::str::from_utf8(xml) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let mut reader = Reader::from_str(s);
    let mut buf = Vec::new();
    let mut in_target = false;
    let mut result = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let nm = e.name();
                let name_bytes = nm.as_ref();
                if tag_names.contains(&name_bytes) {
                    in_target = true;
                }
            }
            Ok(Event::Text(ref e)) if in_target => {
                if let Ok(t) = e.unescape() {
                    let trimmed: &str = t.as_ref().trim();
                    if !trimmed.is_empty() {
                        result.push_str(trimmed);
                        result.push(' ');
                    }
                }
                in_target = false;
            }
            Ok(Event::CData(ref e)) if in_target => {
                if let Ok(t) = std::str::from_utf8(e.as_ref()) {
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        result.push_str(trimmed);
                        result.push(' ');
                    }
                }
                in_target = false;
            }
            Ok(Event::End(ref e)) => {
                let nm = e.name();
                let name_bytes = nm.as_ref();
                if tag_names.contains(&name_bytes) {
                    in_target = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    result.trim().to_string()
}

impl FileParser {
    pub(super) fn extract_xlsx_text(data: &[u8]) -> String {
        let mut all = String::new();
        let cursor = Cursor::new(data);
        let mut archive: ZipArchive<Cursor<&[u8]>> = match ZipArchive::new(cursor) {
            Ok(a) => a,
            Err(_) => return all,
        };

        if let Ok(mut file) = archive.by_name("xl/sharedStrings.xml") {
            let mut buf = Vec::new();
            if file.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                let text = extract_xml_text(&buf, &[b"t"]);
                if !text.is_empty() {
                    all.push_str("[Shared Strings]\n");
                    all.push_str(&text);
                    all.push('\n');
                }
            }
        }

        for i in 1..=100 {
            let path = format!("xl/worksheets/sheet{i}.xml");
            let Ok(mut file) = archive.by_name(&path) else { break };
            let mut buf = Vec::new();
            if file.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                let text = extract_xml_text(&buf, &[b"v", b"t"]);
                if !text.is_empty() {
                    all.push_str(&format!("[Sheet {}]\n", i));
                    all.push_str(&text);
                    all.push('\n');
                }
            }
        }
        all.trim().to_string()
    }

    pub(super) fn extract_pptx_text(data: &[u8]) -> String {
        let mut all = String::new();
        let cursor = Cursor::new(data);
        let mut archive: ZipArchive<Cursor<&[u8]>> = match ZipArchive::new(cursor) {
            Ok(a) => a,
            Err(_) => return all,
        };

        let file_names: Vec<String> = (0..archive.len())
            .filter_map(|i| {
                let file = archive.by_index(i).ok()?;
                let name = file.name().to_string();
                if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();

        for path in file_names {
            if let Ok(mut file) = archive.by_name(&path) {
                let mut buf = Vec::new();
                if file.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                    let text = extract_xml_text(&buf, &[b"a:t"]);
                    if !text.is_empty() {
                        let slide_num = path
                            .trim_start_matches("ppt/slides/slide")
                            .trim_end_matches(".xml");
                        all.push_str(&format!("[Slide {}]\n", slide_num));
                        all.push_str(&text);
                        all.push('\n');
                    }
                }
            }
        }
        all.trim().to_string()
    }
}
