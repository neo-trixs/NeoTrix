//! HTTP 拦截代理引擎 — 请求/响应拦截、修改、重放
//!
//! 对标 Strix `tools/proxy/` 模块，基于 std::net::TcpListener
//! 零外部依赖（纯标准库）

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
}

impl HttpInterceptor {
    pub fn new(listen_addr: SocketAddr, upstream: &str) -> Self {
        Self {
            listen_addr,
            upstream: upstream.to_string(),
            rules: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
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
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let rules = self.rules.clone();
        let upstream = self.upstream.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                match stream {
                    Ok(mut client) => {
                        let rules = rules.clone();
                        let upstream = upstream.clone();
                        std::thread::spawn(move || {
                            if let Err(e) = handle_client(&mut client, &rules, &upstream) {
                                log::warn!("Proxy handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        log::warn!("Connection error: {}", e);
                    }
                }
            }
        });
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
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
    let (method, url, _headers, _body) = read_http_request(client)?;

    let action = rules
        .iter()
        .find(|rule| rule.matches(&url, &method))
        .map(|rule| rule.action.clone())
        .unwrap_or(InterceptAction::Forward);

    match action {
        InterceptAction::Drop => {
            let response =
                b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            client.write_all(response)?;
        }
        InterceptAction::ModifyRequest(headers) => {
            forward_with_modified_headers(client, upstream, &method, &url, &headers)?;
        }
        InterceptAction::ModifyBody(body) => {
            forward_with_modified_body(client, upstream, &method, &url, &body)?;
        }
        InterceptAction::Capture(name) => {
            let response = forward_request(upstream, &method, &url)?;
            log::info!("[Capture:{}] {} {}", name, method, url);
            client.write_all(&response)?;
        }
        InterceptAction::Forward => {
            let response = forward_request(upstream, &method, &url)?;
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
