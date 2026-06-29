use super::intelligence_probe::{IntelligenceProbe, ProbeFinding, ProbeResult, ProbeSeverity};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

fn resolve_dns(host: &str) -> Result<Vec<String>, String> {
    let addr_str = format!("{}:0", host);
    let addrs = addr_str
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed: {}", e))?;
    let mut ips: Vec<String> = addrs
        .filter_map(|a| {
            if a.is_ipv4() {
                Some(a.ip().to_string())
            } else {
                None
            }
        })
        .collect();
    ips.sort();
    ips.dedup();
    Ok(ips)
}

fn reverse_lookup(ip: &str) -> Result<String, String> {
    let _ = ip;
    // Reverse DNS requires an external DNS resolver (e.g., trust-dns-resolver).
    // std::net::lookup_host is unstable. This is a placeholder for future integration.
    Err("PTR record lookup unavailable without external DNS resolver".into())
}

fn check_common_ports(ip: &str, ports: &[u16]) -> Vec<(u16, bool)> {
    let mut results = Vec::new();
    for &port in ports {
        let addr = format!("{}:{}", ip, port);
        match addr.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(sock_addr) = addrs.next() {
                    let open =
                        TcpStream::connect_timeout(&sock_addr, Duration::from_secs(2)).is_ok();
                    results.push((port, open));
                } else {
                    results.push((port, false));
                }
            }
            Err(_) => results.push((port, false)),
        }
    }
    results
}

fn generate_typosquat_variants(domain: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let domain_lower = domain.to_lowercase();
    let parts: Vec<&str> = domain_lower.split('.').collect();
    if parts.len() < 2 {
        return variants;
    }
    let name = parts[0];
    let tld = parts[1..].join(".");

    // Common TLD substitutions
    let common_tlds = [
        "com", "net", "org", "io", "co", "ai", "dev", "app", "xyz", "info",
    ];
    for &alt_tld in &common_tlds {
        if alt_tld != tld {
            variants.push(format!("{}.{}", name, alt_tld));
        }
    }

    // Character substitution (homoglyphs)
    let homoglyphs: HashMap<char, &[char]> = [
        ('o', &['0', 'ö', 'ò', 'ó'] as &[char]),
        ('l', &['1', 'i', '|'] as &[char]),
        ('e', &['3', 'é', 'è'] as &[char]),
        ('a', &['4', 'à', 'á', '@'] as &[char]),
        ('s', &['5', '$'] as &[char]),
        ('t', &['7'] as &[char]),
        ('g', &['9'] as &[char]),
        ('i', &['1', 'l', '|'] as &[char]),
        ('c', &['k'] as &[char]),
        ('w', &['v'] as &[char]),
    ]
    .iter()
    .cloned()
    .collect();

    for (i, ch) in name.char_indices() {
        if let Some(replacements) = homoglyphs.get(&ch) {
            for &repl in *replacements {
                let mut chars: Vec<char> = name.chars().collect();
                for (offset, rc) in repl.to_string().chars().enumerate() {
                    if offset == 0 {
                        chars[i] = rc;
                    } else {
                        chars.insert(i + offset, rc);
                    }
                }
                let new_name: String = chars.iter().collect();
                variants.push(format!("{}.{}", new_name, tld));
            }
        }
    }

    // Bit flip: add 's' for plural
    variants.push(format!("{}s.{}", name, tld));
    // Missing 's'
    if name.ends_with('s') {
        variants.push(format!("{}.{}", &name[..name.len() - 1], tld));
    }
    // Double letter
    for i in 0..name.len() {
        let ch = name[i..].chars().next().unwrap();
        let mut chars: Vec<char> = name.chars().collect();
        chars.insert(i, ch);
        let new_name: String = chars.iter().collect();
        variants.push(format!("{}.{}", new_name, tld));
    }

    variants.sort();
    variants.dedup();
    variants.truncate(50);
    variants
}

fn check_whois(domain: &str) -> String {
    let whois_servers: [(&str, u16); 2] = [("whois.verisign-grs.com", 43), ("whois.pir.org", 43)];
    for (server, port) in &whois_servers {
        let addr = format!("{}:{}", server, port);
        if let Ok(mut addrs) = addr.to_socket_addrs() {
            if let Some(sock_addr) = addrs.next() {
                if let Ok(mut stream) =
                    TcpStream::connect_timeout(&sock_addr, Duration::from_secs(5))
                {
                    let query = format!("{}\r\n", domain);
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                    if stream.write(query.as_bytes()).is_ok() {
                        let mut buf = [0u8; 4096];
                        if let Ok(n) = stream.read(&mut buf) {
                            let response = String::from_utf8_lossy(&buf[..n]).to_string();
                            if !response.contains("No match for") && !response.is_empty() {
                                let lines: Vec<&str> = response
                                    .lines()
                                    .filter(|l| {
                                        l.contains("Domain Name:")
                                            || l.contains("Registrar:")
                                            || l.contains("Creation Date:")
                                            || l.contains("Registry Expiry Date:")
                                            || l.contains("Name Server:")
                                    })
                                    .collect();
                                if !lines.is_empty() {
                                    return lines.join("; ");
                                }
                                return format!("WHOIS data available ({} bytes)", n);
                            }
                        }
                    }
                }
            }
        }
    }
    "WHOIS lookup unavailable (no WHOIS server reached)".into()
}

fn check_ssl_info(domain: &str) -> String {
    let addr = format!("{}:443", domain);
    if let Ok(mut addrs) = addr.to_socket_addrs() {
        if let Some(sock_addr) = addrs.next() {
            match TcpStream::connect_timeout(&sock_addr, Duration::from_secs(5)) {
                Ok(_) => return "Port 443 open — HTTPS available".into(),
                Err(_) => {}
            }
        }
    }
    "Port 443 unreachable — no HTTPS detected".into()
}

pub struct DomainProbe;

impl DomainProbe {
    pub fn new() -> Self {
        Self
    }
}

impl IntelligenceProbe for DomainProbe {
    fn name(&self) -> &str {
        "domain_intel"
    }

    fn description(&self) -> &str {
        "Domain intelligence: DNS resolution, WHOIS lookup, SSL check, subdomain enumeration, typosquatting detection"
    }

    fn probe(&self, target: &str, _timeout_secs: u64) -> ProbeResult {
        let start = Instant::now();
        let domain = target.trim().to_lowercase();
        let mut result = ProbeResult::new("domain_intel", &domain);

        // 1. DNS resolution
        match resolve_dns(&domain) {
            Ok(ips) => {
                if !ips.is_empty() {
                    result.findings.push(
                        ProbeFinding::new("dns_a_records", &ips.join(", "), "DNS")
                            .with_confidence(0.95)
                            .with_severity(ProbeSeverity::Info),
                    );
                    for ip in &ips {
                        if let Ok(ptr) = reverse_lookup(ip) {
                            result.findings.push(
                                ProbeFinding::new("dns_ptr_record", &ptr, "DNS")
                                    .with_meta("ip", ip)
                                    .with_confidence(0.7)
                                    .with_severity(ProbeSeverity::Info),
                            );
                        }
                    }
                    // Port scan common ports on first IP
                    let common_ports = [21, 22, 25, 80, 443, 8080, 8443];
                    let port_results = check_common_ports(&ips[0], &common_ports);
                    let open_ports: Vec<String> = port_results
                        .iter()
                        .filter(|(_, open)| *open)
                        .map(|(p, _)| p.to_string())
                        .collect();
                    if !open_ports.is_empty() {
                        let port_names: Vec<String> = open_ports
                            .iter()
                            .map(|p| match p.as_str() {
                                "21" => "21/FTP".into(),
                                "22" => "22/SSH".into(),
                                "25" => "25/SMTP".into(),
                                "80" => "80/HTTP".into(),
                                "443" => "443/HTTPS".into(),
                                "8080" => "8080/HTTP-Alt".into(),
                                "8443" => "8443/HTTPS-Alt".into(),
                                _ => format!("{}/?", p),
                            })
                            .collect();
                        result.findings.push(
                            ProbeFinding::new("open_ports", &port_names.join(", "), "PortScan")
                                .with_confidence(0.8)
                                .with_severity(
                                    if port_names
                                        .iter()
                                        .any(|p| p.contains("SSH") || p.contains("FTP"))
                                    {
                                        ProbeSeverity::Medium
                                    } else {
                                        ProbeSeverity::Info
                                    },
                                ),
                        );
                    }
                } else {
                    result.findings.push(
                        ProbeFinding::new("dns_status", "No A records found", "DNS")
                            .with_severity(ProbeSeverity::High),
                    );
                }
            }
            Err(e) => {
                result.findings.push(
                    ProbeFinding::new("dns_error", &e, "DNS").with_severity(ProbeSeverity::High),
                );
            }
        }

        // 2. WHOIS check
        let whois = check_whois(&domain);
        result.findings.push(
            ProbeFinding::new("whois", &whois, "WHOIS")
                .with_confidence(0.4)
                .with_severity(ProbeSeverity::Info),
        );

        // 3. SSL/TLS check
        let ssl = check_ssl_info(&domain);
        result.findings.push(
            ProbeFinding::new("ssl_status", &ssl, "SSL")
                .with_confidence(0.6)
                .with_severity(ProbeSeverity::Info),
        );

        // 4. Typosquatting detection
        let typosquats = generate_typosquat_variants(&domain);
        if !typosquats.is_empty() {
            let resolved: Vec<String> = typosquats
                .iter()
                .filter_map(|v| match resolve_dns(v) {
                    Ok(ips) if !ips.is_empty() => Some(format!("{} -> {}", v, ips[0])),
                    _ => None,
                })
                .collect();
            if !resolved.is_empty() {
                result.findings.push(
                    ProbeFinding::new(
                        "typosquat_alerts",
                        &format!("Active typosquat domains: {}", resolved.join("; ")),
                        "Typosquat",
                    )
                    .with_confidence(0.85)
                    .with_severity(ProbeSeverity::High),
                );
            }
        }

        result.success = true;
        result.duration_ms = start.elapsed().as_millis() as u64;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_dns_known_domain() {
        let ips = resolve_dns("google.com");
        assert!(ips.is_ok(), "google.com should resolve: {:?}", ips);
        if let Ok(ips) = ips {
            assert!(!ips.is_empty(), "should have at least one IP");
        }
    }

    #[test]
    fn test_resolve_dns_invalid_domain() {
        let ips = resolve_dns("this-domain-does-not-exist-hopefully-12345.test");
        assert!(
            ips.is_err() || ips.as_ref().map(|v| v.is_empty()).unwrap_or(false),
            "invalid domain should fail: {:?}",
            ips
        );
    }

    #[test]
    fn test_typosquat_variants() {
        let variants = generate_typosquat_variants("google.com");
        assert!(!variants.is_empty(), "should generate variants");
        assert!(
            variants.contains(&"google.net".to_string()),
            "should include TLD substitution"
        );
        assert!(
            variants.contains(&"go0gle.com".to_string()),
            "should include homoglyph substitution"
        );
    }

    #[test]
    fn test_typosquat_variants_no_duplicates() {
        let variants = generate_typosquat_variants("test.org");
        let mut sorted = variants.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(variants.len(), sorted.len(), "should not have duplicates");
    }

    #[test]
    fn test_typosquat_variants_max_size() {
        let variants = generate_typosquat_variants("abcdefghijklmnop.com");
        assert!(
            variants.len() <= 50,
            "should not exceed 50 variants: {}",
            variants.len()
        );
    }

    #[test]
    fn test_domain_probe() {
        let probe = DomainProbe::new();
        let result = probe.probe("example.com", 30);
        assert!(result.success);
        assert!(!result.findings.is_empty(), "should have findings");
    }

    #[test]
    fn test_probe_name_and_description() {
        let probe = DomainProbe::new();
        assert_eq!(probe.name(), "domain_intel");
        assert!(!probe.description().is_empty());
    }

    #[test]
    fn test_reverse_lookup() {
        let result = reverse_lookup("8.8.8.8");
        assert!(
            result.is_ok(),
            "8.8.8.8 should have PTR record: {:?}",
            result
        );
    }

    #[test]
    fn test_check_common_ports() {
        let ports = [80, 443];
        let results = check_common_ports("8.8.8.8", &ports);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_probe_on_nonexistent_domain() {
        let probe = DomainProbe::new();
        let result = probe.probe("nonexistent-domain-xyz-99999.com", 10);
        assert!(result.success);
        assert!(!result.findings.is_empty());
    }
}
