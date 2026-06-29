use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::node::ProtocolKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectStats {
    pub protocol: ProtocolKind,
    pub server: String,
    pub port: u16,
    pub target: String,
    pub success: bool,
    pub latency_ms: u64,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub error: Option<String>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolHealth {
    pub protocol: ProtocolKind,
    pub total_attempts: u64,
    pub total_success: u64,
    pub total_failures: u64,
    pub avg_latency_ms: f64,
    pub last_checked_ms: u64,
}

impl ProtocolHealth {
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 { 0.0 }
        else { self.total_success as f64 / self.total_attempts as f64 }
    }
}

#[derive(Debug, Clone)]
pub struct TelemetryCollector {
    pub stats: VecDeque<ConnectStats>,
    pub per_protocol: std::collections::HashMap<ProtocolKind, ProtocolHealth>,
    max_entries: usize,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            stats: VecDeque::new(),
            per_protocol: std::collections::HashMap::new(),
            max_entries: 10000,
        }
    }

    pub fn record(&mut self, stat: ConnectStats) {
        let proto = stat.protocol;
        let entry = self.per_protocol.entry(proto).or_insert_with(|| ProtocolHealth {
            protocol: proto,
            total_attempts: 0,
            total_success: 0,
            total_failures: 0,
            avg_latency_ms: 0.0,
            last_checked_ms: stat.timestamp_ms,
        });
        entry.total_attempts += 1;
        if stat.success { entry.total_success += 1; } else { entry.total_failures += 1; }
        let n = entry.total_attempts as f64;
        entry.avg_latency_ms = entry.avg_latency_ms * ((n - 1.0) / n) + stat.latency_ms as f64 / n;
        entry.last_checked_ms = stat.timestamp_ms;

        self.stats.push_back(stat);
        if self.stats.len() > self.max_entries {
            self.stats.pop_front();
        }
    }

    pub fn protocol_health(&self, protocol: ProtocolKind) -> ProtocolHealth {
        self.per_protocol.get(&protocol).cloned().unwrap_or(ProtocolHealth {
            protocol,
            total_attempts: 0,
            total_success: 0,
            total_failures: 0,
            avg_latency_ms: 0.0,
            last_checked_ms: 0,
        })
    }

    pub fn summary(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("telemetry: {} total connections", self.stats.len()));
        let mut protocols: Vec<_> = self.per_protocol.iter().collect();
        protocols.sort_by_key(|b| std::cmp::Reverse(b.1.total_attempts));
        for (proto, health) in &protocols {
            lines.push(format!(
                "  {}: {}/{} success ({:.1}%), avg {:.0}ms",
                proto.name(),
                health.total_success,
                health.total_attempts,
                health.success_rate() * 100.0,
                health.avg_latency_ms,
            ));
        }
        lines.join("\n")
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
