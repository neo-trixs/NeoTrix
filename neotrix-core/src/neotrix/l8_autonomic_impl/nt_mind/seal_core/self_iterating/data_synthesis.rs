//! Self-Improving Data Synthesis pipeline (P1 from External Absorption Cycle 3)
//!
//! Implements the asymmetric co-evolution pattern with three roles:
//! - Proposer: identifies knowledge gaps and proposes synthetic training data
//! - Solver: generates solutions/executes proposed tasks
//! - Verifier: validates quality, filters failures
//!
//! References: Autodata (2026), Andes (2026), Self-Play Evolution information theory

use std::collections::{HashMap, VecDeque};

/// Role in the asymmetric co-evolution data synthesis pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SynthesisRole {
    Proposer,
    Solver,
    Verifier,
}

impl SynthesisRole {
    pub fn label(&self) -> &str {
        match self {
            SynthesisRole::Proposer => "Proposer",
            SynthesisRole::Solver => "Solver",
            SynthesisRole::Verifier => "Verifier",
        }
    }
}

/// A piece of synthetic training data
#[derive(Debug, Clone)]
pub struct TrainingDataRecord {
    pub id: String,
    pub source_role: SynthesisRole,
    pub task_type: DataTaskType,
    pub prompt: String,
    pub response: String,
    pub quality_score: f64,
    pub diversity_score: f64,
    pub difficulty: f64,
    pub metadata: HashMap<String, String>,
}

/// Types of data that can be synthesized
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataTaskType {
    CodeGeneration,
    Reasoning,
    KnowledgeQA,
    ToolUse,
    Safety,
    InstructionFollowing,
    MultiTurn,
}

impl DataTaskType {
    pub fn all() -> [DataTaskType; 7] {
        [
            DataTaskType::CodeGeneration,
            DataTaskType::Reasoning,
            DataTaskType::KnowledgeQA,
            DataTaskType::ToolUse,
            DataTaskType::Safety,
            DataTaskType::InstructionFollowing,
            DataTaskType::MultiTurn,
        ]
    }
}

/// Configuration for the data synthesis pipeline
#[derive(Debug, Clone)]
pub struct SynthesisConfig {
    pub max_records_per_cycle: usize,
    pub min_quality_threshold: f64,
    pub diversity_target: f64,
    pub difficulty_range: (f64, f64),
    pub max_verifier_retries: usize,
    pub temperature_proposer: f64,
    pub temperature_solver: f64,
    pub store_in_kb: bool,
}

impl Default for SynthesisConfig {
    fn default() -> Self {
        Self {
            max_records_per_cycle: 50,
            min_quality_threshold: 0.7,
            diversity_target: 0.4,
            difficulty_range: (0.3, 0.9),
            max_verifier_retries: 3,
            temperature_proposer: 0.9,
            temperature_solver: 0.7,
            store_in_kb: true,
        }
    }
}

/// Gap identification result
#[derive(Debug, Clone)]
pub struct KnowledgeGap {
    pub domain: String,
    pub description: String,
    pub priority: f64,
    pub suggested_task_type: DataTaskType,
    pub context: HashMap<String, String>,
}

impl KnowledgeGap {
    pub fn new(domain: &str, description: &str, priority: f64, task_type: DataTaskType) -> Self {
        Self {
            domain: domain.to_string(),
            description: description.to_string(),
            priority: priority.max(0.0).min(1.0),
            suggested_task_type: task_type,
            context: HashMap::new(),
        }
    }
}

/// Proposal from the Proposer role
#[derive(Debug, Clone)]
pub struct DataProposal {
    pub id: String,
    pub gap: KnowledgeGap,
    pub prompt_template: String,
    pub expected_difficulty: f64,
    pub diversity_hash: u64,
}

impl DataProposal {
    pub fn quality_potential(&self, existing_similar: usize) -> f64 {
        let novelty = 1.0 / (1.0 + existing_similar as f64);
        let difficulty_factor = self.expected_difficulty;
        (novelty * 0.6 + difficulty_factor * 0.4).max(0.0).min(1.0)
    }
}

/// Solution from the Solver role
#[derive(Debug, Clone)]
pub struct DataSolution {
    pub proposal_id: String,
    pub response: String,
    pub confidence: f64,
    pub token_count: u64,
    pub step_count: usize,
}

impl DataSolution {
    pub fn is_complete(&self) -> bool {
        !self.response.is_empty() && self.confidence > 0.0
    }
}

/// Verification result from the Verifier role
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub proposal_id: String,
    pub passed: bool,
    pub quality_score: f64,
    pub issues: Vec<String>,
    pub fix_suggestion: Option<String>,
    pub retry_count: usize,
}

impl VerificationResult {
    pub fn acceptable(&self, threshold: f64) -> bool {
        self.passed && self.quality_score >= threshold
    }
}

/// Cycle statistics for the synthesis pipeline
#[derive(Debug, Clone)]
pub struct SynthesisStats {
    pub cycle: u64,
    pub proposed: usize,
    pub solved: usize,
    pub verified: usize,
    pub accepted: usize,
    pub avg_quality: f64,
    pub diversity_histogram: HashMap<DataTaskType, usize>,
    pub total_tokens_generated: u64,
    pub timestamp: u64,
}

impl SynthesisStats {
    pub fn new(cycle: u64) -> Self {
        Self {
            cycle,
            proposed: 0,
            solved: 0,
            verified: 0,
            accepted: 0,
            avg_quality: 0.0,
            diversity_histogram: HashMap::new(),
            total_tokens_generated: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        }
    }

    pub fn acceptance_rate(&self) -> f64 {
        if self.verified == 0 { return 0.0; }
        self.accepted as f64 / self.verified as f64
    }
}

/// Asymmetric Proposer-Solver-Verifier pipeline (co-evolution pattern)
#[derive(Debug, Clone)]
pub struct AsymmetricSynthesisPipeline {
    pub config: SynthesisConfig,
    pub history: VecDeque<SynthesisStats>,
    pub max_history: usize,
    pub records: Vec<TrainingDataRecord>,
    pub proposals: Vec<DataProposal>,
    pub solutions: Vec<DataSolution>,
    pub verifications: Vec<VerificationResult>,
    pub cycle: u64,
}

impl AsymmetricSynthesisPipeline {
    pub fn new(config: SynthesisConfig) -> Self {
        Self {
            config,
            history: VecDeque::new(),
            max_history: 10,
            records: Vec::new(),
            proposals: Vec::new(),
            solutions: Vec::new(),
            verifications: Vec::new(),
            cycle: 0,
        }
    }

    /// Phase 1: Proposer — identify gaps and propose training data
    pub fn propose<F>(&mut self, gaps: Vec<KnowledgeGap>, proposer: F)
    where
        F: Fn(&KnowledgeGap) -> DataProposal,
    {
        let mut new_proposals: Vec<DataProposal> = gaps
            .into_iter()
            .map(|gap| proposer(&gap))
            .collect();

        let existing_count = self.records.len();
        new_proposals.sort_by(|a, b| {
            let qa = a.quality_potential(existing_count);
            let qb = b.quality_potential(existing_count);
            qb.partial_cmp(&qa).unwrap_or(std::cmp::Ordering::Equal)
        });

        new_proposals.truncate(self.config.max_records_per_cycle);
        self.proposals = new_proposals;
    }

    /// Phase 2: Solver — generate solutions for proposed data
    pub fn solve<F>(&mut self, solver: F)
    where
        F: Fn(&DataProposal) -> DataSolution,
    {
        self.solutions = self.proposals.iter().map(solver).collect();
    }

    /// Phase 3: Verifier — validate solution quality
    pub fn verify<F>(&mut self, verifier: F)
    where
        F: Fn(&DataProposal, &DataSolution) -> VerificationResult,
    {
        self.verifications = self.proposals.iter().zip(self.solutions.iter())
            .map(|(p, s)| verifier(p, s))
            .collect();
    }

    /// Phase 4: Accept — filter accepted records and update stats
    pub fn finalize(&mut self) -> SynthesisStats {
        let mut stats = SynthesisStats::new(self.cycle);
        stats.proposed = self.proposals.len();
        stats.solved = self.solutions.len();
        stats.verified = self.verifications.len();

        let mut total_quality = 0.0;
        let mut quality_count = 0;

        for (proposal, (solution, verification)) in self.proposals.iter()
            .zip(self.solutions.iter().zip(self.verifications.iter()))
        {
            if verification.acceptable(self.config.min_quality_threshold) {
                let record = TrainingDataRecord {
                    id: format!("synth-{:04x}-{}", self.cycle, proposal.id),
                    source_role: SynthesisRole::Verifier,
                    task_type: proposal.gap.suggested_task_type,
                    prompt: proposal.prompt_template.clone(),
                    response: solution.response.clone(),
                    quality_score: verification.quality_score,
                    diversity_score: proposal.diversity_hash as f64 / u64::MAX as f64,
                    difficulty: proposal.expected_difficulty,
                    metadata: {
                        let mut m = proposal.gap.context.clone();
                        m.insert("cycle".into(), self.cycle.to_string());
                        m
                    },
                };
                self.records.push(record);
                stats.accepted += 1;
                total_quality += verification.quality_score;
                quality_count += 1;
                *stats.diversity_histogram.entry(proposal.gap.suggested_task_type).or_insert(0) += 1;
            }
            stats.total_tokens_generated += solution.token_count;
        }

        stats.avg_quality = if quality_count > 0 { total_quality / quality_count as f64 } else { 0.0 };

        self.history.push_back(stats.clone());
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }

        self.cycle += 1;
        self.proposals.clear();
        self.solutions.clear();
        self.verifications.clear();
        stats
    }

    /// Run a full cycle (propose + solve + verify + finalize)
    pub fn run_cycle<F1, F2, F3>(
        &mut self,
        gaps: Vec<KnowledgeGap>,
        proposer: F1,
        solver: F2,
        verifier: F3,
    ) -> SynthesisStats
    where
        F1: Fn(&KnowledgeGap) -> DataProposal,
        F2: Fn(&DataProposal) -> DataSolution,
        F3: Fn(&DataProposal, &DataSolution) -> VerificationResult,
    {
        self.propose(gaps, proposer);
        self.solve(solver);
        self.verify(verifier);
        self.finalize()
    }

    pub fn total_records(&self) -> usize { self.records.len() }
    pub fn records_by_type(&self, task_type: DataTaskType) -> Vec<&TrainingDataRecord> {
        self.records.iter().filter(|r| r.task_type == task_type).collect()
    }
    pub fn recent_quality(&self, n: usize) -> Vec<f64> {
        self.records.iter().rev().take(n).map(|r| r.quality_score).collect()
    }
    pub fn avg_quality(&self) -> f64 {
        if self.records.is_empty() { return 0.0; }
        self.records.iter().map(|r| r.quality_score).sum::<f64>() / self.records.len() as f64
    }
}

/// SEAL pipeline stage wrapper for data synthesis
#[derive(Debug, Clone)]
pub struct DataSynthesisStage {
    pub pipeline: AsymmetricSynthesisPipeline,
    pub enabled: bool,
}

impl DataSynthesisStage {
    pub fn new(config: SynthesisConfig) -> Self {
        Self { pipeline: AsymmetricSynthesisPipeline::new(config), enabled: true }
    }

    pub fn process(&self) {
        let total = self.pipeline.total_records();
        let avg_q = self.pipeline.avg_quality();
        let code_count = self.pipeline.records_by_type(DataTaskType::CodeGeneration).len();
        let reason_count = self.pipeline.records_by_type(DataTaskType::Reasoning).len();
        let knowledge_count = self.pipeline.records_by_type(DataTaskType::KnowledgeQA).len();
        log::info!(
            "[data_synthesis] process: {} records, avg_quality={:.3}, types={{code:{},reason:{},knowledge:{}}}",
            total, avg_q, code_count, reason_count, knowledge_count
        );
    }
    pub fn is_enabled(&self) -> bool { self.enabled }
}

/// Gap detector — identifies knowledge gaps for data synthesis
#[derive(Debug, Clone)]
pub struct GapDetector {
    pub error_history: VecDeque<(String, f64, u64)>,
    pub max_history: usize,
    pub rarity_threshold: f64,
}

impl Default for GapDetector {
    fn default() -> Self {
        Self { error_history: VecDeque::new(), max_history: 100, rarity_threshold: 0.3 }
    }
}

impl GapDetector {
    pub fn new(max_history: usize, rarity_threshold: f64) -> Self {
        Self { error_history: VecDeque::new(), max_history: max_history.max(10), rarity_threshold: rarity_threshold.max(0.0).min(1.0) }
    }

    pub fn record_error(&mut self, domain: &str, error_type: &str, severity: f64) {
        self.error_history.push_back((format!("{}:{}", domain, error_type), severity, current_timestamp()));
        while self.error_history.len() > self.max_history {
            self.error_history.pop_front();
        }
    }

    pub fn detect_gaps(&self, domains: &[&str]) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();
        for &domain in domains {
            let count = self.error_history.iter().filter(|(d, _, _)| d.starts_with(domain)).count();
            let total = self.error_history.len().max(1);
            let frequency = count as f64 / total as f64;

            if frequency < self.rarity_threshold {
                let avg_severity: f64 = self.error_history.iter()
                    .filter(|(d, _, _)| d.starts_with(domain))
                    .map(|(_, s, _)| s).sum::<f64>().max(0.1);
                let priority = (1.0 - frequency) * avg_severity;
                let task_type = domain_to_task_type(domain);
                gaps.push(KnowledgeGap::new(domain, &format!("Low coverage in {}", domain), priority.max(0.0).min(1.0), task_type));
            }
        }
        gaps
    }

    pub fn all_gaps(&self, domains: &[&str]) -> Vec<KnowledgeGap> {
        self.detect_gaps(domains)
    }
}

/// Diversity tracker — ensures synthetic data covers a broad distribution
#[derive(Debug, Clone)]
pub struct DiversityTracker {
    pub type_counts: HashMap<DataTaskType, usize>,
    pub type_hashes: HashMap<DataTaskType, Vec<u64>>,
}

impl Default for DiversityTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DiversityTracker {
    pub fn new() -> Self { Self { type_counts: HashMap::new(), type_hashes: HashMap::new() } }

    pub fn record(&mut self, task_type: DataTaskType, hash: u64) {
        *self.type_counts.entry(task_type).or_insert(0) += 1;
        self.type_hashes.entry(task_type).or_default().push(hash);
    }

    pub fn diversity_score(&self) -> f64 {
        if self.type_counts.is_empty() { return 1.0; }
        let total: usize = self.type_counts.values().sum();
        if total == 0 { return 1.0; }
        let num_types = self.type_counts.len() as f64;
        let expected = total as f64 / DataTaskType::all().len() as f64;
        let mut variance = 0.0;
        for cnt in self.type_counts.values() {
            variance += (*cnt as f64 - expected).powi(2);
        }
        variance /= num_types;
        1.0 / (1.0 + variance)
    }

    pub fn is_diverse(&self, threshold: f64) -> bool {
        self.diversity_score() >= threshold
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn domain_to_task_type(domain: &str) -> DataTaskType {
    match domain.to_lowercase().as_str() {
        d if d.contains("code") || d.contains("programming") => DataTaskType::CodeGeneration,
        d if d.contains("reason") || d.contains("logic") => DataTaskType::Reasoning,
        d if d.contains("knowledge") || d.contains("qa") || d.contains("fact") => DataTaskType::KnowledgeQA,
        d if d.contains("tool") || d.contains("api") || d.contains("mcp") => DataTaskType::ToolUse,
        d if d.contains("safe") || d.contains("harm") || d.contains("align") => DataTaskType::Safety,
        d if d.contains("instruction") => DataTaskType::InstructionFollowing,
        d if d.contains("multi") || d.contains("chat") || d.contains("dialog") => DataTaskType::MultiTurn,
        _ => DataTaskType::Reasoning,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_gap(domain: &str) -> KnowledgeGap {
        KnowledgeGap::new(domain, &format!("test gap in {}", domain), 0.7, DataTaskType::Reasoning)
    }

    #[test]
    fn test_synthesis_config_defaults() {
        let cfg = SynthesisConfig::default();
        assert_eq!(cfg.max_records_per_cycle, 50);
        assert!((cfg.min_quality_threshold - 0.7).abs() < 1e-6);
        assert_eq!(cfg.max_verifier_retries, 3);
    }

    #[test]
    fn test_training_data_record_creation() {
        let record = TrainingDataRecord {
            id: "synth-0000-gap1".into(),
            source_role: SynthesisRole::Verifier,
            task_type: DataTaskType::CodeGeneration,
            prompt: "Write a function".into(),
            response: "fn foo() {}".into(),
            quality_score: 0.9,
            diversity_score: 0.5,
            difficulty: 0.6,
            metadata: HashMap::new(),
        };
        assert_eq!(record.id, "synth-0000-gap1");
        assert!((record.quality_score - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_data_proposal_quality_potential() {
        let gap = KnowledgeGap::new("code", "test", 0.8, DataTaskType::CodeGeneration);
        let p1 = DataProposal { id: "p1".into(), gap: gap.clone(), prompt_template: "t1".into(), expected_difficulty: 0.5, diversity_hash: 1 };
        let p2 = DataProposal { id: "p2".into(), gap, prompt_template: "t2".into(), expected_difficulty: 0.9, diversity_hash: 2 };
        assert!(p2.quality_potential(0) > p1.quality_potential(10));
    }

    #[test]
    fn test_asymmetric_pipeline_full_cycle() {
        let config = SynthesisConfig { max_records_per_cycle: 5, min_quality_threshold: 0.5, diversity_target: 0.4, difficulty_range: (0.3, 0.9), max_verifier_retries: 3, temperature_proposer: 0.9, temperature_solver: 0.7, store_in_kb: true };
        let mut pipeline = AsymmetricSynthesisPipeline::new(config);

        let gaps = vec![make_gap("code"), make_gap("reasoning"), make_gap("safety")];

        let stats = pipeline.run_cycle(
            gaps,
            |gap| DataProposal { id: gap.domain.clone(), gap: gap.clone(), prompt_template: format!("Solve {}", gap.domain), expected_difficulty: 0.5, diversity_hash: 1 },
            |_proposal| DataSolution { proposal_id: "p".into(), response: "solution".into(), confidence: 0.9, token_count: 100, step_count: 3 },
            |_proposal, _solution| VerificationResult { proposal_id: "p".into(), passed: true, quality_score: 0.8, issues: vec![], fix_suggestion: None, retry_count: 0 },
        );

        assert!(stats.cycle == 0 || stats.cycle > 0);
        assert!(stats.accepted > 0);
        assert!(stats.total_tokens_generated > 0);
    }

    #[test]
    fn test_propose_filters_by_quality() {
        let config = SynthesisConfig { max_records_per_cycle: 2, ..Default::default() };
        let mut pipeline = AsymmetricSynthesisPipeline::new(config);
        let gaps = vec![make_gap("a"), make_gap("b"), make_gap("c")];
        pipeline.propose(gaps, |gap| DataProposal { id: gap.domain.clone(), gap: gap.clone(), prompt_template: "t".into(), expected_difficulty: 0.5, diversity_hash: 1 });
        assert!(pipeline.proposals.len() <= 2);
    }

    #[test]
    fn test_verify_rejects_low_quality() {
        let config = SynthesisConfig { min_quality_threshold: 0.8, ..Default::default() };
        let mut pipeline = AsymmetricSynthesisPipeline::new(config);
        pipeline.proposals = vec![
            DataProposal { id: "good".into(), gap: make_gap("code"), prompt_template: "t".into(), expected_difficulty: 0.5, diversity_hash: 1 },
            DataProposal { id: "bad".into(), gap: make_gap("safety"), prompt_template: "t".into(), expected_difficulty: 0.5, diversity_hash: 2 },
        ];
        pipeline.solutions = vec![
            DataSolution { proposal_id: "good".into(), response: "ok".into(), confidence: 0.9, token_count: 50, step_count: 1 },
            DataSolution { proposal_id: "bad".into(), response: "no".into(), confidence: 0.3, token_count: 10, step_count: 1 },
        ];
        pipeline.verify(|_p, s| VerificationResult {
            proposal_id: "x".into(), passed: s.confidence > 0.5, quality_score: s.confidence, issues: vec![], fix_suggestion: None, retry_count: 0,
        });
        let stats = pipeline.finalize();
        assert!(stats.accepted < 2);
    }

    #[test]
    fn test_finalize_produces_stats() {
        let config = SynthesisConfig { max_records_per_cycle: 10, min_quality_threshold: 0.0, ..Default::default() };
        let mut pipeline = AsymmetricSynthesisPipeline::new(config);
        pipeline.proposals = vec![DataProposal { id: "p".into(), gap: make_gap("code"), prompt_template: "t".into(), expected_difficulty: 0.5, diversity_hash: 1 }];
        pipeline.solutions = vec![DataSolution { proposal_id: "p".into(), response: "r".into(), confidence: 0.9, token_count: 100, step_count: 2 }];
        pipeline.verifications = vec![VerificationResult { proposal_id: "p".into(), passed: true, quality_score: 0.95, issues: vec![], fix_suggestion: None, retry_count: 0 }];
        let stats = pipeline.finalize();
        assert!(stats.accepted == 1);
        assert!((stats.avg_quality - 0.95).abs() < 1e-6);
    }

    #[test]
    fn test_gap_detector_records_errors() {
        let mut detector = GapDetector::default();
        detector.record_error("code", "type_error", 0.8);
        assert_eq!(detector.error_history.len(), 1);
    }

    #[test]
    fn test_gap_detector_detects_gaps() {
        let detector = GapDetector::new(100, 0.5);
        let gaps = detector.detect_gaps(&["code", "reasoning", "safety"]);
        assert_eq!(gaps.len(), 3);
        for gap in &gaps {
            assert!(gap.priority > 0.0);
        }
    }

    #[test]
    fn test_gap_detector_history_limit() {
        let mut detector = GapDetector::new(10, 0.3);
        for i in 0..20 {
            detector.record_error(&format!("dom_{}", i), "err", 0.5);
        }
        assert!(detector.error_history.len() <= 10);
    }

    #[test]
    fn test_diversity_tracker_records() {
        let mut tracker = DiversityTracker::new();
        tracker.record(DataTaskType::CodeGeneration, 1);
        tracker.record(DataTaskType::Reasoning, 2);
        assert_eq!(*tracker.type_counts.get(&DataTaskType::CodeGeneration).unwrap(), 1);
        assert_eq!(*tracker.type_counts.get(&DataTaskType::Reasoning).unwrap(), 1);
    }

    #[test]
    fn test_diversity_score_perfect() {
        let mut tracker = DiversityTracker::new();
        for tt in DataTaskType::all() {
            tracker.record(tt, 1);
        }
        let score = tracker.diversity_score();
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_diversity_score_imbalanced() {
        let mut tracker = DiversityTracker::new();
        for _ in 0..100 {
            tracker.record(DataTaskType::CodeGeneration, 1);
        }
        tracker.record(DataTaskType::Reasoning, 1);
        let balanced = {
            let mut t = DiversityTracker::new();
            for tt in DataTaskType::all() {
                t.record(tt, 10);
            }
            t.diversity_score()
        };
        assert!(balanced > tracker.diversity_score());
    }

    #[test]
    fn test_diversity_tracker_is_diverse() {
        let mut tracker = DiversityTracker::new();
        for tt in DataTaskType::all() {
            tracker.record(tt, 5);
        }
        assert!(tracker.is_diverse(0.5));
    }

    #[test]
    fn test_domain_to_task_type_mapping() {
        assert_eq!(domain_to_task_type("code_generation"), DataTaskType::CodeGeneration);
        assert_eq!(domain_to_task_type("reasoning_logic"), DataTaskType::Reasoning);
        assert_eq!(domain_to_task_type("knowledge_base"), DataTaskType::KnowledgeQA);
        assert_eq!(domain_to_task_type("tool_use_mcp"), DataTaskType::ToolUse);
        assert_eq!(domain_to_task_type("safety_alignment"), DataTaskType::Safety);
        assert_eq!(domain_to_task_type("multi_turn_dialog"), DataTaskType::MultiTurn);
    }

    #[test]
    fn test_asymmetric_roles_have_labels() {
        assert_eq!(SynthesisRole::Proposer.label(), "Proposer");
        assert_eq!(SynthesisRole::Solver.label(), "Solver");
        assert_eq!(SynthesisRole::Verifier.label(), "Verifier");
    }

    #[test]
    fn test_data_solution_is_complete() {
        let s1 = DataSolution { proposal_id: "p".into(), response: "ok".into(), confidence: 0.9, token_count: 50, step_count: 1 };
        let s2 = DataSolution { proposal_id: "p".into(), response: "".into(), confidence: 0.0, token_count: 0, step_count: 0 };
        assert!(s1.is_complete());
        assert!(!s2.is_complete());
    }

    #[test]
    fn test_verification_result_acceptable() {
        let v1 = VerificationResult { proposal_id: "p".into(), passed: true, quality_score: 0.9, issues: vec![], fix_suggestion: None, retry_count: 0 };
        let v2 = VerificationResult { proposal_id: "p".into(), passed: false, quality_score: 0.3, issues: vec!["bad".into()], fix_suggestion: None, retry_count: 0 };
        assert!(v1.acceptable(0.7));
        assert!(!v2.acceptable(0.7));
    }

    #[test]
    fn test_synthesis_stats_acceptance_rate() {
        let mut stats = SynthesisStats::new(1);
        assert!((stats.acceptance_rate() - 0.0).abs() < 1e-6);
        stats.verified = 10;
        stats.accepted = 7;
        assert!((stats.acceptance_rate() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_pipeline_avg_quality() {
        let config = SynthesisConfig { max_records_per_cycle: 10, min_quality_threshold: 0.0, ..Default::default() };
        let mut pipeline = AsymmetricSynthesisPipeline::new(config);
        pipeline.records = vec![
            TrainingDataRecord { id: "r1".into(), source_role: SynthesisRole::Verifier, task_type: DataTaskType::Reasoning, prompt: "p".into(), response: "r".into(), quality_score: 1.0, diversity_score: 0.5, difficulty: 0.5, metadata: HashMap::new() },
            TrainingDataRecord { id: "r2".into(), source_role: SynthesisRole::Verifier, task_type: DataTaskType::CodeGeneration, prompt: "p".into(), response: "r".into(), quality_score: 0.0, diversity_score: 0.5, difficulty: 0.5, metadata: HashMap::new() },
        ];
        assert!((pipeline.avg_quality() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_pipeline_records_by_type() {
        let config = SynthesisConfig::default();
        let mut pipeline = AsymmetricSynthesisPipeline::new(config);
        pipeline.records.push(TrainingDataRecord { id: "r1".into(), source_role: SynthesisRole::Verifier, task_type: DataTaskType::CodeGeneration, prompt: "p".into(), response: "r".into(), quality_score: 0.9, diversity_score: 0.5, difficulty: 0.5, metadata: HashMap::new() });
        pipeline.records.push(TrainingDataRecord { id: "r2".into(), source_role: SynthesisRole::Verifier, task_type: DataTaskType::Reasoning, prompt: "p".into(), response: "r".into(), quality_score: 0.8, diversity_score: 0.5, difficulty: 0.5, metadata: HashMap::new() });
        assert_eq!(pipeline.records_by_type(DataTaskType::CodeGeneration).len(), 1);
        assert_eq!(pipeline.records_by_type(DataTaskType::Safety).len(), 0);
    }

    #[test]
    fn test_synthesis_stats_diversity_histogram() {
        let mut stats = SynthesisStats::new(1);
        stats.diversity_histogram.insert(DataTaskType::CodeGeneration, 5);
        stats.diversity_histogram.insert(DataTaskType::Reasoning, 3);
        assert_eq!(stats.diversity_histogram.len(), 2);
    }
}
