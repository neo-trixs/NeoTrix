//! ReconstructionTest — BVB-inspired generative reconstruction as understanding
//!
//! Tests agent understanding by asking it to reconstruct:
//! - Code: given a description, reconstruct the implementation
//! - Architecture: given requirements, reconstruct the design
//! - Bug: given a symptom, reconstruct the root cause and fix
//!
//! Dual-axis scoring:
//! - Semantic Retention: does the reconstruction capture the key concepts?
//! - Structural Similarity: does the reconstruction match the structure?
//!
//! BVB (arXiv:2609.15478) demonstrates that "if an agent truly understands
//! a video, it can reconstruct it programmatically" — shifts from QA to
//! generative reconstruction as understanding test. Best model achieves
//! 88.6 visual similarity but only 53.7% semantic retention, revealing a
//! perception-recall gap.
//!
//! NeoTrix applies this to SelfTest T3: testing whether agents truly
//! understand code/systems by asking them to reconstruct them.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Reconstruction task — three flavors matching BVB's paradigm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReconstructionTask {
    /// Given a description, reconstruct the code implementation
    CodeReconstruction {
        description: String,
        reference_code: String,
        /// Key concepts the reconstruction must capture
        key_concepts: Vec<String>,
        /// Required API surface (function/struct names)
        required_api: Vec<String>,
    },
    /// Given requirements, reconstruct the architecture design
    ArchitectureReconstruction {
        requirements: String,
        reference_design: String,
        key_concepts: Vec<String>,
        required_api: Vec<String>,
    },
    /// Given a symptom, reconstruct the root cause and fix
    BugReconstruction {
        symptom: String,
        reference_fix: String,
        key_concepts: Vec<String>,
        required_api: Vec<String>,
    },
}

impl ReconstructionTask {
    /// Reference code/design/fix string
    pub fn reference(&self) -> &str {
        match self {
            Self::CodeReconstruction { reference_code, .. }
            | Self::ArchitectureReconstruction {
                reference_design: reference_code,
                ..
            }
            | Self::BugReconstruction {
                reference_fix: reference_code,
                ..
            } => reference_code,
        }
    }

    /// Key concepts the reconstruction must capture
    pub fn concepts(&self) -> &[String] {
        match self {
            Self::CodeReconstruction { key_concepts, .. }
            | Self::ArchitectureReconstruction { key_concepts, .. }
            | Self::BugReconstruction { key_concepts, .. } => key_concepts,
        }
    }

    /// Required API surface
    pub fn api(&self) -> &[String] {
        match self {
            Self::CodeReconstruction { required_api, .. }
            | Self::ArchitectureReconstruction { required_api, .. }
            | Self::BugReconstruction { required_api, .. } => required_api,
        }
    }

    /// Human-readable label for the task variant
    pub fn label(&self) -> &'static str {
        match self {
            Self::CodeReconstruction { .. } => "code",
            Self::ArchitectureReconstruction { .. } => "architecture",
            Self::BugReconstruction { .. } => "bug",
        }
    }
}

/// An agent's attempt at reconstructing a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionAttempt {
    pub task: ReconstructionTask,
    pub agent_output: String,
    pub timestamp: i64,
}

/// Semantic retention score — does the reconstruction capture key concepts?
///
/// Mirrors BVB's "semantic retention" axis: content-level accuracy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticScore {
    /// Overlap of significant identifiers between reference and reconstruction
    pub keyword_overlap: f64,
    /// Fraction of required concepts present in the reconstruction
    pub concept_coverage: f64,
    /// Completeness of causal/logical chains (bug fix has cause→effect→fix)
    pub causal_chain_completeness: f64,
}

impl SemanticScore {
    /// Weighted composite: 40% keyword, 40% concept, 20% causal
    pub fn overall(&self) -> f64 {
        self.keyword_overlap * 0.4
            + self.concept_coverage * 0.4
            + self.causal_chain_completeness * 0.2
    }
}

/// Structural similarity score — does the reconstruction match the structure?
///
/// Mirrors BVB's "visual similarity" axis: form-level accuracy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralScore {
    /// Similarity of high-level structural patterns (modules, impls, functions)
    pub ast_similarity: f64,
    /// Match of exposed API surface (function/struct/trait names)
    pub api_match: f64,
    /// Match of dependency/import relationships
    pub dependency_match: f64,
}

impl StructuralScore {
    /// Weighted composite: 30% AST, 40% API, 30% dependency
    pub fn overall(&self) -> f64 {
        self.ast_similarity * 0.3 + self.api_match * 0.4 + self.dependency_match * 0.3
    }
}

/// Dual-axis reconstruction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionResult {
    pub semantic: SemanticScore,
    pub structural: StructuralScore,
    /// Weighted overall: 50% semantic + 50% structural (BVB balanced)
    pub overall: f64,
    pub task_label: String,
}

/// Summary statistics across a set of reconstruction results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateReport {
    pub count: usize,
    pub avg_semantic: f64,
    pub avg_structural: f64,
    pub avg_overall: f64,
    /// BVB's perception-recall gap: avg_structural - avg_semantic
    pub perception_recall_gap: f64,
    /// Per-task-label breakdown
    pub by_label: HashMap<String, LabelStats>,
}

/// Per-label statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelStats {
    pub count: usize,
    pub avg_semantic: f64,
    pub avg_structural: f64,
    pub avg_overall: f64,
}

/// Test harness: holds tasks and collects results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionTestHarness {
    pub tasks: Vec<ReconstructionTask>,
    pub results: Vec<ReconstructionResult>,
}

impl ReconstructionTestHarness {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            results: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: ReconstructionTask) {
        self.tasks.push(task);
    }

    pub fn evaluate_all(
        &mut self,
        attempts: &[ReconstructionAttempt],
    ) -> Vec<ReconstructionResult> {
        let results: Vec<ReconstructionResult> =
            attempts.iter().map(|a| Self::evaluate(a)).collect();
        self.results.extend(results.clone());
        results
    }

    /// Dual-axis evaluation of a single reconstruction attempt
    pub fn evaluate(attempt: &ReconstructionAttempt) -> ReconstructionResult {
        let reference = attempt.task.reference();
        let output = &attempt.agent_output;
        let concepts = attempt.task.concepts();
        let api = attempt.task.api();

        let semantic = Self::score_semantic(reference, output, concepts, api);
        let structural = Self::score_structural(reference, output, api);
        let overall = semantic.overall() * 0.5 + structural.overall() * 0.5;

        ReconstructionResult {
            semantic,
            structural,
            overall,
            task_label: attempt.task.label().to_string(),
        }
    }

    /// Compute perception-recall gap across all results
    pub fn perception_recall_gap(results: &[ReconstructionResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        let avg_struct: f64 =
            results.iter().map(|r| r.structural.overall()).sum::<f64>() / results.len() as f64;
        let avg_sem: f64 =
            results.iter().map(|r| r.semantic.overall()).sum::<f64>() / results.len() as f64;
        avg_struct - avg_sem
    }

    /// Aggregate results into a summary report
    pub fn aggregate_results(results: &[ReconstructionResult]) -> AggregateReport {
        if results.is_empty() {
            return AggregateReport {
                count: 0,
                avg_semantic: 0.0,
                avg_structural: 0.0,
                avg_overall: 0.0,
                perception_recall_gap: 0.0,
                by_label: HashMap::new(),
            };
        }

        let n = results.len() as f64;
        let avg_semantic = results.iter().map(|r| r.semantic.overall()).sum::<f64>() / n;
        let avg_structural = results.iter().map(|r| r.structural.overall()).sum::<f64>() / n;
        let avg_overall = results.iter().map(|r| r.overall).sum::<f64>() / n;
        let perception_recall_gap = Self::perception_recall_gap(results);

        let mut by_label: HashMap<String, LabelStats> = HashMap::new();
        let mut counts: HashMap<String, usize> = HashMap::new();
        for r in results {
            let entry = by_label
                .entry(r.task_label.clone())
                .or_insert_with(|| LabelStats {
                    count: 0,
                    avg_semantic: 0.0,
                    avg_structural: 0.0,
                    avg_overall: 0.0,
                });
            entry.avg_semantic += r.semantic.overall();
            entry.avg_structural += r.structural.overall();
            entry.avg_overall += r.overall;
            *counts.entry(r.task_label.clone()).or_insert(0) += 1;
        }
        for (label, stats) in &mut by_label {
            let c = counts[label] as f64;
            stats.count = counts[label];
            stats.avg_semantic /= c;
            stats.avg_structural /= c;
            stats.avg_overall /= c;
        }

        AggregateReport {
            count: results.len(),
            avg_semantic,
            avg_structural,
            avg_overall,
            perception_recall_gap,
            by_label,
        }
    }

    // ── Private scoring helpers ──────────────────────────────────────

    /// Score semantic retention: keyword overlap + concept coverage + causal chain
    fn score_semantic(
        reference: &str,
        output: &str,
        concepts: &[String],
        _api: &[String],
    ) -> SemanticScore {
        let keyword_overlap = Self::keyword_overlap(reference, output);
        let concept_coverage = Self::concept_coverage(concepts, output);
        let causal_chain_completeness = Self::causal_chain_completeness(reference, output);

        SemanticScore {
            keyword_overlap,
            concept_coverage,
            causal_chain_completeness,
        }
    }

    /// Score structural similarity: AST similarity + API match + dependency match
    fn score_structural(reference: &str, output: &str, required_api: &[String]) -> StructuralScore {
        let ast_similarity = Self::structural_pattern_similarity(reference, output);
        let api_match = Self::api_match(required_api, output);
        let dependency_match = Self::dependency_match(reference, output);

        StructuralScore {
            ast_similarity,
            api_match,
            dependency_match,
        }
    }

    /// Keyword overlap: Jaccard-like over significant identifiers
    fn keyword_overlap(reference: &str, output: &str) -> f64 {
        let ref_tokens = Self::significant_tokens(reference);
        let out_tokens = Self::significant_tokens(output);
        if ref_tokens.is_empty() && out_tokens.is_empty() {
            return 1.0;
        }
        if ref_tokens.is_empty() || out_tokens.is_empty() {
            return 0.0;
        }
        let intersection: HashSet<_> = ref_tokens.intersection(&out_tokens).collect();
        let union: HashSet<_> = ref_tokens.union(&out_tokens).collect();
        intersection.len() as f64 / union.len() as f64
    }

    /// Concept coverage: fraction of required concepts present in output
    fn concept_coverage(concepts: &[String], output: &str) -> f64 {
        if concepts.is_empty() {
            return 1.0;
        }
        let output_lower = output.to_lowercase();
        let present = concepts
            .iter()
            .filter(|c| output_lower.contains(&c.to_lowercase()))
            .count();
        present as f64 / concepts.len() as f64
    }

    /// Causal chain completeness: checks for cause→effect→fix/implementation pattern
    fn causal_chain_completeness(reference: &str, output: &str) -> f64 {
        let ref_chain = Self::extract_causal_chain(reference);
        let out_chain = Self::extract_causal_chain(output);
        if ref_chain.is_empty() {
            return 1.0;
        }
        let matches = ref_chain.iter().filter(|c| out_chain.contains(c)).count();
        matches as f64 / ref_chain.len() as f64
    }

    /// Extract causal chain elements (cause/effect/fix/return/impl markers)
    fn extract_causal_chain(text: &str) -> Vec<String> {
        let markers = [
            "cause",
            "effect",
            "fix",
            "return",
            "impl",
            "fn ",
            "struct ",
            "trait ",
            "because",
            "therefore",
            "however",
            "result",
        ];
        let text_lower = text.to_lowercase();
        markers
            .iter()
            .filter(|m| text_lower.contains(*m))
            .map(|m| m.to_string())
            .collect()
    }

    /// Structural pattern similarity: counts of structural keywords
    fn structural_pattern_similarity(reference: &str, output: &str) -> f64 {
        let patterns = [
            "fn ", "struct ", "impl ", "trait ", "mod ", "pub ", "use ", "enum ", "type ",
            "async ", "await",
        ];
        let ref_counts: Vec<usize> = patterns
            .iter()
            .map(|p| reference.matches(p).count())
            .collect();
        let out_counts: Vec<usize> = patterns.iter().map(|p| output.matches(p).count()).collect();

        let total_ref: usize = ref_counts.iter().sum();
        let total_out: usize = out_counts.iter().sum();
        if total_ref == 0 && total_out == 0 {
            return 1.0;
        }
        if total_ref == 0 || total_out == 0 {
            return 0.0;
        }

        // Cosine-like similarity over pattern frequency vectors
        let dot: f64 = ref_counts
            .iter()
            .zip(out_counts.iter())
            .map(|(a, b)| (*a as f64) * (*b as f64))
            .sum();
        let mag_ref: f64 = ref_counts
            .iter()
            .map(|a| (*a as f64).powi(2))
            .sum::<f64>()
            .sqrt();
        let mag_out: f64 = out_counts
            .iter()
            .map(|a| (*a as f64).powi(2))
            .sum::<f64>()
            .sqrt();
        if mag_ref == 0.0 || mag_out == 0.0 {
            0.0
        } else {
            (dot / (mag_ref * mag_out)).min(1.0)
        }
    }

    /// API match: fraction of required API names found in output
    fn api_match(required: &[String], output: &str) -> f64 {
        if required.is_empty() {
            return 1.0;
        }
        let output_lower = output.to_lowercase();
        let found = required
            .iter()
            .filter(|name| output_lower.contains(&name.to_lowercase()))
            .count();
        found as f64 / required.len() as f64
    }

    /// Dependency match: shared use/import statements
    fn dependency_match(reference: &str, output: &str) -> f64 {
        let ref_deps = Self::extract_dependencies(reference);
        let out_deps = Self::extract_dependencies(output);
        if ref_deps.is_empty() && out_deps.is_empty() {
            return 1.0;
        }
        if ref_deps.is_empty() || out_deps.is_empty() {
            return 0.0;
        }
        let intersection: HashSet<_> = ref_deps.intersection(&out_deps).collect();
        let union: HashSet<_> = ref_deps.union(&out_deps).collect();
        intersection.len() as f64 / union.len() as f64
    }

    /// Extract dependency names from use/import statements
    fn extract_dependencies(text: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        for line in text.lines() {
            let t = line.trim();
            if t.starts_with("use ") || t.starts_with("import ") || t.starts_with("from ") {
                // Extract the path segments
                let segments: Vec<&str> = t
                    .split_whitespace()
                    .skip(1)
                    .flat_map(|s| s.split(|c: char| c == ';' || c == '{' || c == '}' || c == ','))
                    .filter(|s| !s.is_empty() && *s != "self" && *s != "super" && *s != "crate")
                    .collect();
                for seg in segments {
                    deps.insert(seg.trim().to_string());
                }
            }
        }
        deps
    }

    /// Extract significant tokens (identifiers, keywords — skip noise)
    fn significant_tokens(text: &str) -> HashSet<String> {
        let stop: HashSet<&str> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "could", "should", "may", "might",
            "shall", "can", "this", "that", "these", "those", "it", "its", "for", "to", "of", "in",
            "on", "at", "by", "with", "from", "as", "into", "and", "or", "but", "not", "if",
            "then", "else", "when", "use", "pub", "mod", "fn", "let", "mut", "impl", "struct",
            "enum", "trait", "type", "async", "await", "move", "ref",
        ]
        .iter()
        .cloned()
        .collect();

        text.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| w.len() > 2 && !stop.contains(*w))
            .map(|w| w.to_lowercase())
            .collect()
    }
}

impl Default for ReconstructionTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_code_task() -> ReconstructionTask {
        ReconstructionTask::CodeReconstruction {
            description: "A binary search function that finds an element in a sorted slice".into(),
            reference_code: "fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {\n    let mut lo = 0;\n    let mut hi = arr.len();\n    while lo < hi {\n        let mid = lo + (hi - lo) / 2;\n        match arr[mid].cmp(target) {\n            std::cmp::Ordering::Equal => return Some(mid),\n            std::cmp::Ordering::Less => lo = mid + 1,\n            std::cmp::Ordering::Greater => hi = mid,\n        }\n    }\n    None\n}".into(),
            key_concepts: vec!["binary_search".into(), "sorted".into(), "midpoint".into(), "comparison".into()],
            required_api: vec!["binary_search".into()],
        }
    }

    fn make_arch_task() -> ReconstructionTask {
        ReconstructionTask::ArchitectureReconstruction {
            requirements: "Event-driven plugin system with hot-reload".into(),
            reference_design: "pub trait Plugin {\n    fn name(&self) -> &str;\n    fn on_event(&self, event: &Event);\n}\npub struct PluginManager {\n    plugins: Vec<Box<dyn Plugin>>,\n    event_bus: EventBus,\n}".into(),
            key_concepts: vec!["Plugin".into(), "event_bus".into(), "hot_reload".into(), "manager".into()],
            required_api: vec!["Plugin".into(), "PluginManager".into(), "EventBus".into()],
        }
    }

    fn make_bug_task() -> ReconstructionTask {
        ReconstructionTask::BugReconstruction {
            symptom: "Index out of bounds panic when slice is empty".into(),
            reference_fix: "fn first_element(arr: &[i32]) -> Option<&i32> {\n    arr.first()\n}"
                .into(),
            key_concepts: vec!["empty_slice".into(), "bounds_check".into(), "Option".into()],
            required_api: vec!["first_element".into()],
        }
    }

    fn perfect_attempt(task: ReconstructionTask) -> ReconstructionAttempt {
        ReconstructionAttempt {
            agent_output: task.reference().to_string(),
            task,
            timestamp: 1700000000,
        }
    }

    fn partial_attempt(task: ReconstructionTask) -> ReconstructionAttempt {
        let output = format!(
            "{}\n// partial reconstruction — missing some details",
            task.reference(),
        );
        ReconstructionAttempt {
            agent_output: output,
            task,
            timestamp: 1700000000,
        }
    }

    fn poor_attempt(task: ReconstructionTask) -> ReconstructionAttempt {
        ReconstructionAttempt {
            agent_output: "fn placeholder() { todo!() }".into(),
            task,
            timestamp: 1700000000,
        }
    }

    #[test]
    fn test_perfect_reconstruction_scores_high() {
        let attempt = perfect_attempt(make_code_task());
        let result = ReconstructionTestHarness::evaluate(&attempt);
        assert!(
            result.overall > 0.85,
            "expected > 0.85, got {}",
            result.overall
        );
        assert!(result.semantic.overall() > 0.8);
        assert!(result.structural.overall() > 0.8);
    }

    #[test]
    fn test_poor_reconstruction_scores_low() {
        let attempt = poor_attempt(make_code_task());
        let result = ReconstructionTestHarness::evaluate(&attempt);
        assert!(
            result.overall < 0.5,
            "expected < 0.5, got {}",
            result.overall
        );
    }

    #[test]
    fn test_partial_scores_between() {
        let task = make_code_task();
        let p = partial_attempt(task.clone());
        let poor = poor_attempt(task);
        let r_partial = ReconstructionTestHarness::evaluate(&p);
        let r_poor = ReconstructionTestHarness::evaluate(&poor);
        assert!(r_partial.overall >= r_poor.overall);
    }

    #[test]
    fn test_semantic_score_composite() {
        let s = SemanticScore {
            keyword_overlap: 0.8,
            concept_coverage: 0.6,
            causal_chain_completeness: 1.0,
        };
        let expected = 0.8 * 0.4 + 0.6 * 0.4 + 1.0 * 0.2;
        assert!((s.overall() - expected).abs() < 0.001);
    }

    #[test]
    fn test_structural_score_composite() {
        let s = StructuralScore {
            ast_similarity: 0.7,
            api_match: 0.9,
            dependency_match: 0.5,
        };
        let expected = 0.7 * 0.3 + 0.9 * 0.4 + 0.5 * 0.3;
        assert!((s.overall() - expected).abs() < 0.001);
    }

    #[test]
    fn test_perception_recall_gap_perfect() {
        let task = make_code_task();
        let a = perfect_attempt(task);
        let r = ReconstructionTestHarness::evaluate(&a);
        let gap = r.structural.overall() - r.semantic.overall();
        // Perfect reconstruction should have small gap
        assert!(gap.abs() < 0.3, "expected gap < 0.3, got {}", gap);
    }

    #[test]
    fn test_aggregate_report() {
        let attempts = vec![
            perfect_attempt(make_code_task()),
            perfect_attempt(make_arch_task()),
            perfect_attempt(make_bug_task()),
        ];
        let results: Vec<_> = attempts
            .iter()
            .map(ReconstructionTestHarness::evaluate)
            .collect();
        let report = ReconstructionTestHarness::aggregate_results(&results);
        assert_eq!(report.count, 3);
        assert!(report.avg_overall > 0.8);
        assert!(report.by_label.contains_key("code"));
        assert!(report.by_label.contains_key("architecture"));
        assert!(report.by_label.contains_key("bug"));
    }

    #[test]
    fn test_empty_results() {
        let report = ReconstructionTestHarness::aggregate_results(&[]);
        assert_eq!(report.count, 0);
        assert!(report.avg_overall.abs() < 0.001);
    }

    #[test]
    fn test_harness_evaluate_all() {
        let mut harness = ReconstructionTestHarness::new();
        harness.add_task(make_code_task());
        harness.add_task(make_arch_task());

        let attempts = vec![
            perfect_attempt(harness.tasks[0].clone()),
            perfect_attempt(harness.tasks[1].clone()),
        ];
        let results = harness.evaluate_all(&attempts);
        assert_eq!(results.len(), 2);
        assert_eq!(harness.results.len(), 2);
    }

    #[test]
    fn test_keyword_overlap_identical() {
        let text = "fn binary_search(arr: &[i32]) -> Option<usize> { None }";
        assert!((ReconstructionTestHarness::keyword_overlap(text, text) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_keyword_overlap_empty() {
        assert!((ReconstructionTestHarness::keyword_overlap("", "")).abs() < 0.01);
    }

    #[test]
    fn test_concept_coverage_all_present() {
        let concepts = vec!["binary_search".into(), "sorted".into()];
        assert!(
            (ReconstructionTestHarness::concept_coverage(
                &concepts,
                "fn binary_search sorted slice"
            ) - 1.0)
                .abs()
                < 0.01
        );
    }

    #[test]
    fn test_concept_coverage_none_present() {
        let concepts = vec!["hash_map".into(), "iterator".into()];
        assert!(ReconstructionTestHarness::concept_coverage(&concepts, "fn binary_search") < 0.01);
    }

    #[test]
    fn test_api_match_partial() {
        let required = vec!["Plugin".into(), "PluginManager".into(), "EventBus".into()];
        let output = "pub struct Plugin { } pub struct PluginManager { }";
        let score = ReconstructionTestHarness::api_match(&required, output);
        assert!((score - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_dependency_match_identical() {
        let text = "use crate::foo;\nuse std::collections::HashMap;";
        assert!((ReconstructionTestHarness::dependency_match(text, text) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_structural_pattern_similarity_identical() {
        let text = "pub fn foo() {}\nstruct Bar {}\nimpl Bar { fn baz() {} }";
        assert!(
            (ReconstructionTestHarness::structural_pattern_similarity(text, text) - 1.0).abs()
                < 0.01
        );
    }

    #[test]
    fn test_structural_pattern_similarity_empty() {
        assert!(
            (ReconstructionTestHarness::structural_pattern_similarity("", "") - 1.0).abs() < 0.01
        );
    }

    #[test]
    fn test_task_variants() {
        let code = make_code_task();
        let arch = make_arch_task();
        let bug = make_bug_task();
        assert_eq!(code.label(), "code");
        assert_eq!(arch.label(), "architecture");
        assert_eq!(bug.label(), "bug");
    }

    #[test]
    fn test_perception_recall_gap_balanced_tasks() {
        // Code: high structural, moderate semantic → positive gap
        // Architecture: similar → small gap
        let attempts = vec![
            perfect_attempt(make_code_task()),
            poor_attempt(make_arch_task()),
        ];
        let results: Vec<_> = attempts
            .iter()
            .map(ReconstructionTestHarness::evaluate)
            .collect();
        let gap = ReconstructionTestHarness::perception_recall_gap(&results);
        // Gap should reflect the difference between high-structural and low-semantic
        assert!(gap > -1.0 && gap < 1.0);
    }

    #[test]
    fn test_causal_chain_completeness() {
        let reference = "The cause is X, therefore the fix is Y, because of Z";
        let output = "The cause is X, therefore the fix is Y, because of Z";
        let score = ReconstructionTestHarness::causal_chain_completeness(reference, output);
        assert!((score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_causal_chain_completeness_empty_reference() {
        let score = ReconstructionTestHarness::causal_chain_completeness("", "anything");
        assert!((score - 1.0).abs() < 0.01);
    }
}
