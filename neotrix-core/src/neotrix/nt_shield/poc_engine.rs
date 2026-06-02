use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use regex::Regex;

use super::http_proxy::HttpInterceptor;

#[derive(Debug, Clone)]
pub struct PoCStep {
    pub description: String,
    pub request: PoCHttpRequest,
    pub expected_result: PoCExpectedResult,
}

#[derive(Debug, Clone)]
pub struct PoCHttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl PoCHttpRequest {
    pub fn to_raw_request(&self) -> String {
        let mut req = format!("{} {} HTTP/1.1\r\n", self.method, self.url);
        for (k, v) in &self.headers {
            req.push_str(&format!("{}: {}\r\n", k, v));
        }
        if let Some(ref body) = self.body {
            req.push_str(&format!("Content-Length: {}\r\n", body.len()));
            req.push_str("\r\n");
            req.push_str(body);
        } else {
            req.push_str("\r\n");
        }
        req
    }
}

#[derive(Debug, Clone)]
pub enum PoCExpectedResult {
    StatusCode(u16),
    BodyContains(String),
    BodyMatches(String),
    ResponseTime { min_ms: u64, max_ms: u64 },
    HeaderPresent(String),
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub response_time_ms: u64,
}

impl HttpResponse {
    pub fn from_raw(data: &[u8], response_time_ms: u64) -> Self {
        let raw = String::from_utf8_lossy(data).to_string();
        let mut parts = raw.splitn(2, "\r\n\r\n");
        let header_section = parts.next().unwrap_or("");
        let body = parts.next().unwrap_or("").to_string();

        let lines: Vec<&str> = header_section.lines().collect();
        let status_code = if !lines.is_empty() {
            let status_parts: Vec<&str> = lines[0].splitn(3, ' ').collect();
            status_parts
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
        } else {
            0
        };

        let headers: Vec<(String, String)> = lines[1..]
            .iter()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ": ");
                match (parts.next(), parts.next()) {
                    (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                    _ => None,
                }
            })
            .collect();

        Self {
            status_code,
            headers,
            body,
            response_time_ms,
        }
    }
}

impl PoCExpectedResult {
    pub fn matches(&self, response: &HttpResponse) -> bool {
        match self {
            PoCExpectedResult::StatusCode(code) => response.status_code == *code,
            PoCExpectedResult::BodyContains(pattern) => response.body.contains(pattern),
            PoCExpectedResult::BodyMatches(pattern) => {
                Regex::new(pattern).map(|re| re.is_match(&response.body)).unwrap_or(false)
            }
            PoCExpectedResult::ResponseTime { min_ms, max_ms } => {
                response.response_time_ms >= *min_ms && response.response_time_ms <= *max_ms
            }
            PoCExpectedResult::HeaderPresent(header) => {
                response.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(header))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Evidence {
    pub description: String,
    pub severity: Severity,
    pub request_snapshot: String,
    pub response_snapshot: String,
    pub reproducible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "Critical",
            Severity::High => "High",
            Severity::Medium => "Medium",
            Severity::Low => "Low",
            Severity::Info => "Info",
        }
    }
}

pub struct PocEngine {
    pub poc_steps: Vec<PoCStep>,
    pub evidence: Vec<Evidence>,
    pub verified: bool,
}

impl PocEngine {
    pub fn new() -> Self {
        Self {
            poc_steps: Vec::new(),
            evidence: Vec::new(),
            verified: false,
        }
    }

    pub fn add_step(&mut self, step: PoCStep) {
        self.poc_steps.push(step);
    }

    pub fn verify(&mut self, interceptor: &HttpInterceptor) -> Result<bool, String> {
        let addr = interceptor.listen_addr();
        let mut any_match = false;

        for step in &self.poc_steps {
            let raw_request = step.request.to_raw_request();
            let request_snapshot = raw_request.clone();

            match TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
                Ok(mut stream) => {
                    let start = std::time::Instant::now();
                    if let Err(e) = stream.write_all(raw_request.as_bytes()) {
                        self.evidence.push(Evidence {
                            description: format!("Write error: {}", e),
                            severity: Severity::Medium,
                            request_snapshot,
                            response_snapshot: String::new(),
                            reproducible: false,
                        });
                        continue;
                    }

                    let mut response_buf = Vec::new();
                    if let Err(e) = stream.read_to_end(&mut response_buf) {
                        self.evidence.push(Evidence {
                            description: format!("Read error: {}", e),
                            severity: Severity::Medium,
                            request_snapshot,
                            response_snapshot: String::new(),
                            reproducible: false,
                        });
                        continue;
                    }
                    let elapsed = start.elapsed();

                    let response =
                        HttpResponse::from_raw(&response_buf, elapsed.as_millis() as u64);
                    let response_snapshot =
                        String::from_utf8_lossy(&response_buf).to_string();

                    let matched = step.expected_result.matches(&response);
                    if matched {
                        any_match = true;
                    }

                    self.evidence.push(Evidence {
                        description: step.description.clone(),
                        severity: Severity::High,
                        request_snapshot,
                        response_snapshot,
                        reproducible: matched,
                    });
                }
                Err(e) => {
                    self.evidence.push(Evidence {
                        description: format!("Connection failed: {}", e),
                        severity: Severity::Medium,
                        request_snapshot,
                        response_snapshot: String::new(),
                        reproducible: false,
                    });
                }
            }
        }

        self.verified = any_match;
        Ok(any_match)
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# PoC Validation Report\n\n");
        report.push_str(&format!("**Verified**: {}\n\n", self.verified));
        report.push_str(&format!("**Steps**: {}\n", self.poc_steps.len()));
        report.push_str(&format!("**Evidence**: {}\n\n", self.evidence.len()));

        report.push_str("## Evidence Details\n\n");
        for (i, ev) in self.evidence.iter().enumerate() {
            report.push_str(&format!("### Evidence {}\n", i + 1));
            report.push_str(&format!("- **Description**: {}\n", ev.description));
            report.push_str(&format!("- **Severity**: {}\n", ev.severity.label()));
            report.push_str(&format!("- **Reproducible**: {}\n", ev.reproducible));
            report.push_str("\n#### Request\n```\n");
            report.push_str(&ev.request_snapshot);
            report.push_str("\n```\n\n");
            report.push_str("#### Response\n```\n");
            report.push_str(&ev.response_snapshot);
            report.push_str("\n```\n\n");
        }
        report
    }
}

impl Default for PocEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poc_step_construction() {
        let req = PoCHttpRequest {
            method: "GET".to_string(),
            url: "/admin".to_string(),
            headers: vec![("Host".to_string(), "example.com".to_string())],
            body: None,
        };
        let step = PoCStep {
            description: "Check admin panel access".to_string(),
            request: req,
            expected_result: PoCExpectedResult::StatusCode(200),
        };
        assert_eq!(step.description, "Check admin panel access");
        assert_eq!(step.request.method, "GET");
        assert_eq!(step.request.url, "/admin");
    }

    #[test]
    fn test_poc_http_request_formatting() {
        let req = PoCHttpRequest {
            method: "POST".to_string(),
            url: "/login".to_string(),
            headers: vec![
                ("Host".to_string(), "example.com".to_string()),
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: Some(r#"{"user":"admin"}"#.to_string()),
        };
        let raw = req.to_raw_request();
        assert!(raw.starts_with("POST /login HTTP/1.1\r\n"));
        assert!(raw.contains("Host: example.com"));
        assert!(raw.contains("Content-Type: application/json"));
        assert!(raw.contains("Content-Length: "));
        assert!(raw.ends_with(r#"{"user":"admin"}"#));
    }

    #[test]
    fn test_poc_http_request_no_body() {
        let req = PoCHttpRequest {
            method: "GET".to_string(),
            url: "/health".to_string(),
            headers: vec![("Host".to_string(), "example.com".to_string())],
            body: None,
        };
        let raw = req.to_raw_request();
        assert!(raw.starts_with("GET /health HTTP/1.1\r\n"));
        assert!(raw.contains("Host: example.com"));
        assert!(raw.ends_with("\r\n"));
    }

    #[test]
    fn test_expected_result_status_code_match() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: "OK".to_string(),
            response_time_ms: 10,
        };
        let expected = PoCExpectedResult::StatusCode(200);
        assert!(expected.matches(&resp));

        let expected_fail = PoCExpectedResult::StatusCode(403);
        assert!(!expected_fail.matches(&resp));
    }

    #[test]
    fn test_expected_result_body_contains() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: "Welcome, admin!".to_string(),
            response_time_ms: 10,
        };
        let expected = PoCExpectedResult::BodyContains("admin".to_string());
        assert!(expected.matches(&resp));

        let expected_fail = PoCExpectedResult::BodyContains("root".to_string());
        assert!(!expected_fail.matches(&resp));
    }

    #[test]
    fn test_expected_result_body_matches_regex() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: "Error: invalid token (code 42)".to_string(),
            response_time_ms: 10,
        };
        let expected = PoCExpectedResult::BodyMatches(r"Error:.*\(code \d+\)".to_string());
        assert!(expected.matches(&resp));

        let expected_fail =
            PoCExpectedResult::BodyMatches(r"Success:.*".to_string());
        assert!(!expected_fail.matches(&resp));
    }

    #[test]
    fn test_expected_result_header_present() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![
                ("Content-Type".to_string(), "text/html".to_string()),
                ("X-Frame-Options".to_string(), "DENY".to_string()),
            ],
            body: String::new(),
            response_time_ms: 10,
        };
        let expected = PoCExpectedResult::HeaderPresent("X-Frame-Options".to_string());
        assert!(expected.matches(&resp));

        let expected_fail =
            PoCExpectedResult::HeaderPresent("X-XSS-Protection".to_string());
        assert!(!expected_fail.matches(&resp));
    }

    #[test]
    fn test_expected_result_response_time() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: String::new(),
            response_time_ms: 150,
        };
        let expected =
            PoCExpectedResult::ResponseTime {
                min_ms: 100,
                max_ms: 200,
            };
        assert!(expected.matches(&resp));

        let expected_outside =
            PoCExpectedResult::ResponseTime {
                min_ms: 200,
                max_ms: 300,
            };
        assert!(!expected_outside.matches(&resp));
    }

    #[test]
    fn test_severity_ordering() {
        let mut severities = vec![
            Severity::Info,
            Severity::Critical,
            Severity::Low,
            Severity::High,
            Severity::Medium,
        ];
        severities.sort();
        assert_eq!(
            severities,
            vec![
                Severity::Critical,
                Severity::High,
                Severity::Medium,
                Severity::Low,
                Severity::Info,
            ]
        );
    }

    #[test]
    fn test_severity_label() {
        assert_eq!(Severity::Critical.label(), "Critical");
        assert_eq!(Severity::High.label(), "High");
        assert_eq!(Severity::Medium.label(), "Medium");
        assert_eq!(Severity::Low.label(), "Low");
        assert_eq!(Severity::Info.label(), "Info");
    }

    #[test]
    fn test_poc_engine_new_and_add_step() {
        let mut engine = PocEngine::new();
        assert!(engine.poc_steps.is_empty());
        assert!(engine.evidence.is_empty());
        assert!(!engine.verified);

        let step = PoCStep {
            description: "Test SQLi".to_string(),
            request: PoCHttpRequest {
                method: "GET".to_string(),
                url: "/api".to_string(),
                headers: vec![],
                body: None,
            },
            expected_result: PoCExpectedResult::StatusCode(500),
        };
        engine.add_step(step);
        assert_eq!(engine.poc_steps.len(), 1);
        assert_eq!(engine.poc_steps[0].description, "Test SQLi");
    }

    #[test]
    fn test_generate_report_empty() {
        let engine = PocEngine::new();
        let report = engine.generate_report();
        assert!(report.contains("# PoC Validation Report"));
        assert!(report.contains("**Verified**: false"));
        assert!(report.contains("**Steps**: 0"));
        assert!(report.contains("**Evidence**: 0"));
    }

    #[test]
    fn test_generate_report_with_evidence() {
        let mut engine = PocEngine::new();
        let step = PoCStep {
            description: "XSS check".to_string(),
            request: PoCHttpRequest {
                method: "GET".to_string(),
                url: "/search?q=<script>".to_string(),
                headers: vec![("Host".to_string(), "x.com".to_string())],
                body: None,
            },
            expected_result: PoCExpectedResult::BodyContains("<script>".to_string()),
        };
        engine.add_step(step);
        engine.evidence.push(Evidence {
            description: "XSS check".to_string(),
            severity: Severity::Critical,
            request_snapshot: "GET /search?q=<script> HTTP/1.1".to_string(),
            response_snapshot: "HTTP/1.1 200 OK\n<body><script>".to_string(),
            reproducible: true,
        });

        let report = engine.generate_report();
        assert!(report.contains("### Evidence 1"));
        assert!(report.contains("XSS check"));
        assert!(report.contains("Critical"));
        assert!(report.contains("**Reproducible**: true"));
        assert!(report.contains("GET /search?q=<script>"));
        assert!(report.contains("HTTP/1.1 200 OK"));
    }

    #[test]
    fn test_http_response_from_raw() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<body>hello</body>";
        let resp = HttpResponse::from_raw(raw, 42);
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.headers.len(), 1);
        assert_eq!(resp.headers[0].0, "Content-Type");
        assert_eq!(resp.headers[0].1, "text/html");
        assert_eq!(resp.body, "<body>hello</body>");
        assert_eq!(resp.response_time_ms, 42);
    }

    #[test]
    fn test_poc_engine_default() {
        let engine: PocEngine = Default::default();
        assert!(engine.poc_steps.is_empty());
        assert!(!engine.verified);
    }
}
