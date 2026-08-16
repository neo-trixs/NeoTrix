#![deny(clippy::unwrap_used)]

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use reqwest::Client;
use sha2::{Digest, Sha256};

use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType;

pub mod dns;
pub mod http;
pub mod url;
pub mod credential;
pub mod person;
pub mod social;
pub mod vuln;
pub mod network;
pub mod dark;
pub mod backend_router;
pub mod sweep;

#[derive(Debug, Clone)]
pub struct OsintConfig {
    pub concurrency: usize,
    pub timeout_secs: u64,
    pub use_proxy: bool,
    pub api_keys: HashMap<String, String>,
    pub dns_wordlist: Vec<String>,
    pub enable_active: bool,
}

impl OsintConfig {
    pub fn default_dns_wordlist() -> Vec<String> {
        ["www","mail","admin","api","blog","dev","test","stage","prod",
         "beta","app","m","mobile","cdn","static","assets","media","img",
         "images","css","js","fonts","upload","download","ftp","ssh","vpn",
         "portal","login","auth","sso","oauth","idp","saml","git","ci","cd",
         "jenkins","jira","confluence","wiki","docs","help","support",
         "monitor","grafana","prometheus","alert","log","syslog","audit",
         "db","database","redis","mysql","postgres","mongo","elastic","kibana",
         "web","webmail","webdisk","ns1","ns2","mx","smtp","pop3","imap",
         "calendar","drive","cloud","hub","edge","core","platform","gateway",
         "graphql","rest","v1","v2","v3","ws","wss","chat","video","stream",
         "live","status","backup","config","admin","root","internal",
         "remote","office","vpn","ns1","ns2","ns3","ns4","mx1","mx2",
         "smtp","mail2","pop","imap","owa","exchange","cpanel","whm",
         "phpmyadmin","pma","server","node","cluster","proxy","cache",
         "origin","www2","www3","www4","web1","web2","app1","app2",
        ].into_iter().map(String::from).collect()
    }
}

impl Default for OsintConfig {
    fn default() -> Self {
        OsintConfig {
            concurrency: 10,
            timeout_secs: 30,
            use_proxy: false,
            api_keys: HashMap::new(),
            dns_wordlist: OsintConfig::default_dns_wordlist(),
            enable_active: true,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OsintTarget {
    pub domain: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub ip: Option<String>,
}

impl OsintTarget {
    pub fn from_domain(domain: impl Into<String>) -> Self {
        OsintTarget { domain: Some(domain.into()), ..Default::default() }
    }
    pub fn from_username(username: impl Into<String>) -> Self {
        OsintTarget { username: Some(username.into()), ..Default::default() }
    }
    pub fn from_email(email: impl Into<String>) -> Self {
        let e = email.into();
        let domain = e.split('@').nth(1).map(|d| d.to_string());
        OsintTarget { email: Some(e), domain, ..Default::default() }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OsintReport {
    pub target: OsintTarget,
    pub dns: Option<dns::DnsFindings>,
    pub http: Option<http::HttpFindings>,
    pub url_history: Option<url::UrlHistoryFindings>,
    pub credential: Option<credential::CredentialFindings>,
    pub person: Option<person::PersonFindings>,
    pub social: Option<social::SocialFindings>,
    pub vuln: Option<vuln::VulnFindings>,
    pub network: Option<network::NetworkFindings>,
    pub dark: Option<dark::DarkFindings>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub errors: Vec<String>,
}

impl OsintReport {
    pub fn new(target: OsintTarget) -> Self {
        OsintReport {
            target,
            started_at: Utc::now(),
            ..Default::default()
        }
    }

    pub fn elapsed_ms(&self) -> i64 {
        match self.completed_at {
            Some(end) => (end - self.started_at).num_milliseconds(),
            None => (Utc::now() - self.started_at).num_milliseconds(),
        }
    }

    pub fn total_findings(&self) -> usize {
        let mut n = 0;
        if let Some(ref d) = self.dns { n += d.subdomains.len() + d.mx_records.len() + d.txt_records.len() + d.ns_records.len(); }
        if let Some(ref h) = self.http { n += h.endpoints.len(); }
        if let Some(ref u) = self.url_history { n += u.snapshots.len(); }
        if let Some(ref c) = self.credential { n += c.breaches.len(); }
        if let Some(ref p) = self.person { n += p.profiles.len(); }
        if let Some(ref s) = self.social { n += s.posts.len(); }
        if let Some(ref v) = self.vuln { n += v.vulnerabilities.len(); }
        if let Some(ref nw) = self.network { n += nw.services.len(); }
        if let Some(ref dk) = self.dark { n += dk.results.len(); }
        n
    }
}

impl OsintReport {
    /// Evidence run identifier for this report (Claude-OSINT pattern: JSONL run_id).
    /// Stable per target+report so all nodes written by one run share traceability.
    fn run_id(&self) -> String {
        let target = self
            .target
            .domain
            .clone()
            .or_else(|| self.target.username.clone())
            .or_else(|| self.target.email.clone())
            .unwrap_or_else(|| "unknown".into());
        let now = chrono::Utc::now().timestamp();
        format!("osint-{}", hex::encode(Sha256::digest(format!("{}|{}", target, now).as_bytes())).chars().take(16).collect::<String>())
    }

    /// Write a node, then attach evidence metadata (sha256 fingerprint of title+summary+url,
    /// run_id, tool version, timestamp) so every KB artifact is traceable.
    fn write_with_evidence(
        kb: &KnowledgeBase,
        title: &str,
        node_type: NodeType,
        summary: Option<&str>,
        url: Option<&str>,
        domain_hint: Option<&str>,
        run_id: &str,
    ) -> Result<String, String> {
        let id = kb.insert_or_get_node(title, node_type, summary, url, domain_hint)?;
        let fingerprint = format!("{}|{}|{}", title, summary.unwrap_or(""), url.unwrap_or(""));
        let sha256 = hex::encode(Sha256::digest(fingerprint.as_bytes()));
        let meta = serde_json::json!({
            "evidence": {
                "run_id": run_id,
                "sha256": sha256,
                "tool": "neotrix-osint",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        });
        kb.update_node_metadata(&id, &meta)?;
        Ok(id)
    }

    pub fn write_to_kb(&self, kb: &KnowledgeBase) -> Vec<(String, NodeType)> {
        let mut written = Vec::new();
        let run_id = self.run_id();
        let domain_hint: Option<&str> = self.target.domain.as_deref()
            .or_else(|| self.target.email.as_ref().and_then(|e| e.split('@').nth(1)));

        if let Some(ref dns) = self.dns {
            for rec in &dns.subdomains {
                if let Ok(id) = Self::write_with_evidence(kb, &format!("subdomain: {} ({})", rec.name, rec.record_type), NodeType::Source, Some(&format!("DNS {} record: {}", rec.record_type, rec.value)), Some(&format!("https://{}", rec.name)), domain_hint, &run_id) {
                    written.push((id, NodeType::Source));
                }
            }
        }

        if let Some(ref http) = self.http {
            for ep in &http.endpoints {
                if let Ok(id) = Self::write_with_evidence(kb, &ep.url, NodeType::Source, ep.title.as_deref(), Some(&ep.url), domain_hint, &run_id) {
                    written.push((id, NodeType::Source));
                }
            }
        }

        if let Some(ref url) = self.url_history {
            for snap in &url.snapshots {
                if let Ok(id) = Self::write_with_evidence(kb, &format!("Wayback: {}", snap.url), NodeType::Reference, Some(&format!("Snapshot {}", snap.timestamp)), Some(&snap.url), domain_hint, &run_id) {
                    written.push((id, NodeType::Reference));
                }
            }
        }

        if let Some(ref cred) = self.credential {
            for b in &cred.breaches {
                let name = b.breach_name.as_deref().unwrap_or(&b.source);
                if let Ok(id) = Self::write_with_evidence(kb, &format!("breach: {}", name), NodeType::DetectionFinding, b.description.as_deref().or(Some("Credential breach")), None, domain_hint, &run_id) {
                    written.push((id, NodeType::DetectionFinding));
                }
            }
        }

        if let Some(ref person) = self.person {
            for p in &person.profiles {
                if let Ok(id) = Self::write_with_evidence(kb, &format!("{} @ {}", p.username, p.platform), NodeType::Person, p.name_display.as_deref().or(Some(&p.username)), Some(&p.url), None, &run_id) {
                    written.push((id, NodeType::Person));
                }
            }
        }

        if let Some(ref social) = self.social {
            for post in &social.posts {
                if let Some(ref url) = post.url {
                    if let Ok(id) = Self::write_with_evidence(kb, url, NodeType::Source, Some(&format!("Social: {} on {}", post.author, post.platform)), Some(url), None, &run_id) {
                        written.push((id, NodeType::Source));
                    }
                }
            }
        }

        if let Some(ref vuln) = self.vuln {
            for v in &vuln.vulnerabilities {
                let summary: String = v.summary.chars().take(80).collect();
                if let Ok(id) = Self::write_with_evidence(kb, &format!("{}/{}", v.id, summary), NodeType::DetectionFinding, Some(&v.summary), Some(&format!("https://nvd.nist.gov/vuln/detail/{}", v.id)), domain_hint, &run_id) {
                    written.push((id, NodeType::DetectionFinding));
                }
            }
        }

        if let Some(ref net) = self.network {
            for svc in &net.services {
                let name = svc.service.as_deref().unwrap_or("unknown");
                if let Ok(id) = Self::write_with_evidence(kb, &format!("{}:{}/{}", svc.host, svc.port, name), NodeType::Source, svc.banner.as_deref(), None, domain_hint, &run_id) {
                    written.push((id, NodeType::Source));
                }
            }
        }

        if let Some(ref dark) = self.dark {
            for result in &dark.results {
                if let Ok(id) = Self::write_with_evidence(kb, &format!("dark: {}", result.title), NodeType::Source, Some(&result.snippet), Some(&result.url), domain_hint, &run_id) {
                    written.push((id, NodeType::Source));
                }
            }
        }

        written
    }
}

impl std::fmt::Display for OsintReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═ OSINT Report ════════════════════════════════════")?;
        if let Some(ref d) = self.target.domain { writeln!(f, "  Domain:     {}", d)?; }
        if let Some(ref u) = self.target.username { writeln!(f, "  Username:   {}", u)?; }
        if let Some(ref e) = self.target.email { writeln!(f, "  Email:      {}", e)?; }
        if let Some(ref u) = self.target.url { writeln!(f, "  URL:        {}", u)?; }
        if let Some(ref i) = self.target.ip { writeln!(f, "  IP:         {}", i)?; }
        writeln!(f, "  Duration:   {}ms", self.elapsed_ms())?;
        writeln!(f, "  Findings:   {}", self.total_findings())?;
        writeln!(f, "  Errors:     {}", self.errors.len())?;
        writeln!(f, "─────────────────────────────────────────────────")?;
        if let Some(ref d) = self.dns { write!(f, "{}", d)?; }
        if let Some(ref h) = self.http { write!(f, "{}", h)?; }
        if let Some(ref u) = self.url_history { write!(f, "{}", u)?; }
        if let Some(ref c) = self.credential { write!(f, "{}", c)?; }
        if let Some(ref p) = self.person { write!(f, "{}", p)?; }
        if let Some(ref s) = self.social { write!(f, "{}", s)?; }
        if let Some(ref v) = self.vuln { write!(f, "{}", v)?; }
        if let Some(ref n) = self.network { write!(f, "{}", n)?; }
        if let Some(ref d) = self.dark { write!(f, "{}", d)?; }
        writeln!(f, "═══════════════════════════════════════════════════")
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DoctorReport {
    pub target: OsintTarget,
    pub dns_ok: bool,
    pub _dns_latency: u64,
    pub http_ok: bool,
    pub _http_latency: u64,
}

impl std::fmt::Display for DoctorReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═ OSINT Doctor Report ══════════════════════════════════")?;
        if let Some(ref d) = self.target.domain { writeln!(f, "  Domain:     {}", d)?; }
        if let Some(ref u) = self.target.username { writeln!(f, "  Username:   {}", u)?; }
        if let Some(ref e) = self.target.email { writeln!(f, "  Email:      {}", e)?; }
        if let Some(ref u) = self.target.url { writeln!(f, "  URL:        {}", u)?; }
        if let Some(ref i) = self.target.ip { writeln!(f, "  IP:         {}", i)?; }
        writeln!(f, "  DNS:    {} ({}ms)", if self.dns_ok { "✓" } else { "✗" }, self._dns_latency)?;
        writeln!(f, "  HTTP:   {} ({}ms)", if self.http_ok { "✓" } else { "✗" }, self._http_latency)?;
        writeln!(f, "═══════════════════════════════════════════════════")
    }
}

fn default_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .https_only(true)
        .build()
        .unwrap_or_else(|_| Client::new())
}

pub async fn run_osint(target: OsintTarget, config: OsintConfig) -> OsintReport {
    let mut report = OsintReport::new(target);
    let client = default_client();

    // DNS: 直接使用 dns::investigate (R-P82 简化版)
    if report.target.domain.is_some() {
        match dns::investigate(&report.target, &client, &config).await {
            Ok(dns) => report.dns = Some(dns),
            Err(e) => report.errors.push(format!("dns: {e}")),
        }

        match http::investigate(&report.target, &client, &config).await {
            Ok(f) => report.http = Some(f),
            Err(e) => report.errors.push(format!("http: {e}")),
        }
        match url::investigate(&report.target, &client, &config).await {
            Ok(f) => report.url_history = Some(f),
            Err(e) => report.errors.push(format!("url: {e}")),
        }
        match vuln::investigate(&report.target, &client, &config).await {
            Ok(f) => report.vuln = Some(f),
            Err(e) => report.errors.push(format!("vuln: {e}")),
        }
        match network::investigate(&report.target, &client, &config).await {
            Ok(f) => report.network = Some(f),
            Err(e) => report.errors.push(format!("network: {e}")),
        }
        match dark::investigate(&report.target, &client, &config).await {
            Ok(f) => report.dark = Some(f),
            Err(e) => report.errors.push(format!("dark: {e}")),
        }
    }

    if report.target.username.is_some() || report.target.email.is_some() {
        match person::investigate(&report.target, &client, &config).await {
            Ok(f) => report.person = Some(f),
            Err(e) => report.errors.push(format!("person: {e}")),
        }
        match social::investigate(&report.target, &client, &config).await {
            Ok(f) => report.social = Some(f),
            Err(e) => report.errors.push(format!("social: {e}")),
        }
    }

    if report.target.email.is_some() {
        match credential::investigate(&report.target, &client, &config).await {
            Ok(f) => report.credential = Some(f),
            Err(e) => report.errors.push(format!("credential: {e}")),
        }
    }

    report.completed_at = Some(Utc::now());
    report
}

/// doctor 体检: 通过有序后端路由探测 DNS + HTTP 可用性 (R-P82)
pub async fn doctor_osint(target: OsintTarget, _config: OsintConfig) -> DoctorReport {
    let client = default_client();

    let mut dns_ok = false;
    let mut _dns_latency = 0u64;
    let mut dns_router = backend_router::BackendRouter::new(backend_router::default_dns_backends());
    let start = std::time::Instant::now();
    if dns_router.probe_and_select(&target, &client).await.is_ok() { dns_ok = true }
    _dns_latency = start.elapsed().as_millis() as u64;

    let mut http_ok = false;
    let mut _http_latency = 0u64;
    let mut http_router = backend_router::BackendRouter::new(backend_router::default_http_backends());
    let start = std::time::Instant::now();
    if http_router.probe_and_select(&target, &client).await.is_ok() { http_ok = true }
    _http_latency = start.elapsed().as_millis() as u64;

    DoctorReport {
        target,
        dns_ok,
        _dns_latency,
        http_ok,
        _http_latency,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_from_domain() {
        let t = OsintTarget::from_domain("example.com");
        assert_eq!(t.domain.unwrap(), "example.com");
    }

    #[test]
    fn test_target_from_email() {
        let t = OsintTarget::from_email("user@example.com");
        assert_eq!(t.email.unwrap(), "user@example.com");
        assert_eq!(t.domain.unwrap(), "example.com");
    }

    #[test]
    fn test_config_default() {
        let c = OsintConfig::default();
        assert_eq!(c.concurrency, 10);
        assert!(c.dns_wordlist.len() > 50);
    }

    #[test]
    fn test_report_new() {
        let r = OsintReport::new(OsintTarget::from_domain("test.com"));
        assert_eq!(r.total_findings(), 0);
        assert!(r.errors.is_empty());
    }

    #[test]
    fn test_report_elapsed() {
        let r = OsintReport::new(OsintTarget::from_domain("test.com"));
        assert!(r.elapsed_ms() >= 0);
    }
}
