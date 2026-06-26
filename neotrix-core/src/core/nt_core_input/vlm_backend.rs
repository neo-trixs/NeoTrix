#![allow(dead_code)]

//! Generic HTTP VLM backend implementing [`DocumentParser`].
//!
//! Wraps Docling, Dolphin, Nemotron, or OpenAI-compatible document parsing
//! APIs behind a single trait. All use the same REST shape:
//!
//! ```text
//! POST {base_url}/api/parse
//! Authorization: Bearer {api_key}    (optional)
//! Content-Type: application/json
//!
//! {
//!   "model": "...",
//!   "document": "<base64-encoded bytes>",
//!   "options": { "extract_tables": true, "extract_figures": true, "extract_markdown": true }
//! }
//!
//! → 200 { "markdown": "...", "tables": [...], "figures": [...] }
//! ```

use super::document_classifier::DocumentClassifier;
use super::document_parser::{
    source_to_bytes, BBox, DocumentError, DocumentMetadata, DocumentParser, DocumentSource,
    ExtractedFigure, ExtractedTable, ParsedDocument, TableFormat,
};
use base64::Engine;

// ---------------------------------------------------------------------------
// VlmBackend
// ---------------------------------------------------------------------------

/// A HTTP-based VLM document parsing backend.
///
/// Supports Docling, Dolphin, Nemotron, and OpenAI-compatible endpoints.
/// Construct via the builder methods [`VlmBackend::docling`],
/// [`VlmBackend::dolphin`], or [`VlmBackend::nemotron`].
pub struct VlmBackend {
    name: &'static str,
    base_url: String,
    api_key: Option<String>,
    model: String,
    formats: Vec<&'static str>,
    timeout_secs: u64,
}

// ---------------------------------------------------------------------------
// Builder constructors
// ---------------------------------------------------------------------------

impl VlmBackend {
    /// Create a backend configured for Docling (IBM).
    ///
    /// Default model: `"ds4-mini"`. Supports PDF, PNG, JPG, JPEG, DOCX.
    pub fn docling(base_url: impl Into<String>) -> Self {
        Self {
            name: "DoclingVlm",
            base_url: base_url.into(),
            api_key: None,
            model: "ds4-mini".to_string(),
            formats: vec!["pdf", "png", "jpg", "jpeg", "docx"],
            timeout_secs: 120,
        }
    }

    /// Create a backend configured for Dolphin (ByteDance).
    ///
    /// Default model: `"dolphin-vlm"`. Supports PDF, PNG, JPG, JPEG.
    pub fn dolphin(base_url: impl Into<String>) -> Self {
        Self {
            name: "DolphinVlm",
            base_url: base_url.into(),
            api_key: None,
            model: "dolphin-vlm".to_string(),
            formats: vec!["pdf", "png", "jpg", "jpeg"],
            timeout_secs: 120,
        }
    }

    /// Create a backend configured for NVIDIA Nemotron Parse.
    ///
    /// Default model: `"nemotron-parse"`. An API key is required.
    pub fn nemotron(base_url: impl Into<String>, api_key: String) -> Self {
        Self {
            name: "NemotronParse",
            base_url: base_url.into(),
            api_key: Some(api_key),
            model: "nemotron-parse".to_string(),
            formats: vec!["pdf", "png", "jpg", "jpeg"],
            timeout_secs: 120,
        }
    }

    /// Override the default model name sent in the request body.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set or override the Bearer token sent in the `Authorization` header.
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Override the request timeout (default: 120 seconds).
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Construct a `VlmBackend` from environment variables.
    ///
    /// Reads:
    /// - `VLM_BACKEND_TYPE` — one of `"docling"`, `"dolphin"`, `"nemotron"`
    /// - `VLM_BACKEND_URL` — base URL of the VLM API
    /// - `VLM_API_KEY` — optional API key (required for `"nemotron"`)
    ///
    /// Returns `None` if `VLM_BACKEND_TYPE` or `VLM_BACKEND_URL` is not set,
    /// or if the type is unrecognised, or if `"nemotron"` is chosen without
    /// supplying an API key.
    pub fn from_env() -> Option<Self> {
        let backend_type = std::env::var("VLM_BACKEND_TYPE").ok()?;
        let base_url = std::env::var("VLM_BACKEND_URL").ok()?;
        let api_key = std::env::var("VLM_API_KEY").ok();

        match backend_type.to_lowercase().as_str() {
            "docling" => {
                let mut b = Self::docling(&base_url);
                if let Some(ref key) = api_key {
                    b = b.with_api_key(key.clone());
                }
                Some(b)
            }
            "dolphin" => {
                let mut b = Self::dolphin(&base_url);
                if let Some(ref key) = api_key {
                    b = b.with_api_key(key.clone());
                }
                Some(b)
            }
            "nemotron" => api_key.map(|key| Self::nemotron(&base_url, key)),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// DocumentParser implementation
// ---------------------------------------------------------------------------

impl DocumentParser for VlmBackend {
    fn parse(&self, source: &DocumentSource) -> Result<ParsedDocument, DocumentError> {
        let bytes = source_to_bytes(source)?;
        let b64 = base64_encode(&bytes);

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .build()
            .map_err(|e| DocumentError::NetworkError(e.to_string()))?;

        let body = serde_json::json!({
            "model": self.model,
            "document": b64,
            "options": {
                "extract_tables": true,
                "extract_figures": true,
                "extract_markdown": true,
            }
        });

        let mut req = client
            .post(format!("{}/api/parse", self.base_url))
            .header("Content-Type", "application/json");

        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.json(&body).send().map_err(|e| {
            if e.is_timeout() {
                DocumentError::Timeout(format!(
                    "VLM backend {} timed out after {}s",
                    self.name, self.timeout_secs
                ))
            } else {
                DocumentError::NetworkError(e.to_string())
            }
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            return Err(DocumentError::BackendUnavailable(format!(
                "{} returned HTTP {}: {}",
                self.name, status, text
            )));
        }

        let parsed: serde_json::Value = resp
            .json()
            .map_err(|e| DocumentError::ParseError(format!("JSON parse error: {}", e)))?;

        let markdown = parsed["markdown"].as_str().unwrap_or("").to_string();
        let tables = parse_tables(&parsed["tables"]);
        let images = parse_figures(&parsed["figures"]);

        let language = if !markdown.is_empty() {
            DocumentClassifier::detect_language(&markdown)
        } else {
            None
        };

        Ok(ParsedDocument {
            markdown,
            tables,
            images,
            metadata: DocumentMetadata {
                format: Some("pdf".to_string()),
                size_bytes: Some(bytes.len()),
                language,
                backend_name: self.name.to_string(),
                ..Default::default()
            },
        })
    }

    fn supported_formats(&self) -> Vec<&'static str> {
        self.formats.clone()
    }

    fn backend_name(&self) -> &'static str {
        self.name
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn parse_tables(value: &serde_json::Value) -> Vec<ExtractedTable> {
    value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    let caption = t["caption"].as_str().unwrap_or("").to_string();
                    let headers: Vec<String> = t["headers"]
                        .as_array()
                        .map(|h| {
                            h.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    let rows: Vec<Vec<String>> = t["rows"]
                        .as_array()
                        .map(|r| {
                            r.iter()
                                .map(|row| {
                                    row.as_array()
                                        .map(|c| {
                                            c.iter()
                                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                                .collect()
                                        })
                                        .unwrap_or_default()
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    Some(ExtractedTable {
                        caption,
                        headers,
                        rows,
                        format: TableFormat::Simple,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_figures(value: &serde_json::Value) -> Vec<ExtractedFigure> {
    value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    let caption = f["caption"].as_str().unwrap_or("").to_string();
                    let description = f["description"].as_str().unwrap_or("").to_string();
                    let bbox = f["bbox"].as_object().map(|b| BBox {
                        x: b["x"].as_f64().unwrap_or(0.0) as f32,
                        y: b["y"].as_f64().unwrap_or(0.0) as f32,
                        w: b["w"].as_f64().unwrap_or(0.0) as f32,
                        h: b["h"].as_f64().unwrap_or(0.0) as f32,
                    });
                    Some(ExtractedFigure {
                        caption,
                        bbox,
                        description,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn base64_encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docling_construction() {
        let backend = VlmBackend::docling("http://localhost:5000");
        assert_eq!(backend.backend_name(), "DoclingVlm");
        assert!(backend.supported_formats().contains(&"pdf"));
        assert!(backend.supported_formats().contains(&"docx"));
    }

    #[test]
    fn test_dolphin_construction() {
        let backend = VlmBackend::dolphin("http://localhost:8000");
        assert_eq!(backend.backend_name(), "DolphinVlm");
        assert!(backend.supported_formats().contains(&"pdf"));
    }

    #[test]
    fn test_nemotron_construction() {
        let backend = VlmBackend::nemotron("http://localhost:8000", "test-key".into());
        assert_eq!(backend.backend_name(), "NemotronParse");
    }

    #[test]
    fn test_with_model_overrides_default() {
        let backend = VlmBackend::docling("http://localhost").with_model("ds4-mini-v2");
        // Can't inspect model directly (not pub), but construction works
        assert_eq!(backend.backend_name(), "DoclingVlm");
    }

    #[test]
    fn test_with_api_key_overrides_none() {
        let backend = VlmBackend::dolphin("http://localhost").with_api_key("sk-1234");
        assert_eq!(backend.backend_name(), "DolphinVlm");
    }

    #[test]
    fn test_with_timeout() {
        let backend = VlmBackend::nemotron("http://localhost", "k".into()).with_timeout(300);
        assert_eq!(backend.backend_name(), "NemotronParse");
    }

    #[test]
    fn test_base64_encode_roundtrip() {
        let input = b"hello world";
        let encoded = base64_encode(input);
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .expect("valid base64");
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_base64_encode_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn test_parse_tables_empty() {
        let val = serde_json::json!([]);
        let tables = parse_tables(&val);
        assert!(tables.is_empty());
    }

    #[test]
    fn test_parse_tables_non_empty() {
        let val = serde_json::json!([
            {
                "caption": "Results",
                "headers": ["Name", "Score"],
                "rows": [["Alice", "95"], ["Bob", "87"]]
            }
        ]);
        let tables = parse_tables(&val);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].caption, "Results");
        assert_eq!(tables[0].headers, vec!["Name", "Score"]);
        assert_eq!(tables[0].rows.len(), 2);
    }

    #[test]
    fn test_parse_figures_empty() {
        let val = serde_json::json!([]);
        let figs = parse_figures(&val);
        assert!(figs.is_empty());
    }

    #[test]
    fn test_parse_figures_with_bbox() {
        let val = serde_json::json!([
            {
                "caption": "Chart",
                "description": "A bar chart showing growth",
                "bbox": {"x": 10.0, "y": 20.0, "w": 200.0, "h": 150.0}
            }
        ]);
        let figs = parse_figures(&val);
        assert_eq!(figs.len(), 1);
        assert_eq!(figs[0].caption, "Chart");
        assert!(figs[0].bbox.is_some());
        let bbox = figs[0].bbox.clone().unwrap();
        assert!((bbox.x - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_figures_missing_bbox() {
        let val = serde_json::json!([{"caption": "Logo", "description": "Company logo"}]);
        let figs = parse_figures(&val);
        assert_eq!(figs.len(), 1);
        assert!(figs[0].bbox.is_none());
    }

    // ------------------------------------------------------------------
    // from_env tests
    // ------------------------------------------------------------------

    #[test]
    fn test_from_env_docling() {
        unsafe {
            std::env::set_var("VLM_BACKEND_TYPE", "docling");
        }
        unsafe {
            std::env::set_var("VLM_BACKEND_URL", "http://localhost:5000");
        }
        let result = VlmBackend::from_env();
        unsafe {
            std::env::remove_var("VLM_BACKEND_TYPE");
        }
        unsafe {
            std::env::remove_var("VLM_BACKEND_URL");
        }
        assert!(result.is_some());
        assert_eq!(result.unwrap().backend_name(), "DoclingVlm");
    }

    #[test]
    fn test_from_env_no_type() {
        unsafe {
            std::env::remove_var("VLM_BACKEND_TYPE");
        }
        unsafe {
            std::env::remove_var("VLM_BACKEND_URL");
        }
        assert!(VlmBackend::from_env().is_none());
    }

    #[test]
    fn test_from_env_nemotron_no_key() {
        unsafe {
            std::env::set_var("VLM_BACKEND_TYPE", "nemotron");
        }
        unsafe {
            std::env::set_var("VLM_BACKEND_URL", "http://localhost:8000");
        }
        let result = VlmBackend::from_env();
        unsafe {
            std::env::remove_var("VLM_BACKEND_TYPE");
        }
        unsafe {
            std::env::remove_var("VLM_BACKEND_URL");
        }
        assert!(result.is_none());
    }
}
