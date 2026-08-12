use ratatui::{
    style::{Style, Color, Modifier},
    text::{Line, Span},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Add,
    Remove,
    Context,
    HunkHeader,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct ParsedBlock {
    pub header: String,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone)]
pub struct DiffViewer {
    pub content: String,
    pub blocks: Vec<ParsedBlock>,
    pub scroll_offset: usize,
}

impl DiffViewer {
    pub fn new(content: String) -> Self {
        let mut viewer = Self { content, blocks: Vec::new(), scroll_offset: 0 };
        viewer.parse_diff();
        viewer
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset += lines;
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn total_rendered_lines(&self) -> usize {
        let mut count = 0;
        for block in &self.blocks {
            count += block.header.lines().count();
            count += block.hunks.len();
            for hunk in &block.hunks {
                count += hunk.lines.len();
            }
        }
        count
    }

    pub fn all_rendered_lines(&self) -> Vec<Line<'static>> {
        let mut result = Vec::new();
        for block in &self.blocks {
            for header_line in block.header.lines() {
                result.push(Line::from(Span::styled(
                    header_line.to_string(),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
            }
            for hunk in &block.hunks {
                result.push(Line::from(Span::styled(
                    hunk.header.clone(),
                    Style::default().fg(Color::Cyan),
                )));
                for diff_line in &hunk.lines {
                    if diff_line.kind == DiffLineKind::HunkHeader {
                        continue;
                    }
                    let content_style = match diff_line.kind {
                        DiffLineKind::Add => Style::default().fg(Color::Green),
                        DiffLineKind::Remove => Style::default().fg(Color::Red),
                        DiffLineKind::Context => Style::default().fg(Color::DarkGray),
                        DiffLineKind::HunkHeader => Style::default().fg(Color::DarkGray),
                    };
                    let line_no = match (diff_line.old_lineno, diff_line.new_lineno) {
                        (Some(o), Some(n)) => format!("{:>4} {:>4} ", o, n),
                        (Some(o), None) => format!("{:>4}      ", o),
                        (None, Some(n)) => format!("     {:>4} ", n),
                        (None, None) => String::new(),
                    };
                    result.push(Line::from(vec![
                        Span::styled(line_no, Style::default().fg(Color::DarkGray)),
                        Span::styled(diff_line.content.clone(), content_style),
                    ]));
                }
            }
        }
        result
    }

    pub fn parse_diff(&mut self) {
        let mut blocks = Vec::new();
        let mut current_block: Option<ParsedBlock> = None;
        let mut current_hunk: Option<DiffHunk> = None;
        let mut old_line: u32 = 0;
        let mut new_line: u32 = 0;

        for line in self.content.lines() {
            if line.starts_with("diff --git") {
                if let Some(hunk) = current_hunk.take() {
                    if let Some(ref mut block) = current_block {
                        block.hunks.push(hunk);
                    }
                }
                if let Some(block) = current_block.take() {
                    blocks.push(block);
                }
                current_block = Some(ParsedBlock { header: line.to_string(), hunks: Vec::new() });
            } else if line.starts_with("@@") && line.rfind("@@").is_some_and(|i| i > 2) {
                if let Some(hunk) = current_hunk.take() {
                    if let Some(ref mut block) = current_block {
                        block.hunks.push(hunk);
                    }
                }
                let (old_start, new_start) = parse_hunk_header(line);
                old_line = old_start;
                new_line = new_start;
                let mut hunk = DiffHunk { header: line.to_string(), lines: Vec::new() };
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::HunkHeader,
                    content: line.to_string(),
                    old_lineno: None,
                    new_lineno: None,
                });
                current_hunk = Some(hunk);
            } else if line.starts_with('+') {
                if let Some(ref mut hunk) = current_hunk {
                    hunk.lines.push(DiffLine {
                        kind: DiffLineKind::Add,
                        content: line.to_string(),
                        old_lineno: None,
                        new_lineno: Some(new_line),
                    });
                    new_line += 1;
                }
            } else if line.starts_with('-') {
                if let Some(ref mut hunk) = current_hunk {
                    hunk.lines.push(DiffLine {
                        kind: DiffLineKind::Remove,
                        content: line.to_string(),
                        old_lineno: Some(old_line),
                        new_lineno: None,
                    });
                    old_line += 1;
                }
            } else if line.starts_with(' ') {
                if let Some(ref mut hunk) = current_hunk {
                    hunk.lines.push(DiffLine {
                        kind: DiffLineKind::Context,
                        content: line.to_string(),
                        old_lineno: Some(old_line),
                        new_lineno: Some(new_line),
                    });
                    old_line += 1;
                    new_line += 1;
                }
            } else if line.starts_with("--- ") || line.starts_with("+++ ")
                || line.starts_with("index ") || line.starts_with("new file")
                || line.starts_with("deleted file") || line.starts_with("similarity index")
                || line.starts_with("rename from") || line.starts_with("rename to")
                || line.starts_with("Binary files")
            {
                if let Some(ref mut block) = current_block {
                    block.header.push('\n');
                    block.header.push_str(line);
                }
            } else if line.starts_with("\\ ") {
                if let Some(ref mut hunk) = current_hunk {
                    hunk.lines.push(DiffLine {
                        kind: DiffLineKind::Context,
                        content: line.to_string(),
                        old_lineno: None,
                        new_lineno: None,
                    });
                }
            }
        }

        if let Some(hunk) = current_hunk.take() {
            if let Some(ref mut block) = current_block {
                block.hunks.push(hunk);
            }
        }
        if let Some(block) = current_block.take() {
            blocks.push(block);
        }

        self.blocks = blocks;
    }
}

fn parse_hunk_header(line: &str) -> (u32, u32) {
    let trimmed = line.trim_start_matches("@@").trim_end_matches("@@").trim();
    let parts: Vec<&str> = trimmed.split(' ').collect();
    let old_part = parts.first().unwrap_or(&"-0,0").trim_start_matches('-');
    let new_part = parts.get(1).unwrap_or(&"+0,0").trim_start_matches('+');
    let old_start = old_part.split(',').next().unwrap_or("0").parse().unwrap_or(0);
    let new_start = new_part.split(',').next().unwrap_or("0").parse().unwrap_or(0);
    (old_start, new_start)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    const SAMPLE: &str = "diff --git a/src/a.rs b/src/a.rs\n\
index 1234567..89abcde 100644\n\
--- a/src/a.rs\n\
+++ b/src/a.rs\n\
@@ -1,4 +1,5 @@\n\
 fn main() {\n\
-    println!(\"old\");\n\
+    println!(\"new\");\n\
     let x = 1;\n\
 }\n\
diff --git a/src/b.rs b/src/b.rs\n\
@@ -10,2 +10,3 @@\n\
-old b\n\
+new b\n";

    #[test]
    fn test_parse_diff_blocks_and_hunks() {
        let viewer = DiffViewer::new(SAMPLE.to_string());
        assert_eq!(viewer.blocks.len(), 2, "两个文件 → 两个 block");
        assert_eq!(viewer.blocks[0].hunks.len(), 1);
        assert_eq!(viewer.blocks[1].hunks.len(), 1);
    }

    #[test]
    fn test_parse_diff_line_kinds_and_linenos() {
        // 用显式构造的字符串 (不用 \ 续行, 避免续行吞行首空格导致 Context 行丢失)
        let diff = concat!(
            "diff --git a/src/a.rs b/src/a.rs\n",
            "index 123..456 100644\n",
            "--- a/src/a.rs\n",
            "+++ b/src/a.rs\n",
            "@@ -1,4 +1,5 @@\n",
            " fn main() {\n",
            "-    println!(\"old\");\n",
            "+    println!(\"new\");\n",
            "     let x = 1;\n",
            " }\n",
            "diff --git a/src/b.rs b/src/b.rs\n",
            "@@ -10,2 +10,3 @@\n",
            "-old b\n",
            "+new b\n",
        );
        let viewer = DiffViewer::new(diff.to_string());
        let hunk = &viewer.blocks[0].hunks[0];
        let kinds: Vec<DiffLineKind> = hunk.lines.iter().map(|l| l.kind).collect();
        assert!(kinds.contains(&DiffLineKind::HunkHeader));
        assert!(kinds.contains(&DiffLineKind::Add));
        assert!(kinds.contains(&DiffLineKind::Remove));
        assert!(kinds.contains(&DiffLineKind::Context));
        // 行号跟踪：@@ -1,4 +1,5 @@ → add 行 new=2，remove 行 old=2
        let add = hunk.lines.iter().find(|l| l.kind == DiffLineKind::Add).unwrap();
        assert_eq!(add.new_lineno, Some(2));
        let remove = hunk.lines.iter().find(|l| l.kind == DiffLineKind::Remove).unwrap();
        assert_eq!(remove.old_lineno, Some(2));
    }

    #[test]
    fn test_parse_diff_empty() {
        let viewer = DiffViewer::new(String::new());
        assert!(viewer.is_empty());
        assert_eq!(viewer.total_rendered_lines(), 0);
    }

    #[test]
    fn test_parse_diff_non_diff_text() {
        let viewer = DiffViewer::new("plain text\nno diff markers\n".to_string());
        assert!(viewer.is_empty(), "非 diff 文本不应产生 block");
    }

    #[test]
    fn test_scroll_bounds() {
        let mut viewer = DiffViewer::new(SAMPLE.to_string());
        viewer.scroll_up(100);
        assert_eq!(viewer.scroll_offset, 0, "上滚不越界");
        viewer.scroll_down(5);
        assert_eq!(viewer.scroll_offset, 5);
    }

    #[test]
    fn test_all_rendered_lines_styled() {
        let viewer = DiffViewer::new(SAMPLE.to_string());
        let lines = viewer.all_rendered_lines();
        assert!(!lines.is_empty());
        // 首行是 block header（黄色加粗）
        assert!(lines[0].spans.iter().any(|s| s.style.fg == Some(Color::Yellow)));
        // 至少有一行绿色（Add）与红色（Remove）
        assert!(lines.iter().any(|l| l.spans.iter().any(|s| s.style.fg == Some(Color::Green))));
        assert!(lines.iter().any(|l| l.spans.iter().any(|s| s.style.fg == Some(Color::Red))));
    }

    #[test]
    fn test_parse_hunk_header_edges() {
        assert_eq!(parse_hunk_header("@@ -1,4 +1,5 @@"), (1, 1));
        assert_eq!(parse_hunk_header("@@ -0,0 +1,3 @@"), (0, 1));
        assert_eq!(parse_hunk_header("@@ -10 +11 @@"), (10, 11));
    }
}
