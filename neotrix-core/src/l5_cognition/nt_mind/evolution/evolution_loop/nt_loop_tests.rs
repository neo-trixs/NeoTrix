//! 自进化循环 — 单测 (`nt_loop_tests`).
//!
//! 原 `evolution_loop.rs` 内 `#[cfg(test)] mod tests` 整体搬移;
//! `use super::*` 改为显式 `use` (跨目录模块后 `super` 不再直达类型)。
//! 从 `evolution_loop.rs` 纯搬移, 行为零变更。

#[cfg(test)]
mod tests {
    use crate::l5_cognition::l1_facade::ProjectSnapshot;
    use crate::l5_cognition::nt_mind::evolution::evolution_daemon::IssueType;
    use crate::l5_cognition::nt_mind::evolution::evolution_loop::nt_loop_evaluate::count_actual_unsafe;
    use crate::l5_cognition::nt_mind::evolution::evolution_loop::{
        Auditor, EvolutionLoop, EvolutionReport, MetaHarnessOptimizer, STAGNATION_LIMIT,
        _AuditorRole,
    };
    use crate::l5_cognition::nt_mind::evolution::harness_optimizer::{_HarnessCandidate, _HarnessTarget};
    use crate::l5_cognition::nt_mind::evolution::rst_flywheel::{_RstFlywheel, _RstTask, _RstVerdict};
    use crate::l5_cognition::nt_mind::evolution::train_pipeline::{_TrainConfig, _TrainPipeline, _TrainStage};
    use crate::l0_substrate::nt_core_self_test::SelfTest;

    #[test]
    fn test_count_actual_unsafe_zero() {
        let s = "fn safe() { let x = 1; }";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_count_actual_unsafe_counts_block() {
        let s = "fn foo() { unsafe { *p = 1; } }";
        assert_eq!(count_actual_unsafe(s), 1);
    }

    #[test]
    fn test_count_actual_unsafe_ignores_comment() {
        let s = "// unsafe { this is just a comment }";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_count_actual_unsafe_ignores_doc_comment() {
        let s = "//! unsafe { doc comment unsafe }";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_count_actual_unsafe_ignores_variable_name() {
        let s = "let unsafe_count = 5;";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_evolution_loop_new() {
        let el = EvolutionLoop::new();
        assert_eq!(el.cycle, 0);
        assert!(el.enabled);
    }

    #[test]
    fn test_scan_project_returns_reasonable_values() {
        let el = EvolutionLoop::new();
        let snap = el.scan_project();
        assert!(snap.total_files > 0 || snap.total_lines == 0);
    }

    #[test]
    fn test_evolution_score_baseline() {
        let el = EvolutionLoop::new();
        let snap = el.scan_project();
        let score = el.compute_evolution_score(&snap, &[]);
        assert!(score >= 0.0 && score <= 100.0);
    }

    #[test]
    fn test_issue_detection_creates_valid_issues() {
        let mut el = EvolutionLoop::new();
        let report = el.run_cycle(Some(0.5), Some(0.3));
        assert_eq!(report.cycle, 1);
        for issue in &report.issues_found {
            assert!(issue.severity >= 1 && issue.severity <= 10);
            assert!(!issue.description.is_empty());
        }
    }

    #[test]
    fn test_high_free_energy_detected() {
        let mut el = EvolutionLoop::new();
        let report = el.run_cycle(Some(5.0), Some(0.3));
        assert!(report.issues_found.iter().any(|i| i.issue_type == IssueType::HighFreeEnergy));
    }

    #[test]
    fn test_low_phi_detected() {
        let mut el = EvolutionLoop::new();
        let report = el.run_cycle(Some(0.5), Some(0.01));
        assert!(report.issues_found.iter().any(|i| i.issue_type == IssueType::LowPhi));
    }

    #[test]
    fn test_stagnation_detection() {
        let mut el = EvolutionLoop::new();
        assert!(!el._needs_human_intervention());
        el.consecutive_stagnant = STAGNATION_LIMIT;
        assert!(el._needs_human_intervention());
    }

    #[test]
    fn test_on_fix_applied_resets_stagnation() {
        let mut el = EvolutionLoop::new();
        el.consecutive_stagnant = 5;
        el.on_fix_applied();
        assert_eq!(el.consecutive_stagnant, 0);
    }

    #[test]
    fn test_dashboard_format() {
        let el = EvolutionLoop::new();
        let snap = el.scan_project();
        let report = EvolutionReport {
            cycle: 1,
            issues_found: vec![],
            issues_fixed: 0,
            snapshot: snap,
            evolution_score: 85.0,
            free_energy: 0.5,
            phi: 0.3,
            suggestions: vec![],
            new_patterns: vec![],
            auto_fixes: 0,
        };
        let db = el.dashboard(&report);
        assert!(db.contains("#"));
        assert!(db.contains("评分"));
    }

    #[test]
    fn test_for_target_sets_target_dir() {
        let el = EvolutionLoop::for_target("/tmp/mock-project");
        assert!(el.target_dir.is_some());
        assert_eq!(el.target_dir.as_deref().unwrap(), std::path::Path::new("/tmp/mock-project"));
    }

    #[test]
    fn test_scan_project_in_arbitrary_dir() {
        // 构造一个临时 mock Rust 项目目录
        let dir = std::env::temp_dir().join(format!("nt-evolve-mock-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create mock src");
        std::fs::write(src.join("main.rs"), "// TODO: fix me\nfn main() { let x = 1; }\n").expect("write mock main");
        std::fs::write(src.join("hot.rs"), "unsafe { }\nunsafe { }\nunsafe { }\nunsafe { }\nunsafe { }\nunsafe { }\n").expect("write mock hot");

        let el = EvolutionLoop::new();
        let snap = el._scan_project_in(Some(&dir));
        assert!(snap.total_files >= 2, "expected >=2 files, got {}", snap.total_files);
        assert!(snap.todo_count >= 1, "expected TODO detected, got {}", snap.todo_count);
        assert!(snap.file_unsafe_hotspots.len() >= 1, "expected unsafe hotspot");
        assert!(snap.unsafe_count >= 6, "expected >=6 unsafe, got {}", snap.unsafe_count);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_run_cycle_in_target_reports_target_snapshot() {
        let dir = std::env::temp_dir().join(format!("nt-evolve-cycle-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create mock src");
        std::fs::write(src.join("lib.rs"), "// TODO x\n").expect("write mock lib");

        let mut el = EvolutionLoop::new();
        let report = el.run_cycle_in(Some(&dir), None, None);
        assert_eq!(report.cycle, 1);
        assert!(report.snapshot.todo_count >= 1, "expected TODO in target, got {}", report.snapshot.todo_count);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_derive_free_energy_phi_non_finite_or_zero() {
        // 显式接世界模型值时应原样透传 (旧语义不被破坏)
        let dir = std::env::temp_dir().join(format!("nt-evolve-fe-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create mock src");
        std::fs::write(src.join("lib.rs"), "pub fn f() {}\n").expect("write mock lib");

        let mut el = EvolutionLoop::new();
        let report = el.run_cycle_in(Some(&dir), Some(0.5), Some(0.3));
        assert_eq!(report.free_energy, 0.5);
        assert_eq!(report.phi, 0.3);

        // None 时从快照派生 (不崩溃, 值为有限数)
        let report2 = el.run_cycle_in(Some(&dir), None, None);
        assert!(report2.free_energy.is_finite(), "free_energy must be finite, got {}", report2.free_energy);
        assert!(report2.phi.is_finite(), "phi must be finite, got {}", report2.phi);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_derive_free_energy_phi_dirty_vs_clean() {
        // 脏项目 (多问题) 自由能应高于干净项目 (风险驱动)
        let clean = ProjectSnapshot {
            total_files: 10,
            total_lines: 1000,
            large_files: vec![],
            modules_without_tests: vec![],
            file_unsafe_hotspots: vec![],
            unsafe_count: 0,
            unwrap_count: 1,
            todo_count: 1,
            test_count: 20,
            test_failures: 0,
            compile_errors: 0,
            compile_warnings: 0,
        };
        let dirty = ProjectSnapshot {
            total_files: 10,
            total_lines: 1000,
            large_files: vec!["big.rs".into()],
            modules_without_tests: vec!["m.rs".into()],
            file_unsafe_hotspots: vec!["u.rs".into()],
            unsafe_count: 8,
            unwrap_count: 40,
            todo_count: 6,
            test_count: 2,
            test_failures: 1,
            compile_errors: 0,
            compile_warnings: 5,
        };
        let (fe_clean, _phi_clean) = EvolutionLoop::derive_free_energy_phi(&clean);
        let (fe_dirty, phi_dirty) = EvolutionLoop::derive_free_energy_phi(&dirty);
        assert!(fe_dirty > fe_clean, "dirty FE ({}) must exceed clean FE ({})", fe_dirty, fe_clean);
        assert!(fe_dirty.is_finite() && phi_dirty.is_finite());
    }

    // ── G8 独立 Auditor 测试 ──────────────────────────────────

    fn snap(unwrap: usize, compile_errors: usize, unsafe_count: usize, todo: usize, hotspots: usize) -> ProjectSnapshot {
        ProjectSnapshot {
            total_files: 10,
            total_lines: 1000,
            large_files: vec![],
            modules_without_tests: vec![],
            file_unsafe_hotspots: (0..hotspots).map(|i| format!("hotspot_{}.rs", i)).collect(),
            unsafe_count,
            unwrap_count: unwrap,
            todo_count: todo,
            test_count: 20,
            test_failures: 0,
            compile_errors,
            compile_warnings: 0,
        }
    }

    #[test]
    fn test_auditor_accepts_improvement() {
        // 修复有效 (指标全面改善) → 三角色共识通过, checkpoint 前移
        let mut auditor = Auditor::new();
        let before = snap(40, 3, 8, 6, 2);
        let after = snap(12, 0, 6, 4, 1);
        auditor.checkpoint(1, &before);

        let v = auditor._verify_change(2, &before, &after);
        assert!(v.passed, "improvement must pass: {}", v.summary());
        assert!(!v.recovered);
        assert_eq!(v.checkpoint_cycle, 2, "pass 后 checkpoint 前移到新快照");
        assert_eq!(auditor.last_checkpoint().unwrap().unwrap_count, 12);
        assert_eq!(auditor.verdict_history.len(), 1);
    }

    #[test]
    fn test_auditor_rejects_regression_and_recovers() {
        // 修复反而引入回归 (unwrap 增加) → Evidence FAIL → 拒绝 + recover 标记
        let mut auditor = Auditor::new();
        let before = snap(10, 0, 5, 3, 1);
        let regression = snap(30, 2, 5, 3, 1);
        auditor.checkpoint(1, &before);

        let v = auditor._verify_change(2, &before, &regression);
        assert!(!v.passed, "regression must be rejected");
        assert!(v.recovered, "拒绝时须标记 recover 回滚至 last-good");
        assert_eq!(v.checkpoint_cycle, 1, "reject 后 checkpoint 保持 last-good");
        assert_eq!(auditor.last_checkpoint().unwrap().unwrap_count, 10);
        assert!(v.role_verdicts.iter().any(|r| !r.pass));
    }

    #[test]
    fn test_auditor_consensus_requires_all_roles() {
        // 仅 Evidence 通过但 Consistency 回归 (todo 激增) → 仍拒绝 (异模型共识)
        let mut auditor = Auditor::new();
        let before = snap(10, 0, 5, 3, 1);
        let side_effect = snap(8, 0, 20, 30, 1);
        auditor.checkpoint(1, &before);

        let v = auditor._verify_change(2, &before, &side_effect);
        assert!(!v.passed, "side-effect regression must fail consistency");
        assert!(v.recovered);
        let consistency = v
            .role_verdicts
            .iter()
            .find(|r| r.role == _AuditorRole::Consistency)
            .expect("consistency role must be present");
        assert!(!consistency.pass);
    }

    #[test]
    fn test_auditor_governance_blocks_new_hotspots() {
        // 治理角色: 新引入 unsafe 热点文件 → Governance FAIL
        let mut auditor = Auditor::new();
        let before = snap(10, 0, 5, 3, 1);
        let new_hotspots = snap(10, 0, 12, 3, 4);
        auditor.checkpoint(1, &before);

        let v = auditor._verify_change(2, &before, &new_hotspots);
        assert!(!v.passed);
        let governance = v
            .role_verdicts
            .iter()
            .find(|r| r.role == _AuditorRole::Governance)
            .expect("governance role must be present");
        assert!(!governance.pass);
    }

    #[test]
    fn test_autofix_cycle_zeroes_fixes_on_reject() {
        // 接线验证: autofix_cycle_in 中 rejected 变更 → auto_fixes=0 + rollback 模式入报告
        let _el = EvolutionLoop::new();
        let dir = std::env::temp_dir().join(format!("nt_audit_gate_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).expect("create mock src");
        std::fs::write(dir.join("src").join("lib.rs"), "pub fn f() {}\n").expect("write mock lib");

        // 无 auto_fixable 问题可修 → 修复数为 0, 且 auditor 已注册裁决 (验证接线编译 + 运行)
        // 用 for_target 锁定扫描目录为 mock 项目 — 否则 PipelineAutoFixer::self_diagnose
        // 会扫整个真实仓库并对真实源文件写修复 (测试隔离纪律)。
        let mut el = EvolutionLoop::for_target(&dir);
        let report = el.autofix_cycle_in(Some(&dir), None, None);
        assert!(el.last_audit.is_some(), "autofix_cycle_in 必须产生审计裁决");
        assert_eq!(report.auto_fixes, 0);
        assert!(report.evolution_score.is_finite());

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── G10: RST flywheel ──────────────────────────────────────────────

    #[test]
    fn rst_seed_creates_verified_gen0() {
        let mut fw = _RstFlywheel::new();
        let seed = fw.seed("parse the config");
        assert_eq!(seed.generation, 0);
        assert!(seed.verified);
        assert_eq!(fw.stats().0, 1);
    }

    #[test]
    fn rst_extend_increments_generation_and_complexity() {
        let mut fw = _RstFlywheel::new();
        let seed = fw.seed("baseline");
        let children = fw.extend(&seed);
        assert_eq!(children.len(), fw.extend_per_gen);
        assert_eq!(children[0].generation, 1);
        assert!(!children[0].verified);
        assert!(children[0].complexity > seed.complexity);
    }

    #[test]
    fn rst_extend_stops_at_max_generation() {
        let mut fw = _RstFlywheel::new();
        let mut t = fw.seed("deep");
        t.generation = fw.max_generation;
        assert!(fw.extend(&t).is_empty());
    }

    #[test]
    fn rst_realign_filters_below_threshold() {
        let fw = _RstFlywheel::new();
        let parent = _RstTask {
            id: "p".into(),
            prompt: "parent".into(),
            generation: 0,
            complexity: 10.0,
            verified: true,
            parent: None,
        };
        let low = _RstTask {
            id: "low".into(),
            prompt: "low".into(),
            generation: 1,
            complexity: 10.4, // < 10×1.1=11 → 淘汰
            verified: false,
            parent: None,
        };
        let mut ok = low.clone();
        ok.id = "ok".into();
        ok.complexity = 12.0;
        let kept = fw._realign(&parent, vec![low, ok]);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].id, "ok");
    }

    #[test]
    fn rst_validate_rejects_empty_prompt() {
        let mut fw = _RstFlywheel::new();
        let bad = _RstTask {
            id: "bad".into(),
            prompt: "  ".into(),
            generation: 1,
            complexity: 5.0,
            verified: false,
            parent: None,
        };
        assert!(fw.validate(vec![bad]).is_empty());
        assert_eq!(fw.rejected_count, 1);
    }

    #[test]
    fn rst_validate_rejects_over_cap() {
        let mut fw = _RstFlywheel::new();
        let over = _RstTask {
            id: "over".into(),
            prompt: "x".into(),
            generation: 1,
            complexity: fw.complexity_cap + 1.0,
            verified: false,
            parent: None,
        };
        assert!(fw.validate(vec![over]).is_empty());
    }

    #[test]
    fn rst_reuse_samples_verified_pool_roundrobin() {
        let mut fw = _RstFlywheel::new();
        fw.seed("a");
        fw.seed("b");
        let first = fw.reuse(0).unwrap().id.clone();
        let second = fw.reuse(1).unwrap().id.clone();
        assert_ne!(first, second, "round-robin rotates across pool");
        let third = fw.reuse(2).unwrap().id.clone();
        assert_eq!(third, first, "offset wraps modulo pool size");
    }

    #[test]
    fn rst_full_generation_grows_pool() {
        let mut fw = _RstFlywheel::new();
        let seed = fw.seed("task");
        let before = fw.stats().0;
        let accepted = fw._run_generation(&seed);
        assert!(accepted > 0);
        assert_eq!(fw.stats().0, before + accepted);
        assert!(fw.accepted_count >= 1 + accepted as u64);
    }

    #[test]
    fn rst_stats_distribution_by_generation() {
        let mut fw = _RstFlywheel::new();
        let seed = fw.seed("gen0");
        fw._run_generation(&seed);
        let (total, dist) = fw.stats();
        assert_eq!(total, 1 + dist.get(1).copied().unwrap_or(0));
        assert!(dist.len() >= 2, "seed(gen0) + children(gen1)");
        assert_eq!(dist[0], 1, "exactly one seed at gen0");
    }

    // ── P0-8 keep-or-revert + seed escalation ──
    #[test]
    fn rst_keep_or_revert_keeps_when_improves() {
        let fw = _RstFlywheel::new();
        assert!(fw._keep_or_revert(10.0, 12.0, 1.05), "12 > 10*1.05 → keep");
    }

    #[test]
    fn rst_keep_or_revert_reverts_when_regresses() {
        let fw = _RstFlywheel::new();
        assert!(!fw._keep_or_revert(10.0, 9.0, 1.05), "9 < 10*1.05 → revert");
    }

    #[test]
    fn rst_keep_or_revert_no_baseline_keeps() {
        let fw = _RstFlywheel::new();
        assert!(fw._keep_or_revert(0.0, 5.0, 1.05), "无基准 → keep");
    }

    #[test]
    fn rst_seed_escalation_tiers() {
        let fw = _RstFlywheel::new();
        assert_eq!(fw._seed_escalation(0.05, 1), (1, false), "低噪声 1 seed");
        assert_eq!(fw._seed_escalation(0.2, 1), (3, true), "中噪声升级到 3");
        assert_eq!(fw._seed_escalation(0.6, 3), (8, true), "高噪声升级到 8");
        assert_eq!(fw._seed_escalation(0.6, 8), (8, false), "已达付费上限");
    }

    // ── P7 MetaHarnessOptimizer ──
    #[test]
    fn test_harness_seed_proposes() {
        let mut opt = MetaHarnessOptimizer::new();
        for c in MetaHarnessOptimizer::_seed_for(&[("compile_ok", _HarnessTarget::Compile), ("bench_fast", _HarnessTarget::Bench)]) {
            opt.propose(c).expect("propose");
        }
        assert_eq!(opt.count(), 2);
    }

    #[test]
    fn test_harness_dedup_by_fingerprint() {
        let mut opt = MetaHarnessOptimizer::new();
        let a = _HarnessCandidate::new(_HarnessTarget::UnitTest, "  fn  a(){} ", vec!["x".into()]);
        let b = _HarnessCandidate::new(_HarnessTarget::UnitTest, "fn a(){}", vec!["x".into()]);
        opt.propose(a).expect("first");
        assert!(opt.propose(b).is_err(), "whitespace-normalized duplicate must be rejected");
    }

    #[test]
    fn test_harness_prune_keeps_top_coverage() {
        let mut opt = MetaHarnessOptimizer::new();
        for c in vec![
            _HarnessCandidate::new(_HarnessTarget::Integration, "c1", vec!["a".into()]),
            _HarnessCandidate::new(_HarnessTarget::Integration, "c2", vec!["a".into(), "b".into()]),
            _HarnessCandidate::new(_HarnessTarget::Integration, "c3", vec!["a".into(), "b".into(), "c".into()]),
        ] {
            opt.propose(c).expect("propose");
        }
        let removed = opt.prune(2);
        assert_eq!(removed, 1);
        assert_eq!(opt.count(), 2);
        assert_eq!(opt.candidates()[0].covers.len(), 3, "top-coverage candidate survives");
    }

    #[test]
    fn test_harness_coverage_grouping() {
        let mut opt = MetaHarnessOptimizer::new();
        opt.propose(_HarnessCandidate::new(_HarnessTarget::Compile, "c1", vec!["a".into()])).unwrap();
        opt.propose(_HarnessCandidate::new(_HarnessTarget::UnitTest, "c2", vec!["b".into()])).unwrap();
        let cov = opt.coverage();
        assert_eq!(cov.get(&_HarnessTarget::Compile), Some(&1));
        assert_eq!(cov.get(&_HarnessTarget::UnitTest), Some(&1));
    }

    #[test]
    fn test_harness_prune_zero_clears() {
        let mut opt = MetaHarnessOptimizer::new();
        opt.propose(_HarnessCandidate::new(_HarnessTarget::Compile, "c1", vec!["a".into()])).unwrap();
        let removed = opt.prune(0);
        assert_eq!(removed, 1);
        assert_eq!(opt.count(), 0);
    }

    #[test]
    fn test_harness_selftest() {
        let opt = MetaHarnessOptimizer::new();
        assert!(opt.self_test().is_ok());
    }

    // ── P15: _TrainPipeline (train-llm-from-scratch 吸收) ──

    #[test]
    fn train_stage_ordering_advances() {
        assert_eq!(_TrainStage::Pretrain.next(), Some(_TrainStage::Sft));
        assert_eq!(_TrainStage::Sft.next(), Some(_TrainStage::Rm));
        assert_eq!(_TrainStage::Rm.next(), Some(_TrainStage::Ppo));
        assert_eq!(_TrainStage::Ppo.next(), Some(_TrainStage::Done));
        assert_eq!(_TrainStage::Dpo.next(), Some(_TrainStage::Done));
        assert_eq!(_TrainStage::Grpo.next(), Some(_TrainStage::Done));
        assert_eq!(_TrainStage::Done.next(), None);
        assert_eq!(_TrainStage::Ppo.label(), "ppo");
        assert_eq!(_TrainStage::Done.label(), "done");
    }

    #[test]
    fn train_pipeline_advance_grows_history() {
        let mut pipe = _TrainPipeline::new(_TrainConfig::default());
        assert_eq!(pipe.current, _TrainStage::Pretrain);
        assert_eq!(pipe.history.len(), 0);
        let next = pipe.advance().expect("advance from Pretrain");
        assert_eq!(next, _TrainStage::Sft);
        assert_eq!(pipe.current, _TrainStage::Sft);
        assert_eq!(pipe.history.len(), 1);
        assert_eq!(pipe.history[0], (_TrainStage::Pretrain, 1));
        assert_eq!(pipe.epochs_run, 1);
    }

    #[test]
    fn train_recommend_strategy_per_stage() {
        let pipe = _TrainPipeline::default();
        assert_eq!(pipe._recommend_strategy(_TrainStage::Pretrain), "pretraining: next-token on corpus");
        assert_eq!(pipe._recommend_strategy(_TrainStage::Sft), "supervised fine-tuning: next-token");
        assert_eq!(pipe._recommend_strategy(_TrainStage::Rm), "reward model: pairwise ranking");
        assert_eq!(pipe._recommend_strategy(_TrainStage::Ppo), "PPO: on-policy RLHF");
        assert_eq!(pipe._recommend_strategy(_TrainStage::Dpo), "DPO: off-policy preference");
        assert_eq!(pipe._recommend_strategy(_TrainStage::Grpo), "GRPO: group relative policy optimization");
        assert_eq!(pipe._recommend_strategy(_TrainStage::Done), "pretraining: next-token on corpus");
    }

    #[test]
    fn train_scale_lr_lowers_for_larger_models() {
        let mut pipe = _TrainPipeline::new(_TrainConfig::default());
        let base = pipe.config.lr;
        let big = pipe._scale_lr(1e10);
        assert!(big < base, "10B model lr ({big}) must be smaller than base ({base})");
        assert_eq!(pipe.config.lr, big);

        let mut small_pipe = _TrainPipeline::new(_TrainConfig::default());
        let small_base = small_pipe.config.lr;
        let small = small_pipe._scale_lr(1e8);
        assert!(small > small_base, "100M model lr ({small}) must be larger than base ({small_base})");
    }

    #[test]
    fn train_is_complete_only_when_done() {
        let mut pipe = _TrainPipeline::default();
        assert!(!pipe.is_complete());
        for _ in 0..4 {
            pipe.advance();
        }
        assert_eq!(pipe.current, _TrainStage::Done);
        assert!(pipe.is_complete());
        assert_eq!(pipe.advance(), None, "Done 之后 advance 返回 None");
        assert!(pipe.is_complete());
    }

    #[test]
    fn train_stage_progress_bounds_and_monotonic() {
        let mut pipe = _TrainPipeline::default();
        assert_eq!(pipe._stage_progress(), 0.0);
        assert!((0.0..=1.0).contains(&pipe._stage_progress()));
        let mut last = 0.0;
        for _ in 0..4 {
            pipe.advance();
            let p = pipe._stage_progress();
            assert!(p >= last && p <= 1.0, "progress must stay in [0,1] and be monotonic");
            last = p;
        }
        assert_eq!(pipe._stage_progress(), 1.0);
    }

    #[test]
    fn train_pipeline_selftest_passes() {
        let pipe = _TrainPipeline::new(_TrainConfig::default());
        assert!(pipe.self_test().is_ok());
    }

    // ── A2 (autoresearch): 有界循环 + commit-then-verify ──
    #[test]
    fn rst_budget_exhausts_after_budget_iters() {
        let mut fw = _RstFlywheel::new();
        fw.iteration_budget = 2;
        let seed = fw.seed("base task");
        assert!(!fw.budget_exhausted());
        assert!(fw._bounded_run_generation(&seed).is_some());
        assert!(fw._bounded_run_generation(&seed).is_some());
        assert!(fw.budget_exhausted(), "预算耗尽后停");
        assert!(fw._bounded_run_generation(&seed).is_none(), "耗尽后不再运行");
        assert_eq!(fw.iterations_used, 2);
    }

    #[test]
    fn rst_commit_then_verify_reverts_on_regression() {
        let fw = _RstFlywheel::new();
        // keep: 新分显著更高
        let (keep, revert) = fw._commit_then_verify(10.0, 20.0, 1.1);
        assert!(keep && !revert);
        // revert: 新分低于旧分 × 阈值
        let (keep, revert) = fw._commit_then_verify(10.0, 5.0, 1.1);
        assert!(!keep && revert);
        // 无基准 (old=0) → keep
        assert!(fw._commit_then_verify(0.0, 1.0, 1.1).0);
    }

    // ── A3 (DarwinX): preserve-and-extend 种群档案 ──
    #[test]
    fn meta_harness_preserve_and_extend_admits_superset() {
        let mut opt = MetaHarnessOptimizer::new();
        opt.propose(_HarnessCandidate::new(
            _HarnessTarget::UnitTest,
            "fn a(){}",
            vec!["tok_a".into()],
        ))
        .unwrap();
        // 扩展覆盖且不回归 → 录取
        let ext = _HarnessCandidate::new(
            _HarnessTarget::UnitTest,
            "fn ab(){}",
            vec!["tok_a".into(), "tok_b".into()],
        );
        let admitted = opt._admit_preserve_and_extend(ext, 10).unwrap();
        assert!(admitted);
        assert_eq!(opt.count(), 2);
    }

    #[test]
    fn meta_harness_preserve_and_extend_rejects_regress_to_archive() {
        let mut opt = MetaHarnessOptimizer::new();
        opt.propose(_HarnessCandidate::new(
            _HarnessTarget::UnitTest,
            "fn a(){}",
            vec!["tok_a".into(), "tok_b".into()],
        ))
        .unwrap();
        // 回归: 丢掉了 tok_b → 拒绝入档案 (供重组)
        let regress = _HarnessCandidate::new(
            _HarnessTarget::UnitTest,
            "fn a_only(){}",
            vec!["tok_a".into()],
        );
        let admitted = opt._admit_preserve_and_extend(regress, 10).unwrap();
        assert!(!admitted);
        assert_eq!(opt.count(), 1, "回归变体不录取");
        assert_eq!(opt._archive_len(), 1, "回归变体入档案");
        assert!(opt._recombine(0).is_some(), "档案可重组");
    }
}
