use std::collections::HashSet;
use std::fs;
use std::time::{Duration, Instant};

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let subs_path = format!("{home}/.neotrix/subscriptions.json");
    let conf_path = format!("{home}/.neotrix/proxy-upstreams.conf");

    let urls: Vec<String> = match fs::read_to_string(&subs_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => {
            eprintln!("[sub-fetcher] no subscriptions.json at {subs_path}");
            Vec::new()
        }
    };

    if urls.is_empty() {
        eprintln!("[sub-fetcher] 0 subscription URLs, nothing to do");
        return;
    }

    // Load all existing entries (any protocol)
    let existing_content = fs::read_to_string(&conf_path).unwrap_or_default();
    let existing_entries: HashSet<String> = existing_content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .map(|l| l.trim().to_string())
        .collect();
    let mut discovered: HashSet<String> = existing_entries.clone();
    eprintln!(
        "[sub-fetcher] loaded {} existing entries",
        existing_entries.len()
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .danger_accept_invalid_certs(true)
        .build()
        .expect("reqwest::Client::new failed - TLS backend init");

    for url in &urls {
        eprintln!("[sub-fetcher] fetching {url}");
        let start = Instant::now();
        let data = match client
            .get(url)
            .timeout(Duration::from_secs(10))
            .send()
        {
            Ok(resp) => match resp.bytes() {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[sub-fetcher]   read error: {e}");
                    continue;
                }
            },
            Err(e) => {
                eprintln!("[sub-fetcher]   fetch error: {e}");
                continue;
            }
        };
        let elapsed = start.elapsed().as_secs_f64();
        eprintln!(
            "[sub-fetcher]   got {} bytes in {elapsed:.1}s",
            data.len()
        );

        let text = try_decode(&data);
        let lines: Vec<&str> = text.lines().collect();
        eprintln!("[sub-fetcher]   {} lines", lines.len());

        let mut found = 0usize;
        for line in &lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let urls = extract_urls(line);
            for u in urls {
                if discovered.insert(u.clone()) {
                    found += 1;
                }
            }
        }
        eprintln!("[sub-fetcher]   +{found} new entries from this source");

        // Write after each source so partial results survive timeout
        write_conf(&conf_path, &discovered);
    }
}

fn write_conf(conf_path: &str, entries: &HashSet<String>) {
    let mut output = String::new();
    output.push_str("# NeoTrix proxy upstreams\n");
    output.push_str(&format!("# {} unique entries\n", entries.len()));

    let mut sorted: Vec<&String> = entries.iter().collect();
    sorted.sort();
    for url in sorted {
        output.push_str(url);
        output.push('\n');
    }

    if let Err(e) = fs::write(conf_path, &output) {
        eprintln!("[sub-fetcher] write error: {e}");
    } else {
        eprintln!(
            "[sub-fetcher] wrote {} entries to {conf_path}",
            entries.len()
        );
    }
}

fn try_decode(data: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(data) {
        if looks_like_base64(s) {
            let mut decoded = String::new();
            for line in s.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(bytes) = decode_base64(line) {
                    if let Ok(text) = String::from_utf8(bytes) {
                        decoded.push_str(&text);
                        decoded.push('\n');
                    } else {
                        decoded.push_str(line);
                        decoded.push('\n');
                    }
                } else {
                    decoded.push_str(line);
                    decoded.push('\n');
                }
            }
            return decoded;
        }
        return s.to_string();
    }
    if let Ok(bytes) = decode_base64_lossy(data) {
        if let Ok(s) = String::from_utf8(bytes) {
            return s;
        }
    }
    String::from_utf8_lossy(data).to_string()
}

fn looks_like_base64(s: &str) -> bool {
    let non_b64 = s
        .chars()
        .filter(|c| !c.is_ascii_alphanumeric() && *c != '+' && *c != '/' && *c != '='
            && *c != '\n' && *c != '\r')
        .count();
    let total = s.chars().count();
    if total == 0 {
        return false;
    }
    let ratio = non_b64 as f64 / total as f64;
    ratio < 0.3
}

fn decode_base64(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    let input = input.trim();
    let padded = match input.len() % 4 {
        0 => input.to_string(),
        r => {
            let mut s = input.to_string();
            s.extend(std::iter::repeat('=').take(4 - r));
            s
        }
    };
    base64::engine::general_purpose::STANDARD.decode(&padded)
}

fn decode_base64_lossy(data: &[u8]) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    let input = String::from_utf8_lossy(data);
    let input = input.trim();
    let padded = match input.len() % 4 {
        0 => input.to_string(),
        r => {
            let mut s = input.to_string();
            s.extend(std::iter::repeat('=').take(4 - r));
            s
        }
    };
    base64::engine::general_purpose::STANDARD.decode(&padded)
}

fn extract_urls(line: &str) -> Vec<String> {
    let line = line.trim();
    let mut results = Vec::new();

    // Keep all standard proxy URIs as-is (pool parses them natively)
    if line.starts_with("socks5://") || line.starts_with("socks5h://")
        || line.starts_with("vmess://")
        || line.starts_with("vless://")
        || line.starts_with("ss://")
        || line.starts_with("ssr://")
        || line.starts_with("trojan://")
        || line.starts_with("hysteria2://") || line.starts_with("hy2://")
    {
        results.push(line.to_string());
        return results;
    }

    // Detect YAML/JSON server:port pairs for Clash/Sing-box formats
    if let Some(server_val) = extract_yaml_json_value(line, "server") {
        if let Some(port_val) = extract_yaml_json_value(line, "port") {
            if let Ok(port) = port_val.parse::<u16>() {
                results.push(format!("socks5h://{server_val}:{port}"));
            }
        }
    }
    // Also check for server_port (Sing-box JSON)
    if let Some(server_val) = extract_yaml_json_value(line, "server") {
        if let Some(port_val) = extract_yaml_json_value(line, "server_port") {
            if let Ok(port) = port_val.parse::<u16>() {
                results.push(format!("socks5h://{server_val}:{port}"));
            }
        }
    }

    if line.starts_with("http://") || line.starts_with("https://") {
        let without_scheme = line
            .strip_prefix("http://")
            .or_else(|| line.strip_prefix("https://"))
            .unwrap_or(line);
        if let Some(host_port) = without_scheme.rsplit_once('@').map(|(_, hp)| hp).or(Some(without_scheme)) {
            if let Some((host, port_str)) = host_port.rsplit_once(':') {
                let host = host.trim_end_matches('/');
                if let Ok(_port) = port_str.parse::<u16>() {
                    if !host.is_empty() {
                        results.push(format!("socks5h://{host}:{port_str}"));
                    }
                }
            }
        }
        return results;
    }

    if let Some((host, port_str)) = line.rsplit_once(':') {
        if let Ok(_port) = port_str.parse::<u16>() {
            if !host.contains('/') && !host.contains('\\') && !host.contains('"') && !host.is_empty() {
                results.push(format!("socks5h://{host}:{port_str}"));
            }
        }
    }

    results
}

fn extract_yaml_json_value(line: &str, key: &str) -> Option<String> {
    let json_key = format!("\"{}\"", key);
    if let Some(pos) = line.find(&json_key) {
        let rest = &line[pos + json_key.len()..];
        let rest = rest.trim_start().strip_prefix(':')?.trim_start();
        if let Some(quoted) = rest.strip_prefix('"') {
            let end = quoted.find('"')?;
            return Some(quoted[..end].to_string());
        }
        let end = rest.find([',', '}', '\n', ' ']).unwrap_or(rest.len());
        if end > 0 {
            let val = rest[..end].trim().to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
        return None;
    }
    let yaml_prefix = format!("{}:", key);
    if let Some(pos) = line.find(&yaml_prefix) {
        let rest = line[pos + yaml_prefix.len()..].trim();
        if let Some(quoted) = rest.strip_prefix('\'') {
            let end = quoted.find('\'')?;
            return Some(quoted[..end].to_string());
        }
        let end = rest.find([' ', '\t', '#']).unwrap_or(rest.len());
        if end > 0 {
            let val = rest[..end].trim().to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}
