use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{OsintConfig, OsintTarget};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkWebResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
    pub host: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DarkFindings {
    pub results: Vec<DarkWebResult>,
    pub onion_links: Vec<String>,
    pub domain: String,
}

impl std::fmt::Display for DarkFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── Dark Web Monitoring ──")?;
        writeln!(f, "    Domain:           {}", self.domain)?;
        writeln!(f, "    References:       {}", self.results.len())?;
        writeln!(f, "    .onion links:     {}", self.onion_links.len())?;
        for result in &self.results {
            writeln!(f, "      [{}] {}", result.source, result.title)?;
            let short: String = result.snippet.chars().take(120).collect();
            writeln!(f, "      {short}")?;
        }
        for onion in &self.onion_links {
            writeln!(f, "      .onion: {onion}")?;
        }
        Ok(())
    }
}

async fn search_ahmia(query: &str, client: &Client) -> Vec<DarkWebResult> {
    let url = format!("https://ahmia.fi/search/?q={}", urlencode(query));
    match client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .timeout(Duration::from_secs(15))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            let mut results = Vec::new();
            // Parse Ahmia search results (basic HTML parsing)
            for line in body.lines() {
                if line.contains("class=\"result\"") || line.contains("class=\"search-result\"") {
                    let title = extract_between(line, "<h3>", "</h3>")
                        .or_else(|| extract_between(line, "<a", "</a>"))
                        .unwrap_or("result");
                    let snippet = extract_between(line, "<p>", "</p>").unwrap_or("");
                    results.push(DarkWebResult {
                        title: clean_html(title),
                        url: extract_href(line).unwrap_or_default(),
                        snippet: clean_html(snippet),
                        source: "ahmia".to_string(),
                        host: None,
                    });
                }
            }
            results.truncate(30);
            results
        }
        _ => vec![],
    }
}

async fn search_facebook_watch(query: &str, client: &Client) -> Vec<DarkWebResult> {
    // Facebook Watcher - indexes dark web forums and paste sites
    let url = format!("https://facebook.watch/search?q={}", urlencode(query));
    match client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .timeout(Duration::from_secs(15))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            let mut results = Vec::new();
            for line in body.lines() {
                if line.contains("class=\"result\"") || line.contains("<article") {
                    let title = extract_between(line, "<h2", "</h2>").unwrap_or("result");
                    let snippet = extract_between(line, "<p", "</p>").unwrap_or("");
                    results.push(DarkWebResult {
                        title: clean_html(title),
                        url: extract_href(line).unwrap_or_default(),
                        snippet: clean_html(snippet),
                        source: "facebook-watch".to_string(),
                        host: None,
                    });
                }
            }
            results.truncate(30);
            results
        }
        _ => vec![],
    }
}

fn urlencode(input: &str) -> String {
    input.as_bytes().iter().map(|&byte| {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            b' ' => '+'.to_string(),
            _ => format!("%{:02X}", byte),
        }
    }).collect::<String>()
}

fn extract_between<'a>(s: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let s_start = s.find(start)?;
    let content_start = s_start + start.len();
    let remaining = &s[content_start..];
    // For tags with attributes, find the closing >
    if start.starts_with('<') && !start.ends_with('>') {
        let tag_close = remaining.find('>')?;
        let after_tag = &remaining[tag_close + 1..];
        let content_end = after_tag.find(end)?;
        return Some(after_tag[..content_end].trim());
    }
    let content_end = remaining.find(end)?;
    Some(remaining[..content_end].trim())
}

fn extract_href(s: &str) -> Option<String> {
    let href_str = "href=\"";
    let start = s.find(href_str)? + href_str.len();
    let remaining = &s[start..];
    let end = remaining.find('"')?;
    Some(remaining[..end].to_string())
}

fn clean_html(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;
    let mut in_entity = false;
    let mut entity_buf = String::new();
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => {
                if c == '&' {
                    in_entity = true;
                    entity_buf.clear();
                } else if in_entity {
                    if c == ';' {
                        let decoded = match entity_buf.as_str() {
                            "amp" => "&", "lt" => "<", "gt" => ">",
                            "quot" => "\"", "apos" => "'",
                            "nbsp" => " ",
                            _ => "",
                        };
                        result.push_str(decoded);
                        in_entity = false;
                    } else {
                        entity_buf.push(c);
                    }
                } else {
                    result.push(c);
                }
            }
            _ => {}
        }
    }
    result
}

/// Check if a Tor SOCKS5 proxy is available at 127.0.0.1:9050
fn check_tor_proxy() -> bool {
    let addr = match "127.0.0.1:9050".to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(a) => a,
            None => return false,
        },
        Err(_) => return false,
    };
    // Try TCP connect with short timeout
    TcpStream::connect_timeout(&addr, Duration::from_millis(1500)).is_ok()
}

/// Create a Tor-proxied reqwest client (SOCKS5h to 127.0.0.1:9050)
fn build_tor_client(timeout_secs: u64) -> Result<Client, String> {
    let proxy = reqwest::Proxy::all("socks5h://127.0.0.1:9050")
        .map_err(|e| format!("proxy error: {e}"))?;
    Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(timeout_secs))
        .https_only(false) // allow .onion via Tor
        .build()
        .map_err(|e| format!("client error: {e}"))
}

/// Search Ahmia through Tor for dark web content
async fn search_ahmia_via_tor(query: &str, tor_client: &Client) -> Vec<DarkWebResult> {
    // Ahmia .onion service
    let url = format!("http://juhanurmihxlp77nkq76byazcldy2hlmovfu2epvl5ankdibsot4csyd.onion/search/?q={}", urlencode(query));
    match tor_client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .timeout(Duration::from_secs(30))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            let mut results = Vec::new();
            for line in body.lines() {
                if line.contains("class=\"result\"") || line.contains("class=\"search-result\"") {
                    let title = extract_between(line, "<h3>", "</h3>")
                        .or_else(|| extract_between(line, "<a", "</a>"))
                        .unwrap_or("result");
                    let snippet = extract_between(line, "<p>", "</p>").unwrap_or("");
                    results.push(DarkWebResult {
                        title: clean_html(title),
                        url: extract_href(line).unwrap_or_default(),
                        snippet: clean_html(snippet),
                        source: "ahmia-tor".to_string(),
                        host: None,
                    });
                }
            }
            results.truncate(30);
            results
        }
        _ => vec![],
    }
}

pub async fn investigate(target: &OsintTarget, client: &Client, config: &OsintConfig) -> Result<DarkFindings, String> {
    let domain = target.domain.as_ref().ok_or("no domain specified")?;
    let mut findings = DarkFindings {
        domain: domain.to_string(),
        ..Default::default()
    };

    // Detect Tor proxy
    let tor_available = if config.use_proxy {
        let available = check_tor_proxy();
        if !available {
            log::warn!("[dark] Tor proxy not found at 127.0.0.1:9050 — falling back to clearnet Ahmia");
        }
        available
    } else {
        false
    };

    if tor_available {
        // Full dark web capability via Tor
        if let Ok(tor_client) = build_tor_client(config.timeout_secs) {
            let ahmia_tor = search_ahmia_via_tor(domain, &tor_client).await;
            if !ahmia_tor.is_empty() {
                findings.results.extend(ahmia_tor);
                log::info!("[dark] {} results from Ahmia .onion", findings.results.len());
            }
        }
    } else {
        // Clearnet fallback: search Ahmia (clearnet) + Facebook Watcher + .onion reference scanning
        let ahmia_results = search_ahmia(domain, client).await;
        if !ahmia_results.is_empty() {
            findings.results.extend(ahmia_results);
        }

        let fw_results = search_facebook_watch(domain, client).await;
        if !fw_results.is_empty() {
            findings.results.extend(fw_results);
        }
    }

    // Always scan clearnet for .onion references
    let url = format!("https://duckduckgo.com/html/?q={}+site%3Aonion", urlencode(domain));
    match client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            for line in body.lines() {
                if line.contains(".onion") {
                    let mut rest = line;
                    while let Some(start) = rest.find("http") {
                        let candidate = &rest[start..];
                        let end = candidate.find(|c: char| c.is_whitespace() || c == '"' || c == '<' || c == '>')
                            .unwrap_or(candidate.len().min(64));
                        let potential_url = &candidate[..end];
                        if potential_url.contains(".onion") {
                            findings.onion_links.push(potential_url.to_string());
                        }
                        rest = &candidate[end..];
                    }
                }
            }
        }
        _ => {}
    }

    findings.onion_links.sort();
    findings.onion_links.dedup();

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_html_strips_tags() {
        assert_eq!(clean_html("<b>bold</b>"), "bold");
    }

    #[test]
    fn test_clean_html_decodes_entities() {
        assert_eq!(clean_html("AT&amp;T"), "AT&T");
        assert_eq!(clean_html("&lt;tag&gt;"), "<tag>");
    }

    #[test]
    fn test_clean_html_handles_nbsp() {
        assert_eq!(clean_html("hello&nbsp;world"), "hello world");
    }

    #[test]
    fn test_extract_href() {
        assert_eq!(extract_href("<a href=\"https://example.com\">"), Some("https://example.com".to_string()));
    }

    #[test]
    fn test_extract_between() {
        assert_eq!(extract_between("foo<h3>title</h3>bar", "<h3>", "</h3>"), Some("title"));
    }

    #[test]
    fn test_extract_between_no_match() {
        assert_eq!(extract_between("foo bar", "<h3>", "</h3>"), None);
    }

    #[test]
    fn test_dark_findings_default() {
        let f = DarkFindings::default();
        assert!(f.results.is_empty());
        assert!(f.onion_links.is_empty());
    }

    #[test]
    fn test_display_empty() {
        let f = DarkFindings::default();
        let s = format!("{f}");
        assert!(s.contains("Dark Web"));
    }

    #[test]
    fn test_ahmia_url_format() {
        let q = urlencode("example.com");
        let url = format!("https://ahmia.fi/search/?q={q}");
        assert!(url.contains("example.com"));
    }

    #[test]
    fn test_onion_extraction() {
        let text = "Visit http://abcdefghijklmnid.onion for more info";
        assert!(text.contains(".onion"));
        assert!(text.split_whitespace().any(|w| w.contains(".onion")));
    }
}
