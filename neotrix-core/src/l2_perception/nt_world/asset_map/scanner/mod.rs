//! NT-WORLD Asset Map: 端口扫描器
//!
//! 实现SYN扫描、Connect扫描、Service探测

use std::net::IpAddr;
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use crate::l2_perception::nt_world::port_service::guess_service as guess_service_shared;

/// 扫描配置
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// 目标IP
    pub target: IpAddr,
    /// 端口范围
    pub ports: Vec<u16>,
    /// 并发数
    pub concurrency: usize,
    /// 超时时间 (毫秒)
    pub timeout_ms: u64,
    /// 扫描类型
    pub scan_type: ScanType,
}

/// 扫描类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanType {
    /// Connect扫描 (全连接)
    Connect,
    /// Banner抓取
    BannerGrab,
    /// Service探测
    ServiceProbe,
}

/// 扫描结果
#[derive(Debug, Clone)]
pub struct PortScanResult {
    /// 端口号
    pub port: u16,
    /// 是否开放
    pub is_open: bool,
    /// Banner
    pub banner: Option<String>,
    /// 服务名称
    pub service: Option<String>,
    /// 耗时 (毫秒)
    pub latency_ms: u64,
}

/// 端口扫描器
pub struct PortScanner;

impl PortScanner {
    /// Connect扫描单个端口
    pub async fn scan_port(
        target: IpAddr,
        port: u16,
        timeout_ms: u64,
    ) -> PortScanResult {
        let start = std::time::Instant::now();
        let addr = format!("{}:{}", target, port);

        let result = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            TcpStream::connect(&addr),
        ).await;

        let latency = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(mut stream)) => {
                // 连接成功，尝试banner抓取
                let banner = Self::grab_banner(&mut stream).await;
                let service = Self::guess_service(port, banner.as_deref());

                PortScanResult {
                    port,
                    is_open: true,
                    banner,
                    service,
                    latency_ms: latency,
                }
            }
            _ => PortScanResult {
                port,
                is_open: false,
                banner: None,
                service: None,
                latency_ms: latency,
            },
        }
    }

    /// 批量扫描端口
    pub async fn scan_ports(config: &ScanConfig) -> Vec<PortScanResult> {
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(config.concurrency));
        let mut handles = Vec::new();

        for &port in &config.ports {
            let sem = semaphore.clone();
            let target = config.target;
            let timeout = config.timeout_ms;

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                Self::scan_port(target, port, timeout).await
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }

        results.sort_by_key(|r| r.port);
        results
    }

    /// 抓取Banner
    async fn grab_banner(stream: &mut TcpStream) -> Option<String> {
        let mut buf = [0u8; 1024];
        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            stream.read(&mut buf),
        ).await {
            Ok(Ok(n)) if n > 0 => {
                let banner = String::from_utf8_lossy(&buf[..n]).to_string();
                Some(banner.trim().to_string())
            }
            _ => None,
        }
    }

    /// 根据端口和Banner猜测服务
    fn guess_service(port: u16, banner: Option<&str>) -> Option<String> {
        let port_service = match port {
            21 => Some("ftp"),
            22 => Some("ssh"),
            23 => Some("telnet"),
            25 => Some("smtp"),
            53 => Some("dns"),
            80 => Some("http"),
            110 => Some("pop3"),
            143 => Some("imap"),
            443 => Some("https"),
            993 => Some("imaps"),
            995 => Some("pop3s"),
            3306 => Some("mysql"),
            5432 => Some("postgresql"),
            6379 => Some("redis"),
            8080 => Some("http-proxy"),
            _ => None,
        };

        // Banner补充
        if let Some(banner) = banner {
            let banner_lower = banner.to_lowercase();
            if banner_lower.contains("ssh") {
                return Some("ssh".into());
            }
            if banner_lower.contains("http") {
                return Some("http".into());
            }
            if banner_lower.contains("smtp") {
                return Some("smtp".into());
            }
        }

        port_service.map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn guess_service_by_port() {
        assert_eq!(PortScanner::guess_service(22, None), Some("ssh".into()));
        assert_eq!(PortScanner::guess_service(80, None), Some("http".into()));
        assert_eq!(PortScanner::guess_service(443, None), Some("https".into()));
        assert_eq!(PortScanner::guess_service(3306, None), Some("mysql".into()));
    }

    #[test]
    fn guess_service_by_banner() {
        assert_eq!(
            PortScanner::guess_service(12345, Some("SSH-2.0-OpenSSH_8.9")),
            Some("ssh".into())
        );
        assert_eq!(
            PortScanner::guess_service(8080, Some("HTTP/1.1 200 OK")),
            Some("http".into())
        );
    }

    #[tokio::test]
    async fn scan_localhost() {
        // 扫描本地不存在的端口，应该返回closed
        let result = PortScanner::scan_port(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            19999,  // 不太可能开放的端口
            1000,
        ).await;
        assert!(!result.is_open);
    }
}
