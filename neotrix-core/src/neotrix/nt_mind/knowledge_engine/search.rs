#![allow(dead_code)]
use super::graph::KnowledgeEngine;
use super::types::{
    extract_authors, extract_categories, extract_xml, strip_html, urlencoding, KnowledgeEntry,
    KnowledgeSourceType,
};
use crate::neotrix::nt_core_signal::ops::cosine_similarity;
use crate::neotrix::nt_mind::embedding::TextEmbedder;
use std::collections::HashSet;

pub struct LiteratureSearcher {
    client: reqwest::blocking::Client,
    embedder: TextEmbedder,
}

impl Default for LiteratureSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl LiteratureSearcher {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("NeoTrix/1.0 (knowledge-engine; research)")
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_default();
        Self {
            client,
            embedder: TextEmbedder::new(),
        }
    }

    pub fn search_arxiv(&mut self, query: &str, max_results: usize) -> Vec<KnowledgeEntry> {
        let url = format!(
            "http://export.arxiv.org/api/query?search_query=all:{}&max_results={}&sortBy=relevance",
            urlencoding(query),
            max_results.min(50)
        );
        match self.client.get(&url).send() {
            Ok(resp) => {
                let text = resp.text().unwrap_or_default();
                self.parse_arxiv_response(&text, query)
            }
            Err(e) => {
                log::error!("[LitSearch] arXiv error: {}", e);
                Vec::new()
            }
        }
    }

    fn parse_arxiv_response(&self, xml: &str, _query: &str) -> Vec<KnowledgeEntry> {
        let mut entries = Vec::new();
        let mut pos = 0;
        while let Some(start) = xml[pos..].find("<entry>") {
            let abs_start = pos + start;
            if let Some(end) = xml[abs_start..].find("</entry>") {
                let entry_xml = &xml[abs_start..abs_start + end + 8];

                let title = extract_xml(entry_xml, "title").unwrap_or_default();
                let summary = extract_xml(entry_xml, "summary").unwrap_or_default();
                let arxiv_id = extract_xml(entry_xml, "id").unwrap_or_default();
                let authors = extract_authors(entry_xml);

                let _published = extract_xml(entry_xml, "published").unwrap_or_default();
                let categories = extract_categories(entry_xml);

                if !title.is_empty() {
                    let entry = KnowledgeEntry::new(
                        title.trim(),
                        summary.trim(),
                        KnowledgeSourceType::ArXiv,
                        &arxiv_id,
                    )
                    .with_tags(categories)
                    .with_importance(0.7);
                    let _ = authors;
                    entries.push(entry);
                }
                pos = abs_start + end + 8;
            } else {
                break;
            }
        }
        entries
    }

    pub fn search_semantic_scholar(&mut self, query: &str, limit: usize) -> Vec<KnowledgeEntry> {
        let url = format!(
            "https://api.semanticscholar.org/graph/v1/paper/search?query={}&limit={}&fields=title,abstract,authors,year,externalIds",
            urlencoding(query), limit.min(100)
        );
        match self.client.get(&url).send() {
            Ok(resp) => {
                let text = resp.text().unwrap_or_default();
                self.parse_s2_response(&text)
            }
            Err(e) => {
                log::error!("[LitSearch] Semantic Scholar error: {}", e);
                Vec::new()
            }
        }
    }

    fn parse_s2_response(&self, json: &str) -> Vec<KnowledgeEntry> {
        let mut entries = Vec::new();
        let v: serde_json::Value = serde_json::from_str(json).unwrap_or(serde_json::Value::Null);
        if let Some(papers) = v["data"].as_array() {
            for paper in papers {
                let title = paper["title"].as_str().unwrap_or("").to_string();
                let abstract_ = paper["abstract"].as_str().unwrap_or("").to_string();
                let paper_id = paper["paperId"].as_str().unwrap_or("").to_string();
                if !title.is_empty() {
                    let entry = KnowledgeEntry::new(
                        &title,
                        &abstract_,
                        KnowledgeSourceType::SemanticScholar,
                        &paper_id,
                    )
                    .with_importance(0.75);
                    entries.push(entry);
                }
            }
        }
        entries
    }

    pub fn search_wikipedia(&mut self, query: &str, limit: usize) -> Vec<KnowledgeEntry> {
        let search_url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json&srlimit={}",
            urlencoding(query), limit.min(50)
        );
        match self.client.get(&search_url).send() {
            Ok(resp) => {
                let text = resp.text().unwrap_or_default();
                self.parse_wiki_search(&text)
            }
            Err(e) => {
                log::error!("[LitSearch] Wikipedia error: {}", e);
                Vec::new()
            }
        }
    }

    fn parse_wiki_search(&self, json: &str) -> Vec<KnowledgeEntry> {
        let mut entries = Vec::new();
        let v: serde_json::Value = serde_json::from_str(json).unwrap_or(serde_json::Value::Null);
        if let Some(results) = v["query"]["search"].as_array() {
            for result in results {
                let title = result["title"].as_str().unwrap_or("").to_string();
                let snippet = result["snippet"].as_str().unwrap_or("").to_string();
                let _page_id = result["pageid"].as_i64().unwrap_or(0);
                let url = format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"));
                if !title.is_empty() {
                    let entry = KnowledgeEntry::new(
                        &title,
                        &strip_html(&snippet),
                        KnowledgeSourceType::Wikipedia,
                        &url,
                    )
                    .with_importance(0.6);
                    entries.push(entry);
                }
            }
        }
        entries
    }

    pub fn search_all(&mut self, query: &str, max_per_source: usize) -> Vec<KnowledgeEntry> {
        let mut seen = HashSet::new();
        let mut results = Vec::new();

        for entry in self.search_arxiv(query, max_per_source) {
            let key = entry.title.clone();
            if seen.insert(key) {
                results.push(entry);
            }
        }
        for entry in self.search_semantic_scholar(query, max_per_source) {
            let key = entry.title.clone();
            if seen.insert(key) {
                results.push(entry);
            }
        }
        for entry in self.search_wikipedia(query, max_per_source) {
            let key = entry.title.clone();
            if seen.insert(key) {
                results.push(entry);
            }
        }

        results.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}

impl KnowledgeEngine {
    pub fn search(&mut self, query: &str, limit: usize) -> Vec<(&KnowledgeEntry, f64)> {
        let qv = self.embedder.embed(query);
        let q_lower = query.to_lowercase();
        let q_chars: Vec<char> = q_lower.chars().collect();

        let mut scored: Vec<(&KnowledgeEntry, f64)> = self
            .entries
            .values()
            .map(|e| {
                let mut score = 0.0f64;

                if let Some(ev) = &e.embedding {
                    score += cosine_similarity(&qv, ev) * 0.4;
                }

                let title_lower = e.title.to_lowercase();
                if title_lower.contains(&q_lower) || q_lower.contains(&title_lower) {
                    score += 0.4;
                } else if title_lower.chars().any(|c| q_chars.contains(&c)) {
                    score += 0.15;
                }

                let body_lower = e.body.to_lowercase();
                let match_count = q_lower
                    .split_whitespace()
                    .filter(|w| w.len() > 2 && body_lower.contains(w))
                    .count();
                score += (match_count as f64).min(5.0) * 0.06;

                let tag_bonus = e
                    .tags
                    .iter()
                    .filter(|t| q_lower.contains(&t.to_lowercase()))
                    .count() as f64
                    * 0.08;
                score += tag_bonus;

                score += e.importance * 0.15;

                (e, score)
            })
            .filter(|(_, s)| *s > 0.01)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored
    }

    pub fn search_by_tag(&self, tag: &str, limit: usize) -> Vec<&KnowledgeEntry> {
        self.tag_index
            .get(tag)
            .map(|ids| {
                let mut entries: Vec<&KnowledgeEntry> =
                    ids.iter().filter_map(|id| self.entries.get(id)).collect();
                entries.sort_by(|a, b| {
                    b.importance
                        .partial_cmp(&a.importance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                entries.truncate(limit);
                entries
            })
            .unwrap_or_default()
    }

    pub fn search_by_source(
        &self,
        source: &KnowledgeSourceType,
        limit: usize,
    ) -> Vec<&KnowledgeEntry> {
        let mut entries: Vec<&KnowledgeEntry> = self.entries.values()
            .filter(|e| matches!(&e.source, s if std::mem::discriminant(s) == std::mem::discriminant(source)))
            .collect();
        entries.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.truncate(limit);
        entries
    }

    pub fn search_by_dimension(&self, dimension: &str, limit: usize) -> Vec<&KnowledgeEntry> {
        let mut entries: Vec<&KnowledgeEntry> = self
            .entries
            .values()
            .filter(|e| e.dimensions.iter().any(|d| d.contains(dimension)))
            .collect();
        entries.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.truncate(limit);
        entries
    }

    pub fn get_related(&self, entry_id: &str, limit: usize) -> Vec<&KnowledgeEntry> {
        let mut related = Vec::new();
        let mut seen = HashSet::new();
        seen.insert(entry_id.to_string());

        for rel in &self.relations {
            let target = if rel.from_id == entry_id {
                Some(&rel.to_id)
            } else if rel.to_id == entry_id {
                Some(&rel.from_id)
            } else {
                None
            };
            if let Some(tid) = target {
                if seen.insert(tid.clone()) {
                    if let Some(entry) = self.entries.get(tid) {
                        related.push(entry);
                    }
                }
            }
        }
        related.truncate(limit);
        related
    }

    pub fn literature_search_and_ingest(&mut self, query: &str, max_results: usize) -> Vec<String> {
        let mut ids = Vec::new();
        if let Some(ref mut searcher) = self.literature_searcher {
            let papers = searcher.search_all(query, max_results);
            for paper in papers {
                let id = self.add_entry(paper);
                ids.push(id);
            }
        }
        ids
    }
}
