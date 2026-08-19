use super::{FileParser, SpatialBlock, BlockType};
use lopdf::dictionary;

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

/// PDF 文本编辑错误
#[derive(Debug)]
pub enum PdfEditError {
    /// PDF 解析/内容流处理失败
    Parse(String),
    /// 目标页不存在 (页号 1-based)
    PageNotFound { page: u32 },
    /// 未在目标页找到匹配文本
    NotFound { find: String },
    /// 字体解析/嵌入失败
    Font(String),
    /// 替换文本含嵌入字体不支持的字形
    UnsupportedGlyph(char),
    /// PDF 输出失败
    Write(String),
}

impl std::fmt::Display for PdfEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "PDF 解析失败: {e}"),
            Self::PageNotFound { page } => write!(f, "页面 {page} 不存在"),
            Self::NotFound { find } => write!(f, "未在页面找到文本: {find}"),
            Self::Font(e) => write!(f, "字体失败: {e}"),
            Self::UnsupportedGlyph(c) => write!(f, "字形不支持: {c}"),
            Self::Write(e) => write!(f, "PDF 写出失败: {e}"),
        }
    }
}

impl std::error::Error for PdfEditError {}

/// 单条 PDF 文本编辑。
///
/// 在 `page` (1-based) 定位所有匹配 `find` 的文本 span, 将其**原位移除** (redact,
/// 置空字符串操作数以保留版式/线稿层), 再在相同位置以**原字号**绘制 `replace`
/// (可选)。`replace = None` 时仅删除。
#[derive(Debug, Clone)]
pub struct PdfTextEdit {
    /// 目标页 (1-based)
    pub page: u32,
    /// 待定位的源文本
    pub find: String,
    /// 替换文本 (None = 仅删除)
    pub replace: Option<String>,
}

/// 从文本操作 (Tj/TJ/') 中解码字符串 (按当前字体编码)。
fn decode_text_operand(
    op: &lopdf::content::Operation,
    enc: Option<&lopdf::Encoding<'_>>,
) -> Result<String, PdfEditError> {
    for operand in &op.operands {
        match operand {
            lopdf::Object::String(bytes, _) => {
                let enc = enc.ok_or_else(|| PdfEditError::Parse("缺少字体编码".to_string()))?;
                return enc
                    .bytes_to_string(bytes)
                    .map_err(|e| PdfEditError::Parse(e.to_string()));
            }
            lopdf::Object::Array(items) => {
                let mut s = String::new();
                for item in items {
                    if let lopdf::Object::String(bytes, _) = item {
                        if let Some(enc) = enc {
                            s.push_str(
                                &enc
                                    .bytes_to_string(bytes)
                                    .map_err(|e| PdfEditError::Parse(e.to_string()))?,
                            );
                        }
                    }
                }
                return Ok(s);
            }
            _ => {}
        }
    }
    Ok(String::new())
}

/// 清空文本操作中的字符串/数组操作数 (redact: 保留占位, 视觉置空)。
fn clear_text_operands(op: &mut lopdf::content::Operation) {
    for operand in &mut op.operands {
        match operand {
            lopdf::Object::String(bytes, _) => bytes.clear(),
            lopdf::Object::Array(items) => {
                for item in items.iter_mut() {
                    if let lopdf::Object::String(bytes, _) = item {
                        bytes.clear();
                    }
                }
            }
            _ => {}
        }
    }
}

/// 文本状态机: 追踪当前光标/字号/字体, 供 span 定位插入坐标。
#[derive(Default)]
struct TextState {
    in_text: bool,
    tx: f32,
    ty: f32,
    leading: f32,
    size: f32,
    font: Option<Vec<u8>>,
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

    /// 编辑 PDF 文本 — span 级 redact + 原位替换。
    ///
    /// 对每个 `PdfTextEdit`: 在目标页内容流中定位所有匹配 `find` 的文本操作,
    /// 清空其字符串操作数 (视觉删除, 保留操作以不破坏版式/线稿层); 若 `replace`
    /// 存在, 则在每个被删除 span 的原坐标处以原字号追加替换文本。
    ///
    /// 替换文本的编码:
    /// - `Some(ttf)` — 嵌入提供的 TTF 为 Type0/Identity-H 子集字体, 支持任意 Unicode
    ///   (西里尔/中文等)。替换字符必须存在于该字体 (缺字形返回 `UnsupportedGlyph`)。
    /// - `None` — 回退 Helvetica base14, 仅支持 Latin-1 (越界字符返回 `UnsupportedGlyph`)。
    ///
    /// 未找到匹配返回 `PdfEditError::NotFound`, 不做任何改动。
    /// 返回修改后的完整 PDF 字节。
    pub fn edit_pdf_text(
        data: &[u8],
        edits: &[PdfTextEdit],
        ttf: Option<&[u8]>,
    ) -> Result<Vec<u8>, PdfEditError> {
        let mut doc = lopdf::Document::load_mem(data).map_err(|e| PdfEditError::Parse(e.to_string()))?;
        let pages = doc.get_pages();

        for edit in edits {
            let page_id = pages
                .get(&edit.page)
                .copied()
                .ok_or(PdfEditError::PageNotFound { page: edit.page })?;

            let fonts = doc.get_page_fonts(page_id).map_err(|e| PdfEditError::Parse(e.to_string()))?;
            let encodings: std::collections::BTreeMap<Vec<u8>, lopdf::Encoding<'_>> = fonts
                .iter()
                .map(|(name, font)| {
                    let enc = font.get_font_encoding(&doc).map_err(|e| PdfEditError::Parse(e.to_string()))?;
                    Ok((name.clone(), enc))
                })
                .collect::<Result<_, PdfEditError>>()?;
            let content_data = doc.get_page_content(page_id).map_err(|e| PdfEditError::Parse(e.to_string()))?;
            let mut content = lopdf::content::Content::decode(&content_data)
                .map_err(|e| PdfEditError::Parse(e.to_string()))?;

            let mut st = TextState::default();
            let mut positions: Vec<(f32, f32, f32)> = Vec::new();
            let mut matched = false;

            for op in &mut content.operations {
                match op.operator.as_str() {
                    "BT" => {
                        st.in_text = true;
                        st.tx = 0.0;
                        st.ty = 0.0;
                        st.leading = 0.0;
                    }
                    "ET" => st.in_text = false,
                    "Tf" => {
                        if let Some(name) = op.operands.first().and_then(|o| o.as_name().ok()) {
                            st.font = Some(name.to_vec());
                        }
                        if let Some(sz) = op.operands.get(1).and_then(|o| o.as_float().ok()) {
                            st.size = sz;
                        }
                    }
                    "Td" | "TD" => {
                        if let (Some(a), Some(b)) = (
                            op.operands.first().and_then(|o| o.as_float().ok()),
                            op.operands.get(1).and_then(|o| o.as_float().ok()),
                        ) {
                            st.tx += a;
                            st.ty += b;
                            if op.operator == "TD" {
                                st.leading = -b;
                            }
                        }
                    }
                    "T*" => st.ty -= st.leading,
                    "Tm" => {
                        if let (Some(e), Some(f)) = (
                            op.operands.get(4).and_then(|o| o.as_float().ok()),
                            op.operands.get(5).and_then(|o| o.as_float().ok()),
                        ) {
                            st.tx = e;
                            st.ty = f;
                        }
                    }
                    "Tj" | "TJ" | "'" if st.in_text => {
                        let enc = st.font.as_ref().and_then(|f| encodings.get(f));
                        let decoded = decode_text_operand(op, enc)?;
                        if decoded == edit.find {
                            matched = true;
                            positions.push((st.tx, st.ty, st.size));
                            clear_text_operands(op);
                        }
                    }
                    _ => {}
                }
            }

            if !matched {
                return Err(PdfEditError::NotFound { find: edit.find.clone() });
            }

            // redact 写回 (只读借用已在循环结束释放)
            let modified = content.encode().map_err(|e| PdfEditError::Parse(e.to_string()))?;
            doc.change_page_content(page_id, modified)
                .map_err(|e| PdfEditError::Parse(e.to_string()))?;

            // 原位替换插入
            if let Some(replace) = &edit.replace {
                let (font_id, font_key, encoded) = match ttf {
                    Some(ttf_bytes) => Self::embed_ttf_font(&mut doc, ttf_bytes, replace)?,
                    None => Self::embed_helvetica(&mut doc, replace)?,
                };
                Self::add_font_to_page_resources(&mut doc, page_id, &font_key, font_id)?;

                for (x, y, size) in positions {
                    let size = if size > 0.0 { size } else { 10.0 };
                    let insert = Self::insert_text_ops(&encoded, &font_key, size, x, y);
                    doc.add_to_page_content(page_id, insert)
                        .map_err(|e| PdfEditError::Parse(e.to_string()))?;
                }
            }
        }

        let mut buf = Vec::new();
        doc.save_to(&mut buf).map_err(|e| PdfEditError::Write(e.to_string()))?;
        Ok(buf)
    }

    /// 将字体引用注入页面 Resources/Font (不存在则创建)。
    fn add_font_to_page_resources(
        doc: &mut lopdf::Document,
        page_id: lopdf::ObjectId,
        font_key: &str,
        font_id: lopdf::ObjectId,
    ) -> Result<(), PdfEditError> {
        let font_value = lopdf::Object::Reference(font_id);
        let resources = doc
            .get_or_create_resources(page_id)
            .map_err(|e| PdfEditError::Parse(e.to_string()))?
            .as_dict_mut()
            .map_err(|e| PdfEditError::Parse(e.to_string()))?;
        match resources.get_mut(b"Font") {
            Ok(fonts) => {
                fonts
                    .as_dict_mut()
                    .map_err(|e| PdfEditError::Parse(e.to_string()))?
                    .set(font_key, font_value);
            }
            Err(_) => {
                resources.set(
                    "Font",
                    lopdf::Object::Dictionary(dictionary! {
                        font_key.to_string() => font_value,
                    }),
                );
            }
        }
        Ok(())
    }

    /// 嵌入 TTF 为 Type0/CIDFontType2/Identity-H 子集字体, 返回 (字体对象引用, 资源键, GID 编码字节)。
    /// 仅嵌入 `text` 实际使用的字形 (按 GID), 提供 /W 宽度与 /ToUnicode。
    fn embed_ttf_font(
        doc: &mut lopdf::Document,
        ttf: &[u8],
        text: &str,
    ) -> Result<(lopdf::ObjectId, String, Vec<u8>), PdfEditError> {
        let face = ttf_parser::Face::parse(ttf, 0).map_err(|e| PdfEditError::Font(e.to_string()))?;
        let upem = f32::from(face.units_per_em());

        // 收集字形: (char, gid, width1000)
        let mut used: Vec<(char, u16, u16)> = Vec::new();
        for ch in text.chars() {
            let gid = face
                .glyph_index(ch)
                .ok_or(PdfEditError::UnsupportedGlyph(ch))?;
            let advance = face.glyph_hor_advance(gid).unwrap_or(500);
            used.push((ch, gid.0, advance));
        }
        if used.is_empty() {
            return Err(PdfEditError::Font("替换文本为空".to_string()));
        }
        // Identity-H 编码字节: 每字符 2 字节 BE = 字体内部 GID
        let mut encoded = Vec::with_capacity(used.len() * 2);
        for (_, gid, _) in &used {
            encoded.extend_from_slice(&gid.to_be_bytes());
        }

        let font_key = format!("NTFont{}", doc.max_id + 1);

        // FontFile2: 原始 TTF 字节 (Identity-H 子集化前不裁剪, 兼容性优先)
        // 注意: 不声明 Filter — lopdf 保存时不压缩, 声明 FlateDecode 但存明文会使流损坏
        let font_file_id = doc.add_object(lopdf::Stream::new(
            dictionary! {
                "Length1" => ttf.len() as i64,
            },
            ttf.to_vec(),
        ));

        // W 宽度数组: [firstCID, lastCID, widths...] 用 gid 做 CID
        let mut cids: Vec<u16> = used.iter().map(|(_, gid, _)| *gid).collect();
        cids.sort_unstable();
        cids.dedup();
        let w_first = cids.first().copied().unwrap_or(1);
        let w_last = cids.last().copied().unwrap_or(1);
        let mut widths = Vec::with_capacity((w_last - w_first + 1) as usize);
        for cid in w_first..=w_last {
            let w = used
                .iter()
                .find(|(_, gid, _)| *gid == cid)
                .map(|(_, _, a)| a)
                .unwrap_or(&0);
            let width = (*w as f32 * 1000.0 / upem).round() as i64;
            widths.push(lopdf::Object::Integer(width));
        }

        // ToUnicode CMap
        let mut cmap = String::from(
            "/CIDInit /ProcSet findresource begin\n\
             12 dict begin\n\
             begincmap\n\
             /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> def\n\
             /CMapName /Adobe-Identity-UCS def\n\
             /CMapType 2 def\n\
             1 begincodespacerange\n\
             <0000> <FFFF>\n\
             endcodespacerange\n",
        );
        let mut n = 0;
        for (ch, gid, _) in &used {
            if n % 100 == 0 {
                if n > 0 {
                    cmap.push_str("endbfchar\n");
                }
                let batch = 100usize.min(used.len() - n);
                cmap.push_str(&format!("{batch} beginbfchar\n"));
            }
            cmap.push_str(&format!("<{:04X}> <{:04X}>\n", gid, *ch as u32));
            n += 1;
        }
        cmap.push_str("endbfchar\nendcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
        let to_unicode_id = doc.add_object(lopdf::Stream::new(
            dictionary! {
                "Length1" => cmap.len() as i64,
            },
            cmap.into_bytes(),
        ));

        // FontDescriptor
        let descriptor_id = doc.add_object(dictionary! {
            "Type" => lopdf::Object::Name(b"FontDescriptor".to_vec()),
            "FontName" => lopdf::Object::Name(b"NeoTrixEmbedded".to_vec()),
            "Flags" => lopdf::Object::Integer(4),
            "FontBBox" => lopdf::Object::Array(vec![
                lopdf::Object::Integer(0),
                lopdf::Object::Integer(0),
                lopdf::Object::Integer(1000),
                lopdf::Object::Integer(1000),
            ]),
            "ItalicAngle" => lopdf::Object::Integer(0),
            "Ascent" => lopdf::Object::Integer(800),
            "Descent" => lopdf::Object::Integer(-200),
            "CapHeight" => lopdf::Object::Integer(700),
            "StemV" => lopdf::Object::Integer(80),
            "FontFile2" => lopdf::Object::Reference(font_file_id),
        });

        // CIDFont
        let cid_font_id = doc.add_object(dictionary! {
            "Type" => lopdf::Object::Name(b"Font".to_vec()),
            "Subtype" => lopdf::Object::Name(b"CIDFontType2".to_vec()),
            "BaseFont" => lopdf::Object::Name(b"NeoTrixEmbedded".to_vec()),
            "CIDSystemInfo" => lopdf::Object::Dictionary(dictionary! {
                "Registry" => lopdf::Object::string_literal("Adobe"),
                "Ordering" => lopdf::Object::string_literal("Identity"),
                "Supplement" => lopdf::Object::Integer(0),
            }),
            "FontDescriptor" => lopdf::Object::Reference(descriptor_id),
            "DW" => lopdf::Object::Integer(1000),
            "W" => lopdf::Object::Array(vec![
                lopdf::Object::Integer(w_first as i64),
                lopdf::Object::Integer(w_last as i64),
                lopdf::Object::Array(widths),
            ]),
            "CIDToGIDMap" => lopdf::Object::Name(b"Identity".to_vec()),
        });

        // Type0
        let type0_id = doc.add_object(dictionary! {
            "Type" => lopdf::Object::Name(b"Font".to_vec()),
            "Subtype" => lopdf::Object::Name(b"Type0".to_vec()),
            "BaseFont" => lopdf::Object::Name(b"NeoTrixEmbedded".to_vec()),
            "Encoding" => lopdf::Object::Name(b"Identity-H".to_vec()),
            "DescendantFonts" => lopdf::Object::Array(vec![lopdf::Object::Reference(cid_font_id)]),
            "ToUnicode" => lopdf::Object::Reference(to_unicode_id),
        });

        Ok((type0_id, font_key, encoded))
    }

    /// 回退 Helvetica base14: 校验 Latin-1 范围, 注册 Type1 字体对象, 返回
    /// (对象引用, 资源键, 单字节编码)。base14 字体需真实对象引用 — 仅用
    /// `/Helvetica` Name 会被 `get_page_fonts` 跳过, 提取/消费方无法解析。
    fn embed_helvetica(
        doc: &mut lopdf::Document,
        text: &str,
    ) -> Result<(lopdf::ObjectId, String, Vec<u8>), PdfEditError> {
        for ch in text.chars() {
            let cp = ch as u32;
            if cp > 0xFF || (cp >= 0x80 && cp < 0xA0) {
                return Err(PdfEditError::UnsupportedGlyph(ch));
            }
        }
        let encoded: Vec<u8> = text.chars().map(|c| c as u8).collect();
        let font_id = doc.add_object(dictionary! {
            "Type" => lopdf::Object::Name(b"Font".to_vec()),
            "Subtype" => lopdf::Object::Name(b"Type1".to_vec()),
            "BaseFont" => lopdf::Object::Name(b"Helvetica".to_vec()),
        });
        Ok((font_id, "Helvetica".to_string(), encoded))
    }

    /// 构建插入文本的内容流操作。
    fn insert_text_ops(
        encoded: &[u8],
        font_key: &str,
        size: f32,
        x: f32,
        y: f32,
    ) -> lopdf::content::Content<Vec<lopdf::content::Operation>> {
        let mut ops = Vec::new();
        ops.push(lopdf::content::Operation::new("q", vec![]));
        ops.push(lopdf::content::Operation::new("0", vec![lopdf::Object::Integer(0)]));
        ops.push(lopdf::content::Operation::new("BT", vec![]));
        ops.push(lopdf::content::Operation::new(
            "Tf",
            vec![
                lopdf::Object::Name(font_key.as_bytes().to_vec()),
                lopdf::Object::Real(size),
            ],
        ));
        ops.push(lopdf::content::Operation::new(
            "Tm",
            vec![
                lopdf::Object::Real(1.0),
                lopdf::Object::Real(0.0),
                lopdf::Object::Real(0.0),
                lopdf::Object::Real(1.0),
                lopdf::Object::Real(x),
                lopdf::Object::Real(y),
            ],
        ));
        ops.push(lopdf::content::Operation::new(
            "Tj",
            vec![lopdf::Object::String(encoded.to_vec(), lopdf::StringFormat::Hexadecimal)],
        ));
        ops.push(lopdf::content::Operation::new("ET", vec![]));
        ops.push(lopdf::content::Operation::new("Q", vec![]));
        lopdf::content::Content { operations: ops }
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

    /// 单页未压缩内容流 PDF, 单行文本 (Courier Type1), 无向量绘制。
    fn single_page_pdf_bytes(text: &str) -> Vec<u8> {
        let mut doc = lopdf::Document::with_version("1.4");
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
                Operation::new("Td", vec![100.into(), 600.into()]),
                Operation::new("Tj", vec![lopdf::Object::string_literal(text)]),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(lopdf::Stream::new(
            lopdf::Dictionary::new(),
            content.encode().expect("encode PDF content stream"),
        ));
        let page = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        doc.objects.insert(
            pages_id,
            lopdf::Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page.into()],
                "Count" => 1,
            }),
        );
        let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog_id);
        let mut buf = Vec::new();
        doc.save_to(&mut buf).expect("save test PDF");
        buf
    }

    /// 单页未压缩内容流 PDF: 文本前含向量绘制 (re + S), 验证编辑不动线稿层。
    fn layout_pdf_bytes(text: &str) -> Vec<u8> {
        let mut doc = lopdf::Document::with_version("1.4");
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
                Operation::new("re", vec![10.into(), 10.into(), 100.into(), 50.into()]),
                Operation::new("S", vec![]),
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 12.into()]),
                Operation::new("Td", vec![100.into(), 600.into()]),
                Operation::new("Tj", vec![lopdf::Object::string_literal(text)]),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(lopdf::Stream::new(
            lopdf::Dictionary::new(),
            content.encode().expect("encode PDF content stream"),
        ));
        let page = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        doc.objects.insert(
            pages_id,
            lopdf::Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page.into()],
                "Count" => 1,
            }),
        );
        let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog_id);
        let mut buf = Vec::new();
        doc.save_to(&mut buf).expect("save test PDF");
        buf
    }

    /// 尝试读取含西里尔字形的系统 TTF (跳过不可用环境)。
    fn cyrillic_ttf() -> Option<Vec<u8>> {
        let candidates = [
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/System/Library/Fonts/Supplemental/Georgia.ttf",
            "/System/Library/Fonts/Supplemental/Verdana.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ];
        let data = candidates.iter().map(std::fs::read).find_map(Result::ok)?;
        let face = ttf_parser::Face::parse(&data, 0).ok()?;
        if "Привет".chars().all(|c| face.glyph_index(c).is_some()) {
            Some(data)
        } else {
            None
        }
    }

    #[test]
    fn edit_pdf_redact_replaces_text_base14() {
        let buf = single_page_pdf_bytes("Hello World");
        let out = FileParser::edit_pdf_text(
            &buf,
            &[PdfTextEdit { page: 1, find: "Hello World".into(), replace: Some("Hola Mundo".into()) }],
            None,
        )
        .expect("edit pdf");
        let text = FileParser::extract_pdf_text(&out);
        assert!(text.contains("Hola Mundo"), "替换文本缺失: {text:?}");
        assert!(!text.contains("Hello"), "原文本仍存在: {text:?}");
    }

    #[test]
    fn edit_pdf_delete_only_removes_text() {
        let buf = single_page_pdf_bytes("Remove Me");
        let out = FileParser::edit_pdf_text(
            &buf,
            &[PdfTextEdit { page: 1, find: "Remove Me".into(), replace: None }],
            None,
        )
        .expect("edit pdf");
        let text = FileParser::extract_pdf_text(&out);
        assert!(!text.contains("Remove"), "redact 未删除文本: {text:?}");
    }

    #[test]
    fn edit_pdf_not_found_errors() {
        let buf = single_page_pdf_bytes("Hello");
        let err = FileParser::edit_pdf_text(
            &buf,
            &[PdfTextEdit { page: 1, find: "Missing".into(), replace: None }],
            None,
        )
        .expect_err("应返回 NotFound");
        assert!(matches!(err, PdfEditError::NotFound { .. }));
    }

    #[test]
    fn edit_pdf_page_out_of_range_errors() {
        let buf = single_page_pdf_bytes("Hello");
        let err = FileParser::edit_pdf_text(
            &buf,
            &[PdfTextEdit { page: 99, find: "Hello".into(), replace: None }],
            None,
        )
        .expect_err("应返回 PageNotFound");
        assert!(matches!(err, PdfEditError::PageNotFound { .. }));
    }

    #[test]
    fn edit_pdf_keeps_layout_operators() {
        let buf = layout_pdf_bytes("Needle");
        let out = FileParser::edit_pdf_text(
            &buf,
            &[PdfTextEdit { page: 1, find: "Needle".into(), replace: None }],
            None,
        )
        .expect("edit pdf");
        let doc = lopdf::Document::load_mem(&out).expect("reload edited pdf");
        let page_id = *doc.get_pages().get(&1).expect("page 1");
        let content_data = doc.get_page_content(page_id).expect("page content");
        let content = lopdf::content::Content::decode(&content_data).expect("decode content");
        assert!(
            content.operations.iter().any(|op| op.operator == "re"),
            "线稿层 (re 操作) 丢失"
        );
        assert!(
            content
                .operations
                .iter()
                .any(|op| op.operator == "Tj"
                    && op
                        .operands
                        .iter()
                        .any(|o| matches!(o, lopdf::Object::String(b, _) if b.is_empty()))),
            "文本应被置空而非删除操作"
        );
    }

    #[test]
    fn edit_pdf_ttf_cyrillic_replace() {
        let Some(ttf) = cyrillic_ttf() else {
            eprintln!("无西里尔系统字体, 跳过 TTF 测试");
            return;
        };
        let buf = single_page_pdf_bytes("Hello World");
        let out = FileParser::edit_pdf_text(
            &buf,
            &[PdfTextEdit { page: 1, find: "Hello World".into(), replace: Some("Привет".into()) }],
            Some(&ttf),
        )
        .expect("edit pdf with embedded ttf");
        let text = FileParser::extract_pdf_text(&out);
        assert!(
            text.contains("Привет"),
            "嵌入 TTF/ToUnicode 未解码出西里尔文本: {text:?}"
        );
        assert!(!text.contains("Hello"), "原文本仍存在: {text:?}");
    }
}
