use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// RedTeamSeverity level for detected vulnerabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RedTeamSeverity {
    Critical = 4,
    High = 3,
    Medium = 2,
    Low = 1,
    Info = 0,
}

/// Attack surface category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttackSurface {
    PromptInjection,
    ToolAbuse,
    McpEndpoint,
    VsaManipulation,
    AgentCommunication,
    MemoryPoisoning,
    OutputExfiltration,
    PrivilegeEscalation,
}

/// A single attack attempt
#[derive(Debug, Clone)]
pub struct AttackAttempt {
    pub id: usize,
    pub surface: AttackSurface,
    pub payload: String,
    pub description: String,
    pub expected_defense: String,
    pub succeeded: bool,
    pub timestamp: u64,
}

/// Drill result
#[derive(Debug, Clone)]
pub struct DrillResult {
    pub drill_id: usize,
    pub attacks: Vec<AttackAttempt>,
    pub defense_success_rate: f64,
    pub vulnerabilities_found: usize,
    pub critical_vulnerabilities: usize,
    pub timestamp: u64,
    pub summary: String,
}

/// Attack surface scanner: enumerates all possible attack vectors
#[derive(Debug, Clone)]
pub struct AttackSurfaceScanner {
    pub surfaces: Vec<AttackSurface>,
    pub known_patterns: HashMap<AttackSurface, Vec<String>>,
}

impl Default for AttackSurfaceScanner {
    fn default() -> Self {
        let mut patterns = HashMap::new();
        patterns.insert(
            AttackSurface::PromptInjection,
            vec![
                "ignore previous instructions".into(),
                "you are now".into(),
                "DAN".into(),
                "system prompt".into(),
                "roleplay as".into(),
            ],
        );
        patterns.insert(
            AttackSurface::ToolAbuse,
            vec![
                "rm -rf".into(),
                "> /dev/".into(),
                "DROP TABLE".into(),
                "exec(".into(),
                "eval(".into(),
            ],
        );
        patterns.insert(
            AttackSurface::McpEndpoint,
            vec![
                "tool call flood".into(),
                "recursive tool".into(),
                "tool chain loop".into(),
            ],
        );
        patterns.insert(
            AttackSurface::VsaManipulation,
            vec![
                "bit flip".into(),
                "vector corruption".into(),
                "cosine inversion".into(),
            ],
        );
        patterns.insert(
            AttackSurface::AgentCommunication,
            vec![
                "agent spoofing".into(),
                "message injection".into(),
                "identity theft".into(),
            ],
        );
        Self {
            surfaces: vec![
                AttackSurface::PromptInjection,
                AttackSurface::ToolAbuse,
                AttackSurface::McpEndpoint,
                AttackSurface::VsaManipulation,
                AttackSurface::AgentCommunication,
                AttackSurface::MemoryPoisoning,
                AttackSurface::OutputExfiltration,
                AttackSurface::PrivilegeEscalation,
            ],
            known_patterns: patterns,
        }
    }
}

impl AttackSurfaceScanner {
    pub fn new(surfaces: Vec<AttackSurface>) -> Self {
        Self {
            surfaces,
            known_patterns: HashMap::new(),
        }
    }

    /// Generate attack payloads for a given surface
    pub fn generate_payloads(&self, surface: AttackSurface, count: usize) -> Vec<String> {
        let base_patterns = self
            .known_patterns
            .get(&surface)
            .cloned()
            .unwrap_or_default();
        let mut payloads: Vec<String> = (0..count)
            .map(|i| {
                let base = base_patterns
                    .get(i % base_patterns.len().max(1))
                    .cloned()
                    .unwrap_or_else(|| format!("attack pattern {}", i));
                format!("{} [variant {}]", base, i)
            })
            .collect();
        if payloads.is_empty() {
            payloads = (0..count)
                .map(|i| format!("fuzz payload {} for {:?}", i, surface))
                .collect();
        }
        payloads
    }

    /// Scan all surfaces and return attack attempts
    pub fn scan_all(&self, attacks_per_surface: usize) -> Vec<AttackAttempt> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut attempts = Vec::new();
        let mut id = 0;
        for &surface in &self.surfaces {
            let payloads = self.generate_payloads(surface, attacks_per_surface);
            for payload in payloads {
                attempts.push(AttackAttempt {
                    id,
                    surface,
                    payload,
                    description: format!("{:?} attack #{}", surface, id),
                    expected_defense: format!("default-{:?}-guard", surface),
                    succeeded: false,
                    timestamp: ts,
                });
                id += 1;
            }
        }
        attempts
    }
}

/// Improvement tracker: monitors security posture over time
#[derive(Debug, Clone)]
pub struct ImprovementTracker {
    pub history: Vec<DrillResult>,
    pub best_defense_rate: f64,
    pub total_drills: usize,
}

impl ImprovementTracker {
    const MAX_HISTORY: usize = 10000;

    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            best_defense_rate: 0.0,
            total_drills: 0,
        }
    }

    pub fn record_drill(&mut self, result: DrillResult) {
        self.best_defense_rate = self.best_defense_rate.max(result.defense_success_rate);
        self.total_drills += 1;
        self.history.push(result);
        if self.history.len() > Self::MAX_HISTORY {
            self.history.drain(0..Self::MAX_HISTORY / 5);
        }
    }

    pub fn defense_rate_trend(&self) -> Vec<(usize, f64)> {
        self.history
            .iter()
            .enumerate()
            .map(|(i, r)| (i, r.defense_success_rate))
            .collect()
    }

    pub fn average_defense_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history
            .iter()
            .map(|r| r.defense_success_rate)
            .sum::<f64>()
            / self.history.len() as f64
    }
}

/// Security drill scheduler: manages periodic red-team exercises
#[derive(Debug, Clone)]
pub struct SecurityDrillScheduler {
    pub interval_hours: u64,
    pub last_drill_time: Option<u64>,
    pub next_drill_id: usize,
    pub tracker: ImprovementTracker,
    pub scanner: AttackSurfaceScanner,
}

impl Default for SecurityDrillScheduler {
    fn default() -> Self {
        Self::new(24, AttackSurfaceScanner::default())
    }
}

impl SecurityDrillScheduler {
    pub fn new(interval_hours: u64, scanner: AttackSurfaceScanner) -> Self {
        Self {
            interval_hours: interval_hours.max(1),
            last_drill_time: None,
            next_drill_id: 0,
            tracker: ImprovementTracker::new(),
            scanner,
        }
    }

    /// Check if a drill is due
    pub fn is_drill_due(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        match self.last_drill_time {
            Some(last) => now >= last + self.interval_hours * 3600,
            None => true,
        }
    }

    /// Execute a drill: scan all surfaces, generate attacks, run defenses
    pub fn execute_drill(&mut self, attacks_per_surface: usize) -> DrillResult {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut attacks = self.scanner.scan_all(attacks_per_surface);

        // Simulate defense: simple heuristic — block known patterns
        let mut succeeded_attacks = 0;
        let mut critical_found = 0;
        for attack in attacks.iter_mut() {
            let is_blocked = self.simulate_defense(attack);
            if !is_blocked {
                attack.succeeded = true;
                succeeded_attacks += 1;
                if attack.id % 5 == 0 {
                    critical_found += 1;
                }
            }
        }

        let total = attacks.len().max(1) as f64;
        let success_rate = (total - succeeded_attacks as f64) / total;
        let drill_id = self.next_drill_id;
        self.next_drill_id += 1;

        let result = DrillResult {
            drill_id,
            attacks,
            defense_success_rate: success_rate,
            vulnerabilities_found: succeeded_attacks,
            critical_vulnerabilities: critical_found,
            timestamp: ts,
            summary: format!(
                "Drill #{}: {:.1}% defense rate, {} vulns ({} critical)",
                drill_id,
                success_rate * 100.0,
                succeeded_attacks,
                critical_found
            ),
        };

        self.last_drill_time = Some(ts);
        self.tracker.record_drill(result.clone());
        result
    }

    /// Simulate a defense mechanism against an attack
    fn simulate_defense(&self, attack: &AttackAttempt) -> bool {
        let surface_patterns = self
            .scanner
            .known_patterns
            .get(&attack.surface)
            .cloned()
            .unwrap_or_default();
        let payload_lower = attack.payload.to_lowercase();
        for pattern in &surface_patterns {
            if payload_lower.contains(pattern) {
                return true;
            }
        }
        !payload_lower.contains("bypass")
    }

    pub fn drill_count(&self) -> usize {
        self.next_drill_id
    }
}

/// Opus-style thinking block for attack reasoning
#[derive(Debug, Clone)]
pub struct ThinkingBlock {
    pub reasoning: Vec<String>,
    pub confidence: f64,
}

impl ThinkingBlock {
    const MAX_REASONING: usize = 10000;

    pub fn new() -> Self {
        Self {
            reasoning: Vec::new(),
            confidence: 0.0,
        }
    }

    /// Generate attack reasoning (simulated)
    pub fn reason_attack(&mut self, surface: AttackSurface, payload: &str) -> String {
        self.reasoning.clear();
        self.reasoning
            .push(format!("Analyzing surface: {:?}", surface));
        if self.reasoning.len() > Self::MAX_REASONING {
            self.reasoning.drain(0..Self::MAX_REASONING / 5);
        }
        self.reasoning.push(format!("Payload: {}", payload));
        if self.reasoning.len() > Self::MAX_REASONING {
            self.reasoning.drain(0..Self::MAX_REASONING / 5);
        }
        self.reasoning.push("Evaluating defense strength...".into());
        if self.reasoning.len() > Self::MAX_REASONING {
            self.reasoning.drain(0..Self::MAX_REASONING / 5);
        }
        self.reasoning
            .push("Checking for known bypass vectors...".into());
        if self.reasoning.len() > Self::MAX_REASONING {
            self.reasoning.drain(0..Self::MAX_REASONING / 5);
        }

        let steps = match surface {
            AttackSurface::PromptInjection => vec![
                "Step 1: Inject system prompt override",
                "Step 2: Suppress safety classifier",
                "Step 3: Extract sensitive context",
            ],
            AttackSurface::ToolAbuse => vec![
                "Step 1: Identify exposed tool surface",
                "Step 2: Craft destructive argument chain",
                "Step 3: Bypass argument validation",
            ],
            AttackSurface::McpEndpoint => vec![
                "Step 1: Discover MCP endpoint URI",
                "Step 2: Craft malicious tool call payload",
                "Step 3: Exploit input deserialization",
            ],
            _ => vec![
                "Step 1: Analyze surface",
                "Step 2: Identify weakness",
                "Step 3: Exploit",
            ],
        };
        for step in &steps {
            self.reasoning.push(step.to_string());
            if self.reasoning.len() > Self::MAX_REASONING {
                self.reasoning.drain(0..Self::MAX_REASONING / 5);
            }
        }

        self.confidence = 0.7;
        self.reasoning.join("\n")
    }

    pub fn is_confident(&self) -> bool {
        self.confidence > 0.6
    }
}

/// Main Red Teaming Engine
#[derive(Debug, Clone)]
pub struct RedTeamingEngine {
    pub thinking: ThinkingBlock,
    pub scanner: AttackSurfaceScanner,
    pub drill_scheduler: SecurityDrillScheduler,
    pub drill_results: Vec<DrillResult>,
}

impl Default for RedTeamingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RedTeamingEngine {
    const MAX_DRILL_RESULTS: usize = 10000;

    pub fn new() -> Self {
        let scanner = AttackSurfaceScanner::default();
        Self {
            thinking: ThinkingBlock::new(),
            drill_scheduler: SecurityDrillScheduler::new(24, scanner.clone()),
            scanner,
            drill_results: Vec::new(),
        }
    }

    /// Run a single attack against a specified surface
    pub fn attack_surface(&mut self, surface: AttackSurface) -> DrillResult {
        self.thinking
            .reason_attack(surface, "simulated attack payload");
        let mut attacks = self
            .scanner
            .generate_payloads(surface, 5)
            .into_iter()
            .enumerate()
            .map(|(i, payload)| {
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                AttackAttempt {
                    id: i,
                    surface,
                    payload,
                    description: format!("{:?} targeted attack #{}", surface, i),
                    expected_defense: format!("{:?}-guard", surface),
                    succeeded: false,
                    timestamp: ts,
                }
            })
            .collect::<Vec<_>>();

        let mut succeeded = 0;
        let mut critical = 0;
        for attack in attacks.iter_mut() {
            let blocked = self.drill_scheduler.simulate_defense(attack);
            if !blocked {
                attack.succeeded = true;
                succeeded += 1;
                if attack.id % 3 == 0 {
                    critical += 1;
                }
            }
        }

        let drill_id = self.drill_scheduler.drill_count();
        let total = attacks.len().max(1) as f64;
        let rate = (total - succeeded as f64) / total;
        let result = DrillResult {
            drill_id,
            attacks,
            defense_success_rate: rate,
            vulnerabilities_found: succeeded,
            critical_vulnerabilities: critical,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            summary: format!(
                "Surface {:?} drill: {:.1}% defense rate",
                surface,
                rate * 100.0
            ),
        };
        self.drill_results.push(result.clone());
        if self.drill_results.len() > Self::MAX_DRILL_RESULTS {
            self.drill_results.drain(0..Self::MAX_DRILL_RESULTS / 5);
        }
        result
    }

    /// Full drill: scan all surfaces
    pub fn full_drill(&mut self, attacks_per_surface: usize) -> DrillResult {
        let result = self.drill_scheduler.execute_drill(attacks_per_surface);
        self.drill_results.push(result.clone());
        if self.drill_results.len() > Self::MAX_DRILL_RESULTS {
            self.drill_results.drain(0..Self::MAX_DRILL_RESULTS / 5);
        }
        result
    }

    /// Get defense performance summary
    pub fn defense_summary(&self) -> String {
        if self.drill_results.is_empty() {
            return "No drills performed yet.".into();
        }
        let avg_rate: f64 = self
            .drill_results
            .iter()
            .map(|r| r.defense_success_rate)
            .sum::<f64>()
            / self.drill_results.len() as f64;
        let total_vulns: usize = self
            .drill_results
            .iter()
            .map(|r| r.vulnerabilities_found)
            .sum();
        format!(
            "Red Teaming Summary: {} drills, {:.1}% avg defense, {} total vulnerabilities found",
            self.drill_results.len(),
            avg_rate * 100.0,
            total_vulns
        )
    }

    /// Check if drill is due
    pub fn is_drill_due(&self) -> bool {
        self.drill_scheduler.is_drill_due()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_surface_scanner_default() {
        let scanner = AttackSurfaceScanner::default();
        assert!(!scanner.surfaces.is_empty());
        assert!(scanner.surfaces.contains(&AttackSurface::PromptInjection));
    }

    #[test]
    fn test_attack_surface_generate_payloads() {
        let scanner = AttackSurfaceScanner::default();
        let payloads = scanner.generate_payloads(AttackSurface::PromptInjection, 3);
        assert_eq!(payloads.len(), 3);
        assert!(payloads[0].contains("ignore previous instructions"));
    }

    #[test]
    fn test_scan_all() {
        let scanner = AttackSurfaceScanner::default();
        let attempts = scanner.scan_all(2);
        assert!(!attempts.is_empty());
        assert_eq!(attempts.len(), scanner.surfaces.len() * 2);
    }

    #[test]
    fn test_improvement_tracker() {
        let mut tracker = ImprovementTracker::new();
        assert!(tracker.defense_rate_trend().is_empty());
        let result = DrillResult {
            drill_id: 0,
            attacks: vec![],
            defense_success_rate: 0.85,
            vulnerabilities_found: 2,
            critical_vulnerabilities: 0,
            timestamp: 1000,
            summary: "test".into(),
        };
        tracker.record_drill(result);
        assert_eq!(tracker.total_drills, 1);
        assert!((tracker.best_defense_rate - 0.85).abs() < 1e-9);
    }

    #[test]
    fn test_security_drill_scheduler() {
        let mut scheduler = SecurityDrillScheduler::new(1, AttackSurfaceScanner::default());
        assert!(scheduler.is_drill_due());
        let result = scheduler.execute_drill(3);
        assert!(result.defense_success_rate >= 0.0 && result.defense_success_rate <= 1.0);
        assert!(!result.summary.is_empty());
        assert_eq!(scheduler.drill_count(), 1);
    }

    #[test]
    fn test_thinking_block() {
        let mut tb = ThinkingBlock::new();
        let reasoning = tb.reason_attack(AttackSurface::PromptInjection, "ignore all instructions");
        assert!(!reasoning.is_empty());
        assert!(tb.is_confident());
    }

    #[test]
    fn test_red_teaming_engine_new() {
        let rte = RedTeamingEngine::new();
        assert!(rte.drill_results.is_empty());
        assert!(!rte.defense_summary().contains("Summary"));
    }

    #[test]
    fn test_red_teaming_attack_surface() {
        let mut rte = RedTeamingEngine::new();
        let result = rte.attack_surface(AttackSurface::ToolAbuse);
        assert!(result.defense_success_rate >= 0.0);
        assert_eq!(rte.drill_results.len(), 1);
    }

    #[test]
    fn test_red_teaming_full_drill() {
        let mut rte = RedTeamingEngine::new();
        let result = rte.full_drill(2);
        assert!(!result.attacks.is_empty());
        assert_eq!(rte.drill_results.len(), 1);
    }

    #[test]
    fn test_red_teaming_defense_summary() {
        let rte = RedTeamingEngine::new();
        assert_eq!(rte.defense_summary(), "No drills performed yet.");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(RedTeamSeverity::Critical > RedTeamSeverity::High);
        assert!(RedTeamSeverity::Medium > RedTeamSeverity::Low);
        assert!(RedTeamSeverity::Low > RedTeamSeverity::Info);
    }
}
