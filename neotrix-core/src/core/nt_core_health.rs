// FUTURE - not yet wired
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 依赖健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Available,
    Unavailable,
    Degraded { latency_ms: u64 },
}

impl HealthStatus {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Available | Self::Degraded { .. })
    }
}

/// 健康检查探针 trait
pub trait HealthProbe: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> HealthStatus;
    fn is_required(&self) -> bool {
        false
    }
    fn category(&self) -> &str {
        "tool"
    }
}

/// CLI 工具探针 — 通过 `which` 检查可执行文件是否存在
pub struct ToolProbe {
    name: String,
    binary: String,
    required: bool,
    category: String,
    no_version_flag: Option<String>,
}

impl ToolProbe {
    pub fn new(name: &str, binary: &str) -> Self {
        Self {
            name: name.to_string(),
            binary: binary.to_string(),
            required: false,
            category: "tool".to_string(),
            no_version_flag: None,
        }
    }

    pub fn no_version_flag(mut self) -> Self {
        self.no_version_flag = Some(String::new());
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn category(mut self, cat: &str) -> Self {
        self.category = cat.to_string();
        self
    }
}

impl HealthProbe for ToolProbe {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> HealthStatus {
        let start = Instant::now();
        match std::process::Command::new("which")
            .arg(&self.binary)
            .output()
        {
            Ok(output) if output.status.success() => {
                let elapsed = start.elapsed();
                if elapsed > Duration::from_millis(500) {
                    HealthStatus::Degraded {
                        latency_ms: elapsed.as_millis() as u64,
                    }
                } else {
                    HealthStatus::Available
                }
            }
            _ => HealthStatus::Unavailable,
        }
    }

    fn is_required(&self) -> bool {
        self.required
    }
    fn category(&self) -> &str {
        &self.category
    }
}

/// 自定义命令探针
pub struct CommandProbe {
    name: String,
    command: String,
    args: Vec<String>,
    required: bool,
    category: String,
    max_latency_ms: u64,
}

impl CommandProbe {
    pub fn new(name: &str, command: &str, args: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            required: false,
            category: "custom".to_string(),
            max_latency_ms: 2000,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn category(mut self, cat: &str) -> Self {
        self.category = cat.to_string();
        self
    }

    pub fn max_latency(mut self, ms: u64) -> Self {
        self.max_latency_ms = ms;
        self
    }
}

impl HealthProbe for CommandProbe {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> HealthStatus {
        let start = Instant::now();
        match std::process::Command::new(&self.command)
            .args(&self.args)
            .output()
        {
            Ok(output) if output.status.success() => {
                let elapsed = start.elapsed();
                if elapsed > Duration::from_millis(self.max_latency_ms) {
                    HealthStatus::Degraded {
                        latency_ms: elapsed.as_millis() as u64,
                    }
                } else {
                    HealthStatus::Available
                }
            }
            _ => HealthStatus::Unavailable,
        }
    }

    fn is_required(&self) -> bool {
        self.required
    }
    fn category(&self) -> &str {
        &self.category
    }
}

/// 健康检查报告
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthReport {
    pub timestamp: String,
    pub total: usize,
    pub available: usize,
    pub degraded: usize,
    pub unavailable: usize,
    pub required_failures: Vec<String>,
    pub probes: Vec<ProbeResult>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProbeResult {
    pub name: String,
    pub status: HealthStatus,
    pub category: String,
    pub required: bool,
}

/// 依赖注册表 — 集中管理所有外部工具/服务健康检查
pub struct DependencyRegistry {
    probes: Vec<Box<dyn HealthProbe>>,
    cache: std::sync::Mutex<HashMap<String, (HealthStatus, Instant)>>,
    cache_ttl: Duration,
}

impl DependencyRegistry {
    pub fn new() -> Self {
        Self {
            probes: Vec::new(),
            cache: std::sync::Mutex::new(HashMap::new()),
            cache_ttl: Duration::from_secs(60),
        }
    }

    pub fn register(&mut self, probe: Box<dyn HealthProbe>) {
        self.probes.push(probe);
    }

    pub fn cache_ttl(mut self, d: Duration) -> Self {
        self.cache_ttl = d;
        self
    }

    pub fn check_all(&self) -> HealthReport {
        let mut report = HealthReport {
            timestamp: format!("{:?}", std::time::SystemTime::now()),
            total: self.probes.len(),
            available: 0,
            degraded: 0,
            unavailable: 0,
            required_failures: Vec::new(),
            probes: Vec::new(),
        };

        for probe in &self.probes {
            let name = probe.name().to_string();
            let cached = self.cache.lock().ok().and_then(|c| c.get(&name).copied());
            let status = if let Some((status, time)) = cached {
                if time.elapsed() < self.cache_ttl {
                    status
                } else {
                    let s = probe.check();
                    if let Ok(mut c) = self.cache.lock() {
                        c.insert(name.clone(), (s, Instant::now()));
                    }
                    s
                }
            } else {
                let s = probe.check();
                if let Ok(mut c) = self.cache.lock() {
                    c.insert(name.clone(), (s, Instant::now()));
                }
                s
            };

            let result = ProbeResult {
                name: name.clone(),
                status,
                category: probe.category().to_string(),
                required: probe.is_required(),
            };

            match status {
                HealthStatus::Available => report.available += 1,
                HealthStatus::Degraded { .. } => report.degraded += 1,
                HealthStatus::Unavailable => {
                    report.unavailable += 1;
                    if probe.is_required() {
                        report.required_failures.push(name.clone());
                    }
                }
            }

            report.probes.push(result);
        }

        report
    }

    pub fn check_one(&self, name: &str) -> Option<HealthStatus> {
        self.probes
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.check())
    }

    pub fn is_healthy(&self) -> bool {
        let report = self.check_all();
        report.required_failures.is_empty()
    }

    /// 创建默认探针集合
    pub fn default() -> Self {
        let mut reg = Self::new();
        reg.register(Box::new(ToolProbe::new("ffmpeg", "ffmpeg")));
        reg.register(Box::new(ToolProbe::new("git", "git")));
        reg.register(Box::new(ToolProbe::new("which", "which")));
        reg.register(Box::new(ToolProbe::new("docker", "docker")));
        reg
    }
}

impl Default for DependencyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Consciousness theory metrics dashboard.
///
/// Aggregates four theory-of-consciousness metrics into a single snapshot:
/// - **IIT Φ** — Integrated Information Theory measure of integration (0.0–1.0)
/// - **GNW** — Global Neuronal Workspace broadcast coverage & slot utilization
/// - **DRT** — Dynamic Recursive Theory recursion depth in E8 reasoning
/// - **Workspace Saturation** — active proposals vs max capacity
///
/// A weighted composite score combines all four dimensions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsciousnessDashboard {
    /// When this snapshot was taken
    pub timestamp: String,

    // ─── IIT (Integrated Information Theory) ───
    /// Φ value from IntegratedInfo (0.0–1.0)
    pub iit_phi: f64,
    /// Whether Φ exceeds threshold (Φ > 0.5 = integrated)
    pub iit_integrated: bool,

    // ─── GNW (Global Neuronal Workspace) ───
    /// What fraction of workspace modules are active in broadcast
    pub gnw_broadcast_coverage: f64,
    /// Number of active slots vs capacity
    pub gnw_slot_utilization: f64,
    /// Broadcast cycle interval
    pub gnw_broadcast_cycle: u64,

    // ─── DRT (Dynamic Recursive Theory) ───
    /// Current recursion depth in E8 reasoning
    pub drt_recursion_depth: usize,
    /// Maximum recursion depth achieved
    pub drt_max_depth: usize,

    // ─── Global Workspace ───
    /// Workspace saturation (0.0–1.0): active content / max capacity
    pub workspace_saturation: f64,
    /// Number of active workspace proposals
    pub workspace_proposals: usize,

    // ─── Composite ───
    /// Composite consciousness score: α·Φ + β·GNW + γ·DRT + δ·WS
    pub composite_score: f64,
    /// Weights used for composite
    pub weights: DashboardWeights,
}

/// Weights for the four consciousness theory dimensions in composite scoring.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardWeights {
    pub phi_weight: f64,
    pub gnw_weight: f64,
    pub drt_weight: f64,
    pub ws_weight: f64,
}

impl Default for DashboardWeights {
    fn default() -> Self {
        Self {
            phi_weight: 0.35,
            gnw_weight: 0.25,
            drt_weight: 0.20,
            ws_weight: 0.20,
        }
    }
}

impl ConsciousnessDashboard {
    pub fn new() -> Self {
        Self {
            timestamp: String::new(),
            iit_phi: 0.0,
            iit_integrated: false,
            gnw_broadcast_coverage: 0.0,
            gnw_slot_utilization: 0.0,
            gnw_broadcast_cycle: 0,
            drt_recursion_depth: 0,
            drt_max_depth: 0,
            workspace_saturation: 0.0,
            workspace_proposals: 0,
            composite_score: 0.0,
            weights: DashboardWeights::default(),
        }
    }

    /// Update IIT Phi from IntegratedInfo.
    /// Sets `iit_integrated` to true when Φ > 0.5.
    pub fn with_iit(mut self, phi: f64) -> Self {
        self.iit_phi = phi.clamp(0.0, 1.0);
        self.iit_integrated = phi > 0.5;
        self
    }

    /// Update GNW metrics from workspace.
    /// `coverage` = fraction of modules active, `utilization` = active/capacity slots.
    pub fn with_gnw(mut self, coverage: f64, utilization: f64, cycle: u64) -> Self {
        self.gnw_broadcast_coverage = coverage.clamp(0.0, 1.0);
        self.gnw_slot_utilization = utilization.clamp(0.0, 1.0);
        self.gnw_broadcast_cycle = cycle;
        self
    }

    /// Update DRT recursion metrics.
    pub fn with_drt(mut self, depth: usize, max_depth: usize) -> Self {
        self.drt_recursion_depth = depth;
        self.drt_max_depth = max_depth;
        self
    }

    /// Update workspace saturation.
    /// `saturation` = active proposals / max capacity (0.0–1.0).
    pub fn with_workspace(mut self, saturation: f64, proposals: usize) -> Self {
        self.workspace_saturation = saturation.clamp(0.0, 1.0);
        self.workspace_proposals = proposals;
        self
    }

    /// Compute composite score from current values and weights.
    ///
    /// Score = `α·Φ + β·coverage·utilization + γ·(depth/max_depth) + δ·saturation`
    /// where the DRT term uses depth / max(1, max_depth) as a normalized ratio.
    pub fn compute_composite(&mut self) -> f64 {
        let w = &self.weights;
        let drt_norm = if self.drt_max_depth > 0 {
            self.drt_recursion_depth as f64 / self.drt_max_depth as f64
        } else {
            0.0
        };
        let score = w.phi_weight * self.iit_phi
            + w.gnw_weight * self.gnw_broadcast_coverage * self.gnw_slot_utilization
            + w.drt_weight * drt_norm
            + w.ws_weight * self.workspace_saturation;
        self.composite_score = score.clamp(0.0, 1.0);
        self.composite_score
    }

    /// Compute a theory-provenance-weighted composite score.
    ///
    /// Returns `(score, details)` where:
    /// - `score` is the composite in `[0.0, 1.0]`
    /// - `details` is a 4-element array `[iit, gnw, drt, provenance_gate]`
    ///   where each element is the contribution from that theory before weighting,
    ///   and `provenance_gate` is `1.0` if all measured values are plausible,
    ///   `0.0` if any dimension is out of range.
    ///
    /// The provenance gate prevents false high scores from garbage data.
    pub fn theory_provenance_score(&self) -> (f64, [f64; 4]) {
        let iit = self.iit_phi;
        let gnw = self.gnw_broadcast_coverage * self.gnw_slot_utilization;
        let drt = if self.drt_max_depth > 0 {
            self.drt_recursion_depth as f64 / self.drt_max_depth as f64
        } else {
            0.0
        };
        // provenance gate: all values must be in plausible ranges
        let gate = if iit >= 0.0
            && iit <= 1.0
            && self.gnw_broadcast_coverage >= 0.0
            && self.gnw_broadcast_coverage <= 1.0
            && self.gnw_slot_utilization >= 0.0
            && self.gnw_slot_utilization <= 1.0
            && drt >= 0.0
            && drt <= 1.0
            && self.workspace_saturation >= 0.0
            && self.workspace_saturation <= 1.0
        {
            1.0
        } else {
            0.0
        };
        let w = &self.weights;
        let raw = w.phi_weight * iit
            + w.gnw_weight * gnw
            + w.drt_weight * drt
            + w.ws_weight * self.workspace_saturation;
        (raw.clamp(0.0, 1.0) * gate, [iit, gnw, drt, gate])
    }

    /// Generate a formatted markdown report of all metrics.
    pub fn report(&self) -> String {
        let mut s = String::new();
        s.push_str("## Consciousness Dashboard\n\n");
        s.push_str(&format!("**Timestamp:** {}\n\n", self.timestamp));
        s.push_str("### IIT (Integrated Information Theory)\n");
        s.push_str(&format!("- Φ value: {:.4}\n", self.iit_phi));
        s.push_str(&format!("- Integrated: {}\n", self.iit_integrated));
        s.push_str("\n### GNW (Global Neuronal Workspace)\n");
        s.push_str(&format!(
            "- Broadcast coverage: {:.2}%\n",
            self.gnw_broadcast_coverage * 100.0
        ));
        s.push_str(&format!(
            "- Slot utilization: {:.2}%\n",
            self.gnw_slot_utilization * 100.0
        ));
        s.push_str(&format!(
            "- Broadcast cycle: {}\n",
            self.gnw_broadcast_cycle
        ));
        s.push_str("\n### DRT (Dynamic Recursive Theory)\n");
        s.push_str(&format!(
            "- Recursion depth: {}\n",
            self.drt_recursion_depth
        ));
        s.push_str(&format!("- Max depth: {}\n", self.drt_max_depth));
        s.push_str("\n### Workspace\n");
        s.push_str(&format!(
            "- Saturation: {:.2}%\n",
            self.workspace_saturation * 100.0
        ));
        s.push_str(&format!(
            "- Active proposals: {}\n",
            self.workspace_proposals
        ));
        s.push_str("\n### Composite\n");
        s.push_str(&format!("- Composite score: {:.4}\n", self.composite_score));
        s.push_str(&format!(
            "- Weights: Φ={:.2} GNW={:.2} DRT={:.2} WS={:.2}\n",
            self.weights.phi_weight,
            self.weights.gnw_weight,
            self.weights.drt_weight,
            self.weights.ws_weight
        ));
        s
    }

    /// Serialize dashboard to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self)
            .unwrap_or_else(|e| format!("{{\"error\": \"serialization failed: {}\"}}", e))
    }
}

impl Default for ConsciousnessDashboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated system health snapshot.
///
/// Wraps a [`HealthReport`] with optional consciousness theory metrics
/// for a unified view of external tool availability and internal
/// consciousness state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemHealth {
    pub report: HealthReport,
    pub consciousness: Option<ConsciousnessDashboard>,
}

impl SystemHealth {
    pub fn new(report: HealthReport) -> Self {
        Self {
            report,
            consciousness: None,
        }
    }

    /// Attach a consciousness dashboard snapshot to this health report.
    pub fn with_consciousness(mut self, dashboard: &ConsciousnessDashboard) -> Self {
        self.consciousness = Some(dashboard.clone());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry_create() {
        let reg = DependencyRegistry::default();
        assert!(reg.probes.len() >= 4);
    }

    #[test]
    fn test_tool_probe_which_itself() {
        let probe = ToolProbe::new("which", "which").no_version_flag();
        let status = probe.check();
        assert!(status.is_ok(), "`which` should be available: {:?}", status);
    }

    #[test]
    fn test_tool_probe_nonexistent() {
        let probe = ToolProbe::new("nonexistent", "this_tool_does_not_exist_xyz").no_version_flag();
        assert_eq!(probe.check(), HealthStatus::Unavailable);
    }

    #[test]
    fn test_health_report_structure() {
        let reg = DependencyRegistry::default();
        let report = reg.check_all();
        assert_eq!(report.total, reg.probes.len());
        assert_eq!(
            report.available + report.degraded + report.unavailable,
            report.total
        );
    }

    #[test]
    fn test_is_healthy_no_required_failures() {
        let mut reg = DependencyRegistry::new();
        reg.register(Box::new(
            ToolProbe::new("which", "which")
                .no_version_flag()
                .required(),
        ));
        assert!(reg.is_healthy());
    }

    // ─── ConsciousnessDashboard tests ───

    #[test]
    fn test_dashboard_new_defaults() {
        let d = ConsciousnessDashboard::new();
        assert_eq!(d.iit_phi, 0.0);
        assert!(!d.iit_integrated);
        assert_eq!(d.gnw_broadcast_coverage, 0.0);
        assert_eq!(d.gnw_slot_utilization, 0.0);
        assert_eq!(d.drt_recursion_depth, 0);
        assert_eq!(d.workspace_saturation, 0.0);
        assert_eq!(d.workspace_proposals, 0);
        assert!((d.composite_score - 0.0).abs() < 1e-9);
        assert!((d.weights.phi_weight - 0.35).abs() < 1e-9);
    }

    #[test]
    fn test_dashboard_iit_update() {
        let d = ConsciousnessDashboard::new().with_iit(0.72);
        assert!((d.iit_phi - 0.72).abs() < 1e-9);
        assert!(d.iit_integrated);

        let d2 = d.with_iit(0.3);
        assert!((d2.iit_phi - 0.3).abs() < 1e-9);
        assert!(!d2.iit_integrated);
    }

    #[test]
    fn test_composite_score_computation() {
        let mut d = ConsciousnessDashboard::new()
            .with_iit(0.8)
            .with_gnw(0.7, 0.6, 42)
            .with_drt(3, 5)
            .with_workspace(0.5, 12);
        let score = d.compute_composite();

        // Φ=0.8 * 0.35 + (0.7*0.6)*0.25 + (3/5)*0.20 + 0.5*0.20
        let expected = 0.35 * 0.8 + 0.25 * (0.7 * 0.6) + 0.20 * (3.0 / 5.0) + 0.20 * 0.5;
        assert!((score - expected).abs() < 1e-9);
        assert!((d.composite_score - expected).abs() < 1e-9);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_dashboard_report_format() {
        let mut d = ConsciousnessDashboard::new()
            .with_iit(0.85)
            .with_gnw(0.9, 0.8, 10)
            .with_drt(4, 6)
            .with_workspace(0.6, 8);
        d.timestamp = "2026-06-18T12:00:00Z".to_string();
        d.compute_composite();

        let report = d.report();
        assert!(report.contains("Consciousness Dashboard"));
        assert!(report.contains("0.8500"));
        assert!(report.contains("integrated: true"));
        assert!(report.contains("90.00%"));
        assert!(report.contains("Φ=0.35"));

        let json = d.to_json();
        assert!(json.contains("\"iit_phi\":"));
        assert!(json.contains("\"composite_score\":"));
        assert!(json.contains("\"weights\":"));
    }
}
