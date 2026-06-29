use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::seq::SliceRandom;
use tokio::sync::RwLock;

use super::types::*;

const REQUEST_TIMEOUT_SECS: u64 = 15;

pub struct ProxySourcing {
    source_health: RwLock<HashMap<&'static str, SourceHealth>>,
    client: reqwest::Client,
}

impl Default for ProxySourcing {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxySourcing {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .danger_accept_invalid_certs(true)
            .no_proxy()
            .build()
            .expect("reqwest client build");
        Self {
            source_health: RwLock::new(HashMap::new()),
            client,
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            source_health: RwLock::new(HashMap::new()),
            client,
        }
    }

    pub async fn source_health(&self, name: &str) -> SourceHealth {
        self.source_health
            .read()
            .await
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn all_source_health(&self) -> Vec<(&'static str, SourceHealth)> {
        let health = self.source_health.read().await;
        ALL_FREE_SOURCES
            .iter()
            .map(|s| (s.name, health.get(s.name).cloned().unwrap_or_default()))
            .collect()
    }

    pub async fn fetch_source(&self, source: &ProxySourceDef) -> Result<Vec<RawProxy>, String> {
        {
            let health = self.source_health.read().await;
            if let Some(h) = health.get(source.name) {
                if h.is_on_cooldown() {
                    return Err(format!(
                        "source '{}' on cooldown for {:?}",
                        source.name,
                        h.cooldown_until
                            .map(|t| t.saturating_duration_since(Instant::now()))
                            .unwrap_or(Duration::ZERO)
                    ));
                }
            }
        }

        let resp = match self.client.get(source.url).send().await {
            Ok(r) => r,
            Err(e) => {
                let msg = format!("fetch {}: {}", source.name, e);
                self.record_failure(source.name, msg.clone()).await;
                return Err(msg);
            }
        };

        let status = resp.status();
        if !status.is_success() {
            let msg = format!("HTTP {} from {}", status, source.name);
            self.record_failure(source.name, msg.clone()).await;
            return Err(msg);
        }

        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                let msg = format!("read body {}: {}", source.name, e);
                self.record_failure(source.name, msg.clone()).await;
                return Err(msg);
            }
        };

        let proxies = match source.format {
            SourceFormat::PlainTxt => parse_plain_text(&text, &source.protocols),
            SourceFormat::JsonArray => parse_json_array(&text, &source.protocols),
            SourceFormat::JsonApi => parse_geonode_json(&text),
        };

        self.record_success(source.name).await;
        Ok(proxies)
    }

    pub async fn fetch_all_sources(
        &self,
        sources: &[&ProxySourceDef],
    ) -> Vec<(&'static str, Vec<RawProxy>)> {
        let fetches: Vec<_> = sources
            .iter()
            .map(|source| async move {
                let result = self.fetch_source(source).await;
                (source.name, result)
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(fetches).await;

        let mut output = Vec::new();
        for (name, result) in results {
            match result {
                Ok(proxies) => {
                    log::info!(
                        "[proxy-sourcing] {}: fetched {} proxies",
                        name,
                        proxies.len()
                    );
                    output.push((name, proxies));
                }
                Err(e) => {
                    log::warn!("[proxy-sourcing] {} failed: {}", name, e);
                }
            }
        }
        output
    }

    pub async fn fetch_proxifly_by_protocol(
        &self,
        protocol: ProxyProtocol,
    ) -> Result<Vec<RawProxy>, String> {
        let source = match protocol {
            ProxyProtocol::Http => &PROXIFLY_HTTP,
            ProxyProtocol::Https => &PROXIFLY_HTTPS,
            ProxyProtocol::Socks4 => &PROXIFLY_SOCKS4,
            ProxyProtocol::Socks5 => &PROXIFLY_SOCKS5,
        };
        self.fetch_source(source).await
    }

    pub async fn fetch_proxifly_all(&self) -> Result<Vec<RawProxy>, String> {
        self.fetch_source(&PROXIFLY_ALL).await
    }

    async fn record_success(&self, name: &'static str) {
        let mut health = self.source_health.write().await;
        let entry = health.entry(name).or_default();
        entry.record_success();
    }

    async fn record_failure(&self, name: &'static str, error: String) {
        let mut health = self.source_health.write().await;
        let entry = health.entry(name).or_default();
        entry.record_failure(error);
    }
}

pub fn parse_plain_text(text: &str, protocols: &[ProxyProtocol]) -> Vec<RawProxy> {
    let default_protocol = protocols.first().copied().unwrap_or(ProxyProtocol::Http);
    let mut proxies = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if let Some(proxy) = parse_proxy_line(line, default_protocol) {
            proxies.push(proxy);
        }
    }
    proxies
}

fn parse_proxy_line(line: &str, default_protocol: ProxyProtocol) -> Option<RawProxy> {
    let line = line.trim();

    if let Some(rest) = line.strip_prefix("socks5://") {
        let (ip, port) = split_host_port(rest)?;
        return Some(RawProxy {
            ip,
            port,
            protocol: ProxyProtocol::Socks5,
            source: None,
        });
    }
    if let Some(rest) = line.strip_prefix("socks4://") {
        let (ip, port) = split_host_port(rest)?;
        return Some(RawProxy {
            ip,
            port,
            protocol: ProxyProtocol::Socks4,
            source: None,
        });
    }
    if let Some(rest) = line.strip_prefix("https://") {
        let (ip, port) = split_host_port(rest)?;
        return Some(RawProxy {
            ip,
            port,
            protocol: ProxyProtocol::Https,
            source: None,
        });
    }
    if let Some(rest) = line.strip_prefix("http://") {
        let (ip, port) = split_host_port(rest)?;
        return Some(RawProxy {
            ip,
            port,
            protocol: ProxyProtocol::Http,
            source: None,
        });
    }

    let (ip, port) = split_host_port(line)?;
    Some(RawProxy {
        ip,
        port,
        protocol: default_protocol,
        source: None,
    })
}

fn split_host_port(s: &str) -> Option<(String, u16)> {
    let s = s.trim();
    let colon = s.rfind(':')?;
    let ip = s[..colon].to_string();
    let port: u16 = s[colon + 1..].trim().parse().ok()?;
    if port == 0 {
        return None;
    }
    Some((ip, port))
}

pub fn parse_json_array(text: &str, _protocols: &[ProxyProtocol]) -> Vec<RawProxy> {
    let mut proxies = Vec::new();
    if let Ok(values) = serde_json::from_str::<Vec<serde_json::Value>>(text) {
        for v in values {
            let ip = v["ip"]
                .as_str()
                .or_else(|| v["host"].as_str())
                .unwrap_or("");
            let port = v["port"].as_u64().unwrap_or(0) as u16;
            if ip.is_empty() || port == 0 {
                continue;
            }
            let protocol = match v["protocol"].as_str() {
                Some("socks5") => ProxyProtocol::Socks5,
                Some("socks4") => ProxyProtocol::Socks4,
                Some("https") => ProxyProtocol::Https,
                _ => ProxyProtocol::Http,
            };
            proxies.push(RawProxy {
                ip: ip.to_string(),
                port,
                protocol,
                source: None,
            });
        }
    }
    proxies
}

fn parse_geonode_json(text: &str) -> Vec<RawProxy> {
    let mut proxies = Vec::new();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(data) = json["data"].as_array() {
            for entry in data {
                let ip = entry["ip"].as_str().unwrap_or("");
                let port = entry["port"].as_str().unwrap_or("");
                let port: u16 = port.parse().unwrap_or(0);
                let protocols_str = entry["protocols"].as_str().unwrap_or("http").to_lowercase();
                if ip.is_empty() || port == 0 {
                    continue;
                }
                let protocol = if protocols_str.contains("socks5") {
                    ProxyProtocol::Socks5
                } else if protocols_str.contains("socks4") {
                    ProxyProtocol::Socks4
                } else if protocols_str.contains("https") {
                    ProxyProtocol::Https
                } else {
                    ProxyProtocol::Http
                };
                proxies.push(RawProxy {
                    ip: ip.to_string(),
                    port,
                    protocol,
                    source: None,
                });
            }
        }
    }
    proxies
}

pub async fn validate_proxy(proxy_url: &str, protocol: ProxyProtocol, timeout_secs: u64) -> bool {
    match protocol {
        ProxyProtocol::Socks4 | ProxyProtocol::Socks5 => {
            let addr = proxy_url
                .trim_start_matches("socks5://")
                .trim_start_matches("socks4://");
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                tokio::net::TcpStream::connect(addr),
            )
            .await
            .is_ok_and(|r| r.is_ok())
        }
        ProxyProtocol::Http | ProxyProtocol::Https => {
            let addr = proxy_url
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                tokio::net::TcpStream::connect(addr),
            )
            .await
            .is_ok_and(|r| r.is_ok())
        }
    }
}

pub async fn validate_proxy_multi_target(
    proxy_url: &str,
    protocol: ProxyProtocol,
    targets: &[(&str, u16)],
    timeout_secs: u64,
) -> ProxyValidationResult {
    let connect_ok = validate_proxy(proxy_url, protocol, timeout_secs).await;
    if !connect_ok {
        return ProxyValidationResult {
            connect_ok: false,
            targets_passed: 0,
            targets_total: 0,
            avg_latency_ms: None,
        };
    }

    let mut passed = 0;
    let total = targets.len();
    let mut total_latency = 0.0;
    let mut measured = 0;

    for (host, port) in targets {
        let start = Instant::now();
        let ok = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            connect_through_proxy(proxy_url, protocol, host, *port),
        )
        .await
        .ok()
        .and_then(|r| r.ok())
        .is_some();
        if ok {
            passed += 1;
            total_latency += start.elapsed().as_millis() as f64;
            measured += 1;
        }
    }

    ProxyValidationResult {
        connect_ok: true,
        targets_passed: passed,
        targets_total: total,
        avg_latency_ms: if measured > 0 {
            Some(total_latency / measured as f64)
        } else {
            None
        },
    }
}

async fn connect_through_proxy(
    proxy_url: &str,
    protocol: ProxyProtocol,
    target_host: &str,
    target_port: u16,
) -> Result<tokio::net::TcpStream, String> {
    let proxy_addr = proxy_url
        .trim_start_matches("socks5://")
        .trim_start_matches("socks4://")
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("");
    if proxy_addr.is_empty() {
        return Err("empty proxy address".into());
    }

    match protocol {
        ProxyProtocol::Socks5 => {
            connect_via_socks5_validate(proxy_addr, target_host, target_port).await
        }
        ProxyProtocol::Socks4 => {
            connect_via_socks4_validate(proxy_addr, target_host, target_port).await
        }
        ProxyProtocol::Http | ProxyProtocol::Https => {
            connect_via_http_validate(proxy_addr, target_host, target_port).await
        }
    }
}

async fn connect_via_socks5_validate(
    proxy_addr: &str,
    target_host: &str,
    target_port: u16,
) -> Result<tokio::net::TcpStream, String> {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = tokio::net::TcpStream::connect(proxy_addr)
        .await
        .map_err(|e| format!("socks5 connect to proxy: {}", e))?;

    // SOCKS5 greet
    stream
        .write_all(&[5u8, 1, 0])
        .await
        .map_err(|e| format!("socks5 greet write: {}", e))?;
    let mut buf = [0u8; 2];
    stream
        .read_exact(&mut buf)
        .await
        .map_err(|e| format!("socks5 greet read: {}", e))?;
    if buf[1] != 0 {
        return Err(format!("socks5 auth required: {:?}", buf));
    }

    // SOCKS5 connect request
    let host_bytes = target_host.as_bytes();
    let mut msg = Vec::with_capacity(7 + host_bytes.len());
    msg.extend_from_slice(&[5u8, 1, 0, 3]);
    msg.push(host_bytes.len() as u8);
    msg.extend_from_slice(host_bytes);
    msg.extend_from_slice(&target_port.to_be_bytes());
    stream
        .write_all(&msg)
        .await
        .map_err(|e| format!("socks5 connect write: {}", e))?;

    let mut resp = [0u8; 4];
    stream
        .read_exact(&mut resp)
        .await
        .map_err(|e| format!("socks5 connect read: {}", e))?;
    if resp[1] != 0 {
        return Err(format!("socks5 rejected: code={}", resp[1]));
    }

    // Skip bound address
    let bound_len = match resp[3] {
        1 => 4usize,
        3 => {
            let mut lb = [0u8; 1];
            stream.read_exact(&mut lb).await.ok();
            lb[0] as usize
        }
        4 => 16usize,
        _ => 0,
    };
    let mut _skip = vec![0u8; bound_len + 2];
    let _ = stream.read_exact(&mut _skip).await;

    Ok(stream)
}

async fn connect_via_socks4_validate(
    proxy_addr: &str,
    target_host: &str,
    target_port: u16,
) -> Result<tokio::net::TcpStream, String> {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = tokio::net::TcpStream::connect(proxy_addr)
        .await
        .map_err(|e| format!("socks4 connect to proxy: {}", e))?;

    // SOCKS4a: [4, 1, port_hi, port_lo, 0, 0, 0, 1, 0, domain, 0]
    let host_bytes = target_host.as_bytes();
    let port_bytes = target_port.to_be_bytes();
    let mut msg = Vec::with_capacity(9 + host_bytes.len() + 1);
    msg.push(4);
    msg.push(1);
    msg.extend_from_slice(&port_bytes);
    msg.extend_from_slice(&[0, 0, 0, 1]);
    msg.push(0);
    msg.extend_from_slice(host_bytes);
    msg.push(0);

    stream
        .write_all(&msg)
        .await
        .map_err(|e| format!("socks4 write: {}", e))?;

    let mut resp = [0u8; 8];
    stream
        .read_exact(&mut resp)
        .await
        .map_err(|e| format!("socks4 read: {}", e))?;

    if resp[1] != 0x5a {
        return Err(format!("socks4 rejected: code={:#x}", resp[1]));
    }

    Ok(stream)
}

async fn connect_via_http_validate(
    proxy_addr: &str,
    target_host: &str,
    target_port: u16,
) -> Result<tokio::net::TcpStream, String> {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = tokio::net::TcpStream::connect(proxy_addr)
        .await
        .map_err(|e| format!("http proxy connect: {}", e))?;

    let connect_req = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
        target_host, target_port, target_host, target_port
    );
    stream
        .write_all(connect_req.as_bytes())
        .await
        .map_err(|e| format!("http proxy write: {}", e))?;

    let mut buf = [0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| format!("http proxy read: {}", e))?;

    let resp = std::str::from_utf8(&buf[..n]).unwrap_or("");
    if !resp.starts_with("HTTP/1.1 200") && !resp.starts_with("HTTP/1.0 200") {
        return Err(format!(
            "http proxy rejected: {}",
            resp.lines().next().unwrap_or("")
        ));
    }

    Ok(stream)
}

pub async fn batch_validate(
    proxies: &[RawProxy],
    concurrency: usize,
    timeout_secs: u64,
    targets: &[(&'static str, u16)],
) -> Vec<(RawProxy, ProxyValidationResult)> {
    let sem = Arc::new(tokio::sync::Semaphore::new(concurrency));
    let t_owned: Vec<(&'static str, u16)> = targets.to_vec();

    let futs: Vec<_> = proxies
        .iter()
        .map(|proxy| {
            let p = proxy.clone();
            let s = sem.clone();
            let t = t_owned.clone();
            async move {
                let _permit = s.acquire().await;
                let result =
                    validate_proxy_multi_target(&p.to_proxy_url(), p.protocol, &t, timeout_secs)
                        .await;
                (p, result)
            }
        })
        .collect();

    let mut results: Vec<_> = futures::future::join_all(futs).await;
    results.sort_by(|a, b| {
        b.1.pass_rate()
            .partial_cmp(&a.1.pass_rate())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

pub async fn fetch_and_validate(
    sourcing: &ProxySourcing,
    sources: &[&ProxySourceDef],
    max_proxies: usize,
    min_quality: ProxyQualityTier,
) -> Vec<(RawProxy, ProxyValidationResult)> {
    let all_results = sourcing.fetch_all_sources(sources).await;

    let mut all_proxies: Vec<RawProxy> = all_results
        .into_iter()
        .flat_map(|(source_name, proxies)| {
            proxies.into_iter().map(move |mut p| {
                p.source = Some(source_name);
                p
            })
        })
        .collect();

    let mut rng = rand::thread_rng();
    all_proxies.shuffle(&mut rng);

    if all_proxies.len() > max_proxies {
        all_proxies.truncate(max_proxies);
    }

    let all_proxies = dedup_proxies(&all_proxies);

    let validated = batch_validate(&all_proxies, 100, 5, DEFAULT_VALIDATION_TARGETS).await;

    validated
        .into_iter()
        .filter(|(_, r)| {
            let quality = r.classify_quality();
            quality as u8 <= min_quality as u8
        })
        .collect()
}

pub fn dedup_proxies(proxies: &[RawProxy]) -> Vec<RawProxy> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::with_capacity(proxies.len());
    for p in proxies {
        let key = (p.ip.clone(), p.port, p.protocol);
        if seen.insert(key) {
            result.push(p.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::*;

    #[test]
    fn test_parse_plain_text_basic() {
        let text = "1.2.3.4:8080\n5.6.7.8:3128\n# comment\n\n";
        let proxies = parse_plain_text(text, &[ProxyProtocol::Http]);
        assert_eq!(proxies.len(), 2);
        assert_eq!(proxies[0].ip, "1.2.3.4");
        assert_eq!(proxies[0].port, 8080);
        assert_eq!(proxies[0].protocol, ProxyProtocol::Http);
        assert_eq!(proxies[1].ip, "5.6.7.8");
        assert_eq!(proxies[1].port, 3128);
    }

    #[test]
    fn test_parse_with_scheme_prefix() {
        let text = "socks5://1.2.3.4:1080\nhttp://5.6.7.8:8080\nhttps://9.10.11.12:443";
        let proxies = parse_plain_text(text, &[ProxyProtocol::Http]);
        assert_eq!(proxies.len(), 3);
        assert_eq!(proxies[0].protocol, ProxyProtocol::Socks5);
        assert_eq!(proxies[0].port, 1080);
        assert_eq!(proxies[1].protocol, ProxyProtocol::Http);
        assert_eq!(proxies[2].protocol, ProxyProtocol::Https);
    }

    #[test]
    fn test_parse_proxifly_all_format() {
        let text = "1.2.3.4:8080\n5.6.7.8:3128\n9.10.11.12:1080\n";
        let proxies = parse_plain_text(text, &[ProxyProtocol::Http]);
        assert_eq!(proxies.len(), 3);
    }

    #[test]
    fn test_parse_proxifly_http_format() {
        let text = "203.0.113.1:8080\n198.51.100.2:3128\n";
        let proxies = parse_plain_text(text, &[ProxyProtocol::Http]);
        assert_eq!(proxies.len(), 2);
        for p in &proxies {
            assert_eq!(p.protocol, ProxyProtocol::Http);
        }
    }

    #[test]
    fn test_split_host_port_valid() {
        let (ip, port) = split_host_port("192.168.1.1:8080").unwrap();
        assert_eq!(ip, "192.168.1.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_split_host_port_ipv6() {
        let (ip, port) = split_host_port("[::1]:1080").unwrap();
        assert_eq!(ip, "[::1]");
        assert_eq!(port, 1080);
    }

    #[test]
    fn test_split_host_port_invalid() {
        assert!(split_host_port("").is_none());
        assert!(split_host_port("no-colon").is_none());
        assert!(split_host_port("1.2.3.4:0").is_none());
        assert!(split_host_port("1.2.3.4:abc").is_none());
    }

    #[test]
    fn test_parse_geonode_json() {
        let json = r#"{
            "data": [
                {"ip": "1.2.3.4", "port": "8080", "protocols": "http"},
                {"ip": "5.6.7.8", "port": "1080", "protocols": "socks5"}
            ]
        }"#;
        let proxies = parse_geonode_json(json);
        assert_eq!(proxies.len(), 2);
        assert_eq!(proxies[0].protocol, ProxyProtocol::Http);
        assert_eq!(proxies[1].protocol, ProxyProtocol::Socks5);
    }

    #[test]
    fn test_raw_proxy_to_url() {
        let p = RawProxy {
            ip: "1.2.3.4".into(),
            port: 8080,
            protocol: ProxyProtocol::Http,
            source: None,
        };
        assert_eq!(p.to_proxy_url(), "http://1.2.3.4:8080");

        let p2 = RawProxy {
            ip: "1.2.3.4".into(),
            port: 1080,
            protocol: ProxyProtocol::Socks5,
            source: None,
        };
        assert_eq!(p2.to_proxy_url(), "socks5://1.2.3.4:1080");
    }

    #[test]
    fn test_proxy_quality_classify() {
        let tier_s = ProxyValidationResult {
            connect_ok: true,
            targets_passed: 4,
            targets_total: 5,
            avg_latency_ms: Some(100.0),
        }
        .classify_quality();
        assert_eq!(tier_s, ProxyQualityTier::S);

        let tier_a = ProxyValidationResult {
            connect_ok: true,
            targets_passed: 4,
            targets_total: 5,
            avg_latency_ms: Some(600.0),
        }
        .classify_quality();
        assert_eq!(tier_a, ProxyQualityTier::A);

        let tier_d = ProxyValidationResult {
            connect_ok: false,
            targets_passed: 0,
            targets_total: 3,
            avg_latency_ms: None,
        }
        .classify_quality();
        assert_eq!(tier_d, ProxyQualityTier::D);
    }

    #[test]
    fn test_source_health_cooldown() {
        let mut health = SourceHealth::default();
        assert!(!health.is_on_cooldown());
        assert!((health.success_rate() - 0.5).abs() < 0.01);

        health.record_failure("timeout".into());
        health.record_failure("refused".into());
        health.record_failure("dns".into());
        assert!(health.is_on_cooldown());
        assert!((health.success_rate() - 0.0).abs() < 0.01);

        health.record_success();
        assert!(!health.is_on_cooldown());
    }

    #[test]
    fn test_dedup_proxies() {
        let proxies = vec![
            RawProxy {
                ip: "1.2.3.4".into(),
                port: 8080,
                protocol: ProxyProtocol::Http,
                source: None,
            },
            RawProxy {
                ip: "1.2.3.4".into(),
                port: 8080,
                protocol: ProxyProtocol::Http,
                source: None,
            },
            RawProxy {
                ip: "5.6.7.8".into(),
                port: 3128,
                protocol: ProxyProtocol::Http,
                source: None,
            },
        ];
        let deduped = dedup_proxies(&proxies);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_protocol_as_str() {
        assert_eq!(ProxyProtocol::Http.as_str(), "http");
        assert_eq!(ProxyProtocol::Https.as_str(), "https");
        assert_eq!(ProxyProtocol::Socks4.as_str(), "socks4");
        assert_eq!(ProxyProtocol::Socks5.as_str(), "socks5");
    }

    #[test]
    fn test_quality_tier_as_str() {
        assert_eq!(ProxyQualityTier::S.as_str(), "S");
        assert_eq!(ProxyQualityTier::A.as_str(), "A");
        assert_eq!(ProxyQualityTier::B.as_str(), "B");
        assert_eq!(ProxyQualityTier::C.as_str(), "C");
        assert_eq!(ProxyQualityTier::D.as_str(), "D");
    }

    #[tokio::test]
    async fn test_proxy_sourcing_fetch_bad_source() {
        let sourcing = ProxySourcing::new();
        let bad_source = ProxySourceDef {
            name: "bad-test",
            url: "http://127.0.0.1:1/nonexistent",
            protocols: &[ProxyProtocol::Http],
            format: SourceFormat::PlainTxt,
            weight: 1,
        };
        let result = sourcing.fetch_source(&bad_source).await;
        assert!(result.is_err());
        let health = sourcing.source_health("bad-test").await;
        assert_eq!(health.total_failures, 1);
    }
}
