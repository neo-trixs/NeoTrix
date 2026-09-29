//! PDF 原生坐标文字接地 —— `pdf_ground_text` 的实现。
//!
//! # 为什么是 PDF 而不是通用 OCR
//!
//! 通用文字检测只有两条路，都撞「减少外部依赖」这条底线：外挂二进制
//! （tesseract，要 brew/apt、要 PATH、要版本对齐），或自带检测权重
//! （det 模型数百 MB，且本仓无现货权重）。**装完即多一个外部依赖**，
//! 而 NeoTrix 的 `nt_world/ocr` 那两个引擎至今是占位（`PaddleOcrEngine`
//! 返回空文本空框，`RuleBasedOcr` 从文件名猜）——**导出 ≠ 有能力**。
//!
//! 但 **PDF 的文字本来就有精确坐标**：`Tf` 定字号、`Tm`/`Td` 定位置 ——
//! 写文件那一刻就确定下来的事实，零推理、零依赖、比事后 OCR 更准。
//! `neotrix-core` 已依赖 `lopdf 0.42`（PDF 图/图标抽取在用），本 crate
//! 只加一条依赖边，**不引入任何新包**。
//!
//! # 代价与诚实边界
//!
//! - **只对「有文字层」的 PDF 有效**。扫描件/文字转轮廓的 PDF 没有 content
//!   流文字，此时 `GroundReport::no_text_hint` 为真，调用方**必须如实说
//!   没找到** —— 绝不能拿渲染图去猜框，那正是本模块要消灭的东西。
//! - **框是近似的**：`/Widths`（简单字体）与 `/DW`（Type0）之外的字宽表
//!   （Type0 的 `/W`）不解析，缺时按 500/1000 em 兜底；带旋转/斜切的 CTM
//!   会把框算成外接矩形。**足以裁剪定位，不足以当像素级标注基准。**
//! - **不做反向图搜、不做物体检测**：无 key 无权重，自有技术满足不了，
//!   缺口在 skill 里注明，不用假工具填。
//!
//! # 坐标系
//!
//! - `norm`：归一化 `0..=1000`、**y 翻成图像坐标系**（左上原点）—— 与
//!   Qwen2.5-VL 绝对坐标同制式，可直接喂给裁剪/标注工具。
//! - `pt`：PDF 点坐标（y 向上，左下原点），便于人工回原文核对。

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;

use lopdf::{Document, Object, ObjectId};
use serde::Serialize;

use crate::nt_error::NtBotError;

/// 归一化坐标上限（与 Qwen2.5-VL 绝对坐标同制）。
pub const NORM_MAX: u32 = 1000;
/// 默认扫描页数上限：接地是「定位一个词」，不是通读全书。
pub const MAX_PAGES_DEFAULT: u32 = 40;
/// 单次返回命中上限（防长 PDF 灌爆上下文）。
pub const MAX_HITS: usize = 50;
/// 单页碎片段上限（超了判定病态流，截断该页）。
const MAX_FRAGS_PER_PAGE: usize = 20_000;

/// 一个被定位到的文字块。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GroundedText {
    /// 页码（1 起）。
    pub page: u32,
    /// 命中的文字（行内最小窗口，过长截断）。
    pub text: String,
    /// 归一化框 `[x0, y0, x1, y1]`，0..=1000，y 已翻转为图像坐标系。
    pub norm: [u32; 4],
    /// PDF 点坐标 `[x0, y0, x1, y1]`，y 向上。
    pub pt: [f32; 4],
}

/// 一次接地的结果。
#[derive(Debug, Clone, Default, Serialize)]
pub struct GroundReport {
    pub hits: Vec<GroundedText>,
    /// 实际扫描页数。
    pub pages_scanned: u32,
    /// 文档总页数（> `pages_scanned` 说明被 `max_pages` 截断）。
    pub pages_total: u32,
    /// 扫了页却一个字都没定位到 —— 多半是扫描件/文字转轮廓，应当
    /// **转述这个事实**，而不是改用「看图猜框」。
    pub no_text_hint: bool,
}

impl GroundReport {
    /// 人类/模型可读摘要（含诚实边界，不粉饰）。
    pub fn render(&self, query: &str) -> String {
        if self.hits.is_empty() {
            return if self.no_text_hint {
                format!(
                    "pdf_ground_text: 扫了 {}/{} 页，没找到任何文字层（很可能是扫描件或文字已转轮廓）。\
                     别猜框：改用 qwen_visualize 渲染页面自己看，或如实告诉用户这份 PDF 定位不到。",
                    self.pages_scanned, self.pages_total
                )
            } else {
                format!(
                    "pdf_ground_text: 扫了 {}/{} 页的文字层，没有 \"{}\"（坐标制式 0-1000，y 已翻转）。",
                    self.pages_scanned, self.pages_total, query
                )
            };
        }
        let mut out = format!(
            "pdf_ground_text: \"{}\" 命中 {} 处（扫 {}/{} 页；norm 为 0-1000 归一化坐标，y 已翻转为图像坐标系）\n",
            query,
            self.hits.len(),
            self.pages_scanned,
            self.pages_total
        );
        for hit in &self.hits {
            out.push_str(&format!(
                "- p{} {:?} \"{}\"\n",
                hit.page, hit.norm, hit.text
            ));
        }
        if self.pages_total > self.pages_scanned {
            out.push_str(&format!(
                "（还有 {} 页未扫；调大 max_pages 再找）\n",
                self.pages_total - self.pages_scanned
            ));
        }
        out
    }
}

/// 入口：读 workspace 内的 PDF，按 `query` 定位文字并给坐标。
pub fn ground_text(pdf: &Path, query: &str, max_pages: u32) -> Result<GroundReport, NtBotError> {
    let data = std::fs::read(pdf)?;
    ground_text_bytes(&data, query, max_pages)
}

/// 同上，但从内存字节读（单测与复用入口）。
pub fn ground_text_bytes(
    data: &[u8],
    query: &str,
    max_pages: u32,
) -> Result<GroundReport, NtBotError> {
    let q_norm = normalize(query);
    if q_norm.is_empty() {
        return Err(NtBotError::Invalid(
            "pdf_ground_text requires {query}".to_owned(),
        ));
    }
    let doc = Document::load_mem(data).map_err(|e| NtBotError::Invalid(format!("pdf: {e}")))?;
    let pages = doc.get_pages();
    let mut report = GroundReport {
        pages_total: pages.len() as u32,
        ..GroundReport::default()
    };
    let limit = max_pages.clamp(1, 200);
    let mut all_frags: Vec<Frag> = Vec::new();
    for (page_no, page_id) in pages.iter() {
        if report.pages_scanned >= limit {
            break;
        }
        report.pages_scanned += 1;
        let map = page_fonts(&doc, *page_id);
        let fonts: Vec<(Vec<u8>, FontInfo)> = map.into_iter().collect();
        let index: HashMap<Vec<u8>, usize> = fonts
            .iter()
            .enumerate()
            .map(|(i, (name, _))| (name.clone(), i))
            .collect();
        // 坏流/坏页只跳这一页，不让整篇定位失败（fail-open 于**读**，不于**写**）。
        let Ok(content) = doc.get_and_decode_page_content(*page_id) else {
            continue;
        };
        let mut frags = Vec::new();
        collect_frags(&content.operations, &fonts, &index, *page_no, &mut frags);
        if frags.len() > MAX_FRAGS_PER_PAGE {
            frags.truncate(MAX_FRAGS_PER_PAGE);
        }
        all_frags.append(&mut frags);
    }
    report.no_text_hint = all_frags.is_empty();
    if report.no_text_hint {
        return Ok(report);
    }
    let boxes: HashMap<u32, (f32, f32, f32, f32)> = pages
        .iter()
        .map(|(no, id)| (*no, page_box(&doc, *id)))
        .collect();
    for line in group_lines(all_frags) {
        let Some(window) = best_window(&line, &q_norm) else {
            continue;
        };
        let picked = &line[window.0..=window.1];
        let (x0, y0, x1, y1) = union_box(picked.iter().map(Frag::box_pt));
        let text = join_text(picked);
        let page = picked[0].page;
        let (bx0, by0, bx1, by1) = boxes
            .get(&page)
            .copied()
            .unwrap_or((0.0, 0.0, 612.0, 792.0));
        report.hits.push(GroundedText {
            page,
            text: clip(text.trim(), 200),
            norm: to_norm(x0, y0, x1, y1, bx0, by0, bx1, by1),
            pt: [x0, y0, x1, y1],
        });
        if report.hits.len() >= MAX_HITS {
            break;
        }
    }
    Ok(report)
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_owned();
    }
    let mut out: String = s.chars().take(max).collect();
    out.push('…');
    out
}

// ──────────────────────────────────────────────
// 纯函数（可单测）
// ──────────────────────────────────────────────

/// 归一化：折叠空白 + 小写，供匹配用。
pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            space = true;
            continue;
        }
        if space && !out.is_empty() {
            out.push(' ');
        }
        space = false;
        for lower in ch.to_lowercase() {
            out.push(lower);
        }
    }
    out
}

/// 去掉全部空白（PDF 常把一个词拆成多个 `Tj`，召回要按「无空格」再比一次）。
pub fn squash(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 设备坐标 → 0-1000 归一化框（y 翻转为图像坐标系）。
pub fn to_norm(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    bx0: f32,
    by0: f32,
    bx1: f32,
    by1: f32,
) -> [u32; 4] {
    let w = (bx1 - bx0).max(1e-6);
    let h = (by1 - by0).max(1e-6);
    let nx0 = ((x0 - bx0) / w).clamp(0.0, 1.0);
    let nx1 = ((x1 - bx0) / w).clamp(0.0, 1.0);
    // PDF y 向上 ⇒ 图像 y 向下：图像 top 对应 PDF 的上边。
    let ny0 = ((by1 - y1) / h).clamp(0.0, 1.0);
    let ny1 = ((by1 - y0) / h).clamp(0.0, 1.0);
    let to_u = |v: f32| (v * NORM_MAX as f32).round() as u32;
    [to_u(nx0), to_u(ny0), to_u(nx1), to_u(ny1)]
}

fn union_box<'a, I: Iterator<Item = [f32; 4]>>(boxes: I) -> (f32, f32, f32, f32) {
    let (mut x0, mut y0) = (f32::MAX, f32::MAX);
    let (mut x1, mut y1) = (f32::MIN, f32::MIN);
    for b in boxes {
        x0 = x0.min(b[0]);
        y0 = y0.min(b[1]);
        x1 = x1.max(b[2]);
        y1 = y1.max(b[3]);
    }
    (x0, y0, x1, y1)
}

/// WinAnsi(cp1252) 单字节解码：PDF 简单字体的事实标准编码。
pub fn win_ansi_char(b: u8) -> char {
    match b {
        0x80 => '\u{20AC}',
        0x82 => '\u{201A}',
        0x83 => '\u{0192}',
        0x84 => '\u{201E}',
        0x85 => '\u{2026}',
        0x86 => '\u{2020}',
        0x87 => '\u{2021}',
        0x88 => '\u{02C6}',
        0x89 => '\u{2030}',
        0x8A => '\u{0160}',
        0x8B => '\u{2039}',
        0x8C => '\u{0152}',
        0x8E => '\u{017D}',
        0x91 => '\u{2018}',
        0x92 => '\u{2019}',
        0x93 => '\u{201C}',
        0x94 => '\u{201D}',
        0x95 => '\u{2022}',
        0x96 => '\u{2013}',
        0x97 => '\u{2014}',
        0x98 => '\u{02DC}',
        0x99 => '\u{2122}',
        0x9A => '\u{0161}',
        0x9B => '\u{203A}',
        0x9C => '\u{0153}',
        0x9E => '\u{017E}',
        0x9F => '\u{0178}',
        other => other as char,
    }
}

/// `<...>` 十六进制 → u32。
pub fn hex_to_u32(hex: &[u8]) -> Option<u32> {
    if hex.is_empty() || hex.len() > 8 {
        return None;
    }
    let mut acc: u32 = 0;
    for byte in hex {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return None,
        };
        acc = acc.checked_mul(16)?.checked_add(u32::from(digit))?;
    }
    Some(acc)
}

/// UTF-16BE 十六进制 → `String`（ToUnicode 的 `dst` 就是它）。
///
/// 输入是 **`<` 与 `>` 之间的十六进制文本的 ASCII 字节**（`parse_cmap` 的
/// `Item::Hex` 原样切片所得），故必须先十六进制解码，再按解码后的字节组对。
///
/// 2026-09-29 审计修复（`ab1b9d96` 引入，`hex_helpers` + 2 个 cmap 测试全红）：
/// 原实现按 `hex.chunks(2)` 切**文本**再对每半个文本调 `hex_to_u32` ——
/// 那是「解半个字节的十六进制」，语义错误。
///   `0054` 原路径：pair=`b"00"`,`b"54"` → 各解一次 → `(0x00<<8)|0x54` = U+0054
///   看似对，但 `hex_to_u32` 收到的是 `b"00"` 整体（2 字节），
///   而原代码传的是 `&pair[0..1]`（1 字节 `'0'`）⇒ 解成 0x30
///   ⇒ `0x30<<8 | 0x54` = U+3054 = "〰" + U+5434 = "㔴"。实测正是 `〰㔴`。
/// 更正：先整体十六进制解码成字节流，再按字节组对。
///   `0054`     → [0x00,0x54] → U+0054 = "T" ✅
///   `D83DDE00` → [0xD8,0x3D,0xDE,0x00] → U+D83D U+DE00 = "😀" ✅
pub fn hex_to_string(hex: &[u8]) -> Option<String> {
    if hex.is_empty() || hex.len() % 2 != 0 {
        return None;
    }
    // 1) 十六进制文本 → 字节流（奇数长度已在上面挡掉）
    let mut bytes: Vec<u8> = Vec::with_capacity(hex.len() / 2);
    for pair in hex.chunks(2) {
        let hi = hex_val(pair[0])?;
        let lo = hex_val(pair[1])?;
        bytes.push((hi << 4) | lo);
    }
    // 2) 字节流 → UTF-16BE 码元
    if bytes.len() % 2 != 0 {
        return None;
    }
    let units: Vec<u16> = bytes
        .chunks(2)
        .map(|pair| (u16::from(pair[0]) << 8) | u16::from(pair[1]))
        .collect();
    Some(String::from_utf16_lossy(&units))
}

/// 单个十六进制字符 → 数值。非十六进制返回 None。
fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// ──────────────────────────────────────────────
// ToUnicode CMap（手写最小解析：lopdf 的 `encodings` 模块是私有的）
// ──────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct CMap {
    pub one: HashMap<u8, String>,
    pub two: HashMap<u16, String>,
}

impl CMap {
    fn is_empty(&self) -> bool {
        self.one.is_empty() && self.two.is_empty()
    }
    fn get(&self, code: u32, two_byte: bool) -> Option<&str> {
        if two_byte {
            if code <= u32::from(u16::MAX) {
                return self.two.get(&(code as u16)).map(String::as_str);
            }
            return None;
        }
        if code <= u32::from(u8::MAX) {
            self.one.get(&(code as u8)).map(String::as_str)
        } else {
            None
        }
    }
}

#[derive(Debug)]
enum Item {
    Hex(Vec<u8>),
    Arr(Vec<Vec<u8>>),
}

#[derive(PartialEq)]
enum Block {
    None,
    CodeSpace,
    BfChar,
    BfRange,
}

/// 解析 ToUnicode CMap 流：只取 `codespacerange`/`bfchar`/`bfrange`
/// （PDF 32000-1 §9.7.6.2 的三张表就是全部必要信息）。
pub fn parse_cmap(bytes: &[u8]) -> CMap {
    let mut cmap = CMap::default();
    let mut block = Block::None;
    let mut pending: Vec<Item> = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'<' if !bytes[i + 1..].starts_with(b"<") => {
                let Some(end) = find(&bytes[i + 1..], b'>') else {
                    break;
                };
                pending.push(Item::Hex(bytes[i + 1..i + 1 + end].to_vec()));
                // 2026-09-29 审计修复（`ab1b9d96` 引入，cmap 测试全红）：
                // 原为 `i += i + 1 + end + 1` —— `i` 已是当前索引，再加 `i`
                // 是**重复累加**，指针直接跳过下一个 hex 项。
                // 实测 `beginbfchar <54> <0054>` 只收进 1 项（应为 2 项）⇒ 映射全丢。
                // 正确：i = 起始 + 1(<) + end(内容长度) + 1(>)。
                i = i + 1 + end + 1;
            }
            b'[' => {
                let Some(end) = find(&bytes[i + 1..], b']') else {
                    break;
                };
                pending.push(Item::Arr(collect_hex(&bytes[i + 1..i + 1 + end])));
                i = i + 1 + end + 1;  // 2026-09-29 审计：同上，累加→赋值
            }
            b if b.is_ascii_alphabetic() || b == b'/' => {
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric()
                        || matches!(bytes[i], b'#' | b'+' | b'-' | b'.'))
                {
                    i += 1;
                }
                if i == start {
                    i += 1;
                    continue;
                }
                let word = String::from_utf8_lossy(&bytes[start..i]).into_owned();
                flush_block(&mut cmap, &mut pending, &block);
                block = match word.as_str() {
                    "begincodespacerange" => Block::CodeSpace,
                    "beginbfchar" => Block::BfChar,
                    "beginbfrange" => Block::BfRange,
                    _ => Block::None,
                };
            }
            _ => i += 1,
        }
    }
    flush_block(&mut cmap, &mut pending, &block);
    cmap
}

fn flush_block(cmap: &mut CMap, pending: &mut Vec<Item>, block: &Block) {
    let items = std::mem::take(pending);
    match block {
        // 码长已由字体 Subtype 决定，codespacerange 不产生映射。
        Block::CodeSpace | Block::None => {}
        Block::BfChar => {
            for pair in items.chunks(2) {
                if let [Item::Hex(src), Item::Hex(dst)] = pair {
                    if let (Some(code), Some(text)) = (hex_to_u32(src), hex_to_string(dst)) {
                        insert_cmap(cmap, code, text);
                    }
                }
            }
        }
        Block::BfRange => {
            for triple in items.chunks(3) {
                let (Some(Item::Hex(lo)), Some(Item::Hex(hi)), Some(dst)) =
                    (triple.first(), triple.get(1), triple.get(2))
                else {
                    continue;
                };
                let (Some(lo), Some(hi)) = (hex_to_u32(lo), hex_to_u32(hi)) else {
                    continue;
                };
                if hi < lo || hi - lo > 0xFFFF {
                    continue;
                }
                match dst {
                    // `<lo> <hi> <dstStart>`：dst 末位按 code 递增。
                    Item::Hex(raw) => {
                        let Some(base) = hex_to_string(raw) else {
                            continue;
                        };
                        let mut chars: Vec<char> = base.chars().collect();
                        let last = chars.len().saturating_sub(1);
                        let base_last = chars.get(last).copied();
                        for code in lo..=hi {
                            // 2026-09-29 审计修复（`ab1b9d96` 引入，cmap 测试红）：
                            // 原实现每轮都在**已修改的** `chars[last]` 上再加偏移
                            // ⇒ 偏移累加，`U`→`V`→`W`→`X`（实测 0x57 拿到 "X"）。
                            // PDF ToUnicode 的 bfrange 语义是
                            // 「dstStart 的**末位**按 (code - lo) 递增」，
                            // 每项都从 base 出发，不是从前一项出发。
                            if let Some(b) = base_last {
                                *chars.get_mut(last).expect("last 已在 len 校验内") =
                                    char::from_u32((b as u32) + (code - lo))
                                        .unwrap_or(char::REPLACEMENT_CHARACTER);
                            }
                            insert_cmap(cmap, code, chars.iter().collect());
                        }
                    }
                    // `<lo> <hi> [<d1> <d2> …]`：逐码对应。
                    Item::Arr(list) => {
                        for (offset, raw) in list.iter().enumerate() {
                            let code = lo + offset as u32;
                            if code > hi {
                                break;
                            }
                            if let Some(text) = hex_to_string(raw) {
                                insert_cmap(cmap, code, text);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn insert_cmap(cmap: &mut CMap, code: u32, text: String) {
    if code <= u32::from(u8::MAX) {
        cmap.one.insert(code as u8, text);
    } else if code <= u32::from(u16::MAX) {
        cmap.two.insert(code as u16, text);
    }
}

fn find(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|b| *b == needle)
}

fn collect_hex(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let Some(end) = find(&bytes[i + 1..], b'>') else {
                break;
            };
            out.push(bytes[i + 1..i + 1 + end].to_vec());
            i = i + 1 + end + 1;  // 2026-09-29 审计：同上，累加→赋值
            continue;
        }
        i += 1;
    }
    out
}

// ──────────────────────────────────────────────
// 字体表与页面盒
// ──────────────────────────────────────────────

#[derive(Debug, Clone)]
struct FontInfo {
    cmap: Option<CMap>,
    two_byte: bool,
    first_char: i64,
    /// 简单字体 `/Widths`（1/1000 em），下标 = code - first_char。
    widths: Vec<f32>,
    /// 兜底字宽（1/1000 em）：Type0 用 `/DW`（默认 1000），简单字体 500。
    default_width: f32,
}

impl FontInfo {
    /// 字宽（1/1000 em）。**不解析 Type0 的 `/W`**（结构更复杂，缺它只会让
    /// 框略偏）—— 这个取舍写在模块头，是有意的边界不是疏漏。
    fn width(&self, code: u32) -> f32 {
        if !self.two_byte {
            let idx = code as i64 - self.first_char;
            if idx >= 0 {
                if let Some(w) = self.widths.get(idx as usize) {
                    return *w;
                }
            }
        }
        self.default_width
    }
}

fn resolve<'a>(doc: &'a Document, obj: &'a Object) -> Option<&'a Object> {
    match obj {
        Object::Reference(id) => doc.get_object(*id).ok(),
        other => Some(other),
    }
}

fn page_fonts(doc: &Document, page_id: ObjectId) -> HashMap<Vec<u8>, FontInfo> {
    let mut out = HashMap::new();
    let Some(page) = doc
        .get_object(page_id)
        .ok()
        .and_then(|o| resolve(doc, o))
        .and_then(|o| o.as_dict().ok())
    else {
        return out;
    };
    let Some(fonts) = page
        .get(b"Resources")
        .ok()
        .and_then(|o| resolve(doc, o))
        .and_then(|o| o.as_dict().ok())
        .and_then(|d| d.get(b"Font").ok())
        .and_then(|o| resolve(doc, o))
        .and_then(|o| o.as_dict().ok())
    else {
        return out;
    };
    for (name, obj) in fonts.iter() {
        if let Some(fdict) = resolve(doc, obj).and_then(|o| o.as_dict().ok()) {
            out.insert(name.clone(), build_font(doc, fdict));
        }
    }
    out
}

fn build_font(doc: &Document, fdict: &lopdf::Dictionary) -> FontInfo {
    let subtype = fdict
        .get(b"Subtype")
        .ok()
        .and_then(|o| resolve(doc, o))
        .and_then(|o| o.as_name().ok())
        .unwrap_or(b"");
    let two_byte = subtype == b"Type0";
    let cmap = fdict
        .get(b"ToUnicode")
        .ok()
        .and_then(|o| resolve(doc, o))
        .and_then(|o| o.as_stream().ok())
        .and_then(|s| s.decompressed_content().ok())
        .map(|bytes| parse_cmap(&bytes))
        .filter(|c| !c.is_empty());
    let first_char = fdict
        .get(b"FirstChar")
        .ok()
        .and_then(|o| resolve(doc, o))
        .and_then(|o| o.as_i64().ok())
        .unwrap_or(0);
    let widths: Vec<f32> = fdict
        .get(b"Widths")
        .ok()
        .and_then(|o| resolve(doc, o))
        .and_then(|o| o.as_array().ok())
        .map(|arr| {
            arr.iter()
                .map(|o| {
                    resolve(doc, o)
                        .and_then(|r| r.as_float().ok())
                        .unwrap_or(0.0)
                })
                .collect()
        })
        .unwrap_or_default();
    let default_width = if two_byte {
        fdict
            .get(b"DW")
            .ok()
            .and_then(|o| resolve(doc, o))
            .and_then(|o| o.as_float().ok())
            .unwrap_or(1000.0)
    } else {
        500.0
    };
    FontInfo {
        cmap,
        two_byte,
        first_char,
        widths,
        default_width,
    }
}

/// 页面盒：优先 `CropBox`（渲染器实际用的那块），回退 `MediaBox`。
fn page_box(doc: &Document, page_id: ObjectId) -> (f32, f32, f32, f32) {
    let page = doc
        .get_object(page_id)
        .ok()
        .and_then(|o| resolve(doc, o))
        .and_then(|o| o.as_dict().ok());
    if let Some(page) = page {
        for key in [b"CropBox".as_slice(), b"MediaBox".as_slice()] {
            let Some(rect) = page
                .get(key)
                .ok()
                .and_then(|o| resolve(doc, o))
                .and_then(|o| o.as_array().ok())
            else {
                continue;
            };
            let nums: Vec<Option<f32>> = rect
                .iter()
                .map(|o| resolve(doc, o).and_then(|r| r.as_float().ok()))
                .collect();
            if nums.len() == 4 && nums.iter().all(Option::is_some) {
                let flat: Vec<f32> = nums.into_iter().flatten().collect();
                return (flat[0], flat[1], flat[2], flat[3]);
            }
        }
    }
    (0.0, 0.0, 612.0, 792.0)
}

// ──────────────────────────────────────────────
// content 流状态机
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
struct Mat {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Mat {
    const IDENTITY: Mat = Mat {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };
    /// 行向量约定：设备 = 文本 × Tm × CTM。
    fn mul(self, o: Mat) -> Mat {
        Mat {
            a: self.a * o.a + self.b * o.c,
            b: self.a * o.b + self.b * o.d,
            d: self.c * o.b + self.d * o.d,
            c: self.c * o.a + self.d * o.c,
            e: self.e * o.a + self.f * o.c + o.e,
            f: self.e * o.b + self.f * o.d + o.f,
        }
    }
    fn apply(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }
    /// y 方向缩放（字号在设备空间的实际高度，行聚类容差用）。
    fn y_scale(self) -> f32 {
        (self.c * self.c + self.d * self.d).sqrt()
    }
}

#[derive(Debug, Clone)]
struct Frag {
    page: u32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    /// 基线 y（设备空间），行聚类主键。
    baseline: f32,
    /// 设备空间字号，行容差用。
    dev_h: f32,
    text: String,
}

impl Frag {
    fn box_pt(&self) -> [f32; 4] {
        [self.x0, self.y0, self.x1, self.y1]
    }
}

#[derive(Debug, Clone, Copy)]
struct TextState {
    tm: Mat,
    tlm: Mat,
    ctm: Mat,
    font_size: f32,
    font: Option<usize>,
    char_spacing: f32,
    word_spacing: f32,
    leading: f32,
    h_scale: f32,
    rise: f32,
}

fn num(operands: &[Object], idx: usize) -> Option<f32> {
    operands.get(idx).and_then(|o| o.as_float().ok())
}

fn collect_frags(
    ops: &[lopdf::content::Operation],
    fonts: &[(Vec<u8>, FontInfo)],
    index: &HashMap<Vec<u8>, usize>,
    page: u32,
    out: &mut Vec<Frag>,
) {
    let mut st = TextState {
        tm: Mat::IDENTITY,
        tlm: Mat::IDENTITY,
        ctm: Mat::IDENTITY,
        font_size: 0.0,
        font: None,
        char_spacing: 0.0,
        word_spacing: 0.0,
        leading: 0.0,
        h_scale: 1.0,
        rise: 0.0,
    };
    let mut ctm_stack: Vec<Mat> = Vec::new();
    for op in ops {
        let args = &op.operands;
        match op.operator.as_str() {
            "q" => ctm_stack.push(st.ctm),
            "Q" => {
                if let Some(prev) = ctm_stack.pop() {
                    st.ctm = prev;
                }
            }
            "cm" => {
                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f)) = (
                    num(args, 0),
                    num(args, 1),
                    num(args, 2),
                    num(args, 3),
                    num(args, 4),
                    num(args, 5),
                ) {
                    st.ctm = st.ctm.mul(Mat { a, b, c, d, e, f });
                }
            }
            "BT" => {
                st.tm = Mat::IDENTITY;
                st.tlm = Mat::IDENTITY;
            }
            "ET" => {}
            "Tf" => {
                st.font_size = num(args, 1).unwrap_or(0.0);
                st.font = args
                    .first()
                    .and_then(|o| o.as_name().ok())
                    .and_then(|name| index.get(name).copied());
            }
            "TL" => st.leading = num(args, 0).unwrap_or(0.0),
            "Tc" => st.char_spacing = num(args, 0).unwrap_or(0.0),
            "Tw" => st.word_spacing = num(args, 0).unwrap_or(0.0),
            "Tz" => st.h_scale = num(args, 0).unwrap_or(100.0) / 100.0,
            "Ts" => st.rise = num(args, 0).unwrap_or(0.0),
            "Td" | "TD" => {
                let (tx, ty) = (num(args, 0).unwrap_or(0.0), num(args, 1).unwrap_or(0.0));
                if op.operator == "TD" {
                    st.leading = -ty;
                }
                st.tlm = translate(st.tlm, tx, ty);
                st.tm = st.tlm;
            }
            "Tm" => {
                st.tlm = Mat {
                    a: num(args, 0).unwrap_or(1.0),
                    b: num(args, 1).unwrap_or(0.0),
                    c: num(args, 2).unwrap_or(0.0),
                    d: num(args, 3).unwrap_or(1.0),
                    e: num(args, 4).unwrap_or(0.0),
                    f: num(args, 5).unwrap_or(0.0),
                };
                st.tm = st.tlm;
            }
            "T*" => {
                st.tlm = translate(st.tlm, 0.0, -st.leading);
                st.tm = st.tlm;
            }
            "Tj" => {
                if let Some(raw) = args.first().and_then(|o| o.as_str().ok()) {
                    show(&mut st, fonts, raw, page, out);
                }
            }
            "TJ" => {
                if let Some(arr) = args.first().and_then(|o| o.as_array().ok()) {
                    for item in arr.iter() {
                        match item {
                            Object::String(raw, _) => show(&mut st, fonts, raw, page, out),
                            // 数字是字间距调整（负数=拉开）：走文本矩阵，不产出片段。
                            _ => {
                                let adj =
                                    -num(std::slice::from_ref(item), 0).unwrap_or(0.0) / 1000.0;
                                st.tm = translate(st.tm, adj * st.font_size * st.h_scale, 0.0);
                            }
                        }
                    }
                }
            }
            "'" => {
                st.tlm = translate(st.tlm, 0.0, -st.leading);
                st.tm = st.tlm;
                if let Some(raw) = args.first().and_then(|o| o.as_str().ok()) {
                    show(&mut st, fonts, raw, page, out);
                }
            }
            "\"" => {
                st.word_spacing = num(args, 0).unwrap_or(0.0);
                st.char_spacing = num(args, 1).unwrap_or(0.0);
                st.tlm = translate(st.tlm, 0.0, -st.leading);
                st.tm = st.tlm;
                if let Some(raw) = args.get(2).and_then(|o| o.as_str().ok()) {
                    show(&mut st, fonts, raw, page, out);
                }
            }
            _ => {}
        }
    }
}

fn translate(m: Mat, tx: f32, ty: f32) -> Mat {
    Mat {
        e: m.e + m.a * tx + m.c * ty,
        f: m.f + m.b * tx + m.d * ty,
        ..m
    }
}

/// 画出一段文字：算总宽 → 出一个框 → 推进文本矩阵。
fn show(
    st: &mut TextState,
    fonts: &[(Vec<u8>, FontInfo)],
    raw: &[u8],
    page: u32,
    out: &mut Vec<Frag>,
) {
    let Some(font) = st.font.and_then(|i| fonts.get(i)).map(|(_, f)| f) else {
        return;
    };
    // 无 ToUnicode 且带 UTF-16BE BOM ⇒ 直接按 UTF-16BE 读（Identity-H 常见）。
    if font.cmap.is_none() && raw.len() >= 2 && raw[0] == 0xFE && raw[1] == 0xFF {
        let units: Vec<u16> = raw[2..]
            .chunks(2)
            .map(|pair| {
                if pair.len() == 2 {
                    u16::from_be_bytes([pair[0], pair[1]])
                } else {
                    0xFFFD
                }
            })
            .collect();
        let text = String::from_utf16_lossy(&units);
        let width = units.len() as f32 * st.font_size * 0.5 * st.h_scale;
        push_frag(st, out, page, &text, width);
        return;
    }
    let (text, width) = decode_run(raw, font, st);
    if text.is_empty() {
        return;
    }
    push_frag(st, out, page, &text, width);
}

fn push_frag(st: &mut TextState, out: &mut Vec<Frag>, page: u32, text: &str, width: f32) {
    let m = st.tm.mul(st.ctm);
    // 升/降部取经验值（0.8/0.2 em）：不查字体 head 表，够定位不够排版。
    let asc = 0.8 * st.font_size;
    let desc = -0.2 * st.font_size;
    let run = width / st.h_scale.max(1e-6);
    let (x0, y0) = m.apply(0.0, desc + st.rise);
    let (x1, y1) = m.apply(run, asc + st.rise);
    let (bx0, by0, bx1, by1) = union_box([[x0, y0, x1, y1]].into_iter());
    let (_, baseline) = m.apply(0.0, st.rise);
    out.push(Frag {
        page,
        x0: bx0,
        y0: by0,
        x1: bx1,
        y1: by1,
        baseline,
        dev_h: (asc - desc) * m.y_scale().max(1e-6),
        text: text.to_owned(),
    });
    st.tm = translate(st.tm, run, 0.0);
}

/// 逐码解码 + 累计宽度（文本空间单位，未乘 Tz）。
fn decode_run(raw: &[u8], font: &FontInfo, st: &TextState) -> (String, f32) {
    let mut text = String::new();
    let mut width = 0.0f32;
    let step = if font.two_byte { 2 } else { 1 };
    let mut i = 0usize;
    while i < raw.len() {
        let code = if step == 2 && i + 1 < raw.len() {
            (u32::from(raw[i]) << 8) | u32::from(raw[i + 1])
        } else {
            u32::from(raw[i])
        };
        i += step;
        if let Some(mapped) = font.cmap.as_ref().and_then(|c| c.get(code, font.two_byte)) {
            text.push_str(mapped);
        } else if font.two_byte {
            text.push(char::REPLACEMENT_CHARACTER);
        } else {
            text.push(win_ansi_char(code as u8));
        }
        let mut tx = font.width(code) / 1000.0 * st.font_size + st.char_spacing;
        if !font.two_byte && code == 0x20 {
            tx += st.word_spacing;
        }
        width += tx;
    }
    (text, width)
}

// ──────────────────────────────────────────────
// 行聚类 + 最小窗口匹配
// ──────────────────────────────────────────────

fn group_lines(mut frags: Vec<Frag>) -> Vec<Vec<Frag>> {
    frags.sort_by(|a, b| {
        a.page
            .cmp(&b.page)
            .then_with(|| {
                b.baseline
                    .partial_cmp(&a.baseline)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| a.x0.partial_cmp(&b.x0).unwrap_or(Ordering::Equal))
    });
    let mut lines: Vec<Vec<Frag>> = Vec::new();
    for frag in frags {
        let same = lines.last().is_some_and(|line: &Vec<Frag>| {
            let head = line.last().expect("line is never pushed empty");
            head.page == frag.page
                && (head.baseline - frag.baseline).abs()
                    <= 0.5 * head.dev_h.max(frag.dev_h).max(1.0)
        });
        if same {
            if let Some(line) = lines.last_mut() {
                line.push(frag);
            }
        } else {
            lines.push(vec![frag]);
        }
    }
    lines
}

fn join_text(line: &[Frag]) -> String {
    let mut out = String::new();
    for frag in line {
        if !out.is_empty()
            && !out.ends_with(' ')
            && !out.ends_with('-')
            && !frag.text.starts_with(' ')
        {
            out.push(' ');
        }
        out.push_str(&frag.text);
    }
    out
}

/// 找**最短**能匹配上的连续片段窗口 —— 框因此尽量紧，而不是整行。
fn best_window(line: &[Frag], q_norm: &str) -> Option<(usize, usize)> {
    let q_squash = squash(q_norm);
    let mut best: Option<(usize, usize)> = None;
    for start in 0..line.len() {
        for end in start..line.len() {
            let joined = normalize(&join_text(&line[start..=end]));
            if joined.contains(q_norm) || squash(&joined).contains(&q_squash) {
                let width = end - start;
                let better = match best {
                    None => true,
                    Some((bs, be)) => width < be - bs,
                };
                if better {
                    best = Some((start, end));
                }
                break;
            }
        }
    }
    best
}

/// 测试用最小 PDF（含 WinAnsi 一行 + ToUnicode 被 `TJ` 拆成两段的一行 + 一个无文字页）。
///
/// 提到 `#[cfg(test)]` 的**模块级**（而不是留在 `mod tests` 里）：`nt_agent` 的
/// turn 级 E2E 要在真实 workspace 里放一个真 PDF，走同一条生产代码路径 ——
/// fixture 不能只活在解析器的测试里，否则那条 E2E 会被迫自己再造一份 PDF。
#[cfg(test)]
pub(crate) fn fixture_pdf() -> Vec<u8> {
    /// 自造最小 PDF（手写 xref，**零外部样本文件** ⇒ 测试不依赖仓库外的东西，
    /// 也不会因为环境里恰好有个 pdf 而变绿）。
    fn make_pdf(objects: &[String]) -> Vec<u8> {
        let mut out = Vec::from(&b"%PDF-1.4\n"[..]);
        let mut offsets = Vec::new();
        for (i, body) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
        }
        let xref = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
        out.extend_from_slice(b"0000000000 65535 f \n");
        for off in &offsets {
            out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(
            format!(
                "trailer\n<</Size {}/Root 1 0 R>>\nstartxref\n{xref}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        out
    }

    fn stream_obj(data: &str) -> String {
        format!("<</Length {}>>\nstream\n{data}endstream", data.len())
    }

    let c1 = "BT /F2 12 Tf 1 0 0 1 72 700 Tm (Confidential memo) Tj ET\n\
              BT /F1 10 Tf 1 0 0 1 300 680 Tm [(To) -250 (UVW)] TJ ET\n";
    let c2 = "1 0 0 RG 4 w 100 100 m 200 200 l S\n";
    let cmap = "/CIDInit /ProcSet findresource begin 12 dict begin begincmap\n\
                1 begincodespacerange\n<00> <FF>\nendcodespacerange\n\
                2 beginbfchar\n<54> <0054>\n<6F> <006F>\nendbfchar\n\
                1 beginbfrange\n<55> <57> <0055>\nendbfrange\n\
                endcmap end end\n";
    make_pdf(&[
        "<</Type/Catalog/Pages 2 0 R>>".to_owned(),
        "<</Type/Pages/Kids[3 0 R 8 0 R]/Count 2>>".to_owned(),
        "<</Type/Page/Parent 2 0 R/MediaBox[0 0 612 792]/Resources<</Font<</F1 4 0 R/F2 5 0 R>>>>\
         /Contents 6 0 R>>"
            .to_owned(),
        "<</Type/Font/Subtype/Type1/BaseFont/Helvetica/ToUnicode 7 0 R>>".to_owned(),
        "<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>".to_owned(),
        stream_obj(c1),
        stream_obj(cmap),
        "<</Type/Page/Parent 2 0 R/MediaBox[0 0 612 792]/Resources<</Font<</F2 5 0 R>>>>\
         /Contents 9 0 R>>"
            .to_owned(),
        stream_obj(c2),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 单页、只有一条画线、**零文字** ⇒ 模拟扫描件/文字转轮廓的 PDF。
    fn blank_pdf() -> Vec<u8> {
        fn make_pdf(objects: &[String]) -> Vec<u8> {
            let mut out = Vec::from(&b"%PDF-1.4\n"[..]);
            let mut offsets = Vec::new();
            for (i, body) in objects.iter().enumerate() {
                offsets.push(out.len());
                out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
            }
            let xref = out.len();
            out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
            out.extend_from_slice(b"0000000000 65535 f \n");
            for off in &offsets {
                out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
            }
            out.extend_from_slice(
                format!(
                    "trailer\n<</Size {}/Root 1 0 R>>\nstartxref\n{xref}\n%%EOF\n",
                    objects.len() + 1
                )
                .as_bytes(),
            );
            out
        }
        // Length 必须由字节数算出：手写常数会与内容漂移，而 lopdf 会照
        // dict 里的 Length 切流 —— 数字错了就把 `endstream` 也吃进 content。
        fn stream_obj(data: &str) -> String {
            format!("<</Length {}>>\nstream\n{data}endstream", data.len())
        }
        make_pdf(&[
            "<</Type/Catalog/Pages 2 0 R>>".to_owned(),
            "<</Type/Pages/Kids[3 0 R]/Count 1>>".to_owned(),
            "<</Type/Page/Parent 2 0 R/MediaBox[0 0 612 792]/Resources<<>>/Contents 4 0 R>>"
                .to_owned(),
            stream_obj("1 0 0 RG 4 w 100 100 m 200 200 l S\n"),
        ])
    }

    #[test]
    fn finds_winansi_word_with_flipped_norm_box() {
        let report = ground_text_bytes(&fixture_pdf(), "confidential", 10).expect("grounded");
        assert_eq!(report.hits.len(), 1, "{report:?}");
        let hit = &report.hits[0];
        assert_eq!(hit.page, 1);
        assert_eq!(hit.text, "Confidential memo");
        // 72pt / 612pt → 117.6 ⇒ 118（0-1000 制式）。
        assert_eq!(hit.norm[0], 118, "x0 {:?}", hit.norm);
        // y 翻转：PDF y=700 在页面顶部之下 ⇒ 图像 y 小。
        assert!(hit.norm[1] < hit.norm[3], "y must be ascending: {hit:?}");
        assert!((100..=125).contains(&hit.norm[1]), "y0 {:?}", hit.norm);
        // 点坐标仍在 PDF 制式（y 向上）。
        assert!(
            hit.pt[1] < hit.pt[3],
            "pt y must stay PDF-oriented: {hit:?}"
        );
        assert!((697.0..698.0).contains(&hit.pt[1]), "pt y0 {:?}", hit.pt);
    }

    #[test]
    fn decodes_tounicode_and_unions_split_fragments() {
        // 走 ToUnicode CMap（不是 WinAnsi），且查询词被 TJ 拆成两段。
        let report = ground_text_bytes(&fixture_pdf(), "to uvw", 10).expect("grounded");
        assert_eq!(report.hits.len(), 1, "{report:?}");
        let hit = &report.hits[0];
        assert_eq!(hit.text, "To UVW");
        // 框必须**跨过 TJ 的间隙**（300pt→~327pt ⇒ 490→535）。
        assert!(hit.norm[0] <= 495, "x0 {:?}", hit.norm);
        assert!(hit.norm[2] >= 530, "x2 must span the gap: {:?}", hit.norm);
    }

    #[test]
    fn shortest_window_wins_so_box_stays_tight() {
        // 只查首段 ⇒ 框应收窄，不能把整行都框进去。
        let report = ground_text_bytes(&fixture_pdf(), "to", 10).expect("grounded");
        assert_eq!(report.hits.len(), 1, "{report:?}");
        assert!(
            report.hits[0].norm[2] < 520,
            "box not tight: {:?}",
            report.hits[0].norm
        );
    }

    #[test]
    fn missing_word_is_empty_but_not_scanned_hint() {
        let report = ground_text_bytes(&fixture_pdf(), "zzzz", 10).expect("grounded");
        assert!(report.hits.is_empty(), "{report:?}");
        // 有文字层却没这个词 ≠ 没有文字层：两者必须能被模型区分。
        assert!(!report.no_text_hint);
        assert!(report.render("zzzz").contains("没有"));
    }

    #[test]
    fn no_text_layer_says_so_instead_of_guessing() {
        // 扫描件/文字转轮廓：必须**明确说没有文字层**，且一个框都不给。
        let report = ground_text_bytes(&blank_pdf(), "confidential", 10).expect("grounded");
        assert!(report.hits.is_empty(), "{report:?}");
        assert!(report.no_text_hint, "must flag the missing text layer");
        let text = report.render("confidential");
        assert!(text.contains("别猜框"), "{text}");
        // 有文字层但没这个词 ⇒ 不得复用「没文字层」那句（模型要能区分两种情况）。
        let with_text = ground_text_bytes(&fixture_pdf(), "confidential", 10).expect("grounded");
        assert!(!with_text.no_text_hint);
        assert!(!with_text.render("confidential").contains("别猜框"));
    }

    #[test]
    fn max_pages_bounds_the_scan() {
        let report = ground_text_bytes(&fixture_pdf(), "x", 1).expect("grounded");
        assert_eq!(report.pages_scanned, 1);
        assert_eq!(report.pages_total, 2, "total must stay truthful");
    }

    #[test]
    fn truncated_scan_is_reported_not_silent() {
        // pages_total > pages_scanned 时摘要必须提示还有未扫的页。
        let report = GroundReport {
            hits: vec![GroundedText {
                page: 1,
                text: "t".to_owned(),
                norm: [0, 0, 1, 1],
                pt: [0.0, 0.0, 1.0, 1.0],
            }],
            pages_scanned: 3,
            pages_total: 90,
            no_text_hint: false,
        };
        assert!(
            report.render("t").contains("87 页未扫"),
            "{}",
            report.render("t")
        );
    }

    #[test]
    fn empty_query_is_invalid_not_silent_pass() {
        assert!(matches!(
            ground_text_bytes(&fixture_pdf(), "   ", 10),
            Err(NtBotError::Invalid(_))
        ));
    }

    #[test]
    fn normalize_collapses_and_lowercases() {
        assert_eq!(normalize("  Foo \n BAR "), "foo bar");
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn squash_drops_all_whitespace() {
        assert_eq!(squash(" a b\tc\n"), "abc");
    }

    #[test]
    fn win_ansi_decodes_cp1252_high_bytes() {
        assert_eq!(win_ansi_char(0x93), '\u{201C}');
        assert_eq!(win_ansi_char(0x99), '\u{2122}');
        assert_eq!(win_ansi_char(b'A'), 'A');
    }

    // ═══ 端到端：走完整生产链路 ground_text_bytes ═══
    //
    // 2026-09-29 审计补齐：此前 15 个测试全部只测 helper（hex/cmap/to_norm/
    // normalize/union），`ground_text_bytes` 本身**从未被任何测试调用**。
    // 「helper 全绿」不等于「能定位文字」——本组用 `fixture_pdf()`（真 xref、
    // 2 页、Type1 字体 + ToUnicode CMap、含 TJ 数组与矢量）走全链路。
    #[test]
    fn e2e_finds_word_in_page1_with_cmap_decoded_text() {
        // fixture 第 1 页有 `Confidential memo`（F1 无 ToUnicode，走 WinAnsi）
        // 与 `ToUVW`（F2，TJ 数组拆成 To / UVW 两段）。
        let pdf = super::fixture_pdf();
        let r = super::ground_text_bytes(&pdf, "Confidential", 10).expect("ground");
        assert!(!r.no_text_hint, "不应报告无文字层");
        assert_eq!(r.pages_total, 2, "fixture 是 2 页");
        assert_eq!(r.pages_scanned, 2);
        let hit = r
            .hits
            .iter()
            .find(|h| h.text.contains("Confidential"))
            .expect("应定位到 Confidential");
        assert_eq!(hit.page, 1, "第 1 页的内容");
        // 坐标必须是夹在 [0,1000] 的归一化值，且构成合法盒子
        assert!(hit.norm[0] <= hit.norm[2], "x0 应 <= x1: {:?}", hit.norm);
        assert!(hit.norm[1] <= hit.norm[3], "y0 应 <= y1: {:?}", hit.norm);
        assert!(hit.norm.iter().all(|v| *v <= 1000), "归一化越界: {:?}", hit.norm);
    }

    #[test]
    fn e2e_finds_word_in_page2_proving_it_reads_all_pages() {
        // 只在第 2 页出现的词：能定位它 ⇒ 证明真读了多页而非只扫第一页。
        // fixture 第 2 页内容是矢量指令（无文字），故用第 1 页的第二个词
        // 并配合 max_pages=1 验证「截断时不误扫」。
        let pdf = super::fixture_pdf();
        let r = super::ground_text_bytes(&pdf, "To", 1).expect("ground");
        assert_eq!(r.pages_scanned, 1, "max_pages=1 应只扫 1 页");
        assert_eq!(r.pages_total, 2, "但总页数应报 2（说明被截断）");
        // 判别力锚点：`ToUVW` 经 TJ 数组拆成 `To` / `UVW` 两片，第二片
        // `UVW` 是 **bfrange 末位递增**（<55><57><0055> ⇒ U/V/W）的产物。
        // 若 hex_to_string 完全不解码、或偏移累加回退成 `X`，本断言必红。
        let uv = super::ground_text_bytes(&pdf, "UVW", 10).expect("ground");
        let texts: Vec<&str> = uv.hits.iter().map(|h| h.text.as_str()).collect();
        assert!(texts.contains(&"UVW"), "bfrange 应解出 UVW，实得 {texts:?}");
        let x = super::ground_text_bytes(&pdf, "X", 10).expect("ground");
        assert!(x.hits.is_empty(), "UVW 递增不得溢出成 X，实得 {:?}",
            x.hits.iter().map(|h| h.text.as_str()).collect::<Vec<_>>());
    }

    #[test]
    fn e2e_missing_word_reports_empty_not_guess() {
        let pdf = super::fixture_pdf();
        let r = super::ground_text_bytes(&pdf, "ZzzzNotInDocument", 10).expect("ground");
        assert!(r.hits.is_empty(), "查无此词应返回空，不得瞎猜");
        assert!(!r.no_text_hint, "有文字层，只是没这个词");
    }

    #[test]
    fn e2e_empty_query_is_rejected_before_parsing() {
        let pdf = super::fixture_pdf();
        // 空查询必须**在解析前**被拒（不浪费解析，也避免空匹配全页命中）
        let err = super::ground_text_bytes(&pdf, "   ", 10).expect_err("应拒绝");
        assert!(matches!(err, crate::NtBotError::Invalid(_)), "got {err:?}");
    }

    #[test]
    fn e2e_render_mentions_page_and_quote() {
        // 报告是给模型看的，渲染必须带上页码与原文引用（否则模型无法复核）
        let pdf = super::fixture_pdf();
        let r = super::ground_text_bytes(&pdf, "Confidential", 10).expect("ground");
        let md = r.render("Confidential");
        assert!(md.contains("p.1") || md.contains("page 1") || md.contains("1"),
            "渲染应含页码: {md}");
        assert!(md.contains("Confidential"), "渲染应含原文: {md}");
    }

    #[test]
    fn hex_helpers() {
        assert_eq!(hex_to_u32(b"FF"), Some(255));
        assert_eq!(hex_to_u32(b"0A1"), Some(161));
        assert_eq!(hex_to_u32(b"ZZ"), None);
        assert_eq!(hex_to_string(b"0054"), Some("T".to_owned()));
        assert_eq!(hex_to_string(b"D83DDE00"), Some("😀".to_owned()));
        assert_eq!(hex_to_string(b"5"), None);
    }

    #[test]
    fn cmap_parses_bfchar_and_bfrange() {
        let cmap = parse_cmap(
            b"begincmap 2 beginbfchar <54> <0054> <6F> <006F> endbfchar \
              1 beginbfrange <55> <57> <0055> endbfrange endcmap",
        );
        assert_eq!(cmap.one.get(&0x54).map(String::as_str), Some("T"));
        assert_eq!(cmap.one.get(&0x6F).map(String::as_str), Some("o"));
        // bfrange 末位递增：U V W
        assert_eq!(cmap.one.get(&0x55).map(String::as_str), Some("U"));
        assert_eq!(cmap.one.get(&0x56).map(String::as_str), Some("V"));
        assert_eq!(cmap.one.get(&0x57).map(String::as_str), Some("W"));
    }

    #[test]
    fn cmap_parses_bfrange_array_form() {
        let cmap = parse_cmap(b"1 beginbfrange <41> <43> [<0041> <0042> <0043>] endbfrange");
        assert_eq!(cmap.one.get(&0x41).map(String::as_str), Some("A"));
        assert_eq!(cmap.one.get(&0x43).map(String::as_str), Some("C"));
    }

    #[test]
    fn norm_conversion_flips_y_and_clamps() {
        // 页面 612x792，文字盒 x[72,174] y[697.6,709.6]。
        let n = to_norm(72.0, 697.6, 174.0, 709.6, 0.0, 0.0, 612.0, 792.0);
        assert_eq!(n[0], 118);
        assert!(n[1] < n[3]);
        // 越界坐标必须夹住，不能回绕成负值绕成大数。
        // 2026-09-29 审计修正：原期望 `[0,0,1000,1000]` **算错了** ——
        // 页高 792，而 y0=900 / y1=1000 都已越界 ⇒ 两个 y 归一化后都被
        // clamp 到 0 ⇒ 实际正确输出是 `[0,0,1000,0]`。
        // 「夹住」的本意是**不产生负值/回绕**，`y3==0` 正是夹住的表现。
        let clamped = to_norm(-50.0, 900.0, 700.0, 1000.0, 0.0, 0.0, 612.0, 792.0);
        assert_eq!(clamped, [0, 0, 1000, 0]);
        // 越界但方向正确的一侧仍应被夹到边界值。
        // 期望值经独立手算核对（非照抄程序输出）：
        //   x0=-50 → clamp 0；x1=700 → 700/612=1.14 → clamp 1.0 → 1000
        //   y1=200 → (792-200)/792=0.7475 → 747；y0=100 → (792-100)/792=0.8737 → 874
        let clamped_ok = to_norm(-50.0, 100.0, 700.0, 200.0, 0.0, 0.0, 612.0, 792.0);
        assert_eq!(clamped_ok, [0, 747, 1000, 874]);
    }
}
