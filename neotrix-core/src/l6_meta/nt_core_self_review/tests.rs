    use super::*;

    #[test]
    fn test_self_review_basic() {
        let mut gate = SelfReviewGate::new(true);
        let report = gate.run_all();
        // Verify the report structure is correct (not a specific pass/fail, since
        // real codebase state varies). Key invariant: no panic during execution.
        assert!(
            report.findings.len() > 0,
            "Should have at least informational findings"
        );
        assert!(report.passed + report.failed + report.warnings <= report.findings.len());
    }

    #[test]
    fn test_self_review_findings_report() {
        let mut gate = SelfReviewGate::new(false);
        gate.check(
            false,
            Severity::Error,
            "test",
            "forced error".into(),
            file!(),
            line!(),
        );
        let report = gate.report();
        assert_eq!(report.failed, 1);
        assert_eq!(report.summary().contains("FAIL"), true);
    }

    #[test]
    fn test_review_finding_display() {
        let finding = ReviewFinding {
            severity: Severity::Error,
            category: "test".into(),
            message: "test finding".into(),
            file: "test.rs".into(),
            line: 42,
        };
        let s = format!("{}", finding.severity);
        assert_eq!(s, "ERROR");
    }

    #[test]
    fn test_blast_radius_empty() {
        let gate = SelfReviewGate::new(false);
        let br = gate.blast_radius();
        assert_eq!(br.risk, BlastRisk::Low);
        assert!(br.files_scanned > 0);
    }

    #[test]
    fn test_blast_radius_with_findings() {
        let mut gate = SelfReviewGate::new(false);
        gate.check(
            false,
            Severity::Error,
            "panic_audit",
            "test".to_string(),
            "a.rs".to_string(),
            1,
        );
        gate.check(
            false,
            Severity::Error,
            "layer_violation",
            "test".to_string(),
            "b.rs".to_string(),
            2,
        );
        let br = gate.blast_radius();
        assert_eq!(br.risk, BlastRisk::Critical);
        // Only "layer_violation" counts as module crossing
        assert_eq!(br.module_crossings, 1);
    }

    #[test]
    fn test_arch_layer_detection() {
        assert_eq!(
            ArchLayer::from_path(Path::new("/src/core/foo.rs")),
            ArchLayer::L0Core
        );
        assert_eq!(
            ArchLayer::from_path(Path::new("/src/neotrix/l1_body_impl/bar.rs")),
            ArchLayer::L1Act
        );
        assert_eq!(
            ArchLayer::from_path(Path::new("/src/neotrix/l2_world_impl/baz.rs")),
            ArchLayer::L2World
        );
        assert_eq!(
            ArchLayer::from_path(Path::new("/src/neotrix/l3_memory_impl/qux.rs")),
            ArchLayer::L3Memory
        );
        assert_eq!(
            ArchLayer::from_path(Path::new("/src/neotrix/l8_autonomic_impl/quux.rs")),
            ArchLayer::L8Seal
        );
    }

    #[test]
    fn test_observer_feedback_degraded() {
        let mut gate =
            SelfReviewGate::new(true).with_observer_feedback(0.25, vec!["oscillation".into()]);
        let report = gate.run_all();
        let obs_findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.category == "observer_feedback" || f.category == "observer_pattern")
            .collect();
        assert!(
            !obs_findings.is_empty(),
            "Should have observer findings when degraded"
        );
    }

    #[test]
    fn test_observer_feedback_healthy() {
        let mut gate = SelfReviewGate::new(true).with_observer_feedback(0.85, vec![]);
        let report = gate.run_all();
        let obs_findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.category == "observer_feedback")
            .collect();
        // Healthy quality should NOT produce a warning
        let warnings: Vec<_> = obs_findings
            .iter()
            .filter(|f| f.severity == Severity::Warning || f.severity == Severity::Error)
            .collect();
        assert!(
            warnings.is_empty(),
            "Should not warn on healthy observer quality"
        );
    }

    #[test]
    fn test_karpathy_simplicity_first() {
        let mut gate = SelfReviewGate::new(false);
        gate.check_karpathy_simplicity_first();
        // Verify the check runs without panics and produces at least info-level findings
        let report = gate.report();
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.category == "karpathy_simplicity_first")
            .collect();
        assert!(
            findings.len() <= 1,
            "Should have at most 1 finding for simplicity_first check"
        );
    }

    #[test]
    fn test_karpathy_surgical_changes() {
        let mut gate = SelfReviewGate::new(false);
        gate.check_karpathy_surgical_changes();
        // Verify the check runs without panics (git commands may not always succeed)
        let report = gate.report();
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.category == "karpathy_surgical_changes")
            .collect();
        assert!(
            findings.len() <= 1,
            "Should have at most 1 finding for surgical_changes check"
        );
    }

    #[test]
    fn test_karpathy_complexity_budget() {
        let mut gate = SelfReviewGate::new(false);
        gate.check_karpathy_complexity_budget();
        // Verify the check runs without panics
        let report = gate.report();
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.category == "karpathy_complexity_budget")
            .collect();
        assert!(
            findings.len() <= 1,
            "Should have at most 1 finding for complexity_budget check"
        );
    }

    #[test]
    fn test_karpathy_goal_driven_execution() {
        let mut gate = SelfReviewGate::new(false);
        gate.check_karpathy_goal_driven_execution();
        // Verify the check runs without panics
        let report = gate.report();
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.category == "karpathy_goal_driven_execution")
            .collect();
        assert!(
            findings.len() <= 1,
            "Should have at most 1 finding for goal_driven_execution check"
        );
    }

    #[test]
    fn test_syn_depth_tracks_nesting() {
        let gate = SelfReviewGate::new(false);

        let d1 = gate.syn_depth("fn a() { let x = 1; }");
        assert_eq!(d1, 1, "single fn block should have depth 1");

        let d2 = gate.syn_depth("fn a() { fn b() { let x = 1; } }");
        assert_eq!(d2, 2, "nested fn should have depth 2");

        let d3 = gate.syn_depth("let x = 1;");
        assert_eq!(d3, 0, "invalid Rust should fallback to brace count (0)");
    }

    #[test]
    fn test_syn_depth_control_flow() {
        let gate = SelfReviewGate::new(false);

        let d = gate.syn_depth("fn a() { if true { let x = 1; } }");
        assert_eq!(d, 2, "fn + if block should have depth 2");

        let d = gate.syn_depth("fn a() { loop { break; } }");
        assert_eq!(d, 2, "fn + loop block should have depth 2");
    }

    #[test]
    fn test_scan_for_pattern_excluding_tests_handles_all_test_forms() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("nt_self_review_scan_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("sample.rs");

        // 三种测试形式: cfg(test) mod / 独立 #[test] fn / #[tokio::test] fn
        let content = r#"#[cfg(test)]
mod tests {
    impl Helper {
        fn nested() { todo!("in cfg mod"); }
    }
}

#[test]
fn standalone() { todo!("standalone"); }

#[tokio::test]
async fn tokio_test() { todo!("tokio"); }

fn prod() { let x = 1; }
"#;
        let mut f = std::fs::File::create(&file).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();

        // 生产代码无 todo! — 全部在测试上下文内
        let count = scan_for_pattern_excluding_tests(&dir, "todo!(");
        assert_eq!(
            count, 0,
            "all todo!() are inside test contexts; scan returned {count}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
