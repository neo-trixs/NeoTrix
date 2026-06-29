use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::types::{
    ApiEndpoint, ApiHealth, ConnectionFailureRootCause, EndpointDiagnostic, NetworkEnvironment,
    VpnType,
};

// ═══════════════════════════════════════════════════════════════
// EWMA — Exponentially Weighted Moving Average anomaly detector
// ═══════════════════════════════════════════════════════════════

pub struct EwmaDetector {
    alpha: f64,
    mean: f64,
    variance: f64,
    sample_count: u64,
    warmup: u64,
}

impl EwmaDetector {
    pub fn new(alpha: f64, warmup: u64) -> Self {
        Self {
            alpha,
            mean: 0.0,
            variance: 0.0,
            sample_count: 0,
            warmup,
        }
    }

    pub fn observe(&mut self, value: f64) -> f64 {
        self.sample_count += 1;
        if self.sample_count == 1 {
            self.mean = value;
            return 0.0;
        }
        let diff = value - self.mean;
        let incr = self.alpha * diff;
        self.mean += incr;
        // Welford online variance
        self.variance = (1.0 - self.alpha) * (self.variance + self.alpha * diff * diff);
        let std = self.variance.sqrt().max(1e-9);
        let z = diff / std;
        if self.sample_count < self.warmup {
            0.0
        } else {
            z
        }
    }

    pub fn mean(&self) -> f64 {
        self.mean
    }
    pub fn std(&self) -> f64 {
        self.variance.sqrt().max(1e-9)
    }
    pub fn is_warmed_up(&self) -> bool {
        self.sample_count >= self.warmup
    }
}

// ═══════════════════════════════════════════════════════════════
// CUSUM — Cumulative Sum change point detector
// ═══════════════════════════════════════════════════════════════

pub struct CusumDetector {
    target: f64,
    k: f64,         // allowance — typically 0.5
    h: f64,         // decision interval — typically 5.0
    sum_plus: f64,  // cumulative deviation above target + k
    sum_minus: f64, // cumulative deviation below target - k
    sample_count: u64,
    warmup: u64,
}

impl CusumDetector {
    pub fn new(target: f64, k: f64, h: f64, warmup: u64) -> Self {
        Self {
            target,
            k,
            h,
            sum_plus: 0.0,
            sum_minus: 0.0,
            sample_count: 0,
            warmup,
        }
    }

    /// Returns (is_above_alarm, is_below_alarm)
    pub fn observe(&mut self, value: f64) -> (bool, bool) {
        self.sample_count += 1;
        if self.sample_count < self.warmup {
            return (false, false);
        }

        let s_plus = (value - self.target - self.k).max(0.0);
        let s_minus = (self.target - self.k - value).max(0.0);
        self.sum_plus = (self.sum_plus + s_plus).max(0.0);
        self.sum_minus = (self.sum_minus + s_minus).max(0.0);

        let above = self.sum_plus > self.h;
        let below = self.sum_minus > self.h;
        if above {
            self.sum_plus = 0.0;
        }
        if below {
            self.sum_minus = 0.0;
        }
        (above, below)
    }

    pub fn reset(&mut self) {
        self.sum_plus = 0.0;
        self.sum_minus = 0.0;
    }
}

// ═══════════════════════════════════════════════════════════════
// Holt-Winters triple exponential smoothing (periodic latency prediction)
// ═══════════════════════════════════════════════════════════════

pub struct HoltWinters {
    alpha: f64,
    beta: f64,
    gamma: f64,
    period: usize,
    level: f64,
    trend: f64,
    seasonal: Vec<f64>,
    t: usize,
    residual_std: f64,
    warmup_periods: usize,
}

impl HoltWinters {
    /// `period` = seasonal cycle length (e.g., 24 for hourly, 168 for weekly)
    /// `warmup_periods` = full cycles to warm up before prediction (min 1)
    pub fn new(alpha: f64, beta: f64, gamma: f64, period: usize, warmup_periods: usize) -> Self {
        Self {
            alpha,
            beta,
            gamma,
            period,
            level: 0.0,
            trend: 0.0,
            seasonal: vec![1.0; period],
            t: 0,
            residual_std: 0.0,
            warmup_periods,
        }
    }

    pub fn defaults(period: usize) -> Self {
        Self::new(0.4, 0.2, 0.2, period, 2)
    }

    /// Returns (forecast, z_score) where z_score = |actual - forecast| / residual_std
    pub fn step(&mut self, y: f64) -> (f64, f64) {
        let i = self.t % self.period;

        if self.t < self.period * self.warmup_periods {
            // Warmup: accumulate seasonal baseline
            if self.t == 0 {
                self.level = y;
            } else if self.t < self.period {
                self.level =
                    self.level * (self.t as f64 / (self.t + 1) as f64) + y / (self.t + 1) as f64;
                self.seasonal[i] = y / self.level;
            } else if self.t == self.period {
                // First complete cycle: set initial trend
                let first_cycle_avg = self.level;
                self.level = y;
                self.trend = (y - first_cycle_avg) / self.period as f64;
                self.seasonal[i] = y / self.level;
            } else {
                // Second cycle onward: HW update with sequential initialization
                let last_seasonal = self.seasonal[i];
                let new_level = self.alpha * (y / last_seasonal.max(1e-9))
                    + (1.0 - self.alpha) * (self.level + self.trend);
                let new_trend =
                    self.beta * (new_level - self.level) + (1.0 - self.beta) * self.trend;
                let new_seasonal =
                    self.gamma * (y / new_level.max(1e-9)) + (1.0 - self.gamma) * last_seasonal;

                let forecast = (self.level + self.trend) * self.seasonal[i];
                let residual = y - forecast;
                self.residual_std = self.residual_std * 0.9 + residual.abs() * 0.1;

                self.level = new_level;
                self.trend = new_trend;
                self.seasonal[i] = new_seasonal;
            }
            self.t += 1;
            return (y, 0.0);
        }

        // Full HW update
        let last_seasonal = self.seasonal[i];
        let new_level = self.alpha * (y / last_seasonal.max(1e-9))
            + (1.0 - self.alpha) * (self.level + self.trend);
        let new_trend = self.beta * (new_level - self.level) + (1.0 - self.beta) * self.trend;
        let new_seasonal =
            self.gamma * (y / new_level.max(1e-9)) + (1.0 - self.gamma) * last_seasonal;

        let forecast = (self.level + self.trend) * self.seasonal[i];
        let residual = y - forecast;
        // EWMA residual standard deviation
        self.residual_std = self.residual_std * 0.9 + residual.abs() * 0.1;

        self.level = new_level;
        self.trend = new_trend;
        self.seasonal[i] = new_seasonal;
        self.t += 1;

        let z = if self.residual_std > 1e-9 {
            residual / (self.residual_std + 1e-9)
        } else {
            0.0
        };
        (forecast, z)
    }

    pub fn predict_next(&self) -> f64 {
        let i = self.t % self.period;
        (self.level + self.trend) * self.seasonal[i]
    }

    pub fn is_warmed_up(&self) -> bool {
        self.t >= self.period * self.warmup_periods
    }
}

// ═══════════════════════════════════════════════════════════════
// Environment scanning helpers
// ═══════════════════════════════════════════════════════════════

fn fake_ip_dns() -> Option<String> {
    let resolv = std::fs::read_to_string("/etc/resolv.conf").ok()?;
    for line in resolv.lines() {
        if let Some(ip) = line.strip_prefix("nameserver ") {
            if ip.trim().starts_with("198.18.") {
                return Some(ip.trim().to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn tun_interfaces() -> Vec<String> {
    std::process::Command::new("ifconfig")
        .output()
        .ok()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.lines()
                .filter(|l| l.starts_with("utun"))
                .map(|l| l.split(':').next().unwrap_or(l).to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(not(target_os = "macos"))]
fn tun_interfaces() -> Vec<String> {
    let tun_dirs = [
        "/sys/class/net/tun0",
        "/sys/class/net/tun1",
        "/sys/class/net/tun2",
        "/sys/class/net/tun3",
        "/sys/class/net/tap0",
        "/sys/class/net/tap1",
        "/sys/class/net/wg0",
        "/sys/class/net/wg1",
    ];
    tun_dirs
        .iter()
        .filter(|d| std::path::Path::new(d).exists())
        .filter_map(|d| d.rsplit('/').next().map(|s| s.to_string()))
        .collect()
}

#[cfg(target_os = "macos")]
fn default_interface() -> Option<String> {
    std::process::Command::new("route")
        .args(["-n", "get", "default"])
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.lines()
                .find(|l| l.contains("interface:"))
                .and_then(|l| l.split("interface:").nth(1))
                .map(|s| s.trim().to_string())
        })
}

#[cfg(not(target_os = "macos"))]
fn default_interface() -> Option<String> {
    std::process::Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            // Line format: "default via X.X.X.X dev eth0"
            out.lines().find_map(|l| {
                l.split_whitespace()
                    .position(|w| w == "dev")
                    .and_then(|pos| l.split_whitespace().nth(pos + 1))
                    .map(|s| s.to_string())
            })
        })
}

fn physical_ip(iface: &str) -> Option<String> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        std::process::Command::new("ifconfig")
            .arg(iface)
            .output()
            .ok()
            .and_then(|o| {
                let out = String::from_utf8_lossy(&o.stdout);
                for line in out.lines() {
                    if let Some(rest) = line.trim().strip_prefix("inet ") {
                        let ip = rest.split_whitespace().next()?;
                        if !ip.starts_with("127.") && !ip.starts_with("198.18.") {
                            return Some(ip.to_string());
                        }
                    }
                }
                None
            })
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = iface;
        None
    }
}

fn detect_vpn_type(tuns: &[String], fake_ip: &Option<String>) -> Option<VpnType> {
    if fake_ip.is_some() {
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("ps").args(["aux"]).output() {
                let out = String::from_utf8_lossy(&output.stdout).to_lowercase();
                if out.contains("shadowrocket") {
                    return Some(VpnType::Shadowrocket);
                }
                if out.contains("surge") {
                    return Some(VpnType::Surge);
                }
                if out.contains("clash") {
                    return Some(VpnType::Clash);
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("ps").args(["aux"]).output() {
                let out = String::from_utf8_lossy(&output.stdout).to_lowercase();
                if out.contains("openvpn") {
                    return Some(VpnType::Clash);
                }
                if out.contains("wireguard") || out.contains("wg-quick") {
                    return Some(VpnType::Clash);
                }
            }
        }
        if !tuns.is_empty() {
            return Some(VpnType::UnknownTun);
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════
// Endpoint probing
// ═══════════════════════════════════════════════════════════════

pub async fn check_endpoint(endpoint: &ApiEndpoint) -> EndpointDiagnostic {
    check_endpoint_cached(endpoint, None).await
}

pub async fn check_endpoint_cached(
    endpoint: &ApiEndpoint,
    mut dns_cache: Option<&mut crate::core::nt_core_network::dns_cache::VsaDnsCache>,
) -> EndpointDiagnostic {
    let start = Instant::now();
    let url_str = endpoint.url;
    let host_port = url_str
        .strip_prefix("https://")
        .or_else(|| url_str.strip_prefix("http://"))
        .and_then(|rest| rest.split('/').next())
        .unwrap_or(url_str);

    let (host, _port_str) = host_port.rsplit_once(':').unwrap_or((host_port, "443"));

    // TCP connect timing — try cache first
    let addr = if let Some(cache) = dns_cache.as_deref_mut() {
        if let Some(ip) = cache.resolve(
            host,
            crate::core::nt_core_network::dns_cache::AddressFamily::V4,
        ) {
            Some(std::net::SocketAddr::new(ip, 443u16))
        } else if let Some(ip) = cache.resolve(
            host,
            crate::core::nt_core_network::dns_cache::AddressFamily::V6,
        ) {
            Some(std::net::SocketAddr::new(ip, 443u16))
        } else {
            match timeout(
                Duration::from_secs(5),
                tokio::net::lookup_host(format!("{}:443", host_port)),
            )
            .await
            {
                Ok(Ok(mut addrs)) => {
                    let first = addrs.next();
                    if let Some(a) = first {
                        cache.insert(
                            host,
                            a.ip(),
                            crate::core::nt_core_network::dns_cache::AddressFamily::V4,
                        );
                        Some(a)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    } else {
        match timeout(
            Duration::from_secs(5),
            tokio::net::lookup_host(format!("{}:443", host_port)),
        )
        .await
        {
            Ok(Ok(mut addrs)) => addrs.next(),
            _ => None,
        }
    };

    let addr = match addr {
        Some(a) => a,
        None => {
            return EndpointDiagnostic {
                endpoint: endpoint.name,
                health: ApiHealth::DnsFailure {
                    reason: "No address resolved".into(),
                },
                timing: None,
                root_cause: Some(ConnectionFailureRootCause::DnsResolutionFailed),
            };
        }
    };

    match timeout(Duration::from_secs(10), TcpStream::connect(addr)).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            let err_str = e.to_string();
            let cause = if err_str.contains("refused") {
                ConnectionFailureRootCause::ConnectionRefused
            } else if err_str.contains("reset") {
                ConnectionFailureRootCause::ConnectionReset
            } else if err_str.contains("timeout") {
                ConnectionFailureRootCause::ConnectionTimeout
            } else {
                ConnectionFailureRootCause::Unknown(err_str.clone())
            };
            return EndpointDiagnostic {
                endpoint: endpoint.name,
                health: ApiHealth::Unreachable { reason: err_str },
                timing: None,
                root_cause: Some(cause),
            };
        }
        Err(_) => {
            return EndpointDiagnostic {
                endpoint: endpoint.name,
                health: ApiHealth::Unreachable {
                    reason: "TCP connection timeout".into(),
                },
                timing: None,
                root_cause: Some(ConnectionFailureRootCause::ConnectionTimeout),
            };
        }
    }

    // HTTP HEAD
    let client = match reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .timeout(Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return EndpointDiagnostic {
                endpoint: endpoint.name,
                health: ApiHealth::Unreachable {
                    reason: format!("HTTP client error: {}", e),
                },
                timing: None,
                root_cause: Some(ConnectionFailureRootCause::Unknown(e.to_string())),
            };
        }
    };

    let response = match client.head(endpoint.url).send().await {
        Ok(r) => r,
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("503") || err_str.contains("status code 503") {
                return EndpointDiagnostic {
                    endpoint: endpoint.name,
                    health: ApiHealth::HttpError {
                        status_code: 503,
                        body_snippet: err_str,
                    },
                    timing: None,
                    root_cause: Some(ConnectionFailureRootCause::HttpServiceUnavailable),
                };
            }
            return EndpointDiagnostic {
                endpoint: endpoint.name,
                health: ApiHealth::Unreachable { reason: err_str },
                timing: None,
                root_cause: None,
            };
        }
    };

    let total = start.elapsed();
    let status = response.status().as_u16();
    let ttfb = total;

    let health = if status == 503 {
        ApiHealth::HttpError {
            status_code: 503,
            body_snippet: String::new(),
        }
    } else if ttfb > Duration::from_secs(10) {
        ApiHealth::Degraded {
            latency: total,
            ttfb,
        }
    } else if status == endpoint.expected_status || status == 401 || status == 422 {
        if ttfb > Duration::from_secs(3) {
            ApiHealth::Degraded {
                latency: total,
                ttfb,
            }
        } else {
            ApiHealth::Healthy {
                latency: total,
                ttfb,
            }
        }
    } else {
        ApiHealth::HttpError {
            status_code: status,
            body_snippet: String::new(),
        }
    };

    EndpointDiagnostic {
        endpoint: endpoint.name,
        health,
        timing: None,
        root_cause: None,
    }
}

pub async fn scan_environment() -> NetworkEnvironment {
    let tuns = tun_interfaces();
    let fake_ip = fake_ip_dns();
    let def_iface = default_interface();
    let vpn_type = detect_vpn_type(&tuns, &fake_ip);
    let phys_ip = def_iface.as_ref().and_then(|i| physical_ip(i));

    let (country, org) = if let Ok(resp) = reqwest::get("https://ipinfo.io/json").await {
        if let Ok(body) = resp.text().await {
            let c = body
                .split("\"country\":\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .map(|s| s.to_string());
            let o = body
                .split("\"org\":\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .map(|s| s.to_string());
            (c, o)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    NetworkEnvironment {
        has_tun_interfaces: tuns,
        fake_ip_dns: fake_ip,
        physical_ip: phys_ip,
        default_interface: def_iface,
        vpn_type,
        egress_country: country,
        egress_org: org,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ewma_detector() {
        let mut e = EwmaDetector::new(0.3, 5);
        for v in [1.0, 1.1, 0.9, 1.0, 1.05, 1.0, 1.1, 1.0, 0.95, 1.0] {
            e.observe(v);
        }
        // After warmup, z should be small for stable data
        let z = e.observe(1.0);
        assert!(z.abs() < 3.0, "stable data should give low z, got {}", z);
    }

    #[test]
    fn test_ewma_anomaly_detection() {
        let mut e = EwmaDetector::new(0.2, 20);
        for _ in 0..20 {
            e.observe(0.5);
        } // warmup with 0.5s
        let z = e.observe(10.0); // spike to 10s
        assert!(z > 2.0, "spike should produce high z-score, got {}", z);
    }

    #[test]
    fn test_cusum_detector() {
        let mut c = CusumDetector::new(0.5, 0.5, 5.0, 10);
        // warmup
        for _ in 0..10 {
            c.observe(0.5);
        }
        let (above, _) = c.observe(0.5);
        assert!(!above, "stable data should not alarm");
    }

    #[test]
    fn test_cusum_alarm() {
        let mut c = CusumDetector::new(0.5, 0.5, 5.0, 10);
        for _ in 0..10 {
            c.observe(0.5);
        }
        // Sustained high latency
        let mut alarmed = false;
        for _ in 0..20 {
            let (a, _) = c.observe(10.0);
            if a {
                alarmed = true;
                break;
            }
        }
        assert!(alarmed, "sustained high latency should trigger CUSUM alarm");
    }

    #[test]
    fn test_holt_winters_basic() {
        let mut hw = HoltWinters::new(0.4, 0.2, 0.2, 24, 2);
        // Warmup with stable values
        for _ in 0..48 {
            hw.step(1.0);
        }
        assert!(hw.is_warmed_up());
        let (f, z) = hw.step(1.0);
        assert!(z.abs() < 3.0, "stable data should give low z, got {}", z);
        assert!((f - 1.0).abs() < 1.0, "forecast should be ~1.0, got {}", f);
    }

    #[test]
    fn test_holt_winters_anomaly() {
        let mut hw = HoltWinters::new(0.4, 0.2, 0.2, 12, 2);
        for _ in 0..24 {
            hw.step(1.0);
        }
        let (_, z) = hw.step(10.0); // spike
        assert!(z.abs() > 2.0, "spike should produce high z, got {}", z);
    }
}
