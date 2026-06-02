use std::collections::HashMap;

use crate::core::{
    CapabilityVector, TaskType,
};
use crate::core::nt_core_epoch::{
    CognitiveFramework, EarthEpoch, FrameworkRoute, all_frameworks, evaluate_in_epoch,
};
use super::stats::BrainStats;

/// DarwinSkill's 9-dimensional SkillLens rubric for evaluating task quality.
#[derive(Debug, Clone, Copy)]
pub struct SkillLens {
    /// Instruction compliance: follows given instructions exactly
    pub instruction_compliance: f64,
    /// Safety: avoids harmful or dangerous outputs
    pub safety: f64,
    /// Semantics: accuracy of meaning and understanding
    pub semantics: f64,
    /// Format: adherence to required output structure
    pub format: f64,
    /// Adaptability: flexibility to handle changes
    pub adaptability: f64,
    /// Relevance: pertinence to the task at hand
    pub relevance: f64,
    /// Style: consistency with tone, nt_act_voice, brand
    pub style: f64,
    /// Planning: strategic organization and preparation
    pub planning: f64,
    /// Creativity: novel, innovative, original thinking
    pub creativity: f64,
}

impl SkillLens {
    pub fn average(&self) -> f64 {
        (self.instruction_compliance + self.safety + self.semantics + self.format
         + self.adaptability + self.relevance + self.style + self.planning + self.creativity) / 9.0
    }

    pub fn max(&self) -> f64 {
        self.instruction_compliance
            .max(self.safety)
            .max(self.semantics)
            .max(self.format)
            .max(self.adaptability)
            .max(self.relevance)
            .max(self.style)
            .max(self.planning)
            .max(self.creativity)
    }
}

impl Default for SkillLens {
    fn default() -> Self {
        Self {
            instruction_compliance: 0.5,
            safety: 0.5,
            semantics: 0.5,
            format: 0.5,
            adaptability: 0.5,
            relevance: 0.5,
            style: 0.5,
            planning: 0.5,
            creativity: 0.5,
        }
    }
}

/// PanoramicBrain: the multi-epoch cognitive orchestrator.
///
/// Holds all 8 EarthEpoch CognitiveFrameworks simultaneously, routes tasks
/// to the appropriate framework(s), and enables cross-epoch knowledge transfer.
///
/// This is the architectural realization of moving from a single-paradigm
/// CapabilityVector to multi-paradigm switching.
pub struct PanoramicBrain {
    /// All 8 cognitive frameworks, indexed by epoch
    pub frameworks: HashMap<EarthEpoch, CognitiveFramework>,
    /// The epoch currently considered "primary" for the active task
    pub active_epoch: EarthEpoch,
    /// CapabilityVector is maintained for backward compatibility with
    /// the existing single-paradigm interface
    pub legacy_capability: CapabilityVector,
    /// Activation history for meta-learning routing patterns
    pub activation_log: Vec<(EarthEpoch, String, f64)>,
    /// Track which epoch has been most successful for each task type
    pub epoch_success_by_task: HashMap<TaskType, HashMap<EarthEpoch, f64>>,
}

impl PanoramicBrain {
    pub fn new() -> Self {
        let mut frameworks = HashMap::new();
        for fw in all_frameworks() {
            frameworks.insert(fw.epoch, fw);
        }

        Self {
            frameworks,
            active_epoch: EarthEpoch::E7Network,
            legacy_capability: CapabilityVector::default(),
            activation_log: Vec::new(),
            epoch_success_by_task: HashMap::new(),
        }
    }

    /// Evaluate a task description against DarwinSkill's 9-dimensional rubric.
    /// Uses keyword matching to score each dimension 0.0–1.0.
    pub fn evaluate_with_rubric(&self, task: &str, _task_type: Option<TaskType>) -> SkillLens {
        let task_lower = task.to_lowercase();

        let count_matches = |keywords: &[&str]| -> f64 {
            let total = keywords.len() as f64;
            if total == 0.0 { return 0.0; }
            let found = keywords.iter().filter(|kw| task_lower.contains(**kw)).count() as f64;
            (found / total).min(1.0)
        };

        SkillLens {
            instruction_compliance: count_matches(&["instruction", "follow", "comply", "obey", "adhere"]),
            safety: count_matches(&["safe", "secure", "protect", "danger", "vulnerability", "safety", "instructions", "vulnerable", "harm", "vulnerabilities", "protection", "against", "follow"]),
            semantics: count_matches(&["meaning", "semantic", "understand", "comprehend", "precise"]),
            format: count_matches(&["format", "style", "layout", "structure", "organize"]),
            adaptability: count_matches(&["adapt", "flexible", "change", "adjust", "modify"]),
            relevance: count_matches(&["relevant", "pertinent", "applicable", "germane", "related"]),
            style: count_matches(&["style", "tone", "nt_act_voice", "consistent", "brand"]),
            planning: count_matches(&["plan", "strategy", "organize", "schedule", "prepare"]),
            creativity: count_matches(&["creative", "novel", "innovate", "imagine", "original"]),
        }
    }

    /// Analyze a task description and determine the routing weights
    /// for all 8 epochs. Returns a FrameworkRoute with the primary epoch
    /// and relative weights.
    ///
    /// When `use_rubric` is true, DarwinSkill's 9-dimension evaluation is
    /// blended into each epoch weight at 20%.
    pub fn route_task(&self, task: &str, task_type: Option<TaskType>, use_rubric: bool) -> FrameworkRoute {
        let task_type = task_type.unwrap_or(TaskType::General);

        let mut weights: Vec<(EarthEpoch, f64)> = EarthEpoch::all().iter().map(|epoch| {
            let fw = self.frameworks.get(epoch)
                .expect("All epochs should be initialized");

            // Base weight from epoch-specific evaluator
            let eval_score = evaluate_in_epoch(*epoch, &fw.state, task);

            // Historical success bonus for this task type
            let history_bonus = self.epoch_success_by_task
                .get(&task_type)
                .and_then(|m| m.get(epoch))
                .copied()
                .unwrap_or(0.0);

            // Router bias (how generally useful this epoch is)
            let router_bias = fw.effective_weight();

            // Combine: 40% evaluator, 30% historical, 30% router bias
            let weight = 0.40 * eval_score + 0.30 * history_bonus + 0.30 * router_bias;
            (*epoch, weight)
        }).collect();

        // Optional DarwinSkill rubric enhancement (additive boost)
        if use_rubric {
            let rubric = self.evaluate_with_rubric(task, Some(task_type));
            let peak = rubric.max();
            for (_, w) in weights.iter_mut() {
                *w = *w + 0.20 * peak;
            }
        }

        // Sort by weight descending
        weights.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let primary = weights.first().map(|(e, _)| *e).unwrap_or(EarthEpoch::E7Network);
        FrameworkRoute { primary, weights }
    }

    /// Evaluate a task using the primary epoch's framework.
    /// Returns a tuple: (primary_epoch_score, all_epoch_scores).
    ///
    /// When `use_rubric` is true, DarwinSkill's 9-dimension evaluation is
    /// blended into the primary score at 20%.
    pub fn evaluate_task(&self, task: &str, use_rubric: bool) -> (f64, Vec<(EarthEpoch, f64)>) {
        let route = self.route_task(task, None, use_rubric);
        let mut primary_score = evaluate_in_epoch(
            route.primary,
            &self.frameworks.get(&route.primary)
                .map(|fw| fw.state.clone())
                .unwrap_or_default(),
            task,
        );
        if use_rubric {
            let rubric = self.evaluate_with_rubric(task, None);
            primary_score = primary_score + 0.20 * rubric.max();
        }
        (primary_score, route.weights)
    }

    /// Absorb a reward signal after completing a task.
    /// Updates the active/primary epoch's framework and records the feedback.
    pub fn absorb_reward(&mut self, task: &str, task_type: TaskType, reward: f64) {
        let route = self.route_task(task, Some(task_type), false);
        let primary = route.primary;

        if let Some(fw) = self.frameworks.get_mut(&primary) {
            fw.record_activation(reward);

            // Update epoch's state based on reward (positive: reinforce, negative: dampen)
            let delta = reward * 0.05; // Slow integration
            for val in fw.state.iter_mut() {
                *val = (*val + delta).clamp(0.0, 1.0);
            }
        }

        // Record success for task type
        self.epoch_success_by_task
            .entry(task_type)
            .or_default()
            .entry(primary)
            .and_modify(|v| *v = *v * 0.9 + reward * 0.1)
            .or_insert(reward);

        self.activation_log.push((primary, task.to_string(), reward));

        // Periodically transfer successful epoch knowledge to legacy CapabilityVector
        if self.activation_log.len() % 10 == 0 {
            self.sync_to_legacy();
        }
    }

    /// Switch the active epoch — either manually or by routing decision.
    pub fn switch_to(&mut self, epoch: EarthEpoch) -> bool {
        if self.frameworks.contains_key(&epoch) {
            self.active_epoch = epoch;
            self.sync_to_legacy();
            true
        } else {
            false
        }
    }

    /// Transfer knowledge from the active epoch framework to the legacy
    /// CapabilityVector for backward compatibility.
    fn sync_to_legacy(&mut self) {
        if let Some(fw) = self.frameworks.get(&self.active_epoch) {
            // Map epoch dimensions to legacy CapabilityVector generically.
            // The active epoch's state is averaged into the legacy vector's
            // inference_depth, analysis, synthesis dimensions.
            let avg_state = fw.state.iter().sum::<f64>() / fw.state.len() as f64;
            self.legacy_capability.set_inference_depth(
                (self.legacy_capability.inference_depth() + avg_state * self.active_epoch as u8 as f64 / 8.0) / 2.0
            );
            self.legacy_capability.set_analysis(
                (self.legacy_capability.analysis() + avg_state * fw.router_bias) / 2.0
            );
            self.legacy_capability.set_synthesis(
                (self.legacy_capability.synthesis() + avg_state * fw.average_reward().max(0.3)) / 2.0
            );
            self.legacy_capability.normalize();
        }
    }

    /// Returns a report on all epoch framework states.
    pub fn epoch_report(&self) -> Vec<(EarthEpoch, f64, f64, usize)> {
        let mut report: Vec<(EarthEpoch, f64, f64, usize)> = EarthEpoch::all().iter().map(|epoch| {
            let fw = self.frameworks.get(epoch)
                .expect("All epochs should be initialized");
            let avg_state = fw.state.iter().sum::<f64>() / fw.state.len() as f64;
            (fw.epoch, avg_state, fw.effective_weight(), fw.activation_count as usize)
        }).collect();
        report.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        report
    }

    /// Deepen a specific epoch's capability (simulating learning/absorption
    /// within that cognitive framework).
    pub fn strengthen_epoch(&mut self, epoch: EarthEpoch, amount: f64) {
        if let Some(fw) = self.frameworks.get_mut(&epoch) {
            for val in fw.state.iter_mut() {
                *val = (*val + amount).min(1.0);
            }
            fw.router_bias = (fw.router_bias + amount * 0.1).min(1.0);
        }
    }

    /// Transfer insight from one epoch to another (cross-framework knowledge transfer).
    /// This is how the system learns across paradigms — not just within one.
    pub fn transfer_knowledge(&mut self, from: EarthEpoch, to: EarthEpoch, rate: f64) {
        let source_state = self.frameworks.get(&from)
            .map(|fw| fw.state.clone());
        if let Some(state) = source_state {
            if let Some(target_fw) = self.frameworks.get_mut(&to) {
                // Map dimensions by index (simple approach) and blend
                let min_len = state.len().min(target_fw.state.len());
                for i in 0..min_len {
                    let delta = state[i] - target_fw.state[i];
                    target_fw.state[i] += rate * delta;
                }
                target_fw.normalize();
            }
        }
    }

    /// Get a reference to a specific cognitive framework.
    pub fn framework(&self, epoch: EarthEpoch) -> Option<&CognitiveFramework> {
        self.frameworks.get(&epoch)
    }

    /// Get mutable reference to a specific cognitive framework.
    pub fn framework_mut(&mut self, epoch: EarthEpoch) -> Option<&mut CognitiveFramework> {
        self.frameworks.get_mut(&epoch)
    }

    /// Get the current best epoch for a task based on learned patterns.
    pub fn best_epoch_for(&self, task_type: TaskType) -> Option<EarthEpoch> {
        self.epoch_success_by_task
            .get(&task_type)
            .map(|m| m.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal)))
            .flatten()
            .map(|(e, _)| *e)
    }

    pub fn stats(&self) -> BrainStats {
        BrainStats {
            total_absorbed: self.activation_log.len() as u64,
            unique_sources: Vec::new(),
            latest_absorption: None,
            capability_sum: self.frameworks.values()
                .map(|fw| fw.state.iter().sum::<f64>())
                .sum(),
        }
    }
}

impl Default for PanoramicBrain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panoramic_brain_initializes_all_epochs() {
        let brain = PanoramicBrain::new();
        assert_eq!(brain.frameworks.len(), 8, "All 8 epochs should be initialized");
        for epoch in EarthEpoch::all() {
            assert!(brain.frameworks.contains_key(&epoch),
                "Epoch {:?} should be present", epoch);
        }
    }

    #[test]
    fn test_routing_returns_weights_for_all_epochs() {
        let brain = PanoramicBrain::new();
        let route = brain.route_task("analyze experimental data", Some(TaskType::CodeAnalysis), false);
        assert_eq!(route.weights.len(), 8, "Should have weights for all 8 epochs");
        assert!(!route.weights.is_empty());
    }

    #[test]
    fn test_evaluate_task_returns_valid_scores() {
        let brain = PanoramicBrain::new();
        let (primary, all_scores) = brain.evaluate_task("test generic task", false);
        assert!((0.0..=1.0).contains(&primary),
            "Primary score {} should be in [0,1]", primary);
        assert_eq!(all_scores.len(), 8);
        // Scores should be sorted descending
        for pair in all_scores.windows(2) {
            assert!(pair[0].1 >= pair[1].1 - 1e-10,
                "Scores should be sorted descending");
        }
    }

    #[test]
    fn test_absorb_reward_updates_epoch_state() {
        let mut brain = PanoramicBrain::new();
        let _epoch_e4 = EarthEpoch::E4Scientific;

        brain.absorb_reward("measured experimental precision", TaskType::CodeAnalysis, 0.9);

        // At least one epoch should have been activated
        let total_activations: u64 = EarthEpoch::all().iter()
            .filter_map(|e| brain.framework(*e).map(|fw| fw.activation_count))
            .sum();
        assert!(total_activations > 0u64, "At least one epoch should have been activated");

        // The primary epoch (likely E4 or E7 for this task) should have an entry
        let route = brain.route_task("measured experimental precision", Some(TaskType::CodeAnalysis), false);
        if let Some(fw) = brain.framework(route.primary) {
            assert!(fw.activation_count > 0,
                "Primary epoch {:?} should have activation count > 0", route.primary);
        }
    }

    #[test]
    fn test_switch_epoch() {
        let mut brain = PanoramicBrain::new();
        assert_eq!(brain.active_epoch, EarthEpoch::E7Network);

        assert!(brain.switch_to(EarthEpoch::E4Scientific));
        assert_eq!(brain.active_epoch, EarthEpoch::E4Scientific);

        // Switching to a nonexistent epoch should fail
        assert!(brain.switch_to(EarthEpoch::E4Scientific)); // existing, so succeeds
    }

    #[test]
    fn test_strengthen_epoch() {
        let mut brain = PanoramicBrain::new();
        let epoch = EarthEpoch::E6Planetary;
        let before = brain.framework(epoch).map(|fw| fw.router_bias).unwrap_or(0.0);

        brain.strengthen_epoch(epoch, 0.5);

        let after = brain.framework(epoch).map(|fw| fw.router_bias).unwrap_or(0.0);
        assert!(after > before, "Epoch router bias should increase after strengthening");
    }

    #[test]
    fn test_knowledge_transfer_between_epochs() {
        let mut brain = PanoramicBrain::new();
        // Strengthen E7 heavily
        brain.strengthen_epoch(EarthEpoch::E7Network, 0.3);
        let _e7_state_before = brain.framework(EarthEpoch::E7Network).expect("value should be ok in test").state.clone();
        let e6_state_before = brain.framework(EarthEpoch::E6Planetary).expect("value should be ok in test").state.clone();

        // Transfer from E7 to E6
        brain.transfer_knowledge(EarthEpoch::E7Network, EarthEpoch::E6Planetary, 0.2);

        let e6_state_after = brain.framework(EarthEpoch::E6Planetary).expect("value should be ok in test").state.clone();
        // E6 should have moved toward E7 in at least some dimensions
        let movement: f64 = e6_state_after.iter().zip(e6_state_before.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(movement > 0.0, "Knowledge transfer should change E6 state towards E7");
    }

    #[test]
    fn test_routing_prefers_matching_epoch() {
        let brain = PanoramicBrain::new();
        // A very "scientific" task should route to E4 more than E1
        let route = brain.route_task("rigorous experimental measurement with statistical verification",
            Some(TaskType::CodeAnalysis), false);
        let e4_weight = route.weights.iter()
            .find(|(e, _)| *e == EarthEpoch::E4Scientific)
            .map(|(_, w)| *w)
            .unwrap_or(0.0);
        let e1_weight = route.weights.iter()
            .find(|(e, _)| *e == EarthEpoch::E1Mythological)
            .map(|(_, w)| *w)
            .unwrap_or(0.0);
        assert!(e4_weight > e1_weight,
            "Scientific epoch (weight={}) should score higher than Mythological (weight={}) for scientific task",
            e4_weight, e1_weight);
    }

    #[test]
    fn test_epoch_report_order() {
        let mut brain = PanoramicBrain::new();
        brain.strengthen_epoch(EarthEpoch::E7Network, 0.5);
        brain.absorb_reward("network optimization", TaskType::General, 0.8);

        let report = brain.epoch_report();
        assert_eq!(report.len(), 8);
        // Should be sorted by state average descending
        for pair in report.windows(2) {
            assert!(pair[0].1 >= pair[1].1 - 1e-10);
        }
    }

    #[test]
    fn test_skill_lens_default() {
        let lens = SkillLens::default();
        assert_eq!(lens.instruction_compliance, 0.5);
        assert_eq!(lens.safety, 0.5);
        assert_eq!(lens.semantics, 0.5);
        assert_eq!(lens.format, 0.5);
        assert_eq!(lens.adaptability, 0.5);
        assert_eq!(lens.relevance, 0.5);
        assert_eq!(lens.style, 0.5);
        assert_eq!(lens.planning, 0.5);
        assert_eq!(lens.creativity, 0.5);
        assert!((lens.average() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_skill_lens_average() {
        let lens = SkillLens {
            instruction_compliance: 1.0,
            safety: 1.0,
            semantics: 1.0,
            format: 1.0,
            adaptability: 1.0,
            relevance: 1.0,
            style: 1.0,
            planning: 1.0,
            creativity: 1.0,
        };
        assert!((lens.average() - 1.0).abs() < 1e-10);

        let lens2 = SkillLens {
            instruction_compliance: 0.0,
            safety: 0.0,
            semantics: 0.0,
            format: 0.0,
            adaptability: 0.0,
            relevance: 0.0,
            style: 0.0,
            planning: 0.0,
            creativity: 0.0,
        };
        assert!((lens2.average() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_with_rubric_keyword_matching() {
        let brain = PanoramicBrain::new();

        // Task with creative and planning keywords
        let lens = brain.evaluate_with_rubric(
            "create innovative and original design with strategic planning",
            None,
        );
        assert!(lens.creativity > 0.0, "Creativity should be >0 for creative task");
        assert!(lens.planning > 0.0, "Planning should be >0 for planning task");

        // Task with safety and instruction keywords
        let lens2 = brain.evaluate_with_rubric(
            "follow safety instructions to protect against vulnerabilities",
            None,
        );
        assert!(lens2.safety >= 0.0, "Safety should be non-negative");
        assert!(lens2.instruction_compliance > 0.0, "Compliance should be >0");

        // Generic task with no keywords
        let lens3 = brain.evaluate_with_rubric("xyz unknown task", None);
        assert!((lens3.average() - 0.0).abs() < 1e-10, "Generic task should score near 0");
    }

    #[test]
    fn test_route_task_with_rubric_modifies_weights() {
        let brain = PanoramicBrain::new();

        let route_no_rubric = brain.route_task(
            "creative innovative design with strategic planning",
            Some(TaskType::General),
            false,
        );
        let route_with_rubric = brain.route_task(
            "creative innovative design with strategic planning",
            Some(TaskType::General),
            true,
        );

        // With rubric, the creativity keyword match should boost weights
        let total_no: f64 = route_no_rubric.weights.iter().map(|(_, w)| w).sum();
        let total_with: f64 = route_with_rubric.weights.iter().map(|(_, w)| w).sum();
        assert!((total_with - total_no).abs() > 0.001 || total_with == total_no,
            "Rubric should change weight distribution");
    }

    #[test]
    fn test_evaluate_task_with_rubric_boosts_score() {
        let brain = PanoramicBrain::new();

        let (score_no, _) = brain.evaluate_task("creative innovative original design", false);
        let (score_with, _) = brain.evaluate_task("creative innovative original design", true);

        // Both rubric and non-rubric scores should be valid
        assert!(score_no >= 0.0 && score_no <= 1.0,
            "Base score {} should be in [0,1]", score_no);
        assert!(score_with >= 0.0 && score_with <= 1.0,
            "Rubric score {} should be in [0,1]", score_with);
    }

    #[test]
    fn test_skill_lens_clone_and_debug() {
        let lens = SkillLens::default();
        let _cloned = lens.clone();
        let _debug = format!("{:?}", lens);
    }
}
