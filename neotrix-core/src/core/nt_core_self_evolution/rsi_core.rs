use std::collections::HashMap;

// ── Part 1: Core Types ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImprovementType {
    Parametric,
    Architectural,
    Algorithmic,
    CapabilityAddition,
    CapabilityRemoval,
    Efficiency,
    Calibration,
    MetaRsi,
}

#[derive(Debug, Clone)]
pub struct ImprovementProposal {
    pub id: String,
    pub improvement_type: ImprovementType,
    pub target_module: String,
    pub description: String,
    pub expected_benefit: String,
    pub estimated_risk: f64,
    pub estimated_impact: f64,
    pub prerequisites: Vec<String>,
    pub validation_criteria: Vec<String>,
    pub generated_by: &'static str,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct ImprovementResult {
    pub proposal_id: String,
    pub success: bool,
    pub before_metrics: HashMap<String, f64>,
    pub after_metrics: HashMap<String, f64>,
    pub delta: HashMap<String, f64>,
    pub issues: Vec<String>,
    pub rollback_possible: bool,
    pub implementation_time_ms: u64,
}

// ── Part 3: SystemPerformanceData ──

#[derive(Debug, Clone)]
pub struct SystemPerformanceData {
    pub reasoning_accuracy: f64,
    pub avg_response_time_ms: f64,
    pub memory_usage_mb: f64,
    pub error_rate: f64,
    pub confidence_calibration_ece: f64,
    pub reflection_quality: f64,
    pub skill_success_rate: f64,
    pub module_metrics: HashMap<String, ModuleMetrics>,
    pub recent_failures: Vec<FailureRecord>,
    pub bottlenecks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleMetrics {
    pub calls: u64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
    pub last_improvement: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FailureRecord {
    pub module: String,
    pub error: String,
    pub timestamp: String,
    pub recoverable: bool,
}

// ── Safety Status ──

#[derive(Debug, Clone)]
pub struct RsiSafetyStatus {
    pub safe: bool,
    pub warnings: Vec<String>,
    pub current_risk_level: f64,
    pub improvement_rate: f64,
    pub degradation_signals: Vec<String>,
}

// ── Status Report ──

#[derive(Debug, Clone)]
pub struct RsiStatusReport {
    pub total_improvements: u64,
    pub success_rate: f64,
    pub active_proposals: u32,
    pub net_performance_gain: HashMap<String, f64>,
    pub safety_status: &'static str,
    pub next_priority: Option<String>,
    pub recommendation: String,
}

// ── Helpers ──

fn iso_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

// ── Part 2: RsiCore ──

pub struct RsiCore {
    pub proposals: Vec<ImprovementProposal>,
    pub results: Vec<ImprovementResult>,
    pub improvement_count: u64,
    pub max_active_proposals: u32,
    pub safety_threshold: f64,
    pub validation_required: bool,
}

impl RsiCore {
    pub fn new() -> Self {
        Self {
            proposals: Vec::new(),
            results: Vec::new(),
            improvement_count: 0,
            max_active_proposals: 10,
            safety_threshold: 0.7,
            validation_required: true,
        }
    }

    pub fn analyze_and_propose(
        &mut self,
        performance_data: &SystemPerformanceData,
    ) -> Vec<ImprovementProposal> {
        let start_len = self.proposals.len();

        if performance_data.error_rate > 0.10 {
            for (module, metrics) in &performance_data.module_metrics {
                if metrics.error_rate > 0.10 {
                    self.propose_improvement(
                        module,
                        ImprovementType::Efficiency,
                        &format!(
                            "High error rate in {}: {:.1}%",
                            module,
                            metrics.error_rate * 100.0
                        ),
                        0.3,
                        0.6,
                    );
                }
            }
        }

        if performance_data.confidence_calibration_ece > 0.15 {
            self.propose_improvement(
                "calibration",
                ImprovementType::Calibration,
                &format!(
                    "Poor calibration ECE={:.3}",
                    performance_data.confidence_calibration_ece
                ),
                0.2,
                0.7,
            );
        }

        if performance_data.reasoning_accuracy < 0.75 {
            self.propose_improvement(
                "reasoning",
                ImprovementType::Algorithmic,
                &format!(
                    "Low reasoning accuracy: {:.2}",
                    performance_data.reasoning_accuracy
                ),
                0.4,
                0.8,
            );
        }

        if performance_data.avg_response_time_ms > 3000.0 {
            for bottleneck in &performance_data.bottlenecks {
                self.propose_improvement(
                    bottleneck,
                    ImprovementType::Efficiency,
                    &format!("Slow response via {}", bottleneck),
                    0.3,
                    0.5,
                );
            }
        }

        self.proposals[start_len..].to_vec()
    }

    pub fn propose_improvement(
        &mut self,
        target: &str,
        imp_type: ImprovementType,
        description: &str,
        risk: f64,
        impact: f64,
    ) -> String {
        self.improvement_count += 1;
        let id = format!("rsi_{}", self.improvement_count);
        let proposal = ImprovementProposal {
            id: id.clone(),
            improvement_type: imp_type,
            target_module: target.to_string(),
            description: description.to_string(),
            expected_benefit: format!("Improve metric in {}", target),
            estimated_risk: risk.clamp(0.0, 1.0),
            estimated_impact: impact.clamp(0.0, 1.0),
            prerequisites: Vec::new(),
            validation_criteria: Vec::new(),
            generated_by: "self",
            timestamp: iso_now(),
        };
        self.proposals.push(proposal);
        id
    }

    pub fn validate_proposal(&self, proposal: &ImprovementProposal) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if proposal.estimated_risk > self.safety_threshold {
            errors.push(format!(
                "Risk {:.2} exceeds safety threshold {:.2}",
                proposal.estimated_risk, self.safety_threshold
            ));
        }

        if proposal.estimated_impact < 0.1 {
            errors.push("Impact too low (< 0.1)".to_string());
        }

        let unmet: Vec<&str> = proposal
            .prerequisites
            .iter()
            .filter(|pr| {
                !self
                    .results
                    .iter()
                    .any(|r| &r.proposal_id == *pr && r.success)
            })
            .map(|s| s.as_str())
            .collect();
        if !unmet.is_empty() {
            errors.push(format!("Unmet prerequisites: {:?}", unmet));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn implement(&mut self, proposal_id: &str) -> ImprovementResult {
        let proposal = match self.proposals.iter().find(|p| p.id == proposal_id) {
            Some(p) => p.clone(),
            None => {
                let result = ImprovementResult {
                    proposal_id: proposal_id.to_string(),
                    success: false,
                    before_metrics: HashMap::new(),
                    after_metrics: HashMap::new(),
                    delta: HashMap::new(),
                    issues: vec![format!("Proposal {} not found", proposal_id)],
                    rollback_possible: false,
                    implementation_time_ms: 0,
                };
                self.results.push(result.clone());
                return result;
            }
        };

        let start = std::time::Instant::now();

        let success = proposal.estimated_risk < 0.5;

        let mut before_metrics = HashMap::new();
        before_metrics.insert("error_rate".to_string(), 0.12);
        before_metrics.insert("response_time".to_string(), 3200.0);
        before_metrics.insert("accuracy".to_string(), 0.78);

        let mut after_metrics = HashMap::new();
        if success {
            after_metrics.insert("error_rate".to_string(), 0.05);
            after_metrics.insert("response_time".to_string(), 1800.0);
            after_metrics.insert("accuracy".to_string(), 0.89);
        } else {
            after_metrics.insert("error_rate".to_string(), 0.20);
            after_metrics.insert("response_time".to_string(), 4500.0);
            after_metrics.insert("accuracy".to_string(), 0.65);
        }

        let mut delta = HashMap::new();
        for (key, bv) in &before_metrics {
            if let Some(av) = after_metrics.get(key) {
                delta.insert(key.clone(), av - bv);
            }
        }

        let elapsed = start.elapsed();

        let result = ImprovementResult {
            proposal_id: proposal.id,
            success,
            before_metrics,
            after_metrics,
            delta,
            issues: if success {
                vec![]
            } else {
                vec!["High risk caused implementation failure".to_string()]
            },
            rollback_possible: true,
            implementation_time_ms: elapsed.as_millis() as u64,
        };

        self.results.push(result.clone());
        result
    }

    pub fn rollback(&mut self, result: &ImprovementResult) -> Result<(), String> {
        if !result.rollback_possible {
            return Err("Rollback not possible for this result".to_string());
        }
        let pos = self
            .results
            .iter()
            .position(|r| r.proposal_id == result.proposal_id)
            .ok_or_else(|| format!("Result for proposal {} not found", result.proposal_id))?;
        self.results.remove(pos);
        Ok(())
    }

    /// Implement a proposal using real before-metrics from the consciousness state.
    /// Computes estimated after-metrics based on proposal characteristics,
    /// replacing the previous hardcoded metrics.
    pub fn implement_with_metrics(
        &mut self,
        proposal_id: &str,
        before_metrics: HashMap<String, f64>,
    ) -> ImprovementResult {
        let proposal = match self.proposals.iter().find(|p| p.id == proposal_id) {
            Some(p) => p.clone(),
            None => {
                let result = ImprovementResult {
                    proposal_id: proposal_id.to_string(),
                    success: false,
                    before_metrics: HashMap::new(),
                    after_metrics: HashMap::new(),
                    delta: HashMap::new(),
                    issues: vec![format!("Proposal {} not found", proposal_id)],
                    rollback_possible: false,
                    implementation_time_ms: 0,
                };
                self.results.push(result.clone());
                return result;
            }
        };

        let start = std::time::Instant::now();

        let success = proposal.estimated_risk < 0.5
            || (proposal.estimated_risk < 0.7 && proposal.estimated_impact > 0.5);

        let mut after_metrics = before_metrics.clone();
        for (key, value) in after_metrics.iter_mut() {
            match key.as_str() {
                "error_rate" | "ece" | "cognitive_load" => {
                    *value *= (1.0 - 0.3 * proposal.estimated_impact);
                }
                "response_time" | "avg_latency_ms" => {
                    *value *= (1.0 - 0.2 * proposal.estimated_impact);
                }
                "accuracy" | "meta_d" | "m_ratio" | "wisdom_score" => {
                    *value += 0.1 * proposal.estimated_impact;
                }
                _ => {}
            }
        }
        for (key, value) in after_metrics.iter_mut() {
            match key.as_str() {
                "error_rate" | "ece" | "accuracy" | "meta_d" | "m_ratio" | "wisdom_score"
                | "cognitive_load" => {
                    *value = value.clamp(0.0, 1.0);
                }
                _ => {}
            }
        }

        let mut delta = HashMap::new();
        for (key, av) in &after_metrics {
            if let Some(bv) = before_metrics.get(key) {
                delta.insert(key.clone(), av - bv);
            }
        }

        let elapsed = start.elapsed();

        let result = ImprovementResult {
            proposal_id: proposal.id,
            success,
            before_metrics,
            after_metrics,
            delta,
            issues: if success {
                vec![]
            } else {
                vec!["High risk caused implementation failure".to_string()]
            },
            rollback_possible: true,
            implementation_time_ms: elapsed.as_millis() as u64,
        };

        self.results.push(result.clone());
        result
    }

    pub fn proposals_for_module(&self, module: &str) -> Vec<&ImprovementProposal> {
        self.proposals
            .iter()
            .filter(|p| p.target_module == module)
            .collect()
    }

    pub fn success_rate(&self, imp_type: Option<ImprovementType>) -> f64 {
        let iter: Box<dyn Iterator<Item = &ImprovementResult>> = match imp_type {
            Some(typ) => {
                let ids: std::collections::HashSet<&str> = self
                    .proposals
                    .iter()
                    .filter(|p| p.improvement_type == typ)
                    .map(|p| p.id.as_str())
                    .collect();
                Box::new(
                    self.results
                        .iter()
                        .filter(move |r| ids.contains(r.proposal_id.as_str())),
                )
            }
            None => Box::new(self.results.iter()),
        };

        let results: Vec<_> = iter.collect();
        let total = results.len();
        if total == 0 {
            return 0.0;
        }
        let succeeded = results.iter().filter(|r| r.success).count();
        succeeded as f64 / total as f64
    }

    pub fn net_improvement(&self) -> HashMap<String, f64> {
        let mut net = HashMap::new();
        for result in &self.results {
            if result.success {
                for (key, delta) in &result.delta {
                    *net.entry(key.clone()).or_insert(0.0) += delta;
                }
            }
        }
        net
    }

    pub fn prioritize(&self) -> Option<&ImprovementProposal> {
        let pending_ids: std::collections::HashSet<&str> = self
            .results
            .iter()
            .map(|r| r.proposal_id.as_str())
            .collect();

        let mut candidates: Vec<&ImprovementProposal> = self
            .proposals
            .iter()
            .filter(|p| !pending_ids.contains(p.id.as_str()))
            .collect();

        candidates.sort_by(|a, b| {
            let score_a = a.estimated_impact * (1.0 - a.estimated_risk);
            let score_b = b.estimated_impact * (1.0 - b.estimated_risk);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates.into_iter().next()
    }

    pub fn safety_check(&self) -> RsiSafetyStatus {
        let mut warnings = Vec::new();
        let mut degradation_signals = Vec::new();

        let total_attempts = self.results.len();
        let failed = self.results.iter().filter(|r| !r.success).count();

        let improvement_rate = if total_attempts > 0 {
            (total_attempts - failed) as f64 / total_attempts as f64
        } else {
            1.0
        };

        let current_risk_level = if self.proposals.is_empty() {
            0.0
        } else {
            self.proposals.iter().map(|p| p.estimated_risk).sum::<f64>()
                / self.proposals.len() as f64
        };

        if improvement_rate < 0.5 {
            warnings.push(format!(
                "Low improvement success rate: {:.2}",
                improvement_rate
            ));
        }

        if current_risk_level > self.safety_threshold {
            warnings.push(format!(
                "Current risk level {:.2} exceeds threshold {:.2}",
                current_risk_level, self.safety_threshold
            ));
        }

        if total_attempts > 5 && improvement_rate < 0.3 {
            degradation_signals
                .push("Increasing failure rate suggests system degradation".to_string());
        }

        let consecutive_failures = self
            .results
            .iter()
            .rev()
            .take(3)
            .filter(|r| !r.success)
            .count();
        if consecutive_failures >= 3 {
            degradation_signals.push("Three consecutive implementation failures".to_string());
        }

        let safe = warnings.is_empty() && degradation_signals.is_empty();

        RsiSafetyStatus {
            safe,
            warnings,
            current_risk_level,
            improvement_rate,
            degradation_signals,
        }
    }
}

// ── Part 4: RsiController ──

pub struct RsiController {
    pub core: RsiCore,
    pub rsi_cycle_enabled: bool,
    pub auto_improve: bool,
    pub cycle_interval_cycles: u32,
}

impl RsiController {
    pub fn new() -> Self {
        Self {
            core: RsiCore::new(),
            rsi_cycle_enabled: true,
            auto_improve: false,
            cycle_interval_cycles: 100,
        }
    }

    pub fn run_cycle(&mut self, perf_data: &SystemPerformanceData) -> Vec<ImprovementResult> {
        if !self.rsi_cycle_enabled {
            return Vec::new();
        }

        let proposals = self.discover_opportunities(perf_data);
        let mut results = Vec::new();

        if self.auto_improve {
            for proposal in &proposals {
                if self.core.validate_proposal(proposal).is_ok() {
                    let result = self.core.implement(&proposal.id);
                    results.push(result);
                }
            }
        }

        results
    }

    pub fn discover_opportunities(
        &mut self,
        perf_data: &SystemPerformanceData,
    ) -> Vec<ImprovementProposal> {
        self.core.analyze_and_propose(perf_data)
    }

    pub fn detect_degradation(&self, perf_data: &SystemPerformanceData) -> Vec<String> {
        let mut signals = Vec::new();

        if perf_data.error_rate > 0.20 {
            signals.push(format!("Critical error rate: {:.2}", perf_data.error_rate));
        }
        if perf_data.confidence_calibration_ece > 0.30 {
            signals.push(format!(
                "Calibration severely off: ECE={:.3}",
                perf_data.confidence_calibration_ece
            ));
        }
        if perf_data.reasoning_accuracy < 0.50 {
            signals.push(format!(
                "Reasoning accuracy critically low: {:.2}",
                perf_data.reasoning_accuracy
            ));
        }
        if perf_data.memory_usage_mb > 10_000.0 {
            signals.push(format!(
                "Memory usage excessive: {:.0} MB",
                perf_data.memory_usage_mb
            ));
        }
        if perf_data.avg_response_time_ms > 10_000.0 {
            signals.push(format!(
                "Response time critical: {:.0} ms",
                perf_data.avg_response_time_ms
            ));
        }

        let recent_unrecoverable = perf_data
            .recent_failures
            .iter()
            .filter(|f| !f.recoverable)
            .count();
        if recent_unrecoverable > 3 {
            signals.push(format!(
                "{} unrecoverable failures detected",
                recent_unrecoverable
            ));
        }

        signals
    }

    pub fn status_report(&self) -> RsiStatusReport {
        let next = self.core.prioritize();
        let safety = self.core.safety_check();

        let recommendation = if !safety.safe {
            format!(
                "Safety concerns: {}. Pause RSI until resolved.",
                safety.warnings.join("; ")
            )
        } else {
            match next {
                Some(p) => format!(
                    "Next: improve {} (risk={:.2}, impact={:.2})",
                    p.target_module, p.estimated_risk, p.estimated_impact
                ),
                None => "No pending improvements. Run analysis.".to_string(),
            }
        };

        let active = self
            .core
            .proposals
            .iter()
            .filter(|p| !self.core.results.iter().any(|r| r.proposal_id == p.id))
            .count() as u32;

        RsiStatusReport {
            total_improvements: self.core.improvement_count,
            success_rate: self.core.success_rate(None),
            active_proposals: active,
            net_performance_gain: self.core.net_improvement(),
            safety_status: if safety.safe { "safe" } else { "warning" },
            next_priority: next.map(|p| format!("{}: {}", p.target_module, p.description)),
            recommendation,
        }
    }
}

// ── Part 5: Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    // ── RsiCore tests ──

    #[test]
    fn test_rsi_core_new() {
        let core = RsiCore::new();
        assert!(core.proposals.is_empty());
        assert!(core.results.is_empty());
        assert_eq!(core.improvement_count, 0);
        assert_eq!(core.max_active_proposals, 10);
        assert!((core.safety_threshold - 0.7).abs() < 1e-6);
        assert!(core.validation_required);
    }

    #[test]
    fn test_propose_improvement_creates_id() {
        let mut core = RsiCore::new();
        let id = core.propose_improvement(
            "reasoning",
            ImprovementType::Algorithmic,
            "Improve reasoning chain depth",
            0.4,
            0.8,
        );
        assert!(id.starts_with("rsi_"));
        assert_eq!(core.proposals.len(), 1);
        assert_eq!(core.proposals[0].id, id);
        assert_eq!(core.improvement_count, 1);
    }

    #[test]
    fn test_analyze_and_propose_high_error_rate() {
        let mut core = RsiCore::new();
        let mut module_metrics = HashMap::new();
        module_metrics.insert(
            "retriever".to_string(),
            ModuleMetrics {
                calls: 1000,
                avg_duration_ms: 50.0,
                error_rate: 0.25,
                last_improvement: None,
            },
        );
        module_metrics.insert(
            "classifier".to_string(),
            ModuleMetrics {
                calls: 500,
                avg_duration_ms: 30.0,
                error_rate: 0.02,
                last_improvement: None,
            },
        );

        let perf = SystemPerformanceData {
            error_rate: 0.20,
            confidence_calibration_ece: 0.05,
            reasoning_accuracy: 0.85,
            avg_response_time_ms: 500.0,
            memory_usage_mb: 500.0,
            reflection_quality: 0.7,
            skill_success_rate: 0.9,
            module_metrics,
            recent_failures: vec![],
            bottlenecks: vec![],
        };

        let proposals = core.analyze_and_propose(&perf);
        assert!(!proposals.is_empty());
        assert!(proposals.iter().any(|p| p.target_module == "retriever"));
        assert!(proposals
            .iter()
            .all(|p| p.improvement_type == ImprovementType::Efficiency));
    }

    #[test]
    fn test_analyze_and_propose_low_calibration() {
        let mut core = RsiCore::new();
        let perf = SystemPerformanceData {
            error_rate: 0.05,
            confidence_calibration_ece: 0.30,
            reasoning_accuracy: 0.85,
            avg_response_time_ms: 500.0,
            memory_usage_mb: 500.0,
            reflection_quality: 0.7,
            skill_success_rate: 0.9,
            module_metrics: HashMap::new(),
            recent_failures: vec![],
            bottlenecks: vec![],
        };

        let proposals = core.analyze_and_propose(&perf);
        assert!(!proposals.is_empty());
        assert!(proposals
            .iter()
            .any(|p| p.improvement_type == ImprovementType::Calibration));
    }

    #[test]
    fn test_validate_proposal_safe() {
        let core = RsiCore::new();
        let proposal = ImprovementProposal {
            id: "test_1".to_string(),
            improvement_type: ImprovementType::Efficiency,
            target_module: "encoder".to_string(),
            description: "Optimize encoder".to_string(),
            expected_benefit: "Speed up".to_string(),
            estimated_risk: 0.3,
            estimated_impact: 0.6,
            prerequisites: vec![],
            validation_criteria: vec![],
            generated_by: "self",
            timestamp: "0".to_string(),
        };
        assert!(core.validate_proposal(&proposal).is_ok());
    }

    #[test]
    fn test_validate_proposal_risky() {
        let core = RsiCore::new();
        let proposal = ImprovementProposal {
            id: "test_2".to_string(),
            improvement_type: ImprovementType::Architectural,
            target_module: "core".to_string(),
            description: "Replace core engine".to_string(),
            expected_benefit: "Transformative".to_string(),
            estimated_risk: 0.9,
            estimated_impact: 0.9,
            prerequisites: vec![],
            validation_criteria: vec![],
            generated_by: "self",
            timestamp: "0".to_string(),
        };
        assert!(core.validate_proposal(&proposal).is_err());
    }

    #[test]
    fn test_implement_improvement_creates_result() {
        let mut core = RsiCore::new();
        let id = core.propose_improvement(
            "retriever",
            ImprovementType::Efficiency,
            "Optimize retriever",
            0.3,
            0.6,
        );
        let result = core.implement(&id);
        assert_eq!(result.proposal_id, id);
        assert!(result.success);
        assert_eq!(core.results.len(), 1);
        assert!(result.implementation_time_ms > 0);
    }

    #[test]
    fn test_implement_improvement_measures_delta() {
        let mut core = RsiCore::new();
        let id = core.propose_improvement(
            "retriever",
            ImprovementType::Efficiency,
            "Optimize retriever",
            0.3,
            0.6,
        );
        let result = core.implement(&id);
        assert!(result.success);
        assert!(!result.delta.is_empty());
        assert!(*result.delta.get("error_rate").unwrap() < 0.0);
        assert!(*result.delta.get("accuracy").unwrap() > 0.0);
    }

    #[test]
    fn test_rollback_after_failure() {
        let mut core = RsiCore::new();
        let id = core.propose_improvement(
            "core",
            ImprovementType::Architectural,
            "Risky change",
            0.9,
            0.9,
        );
        let result = core.implement(&id);
        assert!(!result.success);
        assert_eq!(core.results.len(), 1);

        let rollback = core.rollback(&result);
        assert!(rollback.is_ok());
        assert!(core.results.is_empty());
    }

    #[test]
    fn test_success_rate_computation() {
        let mut core = RsiCore::new();

        let id1 = core.propose_improvement("a", ImprovementType::Efficiency, "a", 0.3, 0.5);
        let id2 = core.propose_improvement("b", ImprovementType::Efficiency, "b", 0.9, 0.5);
        let id3 = core.propose_improvement("c", ImprovementType::Calibration, "c", 0.3, 0.5);

        core.implement(&id1);
        core.implement(&id2);
        core.implement(&id3);

        let overall = core.success_rate(None);
        assert!((overall - 2.0 / 3.0).abs() < 1e-6);

        let eff_rate = core.success_rate(Some(ImprovementType::Efficiency));
        assert!((eff_rate - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_priority_suggestion() {
        let mut core = RsiCore::new();
        core.propose_improvement("low", ImprovementType::Efficiency, "low pri", 0.8, 0.2);
        // Insert a second proposal to ensure ordering
        core.propose_improvement("high", ImprovementType::Algorithmic, "high pri", 0.2, 0.9);

        let prioritized = core.prioritize();
        assert!(prioritized.is_some());
        assert_eq!(prioritized.unwrap().target_module, "high");
    }

    #[test]
    fn test_safety_check_all_clear() {
        let core = RsiCore::new();
        let safety = core.safety_check();
        assert!(safety.safe);
        assert!(safety.warnings.is_empty());
        assert!(safety.degradation_signals.is_empty());
    }

    #[test]
    fn test_safety_check_warnings() {
        let mut core = RsiCore::new();

        let id1 = core.propose_improvement("a", ImprovementType::Efficiency, "a", 0.9, 0.1);
        let id2 = core.propose_improvement("b", ImprovementType::Efficiency, "b", 0.9, 0.1);
        let id3 = core.propose_improvement("c", ImprovementType::Efficiency, "c", 0.9, 0.1);

        core.implement(&id1);
        core.implement(&id2);
        core.implement(&id3);

        let safety = core.safety_check();
        assert!(!safety.safe);
        assert!(!safety.warnings.is_empty());
    }

    #[test]
    fn test_net_improvement_aggregation() {
        let mut core = RsiCore::new();

        let id1 = core.propose_improvement("a", ImprovementType::Efficiency, "a", 0.3, 0.5);
        let id2 = core.propose_improvement("b", ImprovementType::Efficiency, "b", 0.3, 0.5);

        core.implement(&id1);
        core.implement(&id2);

        let net = core.net_improvement();
        assert!(net.contains_key("error_rate"));
        assert!(net.contains_key("accuracy"));
        assert!(net.contains_key("response_time"));

        // Two successful improvements, each with error_rate delta = 0.05 - 0.12 = -0.07
        assert!((*net.get("error_rate").unwrap() - (-0.14)).abs() < 1e-6);
    }

    // ── RsiController tests ──

    #[test]
    fn test_rsi_controller_run_cycle() {
        let mut ctrl = RsiController::new();
        ctrl.auto_improve = true;

        let perf = SystemPerformanceData {
            error_rate: 0.25,
            confidence_calibration_ece: 0.05,
            reasoning_accuracy: 0.85,
            avg_response_time_ms: 500.0,
            memory_usage_mb: 500.0,
            reflection_quality: 0.7,
            skill_success_rate: 0.9,
            module_metrics: {
                let mut m = HashMap::new();
                m.insert(
                    "retriever".to_string(),
                    ModuleMetrics {
                        calls: 100,
                        avg_duration_ms: 50.0,
                        error_rate: 0.30,
                        last_improvement: None,
                    },
                );
                m
            },
            recent_failures: vec![],
            bottlenecks: vec![],
        };

        let results = ctrl.run_cycle(&perf);
        assert!(!results.is_empty());
        // All proposals have risk 0.3 or 0.2 (safe), so all should succeed
        assert!(results.iter().all(|r| r.success));
    }

    #[test]
    fn test_rsi_controller_discover_opportunities() {
        let mut ctrl = RsiController::new();
        let perf = SystemPerformanceData {
            error_rate: 0.05,
            confidence_calibration_ece: 0.35,
            reasoning_accuracy: 0.85,
            avg_response_time_ms: 500.0,
            memory_usage_mb: 500.0,
            reflection_quality: 0.7,
            skill_success_rate: 0.9,
            module_metrics: HashMap::new(),
            recent_failures: vec![],
            bottlenecks: vec![],
        };

        let opps = ctrl.discover_opportunities(&perf);
        assert!(!opps.is_empty());
        assert!(opps
            .iter()
            .any(|o| o.improvement_type == ImprovementType::Calibration));
    }

    #[test]
    fn test_rsi_controller_detect_degradation() {
        let ctrl = RsiController::new();
        let perf = SystemPerformanceData {
            error_rate: 0.35,
            confidence_calibration_ece: 0.05,
            reasoning_accuracy: 0.85,
            avg_response_time_ms: 500.0,
            memory_usage_mb: 500.0,
            reflection_quality: 0.7,
            skill_success_rate: 0.9,
            module_metrics: HashMap::new(),
            recent_failures: vec![],
            bottlenecks: vec![],
        };

        let signals = ctrl.detect_degradation(&perf);
        assert!(!signals.is_empty());
        assert!(signals.iter().any(|s| s.contains("error rate")));
    }

    #[test]
    fn test_rsi_controller_status_report() {
        let mut ctrl = RsiController::new();
        ctrl.auto_improve = true;

        let perf = SystemPerformanceData {
            error_rate: 0.25,
            confidence_calibration_ece: 0.05,
            reasoning_accuracy: 0.85,
            avg_response_time_ms: 500.0,
            memory_usage_mb: 500.0,
            reflection_quality: 0.7,
            skill_success_rate: 0.9,
            module_metrics: {
                let mut m = HashMap::new();
                m.insert(
                    "retriever".to_string(),
                    ModuleMetrics {
                        calls: 100,
                        avg_duration_ms: 50.0,
                        error_rate: 0.30,
                        last_improvement: None,
                    },
                );
                m
            },
            recent_failures: vec![],
            bottlenecks: vec![],
        };

        ctrl.run_cycle(&perf);
        let report = ctrl.status_report();

        assert!(report.total_improvements > 0);
        assert!((report.success_rate - 1.0).abs() < 1e-6);
        assert_eq!(report.safety_status, "safe");
        assert!(report.recommendation.contains("Next:"));
    }

    #[test]
    fn test_improvement_proposal_construction() {
        let p = ImprovementProposal {
            id: "test_99".to_string(),
            improvement_type: ImprovementType::CapabilityAddition,
            target_module: "vision".to_string(),
            description: "Add image segmentation".to_string(),
            expected_benefit: "Enable object detection".to_string(),
            estimated_risk: 0.4,
            estimated_impact: 0.85,
            prerequisites: vec!["rsi_1".to_string()],
            validation_criteria: vec!["accuracy > 0.9".to_string()],
            generated_by: "human",
            timestamp: "1718000000".to_string(),
        };

        assert_eq!(p.id, "test_99");
        assert_eq!(p.target_module, "vision");
        assert_eq!(p.generated_by, "human");
        assert!((p.estimated_risk - 0.4).abs() < 1e-6);
        assert_eq!(p.prerequisites.len(), 1);
    }

    #[test]
    fn test_improvement_result_delta_computation() {
        let mut before = HashMap::new();
        before.insert("accuracy".to_string(), 0.70);
        before.insert("latency".to_string(), 100.0);

        let mut after = HashMap::new();
        after.insert("accuracy".to_string(), 0.92);
        after.insert("latency".to_string(), 60.0);

        let mut delta = HashMap::new();
        for (k, v) in &before {
            if let Some(av) = after.get(k) {
                delta.insert(k.clone(), av - v);
            }
        }

        let result = ImprovementResult {
            proposal_id: "p1".to_string(),
            success: true,
            before_metrics: before,
            after_metrics: after,
            delta,
            issues: vec![],
            rollback_possible: true,
            implementation_time_ms: 150,
        };

        assert!(result.success);
        assert!((*result.delta.get("accuracy").unwrap() - 0.22).abs() < 1e-6);
        assert!((*result.delta.get("latency").unwrap() - (-40.0)).abs() < 1e-6);
    }

    #[test]
    fn test_proposals_for_module() {
        let mut core = RsiCore::new();
        core.propose_improvement("mod_a", ImprovementType::Efficiency, "fix a", 0.3, 0.5);
        core.propose_improvement("mod_b", ImprovementType::Algorithmic, "fix b", 0.3, 0.5);
        core.propose_improvement("mod_a", ImprovementType::Calibration, "fix a2", 0.2, 0.4);

        let for_a = core.proposals_for_module("mod_a");
        assert_eq!(for_a.len(), 2);
        let for_b = core.proposals_for_module("mod_b");
        assert_eq!(for_b.len(), 1);
        let for_c = core.proposals_for_module("mod_c");
        assert!(for_c.is_empty());
    }

    #[test]
    fn test_implement_missing_proposal() {
        let mut core = RsiCore::new();
        let result = core.implement("nonexistent");
        assert!(!result.success);
        assert!(result.issues.iter().any(|i| i.contains("not found")));
    }

    #[test]
    fn test_rollback_unavailable() {
        let mut result = ImprovementResult {
            proposal_id: "x".to_string(),
            success: false,
            before_metrics: HashMap::new(),
            after_metrics: HashMap::new(),
            delta: HashMap::new(),
            issues: vec![],
            rollback_possible: false,
            implementation_time_ms: 0,
        };

        let mut core = RsiCore::new();
        core.results.push(result.clone());
        let rb = core.rollback(&result);
        assert!(rb.is_err());

        result.rollback_possible = true;
        core.results.push(result);
        // First rollback (first result stays, it was the one with rollback_possible=false...
        // Actually let's just test the second one)
        let core2 = &mut RsiCore::new();
        core2.results.push(ImprovementResult {
            proposal_id: "y".to_string(),
            success: true,
            before_metrics: HashMap::new(),
            after_metrics: HashMap::new(),
            delta: HashMap::new(),
            issues: vec![],
            rollback_possible: true,
            implementation_time_ms: 0,
        });
        let res = core2.results[0].clone();
        assert!(core2.rollback(&res).is_ok());
    }

    #[test]
    fn test_controller_cycle_disabled() {
        let mut ctrl = RsiController::new();
        ctrl.rsi_cycle_enabled = false;
        let perf = SystemPerformanceData {
            error_rate: 0.30,
            confidence_calibration_ece: 0.05,
            reasoning_accuracy: 0.85,
            avg_response_time_ms: 500.0,
            memory_usage_mb: 500.0,
            reflection_quality: 0.7,
            skill_success_rate: 0.9,
            module_metrics: HashMap::new(),
            recent_failures: vec![],
            bottlenecks: vec![],
        };
        let results = ctrl.run_cycle(&perf);
        assert!(results.is_empty());
    }

    #[test]
    fn test_safety_status_has_correct_fields() {
        let status = RsiSafetyStatus {
            safe: true,
            warnings: vec![],
            current_risk_level: 0.3,
            improvement_rate: 0.85,
            degradation_signals: vec![],
        };
        assert!(status.safe);
        assert!((status.current_risk_level - 0.3).abs() < 1e-6);
        assert!((status.improvement_rate - 0.85).abs() < 1e-6);
    }

    #[test]
    fn test_success_rate_empty() {
        let core = RsiCore::new();
        assert!((core.success_rate(None) - 0.0).abs() < 1e-6);
        assert!((core.success_rate(Some(ImprovementType::Parametric)) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_net_improvement_empty() {
        let core = RsiCore::new();
        let net = core.net_improvement();
        assert!(net.is_empty());
    }

    #[test]
    fn test_prioritize_empty() {
        let core = RsiCore::new();
        assert!(core.prioritize().is_none());
    }

    #[test]
    fn test_rsi_core_default_handles_risk_clamping() {
        let mut core = RsiCore::new();
        let id = core.propose_improvement("test", ImprovementType::Efficiency, "test", 1.5, 2.0);
        let proposal = core.proposals.iter().find(|p| p.id == id).unwrap();
        assert!(proposal.estimated_risk <= 1.0);
        assert!(proposal.estimated_impact <= 1.0);
    }
}
