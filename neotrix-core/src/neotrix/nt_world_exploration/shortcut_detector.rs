use std::collections::HashMap;

/// Types of search shortcuts detectable by the ShortcutDetector.
///
/// Distilled from: arXiv:2606.12087 FORT-Searcher.
/// Four shortcut risks formalized in the paper:
/// - EvidenceCoCoverage: multiple clues on a single page (cheap answer)
/// - SingleClueSelectivity: one clue alone identifies the answer
/// - ExposedConstants: exact strings on question surface → direct query
/// - PriorKnowledgeBinding: answer from parametric knowledge before retrieval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ShortcutType {
    EvidenceCoCoverage,
    SingleClueSelectivity,
    ExposedConstants,
    PriorKnowledgeBinding,
}

impl ShortcutType {
    pub fn label(&self) -> &'static str {
        match self {
            ShortcutType::EvidenceCoCoverage => "evidence_co_coverage",
            ShortcutType::SingleClueSelectivity => "single_clue_selectivity",
            ShortcutType::ExposedConstants => "exposed_constants",
            ShortcutType::PriorKnowledgeBinding => "prior_knowledge_binding",
        }
    }

    pub fn all() -> Vec<ShortcutType> {
        vec![
            ShortcutType::EvidenceCoCoverage,
            ShortcutType::SingleClueSelectivity,
            ShortcutType::ExposedConstants,
            ShortcutType::PriorKnowledgeBinding,
        ]
    }
}

/// A detected shortcut signal on a search branch.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ShortcutSignal {
    pub shortcut_type: ShortcutType,
    pub branch_id: u64,
    pub confidence: f64,
    pub detail: String,
}

/// ShortcutDetector — detects shortcut risks in search branches.
///
/// FORT-Searcher formalized 4 shortcut risks that cause search tasks to
/// collapse into cheap identification rather than genuine multi-step reasoning.
/// This module detects those patterns in TreeSeeker branches so the system
/// can discount shortcut-tainted branches and favor genuine exploration paths.
///
/// Detection heuristics:
/// - EvidenceCoCoverage: single source provides both high value + high certainty
/// - SingleClueSelectivity: value spikes from a single visit
/// - ExposedConstants: source_label matches a known constant pattern
/// - PriorKnowledgeBinding: value is high but no visits to external sources
#[derive(Debug, Clone, serde::Serialize)]
pub struct ShortcutDetector {
    signals: Vec<ShortcutSignal>,
    co_coverage_threshold: f64,
    value_spike_threshold: f64,
    max_signals: usize,
}

impl ShortcutDetector {
    pub fn new() -> Self {
        Self {
            signals: Vec::new(),
            co_coverage_threshold: 0.8,
            value_spike_threshold: 0.3,
            max_signals: 1000,
        }
    }

    /// Analyze a branch update for shortcut risks.
    /// Returns detected shortcut signals for this update.
    pub fn analyze(
        &mut self,
        branch_id: u64,
        value: f64,
        uncertainty: f64,
        _risk: f64,
        visits: u64,
        source_label: &str,
    ) -> Vec<ShortcutSignal> {
        let mut detected = Vec::new();

        // EvidenceCoCoverage: high value + low uncertainty + single source
        if value > self.co_coverage_threshold && uncertainty < 0.2 && visits <= 2 {
            detected.push(ShortcutSignal {
                shortcut_type: ShortcutType::EvidenceCoCoverage,
                branch_id,
                confidence: (value + (1.0 - uncertainty)) / 2.0,
                detail: format!(
                    "value={:.2} uncertainty={:.2} visits={} label={}",
                    value, uncertainty, visits, source_label
                ),
            });
        }

        // SingleClueSelectivity: value spike between consecutive updates
        // (detected via value > threshold and low visits)
        if value > self.value_spike_threshold + 0.5 && visits <= 1 {
            detected.push(ShortcutSignal {
                shortcut_type: ShortcutType::SingleClueSelectivity,
                branch_id,
                confidence: value.clamp(0.0, 1.0),
                detail: format!(
                    "spike value={:.2} visits={} label={}",
                    value, visits, source_label
                ),
            });
        }

        // ExposedConstants: source_label matches known constant patterns
        let lower_label = source_label.to_lowercase();
        if lower_label.contains("doi:")
            || lower_label.contains("arxiv:")
            || lower_label.starts_with("http")
        {
            detected.push(ShortcutSignal {
                shortcut_type: ShortcutType::ExposedConstants,
                branch_id,
                confidence: 0.6,
                detail: format!("constant pattern in label={}", source_label),
            });
        }

        // PriorKnowledgeBinding: high value with zero uncertainty but no visits
        if value > 0.7 && uncertainty < 0.1 && visits == 0 {
            detected.push(ShortcutSignal {
                shortcut_type: ShortcutType::PriorKnowledgeBinding,
                branch_id,
                confidence: (value + (1.0 - uncertainty)) / 2.0,
                detail: format!(
                    "value={:.2} uncertainty={:.2} visits=0 label={}",
                    value, uncertainty, source_label
                ),
            });
        }

        // Store signals, capping total
        for s in &detected {
            if self.signals.len() < self.max_signals {
                self.signals.push(s.clone());
            }
        }

        detected
    }

    /// Get all shortcut signals for a specific branch.
    pub fn signals_for_branch(&self, branch_id: u64) -> Vec<&ShortcutSignal> {
        self.signals
            .iter()
            .filter(|s| s.branch_id == branch_id)
            .collect()
    }

    /// Get shortcut signals of a specific type.
    pub fn signals_of_type(&self, shortcut_type: ShortcutType) -> Vec<&ShortcutSignal> {
        self.signals
            .iter()
            .filter(|s| s.shortcut_type == shortcut_type)
            .collect()
    }

    /// Calculate a "shortcut discount" for UCB score.
    /// Returns a penalty in [0, 1] to subtract from the raw UCB.
    pub fn shortcut_penalty(&self, branch_id: u64) -> f64 {
        let signals = self.signals_for_branch(branch_id);
        if signals.is_empty() {
            return 0.0;
        }
        // Weighted sum of shortcut confidences, capped at 1.0
        signals.iter().map(|s| s.confidence).sum::<f64>().min(1.0) * 0.3
    }

    /// Clear signals for a specific branch (e.g., after pruning).
    pub fn clear_branch(&mut self, branch_id: u64) {
        self.signals.retain(|s| s.branch_id != branch_id);
    }

    /// Clear all signals.
    pub fn clear_all(&mut self) {
        self.signals.clear();
    }

    pub fn stats(&self) -> ShortcutStats {
        let mut type_counts = HashMap::new();
        for s in &self.signals {
            *type_counts.entry(s.shortcut_type).or_insert(0) += 1;
        }
        ShortcutStats {
            total_signals: self.signals.len() as u64,
            co_coverage_count: *type_counts
                .get(&ShortcutType::EvidenceCoCoverage)
                .unwrap_or(&0) as u64,
            single_clue_count: *type_counts
                .get(&ShortcutType::SingleClueSelectivity)
                .unwrap_or(&0) as u64,
            exposed_constants_count: *type_counts
                .get(&ShortcutType::ExposedConstants)
                .unwrap_or(&0) as u64,
            prior_knowledge_count: *type_counts
                .get(&ShortcutType::PriorKnowledgeBinding)
                .unwrap_or(&0) as u64,
        }
    }
}

impl Default for ShortcutDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ShortcutStats {
    pub total_signals: u64,
    pub co_coverage_count: u64,
    pub single_clue_count: u64,
    pub exposed_constants_count: u64,
    pub prior_knowledge_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_co_coverage_detected() {
        let mut d = ShortcutDetector::new();
        let signals = d.analyze(1, 0.9, 0.1, 0.1, 1, "single_source_page");
        assert!(signals
            .iter()
            .any(|s| s.shortcut_type == ShortcutType::EvidenceCoCoverage));
    }

    #[test]
    fn test_evidence_co_coverage_not_detected_with_multiple_visits() {
        let mut d = ShortcutDetector::new();
        let signals = d.analyze(1, 0.9, 0.1, 0.1, 5, "well_visited_page");
        assert!(!signals
            .iter()
            .any(|s| s.shortcut_type == ShortcutType::EvidenceCoCoverage));
    }

    #[test]
    fn test_single_clue_spike_detected() {
        let mut d = ShortcutDetector::new();
        let signals = d.analyze(1, 0.9, 0.3, 0.1, 0, "first_visit");
        assert!(signals
            .iter()
            .any(|s| s.shortcut_type == ShortcutType::SingleClueSelectivity));
    }

    #[test]
    fn test_exposed_constants_detected() {
        let mut d = ShortcutDetector::new();
        let signals = d.analyze(1, 0.5, 0.3, 0.1, 2, "doi:10.1000/test");
        assert!(signals
            .iter()
            .any(|s| s.shortcut_type == ShortcutType::ExposedConstants));
    }

    #[test]
    fn test_prior_knowledge_binding_detected() {
        let mut d = ShortcutDetector::new();
        let signals = d.analyze(1, 0.8, 0.05, 0.1, 0, "known_topic");
        assert!(signals
            .iter()
            .any(|s| s.shortcut_type == ShortcutType::PriorKnowledgeBinding));
    }

    #[test]
    fn test_shortcut_penalty() {
        let mut d = ShortcutDetector::new();
        d.analyze(1, 0.9, 0.1, 0.1, 1, "page");
        let penalty = d.shortcut_penalty(1);
        assert!(penalty > 0.0);
        assert!(penalty <= 0.3);
    }

    #[test]
    fn test_no_shortcut_penalty_for_clean_branch() {
        let d = ShortcutDetector::new();
        assert!((d.shortcut_penalty(42) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_clear_branch() {
        let mut d = ShortcutDetector::new();
        d.analyze(1, 0.9, 0.1, 0.1, 1, "page");
        assert_eq!(d.signals_for_branch(1).len(), 1);
        d.clear_branch(1);
        assert_eq!(d.signals_for_branch(1).len(), 0);
    }

    #[test]
    fn test_signals_of_type() {
        let mut d = ShortcutDetector::new();
        d.analyze(1, 0.9, 0.1, 0.1, 1, "page");
        d.analyze(2, 0.9, 0.1, 0.1, 1, "page");
        assert_eq!(d.signals_of_type(ShortcutType::EvidenceCoCoverage).len(), 2);
        assert_eq!(
            d.signals_of_type(ShortcutType::SingleClueSelectivity).len(),
            0
        );
    }

    #[test]
    fn test_stats() {
        let mut d = ShortcutDetector::new();
        d.analyze(1, 0.9, 0.1, 0.1, 1, "page");
        d.analyze(2, 0.8, 0.15, 0.1, 0, "doi:10.1000/test");
        let s = d.stats();
        assert_eq!(s.total_signals, 2); // 1 co-coverage + 1 exposed constant (spike not triggered for first due to visit=1)
                                        // Actually: branch 1 gets co-coverage (value=0.9, unc=0.1, visits=1). No spike (value=0.9 > 0.8 threshold but visits=1 so spike not triggered since threshold is value > value_spike_threshold + 0.5 = 0.8 for visits <=1... actually value 0.9 > 0.8 so spike IS triggered too
                                        // Wait: visits <=1 means it could be 0 or 1. So yes, both co-coverage AND spike for branch 1
                                        // Branch 2: visits=0, value=0.8. Co-coverage: value=0.8 = threshold, visits=0 <= 2, unc=0.15 < 0.2. Yes triggered.
                                        // No spike: value=0.8 > 0.8? No, threshold is value > value_spike_threshold + 0.5 = 0.3+0.5 = 0.8, so need > 0.8. 0.8 is NOT > 0.8. So no spike.
                                        // Exposed constants: "doi:10.1000/test" contains "doi:" so yes.
                                        // Prior knowledge: visits=0, value=0.8 > 0.7, unc=0.15 > 0.1... No, 0.15 > 0.1, not < 0.1. So no.
                                        // Total: branch 1: co-coverage (1) + spike (1) = 2 signals. Branch 2: co-coverage (1) + exposed (1) = 2 signals. Total = 4
        assert_eq!(s.total_signals, 4);
    }
}
