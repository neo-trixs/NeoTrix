//! Extract markdown pipe-tables (`| ... |`) from parsed markdown text
//! into structured [`ExtractedTable`] objects.
//!
//! This is a **fallback** stage: when a backend (e.g. simple PDF extractor)
//! does not populate `ParsedDocument.tables`, this module can extract them
//! from the raw markdown text and fill the gap.

use crate::core::nt_core_input::document_parser::{ExtractedTable, TableFormat};

/// Extract all markdown pipe-tables from a markdown string.
///
/// Detection rules:
/// - A table starts with a `|` line containing at least one `|`
/// - Followed by a separator line (`|---|---|` or variants with `:`)
/// - Followed by zero or more data rows
pub fn extract_markdown_tables(markdown: &str) -> Vec<ExtractedTable> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut tables: Vec<ExtractedTable> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        // Look for a header line (starts with | and has at least one |)
        if is_table_row(lines[i]) {
            let header_line = lines[i];
            // Next line must be a separator
            if i + 1 < lines.len() && is_separator(lines[i + 1]) {
                let separator = lines[i + 1];
                let headers = parse_row(header_line);
                let alignment = parse_alignment(separator, headers.len());

                // Collect data rows
                let mut data_rows: Vec<Vec<String>> = Vec::new();
                let mut j = i + 2;
                while j < lines.len() && is_table_row(lines[j]) {
                    let row = parse_row(lines[j]);
                    if !row.is_empty() {
                        data_rows.push(row);
                    }
                    j += 1;
                }

                let table = build_table(&headers, &data_rows, &alignment);
                tables.push(table);
                i = j;
                continue;
            }
        }
        i += 1;
    }

    tables
}

/// Determine whether a line is a markdown table row (starts with `|`).
fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.len() > 1
}

/// Determine whether a line is a table separator (`|---|---|` or similar).
fn is_separator(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return false;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.is_empty() {
        return false;
    }
    inner.split('|').all(|seg| {
        let s = seg.trim();
        s.is_empty() || s.chars().all(|c| c == '-' || c == ':')
    })
}

/// Parse a single markdown table row into column strings.
fn parse_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return Vec::new();
    }
    let inner = &trimmed[1..]; // strip leading |
                               // Remove trailing | if present
    let inner = if inner.ends_with('|') {
        &inner[..inner.len() - 1]
    } else {
        inner
    };

    inner.split('|').map(|col| col.trim().to_string()).collect()
}

/// Parse alignment markers from the separator line.
fn parse_alignment(separator: &str, col_count: usize) -> Vec<TableCellAlignment> {
    let trimmed = separator.trim();
    let inner = if trimmed.starts_with('|') {
        &trimmed[1..]
    } else {
        trimmed
    };
    let inner = if inner.ends_with('|') {
        &inner[..inner.len() - 1]
    } else {
        inner
    };

    let segs: Vec<&str> = inner.split('|').collect();
    segs.iter()
        .take(col_count)
        .map(|seg| {
            let s = seg.trim();
            let left = s.starts_with(':');
            let right = s.ends_with(':');
            match (left, right) {
                (true, true) => TableCellAlignment::Center,
                (true, false) => TableCellAlignment::Left,
                (false, true) => TableCellAlignment::Right,
                (false, false) => TableCellAlignment::Left,
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TableCellAlignment {
    Left,
    Center,
    Right,
}

/// Build an [`ExtractedTable`] from parsed header / data rows.
fn build_table(
    headers: &[String],
    rows: &[Vec<String>],
    _alignment: &[TableCellAlignment],
) -> ExtractedTable {
    // Determine format heuristically
    let format = classify_table_format(headers, rows);

    // Build caption: first line of first cell if notable
    let caption = match rows.first().and_then(|r| r.first()) {
        Some(cell)
            if cell.len() < 80
                && headers.first().map(|h| h.to_lowercase()) == Some("".to_string()) =>
        {
            cell.clone()
        }
        _ => String::new(),
    };

    let data_rows: Vec<Vec<String>> = if caption.is_empty() {
        rows.to_vec()
    } else {
        rows[1..].to_vec()
    };

    ExtractedTable {
        caption,
        headers: headers.to_vec(),
        rows: data_rows,
        format,
    }
}

/// Heuristic table format classification.
fn classify_table_format(headers: &[String], rows: &[Vec<String>]) -> TableFormat {
    if headers.is_empty() {
        return TableFormat::Unknown;
    }
    if rows.is_empty() {
        return TableFormat::Simple;
    }

    // Check for merged cells (empty header or variable row lengths)
    let col_count = headers.len();
    let has_empty_header = headers.iter().any(|h| h.trim().is_empty());
    let variable_length = rows.iter().any(|r| r.len() != col_count);

    if variable_length {
        return TableFormat::Merged;
    }

    // Check for pivot: first column has repeated non-unique values
    if col_count >= 3 && rows.len() >= 3 {
        let first_col: Vec<&str> = rows
            .iter()
            .filter_map(|r| r.first().map(|s| s.as_str()))
            .collect();
        let unique_count = {
            let mut dedup = first_col.clone();
            dedup.sort();
            dedup.dedup();
            dedup.len()
        };
        if unique_count < first_col.len() / 2 && unique_count >= 2 {
            return TableFormat::Pivot;
        }
    }

    if has_empty_header {
        return TableFormat::Merged;
    }

    TableFormat::Simple
}

/// Escape markdown special characters for table cell content.
pub fn escape_markdown_cell(text: &str) -> String {
    text.replace('|', "\\|")
        .replace('\n', " ")
        .replace('\r', "")
}

/// Format a collection of [`ExtractedTable`]s back into a markdown table string.
pub fn format_tables_to_markdown(tables: &[ExtractedTable]) -> String {
    let mut output = String::new();
    for (idx, table) in tables.iter().enumerate() {
        if idx > 0 {
            output.push_str("\n\n");
        }
        if !table.caption.is_empty() {
            output.push_str(&format!("**{}**\n\n", table.caption));
        }
        if table.headers.is_empty() {
            continue;
        }
        // Header row
        output.push('|');
        for h in &table.headers {
            output.push_str(&format!(" {} |", escape_markdown_cell(h)));
        }
        output.push('\n');
        // Separator row
        output.push('|');
        for _ in &table.headers {
            output.push_str(" --- |");
        }
        output.push('\n');
        // Data rows
        for row in &table.rows {
            output.push('|');
            for cell in row {
                output.push_str(&format!(" {} |", escape_markdown_cell(cell)));
            }
            output.push('\n');
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_table() {
        let md = "\
| Name  | Age | City     |
|-------|-----|----------|
| Alice | 30  | Beijing  |
| Bob   | 25  | Shanghai |
";
        let tables = extract_markdown_tables(md);
        assert_eq!(tables.len(), 1);
        let t = &tables[0];
        assert_eq!(t.headers, vec!["Name", "Age", "City"]);
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.rows[0][0], "Alice");
        assert_eq!(t.rows[0][1], "30");
        assert_eq!(t.rows[0][2], "Beijing");
        assert_eq!(t.format, TableFormat::Simple);
    }

    #[test]
    fn test_table_without_trailing_pipe() {
        let md = "\
| Name | Age
|------|-----
| Alice| 30
";
        let tables = extract_markdown_tables(md);
        assert_eq!(tables.len(), 1);
        let t = &tables[0];
        assert_eq!(t.headers, vec!["Name", "Age"]);
        assert_eq!(t.rows.len(), 1);
        assert_eq!(t.rows[0][0], "Alice");
    }

    #[test]
    fn test_multiple_tables() {
        let md = "\
Some text.

| A | B |
|---|---|
| 1 | 2 |

More text.

| X | Y | Z |
|---|---|---|
| p | q | r |
| s | t | u |
";
        let tables = extract_markdown_tables(md);
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].headers, vec!["A", "B"]);
        assert_eq!(tables[1].headers, vec!["X", "Y", "Z"]);
        assert_eq!(tables[1].rows.len(), 2);
    }

    #[test]
    fn test_empty_markdown() {
        let tables = extract_markdown_tables("");
        assert!(tables.is_empty());
    }

    #[test]
    fn test_no_tables() {
        let md = "Just some text.\n\nNo pipes here.";
        let tables = extract_markdown_tables(md);
        assert!(tables.is_empty());
    }

    #[test]
    fn test_alignment_detection() {
        let md = "\
| Left | Center | Right |
|:-----|:------:|------:|
| a    | b      | c     |
";
        let tables = extract_markdown_tables(md);
        assert_eq!(tables.len(), 1);
        // Alignment is internal but doesn't affect output structure
        assert_eq!(tables[0].headers.len(), 3);
        assert_eq!(tables[0].rows.len(), 1);
    }

    #[test]
    fn test_classify_pivot() {
        let headers = vec![
            "Year".to_string(),
            "Metric".to_string(),
            "Value".to_string(),
        ];
        let rows = vec![
            vec!["2023".to_string(), "Revenue".to_string(), "100".to_string()],
            vec!["2023".to_string(), "Cost".to_string(), "60".to_string()],
            vec!["2024".to_string(), "Revenue".to_string(), "120".to_string()],
            vec!["2024".to_string(), "Cost".to_string(), "70".to_string()],
        ];
        let format = classify_table_format(&headers, &rows);
        assert_eq!(format, TableFormat::Pivot);
    }

    #[test]
    fn test_escape_roundtrip() {
        let tables = vec![ExtractedTable {
            caption: String::new(),
            headers: vec!["A".to_string(), "B".to_string()],
            rows: vec![vec!["pipe | char".to_string(), "ok".to_string()]],
            format: TableFormat::Simple,
        }];
        let md = format_tables_to_markdown(&tables);
        assert!(md.contains("\\|"));
        let re_extracted = extract_markdown_tables(&md);
        assert_eq!(re_extracted.len(), 1);
        assert_eq!(re_extracted[0].rows[0][0], "pipe | char");
    }

    #[test]
    fn test_format_tables_empty_headers() {
        let md = format_tables_to_markdown(&[]);
        assert!(md.is_empty());
    }
}
