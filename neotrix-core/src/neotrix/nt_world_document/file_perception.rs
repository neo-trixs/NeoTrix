//! # FilePerceptionModule — Structured Document Perception
//!
//! Universal file perception for consciousness ingestion.
//! Zero hardcoded extractors. Detects layout patterns from cell content.
//!
//! Layout patterns:
//!   ROW    — Row-oriented: each row = one product
//!   PIVOT  — Matrix: size × category → price
//!   SIDE   — Side-by-side: multiple independent tables
//!   MERGED — Multi-section: headers embedded mid-file
//!   FLEX   — Multi-column groups: material/price pairs
//!
//! Design: data-first, not headers-first — infer schema from content.

use std::path::Path;

/// Supported document formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentFormat {
    Xlsx,
    Xls,
    Csv,
    Unknown,
}

/// Detected layout pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutType {
    Row,
    Pivot,
    Side,
    Merged,
    Flex,
    Unknown,
}

impl LayoutType {
    pub fn name(&self) -> &'static str {
        match self {
            LayoutType::Row => "ROW",
            LayoutType::Pivot => "PIVOT",
            LayoutType::Side => "SIDE",
            LayoutType::Merged => "MERGED",
            LayoutType::Flex => "FLEX",
            LayoutType::Unknown => "UNKNOWN",
        }
    }
}

/// A parsed perception — structured fields extracted from document
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub fields: Vec<(String, String)>,
    pub confidence: f64,
    pub layout: LayoutType,
    pub source: String,
    pub section_name: String,
}

/// Report of the perception pipeline run
#[derive(Debug, Clone)]
pub struct PerceptionReport {
    pub file: String,
    pub format: DocumentFormat,
    pub rows: usize,
    pub cols: usize,
    pub layout: LayoutType,
    pub product_count: usize,
    pub phase1_atlas: String,
    pub issues: Vec<String>,
}

/// Cell type classification
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum CellType {
    Empty,
    DnSize,
    Price,
    Weight,
    SeqNum,
    Header,
    Material,
    Name,
    Misc,
    OtherNum,
}

#[allow(dead_code)]
const MAX_COLS: usize = 40;

// ── Header keywords — Chinese price list detection ──
const HEADER_KW: &[&str] = &[
    "名称", "口径", "规格", "单价", "单重", "重量", "价格", "型号", "材质", "标准", "压力",
    "数量", "备注", "序号", "品名", "产品名称", "单位", "配置",
];

const MATERIAL_KW: &[&str] = &[
    "球板", "304板", "316板", "316L板", "2205板", "2507板", "铜板", "球铁", "QT450", "WCB",
    "CF8", "CF8M", "304", "316", "EPDM", "NBR", "PTFE", "2CR13", "黄铜", "青铜", "不锈钢",
    "碳钢", "铸铁", "铸钢", "黑", "热", "镀锌",
];

/// The document perception module — pluggable into ConsciousnessCycle GATHER
#[derive(Debug, Clone)]
pub struct DocumentPerceptionModule {
    /// Whether to shell out to Python smart_schema_mapper_v2.py
    use_python_backend: bool,
    /// Path to the Python script
    python_script_path: Option<String>,
    /// Number of products perceived in last cycle
    pub last_product_count: usize,
    /// Last perception confidence
    pub last_confidence: f64,
    /// Most recent parsed document markdown content (from DocumentParserRegistry)
    pub last_parsed_markdown: Option<String>,
    /// Number of tables extracted in last parse
    pub last_table_count: usize,
    /// Number of figures extracted in last parse
    pub last_figure_count: usize,
    /// Number of formulas extracted in last parse
    pub last_formula_count: usize,
}

impl Default for DocumentPerceptionModule {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentPerceptionModule {
    pub fn new() -> Self {
        Self {
            use_python_backend: true,
            python_script_path: None,
            last_product_count: 0,
            last_confidence: 0.0,
            last_parsed_markdown: None,
            last_table_count: 0,
            last_figure_count: 0,
            last_formula_count: 0,
        }
    }

    pub fn with_python_backend(mut self, script_path: &str) -> Self {
        self.use_python_backend = true;
        self.python_script_path = Some(script_path.to_string());
        self
    }

    /// Perceive a file: detect format → classify → extract → report
    pub fn perceive(&mut self, file_path: &str) -> Result<(Vec<ParseResult>, PerceptionReport), String> {
        let format = detect_format(file_path);
        if format == DocumentFormat::Unknown {
            return Err(format!("Unsupported document format: {}", file_path));
        }
        if self.use_python_backend {
            return self.perceive_via_python(file_path, format);
        }
        self.perceive_native(file_path, format)
    }

    fn perceive_via_python(&mut self, file_path: &str, format: DocumentFormat) -> Result<(Vec<ParseResult>, PerceptionReport), String> {
        let script_path = self.python_script_path.as_deref()
            .unwrap_or("/Users/neo/Downloads/smart_schema_mapper_v2.py");
        let script_dir = Path::new(script_path).parent()
            .map(|p| p.to_string_lossy())
            .unwrap_or_default();
        let script_name = Path::new(script_path).file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();
        let supplier = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // Inline Python: import universal_parse, run it, emit JSON report to stdout
        let inline = format!(
            r#"import sys; sys.path.insert(0, '{}')
from {} import universal_parse
import json
products, report = universal_parse('{}', '{}')
print(json.dumps({{"count": len(products), "layout": report.get("steps", {{}}).get("layout", "UNKNOWN")}}))
"#,
            script_dir.replace("'", "'\\''"),
            script_name.replace("'", "'\\''"),
            file_path.replace("'", "'\\''"),
            supplier.replace("'", "'\\''"),
        );

        let output = std::process::Command::new("python3")
            .arg("-c")
            .arg(&inline)
            .output()
            .map_err(|e| format!("Python backend failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Python parser error: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut count = 0usize;
        let mut layout = LayoutType::Unknown;

        // Parse JSON output line from inline script
        for line in stdout.lines() {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(c) = data.get("count").and_then(|v| v.as_u64()) {
                    count = c as usize;
                }
                if let Some(l) = data.get("layout").and_then(|v| v.as_str()) {
                    layout = match l {
                        "ROW" => LayoutType::Row,
                        "PIVOT" => LayoutType::Pivot,
                        "SIDE" => LayoutType::Side,
                        "MERGED" => LayoutType::Merged,
                        "FLEX" => LayoutType::Flex,
                        _ => LayoutType::Unknown,
                    };
                }
            }
        }

        let report = PerceptionReport {
            file: file_path.to_string(),
            format,
            rows: 0,
            cols: 0,
            layout,
            product_count: count,
            phase1_atlas: format!("via Python backend: {} products", count),
            issues: vec![],
        };

        Ok((vec![], report))
    }

    /// Native Rust perception (future: reimplement the Python algorithm in Rust)
    fn perceive_native(&self, _file_path: &str, _format: DocumentFormat) -> Result<(Vec<ParseResult>, PerceptionReport), String> {
        Err("Native Rust perception not yet implemented — use Python backend".to_string())
    }

    pub fn is_available(&self) -> bool {
        self.use_python_backend
    }

    /// Feed a parsed document result into the VSA/knowledge pipeline.
    /// Stores the content for downstream consciousness steps to consume.
    pub fn feed_parsed_document(&mut self, parsed: &crate::core::nt_core_input::document_parser::ParsedDocument) {
        self.last_parsed_markdown = Some(parsed.markdown.clone());
        self.last_table_count = parsed.tables.len();
        self.last_figure_count = parsed.images.len();
        self.last_formula_count = parsed.metadata.formula_count;
        if !parsed.markdown.is_empty() {
            self.last_confidence = 0.8 + (parsed.tables.len() as f64 * 0.02).min(0.15);
        } else {
            self.last_confidence = 0.2;
        }
    }
}

// ── Format detection ──

fn detect_format(path: &str) -> DocumentFormat {
    let lower = path.to_lowercase();
    if lower.ends_with(".xlsx") {
        DocumentFormat::Xlsx
    } else if lower.ends_with(".xls") {
        DocumentFormat::Xls
    } else if lower.ends_with(".csv") {
        DocumentFormat::Csv
    } else {
        DocumentFormat::Unknown
    }
}

// ── Cell type classification (native Rust, for future use) ──

#[allow(dead_code)]
fn classify_cell(val: &str) -> CellType {
    let s = val.trim();
    if s.is_empty() {
        return CellType::Empty;
    }
    // DN pattern
    let upper = s.to_uppercase();
    if upper.starts_with("DN") || s.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(n) = s.parse::<i32>() {
            if (6..=3000).contains(&n) {
                return CellType::DnSize;
            }
        }
        if upper.starts_with("DN") {
            return CellType::DnSize;
        }
    }
    // Numeric
    if let Ok(num) = s.parse::<f64>() {
        if num < 0.0 {
            return CellType::OtherNum;
        }
        if (1.0..=999.0).contains(&num) && s.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
            return CellType::SeqNum;
        }
        if (3.0..=50000.0).contains(&num) {
            return CellType::Price;
        }
        if (0.1..=2000.0).contains(&num) {
            return CellType::Weight;
        }
        return CellType::OtherNum;
    }
    // Chinese header keywords
    for kw in HEADER_KW {
        if s.contains(kw) {
            return CellType::Header;
        }
    }
    // Material keywords
    for mat in MATERIAL_KW {
        if s.contains(mat) {
            return CellType::Material;
        }
    }
    // Chinese text (2+ chars)
    if s.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}') && s.len() >= 2 {
        return CellType::Name;
    }
    CellType::Misc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_xlsx() {
        assert_eq!(detect_format("prices.xlsx"), DocumentFormat::Xlsx);
        assert_eq!(detect_format("data.XLSX"), DocumentFormat::Xlsx);
    }

    #[test]
    fn test_detect_format_csv() {
        assert_eq!(detect_format("prices.csv"), DocumentFormat::Csv);
    }

    #[test]
    fn test_detect_format_unknown() {
        assert_eq!(detect_format("readme.pdf"), DocumentFormat::Unknown);
        assert_eq!(detect_format("image.png"), DocumentFormat::Unknown);
    }

    #[test]
    fn test_classify_dn() {
        assert_eq!(classify_cell("DN50"), CellType::DnSize);
        assert_eq!(classify_cell("dn100"), CellType::DnSize);
        assert_eq!(classify_cell("150"), CellType::DnSize);
    }

    #[test]
    fn test_classify_price() {
        assert_eq!(classify_cell("123.50"), CellType::Price);
        assert_eq!(classify_cell("5000"), CellType::Price);
    }

    #[test]
    fn test_classify_weight() {
        assert_eq!(classify_cell("12.5"), CellType::Weight);
        assert_eq!(classify_cell("0.5"), CellType::Weight);
    }

    #[test]
    fn test_classify_header() {
        assert_eq!(classify_cell("规格"), CellType::Header);
        assert_eq!(classify_cell("单价"), CellType::Header);
        assert_eq!(classify_cell("产品名称"), CellType::Header);
    }

    #[test]
    fn test_classify_material() {
        assert_eq!(classify_cell("304"), CellType::Material);
        assert_eq!(classify_cell("不锈钢"), CellType::Material);
    }

    #[test]
    fn test_classify_chinese_name() {
        assert_eq!(classify_cell("法兰闸阀"), CellType::Name);
    }

    #[test]
    fn test_classify_empty() {
        assert_eq!(classify_cell(""), CellType::Empty);
        assert_eq!(classify_cell("  "), CellType::Empty);
    }

    #[test]
    fn test_layout_type_name() {
        assert_eq!(LayoutType::Row.name(), "ROW");
        assert_eq!(LayoutType::Pivot.name(), "PIVOT");
        assert_eq!(LayoutType::Unknown.name(), "UNKNOWN");
    }

    #[test]
    fn test_perception_module_default() {
        let pm = DocumentPerceptionModule::new();
        assert!(pm.use_python_backend);
        assert_eq!(pm.last_product_count, 0);
    }

    #[test]
    fn test_is_available() {
        let pm = DocumentPerceptionModule::new();
        assert!(pm.is_available());
    }
}
