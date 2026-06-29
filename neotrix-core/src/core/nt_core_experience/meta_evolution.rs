use std::collections::{HashMap, HashSet};
use std::time::Instant;

use log;

// MetaEvolutionLoop is now owned by ConsciousnessIntegration as a field

#[derive(Debug, Clone)]
pub struct AgentVersion {
    pub version: String,
    pub score: f64,
    pub cost: f64,
    pub time: f64,
    pub utility: f64,
    pub description: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct EvolutionWeights {
    pub score_weight: f64,
    pub cost_weight: f64,
    pub time_weight: f64,
}

impl Default for EvolutionWeights {
    fn default() -> Self {
        Self {
            score_weight: 0.5,
            cost_weight: 0.25,
            time_weight: 0.25,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ChangeType {
    Parametric,
    Structural,
    Behavioral,
    Revert,
}

#[derive(Debug, Clone)]
pub struct EvolutionProposal {
    pub target_module: String,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone)]
pub struct MetaEvolutionLoop {
    pub archive: Vec<AgentVersion>,
    pub max_archive_size: usize,
    pub weights: EvolutionWeights,
    pub improvement_count: u64,
    pub correctness_safeguards: bool,
    best_utility: f64,
    /// DGM-H stepping stones: (archive_entry_hash, quality_score)
    pub stepping_stones: Vec<(String, f64)>,
    /// DGM-H domain performance tracking: domain → recent scores
    pub domain_performance: HashMap<String, Vec<f64>>,
}

impl Default for MetaEvolutionLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaEvolutionLoop {
    pub fn new() -> Self {
        Self {
            archive: Vec::with_capacity(20),
            max_archive_size: 20,
            weights: EvolutionWeights::default(),
            improvement_count: 0,
            correctness_safeguards: true,
            best_utility: f64::NEG_INFINITY,
            stepping_stones: Vec::new(),
            domain_performance: HashMap::new(),
        }
    }

    pub fn compute_utility(&self, score: f64, cost: f64, time: f64) -> f64 {
        self.weights.score_weight * score
            - self.weights.cost_weight * cost
            - self.weights.time_weight * time
    }

    pub fn register_version(
        &mut self,
        version: String,
        score: f64,
        cost: f64,
        time: f64,
        description: String,
        timestamp: String,
    ) {
        let utility = self.compute_utility(score, cost, time);
        if utility > self.best_utility {
            self.best_utility = utility;
        }
        self.archive.push(AgentVersion {
            version,
            score,
            cost,
            time,
            utility,
            description,
            timestamp,
        });
        self.improvement_count += 1;
        if self.archive.len() > self.max_archive_size {
            self.archive.remove(0);
        }
    }

    pub fn best_version(&self) -> Option<&AgentVersion> {
        self.archive.iter().max_by(|a, b| {
            a.utility
                .partial_cmp(&b.utility)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn is_best(&self, utility: f64) -> bool {
        utility > self.best_utility
    }

    pub fn improvement_trend(&self) -> f64 {
        let len = self.archive.len();
        if len < 2 {
            return 0.0;
        }
        let window = self.archive.iter().rev().take(5).rev().collect::<Vec<_>>();
        let n = window.len();
        if n < 2 {
            return 0.0;
        }
        let indices: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let utilities: Vec<f64> = window.iter().map(|v| v.utility).collect();
        let mean_x: f64 = indices.iter().sum::<f64>() / n as f64;
        let mean_y: f64 = utilities.iter().sum::<f64>() / n as f64;
        let num: f64 = indices
            .iter()
            .zip(utilities.iter())
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum();
        let den: f64 = indices.iter().map(|x| (x - mean_x).powi(2)).sum();
        if den.abs() < 1e-12 {
            0.0
        } else {
            num / den
        }
    }

    pub fn stagnation_detected(&self) -> bool {
        self.archive.len() >= 5 && self.improvement_trend() < 0.01
    }

    pub fn propose_improvement(&self) -> Option<EvolutionProposal> {
        let len = self.archive.len();
        if len < 2 {
            return None;
        }
        let trend = self.improvement_trend();
        if trend < -0.05 {
            return Some(EvolutionProposal {
                target_module: "nt_core_self::metacognitive_weights".into(),
                change_type: ChangeType::Revert,
            });
        }
        if trend < 0.01 {
            let last = self.archive.last()?;
            if last.score < 0.5 {
                Some(EvolutionProposal {
                    target_module: "nt_core_consciousness::cognitive_load".into(),
                    change_type: ChangeType::Parametric,
                })
            } else if last.utility < 0.3 {
                Some(EvolutionProposal {
                    target_module: "nt_core_meta::metacognition_loop".into(),
                    change_type: ChangeType::Structural,
                })
            } else {
                Some(EvolutionProposal {
                    target_module: "nt_core_experience::handler_tier".into(),
                    change_type: ChangeType::Behavioral,
                })
            }
        } else {
            None
        }
    }
}

impl MetaEvolutionLoop {
    /// GEPA-style: reflect on prediction error and adapt weights.
    /// High error → cost_weight unreliable, reduce it.
    /// Low error → scores are trustworthy, increase score_weight.
    pub fn reflect_and_assign_cost(
        &mut self,
        actual_improvement: f64,
        expected_utility: f64,
    ) -> f64 {
        let prediction_error = (expected_utility - actual_improvement).abs();
        if prediction_error > 0.3 {
            self.weights.cost_weight = (self.weights.cost_weight - 0.05).clamp(0.1, 0.9);
        }
        if prediction_error < 0.05 {
            self.weights.score_weight = (self.weights.score_weight + 0.05).clamp(0.1, 0.9);
        }
        prediction_error
    }

    /// Return archive versions that are Pareto-optimal:
    /// no other version has BOTH higher score AND lower cost.
    pub fn pareto_frontier(&self) -> Vec<&AgentVersion> {
        self.archive
            .iter()
            .filter(|v| {
                !self
                    .archive
                    .iter()
                    .any(|other| other.score > v.score && other.cost < v.cost)
            })
            .collect()
    }

    /// Replace archive with only Pareto-optimal versions.
    /// Does nothing if all versions are already on the frontier.
    pub fn prune_archive_by_pareto(&mut self) {
        let frontier = self.pareto_frontier();
        if frontier.len() == self.archive.len() {
            return;
        }
        let frontier_indices: HashSet<*const AgentVersion> =
            frontier.iter().map(|v| &**v as *const _).collect();
        self.archive
            .retain(|v| frontier_indices.contains(&(v as *const _)));
    }

    /// DGM-H: record a domain performance score.
    /// Keeps last 10 scores per domain.
    pub fn record_domain_result(&mut self, domain: &str, score: f64) {
        self.domain_performance
            .entry(domain.to_string())
            .or_default()
            .push(score);
        if let Some(scores) = self.domain_performance.get_mut(domain) {
            if scores.len() > 10 {
                scores.remove(0);
            }
        }
    }

    /// DGM-H: compute improvement rate for a domain.
    /// Compares average of last 3 vs first 3 scores.
    pub fn domain_improvement_rate(&self, domain: &str) -> f64 {
        self.domain_performance
            .get(domain)
            .filter(|scores| scores.len() >= 2)
            .map(|scores| {
                let recent: f64 = scores.iter().rev().take(3).sum::<f64>() / 3.0;
                let early: f64 = scores.iter().take(3).sum::<f64>() / 3.0;
                if early > 0.0 {
                    (recent - early) / early
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0)
    }
}

/// A domain identifier for cross-domain transfer tracking
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DomainId(pub String);

/// Tracks accuracy metrics in a specific domain
#[derive(Debug, Clone)]
pub struct DomainAccuracy {
    pub domain: DomainId,
    pub accuracy_before: f64,
    pub accuracy_after: f64,
    pub samples: u64,
    pub last_updated: Instant,
}

/// Cross-domain transfer validator
/// Tracks whether improvements in one domain generalize to others
#[derive(Debug, Clone)]
pub struct TransferValidator {
    pub domains: HashMap<DomainId, DomainAccuracy>,
    pub transfer_successes: u64,
    pub transfer_failures: u64,
    pub transfer_threshold: f64,
}

impl TransferValidator {
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
            transfer_successes: 0,
            transfer_failures: 0,
            transfer_threshold: 0.05,
        }
    }

    /// Record accuracy in a domain (before or after transfer)
    pub fn record_accuracy(&mut self, domain: &str, accuracy: f64) {
        let id = DomainId(domain.to_string());
        if let Some(entry) = self.domains.get_mut(&id) {
            entry.accuracy_before = entry.accuracy_after;
            entry.accuracy_after = accuracy;
            entry.samples += 1;
            entry.last_updated = Instant::now();
        } else {
            let now = Instant::now();
            self.domains.insert(
                id,
                DomainAccuracy {
                    domain: DomainId(domain.to_string()),
                    accuracy_before: accuracy,
                    accuracy_after: accuracy,
                    samples: 1,
                    last_updated: now,
                },
            );
        }
    }

    /// Mark the current state as "after transfer" for a domain
    pub fn mark_transferred(&mut self, domain: &str) {
        let id = DomainId(domain.to_string());
        if let Some(entry) = self.domains.get_mut(&id) {
            entry.accuracy_before = entry.accuracy_after;
        }
    }

    /// Check if a transfer was successful:
    /// accuracy_after - accuracy_before > threshold
    pub fn check_transfer_success(&mut self, _from_domain: &str, to_domain: &str) -> Option<bool> {
        let to_id = DomainId(to_domain.to_string());
        let entry = self.domains.get(&to_id)?;
        let improvement = entry.accuracy_after - entry.accuracy_before;
        let success = improvement > self.transfer_threshold;
        if success {
            self.transfer_successes += 1;
        } else {
            self.transfer_failures += 1;
        }
        Some(success)
    }

    /// Get improvement for a specific domain
    pub fn domain_improvement(&self, domain: &str) -> Option<f64> {
        let id = DomainId(domain.to_string());
        self.domains.get(&id).map(|entry| {
            if entry.accuracy_before > 0.0 {
                (entry.accuracy_after - entry.accuracy_before) / entry.accuracy_before
            } else {
                entry.accuracy_after - entry.accuracy_before
            }
        })
    }

    /// Overall transfer success rate
    pub fn transfer_rate(&self) -> f64 {
        let total = self.transfer_successes + self.transfer_failures;
        if total == 0 {
            return 0.0;
        }
        self.transfer_successes as f64 / total as f64
    }

    /// Summary report
    pub fn report(&self) -> String {
        let mut lines = vec!["=== TransferValidator Report ===".to_string()];
        lines.push(format!(
            "Transfer success rate: {:.2}%",
            self.transfer_rate() * 100.0
        ));
        lines.push(format!(
            "Successes: {} / Failures: {}",
            self.transfer_successes, self.transfer_failures
        ));
        lines.push(format!("Threshold: {:.3}", self.transfer_threshold));
        lines.push(format!("Tracked domains: {}", self.domains.len()));
        for (id, acc) in &self.domains {
            lines.push(format!(
                "  {}: before={:.3} after={:.3} samples={}",
                id.0, acc.accuracy_before, acc.accuracy_after, acc.samples
            ));
        }
        lines.join("\n")
    }
}

impl Default for TransferValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Run cross-domain transfer validation using MetaAccuracy gap
pub fn validate_transfer(
    validator: &mut TransferValidator,
    source_domain: &str,
    target_domain: &str,
    meta_accuracy_delta: f64,
) -> bool {
    validator.record_accuracy(source_domain, meta_accuracy_delta);
    let result = validator.check_transfer_success(source_domain, target_domain);
    result.unwrap_or(false)
}

/// Dispatch an evolution proposal through a caller-provided dispatch closure.
/// The closure receives (module_target, change_type_description) and returns a status string.
/// This avoids importing CI directly (preventing circular deps) while enabling
/// real runtime mutation via the consciousness pipeline.
pub fn execute_proposal<F>(proposal: &EvolutionProposal, dispatch: F) -> String
where
    F: FnOnce(&str, &ChangeType) -> String,
{
    let target = proposal.target_module.as_str();
    log::info!(
        "[meta_evolution] DISPATCHING {:?} on {}",
        proposal.change_type,
        target
    );

    dispatch(target, &proposal.change_type)
}

/// RSI plateau detector (clawRxiv 2604.01236).
/// Tracks the last 5 evolution round metrics and detects
/// when improvement stagnates (<1% for 3 consecutive rounds).
#[derive(Debug, Clone)]
pub struct RsiPlateauDetector {
    metrics: Vec<f64>,
    max_window: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlateauStatus {
    Improving,
    Plateaued,
    Declining,
}

impl RsiPlateauDetector {
    pub fn new() -> Self {
        Self {
            metrics: Vec::with_capacity(5),
            max_window: 5,
        }
    }

    pub fn record_metric(&mut self, metric: f64) {
        if self.metrics.len() >= self.max_window {
            self.metrics.remove(0);
        }
        self.metrics.push(metric);
    }

    pub fn status(&self) -> PlateauStatus {
        let n = self.metrics.len();
        if n < 4 {
            return PlateauStatus::Improving;
        }
        let last = self.metrics[n - 1];
        let prev = self.metrics[n - 2];
        if last < prev && n >= 3 && prev < self.metrics[n - 3] {
            return PlateauStatus::Declining;
        }
        let last_four = &self.metrics[n - 4..];
        let all_flat = last_four.windows(2).all(|w| w[1] - w[0] < 0.01);
        if all_flat {
            PlateauStatus::Plateaued
        } else {
            PlateauStatus::Improving
        }
    }
}

impl Default for RsiPlateauDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_utility_known_values() {
        let mel = MetaEvolutionLoop::new();
        let u = mel.compute_utility(1.0, 0.0, 0.0);
        assert!((u - 0.5).abs() < 1e-9);
        let u2 = mel.compute_utility(0.8, 0.2, 0.1);
        let expected = 0.5 * 0.8 - 0.25 * 0.2 - 0.25 * 0.1;
        assert!((u2 - expected).abs() < 1e-9);
    }

    #[test]
    fn test_archive_management_push_and_prune() {
        let mut mel = MetaEvolutionLoop::new();
        mel.max_archive_size = 3;
        for i in 0..5 {
            mel.register_version(
                format!("v{}", i),
                0.5 + i as f64 * 0.1,
                0.1,
                0.05,
                format!("version {}", i),
                "now".into(),
            );
        }
        assert_eq!(mel.archive.len(), 3);
        assert_eq!(mel.archive[0].version, "v2");
        assert_eq!(mel.archive[2].version, "v4");
        assert_eq!(mel.improvement_count, 5);
    }

    #[test]
    fn test_best_version_selection() {
        let mut mel = MetaEvolutionLoop::new();
        mel.register_version("v1".into(), 0.5, 0.1, 0.05, "first".into(), "t1".into());
        mel.register_version("v2".into(), 0.9, 0.1, 0.05, "best".into(), "t2".into());
        mel.register_version("v3".into(), 0.3, 0.1, 0.05, "worst".into(), "t3".into());
        let best = mel.best_version().unwrap();
        assert_eq!(best.version, "v2");
    }

    #[test]
    fn test_improvement_trend() {
        let mut mel = MetaEvolutionLoop::new();
        for i in 0..5 {
            mel.register_version(
                format!("v{}", i),
                0.5 + i as f64 * 0.1,
                0.1,
                0.05,
                "test".into(),
                "now".into(),
            );
        }
        let trend = mel.improvement_trend();
        assert!(trend > 0.0);
    }

    #[test]
    fn test_stagnation_detected() {
        let mut mel = MetaEvolutionLoop::new();
        assert!(!mel.stagnation_detected());
        for i in 0..5 {
            mel.register_version(
                format!("v{}", i),
                0.5,
                0.5,
                0.5,
                "flat".into(),
                "now".into(),
            );
        }
        assert!(mel.stagnation_detected());
    }

    #[test]
    fn test_stagnation_false_when_improving() {
        let mut mel = MetaEvolutionLoop::new();
        for i in 0..5 {
            mel.register_version(
                format!("v{}", i),
                0.5 + i as f64 * 0.1,
                0.1,
                0.05,
                "improving".into(),
                "now".into(),
            );
        }
        assert!(!mel.stagnation_detected());
    }

    #[test]
    fn test_propose_improvement_plausible() {
        let mut mel = MetaEvolutionLoop::new();
        assert!(mel.propose_improvement().is_none());
        for i in 0..5 {
            mel.register_version(
                format!("v{}", i),
                0.5,
                0.5,
                0.5,
                "flat".into(),
                "now".into(),
            );
        }
        let proposal = mel.propose_improvement();
        assert!(proposal.is_some());
        let p = proposal.unwrap();
        assert!(!p.target_module.is_empty());
        match p.change_type {
            ChangeType::Parametric
            | ChangeType::Structural
            | ChangeType::Behavioral
            | ChangeType::Revert => {}
        }
    }

    #[test]
    fn test_is_best() {
        let mut mel = MetaEvolutionLoop::new();
        mel.register_version("v1".into(), 0.5, 0.1, 0.05, "first".into(), "t1".into());
        assert!(mel.is_best(0.6));
        assert!(!mel.is_best(0.1));
    }

    #[test]
    fn test_local_instance() {
        let mel = MetaEvolutionLoop::new();
        assert_eq!(mel.archive.len(), 0);
        assert!((mel.weights.score_weight - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_trend_insufficient_data() {
        let mel = MetaEvolutionLoop::new();
        assert!((mel.improvement_trend() - 0.0).abs() < 1e-9);
        let mut mel2 = MetaEvolutionLoop::new();
        mel2.register_version("v1".into(), 0.5, 0.1, 0.05, "only".into(), "t1".into());
        assert!((mel2.improvement_trend() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_transfer_validator_new() {
        let tv = TransferValidator::new();
        assert!(tv.domains.is_empty());
        assert_eq!(tv.transfer_successes, 0);
        assert_eq!(tv.transfer_failures, 0);
        assert!((tv.transfer_threshold - 0.05).abs() < 1e-9);
        assert!((tv.transfer_rate() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_record_accuracy_tracking() {
        let mut tv = TransferValidator::new();
        tv.record_accuracy("reasoning", 0.65);
        tv.record_accuracy("planning", 0.70);
        assert_eq!(tv.domains.len(), 2);
        let imp = tv.domain_improvement("reasoning").unwrap();
        assert!((imp - 0.0).abs() < 1e-9);
        tv.record_accuracy("reasoning", 0.82);
        let imp2 = tv.domain_improvement("reasoning").unwrap();
        assert!(imp2 > 0.2);
    }

    #[test]
    fn test_successful_transfer_detection() {
        let mut tv = TransferValidator::new();
        tv.transfer_threshold = 0.10;
        tv.record_accuracy("reasoning", 0.60);
        tv.record_accuracy("planning", 0.62);
        tv.mark_transferred("planning");
        tv.record_accuracy("planning", 0.85);
        let success = tv.check_transfer_success("reasoning", "planning");
        assert_eq!(success, Some(true));
        assert_eq!(tv.transfer_successes, 1);
        assert_eq!(tv.transfer_failures, 0);
    }

    #[test]
    fn test_transfer_rate_calculation() {
        let mut tv = TransferValidator::new();
        tv.transfer_threshold = 0.01;
        tv.record_accuracy("a", 0.50);
        tv.record_accuracy("b", 0.50);
        tv.mark_transferred("b");
        tv.record_accuracy("b", 0.55);
        let _ = tv.check_transfer_success("a", "b");
        tv.record_accuracy("c", 0.50);
        tv.mark_transferred("c");
        tv.record_accuracy("c", 0.50);
        let _ = tv.check_transfer_success("a", "c");
        assert_eq!(tv.transfer_successes, 1);
        assert_eq!(tv.transfer_failures, 1);
        assert!((tv.transfer_rate() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_validate_transfer_function() {
        let mut tv = TransferValidator::new();
        tv.transfer_threshold = 0.05;
        tv.record_accuracy("source", 0.30);
        tv.record_accuracy("target", 0.30);
        tv.mark_transferred("target");
        let result = validate_transfer(&mut tv, "source", "target", 0.15);
        assert!(result);
    }
}
