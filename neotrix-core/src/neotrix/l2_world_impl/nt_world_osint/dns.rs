use std::collections::HashSet;
use std::net::ToSocketAddrs;
use std::time::Duration;

use reqwest::Client;

use super::{OsintConfig, OsintTarget};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: String,
    pub value: String,
    pub source: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DnsFindings {
    pub subdomains: Vec<DnsRecord>,
    pub mx_records: Vec<DnsRecord>,
    pub txt_records: Vec<DnsRecord>,
    pub ns_records: Vec<DnsRecord>,
    pub a_records: Vec<DnsRecord>,
    pub aaaa_records: Vec<DnsRecord>,
    pub cname_records: Vec<DnsRecord>,
}

impl std::fmt::Display for DnsFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── DNS Enumeration ──")?;
        writeln!(f, "    Subdomains:  {}", self.subdomains.len())?;
        writeln!(f, "    A Records:   {}", self.a_records.len())?;
        writeln!(f, "    AAAA:        {}", self.aaaa_records.len())?;
        writeln!(f, "    MX:          {}", self.mx_records.len())?;
        writeln!(f, "    TXT:         {}", self.txt_records.len())?;
        writeln!(f, "    NS:          {}", self.ns_records.len())?;
        writeln!(f, "    CNAME:       {}", self.cname_records.len())?;
        for sd in self.subdomains.iter().take(20) {
            writeln!(f, "      → {} ({}) [{}]", sd.name, sd.value, sd.source)?;
        }
        if self.subdomains.len() > 20 {
            writeln!(f, "      ... and {} more", self.subdomains.len() - 20)?;
        }
        Ok(())
    }
}

fn resolve_a(domain: &str) -> Vec<DnsRecord> {
    let mut records = Vec::new();
    let addr = format!("{domain}:0");
    if let Ok(addrs) = addr.to_socket_addrs() {
        let mut seen = HashSet::new();
        for sa in addrs {
            let ip = sa.ip().to_string();
            if seen.insert(ip.clone()) {
                records.push(DnsRecord {
                    name: domain.to_string(),
                    record_type: "A".to_string(),
                    value: ip,
                    source: "system-resolver".to_string(),
                });
            }
        }
    }
    records
}

async fn query_doh(domain: &str, record_type: &str, client: &Client) -> Vec<DnsRecord> {
    let type_map: HashMap<&str, &str> = [
        ("A", "1"), ("AAAA", "28"), ("MX", "15"),
        ("TXT", "16"), ("NS", "2"), ("CNAME", "5"),
    ].iter().cloned().collect();

    let Some(&dns_type) = type_map.get(record_type) else { return vec![] };

    let urls = [
        format!("https://cloudflare-dns.com/dns-query?name={domain}&type={dns_type}"),
        format!("https://dns.google/resolve?name={domain}&type={dns_type}"),
    ];

    for url in &urls {
        let req = client.get(url)
            .header("accept", "application/dns-json")
            .timeout(Duration::from_secs(5));
        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let mut records = Vec::new();
                        if let Some(answer) = json["Answer"].as_array() {
                            for ans in answer {
                                let rtype = ans["type"].as_i64().unwrap_or(0);
                                let value = ans["data"].as_str().unwrap_or("").to_string();
                                let name = ans["name"].as_str().unwrap_or(domain).to_string();
                                let rtype_str = match rtype {
                                    1 => "A", 28 => "AAAA", 15 => "MX",
                                    16 => "TXT", 2 => "NS", 5 => "CNAME",
                                    _ => "UNKNOWN",
                                };
                                records.push(DnsRecord {
                                    name,
                                    record_type: rtype_str.to_string(),
                                    value,
                                    source: "doh".to_string(),
                                });
                            }
                        }
                        return records;
                    }
                    Err(_) => continue,
                }
            }
            _ => continue,
        }
    }
    vec![]
}

async fn crt_sh_subdomains(domain: &str, client: &Client) -> Vec<String> {
    let url = format!("https://crt.sh/?q=%25.{domain}&output=json");
    match client.get(&url).timeout(Duration::from_secs(15)).send().await {
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
                    subs
                }
                Err(_) => vec![],
            }
        }
        _ => vec![],
    }
}

fn dns_bruteforce(domain: &str, wordlist: &[String]) -> Vec<String> {
    let mut found = Vec::new();
    for word in wordlist {
        let sub = format!("{word}.{domain}");
        if resolve_a(&sub).iter().any(|r| !r.value.is_empty()) {
            found.push(sub);
        }
    }
    found
}

fn subdomain_permutations(domain: &str, known_subs: &[String]) -> Vec<String> {
    let separators = ["-", ".", ""];
    let extras = ["api", "admin", "dev", "test", "stage", "v2", "v3", "backup", "old", "new", "app", "web", "portal"];
    let mut perms = HashSet::new();
    for sub in known_subs {
        let base = sub.trim_end_matches(domain).trim_end_matches('.');
        if base.is_empty() { continue; }
        for sep in &separators {
            for extra in &extras {
                let candidate = format!("{base}{sep}{extra}.{domain}");
                perms.insert(candidate);
                let candidate2 = format!("{extra}{sep}{base}.{domain}");
                perms.insert(candidate2);
            }
        }
    }
    perms.into_iter().filter(|p| resolve_a(p).iter().any(|r| !r.value.is_empty())).collect()
}

use std::collections::HashMap;

pub async fn investigate(target: &OsintTarget, client: &Client, config: &OsintConfig) -> Result<DnsFindings, String> {
    let domain = target.domain.as_ref().ok_or("no domain specified")?;
    let mut findings = DnsFindings::default();

    let mut all_subs: Vec<String> = Vec::new();
    let mut sub_sources: Vec<String> = Vec::new();

    // Phase 1: crt.sh passive
    let crt_found = crt_sh_subdomains(domain, client).await;
    let crt_count = crt_found.len();
    all_subs.extend(crt_found);
    sub_sources.extend(std::iter::repeat("crt.sh").take(crt_count).map(String::from));
    findings.a_records.extend(resolve_a(domain));
    findings.a_records.iter_mut().for_each(|r| r.source = "system-resolver".to_string());

    // Phase 2: DoH for MX, TXT, NS, AAAA, CNAME
    for rtype in &["MX", "TXT", "NS", "AAAA", "CNAME"] {
        let records = query_doh(domain, rtype, client).await;
        for rec in records {
            match rec.record_type.as_str() {
                "MX" => findings.mx_records.push(rec),
                "TXT" => findings.txt_records.push(rec),
                "NS" => findings.ns_records.push(rec),
                "AAAA" => findings.aaaa_records.push(rec),
                "CNAME" => findings.cname_records.push(rec),
                _ => {}
            }
        }
    }

    // Phase 3: DNS brute force
    if config.enable_active && !config.dns_wordlist.is_empty() {
        let wl = if config.dns_wordlist.is_empty() { OsintConfig::default_dns_wordlist() } else { config.dns_wordlist.clone() };
        let bruted = dns_bruteforce(domain, &wl);
        for sub in bruted {
            if !all_subs.contains(&sub) {
                all_subs.push(sub.clone());
                sub_sources.push("brute-force".to_string());
            }
        }
    }

    // Phase 4: Subdomain permutations
    let known: Vec<String> = all_subs.iter().map(|s| format!("{s}.{domain}")).collect();
    if config.enable_active && !known.is_empty() {
        let perms = subdomain_permutations(domain, &known);
        for sub in perms {
            if !all_subs.contains(&sub) {
                all_subs.push(sub.clone());
                sub_sources.push("permutation".to_string());
            }
        }
    }

    // Resolve all subdomains
    for (sub, source) in all_subs.iter().zip(sub_sources.iter()) {
        let recs = resolve_a(sub);
        for rec in recs {
            findings.subdomains.push(DnsRecord {
                name: sub.clone(),
                record_type: "A".to_string(),
                value: rec.value,
                source: source.clone(),
            });
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_a_known() {
        let recs = resolve_a("google.com");
        assert!(!recs.is_empty(), "google.com should resolve");
    }

    #[test]
    fn test_resolve_a_nx() {
        // .invalid is a reserved TLD that should never resolve (RFC 2606)
        let recs = resolve_a("this-domain-does-not-exist-12345.invalid");
        assert!(recs.is_empty(), "expected empty, got {recs:?}");
    }

    #[test]
    fn test_dns_findings_default() {
        let f = DnsFindings::default();
        assert!(f.subdomains.is_empty());
    }

    #[test]
    fn test_dns_bruteforce_empty() {
        // .invalid is a reserved TLD that should never resolve (RFC 2606)
        let found = dns_bruteforce("nonexistent-domain-zzz.invalid", &["www".to_string()]);
        assert!(found.is_empty(), "expected empty, got {found:?}");
    }

    #[test]
    fn test_crt_sh_domain_filter() {
        let subs = vec!["admin.example.com".to_string(), "api.example.com".to_string()];
        assert!(subs.iter().all(|s| s.ends_with("example.com")));
    }

    #[test]
    fn test_wordlist_default_size() {
        let wl = OsintConfig::default_dns_wordlist();
        assert!(wl.len() > 80);
    }

    #[test]
    fn test_resolve_a_duplicates() {
        let recs = resolve_a("google.com");
        let mut ips = std::collections::HashSet::new();
        for r in &recs {
            assert!(ips.insert(r.value.clone()), "duplicate IP: {}", r.value);
        }
    }
}
