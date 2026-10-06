//! Knowledge source adapter — unified interface for different knowledge origins.
//!
//! Extracted from batch-absorb.sh / absorb_to_capability.py essential patterns.
//! Each source type implements `SourceAdapter` to produce `KnowledgeInput`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceKind {
    GitHubRepo,
    GitHubTopic,
    ArxivPaper,
    Article,
    WebPage,
    KnowledgeNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeInput {
    pub source_kind: SourceKind,
    pub title: String,
    pub summary: String,
    pub content: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub language: String,
    pub confidence: f64,
    pub importance: f64,
    pub metadata: HashMap<String, serde_json::Value>,
    pub source_specific: SourceSpecific,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceSpecific {
    GitHubRepo {
        full_name: String,
        stars: u64,
        language: Option<String>,
        topics: Vec<String>,
    },
    GitHubTopic {
        topic_name: String,
        total_count: u64,
        top_repos: Vec<String>,
    },
    ArxivPaper {
        paper_id: String,
        authors: Vec<String>,
        categories: Vec<String>,
    },
    Generic,
}

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("API error: {0}")]
    ApiError(String),
}

pub trait SourceAdapter: Send + Sync {
    fn source_kind(&self) -> SourceKind;
    fn parse(&self, raw: &str) -> Result<KnowledgeInput, SourceError>;
    fn validate(&self, input: &KnowledgeInput) -> bool;
    fn default_domain(&self) -> &str;
    fn default_importance(&self) -> f64 {
        0.5
    }
    fn default_confidence(&self) -> f64 {
        1.0
    }
}

pub struct GitHubRepoAdapter;

impl SourceAdapter for GitHubRepoAdapter {
    fn source_kind(&self) -> SourceKind {
        SourceKind::GitHubRepo
    }
    fn default_domain(&self) -> &str {
        "github.com"
    }

    fn parse(&self, raw: &str) -> Result<KnowledgeInput, SourceError> {
        let input = raw.trim();
        let (owner, repo) = parse_github_url(input)?;
        let full_name = format!("{}/{}", owner, repo);
        let url = format!("https://github.com/{}/{}", owner, repo);

        let mut meta = HashMap::new();
        meta.insert("owner".into(), serde_json::Value::String(owner.clone()));
        meta.insert("repo".into(), serde_json::Value::String(repo.clone()));

        Ok(KnowledgeInput {
            source_kind: SourceKind::GitHubRepo,
            title: full_name.clone(),
            summary: format!("GitHub repository: {}", full_name),
            content: None,
            url: Some(url),
            domain: Some("github.com".into()),
            language: "en".into(),
            confidence: 1.0,
            importance: 0.5,
            metadata: meta,
            source_specific: SourceSpecific::GitHubRepo {
                full_name,
                stars: 0,
                language: None,
                topics: vec![],
            },
        })
    }

    fn validate(&self, input: &KnowledgeInput) -> bool {
        input.url.is_some() && input.source_kind == SourceKind::GitHubRepo
    }
}

pub struct ArxivAdapter;

impl SourceAdapter for ArxivAdapter {
    fn source_kind(&self) -> SourceKind {
        SourceKind::ArxivPaper
    }
    fn default_domain(&self) -> &str {
        "arxiv.org"
    }
    fn default_importance(&self) -> f64 {
        0.6
    }

    fn parse(&self, raw: &str) -> Result<KnowledgeInput, SourceError> {
        let input = raw.trim();
        let paper_id = parse_arxiv_id(input)?;
        let url = format!("https://arxiv.org/abs/{}", paper_id);

        let mut meta = HashMap::new();
        meta.insert(
            "paper_id".into(),
            serde_json::Value::String(paper_id.clone()),
        );

        Ok(KnowledgeInput {
            source_kind: SourceKind::ArxivPaper,
            title: format!("ArXiv {}", paper_id),
            summary: format!("Paper from arxiv: {}", paper_id),
            content: None,
            url: Some(url),
            domain: Some("arxiv.org".into()),
            language: "en".into(),
            confidence: 1.0,
            importance: 0.6,
            metadata: meta,
            source_specific: SourceSpecific::ArxivPaper {
                paper_id,
                authors: vec![],
                categories: vec![],
            },
        })
    }

    fn validate(&self, input: &KnowledgeInput) -> bool {
        input.source_kind == SourceKind::ArxivPaper
    }
}

pub struct GitHubTopicAdapter;

impl SourceAdapter for GitHubTopicAdapter {
    fn source_kind(&self) -> SourceKind {
        SourceKind::GitHubTopic
    }
    fn default_domain(&self) -> &str {
        "github.com/topic"
    }

    fn parse(&self, raw: &str) -> Result<KnowledgeInput, SourceError> {
        let topic = raw.trim().to_string();
        if topic.is_empty() {
            return Err(SourceError::MissingField("topic".into()));
        }
        let url = format!("https://github.com/topics/{}", topic);

        let mut meta = HashMap::new();
        meta.insert("topic".into(), serde_json::Value::String(topic.clone()));

        Ok(KnowledgeInput {
            source_kind: SourceKind::GitHubTopic,
            title: topic.clone(),
            summary: format!("GitHub topic: {}", topic),
            content: None,
            url: Some(url),
            domain: Some("github.com/topic".into()),
            language: "en".into(),
            confidence: 1.0,
            importance: 0.5,
            metadata: meta,
            source_specific: SourceSpecific::GitHubTopic {
                topic_name: topic,
                total_count: 0,
                top_repos: vec![],
            },
        })
    }

    fn validate(&self, input: &KnowledgeInput) -> bool {
        input.source_kind == SourceKind::GitHubTopic
    }
}

pub struct GenericArticleAdapter;

impl SourceAdapter for GenericArticleAdapter {
    fn source_kind(&self) -> SourceKind {
        SourceKind::Article
    }
    fn default_domain(&self) -> &str {
        "unknown"
    }

    fn parse(&self, raw: &str) -> Result<KnowledgeInput, SourceError> {
        let title = raw.trim().to_string();
        if title.is_empty() {
            return Err(SourceError::MissingField("title".into()));
        }
        Ok(KnowledgeInput {
            source_kind: SourceKind::Article,
            title: title.clone(),
            summary: title,
            content: None,
            url: None,
            domain: None,
            language: "en".into(),
            confidence: 0.5,
            importance: 0.5,
            metadata: HashMap::new(),
            source_specific: SourceSpecific::Generic,
        })
    }

    fn validate(&self, input: &KnowledgeInput) -> bool {
        !input.title.is_empty()
    }
}

pub struct SourceAdapterFactory;

impl SourceAdapterFactory {
    /// 判定来源适配器 —— **决定用哪个适配器解析**。
    ///
    /// # 2026-10-03 接入编译树时修三处
    ///
    /// ## ① `GitHubTopicAdapter` 分支**永不可达**（分支顺序 bug）
    ///
    /// ⛔ 原顺序把 `starts_with("https://github.com/")` 放在**第一个**，
    ///    而 topics 分支条件是 `starts_with("https://github.com/topics/")`
    ///    —— 后者必然被前者**先**匹配：
    ///
    /// ```text
    /// "https://github.com/topics/rust"
    ///   .starts_with("https://github.com/")          == true  ← 先命中并 return
    ///   .starts_with("https://github.com/topics/")  == true  ← 永不评估
    /// ```
    ///
    /// ⇒ `GitHubTopicAdapter` 自其存在起**从未被选中过**。
    ///   这是「导出 ≠ 接入」在**分支顺序**上的形态：类型实现了、
    ///   其它测试也过，但运行时永远到不了。
    ///
    /// ## ② 裸 arXiv ID（`2301.12345`）无法路由
    ///
    /// ⛔ 原 `detect` 只认 `https://arxiv.org/` 与 `arxiv:` 两种形态，
    ///    而既有的 [`parse_arxiv_id`] **本就支持裸 ID**（全为合法字符）。
    ///    ⇒ 解析器有能力、路由没接线，导致既有测试
    ///      `test_factory_detection`（期望 `ArxivPaper`）**长期失败**。
    ///
    /// ## ③ 域名判定：`starts_with` → host 判定
    ///
    /// ⛔ `starts_with("https://github.com/")` 会被
    ///    `https://github.com.evil.net/owner/repo` 满足 ⇒
    ///    伪装域名会被判成 GitHubRepo 并用 GitHub 适配器处理。
    pub fn detect(raw: &str) -> Box<dyn SourceAdapter> {
        use crate::l0_substrate::nt_core_platform::url_match::url_matches_domain as m;

        let trimmed = raw.trim();
        let lower = trimmed.to_lowercase();
        let is_github = m(trimmed, "github.com");

        // —— ① topics/ 必须**先于** github.com/ 判定 ——
        if is_github && lower.contains("/topics/") {
            return Box::new(GitHubTopicAdapter);
        }

        // —— ② arXiv：三种形态（URL / arxiv: 前缀 / 裸 ID）——
        if m(trimmed, "arxiv.org")
            || lower.starts_with("arxiv:")
            || is_bare_arxiv_id(trimmed)
        {
            return Box::new(ArxivAdapter);
        }

        // —— ③ github 域名，或「单斜杠」短式（owner/repo）——
        if is_github || (!trimmed.contains("://") && lower.matches('/').count() == 1) {
            return Box::new(GitHubRepoAdapter);
        }

        Box::new(GenericArticleAdapter)
    }
}

fn parse_github_url(input: &str) -> Result<(String, String), SourceError> {
    let path = input
        .strip_prefix("https://github.com/")
        .unwrap_or(input)
        .trim_end_matches('/');
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err(SourceError::InvalidUrl(input.to_string()))
    }
}

/// 判定是否为裸 arXiv ID（含版本号的 `NNNN.NNNNN` 形态）。
fn is_bare_arxiv_id(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    if !input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return false;
    }
    // ⛔ 排除域名：`owner` / `2301.12345` 允许；`github.com` / `a.b.c` 拒绝
    //    判据：含 '.' 时，末段必须是纯数字（arXiv 版本号形态）
    match input.rsplit_once('.') {
        Some((_, last)) => !last.is_empty() && last.chars().all(|c| c.is_ascii_digit()),
        None => true,
    }
}

fn parse_arxiv_id(input: &str) -> Result<String, SourceError> {
    if let Some(id) = input.strip_prefix("https://arxiv.org/abs/") {
        Ok(id.trim_end_matches('/').to_string())
    } else if let Some(id) = input.strip_prefix("arxiv:") {
        Ok(id.to_string())
    } else if input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        && !input.is_empty()
    {
        Ok(input.to_string())
    } else {
        Err(SourceError::InvalidUrl(input.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_parse() {
        let adapter = GitHubRepoAdapter;
        let input = adapter.parse("https://github.com/ollama/ollama").unwrap();
        assert_eq!(input.source_kind, SourceKind::GitHubRepo);
        assert_eq!(input.title, "ollama/ollama");
    }

    #[test]
    fn test_arxiv_parse() {
        let adapter = ArxivAdapter;
        let input = adapter.parse("2605.24517").unwrap();
        assert_eq!(input.source_kind, SourceKind::ArxivPaper);
        assert!(input.url.unwrap().contains("2605.24517"));
    }

    #[test]
    fn test_factory_detection() {
        let adapter = SourceAdapterFactory::detect("https://github.com/foo/bar");
        assert_eq!(adapter.source_kind(), SourceKind::GitHubRepo);

        let adapter = SourceAdapterFactory::detect("2301.12345");
        assert_eq!(adapter.source_kind(), SourceKind::ArxivPaper);
    }
}

#[cfg(test)]
mod wiring_tests {
    use super::*;

    /// 回归①：`GitHubTopicAdapter` 曾**永不可达** ——
    /// 原分支顺序把 `starts_with("github.com/")` 放在前，topics 分支永不评估。
    #[test]
    fn github_topics_adapter_is_reachable_now() {
        for u in [
            "https://github.com/topics/rust",
            "https://www.github.com/topics/ml",
        ] {
            assert_eq!(
                SourceAdapterFactory::detect(u).source_kind(),
                SourceKind::GitHubTopic,
                "{} must select GitHubTopics",
                u
            );
        }
    }

    /// 回归②：裸 arXiv ID 曾无法路由，导致既有测试长期失败。
    /// 解析器本就支持（`parse_arxiv_id` 第三分支），只是路由没接线。
    #[test]
    fn bare_arxiv_id_is_routed() {
        // ⛔ **不含** `cs/0501001`：arXiv 旧式 ID 含 `/`，与 `owner/repo`
        //    短式形态**在原理上不可区分**，且 `parse_arxiv_id` 也不接受它
        //    （`/` 非其合法字符集）。原实现同样把它判成 GitHubRepo ——
        //    我曾把它写成 arXiv，那是臆想的语义，已纠正。
        for id in ["2301.12345", "1234.5678", "2301.00001"] {
            let kind = SourceAdapterFactory::detect(id).source_kind();
            assert_eq!(kind, SourceKind::ArxivPaper, "{} must route to ArxivPaper", id);
        }
        // 显式记录这个**已知歧义**，避免后来者误以为漏了
        assert_eq!(
            SourceAdapterFactory::detect("cs/0501001").source_kind(),
            SourceKind::GitHubRepo,
            "cs/0501001 is indistinguishable from owner/repo — documented ambiguity"
        );
    }

    /// 路由与解析必须用**同一**判据 ——
    /// 若 detect 路由过去了但 parse 失败，就是新的裂缝。
    #[test]
    fn every_routed_arxiv_form_actually_parses() {
        for raw in [
            "https://arxiv.org/abs/2301.12345",
            "arxiv:2301.12345",
            "2301.12345",
        ] {
            let adapter = SourceAdapterFactory::detect(raw);
            assert_eq!(adapter.source_kind(), SourceKind::ArxivPaper);
            assert!(
                adapter.parse(raw).is_ok(),
                "detect routed `{}` to Arxiv but parse failed — 路由/解析判据不一致",
                raw
            );
        }
    }

    /// 伪装域名不再走 GitHub 适配器（旧 starts_with 前缀匹配会误判）。
    #[test]
    fn lookalike_domain_does_not_reach_github_adapter() {
        assert_ne!(
            SourceAdapterFactory::detect("https://github.com.evil.net/o/r")
                .source_kind(),
            SourceKind::GitHubRepo
        );
    }

    /// 分支顺序语义：github.com/…/topics/ 必须判 Topics 而非 Repo。
    #[test]
    fn topics_branch_precedes_plain_github() {
        assert_eq!(
            SourceAdapterFactory::detect("https://github.com/topics/rust").source_kind(),
            SourceKind::GitHubTopic
        );
        assert_eq!(
            SourceAdapterFactory::detect("https://github.com/owner/repo").source_kind(),
            SourceKind::GitHubRepo
        );
    }

    /// 域名形态不能被裸 ID 判据吞掉。
    /// `github.com` 全是合法字符 —— 若 `is_bare_arxiv_id` 不排除域名，
    /// 它会因 arxiv 分支在前而被误判成 ArxivPaper。
    #[test]
    fn domain_shaped_inputs_are_not_treated_as_bare_arxiv_ids() {
        for u in ["github.com", "example.com", "a.b.c", "x.io"] {
            assert!(
                !is_bare_arxiv_id(u),
                "`{}` must not be classified as a bare arXiv id",
                u
            );
            assert_ne!(
                SourceAdapterFactory::detect(u).source_kind(),
                SourceKind::ArxivPaper,
                "`{}` must not route to ArxivPaper",
                u
            );
        }
    }

    /// 保留原有的「单斜杠短式」输入（`owner/repo`）
    #[test]
    fn short_slash_repo_input_still_works() {
        assert_eq!(
            SourceAdapterFactory::detect("owner/repo").source_kind(),
            SourceKind::GitHubRepo
        );
    }

    /// www 子域：旧 starts_with 不含 `www.`，故此前**落到 Generic**
    /// （GitHub 内容被当通用文章处理）。host 判定天然覆盖子域。
    #[test]
    fn www_github_subdomain_now_recognized() {
        assert_eq!(
            SourceAdapterFactory::detect("https://www.github.com/owner/repo").source_kind(),
            SourceKind::GitHubRepo
        );
    }

    #[test]
    fn unrelated_input_is_generic() {
        assert_eq!(
            SourceAdapterFactory::detect("https://example.org/a/b").source_kind(),
            SourceKind::Article
        );
        assert_eq!(
            SourceAdapterFactory::detect("just some prose").source_kind(),
            SourceKind::Article
        );
    }
}
