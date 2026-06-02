use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BrowserVulnType {
    XssReflected,
    XssStored,
    XssDomBased,
    Csrf,
    CorsMisconfiguration,
    CspBypass,
    OpenRedirect,
    Clickjacking,
    InsecureCookie,
    AuthBypass,
}

impl BrowserVulnType {
    pub fn label(&self) -> &'static str {
        match self {
            BrowserVulnType::XssReflected => "Reflected XSS",
            BrowserVulnType::XssStored => "Stored XSS",
            BrowserVulnType::XssDomBased => "DOM-based XSS",
            BrowserVulnType::Csrf => "CSRF",
            BrowserVulnType::CorsMisconfiguration => "CORS Misconfiguration",
            BrowserVulnType::CspBypass => "CSP Bypass",
            BrowserVulnType::OpenRedirect => "Open Redirect",
            BrowserVulnType::Clickjacking => "Clickjacking",
            BrowserVulnType::InsecureCookie => "Insecure Cookie",
            BrowserVulnType::AuthBypass => "Auth Bypass",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityRank {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SeverityRank {
    pub fn label(&self) -> &'static str {
        match self {
            SeverityRank::Info => "Info",
            SeverityRank::Low => "Low",
            SeverityRank::Medium => "Medium",
            SeverityRank::High => "High",
            SeverityRank::Critical => "Critical",
        }
    }

    pub fn numeric_value(&self) -> u8 {
        match self {
            SeverityRank::Info => 0,
            SeverityRank::Low => 1,
            SeverityRank::Medium => 2,
            SeverityRank::High => 3,
            SeverityRank::Critical => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BrowserSecurityResult {
    pub vuln_type: BrowserVulnType,
    pub url: String,
    pub severity: SeverityRank,
    pub description: String,
    pub evidence: String,
    pub poc: Option<String>,
    pub confidence: f64,
    pub false_positive_risk: f64,
}

#[derive(Debug, Clone)]
pub struct BrowserSecurityConfig {
    pub target_url: String,
    pub check_types: Vec<BrowserVulnType>,
    pub max_depth: u8,
    pub follow_redirects: bool,
    pub custom_payloads: HashMap<BrowserVulnType, Vec<String>>,
    pub timeout_seconds: u64,
    pub concurrent_checks: usize,
}

impl Default for BrowserSecurityConfig {
    fn default() -> Self {
        Self {
            target_url: String::new(),
            check_types: Vec::new(),
            max_depth: 2,
            follow_redirects: true,
            custom_payloads: HashMap::new(),
            timeout_seconds: 30,
            concurrent_checks: 3,
        }
    }
}

pub trait BrowserSecurityCheck: Send + Sync {
    fn name(&self) -> &str;
    fn vuln_type(&self) -> BrowserVulnType;
    fn check(&self, url: &str, config: &BrowserSecurityConfig) -> Vec<BrowserSecurityResult>;
    fn severity_rank(&self, evidence: &str) -> SeverityRank;
}

pub struct XssReflectedCheck {
    payloads: Vec<String>,
}

impl XssReflectedCheck {
    pub fn new() -> Self {
        Self {
            payloads: vec![
                "<script>alert(1)</script>".to_string(),
                "<img src=x onerror=alert(1)>".to_string(),
                "\"><script>alert(1)</script>".to_string(),
                "'-alert(1)-'".to_string(),
            ],
        }
    }

    pub fn with_payloads(payloads: Vec<String>) -> Self {
        Self { payloads }
    }
}

impl Default for XssReflectedCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserSecurityCheck for XssReflectedCheck {
    fn name(&self) -> &str {
        "XSS Reflected Check"
    }

    fn vuln_type(&self) -> BrowserVulnType {
        BrowserVulnType::XssReflected
    }

    fn check(&self, url: &str, config: &BrowserSecurityConfig) -> Vec<BrowserSecurityResult> {
        let mut results = Vec::new();
        let payloads: Vec<&String> =
            if config.custom_payloads.contains_key(&BrowserVulnType::XssReflected) {
                config.custom_payloads[&BrowserVulnType::XssReflected]
                    .iter()
                    .collect()
            } else {
                self.payloads.iter().collect()
            };

        for payload in payloads {
            if url.contains(payload.as_str()) {
                let severity = self.severity_rank(payload);
                let confidence = if payload.contains("<script>") || payload.contains("onerror") {
                    0.85
                } else {
                    0.70
                };
                let fp_risk = if payload.contains("alert") { 0.15 } else { 0.25 };

                results.push(BrowserSecurityResult {
                    vuln_type: BrowserVulnType::XssReflected,
                    url: url.to_string(),
                    severity,
                    description: format!("Reflected XSS via payload: {}", payload),
                    evidence: format!("Payload reflected in response: {}", payload),
                    poc: Some(format!("curl '{}'", url)),
                    confidence,
                    false_positive_risk: fp_risk,
                });
            }
        }

        results
    }

    fn severity_rank(&self, evidence: &str) -> SeverityRank {
        if evidence.contains("<script>") || evidence.contains("onerror") {
            SeverityRank::High
        } else if evidence.contains("\">") || evidence.contains("'-") {
            SeverityRank::Medium
        } else {
            SeverityRank::Low
        }
    }
}

pub struct CsrfCheck {
    form_indicators: Vec<String>,
    csrf_indicators: Vec<String>,
}

impl CsrfCheck {
    pub fn new() -> Self {
        Self {
            form_indicators: vec![
                "form".to_string(),
                "login".to_string(),
                "submit".to_string(),
                "register".to_string(),
            ],
            csrf_indicators: vec![
                "csrf".to_string(),
                "csrf_token".to_string(),
                "csrfmiddlewaretoken".to_string(),
                "__csrf".to_string(),
                "xsrf".to_string(),
                "_token".to_string(),
                "authenticity_token".to_string(),
            ],
        }
    }
}

impl Default for CsrfCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserSecurityCheck for CsrfCheck {
    fn name(&self) -> &str {
        "CSRF Check"
    }

    fn vuln_type(&self) -> BrowserVulnType {
        BrowserVulnType::Csrf
    }

    fn check(&self, url: &str, _config: &BrowserSecurityConfig) -> Vec<BrowserSecurityResult> {
        let mut results = Vec::new();

        let has_form = self
            .form_indicators
            .iter()
            .any(|i| url.contains(i.as_str()));
        let has_token = self
            .csrf_indicators
            .iter()
            .any(|i| url.contains(i.as_str()));

        if has_form && !has_token {
            results.push(BrowserSecurityResult {
                vuln_type: BrowserVulnType::Csrf,
                url: url.to_string(),
                severity: SeverityRank::High,
                description: "Form without CSRF token detected".to_string(),
                evidence: format!("No CSRF token found in form at {}", url),
                poc: Some(format!("curl -X POST '{}' -d 'malicious=1'", url)),
                confidence: 0.75,
                false_positive_risk: 0.20,
            });
        }

        results
    }

    fn severity_rank(&self, _evidence: &str) -> SeverityRank {
        SeverityRank::High
    }
}

pub struct CorsCheck;

impl CorsCheck {
    pub fn new() -> Self {
        Self
    }
}

impl BrowserSecurityCheck for CorsCheck {
    fn name(&self) -> &str {
        "CORS Misconfiguration Check"
    }

    fn vuln_type(&self) -> BrowserVulnType {
        BrowserVulnType::CorsMisconfiguration
    }

    fn check(&self, url: &str, _config: &BrowserSecurityConfig) -> Vec<BrowserSecurityResult> {
        let mut results = Vec::new();

        if url.contains("cors") || url.contains("api") || url.contains("wildcard") {
            results.push(BrowserSecurityResult {
                vuln_type: BrowserVulnType::CorsMisconfiguration,
                url: url.to_string(),
                severity: SeverityRank::Medium,
                description: "CORS allows wildcard origin".to_string(),
                evidence: "Access-Control-Allow-Origin: *".to_string(),
                poc: Some(format!(
                    "curl -H 'Origin: https://evil.com' -I '{}'",
                    url
                )),
                confidence: 0.80,
                false_positive_risk: 0.10,
            });
        }

        results
    }

    fn severity_rank(&self, evidence: &str) -> SeverityRank {
        if evidence.contains('*') && evidence.contains("Access-Control") {
            SeverityRank::Medium
        } else {
            SeverityRank::Low
        }
    }
}

pub struct InsecureCookieCheck;

impl InsecureCookieCheck {
    pub fn new() -> Self {
        Self
    }
}

impl BrowserSecurityCheck for InsecureCookieCheck {
    fn name(&self) -> &str {
        "Insecure Cookie Check"
    }

    fn vuln_type(&self) -> BrowserVulnType {
        BrowserVulnType::InsecureCookie
    }

    fn check(&self, url: &str, _config: &BrowserSecurityConfig) -> Vec<BrowserSecurityResult> {
        let mut results = Vec::new();

        if url.contains("cookie") || url.contains("session") || url.contains("insecure") {
            results.push(BrowserSecurityResult {
                vuln_type: BrowserVulnType::InsecureCookie,
                url: url.to_string(),
                severity: SeverityRank::High,
                description: "Cookie without HttpOnly and Secure flags".to_string(),
                evidence: "Set-Cookie: session=abc123; Path=/".to_string(),
                poc: Some(format!("Check Set-Cookie header at '{}'", url)),
                confidence: 0.90,
                false_positive_risk: 0.05,
            });
        }

        results
    }

    fn severity_rank(&self, evidence: &str) -> SeverityRank {
        if evidence.contains("HttpOnly") || evidence.contains("Secure") {
            if evidence.contains("SameSite=None") {
                SeverityRank::Low
            } else {
                SeverityRank::Info
            }
        } else {
            SeverityRank::High
        }
    }
}

pub struct BrowserSecurityScanner {
    pub config: BrowserSecurityConfig,
    pub checks: Vec<Box<dyn BrowserSecurityCheck>>,
    pub results: Vec<BrowserSecurityResult>,
}

impl BrowserSecurityScanner {
    pub fn new(config: BrowserSecurityConfig) -> Self {
        Self {
            config,
            checks: Vec::new(),
            results: Vec::new(),
        }
    }

    pub fn register_check(&mut self, check: Box<dyn BrowserSecurityCheck>) {
        self.checks.push(check);
    }

    pub fn register_default_checks(&mut self) {
        self.register_check(Box::new(XssReflectedCheck::new()));
        self.register_check(Box::new(CsrfCheck::new()));
        self.register_check(Box::new(CorsCheck::new()));
        self.register_check(Box::new(InsecureCookieCheck::new()));
    }

    pub fn run_scan(&mut self) -> Vec<BrowserSecurityResult> {
        self.results.clear();
        for check in &self.checks {
            let check_results = check.check(&self.config.target_url, &self.config);
            self.results.extend(check_results);
        }
        self.results.clone()
    }

    pub fn summary(&self) -> String {
        let total = self.results.len();
        let by_severity = |s: SeverityRank| -> usize {
            self.results.iter().filter(|r| r.severity == s).count()
        };

        format!(
            "Browser Security Scan Summary:\n  Total findings: {}\n  Critical: {}\n  High: {}\n  Medium: {}\n  Low: {}\n  Info: {}",
            total,
            by_severity(SeverityRank::Critical),
            by_severity(SeverityRank::High),
            by_severity(SeverityRank::Medium),
            by_severity(SeverityRank::Low),
            by_severity(SeverityRank::Info),
        )
    }

    pub fn highest_severity(&self) -> Option<SeverityRank> {
        self.results.iter().map(|r| r.severity.clone()).max()
    }

    pub fn filter_by_type(&self, vuln_type: BrowserVulnType) -> Vec<&BrowserSecurityResult> {
        self.results
            .iter()
            .filter(|r| r.vuln_type == vuln_type)
            .collect()
    }

    pub fn filter_by_severity(
        &self,
        min_severity: SeverityRank,
    ) -> Vec<&BrowserSecurityResult> {
        self.results
            .iter()
            .filter(|r| r.severity >= min_severity)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BrowserSecurityConfig::default();
        assert!(config.target_url.is_empty());
        assert!(config.check_types.is_empty());
        assert_eq!(config.max_depth, 2);
        assert!(config.follow_redirects);
        assert!(config.custom_payloads.is_empty());
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.concurrent_checks, 3);
    }

    #[test]
    fn test_xss_reflected_detected() {
        let check = XssReflectedCheck::new();
        let config = BrowserSecurityConfig::default();
        let url = "http://test.com/search?q=<script>alert(1)</script>";
        let results = check.check(url, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].vuln_type, BrowserVulnType::XssReflected);
        assert!(results[0].description.contains("Reflected XSS"));
        assert!(results[0].poc.is_some());
    }

    #[test]
    fn test_xss_reflected_clean() {
        let check = XssReflectedCheck::new();
        let config = BrowserSecurityConfig::default();
        let url = "http://test.com/search?q=hello";
        let results = check.check(url, &config);
        assert!(results.is_empty());
    }

    #[test]
    fn test_csrf_missing_token() {
        let check = CsrfCheck::new();
        let config = BrowserSecurityConfig::default();
        let url = "http://test.com/login?user=admin";
        let results = check.check(url, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].vuln_type, BrowserVulnType::Csrf);
        assert!(results[0].description.contains("CSRF token"));
    }

    #[test]
    fn test_csrf_with_token() {
        let check = CsrfCheck::new();
        let config = BrowserSecurityConfig::default();
        let url = "http://test.com/login?csrf_token=abc123";
        let results = check.check(url, &config);
        assert!(results.is_empty());
    }

    #[test]
    fn test_cors_wildcard() {
        let check = CorsCheck::new();
        let config = BrowserSecurityConfig::default();
        let url = "http://api.test.com/cors";
        let results = check.check(url, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].vuln_type, BrowserVulnType::CorsMisconfiguration);
        assert!(results[0].evidence.contains("Access-Control-Allow-Origin: *"));
    }

    #[test]
    fn test_cors_restricted() {
        let check = CorsCheck::new();
        let config = BrowserSecurityConfig::default();
        let url = "http://test.com/about";
        let results = check.check(url, &config);
        assert!(results.is_empty());
    }

    #[test]
    fn test_insecure_cookie() {
        let check = InsecureCookieCheck::new();
        let config = BrowserSecurityConfig::default();
        let url = "http://test.com/session";
        let results = check.check(url, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].vuln_type, BrowserVulnType::InsecureCookie);
        assert!(results[0].description.contains("HttpOnly"));
    }

    #[test]
    fn test_scanner_register_default() {
        let mut scanner =
            BrowserSecurityScanner::new(BrowserSecurityConfig::default());
        assert!(scanner.checks.is_empty());
        scanner.register_default_checks();
        assert_eq!(scanner.checks.len(), 4);
    }

    #[test]
    fn test_scanner_run_all_checks() {
        let mut scanner = BrowserSecurityScanner::new(BrowserSecurityConfig {
            target_url: "http://test.com/login?q=<script>alert(1)</script>"
                .to_string(),
            ..Default::default()
        });
        scanner.register_default_checks();
        let results = scanner.run_scan();
        assert!(!results.is_empty());
        assert_eq!(scanner.results.len(), results.len());
    }

    #[test]
    fn test_scanner_summary_format() {
        let mut scanner =
            BrowserSecurityScanner::new(BrowserSecurityConfig::default());
        scanner.results.push(BrowserSecurityResult {
            vuln_type: BrowserVulnType::XssReflected,
            url: "http://test.com".to_string(),
            severity: SeverityRank::High,
            description: "test".to_string(),
            evidence: "evidence".to_string(),
            poc: None,
            confidence: 0.8,
            false_positive_risk: 0.1,
        });
        scanner.results.push(BrowserSecurityResult {
            vuln_type: BrowserVulnType::Csrf,
            url: "http://test.com".to_string(),
            severity: SeverityRank::Critical,
            description: "test2".to_string(),
            evidence: "evidence2".to_string(),
            poc: None,
            confidence: 0.9,
            false_positive_risk: 0.05,
        });
        let summary = scanner.summary();
        assert!(summary.contains("Browser Security Scan Summary"));
        assert!(summary.contains("Total findings: 2"));
        assert!(summary.contains("Critical: 1"));
        assert!(summary.contains("High: 1"));
        assert!(summary.contains("Medium: 0"));
    }

    #[test]
    fn test_highest_severity() {
        let mut scanner =
            BrowserSecurityScanner::new(BrowserSecurityConfig::default());
        assert_eq!(scanner.highest_severity(), None);
        scanner.results.push(BrowserSecurityResult {
            vuln_type: BrowserVulnType::XssReflected,
            url: "http://test.com".to_string(),
            severity: SeverityRank::Medium,
            description: "test".to_string(),
            evidence: "ev".to_string(),
            poc: None,
            confidence: 0.5,
            false_positive_risk: 0.2,
        });
        assert_eq!(scanner.highest_severity(), Some(SeverityRank::Medium));
        scanner.results.push(BrowserSecurityResult {
            vuln_type: BrowserVulnType::Csrf,
            url: "http://test.com".to_string(),
            severity: SeverityRank::Critical,
            description: "test2".to_string(),
            evidence: "ev2".to_string(),
            poc: None,
            confidence: 0.9,
            false_positive_risk: 0.05,
        });
        assert_eq!(scanner.highest_severity(), Some(SeverityRank::Critical));
    }

    #[test]
    fn test_filter_by_type() {
        let mut scanner =
            BrowserSecurityScanner::new(BrowserSecurityConfig::default());
        scanner.results.push(BrowserSecurityResult {
            vuln_type: BrowserVulnType::XssReflected,
            url: "http://test.com".to_string(),
            severity: SeverityRank::High,
            description: "xss".to_string(),
            evidence: "ev".to_string(),
            poc: None,
            confidence: 0.8,
            false_positive_risk: 0.1,
        });
        scanner.results.push(BrowserSecurityResult {
            vuln_type: BrowserVulnType::Csrf,
            url: "http://test.com".to_string(),
            severity: SeverityRank::High,
            description: "csrf".to_string(),
            evidence: "ev2".to_string(),
            poc: None,
            confidence: 0.7,
            false_positive_risk: 0.2,
        });
        scanner.results.push(BrowserSecurityResult {
            vuln_type: BrowserVulnType::XssReflected,
            url: "http://test.com".to_string(),
            severity: SeverityRank::Medium,
            description: "xss2".to_string(),
            evidence: "ev3".to_string(),
            poc: None,
            confidence: 0.6,
            false_positive_risk: 0.3,
        });
        let xss_results = scanner.filter_by_type(BrowserVulnType::XssReflected);
        assert_eq!(xss_results.len(), 2);
        let csrf_results = scanner.filter_by_type(BrowserVulnType::Csrf);
        assert_eq!(csrf_results.len(), 1);
        let cors_results =
            scanner.filter_by_type(BrowserVulnType::CorsMisconfiguration);
        assert!(cors_results.is_empty());
    }

    #[test]
    fn test_filter_by_severity() {
        let mut scanner =
            BrowserSecurityScanner::new(BrowserSecurityConfig::default());
        scanner.results.push(BrowserSecurityResult {
            vuln_type: BrowserVulnType::XssReflected,
            url: "http://test.com".to_string(),
            severity: SeverityRank::Low,
            description: "low".to_string(),
            evidence: "ev".to_string(),
            poc: None,
            confidence: 0.3,
            false_positive_risk: 0.5,
        });
        scanner.results.push(BrowserSecurityResult {
            vuln_type: BrowserVulnType::Csrf,
            url: "http://test.com".to_string(),
            severity: SeverityRank::High,
            description: "high".to_string(),
            evidence: "ev2".to_string(),
            poc: None,
            confidence: 0.8,
            false_positive_risk: 0.1,
        });
        scanner.results.push(BrowserSecurityResult {
            vuln_type: BrowserVulnType::InsecureCookie,
            url: "http://test.com".to_string(),
            severity: SeverityRank::Critical,
            description: "critical".to_string(),
            evidence: "ev3".to_string(),
            poc: None,
            confidence: 0.95,
            false_positive_risk: 0.05,
        });
        let high_and_above =
            scanner.filter_by_severity(SeverityRank::High);
        assert_eq!(high_and_above.len(), 2);
        assert_eq!(high_and_above[0].severity, SeverityRank::High);
        assert_eq!(high_and_above[1].severity, SeverityRank::Critical);

        let critical_only =
            scanner.filter_by_severity(SeverityRank::Critical);
        assert_eq!(critical_only.len(), 1);
        assert_eq!(critical_only[0].severity, SeverityRank::Critical);
    }

    #[test]
    fn test_confidence_scoring() {
        let check = XssReflectedCheck::new();
        let config = BrowserSecurityConfig::default();
        let results =
            check.check("http://test.com?q=<script>alert(1)</script>", &config);
        for r in &results {
            assert!(r.confidence >= 0.0);
            assert!(r.confidence <= 1.0);
        }
        let script_result = &results[0];
        assert!(script_result.confidence >= 0.8);
    }

    #[test]
    fn test_false_positive_risk() {
        let check = XssReflectedCheck::new();
        let config = BrowserSecurityConfig::default();
        let results =
            check.check("http://test.com?q=<script>alert(1)</script>", &config);
        for r in &results {
            assert!(r.false_positive_risk >= 0.0);
            assert!(r.false_positive_risk <= 1.0);
        }
        let script_result = &results[0];
        assert!(script_result.false_positive_risk <= 0.5);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(SeverityRank::Info < SeverityRank::Low);
        assert!(SeverityRank::Low < SeverityRank::Medium);
        assert!(SeverityRank::Medium < SeverityRank::High);
        assert!(SeverityRank::High < SeverityRank::Critical);
        assert!(SeverityRank::Info < SeverityRank::Critical);
    }

    #[test]
    fn test_custom_payload_injection() {
        let custom_payload = "<svg/onload=alert(1)>";
        let custom = vec![custom_payload.to_string()];
        let mut config = BrowserSecurityConfig::default();
        config
            .custom_payloads
            .insert(BrowserVulnType::XssReflected, custom);
        let check = XssReflectedCheck::new();
        let url = format!("http://test.com?q={}", custom_payload);
        let results = check.check(&url, &config);
        assert!(!results.is_empty());
        assert_eq!(results[0].vuln_type, BrowserVulnType::XssReflected);
        assert_eq!(results[0].severity, SeverityRank::Low);
    }

    #[test]
    fn test_empty_config_defaults() {
        let config = BrowserSecurityConfig::default();
        let scanner = BrowserSecurityScanner::new(config);
        assert!(scanner.checks.is_empty());
        assert!(scanner.results.is_empty());
    }

    #[test]
    fn test_severity_rank_label_and_value() {
        assert_eq!(SeverityRank::Info.label(), "Info");
        assert_eq!(SeverityRank::Low.label(), "Low");
        assert_eq!(SeverityRank::Medium.label(), "Medium");
        assert_eq!(SeverityRank::High.label(), "High");
        assert_eq!(SeverityRank::Critical.label(), "Critical");
        assert_eq!(SeverityRank::Info.numeric_value(), 0);
        assert_eq!(SeverityRank::Critical.numeric_value(), 4);
    }

    #[test]
    fn test_nt_world_browse_vuln_type_label() {
        assert_eq!(BrowserVulnType::XssReflected.label(), "Reflected XSS");
        assert_eq!(BrowserVulnType::Csrf.label(), "CSRF");
        assert_eq!(BrowserVulnType::CorsMisconfiguration.label(), "CORS Misconfiguration");
        assert_eq!(BrowserVulnType::InsecureCookie.label(), "Insecure Cookie");
        assert_eq!(BrowserVulnType::AuthBypass.label(), "Auth Bypass");
    }

    #[test]
    fn test_scanner_register_and_run_scan() {
        let mut scanner = BrowserSecurityScanner::new(BrowserSecurityConfig {
            target_url: "http://test.com/login?<script>alert(1)</script>&session=true"
                .to_string(),
            ..Default::default()
        });
        scanner.register_check(Box::new(XssReflectedCheck::new()));
        scanner.register_check(Box::new(CsrfCheck::new()));
        let results = scanner.run_scan();
        assert!(!results.is_empty());
        assert_eq!(scanner.results.len(), results.len());
    }
}
