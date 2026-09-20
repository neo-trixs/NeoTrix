//! Format normalizer — standardize titles, URLs, and content from various sources.
//!
//! Extracted from absorb_to_capability.py normalize_repo_title() and
//! batch-absorb.sh store_repo()/store_concept() patterns.

use regex::Regex;
use std::sync::LazyLock;

use super::source_adapter::KnowledgeInput;

static RE_GITHUB_TITLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^GitHub\s*-\s*").unwrap());

static RE_COLON_SUFFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r":\s*$").unwrap());

static RE_MULTI_SLASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/{2,}").unwrap());

pub struct FormatNormalizer;

impl FormatNormalizer {
    pub fn normalize_title(title: &str) -> String {
        let t = RE_GITHUB_TITLE.replace(title, "").to_string();
        let t = RE_COLON_SUFFIX.replace(&t, "").to_string();
        let t = t.trim().to_string();
        if t.is_empty() {
            title.to_string()
        } else {
            t
        }
    }

    pub fn normalize_url(url: &str) -> String {
        let t = url.trim().trim_end_matches('/').to_string();
        RE_MULTI_SLASH.replace_all(&t, "/").to_string()
    }

    pub fn normalize_owner_repo(title: &str) -> String {
        let t = Self::normalize_title(title);
        if let Some(pos) = t.rfind('/') {
            let owner = t[..pos].trim();
            let repo = t[pos + 1..].trim();
            if !owner.is_empty() && !repo.is_empty() {
                return format!("{}/{}", owner, repo);
            }
        }
        t
    }

    pub fn extract_repo_from_url(url: &str) -> Option<(String, String)> {
        let lower = url.to_lowercase();
        let path = lower.strip_prefix("https://github.com/")?;
        let trimmed = path.trim_end_matches('/');
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() >= 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    pub fn truncate_content(content: &str, max_len: usize) -> String {
        if content.len() <= max_len {
            content.to_string()
        } else {
            format!("{}...", &content[..max_len])
        }
    }

    pub fn normalize_blob(parts: &[&str]) -> String {
        parts
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn apply(input: &mut KnowledgeInput) {
        input.title = Self::normalize_title(&input.title);
        if let Some(ref url) = input.url.clone() {
            input.url = Some(Self::normalize_url(url));
        }
        if let Some(ref mut content) = input.content {
            *content = Self::truncate_content(content, 2000);
        }
        input.summary = Self::truncate_content(&input.summary, 500);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title() {
        assert_eq!(
            FormatNormalizer::normalize_title("GitHub - foo/bar: desc"),
            "foo/bar: desc"
        );
        assert_eq!(FormatNormalizer::normalize_title("  hello  "), "hello");
    }

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            FormatNormalizer::normalize_url("https://example.com/foo/"),
            "https://example.com/foo"
        );
    }

    #[test]
    fn test_extract_repo() {
        let r = FormatNormalizer::extract_repo_from_url("https://github.com/ollama/ollama");
        assert_eq!(r, Some(("ollama".into(), "ollama".into())));
    }
}
