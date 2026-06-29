use std::sync::LazyLock;

use crate::neotrix::nt_world_exploration::content::{
    Engagement, ExplorationSourceType, SourceContent,
};
use crate::neotrix::nt_world_exploration::source_trait::ExplorationSource;

static HTTP: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("NeoTrix/0.18 (+https://github.com/neotrix)")
        .build()
        .unwrap_or_else(|e| {
            log::warn!("AlphaXivSource HTTP client init failed: {e}, using default client");
            reqwest::blocking::Client::new()
        })
});

pub struct AlphaXivSource;

impl AlphaXivSource {
    pub fn new() -> Self {
        Self
    }

    fn fetch_homepage(&self) -> Result<String, String> {
        let resp = HTTP
            .get("https://www.alphaxiv.org/")
            .send()
            .map_err(|e| format!("HTTP request to alphaXiv failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("alphaXiv returned status {}", resp.status()));
        }
        resp.text()
            .map_err(|e| format!("Failed to read alphaXiv response body: {e}"))
    }

    fn parse_articles(html: &str) -> Vec<SourceContent> {
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for chunk in html.split("href=\"/abs/").skip(1) {
            let paper_id = match chunk.split('"').next() {
                Some(id) if !id.is_empty() && !seen.contains(id) => id.to_string(),
                _ => continue,
            };
            seen.insert(paper_id.clone());

            let title = chunk
                .split('>')
                .nth(1)
                .and_then(|t| t.split("</a>").next())
                .map(|t| t.trim().to_string())
                .unwrap_or_default();

            let ctx: String = chunk.chars().take(3000).collect();

            let date_str = ctx
                .split("font-medium whitespace-nowrap text-text\">")
                .nth(1)
                .and_then(|s| s.split('<').next())
                .unwrap_or("")
                .to_string();

            let author = ctx
                .split("text-subtext\">")
                .nth(1)
                .and_then(|s| s.split('<').next())
                .unwrap_or("")
                .trim()
                .to_string();

            let tags: Vec<String> = ctx
                .split("hover:text-custom-red\">")
                .skip(1)
                .filter_map(|s| s.split('<').next())
                .map(|t| t.trim().trim_start_matches('#').to_string())
                .filter(|t| !t.is_empty())
                .collect();

            let bookmarks: u64 = ctx
                .split("inline-block\">")
                .nth(1)
                .and_then(|s| s.split('<').next())
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            let _timestamp = parse_date_to_timestamp(&date_str);
            let url = format!("https://www.alphaxiv.org/abs/{paper_id}");

            let tags_text = if tags.is_empty() {
                String::new()
            } else {
                format!(" [Tags: {}]", tags.join(", "))
            };
            let text = format!("Title: {title} | Author: {author} | Date: {date_str}{tags_text}");

            let mut content =
                SourceContent::new(&paper_id, &text, ExplorationSourceType::PaperDatabase)
                    .with_title(&title)
                    .with_url(&url)
                    .with_engagement(Engagement {
                        likes: bookmarks,
                        shares: 0,
                        replies: 0,
                        views: None,
                    })
                    .with_meta("source", "alphaxiv");

            if !author.is_empty() {
                content = content.with_author(&author);
            }
            if !date_str.is_empty() {
                content = content.with_meta("published", &date_str);
            }
            if !tags.is_empty() {
                content = content.with_meta("tags", &tags.join(", "));
            }

            results.push(content);
        }

        results
    }
}

impl ExplorationSource for AlphaXivSource {
    fn name(&self) -> &'static str {
        "alphaxiv"
    }

    fn confidence(&self) -> f64 {
        0.85
    }

    fn explore(&mut self) -> Result<Vec<SourceContent>, String> {
        let html = self.fetch_homepage()?;
        Ok(Self::parse_articles(&html))
    }
}

fn parse_date_to_timestamp(date_str: &str) -> u64 {
    let s = date_str.trim();
    if s.is_empty() {
        return 0;
    }
    chrono::NaiveDate::parse_from_str(s, "%d %b %Y")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_card_html() -> &'static str {
        r#"<div class="rounded-xl border-[0.5px] border-border bg-bg px-4 py-3 backdrop-blur-sm transition-all hover:shadow-md"><div class="flex w-full gap-6"><div class="flex min-w-0 flex-1 flex-col gap-4"><div class="flex flex-col gap-2"><a data-loading-trigger="true" href="/abs/2606.12345" target="_self" class="text-[22px] leading-tight font-bold text-text transition-all hover:underline">Test Paper Title</a></div><div class="flex min-w-0 items-center gap-4 text-sm"><span class="font-medium whitespace-nowrap text-text">15 Jun 2026</span><span class="scrollbar-hide truncate text-subtext">Alice Smith, Bob Jones</span></div><div class="flex flex-col gap-1"><p class="line-clamp-3 text-xs/relaxed tracking-wide text-subtext">This is a test abstract.</p></div><div class="scrollbar-hide flex items-center gap-4 overflow-x-auto"><span class="shrink-0 text-xs font-medium text-text transition-colors hover:text-custom-red">#machine-learning</span><span class="shrink-0 text-xs font-medium text-text transition-colors hover:text-custom-red">#transformers</span></div><div class="mt-auto flex items-center justify-between gap-2"><div class="flex min-w-0 items-center gap-4"><button class="cursor-pointer items-center gap-1.5 text-sm transition-colors flex h-8 shrink-0 rounded-full px-2.5 py-1.5 font-normal bg-surface text-text"><div class="interactable-overlay bg-overlay"></div><svg></svg><span class="inline-block">42</span></button></div></div></div></div></div>"#
    }

    #[test]
    fn test_parse_single_article() {
        let results = AlphaXivSource::parse_articles(sample_card_html());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Paper Title");
        assert_eq!(results[0].id, "2606.12345");
        assert_eq!(
            results[0].url.as_deref(),
            Some("https://www.alphaxiv.org/abs/2606.12345")
        );
        assert!(results[0].text.contains("Alice Smith, Bob Jones"));
        assert!(results[0].text.contains("15 Jun 2026"));
        assert_eq!(results[0].engagement.likes, 42);
        assert_eq!(
            results[0].metadata.get("tags").map(|s| s.as_str()),
            Some("machine-learning, transformers")
        );
        assert_eq!(
            results[0].metadata.get("published").map(|s| s.as_str()),
            Some("15 Jun 2026")
        );
        assert_eq!(results[0].source_type, ExplorationSourceType::PaperDatabase);
    }

    #[test]
    fn test_parse_multiple_articles() {
        let html = format!(
            "{header}{card1}{card2}{footer}",
            header = "<html><body>",
            card1 = sample_card_html(),
            card2 = sample_card_html()
                .replace("2606.12345", "2606.67890")
                .replace("Test Paper Title", "Second Paper")
                .replace("Alice Smith, Bob Jones", "Carol Wang"),
            footer = "</body></html>"
        );
        let results = AlphaXivSource::parse_articles(&html);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Test Paper Title");
        assert_eq!(results[1].title, "Second Paper");
    }

    #[test]
    fn test_dedup_same_paper() {
        let html = format!(
            "{card1a}{card1b}",
            card1a = sample_card_html(),
            card1b = sample_card_html()
        );
        let results = AlphaXivSource::parse_articles(&html);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_empty_html() {
        let results = AlphaXivSource::parse_articles("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_no_abs_links() {
        let results = AlphaXivSource::parse_articles("<html><body>No papers here</body></html>");
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_date_valid() {
        let ts = parse_date_to_timestamp("15 Jun 2026");
        assert!(ts > 0);
    }

    #[test]
    fn test_parse_date_empty() {
        assert_eq!(parse_date_to_timestamp(""), 0);
    }

    #[test]
    fn test_parse_date_invalid() {
        assert_eq!(parse_date_to_timestamp("not a date"), 0);
    }

    #[test]
    fn test_name_and_confidence() {
        let source = AlphaXivSource::new();
        assert_eq!(source.name(), "alphaxiv");
        assert!((source.confidence() - 0.85).abs() < 0.01);
    }
}
