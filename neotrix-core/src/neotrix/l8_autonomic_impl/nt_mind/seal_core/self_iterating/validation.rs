use crate::core::nt_core_knowledge::RewardSource;

/// Result of an external validation check.
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Pass(f64),
    Fail(f64, String),
    Skipped,
}

/// Perform a cargo check validation.
/// Returns Pass(0.8) if compilation succeeds, Fail(-0.3) otherwise.
pub fn cargo_check_validation() -> ValidationResult {
    let output = std::process::Command::new("cargo")
        .args(["check", "--lib"])
        .output();
    match output {
        Ok(out) => {
            if out.status.success() {
                ValidationResult::Pass(0.8)
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let error_count = stderr.matches("error").count();
                ValidationResult::Fail(-0.3, format!("{} compilation errors", error_count))
            }
        }
        Err(e) => ValidationResult::Fail(-0.5, format!("cargo check failed to execute: {}", e)),
    }
}

/// TasteSkill quality gate — evaluates output for anti-slop metrics.
/// Non-blocking: returns Skipped if TasteSkill KS not available.
pub fn taste_skill_gate(output: &str) -> ValidationResult {
    if output.is_empty() {
        return ValidationResult::Skipped;
    }
    let has_variance = output.contains("VARIANCE") || output.contains("variance");
    let has_motion = output.contains("MOTION") || output.contains("motion");
    let has_density = output.contains("DENSITY") || output.contains("density");
    let signal_count = [has_variance, has_motion, has_density].iter().filter(|&&x| x).count();
    if signal_count >= 2 {
        ValidationResult::Pass(0.6)
    } else if signal_count >= 1 {
        ValidationResult::Pass(0.3)
    } else {
        ValidationResult::Skipped
    }
}

/// User feedback interface for MicroEdit proposals.
/// In headless mode, returns None (no feedback available).
pub fn user_accept_reject(_edit_description: &str) -> Option<ValidationResult> {
    None
}

/// P0-5 task-native verification (吸收 Argus 模式):
/// 按任务类型选择对应验证器 — 而非一刀切编译验证。
/// Argus 原文: "task-native verification — the verification method
/// must match the task type, not a one-size-fits-all gate"。
///
/// 任务类型 → 验证器映射:
/// - CodeGeneration / CodeReview / CodeRefactor → 编译 + 测试 (cargo)
/// - Retrieval / Search → 结果非空 + 精度门 (precision gate)
/// - Writing / Content → 质量信号门 (TasteSkill 风格)
/// - General → 编译验证 (默认)
pub fn task_native_validation(task_type: &str, artifact: Option<&str>) -> ValidationResult {
    match task_type {
        "CodeGeneration" | "CodeReview" | "CodeRefactor" | "Code" => cargo_check_validation(),
        "Retrieval" | "Search" | "Query" => {
            match artifact {
                Some(out) if !out.trim().is_empty() => {
                    // 检索任务: 结果非空 + 基础精度信号 (含证据/来源标记)
                    let has_evidence = out.contains("file:") || out.contains(":") || out.contains("source");
                    if has_evidence {
                        ValidationResult::Pass(0.7)
                    } else {
                        ValidationResult::Pass(0.4)
                    }
                }
                _ => ValidationResult::Fail(-0.3, "retrieval returned empty result".into()),
            }
        }
        "Writing" | "Content" | "Documentation" => {
            match artifact {
                Some(out) if !out.trim().is_empty() => {
                    // 写作任务: 质量信号门 (复用 TasteSkill 反 slop 度量)
                    taste_skill_gate(out)
                }
                _ => ValidationResult::Fail(-0.3, "writing produced empty output".into()),
            }
        }
        _ => cargo_check_validation(),
    }
}

/// Aggregate multiple validation results into a single reward.
/// External rewards weighted by RewardSource::priority_multiplier().
pub fn aggregate_reward(results: &[ValidationResult]) -> (f64, RewardSource) {
    let total: f64 = results.iter().map(|r| match r {
        ValidationResult::Pass(v) => *v,
        ValidationResult::Fail(v, _) => *v,
        ValidationResult::Skipped => 0.0,
    }).sum();
    let count = results.iter().filter(|r| !matches!(r, ValidationResult::Skipped)).count() as f64;
    if total >= 0.0 && count > 0.0 {
        (total / count * RewardSource::External.priority_multiplier(), RewardSource::External)
    } else {
        (0.0, RewardSource::Internal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_source_priority() {
        assert_eq!(RewardSource::External.priority_multiplier(), 2.0);
        assert_eq!(RewardSource::Internal.priority_multiplier(), 1.0);
    }

    #[test]
    fn test_aggregate_skipped_only() {
        let results = vec![ValidationResult::Skipped];
        let (reward, source) = aggregate_reward(&results);
        assert_eq!(reward, 0.0);
        assert_eq!(source, RewardSource::Internal);
    }

    #[test]
    fn test_aggregate_pass() {
        let results = vec![ValidationResult::Pass(0.8)];
        let (reward, source) = aggregate_reward(&results);
        assert_eq!(source, RewardSource::External);
        assert!(reward > 0.0);
    }

    #[test]
    fn test_aggregate_mixed() {
        let results = vec![
            ValidationResult::Pass(0.8),
            ValidationResult::Skipped,
        ];
        let (reward, source) = aggregate_reward(&results);
        assert_eq!(source, RewardSource::External);
        assert!((reward - 1.6).abs() < 0.01);
    }

    #[test]
    fn test_taste_skill_gate_empty() {
        let r = taste_skill_gate("");
        assert!(matches!(r, ValidationResult::Skipped));
    }
}
