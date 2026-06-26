use std::collections::HashMap;

use crate::core::nt_core_gwt::curiosity_exploration::{CuriosityExploration, KnowledgeGap};
use crate::core::nt_core_reasoning::counterfactual_simulator::{
    CounterfactualScenario, CounterfactualSimulator,
};
use crate::core::nt_core_reasoning::dead_end_detector::{DeadEndDetector, DeadEndReport};
use crate::core::nt_core_reasoning::mcts_reasoner::MctsReasoner;
use crate::core::nt_core_reasoning::process_reward_model::ProcessRewardModel;

#[derive(Debug, Clone)]
pub struct ReasoningKeBridge {
    pub bridge_id: String,
    pub entries_created: u64,
    pub last_sync: u64,
    stored_entries: Vec<StoredEntry>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct StoredEntry {
    #[allow(dead_code)]
    id: String,
    category: EntryCategory,
    #[allow(dead_code)]
    content: String,
    #[allow(dead_code)]
    tags: Vec<String>,
    #[allow(dead_code)]
    context: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryCategory {
    Mcts,
    Prm,
    DeadEnd,
    Counterfactual,
    Curiosity,
}

impl EntryCategory {
    fn tag(&self) -> &'static str {
        match self {
            EntryCategory::Mcts => "mcts",
            EntryCategory::Prm => "prm",
            EntryCategory::DeadEnd => "dead_end",
            EntryCategory::Counterfactual => "counterfactual",
            EntryCategory::Curiosity => "curiosity",
        }
    }
}

impl ReasoningKeBridge {
    pub fn new() -> Self {
        Self {
            bridge_id: format!("reasoning-ke-bridge-{}", crate::core::unix_now_ms()),
            entries_created: 0,
            last_sync: 0,
            stored_entries: Vec::new(),
        }
    }

    pub fn new_with_id(bridge_id: impl Into<String>) -> Self {
        Self {
            bridge_id: bridge_id.into(),
            entries_created: 0,
            last_sync: 0,
            stored_entries: Vec::new(),
        }
    }

    pub fn store_mcts_path(&mut self, mcts: &MctsReasoner, context: &str) -> String {
        let stats = mcts.stats();
        let entry_id = format!("mcts-{}-{}", self.bridge_id, self.entries_created);

        let content = format!(
            "MCTS Reasoning Path\n\
             Context: {context}\n\
             Nodes explored: {nodes}\n\
             Root visits: {visits}\n\
             Best value: {value:.4}\n\
             PRM mean: {prm_mean:.4}\n\
             PRM std: {prm_std:.4}\n\
             Exploration efficiency: {eff:.4}",
            context = context,
            nodes = stats.total_nodes,
            visits = stats.root_visits,
            value = stats.best_value,
            prm_mean = stats.prm_mean,
            prm_std = stats.prm_std,
            eff = stats.exploration_efficiency,
        );

        let tags = vec![
            "reasoning".to_string(),
            EntryCategory::Mcts.tag().to_string(),
            "evaluation".to_string(),
        ];

        self.stored_entries.push(StoredEntry {
            id: entry_id.clone(),
            category: EntryCategory::Mcts,
            content,
            tags,
            context: context.to_string(),
        });
        self.entries_created += 1;
        entry_id
    }

    pub fn store_prm_evaluation(&mut self, prm: &ProcessRewardModel, context: &str) -> String {
        let stats = prm.stats();
        let entry_id = format!("prm-{}-{}", self.bridge_id, self.entries_created);

        let step_types: Vec<String> = stats
            .step_type_distribution
            .iter()
            .map(|(t, c)| format!("  {t}: {c}"))
            .collect();

        let content = format!(
            "PRM Evaluation\n\
             Context: {context}\n\
             Total steps: {steps}\n\
             Trajectories: {traj}\n\
             Avg step reward: {step_r:.4}\n\
             Avg chunk reward: {chunk_r:.4}\n\
             Avg trajectory reward: {traj_r:.4}\n\
             Reward variance: {var:.6}\n\
             Step type distribution:\n{types}",
            context = context,
            steps = stats.total_steps,
            traj = stats.trajectories_completed,
            step_r = stats.avg_step_reward,
            chunk_r = stats.avg_chunk_reward,
            traj_r = stats.avg_trajectory_reward,
            var = stats.reward_variance,
            types = step_types.join("\n"),
        );

        let tags = vec![
            "reasoning".to_string(),
            EntryCategory::Prm.tag().to_string(),
            "evaluation".to_string(),
        ];

        self.stored_entries.push(StoredEntry {
            id: entry_id.clone(),
            category: EntryCategory::Prm,
            content,
            tags,
            context: context.to_string(),
        });
        self.entries_created += 1;
        entry_id
    }

    pub fn store_dead_end(&mut self, report: &DeadEndReport, context: &str) -> String {
        let entry_id = format!("dead-end-{}-{}", self.bridge_id, self.entries_created);

        let content = format!(
            "Dead-End Detection Report\n\
             Context: {context}\n\
             Type: {detected_type:?}\n\
             Detection step: {step}\n\
             Loop length: {loop_len:?}\n\
             VSA similarity: {vsa_sim:.4}\n\
             Confidence delta: {conf_delta:.4}\n\
             Recovery strategy: {recovery:?}\n\
             Evidence:\n{evidence}",
            context = context,
            detected_type = report.detected_type,
            step = report.detection_step,
            loop_len = report.loop_length,
            vsa_sim = report.vsa_similarity,
            conf_delta = report.confidence_delta,
            recovery = report.recovery,
            evidence = report
                .evidence
                .iter()
                .map(|e| format!("  - {e}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        let tags = vec![
            "reasoning".to_string(),
            EntryCategory::DeadEnd.tag().to_string(),
            "failure".to_string(),
        ];

        self.stored_entries.push(StoredEntry {
            id: entry_id.clone(),
            category: EntryCategory::DeadEnd,
            content,
            tags,
            context: context.to_string(),
        });
        self.entries_created += 1;
        entry_id
    }

    pub fn store_counterfactual(
        &mut self,
        scenario: &CounterfactualScenario,
        context: &str,
    ) -> String {
        let entry_id = format!("counterfactual-{}-{}", self.bridge_id, self.entries_created);

        let outcome_desc = scenario
            .simulated_outcome
            .as_ref()
            .map(|o| format!("{o:?}"))
            .unwrap_or_else(|| "None".to_string());

        let content = format!(
            "Counterfactual Scenario\n\
             Context: {context}\n\
             Scenario ID: {id}\n\
             Type: {cf_type:?}\n\
             Divergence: {divergence:.4}\n\
             Confidence: {confidence:.4}\n\
             Valid: {is_valid}\n\
             Simulated outcome: {outcome}",
            context = context,
            id = scenario.id,
            cf_type = scenario.cf_type,
            divergence = scenario.divergence,
            confidence = scenario.confidence,
            is_valid = scenario.is_valid,
            outcome = outcome_desc,
        );

        let tags = vec![
            "reasoning".to_string(),
            EntryCategory::Counterfactual.tag().to_string(),
            "simulation".to_string(),
        ];

        self.stored_entries.push(StoredEntry {
            id: entry_id.clone(),
            category: EntryCategory::Counterfactual,
            content,
            tags,
            context: context.to_string(),
        });
        self.entries_created += 1;
        entry_id
    }

    pub fn store_curiosity_gap(&mut self, gap: &KnowledgeGap, context: &str) -> String {
        let entry_id = format!("curiosity-gap-{}-{}", self.bridge_id, self.entries_created);

        let content = format!(
            "Curiosity Knowledge Gap\n\
             Context: {context}\n\
             Gap ID: {id}\n\
             Domain: {domain}\n\
             Gap type: {gap_type:?}\n\
             Predicted info gain: {info_gain:.4}\n\
             Curiosity signal: {curiosity:.4}\n\
             Open: {is_open}",
            context = context,
            id = gap.id,
            domain = gap.domain,
            gap_type = gap.gap_type,
            info_gain = gap.predicted_info_gain,
            curiosity = gap.curiosity_signal,
            is_open = gap.is_open,
        );

        let tags = vec![
            "reasoning".to_string(),
            EntryCategory::Curiosity.tag().to_string(),
            "exploration".to_string(),
        ];

        self.stored_entries.push(StoredEntry {
            id: entry_id.clone(),
            category: EntryCategory::Curiosity,
            content,
            tags,
            context: context.to_string(),
        });
        self.entries_created += 1;
        entry_id
    }

    pub fn sync_all(
        &mut self,
        mcts: Option<&MctsReasoner>,
        prm: Option<&ProcessRewardModel>,
        dead_end: Option<&DeadEndDetector>,
        counterfactual: Option<&CounterfactualSimulator>,
        curiosity: Option<&CuriosityExploration>,
        context: &str,
    ) -> HashMap<String, String> {
        let mut result = HashMap::new();

        if let Some(m) = mcts {
            result.insert("mcts".to_string(), self.store_mcts_path(m, context));
        }
        if let Some(p) = prm {
            result.insert("prm".to_string(), self.store_prm_evaluation(p, context));
        }
        if let Some(d) = dead_end {
            let report = DeadEndReport {
                detected_type: crate::core::nt_core_reasoning::dead_end_detector::DeadEndType::Loop,
                detection_step: d.stats().total_checks,
                loop_length: None,
                vsa_similarity: 0.0,
                confidence_delta: 0.0,
                recovery:
                    crate::core::nt_core_reasoning::dead_end_detector::RecoveryStrategy::Backtrack(0),
                evidence: vec!["auto-collected from sync_all".to_string()],
                timestamp: crate::core::unix_now_ms(),
            };
            result.insert(
                "dead_end".to_string(),
                self.store_dead_end(&report, context),
            );
        }
        if let Some(c) = counterfactual {
            if let Some(scenario) = c.scenarios.first() {
                result.insert(
                    "counterfactual".to_string(),
                    self.store_counterfactual(scenario, context),
                );
            }
        }
        if let Some(c) = curiosity {
            if let Some(gap) = c.knowledge_gaps.first() {
                result.insert(
                    "curiosity".to_string(),
                    self.store_curiosity_gap(gap, context),
                );
            }
        }

        self.last_sync = crate::core::unix_now_ms();
        result
    }

    #[allow(private_interfaces)]
    pub fn stored_entries(&self) -> &[StoredEntry] {
        &self.stored_entries
    }

    pub fn stats(&self) -> BridgeStats {
        let mcts_stored = self
            .stored_entries
            .iter()
            .filter(|e| e.category == EntryCategory::Mcts)
            .count() as u64;
        let prm_stored = self
            .stored_entries
            .iter()
            .filter(|e| e.category == EntryCategory::Prm)
            .count() as u64;
        let dead_end_stored = self
            .stored_entries
            .iter()
            .filter(|e| e.category == EntryCategory::DeadEnd)
            .count() as u64;
        let counterfactual_stored = self
            .stored_entries
            .iter()
            .filter(|e| e.category == EntryCategory::Counterfactual)
            .count() as u64;
        let curiosity_stored = self
            .stored_entries
            .iter()
            .filter(|e| e.category == EntryCategory::Curiosity)
            .count() as u64;

        BridgeStats {
            entries_created: self.entries_created,
            mcts_stored,
            prm_stored,
            dead_end_stored,
            counterfactual_stored,
            curiosity_stored,
        }
    }
}

impl Default for ReasoningKeBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BridgeStats {
    pub entries_created: u64,
    pub mcts_stored: u64,
    pub prm_stored: u64,
    pub dead_end_stored: u64,
    pub counterfactual_stored: u64,
    pub curiosity_stored: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mcts_reasoner() -> MctsReasoner {
        use crate::core::nt_core_reasoning::mcts_reasoner::MctsConfig;
        MctsReasoner::new(MctsConfig::default())
    }

    fn make_prm() -> ProcessRewardModel {
        use crate::core::nt_core_reasoning::process_reward_model::PrmConfig;
        ProcessRewardModel::new(PrmConfig::default())
    }

    fn make_dead_end_detector() -> DeadEndDetector {
        use crate::core::nt_core_reasoning::dead_end_detector::DeadEndConfig;
        DeadEndDetector::new(DeadEndConfig::default())
    }

    fn make_counterfactual_simulator() -> CounterfactualSimulator {
        use crate::core::nt_core_reasoning::counterfactual_simulator::CounterfactualConfig;
        CounterfactualSimulator::new(CounterfactualConfig::default())
    }

    fn make_curiosity_exploration() -> CuriosityExploration {
        use crate::core::nt_core_gwt::curiosity_exploration::CuriosityConfig;
        CuriosityExploration::new(CuriosityConfig::default())
    }

    #[test]
    fn test_store_mcts_path() {
        let mut bridge = ReasoningKeBridge::new();
        let mcts = make_mcts_reasoner();
        let id = bridge.store_mcts_path(&mcts, "test-context");
        assert!(id.starts_with("mcts-"));
        assert_eq!(bridge.entries_created, 1);
        assert_eq!(bridge.stored_entries.len(), 1);
        assert!(bridge.stored_entries[0].content.contains("MCTS"));
    }

    #[test]
    fn test_store_prm_evaluation() {
        let mut bridge = ReasoningKeBridge::new();
        let prm = make_prm();
        let id = bridge.store_prm_evaluation(&prm, "test-context");
        assert!(id.starts_with("prm-"));
        assert_eq!(bridge.entries_created, 1);
        assert!(bridge.stored_entries[0].content.contains("PRM"));
    }

    #[test]
    fn test_store_dead_end() {
        let mut bridge = ReasoningKeBridge::new();
        let report = DeadEndReport {
            detected_type: crate::core::nt_core_reasoning::dead_end_detector::DeadEndType::Loop,
            detection_step: 5,
            loop_length: Some(3),
            vsa_similarity: 0.92,
            confidence_delta: -0.15,
            recovery:
                crate::core::nt_core_reasoning::dead_end_detector::RecoveryStrategy::Backtrack(1),
            evidence: vec!["repeated state A->B->A".to_string()],
            timestamp: 1000,
        };
        let id = bridge.store_dead_end(&report, "test-context");
        assert!(id.starts_with("dead-end-"));
        assert!(bridge.stored_entries[0].content.contains("Dead-End"));
        assert!(bridge.stored_entries[0].content.contains("Loop"));
    }

    #[test]
    fn test_store_counterfactual() {
        let mut bridge = ReasoningKeBridge::new();
        let scenario = CounterfactualScenario {
            id: 1,
            cf_type: crate::core::nt_core_reasoning::counterfactual_simulator::CounterfactualType::InputPerturbation,
            factual_state: vec![0u8; 64],
            counterfactual_state: vec![1u8; 64],
            perturbation: vec![0u8; 64],
            divergence: 0.75,
            simulated_outcome: None,
            confidence: 0.85,
            is_valid: true,
        };
        let id = bridge.store_counterfactual(&scenario, "test-context");
        assert!(id.starts_with("counterfactual-"));
        assert!(bridge.stored_entries[0].content.contains("Counterfactual"));
        assert!(bridge.stored_entries[0]
            .content
            .contains("InputPerturbation"));
    }

    #[test]
    fn test_store_curiosity_gap() {
        let mut bridge = ReasoningKeBridge::new();
        let gap = KnowledgeGap {
            id: 1,
            domain: "mathematics".to_string(),
            gap_type: crate::core::nt_core_gwt::curiosity_exploration::GapType::KnowledgeGap,
            predicted_info_gain: 0.65,
            curiosity_signal: 0.82,
            is_open: true,
        };
        let id = bridge.store_curiosity_gap(&gap, "test-context");
        assert!(id.starts_with("curiosity-gap-"));
        assert!(bridge.stored_entries[0].content.contains("Curiosity"));
        assert!(bridge.stored_entries[0].content.contains("mathematics"));
    }

    #[test]
    fn test_multiple_entries_increment_count() {
        let mut bridge = ReasoningKeBridge::new();
        let mcts = make_mcts_reasoner();
        let prm = make_prm();

        bridge.store_mcts_path(&mcts, "ctx-1");
        bridge.store_prm_evaluation(&prm, "ctx-2");
        bridge.store_mcts_path(&mcts, "ctx-3");

        assert_eq!(bridge.entries_created, 3);
        assert_eq!(bridge.stored_entries.len(), 3);
        let stats = bridge.stats();
        assert_eq!(stats.mcts_stored, 2);
        assert_eq!(stats.prm_stored, 1);
    }

    #[test]
    fn test_sync_all_empty() {
        let mut bridge = ReasoningKeBridge::new();
        let result = bridge.sync_all(None, None, None, None, None, "empty");
        assert!(result.is_empty());
        assert_eq!(bridge.entries_created, 0);
    }

    #[test]
    fn test_sync_all_partial() {
        let mut bridge = ReasoningKeBridge::new();
        let mcts = make_mcts_reasoner();
        let curiosity = make_curiosity_exploration();

        let result = bridge.sync_all(Some(&mcts), None, None, None, Some(&curiosity), "partial");

        assert!(result.contains_key("mcts"));
        assert!(!result.contains_key("prm"));
        assert!(!result.contains_key("dead_end"));
        assert!(!result.contains_key("counterfactual"));
        assert!(result.contains_key("curiosity"));
        assert_eq!(bridge.entries_created, 2);
    }

    #[test]
    fn test_bridge_stats() {
        let mut bridge = ReasoningKeBridge::new_with_id("test-bridge");
        let mcts = make_mcts_reasoner();
        let prm = make_prm();
        let gap = KnowledgeGap {
            id: 1,
            domain: "physics".to_string(),
            gap_type: crate::core::nt_core_gwt::curiosity_exploration::GapType::KnowledgeGap,
            predicted_info_gain: 0.5,
            curiosity_signal: 0.6,
            is_open: true,
        };

        bridge.store_mcts_path(&mcts, "c1");
        bridge.store_mcts_path(&mcts, "c2");
        bridge.store_prm_evaluation(&prm, "c3");
        bridge.store_curiosity_gap(&gap, "c4");

        let stats = bridge.stats();
        assert_eq!(stats.entries_created, 4);
        assert_eq!(stats.mcts_stored, 2);
        assert_eq!(stats.prm_stored, 1);
        assert_eq!(stats.dead_end_stored, 0);
        assert_eq!(stats.counterfactual_stored, 0);
        assert_eq!(stats.curiosity_stored, 1);
    }

    #[test]
    fn test_bridge_id_unique() {
        let b1 = ReasoningKeBridge::new();
        let b2 = ReasoningKeBridge::new();
        assert_ne!(b1.bridge_id, b2.bridge_id);
    }

    #[test]
    fn test_stored_entries_content_includes_tags() {
        let mut bridge = ReasoningKeBridge::new();
        let mcts = make_mcts_reasoner();
        bridge.store_mcts_path(&mcts, "tag-test");
        let entry = &bridge.stored_entries[0];
        assert!(entry.tags.contains(&"reasoning".to_string()));
        assert!(entry.tags.contains(&"mcts".to_string()));
    }
}
