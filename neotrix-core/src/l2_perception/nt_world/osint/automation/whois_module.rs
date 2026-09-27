//! WhoisModule — WHOIS lookup via raw TCP (R-P48: zero external binaries).
//!
//! Connects to IANA/verisign whois servers on port 43, sends a query,
//! and parses the text response for registrar, dates, and name servers.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::module::{Finding, ModuleCategory, ModuleInput, ModuleOutput, OsintModule};

/// Parsed WHOIS result for a domain.
#[derive(Debug, Clone)]
pub struct WhoisResult {
    pub domain: String,
    pub registrar: Option<String>,
    pub creation_date: Option<String>,
    pub expiry_date: Option<String>,
    pub name_servers: Vec<String>,
    pub registrant: Option<String>,
    pub status: Option<String>,
    pub raw_response: String,
}

/// TLD → whois server mapping (common TLDs only).
fn whois_server_for_domain(domain: &str) -> &'static str {
    let tld = domain.rsplit('.').next().unwrap_or("");
    match tld {
        "com" | "net" => "whois.verisign-grs.com",
        "org" => "whois.pir.org",
        "io" => "whois.nic.io",
        "dev" => "whois.nic.google",
        "ai" => "whois.nic.ai",
        "co" => "whois.nic.co",
        "uk" => "whois.nic.uk",
        "de" => "whois.denic.de",
        "fr" => "whois.nic.fr",
        "ru" => "whois.tcinet.ru",
        "cn" => "whois.cnnic.cn",
        "jp" => "whois.jprs.jp",
        "kr" => "whois.kr",
        "au" => "whois.auda.org.au",
        "ca" => "whois.cira.ca",
        "nl" => "whois.sidn.nl",
        "be" => "whois.dns.be",
        "ch" => "whois.nic.ch",
        "se" => "whois.iis.se",
        "no" => "whois.norid.no",
        "fi" => "whois.fi",
        "dk" => "whois.dk-hostmaster.dk",
        "pl" => "whois.dns.pl",
        "cz" => "whois.nic.cz",
        "at" => "whois.nic.at",
        "eu" => "whois.eu",
        "info" => "whois.afilias.net",
        "biz" => "whois.neulevel.biz",
        "name" => "whois.nic.name",
        "mobi" => "whois.afilias.net",
        "pro" => "whois.afilias.net",
        "aero" => "whois.aero",
        "coop" => "whois.nic.coop",
        "museum" => "whois.nic.museum",
        "jobs" => "whois.nic.jobs",
        "travel" => "whois.nic.travel",
        "xxx" => "whois.nic.xxx",
        _ => "whois.iana.org",
    }
}

/// Extract a field value from WHOIS text response.
fn extract_field(response: &str, keys: &[&str]) -> Option<String> {
    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('%') || trimmed.is_empty() {
            continue;
        }
        for key in keys {
            // 2026-09-27 修复: 在**小写副本**上剥前缀 → 返回值也被小写化
            // ("Example Inc" 变 "example inc", 实体名失真)。改为小写仅用于
            // 匹配判定, 取值切回原始行。
            if trimmed.len() >= key.len()
                && trimmed[..key.len()].eq_ignore_ascii_case(key)
            {
                let val = trimmed[key.len()..].trim().trim_start_matches(':').trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

/// Extract all name server entries from WHOIS response.
fn extract_name_servers(response: &str) -> Vec<String> {
    let mut servers = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in response.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        if lower.starts_with("name server") || lower.starts_with("nserver") {
            if let Some(val) = trimmed.splitn(2, ':').nth(1) {
                let ns = val.trim().to_lowercase();
                if !ns.is_empty() && seen.insert(ns.clone()) {
                    servers.push(ns);
                }
            }
        }
    }
    servers
}

/// Query WHOIS via raw TCP (R-P48: no external binaries, no reqwest needed).
fn query_whois(domain: &str, server: &str) -> Result<String, String> {
    let stream = TcpStream::connect_timeout(
        &format!("{}:43", server)
            .parse()
            .map_err(|e| format!("bad server addr: {}", e))?,
        Duration::from_secs(10),
    )
    .map_err(|e| format!("TCP connect to {} failed: {}", server, e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| format!("set_read_timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("set_write_timeout: {}", e))?;

    let mut writer = std::io::BufWriter::new(&stream);
    writer
        .write_all(format!("{}\r\n", domain).as_bytes())
        .map_err(|e| format!("write query: {}", e))?;
    writer.flush().map_err(|e| format!("flush: {}", e))?;

    let mut response = String::new();
    let reader = BufReader::new(&stream);
    for line in reader.lines() {
        match line {
            Ok(l) => {
                response.push_str(&l);
                response.push('\n');
            }
            Err(e) => {
                if response.is_empty() {
                    return Err(format!("read response: {}", e));
                }
                break;
            }
        }
    }

    Ok(response)
}

pub struct WhoisModule;

impl WhoisModule {
    pub fn new() -> Self {
        Self
    }

    /// Full WHOIS lookup: connect → query → parse.
    pub fn lookup(domain: &str) -> WhoisResult {
        let server = whois_server_for_domain(domain);
        let raw = query_whois(domain, server).unwrap_or_default();

        let registrar = extract_field(&raw, &["registrar", "registrant organization"]);
        let creation_date = extract_field(
            &raw,
            &[
                "creation date",
                "created",
                "registration time",
                "registered on",
            ],
        );
        let expiry_date = extract_field(
            &raw,
            &[
                "registry expiry date",
                "expiry date",
                "expires",
                "paid-till",
                "validity",
            ],
        );
        let name_servers = extract_name_servers(&raw);
        let registrant = extract_field(&raw, &["registrant name", "registrant"]);
        let status = extract_field(&raw, &["domain status"]);

        WhoisResult {
            domain: domain.to_string(),
            registrar,
            creation_date,
            expiry_date,
            name_servers,
            registrant,
            status,
            raw_response: raw,
        }
    }
}

impl OsintModule for WhoisModule {
    fn name(&self) -> &'static str {
        "whois_lookup"
    }

    fn category(&self) -> ModuleCategory {
        ModuleCategory::Whois
    }

    fn execute(&self, input: &ModuleInput) -> ModuleOutput {
        let result = Self::lookup(&input.target);
        let mut findings = Vec::new();

        if let Some(ref reg) = result.registrar {
            findings.push(Finding::new(
                "whois_registrar",
                "registrar",
                reg.clone(),
                &format!("whois:{}", whois_server_for_domain(&result.domain)),
                0.9,
            ));
        }
        if let Some(ref created) = result.creation_date {
            findings.push(Finding::new(
                "whois_creation_date",
                "date",
                created.clone(),
                "whois",
                0.85,
            ));
        }
        if let Some(ref expiry) = result.expiry_date {
            findings.push(Finding::new(
                "whois_expiry_date",
                "date",
                expiry.clone(),
                "whois",
                0.85,
            ));
        }
        for ns in &result.name_servers {
            findings.push(Finding::new(
                "whois_nameserver",
                "nameserver",
                ns.clone(),
                "whois",
                0.9,
            ));
        }
        if let Some(ref reg) = result.registrant {
            findings.push(Finding::new(
                "whois_registrant",
                "entity",
                reg.clone(),
                "whois",
                0.7,
            ));
        }
        if let Some(ref status) = result.status {
            findings.push(Finding::new(
                "whois_status",
                "domain_status",
                status.clone(),
                "whois",
                0.8,
            ));
        }

        let confidence = if findings.is_empty() {
            0.0
        } else {
            findings.iter().map(|f| f.confidence).sum::<f64>() / findings.len() as f64
        };

        ModuleOutput {
            findings,
            confidence,
            duration_ms: 0,
        }
    }
}

impl Default for WhoisModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_mapping() {
        assert_eq!(
            whois_server_for_domain("example.com"),
            "whois.verisign-grs.com"
        );
        assert_eq!(whois_server_for_domain("example.org"), "whois.pir.org");
        assert_eq!(whois_server_for_domain("example.io"), "whois.nic.io");
        assert_eq!(whois_server_for_domain("example.xyz"), "whois.iana.org");
    }

    #[test]
    fn extract_field_basic() {
        let resp = "Registrar: Example Inc\nCreation Date: 2020-01-01\n";
        assert_eq!(
            extract_field(resp, &["registrar"]),
            Some("Example Inc".into())
        );
        assert_eq!(
            extract_field(resp, &["creation date"]),
            Some("2020-01-01".into())
        );
    }

    #[test]
    fn test_extract_name_servers() {
        let resp = "Name Server: NS1.EXAMPLE.COM\nName Server: ns2.example.com\n";
        let ns = super::extract_name_servers(resp);
        assert_eq!(ns.len(), 2);
        assert!(ns.contains(&"ns1.example.com".to_string()));
    }

    #[test]
    fn module_identity() {
        let m = WhoisModule::new();
        assert_eq!(m.name(), "whois_lookup");
        assert_eq!(m.category(), ModuleCategory::Whois);
    }
}
