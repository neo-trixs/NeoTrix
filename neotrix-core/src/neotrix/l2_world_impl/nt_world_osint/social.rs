// Allow dead code: social content collection functions are scaffolding for
// future API-key-based platform integrations.
#![allow(dead_code)]

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{OsintConfig, OsintTarget};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPost {
    pub platform: String,
    pub author: String,
    pub content: String,
    pub url: Option<String>,
    pub timestamp: Option<String>,
    pub engagement: Option<u64>,
    pub source: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialFindings {
    pub posts: Vec<SocialPost>,
    pub domain_mentions: Vec<String>,
}

impl std::fmt::Display for SocialFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── Social Media ──")?;
        writeln!(f, "    Posts:          {}", self.posts.len())?;
        writeln!(f, "    Domain mentions: {}", self.domain_mentions.len())?;
        for post in self.posts.iter().take(10) {
            let short: String = post.content.chars().take(120).collect();
            writeln!(f, "      [{}] {}: {}", post.platform, post.author, short)?;
        }
        if self.posts.len() > 10 {
            writeln!(f, "      ... and {} more", self.posts.len() - 10)?;
        }
        Ok(())
    }
}

async fn search_web_for_social(query: &str, client: &Client, source_name: &str) -> Vec<SocialPost> {
    // Search for "site:reddit.com username" type queries via DuckDuckGo
    let encoded = urlencoding(query);
    let url = format!("https://duckduckgo.com/html/?q={encoded}");
    match client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            let mut posts = Vec::new();
            // Extract result snippets from DDG HTML
            for line in body.lines() {
                if line.contains("class=\"result__snippet\"") || line.contains("class=\"result__body\"") {
                    let content = extract_text_between(line, ">", "</");
                    if !content.is_empty() {
                        posts.push(SocialPost {
                            platform: source_name.to_string(),
                            author: "search".to_string(),
                            content: content.to_string(),
                            url: None,
                            timestamp: None,
                            engagement: None,
                            source: "web-search".to_string(),
                        });
                    }
                }
            }
            posts.truncate(20);
            posts
        }
        _ => vec![],
    }
}

fn urlencoding(input: &str) -> String {
    input.as_bytes().iter().map(|&byte| {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            b' ' => '+'.to_string(),
            _ => format!("%{:02X}", byte),
        }
    }).collect::<String>()
}

fn extract_text_between<'a>(s: &'a str, start_delim: &str, end_delim: &str) -> &'a str {
    if let Some(start) = s.find(start_delim) {
        let from = start + start_delim.len();
        if let Some(end) = s[from..].find(end_delim) {
            return s[from..from + end].trim();
        }
    }
    ""
}

pub async fn investigate(target: &OsintTarget, _client: &Client, _config: &OsintConfig) -> Result<SocialFindings, String> {
    let mut findings = SocialFindings::default();

    // In the current implementation, we rely on the person::investigate profile checks
    // for social media platform presence. Social media content collection (actual posts)
    // requires platform-specific APIs, which need API keys.
    //
    // This module provides the framework for collecting social media posts from
    // platforms where we have API access. Future enhancements:
    // - Twitter/X API v2 for user timeline
    // - Reddit API for user posts/comments
    // - Bluesky AT Protocol for firehose
    // - Mastodon API for federated search
    //
    // For now, this module returns empty findings with a descriptive message.
    // The person::investigate module handles platform PRESENCE detection.
    // This module will handle platform CONTENT collection when APIs are configured.

    if let Some(ref domain) = target.domain {
        findings.domain_mentions.push(format!("Domain {domain} - social monitoring requires API keys for content collection"));
    }
    if let Some(ref username) = target.username {
        findings.domain_mentions.push(format!("Username {username} - content collection requires platform API keys"));
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencoding_simple() {
        assert_eq!(urlencoding("hello"), "hello");
    }

    #[test]
    fn test_urlencoding_spaces() {
        assert_eq!(urlencoding("hello world"), "hello+world");
    }

    #[test]
    fn test_urlencoding_special() {
        let encoded = urlencoding("a&b=c");
        assert_eq!(encoded, "a%26b%3Dc");
    }

    #[test]
    fn test_extract_text_between() {
        let s = "foo<b>bar</b>baz";
        assert_eq!(extract_text_between(s, "<b>", "</b>"), "bar");
    }

    #[test]
    fn test_extract_text_between_no_match() {
        let s = "foo bar baz";
        assert_eq!(extract_text_between(s, "<b>", "</b>"), "");
    }

    #[test]
    fn test_social_findings_default() {
        let f = SocialFindings::default();
        assert!(f.posts.is_empty());
        assert!(f.domain_mentions.is_empty());
    }

    #[test]
    fn test_social_findings_display_empty() {
        let f = SocialFindings::default();
        let s = format!("{f}");
        assert!(s.contains("Social Media"));
    }

    #[test]
    fn test_duckduckgo_url() {
        let q = urlencoding("site:reddit.com testuser");
        let url = format!("https://duckduckgo.com/html/?q={q}");
        assert!(url.contains("testuser"));
        assert!(url.contains("site%3Areddit"));
    }
}
