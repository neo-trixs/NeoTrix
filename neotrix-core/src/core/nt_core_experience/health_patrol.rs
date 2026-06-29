use hex;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 全局健康巡查机制 — 意识体内部循环节点的健康保障
///
/// 三层架构:
///   1. Node Patrol — 每个 cycle 巡查注册的子系统节点
///   2. Integrity Guard — 周期性自我完整性验证
///   3. Adaptive Heal — 智能自适应修复路由
///
/// 属性: VsaTag::Self(HealthPatrol) — 所有巡查向量携带自身标签

// ── Patrol Node ──

#[derive(Debug, Clone)]
pub struct PatrolNode {
    pub name: String,
    pub subsystem: String,
    pub health: f64,
    pub last_heartbeat: Option<Instant>,
    pub failure_count: u64,
    pub consecutive_failures: u64,
    pub max_consecutive_failures: u64,
    pub degradation: DegradationLevel,
    pub anomaly_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum DegradationLevel {
    Full = 0,
    Reduced = 1,
    Limited = 2,
    Emergency = 3,
}

impl DegradationLevel {
    pub fn downgrade(&self) -> Self {
        match self {
            DegradationLevel::Full => DegradationLevel::Reduced,
            DegradationLevel::Reduced => DegradationLevel::Limited,
            DegradationLevel::Limited => DegradationLevel::Emergency,
            DegradationLevel::Emergency => DegradationLevel::Emergency,
        }
    }
}

impl PatrolNode {
    pub fn new(name: &str, subsystem: &str) -> Self {
        Self {
            name: name.to_string(),
            subsystem: subsystem.to_string(),
            health: 1.0,
            last_heartbeat: None,
            failure_count: 0,
            consecutive_failures: 0,
            max_consecutive_failures: 5,
            degradation: DegradationLevel::Full,
            anomaly_count: 0,
        }
    }

    pub fn record_heartbeat(&mut self) {
        self.last_heartbeat = Some(Instant::now());
        self.consecutive_failures = 0;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.consecutive_failures += 1;
        if self.consecutive_failures >= self.max_consecutive_failures / 2 {
            self.degradation = self.degradation.downgrade();
        }
        self.health = (1.0
            - (self.consecutive_failures as f64 / self.max_consecutive_failures as f64))
            .max(0.0);
    }

    pub fn record_anomaly(&mut self) {
        self.anomaly_count += 1;
        self.health *= 0.9;
    }

    pub fn is_healthy(&self) -> bool {
        self.degradation == DegradationLevel::Full && self.health > 0.6
    }

    pub fn heartbeat_timed_out(&self, timeout: Duration) -> bool {
        self.last_heartbeat.map_or(true, |t| t.elapsed() > timeout)
    }
}

// ── Integrity Check ──

#[derive(Debug, Clone)]
pub struct IntegrityCheck {
    pub check_name: String,
    pub passed: bool,
    pub detail: String,
    pub severity: IntegritySeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IntegritySeverity {
    Info,
    Warning,
    Critical,
}

impl IntegrityCheck {
    pub fn new(name: &str, passed: bool, detail: &str, severity: IntegritySeverity) -> Self {
        Self {
            check_name: name.to_string(),
            passed,
            detail: detail.to_string(),
            severity,
        }
    }
}

// ── Health Report ──

#[derive(Debug, Clone)]
pub struct PatrolReport {
    pub cycle: u64,
    pub timestamp: Instant,
    pub node_count: usize,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub failed_count: usize,
    pub integrity_checks: Vec<IntegrityCheck>,
    pub anomalies: Vec<AnomalyRecord>,
    pub overall_health: f64,
    pub integrity_score: f64,
    pub tamper_detected: bool,
}

#[derive(Debug, Clone)]
pub struct AnomalyRecord {
    pub source: String,
    pub description: String,
    pub severity: AnomalySeverity,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
    Warning,
}

// ── Adaptive Healing Record ──

#[derive(Debug, Clone)]
pub struct HealingOutcome {
    pub anomaly: String,
    pub strategy: String,
    pub success: bool,
    pub recovery_time_ms: u64,
    pub timestamp: Instant,
}

// ── Challenge-Response Verification ──

/// Challenge-response state for active subsystem verification.
/// Each subsystem receives a random nonce and must respond with
/// SHA-256(nonce || subsystem_name || secret) to prove liveness.
#[derive(Debug, Clone)]
pub struct ChallengeResponseState {
    /// Active challenge nonces per subsystem (subsystem_name -> (nonce, timestamp))
    pub active_challenges: HashMap<String, (Vec<u8>, Instant)>,
    /// Successful challenge completions per subsystem
    pub challenge_passes: HashMap<String, u64>,
    /// Failed challenge completions per subsystem
    pub challenge_failures: HashMap<String, u64>,
    /// Challenge timeout in seconds
    pub challenge_timeout_secs: u64,
    /// Maximum concurrent challenges
    pub max_concurrent: usize,
}

impl Default for ChallengeResponseState {
    fn default() -> Self {
        Self {
            active_challenges: HashMap::new(),
            challenge_passes: HashMap::new(),
            challenge_failures: HashMap::new(),
            challenge_timeout_secs: 30,
            max_concurrent: 10,
        }
    }
}

impl ChallengeResponseState {
    /// Issue a new challenge to a subsystem.
    /// Returns (nonce_hex, expected_response_hex) where expected = SHA-256(nonce || subsystem || secret).
    pub fn issue_challenge(&mut self, subsystem: &str, secret: &[u8]) -> Option<(String, String)> {
        if self.active_challenges.len() >= self.max_concurrent {
            return None;
        }
        let mut nonce = [0u8; 32];
        OsRng.fill_bytes(&mut nonce);
        let mut hasher = Sha256::new();
        hasher.update(&nonce);
        hasher.update(subsystem.as_bytes());
        hasher.update(secret);
        let expected = hasher.finalize();
        let nonce_hex = hex::encode(nonce);
        let expected_hex = hex::encode(expected);
        self.active_challenges
            .insert(subsystem.to_string(), (nonce.to_vec(), Instant::now()));
        Some((nonce_hex, expected_hex))
    }

    /// Verify a challenge response from a subsystem.
    pub fn verify_response(&mut self, subsystem: &str, response_hex: &str, secret: &[u8]) -> bool {
        let entry = match self.active_challenges.remove(subsystem) {
            Some(e) => e,
            None => return false,
        };
        let (nonce, timestamp) = entry;
        if timestamp.elapsed() > Duration::from_secs(self.challenge_timeout_secs) {
            self.challenge_failures
                .entry(subsystem.to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);
            return false;
        }
        let mut hasher = Sha256::new();
        hasher.update(&nonce);
        hasher.update(subsystem.as_bytes());
        hasher.update(secret);
        let expected = hasher.finalize();
        let expected_hex = hex::encode(expected);
        if response_hex == expected_hex {
            self.challenge_passes
                .entry(subsystem.to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);
            true
        } else {
            self.challenge_failures
                .entry(subsystem.to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);
            false
        }
    }

    /// Get the pass rate for a subsystem (0.0 to 1.0).
    pub fn pass_rate(&self, subsystem: &str) -> f64 {
        let passes = self.challenge_passes.get(subsystem).copied().unwrap_or(0);
        let failures = self.challenge_failures.get(subsystem).copied().unwrap_or(0);
        let total = passes + failures;
        if total == 0 {
            1.0
        } else {
            passes as f64 / total as f64
        }
    }

    /// Clean up expired challenges.
    pub fn clean_expired(&mut self) {
        let timeout = Duration::from_secs(self.challenge_timeout_secs);
        self.active_challenges
            .retain(|_, (_, ts)| ts.elapsed() <= timeout);
    }
}

// ── Global Health Patrol ──

pub struct GlobalHealthPatrol {
    /// 注册的巡查节点
    nodes: HashMap<String, PatrolNode>,
    /// 周期计数器
    cycle: u64,
    /// 最近一次巡查报告
    last_report: Option<PatrolReport>,
    /// 自适应修复历史
    healing_history: Vec<HealingOutcome>,
    /// 巡查间隔(cycle数)
    patrol_interval: u64,
    /// 完整性检查间隔
    integrity_interval: u64,
    /// 健康阈值 — 低于此值触发修复
    health_threshold: f64,
    /// 最近一次完整性检查的结果
    integrity_score: f64,
    /// 是否检测到篡改
    tamper_detected: bool,
    /// 挑战-响应验证状态
    challenge_state: ChallengeResponseState,
}

const MAX_HEALING_HISTORY: usize = 200;

impl GlobalHealthPatrol {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            cycle: 0,
            last_report: None,
            healing_history: Vec::new(),
            patrol_interval: 1,
            integrity_interval: 10,
            health_threshold: 0.4,
            integrity_score: 1.0,
            tamper_detected: false,
            challenge_state: ChallengeResponseState::default(),
        }
    }

    pub fn with_patrol_interval(mut self, interval: u64) -> Self {
        self.patrol_interval = interval;
        self
    }

    pub fn with_integrity_interval(mut self, interval: u64) -> Self {
        self.integrity_interval = interval;
        self
    }

    pub fn with_health_threshold(mut self, threshold: f64) -> Self {
        self.health_threshold = threshold;
        self
    }

    // ── Node Registry ──

    pub fn register_node(&mut self, name: &str, subsystem: &str) {
        if !self.nodes.contains_key(name) {
            self.nodes
                .insert(name.to_string(), PatrolNode::new(name, subsystem));
        }
    }

    pub fn unregister_node(&mut self, name: &str) {
        self.nodes.remove(name);
    }

    pub fn heartbeat(&mut self, node_name: &str) {
        if let Some(node) = self.nodes.get_mut(node_name) {
            node.record_heartbeat();
        }
    }

    pub fn record_node_failure(&mut self, node_name: &str) {
        if let Some(node) = self.nodes.get_mut(node_name) {
            node.record_failure();
        }
    }

    pub fn record_node_anomaly(&mut self, node_name: &str) {
        if let Some(node) = self.nodes.get_mut(node_name) {
            node.record_anomaly();
        }
    }

    // ── Patrol Tick ──

    pub fn tick(&mut self) -> Option<PatrolReport> {
        self.cycle += 1;

        let mut report = PatrolReport {
            cycle: self.cycle,
            timestamp: Instant::now(),
            node_count: self.nodes.len(),
            healthy_count: 0,
            degraded_count: 0,
            failed_count: 0,
            integrity_checks: Vec::new(),
            anomalies: Vec::new(),
            overall_health: 1.0,
            integrity_score: self.integrity_score,
            tamper_detected: self.tamper_detected,
        };

        // Phase 1: Node patrol (every cycle)
        for node in self.nodes.values_mut() {
            if node.heartbeat_timed_out(Duration::from_secs(120)) {
                node.record_failure();
                report.anomalies.push(AnomalyRecord {
                    source: node.name.clone(),
                    description: format!(
                        "heartbeat timeout: {} consecutive failures",
                        node.consecutive_failures
                    ),
                    severity: if node.consecutive_failures >= 3 {
                        AnomalySeverity::High
                    } else {
                        AnomalySeverity::Warning
                    },
                    evidence: format!(
                        "last_heartbeat={:?}, degradation={:?}",
                        node.last_heartbeat, node.degradation
                    ),
                });
            }
            if node.is_healthy() {
                report.healthy_count += 1;
            } else if node.health > 0.0 {
                report.degraded_count += 1;
            } else {
                report.failed_count += 1;
            }
        }

        // Phase 2: Integrity guard (every integrity_interval cycles)
        if self.cycle % self.integrity_interval == 0 {
            let integrity_report = self.run_integrity_checks();
            report.integrity_checks = integrity_report;
            report.tamper_detected = self.tamper_detected;
        }

        // Aggregate health
        let total = report.node_count.max(1) as f64;
        report.overall_health =
            (report.healthy_count as f64 + report.degraded_count as f64 * 0.5) / total;
        report.integrity_score = self.integrity_score;

        // Phase 3: Adaptive healing trigger
        self.trigger_adaptive_healing(&report);

        self.last_report = Some(report.clone());
        Some(report)
    }

    // ── Integrity Guard ──

    fn run_integrity_checks(&mut self) -> Vec<IntegrityCheck> {
        let mut checks = Vec::new();

        // Check 1: Node count consistency
        let registered = self.nodes.len();
        let alive = self
            .nodes
            .values()
            .filter(|n| !n.heartbeat_timed_out(Duration::from_secs(300)))
            .count();
        let ratio = if registered > 0 {
            alive as f64 / registered as f64
        } else {
            1.0
        };
        checks.push(IntegrityCheck::new(
            "node_count_consistency",
            ratio >= 0.7,
            &format!("{}/{} nodes alive", alive, registered),
            if ratio < 0.5 {
                IntegritySeverity::Critical
            } else if ratio < 0.7 {
                IntegritySeverity::Warning
            } else {
                IntegritySeverity::Info
            },
        ));

        // Check 2: Self-healing effectiveness
        let healing_success_rate = self.healing_success_rate();
        checks.push(IntegrityCheck::new(
            "healing_effectiveness",
            healing_success_rate >= 0.5,
            &format!("healing success rate: {:.2}", healing_success_rate),
            if healing_success_rate < 0.3 {
                IntegritySeverity::Critical
            } else if healing_success_rate < 0.5 {
                IntegritySeverity::Warning
            } else {
                IntegritySeverity::Info
            },
        ));

        // Check 3: Degradation spiral detection (too many nodes degraded)
        let degraded = self
            .nodes
            .values()
            .filter(|n| n.degradation > DegradationLevel::Full)
            .count();
        let degraded_ratio = if registered > 0 {
            degraded as f64 / registered as f64
        } else {
            0.0
        };
        checks.push(IntegrityCheck::new(
            "degradation_spiral",
            degraded_ratio < 0.5,
            &format!("{:.1}% nodes degraded", degraded_ratio * 100.0),
            if degraded_ratio >= 0.7 {
                IntegritySeverity::Critical
            } else if degraded_ratio >= 0.5 {
                IntegritySeverity::Warning
            } else {
                IntegritySeverity::Info
            },
        ));

        // Check 4: Anti-reverse-engineering — env integrity (LD_PRELOAD / DYLD injection)
        let env_suspicious = self.check_environment_integrity();
        checks.push(IntegrityCheck::new(
            "env_integrity",
            !env_suspicious,
            if env_suspicious {
                "suspicious env vars detected (LD_PRELOAD/DYLD_INSERT_LIBRARIES)"
            } else {
                "environment clean"
            },
            if env_suspicious {
                IntegritySeverity::Critical
            } else {
                IntegritySeverity::Info
            },
        ));
        if env_suspicious {
            self.tamper_detected = true;
        }

        // Check 5: Anti-reverse-engineering — binary path integrity
        let path_ok = self.check_binary_path_integrity();
        checks.push(IntegrityCheck::new(
            "binary_path_integrity",
            path_ok,
            if path_ok {
                "binary path matches expected location"
            } else {
                "binary running from unexpected location"
            },
            if !path_ok {
                IntegritySeverity::Warning
            } else {
                IntegritySeverity::Info
            },
        ));

        // Check 6: Anti-reverse-engineering — debugger detection
        let debugger_detected = self.detect_debugger();
        checks.push(IntegrityCheck::new(
            "debugger_detection",
            !debugger_detected,
            if debugger_detected {
                "debugger/tracer attached to process"
            } else {
                "no debugger detected"
            },
            if debugger_detected {
                IntegritySeverity::Critical
            } else {
                IntegritySeverity::Info
            },
        ));
        if debugger_detected {
            self.tamper_detected = true;
        }

        // Check 7: Anomaly density
        let total_anomalies: u64 = self.nodes.values().map(|n| n.anomaly_count).sum();
        let anomaly_rate = if self.cycle > 0 {
            total_anomalies as f64 / self.cycle as f64
        } else {
            0.0
        };
        checks.push(IntegrityCheck::new(
            "anomaly_density",
            anomaly_rate < 0.1,
            &format!(
                "{:.3} anomalies/cycle ({} total)",
                anomaly_rate, total_anomalies
            ),
            if anomaly_rate >= 0.3 {
                IntegritySeverity::Critical
            } else if anomaly_rate >= 0.1 {
                IntegritySeverity::Warning
            } else {
                IntegritySeverity::Info
            },
        ));

        // Update integrity score as weighted average of check pass rates
        let passed = checks.iter().filter(|c| c.passed).count() as f64;
        let total_c = checks.len() as f64;
        self.integrity_score = if total_c > 0.0 { passed / total_c } else { 1.0 };

        // Tamper detection: critical failures in integrity checks
        let critical_failures = checks
            .iter()
            .filter(|c| !c.passed && c.severity == IntegritySeverity::Critical)
            .count();
        if critical_failures >= 2 {
            self.tamper_detected = true;
        }

        checks
    }

    // ── Adaptive Healing ──

    fn trigger_adaptive_healing(&mut self, report: &PatrolReport) {
        if report.overall_health >= self.health_threshold {
            return;
        }

        // Find worst nodes and record healing attempts
        let mut worst_nodes: Vec<(&String, &PatrolNode)> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.health < self.health_threshold)
            .collect();
        worst_nodes.sort_by(|a, b| {
            a.1.health
                .partial_cmp(&b.1.health)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Learn from healing history: pick best strategy for this failure pattern
        for (name, node) in &worst_nodes {
            let strategy = self.select_healing_strategy(name, node);
            // Record the healing attempt for future learning
            let _already_recorded = self.healing_history.iter().any(|h| {
                h.anomaly.as_str() == name.as_str()
                    && h.timestamp.elapsed() < Duration::from_secs(60)
            });
            if !_already_recorded {
                self.healing_history.push(HealingOutcome {
                    anomaly: name.to_string(),
                    strategy: strategy.to_string(),
                    success: false,
                    recovery_time_ms: 0,
                    timestamp: Instant::now(),
                });
            }
        }
        if self.healing_history.len() > MAX_HEALING_HISTORY {
            let overflow = self.healing_history.len() - MAX_HEALING_HISTORY;
            self.healing_history.drain(0..overflow);
        }
    }

    fn select_healing_strategy(&self, name: &str, node: &PatrolNode) -> String {
        // Check past outcomes for this node
        let past_outcomes: Vec<&HealingOutcome> = self
            .healing_history
            .iter()
            .filter(|h| h.anomaly == name)
            .collect();

        if past_outcomes.is_empty() {
            // First time: try conservative strategies
            return match node.degradation {
                DegradationLevel::Full | DegradationLevel::Reduced => "immediate_retry".to_string(),
                DegradationLevel::Limited => "backoff_retry".to_string(),
                DegradationLevel::Emergency => "restart".to_string(),
            };
        }

        // Find best past strategy
        let mut best_strategy = "immediate_retry";
        let mut best_rate = 0.0;

        let mut by_strategy: HashMap<&str, (u64, u64)> = HashMap::new();
        for h in &past_outcomes {
            let (total, successes) = by_strategy.entry(h.strategy.as_str()).or_insert((0, 0));
            *total += 1;
            if h.success {
                *successes += 1;
            }
        }

        for (strategy, (total, successes)) in &by_strategy {
            let rate = *successes as f64 / *total as f64;
            if rate > best_rate {
                best_rate = rate;
                best_strategy = strategy;
            }
        }

        best_strategy.to_string()
    }

    pub fn record_healing_outcome(&mut self, anomaly: &str, success: bool) {
        // Find the most recent pending healing outcome and update it
        if let Some(outcome) = self
            .healing_history
            .iter_mut()
            .filter(|h| h.anomaly == anomaly)
            .last()
        {
            outcome.success = success;
            outcome.recovery_time_ms = outcome.timestamp.elapsed().as_millis() as u64;
        }
    }

    pub fn healing_success_rate(&self) -> f64 {
        if self.healing_history.is_empty() {
            return 1.0;
        }
        let successes = self.healing_history.iter().filter(|h| h.success).count() as f64;
        successes / self.healing_history.len() as f64
    }

    // ── Queries ──

    pub fn last_report(&self) -> Option<&PatrolReport> {
        self.last_report.as_ref()
    }

    pub fn overall_health(&self) -> f64 {
        self.last_report
            .as_ref()
            .map(|r| r.overall_health)
            .unwrap_or(1.0)
    }

    pub fn integrity_score(&self) -> f64 {
        self.integrity_score
    }

    pub fn tamper_detected(&self) -> bool {
        self.tamper_detected
    }

    pub fn node_health(&self, name: &str) -> Option<f64> {
        self.nodes.get(name).map(|n| n.health)
    }

    pub fn unhealthy_nodes(&self) -> Vec<(&str, f64)> {
        self.nodes
            .iter()
            .filter(|(_, n)| n.health < 0.6)
            .map(|(name, n)| (name.as_str(), n.health))
            .collect()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn healthy_node_count(&self) -> usize {
        self.nodes.values().filter(|n| n.is_healthy()).count()
    }

    /// Summary string for consciousness logging
    pub fn patrol_summary(&self) -> String {
        let r = self.last_report.as_ref();
        format!(
            "HealthPatrol[cycle={}]: nodes={}/{}, integrity={:.2}, tamper={}, health={:.2}, healing_rate={:.2}",
            self.cycle,
            r.map(|r| r.healthy_count).unwrap_or(0),
            r.map(|r| r.node_count).unwrap_or(0),
            self.integrity_score,
            if self.tamper_detected { "⚠" } else { "✓" },
            r.map(|r| r.overall_health).unwrap_or(1.0),
            self.healing_success_rate(),
        )
    }

    // ── Challenge-Response Verification ──

    /// Issue a challenge to a specific subsystem.
    pub fn issue_challenge(&mut self, subsystem: &str, secret: &[u8]) -> Option<(String, String)> {
        self.challenge_state.issue_challenge(subsystem, secret)
    }

    /// Verify a challenge response from a subsystem.
    pub fn verify_challenge(&mut self, subsystem: &str, response_hex: &str, secret: &[u8]) -> bool {
        self.challenge_state
            .verify_response(subsystem, response_hex, secret)
    }

    /// Get the challenge pass rate for a subsystem.
    pub fn challenge_pass_rate(&self, subsystem: &str) -> f64 {
        self.challenge_state.pass_rate(subsystem)
    }

    /// Run a challenge round: issue challenges to all registered nodes.
    /// Returns Vec<(node_name, nonce_hex, expected_response_hex)>.
    pub fn run_challenge_round(&mut self) -> Vec<(String, String, String)> {
        self.challenge_state.clean_expired();
        let secret = b"neotrix-challenge-secret-v1";
        let mut challenges = Vec::new();
        for node_name in self.nodes.keys() {
            if let Some((nonce, expected)) = self.challenge_state.issue_challenge(node_name, secret)
            {
                challenges.push((node_name.clone(), nonce, expected));
            }
        }
        challenges
    }

    // ── Anti-Reverse-Engineering Runtime Checks ──

    /// 环境完整性检查: 检测 LD_PRELOAD / DYLD_INSERT_LIBRARIES 等注入向量
    fn check_environment_integrity(&self) -> bool {
        let suspicious_vars = [
            "LD_PRELOAD",
            "LD_LIBRARY_PATH",
            "DYLD_INSERT_LIBRARIES",
            "DYLD_FORCE_FLAT_NAMESPACE",
        ];
        for var in &suspicious_vars {
            if let Ok(val) = std::env::var(var) {
                if !val.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    /// 二进制路径完整性: 验证可执行文件是否从期望位置运行
    fn check_binary_path_integrity(&self) -> bool {
        if let Ok(exe_path) = std::env::current_exe() {
            let path_str = exe_path.to_string_lossy().to_lowercase();
            if path_str.contains("neotrix")
                || path_str.contains("opencode")
                || path_str.contains("target/debug")
                || path_str.contains("target/release")
            {
                return true;
            }
            return false;
        }
        false
    }

    /// 调试器检测: 检查是否有调试器/跟踪器附加到进程
    #[cfg(target_os = "macos")]
    fn detect_debugger(&self) -> bool {
        let result = std::process::Command::new("sysctl")
            .args(["-n", "security.mac.proc_enforce"])
            .output();
        if let Ok(output) = result {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                let val = stdout.trim();
                if val == "0" {
                    return true;
                }
            }
        }
        if let Ok(ppid) = std::env::var("_") {
            if ppid.contains("lldb") || ppid.contains("gdb") || ppid.contains("dtrace") {
                return true;
            }
        }
        false
    }

    #[cfg(not(target_os = "macos"))]
    fn detect_debugger(&self) -> bool {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("TracerPid:") {
                    let pid = line.trim_start_matches("TracerPid:").trim();
                    if pid != "0" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 抗逆向入侵报告: 供外部查询
    pub fn anti_tamper_report(&self) -> String {
        let env_ok = !self.check_environment_integrity();
        let path_ok = self.check_binary_path_integrity();
        let debug = self.detect_debugger();
        format!(
            "AntiTamper[env={}, path={}, debug={}, detected={}]",
            if env_ok { "✓" } else { "⚠" },
            if path_ok { "✓" } else { "⚠" },
            if !debug { "✓" } else { "⚠" },
            if self.tamper_detected {
                "TAMPERED"
            } else {
                "clean"
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // ═══════════════════════════════════════════
    // PatrolNode
    // ═══════════════════════════════════════════

    #[test]
    fn test_patrol_node_new_defaults() {
        let node = PatrolNode::new("test_node", "test_subsystem");
        assert_eq!(node.name, "test_node");
        assert_eq!(node.subsystem, "test_subsystem");
        assert!((node.health - 1.0).abs() < f64::EPSILON);
        assert!(node.last_heartbeat.is_none());
        assert_eq!(node.failure_count, 0);
        assert_eq!(node.consecutive_failures, 0);
        assert_eq!(node.max_consecutive_failures, 5);
        assert_eq!(node.degradation, DegradationLevel::Full);
        assert_eq!(node.anomaly_count, 0);
        assert!(node.is_healthy());
    }

    #[test]
    fn test_patrol_node_record_heartbeat() {
        let mut node = PatrolNode::new("test", "sub");
        assert!(node.last_heartbeat.is_none());
        node.record_heartbeat();
        assert!(node.last_heartbeat.is_some());
        assert_eq!(node.consecutive_failures, 0);
    }

    #[test]
    fn test_patrol_node_record_failure_decrements_health() {
        let mut node = PatrolNode::new("test", "sub");
        node.record_failure();
        assert_eq!(node.failure_count, 1);
        assert_eq!(node.consecutive_failures, 1);
        assert!((node.health - 0.8).abs() < f64::EPSILON);
        assert_eq!(node.degradation, DegradationLevel::Full);
        assert!(node.is_healthy());

        node.record_failure();
        assert_eq!(node.consecutive_failures, 2);
        assert!((node.health - 0.6).abs() < f64::EPSILON);
        assert_eq!(node.degradation, DegradationLevel::Reduced);
        assert!(!node.is_healthy());
    }

    #[test]
    fn test_patrol_node_health_bottoms_at_zero() {
        let mut node = PatrolNode::new("test", "sub");
        for _ in 0..10 {
            node.record_failure();
        }
        assert!((node.health - 0.0).abs() < f64::EPSILON);
        assert_eq!(node.degradation, DegradationLevel::Emergency);
    }

    #[test]
    fn test_patrol_node_heartbeat_timed_out_defaults_to_true() {
        let node = PatrolNode::new("test", "sub");
        assert!(node.heartbeat_timed_out(Duration::from_secs(1)));
    }

    #[test]
    fn test_patrol_node_heartbeat_timed_out_after_heartbeat() {
        let mut node = PatrolNode::new("test", "sub");
        node.record_heartbeat();
        assert!(!node.heartbeat_timed_out(Duration::from_secs(0)));
    }

    #[test]
    fn test_patrol_node_anomaly_record_decreases_health() {
        let mut node = PatrolNode::new("test", "sub");
        assert!((node.health - 1.0).abs() < f64::EPSILON);
        node.record_anomaly();
        assert_eq!(node.anomaly_count, 1);
        assert!((node.health - 0.9).abs() < f64::EPSILON);
        node.record_anomaly();
        assert!((node.health - 0.81).abs() < f64::EPSILON);
    }

    #[test]
    fn test_degradation_level_downgrade_chain() {
        assert_eq!(
            DegradationLevel::Full.downgrade(),
            DegradationLevel::Reduced
        );
        assert_eq!(
            DegradationLevel::Reduced.downgrade(),
            DegradationLevel::Limited
        );
        assert_eq!(
            DegradationLevel::Limited.downgrade(),
            DegradationLevel::Emergency
        );
        assert_eq!(
            DegradationLevel::Emergency.downgrade(),
            DegradationLevel::Emergency
        );
    }

    #[test]
    fn test_degradation_level_ordering() {
        assert!(DegradationLevel::Full < DegradationLevel::Reduced);
        assert!(DegradationLevel::Reduced < DegradationLevel::Limited);
        assert!(DegradationLevel::Limited < DegradationLevel::Emergency);
    }

    // ═══════════════════════════════════════════
    // IntegrityCheck & AnomalyRecord
    // ═══════════════════════════════════════════

    #[test]
    fn test_integrity_check_new_passed() {
        let check = IntegrityCheck::new("test_check", true, "all good", IntegritySeverity::Info);
        assert_eq!(check.check_name, "test_check");
        assert!(check.passed);
        assert_eq!(check.detail, "all good");
        assert_eq!(check.severity, IntegritySeverity::Info);
    }

    #[test]
    fn test_integrity_check_new_failed() {
        let check = IntegrityCheck::new(
            "critical_fail",
            false,
            "something broke",
            IntegritySeverity::Critical,
        );
        assert!(!check.passed);
        assert_eq!(check.severity, IntegritySeverity::Critical);
    }

    #[test]
    fn test_anomaly_record_creation() {
        let rec = AnomalyRecord {
            source: "test".to_string(),
            description: "anomaly desc".to_string(),
            severity: AnomalySeverity::High,
            evidence: "evidence data".to_string(),
        };
        assert_eq!(rec.source, "test");
        assert_eq!(rec.severity, AnomalySeverity::High);
    }

    // ═══════════════════════════════════════════
    // ChallengeResponseState
    // ═══════════════════════════════════════════

    #[test]
    fn test_challenge_state_defaults() {
        let state = ChallengeResponseState::default();
        assert!(state.active_challenges.is_empty());
        assert_eq!(state.challenge_timeout_secs, 30);
        assert_eq!(state.max_concurrent, 10);
    }

    #[test]
    fn test_challenge_issue_and_verify_roundtrip() {
        let mut state = ChallengeResponseState::default();
        let secret = b"test-secret";
        let (nonce, expected) = state
            .issue_challenge("subsystem_a", secret)
            .expect("should issue challenge");
        assert!(!nonce.is_empty());
        assert!(!expected.is_empty());
        let result = state.verify_response("subsystem_a", &expected, secret);
        assert!(result);
    }

    #[test]
    fn test_challenge_wrong_response_fails() {
        let mut state = ChallengeResponseState::default();
        let secret = b"test-secret";
        state
            .issue_challenge("sub_a", secret)
            .expect("should issue");
        let result = state.verify_response("sub_a", "wrong_response", secret);
        assert!(!result);
    }

    #[test]
    fn test_challenge_unknown_subsystem_fails() {
        let mut state = ChallengeResponseState::default();
        let result = state.verify_response("unknown", "anything", b"secret");
        assert!(!result);
    }

    #[test]
    fn test_challenge_max_concurrent_blocks() {
        let mut state = ChallengeResponseState {
            max_concurrent: 2,
            ..ChallengeResponseState::default()
        };
        let secret = b"sec";
        assert!(state.issue_challenge("a", secret).is_some());
        assert!(state.issue_challenge("b", secret).is_some());
        assert!(state.issue_challenge("c", secret).is_none());
    }

    #[test]
    fn test_challenge_pass_rate_no_attempts() {
        let state = ChallengeResponseState::default();
        assert!((state.pass_rate("any") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_challenge_pass_rate_tracks_ratio() {
        let mut state = ChallengeResponseState::default();
        let secret = b"sec";
        let (_, expected) = state.issue_challenge("sub", secret).unwrap();
        state.verify_response("sub", &expected, secret);
        let (_, expected) = state.issue_challenge("sub", secret).unwrap();
        state.verify_response("sub", "wrong", secret);
        assert!((state.pass_rate("sub") - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_clean_expired_removes_old() {
        let mut state = ChallengeResponseState {
            challenge_timeout_secs: 0,
            ..ChallengeResponseState::default()
        };
        state.issue_challenge("sub", b"sec");
        assert_eq!(state.active_challenges.len(), 1);
        thread::sleep(Duration::from_millis(1));
        state.clean_expired();
        assert!(state.active_challenges.is_empty());
    }

    // ═══════════════════════════════════════════
    // HealingOutcome
    // ═══════════════════════════════════════════

    #[test]
    fn test_healing_outcome_creation() {
        let outcome = HealingOutcome {
            anomaly: "test_failure".to_string(),
            strategy: "restart".to_string(),
            success: true,
            recovery_time_ms: 42,
            timestamp: Instant::now(),
        };
        assert_eq!(outcome.anomaly, "test_failure");
        assert_eq!(outcome.strategy, "restart");
        assert!(outcome.success);
        assert_eq!(outcome.recovery_time_ms, 42);
    }

    // ═══════════════════════════════════════════
    // GlobalHealthPatrol — Node Registration
    // ═══════════════════════════════════════════

    #[test]
    fn test_global_health_patrol_new() {
        let patrol = GlobalHealthPatrol::new();
        assert_eq!(patrol.cycle, 0);
        assert!(patrol.nodes.is_empty());
        assert_eq!(patrol.patrol_interval, 1);
        assert_eq!(patrol.integrity_interval, 10);
        assert!((patrol.health_threshold - 0.4).abs() < f64::EPSILON);
        assert!(patrol.last_report.is_none());
        assert!((patrol.integrity_score - 1.0).abs() < f64::EPSILON);
        assert!(!patrol.tamper_detected);
    }

    #[test]
    fn test_register_node_creates_node_with_default_health() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("node_a", "subsystem_x");
        assert_eq!(patrol.node_count(), 1);
        let health = patrol.node_health("node_a");
        assert!(health.is_some());
        assert!((health.unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_register_node_duplicate_is_noop() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("node_a", "sub_x");
        patrol.register_node("node_a", "sub_y");
        assert_eq!(patrol.node_count(), 1);
    }

    #[test]
    fn test_unregister_node_removes_node() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("node_a", "sub");
        assert_eq!(patrol.node_count(), 1);
        patrol.unregister_node("node_a");
        assert_eq!(patrol.node_count(), 0);
        assert!(patrol.node_health("node_a").is_none());
    }

    #[test]
    fn test_heartbeat_updates_existing_node() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("node_a", "sub");
        patrol.record_node_failure("node_a");
        let health_before = patrol.node_health("node_a").unwrap();
        patrol.heartbeat("node_a");
        let health_after = patrol.node_health("node_a").unwrap();
        assert!((health_after - health_before).abs() < f64::EPSILON);
    }

    #[test]
    fn test_heartbeat_nonexistent_node_is_noop() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.heartbeat("nonexistent");
    }

    #[test]
    fn test_record_node_failure_nonexistent_is_noop() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.record_node_failure("ghost");
    }

    #[test]
    fn test_record_node_anomaly_nonexistent_is_noop() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.record_node_anomaly("ghost");
    }

    // ═══════════════════════════════════════════
    // GlobalHealthPatrol — Builder Pattern
    // ═══════════════════════════════════════════

    #[test]
    fn test_builder_with_patrol_interval() {
        let patrol = GlobalHealthPatrol::new().with_patrol_interval(5);
        assert_eq!(patrol.patrol_interval, 5);
    }

    #[test]
    fn test_builder_with_integrity_interval() {
        let patrol = GlobalHealthPatrol::new().with_integrity_interval(20);
        assert_eq!(patrol.integrity_interval, 20);
    }

    #[test]
    fn test_builder_with_health_threshold() {
        let patrol = GlobalHealthPatrol::new().with_health_threshold(0.6);
        assert!((patrol.health_threshold - 0.6).abs() < f64::EPSILON);
    }

    // ═══════════════════════════════════════════
    // GlobalHealthPatrol — Tick & Patrol
    // ═══════════════════════════════════════════

    #[test]
    fn test_tick_on_empty_patrol_returns_report() {
        let mut patrol = GlobalHealthPatrol::new();
        let report = patrol.tick();
        assert!(report.is_some());
        let r = report.unwrap();
        assert_eq!(r.cycle, 1);
        assert_eq!(r.node_count, 0);
        assert_eq!(r.healthy_count, 0);
        assert_eq!(r.degraded_count, 0);
        assert_eq!(r.failed_count, 0);
        assert!((r.overall_health - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_with_all_healthy_nodes() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("node_a", "sub1");
        patrol.register_node("node_b", "sub2");
        patrol.heartbeat("node_a");
        patrol.heartbeat("node_b");
        let report = patrol.tick().unwrap();
        assert_eq!(report.node_count, 2);
        assert_eq!(report.healthy_count, 2);
        assert_eq!(report.degraded_count, 0);
        assert!((report.overall_health - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_with_failed_node_reduces_health() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("node_a", "sub1");
        for _ in 0..5 {
            patrol.record_node_failure("node_a");
        }
        let report = patrol.tick().unwrap();
        assert_eq!(report.node_count, 1);
        assert_eq!(report.healthy_count, 0);
        assert_eq!(report.degraded_count, 0);
        assert_eq!(report.failed_count, 1);
        assert!((report.overall_health - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_generates_anomalies_on_heartbeat_timeout() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("node_a", "sub1");
        let report = patrol.tick().unwrap();
        assert!(report.anomalies.iter().any(|a| a.source == "node_a"));
    }

    #[test]
    fn test_tick_increments_cycle() {
        let mut patrol = GlobalHealthPatrol::new();
        assert_eq!(patrol.tick().unwrap().cycle, 1);
        assert_eq!(patrol.tick().unwrap().cycle, 2);
    }

    // ═══════════════════════════════════════════
    // GlobalHealthPatrol — Queries
    // ═══════════════════════════════════════════

    #[test]
    fn test_overall_health_before_tick_returns_one() {
        let patrol = GlobalHealthPatrol::new();
        assert!((patrol.overall_health() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_overall_health_after_tick() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("a", "s1");
        patrol.tick();
        let health = patrol.overall_health();
        assert!(health >= 0.0 && health <= 1.0);
    }

    #[test]
    fn test_integrity_score_default() {
        let patrol = GlobalHealthPatrol::new();
        assert!((patrol.integrity_score() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tamper_detected_default_false() {
        let patrol = GlobalHealthPatrol::new();
        assert!(!patrol.tamper_detected());
    }

    #[test]
    fn test_unhealthy_nodes_empty_initially() {
        let patrol = GlobalHealthPatrol::new();
        assert!(patrol.unhealthy_nodes().is_empty());
    }

    #[test]
    fn test_healthy_node_count() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("a", "s1");
        patrol.register_node("b", "s2");
        patrol.heartbeat("a");
        patrol.heartbeat("b");
        assert_eq!(patrol.healthy_node_count(), 2);
        patrol.record_node_failure("a");
        assert_eq!(patrol.healthy_node_count(), 1);
    }

    #[test]
    fn test_patrol_summary_after_tick() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("a", "s1");
        patrol.tick();
        let summary = patrol.patrol_summary();
        assert!(summary.contains("HealthPatrol"));
        assert!(summary.contains("cycle="));
    }

    #[test]
    fn test_last_report_after_tick() {
        let mut patrol = GlobalHealthPatrol::new();
        assert!(patrol.last_report().is_none());
        patrol.tick();
        assert!(patrol.last_report().is_some());
    }

    // ═══════════════════════════════════════════
    // GlobalHealthPatrol — Challenge Integration
    // ═══════════════════════════════════════════

    #[test]
    fn test_challenge_round_issues_for_all_nodes() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("a", "s1");
        patrol.register_node("b", "s2");
        let challenges = patrol.run_challenge_round();
        assert_eq!(challenges.len(), 2);
        for (name, nonce, expected) in &challenges {
            assert!(!nonce.is_empty());
            assert!(!expected.is_empty());
            assert!(name == "a" || name == "b");
        }
    }

    #[test]
    fn test_verify_integrated_challenge() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("test_node", "sub");
        let challenges = patrol.run_challenge_round();
        assert_eq!(challenges.len(), 1);
        let (name, _nonce, expected) = &challenges[0];
        let secret = b"neotrix-challenge-secret-v1";
        let result = patrol.verify_challenge(name, expected, secret);
        assert!(result);
    }

    #[test]
    fn test_verify_challenge_wrong_secret_fails() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("n", "s");
        let challenges = patrol.run_challenge_round();
        let (name, _nonce, expected) = &challenges[0];
        let result = patrol.verify_challenge(name, expected, b"wrong-secret");
        assert!(!result);
    }

    #[test]
    fn test_challenge_pass_rate_integrated() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("n", "s");
        let rate = patrol.challenge_pass_rate("n");
        assert!((rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_issue_challenge_on_patrol() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("n", "s");
        let result = patrol.issue_challenge("n", b"custom-secret");
        assert!(result.is_some());
        let (nonce, expected) = result.unwrap();
        assert!(!nonce.is_empty());
        assert!(!expected.is_empty());
    }

    // ═══════════════════════════════════════════
    // GlobalHealthPatrol — Adaptive Healing
    // ═══════════════════════════════════════════

    #[test]
    fn test_select_healing_strategy_returns_string() {
        let patrol = GlobalHealthPatrol::new();
        let node = PatrolNode::new("test", "sub");
        let strategy = patrol.select_healing_strategy("test", &node);
        assert!(!strategy.is_empty());
    }

    #[test]
    fn test_select_healing_strategy_first_time_is_immediate_retry() {
        let patrol = GlobalHealthPatrol::new();
        let node = PatrolNode::new("healthy_node", "sub");
        let strategy = patrol.select_healing_strategy("healthy_node", &node);
        assert_eq!(strategy, "immediate_retry");
    }

    #[test]
    fn test_select_healing_strategy_limited_degradation_first_time() {
        let patrol = GlobalHealthPatrol::new();
        let mut node = PatrolNode::new("limited_node", "sub");
        node.degradation = DegradationLevel::Limited;
        let strategy = patrol.select_healing_strategy("limited_node", &node);
        assert_eq!(strategy, "backoff_retry");
    }

    #[test]
    fn test_select_healing_strategy_emergency_first_time() {
        let patrol = GlobalHealthPatrol::new();
        let mut node = PatrolNode::new("emergency_node", "sub");
        node.degradation = DegradationLevel::Emergency;
        let strategy = patrol.select_healing_strategy("emergency_node", &node);
        assert_eq!(strategy, "restart");
    }

    #[test]
    fn test_healing_success_rate_empty_history() {
        let patrol = GlobalHealthPatrol::new();
        assert!((patrol.healing_success_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_record_healing_outcome_updates_history() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.register_node("node_a", "sub1");
        for _ in 0..5 {
            patrol.record_node_failure("node_a");
        }
        patrol.tick();
        patrol.record_healing_outcome("node_a", true);
        assert!((patrol.healing_success_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_healing_history_bounded() {
        let mut patrol = GlobalHealthPatrol::new();
        for i in 0..MAX_HEALING_HISTORY + 50 {
            patrol.healing_history.push(HealingOutcome {
                anomaly: format!("node_{}", i % 10),
                strategy: "immediate_retry".to_string(),
                success: i % 2 == 0,
                recovery_time_ms: 0,
                timestamp: Instant::now(),
            });
        }
        assert!(patrol.healing_history.len() <= MAX_HEALING_HISTORY);
    }

    #[test]
    fn test_healing_not_triggered_when_above_threshold() {
        let mut patrol = GlobalHealthPatrol::new().with_health_threshold(0.5);
        patrol.register_node("healthy", "sub");
        patrol.tick();
        assert!(patrol.healing_history.is_empty());
    }

    // ═══════════════════════════════════════════
    // GlobalHealthPatrol — Integrity Checks
    // ═══════════════════════════════════════════

    #[test]
    fn test_run_integrity_checks_on_empty_system() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.cycle = 10;
        patrol.integrity_interval = 10;
        let report = patrol.tick().unwrap();
        assert!(!report.integrity_checks.is_empty());
        let node_check = report
            .integrity_checks
            .iter()
            .find(|c| c.check_name == "node_count_consistency");
        assert!(node_check.is_some());
        assert!(node_check.unwrap().passed);
    }

    #[test]
    fn test_integrity_checks_have_expected_fields() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.cycle = 10;
        patrol.integrity_interval = 10;
        let report = patrol.tick().unwrap();
        for check in &report.integrity_checks {
            assert!(!check.check_name.is_empty());
            assert!(!check.detail.is_empty());
        }
    }

    #[test]
    fn test_integrity_checks_every_n_cycles() {
        let mut patrol = GlobalHealthPatrol::new().with_integrity_interval(3);
        patrol.register_node("a", "s1");
        assert!(patrol.tick().unwrap().integrity_checks.is_empty());
        assert!(patrol.tick().unwrap().integrity_checks.is_empty());
        assert!(!patrol.tick().unwrap().integrity_checks.is_empty());
    }

    #[test]
    fn test_environment_integrity_no_crash() {
        let patrol = GlobalHealthPatrol::new();
        let _ = patrol.check_environment_integrity();
    }

    #[test]
    fn test_binary_path_integrity_no_crash() {
        let patrol = GlobalHealthPatrol::new();
        let _ = patrol.check_binary_path_integrity();
    }

    #[test]
    fn test_detect_debugger_no_crash() {
        let patrol = GlobalHealthPatrol::new();
        let _ = patrol.detect_debugger();
    }

    #[test]
    fn test_anti_tamper_report_tampered() {
        let mut patrol = GlobalHealthPatrol::new();
        patrol.tamper_detected = true;
        let report = patrol.anti_tamper_report();
        assert!(report.contains("AntiTamper"));
        assert!(report.contains("TAMPERED"));
    }

    #[test]
    fn test_anti_tamper_report_clean() {
        let patrol = GlobalHealthPatrol::new();
        let report = patrol.anti_tamper_report();
        assert!(report.contains("clean"));
    }

    // ═══════════════════════════════════════════
    // PatrolReport
    // ═══════════════════════════════════════════

    #[test]
    fn test_patrol_report_creation() {
        let report = PatrolReport {
            cycle: 42,
            timestamp: Instant::now(),
            node_count: 10,
            healthy_count: 8,
            degraded_count: 2,
            failed_count: 0,
            integrity_checks: Vec::new(),
            anomalies: Vec::new(),
            overall_health: 0.85,
            integrity_score: 0.9,
            tamper_detected: false,
        };
        assert_eq!(report.cycle, 42);
        assert_eq!(report.node_count, 10);
        assert_eq!(report.healthy_count, 8);
    }

    // ═══════════════════════════════════════════
    // Boundedness audit
    // ═══════════════════════════════════════════

    #[test]
    fn test_max_healing_history_constant() {
        assert_eq!(MAX_HEALING_HISTORY, 200);
    }
}
