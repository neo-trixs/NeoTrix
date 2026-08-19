use super::{FileParser, SpatialBlock, BlockType};

/// 判断字符串是否为可读 PDF 文本 (而非 FlateDecode 压缩数据的伪匹配)。
/// 压缩流乱码特征: 高控制字符密度 / 低可打印比例 / 无词边界。
fn is_readable_pdf_text(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let total = s.chars().count();
    if total < 2 {
        return false;
    }
    let (readable, control, spaces) = s.chars().fold((0usize, 0usize, 0usize), |(r, c, sp), ch| {
        if ch.is_alphabetic() || ch.is_ascii_digit() || ch.is_whitespace() || ch.is_ascii_punctuation() {
            (r + 1, c, sp + usize::from(ch.is_whitespace()))
        } else {
            (r, c + 1, sp)
        }
    });
    let readable_ratio = readable as f64 / total as f64;
    let control_ratio = control as f64 / total as f64;
    let space_ratio = spaces as f64 / total as f64;
    // 可读字符占比 ≥ 80% 且控制字符 ≤ 10%: 判为真实文本
    if readable_ratio < 0.8 || control_ratio > 0.1 {
        return false;
    }
    // 长文本 (≥16 字符) 需存在词边界 (空白 ≥ 2%) 排除压缩乱码的无空格长串;
    // 短文本 (单字/短词) 是可接受布局块, 不做词边界检查
    if total >= 16 && space_ratio < 0.02 {
        return false;
    }
    true
}

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

    /// 分页 PDF 文本提取 (公开生产路径, 供 CLI/example/消费方按页处理)。
    /// 走 lopdf 完整解析 (压缩流/ToUnicode/TJ), 逐页提取, 失败页跳过不 panic。
    /// 返回 (页号 1-based, 该页文本), 保持文档页序。
    ///
    /// 注意: lopdf `get_pages()` 的 key 是页序号 (1-based), `extract_text_chunks`
    /// 同样接收页序号而非对象引用号; 且一页可能产生多个 chunk (按字体编码切分),
    /// 必须逐页调用并拼接该页全部 chunk, 不能把扁平 chunk 列表与页号 zip 配对。
    pub fn extract_pdf_pages(data: &[u8]) -> Vec<(u32, String)> {
        let Ok(doc) = lopdf::Document::load_mem(data) else {
            return Vec::new();
        };
        let page_indices: Vec<u32> = doc.get_pages().keys().copied().collect();
        if page_indices.is_empty() {
            return Vec::new();
        }
        let mut result = Vec::new();
        for page_index in page_indices {
            let chunks = doc.extract_text_chunks(&[page_index]);
            let mut page_text = String::new();
            for chunk in chunks {
                match chunk {
                    Ok(text) => page_text.push_str(text.trim()),
                    Err(_) => continue,
                }
            }
            if !page_text.is_empty() {
                result.push((page_index, page_text));
            }
        }
        result
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
                        if is_readable_pdf_text(text) {
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
                            if is_readable_pdf_text(text) {
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
    /// 每页一行文本: "NeoTrix PDF Extract Page N"
    fn compressed_pdf_bytes() -> Vec<u8> {
        multi_page_pdf_bytes(&["NeoTrix PDF Extract"])
    }

    /// 多页压缩内容流 PDF: texts[i] 落第 i 页 (1-based)。
    fn multi_page_pdf_bytes(texts: &[&str]) -> Vec<u8> {
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
        let mut kids: Vec<lopdf::Object> = Vec::new();
        for (i, text) in texts.iter().enumerate() {
            let content = Content {
                operations: vec![
                    Operation::new("BT", vec![]),
                    Operation::new("Tf", vec!["F1".into(), 48.into()]),
                    Operation::new("Td", vec![100.into(), 600.into()]),
                    Operation::new(
                        "Tj",
                        vec![lopdf::Object::string_literal(format!("{text} Page {}", i + 1))],
                    ),
                    Operation::new("ET", vec![]),
                ],
            };
            let content_id = doc.add_object(lopdf::Stream::new(
                lopdf::Dictionary::new(),
                content
                    .encode()
                    .expect("encode PDF content stream for test"),
            ));
            let page = doc.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
                "Contents" => content_id,
            });
            kids.push(page.into());
        }
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => texts.len() as u32,
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
        doc.save_to(&mut buf).expect("save test PDF to buffer");
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

    #[test]
    fn extract_pdf_pages_returns_ordered_page_text() {
        let buf = multi_page_pdf_bytes(&["Alpha", "Beta", "Gamma"]);
        let pages = FileParser::extract_pdf_pages(&buf);
        assert_eq!(pages.len(), 3, "应返回 3 页, 得到 {pages:?}");
        assert_eq!(pages[0].0, 1);
        assert_eq!(pages[0].1, "Alpha Page 1");
        assert_eq!(pages[1].0, 2);
        assert_eq!(pages[1].1, "Beta Page 2");
        assert_eq!(pages[2].0, 3);
        assert_eq!(pages[2].1, "Gamma Page 3");
    }

    #[test]
    fn extract_pdf_pages_handles_garbage_safely() {
        // 垃圾字节: 返回空 Vec, 不 panic
        let pages = FileParser::extract_pdf_pages(b"%PDF-1.7 not a real pdf");
        assert!(pages.is_empty());
    }

    #[test]
    fn extract_pdf_pages_concatenates_multi_chunk_single_page() {
        // reportlab 生成的 PDF 单页每个文本块独立 BT/ET, lopdf 按块/编码切多个 chunk。
        // 修复前把扁平 chunk 列表与页号 zip 配对, 只取首个 chunk 导致整页文本丢失。
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
                Operation::new("Tf", vec!["F1".into(), 12.into()]),
                Operation::new("Td", vec![72.into(), 700.into()]),
                Operation::new("Tj", vec![lopdf::Object::string_literal("Alpha line one")]),
                Operation::new("ET", vec![]),
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 12.into()]),
                Operation::new("Td", vec![72.into(), 680.into()]),
                Operation::new("Tj", vec![lopdf::Object::string_literal("Beta line two")]),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(lopdf::Stream::new(
            lopdf::Dictionary::new(),
            content.encode().expect("encode content stream"),
        ));
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
        doc.save_to(&mut buf).expect("save test PDF");

        let pages = FileParser::extract_pdf_pages(&buf);
        assert_eq!(pages.len(), 1, "单页应聚合为一个条目, 得到 {pages:?}");
        assert_eq!(pages[0].0, 1);
        assert!(
            pages[0].1.contains("Alpha line one"),
            "应包含第一个 chunk, 得到 {:?}",
            pages[0].1
        );
        assert!(
            pages[0].1.contains("Beta line two"),
            "应包含第二个 chunk, 得到 {:?}",
            pages[0].1
        );
    }

    #[test]
    fn spatial_filters_compressed_stream_garbage() {
        // 模拟压缩流内字节: 高控制字符密度 (随机二进制经 UTF-8 lossy) 应被过滤
        let garbage = "BT\n(\x00\x01\x02\x03\x7f\x01\x02 garbled \x00\x01) Tj\nET";
        let blocks = FileParser::extract_pdf_spatial(garbage.as_bytes());
        assert!(
            blocks.is_empty(),
            "压缩流乱码不应产生 spatial 块, 得到 {blocks:?}"
        );
    }

    #[test]
    fn spatial_keeps_readable_text() {
        let pdf = b"%PDF-1.4\nBT\n1 0 0 1 100 700 Tm\n(Readable Hello World) Tj\nET";
        let blocks = FileParser::extract_pdf_spatial(pdf);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text, "Readable Hello World");
    }

    #[test]
    fn spatial_filters_nonsense_alphabetic_burst() {
        // 长二进制无空格 (压缩数据伪匹配): 即使含字母也应过滤
        let nonsense = "BT\n(AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB) Tj\nET";
        let blocks = FileParser::extract_pdf_spatial(nonsense.as_bytes());
        assert!(
            blocks.is_empty(),
            "无词边界长串不应产生 spatial 块, 得到 {blocks:?}"
        );
    }
}
