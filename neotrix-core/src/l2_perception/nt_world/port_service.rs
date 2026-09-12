//! 端口→服务映射 (单一事实源)
//!
//! 消除 `osint/network.rs` COMMON_PORTS 与 `asset_map/scanner/mod.rs` guess_service 的重复。

/// 端口→(服务名, 协议) 映射表
pub const PORT_SERVICE_MAP: &[(u16, &str, &str)] = &[
    (21, "ftp", "tcp"), (22, "ssh", "tcp"), (23, "telnet", "tcp"),
    (25, "smtp", "tcp"), (53, "dns", "tcp"), (80, "http", "tcp"),
    (110, "pop3", "tcp"), (143, "imap", "tcp"), (443, "https", "tcp"),
    (445, "smb", "tcp"), (993, "imaps", "tcp"), (995, "pop3s", "tcp"),
    (1433, "mssql", "tcp"), (1521, "oracle", "tcp"), (2049, "nfs", "tcp"),
    (2375, "docker", "tcp"), (2376, "docker-tls", "tcp"),
    (3306, "mysql", "tcp"), (3389, "rdp", "tcp"), (5432, "postgresql", "tcp"),
    (5900, "vnc", "tcp"), (6379, "redis", "tcp"), (6443, "kubernetes", "tcp"),
    (8080, "http-alt", "tcp"), (8443, "https-alt", "tcp"),
    (9000, "php-fpm", "tcp"), (9090, "prometheus", "tcp"),
    (27017, "mongodb", "tcp"),
];

/// 根据端口号猜测服务名
pub fn guess_service_by_port(port: u16) -> Option<&'static str> {
    PORT_SERVICE_MAP.iter()
        .find(|(p, _, _)| *p == port)
        .map(|(_, svc, _)| *svc)
}

/// 根据端口号获取协议
pub fn _guess_protocol_by_port(port: u16) -> Option<&'static str> {
    PORT_SERVICE_MAP.iter()
        .find(|(p, _, _)| *p == port)
        .map(|(_, _, proto)| *proto)
}

/// 根据 Banner 补充服务识别
pub fn guess_service_by_banner(banner: &str) -> Option<&'static str> {
    let b = banner.to_lowercase();
    if b.contains("ssh") { return Some("ssh"); }
    if b.contains("http") { return Some("http"); }
    if b.contains("smtp") { return Some("smtp"); }
    if b.contains("ftp") { return Some("ftp"); }
    if b.contains("mysql") { return Some("mysql"); }
    if b.contains("redis") { return Some("redis"); }
    if b.contains("mongo") { return Some("mongodb"); }
    if b.contains("postgresql") || b.contains("postgres") { return Some("postgresql"); }
    None
}

/// 综合猜测: Banner 优先，端口兜底
pub fn guess_service(port: u16, banner: Option<&str>) -> Option<String> {
    if let Some(b) = banner {
        if let Some(svc) = guess_service_by_banner(b) {
            return Some(svc.to_string());
        }
    }
    guess_service_by_port(port).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_by_port() {
        assert_eq!(guess_service_by_port(22), Some("ssh"));
        assert_eq!(guess_service_by_port(80), Some("http"));
        assert_eq!(guess_service_by_port(443), Some("https"));
        assert_eq!(guess_service_by_port(3306), Some("mysql"));
        assert_eq!(guess_service_by_port(5432), Some("postgresql"));
        assert_eq!(guess_service_by_port(6379), Some("redis"));
        assert_eq!(guess_service_by_port(99999), None);
    }

    #[test]
    fn test_guess_by_banner() {
        assert_eq!(guess_service_by_banner("SSH-2.0-OpenSSH_8.9"), Some("ssh"));
        assert_eq!(guess_service_by_banner("HTTP/1.1 200 OK"), Some("http"));
        assert_eq!(guess_service_by_banner("220 mail.example.com ESMTP"), Some("smtp"));
        assert_eq!(guess_service_by_banner("random stuff"), None);
    }

    #[test]
    fn test_guess_combined() {
        // Banner takes priority
        assert_eq!(guess_service(12345, Some("SSH-2.0-OpenSSH")), Some("ssh".into()));
        // Fallback to port
        assert_eq!(guess_service(22, None), Some("ssh".into()));
        assert_eq!(guess_service(99999, None), None);
    }

    #[test]
    fn test_protocol() {
        assert_eq!(_guess_protocol_by_port(22), Some("tcp"));
        assert_eq!(_guess_protocol_by_port(80), Some("tcp"));
    }
}
