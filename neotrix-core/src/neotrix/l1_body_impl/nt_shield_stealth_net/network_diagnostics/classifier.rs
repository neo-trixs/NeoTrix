use std::time::Duration;

use super::types::{
    ApiHealth, ClassifiedRootCause, ConnectionFailureRootCause, NetworkEnvironment,
};

pub struct RootCauseClassifier;

impl RootCauseClassifier {
    pub fn classify_deterministic(
        health: &ApiHealth,
        env: &NetworkEnvironment,
    ) -> Option<ClassifiedRootCause> {
        match health {
            ApiHealth::DnsFailure { reason } => {
                if env.fake_ip_dns.is_some() {
                    return Some(ClassifiedRootCause {
                        cause: ConnectionFailureRootCause::FakeIpDns,
                        confidence: 0.95,
                        evidence: vec!["DNS failure + 198.18.x.x DNS server".into()],
                    });
                }
                Some(ClassifiedRootCause {
                    cause: ConnectionFailureRootCause::DnsResolutionFailed,
                    confidence: 0.90,
                    evidence: vec![format!("DNS error: {}", reason)],
                })
            }
            ApiHealth::TlsFailure { reason } => {
                Some(ClassifiedRootCause {
                    cause: ConnectionFailureRootCause::TlsHandshakeFailed,
                    confidence: 0.95,
                    evidence: vec![format!("TLS error: {}", reason)],
                })
            }
            ApiHealth::HttpError { status_code, .. } if *status_code == 503 => {
                Some(ClassifiedRootCause {
                    cause: ConnectionFailureRootCause::HttpServiceUnavailable,
                    confidence: 0.98,
                    evidence: vec!["HTTP 503 Service Unavailable".into()],
                })
            }
            ApiHealth::Unreachable { reason } => {
                let r = reason.to_lowercase();
                if r.contains("refused") {
                    return Some(ClassifiedRootCause {
                        cause: ConnectionFailureRootCause::ConnectionRefused,
                        confidence: 0.95,
                        evidence: vec![reason.clone()],
                    });
                }
                if r.contains("reset") {
                    return Some(ClassifiedRootCause {
                        cause: ConnectionFailureRootCause::ConnectionReset,
                        confidence: 0.90,
                        evidence: vec![reason.clone()],
                    });
                }
                if r.contains("timeout") {
                    if env.fake_ip_dns.is_some() {
                        return Some(ClassifiedRootCause {
                            cause: ConnectionFailureRootCause::VpnRoutingIssue,
                            confidence: 0.75,
                            evidence: vec![
                                "timeout + VPN TUN active → VPN routing issue".into(),
                            ],
                        });
                    }
                    return Some(ClassifiedRootCause {
                        cause: ConnectionFailureRootCause::ConnectionTimeout,
                        confidence: 0.85,
                        evidence: vec![reason.clone()],
                    });
                }
                if r.contains("dns") || r.contains("name or service") {
                    return Some(ClassifiedRootCause {
                        cause: ConnectionFailureRootCause::DnsResolutionFailed,
                        confidence: 0.90,
                        evidence: vec![reason.clone()],
                    });
                }
                None
            }
            ApiHealth::Degraded { ttfb, .. } if *ttfb > Duration::from_secs(10) => {
                Some(ClassifiedRootCause {
                    cause: ConnectionFailureRootCause::ProviderTooSlow { ttfb: *ttfb },
                    confidence: 0.92,
                    evidence: vec![format!("TTFB {:.1}s > 10s threshold", ttfb.as_secs_f64())],
                })
            }
            _ => None,
        }
    }

    pub fn classify_probabilistic(
        health: &ApiHealth,
        env: &NetworkEnvironment,
        latency_z: f64,
        has_tun: bool,
    ) -> Vec<ClassifiedRootCause> {
        let mut candidates = Vec::new();

        if let Some(det) = Self::classify_deterministic(health, env) {
            candidates.push(det);
        }

        match health {
            ApiHealth::Healthy { latency, .. } => {
                let l = latency.as_secs_f64();
                if l > 5.0 && latency_z > 3.0 {
                    candidates.push(ClassifiedRootCause {
                        cause: ConnectionFailureRootCause::ProviderTooSlow { ttfb: *latency },
                        confidence: 0.60,
                        evidence: vec![format!("latency {:.1}s, z-score {:.1}", l, latency_z)],
                    });
                }
                if !candidates.is_empty() && has_tun {
                    if let Some(ref ip) = env.fake_ip_dns {
                        candidates.push(ClassifiedRootCause {
                            cause: ConnectionFailureRootCause::FakeIpDns,
                            confidence: 0.30,
                            evidence: vec![format!(
                                "VPN TUN active (DNS {}) but no failure",
                                ip
                            )],
                        });
                    }
                }
            }
            ApiHealth::Degraded { ttfb, .. } => {
                let t = ttfb.as_secs_f64();
                if t > 5.0 && t <= 10.0 {
                    candidates.push(ClassifiedRootCause {
                        cause: ConnectionFailureRootCause::ProviderTooSlow { ttfb: *ttfb },
                        confidence: 0.70,
                        evidence: vec![format!("TTFB {:.1}s — approaching 10s threshold", t)],
                    });
                }
            }
            _ => {}
        }

        candidates
    }
}

pub fn recommendation_for(cause: &ConnectionFailureRootCause) -> Option<&'static str> {
    match cause {
        ConnectionFailureRootCause::FakeIpDns => {
            Some("VPN TUN cold-start: run scripts/tor-warmup.sh first")
        }
        ConnectionFailureRootCause::HttpServiceUnavailable => {
            Some("Provider HTTP 503 — switch to opencode/deepseek-v4-flash-free")
        }
        ConnectionFailureRootCause::ProviderTooSlow { .. } => {
            Some("Provider >10s TTFB — set opencode/deepseek-v4-flash-free as default")
        }
        ConnectionFailureRootCause::DnsResolutionFailed => {
            Some("DNS failure — check VPN or run scripts/tor-warmup.sh")
        }
        ConnectionFailureRootCause::ConnectionRefused => {
            Some("Connection refused — provider down or firewall")
        }
        ConnectionFailureRootCause::ConnectionTimeout => {
            Some("Connection timeout — check VPN/provider routing")
        }
        ConnectionFailureRootCause::TlsHandshakeFailed => {
            Some("TLS failure — possible MITM or cert issue with VPN")
        }
        ConnectionFailureRootCause::ConnectionReset => {
            Some("Connection reset — VPN unstable or rate limit")
        }
        _ => None,
    }
}
