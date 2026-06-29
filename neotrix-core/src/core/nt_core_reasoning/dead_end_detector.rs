use std::collections::VecDeque;

use super::vsa_blackboard::{Hypothesis, VsaBlackboard};

#[derive(Debug, Clone)]
pub struct DeadEndConfig {
    pub max_loop_length: usize,
    pub min_divergence_threshold: f64,
    pub max_contradiction_ratio: f64,
    pub stagnation_window: usize,
    pub confidence_stagnation_threshold: f64,
    pub max_depth_without_progress: usize,
    pub fast_monitor_interval: usize,
}

impl Default for DeadEndConfig {
    fn default() -> Self {
        Self {
            max_loop_length: 8,
            min_divergence_threshold: 0.95,
            max_contradiction_ratio: 0.4,
            stagnation_window: 5,
            confidence_stagnation_threshold: 0.02,
            max_depth_without_progress: 20,
            fast_monitor_interval: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeadEndType {
    Loop,
    Divergence,
    ContradictionFlood,
    ConfidenceStagnation,
    DepthExceeded,
    SemanticDeadlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoveryStrategy {
    Backtrack(usize),
    SwitchReasoningMode,
    InjectCounterfactual,
    DecomposeSubProblem,
    RequestClarification,
    FallbackToHeuristic,
}

#[derive(Debug, Clone)]
pub struct DeadEndReport {
    pub detected_type: DeadEndType,
    pub detection_step: usize,
    pub loop_length: Option<usize>,
    pub vsa_similarity: f64,
    pub confidence_delta: f64,
    pub recovery: RecoveryStrategy,
    pub evidence: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct DeadEndStats {
    pub total_checks: usize,
    pub dead_ends_detected: usize,
    pub loop_count: usize,
    pub divergence_count: usize,
    pub contradiction_count: usize,
    pub stagnation_count: usize,
    pub recovery_success_rate: f64,
    pub most_common_type: Option<DeadEndType>,
}

#[derive(Debug, Clone)]
pub struct DeadEndDetector {
    pub config: DeadEndConfig,
    pub step_history: VecDeque<(u64, f64)>,
    pub vsa_signatures: VecDeque<Vec<u8>>,
    pub contradiction_count: usize,
    pub total_hypotheses_generated: usize,
    pub reports: Vec<DeadEndReport>,
    pub recovery_history: Vec<(DeadEndType, RecoveryStrategy, bool)>,
    step_counter: usize,
    recovery_success: std::collections::HashMap<(DeadEndType, RecoveryStrategy), (usize, usize)>,
}

impl DeadEndDetector {
    pub fn new(config: DeadEndConfig) -> Self {
        Self {
            config: config.clone(),
            step_history: VecDeque::with_capacity(config.max_depth_without_progress + 10),
            vsa_signatures: VecDeque::with_capacity(config.max_loop_length * 2),
            contradiction_count: 0,
            total_hypotheses_generated: 0,
            reports: Vec::new(),
            recovery_history: Vec::new(),
            step_counter: 0,
            recovery_success: std::collections::HashMap::new(),
        }
    }

    pub fn fast_check(
        &mut self,
        current_hypothesis: &Hypothesis,
        _blackboard: &VsaBlackboard,
    ) -> Option<DeadEndReport> {
        self.step_counter += 1;
        if self.step_counter % self.config.fast_monitor_interval != 0 {
            return None;
        }

        if let Some(loop_len) = self.detect_loop(&current_hypothesis.content) {
            let report = DeadEndReport {
                detected_type: DeadEndType::Loop,
                detection_step: self.step_counter,
                loop_length: Some(loop_len),
                vsa_similarity: self.config.min_divergence_threshold,
                confidence_delta: 0.0,
                recovery: self.select_recovery(DeadEndType::Loop),
                evidence: vec![
                    format!("VSA state repeated {} consecutive times", loop_len),
                    "High similarity in consecutive reasoning states".into(),
                ],
                timestamp: current_hypothesis.created_at,
            };
            return Some(report);
        }

        if self.detect_confidence_stagnation() {
            let report = DeadEndReport {
                detected_type: DeadEndType::ConfidenceStagnation,
                detection_step: self.step_counter,
                loop_length: None,
                vsa_similarity: 0.0,
                confidence_delta: self.config.confidence_stagnation_threshold,
                recovery: self.select_recovery(DeadEndType::ConfidenceStagnation),
                evidence: vec![
                    format!(
                        "Confidence unchanged for {} steps",
                        self.config.stagnation_window
                    ),
                    "No meaningful confidence improvement".into(),
                ],
                timestamp: current_hypothesis.created_at,
            };
            return Some(report);
        }

        None
    }

    pub fn slow_check(
        &mut self,
        history: &[Hypothesis],
        blackboard: &VsaBlackboard,
    ) -> Option<DeadEndReport> {
        if self.step_counter % 10 != 0 {
            return None;
        }

        if self.detect_contradiction_flood(blackboard) {
            let report = DeadEndReport {
                detected_type: DeadEndType::ContradictionFlood,
                detection_step: self.step_counter,
                loop_length: None,
                vsa_similarity: 0.0,
                confidence_delta: 0.0,
                recovery: self.select_recovery(DeadEndType::ContradictionFlood),
                evidence: vec![
                    format!(
                        "Contradiction ratio exceeded {:.0}%",
                        self.config.max_contradiction_ratio * 100.0
                    ),
                    "Too many mutually exclusive hypotheses".into(),
                ],
                timestamp: history.last().map_or(0, |h| h.created_at),
            };
            return Some(report);
        }

        if let Some(sim) =
            self.detect_divergence(history.last().unwrap_or(history.first()?), blackboard)
        {
            let report = DeadEndReport {
                detected_type: DeadEndType::Divergence,
                detection_step: self.step_counter,
                loop_length: None,
                vsa_similarity: sim,
                confidence_delta: 0.0,
                recovery: self.select_recovery(DeadEndType::Divergence),
                evidence: vec![
                    format!("VSA similarity {:.4} exceeds divergence threshold", sim),
                    "Reasoning state diverging from productive path".into(),
                ],
                timestamp: history.last().map_or(0, |h| h.created_at),
            };
            return Some(report);
        }

        if self.detect_semantic_deadlock(history) {
            let report = DeadEndReport {
                detected_type: DeadEndType::SemanticDeadlock,
                detection_step: self.step_counter,
                loop_length: None,
                vsa_similarity: 0.0,
                confidence_delta: 0.0,
                recovery: self.select_recovery(DeadEndType::SemanticDeadlock),
                evidence: vec![
                    "Circular dependency chain detected in reasoning".into(),
                    "Hypotheses reference each other cyclically".into(),
                ],
                timestamp: history.last().map_or(0, |h| h.created_at),
            };
            return Some(report);
        }

        if self.step_counter >= self.config.max_depth_without_progress
            && self.step_history.len() >= self.config.max_depth_without_progress
        {
            let recent_confidences: Vec<f64> = self
                .step_history
                .iter()
                .rev()
                .take(self.config.stagnation_window)
                .map(|&(_, c)| c)
                .collect();
            if recent_confidences.len() >= self.config.stagnation_window {
                let max_conf = recent_confidences
                    .iter()
                    .cloned()
                    .fold(f64::NEG_INFINITY, f64::max);
                let min_conf = recent_confidences
                    .iter()
                    .cloned()
                    .fold(f64::INFINITY, f64::min);
                if max_conf - min_conf < self.config.confidence_stagnation_threshold {
                    let report = DeadEndReport {
                        detected_type: DeadEndType::DepthExceeded,
                        detection_step: self.step_counter,
                        loop_length: None,
                        vsa_similarity: 0.0,
                        confidence_delta: max_conf - min_conf,
                        recovery: self.select_recovery(DeadEndType::DepthExceeded),
                        evidence: vec![
                            format!("Reached depth {} without conclusion", self.step_counter),
                            "No resolution found within depth limit".into(),
                        ],
                        timestamp: history.last().map_or(0, |h| h.created_at),
                    };
                    return Some(report);
                }
            }
        }

        None
    }

    fn detect_loop(&self, _current: &[u8]) -> Option<usize> {
        if self.vsa_signatures.len() < self.config.max_loop_length {
            return None;
        }

        let recent: Vec<&[u8]> = self
            .vsa_signatures
            .iter()
            .map(Vec::as_slice)
            .rev()
            .take(self.config.max_loop_length)
            .collect();

        if recent.len() < 2 {
            return None;
        }

        let mut consecutive = 1;
        for i in 1..recent.len() {
            let sim = cosine_similarity(recent[i], recent[i - 1]);
            if sim >= self.config.min_divergence_threshold {
                consecutive += 1;
                if consecutive >= self.config.max_loop_length {
                    return Some(consecutive);
                }
            } else {
                consecutive = 1;
            }
        }

        None
    }

    fn detect_divergence(&self, current: &Hypothesis, _blackboard: &VsaBlackboard) -> Option<f64> {
        if self.step_history.len() < 2 {
            return None;
        }

        let recent_sigs: Vec<&[u8]> = self
            .vsa_signatures
            .iter()
            .map(Vec::as_slice)
            .rev()
            .skip(1)
            .take(self.config.stagnation_window)
            .collect();

        if recent_sigs.is_empty() {
            return None;
        }

        let avg_similarity: f64 = recent_sigs
            .iter()
            .map(|sig| cosine_similarity(current.content.as_slice(), sig))
            .sum::<f64>()
            / recent_sigs.len() as f64;

        if avg_similarity < 0.3 {
            Some(avg_similarity)
        } else {
            None
        }
    }

    fn detect_contradiction_flood(&self, blackboard: &VsaBlackboard) -> bool {
        let total = blackboard.hypotheses.len();
        if total == 0 {
            return false;
        }
        let contradicted = blackboard
            .hypotheses
            .iter()
            .filter(|h| h.is_contradicted)
            .count();
        contradicted as f64 / total as f64 >= self.config.max_contradiction_ratio
    }

    fn detect_confidence_stagnation(&self) -> bool {
        if self.step_history.len() < self.config.stagnation_window {
            return false;
        }

        let recent: Vec<f64> = self
            .step_history
            .iter()
            .rev()
            .take(self.config.stagnation_window)
            .map(|&(_, c)| c)
            .collect();

        if recent.len() < 2 {
            return false;
        }

        let max_delta = recent
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(f64::NEG_INFINITY, f64::max);

        max_delta < self.config.confidence_stagnation_threshold
    }

    fn detect_semantic_deadlock(&self, history: &[Hypothesis]) -> bool {
        if history.len() < 4 {
            return false;
        }

        let recent: Vec<&Hypothesis> = history.iter().rev().take(6).collect();
        if recent.len() < 4 {
            return false;
        }

        for i in 0..recent.len().saturating_sub(2) {
            for j in (i + 2)..recent.len() {
                let sim = cosine_similarity(&recent[i].content, &recent[j].content);
                if sim > 0.9 {
                    let mid_sim =
                        cosine_similarity(&recent[(i + j) / 2].content, &recent[i].content);
                    if mid_sim > 0.85 {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn select_recovery(&self, dead_end: DeadEndType) -> RecoveryStrategy {
        let strategies = match dead_end {
            DeadEndType::Loop => {
                vec![
                    RecoveryStrategy::InjectCounterfactual,
                    RecoveryStrategy::Backtrack(3),
                    RecoveryStrategy::SwitchReasoningMode,
                ]
            }
            DeadEndType::Divergence => {
                vec![
                    RecoveryStrategy::Backtrack(2),
                    RecoveryStrategy::RequestClarification,
                    RecoveryStrategy::DecomposeSubProblem,
                ]
            }
            DeadEndType::ContradictionFlood => {
                vec![
                    RecoveryStrategy::InjectCounterfactual,
                    RecoveryStrategy::DecomposeSubProblem,
                    RecoveryStrategy::SwitchReasoningMode,
                ]
            }
            DeadEndType::ConfidenceStagnation => {
                vec![
                    RecoveryStrategy::SwitchReasoningMode,
                    RecoveryStrategy::FallbackToHeuristic,
                    RecoveryStrategy::InjectCounterfactual,
                ]
            }
            DeadEndType::DepthExceeded => {
                vec![
                    RecoveryStrategy::DecomposeSubProblem,
                    RecoveryStrategy::Backtrack(5),
                    RecoveryStrategy::RequestClarification,
                ]
            }
            DeadEndType::SemanticDeadlock => {
                vec![
                    RecoveryStrategy::InjectCounterfactual,
                    RecoveryStrategy::Backtrack(4),
                    RecoveryStrategy::FallbackToHeuristic,
                ]
            }
        };

        let mut best: Option<RecoveryStrategy> = None;
        let mut best_rate = 0.0;

        for s in &strategies {
            let key = (dead_end, *s);
            let rate = self
                .recovery_success
                .get(&key)
                .map(|&(success, total)| {
                    if total > 0 {
                        success as f64 / total as f64
                    } else {
                        0.0
                    }
                })
                .unwrap_or(0.5);
            if rate > best_rate {
                best_rate = rate;
                best = Some(*s);
            }
        }

        best.unwrap_or(
            strategies
                .into_iter()
                .next()
                .unwrap_or(RecoveryStrategy::SwitchReasoningMode),
        )
    }

    pub fn record_recovery_outcome(
        &mut self,
        dead_end: DeadEndType,
        strategy: RecoveryStrategy,
        success: bool,
    ) {
        self.recovery_history.push((dead_end, strategy, success));
        let key = (dead_end, strategy);
        let entry = self.recovery_success.entry(key).or_insert((0, 0));
        if success {
            entry.0 += 1;
        }
        entry.1 += 1;
    }

    pub fn record_step(&mut self, hypothesis: &Hypothesis) {
        let hash = compute_vsa_hash(&hypothesis.content);
        self.step_history.push_back((hash, hypothesis.confidence));
        self.vsa_signatures.push_back(hypothesis.content.clone());
        self.total_hypotheses_generated += 1;
        if hypothesis.is_contradicted {
            self.contradiction_count += 1;
        }

        if self.step_history.len() > self.config.max_depth_without_progress + 10 {
            self.step_history.pop_front();
        }
        if self.vsa_signatures.len() > self.config.max_loop_length * 3 {
            self.vsa_signatures.pop_front();
        }
    }

    pub fn check_all(
        &mut self,
        current: &Hypothesis,
        blackboard: &VsaBlackboard,
    ) -> Vec<DeadEndReport> {
        let mut reports = Vec::new();

        if let Some(r) = self.fast_check(current, blackboard) {
            reports.push(r);
        }

        let history: Vec<Hypothesis> = blackboard.hypotheses.clone();
        if let Some(r) = self.slow_check(&history, blackboard) {
            reports.push(r);
        }

        for report in &reports {
            self.reports.push(report.clone());
        }

        reports
    }

    pub fn stats(&self) -> DeadEndStats {
        let mut loop_count = 0;
        let mut divergence_count = 0;
        let mut contradiction_count = 0;
        let mut stagnation_count = 0;
        let mut type_counts: std::collections::HashMap<DeadEndType, usize> =
            std::collections::HashMap::new();

        for r in &self.reports {
            let counter = type_counts.entry(r.detected_type).or_insert(0);
            *counter += 1;
            match r.detected_type {
                DeadEndType::Loop => loop_count += 1,
                DeadEndType::Divergence => divergence_count += 1,
                DeadEndType::ContradictionFlood => contradiction_count += 1,
                DeadEndType::ConfidenceStagnation | DeadEndType::DepthExceeded => {
                    stagnation_count += 1
                }
                DeadEndType::SemanticDeadlock => {}
            }
        }

        let dead_ends_detected = self.reports.len();
        let total_recoveries = self.recovery_history.len();
        let successful_recoveries = self
            .recovery_history
            .iter()
            .filter(|&&(_, _, success)| success)
            .count();
        let recovery_success_rate = if total_recoveries > 0 {
            successful_recoveries as f64 / total_recoveries as f64
        } else {
            0.0
        };

        let most_common_type = type_counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(ty, _)| ty);

        DeadEndStats {
            total_checks: self.step_counter,
            dead_ends_detected,
            loop_count,
            divergence_count,
            contradiction_count,
            stagnation_count,
            recovery_success_rate,
            most_common_type,
        }
    }
}

fn compute_vsa_hash(content: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

fn cosine_similarity(a: &[u8], b: &[u8]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }

    let dot: u64 = a[..len]
        .iter()
        .zip(b[..len].iter())
        .map(|(x, y)| (*x as u64).saturating_mul(*y as u64))
        .sum();

    let mag_a: u64 = a[..len]
        .iter()
        .map(|x| (*x as u64).saturating_mul(*x as u64))
        .sum();
    let mag_b: u64 = b[..len]
        .iter()
        .map(|x| (*x as u64).saturating_mul(*x as u64))
        .sum();

    if mag_a == 0 || mag_b == 0 {
        return 0.0;
    }

    let denominator = (mag_a as f64).sqrt() * (mag_b as f64).sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        dot as f64 / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_reasoning::ExpertType;
    use crate::core::unix_now_ms;

    fn make_hypothesis(
        id: u64,
        content: Vec<u8>,
        confidence: f64,
        contradicted: bool,
    ) -> Hypothesis {
        Hypothesis {
            id,
            content,
            confidence,
            expert: ExpertType::Causal,
            supporting_evidence: vec![],
            created_at: unix_now_ms(),
            is_contradicted: contradicted,
        }
    }

    fn make_blackboard(hypotheses: Vec<Hypothesis>) -> VsaBlackboard {
        let mut bb = VsaBlackboard::new(100);
        for h in hypotheses {
            let _ = bb.post_hypothesis(h.content, h.confidence, h.expert, h.supporting_evidence);
        }
        bb
    }

    #[test]
    fn test_loop_detection() {
        let config = DeadEndConfig {
            max_loop_length: 3,
            min_divergence_threshold: 0.95,
            fast_monitor_interval: 1,
            ..Default::default()
        };
        let mut detector = DeadEndDetector::new(config);
        let content = vec![1u8, 2, 3, 4];
        for i in 0..5 {
            let h = make_hypothesis(i, content.clone(), 0.5 + i as f64 * 0.01, false);
            detector.record_step(&h);
        }
        let h = make_hypothesis(5, content.clone(), 0.55, false);
        let bb = make_blackboard(vec![]);
        assert!(detector.detect_loop(&h.content).is_some());
    }

    #[test]
    fn test_contradiction_flood() {
        let config = DeadEndConfig {
            max_contradiction_ratio: 0.3,
            ..Default::default()
        };
        let detector = DeadEndDetector::new(config);
        let mut hyps = Vec::new();
        for i in 0..10 {
            let c = i < 4;
            hyps.push(make_hypothesis(i as u64, vec![i as u8], 0.5, c));
        }
        let bb = make_blackboard(hyps);
        assert!(detector.detect_contradiction_flood(&bb));
    }

    #[test]
    fn test_confidence_stagnation() {
        let config = DeadEndConfig {
            stagnation_window: 4,
            confidence_stagnation_threshold: 0.05,
            ..Default::default()
        };
        let mut detector = DeadEndDetector::new(config);
        for i in 0..6 {
            let h = make_hypothesis(i, vec![i as u8], 0.5, false);
            detector.record_step(&h);
        }
        assert!(detector.detect_confidence_stagnation());
    }

    #[test]
    fn test_select_recovery_prefers_known_good() {
        let config = DeadEndConfig::default();
        let mut detector = DeadEndDetector::new(config);
        detector.record_recovery_outcome(
            DeadEndType::Loop,
            RecoveryStrategy::InjectCounterfactual,
            true,
        );
        detector.record_recovery_outcome(
            DeadEndType::Loop,
            RecoveryStrategy::InjectCounterfactual,
            true,
        );
        detector.record_recovery_outcome(DeadEndType::Loop, RecoveryStrategy::Backtrack(3), false);
        let strategy = detector.select_recovery(DeadEndType::Loop);
        assert_eq!(strategy, RecoveryStrategy::InjectCounterfactual);
    }

    #[test]
    fn test_semantic_deadlock_detection() {
        let mut detector = DeadEndDetector::new(DeadEndConfig::default());
        let content_a = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let content_b = vec![8u8, 7, 6, 5, 4, 3, 2, 1];
        let history = vec![
            make_hypothesis(1, content_a.clone(), 0.5, false),
            make_hypothesis(2, content_b.clone(), 0.5, false),
            make_hypothesis(3, content_a.clone(), 0.5, false),
            make_hypothesis(4, content_b.clone(), 0.5, false),
            make_hypothesis(5, content_a.clone(), 0.5, false),
            make_hypothesis(6, content_b.clone(), 0.5, false),
        ];
        assert!(detector.detect_semantic_deadlock(&history));
    }

    #[test]
    fn test_stats() {
        let mut detector = DeadEndDetector::new(DeadEndConfig::default());
        detector.record_recovery_outcome(DeadEndType::Loop, RecoveryStrategy::Backtrack(3), true);
        detector.record_recovery_outcome(DeadEndType::Loop, RecoveryStrategy::Backtrack(3), false);
        let stats = detector.stats();
        assert_eq!(stats.recovery_success_rate, 0.5);
        assert_eq!(stats.total_checks, 0);
    }

    #[test]
    fn test_fast_check_skips_when_not_interval() {
        let config = DeadEndConfig {
            fast_monitor_interval: 5,
            ..Default::default()
        };
        let mut detector = DeadEndDetector::new(config);
        let h = make_hypothesis(1, vec![1, 2, 3], 0.5, false);
        let bb = make_blackboard(vec![]);
        assert!(detector.fast_check(&h, &bb).is_none());
        detector.record_step(&h);
        assert!(detector.fast_check(&h, &bb).is_none());
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1u8, 2, 3, 4, 5];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1u8, 0, 1, 0];
        let b = vec![0u8, 1, 0, 1];
        assert!((cosine_similarity(&a, &b)).abs() < 1e-6);
    }
}
