use base64::Engine;

/// Extract port from a proxy URL
#[allow(dead_code)]
pub(crate) fn extract_port(url: &str) -> Option<u16> {
    let url = url.trim();
    if let Some(pos) = url.rfind(':') {
        let rest = &url[pos + 1..];
        if let Some(at_pos) = url.rfind('@') {
            if pos < at_pos {
                let after_at = &url[at_pos + 1..];
                if let Some(colon) = after_at.rfind(':') {
                    return after_at[colon + 1..].parse::<u16>().ok();
                }
                return None;
            }
        }
        let port_str = if let Some(slash) = rest.find('/') {
            &rest[..slash]
        } else {
            rest
        };
        port_str.parse::<u16>().ok()
    } else {
        None
    }
}

#[allow(dead_code)]
pub(crate) fn parse_proxy_url(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.starts_with("ss://") || line.starts_with("ssr://") || line.starts_with("vmess://") || line.starts_with("trojan://") {
        let scheme_end = line.find("://")?;
        let scheme = &line[..scheme_end];
        let content = &line[scheme_end + 3..];
        let tag = match scheme {
            "ss" | "ssr" => {
                let decoded = base64_decode(content).unwrap_or_default();
                if let Some(at) = decoded.rfind('@') {
                    let after_at = &decoded[at + 1..];
                    after_at.split(':').next().unwrap_or("").to_string()
                } else {
                    "unknown".to_string()
                }
            }
            "vmess" => {
                let decoded = base64_decode(content).unwrap_or_default();
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&decoded) {
                    json.get("add").and_then(|v| v.as_str()).unwrap_or("unknown").to_string()
                } else {
                    "unknown".to_string()
                }
            }
            "trojan" => {
                content.split('@').nth(1)
                    .and_then(|s| s.split(':').next())
                    .unwrap_or("unknown")
                    .to_string()
            }
            _ => content.split('@').nth(1)
                .and_then(|s| s.split(':').next())
                .unwrap_or("unknown")
                .to_string(),
        };
        Some((line.to_string(), tag))
    } else if line.starts_with("http://") || line.starts_with("https://") || line.starts_with("socks5://") || line.starts_with("socks4://") {
        let host = line.split("://").nth(1)
            .and_then(|s| s.split(':').next())
            .unwrap_or("unknown");
        Some((line.to_string(), host.to_string()))
    } else {
        None
    }
}

pub fn base64_decode(text: &str) -> Result<String, String> {
    let text = text.trim().replace('\n', "");
    let text = if text.contains('-') || text.contains('_') {
        text.replace('-', "+").replace('_', "/")
    } else {
        text
    };
    let padded = match text.len() % 4 {
        0 => text,
        r => {
            let pad = 4 - r;
            format!("{}{}", text, "=".repeat(pad))
        }
    };
    let bytes = Engine::decode(&base64::engine::general_purpose::STANDARD, padded.as_bytes())
        .map_err(|e| format!("base64 decode error: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("utf8 error: {}", e))
}

#[allow(dead_code)]
pub(crate) async fn check_proxy(url: &str) -> bool {
    if url.starts_with("socks5://") || url.starts_with("socks4://") {
        let addr = url.trim_start_matches("socks5://").trim_start_matches("socks4://");
        tokio::net::TcpStream::connect(addr).await.is_ok()
    } else {
        let addr = url.trim_start_matches("http://").trim_start_matches("https://").trim_start_matches("ss://").trim_start_matches("ssr://");
        tokio::net::TcpStream::connect(addr).await.is_ok()
    }
}

pub const FREE_PROXY_SCRAPERS: &[(&str, &str)] = &[
    ("geonode", "https://proxylist.geonode.com/api/proxy-list?limit=100&page=1&sort_by=lastChecked&sort_type=desc"),
    ("proxy-list", "https://www.proxy-list.download/api/v1/get?type=http"),
    ("proxyscrape", "https://api.proxyscrape.com/v2/?request=displayproxies&protocol=http&timeout=10000&country=all&ssl=all&anonymity=all"),
];

pub const DEFAULT_SUBSCRIPTIONS: &[&str] = &[
    "https://raw.githubusercontent.com/freefq/free/master/v2",
    "https://raw.githubusercontent.com/mahdibland/ShadowsocksAggregator/master/Eternity.txt",
    "https://raw.githubusercontent.com/ssrsub/ssr/master/ss-sub",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http() {
        let r = parse_proxy_url("http://proxy.example.com:8080");
        assert!(r.is_some());
    }

    #[test]
    fn test_parse_ss() {
        let r = parse_proxy_url("ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ@example.com:8443");
        assert!(r.is_some());
    }
}
