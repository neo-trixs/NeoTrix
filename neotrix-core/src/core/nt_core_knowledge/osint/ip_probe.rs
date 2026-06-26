use super::intelligence_probe::{IntelligenceProbe, ProbeFinding, ProbeResult, ProbeSeverity};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs, UdpSocket};
use std::str::FromStr;
use std::time::{Duration, Instant};

fn classify_v4(v4: Ipv4Addr) -> Vec<&'static str> {
    let mut tags = Vec::new();
    let octets = v4.octets();

    if v4.is_loopback() {
        tags.push("loopback");
    }
    // RFC1918 private: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
    if octets[0] == 10
        || (octets[0] == 172 && (16..=31).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 168)
    {
        tags.push("private/RFC1918");
    }
    // Link-local: 169.254.0.0/16
    if octets[0] == 169 && octets[1] == 254 {
        tags.push("link-local");
    }
    if v4.is_multicast() {
        tags.push("multicast");
    }
    if v4.is_unspecified() {
        tags.push("unspecified");
    }
    // Documentation: 192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24
    if (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
        || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
        || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
    {
        tags.push("documentation/RFC5737");
    }
    if tags.is_empty() {
        tags.push("public");
    }
    tags.push("IPv4");
    tags
}

fn classify_v6(v6: &std::net::Ipv6Addr) -> Vec<&'static str> {
    let mut tags = Vec::new();
    if v6.is_loopback() {
        tags.push("loopback");
    }
    if v6.is_unspecified() {
        tags.push("unspecified");
    }
    if v6.is_multicast() {
        tags.push("multicast");
    }
    if tags.is_empty() {
        tags.push("public");
    }
    tags.push("IPv6");
    tags
}

fn classify_ip(ip: &str) -> String {
    let ip_addr: IpAddr = match FromStr::from_str(ip) {
        Ok(a) => a,
        Err(_) => return "Invalid IP address".into(),
    };
    match ip_addr {
        IpAddr::V4(v4) => classify_v4(v4).join(", "),
        IpAddr::V6(ref v6) => classify_v6(v6).join(", "),
    }
}

fn ip_get_asn(ip: &str) -> Result<String, String> {
    let whois_server = "whois.cymru.com";
    let query = format!(" -r {}\r\n", ip);
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Cannot bind UDP: {}", e))?;
    socket.set_read_timeout(Some(Duration::from_secs(5))).ok();
    socket.set_write_timeout(Some(Duration::from_secs(5))).ok();
    socket
        .connect(format!("{}:43", whois_server))
        .map_err(|e| format!("Cannot connect to WHOIS server: {}", e))?;
    socket
        .send(query.as_bytes())
        .map_err(|e| format!("Send failed: {}", e))?;
    let mut buf = [0u8; 4096];
    let n = socket
        .recv(&mut buf)
        .map_err(|e| format!("Recv failed: {}", e))?;
    let response = String::from_utf8_lossy(&buf[..n]).to_string();
    if response.contains('|') {
        let parts: Vec<&str> = response.split('|').collect();
        if parts.len() >= 4 {
            return Ok(format!(
                "AS{} | {} | {}",
                parts[0].trim(),
                parts[1].trim(),
                parts[2].trim()
            ));
        }
    }
    Ok(response.trim().into())
}

fn estimate_geolocation(ip: &str) -> String {
    let classification = classify_ip(ip);
    if classification.contains("private") {
        return "Private/internal network — geolocation not applicable".into();
    }
    if classification.contains("documentation") {
        return "Documentation/test range — not geographically assigned".into();
    }
    format!(
        "IP classification: {}. Full geolocation requires GeoIP database or API.",
        classification
    )
}

pub struct IPProbe {
    asn_cache: HashMap<String, String>,
}

impl IPProbe {
    pub fn new() -> Self {
        Self {
            asn_cache: HashMap::new(),
        }
    }
}

impl IntelligenceProbe for IPProbe {
    fn name(&self) -> &str {
        "ip_intel"
    }

    fn description(&self) -> &str {
        "IP intelligence: classification, ASN lookup, geolocation estimation, threat context"
    }

    fn probe(&self, target: &str, _timeout: u64) -> ProbeResult {
        let start = Instant::now();
        let ip = target.trim().to_string();
        let mut result = ProbeResult::new("ip_intel", &ip);

        let classification = classify_ip(&ip);
        let severity = if classification.contains("public") {
            ProbeSeverity::Info
        } else {
            ProbeSeverity::Low
        };
        result.findings.push(
            ProbeFinding::new("ip_classification", &classification, "IP")
                .with_confidence(1.0)
                .with_severity(severity),
        );

        if classification.contains("public") {
            let asn_info = match ip_get_asn(&ip) {
                Ok(info) => {
                    let _ = &info;
                    info
                }
                Err(e) => format!("ASN lookup failed: {}", e),
            };
            result.findings.push(
                ProbeFinding::new("asn_info", &asn_info, "ASN")
                    .with_confidence(0.6)
                    .with_severity(ProbeSeverity::Info),
            );
        }

        let geo = estimate_geolocation(&ip);
        result.findings.push(
            ProbeFinding::new("geolocation", &geo, "GeoIP")
                .with_confidence(0.3)
                .with_severity(ProbeSeverity::Info),
        );

        result.success = true;
        result.duration_ms = start.elapsed().as_millis() as u64;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_loopback() {
        let tags = classify_ip("127.0.0.1");
        assert!(tags.contains("loopback"));
    }

    #[test]
    fn test_classify_private_10() {
        let tags = classify_ip("10.0.0.1");
        assert!(tags.contains("private"));
    }

    #[test]
    fn test_classify_private_192() {
        let tags = classify_ip("192.168.1.1");
        assert!(tags.contains("private"));
    }

    #[test]
    fn test_classify_public() {
        let tags = classify_ip("8.8.8.8");
        assert!(tags.contains("public"));
        assert!(tags.contains("IPv4"));
    }

    #[test]
    fn test_classify_invalid() {
        let tags = classify_ip("not-an-ip");
        assert_eq!(tags, "Invalid IP address");
    }

    #[test]
    fn test_classify_multicast() {
        let tags = classify_ip("224.0.0.1");
        assert!(tags.contains("multicast"));
    }

    #[test]
    fn test_classify_link_local() {
        let tags = classify_ip("169.254.1.1");
        assert!(tags.contains("link-local"));
    }

    #[test]
    fn test_classify_documentation() {
        let tags = classify_ip("192.0.2.1");
        assert!(tags.contains("documentation"));
    }

    #[test]
    fn test_classify_v6_loopback() {
        let tags = classify_ip("::1");
        assert!(tags.contains("loopback"));
        assert!(tags.contains("IPv6"));
    }

    #[test]
    fn test_classify_v6_public() {
        let tags = classify_ip("2606:4700:4700::1111");
        assert!(tags.contains("public"));
        assert!(tags.contains("IPv6"));
    }

    #[test]
    fn test_ip_probe() {
        let probe = IPProbe::new();
        let result = probe.probe("8.8.8.8", 30);
        assert!(result.success);
        assert!(!result.findings.is_empty());
    }

    #[test]
    fn test_ip_probe_private() {
        let probe = IPProbe::new();
        let result = probe.probe("192.168.1.1", 10);
        assert!(result.success);
        let has_private = result.findings.iter().any(|f| f.value.contains("private"));
        assert!(has_private);
    }

    #[test]
    fn test_probe_name() {
        let probe = IPProbe::new();
        assert_eq!(probe.name(), "ip_intel");
    }

    #[test]
    fn test_v4_broadcast_v6() {
        let v4 = classify_ip("255.255.255.255");
        assert!(v4.contains("public") || !v4.contains("private"));
    }
}
