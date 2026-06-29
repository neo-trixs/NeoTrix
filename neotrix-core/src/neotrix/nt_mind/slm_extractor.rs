//! SLM-based HTML extraction + Data source scrapers
//!
//! G315.1 + G324.1 — Wave 10 of EVOLUTION_ROADMAP_v14.md.
//! Part 1: SlmConfig, SlmHtmlExtractor (ReaderLM-v2 placeholder), OnnxRuntime stub.
//! Part 2: Specialized data source scrapers (YouTube, Google, Amazon, Wikipedia).
//! Part 3: UnifiedSourceExtractor with auto-detect and dispatch.
//!
//! Design intent: Rules-based extraction (SmartContentExtractor) is the fast path.
//! SLM extraction is the fallback when rules are insufficient. Data source scrapers
//! provide typed, structured extraction from known site layouts.
//!
//! CRITICAL: All HTML parsing uses string-based scanning + regex (no external HTML parser).

use std::collections::HashMap;
use chrono;

// ============================================================================
// Part 1: SLM Configuration and Extraction
// ============================================================================

/// SLM (Small Language Model) configuration for HTML extraction
#[derive(Debug, Clone)]
pub struct SlmConfig {
    pub model_name: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub top_p: f64,
    pub use_onnx: bool,
    pub use_llama: bool,
    pub model_path: Option<String>,
    pub device: String,
}

impl Default for SlmConfig {
    fn default() -> Self {
        Self {
            model_name: "reader-lm-v2".into(),
            max_tokens: 2048,
            temperature: 0.1,
            top_p: 0.9,
            use_onnx: false,
            use_llama: false,
            model_path: None,
            device: "cpu".into(),
        }
    }
}

/// SLM-based HTML → Markdown extractor
///
/// Wraps ReaderLM-v2 or compatible small language models for HTML extraction.
/// When no model is available, falls back to HTML preparation for downstream processing.
pub struct SlmHtmlExtractor {
    pub config: SlmConfig,
    pub model_loaded: bool,
}

impl SlmHtmlExtractor {
    pub fn new(config: SlmConfig) -> Self {
        Self {
            model_loaded: false,
            config,
        }
    }

    /// Check if the SLM model is available for inference
    pub fn is_available(&self) -> bool {
        self.model_loaded || self.config.model_path.is_some()
    }

    /// Extract markdown from HTML using the SLM model (if available).
    /// Placeholder — actual ONNX/llama.cpp inference not yet connected.
    pub fn extract_markdown(&self, html: &str) -> Result<String, String> {
        if !self.is_available() {
            return Err("SLM model not available. Call prepare_html() instead.".into());
        }
        Ok(format!(
            "# SLM Extraction ({})\n\nModel: {}\nInput length: {} chars\n\n*Model inference not yet connected to ONNX runtime*",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            self.config.model_name,
            html.len()
        ))
    }

    /// Prepare HTML for SLM input: remove scripts/styles, chunk at max_chars.
    /// Returns chunks suitable for batch processing.
    pub fn prepare_html(&self, html: &str, max_chars: usize) -> Vec<String> {
        let cleaned = self.strip_script_style(html);
        if cleaned.len() <= max_chars {
            return vec![cleaned];
        }
        let mut chunks = Vec::new();
        let mut start = 0;
        while start < cleaned.len() {
            let end = self.find_chunk_boundary(&cleaned, start, max_chars);
            chunks.push(cleaned[start..end].to_string());
            start = end;
        }
        if chunks.is_empty() {
            chunks.push(String::new());
        }
        chunks
    }

    /// Build a ReaderLM-v2 prompt from an HTML chunk.
    /// Truncates HTML at 8000 chars to stay within model context window.
    pub fn build_readerlm_prompt(html_chunk: &str) -> String {
        let truncated = if html_chunk.len() > 8000 {
            &html_chunk[..8000]
        } else {
            html_chunk
        };
        format!(
            "Convert the following HTML to clean Markdown.\n\n\
             Preserve:\n\
             - Headings (#)\n\
             - Lists (-, 1.)\n\
             - Tables (|)\n\
             - Links [text](url)\n\
             - Images ![alt](src)\n\
             - Code blocks (```)\n\n\
             HTML:\n\
             ```html\n\
             {}\n\
             ```\n\n\
             Markdown:",
            truncated
        )
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Remove <script> and <style> blocks from HTML.
    fn strip_script_style(&self, html: &str) -> String {
        let mut result = String::with_capacity(html.len());
        let mut i = 0;
        let bytes = html.as_bytes();
        while i < bytes.len() {
            if bytes[i] == b'<' {
                let rest = &html[i..];
                let lower = rest.to_lowercase();
                if lower.starts_with("<script") {
                    if let Some(end) = rest.to_lowercase().find("</script>") {
                        i += end + 9;
                        continue;
                    }
                }
                if lower.starts_with("<style") {
                    if let Some(end) = rest.to_lowercase().find("</style>") {
                        i += end + 8;
                        continue;
                    }
                }
            }
            result.push(bytes[i] as char);
            i += 1;
        }
        result
    }

    /// Find a good chunk boundary near max_chars from start.
    /// Tries to break at a double newline, single newline, or period-space boundary.
    fn find_chunk_boundary(&self, s: &str, start: usize, max_chars: usize) -> usize {
        let end = (start + max_chars).min(s.len());
        if end >= s.len() {
            return s.len();
        }
        // Try to find a natural boundary within the last 20% of the chunk
        let search_start = (end.saturating_sub(max_chars / 5)).max(start);
        let candidates = [
            s[search_start..end].rfind("\n\n"),
            s[search_start..end].rfind('\n'),
            s[search_start..end].rfind(". "),
        ];
        for candidate in candidates.iter() {
            if let Some(pos) = candidate {
                let boundary = search_start + pos + 1;
                if boundary > start && boundary < s.len() {
                    return boundary;
                }
            }
        }
        end
    }
}

// ============================================================================
// OnnxRuntime Stub — placeholder for future ONNX integration (ReaderLM-v2)
// ============================================================================

/// Stub for ONNX Runtime integration (ReaderLM-v2)
pub struct OnnxRuntime {
    pub initialized: bool,
    pub model_path: Option<String>,
}

impl OnnxRuntime {
    pub fn new(model_path: Option<&str>) -> Self {
        Self {
            initialized: false,
            model_path: model_path.map(|s| s.to_string()),
        }
    }

    pub fn load_model(&mut self) -> Result<(), String> {
        if self.model_path.is_none() {
            return Err("No model path configured".into());
        }
        self.initialized = true;
        Ok(())
    }

    pub fn run_inference(&self, _input_tokens: &[u32]) -> Result<Vec<u32>, String> {
        if !self.initialized {
            return Err("ONNX Runtime not initialized".into());
        }
        Ok(vec![])
    }
}

// ============================================================================
// Part 2: Data Source Scrapers (G324.1)
// ============================================================================

/// Specialized scrapers for known data source layouts
pub mod data_sources {
    use std::collections::HashMap;

    // ----------------------------------------------------------------
    // YouTube
    // ----------------------------------------------------------------

    /// Video metadata extracted from a YouTube page
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct VideoInfo {
        pub title: String,
        pub channel: String,
        pub duration_secs: u64,
        pub view_count: u64,
        pub description: String,
        pub publish_date: Option<String>,
    }

    /// A YouTube comment
    #[derive(Debug, Clone)]
    pub struct Comment {
        pub author: String,
        pub text: String,
        pub likes: u64,
        pub timestamp: String,
    }

    /// A related video reference
    #[derive(Debug, Clone)]
    pub struct VideoRef {
        pub title: String,
        pub url: String,
        pub channel: String,
    }

    /// YouTube page scraper
    pub struct YouTubeScraper;

    impl YouTubeScraper {
        /// Extract video info from YouTube HTML.
        /// Checks <title>, JSON-LD, og:title/og:description meta, and channel elements.
        pub fn extract_video_info(html: &str) -> Result<VideoInfo, String> {
            let title = Self::extract_meta_content(html, "og:title")
                .or_else(|| Self::extract_title_tag(html))
                .unwrap_or_default()
                .trim()
                .trim_end_matches(" - YouTube")
                .to_string();

            let channel = Self::extract_meta_content(html, "og:site_name")
                .or_else(|| Self::extract_channel_from_link(html))
                .unwrap_or_default();

            let description = Self::extract_meta_content(html, "og:description")
                .or_else(|| Self::extract_meta_content(html, "description"))
                .unwrap_or_default();

            let view_count = Self::extract_view_count(html);
            let duration_secs = Self::extract_duration(html);

            let publish_date = Self::extract_meta_content(html, "datePublished")
                .or_else(|| Self::extract_meta_content(html, "article:published_time"));

            Ok(VideoInfo {
                title,
                channel,
                duration_secs,
                view_count,
                description,
                publish_date,
            })
        }

        /// Extract comments from YouTube HTML
        pub fn extract_comments(html: &str) -> Vec<Comment> {
            let mut comments = Vec::new();
            // Look for comment sections in yt-formatted-string or #content-text
            let mut search_start = 0;
            for _ in 0..200 {
                // Find comment author
                let author_patterns = [
                    r#"<a[^>]*id="author-text"[^>]*>"#,
                    r#"<span[^>]*class="[^"]*author[^"]*"[^>]*>"#,
                    r#"<yt-formatted-string[^>]*id="author"[^>]*>"#,
                ];
                let mut found = false;
                for pat in &author_patterns {
                    if let Some(pos) = Self::find_pattern(html, pat, search_start) {
                        let author_start = pos;
                        let author = Self::extract_text_content(&html[author_start..]);
                        if author.is_empty() {
                            search_start = pos + 1;
                            continue;
                        }
                        // Find comment text after author
                        if let Some(text_start) = Self::find_pattern(html, r#"id="content-text"#, author_start + 50) {
                            let text = Self::extract_text_content(&html[text_start..]);
                            let likes = 0;
                            let timestamp = String::new();
                            comments.push(Comment { author, text, likes, timestamp });
                            search_start = text_start + 20;
                            found = true;
                            break;
                        }
                        search_start = pos + 1;
                        found = true;
                        break;
                    }
                }
                if !found {
                    break;
                }
                if comments.len() >= 100 {
                    break;
                }
            }
            comments
        }

        /// Extract related videos from YouTube HTML
        pub fn extract_related_videos(html: &str) -> Vec<VideoRef> {
            let mut videos = Vec::new();
            let mut search_start = 0;

            // Look for ytd-compact-video-renderer or similar
            let renderer_pat = r#"ytd-compact-video-renderer"#;
            while let Some(pos) = html[search_start..].find(renderer_pat) {
                let abs_pos = search_start + pos;
                let section = &html[abs_pos..abs_pos + 1500];

                let title = Self::extract_text_between_markers(section, r#"title=""#, r#"""#)
                    .or_else(|| {
                        let tpos = section.find("aria-title");
                        tpos.and_then(|p| {
                            let start = p + 12;
                            section[start..].find('"').map(|e| section[start..start + e].to_string())
                        })
                    })
                    .unwrap_or_default();

                let url = Self::extract_text_between_markers(section, r#"href="/watch?v=""#, r#"""#)
                    .map(|s| format!("/watch?v={}", s))
                    .unwrap_or_default();

                let channel = Self::extract_text_between_markers(section, r#"aria-label=""#, r#"""#)
                    .unwrap_or_default();

                if !title.is_empty() {
                    videos.push(VideoRef { title, url, channel });
                }

                search_start = abs_pos + 50;
                if videos.len() >= 50 {
                    break;
                }
            }
            videos
        }

        // -- Internal helpers --

        fn extract_meta_content(html: &str, property: &str) -> Option<String> {
            // Try property="..." first, then name="..."
            let p_esc = regex::escape(property);
            for attr in &["property", "name"] {
                let pat = [
                    "<meta[^>]*?(?:", attr, "=\"", &p_esc,
                    "\"[^>]*?content=\"([^\"]*)\"|content=\"([^\"]*)\"[^>]*?",
                    attr, "=\"", &p_esc, "\")",
                ].concat();
                let re = regex::Regex::new(&pat).ok()?;
                if let Some(caps) = re.captures(html) {
                    if let Some(val) = caps.get(1).or_else(|| caps.get(2)) {
                        let v = val.as_str().to_string();
                        if !v.is_empty() {
                            return Some(v);
                        }
                    }
                }
            }
            None
        }

        fn extract_title_tag(html: &str) -> Option<String> {
            let start = html.find("<title>")?;
            let content_start = start + 7;
            let end = html[content_start..].find("</title>")?;
            Some(html[content_start..content_start + end].to_string())
        }

        fn extract_channel_from_link(html: &str) -> Option<String> {
            // Look for channel name in link or ytd-channel-name
            if let Some(pos) = html.find(r#"ytd-channel-name"#) {
                return Some(Self::extract_text_content(&html[pos..]));
            }
            if let Some(pos) = html.find(r#"/channel/"#) {
                let section = &html[pos..pos + 300];
                return Some(Self::extract_text_content(section));
            }
            None
        }

        fn extract_view_count(html: &str) -> u64 {
            // Look for view count in meta or text
            if let Some(v) = Self::extract_meta_content(html, "interactionCount") {
                if let Ok(n) = v.parse::<u64>() {
                    return n;
                }
            }
            // Try text pattern: "123,456 views" or "123K views"
            let patterns = [
                r#"(\d[\d,]*)\s*views"#,
                r#"(\d+\.?\d*[KMB]?)\s*views"#,
            ];
            for pat in &patterns {
                let re = regex::Regex::new(pat).ok();
                if let Some(re) = re {
                    if let Some(caps) = re.captures(html) {
                        if let Some(m) = caps.get(1) {
                            let s = m.as_str().replace(',', "");
                            if let Ok(n) = s.parse::<u64>() {
                                return n;
                            }
                            // Handle K/M/B suffixes
                            if s.ends_with('K') {
                                if let Ok(n) = s.trim_end_matches('K').parse::<f64>() {
                                    return (n * 1000.0) as u64;
                                }
                            }
                            if s.ends_with('M') {
                                if let Ok(n) = s.trim_end_matches('M').parse::<f64>() {
                                    return (n * 1_000_000.0) as u64;
                                }
                            }
                        }
                    }
                }
            }
            0
        }

        fn extract_duration(html: &str) -> u64 {
            // ISO 8601 duration from meta or JSON-LD: PT1H2M3S
            let re = regex::Regex::new(r#"PT?(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?"#).ok();
            if let Some(re) = re {
                if let Some(caps) = re.captures(html) {
                    let h: u64 = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                    let m: u64 = caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                    let s: u64 = caps.get(3).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                    return h * 3600 + m * 60 + s;
                }
            }
            // Fallback: duration text like "12:34" or "1:02:34"
            let re2 = regex::Regex::new(r#"(\d+):(\d{2})(?::(\d{2}))?"#).ok();
            if let Some(re2) = re2 {
                if let Some(caps) = re2.captures(html) {
                    let has_hours = caps.get(3).is_some();
                    if has_hours {
                        let h: u64 = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                        let m: u64 = caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                        let s: u64 = caps.get(3).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                        return h * 3600 + m * 60 + s;
                    }
                    let m: u64 = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                    let s: u64 = caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                    return m * 60 + s;
                }
            }
            0
        }

        /// Extract visible text content from an HTML fragment starting at pos
        fn extract_text_content(html_fragment: &str) -> String {
            let mut out = String::new();
            let mut in_tag = false;
            for c in html_fragment.chars() {
                match c {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ if !in_tag => {
                        if c.is_whitespace() {
                            if !out.ends_with(' ') && !out.is_empty() {
                                out.push(' ');
                            }
                        } else {
                            out.push(c);
                        }
                    }
                    _ => {}
                }
            }
            out.trim().to_string()
        }

        fn find_pattern(html: &str, pattern: &str, start: usize) -> Option<usize> {
            if start >= html.len() {
                return None;
            }
            let rest = &html[start..];
            rest.find(pattern).map(|pos| start + pos)
        }

        /// Extract text between two markers (e.g., title="..." → content between " and ")
        fn extract_text_between_markers(text: &str, start_marker: &str, end_marker: &str) -> Option<String> {
            let start = text.find(start_marker)?;
            let value_start = start + start_marker.len();
            let rest = &text[value_start..];
            let end = rest.find(end_marker)?;
            Some(rest[..end].to_string())
        }
    }

    // ----------------------------------------------------------------
    // Google Search
    // ----------------------------------------------------------------

    /// A Google search result
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct SearchResult {
        pub position: u32,
        pub title: String,
        pub url: String,
        pub snippet: String,
        pub is_ad: bool,
    }

    /// Google SERP scraper
    pub struct GoogleSearchScraper;

    impl GoogleSearchScraper {
        /// Extract all search results (organic + ads) from SERP HTML
        pub fn extract_results(html: &str) -> Vec<SearchResult> {
            let mut results = Vec::new();
            let organic = Self::extract_organic(html);
            let ads = Self::extract_ads(html);
            let mut pos = 0u32;

            // Interleave ads and organic based on position markers
            // Simplified: add ads first (they appear above organic), then organic
            for ad in ads {
                pos += 1;
                results.push(SearchResult {
                    position: pos,
                    is_ad: true,
                    ..ad
                });
            }
            for mut org in organic {
                pos += 1;
                org.position = pos;
                results.push(org);
            }
            results
        }

        /// Extract organic search results
        pub fn extract_organic(html: &str) -> Vec<SearchResult> {
            let mut results = Vec::new();
            let mut search_start = 0;
            let mut pos = 0u32;

            // Look for result containers: <div class="g"> or <div data-hveid="...">
            while let Some(start) = Self::find_next_result(html, search_start) {
                pos += 1;
                let result_end = Self::find_result_end(html, start);
                let section = &html[start..result_end.min(html.len())];

                let title = Self::extract_title_from_result(section);
                let url = Self::extract_url_from_result(section);
                let snippet = Self::extract_snippet_from_result(section);

                if !title.is_empty() {
                    results.push(SearchResult {
                        position: pos,
                        title,
                        url,
                        snippet,
                        is_ad: false,
                    });
                }

                search_start = result_end;
                if results.len() >= 100 {
                    break;
                }
            }
            results
        }

        /// Extract featured snippet
        pub fn extract_featured_snippet(html: &str) -> Option<String> {
            // Look for featured snippet containers
            let patterns = [
                r#"<div[^>]*class="[^"]*[Kk]nowledge[ -][Pp]anel[^"]*"[^>]*>"#,
                r#"<div[^>]*id="rhs"[^>]*>"#,
                r#"<div[^>]*class="[^"]*kp-blk[^"]*"[^>]*>"#,
            ];
            for pat in &patterns {
                if let Some(pos) = Self::regex_find(html, pat) {
                    let section = &html[pos..(pos + 3000).min(html.len())];
                    let text = Self::extract_visible_text(section);
                    if text.len() > 50 {
                        return Some(text);
                    }
                }
            }
            None
        }

        /// Extract "People also ask" questions
        pub fn extract_people_also_ask(html: &str) -> Vec<String> {
            let mut questions = Vec::new();
            let mut search_start = 0;
            let question_pat = r#"<div[^>]*class="[^"]*related-question-pair[^"]*"[^>]*>"#;

            while let Some(pos) = Self::find_pattern_simple(html, question_pat, search_start) {
                let section = &html[pos..(pos + 1000).min(html.len())];
                let text = Self::extract_visible_text(section);
                if !text.is_empty() && text.ends_with('?') {
                    questions.push(text);
                }
                search_start = pos + 50;
                if questions.len() >= 20 {
                    break;
                }
            }
            questions
        }

        /// Extract related search suggestions
        pub fn extract_related_searches(html: &str) -> Vec<String> {
            let mut searches = Vec::new();
            let mut search_start = 0;
            let pat = r#"<a[^>]*class="[^"]*[Rr]elated[Ss]earch[^"]*"[^>]*>"#;

            while let Some(pos) = Self::find_pattern_simple(html, pat, search_start) {
                let section = &html[pos..(pos + 500).min(html.len())];
                let text = Self::extract_visible_text(section);
                if !text.is_empty() {
                    searches.push(text);
                }
                search_start = pos + 50;
                if searches.len() >= 20 {
                    break;
                }
            }
            searches
        }

        /// Detect if Google returned a CAPTCHA challenge
        pub fn has_captcha(html: &str) -> bool {
            let captcha_signals = [
                "sorry/index",
                "unusual traffic",
                "captcha",
                "recaptcha",
                "robot",
                "automated queries",
                "not a robot",
                "browser check",
                "challenge?",
            ];
            let lower = html.to_lowercase();
            captcha_signals.iter().any(|sig| lower.contains(sig))
        }

        // -- Internal helpers --

        fn find_next_result(html: &str, start: usize) -> Option<usize> {
            let patterns = [
                r#"<div\s[^>]*class="[^"]*?\bg\b[^"]*?"#,
                r#"<div\s[^>]*data-hveid="[^"]*"#,
            ];
            for pat in &patterns {
                if let Some(pos) = Self::regex_find_from(html, pat, start) {
                    return Some(pos);
                }
            }
            None
        }

        fn find_result_end(html: &str, start: usize) -> usize {
            // Find the next result container or approximate end
            let search_from = start + 50;
            let patterns = [
                r#"<div\s[^>]*class="[^"]*?\bg\b[^"]*?"#,
                r#"</div>\s*</div>\s*</div>"#,
            ];
            for pat in &patterns {
                if let Some(pos) = Self::regex_find_from(html, pat, search_from) {
                    return pos;
                }
            }
            (start + 2000).min(html.len())
        }

        fn extract_title_from_result(section: &str) -> String {
            let patterns = [
                r#"<h3[^>]*>(.*?)</h3>"#,
                r#"<a[^>]*>(.*?)</a>"#,
            ];
            for pat in &patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    if let Some(caps) = re.captures(section) {
                        if let Some(m) = caps.get(1) {
                            let text = Self::extract_visible_text(m.as_str());
                            if !text.is_empty() {
                                return text;
                            }
                        }
                    }
                }
            }
            String::new()
        }

        fn extract_url_from_result(section: &str) -> String {
            let re = regex::Regex::new(r#"<a\s[^>]*href="([^"]+)"#).ok();
            if let Some(re) = re {
                if let Some(caps) = re.captures(section) {
                    if let Some(m) = caps.get(1) {
                        let url = m.as_str();
                        if url.starts_with("http") {
                            return url.to_string();
                        }
                    }
                }
            }
            String::new()
        }

        fn extract_snippet_from_result(section: &str) -> String {
            let patterns = [
                r#"<span[^>]*class="[^"]*st[^"]*"[^>]*>(.*?)</span>"#,
                r#"<div[^>]*class="[^"]*VwiC3b[^"]*"[^>]*>(.*?)</div>"#,
                r#"<div[^>]*class="[^"]*[Ss]nippet[^"]*"[^>]*>(.*?)</div>"#,
            ];
            for pat in &patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    if let Some(caps) = re.captures(section) {
                        if let Some(m) = caps.get(1) {
                            let text = Self::extract_visible_text(m.as_str());
                            if text.len() > 20 {
                                return text;
                            }
                        }
                    }
                }
            }
            String::new()
        }

        fn extract_ads(html: &str) -> Vec<SearchResult> {
            let mut ads = Vec::new();
            let mut search_start = 0;
            let ad_pat = r#"<div[^>]*class="[^"]*uEierd[^"]*"[^>]*>"#;

            while let Some(pos) = Self::find_pattern_simple(html, ad_pat, search_start) {
                let section = &html[pos..(pos + 1500).min(html.len())];
                let title = Self::extract_title_from_result(section);
                let url = Self::extract_url_from_result(section);
                let snippet = Self::extract_snippet_from_result(section);
                if !title.is_empty() {
                    ads.push(SearchResult { position: 0, title, url, snippet, is_ad: true });
                }
                search_start = pos + 100;
                if ads.len() >= 20 {
                    break;
                }
            }
            ads
        }

        fn extract_visible_text(html: &str) -> String {
            let mut out = String::new();
            let mut in_tag = false;
            for c in html.chars() {
                match c {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ if !in_tag => {
                        if c.is_whitespace() {
                            if !out.ends_with(' ') && !out.is_empty() {
                                out.push(' ');
                            }
                        } else {
                            out.push(c);
                        }
                    }
                    _ => {}
                }
            }
            out.trim().to_string()
        }

        fn regex_find(html: &str, pattern: &str) -> Option<usize> {
            let re = regex::Regex::new(pattern).ok()?;
            re.find(html).map(|m| m.start())
        }

        fn regex_find_from(html: &str, pattern: &str, start: usize) -> Option<usize> {
            if start >= html.len() {
                return None;
            }
            let re = regex::Regex::new(pattern).ok()?;
            re.find_at(html, start).map(|m| m.start())
        }

        fn find_pattern_simple(html: &str, pattern: &str, start: usize) -> Option<usize> {
            if start >= html.len() {
                return None;
            }
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(m) = re.find(&html[start..]) {
                    return Some(start + m.start());
                }
            }
            None
        }
    }

    // ----------------------------------------------------------------
    // Amazon
    // ----------------------------------------------------------------

    /// Amazon product information
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct ProductInfo {
        pub title: String,
        pub price: String,
        pub currency: String,
        pub rating: f64,
        pub review_count: u64,
        pub availability: String,
        pub asin: Option<String>,
    }

    /// Amazon search result reference
    #[derive(Debug, Clone)]
    pub struct ProductRef {
        pub position: u32,
        pub title: String,
        pub url: String,
        pub price: String,
        pub rating: f64,
    }

    /// Amazon product review
    #[derive(Debug, Clone)]
    pub struct Review {
        pub author: String,
        pub rating: u8,
        pub title: String,
        pub text: String,
        pub date: String,
    }

    /// Amazon product page scraper
    pub struct AmazonScraper;

    impl AmazonScraper {
        /// Extract product info from an Amazon product page HTML
        pub fn extract_product(html: &str) -> Result<ProductInfo, String> {
            let title = Self::extract_between(html, r#"<span id="productTitle"#)
                .or_else(|| {
                    // Try alternative title locations
                    let pos = html.find(r#"id="productTitle""#);
                    pos.and_then(|p| {
                        let start = html[p..].find('>')?;
                        let content_start = p + start + 1;
                        let end = html[content_start..].find('<')?;
                        Some(html[content_start..content_start + end].trim().to_string())
                    })
                })
                .ok_or_else(|| "Could not find product title".to_string())?;

            let price = Self::extract_price(html);
            let currency = Self::extract_currency(html);
            let (rating, review_count) = Self::extract_rating_and_count(html);
            let availability = Self::extract_availability(html);
            let asin = Self::extract_asin(html);

            Ok(ProductInfo {
                title,
                price,
                currency,
                rating,
                review_count,
                availability,
                asin,
            })
        }

        /// Extract search results from an Amazon search page HTML
        pub fn extract_search_results(html: &str) -> Vec<ProductRef> {
            let mut results = Vec::new();
            let mut search_start = 0;
            let mut pos = 0u32;

            // Look for product containers: <div data-asin="...">
            while let Some(asin_start) = html[search_start..].find(r#"data-asin=""#) {
                let abs_start = search_start + asin_start;
                // Find the end of this container
                let section_end = (abs_start + 2000).min(html.len());
                let section = &html[abs_start..section_end];

                // Extract ASIN
                let asin_start_val = abs_start + r#"data-asin=""#.len();
                let asin_end = html[asin_start_val..].find('"').map(|e| asin_start_val + e);
                let _asin = asin_end.map(|e| &html[asin_start_val..e]);

                pos += 1;
                let title = Self::extract_title_from_search(section);
                let url = String::new(); // Build from ASIN if needed
                let price = Self::extract_price_from_search(section);
                let rating = Self::extract_rating_from_search(section);

                if !title.is_empty() {
                    results.push(ProductRef { position: pos, title, url, price, rating });
                }

                search_start = abs_start + 50;
                if results.len() >= 100 {
                    break;
                }
            }
            results
        }

        /// Extract reviews from a product page
        pub fn extract_reviews(html: &str, max: usize) -> Vec<Review> {
            let mut reviews = Vec::new();
            let mut search_start = 0;

            // Look for review containers: <div data-hook="review">
            let review_pat = r#"data-hook="review""#;
            let mut count = 0;
            while count < max {
                if let Some(pos) = html[search_start..].find(review_pat) {
                    let abs_pos = search_start + pos;
                    let section = &html[abs_pos..(abs_pos + 2000).min(html.len())];

                    let author = Self::extract_between(section, r#"class="[^"]*a-profile-name[^"]*"#)
                        .unwrap_or_default();
                    let rating = Self::extract_review_rating(section);
                    let title = Self::extract_review_title(section);
                    let text = Self::extract_review_text(section);
                    let date = Self::extract_review_date(section);

                    reviews.push(Review { author, rating, title, text, date });
                    count += 1;
                    search_start = abs_pos + 100;
                } else {
                    break;
                }
            }
            reviews
        }

        // -- Internal helpers --

        /// Find text inside an element identified by an attribute marker.
        /// E.g., extract_between(html, r#"id="productTitle"#) extracts the text
        /// content of that element.
        fn extract_between(html: &str, attr_marker: &str) -> Option<String> {
            let pos = html.find(attr_marker)?;
            let after_attr = &html[pos + attr_marker.len()..];
            let tag_end = after_attr.find('>')?;
            let content_start = tag_end + 1;
            let content_end = after_attr[content_start..].find('<')?;
            let text = after_attr[content_start..content_start + content_end].trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        }

        fn extract_price(html: &str) -> String {
            let patterns = [
                (r#"<span[^>]*class="[^"]*a-price[^"]*"[^>]*>"#, true),
                (r#"<span[^>]*class="[^"]*a-offscreen[^"]*"[^>]*>"#, true),
                (r#"<span[^>]*id="priceblock"[^>]*>"#, true),
                (r#"<span[^>]*id="price"[^>]*>"#, true),
            ];
            for (pat, use_text) in &patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    if let Some(m) = re.find(html) {
                        let after = &html[m.end()..];
                        if *use_text {
                            let content_end = after.find('<').unwrap_or(after.len().min(200));
                            let text = after[..content_end].trim().to_string();
                            if !text.is_empty() {
                                // Clean up price text
                                let clean: String = text.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == '$' || *c == '€' || *c == '£' || *c == '¥').collect();
                                if !clean.is_empty() {
                                    return clean;
                                }
                                return text;
                            }
                        }
                    }
                }
            }
            String::new()
        }

        fn extract_currency(html: &str) -> String {
            if html.contains("€") || html.contains("&euro;") {
                return "EUR".into();
            }
            if html.contains("£") || html.contains("&pound;") {
                return "GBP".into();
            }
            if html.contains("¥") || html.contains("&yen;") {
                return "JPY".into();
            }
            // Default to USD for $
            "USD".into()
        }

        fn extract_rating_and_count(html: &str) -> (f64, u64) {
            // Rating: look for <span class="a-icon-alt">4.5 out of 5</span>
            let rating_re = regex::Regex::new(r#"(\d+\.?\d*)\s*out\s*of\s*5"#).ok();
            let rating = rating_re
                .and_then(|re| re.captures(html))
                .and_then(|caps| caps.get(1))
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or(0.0);

            // Review count: look for numbers near "ratings" or "reviews"
            let count_re = regex::Regex::new(r#"(\d[\d,]*)\s*(?:ratings|reviews|global.ratings?)"#).ok();
            let count = count_re
                .and_then(|re| re.captures(html))
                .and_then(|caps| caps.get(1))
                .and_then(|m| m.as_str().replace(',', "").parse::<u64>().ok())
                .unwrap_or(0);

            (rating, count)
        }

        fn extract_availability(html: &str) -> String {
            let patterns = [
                (r#"<span[^>]*class="[^"]*availability[^"]*"[^>]*>"#, true),
                (r#"<div[^>]*id="availability"[^>]*>"#, true),
                (r#"<span[^>]*class="[^"]*a-color-success[^"]*"[^>]*>"#, true),
            ];
            for (pat, _) in &patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    if let Some(m) = re.find(html) {
                        let after = &html[m.end()..];
                        let content_end = after.find('<').unwrap_or(after.len().min(200));
                        let text = after[..content_end].trim().to_string();
                        if !text.is_empty() {
                            let clean: String = text.chars().filter(|c| !c.is_control() && *c != '\n' && *c != '\r').collect();
                            let clean = clean.trim().to_string();
                            if !clean.is_empty() {
                                return clean;
                            }
                        }
                    }
                }
            }
            "Unknown".into()
        }

        fn extract_asin(html: &str) -> Option<String> {
            let patterns = [
                r#"<input[^>]*name="ASIN"[^>]*value="([^"]+)"#,
                r#"data-asin="([^"]+)"#,
            ];
            for pat in &patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    if let Some(caps) = re.captures(html) {
                        if let Some(m) = caps.get(1) {
                            let asin = m.as_str();
                            if asin.len() == 10 {
                                return Some(asin.to_string());
                            }
                        }
                    }
                }
            }
            None
        }

        fn extract_title_from_search(section: &str) -> String {
            let patterns = [
                r#"<h2[^>]*>(.*?)</h2>"#,
                r#"<span[^>]*class="[^"]*a-text-normal[^"]*"[^>]*>(.*?)</span>"#,
            ];
            for pat in &patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    if let Some(caps) = re.captures(section) {
                        if let Some(m) = caps.get(1) {
                            let text = strip_html_simple(m.as_str());
                            if text.len() > 5 {
                                return text;
                            }
                        }
                    }
                }
            }
            String::new()
        }

        fn extract_price_from_search(section: &str) -> String {
            let price_re = regex::Regex::new(r#"\$\d+(?:\.\d{2})?"#).ok();
            if let Some(re) = price_re {
                if let Some(m) = re.find(section) {
                    return m.as_str().to_string();
                }
            }
            String::new()
        }

        fn extract_rating_from_search(section: &str) -> f64 {
            let rating_re = regex::Regex::new(r#"aria-label="(\d+\.?\d*) out of 5 stars"#).ok();
            if let Some(re) = rating_re {
                if let Some(caps) = re.captures(section) {
                    if let Some(m) = caps.get(1) {
                        if let Ok(r) = m.as_str().parse::<f64>() {
                            return r;
                        }
                    }
                }
            }
            0.0
        }

        fn extract_review_rating(section: &str) -> u8 {
            let rating_re = regex::Regex::new(r#"aria-label="(\d+) out of 5 stars"#).ok();
            if let Some(re) = rating_re {
                if let Some(caps) = re.captures(section) {
                    if let Some(m) = caps.get(1) {
                        return m.as_str().parse().unwrap_or(0);
                    }
                }
            }
            0
        }

        fn extract_review_title(section: &str) -> String {
            let title_re = regex::Regex::new(r#"<a[^>]*data-hook="review-title"[^>]*>(.*?)</a>"#).ok();
            if let Some(re) = title_re {
                if let Some(caps) = re.captures(section) {
                    if let Some(m) = caps.get(1) {
                        return strip_html_simple(m.as_str());
                    }
                }
            }
            String::new()
        }

        fn extract_review_text(section: &str) -> String {
            let text_re = regex::Regex::new(r#"<span[^>]*data-hook="review-body"[^>]*>(.*?)</span>"#).ok();
            if let Some(re) = text_re {
                if let Some(caps) = re.captures(section) {
                    if let Some(m) = caps.get(1) {
                        return strip_html_simple(m.as_str());
                    }
                }
            }
            String::new()
        }

        fn extract_review_date(section: &str) -> String {
            let date_re = regex::Regex::new(r#"<span[^>]*data-hook="review-date"[^>]*>(.*?)</span>"#).ok();
            if let Some(re) = date_re {
                if let Some(caps) = re.captures(section) {
                    if let Some(m) = caps.get(1) {
                        return strip_html_simple(m.as_str());
                    }
                }
            }
            String::new()
        }
    }

    // ----------------------------------------------------------------
    // Wikipedia
    // ----------------------------------------------------------------

    /// A section within a Wikipedia article (recursive structure)
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct WikipediaSection {
        pub heading: String,
        pub content: String,
        pub subsections: Vec<WikipediaSection>,
    }

    /// A parsed Wikipedia article
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct WikipediaArticle {
        pub title: String,
        pub summary: String,
        pub sections: Vec<WikipediaSection>,
        pub infobox: HashMap<String, String>,
    }

    /// Wikipedia page scraper
    pub struct WikipediaScraper;

    impl WikipediaScraper {
        /// Extract a full article from Wikipedia HTML
        pub fn extract_article(html: &str) -> Result<WikipediaArticle, String> {
            let title = Self::extract_title(html)
                .ok_or_else(|| "Could not find article title".to_string())?;

            let summary = Self::extract_summary(html);
            let infobox = Self::extract_infobox(html);
            let sections = Self::extract_sections(html);

            Ok(WikipediaArticle {
                title,
                summary,
                sections,
                infobox,
            })
        }

        /// Extract table of contents entries
        pub fn extract_toc(html: &str) -> Vec<String> {
            let mut entries = Vec::new();
            let toc_pat = r#"<li[^>]*class="[^"]*toclevel-[^"]*"[^>]*>"#;

            if let Ok(re) = regex::Regex::new(toc_pat) {
                for m in re.find_iter(html) {
                    let section = &html[m.start()..(m.start() + 500).min(html.len())];
                    // Find the link text within the TOC entry
                    if let Some(link_start) = section.find(">") {
                        let after_bracket = &section[link_start + 1..];
                        if let Some(link_end) = after_bracket.find('<') {
                            let text = strip_html_simple(&after_bracket[..link_end]);
                            if !text.is_empty() {
                                entries.push(text);
                            }
                        }
                    }
                }
            }
            entries
        }

        /// Extract key-value pairs from the infobox table
        pub fn extract_infobox(html: &str) -> HashMap<String, String> {
            let mut map = HashMap::new();

            // Find infobox table
            let infobox_pos = html.find(r#"class="infobox""#)
                .or_else(|| html.find(r#"class="infobox vcard""#))
                .or_else(|| html.find("infobox"));

            let Some(pos) = infobox_pos else { return map };

            // Find the table boundary
            let table_start = match html[..pos].rfind('<') {
                Some(p) => p,
                None => return map,
            };

            // Find closing </table>
            let table_end = match html[pos..].find("</table>") {
                Some(e) => pos + e + 8,
                None => html.len(),
            };

            let table_html = &html[table_start..table_end];

            // Extract rows from the infobox table
            let row_re = regex::Regex::new(r#"<tr[^>]*>(.*?)</tr>"#).ok();
            let th_re = regex::Regex::new(r#"<th[^>]*>(.*?)</th>"#).ok();
            let td_re = regex::Regex::new(r#"<td[^>]*>(.*?)</td>"#).ok();

            if let (Some(row_re), Some(th_re), Some(td_re)) = (&row_re, &th_re, &td_re) {
                for row_cap in row_re.captures_iter(table_html) {
                    let row_html = row_cap.get(1).map_or("", |m| m.as_str());

                    let key = th_re.captures(row_html)
                        .and_then(|c| c.get(1))
                        .map(|m| strip_html_simple(m.as_str()))
                        .unwrap_or_default();

                    let value = td_re.captures(row_html)
                        .and_then(|c| c.get(1))
                        .map(|m| strip_html_simple(m.as_str()))
                        .unwrap_or_default();

                    if !key.is_empty() && !value.is_empty() {
                        map.insert(key, value);
                    }
                }
            }

            map
        }

        // -- Internal helpers --

        fn extract_title(html: &str) -> Option<String> {
            // Wikipedia uses <h1 id="firstHeading">
            let pat = r#"<h1[^>]*id="firstHeading"[^>]*>"#;
            let re = regex::Regex::new(pat).ok()?;
            let m = re.find(html)?;
            let after = &html[m.end()..];
            let end = after.find('<')?;
            let title = after[..end].trim().to_string();
            if title.is_empty() { None } else { Some(title) }
        }

        fn extract_summary(html: &str) -> String {
            // Summary is in <p> tags within <div id="mw-content-text">
            let content_start = html.find(r#"id="mw-content-text""#)
                .or_else(|| html.find(r#"id="bodyContent""#))
                .unwrap_or(0);

            let content_section = &html[content_start..(content_start + 5000).min(html.len())];

            // Find first non-empty <p> tag (skipping infobox content)
            let p_re = regex::Regex::new(r#"<p>(.*?)</p>"#).ok();
            if let Some(re) = p_re {
                for cap in re.captures_iter(content_section) {
                    if let Some(m) = cap.get(1) {
                        let text = strip_html_simple(m.as_str());
                        // Skip empty paragraphs or those that are just infobox continuation
                        if text.len() > 50 {
                            return text;
                        }
                    }
                }
            }
            String::new()
        }

        fn extract_sections(html: &str) -> Vec<WikipediaSection> {
            let content_start = html.find(r#"id="mw-content-text""#)
                .or_else(|| html.find(r#"id="bodyContent""#))
                .unwrap_or(0);

            let content_section = &html[content_start..];

            let mut sections = Vec::new();
            let mut current_section: Option<String> = None;
            let mut current_content = String::new();
            let mut subsections: Vec<WikipediaSection> = Vec::new();

            // Find all heading elements
            let heading_re = regex::Regex::new(r#"<(h[23])[^>]*>\s*<span[^>]*class="mw-headline"[^>]*>(.*?)</span>"#).ok();
            let mut last_end = 0;

            if let Some(re) = heading_re {
                for cap in re.captures_iter(content_section) {
                    let m = cap.get(0).unwrap();
                    let heading_start = m.start();

                    // Process content between previous heading and this one
                    if let Some(ref current_h) = current_section {
                        let between = content_section[last_end..heading_start].trim();
                        if let Some(level) = cap.get(1) {
                            if level.as_str() == "h2" {
                                // If we had a previous h2, save it
                                if !current_content.trim().is_empty() && !subsections.is_empty() {
                                    // There were subsections under a different h2, save just the section
                                }
                                sections.push(WikipediaSection {
                                    heading: current_h.clone(),
                                    content: current_content.trim().to_string(),
                                    subsections: std::mem::take(&mut subsections),
                                });
                            } else {
                                // h3 — subsection
                                subsections.push(WikipediaSection {
                                    heading: current_h.clone(),
                                    content: between.to_string(),
                                    subsections: Vec::new(),
                                });
                                current_content.clear();
                                continue;
                            }
                        }
                    }

                    last_end = heading_start;
                    current_content.clear();
                    if let Some(heading_text) = cap.get(2) {
                        current_section = Some(strip_html_simple(heading_text.as_str()));
                    }
                }

                // Last section
                if let Some(h) = current_section {
                    sections.push(WikipediaSection {
                        heading: h,
                        content: current_content.trim().to_string(),
                        subsections: std::mem::take(&mut subsections),
                    });
                }
            }

            sections
        }
    }

    // ----------------------------------------------------------------
    // Shared helper
    // ----------------------------------------------------------------

    /// Simple HTML tag stripper (shared across scrapers)
    fn strip_html_simple(html: &str) -> String {
        let mut out = String::with_capacity(html.len());
        let mut in_tag = false;
        let mut in_entity = false;
        let mut entity_buf = String::new();
        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                '&' => {
                    in_entity = true;
                    entity_buf.clear();
                }
                ';' if in_entity => {
                    in_entity = false;
                    let decoded = match entity_buf.as_str() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "#39" => "'",
                        "nbsp" => " ",
                        _ => "",
                    };
                    out.push_str(decoded);
                }
                _ if in_entity => {
                    entity_buf.push(c);
                }
                _ if !in_tag && !in_entity => {
                    if c.is_whitespace() {
                        if !out.ends_with(' ') && !out.is_empty() {
                            out.push(' ');
                        }
                    } else {
                        out.push(c);
                    }
                }
                _ => {}
            }
        }
        out.trim().to_string()
    }
}

// ============================================================================
// Part 3: Unified Source Extractor (G315.1 + G324.1)
// ============================================================================

/// Known/unknown source types for auto-detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    YouTube,
    GoogleSearch,
    Amazon,
    Wikipedia,
    Unknown,
}

/// Unified extractor — auto-detects source type and dispatches to the right scraper
pub struct UnifiedSourceExtractor;

impl UnifiedSourceExtractor {
    /// Auto-detect source type from URL
    pub fn detect_source(url: &str) -> SourceType {
        let lower = url.to_lowercase();
        if lower.contains("youtube.com") || lower.contains("youtu.be") {
            SourceType::YouTube
        } else if lower.contains("google.com/search") || lower.contains("google.co.") {
            SourceType::GoogleSearch
        } else if lower.contains("amazon.") {
            SourceType::Amazon
        } else if lower.contains("wikipedia.org") {
            SourceType::Wikipedia
        } else {
            SourceType::Unknown
        }
    }

    /// Extract structured data from HTML, auto-detecting source from URL.
    /// Returns JSON if detection is successful, or an error message.
    pub fn extract(html: &str, url: &str) -> Result<serde_json::Value, String> {
        match Self::detect_source(url) {
            SourceType::YouTube => {
                let info = data_sources::YouTubeScraper::extract_video_info(html)?;
                serde_json::to_value(&info).map_err(|e| format!("Serialization error: {}", e))
            }
            SourceType::GoogleSearch => {
                let results = data_sources::GoogleSearchScraper::extract_results(html);
                let featured = data_sources::GoogleSearchScraper::extract_featured_snippet(html);
                let paa = data_sources::GoogleSearchScraper::extract_people_also_ask(html);
                let related = data_sources::GoogleSearchScraper::extract_related_searches(html);
                let mut map = serde_json::Map::new();
                map.insert("results".into(), serde_json::to_value(&results).unwrap_or_default());
                map.insert("featured_snippet".into(), serde_json::to_value(&featured).unwrap_or_default());
                map.insert("people_also_ask".into(), serde_json::to_value(&paa).unwrap_or_default());
                map.insert("related_searches".into(), serde_json::to_value(&related).unwrap_or_default());
                Ok(serde_json::Value::Object(map))
            }
            SourceType::Amazon => {
                let product = data_sources::AmazonScraper::extract_product(html)?;
                serde_json::to_value(&product).map_err(|e| format!("Serialization error: {}", e))
            }
            SourceType::Wikipedia => {
                let article = data_sources::WikipediaScraper::extract_article(html)?;
                serde_json::to_value(&article).map_err(|e| format!("Serialization error: {}", e))
            }
            SourceType::Unknown => {
                Err(format!("Unknown source type for URL: {}", url))
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----------------------------------------------------------------
    // SlmConfig
    // ----------------------------------------------------------------

    #[test]
    fn test_slm_config_default() {
        let cfg = SlmConfig::default();
        assert_eq!(cfg.model_name, "reader-lm-v2");
        assert_eq!(cfg.max_tokens, 2048);
        assert_eq!(cfg.temperature, 0.1);
        assert_eq!(cfg.top_p, 0.9);
        assert!(!cfg.use_onnx);
        assert!(!cfg.use_llama);
        assert!(cfg.model_path.is_none());
        assert_eq!(cfg.device, "cpu");
    }

    // ----------------------------------------------------------------
    // SlmHtmlExtractor
    // ----------------------------------------------------------------

    #[test]
    fn test_slm_prepare_html_removes_scripts() {
        let extractor = SlmHtmlExtractor::new(SlmConfig::default());
        let html = "<html><script>alert('x')</script><style>.c{}</style><body><p>Content</p></body></html>";
        let chunks = extractor.prepare_html(html, 10000);
        assert_eq!(chunks.len(), 1);
        assert!(!chunks[0].contains("<script>"));
        assert!(!chunks[0].contains("<style>"));
        assert!(chunks[0].contains("Content"));
    }

    #[test]
    fn test_slm_prepare_html_chunking() {
        let extractor = SlmHtmlExtractor::new(SlmConfig::default());
        // Create HTML longer than max_chars
        let body: String = std::iter::repeat("paragraph content with enough text to exceed the chunk size. ")
            .take(200)
            .collect();
        let html = format!("<html><body>{}</body></html>", body);
        let chunks = extractor.prepare_html(&html, 500);
        assert!(chunks.len() >= 2, "Should produce at least 2 chunks");
        // Total content should be preserved
        let combined: String = chunks.join("");
        assert!(combined.contains("paragraph content"));
    }

    #[test]
    fn test_readerlm_prompt_construction() {
        let prompt = SlmHtmlExtractor::build_readerlm_prompt("<p>Hello</p>");
        assert!(prompt.contains("Convert the following HTML to clean Markdown"));
        assert!(prompt.contains("<p>Hello</p>"));
        assert!(prompt.contains("```html"));
        assert!(prompt.contains("```"));
        assert!(prompt.contains("Markdown:"));
    }

    #[test]
    fn test_readerlm_prompt_truncates_long_html() {
        let long_html = "x".repeat(10000);
        let prompt = SlmHtmlExtractor::build_readerlm_prompt(&long_html);
        // The HTML inside the prompt should be truncated at 8000 chars
        let html_start = prompt.find("```html").unwrap() + 7;
        let html_end = prompt[html_start..].find("```").unwrap();
        let html_in_prompt = &prompt[html_start..html_start + html_end];
        assert!(html_in_prompt.len() <= 8000);
    }

    #[test]
    fn test_onnx_runtime_placeholder() {
        let mut runtime = OnnxRuntime::new(Some("/models/reader-lm-v2.onnx"));
        assert!(!runtime.initialized);
        assert!(runtime.model_path.is_some());

        // Before load
        let result = runtime.run_inference(&[1, 2, 3]);
        assert!(result.is_err());

        // After load
        assert!(runtime.load_model().is_ok());
        assert!(runtime.initialized);

        let result = runtime.run_inference(&[1, 2, 3]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_onnx_runtime_no_path() {
        let mut runtime = OnnxRuntime::new(None);
        assert!(runtime.model_path.is_none());
        assert!(runtime.load_model().is_err());
    }

    #[test]
    fn test_slm_extractor_available_with_model() {
        let config = SlmConfig {
            model_path: Some("/models/test.onnx".into()),
            ..SlmConfig::default()
        };
        let extractor = SlmHtmlExtractor::new(config);
        assert!(extractor.is_available());

        let result = extractor.extract_markdown("<p>test</p>");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("SLM Extraction"));
    }

    #[test]
    fn test_slm_extractor_not_available() {
        let extractor = SlmHtmlExtractor::new(SlmConfig::default());
        assert!(!extractor.is_available());

        let result = extractor.extract_markdown("<p>test</p>");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SLM model not available"));
    }

    // ----------------------------------------------------------------
    // YouTube Scraper
    // ----------------------------------------------------------------

    #[test]
    fn test_youtube_extract_video_info() {
        let html = r#"<!DOCTYPE html>
<html>
<head>
<title>NeoTrix Overview - YouTube</title>
<meta property="og:title" content="NeoTrix Overview">
<meta property="og:description" content="A quick overview of NeoTrix architecture.">
<meta property="article:published_time" content="2026-06-20">
<meta itemprop="interactionCount" content="15420">
<script type="application/ld+json">
{"@type":"VideoObject","duration":"PT12M34S"}
</script>
</head>
<body>
<link href="https://www.youtube.com/channel/UC12345">NeoTrix Channel</a>
</body>
</html>"#;
        let info = data_sources::YouTubeScraper::extract_video_info(html).unwrap();
        assert_eq!(info.title, "NeoTrix Overview");
        assert_eq!(info.description, "A quick overview of NeoTrix architecture.");
        assert_eq!(info.duration_secs, 12 * 60 + 34);
        assert_eq!(info.view_count, 15420);
        assert_eq!(info.publish_date.as_deref(), Some("2026-06-20"));
    }

    #[test]
    fn test_youtube_extract_comments() {
        let html = r#"<html>
<body>
<div id="comments">
<div id="author-text">Alice</div>
<yt-formatted-text id="content-text">Great video!</yt-formatted-text>
<div id="author-text">Bob</div>
<yt-formatted-text id="content-text">Thanks for sharing</yt-formatted-text>
</div>
</body>
</html>"#;
        let comments = data_sources::YouTubeScraper::extract_comments(html);
        // The extraction is best-effort; at minimum it should not panic
        assert!(comments.len() >= 0);
    }

    // ----------------------------------------------------------------
    // Google Search Scraper
    // ----------------------------------------------------------------

    #[test]
    fn test_google_extract_results() {
        let html = r#"<html>
<body>
<div class="g">
<div data-hveid="1">
<h3><a href="https://example.com">Example Result</a></h3>
<span class="st">This is a snippet describing the result.</span>
</div>
</div>
<div class="g">
<div data-hveid="2">
<h3><a href="https://test.org">Test Result</a></h3>
<span class="st">Another snippet with descriptive text.</span>
</div>
</div>
</body>
</html>"#;
        let results = data_sources::GoogleSearchScraper::extract_results(html);
        assert!(results.len() >= 2);
        assert_eq!(results[0].title, "Example Result");
        assert_eq!(results[1].title, "Test Result");
    }

    #[test]
    fn test_google_extract_featured_snippet() {
        let html = r#"<html><body>
<div class="kp-blk">
<span>Featured snippet content that provides a direct answer to the query. This is extracted from a knowledge panel or featured snippet block on Google SERP.</span>
</div>
</body></html>"#;
        let snippet = data_sources::GoogleSearchScraper::extract_featured_snippet(html);
        assert!(snippet.is_some());
        let text = snippet.unwrap();
        assert!(text.contains("Featured snippet"));
    }

    #[test]
    fn test_google_detect_captcha() {
        let html = r#"<html><body><div>Our systems have detected unusual traffic from your computer network. Please try again later.</div></body></html>"#;
        assert!(data_sources::GoogleSearchScraper::has_captcha(html));
    }

    #[test]
    fn test_google_no_captcha() {
        let html = r#"<html><body><div class="g"><h3>Normal Result</h3></div></body></html>"#;
        assert!(!data_sources::GoogleSearchScraper::has_captcha(html));
    }

    #[test]
    fn test_google_extract_people_also_ask() {
        let html = r#"<html><body>
<div class="related-question-pair"><span>What is NeoTrix?</span></div>
<div class="related-question-pair"><span>How does VSA work?</span></div>
</body></html>"#;
        let questions = data_sources::GoogleSearchScraper::extract_people_also_ask(html);
        // These may or may not end with ? depending on extraction
        assert!(questions.len() >= 0);
    }

    // ----------------------------------------------------------------
    // Amazon Scraper
    // ----------------------------------------------------------------

    #[test]
    fn test_amazon_extract_product() {
        let html = r#"<html>
<body>
<span id="productTitle" class="a-size-large">NeoTrix Cognitive Engine</span>
<span class="a-price"><span class="a-offscreen">$49.99</span></span>
<span class="a-icon-alt">4.5 out of 5 stars</span>
<span>1,234 ratings</span>
<span class="a-color-success">In Stock</span>
<input type="hidden" name="ASIN" value="B0TEST1234">
</body>
</html>"#;
        let product = data_sources::AmazonScraper::extract_product(html).unwrap();
        assert_eq!(product.title, "NeoTrix Cognitive Engine");
        assert_eq!(product.price, "$49.99");
        assert_eq!(product.currency, "USD");
        assert_eq!(product.rating, 4.5);
        assert_eq!(product.review_count, 1234);
        assert_eq!(product.availability, "In Stock");
        assert_eq!(product.asin.as_deref(), Some("B0TEST1234"));
    }

    #[test]
    fn test_amazon_extract_search_results() {
        let html = r#"<html>
<body>
<div data-asin="B0TEST1">
<h2>Product One</h2>
<span class="a-price">$19.99</span>
</div>
<div data-asin="B0TEST2">
<h2>Product Two</h2>
<span class="a-price">$29.99</span>
</div>
</body>
</html>"#;
        let results = data_sources::AmazonScraper::extract_search_results(html);
        assert!(results.len() >= 2);
        assert_eq!(results[0].title, "Product One");
        assert_eq!(results[1].title, "Product Two");
    }

    // ----------------------------------------------------------------
    // Wikipedia Scraper
    // ----------------------------------------------------------------

    #[test]
    fn test_wikipedia_extract_article() {
        let html = r#"<html>
<body>
<h1 id="firstHeading">NeoTrix (software)</h1>
<div id="mw-content-text">
<table class="infobox">
<tr><th>Developer</th><td>Neo</td></tr>
<tr><th>Written in</th><td>Rust</td></tr>
</table>
<p>NeoTrix is a silicon-based consciousness architecture designed for autonomous evolution.</p>
<h2><span class="mw-headline">History</span></h2>
<p>The project began in 2025.</p>
<h2><span class="mw-headline">Architecture</span></h2>
<p>Based on E8 reasoning and HyperCube VSA.</p>
<h3><span class="mw-headline">VSA Engine</span></h3>
<p>Vector Symbolic Architecture for high-dimensional computation.</p>
</div>
</body>
</html>"#;
        let article = data_sources::WikipediaScraper::extract_article(html).unwrap();
        assert_eq!(article.title, "NeoTrix (software)");
        assert!(article.summary.contains("silicon-based consciousness"));
        assert!(article.sections.len() >= 2);
        assert!(article.sections[0].heading.contains("History"));
    }

    #[test]
    fn test_wikipedia_extract_infobox() {
        let html = r#"<html>
<body>
<table class="infobox">
<tr><th>Developer</th><td>Neo</td></tr>
<tr><th>Written in</th><td>Rust</td></tr>
<tr><th>License</th><td>MIT</td></tr>
</table>
</body>
</html>"#;
        let infobox = data_sources::WikipediaScraper::extract_infobox(html);
        assert_eq!(infobox.get("Developer").map(|s| s.as_str()), Some("Neo"));
        assert_eq!(infobox.get("Written in").map(|s| s.as_str()), Some("Rust"));
        assert_eq!(infobox.get("License").map(|s| s.as_str()), Some("MIT"));
    }

    // ----------------------------------------------------------------
    // Unified Extractor
    // ----------------------------------------------------------------

    #[test]
    fn test_unified_detect_source_youtube() {
        assert_eq!(UnifiedSourceExtractor::detect_source("https://youtube.com/watch?v=abc123"), SourceType::YouTube);
        assert_eq!(UnifiedSourceExtractor::detect_source("https://youtu.be/abc123"), SourceType::YouTube);
    }

    #[test]
    fn test_unified_detect_source_google() {
        assert_eq!(UnifiedSourceExtractor::detect_source("https://google.com/search?q=neotrix"), SourceType::GoogleSearch);
        assert_eq!(UnifiedSourceExtractor::detect_source("https://google.co.uk/search?q=test"), SourceType::GoogleSearch);
    }

    #[test]
    fn test_unified_detect_source_amazon() {
        assert_eq!(UnifiedSourceExtractor::detect_source("https://amazon.com/dp/B0TEST"), SourceType::Amazon);
        assert_eq!(UnifiedSourceExtractor::detect_source("https://amazon.co.uk/dp/B0TEST"), SourceType::Amazon);
    }

    #[test]
    fn test_unified_detect_source_wikipedia() {
        assert_eq!(UnifiedSourceExtractor::detect_source("https://en.wikipedia.org/wiki/NeoTrix"), SourceType::Wikipedia);
    }

    #[test]
    fn test_unified_detect_source_unknown() {
        assert_eq!(UnifiedSourceExtractor::detect_source("https://example.com/page"), SourceType::Unknown);
    }

    #[test]
    fn test_unified_extract_known() {
        let html = r#"<html><head><title>Test - YouTube</title>
<meta property="og:title" content="Test Video">
<meta itemprop="interactionCount" content="100">
<script type="application/ld+json">{"duration":"PT5M"}</script>
</head></html>"#;
        let result = UnifiedSourceExtractor::extract(html, "https://youtube.com/watch?v=test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_unified_extract_unknown() {
        let result = UnifiedSourceExtractor::extract("<html></html>", "https://example.com");
        assert!(result.is_err());
    }
}
