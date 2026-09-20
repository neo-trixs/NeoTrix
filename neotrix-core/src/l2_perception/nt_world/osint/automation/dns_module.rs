//! DnsModule — DNS record lookup via std::net + DoH fallback.
//!
//! R-P48: Zero external binary deps. Uses `std::net::ToSocketAddrs` for A/AAAA
//! and Cloudflare/Google DoH for MX, NS, TXT, CNAME.

use std::collections::HashSet;
use std::net::ToSocketAddrs;

use super::module::{Finding, ModuleCategory, ModuleInput, ModuleOutput, OsintModule};

/// DNS result from resolving a domain.
#[derive(Debug, Clone)]
pub struct DnsResult {
    pub a_records: Vec<String>,
    pub aaaa_records: Vec<String>,
    pub mx_records: Vec<DnsRecord>,
    pub ns_records: Vec<DnsRecord>,
    pub txt_records: Vec<DnsRecord>,
    pub cname_records: Vec<DnsRecord>,
}

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub host: String,
    pub value: String,
}

pub struct DnsModule;

impl DnsModule {
    pub fn new() -> Self {
        Self
    }

    /// Pure Rust A/AAAA resolution via std::net (R-P48).
    fn resolve_a(domain: &str) -> Vec<String> {
        let mut ips = Vec::new();
        let mut seen = HashSet::new();
        let addr = format!("{}:0", domain);
        if let Ok(addrs) = addr.to_socket_addrs() {
            for sa in addrs {
                let ip = sa.ip().to_string();
                if seen.insert(ip.clone()) {
                    ips.push(ip);
                }
            }
        }
        ips
    }

    /// DoH query via reqwest (allowed by R-P48 — reqwest is an explicit carve-out).
    fn query_doh(domain: &str, record_type: &str) -> Vec<DnsRecord> {
        let type_map: [(&str, &str); 6] = [
            ("A", "1"),
            ("AAAA", "28"),
            ("MX", "15"),
            ("TXT", "16"),
            ("NS", "2"),
            ("CNAME", "5"),
        ];
        let Some(&dns_type) = type_map
            .iter()
            .find(|(t, _)| *t == record_type)
            .map(|(_, v)| v)
        else {
            return Vec::new();
        };

        let urls = [
            format!(
                "https://cloudflare-dns.com/dns-query?name={}&type={}",
                domain, dns_type
            ),
            format!(
                "https://dns.google/resolve?name={}&type={}",
                domain, dns_type
            ),
        ];

        for url in &urls {
            if let Ok(resp) = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .and_then(|c| c.get(url).header("accept", "application/dns-json").send())
            {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<serde_json::Value>() {
                        if let Some(answer) = json["Answer"].as_array() {
                            return answer
                                .iter()
                                .filter_map(|ans| {
                                    let data = ans["data"].as_str()?;
                                    let name = ans["name"].as_str().unwrap_or(domain);
                                    Some(DnsRecord {
                                        host: name.to_string(),
                                        value: data.to_string(),
                                    })
                                })
                                .collect();
                        }
                    }
                }
            }
        }
        Vec::new()
    }

    /// Full DNS resolution combining std::net + DoH.
    pub fn resolve(domain: &str) -> DnsResult {
        let a_records = Self::resolve_a(domain);
        let aaaa_records = Self::query_doh(domain, "AAAA")
            .into_iter()
            .map(|r| r.value)
            .collect();
        let mx_records = Self::query_doh(domain, "MX");
        let ns_records = Self::query_doh(domain, "NS");
        let txt_records = Self::query_doh(domain, "TXT");
        let cname_records = Self::query_doh(domain, "CNAME");

        DnsResult {
            a_records,
            aaaa_records,
            mx_records,
            ns_records,
            txt_records,
            cname_records,
        }
    }
}

impl OsintModule for DnsModule {
    fn name(&self) -> &'static str {
        "dns_lookup"
    }

    fn category(&self) -> ModuleCategory {
        ModuleCategory::Dns
    }

    fn execute(&self, input: &ModuleInput) -> ModuleOutput {
        let result = Self::resolve(&input.target);
        let mut findings = Vec::new();

        for ip in &result.a_records {
            findings.push(Finding::new("dns_a", "ip", ip.clone(), "std::net", 1.0));
        }
        for ip in &result.aaaa_records {
            findings.push(Finding::new("dns_aaaa", "ip", ip.clone(), "doh", 0.95));
        }
        for mx in &result.mx_records {
            findings.push(Finding::new(
                "dns_mx",
                "mail_server",
                format!("{} -> {}", mx.host, mx.value),
                "doh",
                0.95,
            ));
        }
        for ns in &result.ns_records {
            findings.push(Finding::new(
                "dns_ns",
                "nameserver",
                format!("{} -> {}", ns.host, ns.value),
                "doh",
                0.95,
            ));
        }
        for txt in &result.txt_records {
            findings.push(Finding::new(
                "dns_txt",
                "record",
                format!("{} -> {}", txt.host, txt.value),
                "doh",
                0.9,
            ));
        }
        for cname in &result.cname_records {
            findings.push(Finding::new(
                "dns_cname",
                "alias",
                format!("{} -> {}", cname.host, cname.value),
                "doh",
                0.9,
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
            duration_ms: 0, // filled by execute_timed
        }
    }
}

impl Default for DnsModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_a_localhost() {
        let result = DnsModule::resolve_a("localhost");
        assert!(!result.is_empty(), "localhost should resolve");
    }

    #[test]
    fn module_name() {
        assert_eq!(DnsModule::new().name(), "dns_lookup");
    }

    #[test]
    fn module_category() {
        assert_eq!(DnsModule::new().category(), ModuleCategory::Dns);
    }

    #[test]
    fn execute_returns_findings_for_localhost() {
        let input = ModuleInput::new("localhost");
        let output = DnsModule::new().execute(&input);
        assert!(!output.findings.is_empty());
        assert!(output.confidence > 0.0);
    }
}
