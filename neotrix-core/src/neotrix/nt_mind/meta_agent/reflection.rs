#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::super::{
        ArchiveEntry, EvolutionArchive, FileDiff, HyperAgent, MetaAgent, MetaAgentConfig,
        ModificationTarget, SafetyCheckResult, SelfModificationProposal, StagedEvaluation,
    };
    use crate::neotrix::nt_mind::evolution_types::{ParentSelection, SelectionConfig};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_entry(id: &str, score: f64, generation: u64, timestamp: u64) -> ArchiveEntry {
        ArchiveEntry {
            id: id.to_string(),
            parent_id: None,
            score,
            diffs: Vec::new(),
            generation,
            timestamp,
            lineage: Vec::new(),
            metadata: HashMap::new(),
            diversity_score: 0.0,
        }
    }

    fn make_diff(file_path: &str, content: &str) -> FileDiff {
        FileDiff {
            file_path: PathBuf::from(file_path),
            diff_content: content.to_string(),
            parent_hash: "hash".to_string(),
        }
    }

    #[test]
    fn test_archive_add_and_count() {
        let mut archive = EvolutionArchive::new(SelectionConfig::default());
        assert!(archive.is_empty());
        assert_eq!(archive.len(), 0);

        archive.add(make_entry("e1", 0.5, 0, 100));
        assert_eq!(archive.len(), 1);
        assert!(!archive.is_empty());

        archive.add(make_entry("e2", 0.8, 0, 200));
        assert_eq!(archive.len(), 2);
    }

    #[test]
    fn test_archive_select_best() {
        let mut archive = EvolutionArchive::new(SelectionConfig::default());
        archive.add(make_entry("e1", 0.5, 0, 100));
        archive.add(make_entry("e2", 0.9, 0, 200));
        archive.add(make_entry("e3", 0.3, 0, 300));

        let best = archive.select_by_best().expect("should select best entry");
        assert_eq!(best.id, "e2");
        assert!((best.score - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_archive_select_latest() {
        let mut archive = EvolutionArchive::new(SelectionConfig::default());
        archive.add(make_entry("e1", 0.5, 0, 100));
        archive.add(make_entry("e2", 0.9, 0, 300));
        archive.add(make_entry("e3", 0.3, 0, 200));

        let latest = archive
            .select_by_latest()
            .expect("should select latest entry");
        assert_eq!(latest.id, "e2");
    }

    #[test]
    fn test_archive_select_random() {
        let mut archive = EvolutionArchive::new(SelectionConfig::default());
        archive.add(make_entry("e1", 0.5, 0, 100));
        archive.add(make_entry("e2", 0.9, 0, 200));

        let r = archive.select_by_random();
        assert!(r.is_some());
        assert!(["e1", "e2"].contains(
            &r.expect("random selection should return an entry")
                .id
                .as_str()
        ));
    }

    #[test]
    fn test_archive_select_random_empty() {
        let archive = EvolutionArchive::new(SelectionConfig::default());
        assert!(archive.select_by_random().is_none());
    }

    #[test]
    fn test_archive_prune_low_score() {
        let config = SelectionConfig {
            archive_capacity: 10,
            ..Default::default()
        };
        let mut archive = EvolutionArchive::new(config);
        archive.add(make_entry("e1", 0.9, 0, 100));
        archive.add(make_entry("e2", 0.1, 0, 200));
        archive.add(make_entry("e3", 0.05, 0, 300));
        archive.add(make_entry("e4", 0.8, 0, 400));

        let pruned = archive.prune(0.2);
        assert_eq!(pruned, 2);
        assert_eq!(archive.len(), 2);
        assert!(archive.entries.iter().all(|e| e.score >= 0.2));
    }

    #[test]
    fn test_file_diff_size() {
        let diff = make_diff("src/main.rs", "line1\nline2\nline3\n");
        assert_eq!(diff.diff_size(), 3);

        let empty = make_diff("src/lib.rs", "");
        assert_eq!(empty.diff_size(), 0);

        let single = make_diff("src/lib.rs", "only");
        assert_eq!(single.diff_size(), 1);
    }

    #[test]
    fn test_meta_agent_filter_diff_protected() {
        let config = MetaAgentConfig {
            protected_paths: vec!["domains/".to_string(), "tests/".to_string()],
            ..Default::default()
        };
        let agent = MetaAgent::new(config);

        let safe_diffs = vec![make_diff("src/agent.rs", "change")];
        assert!(agent.filter_diff(&safe_diffs).is_ok());

        let blocked_diffs = vec![make_diff("domains/eval.rs", "change")];
        let result = agent.filter_diff(&blocked_diffs);
        assert!(result.is_err());
        assert!(result
            .expect_err("filter_diff should error on blocked path")
            .contains("blocked"));

        let mixed = vec![
            make_diff("src/agent.rs", "change"),
            make_diff("tests/integration.rs", "change"),
        ];
        let result = agent.filter_diff(&mixed);
        assert!(result.is_err());
    }

    #[test]
    fn test_meta_agent_safety_check() {
        let config = MetaAgentConfig {
            self_referential: false,
            ..Default::default()
        };
        let agent = MetaAgent::new(config);

        let safe = SelfModificationProposal {
            target: ModificationTarget::TaskAgent,
            diffs: vec![make_diff("src/agent.rs", "change")],
            expected_impact: "fix".to_string(),
            safety_check: SafetyCheckResult::Passed,
        };
        let result = agent.safety_check(&safe);
        assert_eq!(result, SafetyCheckResult::Passed);

        let self_ref = SelfModificationProposal {
            target: ModificationTarget::MetaAgent,
            diffs: vec![make_diff("meta_agent.rs", "change")],
            expected_impact: "self-improve".to_string(),
            safety_check: SafetyCheckResult::Passed,
        };
        let result = agent.safety_check(&self_ref);
        assert!(matches!(result, SafetyCheckResult::Failed { .. }));
    }

    #[test]
    fn test_meta_agent_should_continue_budget() {
        let config = MetaAgentConfig {
            budget: 5,
            ..Default::default()
        };
        let agent = MetaAgent::new(config);
        assert!(agent.should_continue());

        let mut agent = MetaAgent::new(MetaAgentConfig {
            budget: 3,
            ..Default::default()
        });
        agent.iteration = 3;
        assert!(!agent.should_continue());

        agent.iteration = 10;
        assert!(!agent.should_continue());
    }

    #[test]
    fn test_meta_agent_should_rollback() {
        let agent = MetaAgent::new(MetaAgentConfig::default());
        assert!(agent.should_rollback(0.5, 0.9));
        assert!(!agent.should_rollback(0.85, 0.9));
        assert!(!agent.should_rollback(1.0, 0.9));
        assert!(!agent.should_rollback(0.81, 0.9));
    }

    #[test]
    fn test_staged_eval_default_config() {
        let eval = StagedEvaluation::default();
        assert_eq!(eval.subset_size, 10);
        assert_eq!(eval.full_size, 100);
        assert!((eval.subset_threshold - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_archive_lineage_tracking() {
        let mut archive = EvolutionArchive::new(SelectionConfig::default());

        let parent = ArchiveEntry {
            id: "p1".to_string(),
            parent_id: None,
            score: 0.8,
            diffs: Vec::new(),
            generation: 0,
            timestamp: 100,
            lineage: Vec::new(),
            metadata: HashMap::new(),
            diversity_score: 0.0,
        };
        archive.add(parent);

        let parent_ref = archive
            .select_by_best()
            .expect("should select best entry for lineage");
        let mut child_lineage = parent_ref.lineage.clone();
        child_lineage.push(parent_ref.id.clone());

        let child = ArchiveEntry {
            id: "c1".to_string(),
            parent_id: Some("p1".to_string()),
            score: 0.9,
            diffs: Vec::new(),
            generation: 1,
            timestamp: 200,
            lineage: child_lineage,
            metadata: HashMap::new(),
            diversity_score: 0.0,
        };
        archive.add(child);

        let grandchild_lineage = {
            let child_ref = archive
                .entries
                .iter()
                .find(|e| e.id == "c1")
                .expect("c1 entry should exist in archive");
            let mut l = child_ref.lineage.clone();
            l.push(child_ref.id.clone());
            l
        };

        let grandchild = ArchiveEntry {
            id: "gc1".to_string(),
            parent_id: Some("c1".to_string()),
            score: 0.95,
            diffs: Vec::new(),
            generation: 2,
            timestamp: 300,
            lineage: grandchild_lineage,
            metadata: HashMap::new(),
            diversity_score: 0.0,
        };
        archive.add(grandchild);

        let gc = archive
            .entries
            .iter()
            .find(|e| e.id == "gc1")
            .expect("gc1 entry should exist in archive");
        assert_eq!(gc.lineage.len(), 2);
        assert_eq!(gc.lineage[0], "p1");
        assert_eq!(gc.lineage[1], "c1");
    }

    #[test]
    fn test_hyper_agent_default() {
        let ha = HyperAgent {
            id: "test-id".to_string(),
            parent_id: None,
            score: Some(0.0),
            diffs_applied: Vec::new(),
            generation: 0,
        };
        assert_eq!(ha.id, "test-id");
        assert!(ha.parent_id.is_none());
        assert_eq!(ha.score, Some(0.0));
    }

    #[test]
    fn test_meta_agent_run_generation_basic() {
        let config = MetaAgentConfig {
            budget: 3,
            ..Default::default()
        };
        let mut agent = MetaAgent::new(config);
        let result = agent.run_generation();
        assert_eq!(result.generation, 1);
        assert!(result.archive_size >= 1);
        assert!(!agent.archive.is_empty());
    }

    #[test]
    fn test_select_parent_empty() {
        let archive = EvolutionArchive::new(SelectionConfig::default());
        assert!(archive.select_parent().is_none());
    }

    #[test]
    fn test_meta_agent_config_defaults() {
        let config = MetaAgentConfig::default();
        assert_eq!(config.budget, 10);
        assert!(config.self_referential);
        assert_eq!(config.protected_paths.len(), 2);
        assert!((config.llm_temperature - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_selection_config_defaults() {
        let config = SelectionConfig::default();
        assert_eq!(config.strategy, ParentSelection::Best);
        assert!((config.temperature - 1.0).abs() < 1e-6);
        assert_eq!(config.min_generations, 1);
        assert_eq!(config.archive_capacity, 100);
    }

    #[test]
    fn test_forward_zero_iterations() {
        let agent = MetaAgent::new(MetaAgentConfig::default());
        let proposals = agent.forward(&[], &[], 0);
        assert!(proposals.is_empty());
    }

    #[test]
    fn test_file_diff_new() {
        let diff = FileDiff::new(
            PathBuf::from("src/main.rs"),
            "+new line".to_string(),
            "abc".to_string(),
        );
        assert_eq!(diff.file_path.to_string_lossy(), "src/main.rs");
        assert_eq!(diff.diff_content, "+new line");
        assert_eq!(diff.parent_hash, "abc");
    }

    #[test]
    fn test_prune_to_capacity() {
        let config = SelectionConfig {
            archive_capacity: 2,
            ..Default::default()
        };
        let mut archive = EvolutionArchive::new(config);
        archive.add(make_entry("e1", 0.9, 0, 100));
        archive.add(make_entry("e2", 0.8, 0, 200));
        archive.add(make_entry("e3", 0.7, 0, 300));

        let pruned = archive.prune(0.0);
        assert_eq!(archive.len(), 2, "should truncate to capacity");
        assert_eq!(pruned, 1);
        assert_eq!(archive.entries[0].id, "e1");
        assert_eq!(archive.entries[1].id, "e2");
    }

    #[test]
    fn test_safety_check_protected_path() {
        let config = MetaAgentConfig {
            protected_paths: vec!["domains/".to_string()],
            ..Default::default()
        };
        let agent = MetaAgent::new(config);

        let proposal = SelfModificationProposal {
            target: ModificationTarget::TaskAgent,
            diffs: vec![make_diff("domains/benchmark.rs", "change")],
            expected_impact: "fix".to_string(),
            safety_check: SafetyCheckResult::Passed,
        };
        let result = agent.safety_check(&proposal);
        assert!(matches!(result, SafetyCheckResult::Failed { .. }));
    }

    #[test]
    fn test_staged_eval_threshold() {
        let agent = MetaAgent::new(MetaAgentConfig::default());
        let entry = make_entry("test", 0.0, 0, 0);
        let score = agent.staged_eval(&entry);
        assert!((score - 0.5).abs() < 1e-6);
    }
}
