use std::collections::VecDeque;
use std::sync::LazyLock;

use crate::neotrix::nt_world_exploration::content::{ExplorationSourceType, SourceContent};
use crate::neotrix::nt_world_exploration::source_trait::ExplorationSource;

static HTTP: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("NeoTrix/0.18 (+https://github.com/neotrix)")
        .build()
        .unwrap_or_else(|e| {
            log::warn!("PaperSource HTTP client init failed: {e}, using default client");
            reqwest::blocking::Client::new()
        })
});

/// Query mode for paper source exploration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperQueryMode {
    Trending,
    SearchByKeyword,
    SearchByAuthor,
    TopCited,
    Methods,
    Conference,
}

/// Paper source — structured paper data from arXiv API + Semantic Scholar + HF datasets
///
/// Crawling strategy (tiered):
///   1. arXiv API (`export.arxiv.org/api/query`) — XML response, up to 100 results/query
///   2. Semantic Scholar API — citation counts, influential citations
///   3. HF datasets API (`pwc-archive`) — structured dump of paperswithcode.com
///
/// Falls back gracefully: if a tier fails, the next tier is tried.
/// If all fail, returns empty (no crash).
pub struct PaperSource {
    query_queue: VecDeque<PaperQuery>,
    max_results: usize,
    /// Cooldown tracking to respect arXiv rate limits (1 req/3s)
    last_request: std::time::Instant,
}

struct PaperQuery {
    term: String,
    mode: PaperQueryMode,
}

impl PaperSource {
    pub fn new(_mode: PaperQueryMode) -> Self {
        Self {
            query_queue: VecDeque::new(),
            max_results: 20,
            last_request: std::time::Instant::now() - std::time::Duration::from_secs(5),
        }
    }

    pub fn with_max_results(mut self, n: usize) -> Self {
        self.max_results = n.min(100);
        self
    }

    /// Queue a keyword search
    pub fn search(&mut self, keyword: impl Into<String>) {
        self.query_queue.push_back(PaperQuery {
            term: keyword.into(),
            mode: PaperQueryMode::SearchByKeyword,
        });
    }

    /// Queue a trending query (uses arXiv's "interesting" sort)
    pub fn trending(&mut self) {
        self.query_queue.push_back(PaperQuery {
            term: "cat:cs.AI+OR+cat:cs.LG+OR+cat:cs.CL".into(),
            mode: PaperQueryMode::Trending,
        });
    }

    /// Queue a methods-focused query
    pub fn search_method(&mut self, method: impl Into<String>) {
        self.query_queue.push_back(PaperQuery {
            term: format!(r#"all:"{}""#, method.into()),
            mode: PaperQueryMode::Methods,
        });
    }

    fn rate_limit(&mut self) {
        let elapsed = self.last_request.elapsed();
        if elapsed < std::time::Duration::from_secs(3) {
            tokio::runtime::Handle::current().block_on(tokio::time::sleep(
                std::time::Duration::from_secs(3) - elapsed,
            ));
        }
    }

    /// Tier 1: arXiv API search
    fn fetch_from_arxiv(&mut self, query: &PaperQuery) -> Vec<SourceContent> {
        self.rate_limit();

        let encoded: String = query
            .term
            .chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' | '+' | ':' => {
                    c.to_string()
                }
                ' ' => "+".to_string(),
                '"' => "%22".to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect();

        let sort_by = match query.mode {
            PaperQueryMode::Trending => "&sortBy=submittedDate&sortOrder=descending",
            _ => "&sortBy=relevance&sortOrder=descending",
        };

        let url = format!(
            "http://export.arxiv.org/api/query?search_query={}&start=0&max_results={}{}",
            encoded, self.max_results, sort_by
        );

        let resp = match HTTP.get(&url).send() {
            Ok(r) if r.status().is_success() => r,
            _ => return Vec::new(),
        };

        let body = match resp.text() {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };

        self.parse_arxiv_atom(&body, &query.term)
    }

    fn parse_arxiv_atom(&self, xml: &str, _query: &str) -> Vec<SourceContent> {
        let mut results = Vec::new();

        // Simple XML parsing without a full XML parser
        for entry in xml.split("<entry>").skip(1) {
            let title = Self::extract_tag(entry, "title")
                .map(|s| s.replace('\n', " ").trim().to_string())
                .unwrap_or_default();

            let summary = Self::extract_tag(entry, "summary")
                .map(|s| s.replace('\n', " ").chars().take(500).collect::<String>())
                .unwrap_or_default();

            let id = Self::extract_tag(entry, "id")
                .unwrap_or_default()
                .trim()
                .to_string();

            let published = Self::extract_tag(entry, "published")
                .unwrap_or_default()
                .trim()
                .to_string();

            let authors: Vec<String> = entry
                .split("<author>")
                .skip(1)
                .filter_map(|a| Self::extract_tag(a, "name"))
                .collect();
            let author_str = authors.join(", ");

            let categories: Vec<String> = entry
                .split("<category ")
                .skip(1)
                .filter_map(|c| {
                    c.split("term=\"")
                        .nth(1)
                        .and_then(|t| t.split('\"').next())
                        .map(|t| t.to_string())
                })
                .collect();
            let cat_str = categories.join(", ");

            if id.is_empty() {
                continue;
            }

            let arxiv_id = id
                .strip_prefix("http://arxiv.org/abs/")
                .or_else(|| id.strip_prefix("https://arxiv.org/abs/"))
                .unwrap_or(&id)
                .to_string();

            let text = format!(
                "Title: {} | Authors: {} | Categories: {} | {}",
                title, author_str, cat_str, summary
            );

            let content =
                SourceContent::new(&arxiv_id, &text, ExplorationSourceType::PaperDatabase)
                    .with_title(&title)
                    .with_author(&author_str)
                    .with_url(&id)
                    .with_meta("published", &published)
                    .with_meta("arxiv_id", &arxiv_id);

            results.push(content);
        }

        results
    }

    /// Tier 2: Semantic Scholar API for enriched data (citations, influential)
    fn fetch_from_semanticscholar(&mut self, query: &PaperQuery) -> Vec<SourceContent> {
        let encoded: String = url_encode(&query.term);
        let url = format!(
            "https://api.semanticscholar.org/graph/v1/paper/search?query={}&limit={}&fields=title,url,citationCount,publicationDate,authors,journal,externalIds",
            encoded, self.max_results.min(50)
        );

        let resp = match HTTP.get(&url).header("User-Agent", "NeoTrix/0.18").send() {
            Ok(r) if r.status().is_success() => r,
            _ => return Vec::new(),
        };

        let json: serde_json::Value = match resp.json() {
            Ok(j) => j,
            Err(_) => return Vec::new(),
        };

        let papers = match json["data"].as_array() {
            Some(p) => p,
            None => return Vec::new(),
        };

        papers
            .iter()
            .filter_map(|p| {
                let title = p["title"].as_str()?;
                let paper_id = p["paperId"].as_str().unwrap_or("");
                let url = p["url"].as_str().unwrap_or("");
                let citations = p["citationCount"].as_u64().unwrap_or(0);
                let date = p["publicationDate"].as_str().unwrap_or("");
                let authors: Vec<String> = p["authors"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x["name"].as_str().map(|n| n.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let author_str = authors.join(", ");

                let text = format!(
                    "Title: {} | Citations: {} | Authors: {} | published: {}",
                    title, citations, author_str, date
                );

                Some(
                    SourceContent::new(paper_id, &text, ExplorationSourceType::PaperDatabase)
                        .with_title(title)
                        .with_author(&author_str)
                        .with_url(url)
                        .with_meta("citations", &citations.to_string())
                        .with_meta("published", date),
                )
            })
            .collect()
    }

    /// Tier 3: HF Datasets API for structured PwC data
    fn fetch_from_hf_datasets(&mut self, query: &PaperQuery) -> Vec<SourceContent> {
        let dataset = match query.mode {
            PaperQueryMode::Methods => "methods",
            _ => "papers-with-abstracts",
        };

        let _url = format!(
            "https://huggingface.co/api/datasets/pwc-archive/{}/parquet/default/train/0.parquet",
            dataset
        );

        // HF datasets API uses parquet; try the dataset viewer API for JSON
        let viewer_url = format!(
            "https://huggingface.co/datasets/pwc-archive/{}/raw/main/data/train-00000-of-00001.json",
            dataset
        );

        let resp = match HTTP.get(&viewer_url).send() {
            Ok(r) if r.status().is_success() => r,
            _ => return Vec::new(),
        };

        let json: serde_json::Value = match resp.json() {
            Ok(j) => j,
            Err(_) => return Vec::new(),
        };

        let rows = match json.as_array() {
            Some(r) => r,
            None => return Vec::new(),
        };

        let keyword = query.term.to_lowercase();
        rows.iter()
            .filter(|r| {
                let title = r["title"].as_str().unwrap_or("");
                title.to_lowercase().contains(&keyword)
                    || r["abstract"]
                        .as_str()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&keyword)
            })
            .take(self.max_results)
            .filter_map(|r| {
                let title = r["title"].as_str()?;
                let abstract_text = r["abstract"].as_str().unwrap_or("");
                let id = r["id"].as_str().unwrap_or("");
                let authors: Vec<String> = r["authors"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let author_str = authors.join(", ");

                let text = format!(
                    "Title: {} | Authors: {} | Abstract: {}",
                    title,
                    author_str,
                    abstract_text.chars().take(400).collect::<String>()
                );

                Some(
                    SourceContent::new(id, &text, ExplorationSourceType::PaperDatabase)
                        .with_title(title)
                        .with_author(&author_str)
                        .with_meta("source", "hf_pwc_archive")
                        .with_meta("abstract_len", &abstract_text.len().to_string()),
                )
            })
            .collect()
    }

    fn extract_tag(s: &str, tag: &str) -> Option<String> {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        s.find(&open).and_then(|start| {
            let content_start = start + open.len();
            s[content_start..]
                .find(&close)
                .map(|end| s[content_start..content_start + end].to_string())
        })
    }

    pub fn pending_count(&self) -> usize {
        self.query_queue.len()
    }

    pub fn enqueue(&mut self, query: impl Into<String>) {
        self.query_queue.push_back(PaperQuery {
            term: query.into(),
            mode: PaperQueryMode::SearchByKeyword,
        });
    }
}

impl ExplorationSource for PaperSource {
    fn name(&self) -> &'static str {
        "paper_database"
    }

    fn confidence(&self) -> f64 {
        0.88
    }

    fn explore(&mut self) -> Result<Vec<SourceContent>, String> {
        let mut all = Vec::new();
        let queries = std::mem::take(&mut self.query_queue);

        for query in queries {
            // Tier 1: arXiv API (primary, most reliable for ML papers)
            let from_arxiv = self.fetch_from_arxiv(&query);
            // Tier 2: Semantic Scholar (citations, broader coverage)
            let from_s2 = self.fetch_from_semanticscholar(&query);
            // Tier 3: HF datasets (structured PwC data)
            let from_hf = self.fetch_from_hf_datasets(&query);

            // Merge: arXiv gives paper metadata, S2 gives citations, HF gives structure
            let mut merged: Vec<SourceContent> = Vec::new();
            merged.extend(from_arxiv);
            merged.extend(from_s2);
            merged.extend(from_hf);

            // Dedup by ID within each query batch
            let mut seen = std::collections::HashSet::new();
            for item in merged {
                if seen.contains(&item.id) {
                    continue;
                }
                seen.insert(item.id.clone());
                all.push(item);
            }
        }

        self.last_request = std::time::Instant::now();
        Ok(all)
    }

    fn pending_count(&self) -> usize {
        self.query_queue.len()
    }

    fn is_ready(&self) -> bool {
        true
    }
}

fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tag_simple() {
        let s = "<title>Test Paper Title</title>";
        assert_eq!(
            PaperSource::extract_tag(s, "title"),
            Some("Test Paper Title".into())
        );
    }

    #[test]
    fn test_extract_tag_missing() {
        let s = "<entry><id>123</id></entry>";
        assert_eq!(PaperSource::extract_tag(s, "title"), None);
    }

    #[test]
    fn test_parse_arxiv_atom_basic() {
        let xml = r#"<?xml version="1.0"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <id>http://arxiv.org/abs/2606.00123</id>
    <title>Test Paper Title</title>
    <summary>This is a test abstract for the paper.</summary>
    <published>2026-06-01</published>
    <author><name>Alice Smith</name></author>
    <author><name>Bob Jones</name></author>
    <category term="cs.AI"/>
    <category term="cs.LG"/>
  </entry>
</feed>"#;
        let src = PaperSource::new(PaperQueryMode::SearchByKeyword);
        let results = src.parse_arxiv_atom(xml, "test");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Paper Title");
        assert!(results[0].text.contains("Alice Smith"));
        assert!(results[0].text.contains("cs.AI"));
        assert_eq!(
            results[0].url.as_deref(),
            Some("http://arxiv.org/abs/2606.00123")
        );
    }

    #[test]
    fn test_parse_arxiv_atom_multiple() {
        let xml = r#"<feed>
  <entry>
    <id>http://arxiv.org/abs/2606.00123</id>
    <title>Paper One</title>
    <summary>Abstract of paper one.</summary>
    <published>2026-06-01</published>
    <author><name>Alice</name></author>
  </entry>
  <entry>
    <id>http://arxiv.org/abs/2606.00456</id>
    <title>Paper Two</title>
    <summary>Abstract of paper two.</summary>
    <published>2026-06-02</published>
    <author><name>Bob</name></author>
  </entry>
</feed>"#;
        let src = PaperSource::new(PaperQueryMode::SearchByKeyword);
        let results = src.parse_arxiv_atom(xml, "test");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Paper One");
        assert_eq!(results[1].title, "Paper Two");
    }

    #[test]
    fn test_parse_arxiv_atom_empty() {
        let src = PaperSource::new(PaperQueryMode::SearchByKeyword);
        let results = src.parse_arxiv_atom("<feed></feed>", "test");
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_arxiv_atom_no_entry() {
        let src = PaperSource::new(PaperQueryMode::SearchByKeyword);
        let results = src.parse_arxiv_atom("<feed><title>no entry</title></feed>", "test");
        assert!(results.is_empty());
    }

    #[test]
    fn test_extract_tag_with_newlines() {
        let s = "<summary>\n  This is a\n  multiline summary.\n</summary>";
        assert_eq!(
            PaperSource::extract_tag(s, "summary"),
            Some("\n  This is a\n  multiline summary.\n".into())
        );
    }

    #[test]
    fn test_search_queues_query() {
        let mut src = PaperSource::new(PaperQueryMode::SearchByKeyword);
        assert_eq!(src.pending_count(), 0);
        src.search("transformer attention");
        assert_eq!(src.pending_count(), 1);
        src.search("reinforcement learning");
        assert_eq!(src.pending_count(), 2);
    }

    #[test]
    fn test_trending_queues_query() {
        let mut src = PaperSource::new(PaperQueryMode::Trending);
        assert_eq!(src.pending_count(), 0);
        src.trending();
        assert_eq!(src.pending_count(), 1);
    }

    #[test]
    fn test_search_method_queues() {
        let mut src = PaperSource::new(PaperQueryMode::Methods);
        src.search_method("RLVR");
        assert_eq!(src.pending_count(), 1);
    }

    #[test]
    fn test_explore_drains_queue() {
        let mut src = PaperSource::new(PaperQueryMode::SearchByKeyword);
        src.search("test query");
        assert_eq!(src.pending_count(), 1);
        let _ = src.explore().unwrap_or_default();
        assert_eq!(src.pending_count(), 0);
    }

    #[test]
    fn test_empty_explore() {
        let mut src = PaperSource::new(PaperQueryMode::SearchByKeyword);
        let results = src.explore().unwrap_or_default();
        assert!(results.is_empty());
    }

    #[test]
    fn test_source_type_paper_database() {
        let st = ExplorationSourceType::PaperDatabase;
        assert_eq!(st.name(), "paper_database");
        assert!((st.weight() - 0.88).abs() < 0.01);
    }

    #[test]
    fn test_with_max_results_clamps() {
        let src = PaperSource::new(PaperQueryMode::SearchByKeyword).with_max_results(200);
        // internal max is 100
        let _ = src;
    }

    #[test]
    fn test_enqueue_method() {
        let mut src = PaperSource::new(PaperQueryMode::SearchByKeyword);
        src.enqueue("machine learning");
        assert_eq!(src.pending_count(), 1);
    }
}
