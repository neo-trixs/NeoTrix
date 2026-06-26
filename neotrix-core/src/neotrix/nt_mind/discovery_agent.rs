use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::neotrix::nt_mind::exploration_pipeline::ExplorationPipeline;

const SEARCH_QUERIES: [&str; 5] = [
    "Rust AI agent framework",
    "self-evolving LLM systems",
    "agent orchestration autonomous",
    "code generation LLM tools",
    "multi-agent reasoning systems",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub source: DiscoverySource,
    pub url: String,
    pub title: String,
    pub relevance_score: f64,
    pub matched_dimensions: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoverySource {
    ArXiv,
    GitHub,
    SemanticScholar,
}

pub struct DiscoveryAgent {
    pub pipeline: Option<ExplorationPipeline>,
    pub discovered: Vec<DiscoveryResult>,
    processed_urls: HashMap<String, bool>,
    _work_dir: PathBuf,
    pub auto_discovery_enabled: bool,
}

impl DiscoveryAgent {
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            pipeline: None,
            discovered: Vec::new(),
            processed_urls: HashMap::new(),
            _work_dir: work_dir,
            auto_discovery_enabled: true,
        }
    }

    pub fn attach_pipeline(&mut self, pipeline: ExplorationPipeline) {
        self.pipeline = Some(pipeline);
    }

    pub fn run_discovery_cycle(&mut self) -> DiscoveryCycleReport {
        let mut report = DiscoveryCycleReport::default();

        if !self.auto_discovery_enabled {
            report.skipped_reason = Some("auto-discovery disabled".into());
            return report;
        }

        // Phase 1: Search arXiv
        let arxiv_results = self.search_arxiv();
        report.arxiv_found = arxiv_results.len() as u64;

        for result in &arxiv_results {
            if !self.processed_urls.contains_key(&result.url) {
                self.processed_urls.insert(result.url.clone(), true);
                self.discovered.push(result.clone());
                report.new_high_value += 1;

                if self.pipeline.is_some() {
                    self.enqueue_to_pipeline(result.url.clone(), None);
                }
            }
        }

        // Phase 2: Search GitHub
        let github_results = self.search_github();
        report.github_found = github_results.len() as u64;

        for result in &github_results {
            if !self.processed_urls.contains_key(&result.url) {
                self.processed_urls.insert(result.url.clone(), true);
                self.discovered.push(result.clone());
                report.new_high_value += 1;

                if self.pipeline.is_some() {
                    self.enqueue_to_pipeline(result.url.clone(), None);
                }
            }
        }

        report.total_discovered = self.discovered.len() as u64;
        report
    }

    fn search_arxiv(&self) -> Vec<DiscoveryResult> {
        let mut results = Vec::new();
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .ok();

        let client = match client {
            Some(c) => c,
            None => return results,
        };

        for query in SEARCH_QUERIES.iter() {
            let encoded: String = query.split_whitespace().collect::<Vec<&str>>().join("+");
            let url = format!("http://export.arxiv.org/api/query?search_query=all:{}&start=0&max_results=5&sortBy=submittedDate&sortOrder=descending", encoded);

            let resp = match client.get(&url).send().and_then(|r| r.text()) {
                Ok(text) => text,
                Err(_) => continue,
            };

            for entry in resp.split("<entry>").skip(1) {
                let title = Self::extract_xml_tag(entry, "title").unwrap_or_default();
                let id = Self::extract_xml_tag(entry, "id").unwrap_or_default();
                let summary = Self::extract_xml_tag(entry, "summary").unwrap_or_default();
                let clean_title: String = title.replace('\n', " ").trim().to_string();
                let clean_summary: String = summary
                    .replace('\n', " ")
                    .chars()
                    .take(300)
                    .collect::<String>();

                let score = self.score_relevance(&clean_title, &clean_summary);
                if score > 0.3 {
                    let matched_dims = self.matched_dims(&clean_title, &clean_summary);
                    results.push(DiscoveryResult {
                        source: DiscoverySource::ArXiv,
                        url: id.trim().to_string(),
                        title: clean_title,
                        relevance_score: score,
                        matched_dimensions: matched_dims,
                        summary: clean_summary,
                    });
                }
            }
        }
        results
    }

    fn search_github(&self) -> Vec<DiscoveryResult> {
        let mut results = Vec::new();
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("NeoTrix/1.0")
            .build()
            .ok();

        let client = match client {
            Some(c) => c,
            None => return results,
        };

        let queries = [
            "Rust+AI+agent",
            "LLM+orchestration",
            "self-improving+code",
            "multi-agent+framework",
        ];

        for q in &queries {
            let url = format!(
                "https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page=5",
                q
            );
            let resp = match client.get(&url).send().and_then(|r| r.text()) {
                Ok(text) => text,
                Err(_) => continue,
            };

            let json: serde_json::Value = match serde_json::from_str(&resp) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
                for item in items {
                    let full_name = item.get("full_name").and_then(|n| n.as_str()).unwrap_or("");
                    let description = item
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("");
                    let stars = item
                        .get("stargazers_count")
                        .and_then(|s| s.as_u64())
                        .unwrap_or(0);
                    let html_url = item.get("html_url").and_then(|u| u.as_str()).unwrap_or("");

                    let title = format!("{} ({} stars)", full_name, stars);
                    let score = self.score_relevance(&title, description);

                    if score > 0.3 {
                        results.push(DiscoveryResult {
                            source: DiscoverySource::GitHub,
                            url: html_url.to_string(),
                            title,
                            relevance_score: score,
                            matched_dimensions: self.matched_dims(full_name, description),
                            summary: description.to_string(),
                        });
                    }
                }
            }
        }
        results
    }

    fn score_relevance(&self, title: &str, summary: &str) -> f64 {
        let keywords = [
            ("agent", 0.3),
            ("llm", 0.25),
            ("rust", 0.2),
            ("autonomous", 0.25),
            ("self", 0.2),
            ("evolv", 0.25),
            ("orchestrat", 0.2),
            ("framework", 0.15),
            ("reasoning", 0.2),
            ("multi", 0.15),
            ("code", 0.15),
            ("tool", 0.15),
            ("chain", 0.1),
            ("pipeline", 0.1),
            ("memory", 0.15),
            ("learn", 0.1),
            ("mcp", 0.2),
            ("protocol", 0.1),
            ("language model", 0.2),
        ];

        let combined = format!("{} {}", title, summary).to_lowercase();
        let mut score: f64 = 0.0;

        for (kw, weight) in &keywords {
            if combined.contains(kw) {
                score += weight;
            }
        }

        score.min(1.0)
    }

    fn matched_dims(&self, title: &str, summary: &str) -> Vec<String> {
        let combined = format!("{} {}", title, summary).to_lowercase();
        let mut matched = Vec::new();

        let dim_keywords: [(&str, &[&str]); 7] = [
            (
                "agent_orchestration",
                &["agent", "orchestrat", "multi-agent", "swarm"],
            ),
            (
                "code_understanding",
                &["code", "program", "compil", "static analysis"],
            ),
            (
                "self_evolution",
                &["evolv", "self-improve", "adapt", "learn"],
            ),
            (
                "knowledge_integration",
                &["knowledge", "memory", "retriev", "rag"],
            ),
            ("multi_modal", &["multimodal", "vision", "audio", "image"]),
            (
                "reasoning_depth",
                &["reason", "infer", "think", "chain-of-thought"],
            ),
            ("tool_use", &["tool", "mcp", "function call", "plugin"]),
        ];

        for (dim, kws) in &dim_keywords {
            if kws.iter().any(|kw| combined.contains(kw)) {
                matched.push(dim.to_string());
            }
        }
        matched
    }

    fn enqueue_to_pipeline(&mut self, url: String, _domain: Option<&str>) {
        if let Some(ref mut pipeline) = self.pipeline {
            pipeline.ingest_url(&url);
        }
    }

    fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        xml.find(&open).and_then(|start| {
            let start = start + open.len();
            xml[start..]
                .find(&close)
                .map(|end| xml[start..start + end].to_string())
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryCycleReport {
    pub arxiv_found: u64,
    pub github_found: u64,
    pub new_high_value: u64,
    pub total_discovered: u64,
    pub skipped_reason: Option<String>,
}
