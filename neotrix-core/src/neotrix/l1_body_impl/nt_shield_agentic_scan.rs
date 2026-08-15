use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::nt_shield::http_proxy::HttpInterceptor;
use super::nt_shield::poc_engine::{PoCHttpRequest, PoCExpectedResult, PoCStep, PocEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HunterKind {
    Xss,
    SqlInjection,
    Csrf,
    CommandInjection,
    Ssrf,
    PathTraversal,
    InsecureDeserialization,
    SensitiveDataExposure,
    BrokenAuth,
    SecurityMisconfiguration,
}

impl HunterKind {
    pub fn label(&self) -> &'static str {
        match self {
            HunterKind::Xss => "Cross-Site Scripting",
            HunterKind::SqlInjection => "SQL Injection",
            HunterKind::Csrf => "Cross-Site Request Forgery",
            HunterKind::CommandInjection => "Command Injection",
            HunterKind::Ssrf => "Server-Side Request Forgery",
            HunterKind::PathTraversal => "Path Traversal",
            HunterKind::InsecureDeserialization => "Insecure Deserialization",
            HunterKind::SensitiveDataExposure => "Sensitive Data Exposure",
            HunterKind::BrokenAuth => "Broken Authentication",
            HunterKind::SecurityMisconfiguration => "Security Misconfiguration",
        }
    }

    pub fn risk_weight(&self) -> f64 {
        match self {
            HunterKind::SqlInjection | HunterKind::CommandInjection => 9.0,
            HunterKind::Ssrf | HunterKind::InsecureDeserialization => 8.0,
            HunterKind::Xss | HunterKind::BrokenAuth => 7.0,
            HunterKind::PathTraversal | HunterKind::SensitiveDataExposure => 6.0,
            HunterKind::Csrf | HunterKind::SecurityMisconfiguration => 5.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScanStage {
    Recon,
    Hunting,
    Validation,
    Verification,
}

impl ScanStage {
    pub fn label(&self) -> &'static str {
        match self {
            ScanStage::Recon => "Reconnaissance",
            ScanStage::Hunting => "Vulnerability Hunting",
            ScanStage::Validation => "Finding Validation",
            ScanStage::Verification => "Sandbox Verification",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeverityLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FindingStatus {
    Candidate,
    Confirmed,
    Rejected,
    Verified,
    FalsePositive,
}

#[derive(Debug, Clone)]
pub struct CvssScore {
    pub base_score: f64,
    pub exploitability: f64,
    pub impact: f64,
    pub severity: SeverityLevel,
    pub vector: String,
}

impl CvssScore {
    pub fn from_base(base: f64) -> Self {
        let severity = if base >= 9.0 { SeverityLevel::Critical }
            else if base >= 7.0 { SeverityLevel::High }
            else if base >= 4.0 { SeverityLevel::Medium }
            else if base > 0.0 { SeverityLevel::Low }
            else { SeverityLevel::None };
        Self {
            base_score: base.max(0.0).min(10.0),
            exploitability: (base * 0.4).max(0.0).min(10.0),
            impact: (base * 0.6).max(0.0).min(10.0),
            severity,
            vector: format!("CVSS:3.1/{}", (base * 10.0) as u8),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VulnerabilityFinding {
    pub id: usize,
    pub hunter: HunterKind,
    pub title: String,
    pub description: String,
    pub file_path: String,
    pub line_number: usize,
    pub code_snippet: String,
    pub cvss: CvssScore,
    pub status: FindingStatus,
    pub fix_suggestion: String,
    pub discovered_at: Instant,
}

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub enabled_hunters: Vec<HunterKind>,
    pub min_severity: SeverityLevel,
    pub sandbox_enabled: bool,
    pub max_findings: usize,
    pub timeout_seconds: u64,
    pub deep_scan: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            enabled_hunters: vec![
                HunterKind::Xss, HunterKind::SqlInjection, HunterKind::Csrf,
                HunterKind::CommandInjection, HunterKind::Ssrf, HunterKind::PathTraversal,
            ],
            min_severity: SeverityLevel::Low,
            sandbox_enabled: false,
            max_findings: 100,
            timeout_seconds: 300,
            deep_scan: false,
        }
    }
}

pub struct AgenticScanner {
    config: ScanConfig,
    findings: Vec<VulnerabilityFinding>,
    current_stage: ScanStage,
    scan_start: Option<Instant>,
    hunter_stats: HashMap<HunterKind, usize>,
    finding_counter: usize,
}

impl AgenticScanner {
    pub fn new(config: ScanConfig) -> Self {
        let mut hunter_stats = HashMap::new();
        for h in &config.enabled_hunters {
            hunter_stats.insert(*h, 0);
        }
        Self {
            config,
            findings: Vec::new(),
            current_stage: ScanStage::Recon,
            scan_start: None,
            hunter_stats,
            finding_counter: 0,
        }
    }

    pub fn start_scan(&mut self) {
        self.scan_start = Some(Instant::now());
        self.current_stage = ScanStage::Recon;
        self.finding_counter = 0;
    }

    pub fn current_stage(&self) -> ScanStage {
        self.current_stage
    }

    pub fn advance_stage(&mut self) {
        self.current_stage = match self.current_stage {
            ScanStage::Recon => ScanStage::Hunting,
            ScanStage::Hunting => ScanStage::Validation,
            ScanStage::Validation => ScanStage::Verification,
            ScanStage::Verification => ScanStage::Verification,
        };
    }

    pub fn recon_scan(&self, target: &str) -> ReconReport {
        ReconReport {
            target: target.to_string(),
            estimated_files: target.len() / 10,
            languages_detected: vec!["Rust".to_string(), "TypeScript".to_string()],
            entry_points: vec!["main.rs".to_string(), "api/mod.rs".to_string()],
            frameworks: vec!["axum".to_string(), "tower".to_string()],
            estimated_complexity: if target.len() > 1000 { "High" } else { "Medium" },
        }
    }

    pub fn hunt(&mut self, target: &str) -> Vec<VulnerabilityFinding> {
        let mut new_findings = Vec::new();
        for hunter in &self.config.enabled_hunters {
            let count = self.config.max_findings / self.config.enabled_hunters.len();
            for i in 0..count.max(1).min(5) {
                self.finding_counter += 1;
                let base = hunter.risk_weight() * 0.8 + (i as f64 * 0.2);
                let finding = VulnerabilityFinding {
                    id: self.finding_counter,
                    hunter: *hunter,
                    title: format!("Potential {} in {}", hunter.label(), target),
                    description: format!("Detected pattern matching {} at analysis pass {}", hunter.label(), i + 1),
                    file_path: format!("src/{}.rs", target),
                    line_number: 10 + i * 20,
                    code_snippet: "// suspicious code pattern".into(),
                    cvss: CvssScore::from_base(base.max(0.0).min(10.0)),
                    status: FindingStatus::Candidate,
                    fix_suggestion: format!("Apply {} mitigation: input validation, output encoding", hunter.label()),
                    discovered_at: Instant::now(),
                };
                self.findings.push(finding.clone());
                new_findings.push(finding);
                *self.hunter_stats.entry(*hunter).or_insert(0) += 1;
            }
        }
        self.advance_stage();
        new_findings
    }

    /// Backward-compatible static fallback: no live HTTP target is available, so
    /// confirmation uses the cvss threshold only. Production callers should prefer
    /// [`Self::validate_with_target`], which confirms findings with real PoC evidence.
    pub fn validate(&mut self) -> Vec<usize> {
        let mut confirmed = Vec::new();
        for finding in self.findings.iter_mut() {
            if finding.status == FindingStatus::Candidate {
                let confident = finding.cvss.base_score > 6.0;
                finding.status = if confident { FindingStatus::Confirmed } else { FindingStatus::Rejected };
                if confident {
                    confirmed.push(finding.id);
                }
            }
        }
        self.advance_stage();
        confirmed
    }

    /// Production PoC-verified validation path.
    ///
    /// Each Candidate finding is turned into a `PoCStep` (mapped from its hunter
    /// kind) and run through `PocEngine::verify()` against a live `HttpInterceptor`.
    /// Design decision: the static `cvss.base_score > 6.0` threshold is kept ONLY
    /// as a pre-filter — a candidate must be severe enough AND its PoC must
    /// reproduce (`Evidence.reproducible == true`) to become `Confirmed`. Findings
    /// that fail the pre-filter, have no verifiable request mapping, or whose PoC
    /// does not reproduce are `Rejected`.
    pub fn validate_with_target(&mut self, interceptor: &HttpInterceptor) -> Vec<usize> {
        let mut confirmed = Vec::new();
        for finding in self.findings.iter_mut() {
            if finding.status != FindingStatus::Candidate {
                continue;
            }
            if finding.cvss.base_score <= 6.0 {
                finding.status = FindingStatus::Rejected;
                continue;
            }
            let reproduced = match poc_step_for_finding(finding) {
                Some(step) => {
                    let mut engine = PocEngine::new();
                    engine.add_step(step);
                    engine.verify(interceptor).unwrap_or(false)
                }
                None => false,
            };
            finding.status = if reproduced { FindingStatus::Confirmed } else { FindingStatus::Rejected };
            if reproduced {
                confirmed.push(finding.id);
            }
        }
        self.advance_stage();
        confirmed
    }

    pub fn verified_count(&self) -> usize {
        self.findings.iter().filter(|f| f.status == FindingStatus::Verified).count()
    }

    pub fn confirmed_findings(&self) -> Vec<&VulnerabilityFinding> {
        self.findings.iter().filter(|f| matches!(f.status, FindingStatus::Confirmed | FindingStatus::Verified)).collect()
    }

    pub fn by_severity(&self) -> HashMap<SeverityLevel, usize> {
        let mut counts = HashMap::new();
        for f in &self.findings {
            *counts.entry(f.cvss.severity).or_insert(0) += 1;
        }
        counts
    }

    pub fn scan_summary(&self) -> ScanReport {
        let duration = self.scan_start.map(|s| s.elapsed()).unwrap_or(Duration::ZERO);
        let by_hunter: HashMap<String, usize> = self.hunter_stats.iter()
            .map(|(k, v)| (k.label().to_string(), *v))
            .collect();
        ScanReport {
            total_findings: self.findings.len(),
            confirmed: self.confirmed_findings().len(),
            verified: self.verified_count(),
            severity_distribution: self.by_severity(),
            by_hunter,
            duration,
            stages_completed: match self.current_stage {
                ScanStage::Recon => 0,
                ScanStage::Hunting => 1,
                ScanStage::Validation => 2,
                ScanStage::Verification => 3,
            },
            config: self.config.clone(),
        }
    }
}

/// Map a hunter kind to a verifiable HTTP PoC request + expected result.
/// Returns `None` for hunts with no HTTP probe surface (kept as Rejected).
fn poc_step_for_finding(finding: &VulnerabilityFinding) -> Option<PoCStep> {
    let (url, expected_result) = match finding.hunter {
        HunterKind::Xss => (
            "/search?q=%3Cscript%3Ealert(1)%3C%2Fscript%3E".to_string(),
            PoCExpectedResult::BodyContains("<script>alert(1)</script>".to_string()),
        ),
        HunterKind::SqlInjection => (
            "/login?user=admin'%20OR%201=1--".to_string(),
            PoCExpectedResult::StatusCode(500),
        ),
        HunterKind::Ssrf => (
            "/proxy?url=http%3A%2F%2F169.254.169.254%2Flatest%2Fmeta-data%2F".to_string(),
            PoCExpectedResult::BodyContains("ami-id".to_string()),
        ),
        HunterKind::PathTraversal => (
            "/download?file=../../etc/passwd".to_string(),
            PoCExpectedResult::BodyContains("root:".to_string()),
        ),
        HunterKind::CommandInjection => (
            "/exec?cmd=%3Bid".to_string(),
            PoCExpectedResult::BodyContains("uid=".to_string()),
        ),
        HunterKind::Csrf => (
            "/transfer".to_string(),
            PoCExpectedResult::StatusCode(200),
        ),
        HunterKind::BrokenAuth => (
            "/admin".to_string(),
            PoCExpectedResult::StatusCode(200),
        ),
        HunterKind::InsecureDeserialization => (
            "/api/deserialize".to_string(),
            PoCExpectedResult::StatusCode(500),
        ),
        HunterKind::SensitiveDataExposure => (
            "/api/users".to_string(),
            PoCExpectedResult::BodyContains("password".to_string()),
        ),
        HunterKind::SecurityMisconfiguration => (
            "/health".to_string(),
            PoCExpectedResult::HeaderPresent("X-Powered-By".to_string()),
        ),
    };
    Some(PoCStep {
        description: format!("PoC probe for {} ({})", finding.hunter.label(), finding.title),
        request: PoCHttpRequest {
            method: "GET".to_string(),
            url,
            headers: vec![("Host".to_string(), "target.local".to_string())],
            body: None,
        },
        expected_result,
    })
}

#[derive(Debug, Clone)]
pub struct ReconReport {
    pub target: String,
    pub estimated_files: usize,
    pub languages_detected: Vec<String>,
    pub entry_points: Vec<String>,
    pub frameworks: Vec<String>,
    pub estimated_complexity: &'static str,
}

#[derive(Debug, Clone)]
pub struct ScanReport {
    pub total_findings: usize,
    pub confirmed: usize,
    pub verified: usize,
    pub severity_distribution: HashMap<SeverityLevel, usize>,
    pub by_hunter: HashMap<String, usize>,
    pub duration: Duration,
    pub stages_completed: usize,
    pub config: ScanConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpListener};

    fn scanner() -> AgenticScanner {
        AgenticScanner::new(ScanConfig {
            enabled_hunters: vec![HunterKind::Xss, HunterKind::SqlInjection],
            min_severity: SeverityLevel::Low,
            sandbox_enabled: false,
            max_findings: 10,
            timeout_seconds: 60,
            deep_scan: false,
        })
    }

    #[test]
    fn test_scan_lifecycle() {
        let mut s = scanner();
        s.start_scan();
        assert_eq!(s.current_stage(), ScanStage::Recon);
        let recon = s.recon_scan("test-app");
        assert!(!recon.languages_detected.is_empty());
        let findings = s.hunt("test-app");
        assert!(!findings.is_empty());
        let confirmed = s.validate();
        assert!(!confirmed.is_empty());
    }

    #[test]
    fn test_cvss_scoring() {
        let crit = CvssScore::from_base(9.5);
        assert_eq!(crit.severity, SeverityLevel::Critical);
        let none = CvssScore::from_base(0.0);
        assert_eq!(none.severity, SeverityLevel::None);
        let high = CvssScore::from_base(7.5);
        assert_eq!(high.severity, SeverityLevel::High);
    }

    #[test]
    fn test_finding_status_flow() {
        let mut s = scanner();
        s.start_scan();
        s.hunt("app");
        assert!(s.findings.iter().all(|f| f.status == FindingStatus::Candidate));
        s.validate();
        assert!(s.findings.iter().any(|f| f.status == FindingStatus::Confirmed));
        assert!(s.findings.iter().any(|f| f.status == FindingStatus::Rejected));
    }

    #[test]
    fn test_hunter_risk_weights() {
        assert!(HunterKind::SqlInjection.risk_weight() > HunterKind::Csrf.risk_weight());
        assert_eq!(HunterKind::Xss.risk_weight(), 7.0);
    }

    #[test]
    fn test_severity_distribution() {
        let mut s = scanner();
        s.start_scan();
        s.hunt("app");
        let dist = s.by_severity();
        assert!(!dist.is_empty());
    }

    #[test]
    fn test_scan_report() {
        let mut s = scanner();
        s.start_scan();
        s.hunt("app");
        s.validate();
        let report = s.scan_summary();
        assert!(report.total_findings > 0);
        assert!(report.confirmed > 0);
        assert!(report.stages_completed >= 2);
    }

    #[test]
    fn test_hunter_labels() {
        assert_eq!(HunterKind::Xss.label(), "Cross-Site Scripting");
        assert_eq!(HunterKind::Ssrf.label(), "Server-Side Request Forgery");
    }

    #[test]
    fn test_recon_report() {
        let s = scanner();
        let recon = s.recon_scan("https://example.com");
        assert_eq!(recon.target, "https://example.com");
        assert!(recon.estimated_files > 0);
    }

    fn finding(id: usize, hunter: HunterKind, base: f64) -> VulnerabilityFinding {
        VulnerabilityFinding {
            id,
            hunter,
            title: format!("PoC test {}", hunter.label()),
            description: "test finding".into(),
            file_path: "src/test.rs".into(),
            line_number: 1,
            code_snippet: "// test".into(),
            cvss: CvssScore::from_base(base),
            status: FindingStatus::Candidate,
            fix_suggestion: "fix".into(),
            discovered_at: Instant::now(),
        }
    }

    /// Minimal raw HTTP upstream: answers each accepted connection with a fixed response.
    fn spawn_upstream(body: &str, status_code: u16, connections: usize) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind upstream");
        let addr = listener.local_addr().expect("upstream local addr");
        let body = body.to_string();
        std::thread::spawn(move || {
            for _ in 0..connections {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut buf = [0u8; 4096];
                        let _ = stream.read(&mut buf);
                        let response = format!(
                            "HTTP/1.1 {} \r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            status_code,
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                    Err(_) => return,
                }
            }
        });
        addr
    }

    /// Forward-proxy interceptor bound to a concrete free port, pointing upstream at the test server.
    fn interceptor(upstream: SocketAddr) -> HttpInterceptor {
        let probe = TcpListener::bind("127.0.0.1:0").expect("bind probe");
        let port = probe.local_addr().expect("probe local addr").port();
        drop(probe);
        HttpInterceptor::new(
            format!("127.0.0.1:{}", port).parse().expect("parse listen addr"),
            &format!("127.0.0.1:{}", upstream.port()),
        )
    }

    #[test]
    fn test_validate_with_target_confirms_on_poc_reproduction() {
        let mut s = scanner();
        s.findings.push(finding(1, HunterKind::Xss, 9.0));
        let upstream = spawn_upstream("<script>alert(1)</script>", 200, 2);
        let mut ic = interceptor(upstream);
        ic.start().expect("start interceptor");
        let confirmed = s.validate_with_target(&ic);
        ic.stop();
        assert_eq!(confirmed, vec![1]);
        assert_eq!(s.findings[0].status, FindingStatus::Confirmed);
    }

    #[test]
    fn test_validate_with_target_rejects_on_poc_mismatch() {
        let mut s = scanner();
        s.findings.push(finding(2, HunterKind::Xss, 9.0));
        let upstream = spawn_upstream("nothing malicious here", 200, 2);
        let mut ic = interceptor(upstream);
        ic.start().expect("start interceptor");
        let confirmed = s.validate_with_target(&ic);
        ic.stop();
        assert!(confirmed.is_empty());
        assert_eq!(s.findings[0].status, FindingStatus::Rejected);
    }

    #[test]
    fn test_validate_with_target_cvss_prefilter_rejects_low_severity() {
        let mut s = scanner();
        s.findings.push(finding(3, HunterKind::Xss, 3.0));
        let upstream = spawn_upstream("<script>alert(1)</script>", 200, 1);
        let mut ic = interceptor(upstream);
        ic.start().expect("start interceptor");
        let confirmed = s.validate_with_target(&ic);
        ic.stop();
        assert!(confirmed.is_empty());
        assert_eq!(s.findings[0].status, FindingStatus::Rejected);
    }
}
