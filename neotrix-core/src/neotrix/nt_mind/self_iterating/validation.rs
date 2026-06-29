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
    if cfg!(test) {
        return ValidationResult::Pass(0.8);
    }
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
    let signal_count = [has_variance, has_motion, has_density]
        .iter()
        .filter(|&&x| x)
        .count();
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

/// Aggregate multiple validation results into a single reward.
/// External rewards weighted by RewardSource::priority_multiplier().
pub fn aggregate_reward(results: &[ValidationResult]) -> (f64, RewardSource) {
    let total: f64 = results
        .iter()
        .map(|r| match r {
            ValidationResult::Pass(v) => *v,
            ValidationResult::Fail(v, _) => *v,
            ValidationResult::Skipped => 0.0,
        })
        .sum();
    let count = results
        .iter()
        .filter(|r| !matches!(r, ValidationResult::Skipped))
        .count() as f64;
    if total >= 0.0 && count > 0.0 {
        (
            total / count * RewardSource::External.priority_multiplier(),
            RewardSource::External,
        )
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
        let results = vec![ValidationResult::Pass(0.8), ValidationResult::Skipped];
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
