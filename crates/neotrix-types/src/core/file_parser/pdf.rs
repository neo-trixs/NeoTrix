use super::{FileParser, SpatialBlock, BlockType};

impl FileParser {
    /// PDF 文本提取 — 首选 lopdf 完整解析 (支持 FlateDecode 压缩流 / TJ 数组 / 字体映射),
    /// 失败或空结果时回退到朴素正则提取 (未压缩内容流)。
    pub(super) fn extract_pdf_text(data: &[u8]) -> String {
        if let Ok(text) = Self::extract_pdf_text_lopdf(data) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        let blocks = Self::extract_pdf_spatial(data);
        blocks.iter().map(|b| b.text.as_str()).collect::<Vec<_>>().join(" ")
    }

    /// lopdf 完整解析路径: 加载全部对象图并按页提取文本。
    fn extract_pdf_text_lopdf(data: &[u8]) -> std::result::Result<String, lopdf::Error> {
        let doc = lopdf::Document::load_mem(data)?;
        let page_nums: Vec<u32> = doc.get_pages().keys().copied().collect();
        if page_nums.is_empty() {
            return Ok(String::new());
        }
        doc.extract_text(&page_nums)
    }

    pub(super) fn extract_pdf_spatial(data: &[u8]) -> Vec<SpatialBlock> {
        let content = String::from_utf8_lossy(data);
        let mut blocks = Vec::new();
        let mut cur_x = 0.0f32;
        let mut cur_y = 0.0f32;
        let mut in_text = false;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("BT") {
                in_text = true;
                cur_x = 0.0;
                cur_y = 0.0;
                continue;
            }
            if line.starts_with("ET") {
                in_text = false;
                continue;
            }
            if !in_text {
                continue;
            }

            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            for (i, token) in tokens.iter().enumerate() {
                match *token {
                    "Tm" if i >= 6 => {
                        let e = tokens[i - 2].parse::<f32>().unwrap_or(cur_x);
                        let f = tokens[i - 1].parse::<f32>().unwrap_or(cur_y);
                        cur_x = e;
                        cur_y = f;
                    }
                    "Td" if i >= 2 => {
                        let tx = tokens[i - 2].parse::<f32>().unwrap_or(0.0);
                        let ty = tokens[i - 1].parse::<f32>().unwrap_or(0.0);
                        cur_x += tx;
                        cur_y += ty;
                    }
                    "T*" => {
                        cur_y -= 14.0;
                    }
                    _ => {}
                }
            }

            if let Some(start) = line.find('(') {
                if let Some(end) = line.rfind(')') {
                    if start < end && line[end..].contains("Tj") {
                        let text = &line[start + 1..end];
                        if !text.is_empty() && text.chars().any(|c| c.is_alphabetic()) {
                            blocks.push(SpatialBlock {
                                x: cur_x,
                                y: cur_y,
                                width: text.len() as f32 * 5.0,
                                height: 12.0,
                                text: text.to_string(),
                                block_type: BlockType::TextBlock,
                            });
                        }
                    }
                }
            }
        }

        if blocks.is_empty() {
            for line in content.lines() {
                let line = line.trim();
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.rfind(')') {
                        if start < end && (start == 0 || !line[..start].contains('\\')) {
                            let text = &line[start + 1..end];
                            if text.len() > 3 && text.chars().filter(|&c| c.is_alphabetic()).count() > 3 {
                                blocks.push(SpatialBlock {
                                    x: 0.0,
                                    y: 0.0,
                                    width: text.len() as f32 * 5.0,
                                    height: 12.0,
                                    text: text.to_string(),
                                    block_type: BlockType::TextBlock,
                                });
                            }
                        }
                    }
                }
            }
        }

        blocks.sort_by(|a, b| {
            b.y.partial_cmp(&a.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.x.partial_cmp(&b.x)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::content::{Content, Operation};
    use lopdf::dictionary;

    /// 生成 FlateDecode 压缩内容流的最小 PDF (lopdf), 验证完整解析路径
    /// 而非正则回退 — 正则无法读取压缩流, 若回退则断言失败。
    fn compressed_pdf_bytes() -> Vec<u8> {
        let mut doc = lopdf::Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Courier",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 48.into()]),
                Operation::new("Td", vec![100.into(), 600.into()]),
                Operation::new(
                    "Tj",
                    vec![lopdf::Object::string_literal("NeoTrix PDF Extract")],
                ),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id =
            doc.add_object(lopdf::Stream::new(lopdf::Dictionary::new(), content.encode().unwrap()));
        let page = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page.into()],
            "Count" => 1,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        };
        doc.objects.insert(pages_id, lopdf::Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        doc.compress();
        let mut buf = Vec::new();
        doc.save_to(&mut buf).unwrap();
        buf
    }

    #[test]
    fn extract_pdf_text_parses_compressed_content_stream() {
        let buf = compressed_pdf_bytes();
        // 确认确实走压缩 (正则回退会因找不到 BT/Tj 返回空)
        let text = FileParser::extract_pdf_text(&buf);
        assert!(
            text.contains("NeoTrix PDF Extract"),
            "lopdf 压缩流解析失败, 得到: {text:?}"
        );
    }

    #[test]
    fn extract_pdf_text_falls_back_to_spatial_on_garbage() {
        // 非 PDF 字节: lopdf 失败 → 正则回退为空 (不 panic, 不泄漏)
        let text = FileParser::extract_pdf_text(b"%PDF-1.7 junk not a real pdf");
        let _ = text;
    }
}
