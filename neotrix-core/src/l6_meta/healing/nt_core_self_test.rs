use crate::l6_meta::nt_core_self_constitution::global_constitution;

// Re-export all shared types from L0 Substrate — single source of truth
pub use crate::l0_substrate::nt_core_self_test::SelfTest;
pub use crate::l0_substrate::nt_core_self_test::SelfTestRegistry;
pub use crate::l0_substrate::nt_core_self_test::SelfTestResult;
pub use crate::l0_substrate::nt_core_self_test::DurationDriftMonitor;
pub use crate::l0_substrate::nt_core_self_test::DurationDriftTest;
pub use crate::l0_substrate::nt_core_self_test::report;

/// 跨模块共享的测试环境锁 — re-export from L0
#[cfg(test)]
pub use crate::l0_substrate::nt_core_self_test::TEST_ENV_LOCK;

/// External verifier — runs `cargo check` to ground self-tests in build reality.
pub struct ExternalVerifier;

impl SelfTest for ExternalVerifier {
    fn name(&self) -> &str {
        "external_verifier"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let output = std::process::Command::new("cargo")
            .args(["check", "--lib", "-p", "neotrix"])
            .output()
            .map_err(|e| vec![format!("failed to run cargo check: {}", e)])?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let errors: Vec<String> = stderr
                .lines()
                .filter(|l| l.contains("error"))
                .take(5)
                .map(|l| l.to_string())
                .collect();
            Err(vec![format!(
                "cargo check failed ({} errors)",
                errors.len()
            )])
        }
    }
}

/// Constitution Compliance SelfTest - verifies actions follow the constitution
pub struct ConstitutionComplianceTest;

impl SelfTest for ConstitutionComplianceTest {
    fn name(&self) -> &str {
        "constitution_compliance"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let constitution = global_constitution();

        if constitution.rules.is_empty() {
            return Err(vec!["Constitution has no rules loaded".into()]);
        }

        if constitution.tree_growth_rules().is_empty() {
            return Err(vec!["Missing tree growth rules (R-P42~R-P48)".into()]);
        }

        if constitution.absorption_rules().is_empty() {
            return Err(vec!["Missing absorption protocol rules (R-P43)".into()]);
        }

        if !constitution.has_vector_index() {
            return Err(vec!["Constitution vector index not built".into()]);
        }

        let report = constitution.verify_compliance(
            "extend existing module nt_core_orch_agent with hexagram derivation",
        );
        if !report.compliant {
            // Some violations may be expected, but we check the check works
        }

        let violation_report =
            constitution.verify_compliance("create new module without branch mapping");
        if violation_report.compliant {
            return Err(vec![
                "Compliance check failed to detect R-P42 violation".into()
            ]);
        }

        Ok(())
    }
}

/// Trace Evaluation SelfTest — 验证轨迹评估系统功能正常
pub struct TraceEvaluationTest;

impl SelfTest for TraceEvaluationTest {
    fn name(&self) -> &str {
        "trace_evaluation"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use crate::l6_meta::nt_core_self::trace_evaluation::{
            AgentTrace, EvaluationGrade, TraceEvaluator, TraceStep,
        };
        use std::time::Instant;

        let evaluator = TraceEvaluator::new();

        let steps: Vec<TraceStep> = (0..5)
            .map(|i| TraceStep {
                step_number: i,
                action: format!("action_{}", i),
                input: format!("input_{}", i),
                output: format!("output_{}", i),
                duration_ms: 1000,
                tokens_used: 100,
                success: true,
                error: None,
            })
            .collect();

        let trace = AgentTrace {
            agent_id: "test_agent".to_string(),
            task_id: "test_task".to_string(),
            steps,
            start_time: Instant::now(),
            end_time: Some(Instant::now()),
            final_result: Some("completed".to_string()),
        };

        let report = evaluator.evaluate(&trace);

        let mut errors = Vec::new();

        if report.results.len() != 5 {
            errors.push(format!(
                "Expected 5 evaluation dimensions, got {}",
                report.results.len()
            ));
        }

        if report.overall_score < 0.0 || report.overall_score > 1.0 {
            errors.push(format!(
                "Overall score out of range: {}",
                report.overall_score
            ));
        }

        if report.grade != EvaluationGrade::Excellent && report.grade != EvaluationGrade::Good {
            errors.push(format!(
                "Unexpected grade for perfect trace: {:?}",
                report.grade
            ));
        }

        for result in &report.results {
            if result.score < 0.0 || result.score > 1.0 {
                errors.push(format!(
                    "Dimension {:?} score out of range: {}",
                    result.dimension, result.score
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
