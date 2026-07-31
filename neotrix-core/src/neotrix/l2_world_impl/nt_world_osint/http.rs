use std::collections::HashMap;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{OsintConfig, OsintTarget};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpEndpoint {
    pub url: String,
    pub status: u16,
    pub title: Option<String>,
    pub server: Option<String>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub tech_stack: Vec<String>,
    pub headers: HashMap<String, String>,
    pub response_time_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HttpFindings {
    pub endpoints: Vec<HttpEndpoint>,
}

impl std::fmt::Display for HttpFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── HTTP Probing ──")?;
        writeln!(f, "    Endpoints: {}", self.endpoints.len())?;
        for ep in &self.endpoints {
            writeln!(f, "    {} {} [{}ms]", ep.status, ep.url, ep.response_time_ms)?;
            if let Some(ref title) = ep.title {
                writeln!(f, "      Title: {}", title)?;
            }
            if let Some(ref srv) = ep.server {
                writeln!(f, "      Server: {}", srv)?;
            }
            if !ep.tech_stack.is_empty() {
                writeln!(f, "      Tech: {}", ep.tech_stack.join(", "))?;
            }
        }
        Ok(())
    }
}

fn detect_tech(headers: &HashMap<String, String>, title: &Option<String>, body_preview: &str) -> Vec<String> {
    let mut tech = Vec::new();
    for (key, val) in headers {
        let kl = key.to_lowercase();
        let vl = val.to_lowercase();
        match kl.as_str() {
            "server" => {
                if vl.contains("nginx") { tech.push("Nginx".to_string()); }
                if vl.contains("apache") { tech.push("Apache".to_string()); }
                if vl.contains("cloudflare") { tech.push("Cloudflare".to_string()); }
                if vl.contains("caddy") { tech.push("Caddy".to_string()); }
                if vl.contains("openresty") { tech.push("OpenResty".to_string()); }
                if vl.contains("iis") { tech.push("IIS".to_string()); }
                if vl.contains("gunicorn") { tech.push("Gunicorn".to_string()); }
                if vl.contains("uwsgi") { tech.push("uWSGI".to_string()); }
                if vl.contains("express") { tech.push("Express".to_string()); }
            }
            "x-powered-by" => {
                if vl.contains("php") { tech.push("PHP".to_string()); }
                if vl.contains("asp.net") { tech.push("ASP.NET".to_string()); }
                if vl.contains("express") { tech.push("Express".to_string()); }
                if vl.contains("rails") { tech.push("Ruby on Rails".to_string()); }
                if vl.contains("django") { tech.push("Django".to_string()); }
                if vl.contains("flask") { tech.push("Flask".to_string()); }
                if vl.contains("next.js") || vl.contains("nextjs") { tech.push("Next.js".to_string()); }
            }
            "x-generator" | "generator" => {
                if vl.contains("wordpress") { tech.push("WordPress".to_string()); }
                if vl.contains("drupal") { tech.push("Drupal".to_string()); }
                if vl.contains("joomla") { tech.push("Joomla".to_string()); }
                if vl.contains("hugo") { tech.push("Hugo".to_string()); }
                if vl.contains("jekyll") { tech.push("Jekyll".to_string()); }
                if vl.contains("ghost") { tech.push("Ghost".to_string()); }
                if vl.contains("shopify") { tech.push("Shopify".to_string()); }
                if vl.contains("wix") { tech.push("Wix".to_string()); }
                if vl.contains("squarespace") { tech.push("Squarespace".to_string()); }
            }
            "cf-ray" => { tech.push("Cloudflare".to_string()); }
            "x-served-by" => { tech.push(vl.clone()); }
            "x-amzn-requestid" | "x-amz-cf-id" => { tech.push("AWS CloudFront".to_string()); }
            "set-cookie" => {
                if vl.contains("laravel_session") { tech.push("Laravel".to_string()); }
                if vl.contains("ci_session") { tech.push("CodeIgniter".to_string()); }
                if vl.contains("django_language") || vl.contains("django_session") { tech.push("Django".to_string()); }
                if vl.contains("rails_session") || vl.contains("_rails_session") { tech.push("Ruby on Rails".to_string()); }
                if vl.contains("symfony") { tech.push("Symfony".to_string()); }
                if vl.contains("wordpress_logged_in") || vl.contains("wp_") { tech.push("WordPress".to_string()); }
            }
            _ => {}
        }
    }
    let bp = body_preview.to_lowercase();
    if let Some(ref t) = title {
        let tl = t.to_lowercase();
        if tl.contains("wordpress") || tl.contains("wp-") { tech.push("WordPress".to_string()); }
        if tl.contains("shopify") { tech.push("Shopify".to_string()); }
    }
    if bp.contains("wp-content") || bp.contains("wp-json") { tech.push("WordPress".to_string()); }
    if bp.contains("react") || bp.contains("react-dom") { tech.push("React".to_string()); }
    if bp.contains("vue") || bp.contains("vuejs") { tech.push("Vue.js".to_string()); }
    if bp.contains("angular") || bp.contains("ng-") { tech.push("Angular".to_string()); }
    if bp.contains("jquery") { tech.push("jQuery".to_string()); }
    if bp.contains("next.js") || bp.contains("__next") { tech.push("Next.js".to_string()); }
    if bp.contains("api.json") || bp.contains("graphql") { tech.push("GraphQL".to_string()); }
    if bp.contains("turbolinks") || bp.contains("turbo-frame") { tech.push("Hotwire/Turbo".to_string()); }
    if bp.contains("alpinejs") || bp.contains("alpine.js") { tech.push("Alpine.js".to_string()); }
    if bp.contains("tailwind") { tech.push("Tailwind CSS".to_string()); }
    if bp.contains("bootstrap") { tech.push("Bootstrap".to_string()); }
    tech.sort();
    tech.dedup();
    tech
}

fn extract_title(body: &str) -> Option<String> {
    let body_lower = body.to_lowercase();
    let markers = ["<title", "<TITLE"];
    for marker in &markers {
        if let Some(start) = body_lower.find(marker) {
            let after_start = start + marker.len();
            let remaining = &body[after_start..];
            let close = remaining.find('>')?;
            let content_start = after_start + close + 1;
            let content_remaining = &body[content_start..];
            let end = content_remaining.find("</title").or_else(|| content_remaining.find("</TITLE"))?;
            let t = content_remaining[..end].trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

async fn probe_url(url: &str, client: &Client) -> Result<HttpEndpoint, String> {
    let start = std::time::Instant::now();
    let resp = client.get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("{url}: {e}"))?;

    let status = resp.status().as_u16();
    let response_time_ms = start.elapsed().as_millis() as u64;
    let headers: HashMap<String, String> = resp.headers().iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let content_type = headers.get("content-type").cloned();
    let content_length = headers.get("content-length").and_then(|v| v.parse::<u64>().ok());
    let server = headers.get("server").cloned();
    let body_bytes = resp.bytes().await.unwrap_or_default();
    let body = String::from_utf8_lossy(&body_bytes);
    let body_preview = if body.len() > 50000 { &body[..50000] } else { &body };
    let title = extract_title(body_preview);
    let tech_stack = detect_tech(&headers, &title, body_preview);

    Ok(HttpEndpoint {
        url: url.to_string(),
        status,
        title,
        server,
        content_type,
        content_length,
        tech_stack,
        headers,
        response_time_ms,
    })
}

pub async fn investigate(target: &OsintTarget, client: &Client, _config: &OsintConfig) -> Result<HttpFindings, String> {
    let domain = target.domain.as_ref().ok_or("no domain specified")?;
    let mut findings = HttpFindings::default();

    let schemes = ["https", "http"];
    let prefixes = ["", "www."];
    let paths = ["", "/robots.txt", "/sitemap.xml", "/.well-known/security.txt",
                  "/favicon.ico", "/api/", "/health", "/healthz", "/status",
                  "/.env", "/admin", "/login", "/wp-admin"];

    for scheme in &schemes {
        for prefix in &prefixes {
            let base_url = format!("{scheme}://{prefix}{domain}");
            for path in &paths {
                let url = format!("{base_url}{path}");
                if let Ok(ep) = probe_url(&url, client).await {
                    let already_exists = findings.endpoints.iter().any(|e| e.url == ep.url);
                    if !already_exists && (ep.status < 400 || ep.status == 403 || ep.status == 401) {
                        findings.endpoints.push(ep);
                    }
                }
            }
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_simple() {
        let html = "<html><head><title>Test Page</title></head></html>";
        assert_eq!(extract_title(html), Some("Test Page".to_string()));
    }

    #[test]
    fn test_extract_title_no_title() {
        let html = "<html><head></head></html>";
        assert_eq!(extract_title(html), None);
    }

    #[test]
    fn test_extract_title_case_insensitive() {
        let html = "<html><head><TITLE>Case Test</TITLE></head></html>";
        assert_eq!(extract_title(html), Some("Case Test".to_string()));
    }

    #[test]
    fn test_detect_tech_nginx() {
        let mut h = HashMap::new();
        h.insert("server".to_string(), "nginx/1.24.0".to_string());
        let tech = detect_tech(&h, &None, "");
        assert!(tech.contains(&"Nginx".to_string()));
    }

    #[test]
    fn test_detect_tech_cloudflare() {
        let mut h = HashMap::new();
        h.insert("server".to_string(), "cloudflare".to_string());
        h.insert("cf-ray".to_string(), "abc123".to_string());
        let tech = detect_tech(&h, &None, "");
        assert!(tech.contains(&"Cloudflare".to_string()));
    }

    #[test]
    fn test_detect_tech_wordpress_body() {
        let h = HashMap::new();
        let body = "this site uses wp-content and wp-json";
        let tech = detect_tech(&h, &Some("My Site".to_string()), body);
        assert!(tech.contains(&"WordPress".to_string()));
    }

    #[test]
    fn test_detect_tech_react_body() {
        let h = HashMap::new();
        let body = "React testing with react-dom";
        let tech = detect_tech(&h, &None, body);
        assert!(tech.contains(&"React".to_string()));
    }

    #[test]
    fn test_http_findings_display() {
        let f = HttpFindings::default();
        let s = format!("{f}");
        assert!(s.contains("HTTP Probing"));
    }
}
