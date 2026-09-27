use std::collections::HashSet;
use std::net::ToSocketAddrs;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::super::OsintConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubdomainSource {
    DnsBrute,
    CrtSh,
    Permutation,
}

impl std::fmt::Display for SubdomainSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubdomainSource::DnsBrute => write!(f, "dns_brute"),
            SubdomainSource::CrtSh => write!(f, "crt_sh"),
            SubdomainSource::Permutation => write!(f, "permutation"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubdomainResult {
    pub subdomain: String,
    pub ip_addresses: Vec<String>,
    pub source: SubdomainSource,
}

pub struct SubdomainHarvester {
    client: Client,
}

const COMMON_PREFIXES: &[&str] = &[
    "www", "mail", "admin", "api", "blog", "dev", "test", "stage", "prod",
    "beta", "app", "m", "mobile", "cdn", "static", "assets", "media", "img",
    "images", "css", "js", "fonts", "upload", "download", "ftp", "ssh", "vpn",
    "portal", "login", "auth", "sso", "oauth", "git", "ci", "cd",
    "jenkins", "jira", "confluence", "wiki", "docs", "help", "support",
    "monitor", "grafana", "prometheus", "log", "syslog", "audit",
    "db", "database", "redis", "mysql", "postgres", "mongo", "elastic", "kibana",
    "web", "webmail", "ns1", "ns2", "mx", "smtp", "pop3", "imap",
    "calendar", "drive", "cloud", "hub", "edge", "core", "platform", "gateway",
    "graphql", "rest", "v1", "v2", "v3", "ws", "wss", "chat", "video", "stream",
    "live", "status", "backup", "config", "root", "internal",
    "remote", "office", "owa", "exchange", "cpanel", "whm",
    "phpmyadmin", "pma", "server", "node", "cluster", "proxy", "cache",
    "origin", "www2", "www3", "web1", "web2", "app1", "app2",
];

impl SubdomainHarvester {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn harvest(&self, domain: &str, config: &OsintConfig) -> Vec<SubdomainResult> {
        let mut results = Vec::new();
        let mut seen = HashSet::new();

        let mut crt_results = self.query_crt_sh(domain).await;
        for r in &crt_results {
            seen.insert(r.subdomain.clone());
        }
        results.append(&mut crt_results);

        if config.enable_active {
            let mut brute_results = self.dns_brute(domain, &config.dns_wordlist).await;
            for r in &mut brute_results {
                if seen.insert(r.subdomain.clone()) {
                    results.push(r.clone());
                }
            }

            let known: Vec<String> = seen.iter().cloned().collect();
            let mut perm_results = self.permute_subdomains(domain, &known);
            for r in &mut perm_results {
                if seen.insert(r.subdomain.clone()) {
                    results.push(r.clone());
                }
            }
        }

        results
    }

    async fn query_crt_sh(&self, domain: &str) -> Vec<SubdomainResult> {
        let url = format!("https://crt.sh/?q=%25.{domain}&output=json");
        match self.client.get(&url)
            .timeout(Duration::from_secs(15))
            .send().await
        {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<Vec<serde_json::Value>>().await {
                    Ok(entries) => {
                        let mut subs: Vec<String> = entries.iter()
                            .filter_map(|e| e["name_value"].as_str())
                            .flat_map(|v| v.split('\n'))
                            .map(|s| s.trim().trim_start_matches("*.").to_string())
                            .filter(|s| s.ends_with(domain) && s.len() > domain.len())
                            .collect();
                        subs.sort();
                        subs.dedup();
                        subs.into_iter().map(|sub| {
                            let ips = resolve_subdomain_ips(&sub);
                            SubdomainResult {
                                subdomain: sub,
                                ip_addresses: ips,
                                source: SubdomainSource::CrtSh,
                            }
                        }).collect()
                    }
                    Err(_) => vec![],
                }
            }
            _ => vec![],
        }
    }

    async fn dns_brute(&self, domain: &str, wordlist: &[String]) -> Vec<SubdomainResult> {
        let _ = &self.client;
        let mut results = Vec::new();
        for word in wordlist {
            let sub = format!("{word}.{domain}");
            let ips = resolve_subdomain_ips(&sub);
            if !ips.is_empty() {
                results.push(SubdomainResult {
                    subdomain: sub,
                    ip_addresses: ips,
                    source: SubdomainSource::DnsBrute,
                });
            }
        }
        results
    }

    fn permute_subdomains(&self, domain: &str, known_subs: &[String]) -> Vec<SubdomainResult> {
        let extras = ["api", "admin", "dev", "test", "stage", "v2", "v3", "backup", "old", "new", "app", "web", "portal"];
        let separators = ["-", ".", ""];
        let mut perms = HashSet::new();
        for sub in known_subs {
            let base = sub.trim_end_matches(domain).trim_end_matches('.');
            if base.is_empty() { continue; }
            for sep in &separators {
                for extra in &extras {
                    perms.insert(format!("{base}{sep}{extra}.{domain}"));
                    perms.insert(format!("{extra}{sep}{base}.{domain}"));
                }
            }
        }
        perms.into_iter()
            .filter_map(|candidate| {
                let ips = resolve_subdomain_ips(&candidate);
                if !ips.is_empty() {
                    Some(SubdomainResult {
                        subdomain: candidate,
                        ip_addresses: ips,
                        source: SubdomainSource::Permutation,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

fn resolve_subdomain_ips(subdomain: &str) -> Vec<String> {
    let addr = format!("{subdomain}:0");
    let mut ips = Vec::new();
    if let Ok(addrs) = addr.to_socket_addrs() {
        let mut seen = HashSet::new();
        for sa in addrs {
            let ip = sa.ip().to_string();
            if seen.insert(ip.clone()) {
                ips.push(ip);
            }
        }
    }
    ips
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_prefixes_count() {
        assert!(COMMON_PREFIXES.len() > 80);
    }

    #[test]
    fn test_subdomain_result_serialization() {
        let r = SubdomainResult {
            subdomain: "api.example.com".into(),
            ip_addresses: vec!["1.2.3.4".into()],
            source: SubdomainSource::DnsBrute,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("api.example.com"));
        assert!(json.contains("dns_brute"));
    }

    #[test]
    fn test_subdomain_source_serde_roundtrip_matches_display() {
        for src in [
            SubdomainSource::DnsBrute,
            SubdomainSource::CrtSh,
            SubdomainSource::Permutation,
        ] {
            let json = serde_json::to_string(&src).unwrap();
            let name = json.trim_matches('"');
            assert_eq!(name, format!("{src}"), "serde 名应与 Display 一致");
            let back: SubdomainSource = serde_json::from_str(&json).unwrap();
            assert_eq!(back, src);
        }
    }

    #[test]
    fn test_subdomain_source_display() {
        assert_eq!(format!("{}", SubdomainSource::DnsBrute), "dns_brute");
        assert_eq!(format!("{}", SubdomainSource::CrtSh), "crt_sh");
        assert_eq!(format!("{}", SubdomainSource::Permutation), "permutation");
    }

    #[test]
    fn test_resolve_subdomain_ips_localhost() {
        let ips = resolve_subdomain_ips("localhost");
        assert!(!ips.is_empty());
    }

    #[test]
    fn test_permute_subdomains_empty_known() {
        let h = SubdomainHarvester::new(Client::new());
        let results = h.permute_subdomains("example.com", &[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_permute_subdomains_generates() {
        let h = SubdomainHarvester::new(Client::new());
        let known = vec!["api.example.com".to_string()];
        let results = h.permute_subdomains("example.com", &known);
        assert!(!results.is_empty());
        for r in &results {
            assert!(r.subdomain.ends_with("example.com"));
        }
    }

    #[test]
    fn test_subdomain_result_new() {
        let r = SubdomainResult {
            subdomain: "test.example.com".into(),
            ip_addresses: vec!["10.0.0.1".into()],
            source: SubdomainSource::CrtSh,
        };
        assert_eq!(r.subdomain, "test.example.com");
        assert_eq!(r.ip_addresses.len(), 1);
        assert_eq!(r.source, SubdomainSource::CrtSh);
    }

    #[test]
    #[ignore = "depends on live DNS resolver"]
    fn test_resolve_subdomain_ips_nonexistent() {
        let ips = resolve_subdomain_ips("this-does-not-exist-xyz.invalid");
        assert!(ips.is_empty());
    }

    #[tokio::test]
    async fn test_dns_brute_empty_wordlist_makes_no_lookups() {
        // 空词表 → 循环体不执行 → 无任何 DNS 查询, 结果必空 (不依赖 live resolver)
        let h = SubdomainHarvester::new(Client::new());
        let results = h.dns_brute("example.invalid", &[]).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_harvest_empty_config() {
        let h = SubdomainHarvester::new(Client::new());
        let config = OsintConfig {
            enable_active: false,
            ..Default::default()
        };
        let results = h.harvest("example.com", &config).await;
        assert!(results.iter().all(|r| r.source == SubdomainSource::CrtSh));
    }
}
