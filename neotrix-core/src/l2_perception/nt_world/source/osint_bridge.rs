//! OSINT 源桥接层 — 将现有 30+ 个 OSINT 模块适配到统一架构
//!
//! 将 `osint/` 下的真实 OSINT 模块包装为统一的 `OsintSource` trait，
//! 每个桥接器内部调用对应模块的 `investigate()` 并将 findings 转换为 `OsintResult`。

use super::unified::*;
use super::super::osint::{OsintConfig, OsintTarget};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// ───────────────────────────────────────────────────────────────────────────
// Target 解析工具
// ───────────────────────────────────────────────────────────────────────────

fn parse_target(target: &str) -> OsintTarget {
    let t = target.trim();
    if t.contains('@') {
        OsintTarget::from_email(t)
    } else if t.starts_with("http://") || t.starts_with("https://") {
        OsintTarget { url: Some(t.to_string()), ..Default::default() }
    } else if t.parse::<std::net::IpAddr>().is_ok() {
        OsintTarget { ip: Some(t.to_string()), ..Default::default() }
    } else if t.contains('.') && !t.starts_with('.') {
        OsintTarget::from_domain(t)
    } else {
        OsintTarget::from_username(t)
    }
}

fn default_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .https_only(true)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn default_config() -> OsintConfig {
    OsintConfig::default()
}

fn findings_to_result(source_id: &str, target: &str, _category: &str, items: Vec<OsintFinding>) -> OsintResult {
    let confidence = if items.is_empty() { 0.0 } else { items.iter().map(|f| f.confidence).sum::<f64>() / items.len() as f64 };
    OsintResult {
        source_id: source_id.to_string(),
        target: target.to_string(),
        findings: items,
        confidence,
    }
}

// ───────────────────────────────────────────────────────────────────────────
// DNS Bridge
// ───────────────────────────────────────────────────────────────────────────

struct DnsBridge;

impl DataSource for DnsBridge {
    fn id(&self) -> &str { "dns" }
    fn name(&self) -> &str { "DNS Investigation" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for DnsBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::dns::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for rec in &findings.subdomains {
                items.push(OsintFinding { category: "dns.subdomain".into(), key: rec.name.clone(), value: rec.value.clone(), confidence: 0.9, source: rec.source.clone() });
            }
            for rec in &findings.a_records {
                items.push(OsintFinding { category: "dns.a".into(), key: rec.name.clone(), value: rec.value.clone(), confidence: 1.0, source: "system-resolver".into() });
            }
            for rec in &findings.mx_records {
                items.push(OsintFinding { category: "dns.mx".into(), key: rec.name.clone(), value: rec.value.clone(), confidence: 1.0, source: "doh".into() });
            }
            for rec in &findings.txt_records {
                items.push(OsintFinding { category: "dns.txt".into(), key: rec.name.clone(), value: rec.value.clone(), confidence: 1.0, source: "doh".into() });
            }
            for rec in &findings.ns_records {
                items.push(OsintFinding { category: "dns.ns".into(), key: rec.name.clone(), value: rec.value.clone(), confidence: 1.0, source: "doh".into() });
            }
            for rec in &findings.cname_records {
                items.push(OsintFinding { category: "dns.cname".into(), key: rec.name.clone(), value: rec.value.clone(), confidence: 1.0, source: "doh".into() });
            }
            for rec in &findings.aaaa_records {
                items.push(OsintFinding { category: "dns.aaaa".into(), key: rec.name.clone(), value: rec.value.clone(), confidence: 1.0, source: "doh".into() });
            }
            Ok(findings_to_result("dns", &t, "dns", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["domain", "ip"] }
}

// ───────────────────────────────────────────────────────────────────────────
// HTTP Bridge
// ───────────────────────────────────────────────────────────────────────────

struct HttpBridge;

impl DataSource for HttpBridge {
    fn id(&self) -> &str { "http" }
    fn name(&self) -> &str { "HTTP Analysis" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for HttpBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::http::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for ep in &findings.endpoints {
                items.push(OsintFinding {
                    category: "http.endpoint".into(),
                    key: ep.url.clone(),
                    value: format!("{} [{}ms]", ep.status, ep.response_time_ms),
                    confidence: 1.0,
                    source: "http-probe".into(),
                });
                if let Some(ref title) = ep.title {
                    items.push(OsintFinding { category: "http.title".into(), key: ep.url.clone(), value: title.clone(), confidence: 0.9, source: "http-probe".into() });
                }
                if let Some(ref server) = ep.server {
                    items.push(OsintFinding { category: "http.server".into(), key: ep.url.clone(), value: server.clone(), confidence: 0.95, source: "http-probe".into() });
                }
                if !ep.tech_stack.is_empty() {
                    items.push(OsintFinding { category: "http.tech".into(), key: ep.url.clone(), value: ep.tech_stack.join(", "), confidence: 0.8, source: "tech-detect".into() });
                }
            }
            Ok(findings_to_result("http", &t, "http", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["domain", "url"] }
}

// ───────────────────────────────────────────────────────────────────────────
// Shodan Bridge
// ───────────────────────────────────────────────────────────────────────────

struct ShodanBridge;

impl DataSource for ShodanBridge {
    fn id(&self) -> &str { "shodan" }
    fn name(&self) -> &str { "Shodan" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for ShodanBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::shodan::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for svc in &findings.services {
                let label = svc.product.as_deref().unwrap_or("unknown");
                items.push(OsintFinding { category: "shodan.service".into(), key: format!("{}:{}", findings.ip, svc.port), value: format!("{}/{} [{}]", svc.port, svc.protocol, label), confidence: 0.9, source: "shodan".into() });
                if let Some(ref banner) = svc.banner {
                    items.push(OsintFinding { category: "shodan.banner".into(), key: format!("{}:{}", findings.ip, svc.port), value: banner.clone(), confidence: 0.85, source: "shodan".into() });
                }
            }
            for vuln in &findings.vulns {
                items.push(OsintFinding { category: "shodan.vuln".into(), key: vuln.clone(), value: format!("vulnerability on {}", findings.ip), confidence: 0.8, source: "shodan".into() });
            }
            if let Some(ref os) = findings.os {
                items.push(OsintFinding { category: "shodan.os".into(), key: findings.ip.clone(), value: os.clone(), confidence: 0.7, source: "shodan".into() });
            }
            if let Some(ref org) = findings.org {
                items.push(OsintFinding { category: "shodan.org".into(), key: findings.ip.clone(), value: org.clone(), confidence: 0.85, source: "shodan".into() });
            }
            Ok(findings_to_result("shodan", &t, "shodan", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["domain", "ip"] }
}

// ───────────────────────────────────────────────────────────────────────────
// Censys Bridge
// ───────────────────────────────────────────────────────────────────────────

struct CensysBridge;

impl DataSource for CensysBridge {
    fn id(&self) -> &str { "censys" }
    fn name(&self) -> &str { "Censys" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for CensysBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::censys::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for svc in &findings.services {
                items.push(OsintFinding { category: "censys.service".into(), key: format!("{}:{}", findings.ip, svc.port), value: format!("{} [{}]", svc.service_name, svc.transport_protocol), confidence: 0.9, source: "censys".into() });
                if let Some(ref banner) = svc.banner {
                    items.push(OsintFinding { category: "censys.banner".into(), key: format!("{}:{}", findings.ip, svc.port), value: banner.clone(), confidence: 0.85, source: "censys".into() });
                }
            }
            if let Some(ref country) = findings.country {
                items.push(OsintFinding { category: "censys.geo".into(), key: findings.ip.clone(), value: country.clone(), confidence: 0.8, source: "censys".into() });
            }
            if let Some(ref asn) = findings.autonomous_system {
                items.push(OsintFinding { category: "censys.asn".into(), key: findings.ip.clone(), value: asn.clone(), confidence: 0.85, source: "censys".into() });
            }
            Ok(findings_to_result("censys", &t, "censys", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["domain", "ip"] }
}

// ───────────────────────────────────────────────────────────────────────────
// ZoomEye Bridge
// ───────────────────────────────────────────────────────────────────────────

struct ZoomEyeBridge;

impl DataSource for ZoomEyeBridge {
    fn id(&self) -> &str { "zoomeye" }
    fn name(&self) -> &str { "ZoomEye" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for ZoomEyeBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::zoomeye::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for svc in &findings.services {
                let label = svc.product.as_deref().unwrap_or(&svc.service);
                items.push(OsintFinding { category: "zoomeye.service".into(), key: format!("{}:{}", findings.ip, svc.port), value: format!("{} [{}]", label, svc.service), confidence: 0.9, source: "zoomeye".into() });
            }
            if let Some(ref os) = findings.os {
                items.push(OsintFinding { category: "zoomeye.os".into(), key: findings.ip.clone(), value: os.clone(), confidence: 0.7, source: "zoomeye".into() });
            }
            if let Some(ref country) = findings.country {
                items.push(OsintFinding { category: "zoomeye.geo".into(), key: findings.ip.clone(), value: country.clone(), confidence: 0.8, source: "zoomeye".into() });
            }
            Ok(findings_to_result("zoomeye", &t, "zoomeye", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["domain", "ip"] }
}

// ───────────────────────────────────────────────────────────────────────────
// FOFA Bridge
// ───────────────────────────────────────────────────────────────────────────

struct FofaBridge;

impl DataSource for FofaBridge {
    fn id(&self) -> &str { "fofa" }
    fn name(&self) -> &str { "FOFA" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for FofaBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::fofa::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for asset in &findings.assets {
                items.push(OsintFinding { category: "fofa.asset".into(), key: asset.host.clone(), value: format!("{}:{}/{} [{}]", asset.ip, asset.port, asset.protocol, asset.product), confidence: 0.85, source: "fofa".into() });
            }
            Ok(findings_to_result("fofa", &t, "fofa", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["domain", "ip"] }
}

// ───────────────────────────────────────────────────────────────────────────
// SecurityTrails Bridge
// ───────────────────────────────────────────────────────────────────────────

struct SecurityTrailsBridge;

impl DataSource for SecurityTrailsBridge {
    fn id(&self) -> &str { "securitytrails" }
    fn name(&self) -> &str { "SecurityTrails" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for SecurityTrailsBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::securitytrails::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for sub in &findings.subdomains {
                let value = sub.ip.as_deref().unwrap_or("unknown");
                items.push(OsintFinding { category: "securitytrails.subdomain".into(), key: sub.name.clone(), value: value.to_string(), confidence: 0.9, source: "securitytrails".into() });
            }
            for rec in &findings.records {
                items.push(OsintFinding { category: "securitytrails.dns".into(), key: format!("{}:{}", rec.record_type, rec.value), value: format!("ttl={}", rec.ttl.unwrap_or(0)), confidence: 0.95, source: "securitytrails".into() });
            }
            if let Some(ref whois) = findings.whois {
                if let Some(ref reg) = whois.registrar {
                    items.push(OsintFinding { category: "securitytrails.whois".into(), key: findings.domain.clone(), value: reg.clone(), confidence: 0.85, source: "securitytrails".into() });
                }
            }
            Ok(findings_to_result("securitytrails", &t, "securitytrails", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["domain"] }
}

// ───────────────────────────────────────────────────────────────────────────
// Vulnerability Bridge
// ───────────────────────────────────────────────────────────────────────────

struct VulnBridge;

impl DataSource for VulnBridge {
    fn id(&self) -> &str { "vuln" }
    fn name(&self) -> &str { "Vulnerability Scan" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for VulnBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::vuln::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for v in &findings.vulnerabilities {
                let severity = v.severity.as_deref().unwrap_or("unknown");
                let cvss = v.cvss_score.map(|s| format!(" cvss={}", s)).unwrap_or_default();
                items.push(OsintFinding { category: "vuln.cve".into(), key: v.id.clone(), value: format!("[{}]{} {}", severity, cvss, v.summary), confidence: 0.9, source: v.source.clone() });
            }
            for adv in &findings.advisories {
                items.push(OsintFinding { category: "vuln.advisory".into(), key: adv.clone(), value: "advisory".into(), confidence: 0.7, source: "vuln-scan".into() });
            }
            Ok(findings_to_result("vuln", &t, "vuln", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["domain", "ip", "cve"] }
}

// ───────────────────────────────────────────────────────────────────────────
// Network Bridge
// ───────────────────────────────────────────────────────────────────────────

struct NetworkBridge;

impl DataSource for NetworkBridge {
    fn id(&self) -> &str { "network" }
    fn name(&self) -> &str { "Network Analysis" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for NetworkBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::network::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for ip in &findings.ip_addresses {
                items.push(OsintFinding { category: "network.ip".into(), key: findings.domain.clone(), value: ip.clone(), confidence: 1.0, source: "dns-resolve".into() });
            }
            for svc in &findings.services {
                let name = svc.service.as_deref().unwrap_or("unknown");
                items.push(OsintFinding { category: "network.service".into(), key: format!("{}:{}", svc.host, svc.port), value: format!("{}/{} [{}]", svc.port, svc.protocol, name), confidence: 0.9, source: "port-scan".into() });
                if let Some(ref banner) = svc.banner {
                    items.push(OsintFinding { category: "network.banner".into(), key: format!("{}:{}", svc.host, svc.port), value: banner.clone(), confidence: 0.85, source: "port-scan".into() });
                }
            }
            if let Some(ref asn) = findings.asn_info {
                items.push(OsintFinding { category: "network.asn".into(), key: findings.domain.clone(), value: asn.clone(), confidence: 0.8, source: "asn-lookup".into() });
            }
            Ok(findings_to_result("network", &t, "network", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["domain", "ip"] }
}

// ───────────────────────────────────────────────────────────────────────────
// Dark Web Bridge
// ───────────────────────────────────────────────────────────────────────────

struct DarkBridge;

impl DataSource for DarkBridge {
    fn id(&self) -> &str { "dark" }
    fn name(&self) -> &str { "Dark Web Monitor" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for DarkBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::dark::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for r in &findings.results {
                items.push(OsintFinding { category: "dark.reference".into(), key: r.url.clone(), value: format!("[{}] {}", r.source, r.title), confidence: 0.7, source: "dark-web".into() });
            }
            for onion in &findings.onion_links {
                items.push(OsintFinding { category: "dark.onion".into(), key: onion.clone(), value: ".onion link".into(), confidence: 0.6, source: "dark-web".into() });
            }
            Ok(findings_to_result("dark", &t, "dark", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["domain", "keyword"] }
}

// ───────────────────────────────────────────────────────────────────────────
// Person Bridge
// ───────────────────────────────────────────────────────────────────────────

struct PersonBridge;

impl DataSource for PersonBridge {
    fn id(&self) -> &str { "person" }
    fn name(&self) -> &str { "Person Investigation" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for PersonBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::person::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for profile in &findings.profiles {
                items.push(OsintFinding {
                    category: "person.profile".into(),
                    key: format!("{}:{}", profile.platform, profile.username),
                    value: profile.url.clone(),
                    confidence: 0.9,
                    source: "profile-check".into(),
                });
                if let Some(ref bio) = profile.bio {
                    items.push(OsintFinding { category: "person.bio".into(), key: profile.username.clone(), value: bio.clone(), confidence: 0.7, source: "profile-check".into() });
                }
            }
            Ok(findings_to_result("person", &t, "person", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["email", "username", "name"] }
}

// ───────────────────────────────────────────────────────────────────────────
// Social Bridge
// ───────────────────────────────────────────────────────────────────────────

struct SocialBridge;

impl DataSource for SocialBridge {
    fn id(&self) -> &str { "social" }
    fn name(&self) -> &str { "Social Media OSINT" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for SocialBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::social::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for post in &findings.posts {
                items.push(OsintFinding {
                    category: "social.post".into(),
                    key: post.url.clone().unwrap_or_default(),
                    value: format!("[{}] {}: {}", post.platform, post.author, post.content.chars().take(200).collect::<String>()),
                    confidence: 0.8,
                    source: "social-media".into(),
                });
            }
            Ok(findings_to_result("social", &t, "social", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["username", "profile_url"] }
}

// ───────────────────────────────────────────────────────────────────────────
// Credential Bridge
// ───────────────────────────────────────────────────────────────────────────

struct CredentialBridge;

impl DataSource for CredentialBridge {
    fn id(&self) -> &str { "credential" }
    fn name(&self) -> &str { "Credential Check" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for CredentialBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::credential::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for breach in &findings.breaches {
                let name = breach.breach_name.as_deref().unwrap_or("unknown");
                items.push(OsintFinding {
                    category: "credential.breach".into(),
                    key: breach.source.clone(),
                    value: format!("[{}] {} — {}", name, breach.breach_date.as_deref().unwrap_or("?"), breach.data_classes.join(", ")),
                    confidence: 0.85,
                    source: breach.source.clone(),
                });
            }
            Ok(findings_to_result("credential", &t, "credential", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["email", "password_hash"] }
}

// ───────────────────────────────────────────────────────────────────────────
// URL History Bridge
// ───────────────────────────────────────────────────────────────────────────

struct UrlHistoryBridge;

impl DataSource for UrlHistoryBridge {
    fn id(&self) -> &str { "url" }
    fn name(&self) -> &str { "URL Analysis" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for UrlHistoryBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::url::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            for snap in &findings.snapshots {
                let status = snap.status.map(|s| s.to_string()).unwrap_or_else(|| "?".into());
                items.push(OsintFinding { category: "url.snapshot".into(), key: snap.url.clone(), value: format!("[{}] {}", snap.timestamp, status), confidence: 0.95, source: "wayback".into() });
            }
            Ok(findings_to_result("url", &t, "url", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["url", "domain"] }
}

// ───────────────────────────────────────────────────────────────────────────
// CryptoPub Bridge
// ───────────────────────────────────────────────────────────────────────────

struct CryptoPubBridge;

impl DataSource for CryptoPubBridge {
    fn id(&self) -> &str { "cryptopub" }
    fn name(&self) -> &str { "Crypto Publication" }
    fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
}

impl OsintSource for CryptoPubBridge {
    fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
        let t = target.to_string();
        Box::pin(async move {
            let osint_target = parse_target(&t);
            let client = default_client();
            let config = default_config();
            let findings = super::super::osint::cryptopub::investigate(&osint_target, &client, &config).await?;
            let mut items = Vec::new();
            items.push(OsintFinding { category: "cryptopub.address".into(), key: findings.address.clone(), value: format!("chain={} balance={} txs={}", findings.chain, findings.balance, findings.tx_count), confidence: 0.8, source: "cryptopub".into() });
            Ok(findings_to_result("cryptopub", &t, "cryptopub", items))
        })
    }
    fn supported_targets(&self) -> Vec<&str> { vec!["address", "tx_hash", "domain"] }
}

// ───────────────────────────────────────────────────────────────────────────
// Placeholder bridges (modules without standalone investigate functions)
//
// ⛔⛔ **ZERO_PRODUCTION_CONSUMER**（2026-10-08 取证；判据落 scripts/security-wiring-baseline.txt）
//
// 本文件 788 行看着像「15 个真桥已接进生产」，实测**生产零消费**，且现有门抓不到：
//
// 1. **10 个桩**（`placeholder_bridge!` 实例化，:639-648）：sweep / ad_graph / harvest /
//    auto_patrol / automation / search_engine / report / social_search / username_checker / metadata
//    —— `investigate` 恒返回空 findings + `confidence: 0.0`（:631）⇒ 连真实逻辑都没调。
//    其中 **ad_graph(578 行) + harvest(1,017 行) 共 1,595 行真实实现被桩悬空**。
// 2. **15 个真桥**（:658-672，DnsBridge…CryptoPubBridge）实现完整且正确，但消费链断两次：
//    create_osint_bridges() → UnifiedEngine::investigate_osint()，而 with_osint_sources()
//    的**唯一**调用点（unified_engine.rs:389）在 `#[cfg(test)]` 内；UnifiedEngine 生产消费者 = 0。
// 3. ⚠️ **不是「整块零消费」**：osint/mod.rs:702 `doctor_osint` 与 :676 `run_osint` 是**生产代码**，
//    经 backend_router.rs 分发真实 investigate —— 但它们同样零外部调用者。
//    ⇒ 准确表述：**15 真桥 + 2 条真实分发路径，全部止步于零生产消费者**。
//
// ⛔ 为什么门抓不到（已实测，非推测）：check-truth-surface.sh 的 UNREACHABLE 判的是
//    「文件能否从 crate root 编译」，而 source/mod.rs:89 已 `pub mod osint_bridge;`
//    ⇒ 可达 ⇒ 不报；nt_orphan_dir.py 判「目录是否被 mod 挂载」，同理不报。
//    ⇒ **「编译进库但生产零消费」是现有门体系的结构性盲区**（L8「绿色≠有效」）。
//
// 判据：**不接线、不删除**。真桥是规格资产且 TODO.yml ds-2/ds-4 已在规划接线；
// 删除会销毁那 1,595 行真实实现。
// ───────────────────────────────────────────────────────────────────────────

macro_rules! placeholder_bridge {
    ($struct_name:ident, $id:expr, $name:expr, $targets:expr) => {
        struct $struct_name;
        impl DataSource for $struct_name {
            fn id(&self) -> &str { $id }
            fn name(&self) -> &str { $name }
            fn domains(&self) -> Vec<SourceDomain> { vec![SourceDomain::Osint] }
        }
        impl OsintSource for $struct_name {
            fn investigate(&self, target: &str) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>> {
                let t = target.to_string();
                let sid = $id.to_string();
                Box::pin(async move {
                    Ok(OsintResult { source_id: sid.clone(), target: t.clone(), findings: vec![], confidence: 0.0 })
                })
            }
            fn supported_targets(&self) -> Vec<&str> { $targets }
        }
    };
}

placeholder_bridge!(SweepBridge, "sweep", "Asset Sweep", ["domain", "ip_range"].to_vec());
placeholder_bridge!(AdGraphBridge, "ad_graph", "Adversary Graph", ["actor", "campaign"].to_vec());
placeholder_bridge!(HarvestBridge, "harvest", "Data Harvest", ["domain", "keyword"].to_vec());
placeholder_bridge!(AutoPatrolBridge, "auto_patrol", "Auto Patrol", ["domain", "ip"].to_vec());
placeholder_bridge!(AutomationBridge, "automation", "OSINT Automation", ["target"].to_vec());
placeholder_bridge!(SearchEngineBridge, "search_engine", "Search Engine OSINT", ["query", "keyword"].to_vec());
placeholder_bridge!(ReportBridge, "report", "OSINT Report", ["target"].to_vec());
placeholder_bridge!(SocialSearchBridge, "social_search", "Social Search (55+ platforms)", ["username"].to_vec());
placeholder_bridge!(UsernameCheckerBridge, "username_checker", "Username Checker", ["username"].to_vec());
placeholder_bridge!(MetadataBridge, "metadata", "Metadata Extraction", ["url", "file"].to_vec());

// ───────────────────────────────────────────────────────────────────────────
// Public API
// ───────────────────────────────────────────────────────────────────────────

/// 创建所有 OSINT 源桥接器 — 真实实现 + placeholder
pub fn create_osint_bridges() -> Vec<Arc<dyn OsintSource>> {
    vec![
        // ── 真实实现 (15 个) ──
        Arc::new(DnsBridge),
        Arc::new(HttpBridge),
        Arc::new(ShodanBridge),
        Arc::new(CensysBridge),
        Arc::new(ZoomEyeBridge),
        Arc::new(FofaBridge),
        Arc::new(SecurityTrailsBridge),
        Arc::new(VulnBridge),
        Arc::new(NetworkBridge),
        Arc::new(DarkBridge),
        Arc::new(PersonBridge),
        Arc::new(SocialBridge),
        Arc::new(CredentialBridge),
        Arc::new(UrlHistoryBridge),
        Arc::new(CryptoPubBridge),
        // ── Placeholder (10 个, 待实现) ──
        Arc::new(SweepBridge),
        Arc::new(AdGraphBridge),
        Arc::new(HarvestBridge),
        Arc::new(AutoPatrolBridge),
        Arc::new(AutomationBridge),
        Arc::new(SearchEngineBridge),
        Arc::new(ReportBridge),
        Arc::new(SocialSearchBridge),
        Arc::new(UsernameCheckerBridge),
        Arc::new(MetadataBridge),
    ]
}

/// 创建所有 OSINT 源桥接器 (统一入口)
pub fn bridge_all_osint_sources() -> Vec<Arc<dyn OsintSource>> {
    create_osint_bridges()
}

/// 按目标类型查找可用的 OSINT 源
pub fn find_sources_for_target<'a>(
    bridges: &'a [Arc<dyn OsintSource>],
    target_type: &str,
) -> Vec<&'a Arc<dyn OsintSource>> {
    bridges
        .iter()
        .filter(|b| b.supported_targets().contains(&target_type))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target_domain() {
        let t = parse_target("example.com");
        assert_eq!(t.domain.as_deref(), Some("example.com"));
    }

    #[test]
    fn test_parse_target_ip() {
        let t = parse_target("8.8.8.8");
        assert_eq!(t.ip.as_deref(), Some("8.8.8.8"));
    }

    #[test]
    fn test_parse_target_email() {
        let t = parse_target("user@example.com");
        assert_eq!(t.email.as_deref(), Some("user@example.com"));
    }

    #[test]
    fn test_parse_target_url() {
        let t = parse_target("https://example.com/path");
        assert_eq!(t.url.as_deref(), Some("https://example.com/path"));
    }

    #[test]
    fn test_parse_target_username() {
        let t = parse_target("johndoe");
        assert_eq!(t.username.as_deref(), Some("johndoe"));
    }

    #[test]
    fn test_create_osint_bridges() {
        let bridges = create_osint_bridges();
        assert_eq!(bridges.len(), 25);
    }

    #[test]
    fn test_bridge_all_osint_sources() {
        let bridges = bridge_all_osint_sources();
        assert_eq!(bridges.len(), 25);
    }

    #[test]
    fn test_find_sources_for_target_domain() {
        let bridges = create_osint_bridges();
        let domain_sources = find_sources_for_target(&bridges, "domain");
        assert!(domain_sources.len() >= 10);
    }

    #[test]
    fn test_find_sources_for_target_ip() {
        let bridges = create_osint_bridges();
        let ip_sources = find_sources_for_target(&bridges, "ip");
        assert!(!ip_sources.is_empty());
    }

    #[test]
    fn test_find_sources_for_target_username() {
        let bridges = create_osint_bridges();
        let username_sources = find_sources_for_target(&bridges, "username");
        assert!(!username_sources.is_empty());
    }

    #[test]
    fn test_bridge_ids() {
        let bridges = create_osint_bridges();
        let ids: Vec<&str> = bridges.iter().map(|b| b.id()).collect();
        assert!(ids.contains(&"dns"));
        assert!(ids.contains(&"shodan"));
        assert!(ids.contains(&"person"));
        assert!(ids.contains(&"credential"));
    }

    #[test]
    fn test_bridge_names() {
        let bridges = create_osint_bridges();
        let names: Vec<&str> = bridges.iter().map(|b| b.name()).collect();
        assert!(names.contains(&"DNS Investigation"));
        assert!(names.contains(&"Shodan"));
        assert!(names.contains(&"Person Investigation"));
    }
}
