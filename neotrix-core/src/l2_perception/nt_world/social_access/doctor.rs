//! Doctor — system-wide channel health diagnostics
//!
//! Modeled after Agent-Reach's doctor command:
//! - Checks all channels and their backends
//! - Reports active_backend for each channel
//! - Identifies missing/errored backends
//! - Provides actionable hints

use std::fmt;

use super::channel::{BackendStatus, ChannelRegistry};

/// Doctor report for a single channel
#[derive(Debug, Clone)]
pub struct ChannelReport {
    pub channel_name: String,
    pub platform: String,
    pub tier: u8,
    pub active_backend: Option<String>,
    pub backend_results: Vec<BackendReport>,
}

/// Doctor report for a single backend
#[derive(Debug, Clone)]
pub struct BackendReport {
    pub name: String,
    pub status: BackendStatus,
    pub latency_ms: u64,
    pub hint: Option<String>,
}

/// Full doctor report
#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub channels: Vec<ChannelReport>,
    pub summary: DoctorSummary,
}

/// Summary statistics
#[derive(Debug, Clone)]
pub struct DoctorSummary {
    pub total_channels: usize,
    pub healthy_channels: usize,
    pub degraded_channels: usize,
    pub failed_channels: usize,
    pub total_backends: usize,
    pub healthy_backends: usize,
}

impl fmt::Display for DoctorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "═══ NeoTrix Channel Health Report ═══")?;
        writeln!(f)?;

        for channel in &self.channels {
            let status_icon = if channel.active_backend.is_some() {
                "✓"
            } else {
                "✗"
            };

            writeln!(
                f,
                "{} {} (tier={})",
                status_icon,
                channel.channel_name,
                channel.tier
            )?;

            if let Some(ref backend) = channel.active_backend {
                writeln!(f, "  active: {}", backend)?;
            } else {
                writeln!(f, "  active: none")?;
            }

            for report in &channel.backend_results {
                let icon = match &report.status {
                    BackendStatus::Ok => "  ✓",
                    BackendStatus::Warn(_) => "  ⚠",
                    BackendStatus::Error(_) => "  ✗",
                    BackendStatus::Missing => "  ·",
                    BackendStatus::Timeout => "  ⏱",
                };

                write!(f, "{} {}", icon, report.name)?;
                if report.latency_ms > 0 {
                    write!(f, " ({}ms)", report.latency_ms)?;
                }
                writeln!(f)?;

                if let Some(ref hint) = report.hint {
                    writeln!(f, "    hint: {}", hint)?;
                }
            }

            writeln!(f)?;
        }

        writeln!(f, "═══ Summary ═══")?;
        writeln!(
            f,
            "Channels: {}/{} healthy",
            self.summary.healthy_channels, self.summary.total_channels
        )?;
        writeln!(
            f,
            "Backends: {}/{} healthy",
            self.summary.healthy_backends, self.summary.total_backends
        )?;

        Ok(())
    }
}

impl DoctorReport {
    /// Generate JSON summary
    pub fn to_json(&self) -> serde_json::Value {
        let channels: Vec<serde_json::Value> = self.channels.iter().map(|c| {
            let backends: Vec<serde_json::Value> = c.backend_results.iter().map(|b| {
                serde_json::json!({
                    "name": b.name,
                    "status": format!("{:?}", b.status),
                    "latency_ms": b.latency_ms,
                    "hint": b.hint,
                })
            }).collect();

            serde_json::json!({
                "channel": c.channel_name,
                "platform": c.platform,
                "tier": c.tier,
                "active_backend": c.active_backend,
                "backends": backends,
            })
        }).collect();

        serde_json::json!({
            "channels": channels,
            "summary": {
                "total_channels": self.summary.total_channels,
                "healthy_channels": self.summary.healthy_channels,
                "degraded_channels": self.summary.degraded_channels,
                "failed_channels": self.summary.failed_channels,
                "total_backends": self.summary.total_backends,
                "healthy_backends": self.summary.healthy_backends,
            }
        })
    }
}

/// Run doctor diagnostics on the channel registry
pub fn run_doctor(registry: &ChannelRegistry) -> DoctorReport {
    let mut channels = Vec::new();
    let mut total_backends = 0;
    let mut healthy_backends = 0;
    let mut healthy_channels = 0;
    let mut degraded_channels = 0;

    for channel in registry.all_channels() {
        let mut backend_reports = Vec::new();
        let mut channel_healthy = false;

        for backend in &channel.backends {
            let result = super::channel::probe_backend(backend);
            if result.status.is_healthy() {
                healthy_backends += 1;
                if matches!(result.status, BackendStatus::Ok) {
                    channel_healthy = true;
                }
            }
            total_backends += 1;

            backend_reports.push(BackendReport {
                name: backend.name.clone(),
                status: result.status,
                latency_ms: result.latency_ms,
                hint: result.hint,
            });
        }

        if channel_healthy {
            healthy_channels += 1;
        } else if channel.active_backend.is_some() {
            degraded_channels += 1;
        }

        channels.push(ChannelReport {
            channel_name: channel.name.clone(),
            platform: format!("{:?}", channel.platform),
            tier: channel.tier,
            active_backend: channel.active_backend.clone(),
            backend_results: backend_reports,
        });
    }

    let total_channels = channels.len();

    DoctorReport {
        channels,
        summary: DoctorSummary {
            total_channels,
            healthy_channels,
            degraded_channels,
            failed_channels: total_channels - healthy_channels - degraded_channels,
            total_backends,
            healthy_backends,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_report_format() {
        let mut registry = ChannelRegistry::new();
        registry.register(super::super::channel::Channel::new(
            "test",
            super::super::traits::SocialPlatform::Twitter,
            vec![super::super::channel::Backend::new("echo", "echo")],
        ));

        let report = run_doctor(&registry);
        let output = format!("{}", report);
        assert!(output.contains("NeoTrix Channel Health Report"));
        assert!(output.contains("echo"));
    }

    #[test]
    fn test_doctor_report_json() {
        let mut registry = ChannelRegistry::new();
        registry.register(super::super::channel::Channel::new(
            "test",
            super::super::traits::SocialPlatform::Twitter,
            vec![super::super::channel::Backend::new("echo", "echo")],
        ));

        let report = run_doctor(&registry);
        let json = report.to_json();
        assert!(json["channels"].is_array());
        assert!(json["summary"]["total_channels"].as_u64().unwrap() == 1);
    }
}
