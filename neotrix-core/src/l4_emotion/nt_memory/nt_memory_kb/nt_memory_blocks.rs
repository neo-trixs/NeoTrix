//! 类型化块分块 (Typed Block Chunking) — P2 接线。
//!
//! MinerU/Claude-OSINT 吸收: 记忆摄入时按结构保留块类型 (表格/公式/代码/标题/段落),
//! 避免 naive 空白切分破坏表格行与公式语义。块类型随节点 metadata 落库,
//! 检索/再加工阶段可按类型过滤。

/// 结构化块类型 — 保留原文语义的单元划分。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    /// 普通段落
    Paragraph,
    /// Markdown 表格 (连续含 `|` 分隔符的行)
    Table,
    /// 公式块 (`$$...$$` 或 `$...$` 行)
    Formula,
    /// 围栏代码块 (``` ... ```)
    Code,
    /// 标题 (# / ## / ### ...)
    Heading,
    /// 列表 (有序/无序)
    List,
    /// 无法归类的其他块
    Other,
}

impl BlockKind {
    /// 人类可读的块类型标签, 用作 metadata 键。
    pub fn label(&self) -> &'static str {
        match self {
            BlockKind::Paragraph => "paragraph",
            BlockKind::Table => "table",
            BlockKind::Formula => "formula",
            BlockKind::Code => "code",
            BlockKind::Heading => "heading",
            BlockKind::List => "list",
            BlockKind::Other => "other",
        }
    }
}

/// 单个内容块 — 类型 + 原文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentBlock {
    pub kind: BlockKind,
    pub text: String,
}

/// 将原始文档按结构切分为保留类型的块。
///
/// 来源: MinerU 结构化文档解析 + Claude-OSINT 溯源要求 (P2 接线)。
/// 规则顺序: 围栏代码块 > 公式块 > 表格 > 标题 > 列表 > 段落。
pub fn split_typed_blocks(text: &str) -> Vec<ContentBlock> {
    let mut blocks: Vec<ContentBlock> = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    let mut pending: Vec<&str> = Vec::new();

    fn flush(pending: &mut Vec<&str>, blocks: &mut Vec<ContentBlock>) {
        if pending.is_empty() {
            return;
        }
        let joined = pending.join("\n");
        let kind = classify_block_lines(&joined);
        blocks.push(ContentBlock { kind, text: joined });
        pending.clear();
    }

    while i < lines.len() {
        let line = lines[i];

        // 围栏代码块: ``` 或 ~~~ 开头
        if is_fence_open(line) {
            flush(&mut pending, &mut blocks);
            let fence = line;
            let mut body: Vec<&str> = vec![line];
            i += 1;
            while i < lines.len() && !is_fence_close(lines[i], fence) {
                body.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                body.push(lines[i]);
                i += 1;
            }
            blocks.push(ContentBlock {
                kind: BlockKind::Code,
                text: body.join("\n"),
            });
            continue;
        }

        // 公式块: $$ 围栏
        if line.trim_start().starts_with("$$") {
            flush(&mut pending, &mut blocks);
            let mut body: Vec<&str> = vec![line];
            i += 1;
            while i < lines.len() && !lines[i].trim_end().ends_with("$$") {
                body.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                body.push(lines[i]);
                i += 1;
            }
            blocks.push(ContentBlock {
                kind: BlockKind::Formula,
                text: body.join("\n"),
            });
            continue;
        }

        // 表格: 当前行是表格行 (含 | 且下行为分隔行或也是表格行)
        if is_table_row(line) {
            let mut body: Vec<&str> = vec![line];
            i += 1;
            while i < lines.len() && is_table_row(lines[i]) {
                body.push(lines[i]);
                i += 1;
            }
            flush(&mut pending, &mut blocks);
            blocks.push(ContentBlock {
                kind: BlockKind::Table,
                text: body.join("\n"),
            });
            continue;
        }

        // 行内公式: $...$ 整行
        if is_inline_formula_line(line) {
            flush(&mut pending, &mut blocks);
            blocks.push(ContentBlock {
                kind: BlockKind::Formula,
                text: line.to_string(),
            });
            i += 1;
            continue;
        }

        // 标题 / 列表 / 其他积累到段落缓冲
        pending.push(line);
        i += 1;
    }
    flush(&mut pending, &mut blocks);
    blocks
}

/// 对已合并的块文本分类 (用于段落缓冲 flush)。
fn classify_block_lines(joined: &str) -> BlockKind {
    let first = joined.lines().next().unwrap_or("").trim();
    if first.starts_with('#') {
        BlockKind::Heading
    } else if first.starts_with('-') || first.starts_with('*')
        || first.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
    {
        BlockKind::List
    } else {
        BlockKind::Paragraph
    }
}

fn is_fence_open(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("```") || t.starts_with("~~~")
}

fn is_fence_close(line: &str, _fence: &str) -> bool {
    is_fence_open(line)
}

fn is_table_row(line: &str) -> bool {
    let t = line.trim();
    t.contains('|')
}

fn is_inline_formula_line(line: &str) -> bool {
    let t = line.trim();
    t.starts_with('$') && t.ends_with('$') && t.len() >= 2
}

/// 统计各类块数量, 供写库 metadata 使用。
pub fn block_stats(blocks: &[ContentBlock]) -> std::collections::HashMap<String, usize> {
    let mut m = std::collections::HashMap::new();
    for b in blocks {
        *m.entry(b.kind.label().to_string()).or_insert(0) += 1;
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_table_and_formula_blocks() {
        let doc = "# Title\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n$$E = mc^2$$\n\npara text here\n";
        let blocks = split_typed_blocks(doc);
        let kinds: Vec<BlockKind> = blocks.iter().map(|b| b.kind).collect();
        assert!(kinds.contains(&BlockKind::Heading));
        assert!(kinds.contains(&BlockKind::Table));
        assert!(kinds.contains(&BlockKind::Formula));
        assert!(kinds.contains(&BlockKind::Paragraph));
        let table = blocks.iter().find(|b| b.kind == BlockKind::Table).unwrap();
        assert!(table.text.contains("| A | B |"));
    }

    #[test]
    fn splits_code_fence() {
        let doc = "```rust\nfn main() {}\n```\n\nafter\n";
        let blocks = split_typed_blocks(doc);
        let code = blocks.iter().find(|b| b.kind == BlockKind::Code).unwrap();
        assert!(code.text.contains("fn main() {}"));
    }
}
