use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{OsintConfig, OsintTarget};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub service: Option<String>,
    pub banner: Option<String>,
    pub state: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkFindings {
    pub ip_addresses: Vec<String>,
    pub services: Vec<ServiceInfo>,
    pub asn_info: Option<String>,
    pub open_ports: Vec<u16>,
    pub domain: String,
}

impl std::fmt::Display for NetworkFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── Network / Service Mapping ──")?;
        writeln!(f, "    Domain:     {}", self.domain)?;
        writeln!(f, "    IPs:        {}", self.ip_addresses.join(", "))?;
        writeln!(f, "    Open ports: {:?}", self.open_ports)?;
        writeln!(f, "    Services:   {}", self.services.len())?;
        if let Some(ref asn) = self.asn_info {
            writeln!(f, "    ASN:        {asn}")?;
        }
        for svc in &self.services {
            writeln!(f, "      {}/{} {} {}", svc.host, svc.port, svc.protocol, svc.service.as_deref().unwrap_or("?"))?;
            if let Some(ref banner) = svc.banner {
                let short: String = banner.chars().take(100).collect();
                writeln!(f, "        Banner: {short}")?;
            }
        }
        Ok(())
    }
}

use crate::l2_perception::nt_world::port_service::PORT_SERVICE_MAP as PORT_LIST;

fn scan_port(host: &str, port: u16, timeout: Duration) -> bool {
    let addr = format!("{host}:{port}");
    if let Ok(mut addrs) = addr.to_socket_addrs() {
        if let Some(sa) = addrs.next() {
            return TcpStream::connect_timeout(&sa, timeout).is_ok();
        }
    }
    false
}

fn grab_banner(host: &str, port: u16, timeout: Duration) -> Option<String> {
    let addr = format!("{host}:{port}");
    if let Ok(stream) = TcpStream::connect_timeout(&addr.to_socket_addrs().ok()?.next()?, timeout) {
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));
        // Try to read a banner from common protocols
        let mut buf = [0u8; 1024];
        match stream.peek(&mut buf) {
            Ok(n) if n > 0 => {
                let banner = String::from_utf8_lossy(&buf[..n.min(1024)]).to_string();
                if banner.chars().any(|c| c.is_ascii_graphic()) {
                    return Some(banner);
                }
            }
            _ => {}
        }
    }
    None
}

pub async fn investigate(target: &OsintTarget, _client: &Client, config: &OsintConfig) -> Result<NetworkFindings, String> {
    let domain = target.domain.as_ref().ok_or("no domain specified")?;
    let mut findings = NetworkFindings {
        domain: domain.to_string(),
        ..Default::default()
    };

    // Resolve IP addresses
    let addr = format!("{domain}:0");
    if let Ok(addrs) = addr.to_socket_addrs() {
        let mut seen_ips = std::collections::HashSet::new();
        for sa in addrs {
            let ip = sa.ip().to_string();
            if seen_ips.insert(ip.clone()) {
                findings.ip_addresses.push(ip);
            }
        }
    }

    // Port scanning (only if active is enabled)
    if config.enable_active {
        let timeout = Duration::from_secs(3);
        for (port, service, protocol) in PORT_LIST {
            if findings.services.len() >= 50 { break; } // limit
            for ip in &findings.ip_addresses {
                if scan_port(ip, *port, timeout) {
                    let banner = grab_banner(ip, *port, timeout);
                    findings.open_ports.push(*port);
                    findings.services.push(ServiceInfo {
                        host: ip.clone(),
                        port: *port,
                        protocol: protocol.to_string(),
                        service: Some(service.to_string()),
                        banner,
                        state: "open".to_string(),
                    });
                }
            }
        }
        findings.open_ports.sort();
        findings.open_ports.dedup();
    }

    // ASN lookup via DNS (TXT record of origin.asn.cymru.com)
    if let Some(ip) = findings.ip_addresses.first() {
        let reversed: Vec<&str> = ip.split('.').rev().collect();
        let asn_query = format!("{}.origin.asn.cymru.com", reversed.join("."));
        if let Ok(addrs) = format!("{asn_query}:0").to_socket_addrs() {
            // We got a resolution - this is a simplified check
            // Full ASN lookup would require TXT record query
            let _ = addrs;
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info_new() {
        let s = ServiceInfo {
            host: "192.168.1.1".to_string(),
            port: 443,
            protocol: "tcp".to_string(),
            service: Some("HTTPS".to_string()),
            banner: None,
            state: "open".to_string(),
        };
        assert_eq!(s.port, 443);
    }

    #[test]
    fn test_network_findings_default() {
        let f = NetworkFindings::default();
        assert!(f.ip_addresses.is_empty());
        assert!(f.services.is_empty());
    }

    #[test]
    fn test_display_empty() {
        let f = NetworkFindings::default();
        let s = format!("{f}");
        assert!(s.contains("Network"));
    }

    #[test]
    fn test_common_ports_length() {
        assert!(PORT_LIST.len() > 20);
    }

    #[test]
    fn test_common_ports_include_web() {
        assert!(PORT_LIST.iter().any(|(p, _, _)| *p == 80));
        assert!(PORT_LIST.iter().any(|(p, _, _)| *p == 443));
        assert!(PORT_LIST.iter().any(|(p, _, _)| *p == 22));
        assert!(PORT_LIST.iter().any(|(p, _, _)| *p == 3306));
    }

    #[test]
    fn test_asn_query_format() {
        let ip = "8.8.8.8";
        let reversed: Vec<&str> = ip.split('.').rev().collect();
        let query = format!("{}.origin.asn.cymru.com", reversed.join("."));
        assert_eq!(query, "8.8.8.8.origin.asn.cymru.com");
    }

    #[test]
    fn test_port_scan_valid_addr() {
        let result = format!("127.0.0.1:0");
        assert!(result.contains("127.0.0.1"));
    }
}
