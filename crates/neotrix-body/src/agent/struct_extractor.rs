//! # Structured Extractor (G309)
//!
//! Schema-driven field extraction from HTML/Markdown with type coercion
//! and VSA role-filler binding for schema encoding.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Data type for an extraction field, supporting primitive and composite types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Date,
    Currency,
    Url,
    Email,
    Array(Box<FieldType>),
    Nested(Vec<ExtractionField>),
}

/// Optional transformation applied to extracted raw text before coercion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldTransform {
    None,
    Trim,
    UpperCase,
    LowerCase,
    StripHtml,
    ParseNumber,
    ParseDate,
}

/// A single field within an extraction schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractionField {
    pub name: String,
    pub selector: Option<String>,
    pub field_type: FieldType,
    pub required: bool,
    pub transform: Option<FieldTransform>,
    /// VSA role encoding (e.g. SUBJECT / PREDICATE / OBJECT).
    pub vsa_role: [u64; 2],
}

/// Schema describing how to extract structured data from HTML/Markdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionSchema {
    pub name: String,
    pub fields: Vec<ExtractionField>,
    /// VSA 4096-dim encoding of the entire schema (truncated to [u64; 4]).
    pub vsa_schema: [u64; 4],
}

/// Schema-driven extractor engine.
#[derive(Debug, Clone)]
pub struct Extractor {
    pub schema: ExtractionSchema,
}

// ---------------------------------------------------------------------------
// VSA hash utilities
// ---------------------------------------------------------------------------

fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Encode a semantic role name into a [u64; 2] VSA vector.
pub fn encode_vsa_role(role_name: &str) -> [u64; 2] {
    [
        hash_string(&format!("vsa_role:{}:lo", role_name)),
        hash_string(&format!("vsa_role:{}:hi", role_name)),
    ]
}

fn compute_vsa_schema(name: &str, fields: &[ExtractionField]) -> [u64; 4] {
    let name_h = hash_string(&format!("schema:{}", name));
    let fields_h = fields
        .iter()
        .map(|f| {
            hash_string(&format!(
                "{}:{:?}:{:?}:{}",
                f.name, f.field_type, f.vsa_role, f.required
            ))
        })
        .fold(0u64, |acc, h| acc ^ h);
    let type_h = hash_string("schema_v3");
    [
        name_h,
        fields_h,
        type_h,
        name_h ^ fields_h ^ type_h,
    ]
}

// ---------------------------------------------------------------------------
// HTML extraction helpers
// ---------------------------------------------------------------------------

fn extract_by_selector(html: &str, selector: &str) -> Option<String> {
    if selector.starts_with("//") {
        extract_by_xpath(html, selector)
    } else if selector.contains('#') {
        extract_by_id(html, selector)
    } else if selector.contains('.') {
        extract_by_class(html, selector)
    } else {
        extract_by_tag(html, selector)
    }
}

fn extract_by_id(html: &str, selector: &str) -> Option<String> {
    let id_part = selector
        .split('#')
        .nth(1)
        .and_then(|s| s.split('.').next())
        .unwrap_or(selector);
    let tag = selector
        .split(&['#', '.'][..])
        .next()
        .unwrap_or("div");
    let escaped_tag = regex::escape(tag);
    let escaped_id = regex::escape(id_part);
    let pattern = format!(
        r#"<{escaped_tag}[^>]*\bid\s*=\s*["']{}["'][^>]*>(.*?)</{escaped_tag}>"#,
        escaped_id,
    );
    Regex::new(&pattern)
        .ok()?
        .captures(html)?
        .get(1)
        .map(|m| m.as_str().trim().to_string())
}

fn extract_by_class(html: &str, selector: &str) -> Option<String> {
    let class_part = selector.split('.').nth(1).unwrap_or(selector);
    let tag = selector.split('.').next().unwrap_or("div");
    let escaped_tag = regex::escape(tag);
    let escaped_class = regex::escape(class_part);
    let pattern = format!(
        r#"<{escaped_tag}[^>]*\bclass\s*=\s*["'][^"']*{escaped_class}[^"']*["'][^>]*>(.*?)</{escaped_tag}>"#,
    );
    Regex::new(&pattern)
        .ok()?
        .captures(html)?
        .get(1)
        .map(|m| m.as_str().trim().to_string())
}

fn extract_by_tag(html: &str, tag: &str) -> Option<String> {
    let pattern = format!(
        r"<{tag}[^>]*>(.*?)</{tag}>",
        tag = regex::escape(tag)
    );
    Regex::new(&pattern)
        .ok()?
        .captures(html)?
        .get(1)
        .map(|m| m.as_str().trim().to_string())
}

fn extract_by_xpath(html: &str, xpath: &str) -> Option<String> {
    let stripped = xpath.trim_start_matches("//");
    let tag = stripped.split('[').next().unwrap_or(stripped);
    if stripped.contains("text()") {
        let pattern = format!(
            r"<{tag}[^>]*>([^<]*)</{tag}>",
            tag = regex::escape(tag)
        );
        Regex::new(&pattern)
            .ok()?
            .captures(html)?
            .get(1)
            .map(|m| m.as_str().trim().to_string())
    } else {
        extract_by_tag(html, tag)
    }
}

// ---------------------------------------------------------------------------
// Transform helpers
// ---------------------------------------------------------------------------

fn strip_html_tags(input: &str) -> String {
    let re = Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(input, "").trim().to_string()
}

fn apply_transform(raw: &str, transform: &Option<FieldTransform>) -> Result<String, String> {
    match transform {
        None | Some(FieldTransform::None) => Ok(raw.to_string()),
        Some(FieldTransform::Trim) => Ok(raw.trim().to_string()),
        Some(FieldTransform::UpperCase) => Ok(raw.to_uppercase()),
        Some(FieldTransform::LowerCase) => Ok(raw.to_lowercase()),
        Some(FieldTransform::StripHtml) => Ok(strip_html_tags(raw)),
        Some(FieldTransform::ParseNumber) => {
            let cleaned: String = raw
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect();
            if cleaned.is_empty() {
                Err(format!("cannot parse '{}' as number", raw))
            } else {
                Ok(cleaned)
            }
        }
        Some(FieldTransform::ParseDate) => {
            let cleaned = raw.trim().to_string();
            if cleaned.is_empty() {
                Err("empty date string".into())
            } else {
                Ok(cleaned)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Type coercion
// ---------------------------------------------------------------------------

/// Coerce a raw string to a `serde_json::Value` matching the expected `FieldType`.
pub fn coerce_value(raw: &str, field_type: &FieldType) -> serde_json::Value {
    match field_type {
        FieldType::String => serde_json::Value::String(raw.to_string()),
        FieldType::Number => {
            let cleaned: String = raw
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect();
            if let Ok(n) = cleaned.parse::<f64>() {
                serde_json::Number::from_f64(n)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        FieldType::Currency => {
            let cleaned: String = raw
                .chars()
                .filter(|c| {
                    c.is_ascii_digit()
                        || *c == '.'
                        || *c == '-'
                        || *c == ','
                        || *c == '$'
                        || *c == '€'
                        || *c == '¥'
                        || *c == '£'
                })
                .collect();
            let cleaned = cleaned
                .replace(',', "")
                .replace(|c: char| !c.is_ascii_digit() && c != '.' && c != '-', "");
            if let Ok(n) = cleaned.parse::<f64>() {
                serde_json::Number::from_f64(n)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        FieldType::Date => serde_json::Value::String(raw.trim().to_string()),
        FieldType::Url => serde_json::Value::String(raw.trim().to_string()),
        FieldType::Email => serde_json::Value::String(raw.trim().to_string()),
        FieldType::Array(inner) => {
            // Try JSON array first, then split by delimiters
            if let Ok(json_array) = serde_json::from_str::<Vec<serde_json::Value>>(raw) {
                return serde_json::Value::Array(json_array);
            }
            let items: Vec<serde_json::Value> = raw
                .split(|c| c == ',' || c == ';' || c == '\n')
                .map(|s| coerce_value(s.trim(), inner))
                .filter(|v| !v.is_null())
                .collect();
            serde_json::Value::Array(items)
        }
        FieldType::Nested(_sub_fields) => {
            // Nested requires recursive extraction; raw string as fallback
            serde_json::Value::String(raw.to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// Extractor impl
// ---------------------------------------------------------------------------

impl Extractor {
    /// Create a new `Extractor` with the given name and fields.
    /// The VSA schema encoding is computed automatically.
    pub fn new(name: &str, fields: Vec<ExtractionField>) -> Self {
        let schema = ExtractionSchema {
            name: name.to_string(),
            fields,
            vsa_schema: [0; 4],
        };
        let mut extractor = Self { schema };
        extractor.schema.vsa_schema =
            compute_vsa_schema(&extractor.schema.name, &extractor.schema.fields);
        extractor
    }

    /// Create an `Extractor` from an existing schema (re-computes VSA encoding).
    pub fn new_with_schema(schema: ExtractionSchema) -> Self {
        let mut extractor = Self { schema };
        extractor.schema.vsa_schema =
            compute_vsa_schema(&extractor.schema.name, &extractor.schema.fields);
        extractor
    }

    // -- internal extraction logic --

    fn extract_field(
        &self,
        html: &str,
        field: &ExtractionField,
    ) -> Result<serde_json::Value, String> {
        match &field.field_type {
            FieldType::Nested(sub_fields) => {
                let sub_html = match &field.selector {
                    Some(sel) => extract_by_selector(html, sel).unwrap_or_default(),
                    None => html.to_string(),
                };
                let mut sub_map = serde_json::Map::new();
                for sub in sub_fields {
                    let val = self.extract_field(&sub_html, sub)?;
                    sub_map.insert(sub.name.clone(), val);
                }
                Ok(serde_json::Value::Object(sub_map))
            }
            _ => {
                let raw = match &field.selector {
                    Some(sel) => extract_by_selector(html, sel).unwrap_or_default(),
                    None => String::new(),
                };
                let transformed = apply_transform(&raw, &field.transform)?;
                Ok(coerce_value(&transformed, &field.field_type))
            }
        }
    }

    /// Extract all fields from HTML and return a map of field names to values.
    pub fn extract(&self, html: &str) -> Result<HashMap<String, serde_json::Value>, String> {
        let mut result = HashMap::new();
        for field in &self.schema.fields {
            let value = self.extract_field(html, field)?;
            result.insert(field.name.clone(), value);
        }
        Ok(result)
    }

    /// Extract and serialize to a compact JSON string.
    pub fn extract_json(&self, html: &str) -> Result<String, String> {
        let data = self.extract(html)?;
        serde_json::to_string(&data).map_err(|e| format!("serialization error: {}", e))
    }

    /// Extract and serialize to a pretty-printed JSON string.
    pub fn extract_json_pretty(&self, html: &str) -> Result<String, String> {
        let data = self.extract(html)?;
        serde_json::to_string_pretty(&data).map_err(|e| format!("serialization error: {}", e))
    }

    /// Validate extracted data against the schema.
    /// Returns a list of error messages (empty = valid).
    pub fn validate(&self, data: &HashMap<String, serde_json::Value>) -> Vec<String> {
        let mut errors = Vec::new();
        for field in &self.schema.fields {
            let value = data.get(&field.name);

            // — required check —
            match (field.required, value) {
                (true, None) => {
                    errors.push(format!("missing required field: {}", field.name));
                    continue;
                }
                (true, Some(val)) if val.is_null() => {
                    errors.push(format!("field '{}' is null but required", field.name));
                    continue;
                }
                _ => {}
            }

            // — type check —
            if let Some(val) = value {
                if !val.is_null() {
                    let type_ok = match (&field.field_type, val) {
                        (FieldType::String, v) => v.is_string(),
                        (FieldType::Number, v) => v.is_number(),
                        (FieldType::Currency, v) => v.is_number(),
                        (FieldType::Url, v) => v.is_string(),
                        (FieldType::Email, v) => v.is_string(),
                        (FieldType::Date, v) => v.is_string(),
                        (FieldType::Array(_), v) => v.is_array(),
                        (FieldType::Nested(_), v) => v.is_object(),
                    };
                    if !type_ok {
                        errors.push(format!(
                            "field '{}' type mismatch: expected {:?}, got {}",
                            field.name,
                            field.field_type,
                            val
                        ));
                    }
                }
            }
        }
        errors
    }
}

// ---------------------------------------------------------------------------
// Display impls
// ---------------------------------------------------------------------------

impl fmt::Display for ExtractionSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Schema: {} ({} fields, vsa:{:?})",
            self.name,
            self.fields.len(),
            self.vsa_schema
        )?;
        for field in &self.fields {
            write!(f, "  - {}: {:?}", field.name, field.field_type)?;
            if let Some(ref sel) = field.selector {
                write!(f, " [selector: {}]", sel)?;
            }
            if field.required {
                write!(f, " (required)")?;
            }
            if let Some(ref t) = field.transform {
                if *t != FieldTransform::None {
                    write!(f, " [transform: {:?}]", t)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl fmt::Display for Extractor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.schema)
    }
}

// ---------------------------------------------------------------------------
// Builder helpers for ExtractionField
// ---------------------------------------------------------------------------

impl ExtractionField {
    pub fn new(name: &str, field_type: FieldType) -> Self {
        Self {
            name: name.to_string(),
            selector: None,
            field_type,
            required: false,
            transform: None,
            vsa_role: [0; 2],
        }
    }

    pub fn with_selector(mut self, selector: &str) -> Self {
        self.selector = Some(selector.to_string());
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_transform(mut self, transform: FieldTransform) -> Self {
        self.transform = Some(transform);
        self
    }

    /// Assign a VSA role to this field (e.g. "SUBJECT", "PRICE", "DATE").
    pub fn with_role(mut self, role_name: &str) -> Self {
        self.vsa_role = encode_vsa_role(role_name);
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_HTML: &str = r#"
        <div class="product">
            <h1 id="title">Super Widget</h1>
            <p class="description">The best widget ever!</p>
            <span class="price" data-value="29.99">$29.99</span>
            <a href="https://example.com" class="buy-link">Buy Now</a>
            <div class="specs">
                <span class="weight">1.5 kg</span>
                <span class="color">Red</span>
            </div>
            <ul class="features">
                <li>Feature 1</li>
                <li>Feature 2</li>
                <li>Feature 3</li>
            </ul>
        </div>
    "#;

    fn make_product_extractor() -> Extractor {
        let fields = vec![
            ExtractionField::new("title", FieldType::String)
                .with_selector("h1#title")
                .with_role("SUBJECT")
                .required(),
            ExtractionField::new("price", FieldType::Currency)
                .with_selector("span.price")
                .with_transform(FieldTransform::Trim)
                .with_role("PRICE")
                .required(),
            ExtractionField::new("description", FieldType::String)
                .with_selector("p.description")
                .with_role("PREDICATE"),
            ExtractionField::new("stock_quantity", FieldType::Number)
                .with_selector("span.quantity")
                .required(),
            ExtractionField::new("specs", FieldType::Nested(vec![
                ExtractionField::new("weight", FieldType::String)
                    .with_selector("span.weight"),
                ExtractionField::new("color", FieldType::String)
                    .with_selector("span.color"),
            ]))
            .with_selector("div.specs"),
        ];
        Extractor::new("product", fields)
    }

    // ---- extraction ----

    #[test]
    fn test_extract_title() {
        let extractor = make_product_extractor();
        let result = extractor.extract(TEST_HTML).unwrap();
        assert_eq!(
            result.get("title").and_then(|v| v.as_str()),
            Some("Super Widget")
        );
    }

    #[test]
    fn test_extract_price() {
        let extractor = make_product_extractor();
        let result = extractor.extract(TEST_HTML).unwrap();
        let price = result.get("price").unwrap();
        assert!(price.is_number(), "price should be number: {:?}", price);
        assert!((price.as_f64().unwrap() - 29.99).abs() < 0.01);
    }

    #[test]
    fn test_extract_optional_field() {
        let extractor = make_product_extractor();
        let result = extractor.extract(TEST_HTML).unwrap();
        assert!(result.get("description").is_some());
    }

    #[test]
    fn test_extract_nested() {
        let extractor = make_product_extractor();
        let result = extractor.extract(TEST_HTML).unwrap();
        let specs = result.get("specs");
        assert!(specs.is_some(), "specs should be present");
        if let Some(serde_json::Value::Object(map)) = specs {
            assert!(map.contains_key("weight"));
            assert!(map.contains_key("color"));
        } else {
            panic!("specs should be an object, got {:?}", specs);
        }
    }

    // ---- JSON ----

    #[test]
    fn test_extract_json_output() {
        let extractor = make_product_extractor();
        let json_str = extractor.extract_json(TEST_HTML).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("title").is_some());
    }

    #[test]
    fn test_extract_json_pretty() {
        let extractor = make_product_extractor();
        let json_str = extractor.extract_json_pretty(TEST_HTML).unwrap();
        assert!(json_str.contains('\n'), "pretty should have newlines");
    }

    // ---- coercion ----

    #[test]
    fn test_coerce_string() {
        assert_eq!(
            coerce_value("hello", &FieldType::String),
            serde_json::Value::String("hello".into())
        );
    }

    #[test]
    fn test_coerce_number() {
        let val = coerce_value("42.99", &FieldType::Number);
        assert!(val.is_number());
        assert!((val.as_f64().unwrap() - 42.99).abs() < 0.01);
    }

    #[test]
    fn test_coerce_number_invalid() {
        let val = coerce_value("abc", &FieldType::Number);
        assert!(val.is_null());
    }

    #[test]
    fn test_coerce_currency() {
        let val = coerce_value("$1,234.56", &FieldType::Currency);
        assert!(val.is_number());
        assert!((val.as_f64().unwrap() - 1234.56).abs() < 0.01);
    }

    #[test]
    fn test_coerce_currency_eur() {
        let val = coerce_value("€49.99", &FieldType::Currency);
        assert!(val.is_number());
        assert!((val.as_f64().unwrap() - 49.99).abs() < 0.01);
    }

    #[test]
    fn test_coerce_currency_yen() {
        let val = coerce_value("¥2500", &FieldType::Currency);
        assert!(val.is_number());
        assert!((val.as_f64().unwrap() - 2500.0).abs() < 0.01);
    }

    #[test]
    fn test_coerce_array_csv() {
        let val = coerce_value("a,b,c", &FieldType::Array(Box::new(FieldType::String)));
        assert!(val.is_array());
        assert_eq!(val.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_coerce_array_json() {
        let val = coerce_value(
            r#"["x","y","z"]"#,
            &FieldType::Array(Box::new(FieldType::String)),
        );
        assert!(val.is_array());
        assert_eq!(val.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_coerce_array_semicolon() {
        let val = coerce_value(
            "x;y;z",
            &FieldType::Array(Box::new(FieldType::String)),
        );
        assert!(val.is_array());
        assert_eq!(val.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_coerce_date() {
        let val = coerce_value("2024-01-15", &FieldType::Date);
        assert_eq!(val, serde_json::Value::String("2024-01-15".into()));
    }

    #[test]
    fn test_coerce_url() {
        let val = coerce_value("https://example.com", &FieldType::Url);
        assert_eq!(
            val,
            serde_json::Value::String("https://example.com".into())
        );
    }

    #[test]
    fn test_coerce_email() {
        let val = coerce_value("user@example.com", &FieldType::Email);
        assert_eq!(
            val,
            serde_json::Value::String("user@example.com".into())
        );
    }

    // ---- transform ----

    #[test]
    fn test_transform_trim() {
        let result = apply_transform("  hello  ", &Some(FieldTransform::Trim)).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_transform_upper() {
        let result = apply_transform("hello", &Some(FieldTransform::UpperCase)).unwrap();
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_transform_lower() {
        let result = apply_transform("HELLO", &Some(FieldTransform::LowerCase)).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_transform_strip_html() {
        let result =
            apply_transform("<b>bold</b> text", &Some(FieldTransform::StripHtml)).unwrap();
        assert_eq!(result, "bold text");
    }

    #[test]
    fn test_transform_parse_number() {
        let result =
            apply_transform(" 42.5px ", &Some(FieldTransform::ParseNumber)).unwrap();
        assert_eq!(result, "42.5");
    }

    #[test]
    fn test_transform_parse_number_fail() {
        let result = apply_transform("abc", &Some(FieldTransform::ParseNumber));
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_parse_date() {
        let result =
            apply_transform(" 2024-06-15 ", &Some(FieldTransform::ParseDate)).unwrap();
        assert_eq!(result, "2024-06-15");
    }

    #[test]
    fn test_transform_parse_date_empty() {
        let result = apply_transform("", &Some(FieldTransform::ParseDate));
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_none() {
        let result = apply_transform("raw", &None).unwrap();
        assert_eq!(result, "raw");
    }

    // ---- validation ----

    #[test]
    fn test_validate_missing_field() {
        let extractor = make_product_extractor();
        let mut data = HashMap::new();
        data.insert("title".into(), serde_json::Value::String("Test".into()));
        data.insert("price".into(), serde_json::json!(19.99));
        // stock_quantity is required but missing
        let errors = extractor.validate(&data);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("stock_quantity")));
    }

    #[test]
    fn test_validate_type_mismatch() {
        let extractor = make_product_extractor();
        let mut data = HashMap::new();
        data.insert("title".into(), serde_json::Value::String("Test".into()));
        data.insert(
            "price".into(),
            serde_json::Value::String("not-a-number".into()),
        );
        data.insert("stock_quantity".into(), serde_json::json!(100));
        let errors = extractor.validate(&data);
        assert!(errors.iter().any(|e| e.contains("price") && e.contains("type mismatch")));
    }

    #[test]
    fn test_validate_pass() {
        let extractor = make_product_extractor();
        let mut data = HashMap::new();
        data.insert("title".into(), serde_json::Value::String("Test".into()));
        data.insert("price".into(), serde_json::json!(19.99));
        data.insert("stock_quantity".into(), serde_json::json!(100));
        data.insert("description".into(), serde_json::Value::String("desc".into()));
        let errors = extractor.validate(&data);
        assert!(errors.is_empty(), "expected no errors: {:?}", errors);
    }

    #[test]
    fn test_validate_no_errors_for_optional_null() {
        let fields = vec![
            ExtractionField::new("name", FieldType::String).required(),
            ExtractionField::new("nickname", FieldType::String),
        ];
        let schema = ExtractionSchema {
            name: "person".into(),
            vsa_schema: [0; 4],
            fields,
        };
        let mut data = HashMap::new();
        data.insert(
            "name".into(),
            serde_json::Value::String("Alice".into()),
        );
        let extractor = Extractor::new_with_schema(schema);
        let errors = extractor.validate(&data);
        assert!(errors.is_empty());
    }

    // ---- VSA ----

    #[test]
    fn test_vsa_role_encoding() {
        let role = encode_vsa_role("PRICE");
        assert_ne!(role[0], 0);
        assert_ne!(role[1], 0);
        let role2 = encode_vsa_role("PRICE");
        assert_eq!(role, role2, "VSA encoding should be deterministic");
    }

    #[test]
    fn test_vsa_role_different() {
        let price = encode_vsa_role("PRICE");
        let date = encode_vsa_role("DATE");
        assert_ne!(price, date, "different roles should differ");
    }

    #[test]
    fn test_schema_vsa_computation() {
        let extractor = make_product_extractor();
        let vsa = extractor.schema.vsa_schema;
        assert_ne!(vsa, [0; 4], "VSA schema should be non-zero");
        let extractor2 = make_product_extractor();
        assert_eq!(
            extractor.schema.vsa_schema,
            extractor2.schema.vsa_schema
        );
    }

    // ---- builder ----

    #[test]
    fn test_field_builder() {
        let field = ExtractionField::new("email", FieldType::Email)
            .with_selector("input#email")
            .required()
            .with_role("OBJECT");
        assert!(field.required);
        assert_eq!(field.name, "email");
        assert_eq!(field.selector, Some("input#email".into()));
        assert_ne!(field.vsa_role, [0; 2]);
    }

    // ---- Display ----

    #[test]
    fn test_schema_display() {
        let fields = vec![
            ExtractionField::new("name", FieldType::String).required(),
            ExtractionField::new("price", FieldType::Currency)
                .with_selector("span.price"),
        ];
        let schema = ExtractionSchema {
            name: "test".into(),
            fields,
            vsa_schema: [1, 2, 3, 4],
        };
        let display = format!("{}", schema);
        assert!(display.contains("test"));
        assert!(display.contains("name"));
        assert!(display.contains("price"));
    }

    #[test]
    fn test_extractor_display() {
        let extractor = make_product_extractor();
        let display = format!("{}", extractor);
        assert!(display.contains("product"));
        assert!(display.contains("title"));
    }

    // ---- error handling ----

    #[test]
    fn test_extract_returns_err_on_bad_transform() {
        let fields = vec![
            ExtractionField::new("qty", FieldType::Number)
                .with_selector("span.quantity")
                .with_transform(FieldTransform::ParseNumber),
        ];
        let extractor = Extractor::new("inventory", fields);
        let html = r#"<span class="quantity">N/A</span>"#;
        // ParseNumber will fail on "N/A"
        let result = extractor.extract(html);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_missing_selector() {
        let fields = vec![
            ExtractionField::new("fallback", FieldType::String)
                .with_selector("span.nonexistent"),
        ];
        let extractor = Extractor::new("test", fields);
        let result = extractor.extract("<div>no match</div>").unwrap();
        assert_eq!(
            result.get("fallback").and_then(|v| v.as_str()),
            Some("")
        );
    }

    #[test]
    fn test_empty_schema() {
        let extractor = Extractor::new("empty", vec![]);
        let result = extractor.extract("<html></html>").unwrap();
        assert!(result.is_empty());
        let json = extractor.extract_json("<html></html>").unwrap();
        assert_eq!(json, "{}");
    }

    // ---- extract_by_selector integration ----

    #[test]
    fn test_extract_by_tag_selector() {
        let result = extract_by_selector(
            r#"<div class="x"><span>inner</span></div>"#,
            "span",
        );
        assert_eq!(result, Some("inner".into()));
    }

    #[test]
    fn test_extract_by_xpath_selector() {
        let result = extract_by_selector(
            r#"<p class="desc">Hello</p>"#,
            "//p",
        );
        assert_eq!(result, Some("Hello".into()));
    }

    #[test]
    fn test_extract_by_xpath_with_text() {
        let result = extract_by_selector(
            r#"<p>Hello world</p>"#,
            "//p[contains(text(),'Hello')]",
        );
        assert_eq!(result, Some("Hello world".into()));
    }
}
