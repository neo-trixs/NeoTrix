//! HTTP 拦截代理引擎 — 请求/响应拦截、修改、重放
//!
//! 对标 Strix `tools/proxy/` 模块，基于 std::net::TcpListener
//! 零外部依赖（纯标准库）

use regex::Regex;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// RAII：离开作用域时递减并发连接计数
struct DecrementGuard(Arc<AtomicUsize>);

impl Drop for DecrementGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/// 拦截动作
#[derive(Debug, Clone)]
pub enum InterceptAction {
    Forward,
    ModifyRequest(Vec<(String, String)>),
    ModifyBody(String),
    Drop,
    Capture(String),
}

/// 拦截规则
#[derive(Debug, Clone)]
pub struct InterceptRule {
    pub match_url: String,
    pub match_method: Option<String>,
    pub action: InterceptAction,
}

impl InterceptRule {
    pub fn matches(&self, url: &str, method: &str) -> bool {
        if let Some(ref m) = self.match_method {
            if m != method {
                return false;
            }
        }
        url.starts_with(&self.match_url)
    }
}

/// NeoTrix 内部标识模式列表 —— 与 Python nt_comm_router.py HEAD 保持同步
const INTERNAL_PATTERNS: &[(&str, &str)] = &[
    (r"neotrix", "client"),
    (r"\bnt_[a-z]", "sys_"),
    (r"\bNEOTRIX_", "CLIENT_"),
    (r"x-neotrix-", "x-client-"),
    (r"x-nt-", "x-client-"),
];

/// 剥离内部标识 —— 对每一个 header name/value 应用 INTERNAL_PATTERNS
/// 等价于 Python HeaderObfuscator._strip_internal_headers()
pub fn strip_internal_patterns(input: &str) -> String {
    let mut result = input.to_string();
    for (pattern, replacement) in INTERNAL_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            result = re.replace_all(&result, *replacement).to_string();
        }
    }
    // 同时剥离 UUID 和文件路径 (与 Python 同步)
    if let Ok(re) = Regex::new(r"\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b") {
        result = re.replace_all(&result, "00000000-0000-0000-0000-000000000000").to_string();
    }
    if let Ok(re) = Regex::new(r"/Users/[^/]+/") {
        result = re.replace_all(&result, "/home/user/").to_string();
    }
    result
}

/// 对一组 header (name, value) 的 value 应用剥离
pub fn strip_header_values(headers: &[(String, String)]) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(k, v)| {
            let k_clean = strip_internal_patterns(k);
            let v_clean = strip_internal_patterns(v);
            (k_clean, v_clean)
        })
        .collect()
}

/// HTTP 请求摘要（最小解析）
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

/// 安全测试模板
#[derive(Debug, Clone)]
pub enum SecurityTest {
    XssReflected(String),
    XssStored(String),
    CsrfTokenBypass,
    SqlInjection(String),
    SsrfCheck(String),
}

impl std::fmt::Display for SecurityTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityTest::XssReflected(payload) => write!(f, "XSS-Reflected({})", payload),
            SecurityTest::XssStored(payload) => write!(f, "XSS-Stored({})", payload),
            SecurityTest::CsrfTokenBypass => write!(f, "CSRF-Token-Bypass"),
            SecurityTest::SqlInjection(payload) => write!(f, "SQL-Injection({})", payload),
            SecurityTest::SsrfCheck(target) => write!(f, "SSRF-Check({})", target),
        }
    }
}

/// HTTP 拦截代理
#[derive(Debug)]
pub struct HttpInterceptor {
    listen_addr: SocketAddr,
    upstream: String,
    rules: Vec<InterceptRule>,
    running: Arc<AtomicBool>,
    listener_handle: Option<std::thread::JoinHandle<()>>,
}

impl HttpInterceptor {
    pub fn new(listen_addr: SocketAddr, upstream: &str) -> Self {
        Self {
            listen_addr,
            upstream: upstream.to_string(),
            rules: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
            listener_handle: None,
        }
    }

    pub fn add_rule(&mut self, rule: InterceptRule) {
        self.rules.push(rule);
    }

    pub fn start(&mut self) -> std::io::Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }
        let listener = TcpListener::bind(self.listen_addr)?;
        listener.set_nonblocking(true)?;
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let rules = self.rules.clone();
        let upstream = self.upstream.clone();
        let handle = std::thread::spawn(move || {
            // 并发连接上限：thread-per-connection 模型下必须防止批量连接打满线程
            let max_concurrent = 128usize;
            let active = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            while running.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut client, _)) => {
                        if active.load(Ordering::SeqCst) >= max_concurrent {
                            log::warn!("http_proxy: max concurrent connections reached, dropping");
                            continue;
                        }
                        active.fetch_add(1, Ordering::SeqCst);
                        let rules = rules.clone();
                        let upstream = upstream.clone();
                        let running = running.clone();
                        let active = active.clone();
                        std::thread::spawn(move || {
                            let _decrement = DecrementGuard(active);
                            if running.load(Ordering::SeqCst) {
                                if let Err(e) = handle_client(&mut client, &rules, &upstream) {
                                    log::warn!("Proxy handler error: {}", e);
                                }
                            }
                        });
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        if running.load(Ordering::SeqCst) {
                            log::warn!("Accept error: {}", e);
                        }
                        break;
                    }
                }
            }
        });
        self.listener_handle = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.listener_handle.take() {
            let _ = handle.join();
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn rules(&self) -> &[InterceptRule] {
        &self.rules
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }
}

fn handle_client(
    client: &mut TcpStream,
    rules: &[InterceptRule],
    upstream: &str,
) -> std::io::Result<()> {
    let (method, url, headers, _body) = read_http_request(client)?;

    // Strip internal NeoTrix identifiers from headers and URL
    let stripped_headers = strip_header_values(&headers);
    let stripped_url = strip_internal_patterns(&url);

    let action = rules
        .iter()
        .find(|rule| rule.matches(&stripped_url, &method))
        .map(|rule| rule.action.clone())
        .unwrap_or(InterceptAction::Forward);

    match action {
        InterceptAction::Drop => {
            let response =
                b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            client.write_all(response)?;
        }
        InterceptAction::ModifyRequest(headers) => {
            let mut all_headers = stripped_headers.clone();
            all_headers.extend(headers);
            forward_with_modified_headers(client, upstream, &method, &stripped_url, &all_headers)?;
        }
        InterceptAction::ModifyBody(body) => {
            forward_with_modified_body(client, upstream, &method, &stripped_url, &body)?;
        }
        InterceptAction::Capture(name) => {
            let response = forward_request(upstream, &method, &stripped_url)?;
            log::info!("[Capture:{}] {} {}", name, method, stripped_url);
            client.write_all(&response)?;
        }
        InterceptAction::Forward => {
            let response = forward_request(upstream, &method, &stripped_url)?;
            client.write_all(&response)?;
        }
    }

    Ok(())
}

fn read_http_request(
    stream: &mut TcpStream,
) -> std::io::Result<(String, String, Vec<(String, String)>, String)> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok((String::new(), String::new(), Vec::new(), String::new()));
    }

    let raw = String::from_utf8_lossy(&buf[..n]).to_string();
    let mut parts = raw.splitn(2, "\r\n\r\n");
    let header_section = parts.next().unwrap_or("");
    let body = parts.next().unwrap_or("").to_string();

    let lines: Vec<&str> = header_section.lines().collect();
    let (method, url) = if lines.is_empty() {
        (String::new(), String::new())
    } else {
        let first_parts: Vec<&str> = lines[0].splitn(3, ' ').collect();
        if first_parts.len() >= 2 {
            (first_parts[0].to_string(), first_parts[1].to_string())
        } else {
            (String::new(), String::new())
        }
    };

    let headers: Vec<(String, String)> = lines[1..]
        .iter()
        .filter_map(|line| {
            let mut line_parts = line.splitn(2, ": ");
            match (line_parts.next(), line_parts.next()) {
                (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                _ => None,
            }
        })
        .collect();

    Ok((method, url, headers, body))
}

fn forward_request(upstream: &str, method: &str, url: &str) -> std::io::Result<Vec<u8>> {
    let mut upstream_stream = TcpStream::connect(upstream)?;
    let request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        method, url, upstream
    );
    upstream_stream.write_all(request.as_bytes())?;
    let mut response = Vec::new();
    upstream_stream.read_to_end(&mut response)?;
    Ok(response)
}

fn forward_with_modified_headers(
    client: &mut TcpStream,
    upstream: &str,
    method: &str,
    url: &str,
    new_headers: &[(String, String)],
) -> std::io::Result<()> {
    let extra_headers: String = new_headers
        .iter()
        .map(|(k, v)| format!("{}: {}\r\n", k, v))
        .collect();
    let request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\n{}Connection: close\r\n\r\n",
        method, url, upstream, extra_headers
    );

    let mut upstream_stream = TcpStream::connect(upstream)?;
    upstream_stream.write_all(request.as_bytes())?;
    let mut response = Vec::new();
    upstream_stream.read_to_end(&mut response)?;
    client.write_all(&response)?;
    Ok(())
}

fn forward_with_modified_body(
    client: &mut TcpStream,
    upstream: &str,
    method: &str,
    url: &str,
    body: &str,
) -> std::io::Result<()> {
    let request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nContent-Type: application/x-www-form-urlencoded\r\nConnection: close\r\n\r\n{}",
        method, url, upstream, body.len(), body
    );

    let mut upstream_stream = TcpStream::connect(upstream)?;
    upstream_stream.write_all(request.as_bytes())?;
    let mut response = Vec::new();
    upstream_stream.read_to_end(&mut response)?;
    client.write_all(&response)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_match_by_url_prefix() {
        let rule = InterceptRule {
            match_url: "/api".to_string(),
            match_method: None,
            action: InterceptAction::Drop,
        };
        assert!(rule.matches("/api/users", "GET"));
        assert!(rule.matches("/api", "POST"));
        assert!(!rule.matches("/other", "GET"));
    }

    #[test]
    fn test_rule_match_by_method() {
        let rule = InterceptRule {
            match_url: "/api".to_string(),
            match_method: Some("POST".to_string()),
            action: InterceptAction::Drop,
        };
        assert!(rule.matches("/api/data", "POST"));
        assert!(!rule.matches("/api/data", "GET"));
    }

    #[test]
    fn test_no_matching_rule_falls_to_forward() {
        let rules: Vec<InterceptRule> = vec![];
        let action = rules
            .iter()
            .find(|rule| rule.matches("/api", "GET"))
            .map(|rule| rule.action.clone())
            .unwrap_or(InterceptAction::Forward);
        assert!(matches!(action, InterceptAction::Forward));
    }

    #[test]
    fn test_first_match_wins() {
        let rules = vec![
            InterceptRule {
                match_url: "/api".to_string(),
                match_method: None,
                action: InterceptAction::Drop,
            },
            InterceptRule {
                match_url: "/api/users".to_string(),
                match_method: None,
                action: InterceptAction::Forward,
            },
        ];
        let action = rules
            .iter()
            .find(|rule| rule.matches("/api/users", "GET"))
            .map(|rule| rule.action.clone())
            .unwrap_or(InterceptAction::Forward);
        assert!(matches!(action, InterceptAction::Drop));
    }

    #[test]
    fn test_modify_request_action_headers() -> Result<(), String> {
        let headers = vec![("X-Test".to_string(), "value".to_string())];
        let action = InterceptAction::ModifyRequest(headers.clone());
        match action {
            InterceptAction::ModifyRequest(h) => {
                assert_eq!(h.len(), 1);
                assert_eq!(h[0], ("X-Test".to_string(), "value".to_string()));
            }
            other => return Err(format!("Expected ModifyRequest, got {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_drop_action() {
        let rule = InterceptRule {
            match_url: "/block".to_string(),
            match_method: None,
            action: InterceptAction::Drop,
        };
        assert!(rule.matches("/block", "GET"));
        assert!(matches!(rule.action, InterceptAction::Drop));
    }

    #[test]
    fn test_nt_shield_test_display() {
        let xss = SecurityTest::XssReflected("<script>".to_string());
        assert_eq!(xss.to_string(), "XSS-Reflected(<script>)");

        let csrf = SecurityTest::CsrfTokenBypass;
        assert_eq!(csrf.to_string(), "CSRF-Token-Bypass");

        let sqli = SecurityTest::SqlInjection("' OR 1=1 --".to_string());
        assert_eq!(sqli.to_string(), "SQL-Injection(' OR 1=1 --)");

        let ssrf = SecurityTest::SsrfCheck("http://169.254.169.254".to_string());
        assert_eq!(ssrf.to_string(), "SSRF-Check(http://169.254.169.254)");

        let stored = SecurityTest::XssStored("<img src=x>".to_string());
        assert_eq!(stored.to_string(), "XSS-Stored(<img src=x>)");
    }

    #[test]
    fn test_interceptor_default_state() {
        let addr: SocketAddr = "127.0.0.1:0".parse().expect("value should be ok in test");
        let interceptor = HttpInterceptor::new(addr, "http://example.com");
        assert!(!interceptor.is_running());
        assert!(interceptor.rules().is_empty());
    }

    #[test]
    fn test_add_rule() {
        let addr: SocketAddr = "127.0.0.1:0".parse().expect("value should be ok in test");
        let mut interceptor = HttpInterceptor::new(addr, "http://example.com");
        interceptor.add_rule(InterceptRule {
            match_url: "/test".to_string(),
            match_method: None,
            action: InterceptAction::Drop,
        });
        assert_eq!(interceptor.rules().len(), 1);
        assert!(interceptor.rules()[0].matches("/test", "GET"));
    }

    #[test]
    fn test_http_request_struct() {
        let req = HttpRequest {
            method: "POST".to_string(),
            url: "/submit".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: "{\"key\":\"value\"}".to_string(),
        };
        assert_eq!(req.method, "POST");
        assert_eq!(req.url, "/submit");
        assert_eq!(req.headers[0].0, "Content-Type");
        assert_eq!(req.body, "{\"key\":\"value\"}");
    }

    #[test]
    fn test_request_parsing_logic() {
        let raw = "GET /test HTTP/1.1\r\nHost: localhost\r\nUser-Agent: test\r\n\r\n";
        let mut parts = raw.splitn(2, "\r\n\r\n");
        let header_section = parts.next().expect("value should be ok in test");
        let lines: Vec<&str> = header_section.lines().collect();

        let first_parts: Vec<&str> = lines[0].splitn(3, ' ').collect();
        assert_eq!(first_parts[0], "GET");
        assert_eq!(first_parts[1], "/test");

        let headers: Vec<(String, String)> = lines[1..]
            .iter()
            .filter_map(|line| {
                let mut lp = line.splitn(2, ": ");
                match (lp.next(), lp.next()) {
                    (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                    _ => None,
                }
            })
            .collect();
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn test_intercept_rule_clone() {
        let rule = InterceptRule {
            match_url: "/clone".to_string(),
            match_method: Some("GET".to_string()),
            action: InterceptAction::Capture("test".to_string()),
        };
        let cloned = rule.clone();
        assert_eq!(rule.match_url, cloned.match_url);
        assert_eq!(rule.match_method, cloned.match_method);
    }
}
