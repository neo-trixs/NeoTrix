use std::collections::{HashMap, HashSet};

pub(crate) fn urlencoding(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

pub(crate) fn extract_title(html: &str) -> Option<String> {
    html.find("<title").and_then(|start| {
        let title_start = html[start..].find('>')?;
        let content_start = start + title_start + 1;
        let content_end = html[content_start..].find("</title>")?;
        let raw = &html[content_start..content_start + content_end];
        Some(raw.trim().to_string())
    })
}

pub(crate) fn extract_body_text(html: &str, max_len: usize) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut tag_name = String::new();

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            tag_name.clear();
            continue;
        }
        if ch == '>' {
            in_tag = false;
            let tn = tag_name.to_lowercase();
            if tn == "script" {
                in_script = true;
            }
            if tn == "/script" {
                in_script = false;
            }
            if tn == "style" {
                in_style = true;
            }
            if tn == "/style" {
                in_style = false;
            }
            continue;
        }
        if in_tag {
            tag_name.push(ch);
            continue;
        }
        if in_script || in_style {
            continue;
        }
        if ch.is_alphanumeric() || ch == ' ' {
            text.push(ch);
            if text.len() >= max_len {
                break;
            }
        }
    }
    text.truncate(max_len);
    text.trim().to_string()
}

pub(crate) fn extract_keywords(body: &str, title: &str) -> Vec<String> {
    let mut freq: HashMap<String, usize> = HashMap::new();
    let combined = format!("{} {}", title, body).to_lowercase();

    let stop_words: HashSet<&str> = [
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "by", "with",
        "from", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do",
        "does", "did", "will", "would", "can", "could", "shall", "should", "may", "might", "must",
        "this", "that", "these", "those", "it", "its", "i", "me", "my", "we", "our", "you", "your",
        "he", "him", "his", "she", "her", "they", "them", "their", "what", "which", "who", "whom",
        "when", "where", "why", "how", "all", "each", "every", "both", "few", "more", "most",
        "other", "some", "such", "no", "nor", "not", "only", "own", "same", "so", "than", "too",
        "very", "just", "because", "as", "until", "while", "about", "between", "through", "during",
        "before", "after", "above", "below", "up", "down", "out", "off", "over", "under", "again",
        "further", "then", "once", "here", "there", "http", "https", "www", "com", "org", "net",
        "html", "onion",
    ]
    .into_iter()
    .collect();

    for word in combined.split_whitespace() {
        let clean: String = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_string();
        if clean.len() >= 4 && !stop_words.contains(&clean.as_str()) {
            *freq.entry(clean).or_insert(0) += 1;
        }
    }

    for word in title.to_lowercase().split_whitespace() {
        let clean: String = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_string();
        if clean.len() >= 3 {
            *freq.entry(clean).or_insert(0) += 5;
        }
    }

    let mut words: Vec<(usize, String)> = freq.into_iter().map(|(k, v)| (v, k)).collect();
    words.sort_by(|a, b| b.0.cmp(&a.0));
    words.truncate(20);
    words.into_iter().map(|(_, w)| w).collect()
}

pub(crate) fn extract_onion_links(html: &str, base_url: &str) -> Vec<String> {
    let mut links = Vec::new();
    let lower = html.to_lowercase();
    let mut pos = 0;

    while let Some(href_start) = lower[pos..].find("href=\"") {
        let start = pos + href_start + 6;
        if let Some(end) = lower[start..].find('"') {
            let url = &html[start..start + end];
            if url.contains(".onion") {
                let full = if url.starts_with("http://") || url.starts_with("https://") {
                    url.to_string()
                } else if url.starts_with("//") {
                    format!("http:{}", url)
                } else if url.starts_with('/') {
                    let base = base_url.trim_end_matches('/');
                    format!("{}{}", base, url)
                } else {
                    format!("{}/{}", base_url.trim_end_matches('/'), url)
                };
                links.push(full);
            }
            pos = start + end;
        } else {
            break;
        }
    }

    links.sort();
    links.dedup();
    links
}
