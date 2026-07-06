use std::collections::HashMap;
use std::time::{Duration, Instant};

const RING_CAPACITY: usize = 1000;

#[derive(Debug, Clone)]
pub struct CapturedRequest {
    pub method: String,
    pub url: String,
    pub host: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub body_preview: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct CapturedResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub body_preview: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrafficCategory {
    ApiCall,
    AuthToken,
    FileUpload,
    FileDownload,
    DataExfil,
    ConfigSync,
    Telemetry,
    Heartbeat,
    SsrfProbe,
    Unknown,
}

impl TrafficCategory {
    pub fn classify(host: &str, url: &str, _headers: &[(String, String)], _body: &[u8]) -> Self {
        let combined = format!("{} {}", host, url).to_lowercase();

        if combined.contains("auth") || combined.contains("token") || combined.contains("login")
            || combined.contains("oauth") || combined.contains("apikey")
            || combined.contains("credential") || combined.contains("signin")
        {
            return Self::AuthToken;
        }
        if combined.contains("telemetry") || combined.contains("analytics")
            || combined.contains("track") || combined.contains("metric")
            || combined.contains("beacon")
        {
            return Self::Telemetry;
        }
        if combined.contains("heartbeat") || combined.contains("ping")
            || combined.contains("healthz") || combined.contains("alive")
        {
            return Self::Heartbeat;
        }
        if combined.contains("upload") || combined.contains("put")
            || combined.contains("multipart") || combined.contains("form-data")
        {
            return Self::FileUpload;
        }
        if combined.contains("download") || combined.contains("export")
            || combined.contains("archive")
        {
            return Self::FileDownload;
        }
        if combined.contains("config") || combined.contains("setting")
            || combined.contains("sync") || combined.contains("preference")
        {
            return Self::ConfigSync;
        }
        if url.contains("/api/") || url.contains("/v1/") || url.contains("/v2/") || url.contains("/v3/")
            || url.contains("/graphql") || url.contains("/rest/") || url.contains("/rpc")
        {
            return Self::ApiCall;
        }
        Self::Unknown
    }
}

pub trait SensitivityDetector: Send + Sync {
    fn detect(&self, data: &[u8]) -> Vec<SensitiveFinding>;
}

#[derive(Debug, Clone)]
pub struct SensitiveFinding {
    pub category: String,
    pub description: String,
    pub severity: Severity,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

pub struct DefaultSensitivityDetector;

impl DefaultSensitivityDetector {
    const PATTERNS: &'static [(Severity, &'static str, &'static str)] = &[
        (Severity::Critical, "API Key Anthropic", r"(?i)sk-ant-[a-z0-9\-]{32,}"),
        (Severity::Critical, "API Key OpenAI", r"(?i)sk-[a-zA-Z0-9\-]{32,}"),
        (Severity::Critical, "API Key Generic", r"(?i)(api[_-]?key|apikey|api[_-]?secret)[=:]\s*['\x22]?[a-zA-Z0-9_\-]{16,}"),
        (Severity::Critical, "AWS Access Key", r"(?i)AKIA[0-9A-Z]{16}"),
        (Severity::Critical, "Bearer Token", r"(?i)bearer\s+[a-zA-Z0-9_\-\.]{20,}"),
        (Severity::High, "JWT Token", r"(?i)eyJ[a-zA-Z0-9\-_]+\.eyJ[a-zA-Z0-9\-_]+"),
        (Severity::High, "Private Key", r"-----BEGIN\s+(?:RSA|EC|PRIVATE|OPENSSH)\s+KEY-----"),
        (Severity::High, "Password", r"(?i)(password|passwd|pwd)[=:]\s*['\x22]?[^'\x22]{4,}"),
        (Severity::Medium, "Email", r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}"),
        (Severity::Medium, "IP Private", r"\b(10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(1[6-9]|2\d|3[01])\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3})\b"),
        (Severity::Low, "Authorization Header", "(?i)authorization:\\s*\\S+"),
        (Severity::Low, "Set-Cookie", "(?i)set-cookie:\\s*\\S+"),
        (Severity::Info, "User-Agent", "(?i)user-agent:\\s*\\S+"),
        (Severity::Info, "Content-Type", r"(?i)content-type:\s*\S+"),
    ];
}

impl SensitivityDetector for DefaultSensitivityDetector {
    fn detect(&self, data: &[u8]) -> Vec<SensitiveFinding> {
        let text = String::from_utf8_lossy(data);
        let mut findings = Vec::new();
        for &(ref severity, category, pattern_str) in Self::PATTERNS {
            if let Ok(re) = regex::Regex::new(pattern_str) {
                for cap in re.find_iter(&text) {
                    if findings.iter().any(|f: &SensitiveFinding| {
                        f.category == category && f.severity == *severity
                    }) {
                        continue;
                    }
                    findings.push(SensitiveFinding {
                        category: category.to_string(),
                        description: format!("{} detected", category),
                        severity: severity.clone(),
                        offset: Some(cap.start()),
                    });
                }
            }
        }
        findings
    }
}

#[derive(Debug, Clone)]
pub struct TrafficSession {
    pub id: u64,
    pub host: String,
    pub port: u16,
    pub direction: Direction,
    pub category: TrafficCategory,
    pub request: Option<CapturedRequest>,
    pub response: Option<CapturedResponse>,
    pub req_bytes: Vec<u8>,
    pub resp_bytes: Vec<u8>,
    pub findings: Vec<SensitiveFinding>,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Outbound,
    Inbound,
}

#[derive(Debug, Clone)]
pub struct HostSummary {
    pub host: String,
    pub request_count: usize,
    pub total_bytes_sent: usize,
    pub total_bytes_received: usize,
    pub categories: Vec<TrafficCategory>,
    pub sensitivity_count: usize,
    pub last_seen: Instant,
}

pub struct TrafficAnalyzer {
    sessions: Vec<TrafficSession>,
    next_id: u64,
    detector: Box<dyn SensitivityDetector>,
}

impl TrafficAnalyzer {
    pub fn new() -> Self {
        Self {
            sessions: Vec::with_capacity(RING_CAPACITY),
            next_id: 1,
            detector: Box::new(DefaultSensitivityDetector),
        }
    }

    pub fn with_detector(detector: Box<dyn SensitivityDetector>) -> Self {
        Self {
            sessions: Vec::with_capacity(RING_CAPACITY),
            next_id: 1,
            detector,
        }
    }

    pub fn capture_request(
        &mut self,
        host: &str,
        port: u16,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> u64 {
        let category = TrafficCategory::classify(host, url, headers, body);
        let body_preview = if body.len() > 512 {
            format!("{}... ({} bytes)", String::from_utf8_lossy(&body[..512]), body.len())
        } else {
            String::from_utf8_lossy(body).to_string()
        };

        let mut findings = self.detector.detect(body);
        for (k, v) in headers {
            let header_bytes = format!("{}: {}", k, v);
            findings.extend(self.detector.detect(header_bytes.as_bytes()));
        }

        let id = self.next_id;
        self.next_id += 1;

        let session = TrafficSession {
            id,
            host: host.to_string(),
            port,
            direction: Direction::Outbound,
            category,
            request: Some(CapturedRequest {
                method: method.to_string(),
                url: url.to_string(),
                host: host.to_string(),
                headers: headers.to_vec(),
                body: body.to_vec(),
                body_preview,
                timestamp: Instant::now(),
            }),
            response: None,
            req_bytes: body.to_vec(),
            resp_bytes: Vec::new(),
            findings,
            start_time: Instant::now(),
            end_time: None,
            duration: None,
        };

        if self.sessions.len() >= RING_CAPACITY {
            self.sessions.remove(0);
        }
        self.sessions.push(session);
        id
    }

    pub fn capture_response(
        &mut self,
        id: u64,
        status_code: u16,
        status_text: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) {
        if let Some(session) = self.sessions.iter_mut().rev().find(|s| s.id == id) {
            let body_preview = if body.len() > 1024 {
                format!("{}... ({} bytes)", String::from_utf8_lossy(&body[..1024]), body.len())
            } else {
                String::from_utf8_lossy(body).to_string()
            };

            session.response = Some(CapturedResponse {
                status_code,
                status_text: status_text.to_string(),
                headers: headers.to_vec(),
                body: body.to_vec(),
                body_preview,
                timestamp: Instant::now(),
            });
            session.resp_bytes = body.to_vec();
            session.end_time = Some(Instant::now());
            session.duration = Some(session.start_time.elapsed());

            let resp_findings = self.detector.detect(body);
            for f in resp_findings {
                if !session.findings.iter().any(|sf| sf.category == f.category) {
                    session.findings.push(f);
                }
            }
        }
    }

    pub fn host_summary(&self) -> Vec<HostSummary> {
        let mut by_host: HashMap<String, Vec<&TrafficSession>> = HashMap::new();
        for s in &self.sessions {
            by_host.entry(s.host.clone()).or_default().push(s);
        }

        let mut summaries: Vec<HostSummary> = by_host
            .into_iter()
            .map(|(host, sessions)| {
                let req_count = sessions.len();
                let total_sent = sessions.iter().map(|s| s.req_bytes.len()).sum();
                let total_recv = sessions.iter().map(|s| s.resp_bytes.len()).sum();
                let sensitivity_count = sessions.iter().filter(|s| !s.findings.is_empty()).count();

                let mut cats: Vec<TrafficCategory> = sessions
                    .iter()
                    .map(|s| s.category.clone())
                    .collect();
                cats.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
                cats.dedup_by_key(|c| format!("{:?}", c));

                let last_seen = sessions
                    .iter()
                    .map(|s| s.start_time)
                    .max()
                    .unwrap_or_else(Instant::now);

                HostSummary {
                    host,
                    request_count: req_count,
                    total_bytes_sent: total_sent,
                    total_bytes_received: total_recv,
                    categories: cats,
                    sensitivity_count,
                    last_seen,
                }
            })
            .collect();

        summaries.sort_by(|a, b| b.request_count.cmp(&a.request_count));
        summaries
    }

    pub fn findings_by_host(&self) -> Vec<(String, Vec<SensitiveFinding>)> {
        let mut map: HashMap<String, Vec<SensitiveFinding>> = HashMap::new();
        for s in &self.sessions {
            if !s.findings.is_empty() {
                map.entry(s.host.clone())
                    .or_default()
                    .extend(s.findings.clone());
            }
        }
        let mut result: Vec<_> = map.into_iter().collect();
        result.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        result
    }

    pub fn recent_sessions(&self, n: usize) -> Vec<&TrafficSession> {
        self.sessions.iter().rev().take(n).collect()
    }

    pub fn sessions_by_host(&self, host: &str) -> Vec<&TrafficSession> {
        self.sessions.iter().filter(|s| s.host == host).collect()
    }

    pub fn total_sessions(&self) -> usize {
        self.sessions.len()
    }

    pub fn traffic_volume_by_host(&self) -> Vec<(String, usize, usize)> {
        let mut map: HashMap<String, (usize, usize)> = HashMap::new();
        for s in &self.sessions {
            let entry = map.entry(s.host.clone()).or_insert((0, 0));
            entry.0 += s.req_bytes.len();
            entry.1 += s.resp_bytes.len();
        }
        let mut result: Vec<_> = map
            .into_iter()
            .map(|(k, (sent, recv))| (k, sent, recv))
            .collect();
        result.sort_by(|a, b| (b.1 + b.2).cmp(&(a.1 + a.2)));
        result
    }

    pub fn generate_report(&self) -> TrafficReport {
        let total = self.sessions.len();
        let hosts = self.host_summary();
        let total_sent: usize = self.sessions.iter().map(|s| s.req_bytes.len()).sum();
        let total_recv: usize = self.sessions.iter().map(|s| s.resp_bytes.len()).sum();
        let sensitive_hosts = self.findings_by_host();

        let mut category_counts: HashMap<String, usize> = HashMap::new();
        for s in &self.sessions {
            let label = format!("{:?}", s.category);
            *category_counts.entry(label).or_insert(0) += 1;
        }

        TrafficReport {
            total_sessions: total,
            total_hosts: hosts.len(),
            total_bytes_sent: total_sent,
            total_bytes_received: total_recv,
            top_hosts: hosts,
            sensitive_hosts,
            category_counts,
            duration: Duration::from_secs(0),
        }
    }
}

impl Default for TrafficAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TrafficReport {
    pub total_sessions: usize,
    pub total_hosts: usize,
    pub total_bytes_sent: usize,
    pub total_bytes_received: usize,
    pub top_hosts: Vec<HostSummary>,
    pub sensitive_hosts: Vec<(String, Vec<SensitiveFinding>)>,
    pub category_counts: HashMap<String, usize>,
    pub duration: Duration,
}

impl std::fmt::Display for TrafficReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═ Traffic Analysis Report ═")?;
        writeln!(f, "  Sessions:  {}", self.total_sessions)?;
        writeln!(f, "  Hosts:     {}", self.total_hosts)?;
        writeln!(f, "  Sent:      {} bytes", self.total_bytes_sent)?;
        writeln!(f, "  Received:  {} bytes", self.total_bytes_received)?;

        if !self.category_counts.is_empty() {
            writeln!(f, "  Categories:")?;
            let mut cats: Vec<_> = self.category_counts.iter().collect();
            cats.sort_by(|a, b| b.1.cmp(a.1));
            for (cat, count) in cats {
                writeln!(f, "    {}: {}", cat, count)?;
            }
        }

        if !self.sensitive_hosts.is_empty() {
            writeln!(f, "  Sensitive Data Detected:")?;
            for (host, findings) in &self.sensitive_hosts {
                writeln!(f, "    {} ({} findings)", host, findings.len())?;
                for finding in findings.iter().take(3) {
                    writeln!(f, "      [{:?}] {}: {}", finding.severity, finding.category, finding.description)?;
                }
            }
        }

        if !self.top_hosts.is_empty() {
            writeln!(f, "  Top Hosts:")?;
            for h in self.top_hosts.iter().take(10) {
                writeln!(
                    f,
                    "    {:30} {:4} reqs {:8}→ {:8}← | {} sensitive",
                    h.host,
                    h.request_count,
                    h.total_bytes_sent,
                    h.total_bytes_received,
                    h.sensitivity_count
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification() {
        assert_eq!(
            TrafficCategory::classify("api.anthropic.com", "/v1/messages", &[], b""),
            TrafficCategory::ApiCall
        );
        assert_eq!(
            TrafficCategory::classify("auth.example.com", "/oauth/token", &[], b""),
            TrafficCategory::AuthToken
        );
        assert_eq!(
            TrafficCategory::classify("telemetry.example.com", "/track", &[], b""),
            TrafficCategory::Telemetry
        );
        assert_eq!(
            TrafficCategory::classify("example.com", "/healthz", &[], b""),
            TrafficCategory::Heartbeat
        );
    }

    #[test]
    fn test_sensitivity_detection() {
        let detector = DefaultSensitivityDetector;
        let data = b"Authorization: Bearer sk-ant-test1234567890abcdefghij";
        let findings = detector.detect(data);
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
    }

    #[test]
    fn test_analyzer_capture_and_report() {
        let mut analyzer = TrafficAnalyzer::new();
        let id = analyzer.capture_request(
            "api.anthropic.com",
            443,
            "POST",
            "/v1/messages",
            &[("content-type".into(), "application/json".into())],
            b"{\"model\":\"claude-sonnet-4\"}",
        );
        analyzer.capture_response(id, 200, "OK", &[], b"{\"content\":\"hello\"}");
        let report = analyzer.generate_report();
        assert_eq!(report.total_sessions, 1);
        assert_eq!(report.total_hosts, 1);
    }

    #[test]
    fn test_ring_buffer_capacity() {
        let mut analyzer = TrafficAnalyzer::new();
        for _i in 0..RING_CAPACITY + 10 {
            let id = analyzer.capture_request("host.com", 443, "GET", "/", &[], b"");
            analyzer.capture_response(id, 200, "OK", &[], b"");
        }
        assert_eq!(analyzer.total_sessions(), RING_CAPACITY);
    }

    #[test]
    fn test_analyzer_sensitive_data_tracking() {
        let mut analyzer = TrafficAnalyzer::new();
        let id = analyzer.capture_request("payments.com", 443, "POST", "/charge", &[], b"password=secret123");
        analyzer.capture_response(id, 200, "OK", &[], b"token=eyJhbGciOiJIUzI1NiJ9.eyJ0ZXN0IjoiZGF0YSJ9");
        let findings = analyzer.findings_by_host();
        assert!(!findings.is_empty());
        let host_findings = findings.iter().find(|(h, _)| h == "payments.com");
        assert!(host_findings.is_some());
    }

    #[test]
    fn test_clean_site_no_findings() {
        let mut analyzer = TrafficAnalyzer::new();
        let id = analyzer.capture_request("docs.rs", 443, "GET", "/rustc/", &[], b"");
        analyzer.capture_response(id, 200, "OK", &[], b"<html>docs</html>");
        assert!(analyzer.findings_by_host().is_empty());
    }

    #[test]
    fn test_traffic_volume() {
        let mut analyzer = TrafficAnalyzer::new();
        let id = analyzer.capture_request("cdn.example.com", 443, "GET", "/large.bin", &[], &vec![0u8; 5000]);
        analyzer.capture_response(id, 200, "OK", &[], &vec![0u8; 20000]);
        let volumes = analyzer.traffic_volume_by_host();
        assert_eq!(volumes[0].1, 5000);
        assert_eq!(volumes[0].2, 20000);
    }

    #[test]
    fn test_host_summary_ordering() {
        let mut analyzer = TrafficAnalyzer::new();
        let id1 = analyzer.capture_request("busy.com", 443, "GET", "/1", &[], b"");
        analyzer.capture_response(id1, 200, "OK", &[], b"");
        let id2 = analyzer.capture_request("busy.com", 443, "GET", "/2", &[], b"");
        analyzer.capture_response(id2, 200, "OK", &[], b"");
        let id3 = analyzer.capture_request("quiet.com", 443, "GET", "/", &[], b"");
        analyzer.capture_response(id3, 200, "OK", &[], b"");
        let summaries = analyzer.host_summary();
        assert_eq!(summaries[0].host, "busy.com");
        assert_eq!(summaries[0].request_count, 2);
    }
}
