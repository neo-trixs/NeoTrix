#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::neotrix::nt_mind::self_iterating::pipeline::BrainStage;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn make_diff(path: &str) -> FileDiff {
        FileDiff {
            file_path: PathBuf::from(path),
            diff_content: "--- a/file\n+++ b/file\n@@ -1 +1 @@\n-old\n+new".to_string(),
            parent_hash: "abc123".to_string(),
        }
    }

    fn make_record(generation: u64, score: Option<f64>) -> HyperAgentRecord {
        HyperAgentRecord {
            id: Uuid::new_v4(),
            parent_id: None,
            diffs_applied: vec![],
            score,
            novelty_score: 0.0,
            latent_snapshot: vec![0.0; 32],
            generation,
            proposal: SelfModificationProposal {
                target: ModificationTarget::CapabilityExtension,
                diffs: vec![],
                expected_impact: "test".to_string(),
                safety_check: SafetyCheckResult::Passed,
            },
        }
    }

    fn make_record_with_latent(
        generation: u64,
        score: Option<f64>,
        latent: Vec<f64>,
    ) -> HyperAgentRecord {
        let mut r = make_record(generation, score);
        r.latent_snapshot = latent;
        r
    }

    #[test]
    fn test_file_diff() {
        let diff = make_diff("/tmp/test.rs");
        assert_eq!(diff.file_path, PathBuf::from("/tmp/test.rs"));
        assert!(diff.diff_content.contains("+new"));
        assert_eq!(diff.parent_hash, "abc123");
    }

    #[test]
    fn test_self_modification_proposal() {
        let proposal = SelfModificationProposal {
            target: ModificationTarget::MetaAgent,
            diffs: vec![make_diff("meta.rs")],
            expected_impact: "improve mutation rate".to_string(),
            safety_check: SafetyCheckResult::Passed,
        };
        assert_eq!(proposal.target, ModificationTarget::MetaAgent);
        assert_eq!(proposal.diffs.len(), 1);
    }

    #[test]
    fn test_parent_selection_best() {
        let config = SelectionConfig::default();
        let mut archive = HyperAgentArchive::new(config);
        archive.add_record(make_record(0, Some(0.3)));
        archive.add_record(make_record(0, Some(0.9)));
        archive.add_record(make_record(0, Some(0.6)));

        let parent = archive.select_parent().expect("should select a parent");
        let best = parent.score.expect("score should be present");
        assert!(
            (best - 0.9).abs() < 1e-6,
            "Best strategy should pick highest score"
        );
    }

    #[test]
    fn test_parent_selection_latest() {
        let config = SelectionConfig {
            strategy: ParentSelection::Latest,
            ..Default::default()
        };
        let mut archive = HyperAgentArchive::new(config);
        archive.add_record(make_record(0, Some(0.3)));
        archive.add_record(make_record(1, Some(0.9)));
        archive.add_record(make_record(2, Some(0.6)));

        let parent = archive.select_parent().expect("should select a parent");
        assert_eq!(
            parent.generation, 2,
            "Latest strategy should pick last added"
        );
    }

    #[test]
    fn test_archive_add_record() {
        let config = SelectionConfig::default();
        let mut archive = HyperAgentArchive::new(config);
        assert_eq!(archive.records.len(), 0);

        archive.add_record(make_record(0, None));
        assert_eq!(archive.records.len(), 1);

        archive.add_record(make_record(1, Some(0.5)));
        assert_eq!(archive.records.len(), 2);
    }

    #[test]
    fn test_meta_agent_forward() {
        let config = SelectionConfig::default();
        let archive = HyperAgentArchive::new(config);
        let agent = HyperMetaAgent::new(100, false);

        let proposal = agent.forward(&archive);
        assert_eq!(proposal.target, ModificationTarget::CapabilityExtension);
        assert_eq!(proposal.safety_check, SafetyCheckResult::Passed);
    }

    #[test]
    fn test_filter_protected_paths() {
        let mut agent = HyperMetaAgent::new(100, true);
        agent.protected_paths.push("secret/".to_string());

        let diffs = vec![
            make_diff("src/main.rs"),
            make_diff("secret/credentials.yml"),
            make_diff("src/lib.rs"),
        ];

        let filtered = agent.filter_protected_paths(&diffs);
        assert_eq!(filtered.len(), 2);
        assert!(!filtered
            .iter()
            .any(|d| d.file_path.to_string_lossy().contains("secret/")));
    }

    #[test]
    fn test_latest_generation_empty() {
        let config = SelectionConfig::default();
        let archive = HyperAgentArchive::new(config);
        assert_eq!(archive.latest_generation(), 0);
    }

    #[test]
    fn test_latest_generation_with_records() {
        let config = SelectionConfig::default();
        let mut archive = HyperAgentArchive::new(config);
        archive.add_record(make_record(3, None));
        archive.add_record(make_record(7, None));
        assert_eq!(archive.latest_generation(), 7);
    }

    #[test]
    fn test_best_score_empty() {
        let config = SelectionConfig::default();
        let archive = HyperAgentArchive::new(config);
        assert!(archive.best_score().is_none());
    }

    #[test]
    fn test_best_score_with_records() {
        let config = SelectionConfig::default();
        let mut archive = HyperAgentArchive::new(config);
        archive.add_record(make_record(0, Some(0.2)));
        archive.add_record(make_record(0, Some(0.8)));
        let best = archive.best_score().expect("should have a best score");
        assert!((best - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_meta_evolve_stage_continue_on_passed() {
        let agent = HyperMetaAgent::new(50, false);
        let config = SelectionConfig::default();
        let archive = HyperAgentArchive::new(config);
        let stage = MetaEvolveStage::new(agent, archive);

        let mut brain = crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain::new();
        let decision = stage.process(&mut brain).expect("process should succeed");
        assert!(
            matches!(
                decision,
                crate::neotrix::nt_mind::self_iterating::StageDecision::Continue
            ),
            "expected Continue, got {:?}",
            decision
        );
    }

    #[test]
    fn test_filter_protected_paths_no_match() {
        let mut agent = HyperMetaAgent::new(100, true);
        agent.protected_paths.push("config/".to_string());

        let diffs = vec![make_diff("src/main.rs"), make_diff("src/lib.rs")];
        let filtered = agent.filter_protected_paths(&diffs);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_select_parent_empty_archive() {
        let config = SelectionConfig::default();
        let archive = HyperAgentArchive::new(config);
        assert!(archive.select_parent().is_none());
    }

    #[test]
    fn test_cosine_distance_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let d = cosine_distance(&a, &a);
        assert!(
            (d - 0.0).abs() < 1e-6,
            "identical vectors should have distance 0"
        );
    }

    #[test]
    fn test_cosine_distance_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let d = cosine_distance(&a, &b);
        assert!(
            (d - 1.0).abs() < 1e-6,
            "opposite vectors should have distance 1"
        );
    }

    #[test]
    fn test_cosine_distance_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let d = cosine_distance(&a, &b);
        assert!(
            (d - 0.5).abs() < 1e-6,
            "orthogonal vectors should have distance 0.5"
        );
    }

    #[test]
    fn test_compute_novelty_empty_archive() {
        let config = SelectionConfig::default();
        let archive = HyperAgentArchive::new(config);
        assert!((archive.compute_novelty(&[1.0; 32]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_novelty_with_similar_record() {
        let config = SelectionConfig::default();
        let mut archive = HyperAgentArchive::new(config);
        archive.add_record(make_record_with_latent(0, Some(0.5), vec![1.0; 32]));
        let novelty = archive.compute_novelty(&vec![1.0; 32]);
        assert!(
            novelty < 0.1,
            "similar latent should have low novelty: {}",
            novelty
        );
    }

    #[test]
    fn test_diversity_weighted_selection() {
        let config = SelectionConfig {
            strategy: ParentSelection::DiversityWeighted,
            novelty_weight: 0.5,
            ..Default::default()
        };
        let mut archive = HyperAgentArchive::new(config);

        let mut r1 = make_record(0, Some(0.2));
        r1.novelty_score = 0.9;
        r1.latent_snapshot = vec![1.0; 32];
        archive.add_record(r1);

        let mut r2 = make_record(0, Some(0.9));
        r2.novelty_score = 0.1;
        r2.latent_snapshot = vec![0.0; 32];
        archive.add_record(r2);

        let parent = archive.select_parent().expect("should select a parent");
        let best = parent.score.expect("score should be present");
        assert!(
            (best - 0.9).abs() < 1e-6,
            "DiversityWeighted should still prefer high score"
        );
    }

    #[test]
    fn test_capacity_eviction() {
        let config = SelectionConfig {
            strategy: ParentSelection::Best,
            archive_capacity: 3,
            ..Default::default()
        };
        let mut archive = HyperAgentArchive::new(config);
        archive.add_record(make_record(0, Some(0.1)));
        archive.add_record(make_record(1, Some(0.9)));
        archive.add_record(make_record(2, Some(0.5)));
        archive.add_record(make_record(3, Some(0.3)));
        assert!(archive.records.len() <= 3);
        assert!(archive.records.iter().all(|r| r.score.unwrap_or(0.0) > 0.1));
    }

    #[test]
    fn test_evaluate_proposal_lower_fe_is_better() {
        let agent = HyperMetaAgent::new(100, false);
        let proposal = SelfModificationProposal {
            target: ModificationTarget::CapabilityExtension,
            diffs: vec![],
            expected_impact: "test".to_string(),
            safety_check: SafetyCheckResult::Passed,
        };
        let high_fe_score = agent.evaluate_proposal(&proposal, 5.0);
        let low_fe_score = agent.evaluate_proposal(&proposal, 0.1);
        assert!(
            low_fe_score > high_fe_score,
            "lower FE should give higher score"
        );
    }

    #[test]
    fn test_evaluate_proposal_target_weights() {
        let agent = HyperMetaAgent::new(100, false);
        let fe = 1.0;
        let base = agent.evaluate_proposal(
            &SelfModificationProposal {
                target: ModificationTarget::CapabilityExtension,
                diffs: vec![],
                expected_impact: "".into(),
                safety_check: SafetyCheckResult::Passed,
            },
            fe,
        );
        let meta = agent.evaluate_proposal(
            &SelfModificationProposal {
                target: ModificationTarget::MetaAgent,
                diffs: vec![],
                expected_impact: "".into(),
                safety_check: SafetyCheckResult::Passed,
            },
            fe,
        );
        let improve = agent.evaluate_proposal(
            &SelfModificationProposal {
                target: ModificationTarget::ImprovementMechanism,
                diffs: vec![],
                expected_impact: "".into(),
                safety_check: SafetyCheckResult::Passed,
            },
            fe,
        );
        assert!(
            meta > base,
            "MetaAgent should score higher than CapabilityExtension"
        );
        assert!(
            improve > meta,
            "ImprovementMechanism should score highest (exploration bonus)"
        );
    }

    #[test]
    fn test_forward_with_nonempty_archive() {
        let config = SelectionConfig::default();
        let mut archive = HyperAgentArchive::new(config);
        archive.add_record(make_record(0, Some(0.5)));
        let agent = HyperMetaAgent::new(100, false);
        let proposal = agent.forward(&archive);
        assert!(proposal.expected_impact.contains("parent_score"));
        assert!(proposal.expected_impact.contains("generation=1"));
    }

    #[test]
    fn test_add_record_tracking_latent() {
        let config = SelectionConfig::default();
        let mut archive = HyperAgentArchive::new(config);
        let latent = vec![0.5; 32];
        let mut record = make_record(0, Some(0.8));
        record.latent_snapshot = latent.clone();
        archive.add_record(record);
        let novelty = archive.compute_novelty(&latent);
        assert!(novelty < 0.01, "same latent should give near-zero novelty");
    }

    // ==========================================================================
    // DGM-Hyperagent SEAL tests
    // ==========================================================================

    fn make_archive_with_records(n: usize) -> HyperAgentArchive {
        let mut archive = HyperAgentArchive::new(SelectionConfig::default());
        for i in 0..n {
            let score = 0.3 + (i as f64) * 0.1;
            let latent: Vec<f64> = (0..32).map(|j| ((i + j) as f64).sin()).collect();
            let mut record = make_record_with_latent(i as u64, Some(score), latent);
            record.novelty_score = 0.2;
            archive.add_record(record);
        }
        archive
    }

    #[test]
    fn test_latent_edit_new() {
        let delta = vec![0.1, -0.2, 0.3];
        let edit = LatentEdit::new(delta.clone());
        assert_eq!(edit.delta, delta);
        assert!(edit.generative_entropy > 0.0);
        assert!(edit.hypervector.is_none());
    }

    #[test]
    fn test_latent_edit_apply_to() {
        let base = vec![0.5, 0.5, 0.5];
        let edit = LatentEdit::new(vec![0.1, -0.2, 0.0]);
        let result = edit.apply_to(&base);
        assert!((result[0] - 0.6).abs() < 1e-10);
        assert!((result[1] - 0.3).abs() < 1e-10);
        assert!((result[2] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_latent_edit_magnitude() {
        let edit = LatentEdit::new(vec![3.0, 4.0]);
        assert!((edit.magnitude() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_dgm_agent_generate_edit_with_parents() {
        let archive = make_archive_with_records(5);
        let dgm = DGMMetaAgent::new(64, 3, 0.1);
        let edit = dgm.generate_edit(&archive);
        assert_eq!(edit.delta.len(), 32);
        assert!(edit.generative_entropy > 0.0);
        assert!(edit.hypervector.is_some());
    }

    #[test]
    fn test_dgm_agent_generate_edit_empty_archive() {
        let archive = HyperAgentArchive::new(SelectionConfig::default());
        let dgm = DGMMetaAgent::new(64, 3, 0.1);
        let edit = dgm.generate_edit(&archive);
        assert!(!edit.delta.is_empty());
        assert_eq!(edit.delta.len(), 64);
    }

    #[test]
    fn test_dgm_proposal_from_edit() {
        let archive = make_archive_with_records(3);
        let dgm = DGMMetaAgent::new(64, 3, 0.1);
        let edit = dgm.generate_edit(&archive);
        let proposal = dgm.proposal_from_edit(&edit, &archive);
        assert!(proposal.expected_impact.contains("dgm_h"));
        assert_eq!(proposal.safety_check, SafetyCheckResult::Passed);
    }

    #[test]
    fn test_generative_replay_no_records() {
        let archive = HyperAgentArchive::new(SelectionConfig::default());
        let replay = GenerativeReplay::default();
        let samples = replay.replay(&archive);
        assert!(samples.is_empty());
    }

    #[test]
    fn test_generative_replay_with_records() {
        let archive = make_archive_with_records(5);
        let replay = GenerativeReplay::new(2, 0.3);
        let samples = replay.replay(&archive);
        assert!(!samples.is_empty());
        for s in &samples {
            assert_eq!(s.len(), 32);
        }
    }

    #[test]
    fn test_generative_replay_disabled() {
        let archive = make_archive_with_records(5);
        let mut replay = GenerativeReplay::default();
        replay.enabled = false;
        let samples = replay.replay(&archive);
        assert!(samples.is_empty());
    }

    #[test]
    fn test_self_ref_check_stable() {
        let mut archive = make_archive_with_records(5);
        let latent = vec![0.5; 32];
        let mut record = make_record_with_latent(0, Some(0.9), latent.clone());
        record.novelty_score = 0.0;
        archive.add_record(record);

        let check = SelfReferentialCheck::default();
        let edit = LatentEdit::new(vec![0.01; 32]);
        let result = check.check(&edit, &archive, &latent);
        assert_eq!(result, SelfRefCheckResult::Stable);
    }

    #[test]
    fn test_self_ref_check_destabilizing() {
        let config = SelectionConfig::default();
        let mut archive = HyperAgentArchive::new(config);
        let base_latent = vec![0.5; 32];
        for i in 0..5 {
            let mut latent: Vec<f64> = base_latent.clone();
            latent[i] += 0.01;
            let mut record = make_record_with_latent(i as u64, Some(0.7 + i as f64 * 0.05), latent);
            record.novelty_score = 0.0;
            archive.add_record(record);
        }
        let mut record = make_record_with_latent(5, Some(0.9), base_latent.clone());
        record.novelty_score = 0.0;
        archive.add_record(record);

        let check = SelfReferentialCheck::new(1.5, 1.5, 0.7);
        let mut delta = vec![0.0; 32];
        delta[0] = -10.0;
        delta[1] = 10.0;
        let edit = LatentEdit::new(delta);
        let result = check.check(&edit, &archive, &base_latent);
        assert!(matches!(result, SelfRefCheckResult::Destabilizing { .. }));
    }

    #[test]
    fn test_dgm_evolve_stage() {
        let archive = make_archive_with_records(5);
        let dgm = DGMMetaAgent::new(64, 3, 0.1);
        let replay = GenerativeReplay::default();
        let self_check = SelfReferentialCheck::default();
        let stage = DGMMetaEvolveStage::new(dgm, archive, replay, self_check);

        let (_edit, proposal) = stage.evolve();
        assert!(proposal.expected_impact.contains("dgm_h"));
    }

    #[test]
    fn test_self_ref_check_empty_archive() {
        let archive = HyperAgentArchive::new(SelectionConfig::default());
        let check = SelfReferentialCheck::default();
        let edit = LatentEdit::new(vec![1.0; 32]);
        let result = check.check(&edit, &archive, &[0.0; 32]);
        assert_eq!(result, SelfRefCheckResult::Stable);
    }

    #[test]
    fn test_dgm_select_top_k() {
        let archive = make_archive_with_records(10);
        let dgm = DGMMetaAgent::new(64, 3, 0.1);
        let top = dgm.select_top_k(&archive);
        assert_eq!(top.len(), 3);
        for i in 1..top.len() {
            let prev = top[i - 1].score.unwrap_or(0.0);
            let curr = top[i].score.unwrap_or(0.0);
            assert!(prev >= curr, "top-k should be sorted descending by score");
        }
    }
}
