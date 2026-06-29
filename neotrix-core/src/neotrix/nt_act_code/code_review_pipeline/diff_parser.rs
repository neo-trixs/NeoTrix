use super::types::{DiffHunk, DiffLine, DiffLineType, DiffStatus, ReviewFileDiff};

pub struct DiffParser;

impl DiffParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_diff(&self, input: &str) -> Vec<ReviewFileDiff> {
        let mut files = Vec::new();
        let mut current_file: Option<ReviewFileDiff> = None;
        let mut current_hunk: Option<DiffHunk> = None;
        let mut old_line: u32 = 0;
        let mut new_line: u32 = 0;

        for line in input.lines() {
            if line.starts_with("diff --git ") {
                if let Some(file) = current_file.take() {
                    files.push(file);
                }
                current_hunk = None;
                let parts: Vec<&str> = line.split_whitespace().collect();
                let b_path = parts.get(2).and_then(|s| {
                    let s = s.trim_start_matches("a/");
                    Some(s.to_string())
                });
                current_file = Some(ReviewFileDiff {
                    file: b_path.unwrap_or_default(),
                    status: DiffStatus::Modified,
                    old_path: None,
                    hunks: Vec::new(),
                });
            } else if line.starts_with("--- a/") {
            } else if line.starts_with("+++ b/") {
            } else if line.starts_with("@@") {
                if let Some(ref mut cf) = current_file {
                    if let Some(hunk) = current_hunk.take() {
                        cf.hunks.push(hunk);
                    }
                    if let Some((old_s, old_c, new_s, new_c)) = self.parse_hunk_header(line) {
                        current_hunk = Some(DiffHunk {
                            old_start: old_s,
                            old_count: old_c,
                            new_start: new_s,
                            new_count: new_c,
                            lines: Vec::new(),
                        });
                        old_line = old_s;
                        new_line = new_s;
                    }
                }
            } else if let Some(ref mut hunk) = current_hunk {
                let (line_type, content) = self.classify_line(line);
                let (o_line, n_line) = match line_type {
                    DiffLineType::Addition => (None, Some(new_line)),
                    DiffLineType::Deletion => (Some(old_line), None),
                    DiffLineType::Context => (Some(old_line), Some(new_line)),
                };
                hunk.lines.push(DiffLine {
                    line_type,
                    old_line: o_line,
                    new_line: n_line,
                    content: content.to_string(),
                });
                match line_type {
                    DiffLineType::Addition => new_line += 1,
                    DiffLineType::Deletion => old_line += 1,
                    DiffLineType::Context => {
                        old_line += 1;
                        new_line += 1;
                    }
                }
            } else if line.starts_with("rename from ") {
                if let Some(ref mut cf) = current_file {
                    cf.status = DiffStatus::Renamed;
                    cf.old_path = Some(line.trim_start_matches("rename from ").to_string());
                }
            } else if line.starts_with("rename to ") {
                if let Some(ref mut cf) = current_file {
                    cf.file = line.trim_start_matches("rename to ").to_string();
                }
            } else if line.starts_with("new file mode ") {
                if let Some(ref mut cf) = current_file {
                    cf.status = DiffStatus::Added;
                }
            } else if line.starts_with("deleted file mode ") {
                if let Some(ref mut cf) = current_file {
                    cf.status = DiffStatus::Deleted;
                }
            }
        }

        if let Some(mut cf) = current_file {
            if let Some(hunk) = current_hunk.take() {
                cf.hunks.push(hunk);
            }
            files.push(cf);
        }

        files
    }

    fn parse_hunk_header(&self, line: &str) -> Option<(u32, u32, u32, u32)> {
        let rest = line.strip_prefix("@@ ")?;
        let parts: Vec<&str> = rest.splitn(2, " @@").next()?.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }
        let parse_range = |s: &str| -> Option<(u32, u32)> {
            let s = s.trim_start_matches('-').trim_start_matches('+');
            if let Some(comma) = s.find(',') {
                let start: u32 = s[..comma].parse().ok()?;
                let count: u32 = s[comma + 1..].parse().ok()?;
                Some((start, count))
            } else {
                let start: u32 = s.parse().ok()?;
                Some((start, 1))
            }
        };
        let old = parse_range(parts[0])?;
        let new = parse_range(parts[1])?;
        Some((old.0, old.1, new.0, new.1))
    }

    fn classify_line<'a>(&self, line: &'a str) -> (DiffLineType, &'a str) {
        if line.starts_with("+") {
            (DiffLineType::Addition, &line[1..])
        } else if line.starts_with("-") {
            (DiffLineType::Deletion, &line[1..])
        } else {
            (
                DiffLineType::Context,
                if line.starts_with(' ') {
                    &line[1..]
                } else {
                    line
                },
            )
        }
    }
}
