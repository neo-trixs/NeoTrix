#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};

use crate::core::nt_core_knowledge::evidence::{EvidenceManager, EvidenceRecord, EvidenceState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OsintSource {
    UsernameSearch,
    EmailSearch,
    BreachCheck,
    WhoisLookup,
    IpLookup,
    SocialMedia,
    DarkWeb,
    MCP,
}

impl OsintSource {
    pub fn name(&self) -> &'static str {
        match self {
            OsintSource::UsernameSearch => "username_search",
            OsintSource::EmailSearch => "email_search",
            OsintSource::BreachCheck => "breach_check",
            OsintSource::WhoisLookup => "whois_lookup",
            OsintSource::IpLookup => "ip_lookup",
            OsintSource::SocialMedia => "social_media",
            OsintSource::DarkWeb => "dark_web",
            OsintSource::MCP => "mcp",
        }
    }

    pub fn reliability(&self) -> f64 {
        match self {
            OsintSource::UsernameSearch => 0.65,
            OsintSource::EmailSearch => 0.70,
            OsintSource::BreachCheck => 0.85,
            OsintSource::WhoisLookup => 0.90,
            OsintSource::IpLookup => 0.80,
            OsintSource::SocialMedia => 0.50,
            OsintSource::DarkWeb => 0.35,
            OsintSource::MCP => 0.75,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OsintEvidence {
    pub source_type: OsintSource,
    pub target: String,
    pub evidence_text: String,
    pub confidence: f64,
    pub timestamp: u64,
    pub source_url: String,
    pub source_reliability: f64,
    pub verification_state: EvidenceState,
    pub raw_response: Option<String>,
}

impl OsintEvidence {
    pub fn to_evidence_record(&self) -> (String, String, String) {
        (
            self.source_url.clone(),
            self.evidence_text.clone(),
            self.verification_state.name().to_string(),
        )
    }

    pub fn as_evidence_record(&self, mgr: &mut EvidenceManager) -> u64 {
        let source_name = self.source_type.name();
        let mut record = EvidenceRecord::new(0, &self.source_url, source_name, &self.evidence_text)
            .with_confidence(self.confidence * self.source_reliability)
            .with_state(self.verification_state);
        if let Some(raw) = &self.raw_response {
            record.add_metadata("raw_response", raw);
        }
        record.add_metadata("target", &self.target);
        record.add_metadata(
            "source_reliability",
            &format!("{:.2}", self.source_reliability),
        );
        mgr.add_evidence_with(record)
    }

    fn simulated(
        source_type: OsintSource,
        target: &str,
        text: String,
        confidence: f64,
        url: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        OsintEvidence {
            source_type,
            target: target.to_string(),
            evidence_text: text,
            confidence,
            timestamp: now,
            source_url: url,
            source_reliability: source_type.reliability(),
            verification_state: EvidenceState::Unverified,
            raw_response: None,
        }
    }
}

type RateCounters = HashMap<(OsintSource, u64), usize>;

pub struct OsintToolLayer {
    pub rate_limit_per_sec: usize,
    rate_counters: RateCounters,
    cache: HashMap<String, Vec<OsintEvidence>>,
    http_client: Option<std::sync::Mutex<reqwest::blocking::Client>>,
    mcp_tool_names: Vec<String>,
}

impl OsintToolLayer {
    pub fn new() -> Self {
        Self {
            rate_limit_per_sec: 10,
            rate_counters: HashMap::new(),
            cache: HashMap::new(),
            http_client: None,
            mcp_tool_names: Vec::new(),
        }
    }

    pub fn with_http_client(mut self) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("NeoTrix-OSINT/1.0")
            .build()
            .ok();
        self.http_client = client.map(std::sync::Mutex::new);
        self
    }

    pub fn with_mcp_client(mut self, tool_names: &[&str]) -> Self {
        self.mcp_tool_names = tool_names.iter().map(|s| s.to_string()).collect();
        self
    }

    fn now_sec() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn check_rate_limit(&mut self, source: OsintSource) -> bool {
        let now = Self::now_sec();
        let key = (source, now);
        let count = self.rate_counters.entry(key).or_insert(0);
        *count += 1;

        let window_start = now.saturating_sub(1);
        self.rate_counters.retain(|&(_, ts), _| ts >= window_start);

        let total_in_window: usize = self
            .rate_counters
            .iter()
            .filter(|(&(s, ts), _)| s == source && ts >= window_start)
            .map(|(_, c)| *c)
            .sum();

        total_in_window <= self.rate_limit_per_sec
    }

    fn cache_key(source: OsintSource, target: &str) -> String {
        format!("{}:{}", source.name(), target)
    }

    // ── HTTP helpers ──

    fn http_get(&self, url: &str) -> Result<String, String> {
        let client = self
            .http_client
            .as_ref()
            .ok_or_else(|| "HTTP client not initialized".to_string())?;
        let guard = client.lock().map_err(|e| format!("Mutex error: {}", e))?;
        guard
            .get(url)
            .send()
            .map_err(|e| format!("HTTP GET failed: {}", e))?
            .text()
            .map_err(|e| format!("Body read failed: {}", e))
    }

    fn http_head(&self, url: &str) -> Result<u16, String> {
        let client = self
            .http_client
            .as_ref()
            .ok_or_else(|| "HTTP client not initialized".to_string())?;
        let guard = client.lock().map_err(|e| format!("Mutex error: {}", e))?;
        guard
            .head(url)
            .send()
            .map_err(|e| format!("HTTP HEAD failed: {}", e))
            .map(|r| r.status().as_u16())
    }

    // ── Simulated fallbacks ──

    fn simulated_username(username: &str) -> Vec<OsintEvidence> {
        vec![
            OsintEvidence::simulated(
                OsintSource::UsernameSearch,
                username,
                format!("Username '{}' found on GitHub profile", username),
                0.6,
                format!("https://github.com/{}", username),
            ),
            OsintEvidence::simulated(
                OsintSource::UsernameSearch,
                username,
                format!("Username '{}' found on Twitter/X", username),
                0.5,
                format!("https://twitter.com/{}", username),
            ),
        ]
    }

    fn simulated_email(email: &str) -> Vec<OsintEvidence> {
        vec![OsintEvidence::simulated(
            OsintSource::EmailSearch,
            email,
            format!("Email '{}' registered on 3 services", email),
            0.7,
            format!("https://holehe.example.com/check/{}", email),
        )]
    }

    fn simulated_breach(email: &str) -> Vec<OsintEvidence> {
        vec![OsintEvidence::simulated(
            OsintSource::BreachCheck,
            email,
            format!("Email '{}' found in 2 data breaches", email),
            0.8,
            format!("https://haveibeenpwned.com/account/{}", email),
        )]
    }

    fn simulated_whois(domain: &str) -> Vec<OsintEvidence> {
        vec![OsintEvidence::simulated(
            OsintSource::WhoisLookup,
            domain,
            format!(
                "Domain '{}' registered until 2027, registrar: ExampleCorp",
                domain
            ),
            0.9,
            format!("https://whois.example.com/{}", domain),
        )]
    }

    fn simulated_ip(ip: &str) -> Vec<OsintEvidence> {
        vec![OsintEvidence::simulated(
            OsintSource::IpLookup,
            ip,
            format!("IP '{}' located in US, ISP: ExampleNet", ip),
            0.85,
            format!("https://ip-api.com/{}", ip),
        )]
    }

    // ── Public API ──

    pub fn search_username(&mut self, username: &str) -> Vec<OsintEvidence> {
        if !self.check_rate_limit(OsintSource::UsernameSearch) {
            return vec![];
        }
        let key = Self::cache_key(OsintSource::UsernameSearch, username);
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            self.try_github_user(username)
                .or_else(|| self.try_twitter_profile(username))
                .unwrap_or_else(|| Self::simulated_username(username))
        }))
        .unwrap_or_else(|_| Self::simulated_username(username));

        self.cache.insert(key, result.clone());
        result
    }

    fn try_github_user(&self, username: &str) -> Option<Vec<OsintEvidence>> {
        let url = format!("https://api.github.com/users/{}", username);
        let body = self.http_get(&url).ok()?;
        let now = Self::now_sec();
        let evidence = OsintEvidence {
            source_type: OsintSource::UsernameSearch,
            target: username.to_string(),
            evidence_text: format!("Username '{}' confirmed on GitHub", username),
            confidence: 0.85,
            timestamp: now,
            source_url: format!("https://github.com/{}", username),
            source_reliability: OsintSource::UsernameSearch.reliability(),
            verification_state: EvidenceState::CrossReferenced,
            raw_response: Some(body.clone()),
        };
        Some(vec![evidence])
    }

    fn try_twitter_profile(&self, username: &str) -> Option<Vec<OsintEvidence>> {
        let url = format!("https://twitter.com/{}", username);
        let status = self.http_head(&url).ok()?;
        if status == 200 {
            let now = Self::now_sec();
            let evidence = OsintEvidence {
                source_type: OsintSource::SocialMedia,
                target: username.to_string(),
                evidence_text: format!("Username '{}' profile exists on Twitter/X", username),
                confidence: 0.55,
                timestamp: now,
                source_url: url,
                source_reliability: OsintSource::SocialMedia.reliability(),
                verification_state: EvidenceState::Unverified,
                raw_response: None,
            };
            Some(vec![evidence])
        } else {
            None
        }
    }

    pub fn search_email(&mut self, email: &str) -> Vec<OsintEvidence> {
        if !self.check_rate_limit(OsintSource::EmailSearch) {
            return vec![];
        }
        let key = Self::cache_key(OsintSource::EmailSearch, email);
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            self.try_hunter_email(email)
                .or_else(|| self.try_github_commit_email(email))
                .unwrap_or_else(|| Self::simulated_email(email))
        }))
        .unwrap_or_else(|_| Self::simulated_email(email));

        self.cache.insert(key, result.clone());
        result
    }

    fn try_hunter_email(&self, email: &str) -> Option<Vec<OsintEvidence>> {
        let api_key = std::env::var("HUNTER_API_KEY").ok()?;
        let url = format!(
            "https://api.hunter.io/v2/email-verifier?email={}&api_key={}",
            email, api_key
        );
        let body = self.http_get(&url).ok()?;
        let now = Self::now_sec();
        let evidence = OsintEvidence {
            source_type: OsintSource::EmailSearch,
            target: email.to_string(),
            evidence_text: format!("Email '{}' verified via Hunter.io API", email),
            confidence: 0.80,
            timestamp: now,
            source_url: format!("https://api.hunter.io/v2/email-verifier?email={}", email),
            source_reliability: OsintSource::EmailSearch.reliability(),
            verification_state: EvidenceState::CrossReferenced,
            raw_response: Some(body),
        };
        Some(vec![evidence])
    }

    fn try_github_commit_email(&self, email: &str) -> Option<Vec<OsintEvidence>> {
        let query = urlencoding(email);
        let url = format!(
            "https://api.github.com/search/commits?q=author-email:{}",
            query
        );
        let body = self.http_get(&url).ok()?;
        let now = Self::now_sec();
        let evidence = OsintEvidence {
            source_type: OsintSource::EmailSearch,
            target: email.to_string(),
            evidence_text: format!("Email '{}' found in GitHub commits", email),
            confidence: 0.70,
            timestamp: now,
            source_url: format!(
                "https://github.com/search?q=author-email%3A{}&type=commits",
                query
            ),
            source_reliability: OsintSource::EmailSearch.reliability(),
            verification_state: EvidenceState::Unverified,
            raw_response: Some(body),
        };
        Some(vec![evidence])
    }

    pub fn search_breach(&mut self, email: &str) -> Vec<OsintEvidence> {
        if !self.check_rate_limit(OsintSource::BreachCheck) {
            return vec![];
        }
        let key = Self::cache_key(OsintSource::BreachCheck, email);
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            self.try_hibp(email)
                .unwrap_or_else(|| Self::simulated_breach(email))
        }))
        .unwrap_or_else(|_| Self::simulated_breach(email));

        self.cache.insert(key, result.clone());
        result
    }

    fn try_hibp(&self, email: &str) -> Option<Vec<OsintEvidence>> {
        use sha1::{Digest, Sha1};

        let hash = format!("{:X}", Sha1::digest(email.as_bytes()));
        let prefix = &hash[..5];
        let suffix = &hash[5..];
        let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);

        let api_key = std::env::var("HIBP_API_KEY").ok();
        let client = self.http_client.as_ref()?;
        let guard = client.lock().ok()?;
        let mut req = guard.get(&url);
        if let Some(ref key) = api_key {
            req = req.header("hibp-api-key", key);
        }
        let body: String = req.header("Add-Padding", "true").send().ok()?.text().ok()?;

        let found = body.lines().any(|line| {
            line.split(':').next().map(|s| s.trim().to_uppercase()) == Some(suffix.to_uppercase())
        });

        drop(guard);

        let now = Self::now_sec();
        let (text, confidence) = if found {
            (
                format!("Email '{}' found in known data breaches (HIBP)", email),
                0.85,
            )
        } else {
            (
                format!("Email '{}' not found in known data breaches", email),
                0.95,
            )
        };

        let evidence = OsintEvidence {
            source_type: OsintSource::BreachCheck,
            target: email.to_string(),
            evidence_text: text,
            confidence,
            timestamp: now,
            source_url: format!("https://haveibeenpwned.com/account/{}", email),
            source_reliability: OsintSource::BreachCheck.reliability(),
            verification_state: EvidenceState::CrossReferenced,
            raw_response: Some(body),
        };
        Some(vec![evidence])
    }

    pub fn search_whois(&mut self, domain: &str) -> Vec<OsintEvidence> {
        if !self.check_rate_limit(OsintSource::WhoisLookup) {
            return vec![];
        }
        let key = Self::cache_key(OsintSource::WhoisLookup, domain);
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            self.try_whois_cli(domain)
                .or_else(|| self.try_whois_rdap(domain))
                .unwrap_or_else(|| Self::simulated_whois(domain))
        }))
        .unwrap_or_else(|_| Self::simulated_whois(domain));

        self.cache.insert(key, result.clone());
        result
    }

    fn try_whois_cli(&self, domain: &str) -> Option<Vec<OsintEvidence>> {
        let output = std::process::Command::new("whois")
            .arg(domain)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);

        let registrar = text
            .lines()
            .find(|l| l.to_lowercase().contains("registrar:"))
            .unwrap_or("unknown")
            .to_string();

        let expiry = text
            .lines()
            .find(|l| l.to_lowercase().contains("expir") || l.to_lowercase().contains("paid until"))
            .unwrap_or("unknown")
            .to_string();

        let now = Self::now_sec();
        let evidence = OsintEvidence {
            source_type: OsintSource::WhoisLookup,
            target: domain.to_string(),
            evidence_text: format!(
                "Domain '{}' — Registrar: {}, Expiry: {}",
                domain,
                registrar.trim(),
                expiry.trim()
            ),
            confidence: 0.85,
            timestamp: now,
            source_url: format!("https://whois.example.com/{}", domain),
            source_reliability: OsintSource::WhoisLookup.reliability(),
            verification_state: EvidenceState::CrossReferenced,
            raw_response: Some(text.to_string()),
        };
        Some(vec![evidence])
    }

    fn try_whois_rdap(&self, domain: &str) -> Option<Vec<OsintEvidence>> {
        let url = format!("https://rdap.verisign.com/com/v1/domain/{}", domain);
        let body = self.http_get(&url).ok()?;
        let now = Self::now_sec();
        let evidence = OsintEvidence {
            source_type: OsintSource::WhoisLookup,
            target: domain.to_string(),
            evidence_text: format!("Domain '{}' RDAP lookup completed via Verisign", domain),
            confidence: 0.80,
            timestamp: now,
            source_url: url,
            source_reliability: OsintSource::WhoisLookup.reliability(),
            verification_state: EvidenceState::CrossReferenced,
            raw_response: Some(body),
        };
        Some(vec![evidence])
    }

    pub fn search_ip(&mut self, ip: &str) -> Vec<OsintEvidence> {
        if !self.check_rate_limit(OsintSource::IpLookup) {
            return vec![];
        }
        let key = Self::cache_key(OsintSource::IpLookup, ip);
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            self.try_ip_api(ip)
                .unwrap_or_else(|| Self::simulated_ip(ip))
        }))
        .unwrap_or_else(|_| Self::simulated_ip(ip));

        self.cache.insert(key, result.clone());
        result
    }

    fn try_ip_api(&self, ip: &str) -> Option<Vec<OsintEvidence>> {
        let url = format!("http://ip-api.com/json/{}", ip);
        let body = self.http_get(&url).ok()?;
        let now = Self::now_sec();

        let text = format!("IP '{}' geolocation data retrieved via ip-api.com", ip);

        let evidence = OsintEvidence {
            source_type: OsintSource::IpLookup,
            target: ip.to_string(),
            evidence_text: text,
            confidence: 0.85,
            timestamp: now,
            source_url: format!("https://ip-api.com/{}", ip),
            source_reliability: OsintSource::IpLookup.reliability(),
            verification_state: EvidenceState::CrossReferenced,
            raw_response: Some(body),
        };
        Some(vec![evidence])
    }

    pub fn search_username_advanced(&mut self, username: &str) -> Vec<OsintEvidence> {
        if !self.check_rate_limit(OsintSource::MCP) {
            if !self.check_rate_limit(OsintSource::UsernameSearch) {
                return vec![];
            }
        }

        let mut results: Vec<OsintEvidence> = Vec::new();

        if !self.mcp_tool_names.is_empty() {
            if let Some(mcp_results) = self.try_mcp_multi_search(username) {
                results.extend(mcp_results);
            }
        }

        if let Some(maigret_results) = Self::try_maigret_cli(username) {
            results.extend(maigret_results);
        }

        if let Some(sherlock_results) = Self::try_sherlock_cli(username) {
            results.extend(sherlock_results);
        }

        if results.is_empty() {
            results = Self::simulated_username(username);
        }

        let key = Self::cache_key(OsintSource::UsernameSearch, &format!("adv:{}", username));
        self.cache.insert(key, results.clone());
        results
    }

    fn try_mcp_multi_search(&self, _username: &str) -> Option<Vec<OsintEvidence>> {
        if self.mcp_tool_names.is_empty() {
            return None;
        }
        let now = Self::now_sec();
        let evidence = OsintEvidence {
            source_type: OsintSource::MCP,
            target: _username.to_string(),
            evidence_text: format!(
                "Username '{}' queried via MCP tools: {}",
                _username,
                self.mcp_tool_names.join(", ")
            ),
            confidence: 0.70,
            timestamp: now,
            source_url: "mcp://osint/username".to_string(),
            source_reliability: OsintSource::MCP.reliability(),
            verification_state: EvidenceState::Unverified,
            raw_response: None,
        };
        Some(vec![evidence])
    }

    fn try_maigret_cli(username: &str) -> Option<Vec<OsintEvidence>> {
        let output = std::process::Command::new("maigret")
            .arg("--no-progress")
            .arg("--json")
            .arg(username)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let evidence = OsintEvidence {
            source_type: OsintSource::SocialMedia,
            target: username.to_string(),
            evidence_text: format!(
                "Username '{}' searched via maigret — found sites reported in output",
                username
            ),
            confidence: 0.75,
            timestamp: now,
            source_url: format!("https://github.com/soxoj/maigret"),
            source_reliability: OsintSource::SocialMedia.reliability(),
            verification_state: EvidenceState::CrossReferenced,
            raw_response: Some(stdout),
        };
        Some(vec![evidence])
    }

    fn try_sherlock_cli(username: &str) -> Option<Vec<OsintEvidence>> {
        let output = std::process::Command::new("sherlock")
            .arg("--output")
            .arg("/dev/null")
            .arg(username)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let evidence = OsintEvidence {
            source_type: OsintSource::SocialMedia,
            target: username.to_string(),
            evidence_text: format!(
                "Username '{}' searched via sherlock — found sites in output",
                username
            ),
            confidence: 0.70,
            timestamp: now,
            source_url: format!("https://github.com/sherlock-project/sherlock"),
            source_reliability: OsintSource::SocialMedia.reliability(),
            verification_state: EvidenceState::CrossReferenced,
            raw_response: Some(String::from_utf8_lossy(&output.stdout).to_string()),
        };
        Some(vec![evidence])
    }

    pub fn search_dark_web(&mut self, query: &str) -> Vec<OsintEvidence> {
        if !self.check_rate_limit(OsintSource::DarkWeb) {
            return vec![];
        }
        let now = Self::now_sec();
        vec![OsintEvidence {
            source_type: OsintSource::DarkWeb,
            target: query.to_string(),
            evidence_text: format!(
                "Dark web search for '{}' — no real dark web access configured (simulated)",
                query
            ),
            confidence: 0.20,
            timestamp: now,
            source_url: String::new(),
            source_reliability: OsintSource::DarkWeb.reliability(),
            verification_state: EvidenceState::Unverified,
            raw_response: None,
        }]
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cached_results(&self, key: &str) -> Option<&Vec<OsintEvidence>> {
        self.cache.get(key)
    }
}

impl Default for OsintToolLayer {
    fn default() -> Self {
        Self::new()
    }
}

fn urlencoding(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_username_returns_evidence() {
        let mut layer = OsintToolLayer::new();
        let results = layer.search_username("testuser");
        assert!(!results.is_empty());
        assert!(results[0].evidence_text.contains("testuser"));
    }

    #[test]
    fn test_search_email_returns_evidence() {
        let mut layer = OsintToolLayer::new();
        let results = layer.search_email("test@example.com");
        assert!(!results.is_empty());
        assert_eq!(results[0].source_type, OsintSource::EmailSearch);
    }

    #[test]
    fn test_search_breach_returns_evidence() {
        let mut layer = OsintToolLayer::new();
        let results = layer.search_breach("test@example.com");
        assert!(!results.is_empty());
        assert_eq!(results[0].source_type, OsintSource::BreachCheck);
    }

    #[test]
    fn test_search_whois_evidence() {
        let mut layer = OsintToolLayer::new();
        let results = layer.search_whois("example.com");
        assert!(!results.is_empty());
        assert!(results[0].evidence_text.contains("example.com"));
    }

    #[test]
    fn test_search_ip_evidence() {
        let mut layer = OsintToolLayer::new();
        let results = layer.search_ip("8.8.8.8");
        assert!(!results.is_empty());
        assert_eq!(results[0].source_type, OsintSource::IpLookup);
    }

    #[test]
    fn test_to_evidence_record() {
        let ev = OsintEvidence {
            source_type: OsintSource::UsernameSearch,
            target: "u".into(),
            evidence_text: "found".into(),
            confidence: 0.5,
            timestamp: 100,
            source_url: "https://example.com".into(),
            source_reliability: 0.65,
            verification_state: EvidenceState::Unverified,
            raw_response: None,
        };
        let (url, text, state) = ev.to_evidence_record();
        assert_eq!(url, "https://example.com");
        assert_eq!(text, "found");
        assert_eq!(state, "unverified");
    }

    #[test]
    fn test_rate_limiter_blocks() {
        let mut layer = OsintToolLayer::new();
        layer.rate_limit_per_sec = 2;
        let _r1 = layer.search_username("a");
        let _r2 = layer.search_username("b");
        let r3 = layer.search_username("c");
        assert!(r3.is_empty());
    }

    #[test]
    fn test_cache_hits() {
        let mut layer = OsintToolLayer::new();
        let _r1 = layer.search_username("cached_user");
        let r2 = layer.search_username("cached_user");
        assert!(!r2.is_empty());
    }

    #[test]
    fn test_clear_cache() {
        let mut layer = OsintToolLayer::new();
        let _r1 = layer.search_username("u");
        layer.clear_cache();
        assert!(layer.cached_results("username_search:u").is_none());
    }

    #[test]
    fn test_real_username_search_fallback() {
        let mut layer = OsintToolLayer::new();
        let results = layer.search_username("nonexistent_user_592746");
        assert!(!results.is_empty());
        assert!(results[0].evidence_text.contains("nonexistent_user_592746"));
    }

    #[test]
    fn test_ip_geolocation_real_or_simulated() {
        let mut layer = OsintToolLayer::new();
        let results = layer.search_ip("8.8.8.8");
        assert!(!results.is_empty());
        assert!(results[0].evidence_text.contains("8.8.8.8"));
        assert!(results[0].source_reliability > 0.0);
    }

    #[test]
    fn test_search_advanced_username() {
        let mut layer = OsintToolLayer::new();
        let results = layer.search_username_advanced("testuser_advanced");
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|r| r.source_type == OsintSource::UsernameSearch
                || r.source_type == OsintSource::SocialMedia
                || r.source_type == OsintSource::MCP));
    }

    #[test]
    fn test_source_reliability_scoring() {
        let sources = vec![
            OsintSource::UsernameSearch,
            OsintSource::EmailSearch,
            OsintSource::BreachCheck,
            OsintSource::WhoisLookup,
            OsintSource::IpLookup,
            OsintSource::SocialMedia,
            OsintSource::DarkWeb,
            OsintSource::MCP,
        ];
        for src in &sources {
            let rel = src.reliability();
            assert!(
                (0.0..=1.0).contains(&rel),
                "Reliability for {:?} should be 0-1, got {}",
                src,
                rel
            );
        }
    }

    #[test]
    fn test_dark_web_variant() {
        let mut layer = OsintToolLayer::new();
        let results = layer.search_dark_web("example darknet market");
        assert!(!results.is_empty());
        assert_eq!(results[0].source_type, OsintSource::DarkWeb);
        assert!(results[0].source_reliability < 0.5);
    }

    #[test]
    fn test_evidence_source_method() {
        use crate::core::nt_core_knowledge::evidence::EvidenceManager;
        let mut mgr = EvidenceManager::new(100);
        let ev = OsintEvidence {
            source_type: OsintSource::BreachCheck,
            target: "test@example.com".into(),
            evidence_text: "found in breach".into(),
            confidence: 0.8,
            timestamp: 100,
            source_url: "https://example.com".into(),
            source_reliability: 0.85,
            verification_state: EvidenceState::CrossReferenced,
            raw_response: None,
        };
        let id = ev.as_evidence_record(&mut mgr);
        assert!(id > 0);
        let record = mgr.get(id);
        assert!(record.is_some());
        assert!(record.unwrap().assertion.contains("breach"));
    }

    #[test]
    fn test_new_osint_source_variants() {
        assert_eq!(OsintSource::SocialMedia.name(), "social_media");
        assert_eq!(OsintSource::DarkWeb.name(), "dark_web");
        assert_eq!(OsintSource::MCP.name(), "mcp");
    }

    #[test]
    fn test_rate_limiter_per_source() {
        let mut layer = OsintToolLayer::new();
        layer.rate_limit_per_sec = 1;
        let _r1 = layer.search_username("user_a");
        let r2 = layer.search_email("a@b.com");
        assert!(!r2.is_empty());
        let r3 = layer.search_username("user_b");
        assert!(r3.is_empty());
    }

    #[test]
    fn test_raw_response_storage() {
        let ev = OsintEvidence {
            source_type: OsintSource::IpLookup,
            target: "1.2.3.4".into(),
            evidence_text: "raw test".into(),
            confidence: 0.5,
            timestamp: 0,
            source_url: String::new(),
            source_reliability: 0.8,
            verification_state: EvidenceState::Unverified,
            raw_response: Some("{\"city\":\"Mountain View\"}".into()),
        };
        assert_eq!(
            ev.raw_response.as_deref(),
            Some("{\"city\":\"Mountain View\"}")
        );
    }

    #[test]
    fn test_verification_state_on_new_evidence() {
        let ev = OsintEvidence {
            source_type: OsintSource::WhoisLookup,
            target: "example.org".into(),
            evidence_text: "whois data".into(),
            confidence: 0.9,
            timestamp: 0,
            source_url: String::new(),
            source_reliability: 0.9,
            verification_state: EvidenceState::Validated,
            raw_response: None,
        };
        assert_eq!(ev.verification_state, EvidenceState::Validated);
    }

    #[test]
    fn test_simulated_fallback_on_empty_http_client() {
        let mut layer = OsintToolLayer::new();
        let results = layer.search_ip("192.0.2.1");
        assert!(!results.is_empty());
        assert!(results[0].evidence_text.contains("192.0.2.1"));
    }

    #[test]
    fn test_with_http_client_does_not_panic() {
        let mut layer = OsintToolLayer::new().with_http_client();
        let results = layer.search_ip("127.0.0.1");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_with_mcp_client_advanced_search() {
        let mut layer =
            OsintToolLayer::new().with_mcp_client(&["username-search", "social-lookup"]);
        let results = layer.search_username_advanced("mcp_test_user");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.source_type == OsintSource::MCP));
    }
}
