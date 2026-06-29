use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExtractionBridge {
    pipeline_history: Vec<ExtractionResult>,
    max_history: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractionResult {
    pub source_url: String,
    pub title: String,
    pub content_markdown: String,
    pub extracted_fields: HashMap<String, serde_json::Value>,
    pub confidence: f64,
    pub vsa_fingerprint: [u64; 4],
    pub extracted_at_ms: u64,
    pub extraction_method: ExtractionMethod,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ExtractionMethod {
    BrowserAgent, HtmlParse, DocumentConversion, FinancialApi, CaptchaSolve,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgePackage {
    pub entries: Vec<KnowledgeEntryPackage>,
    pub relations: Vec<RelationPackage>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeEntryPackage {
    pub id: String,
    pub title: String,
    pub body: String,
    pub summary: String,
    pub source_url: String,
    pub tags: Vec<String>,
    pub confidence: f64,
    pub importance: f64,
    pub vsa_bytes: Option<[u64; 4]>,
    pub extracted_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelationPackage {
    pub from_id: String,
    pub to_id: String,
    pub relation_type: String,
    pub weight: f64,
    pub description: String,
}

impl ExtractionBridge {
    pub fn new(max_history: usize) -> Self {
        Self { pipeline_history: Vec::with_capacity(max_history), max_history }
    }

    pub fn record_extraction(&mut self, result: ExtractionResult) {
        self.pipeline_history.push(result);
        if self.pipeline_history.len() > self.max_history {
            self.pipeline_history.remove(0);
        }
    }

    pub fn to_knowledge_package(&self, results: &[ExtractionResult]) -> KnowledgePackage {
        let mut entries = Vec::new();
        let mut relations = Vec::new();

        for (i, r) in results.iter().enumerate() {
            let entry = KnowledgeEntryPackage {
                id: format!("ext_{}", i),
                title: r.title.clone(),
                body: r.content_markdown.clone(),
                summary: r.content_markdown.chars().take(200).collect(),
                source_url: r.source_url.clone(),
                tags: r.tags.clone(),
                confidence: r.confidence,
                importance: r.confidence * 0.8,
                vsa_bytes: Some(r.vsa_fingerprint),
                extracted_fields: r.extracted_fields.clone(),
            };
            entries.push(entry);
        }

        for i in 1..entries.len() {
            relations.push(RelationPackage {
                from_id: entries[i - 1].id.clone(),
                to_id: entries[i].id.clone(),
                relation_type: "sequential_extraction".into(),
                weight: 0.7,
                description: format!("extracted after {}", entries[i - 1].title),
            });
        }

        KnowledgePackage { entries, relations }
    }

    pub fn from_browser_extraction(
        url: &str, html: &str, fields: HashMap<String, serde_json::Value>,
    ) -> ExtractionResult {
        let title = Self::extract_title(html);
        ExtractionResult {
            source_url: url.into(),
            title,
            content_markdown: Self::simple_html_to_text(html),
            extracted_fields: fields,
            confidence: 0.7,
            vsa_fingerprint: Self::compute_vsa(url, html),
            extracted_at_ms: Self::now_ms(),
            extraction_method: ExtractionMethod::BrowserAgent,
            tags: vec!["web_extraction".into(), "browser".into()],
        }
    }

    pub fn from_document_conversion(url: &str, markdown: &str) -> ExtractionResult {
        let title = markdown.lines().next().unwrap_or("Untitled").trim_start_matches("# ").to_string();
        ExtractionResult {
            source_url: url.into(),
            title,
            content_markdown: markdown.into(),
            extracted_fields: HashMap::new(),
            confidence: 0.8,
            vsa_fingerprint: Self::compute_vsa(url, markdown),
            extracted_at_ms: Self::now_ms(),
            extraction_method: ExtractionMethod::DocumentConversion,
            tags: vec!["document".into(), "conversion".into()],
        }
    }

    pub fn from_financial_data(
        symbol: &str, data_type: &str, json_data: serde_json::Value,
    ) -> ExtractionResult {
        let mut fields = HashMap::new();
        fields.insert("data_type".into(), serde_json::Value::String(data_type.into()));
        fields.insert("symbol".into(), serde_json::Value::String(symbol.into()));
        fields.insert("data".into(), json_data);
        ExtractionResult {
            source_url: format!("finance://{}", symbol),
            title: format!("Financial Data: {} ({})", symbol, data_type),
            content_markdown: format!("# Financial Data: {}\n\n{}", symbol, data_type),
            extracted_fields: fields,
            confidence: 0.85,
            vsa_fingerprint: Self::compute_vsa(symbol, data_type),
            extracted_at_ms: Self::now_ms(),
            extraction_method: ExtractionMethod::FinancialApi,
            tags: vec!["finance".into(), symbol.into(), data_type.into()],
        }
    }

    pub fn recent_results(&self, count: usize) -> Vec<&ExtractionResult> {
        self.pipeline_history.iter().rev().take(count).collect()
    }

    pub fn pipeline_report(&self) -> String {
        let total = self.pipeline_history.len();
        let by_method: HashMap<ExtractionMethod, usize> = self.pipeline_history.iter()
            .fold(HashMap::new(), |mut acc, r| {
                *acc.entry(r.extraction_method).or_insert(0) += 1;
                acc
            });
        let methods: String = by_method.iter()
            .map(|(m, c)| format!("{:?}:{}", m, c))
            .collect::<Vec<_>>()
            .join(",");
        format!("ExtBridge[total={} methods=[{}] avg_conf={:.2}]",
            total, methods,
            if total > 0 { self.pipeline_history.iter().map(|r| r.confidence).sum::<f64>() / total as f64 } else { 0.0 })
    }

    fn extract_title(html: &str) -> String {
        if let Some(start) = html.find("<title>") {
            let after = &html[start + 7..];
            if let Some(end) = after.find("</title>") {
                return after[..end].to_string();
            }
        }
        if let Some(start) = html.find("<h1") {
            let after = &html[start..];
            if let Some(gt) = after.find('>') {
                let after_gt = &after[gt + 1..];
                if let Some(end) = after_gt.find("</h1>") {
                    return after_gt[..end].to_string();
                }
            }
        }
        "Untitled".into()
    }

    fn simple_html_to_text(html: &str) -> String {
        html.replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<p>", "\n")
            .replace("</p>", "\n")
            .replace("<div>", "\n")
            .replace("</div>", "")
            .replace("<li>", "\n- ")
            .replace("</li>", "")
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn compute_vsa(a: &str, b: &str) -> [u64; 4] {
        let combined: Vec<u8> = a.bytes().chain(b.bytes()).collect();
        let h1 = combined.iter().enumerate().fold(0u64, |acc, (i, b)| acc.wrapping_mul(31).wrapping_add(*b as u64 ^ (i as u64 * 7)));
        let h2 = combined.iter().rev().enumerate().fold(0u64, |acc, (i, b)| acc.wrapping_mul(37).wrapping_add(*b as u64 ^ (i as u64 * 13)));
        let h3 = combined.iter().step_by(2).fold(0u64, |acc, b| acc.wrapping_mul(41).wrapping_add(*b as u64));
        let h4 = combined.iter().skip(1).step_by(2).fold(0u64, |acc, b| acc.wrapping_mul(43).wrapping_add(*b as u64));
        [h1 ^ h3, h2 ^ h4, h1.wrapping_add(h2), h3.wrapping_add(h4)]
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let bridge = ExtractionBridge::new(100);
        assert_eq!(bridge.recent_results(10).len(), 0);
    }

    #[test]
    fn test_record_extraction() {
        let mut bridge = ExtractionBridge::new(100);
        let result = ExtractionResult {
            source_url: "https://example.com".into(),
            title: "Test".into(),
            content_markdown: "# Test".into(),
            extracted_fields: HashMap::new(),
            confidence: 0.9,
            vsa_fingerprint: [1, 2, 3, 4],
            extracted_at_ms: 0,
            extraction_method: ExtractionMethod::HtmlParse,
            tags: vec!["test".into()],
        };
        bridge.record_extraction(result);
        assert_eq!(bridge.recent_results(10).len(), 1);
    }

    #[test]
    fn test_to_knowledge_package() {
        let bridge = ExtractionBridge::new(100);
        let r1 = ExtractionResult {
            source_url: "https://a.com".into(), title: "A".into(),
            content_markdown: "Content A".into(), extracted_fields: HashMap::new(),
            confidence: 0.8, vsa_fingerprint: [0; 4], extracted_at_ms: 0,
            extraction_method: ExtractionMethod::BrowserAgent, tags: vec![],
        };
        let r2 = ExtractionResult {
            source_url: "https://b.com".into(), title: "B".into(),
            content_markdown: "Content B".into(), extracted_fields: HashMap::new(),
            confidence: 0.9, vsa_fingerprint: [1; 4], extracted_at_ms: 1,
            extraction_method: ExtractionMethod::HtmlParse, tags: vec![],
        };
        let pkg = bridge.to_knowledge_package(&[r1, r2]);
        assert_eq!(pkg.entries.len(), 2);
        assert_eq!(pkg.relations.len(), 1);
        assert_eq!(pkg.relations[0].relation_type, "sequential_extraction");
    }

    #[test]
    fn test_from_browser_extraction() {
        let html = "<html><head><title>Hello World</title></head><body><p>Test</p></body></html>";
        let result = ExtractionBridge::from_browser_extraction("https://example.com", html, HashMap::new());
        assert_eq!(result.title, "Hello World");
        assert_eq!(result.extraction_method, ExtractionMethod::BrowserAgent);
    }

    #[test]
    fn test_from_document_conversion() {
        let md = "# Document Title\n\nSome content";
        let result = ExtractionBridge::from_document_conversion("https://doc.com/report", md);
        assert_eq!(result.title, "Document Title");
        assert_eq!(result.extraction_method, ExtractionMethod::DocumentConversion);
    }

    #[test]
    fn test_from_financial_data() {
        let json = serde_json::json!({"price": 42.5, "volume": 10000});
        let result = ExtractionBridge::from_financial_data("AAPL", "quote", json);
        assert_eq!(result.title, "Financial Data: AAPL (quote)");
        assert!(result.tags.contains(&"finance".to_string()));
    }

    #[test]
    fn test_pipeline_report() {
        let mut bridge = ExtractionBridge::new(100);
        bridge.record_extraction(ExtractionResult {
            source_url: "https://a.com".into(), title: "A".into(),
            content_markdown: "".into(), extracted_fields: HashMap::new(),
            confidence: 0.8, vsa_fingerprint: [0; 4], extracted_at_ms: 0,
            extraction_method: ExtractionMethod::HtmlParse, tags: vec![],
        });
        let report = bridge.pipeline_report();
        assert!(report.starts_with("ExtBridge["));
        assert!(report.contains("HtmlParse:1"));
    }

    #[test]
    fn test_vsa_fingerprint_deterministic() {
        let a = ExtractionBridge::compute_vsa("url1", "content1");
        let b = ExtractionBridge::compute_vsa("url1", "content1");
        assert_eq!(a, b);
    }

    #[test]
    fn test_vsa_fingerprint_different() {
        let a = ExtractionBridge::compute_vsa("url1", "content1");
        let b = ExtractionBridge::compute_vsa("url2", "content2");
        assert_ne!(a, b);
    }

    #[test]
    fn test_history_bounded() {
        let mut bridge = ExtractionBridge::new(5);
        for i in 0..20 {
            bridge.record_extraction(ExtractionResult {
                source_url: format!("https://{}.com", i), title: i.to_string(),
                content_markdown: "".into(), extracted_fields: HashMap::new(),
                confidence: 0.5, vsa_fingerprint: [i; 4], extracted_at_ms: i as u64,
                extraction_method: ExtractionMethod::HtmlParse, tags: vec![],
            });
        }
        assert_eq!(bridge.recent_results(100).len(), 5);
    }

    #[test]
    fn test_knowledge_package_entry_has_vsa() {
        let bridge = ExtractionBridge::new(100);
        let r = ExtractionResult {
            source_url: "https://x.com".into(), title: "X".into(),
            content_markdown: "test".into(), extracted_fields: HashMap::new(),
            confidence: 0.9, vsa_fingerprint: [42, 43, 44, 45], extracted_at_ms: 0,
            extraction_method: ExtractionMethod::FinancialApi, tags: vec!["finance".into()],
        };
        let pkg = bridge.to_knowledge_package(&[r]);
        assert!(pkg.entries[0].vsa_bytes.is_some());
        assert_eq!(pkg.entries[0].vsa_bytes.unwrap(), [42, 43, 44, 45]);
    }

    #[test]
    fn test_simple_html_to_text_removes_tags() {
        let html = "<p>Hello</p><br/><li>Item</li>";
        let text = ExtractionBridge::simple_html_to_text(html);
        assert!(!text.contains('<'));
        assert!(text.contains("Hello"));
        assert!(text.contains("Item"));
    }
}
