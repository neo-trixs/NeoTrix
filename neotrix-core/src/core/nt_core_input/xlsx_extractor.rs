#![allow(dead_code)]

use super::office_common::{
    extract_rels_map, extract_shared_strings, extract_sheet_names, list_zip_entries,
    read_zip_entry, xml_unescape, ZipEntry,
};

/// Parsed xlsx workbook: sheet name → table data.
#[derive(Debug, Clone)]
pub struct XlsxSheet {
    pub name: String,
    pub rows: Vec<Vec<String>>,
    pub max_col: usize,
}

/// xlsx extraction engine: reads raw .xlsx bytes (ZIP container) and extracts
/// sheet data by traversing shared strings, sheets, and cell references.
#[derive(Debug, Clone)]
pub struct XlsxExtractor {
    max_rows: usize,
    max_sheets: usize,
}

impl Default for XlsxExtractor {
    fn default() -> Self {
        Self {
            max_rows: 10000,
            max_sheets: 50,
        }
    }
}

impl XlsxExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_rows(mut self, n: usize) -> Self {
        self.max_rows = n;
        self
    }
    pub fn with_max_sheets(mut self, n: usize) -> Self {
        self.max_sheets = n;
        self
    }

    /// Extract all sheets from an xlsx file.
    pub fn extract(&self, data: &[u8]) -> Result<Vec<XlsxSheet>, String> {
        let entries = list_zip_entries(data)?;

        // Read shared strings
        let shared_strings = match entries
            .iter()
            .find(|e| e.name.ends_with("sharedStrings.xml"))
        {
            Some(entry) => {
                let xml = read_zip_entry(data, entry)?;
                extract_shared_strings(&xml)
            }
            None => Vec::new(),
        };

        // Read relationships to map sheet rId to file path
        let rels = match entries
            .iter()
            .find(|e| e.name.ends_with("xl/_rels/workbook.xml.rels"))
        {
            Some(entry) => {
                let xml = read_zip_entry(data, entry)?;
                extract_rels_map(&xml)
            }
            None => std::collections::HashMap::new(),
        };

        // Read workbook to get sheet names
        let wb_entry = entries
            .iter()
            .find(|e| e.name.ends_with("workbook.xml"))
            .ok_or_else(|| "workbook.xml not found".to_string())?;
        let wb_xml = read_zip_entry(data, wb_entry)?;
        let sheet_refs = extract_sheet_names(&wb_xml);

        // Build a map from filename to content
        let entry_map: std::collections::HashMap<&str, &ZipEntry> =
            entries.iter().map(|e| (e.name.as_str(), e)).collect();

        let mut sheets = Vec::new();
        for (rid, sname) in sheet_refs.iter().take(self.max_sheets) {
            // Resolve rId to sheet file path
            let sheet_path = rels
                .get(rid.as_str())
                .cloned()
                .unwrap_or_else(|| format!("worksheets/sheet{}.xml", sheets.len() + 1));
            let full_path = if sheet_path.starts_with("xl/") {
                sheet_path.clone()
            } else if sheet_path.starts_with("worksheets/")
                || sheet_path.starts_with("/xl/worksheets/")
            {
                if sheet_path.starts_with('/') {
                    sheet_path[1..].to_string()
                } else {
                    format!("xl/{sheet_path}")
                }
            } else {
                format!("xl/worksheets/{sheet_path}")
            };

            // Find the sheet XML entry
            let sheet_entry = entry_map
                .get(full_path.as_str())
                .or_else(|| {
                    // Try alternative paths
                    let alt = format!("/{full_path}");
                    entry_map.get(alt.as_str())
                })
                .or_else(|| {
                    // Try xl/worksheets/sheetN.xml pattern
                    let n = sheets.len() + 1;
                    let alt = format!("xl/worksheets/sheet{n}.xml");
                    entry_map.get(alt.as_str())
                });

            if let Some(entry) = sheet_entry {
                if let Ok(xml) = read_zip_entry(data, entry) {
                    let parsed = self.parse_sheet(&xml, &shared_strings);
                    sheets.push(XlsxSheet {
                        name: sname.clone(),
                        rows: parsed.0,
                        max_col: parsed.1,
                    });
                }
            }
        }

        Ok(sheets)
    }

    /// Parse sheet XML into rows of cell text.
    fn parse_sheet(&self, xml: &[u8], shared_strings: &[String]) -> (Vec<Vec<String>>, usize) {
        let s = String::from_utf8_lossy(xml);
        use std::collections::HashMap;
        let mut cell_map: HashMap<(usize, usize), String> = HashMap::new();
        let mut max_row = 0usize;
        let mut max_col = 0usize;

        let mut pos = 0;
        // Find all <row> elements and their <c> children
        while let Some(row_start) = s[pos..].find("<row ") {
            let abs_row = pos + row_start;
            let row_close = match s[abs_row..].find('>') {
                Some(o) => abs_row + o,
                None => break,
            };
            let row_tag = &s[abs_row..row_close + 1];

            // Extract row number
            let r_attr = extract_attr(row_tag, "r");
            let row_num: usize = r_attr.and_then(|r| r.parse().ok()).unwrap_or(0);

            // Find all <c> cells within this row
            let mut cell_pos = row_close + 1;
            loop {
                // Find next <c> or </row>
                let next_c = s[cell_pos..].find("<c ");
                let next_row_end = s[cell_pos..].find("</row>");

                match (next_c, next_row_end) {
                    (Some(_), Some(re)) if re < next_c.unwrap() => break,
                    (None, _) => break,
                    (Some(_), None) => break,
                    _ => {}
                }

                let ci = match next_c {
                    Some(c) => c,
                    None => break,
                };
                let abs_ci = cell_pos + ci;

                // Find end of this <c> element
                let c_close = match s[abs_ci..].find("</c>") {
                    Some(o) => o,
                    None => break,
                };
                let cell_block = &s[abs_ci..abs_ci + c_close + 4];

                // Extract cell reference (r attribute)
                let cell_ref = extract_attr(cell_block, "r");
                // Extract type (t attribute): t="s" means shared string
                let cell_type = extract_attr(cell_block, "t");
                // Extract value from <v> tag
                let value = extract_xml_tag_text_in(cell_block, "v");

                if let Some(ref cr) = cell_ref {
                    if let Some((col, _)) = parse_cell_ref(cr) {
                        let cell_text = match (cell_type.as_deref(), value) {
                            (Some("s"), Some(idx_str)) => idx_str
                                .parse::<usize>()
                                .ok()
                                .and_then(|idx| shared_strings.get(idx))
                                .cloned()
                                .unwrap_or_default(),
                            (_, Some(val)) => xml_unescape(&val),
                            _ => String::new(),
                        };
                        if !cell_text.is_empty() || cell_type.is_some() {
                            cell_map.insert((col, row_num), cell_text);
                            if col > max_col {
                                max_col = col;
                            }
                            if row_num > max_row {
                                max_row = row_num;
                            }
                        }
                    }
                }

                cell_pos = abs_ci + c_close + 4;
                let _ = cell_pos; // suppress unused warning
            }

            pos = abs_row + 1;
        }

        // If no <row> tags found, try inline cell parsing
        if cell_map.is_empty() {
            return self.parse_sheet_flat(&s, shared_strings);
        }

        // Convert cell map to row-major vec
        let mut rows: Vec<Vec<String>> = Vec::new();
        for r in 1..=max_row {
            let mut row_data: Vec<String> = Vec::new();
            for c in 0..=max_col {
                row_data.push(cell_map.get(&(c, r)).cloned().unwrap_or_default());
            }
            // Trim trailing empty cells
            while row_data.last().map(|s| s.is_empty()).unwrap_or(false) {
                row_data.pop();
            }
            if !row_data.is_empty() || r == 1 {
                rows.push(row_data);
            }
        }

        (rows, max_col + 1)
    }

    /// Fallback flat XML parser when no <row> tags are found.
    fn parse_sheet_flat(&self, s: &str, shared_strings: &[String]) -> (Vec<Vec<String>>, usize) {
        use std::collections::HashMap;
        let mut cell_map: HashMap<(usize, usize), String> = HashMap::new();
        let mut max_col = 0usize;
        let mut max_row = 0usize;

        let mut pos = 0;
        while let Some(c_start) = s[pos..].find("<c ") {
            let abs_c = pos + c_start;
            let c_close = match s[abs_c..].find("</c>") {
                Some(o) => abs_c + o + 4,
                None => break,
            };
            let block = &s[abs_c..c_close];
            let ref_str = extract_attr(block, "r");
            let cell_type = extract_attr(block, "t");
            let value = extract_xml_tag_text_in(block, "v");

            if let Some(cr) = ref_str {
                let (col, row) = parse_cell_ref(&cr).unwrap_or((0, 0));
                let cell_text = match (cell_type.as_deref(), value) {
                    (Some("s"), Some(idx_str)) => idx_str
                        .parse::<usize>()
                        .ok()
                        .and_then(|idx| shared_strings.get(idx))
                        .cloned()
                        .unwrap_or_default(),
                    (_, Some(val)) => xml_unescape(&val),
                    _ => String::new(),
                };
                cell_map.insert((col, row), cell_text);
                if col > max_col {
                    max_col = col;
                }
                if row > max_row {
                    max_row = row;
                }
            }
            pos = c_close;
        }

        let mut rows: Vec<Vec<String>> = Vec::new();
        for r in 0..=max_row {
            let mut row_data: Vec<String> = Vec::new();
            for c in 0..=max_col {
                row_data.push(cell_map.get(&(c, r)).cloned().unwrap_or_default());
            }
            while row_data.last().map(|s| s.is_empty()).unwrap_or(false) {
                row_data.pop();
            }
            rows.push(row_data);
        }
        (rows, max_col + 1)
    }

    /// Extract all sheets as a single markdown string.
    pub fn to_markdown(&self, data: &[u8]) -> Result<String, String> {
        let sheets = self.extract(data)?;
        let mut md = String::new();
        for sheet in &sheets {
            md.push_str(&format!("## Sheet: {}\n\n", sheet.name));
            if sheet.rows.is_empty() {
                md.push_str("*(empty)*\n\n");
                continue;
            }
            for row in &sheet.rows {
                let cols: Vec<String> = row
                    .iter()
                    .map(|c| {
                        if c.contains(',') || c.contains('"') {
                            format!("\"{}\"", c.replace('"', "\"\""))
                        } else {
                            c.clone()
                        }
                    })
                    .collect();
                md.push_str(&cols.join(" | "));
                md.push('\n');
            }
            md.push('\n');
        }
        Ok(md)
    }
}

// Helper: extract attribute from a tag string
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let search = format!("{attr}=\"");
    let start = tag.find(&search)?;
    let value_start = start + search.len();
    let end = tag[value_start..].find('"')? + value_start;
    Some(tag[value_start..end].to_string())
}

// Helper: extract text content from <tag>...</tag> within a string
fn extract_xml_tag_text_in(s: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = s.find(&open)?;
    let content_start = start + open.len();
    let end = s[content_start..].find(&close)? + content_start;
    Some(s[content_start..end].to_string())
}

// Helper: parse "A1" → (0, 0)
fn parse_cell_ref(ref_str: &str) -> Option<(usize, usize)> {
    let col_end = ref_str.find(|c: char| c.is_ascii_digit())?;
    let col_str = &ref_str[..col_end];
    let row_str = &ref_str[col_end..];
    let mut col = 0usize;
    for c in col_str.chars() {
        col = col * 26 + (c as usize - 'A' as usize + 1);
    }
    let row: usize = row_str.parse().ok()?;
    Some((col.saturating_sub(1), row.saturating_sub(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_minimal_xlsx() -> Vec<u8> {
        // Build a minimal xlsx with 1 sheet containing "Hello" in A1
        // This requires building a ZIP with the right OOXML structure
        let mut zip_data = Vec::new();

        // We'll create entries using the ZIP central directory builder pattern
        // For tests, we just verify the parser handles empty/invalid data gracefully
        vec![]
    }

    #[test]
    fn test_extractor_defaults() {
        let ex = XlsxExtractor::new();
        assert_eq!(ex.max_rows, 10000);
        assert_eq!(ex.max_sheets, 50);
    }

    #[test]
    fn test_invalid_xlsx() {
        let ex = XlsxExtractor::new();
        let result = ex.extract(b"not a zip file");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cell_ref_internal() {
        assert_eq!(parse_cell_ref("A1"), Some((0, 0)));
        assert_eq!(parse_cell_ref("B2"), Some((1, 1)));
        assert_eq!(parse_cell_ref("Z100"), Some((25, 99)));
        assert_eq!(parse_cell_ref("AA1"), Some((26, 0)));
    }

    #[test]
    fn test_extract_attr_internal() {
        let tag = r#"<c r="A1" t="s"><v>0</v></c>"#;
        assert_eq!(extract_attr(tag, "r"), Some("A1".to_string()));
        assert_eq!(extract_attr(tag, "t"), Some("s".to_string()));
    }

    #[test]
    fn test_to_markdown_empty_xlsx() {
        let ex = XlsxExtractor::new();
        let result = ex.to_markdown(b"not a zip");
        assert!(result.is_err());
    }
}
