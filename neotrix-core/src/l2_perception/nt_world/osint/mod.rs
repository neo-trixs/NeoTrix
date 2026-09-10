pub mod dns;
pub mod http;
pub mod url;
pub mod credential;
pub mod person;
pub mod social;
pub mod vuln;
pub mod network;
pub mod dark;
pub mod fofa;
pub mod censys;
pub mod shodan;
pub mod zoomeye;
pub mod backend_router;
pub mod sweep;
pub mod self_curriculum;
pub mod api_registry;
pub mod metadata;
pub mod repo_reverse_prompt;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::NodeType;
use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::CrawlCycleReport;
use crate::l2_perception::nt_world::nt_world_github_absorber::GitHubAbsorber;
pub use crate::l2_perception::nt_world::nt_world_github_absorber::GitHubAbsorbReport;
use rusqlite::Connection;

// GitHubAbsorbReport is re-exported from nt_world_github_absorber (single fact source)

/// OSINT 模块声明式宏 — 一次定义，自动生成 dispatch
///
/// 用法:
/// ```ignore
/// osint_modules!(target, client, config, report, [
///     dns::investigate       => dns:       |t| t.domain.is_some(),
///     shodan::investigate    => shodan:    |t| t.domain.is_some() || t.ip.is_some(),
/// ]);
/// ```
macro_rules! osint_modules {
    ($target:expr, $client:expr, $config:expr, $report:expr,
     [ $( $mod:path => $field:ident : $gate:expr ),* $(,)? ]
    ) => {{
        $(
            if ($gate)(& $target) {
                match $mod(& $target, & $client, & $config).await {
                    Ok(f) => { $report.$field = Some(f); }
                    Err(e) => { $report.errors.push(format!("{}: {}", stringify!($mod), e)); }
                }
            }
        )*
    }};
}

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

/// OSINT Source trait — 所有 OSINT 调查源必须实现此 trait
pub trait OsintSource: Send + Sync {
    /// 调查结果类型
    type Findings: Send + Sync;
    
    /// 源名称
    fn name(&self) -> &'static str;
    
    /// 是否需要 API key
    fn needs_api_key(&self) -> bool;
    
    /// 优先级 (越高越优先)
    fn priority(&self) -> u8;
    
    /// 执行调查
    fn investigate(
        &self,
        target: &OsintTarget,
        client: &Client,
        config: &OsintConfig,
    ) -> impl std::future::Future<Output = Result<Self::Findings, String>> + Send;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// 返回目标的主标识符 (单一事实源): domain > ip > email > url > username > "unknown"
    pub fn primary_label(&self) -> String {
        self.domain.clone()
            .or_else(|| self.ip.clone())
            .or_else(|| self.email.clone())
            .or_else(|| self.url.clone())
            .or_else(|| self.username.clone())
            .unwrap_or_else(|| "unknown".into())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    pub fofa: Option<fofa::FofaFindings>,
    pub shodan: Option<shodan::ShodanFindings>,
    pub censys: Option<censys::CensysFindings>,
    pub zoomeye: Option<zoomeye::ZoomEyeFindings>,
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
        if let Some(ref f) = self.fofa { n += f.assets.len(); }
        if let Some(ref s) = self.shodan { n += s.services.len(); }
        if let Some(ref c) = self.censys { n += c.services.len(); }
        if let Some(ref z) = self.zoomeye { n += z.host.len(); }
        n
    }
}

impl OsintReport {
    fn run_id(&self) -> String {
        let target = self.target.primary_label();
        let now = chrono::Utc::now().timestamp();
        format!("osint-{}", hex::encode(Sha256::digest(format!("{}|{}", target, now).as_bytes())).chars().take(16).collect::<String>())
    }

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

        if let Some(ref fofa) = self.fofa {
            for r in &fofa.assets {
                if let Ok(id) = Self::write_with_evidence(kb, &format!("fofa: {}", r.url), NodeType::Source, r.title.as_deref().or(Some("FOFA result")), Some(&r.url), domain_hint, &run_id) {
                    written.push((id, NodeType::Source));
                }
            }
        }

        if let Some(ref shodan) = self.shodan {
            for svc in &shodan.services {
                let name = svc.product.as_deref().unwrap_or("unknown");
                if let Ok(id) = Self::write_with_evidence(kb, &format!("shodan: {}", svc.port), NodeType::Source, svc.banner.as_deref(), None, domain_hint, &run_id) {
                    written.push((id, NodeType::Source));
                }
            }
        }

        if let Some(ref censys) = self.censys {
            for r in &censys.services {
                if let Ok(id) = Self::write_with_evidence(kb, &format!("censys: {}", censys.ip), NodeType::Source, r.product.as_deref().or(Some("Censys service")), None, domain_hint, &run_id) {
                    written.push((id, NodeType::Source));
                }
            }
        }

        if let Some(ref zoomeye) = self.zoomeye {
            for svc in &zoomeye.services {
                if let Ok(id) = Self::write_with_evidence(kb, &format!("zoomeye: {}", zoomeye.ip), NodeType::Source, svc.product.as_deref().or(Some("ZoomEye service")), None, domain_hint, &run_id) {
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
        if let Some(ref fofa) = self.fofa { write!(f, "{}", fofa)?; }
        if let Some(ref s) = self.shodan { write!(f, "{}", s)?; }
        if let Some(ref c) = self.censys { write!(f, "{}", c)?; }
        if let Some(ref z) = self.zoomeye { write!(f, "{}", z)?; }
        writeln!(f, "═══════════════════════════════════════════════════")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub target: OsintTarget,
    pub dns_ok: bool,
    pub _dns_latency: u64,
    pub http_ok: bool,
    pub _http_latency: u64,
    pub fofa_ok: bool,
    pub _fofa_latency: u64,
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
        writeln!(f, "  FOFA:   {} ({}ms)", if self.fofa_ok { "✓" } else { "✗" }, self._fofa_latency)?;
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

    osint_modules!(report.target, client, config, report, [
        dns::investigate       => dns:           |t: &OsintTarget| t.domain.is_some(),
        http::investigate      => http:          |t: &OsintTarget| t.domain.is_some(),
        url::investigate       => url_history:   |t: &OsintTarget| t.domain.is_some(),
        vuln::investigate      => vuln:          |t: &OsintTarget| t.domain.is_some(),
        network::investigate   => network:       |t: &OsintTarget| t.domain.is_some(),
        dark::investigate      => dark:          |t: &OsintTarget| t.domain.is_some(),
        fofa::investigate      => fofa:          |t: &OsintTarget| t.domain.is_some(),
        shodan::investigate    => shodan:        |t: &OsintTarget| t.domain.is_some() || t.ip.is_some(),
        censys::investigate    => censys:        |t: &OsintTarget| t.ip.is_some(),
        zoomeye::investigate   => zoomeye:       |t: &OsintTarget| t.domain.is_some() || t.ip.is_some(),
        person::investigate    => person:        |t: &OsintTarget| t.username.is_some() || t.email.is_some(),
        social::investigate    => social:        |t: &OsintTarget| t.username.is_some() || t.email.is_some(),
        credential::investigate => credential:  |t: &OsintTarget| t.email.is_some(),
    ]);

    report.completed_at = Some(Utc::now());
    report
}

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

    let mut fofa_ok = false;
    let mut _fofa_latency = 0u64;
    let mut fofa_router = backend_router::BackendRouter::new(backend_router::default_fofa_backends());
    let start = std::time::Instant::now();
    if fofa_router.probe_and_select(&target, &client).await.is_ok() { fofa_ok = true }
    _fofa_latency = start.elapsed().as_millis() as u64;

    DoctorReport {
        target,
        dns_ok,
        _dns_latency,
        http_ok,
        _http_latency,
        fofa_ok,
        _fofa_latency,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AbsorbSource {
    GitHub { owner: String, repo: String },
    GitHubUrl(String),
    ArXiv(String),
    Wikipedia(String),
    WebPage(String),
    DiscoveryTopic(String),
}

impl AbsorbSource {
    pub fn from_url(url: &str) -> Option<Self> {
        let url = url.trim();
        if url.contains("github.com") {
            return Some(AbsorbSource::GitHubUrl(url.to_string()));
        }
        if url.contains("arxiv.org") {
            let id = url.trim_end_matches('/').split('/').next_back().unwrap_or(url);
            return Some(AbsorbSource::ArXiv(id.to_string()));
        }
        if url.contains("wikipedia.org") {
            let topic = url.split('/').next_back().unwrap_or(url).replace('_', " ");
            return Some(AbsorbSource::Wikipedia(topic));
        }
        Some(AbsorbSource::WebPage(url.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbCycleReport {
    pub sources_attempted: usize,
    pub sources_succeeded: usize,
    pub sources_failed: usize,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub github_repos: Vec<GitHubAbsorbReport>,
    pub web_pages: Vec<WebPageReport>,
    pub errors: Vec<(String, String)>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPageReport {
    pub url: String,
    pub title: String,
    pub nodes_created: usize,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorberConfig {
    pub max_github_stars: i64,
    pub max_source_files: usize,
    pub enable_readme: bool,
    pub enable_deps: bool,
    pub enable_insights: bool,
    pub auto_refresh_days: i64,
    pub max_concurrent: usize,
}

impl Default for AbsorberConfig {
    fn default() -> Self {
        Self {
            max_github_stars: 0,
            max_source_files: 30,
            enable_readme: true,
            enable_deps: true,
            enable_insights: true,
            auto_refresh_days: 7,
            max_concurrent: 3,
        }
    }
}

pub struct UnifiedAbsorber {
    kb: KnowledgeBase,
    github: GitHubAbsorber,
    api_registry: api_registry::ApiRegistry,
}

impl UnifiedAbsorber {
    pub fn new(kb: KnowledgeBase, _config: AbsorberConfig) -> Result<Self, String> {
        Ok(Self {
            github: GitHubAbsorber::new(kb.clone_connection()),
            kb,
            api_registry: api_registry::ApiRegistry::new(),
        })
    }

    pub fn register_api(&mut self, entry: api_registry::ApiEntry) {
        self.api_registry.register(entry);
    }

    pub fn seed_api_discovery(&self, conn: &Connection) -> Result<usize, String> {
        let seeds = self.api_registry.seed_urls();
        let mut injected = 0usize;
        for url in seeds {
            let domain = crate::l2_perception::nt_world::crawl::frontier::extract_domain(&url);
            if let Err(e) = nt_memory_store::upsert_crawl_queue(conn, &url, 0, &domain, 50, now()) {
                log::warn!("[absorber] seed api discovery {}: {}", url, e);
                continue;
            }
            injected += 1;
        }
        Ok(injected)
    }

    pub fn api_registry_stats(&self) -> (usize, f64, Vec<(String, usize)>) {
        self.api_registry.stats()
    }

    pub fn absorb(&self, source: &AbsorbSource) -> Result<AbsorbCycleReport, String> {
        let mut report = AbsorbCycleReport {
            sources_attempted: 1,
            sources_succeeded: 0,
            sources_failed: 0,
            nodes_created: 0,
            edges_created: 0,
            github_repos: Vec::new(),
            web_pages: Vec::new(),
            errors: Vec::new(),
            timestamp: now(),
        };

        match source {
            AbsorbSource::GitHub { owner, repo } => {
                match self.github.absorb(owner, repo) {
                    Ok(gr) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += gr.nodes_created;
                        report.edges_created += gr.edges_created;
                        report.github_repos.push(gr);
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((format!("{}/{}", owner, repo), e));
                    }
                }
            }
            AbsorbSource::GitHubUrl(url) => {
                match self.github.absorb_url(url) {
                    Ok(gr) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += gr.nodes_created;
                        report.edges_created += gr.edges_created;
                        report.github_repos.push(gr);
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((url.clone(), e));
                    }
                }
            }
            AbsorbSource::ArXiv(id) => {
                match self.kb.ingest_arxiv(id) {
                    Ok(n) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += n;
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((format!("arxiv:{}", id), e));
                    }
                }
            }
            AbsorbSource::Wikipedia(topic) => {
                match self.kb.ingest_wikipedia(topic) {
                    Ok(n) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += n;
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((format!("wiki:{}", topic), e));
                    }
                }
            }
            AbsorbSource::WebPage(url) => {
                match self.absorb_webpage(url) {
                    Ok(wr) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += wr.nodes_created;
                        report.web_pages.push(wr);
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((url.clone(), e));
                    }
                }
            }
            AbsorbSource::DiscoveryTopic(topic) => {
                match self.kb.ingest_wikipedia(topic) {
                    Ok(n) => {
                        report.sources_succeeded += 1;
                        report.nodes_created += n;
                        let conn = match self.kb.conn.lock() {
                            Ok(c) => c,
                            Err(e) => { report.errors.push(("lock".into(), format!("{}", e))); return Ok(report); }
                        };
                        let _ = nt_memory_kb_crawl::discover_from_seed(&conn, topic);
                        report.nodes_created += 1;
                    }
                    Err(e) => {
                        report.sources_failed += 1;
                        report.errors.push((format!("discover:{}", topic), e));
                    }
                }
            }
        }

        self.persist_cycle_report(&report)?;
        Ok(report)
    }

    pub fn absorb_batch(&self, sources: &[AbsorbSource]) -> Result<AbsorbCycleReport, String> {
        let mut aggregated = AbsorbCycleReport {
            sources_attempted: sources.len(),
            sources_succeeded: 0,
            sources_failed: 0,
            nodes_created: 0,
            edges_created: 0,
            github_repos: Vec::new(),
            web_pages: Vec::new(),
            errors: Vec::new(),
            timestamp: now(),
        };

        for source in sources {
            match self.absorb(source) {
                Ok(report) => {
                    aggregated.sources_succeeded += report.sources_succeeded;
                    aggregated.sources_failed += report.sources_failed;
                    aggregated.nodes_created += report.nodes_created;
                    aggregated.edges_created += report.edges_created;
                    aggregated.github_repos.extend(report.github_repos);
                    aggregated.web_pages.extend(report.web_pages);
                    aggregated.errors.extend(report.errors);
                }
                Err(e) => {
                    aggregated.sources_failed += 1;
                    aggregated.errors.push((format!("{:?}", source), e));
                }
            }
        }

        self.persist_cycle_report(&aggregated)?;
        Ok(aggregated)
    }

    pub fn run_cycle(&self, topics: &[&str]) -> Result<AbsorbCycleReport, String> {
        let mut report = AbsorbCycleReport {
            sources_attempted: 0,
            sources_succeeded: 0,
            sources_failed: 0,
            nodes_created: 0,
            edges_created: 0,
            github_repos: Vec::new(),
            web_pages: Vec::new(),
            errors: Vec::new(),
            timestamp: now(),
        };

        for topic in topics {
            match self.absorb(&AbsorbSource::DiscoveryTopic(topic.to_string())) {
                Ok(r) => {
                    report.sources_succeeded += r.sources_succeeded;
                    report.sources_failed += r.sources_failed;
                    report.nodes_created += r.nodes_created;
                    report.edges_created += r.edges_created;
                    report.errors.extend(r.errors);
                }
                Err(e) => report.errors.push((topic.to_string(), e)),
            }
            report.sources_attempted += 1;
        }

        let discovery_cfg = nt_memory_kb_discovery::DiscoveryPipelineConfig::default();
        match self.kb.run_github_topics_discovery(&discovery_cfg) {
            Ok(stats) => {
                report.nodes_created += stats.repos_ingested;
                report.sources_succeeded += 1;
            }
            Err(e) => report.errors.push(("github_topics_discovery".into(), e)),
        }

        let conn = match self.kb.conn.lock() {
            Ok(c) => c,
            Err(e) => { report.errors.push(("lock".into(), format!("{}", e))); return Ok(report); }
        };
        if let Ok(crawl_report) = nt_memory_kb_crawl::run_crawl_cycle(&conn, 10) {
            report.nodes_created += crawl_report.nodes_created;
            report.edges_created += crawl_report.edges_created;
            report.sources_attempted += crawl_report.attempted;
            report.sources_succeeded += crawl_report.completed;
            report.sources_failed += crawl_report.failed;
            for (url, err) in &crawl_report.errors {
                let url: String = url.clone();
                let err: String = err.clone();
                report.errors.push((url, err));
            }
        }
        drop(conn);

        let repos = self.kb.find_repositories("github.com", None).unwrap_or_default();
        for node in repos {
            let stale = node.metadata.as_ref()
                .map(|m| {
                    let pushed = m.get("pushed_at").and_then(|v| v.as_i64()).unwrap_or(0);
                    let absorbed = m.get("last_absorbed").and_then(|v| v.as_i64()).unwrap_or(0);
                    pushed > absorbed
                })
                .unwrap_or(false);
            if stale {
                let parts: Vec<&str> = node.title.split('/').collect();
                if parts.len() == 2 {
                    if let Ok(gr) = self.github.refresh(parts[0], parts[1]) {
                        if gr.is_update {
                            report.github_repos.push(gr);
                        }
                    }
                }
            }
        }

        self.persist_cycle_report(&report)?;
        Ok(report)
    }

    pub fn status(&self) -> Result<AbsorberStatus, String> {
        let repos = self.kb.find_repositories("github.com", None).unwrap_or_default();
        let conn = self.kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let total_nodes = nt_memory_store::count_nodes(&conn).map_err(|e| format!("count: {}", e))?;
        let paper_count = nt_memory_store::count_nodes_by_type(&conn, "Paper").map_err(|e| format!("count_papers: {}", e))?;
        let article_count = nt_memory_store::count_nodes_by_type(&conn, "Article").map_err(|e| format!("count_articles: {}", e))?;
        let concept_count = nt_memory_store::count_nodes_by_type(&conn, "Concept").map_err(|e| format!("count_concepts: {}", e))?;
        let code_count = nt_memory_store::count_nodes_by_type(&conn, "CodeSnippet").map_err(|e| format!("count_code: {}", e))?;
        let insight_count = nt_memory_store::count_nodes_by_type(&conn, "Insight").map_err(|e| format!("count_insights: {}", e))?;
        let stale_repos = repos.iter().filter(|n| {
            n.metadata.as_ref()
                .map(|m| {
                    let pushed = m.get("pushed_at").and_then(|v| v.as_i64()).unwrap_or(0);
                    let absorbed = m.get("last_absorbed").and_then(|v| v.as_i64()).unwrap_or(0);
                    pushed > absorbed
                })
                .unwrap_or(false)
        }).count();
        drop(conn);

        Ok(AbsorberStatus {
            total_nodes,
            repositories: repos.len(),
            papers: paper_count,
            articles: article_count,
            concepts: concept_count,
            code_snippets: code_count,
            insights: insight_count,
            stale_repos,
            last_cycle: self.kb.kv_get("absorber", "last_cycle").unwrap_or(None),
        })
    }

    fn absorb_webpage(&self, url: &str) -> Result<WebPageReport, String> {
        let (html, host) = crate::l1_action::nt_memory::nt_memory_kb::nt_http::fetch_safe_http(url)?;
        let (title, text) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(&html);
        if text.is_empty() {
            return Err("Empty content".into());
        }
        let summary = text.chars().take(2000).collect::<String>();
        let domain = if host.is_empty() { "unknown" } else { &host };
        let _node_id = self.kb.insert_or_get_node(
            &if title.is_empty() { url.to_string() } else { title.clone() },
            NodeType::Article,
            Some(&summary),
            Some(url),
            Some(domain),
        )?;

        let links = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_links(&html, url);
        let ts = now();
        for link in links.iter().take(20) {
            let link_domain = link.split('/').nth(2).unwrap_or("").trim_start_matches("www.").to_string();
            if link_domain == domain || link_domain.is_empty() { continue; }
            let conn = self.kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
            let _ = nt_memory_store::upsert_crawl_queue(&conn, link, 1, &link_domain, 0, ts);
            drop(conn);
        }

        Ok(WebPageReport {
            url: url.to_string(),
            title,
            nodes_created: 1,
            success: true,
            error: None,
        })
    }

    fn persist_cycle_report(&self, report: &AbsorbCycleReport) -> Result<(), String> {
        let json = serde_json::to_string(report).map_err(|e| format!("serde: {}", e))?;
        let _ = self.kb.kv_set("absorber", "last_cycle", &json);
        Ok(())
    }

    pub fn absorb_video_production(
        &self,
        topic: &str,
        manifest: &[(crate::l2_perception::nt_world::nt_world_video_pipeline::ProductionStage, String)],
        asset_stats: (usize, usize, usize),
    ) -> Result<String, String> {
        let mut summary = format!("video production: {}", topic);
        for (stage, artifact) in manifest {
            summary.push_str(&format!("\n- {}: {}", stage.label(), artifact));
        }
        summary.push_str(&format!(
            "\nasset enrichment: total={} dup={} kept={}",
            asset_stats.0, asset_stats.1, asset_stats.2
        ));
        let _node_id = self.kb.insert_or_get_node(
            &format!("video-production-{}", topic),
            NodeType::Resource,
            Some(&summary),
            None,
            Some("nvda"),
        )?;
        let _ = self.kb.kv_set("absorber", "last_video_production", &summary);
        Ok(summary)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorberStatus {
    pub total_nodes: usize,
    pub repositories: usize,
    pub papers: usize,
    pub articles: usize,
    pub concepts: usize,
    pub code_snippets: usize,
    pub insights: usize,
    pub stale_repos: usize,
    pub last_cycle: Option<String>,
}

mod nt_memory_kb_crawl {
    use rusqlite::Connection;
    pub fn run_crawl_cycle(conn: &Connection, max: usize) -> Result<super::CrawlCycleReport, String> {
        let r = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::run_crawl_cycle(conn, max)?;
        Ok(super::CrawlCycleReport {
            attempted: r.attempted,
            completed: r.completed,
            failed: r.failed,
            nodes_created: r.nodes_created,
            edges_created: r.edges_created,
            urls_processed: r.urls_processed,
            errors: r.errors,
            by_domain: r.by_domain,
        })
    }
    pub fn discover_from_seed(conn: &Connection, topic: &str) -> Result<usize, String> {
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::discover_from_seed(conn, topic)
    }
}

mod nt_memory_kb_discovery {
    pub use crate::l1_action::nt_memory::nt_memory_kb::nt_discovery_github_topics::DiscoveryPipelineConfig;
}

mod nt_memory_store {
    use rusqlite::Connection;
    pub fn upsert_crawl_queue(conn: &Connection, url: &str, depth: i64, domain: &str, priority: i64, ts: i64) -> rusqlite::Result<()> {
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::upsert_crawl_queue(conn, url, depth, domain, priority, ts)
    }
    pub fn count_nodes(conn: &Connection) -> rusqlite::Result<usize> {
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::count_nodes(conn)
    }
    pub fn count_nodes_by_type(conn: &Connection, node_type: &str) -> rusqlite::Result<usize> {
        crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::count_nodes_by_type(conn, node_type)
    }
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
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

    #[test]
    fn test_source_from_url() {
        assert_eq!(
            AbsorbSource::from_url("https://github.com/rust-lang/rust").unwrap(),
            AbsorbSource::GitHubUrl("https://github.com/rust-lang/rust".into())
        );
        assert_eq!(
            AbsorbSource::from_url("https://arxiv.org/abs/2301.12345").unwrap(),
            AbsorbSource::ArXiv("2301.12345".into())
        );
        assert_eq!(
            AbsorbSource::from_url("https://en.wikipedia.org/wiki/Artificial_intelligence").unwrap(),
            AbsorbSource::Wikipedia("Artificial intelligence".into())
        );
        assert!(
            AbsorbSource::from_url("https://example.com/page").is_some(),
            "Plain webpage should be caught as WebPage"
        );
    }

    #[test]
    fn test_extract_html() {
        let (title, text) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(
            "<html><title>Test</title><body><p>Hello world</p></body></html>");
        assert_eq!(title, "Test");
        assert!(text.contains("Hello world"));
    }

    #[test]
    fn test_extract_html_script_and_style_stripping() {
        let html = r#"<html><title>Page</title><body>
<script>alert("xss")</script>
<style>.cls{color:red}</style>
<p>Visible content</p>
</body></html>"#;
        let (title, text) = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_html_content(html);
        assert_eq!(title, "Page");
        assert!(text.contains("Visible content"), "Visible text should survive");
        assert!(!text.contains("alert"), "Script content should be stripped");
        assert!(!text.contains(".cls"), "Style content should be stripped");
    }

    #[test]
    fn test_extract_links() {
        let html = r#"<a href="http://8.8.8.8/page1">Link 1</a><a href="http://8.8.8.8/page2">Link 2</a>"#;
        let links = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_links(html, "");
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|l| l.contains("page1")));
        assert!(links.iter().any(|l| l.contains("page2")));
    }

    #[test]
    fn test_extract_links_deduplication() {
        let html = r#"<a href="http://8.8.8.8/page">Link</a><a href="http://8.8.8.8/page">Dup</a>"#;
        let links = crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::extract_links(html, "");
        assert_eq!(links.len(), 1, "Duplicate links should be deduped");
    }

    #[test]
    fn test_absorb_source_from_url_edge_cases() {
        let result = AbsorbSource::from_url("https://github.com/user/repo.git").unwrap();
        assert_eq!(result, AbsorbSource::GitHubUrl("https://github.com/user/repo.git".into()));

        let result = AbsorbSource::from_url("https://arxiv.org/abs/2301.12345v2").unwrap();
        assert_eq!(result, AbsorbSource::ArXiv("2301.12345v2".into()));
    }

    #[test]
    fn test_absorb_cycle_report_default_values() {
        let report = AbsorbCycleReport {
            sources_attempted: 0,
            sources_succeeded: 0,
            sources_failed: 0,
            nodes_created: 0,
            edges_created: 0,
            github_repos: Vec::new(),
            web_pages: Vec::new(),
            errors: Vec::new(),
            timestamp: now(),
        };
        assert_eq!(report.sources_attempted, 0);
        assert_eq!(report.sources_succeeded, 0);
        assert_eq!(report.nodes_created, 0);
    }

    #[test]
    fn test_absorber_status_unknown_domain() {
        let status = AbsorberStatus {
            total_nodes: 10,
            repositories: 3,
            papers: 1,
            articles: 2,
            concepts: 4,
            code_snippets: 0,
            insights: 0,
            stale_repos: 0,
            last_cycle: None,
        };
        assert_eq!(status.total_nodes, 10);
        assert_eq!(status.repositories, 3);
        assert_eq!(status.papers, 1);
        assert!(status.last_cycle.is_none());
    }

    #[test]
    fn test_api_registry_register_and_stats_wired() {
        let entry = api_registry::ApiEntry {
            name: "OpenAI".into(),
            description: "AI".into(),
            auth: "apiKey".into(),
            https: true,
            category: "Machine Learning".into(),
            cors: true,
            url: "https://api.openai.com".into(),
        };
        let kb = KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap();
        let mut absorber = UnifiedAbsorber::new(kb, AbsorberConfig {
            max_github_stars: 100,
            max_source_files: 10,
            enable_readme: true,
            enable_deps: false,
            enable_insights: false,
            auto_refresh_days: 7,
            max_concurrent: 4,
        }).unwrap();
        absorber.register_api(entry);
        let (n, https, _auths) = absorber.api_registry_stats();
        assert_eq!(n, 1);
        assert_eq!(https, 1.0);
    }

    #[test]
    fn test_absorb_video_production_writes_kb_node() {
        use crate::l2_perception::nt_world::nt_world_video_pipeline::ProductionStage;
        let kb = KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap();
        let absorber = UnifiedAbsorber::new(kb, AbsorberConfig {
            max_github_stars: 100,
            max_source_files: 10,
            enable_readme: true,
            enable_deps: false,
            enable_insights: false,
            auto_refresh_days: 7,
            max_concurrent: 4,
        }).unwrap();
        let manifest = vec![
            (ProductionStage::Script, "script-x".into()),
            (ProductionStage::Compose, "out/x-final.mp4".into()),
            (ProductionStage::Publish, "published/x-final.mp4".into()),
        ];
        let summary = absorber.absorb_video_production("X", &manifest, (3, 1, 2)).unwrap();
        assert!(summary.contains("video production: X"));
        assert!(summary.contains("asset enrichment: total=3 dup=1 kept=2"));
    }
}
