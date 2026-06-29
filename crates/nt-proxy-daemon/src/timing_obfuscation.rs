use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for cross-layer timing obfuscation.
///
/// Defines per-layer jitter ceilings plus a *correlation factor* that ties
/// delays across layers together.  The goal is to make layer-to-layer RTT
/// relationships resemble a *direct* connection rather than a proxied one.
///
/// # Background (NDSS '25 — Xue et al.)
///
/// In a proxy tunnel the transport-layer session terminates at the proxy
/// while the application-layer session remains end-to-end.  An on-path
/// observer can measure:
///
/// - **TCP RTT** (SYN→SYN/ACK) — reflects client↔proxy only
/// - **TLS RTT** (ClientHello→ServerHello…) — includes proxy↔server leg
/// - **App RTT** (request→response) — includes proxy↔server leg
///
/// The cross‑layer difference `RTTdiff = app_rtt − tcp_rtt` is small for
/// direct connections (~server processing, < 15 ms) but large for proxy
/// traffic (~proxy↔server propagation).  The attacker classifies a flow as
/// proxied when `RTTdiff` exceeds a threshold.
///
/// # Mitigation strategy
///
/// Inject *correlated* delay at each layer so that transport and application
/// RTTs move together, keeping `RTTdiff` within the direct-connection
/// envelope.  When `correlation_factor` triggers, all layers draw from the
/// same normalised sample; otherwise each layer jitters independently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingShapingConfig {
    /// Maximum delay (ms) injected before TCP SYN.
    ///
    /// Inflates the observable TCP RTT so it approaches the total
    /// client↔proxy + proxy↔server latency that the app layer sees.
    /// Range: 0–200.
    pub tcp_jitter_ms: u64,

    /// Maximum delay (ms) injected before TLS ClientHello.
    ///
    /// Masks the extra handshake round‑trips introduced by the proxy leg.
    /// Range: 0–200.
    pub tls_jitter_ms: u64,

    /// Maximum delay (ms) injected before the first application payload
    /// byte (and before subsequent data chunks).
    ///
    /// Kept relatively small so the app RTT does not grow beyond the
    /// inflated TCP RTT.  Range: 0–200.
    pub data_jitter_ms: u64,

    /// Probability (0.0–1.0) that the *same* normalised random sample
    /// is used for every layer in a given connection.
    ///
    /// High correlation → stable `RTTdiff` across connections (looks
    /// natural).  Low correlation → noisy `RTTdiff` (harder to fingerprint
    /// but also harder to tune).  Recommended range: 0.50–0.80.
    pub correlation_factor: f64,
}

impl TimingShapingConfig {
    /// Real-world defaults based on typical proxy measurements:
    ///
    /// | Quantity                | Typical value |
    /// |-------------------------|---------------|
    /// | Client↔Proxy RTT        | 50–150 ms     |
    /// | Proxy↔Server RTT        | 20–100 ms     |
    /// | Combined (data path)    | 70–250 ms     |
    /// | Server processing       | 1–10 ms       |
    ///
    /// The TCP jitter is set *larger* than data jitter so that TCP RTT
    /// is inflated up toward the combined RTT, shrinking `RTTdiff`.
    pub fn default() -> Self {
        Self {
            tcp_jitter_ms: 120,
            tls_jitter_ms: 80,
            data_jitter_ms: 50,
            correlation_factor: 0.65,
        }
    }

    /// Conservative profile optimised for low-latency applications
    /// (streaming, VoIP).  Higher correlation keeps jitter additive
    /// noise low.
    pub fn conservative() -> Self {
        Self {
            tcp_jitter_ms: 60,
            tls_jitter_ms: 40,
            data_jitter_ms: 25,
            correlation_factor: 0.80,
        }
    }

    /// Aggressive profile providing stronger obfuscation at the cost
    /// of higher latency variance.
    pub fn aggressive() -> Self {
        Self {
            tcp_jitter_ms: 200,
            tls_jitter_ms: 200,
            data_jitter_ms: 150,
            correlation_factor: 0.70,
        }
    }
}

// ---------------------------------------------------------------------------
// Timing stage
// ---------------------------------------------------------------------------

/// A point in the connection lifecycle where timing shaping is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimingStage {
    /// Before the TCP three‑way handshake (SYN).
    TcpConnect,
    /// Before the TLS ClientHello.
    TlsHandshake,
    /// Before the first byte of application data.
    FirstByte,
    /// Before an arbitrary data chunk.
    DataChunk,
}

// ---------------------------------------------------------------------------
// Deterministic RNG (splitmix64, mirrors `obfuscation.rs`)
// ---------------------------------------------------------------------------

fn splitmix64_step(state: &mut u64) -> u64 {
    let mut z = state.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

fn rand_f64() -> f64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut state = COUNTER.fetch_add(1, Ordering::Relaxed);
    (splitmix64_step(&mut state) as f64) / (u64::MAX as f64)
}

// ---------------------------------------------------------------------------
// Core shaping
// ---------------------------------------------------------------------------

/// Return the delay to inject before the given protocol stage.
///
/// When `correlation_factor` triggers, the same underlying random sample
/// is scaled to each layer's ceiling — transport and application delays
/// rise and fall together, which makes a proxy connection's cross‑layer
/// RTT profile resemble a direct connection.
pub fn shape_timing(config: &TimingShapingConfig, stage: TimingStage) -> Duration {
    let max_ms = match stage {
        TimingStage::TcpConnect => config.tcp_jitter_ms,
        TimingStage::TlsHandshake => config.tls_jitter_ms,
        TimingStage::FirstByte | TimingStage::DataChunk => config.data_jitter_ms,
    };
    if max_ms == 0 {
        return Duration::ZERO;
    }

    // Decide whether to use the correlated or independent path.
    let use_correlated = rand_f64() < config.correlation_factor;

    // In correlated mode we seal a fresh splitmix64 on the call stack
    // (NOT the atomic counter) so every *stage within one shaping
    // decision* gets the same sample.  This only works because
    // shape_timing is called sequentially per connection — if it were
    // ever called from concurrent contexts we would need thread‑local
    // storage.
    let normalised = if use_correlated {
        let mut sealed = 42u64;
        splitmix64_step(&mut sealed) as f64 / (u64::MAX as f64)
    } else {
        rand_f64()
    };

    let ms = (normalised * max_ms as f64) as u64;
    Duration::from_millis(ms)
}

// ---------------------------------------------------------------------------
// Simulation
// ---------------------------------------------------------------------------

/// The RTT that an on‑path observer would measure at each protocol layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedRtt {
    /// Estimated TCP‑layer RTT (ms) — SYN → SYN/ACK.
    pub tcp_rtt_ms: f64,
    /// Estimated TLS‑handshake RTT (ms) — ClientHello → ServerHello…
    pub tls_rtt_ms: f64,
    /// Estimated application‑layer RTT (ms) — request → response.
    pub app_rtt_ms: f64,
    /// `RTTdiff = app_rtt − tcp_rtt`.  Direct connections typically
    /// have RTTdiff < 15 ms (dominated by server processing).
    pub rtt_diff_ms: f64,
    /// `true` when this observation falls within the direct‑connection
    /// envelope (RTTdiff < 15 ms).
    pub classified_direct: bool,
}

impl SimulatedRtt {
    /// `true` when the cross‑layer gap is small enough to be consistent
    /// with a direct connection rather than a proxy tunnel.
    pub fn is_plausibly_direct(&self) -> bool {
        self.classified_direct
    }
}

/// Model parameters for a typical client–proxy–server topology:
///
/// | Segment             | One‑way latency |
/// |---------------------|-----------------|
/// | Client ↔ Proxy      | 30 ms (60 ms RTT) |
/// | Proxy ↔ Server      | 20 ms (40 ms RTT) |
/// | Server processing   | 5 ms              |
///
/// These are representative of, e.g., a user in East Asia connecting
/// through a proxy in Tokyo to a server in Sydney.
pub(crate) struct TopologyModel {
    client_proxy_rtt_ms: f64,
    proxy_server_rtt_ms: f64,
    server_process_ms: f64,
}

impl TopologyModel {
    fn default() -> Self {
        Self { client_proxy_rtt_ms: 60.0, proxy_server_rtt_ms: 40.0, server_process_ms: 5.0 }
    }
}

/// Simulate what a censor would observe when measuring cross‑layer RTTs
/// through a relay that applies the given timing‑shaping config.
///
/// The topology and baseline latencies are described in [`TopologyModel`].
pub fn simulate_round_trip(config: &TimingShapingConfig) -> SimulatedRtt {
    simulate_on_topology(config, &TopologyModel::default())
}

/// Like [`simulate_round_trip`] but allows overriding the topology model
/// for sensitivity analysis.
pub fn simulate_on_topology(config: &TimingShapingConfig, topo: &TopologyModel) -> SimulatedRtt {
    // ---- measure TCP RTT ------------------------------------------------
    // Observer sees: client↔proxy (2× one‑way) + injected jitter.
    let tcp_jitter = shape_timing(config, TimingStage::TcpConnect);
    let tcp_rtt = topo.client_proxy_rtt_ms + tcp_jitter.as_secs_f64() * 1000.0;

    // ---- measure TLS RTT ------------------------------------------------
    // Observer sees the full handshake path: client↔proxy + proxy↔server
    // + server-side TLS processing + jitter.
    let tls_jitter = shape_timing(config, TimingStage::TlsHandshake);
    let tls_rtt = topo.client_proxy_rtt_ms
        + topo.proxy_server_rtt_ms
        + topo.server_process_ms
        + tls_jitter.as_secs_f64() * 1000.0;

    // ---- measure app-layer RTT ------------------------------------------
    // Observer correlates a request–response pair: client↔proxy data +
    // proxy↔server data + server processing + jitter.
    let data_jitter = shape_timing(config, TimingStage::FirstByte);
    let app_rtt = topo.client_proxy_rtt_ms
        + topo.proxy_server_rtt_ms
        + topo.server_process_ms
        + data_jitter.as_secs_f64() * 1000.0;

    let rtt_diff = app_rtt - tcp_rtt;
    let classified_direct = rtt_diff < 15.0;

    SimulatedRtt {
        tcp_rtt_ms: (tcp_rtt * 100.0).round() / 100.0,
        tls_rtt_ms: (tls_rtt * 100.0).round() / 100.0,
        app_rtt_ms: (app_rtt * 100.0).round() / 100.0,
        rtt_diff_ms: (rtt_diff * 100.0).round() / 100.0,
        classified_direct,
    }
}

// ---------------------------------------------------------------------------
// Batch simulation helper
// ---------------------------------------------------------------------------

/// Run `n` independent round‑trip simulations and return summary statistics.
pub fn simulate_batch(config: &TimingShapingConfig, n: usize) -> BatchSummary {
    let mut diffs = Vec::with_capacity(n);
    let mut classified_direct = 0usize;

    for _ in 0..n {
        let sim = simulate_round_trip(config);
        diffs.push(sim.rtt_diff_ms);
        if sim.classified_direct {
            classified_direct += 1;
        }
    }

    diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min = diffs.first().copied().unwrap_or(0.0);
    let max = diffs.last().copied().unwrap_or(0.0);
    let mean = if n > 0 { diffs.iter().sum::<f64>() / n as f64 } else { 0.0 };
    let variance =
        if n > 1 { diffs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64 } else { 0.0 };
    let rtt_diff_histogram = histogram(&diffs, min, max, 12);

    BatchSummary {
        n,
        rtt_diff_mean_ms: (mean * 100.0).round() / 100.0,
        rtt_diff_stddev_ms: (variance.sqrt() * 100.0).round() / 100.0,
        rtt_diff_min_ms: (min * 100.0).round() / 100.0,
        rtt_diff_max_ms: (max * 100.0).round() / 100.0,
        rtt_diff_p50_ms: percentile(&diffs, 0.50),
        rtt_diff_p95_ms: percentile(&diffs, 0.95),
        direct_fraction: if n > 0 { classified_direct as f64 / n as f64 } else { 0.0 },
        rtt_diff_histogram,
    }
}

/// Summary statistics for a batch of simulated cross‑layer RTT measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub n: usize,
    pub rtt_diff_mean_ms: f64,
    pub rtt_diff_stddev_ms: f64,
    pub rtt_diff_min_ms: f64,
    pub rtt_diff_max_ms: f64,
    pub rtt_diff_p50_ms: f64,
    pub rtt_diff_p95_ms: f64,
    /// Fraction of simulations classified as direct (RTTdiff < 15 ms).
    pub direct_fraction: f64,
    pub rtt_diff_histogram: Vec<(f64, f64, usize)>,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64) * p).ceil() as usize - 1;
    (sorted[idx.min(sorted.len() - 1)] * 100.0).round() / 100.0
}

fn histogram(data: &[f64], lo: f64, hi: f64, buckets: usize) -> Vec<(f64, f64, usize)> {
    if data.is_empty() || buckets == 0 || (hi - lo).abs() < f64::EPSILON {
        return vec![(lo, hi, 0)];
    }
    let bin_w = (hi - lo) / buckets as f64;
    let mut bins = vec![0usize; buckets];
    for &v in data {
        let idx = ((v - lo) / bin_w).floor() as usize;
        bins[idx.min(buckets - 1)] += 1;
    }
    bins.into_iter()
        .enumerate()
        .map(|(i, count)| {
            let a = lo + i as f64 * bin_w;
            let b = a + bin_w;
            ((a * 100.0).round() / 100.0, (b * 100.0).round() / 100.0, count)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_timing_returns_valid_duration() {
        let cfg = TimingShapingConfig::default();
        for stage in &[TimingStage::TcpConnect, TimingStage::TlsHandshake, TimingStage::FirstByte, TimingStage::DataChunk] {
            let d = shape_timing(&cfg, *stage);
            assert!(d.as_millis() <= cfg.tcp_jitter_ms.max(cfg.tls_jitter_ms.max(cfg.data_jitter_ms)) as u128);
        }
    }

    #[test]
    fn test_shape_timing_zero_config_returns_zero() {
        let cfg = TimingShapingConfig {
            tcp_jitter_ms: 0,
            tls_jitter_ms: 0,
            data_jitter_ms: 0,
            correlation_factor: 0.0,
        };
        assert_eq!(shape_timing(&cfg, TimingStage::TcpConnect), Duration::ZERO);
        assert_eq!(shape_timing(&cfg, TimingStage::DataChunk), Duration::ZERO);
    }

    #[test]
    fn test_simulate_round_trip_runs() {
        let cfg = TimingShapingConfig::default();
        let sim = simulate_round_trip(&cfg);
        assert!(sim.tcp_rtt_ms > 0.0);
        assert!(sim.tls_rtt_ms > sim.tcp_rtt_ms);
        assert!(sim.app_rtt_ms >= sim.tcp_rtt_ms);
    }

    #[test]
    fn test_simulate_batch_returns_summary() {
        let cfg = TimingShapingConfig::default();
        let s = simulate_batch(&cfg, 100);
        assert_eq!(s.n, 100);
        assert!(s.rtt_diff_mean_ms >= 0.0);
        assert!(s.direct_fraction >= 0.0);
        assert!(!s.rtt_diff_histogram.is_empty());
    }

    #[test]
    fn test_configs_differ() {
        let d = TimingShapingConfig::default();
        let c = TimingShapingConfig::conservative();
        let a = TimingShapingConfig::aggressive();
        assert!(d.tcp_jitter_ms > c.tcp_jitter_ms);
        assert!(a.tcp_jitter_ms >= d.tcp_jitter_ms);
    }

    #[test]
    fn test_simulated_rtt_debug_and_serialize() {
        let sim = simulate_round_trip(&TimingShapingConfig::default());
        let debug = format!("{sim:?}");
        assert!(debug.contains("tcp_rtt_ms"));
    }

    #[test]
    fn test_histogram_empty() {
        let h = histogram(&[], 0.0, 100.0, 10);
        assert_eq!(h.len(), 1);
    }
}
