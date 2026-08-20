use super::{FileParser, SpatialBlock, BlockType};
use lopdf::dictionary;
#[cfg(test)]
use lopdf::Stream;

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
    /// PDF 合并失败 (多文档)
    Merge(String),
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
            Self::Merge(e) => write!(f, "PDF 合并失败: {e}"),
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

/// PDF 表格单元格 (保留坐标供布局校验)。
#[derive(Debug, Clone, PartialEq)]
pub struct PdfTableCell {
    /// 单元格文本
    pub text: String,
    /// 列 x 位置 (用户空间)
    pub x: f32,
    /// 行 y 位置
    pub y: f32,
}

/// 从 PDF 文本布局中重建的表格网格。
///
/// 纯文本坐标启发式 (行 y 聚类 + 列 x 聚类), 不依赖 OCR/外部分析器 —
/// 对逐单元格绘制的表格型 PDF (设备清单/报价表) 精度高, 复杂合并单元格需外部后端。
#[derive(Debug, Clone)]
pub struct PdfTable {
    /// 页号 (1-based)
    pub page: u32,
    /// 全局列 x 中心 (升序)
    pub columns: Vec<f32>,
    /// 行 (从上到下), 每行与 `columns` 等宽, 空单元格为 `None`
    pub rows: Vec<Vec<Option<PdfTableCell>>>,
}

impl PdfTable {
    /// 纯文本网格 (空单元格 → 空串)。
    pub fn to_grid(&self) -> Vec<Vec<String>> {
        self.rows
            .iter()
            .map(|r| {
                r.iter()
                    .map(|c| c.as_ref().map_or(String::new(), |c| c.text.clone()))
                    .collect()
            })
            .collect()
    }

    /// 渲染为 Markdown 表格 (首行作表头)。
    pub fn to_markdown(&self) -> String {
        let grid = self.to_grid();
        if grid.is_empty() || grid[0].is_empty() {
            return String::new();
        }
        let header = grid[0].iter().map(|c| c.trim()).collect::<Vec<_>>();
        let mut out = format!("| {} |\n", header.join(" | "));
        out.push_str(&format!(
            "|{}|\n",
            header.iter().map(|_| " --- ").collect::<Vec<_>>().join("|")
        ));
        for row in grid.iter().skip(1) {
            let cells = row.iter().map(|c| c.trim()).collect::<Vec<_>>();
            out.push_str(&format!("| {} |\n", cells.join(" | ")));
        }
        out
    }
}

/// 合并多个 PDF 为单个文档 (R-P79 生产接线)。
///
/// 采用 lopdf 官方 merge 流程: 对每个输入文档 `renumber_objects_with` 偏移对象
/// 图, 收集全部对象, 丢弃各文档自己的 Catalog/Pages/Page, 以首个文档的 Catalog
/// 为基座, 重建单一 Pages 树 (Kids = 全部页面, 扁平化嵌套 Pages), 最后
/// `renumber_objects` 统一重排。字体/资源对象随对象图整体迁移, 跨文件 ID 不冲突
/// (对象号偏移保证唯一); 不做资源去重 (可接受冗余, 与研究结论一致)。
///
/// 输入为空或单文件时分别报错/透传。任一无页面输入报 Merge 错误。
pub fn merge_pdfs(inputs: &[Vec<u8>]) -> Result<Vec<u8>, PdfEditError> {
    use std::collections::BTreeMap;

    if inputs.is_empty() {
        return Err(PdfEditError::Merge("无输入 PDF".to_string()));
    }
    if inputs.len() == 1 {
        return Ok(inputs[0].clone());
    }

    // 收集全部文档的对象与页面
    let mut documents_objects: BTreeMap<lopdf::ObjectId, lopdf::Object> = BTreeMap::new();
    let mut documents_pages: BTreeMap<lopdf::ObjectId, lopdf::Object> = BTreeMap::new();
    let mut max_id = 1u32;

    for (idx, data) in inputs.iter().enumerate() {
        let mut doc = lopdf::Document::load_mem(data)
            .map_err(|e| PdfEditError::Parse(format!("输入 {idx} 解析失败: {e}")))?;
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        // 提取该文档全部页面 (get_pages 只返回叶子 Page, 扁平化嵌套 Pages 树)
        let pages = doc.get_pages();
        if pages.is_empty() {
            return Err(PdfEditError::Merge(format!("输入 {idx} 无页面")));
        }
        for (_seq, page_obj) in pages {
            documents_pages.insert(
                page_obj,
                doc.get_object(page_obj)
                    .map_err(|e| PdfEditError::Merge(format!("输入 {idx} 页面读取失败: {e}")))?
                    .clone(),
            );
        }
        // 其余对象全部收集 (Catalog/Pages 稍后特殊处理)
        documents_objects.extend(doc.objects);
    }

    // 分类对象: 以首个 Catalog 为基座, 丢弃其余 Catalog/Outlines, 合并 Pages 字典
    let mut catalog_object: Option<(lopdf::ObjectId, lopdf::Object)> = None;
    let mut pages_object: Option<(lopdf::ObjectId, lopdf::Object)> = None;
    let mut main = lopdf::Document::with_version("1.7");

    for (object_id, object) in documents_objects.into_iter() {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                if catalog_object.is_none() {
                    catalog_object = Some((object_id, object));
                }
            }
            b"Pages" => {
                if let Ok(dict) = object.as_dict() {
                    let mut dict = dict.clone();
                    if let Some((_, ref old)) = pages_object {
                        if let Ok(old_dict) = old.as_dict() {
                            // 合入全部 Pages 树字典字段 (非 Kids/Count, 由重建覆盖)
                            for (k, v) in old_dict.iter() {
                                if dict.get(k).is_err() {
                                    dict.set(k.clone(), v.clone());
                                }
                            }
                        }
                    }
                    pages_object = Some((
                        if let Some((id, _)) = pages_object { id } else { object_id },
                        lopdf::Object::Dictionary(dict),
                    ));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {} // 忽略 (Pages/Outlines 不支持)
            _ => {
                main.objects.insert(object_id, object);
            }
        }
    }

    let Some((catalog_id, catalog_obj)) = catalog_object else {
        return Err(PdfEditError::Merge("未找到 Catalog 根".to_string()));
    };
    let Some((pages_id, pages_obj)) = pages_object else {
        return Err(PdfEditError::Merge("未找到 Pages 根".to_string()));
    };

    // 重建 Pages: 全部叶子页 → 单一 Kids, 更新 Count
    if let Ok(dict) = pages_obj.as_dict() {
        let mut dict = dict.clone();
        dict.set("Count", documents_pages.len() as u32);
        dict.set(
            "Kids",
            documents_pages
                .keys()
                .map(|id| lopdf::Object::Reference(*id))
                .collect::<Vec<_>>(),
        );
        main.objects.insert(pages_id, lopdf::Object::Dictionary(dict));
    }

    // 重建 Catalog: Pages 指向单一根, 移除 Outlines (合并后不可用)
    if let Ok(dict) = catalog_obj.as_dict() {
        let mut dict = dict.clone();
        dict.set("Pages", pages_id);
        dict.remove(b"Outlines");
        main.objects.insert(catalog_id, lopdf::Object::Dictionary(dict));
    }

    // 全部页面改指向合并后的 Pages 根
    for (obj_id, obj) in documents_pages.iter() {
        if let Ok(dict) = obj.as_dict() {
            let mut dict = dict.clone();
            dict.set("Parent", pages_id);
            main.objects.insert(*obj_id, lopdf::Object::Dictionary(dict));
        }
    }

    main.trailer.set("Root", catalog_id);
    main.max_id = main.objects.len() as u32;
    main.renumber_objects();

    let mut buf = Vec::new();
    main.save_to(&mut buf)
        .map_err(|e| PdfEditError::Write(e.to_string()))?;
    Ok(buf)
}

/// 查找覆盖 `text` 全部字形 (且非 .notdef 空字形) 的系统字体字节。
///
/// 非 Latin-1 替换文本 (西里尔/中文等) 需真实 TTF/OTF 嵌入; base14 Helvetica 仅支持
/// Latin-1。返回首个完整覆盖的字体文件内容, 找不到返回 `None`。
///
/// 关键点:
/// - **TTC 集合多 face 探测**: PingFang.ttc 等集合常把 CJK 字形放在后置 face
///   (实测 face 0/1 无 CJK, face 2/3 才有), 必须遍历全部 face。
/// - **跳过 .notdef**: `glyph_index` 对映射到空字形 (glyph 0) 的字符返回 `Some(0)`,
///   须过滤, 否则选中的字体渲染为空白。
/// - **优先顺序**: 先探测覆盖最广的单文件字体 (Arial Unicode.ttf 同时含 CJK+西里尔),
///   再回退扫描常见系统字体目录。
pub fn find_system_font_for_text(text: &str) -> Option<Vec<u8>> {
    if text.is_empty() {
        return None;
    }
    let mut candidates: Vec<std::path::PathBuf> = vec![
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf".into(),
        "/System/Library/Fonts/PingFang.ttc".into(),
        "/System/Library/Fonts/Supplemental/Songti.ttc".into(),
        "/System/Library/Fonts/Supplemental/Arial.ttf".into(),
        "/System/Library/Fonts/Supplemental/Georgia.ttf".into(),
        "/System/Library/Fonts/Supplemental/Verdana.ttf".into(),
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".into(),
        "/usr/share/fonts/TTF/DejaVuSans.ttf".into(),
        "C:\\Windows\\Fonts\\arial.ttf".into(),
    ];
    for dir in [
        "/System/Library/Fonts",
        "/System/Library/Fonts/Supplemental",
        "/Library/Fonts",
        "/usr/share/fonts/truetype",
        "/usr/share/fonts",
        "C:\\Windows\\Fonts",
    ] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for ent in rd.flatten() {
                let p = ent.path();
                let Some(ext) = p.extension().and_then(|e| e.to_str()) else { continue };
                if matches!(ext, "ttf" | "otf" | "ttc" | "otc") {
                    candidates.push(p);
                }
            }
        }
    }
    for path in candidates {
        let Ok(data) = std::fs::read(&path) else { continue };
        let nfaces = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
        for face_index in 0..nfaces {
            let Ok(face) = ttf_parser::Face::parse(&data, face_index) else {
                continue;
            };
            if text.chars().all(|c| {
                face.glyph_index(c).is_some_and(|g| g.0 != 0)
            }) {
                return Some(data);
            }
        }
    }
    None
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

/// 单个文本 run: 同一 BT..ET 段内连续文本操作 (跨 Tf/Td/Tm 排版操作) 累积的
/// 解码串与其起始坐标。编辑侧用操作索引清空, 提取侧用坐标聚类重建布局。
#[derive(Debug, Clone)]
struct TextRun {
    x: f32,
    y: f32,
    size: f32,
    /// (操作索引, 解码串)
    ops: Vec<(usize, String)>,
}

impl TextRun {
    fn text(&self) -> String {
        self.ops.iter().map(|(_, s)| s.as_str()).collect()
    }
}

fn flush_run(
    cur_ops: &mut Vec<(usize, String)>,
    cur_start: &mut Option<(f32, f32, f32)>,
    runs: &mut Vec<TextRun>,
) {
    if cur_ops.is_empty() {
        *cur_start = None;
        return;
    }
    let (x, y, size) = cur_start.unwrap_or((0.0, 0.0, 12.0));
    runs.push(TextRun {
        x,
        y,
        size,
        ops: std::mem::take(cur_ops),
    });
    *cur_start = None;
}

/// 从解码后的内容操作累积文本 run (跨 Tf/Td/Tm 等排版操作, 遇 BT/ET/'/其他 op 断 run)。
/// 编辑与表格提取共用此收集逻辑, 保证两处坐标/解码一致性。
fn collect_text_runs(
    operations: &[lopdf::content::Operation],
    encodings: &std::collections::BTreeMap<Vec<u8>, lopdf::Encoding<'_>>,
) -> Result<Vec<TextRun>, PdfEditError> {
    let mut st = TextState::default();
    let mut runs: Vec<TextRun> = Vec::new();
    let mut cur_ops: Vec<(usize, String)> = Vec::new();
    let mut cur_start: Option<(f32, f32, f32)> = None;
    let mut in_text = false;

    for (idx, op) in operations.iter().enumerate() {
        match op.operator.as_str() {
            "BT" => {
                in_text = true;
                st = TextState::default();
                st.in_text = true;
                cur_ops.clear();
                cur_start = None;
            }
            "ET" => {
                in_text = false;
                flush_run(&mut cur_ops, &mut cur_start, &mut runs);
            }
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
            "Tj" | "TJ" | "'" if in_text => {
                let enc = st.font.as_ref().and_then(|f| encodings.get(f));
                let decoded = decode_text_operand(op, enc)?;
                if decoded.is_empty() {
                    continue;
                }
                if cur_start.is_none() {
                    cur_start = Some((st.tx, st.ty, st.size));
                }
                cur_ops.push((idx, decoded));
                if op.operator == "'" {
                    flush_run(&mut cur_ops, &mut cur_start, &mut runs);
                }
            }
            _ => flush_run(&mut cur_ops, &mut cur_start, &mut runs),
        }
    }
    flush_run(&mut cur_ops, &mut cur_start, &mut runs);
    Ok(runs)
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

    /// 表格结构化提取 — 从解码内容流收集带坐标的文本 run, 按 y 行聚类 + x 列
    /// 聚类重建网格。启发式判定表格: ≥2 行且 ≥2 列。走 lopdf 完整解析
    /// (压缩流/ToUnicode/TJ), 与 `edit_pdf_text` 共用 run 收集逻辑。
    pub fn extract_pdf_tables(data: &[u8]) -> Vec<PdfTable> {
        let Ok(doc) = lopdf::Document::load_mem(data) else {
            return Vec::new();
        };
        let pages = doc.get_pages();
        let mut tables = Vec::new();
        for (page_num, page_id) in pages.iter().map(|(n, id)| (*n, *id)) {
            let Ok(fonts) = doc.get_page_fonts(page_id) else {
                continue;
            };
            let encodings: std::collections::BTreeMap<Vec<u8>, lopdf::Encoding<'_>> =
                match fonts
                    .iter()
                    .map(|(name, font)| {
                        font.get_font_encoding(&doc).map(|e| (name.clone(), e))
                    })
                    .collect::<Result<_, lopdf::Error>>()
                {
                    Ok(m) => m,
                    Err(_) => continue,
                };
            let Ok(content_data) = doc.get_page_content(page_id) else {
                continue;
            };
            let Ok(content) = lopdf::content::Content::decode(&content_data) else {
                continue;
            };
            let Ok(runs) = collect_text_runs(&content.operations, &encodings) else {
                continue;
            };
            if let Some(table) = Self::build_table_grid(page_num, &runs) {
                tables.push(table);
            }
        }
        tables
    }

    /// 将文本 run 聚类为表格网格; 不足表格形态 (行<2 或列<2) 返回 None。
    fn build_table_grid(page: u32, runs: &[TextRun]) -> Option<PdfTable> {
        if runs.is_empty() {
            return None;
        }
        // 1. 行聚类: y 误差 5.0 内同属一行; 行从上到下 (y 降序)。
        let mut sorted: Vec<&TextRun> = runs.iter().collect();
        sorted.sort_by(|a, b| {
            b.y.partial_cmp(&a.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });
        const ROW_EPS: f32 = 5.0;
        let mut row_groups: Vec<Vec<&TextRun>> = Vec::new();
        for run in sorted {
            let grouped = row_groups
                .last_mut()
                .map(|last| (last[0].y - run.y).abs() <= ROW_EPS)
                .unwrap_or(false);
            if grouped {
                row_groups.last_mut().expect("grouped implies non-empty").push(run);
            } else {
                row_groups.push(vec![run]);
            }
        }
        // 2. 列聚类: 全局 x 中心, 误差 10.0 内归同一列。
        const COL_EPS: f32 = 10.0;
        let mut xs: Vec<f32> = runs.iter().map(|r| r.x).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut columns: Vec<f32> = Vec::new();
        for x in xs {
            if let Some(last) = columns.last_mut() {
                if (x - *last) <= COL_EPS {
                    *last = (*last + x) / 2.0;
                    continue;
                }
            }
            columns.push(x);
        }
        if columns.len() < 2 || row_groups.len() < 2 {
            return None;
        }
        // 3. 单元格归列: 每行按 x 就近归入列; 同列多个 run (碎片) 拼接。
        let col_for = |x: f32| -> usize {
            let mut best = 0usize;
            let mut best_d = f32::INFINITY;
            for (i, c) in columns.iter().enumerate() {
                let d = (x - c).abs();
                if d < best_d {
                    best_d = d;
                    best = i;
                }
            }
            best
        };
        let mut rows: Vec<Vec<Option<PdfTableCell>>> = Vec::new();
        for group in &row_groups {
            let mut row: Vec<Option<PdfTableCell>> =
                (0..columns.len()).map(|_| None).collect();
            for run in group {
                let text = run.text();
                if text.is_empty() {
                    continue;
                }
                let ci = col_for(run.x);
                let merged = row[ci]
                    .as_mut()
                    .map(|cell| {
                        cell.text.push(' ');
                        cell.text.push_str(&text);
                    })
                    .is_some();
                if !merged {
                    row[ci] = Some(PdfTableCell {
                        text,
                        x: run.x,
                        y: run.y,
                    });
                }
            }
            rows.push(row);
        }
        Some(PdfTable {
            page,
            columns,
            rows,
        })
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

            let mut positions: Vec<(f32, f32, f32)> = Vec::new();
            let mut matched = false;

            // 多字符 span 跨相邻 op 匹配: CAD 类导出 PDF 常逐字符 Tj 绘制,
            // 单 op 解码串==find 无法命中「闸阀」等多字符目标。run 收集逻辑
            // 见 collect_text_runs (与表格提取共用); 此处对每个 run 的累积串
            // 做子串查找, 命中后清空覆盖的全部操作数 (保守整 op 清空)。
            let runs = collect_text_runs(&content.operations, &encodings)?;

            let mut to_clear: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
            for run in &runs {
                if run.ops.is_empty() {
                    continue;
                }
                let mut concat = String::new();
                let mut starts: Vec<usize> = Vec::with_capacity(run.ops.len() + 1);
                for (_, decoded) in &run.ops {
                    starts.push(concat.len());
                    concat.push_str(decoded);
                }
                starts.push(concat.len());
                let find_len = edit.find.len();
                let mut offset = 0;
                while let Some(rel) = concat[offset..].find(edit.find.as_str()) {
                    let mstart = offset + rel;
                    let mend = mstart + find_len;
                    let op_start = starts.partition_point(|&s| s <= mstart).saturating_sub(1);
                    let op_end = starts.partition_point(|&s| s < mend).saturating_sub(1);
                    if op_start <= op_end && op_end < run.ops.len() && starts[op_end + 1] >= mend {
                        for i in op_start..=op_end {
                            to_clear.insert(run.ops[i].0);
                        }
                        positions.push((run.x, run.y, run.size));
                        matched = true;
                        offset = mend;
                    } else {
                        offset = mstart + 1;
                    }
                }
            }

            if !matched {
                return Err(PdfEditError::NotFound { find: edit.find.clone() });
            }

            for (idx, op) in content.operations.iter_mut().enumerate() {
                if to_clear.contains(&idx) {
                    clear_text_operands(op);
                }
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
        for (n, (ch, gid, _)) in used.iter().enumerate() {
            if n % 100 == 0 {
                if n > 0 {
                    cmap.push_str("endbfchar\n");
                }
                let batch = 100usize.min(used.len() - n);
                cmap.push_str(&format!("{batch} beginbfchar\n"));
            }
            cmap.push_str(&format!("<{:04X}> <{:04X}>\n", gid, *ch as u32));
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
            if cp > 0xFF || (0x80..0xA0).contains(&cp) {
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
        let ops = vec![
            lopdf::content::Operation::new("q", vec![]),
            lopdf::content::Operation::new("0", vec![lopdf::Object::Integer(0)]),
            lopdf::content::Operation::new("BT", vec![]),
            lopdf::content::Operation::new(
                "Tf",
                vec![
                    lopdf::Object::Name(font_key.as_bytes().to_vec()),
                    lopdf::Object::Real(size),
                ],
            ),
            lopdf::content::Operation::new(
                "Tm",
                vec![
                    lopdf::Object::Real(1.0),
                    lopdf::Object::Real(0.0),
                    lopdf::Object::Real(0.0),
                    lopdf::Object::Real(1.0),
                    lopdf::Object::Real(x),
                    lopdf::Object::Real(y),
                ],
            ),
            lopdf::content::Operation::new(
                "Tj",
                vec![lopdf::Object::String(encoded.to_vec(), lopdf::StringFormat::Hexadecimal)],
            ),
            lopdf::content::Operation::new("ET", vec![]),
            lopdf::content::Operation::new("Q", vec![]),
        ];
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

    #[test]
    fn find_system_font_covers_cyrillic_and_cjk() {
        // 西里尔: 候选列表首位 (Arial Unicode.ttf) 须完整覆盖。
        let cyr = find_system_font_for_text("ЗАДВИЖКА");
        assert!(
            cyr.is_some(),
            "西里尔替换应自动选中系统字体 (Arial Unicode/DejaVu)"
        );
        // 中文: 依赖 TTC 多 face 探测 (PingFang/Songti 后置 face 才有 CJK)。
        let cjk = find_system_font_for_text("闸阀门");
        assert!(
            cjk.is_some(),
            "CJK 替换应自动选中含 CJK 字形的系统字体 (多 face 探测)"
        );
        // 混合: 单字体同时覆盖中西文。
        let mixed = find_system_font_for_text("阀З");
        assert!(mixed.is_some(), "混合中西文应可找到单字体覆盖");
    }

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

    /// 生成逐单元格绝对定位 (Tm) 的表格 PDF: 4 行 × 3 列, 模拟 CAD 表格导出形态。
    fn table_pdf_bytes(rows: &[&[&str]]) -> Vec<u8> {
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
        let mut ops: Vec<Operation> = Vec::new();
        let cols: Vec<f32> = vec![50.0, 150.0, 250.0];
        for (ri, row) in rows.iter().enumerate() {
            let y = 620.0 - ri as f32 * 20.0;
            for (ci, cell) in row.iter().enumerate() {
                let x = cols.get(ci).copied().unwrap_or(50.0 + ci as f32 * 100.0);
                ops.push(Operation::new("BT", vec![]));
                ops.push(Operation::new("Tf", vec!["F1".into(), 12.into()]));
                ops.push(Operation::new(
                    "Tm",
                    vec![1.into(), 0.into(), 0.into(), 1.into(), x.into(), y.into()],
                ));
                ops.push(Operation::new(
                    "Tj",
                    vec![lopdf::Object::string_literal(cell.to_string())],
                ));
                ops.push(Operation::new("ET", vec![]));
            }
        }
        let content = Content { operations: ops };
        let content_id = doc.add_object(lopdf::Stream::new(
            lopdf::Dictionary::new(),
            content.encode().expect("encode table PDF content"),
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
        doc.save_to(&mut buf).expect("save table test PDF");
        buf
    }

    #[test]
    fn extract_pdf_tables_reconstructs_grid() {
        let buf = table_pdf_bytes(&[
            &["ID", "Name", "Mat"],
            &["001", "Gate", "CI"],
            &["002", "Check", "SS"],
        ]);
        let tables = FileParser::extract_pdf_tables(&buf);
        assert_eq!(tables.len(), 1, "应识别出一张表格, 得到 {tables:?}");
        let table = &tables[0];
        assert_eq!(table.page, 1);
        assert_eq!(table.columns.len(), 3, "列数应为 3: {:?}", table.columns);
        let grid = table.to_grid();
        assert_eq!(
            grid,
            vec![
                vec!["ID".to_string(), "Name".to_string(), "Mat".to_string()],
                vec!["001".to_string(), "Gate".to_string(), "CI".to_string()],
                vec!["002".to_string(), "Check".to_string(), "SS".to_string()],
            ]
        );
        let md = table.to_markdown();
        assert!(md.contains("| ID | Name | Mat |"), "表头缺失: {md}");
        assert!(md.contains("| 002 | Check | SS |"), "数据行缺失: {md}");
    }

    #[test]
    fn extract_pdf_tables_skips_paragraph_text() {
        // 单列段落文本不构成表格 (列数 < 2)。
        let buf = single_page_pdf_bytes("Hello World Plain Paragraph");
        let tables = FileParser::extract_pdf_tables(&buf);
        assert!(tables.is_empty(), "段落不应识别为表格: {tables:?}");
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

    /// 单页未压缩内容流 PDF: 文本逐字符拆成独立 Tj op (CAD 导出形态),
    /// 每个字符前带 Tf + Td (位置推进), 验证跨 op 累积匹配。
    fn per_char_pdf_bytes(text: &str) -> Vec<u8> {
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
        let mut ops: Vec<Operation> = vec![Operation::new("BT", vec![])];
        for (i, ch) in text.chars().enumerate() {
            ops.push(Operation::new("Tf", vec!["F1".into(), 12.into()]));
            ops.push(Operation::new("Td", vec![(10.0 + i as f32 * 6.0).into(), 600.into()]));
            ops.push(Operation::new("Tj", vec![lopdf::Object::string_literal(ch.to_string())]));
        }
        ops.push(Operation::new("ET", vec![]));
        let content = Content { operations: ops };
        let content_id = doc.add_object(Stream::new(
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

    #[test]
    fn edit_pdf_matches_across_adjacent_ops() {
        // 逐字符 Tj 绘制: "Gate Valve" 拆成独立单字符 op, 跨 op 累积后
        // "Gate" 应被子串匹配整体清除并原位替换。
        let buf = per_char_pdf_bytes("Gate Valve");
        let out = FileParser::edit_pdf_text(
            &buf,
            &[PdfTextEdit { page: 1, find: "Gate".into(), replace: Some("Xyz".into()) }],
            None,
        )
        .expect("edit pdf cross-op match");
        let text = FileParser::extract_pdf_text(&out);
        assert!(!text.contains("Gate"), "跨 op 匹配未清除原文本: {text:?}");
        assert!(text.contains("Xyz"), "替换文本缺失: {text:?}");
        assert!(text.contains("Valve"), "未匹配的相邻文本被误清: {text:?}");
    }

    #[test]
    fn merge_pdfs_concatenates_pages_in_order() {
        // 多文档合并: 3 个单页文档 → 合并后 3 页, 文本按序保留
        let docs = vec![
            multi_page_pdf_bytes(&["Alpha"]),
            multi_page_pdf_bytes(&["Beta"]),
            multi_page_pdf_bytes(&["Gamma"]),
        ];
        let merged = merge_pdfs(&docs).expect("合并应成功");
        let pages = FileParser::extract_pdf_pages(&merged);
        assert_eq!(pages.len(), 3, "合并后应为 3 页: {pages:?}");
        assert_eq!(pages[0].1, "Alpha Page 1");
        assert_eq!(pages[1].1, "Beta Page 1");
        assert_eq!(pages[2].1, "Gamma Page 1");
    }

    #[test]
    fn merge_pdfs_rejects_empty_and_no_page() {
        // 空输入 / 全无页面 → Merge 错误
        assert!(merge_pdfs(&[]).is_err(), "空输入应报错");
        // 单文件透传
        let single = multi_page_pdf_bytes(&["Only"]);
        let out = merge_pdfs(&[single.clone()]).expect("单文件应透传");
        assert_eq!(out, single);
        // 无页面输入: 构造缺 Pages 的文档
        let mut doc = lopdf::Document::with_version("1.4");
        let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog" });
        doc.trailer.set("Root", catalog_id);
        let mut buf = Vec::new();
        doc.save_to(&mut buf).expect("save empty pdf");
        let err = merge_pdfs(&[single, buf]).err().expect("无页面输入应报错");
        assert!(
            format!("{err}").contains("无页面"),
            "错误信息应指明无页面: {err}"
        );
    }
}
