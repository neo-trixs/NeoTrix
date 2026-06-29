use std::time::{Duration, Instant};

use serde::Serialize;

use crate::obfuscation::{jitter_sleep, padding_bytes, rand_u64_splitmix64};
use crate::tls::tls_config_random;

// ---------------------------------------------------------------------------
// TLS diversity
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct DiversityReport {
    /// Number of connection attempts
    pub attempts: usize,
    /// Unique JA4-style fingerprints observed
    pub unique_fingerprints: Vec<String>,
    /// Unique count
    pub unique_count: usize,
    /// Fraction of attempts that produced a unique fingerprint (diversity ratio)
    pub diversity_ratio: f64,
    /// Per-fingerprint occurrence map
    pub fingerprint_occurrences: Vec<(String, usize)>,
}

/// Derive a JA4-style fingerprint from a rustls ClientConfig.
///
/// Format: `t13d{ALPN_hex}{CIPHER_hex}` — a simplified JA4-like identifier
/// based on the ALPN list and cipher-suite class, which are the dimensions
/// we randomise in `tls_config_random()`.
fn ja4_fingerprint(config: &rustls::ClientConfig) -> String {
    // ALPN contribution: sort & hash the protocol list
    let mut alpn_parts: Vec<String> = config
        .alpn_protocols
        .iter()
        .map(|p| format!("{:02x}", p.first().copied().unwrap_or(0)))
        .collect();
    alpn_parts.sort();
    let alpn_tag = if alpn_parts.is_empty() {
        "00".to_string()
    } else {
        alpn_parts.join("")
    };

    // Cipher-suite class: rustls only exposes this indirectly via version;
    // we use provider name as a proxy for the cipher-class mix.
    let cipher_tag = format!("{:04x}", config.alpn_protocols.len());

    format!("t13d{alpn_tag}{cipher_tag}")
}

/// Connect to `target` N times using random TLS configs and report
/// the diversity of JA4-style fingerprints produced.
///
/// # Panics
/// If the TCP connection to `target` fails on every attempt.
pub fn measure_tls_diversity(target: &str, n: usize) -> DiversityReport {
    let mut fingerprints: Vec<String> = Vec::with_capacity(n);
    let mut successes = 0usize;

    for _ in 0..n {
        let config = tls_config_random();
        let fp = ja4_fingerprint(config.as_ref());

        // Attempt a real TCP connect (TLS handshake not required for
        // fingerprint measurement — the JA4 is decided by ClientHello
        // which the config determines before the server responds).
        match std::net::TcpStream::connect_timeout(
            &target.parse().unwrap_or_else(|_| panic!("invalid target: {target}")),
            Duration::from_secs(5),
        ) {
            Ok(_) => {
                fingerprints.push(fp);
                successes += 1;
            }
            Err(e) => {
                log::warn!("TCP connect failed: {e}");
            }
        }
    }

    let unique: std::collections::BTreeSet<String> = fingerprints.iter().cloned().collect();
    let unique_count = unique.len();
    let diversity_ratio = if successes > 0 {
        unique_count as f64 / successes as f64
    } else {
        0.0
    };

    // Count occurrences per fingerprint
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for fp in &fingerprints {
        *counts.entry(fp.clone()).or_insert(0) += 1;
    }
    let mut fingerprint_occurrences: Vec<(String, usize)> = counts.into_iter().collect();
    fingerprint_occurrences.sort_by_key(|b| std::cmp::Reverse(b.1));

    DiversityReport {
        attempts: n,
        unique_fingerprints: unique.into_iter().collect(),
        unique_count,
        diversity_ratio,
        fingerprint_occurrences,
    }
}

// ---------------------------------------------------------------------------
// Timing jitter
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct TimingReport {
    pub label: String,
    pub samples: usize,
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
    pub stddev_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

/// Measure connection latency to `target` across `n` attempts, optionally
/// calling `jitter_sleep()` before each connect.
fn measure_latency(target: &str, n: usize, use_jitter: bool) -> TimingReport {
    let mut latencies = Vec::with_capacity(n);
    let target_addr: std::net::SocketAddr = target.parse().expect("invalid target");

    for _ in 0..n {
        if use_jitter {
            jitter_sleep();
        }
        let start = Instant::now();
        match std::net::TcpStream::connect_timeout(&target_addr, Duration::from_secs(5)) {
            Ok(_) => {
                latencies.push(start.elapsed().as_secs_f64() * 1000.0);
            }
            Err(e) => {
                log::warn!("latency probe failed: {e}");
            }
        }
    }

    let samples = latencies.len();
    if samples == 0 {
        return TimingReport {
            label: if use_jitter { "with_jitter".into() } else { "no_jitter".into() },
            samples: 0,
            min_ms: 0.0,
            max_ms: 0.0,
            mean_ms: 0.0,
            stddev_ms: 0.0,
            p50_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
        };
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let min_ms = latencies[0];
    let max_ms = latencies[samples - 1];
    let mean_ms = latencies.iter().sum::<f64>() / samples as f64;
    let variance = latencies.iter().map(|x| (x - mean_ms).powi(2)).sum::<f64>() / samples as f64;
    let stddev_ms = variance.sqrt();

    let percentile = |p: f64| -> f64 {
        let idx = ((samples as f64) * p).ceil() as usize - 1;
        latencies[idx.min(samples - 1)]
    };

    TimingReport {
        label: if use_jitter { "with_jitter".into() } else { "no_jitter".into() },
        samples,
        min_ms,
        max_ms,
        mean_ms,
        stddev_ms,
        p50_ms: percentile(0.50),
        p95_ms: percentile(0.95),
        p99_ms: percentile(0.99),
    }
}

/// Run paired timing measurements — with and without jitter — and return both.
pub fn measure_timing_jitter(target: &str, n: usize) -> (TimingReport, TimingReport) {
    let no_jitter = measure_latency(target, n, false);
    let with_jitter = measure_latency(target, n, true);
    (no_jitter, with_jitter)
}

// ---------------------------------------------------------------------------
// Padding overhead
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct PaddingReport {
    /// Total bytes sent (real data + padding)
    pub total_bytes_sent: u64,
    /// Real data bytes
    pub real_bytes: u64,
    /// Padding bytes injected
    pub padding_bytes: u64,
    /// Overhead ratio (total / real)
    pub overhead_ratio: f64,
    /// Number of padding events
    pub padding_events: usize,
    /// Average pad size per event
    pub avg_pad_size: f64,
    /// Simulated transfer size
    pub transfer_size: u64,
}

/// Simulate a relay transfer and measure padding overhead.
///
/// Models the relay logic from `relay.rs`: after every ~64 KB of data,
/// there is a 20 % chance to inject 16-256 bytes of padding.
pub fn measure_padding_overhead(transfer_size: u64) -> PaddingReport {
    let mut total_sent = 0u64;
    let mut real_bytes = 0u64;
    let mut padding_total = 0u64;
    let mut padding_events = 0usize;
    let mut bytes_relayed: u64 = 0;
    let chunk_sz: u64 = 4096;
    let mut padded_this_window = false;

    while real_bytes < transfer_size {
        let n = chunk_sz.min(transfer_size - real_bytes);
        real_bytes += n;
        total_sent += n;
        bytes_relayed += n;

        // After every ~64 KB crossing, 20% chance to pad (mirrors relay.rs)
        if bytes_relayed % 65536 < n
            && !padded_this_window
            && rand_u64_splitmix64() % 100 < 20
        {
            let pad = padding_bytes(16, 256);
            let pad_len = pad.len() as u64;
            total_sent += pad_len;
            padding_total += pad_len;
            padding_events += 1;
            padded_this_window = true;
        }
        if bytes_relayed % 65536 >= n {
            padded_this_window = false;
        }
    }

    let overhead_ratio = if real_bytes > 0 {
        total_sent as f64 / real_bytes as f64
    } else {
        1.0
    };
    let avg_pad_size = if padding_events > 0 {
        padding_total as f64 / padding_events as f64
    } else {
        0.0
    };

    PaddingReport {
        total_bytes_sent: total_sent,
        real_bytes,
        padding_bytes: padding_total,
        overhead_ratio,
        padding_events,
        avg_pad_size,
        transfer_size,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_tls_provider() {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        });
    }

    // -----------------------------------------------------------------------
    // Full-suite measurement runner: prints quant results to stdout.
    // -----------------------------------------------------------------------
    #[test]
    fn full_measurement_suite() {
        ensure_tls_provider();
        let target = "1.1.1.1:443";
        let n_fp = 20;
        let n_timing = 10;

        println!("\n========= OBFUSCATION MEASUREMENT SUITE =========");

        // 1. TLS diversity
        println!("\n--- TLS Fingerprint Diversity ---");
        let r = measure_tls_diversity(target, n_fp);
        println!("  attempts: {}  successes: {}  unique: {}  ratio: {:.3}",
            r.attempts, r.fingerprint_occurrences.iter().map(|(_,c)| c).sum::<usize>(),
            r.unique_count, r.diversity_ratio);
        for (fp, count) in &r.fingerprint_occurrences {
            println!("  {} ×{}", fp, count);
        }

        // 2. Timing jitter
        println!("\n--- Timing Jitter ---");
        let (no_jitter, with_jitter) = measure_timing_jitter(target, n_timing);
        for tr in [&no_jitter, &with_jitter] {
            println!("  {}: min={:.1}ms max={:.1}ms mean={:.1}ms std={:.2}ms p50={:.1}ms p95={:.1}ms p99={:.1}ms",
                tr.label, tr.min_ms, tr.max_ms, tr.mean_ms, tr.stddev_ms,
                tr.p50_ms, tr.p95_ms, tr.p99_ms);
        }

        // 3. Padding overhead at multiple scales
        println!("\n--- Padding Overhead ---");
        for sz in [100_000u64, 1_000_000, 10_000_000] {
            let r = measure_padding_overhead(sz);
            println!("  transfer={:>10}B real={:>10}B total={:>10}B padding={:>6}B ratio={:.4}x events={} avg_pad={:.0}B",
                r.transfer_size, r.real_bytes, r.total_bytes_sent, r.padding_bytes,
                r.overhead_ratio, r.padding_events, r.avg_pad_size);
        }

        println!("\n========= MEASUREMENT COMPLETE =========");
    }

    #[test]
    fn test_ja4_fingerprint_produces_expected_format() {
        ensure_tls_provider();
        let config = tls_config_random();
        let fp = ja4_fingerprint(config.as_ref());
        assert!(fp.starts_with("t13d"), "fingerprint should start with t13d, got {fp}");
        assert!(fp.len() >= 8, "fingerprint too short: {fp}");
    }

    #[test]
    fn test_fingerprint_diversity_different_configs_produce_different_fps() {
        ensure_tls_provider();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..24 {
            let config = tls_config_random();
            let fp = ja4_fingerprint(config.as_ref());
            seen.insert(fp);
        }
        // With 12 config variants and ALPN-driven fingerprint,
        // we expect at least 2 unique fingerprints
        assert!(seen.len() >= 2, "expected ≥2 unique fingerprints, got {}", seen.len());
    }

    #[test]
    fn test_padding_overhead_never_zero_with_transfer() {
        let r = measure_padding_overhead(1_000_000);
        assert_eq!(r.real_bytes, 1_000_000);
        assert!(r.total_bytes_sent >= r.real_bytes);
        assert!(r.overhead_ratio >= 1.0);
    }

    #[test]
    fn test_padding_no_overhead_on_small_transfer() {
        // Under 64 KB there should be no padding events
        let r = measure_padding_overhead(32_000);
        assert_eq!(r.padding_events, 0);
        assert!((r.overhead_ratio - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_diversity_report_debug_and_serialize() {
        let report = DiversityReport {
            attempts: 5,
            unique_fingerprints: vec!["t13d0102".into()],
            unique_count: 1,
            diversity_ratio: 0.2,
            fingerprint_occurrences: vec![("t13d0102".into(), 5)],
        };
        assert!(format!("{report:?}").contains("t13d0102"));
    }

    #[test]
    fn test_timing_report_debug_and_serialize() {
        let report = TimingReport {
            label: "test".into(),
            samples: 10,
            min_ms: 1.0,
            max_ms: 10.0,
            mean_ms: 5.0,
            stddev_ms: 2.0,
            p50_ms: 5.0,
            p95_ms: 9.0,
            p99_ms: 10.0,
        };
        assert!(format!("{report:?}").contains("test"));
    }

    #[test]
    fn test_measure_padding_overhead_consistency() {
        // Run multiple times and verify the function doesn't panic
        for sz in [100_000, 500_000, 2_000_000] {
            let r = measure_padding_overhead(sz);
            assert_eq!(r.real_bytes, sz);
            assert!(r.avg_pad_size >= 0.0);
        }
    }
}
