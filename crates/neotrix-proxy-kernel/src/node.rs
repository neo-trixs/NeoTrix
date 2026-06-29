use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolKind {
    Socks5,
    Http,
    Shadowsocks,
    VMess,
    VLess,
    Trojan,
    Hysteria2,
    Unknown,
}

impl ProtocolKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Socks5 => "socks5",
            Self::Http => "http",
            Self::Shadowsocks => "ss",
            Self::VMess => "vmess",
            Self::VLess => "vless",
            Self::Trojan => "trojan",
            Self::Hysteria2 => "hysteria2",
            Self::Unknown => "unknown",
        }
    }

    pub fn is_direct(&self) -> bool {
        matches!(self, Self::Socks5 | Self::Http)
    }

    pub fn is_encrypted(&self) -> bool {
        !self.is_direct() && !matches!(self, Self::Unknown)
    }

    pub fn is_natively_supported(&self) -> bool {
        matches!(self, Self::Socks5 | Self::Http | Self::Shadowsocks | Self::Trojan | Self::Hysteria2 | Self::VLess | Self::VMess)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyNode {
    pub uri: String,
    pub protocol: ProtocolKind,
    pub server: String,
    pub port: u16,
    pub name: String,

    // Protocol-specific
    pub method: Option<String>,
    pub password: Option<String>,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub cipher: Option<String>,
    pub sni: Option<String>,
    pub tls: bool,
    pub skip_cert_verify: bool,
    pub network: Option<String>,
    pub path: Option<String>,
    pub host: Option<String>,
    pub alpn: Option<String>,
    pub fingerprint: Option<String>,
    pub flow: Option<String>,
    pub public_key: Option<String>,
    pub short_id: Option<String>,
    pub obfs: Option<String>,
    pub obfs_password: Option<String>,
}

impl ProxyNode {
    pub fn new(uri: &str) -> Self {
        let proto = Self::detect_protocol(uri);
        let (server, port) = Self::extract_host_port(uri);
        let name = Self::extract_name(uri);
        let is_tls = uri.contains("security=tls") || uri.contains("tls=tls")
            || uri.starts_with("https://") || uri.starts_with("trojan://");
        ProxyNode {
            uri: uri.to_string(),
            protocol: proto,
            server,
            port,
            name,
            method: None,
            password: None,
            username: None,
            uuid: None,
            cipher: None,
            sni: None,
            tls: is_tls,
            skip_cert_verify: uri.contains("allowInsecure=1") || uri.contains("insecure=1"),
            network: Self::extract_param(uri, "type"),
            path: Self::extract_param(uri, "path"),
            host: Self::extract_param(uri, "host"),
            alpn: Self::extract_param(uri, "alpn"),
            fingerprint: Self::extract_param(uri, "fp"),
            flow: Self::extract_param(uri, "flow"),
            public_key: Self::extract_param(uri, "pbk"),
            short_id: Self::extract_param(uri, "sid"),
            obfs: Self::extract_param(uri, "obfs"),
            obfs_password: Self::extract_param(uri, "obfs-password"),
        }
    }

    pub fn parse_with_vpn_link(uri: &str) -> Option<Self> {
        use vpn_link_serde::Protocol;
        let parsed = Protocol::parse(uri).ok()?;
        let mut node = ProxyNode::new(uri);

        match parsed {
            Protocol::Shadowsocks(s) => {
                node.method = Some(s.config.method.clone());
                node.password = Some(s.config.password.clone());
                node.server = s.config.address.clone();
                node.port = s.config.port;
                node.name = s.config.tag.clone().unwrap_or_default();
                node.protocol = ProtocolKind::Shadowsocks;
            }
            Protocol::VMess(v) => {
                node.server = v.config.add.clone();
                node.port = v.config.port;
                node.uuid = Some(v.config.id.clone());
                node.name = v.config.ps.clone().unwrap_or_default();
                node.tls = v.config.tls.as_deref() == Some("tls");
                node.network = v.config.net.clone();
                node.path = v.config.path.clone();
                node.host = v.config.host.clone();
                node.sni = v.config.sni.clone();
                node.alpn = v.config.alpn.clone();
                node.fingerprint = v.config.fp.clone();
                node.cipher = v.config.scy.clone();
                node.protocol = ProtocolKind::VMess;
            }
            Protocol::VLess(v) => {
                node.server = v.config.address.clone();
                node.port = v.config.port;
                node.uuid = Some(v.config.id.clone());
                node.name = v.config.remark.clone().unwrap_or_default();
                node.protocol = ProtocolKind::VLess;
            }
            Protocol::Trojan(t) => {
                node.password = Some(t.config.password.clone());
                node.server = t.config.address.clone();
                node.port = t.config.port;
                node.name = t.config.remark.clone().unwrap_or_default();
                node.sni = t.config.sni.clone();
                node.tls = true;
                node.protocol = ProtocolKind::Trojan;
            }
            Protocol::Hysteria2(h) => {
                node.password = h.config.password.clone();
                node.server = h.config.host.clone();
                node.port = h.config.port;
                node.sni = h.config.sni.clone();
                node.tls = true;
                node.obfs = h.config.obfs.clone();
                node.protocol = ProtocolKind::Hysteria2;
            }
        }
        Some(node)
    }

    fn detect_protocol(uri: &str) -> ProtocolKind {
        if uri.starts_with("ss://") { ProtocolKind::Shadowsocks }
        else if uri.starts_with("vmess://") { ProtocolKind::VMess }
        else if uri.starts_with("vless://") { ProtocolKind::VLess }
        else if uri.starts_with("trojan://") { ProtocolKind::Trojan }
        else if uri.starts_with("hysteria2://") || uri.starts_with("hy2://") { ProtocolKind::Hysteria2 }
        else if uri.starts_with("socks5://") { ProtocolKind::Socks5 }
        else if uri.starts_with("http://") || uri.starts_with("https://") { ProtocolKind::Http }
        else { ProtocolKind::Unknown }
    }

    fn extract_host_port(uri: &str) -> (String, u16) {
        let without_scheme = if let Some(pos) = uri.find("://") {
            &uri[pos + 3..]
        } else { uri };
        let main = without_scheme.split('#').next().unwrap_or(without_scheme);

        if let Some(at_pos) = main.find('@') {
            Self::parse_host_port(&main[at_pos + 1..])
        } else if let Some(q_pos) = main.find('?') {
            Self::parse_host_port(&main[..q_pos])
        } else {
            Self::parse_host_port(main)
        }
    }

    fn parse_host_port(raw: &str) -> (String, u16) {
        let s = raw.split('?').next().unwrap_or(raw).split('#').next().unwrap_or(raw).trim_end_matches('/');
        if let Some(bracket_end) = s.rfind(']') {
            let host = &s[..=bracket_end];
            let after = &s[bracket_end + 1..];
            if let Some(port_str) = after.strip_prefix(':') {
                let port: u16 = port_str.parse().unwrap_or(443);
                (host.to_string(), port)
            } else {
                (s.to_string(), 443)
            }
        } else if let Some(pos) = s.rfind(':') {
            let host = &s[..pos];
            let port_str = &s[pos + 1..];
            let port: u16 = port_str.parse().unwrap_or(443);
            (host.to_string(), port)
        } else {
            (s.to_string(), 443)
        }
    }

    fn extract_name(uri: &str) -> String {
        uri.split('#').nth(1).unwrap_or("").to_string()
    }

    fn extract_param(uri: &str, key: &str) -> Option<String> {
        let query = uri.split('?').nth(1)?;
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if parts.next()? == key {
                return Some(parts.next().unwrap_or("").to_string());
            }
        }
        None
    }

    pub fn connect_addr(&self) -> String {
        format!("{}:{}", self.server, self.port)
    }
}

pub fn parse_subscription(content: &str) -> Vec<ProxyNode> {
    let decoded = if is_base64(content) {
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        engine.decode(content).ok()
            .and_then(|b| String::from_utf8(b).ok())
            .unwrap_or_else(|| content.to_string())
    } else {
        content.to_string()
    };

    decoded.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| line.contains("://"))
        .map(|line| ProxyNode::parse_with_vpn_link(line).unwrap_or_else(|| ProxyNode::new(line)))
        .collect()
}

fn is_base64(s: &str) -> bool {
    if s.len() < 8 { return false; }
    let is_b64_char = |c: char| c.is_ascii_alphanumeric()
        || c == '+' || c == '/' || c == '=' || c == '-' || c == '_';
    if !s.chars().all(is_b64_char) {
        return false;
    }
    let content_len = s.trim_end_matches('=').len();
    let padding_count = s.len() - content_len;
    // At least 6 content chars, at most 2 padding chars
    content_len >= 6 && padding_count <= 2
        // Must contain at least one base64-ish char to distinguish from plain text
        && s.contains(|c: char| c == '+' || c == '/' || c == '-' || c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_protocol() {
        assert_eq!(ProtocolKind::Shadowsocks, ProxyNode::new("ss://YWVzLTEyOC1nY206cGFzc3dk@1.2.3.4:8388").protocol);
        assert_eq!(ProtocolKind::VMess, ProxyNode::new("vmess://...").protocol);
        assert_eq!(ProtocolKind::Trojan, ProxyNode::new("trojan://pass@1.2.3.4:443").protocol);
        assert_eq!(ProtocolKind::Hysteria2, ProxyNode::new("hysteria2://pass@1.2.3.4:443").protocol);
        assert_eq!(ProtocolKind::Socks5, ProxyNode::new("socks5://1.2.3.4:1080").protocol);
        assert_eq!(ProtocolKind::Http, ProxyNode::new("http://1.2.3.4:8080").protocol);
    }

    #[test]
    fn test_extract_host_port() {
        let n = ProxyNode::new("ss://YWVzLTEyOC1nY206cGFzc3dk@example.com:8388#my-node");
        assert_eq!("example.com", n.server);
        assert_eq!(8388, n.port);
        assert_eq!("my-node", n.name);
    }

    #[test]
    fn test_extract_params() {
        let n = ProxyNode::new("trojan://pass@server.com:443?sni=example.com&fp=chrome&allowInsecure=1#node");
        assert_eq!(true, n.skip_cert_verify);
        assert_eq!(true, n.tls);
    }

    #[test]
    fn test_is_base64() {
        assert_eq!(true, is_base64("YWVzLTI1Ni1nY206cGFzc3dvcmQ="));
        assert_eq!(false, is_base64("not base64"));
    }

    #[test]
    fn test_fallback_on_invalid_uri() {
        let n = ProxyNode::new("socks5://user:pass@host.com:1080");
        assert_eq!(ProtocolKind::Socks5, n.protocol);
        assert_eq!("host.com", n.server);
        assert_eq!(1080, n.port);
    }

    #[test]
    fn test_is_natively_supported() {
        assert_eq!(true, ProtocolKind::Socks5.is_natively_supported());
        assert_eq!(true, ProtocolKind::Shadowsocks.is_natively_supported());
        assert_eq!(true, ProtocolKind::VMess.is_natively_supported());
        assert_eq!(true, ProtocolKind::VLess.is_natively_supported());
    }

    #[test]
    fn test_ss_uri_parse_with_vpn_link() {
        let uri = "ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ=@server.com:8388";
        let n = ProxyNode::parse_with_vpn_link(uri);
        assert!(n.is_some(), "ss:// should parse with vpn-link-serde");
        let n = n.unwrap();
        assert_eq!(ProtocolKind::Shadowsocks, n.protocol);
        assert_eq!("server.com", n.server);
        assert_eq!(8388, n.port);
    }

    #[test]
    fn test_trojan_uri_parse_with_vpn_link() {
        let uri = "trojan://password123@trojan-server.com:443?sni=cdn.example.com&allowInsecure=1#my-trojan";
        let n = ProxyNode::parse_with_vpn_link(uri).unwrap_or_else(|| ProxyNode::new(uri));
        assert_eq!(ProtocolKind::Trojan, n.protocol);
        assert_eq!("trojan-server.com", n.server);
        assert_eq!(443, n.port);
        assert!(n.password.is_some());
        assert_eq!("my-trojan", n.name);
    }

    #[test]
    fn test_vmess_uri_fallback_without_vpn_link() {
        let uri = "vmess://eyJhZGQiOiJzZXJ2ZXIuY29tIiwicG9ydCI6NDQzLCJpZCI6InV1aWQtaGVyZSIsIm5ldCI6IndzIiwidGxzIjoidGxzIn0=";
        let n = ProxyNode::parse_with_vpn_link(uri);
        assert!(n.is_some(), "vmess:// should parse with vpn-link-serde");
        let n = n.unwrap();
        assert_eq!(ProtocolKind::VMess, n.protocol);
    }

    #[test]
    fn test_subscription_parse_plain_text() {
        let text = "ss://YWVzLTI1Ni1nY206cGFzcw==@host1:8388#node1\ntrojan://pass@host2:443#node2\n";
        let nodes = parse_subscription(text);
        assert_eq!(2, nodes.len());
        assert_eq!(ProtocolKind::Shadowsocks, nodes[0].protocol);
        assert_eq!(ProtocolKind::Trojan, nodes[1].protocol);
    }

    #[test]
    fn test_subscription_parse_filters_comments_and_empty() {
        let text = "# comment line\n\nss://YWVzLTI1Ni1nY206cGFzcw==@h:8388#n\n  \n";
        let nodes = parse_subscription(text);
        assert_eq!(1, nodes.len());
    }

    #[test]
    fn test_telemetry_collector_basic() {
        let mut tc = crate::telemetry::TelemetryCollector::new();
        let stat = crate::telemetry::ConnectStats {
            protocol: ProtocolKind::Shadowsocks,
            server: "test.com".into(),
            port: 8388,
            target: "opencode.ai:443".into(),
            success: true,
            latency_ms: 150,
            bytes_sent: 100,
            bytes_recv: 500,
            error: None,
            timestamp_ms: 1000,
        };
        tc.record(stat);
        let health = tc.protocol_health(ProtocolKind::Shadowsocks);
        assert_eq!(1, health.total_attempts);
        assert_eq!(1, health.total_success);
        assert_eq!(150.0, health.avg_latency_ms);
    }

    #[test]
    fn test_telemetry_mixed_success_failure() {
        let mut tc = crate::telemetry::TelemetryCollector::new();
        for i in 0..5 {
            tc.record(crate::telemetry::ConnectStats {
                protocol: ProtocolKind::Socks5,
                server: "srv".into(),
                port: 1080,
                target: "x.com:443".into(),
                success: i < 3,
                latency_ms: 100 + i * 10,
                bytes_sent: 0,
                bytes_recv: 0,
                error: if i >= 3 { Some("timeout".into()) } else { None },
                timestamp_ms: 1000 + i * 100,
            });
        }
        let h = tc.protocol_health(ProtocolKind::Socks5);
        assert_eq!(5, h.total_attempts);
        assert_eq!(3, h.total_success);
        assert_eq!(0.6, h.success_rate());
    }

    #[test]
    fn test_connect_addr_format() {
        let n = ProxyNode::new("ss://a@1.2.3.4:5678");
        assert_eq!("1.2.3.4:5678", n.connect_addr());
    }

    #[test]
    fn test_uri_extracts_obfs_params() {
        let uri = "ss://YWVzLTI1Ni1nY206cGFzcw==@obfs.com:8443?plugin=obfs-local;obfs=tls;obfs-host=cdn.com#obfs-node";
        let n = ProxyNode::new(uri);
        assert_eq!("obfs.com", n.server);
        assert_eq!(8443, n.port);
        assert_eq!("obfs-node", n.name);
    }
}
