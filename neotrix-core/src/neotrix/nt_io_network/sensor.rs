use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_network::dns_cache::AddressFamily;
use crate::core::nt_core_network::dns_cache::VsaDnsCache;
use crate::core::nt_core_util;

fn proxy_socket_path() -> String {
    let home = nt_core_util::home_dir().to_string_lossy().to_string();
    format!("{}/.neotrix/neotrix-proxy.sock", home)
}

const PROXY_BIN: &str = "neotrix-proxy";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkSignal {
    ConnectionSuccess,
    ConnectionTimeout,
    TlsError,
    DnsFailed,
    FingerprintBlocked,
    RateLimited,
}

impl NetworkSignal {
    pub fn to_vsa(&self, _vsa: &QuantizedVSA, seed: u64) -> Vec<u8> {
        let tag = match self {
            NetworkSignal::ConnectionSuccess => 0,
            NetworkSignal::ConnectionTimeout => 1,
            NetworkSignal::TlsError => 2,
            NetworkSignal::DnsFailed => 3,
            NetworkSignal::FingerprintBlocked => 4,
            NetworkSignal::RateLimited => 5,
        };
        QuantizedVSA::seeded_random(seed + tag as u64, 1024)
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionOutcome {
    pub domain: String,
    pub signal: NetworkSignal,
    pub fingerprint: String,
    pub latency_ms: u64,
    pub timestamp: SystemTime,
}

pub struct NetworkSensor {
    proxy_path: Option<PathBuf>,
    outcomes: Vec<ConnectionOutcome>,
    fingerprint_confidence: HashMap<String, HashMap<String, f64>>,
    last_fingerprint: Option<String>,
    last_domain: Option<String>,
    pub dns_cache: VsaDnsCache,
}

impl NetworkSensor {
    pub fn new(_dim: usize) -> Self {
        Self {
            proxy_path: Self::find_proxy_binary(),
            outcomes: Vec::with_capacity(4096),
            fingerprint_confidence: HashMap::new(),
            last_fingerprint: None,
            last_domain: None,
            dns_cache: VsaDnsCache::new(500, Duration::from_secs(300)),
        }
    }

    fn find_proxy_binary() -> Option<PathBuf> {
        if let Ok(path) = std::env::var("NEOTRIX_PROXY_PATH") {
            let p = PathBuf::from(&path);
            if p.exists() {
                return Some(p);
            }
        }
        if let Ok(paths) = std::env::var("PATH") {
            for dir in std::env::split_paths(&paths) {
                let p = dir.join(PROXY_BIN);
                if p.exists() {
                    return Some(p);
                }
            }
        }
        let target = PathBuf::from(std::env::current_dir().unwrap_or_default())
            .parent()
            .map(|p| p.join("target/debug").join(PROXY_BIN));
        if let Some(p) = target {
            if p.exists() {
                return Some(p);
            }
        }
        None
    }

    pub fn proxy_available(&self) -> bool {
        self.proxy_path.is_some()
    }

    pub async fn start_proxy(&self) -> Result<(), String> {
        let path = self.proxy_path.as_ref().ok_or("proxy binary not found")?;
        if self.is_proxy_running().await {
            return Ok(());
        }
        let home = nt_core_util::home_dir().to_string_lossy().to_string();
        let out = std::fs::File::create(format!("{}/.neotrix/neotrix-proxy.out.log", home))
            .map_err(|e| format!("log: {}", e))?;
        let err = std::fs::File::create(format!("{}/.neotrix/neotrix-proxy.err.log", home))
            .map_err(|e| format!("log: {}", e))?;
        Command::new(path)
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err))
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn: {}", e))?;

        for _ in 0..10 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if self.is_proxy_running().await {
                return Ok(());
            }
        }
        Err("proxy not ready within 10s".into())
    }

    pub async fn stop_proxy(&self) -> Result<(), String> {
        if self.is_proxy_running().await {
            let _ = Command::new("pkill").args(["-x", PROXY_BIN]).output();
        }
        Ok(())
    }

    pub async fn is_proxy_running(&self) -> bool {
        if let Ok(out) = Command::new("ps").args(["-ax", "-o", "comm="]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            return s.lines().any(|l| l.contains(PROXY_BIN));
        }
        false
    }

    pub async fn send_command(&self, cmd: &str) -> Result<String, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let sock_path = proxy_socket_path();
        let mut stream = tokio::net::UnixStream::connect(&sock_path)
            .await
            .map_err(|e| format!("connect socket: {}", e))?;
        stream
            .write_all(format!("{}\n", cmd).as_bytes())
            .await
            .map_err(|e| format!("write: {}", e))?;

        let mut reader = BufReader::new(&mut stream);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .await
            .map_err(|e| format!("read: {}", e))?;
        Ok(response.trim().to_string())
    }

    pub async fn set_stealth_mode(&self) -> Result<(), String> {
        self.send_command("mode=stealth").await?;
        Ok(())
    }

    pub async fn set_direct_mode(&self) -> Result<(), String> {
        self.send_command("mode=direct").await?;
        Ok(())
    }

    pub async fn rotate_fingerprint(&self) -> Result<String, String> {
        let resp = self.send_command("fingerprint=rotate").await?;
        Ok(resp)
    }

    pub async fn get_status(&self) -> Result<String, String> {
        self.send_command("status").await
    }

    pub fn record_outcome(&mut self, outcome: ConnectionOutcome) {
        let fp = outcome.fingerprint.clone();
        let domain = outcome.domain.clone();
        let _key = format!("{}:{}", fp, domain);

        let entry = self.fingerprint_confidence.entry(fp.clone()).or_default();
        let conf = entry.entry(domain.clone()).or_insert(1.0);
        match outcome.signal {
            NetworkSignal::ConnectionSuccess => {
                *conf = (*conf + 0.05).min(1.0);
            }
            NetworkSignal::ConnectionTimeout
            | NetworkSignal::TlsError
            | NetworkSignal::FingerprintBlocked => {
                *conf = (*conf * 0.8).max(0.1);
            }
            _ => {}
        }

        self.last_fingerprint = Some(fp);
        self.last_domain = Some(domain);
        self.outcomes.push(outcome);
    }

    pub fn best_fingerprint_for(&self, domain: &str) -> Option<String> {
        let ranked: Vec<(&String, &f64)> = self
            .fingerprint_confidence
            .iter()
            .flat_map(|(fp, domains)| domains.get(domain).map(|c| (fp, c)))
            .collect();
        ranked
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(fp, _)| fp.clone())
    }

    pub fn novelty_bonus(&self) -> Option<String> {
        let known: Vec<&str> = self
            .fingerprint_confidence
            .keys()
            .map(|s| s.as_str())
            .collect();
        let candidates = [
            "ChromeMac",
            "FirefoxMac",
            "SafariMac",
            "ChromeLinux",
            "EdgeWin",
        ];
        for c in &candidates {
            if !known.contains(c) {
                return Some(c.to_string());
            }
        }
        None
    }

    pub fn network_negentropy(&self) -> f64 {
        let recent: Vec<&ConnectionOutcome> = self.outcomes.iter().rev().take(100).collect();
        if recent.is_empty() {
            return 0.5;
        }
        let successes = recent
            .iter()
            .filter(|o| matches!(o.signal, NetworkSignal::ConnectionSuccess))
            .count();
        successes as f64 / recent.len() as f64
    }

    pub fn most_challenging_domain(&self) -> Option<String> {
        let mut domain_confidence: HashMap<&str, Vec<f64>> = HashMap::new();
        for (_fp, domains) in &self.fingerprint_confidence {
            for (domain, conf) in domains {
                domain_confidence
                    .entry(domain.as_str())
                    .or_default()
                    .push(*conf);
            }
        }
        domain_confidence
            .into_iter()
            .map(|(d, scores)| {
                let avg = scores.iter().sum::<f64>() / scores.len() as f64;
                (d, avg)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(d, _)| d.to_string())
    }

    pub fn resolve_dns(&mut self, domain: &str) -> Option<IpAddr> {
        let ip_v4 = self.dns_cache.resolve(domain, AddressFamily::V4);
        if ip_v4.is_some() {
            return ip_v4;
        }
        self.dns_cache.resolve(domain, AddressFamily::V6)
    }

    pub fn cache_dns(&mut self, domain: &str, ip: IpAddr) {
        let family = if ip.is_ipv4() {
            AddressFamily::V4
        } else {
            AddressFamily::V6
        };
        self.dns_cache.insert(domain, ip, family);
    }

    pub fn dns_cache_mut(&mut self) -> &mut VsaDnsCache {
        &mut self.dns_cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_signal_to_vsa_different() {
        let vsa = QuantizedVSA::new(64);
        let s1 = NetworkSignal::ConnectionSuccess.to_vsa(&vsa, 42);
        let s2 = NetworkSignal::ConnectionTimeout.to_vsa(&vsa, 42);
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_network_signal_to_vsa_deterministic() {
        let vsa = QuantizedVSA::new(64);
        let a = NetworkSignal::RateLimited.to_vsa(&vsa, 100);
        let b = NetworkSignal::RateLimited.to_vsa(&vsa, 100);
        assert_eq!(a, b);
    }

    #[test]
    fn test_proxy_available_initially_false() {
        let sensor = NetworkSensor::new(64);
        assert!(!sensor.proxy_available());
    }

    #[test]
    fn test_best_fingerprint_for_empty() {
        let sensor = NetworkSensor::new(64);
        assert_eq!(sensor.best_fingerprint_for("example.com"), None);
    }

    #[test]
    fn test_best_fingerprint_for_after_record() {
        let mut sensor = NetworkSensor::new(64);
        sensor.record_outcome(ConnectionOutcome {
            domain: "example.com".into(),
            signal: NetworkSignal::ConnectionSuccess,
            fingerprint: "ChromeMac".into(),
            latency_ms: 50,
            timestamp: SystemTime::now(),
        });
        let best = sensor.best_fingerprint_for("example.com");
        assert_eq!(best, Some("ChromeMac".to_string()));
    }

    #[test]
    fn test_novelty_bonus_returns_candidate() {
        let sensor = NetworkSensor::new(64);
        let bonus = sensor.novelty_bonus();
        assert!(bonus.is_some());
    }

    #[test]
    fn test_network_negentropy_empty() {
        let sensor = NetworkSensor::new(64);
        let n = sensor.network_negentropy();
        assert!((n - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_network_negentropy_all_success() {
        let mut sensor = NetworkSensor::new(64);
        for i in 0..10 {
            sensor.record_outcome(ConnectionOutcome {
                domain: format!("d{}", i),
                signal: NetworkSignal::ConnectionSuccess,
                fingerprint: "fp".into(),
                latency_ms: 10,
                timestamp: SystemTime::now(),
            });
        }
        let n = sensor.network_negentropy();
        assert!((n - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_most_challenging_domain_empty() {
        let sensor = NetworkSensor::new(64);
        assert_eq!(sensor.most_challenging_domain(), None);
    }

    #[test]
    fn test_most_challenging_domain_after_record() {
        let mut sensor = NetworkSensor::new(64);
        sensor.record_outcome(ConnectionOutcome {
            domain: "good.com".into(),
            signal: NetworkSignal::ConnectionSuccess,
            fingerprint: "ChromeMac".into(),
            latency_ms: 10,
            timestamp: SystemTime::now(),
        });
        sensor.record_outcome(ConnectionOutcome {
            domain: "bad.com".into(),
            signal: NetworkSignal::FingerprintBlocked,
            fingerprint: "ChromeMac".into(),
            latency_ms: 5000,
            timestamp: SystemTime::now(),
        });
        let challenging = sensor.most_challenging_domain();
        assert_eq!(challenging, Some("bad.com".to_string()));
    }
}
