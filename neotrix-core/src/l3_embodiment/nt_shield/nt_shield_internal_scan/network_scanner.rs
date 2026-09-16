use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTarget {
    pub host: String,
    pub ports: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortResult {
    pub port: u16,
    pub state: PortState,
    pub service: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortState {
    Open,
    Closed,
    Filtered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub target: String,
    pub results: Vec<PortResult>,
    pub open_count: usize,
}

pub struct NetworkScanner {
    timeout_ms: u64,
}

impl NetworkScanner {
    pub fn new() -> Self {
        Self { timeout_ms: 3000 }
    }

    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    pub fn scan_target(&self, target: &ScanTarget) -> ScanReport {
        let results: Vec<PortResult> = target.ports.iter().map(|&port| PortResult {
            port,
            state: PortState::Closed,
            service: None,
        }).collect();
        let open_count = results.iter().filter(|r| matches!(r.state, PortState::Open)).count();
        ScanReport {
            target: target.host.clone(),
            results,
            open_count,
        }
    }
}

impl Default for NetworkScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_empty_ports() {
        let scanner = NetworkScanner::new();
        let target = ScanTarget { host: "127.0.0.1".into(), ports: vec![] };
        let report = scanner.scan_target(&target);
        assert_eq!(report.open_count, 0);
    }
}
