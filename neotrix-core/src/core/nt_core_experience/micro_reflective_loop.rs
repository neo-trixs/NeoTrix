use std::collections::VecDeque;

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

const HISTORY_CAPACITY: usize = 64;
const DIAGNOSIS_SIMILARITY_THRESHOLD: f64 = 0.75;
const MAX_TRAINING_PAIRS: usize = 128;
const DIFFICULTY_LEVELS: usize = 5;

#[derive(Debug, Clone)]
pub struct ErrorTrajectory {
    pub context_vsa: Vec<u8>,
    pub attempted_action: String,
    pub result_description: String,
    pub error_label: String,
    pub difficulty: u8,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct Diagnosis {
    pub error_pattern: String,
    pub root_cause: String,
    pub severity: u8,
}

#[derive(Debug, Clone)]
pub struct CorrectedTrajectory {
    pub original_context_vsa: Vec<u8>,
    pub corrected_action: String,
    pub diagnosis: Diagnosis,
    pub difficulty: u8,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct TrainingPair {
    pub input_vsa: Vec<u8>,
    pub error_pattern_vsa: Vec<u8>,
    pub corrected_action_vsa: Vec<u8>,
    pub difficulty: u8,
    pub success_count: u32,
    pub total_attempts: u32,
}

pub struct MicroReflectiveLoop {
    history: VecDeque<ErrorTrajectory>,
    training_pairs: Vec<TrainingPair>,
    difficulty_bounds: [f64; DIFFICULTY_LEVELS],
    per_difficulty_accuracy: [f64; DIFFICULTY_LEVELS],
    per_difficulty_counts: [u32; DIFFICULTY_LEVELS],
}

impl MicroReflectiveLoop {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(HISTORY_CAPACITY),
            training_pairs: Vec::with_capacity(MAX_TRAINING_PAIRS),
            difficulty_bounds: [0.2, 0.4, 0.6, 0.8, 1.0],
            per_difficulty_accuracy: [0.0; DIFFICULTY_LEVELS],
            per_difficulty_counts: [0; DIFFICULTY_LEVELS],
        }
    }

    pub fn record_error(&mut self, trajectory: ErrorTrajectory) {
        if self.history.len() >= HISTORY_CAPACITY {
            self.history.pop_front();
        }
        self.history.push_back(trajectory);
    }

    pub fn diagnose(&self) -> Vec<Diagnosis> {
        let recent: Vec<&ErrorTrajectory> = self.history.iter().filter(|t| !t.success).collect();

        if recent.is_empty() {
            return Vec::new();
        }

        let mut diagnoses = Vec::new();
        let mut seen_patterns: Vec<Vec<u8>> = Vec::new();

        for traj in &recent {
            let pattern_vsa = error_to_pattern_vsa(traj);
            if seen_patterns
                .iter()
                .any(|p| QuantizedVSA::similarity(p, &pattern_vsa) > DIAGNOSIS_SIMILARITY_THRESHOLD)
            {
                continue;
            }
            seen_patterns.push(pattern_vsa);
            diagnoses.push(Diagnosis {
                error_pattern: classify_error_pattern(traj),
                root_cause: infer_root_cause(traj),
                severity: rate_severity(traj),
            });
        }

        diagnoses
    }

    pub fn generate_corrected_trajectories(
        &self,
        diagnoses: &[Diagnosis],
    ) -> Vec<CorrectedTrajectory> {
        let mut corrected = Vec::new();
        for diag in diagnoses {
            let matching_errors: Vec<&ErrorTrajectory> = self
                .history
                .iter()
                .filter(|t| !t.success && classify_error_pattern(t) == diag.error_pattern)
                .collect();

            if matching_errors.is_empty() {
                continue;
            }

            let representative = matching_errors[0];
            let corrected_action =
                suggest_correction(&diag.error_pattern, &representative.attempted_action);

            corrected.push(CorrectedTrajectory {
                original_context_vsa: representative.context_vsa.clone(),
                corrected_action,
                diagnosis: diag.clone(),
                difficulty: representative.difficulty,
                verified: false,
            });
        }
        corrected
    }

    pub fn distill_training_pairs(&mut self) -> &[TrainingPair] {
        let diagnoses = self.diagnose();
        let corrected = self.generate_corrected_trajectories(&diagnoses);

        for trajectory in corrected {
            let error_pattern_vsa = error_to_pattern_vsa(&ErrorTrajectory {
                context_vsa: trajectory.original_context_vsa.clone(),
                attempted_action: String::new(),
                result_description: trajectory.diagnosis.error_pattern.clone(),
                error_label: trajectory.diagnosis.root_cause.clone(),
                difficulty: trajectory.difficulty,
                success: false,
            });

            let corrected_action_vsa = text_to_vsa(&trajectory.corrected_action);

            if let Some(existing) = self.training_pairs.iter_mut().find(|p| {
                QuantizedVSA::similarity(&p.error_pattern_vsa, &error_pattern_vsa)
                    > DIAGNOSIS_SIMILARITY_THRESHOLD
            }) {
                existing.total_attempts += 1;
            } else if self.training_pairs.len() < MAX_TRAINING_PAIRS {
                self.training_pairs.push(TrainingPair {
                    input_vsa: trajectory.original_context_vsa,
                    error_pattern_vsa,
                    corrected_action_vsa,
                    difficulty: trajectory.difficulty,
                    success_count: 0,
                    total_attempts: 1,
                });
            }
        }

        &self.training_pairs
    }

    pub fn record_outcome(&mut self, error_pattern: &str, succeeded: bool) {
        if let Some(pair) = self.training_pairs.iter_mut().find(|p| {
            let pattern_vsa = text_to_vsa(error_pattern);
            QuantizedVSA::similarity(&p.error_pattern_vsa, &pattern_vsa)
                > DIAGNOSIS_SIMILARITY_THRESHOLD
        }) {
            pair.total_attempts += 1;
            if succeeded {
                pair.success_count += 1;
            }
        }
    }

    pub fn compute_capability_boundary(&mut self) -> Vec<(u8, f64, f64)> {
        let mut boundaries = Vec::new();
        for d in 0..DIFFICULTY_LEVELS {
            let level = d as u8;
            let total: u32 = self.per_difficulty_counts[d];
            let accuracy = if total > 0 {
                self.per_difficulty_accuracy[d] / total as f64
            } else {
                1.0
            };
            let bound = self.difficulty_bounds[d];
            boundaries.push((level, bound, accuracy));
        }
        boundaries
    }

    pub fn training_pairs(&self) -> &[TrainingPair] {
        &self.training_pairs
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

fn error_to_pattern_vsa(traj: &ErrorTrajectory) -> Vec<u8> {
    let combined = format!(
        "{}:{}:{}",
        traj.error_label, traj.result_description, traj.attempted_action
    );
    let seed: u64 = combined
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    QuantizedVSA::seeded_random(seed, VSA_DIM)
}

fn text_to_vsa(text: &str) -> Vec<u8> {
    let seed: u64 = text
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    QuantizedVSA::seeded_random(seed, VSA_DIM)
}

fn classify_error_pattern(traj: &ErrorTrajectory) -> String {
    let error_lower = traj.error_label.to_lowercase();
    if error_lower.contains("type") || error_lower.contains("mismatch") {
        "type_mismatch".to_string()
    } else if error_lower.contains("timeout") || error_lower.contains("time_out") {
        "timeout".to_string()
    } else if error_lower.contains("not found") || error_lower.contains("missing") {
        "missing_resource".to_string()
    } else if error_lower.contains("permission") || error_lower.contains("denied") {
        "permission_denied".to_string()
    } else if error_lower.contains("parse") || error_lower.contains("syntax") {
        "parse_error".to_string()
    } else {
        "unknown_error".to_string()
    }
}

fn infer_root_cause(traj: &ErrorTrajectory) -> String {
    let pattern = classify_error_pattern(traj);
    match pattern.as_str() {
        "type_mismatch" => format!(
            "expected type constraint violated by action: {}",
            traj.attempted_action
        ),
        "timeout" => "operation exceeded allocated time budget".to_string(),
        "missing_resource" => "required dependency not satisfied before execution".to_string(),
        "permission_denied" => "insufficient authority level for requested operation".to_string(),
        "parse_error" => "input format did not match expected schema".to_string(),
        _ => format!("unclassified error in: {}", traj.result_description),
    }
}

fn rate_severity(traj: &ErrorTrajectory) -> u8 {
    let pattern = classify_error_pattern(traj);
    match pattern.as_str() {
        "type_mismatch" => 7,
        "timeout" => 5,
        "missing_resource" => 4,
        "permission_denied" => 8,
        "parse_error" => 3,
        _ => 6,
    }
}

fn suggest_correction(pattern: &str, action: &str) -> String {
    match pattern {
        "type_mismatch" => format!("verify_type({}) then retry", action),
        "timeout" => format!("retry_with_backoff({})", action),
        "missing_resource" => format!("ensure_dependencies({}) then retry", action),
        "permission_denied" => format!("escalate_authority({})", action),
        "parse_error" => format!("validate_input_before({})", action),
        _ => format!("diagnose_and_retry({})", action),
    }
}

impl Default for MicroReflectiveLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_error(label: &str, action: &str, difficulty: u8) -> ErrorTrajectory {
        ErrorTrajectory {
            context_vsa: QuantizedVSA::random_binary(),
            attempted_action: action.to_string(),
            result_description: format!("failed: {}", label),
            error_label: label.to_string(),
            difficulty,
            success: false,
        }
    }

    #[test]
    fn test_new_loop_empty() {
        let loop_ = MicroReflectiveLoop::new();
        assert_eq!(loop_.history_len(), 0);
        assert!(loop_.training_pairs().is_empty());
    }

    #[test]
    fn test_record_error_adds_to_history() {
        let mut loop_ = MicroReflectiveLoop::new();
        loop_.record_error(make_error("type_mismatch", "call_fn(x)", 1));
        assert_eq!(loop_.history_len(), 1);
    }

    #[test]
    fn test_diagnose_returns_diagnoses() {
        let mut loop_ = MicroReflectiveLoop::new();
        loop_.record_error(make_error("type_mismatch", "call_fn(x)", 1));
        loop_.record_error(make_error("timeout", "fetch_data()", 2));
        let diagnoses = loop_.diagnose();
        assert!(!diagnoses.is_empty());
        assert!(diagnoses.iter().any(|d| d.error_pattern == "type_mismatch"));
    }

    #[test]
    fn test_distill_creates_training_pairs() {
        let mut loop_ = MicroReflectiveLoop::new();
        loop_.record_error(make_error("type_mismatch", "call_fn(x)", 1));
        loop_.record_error(make_error("timeout", "fetch_data()", 2));
        let pairs = loop_.distill_training_pairs();
        assert!(!pairs.is_empty());
    }

    #[test]
    fn test_compute_boundary() {
        let mut loop_ = MicroReflectiveLoop::new();
        let boundaries = loop_.compute_capability_boundary();
        assert_eq!(boundaries.len(), DIFFICULTY_LEVELS);
    }

    #[test]
    fn test_classify_patterns() {
        let t1 = make_error("parse_error", "read_config()", 1);
        assert_eq!(classify_error_pattern(&t1), "parse_error");
        let t2 = make_error("permission denied", "write_file()", 2);
        assert_eq!(classify_error_pattern(&t2), "permission_denied");
    }

    #[test]
    fn test_corrected_trajectory_generation() {
        let mut loop_ = MicroReflectiveLoop::new();
        loop_.record_error(make_error("timeout", "query_api()", 3));
        let diagnoses = loop_.diagnose();
        let corrected = loop_.generate_corrected_trajectories(&diagnoses);
        assert!(!corrected.is_empty());
        assert!(corrected[0].corrected_action.contains("retry_with_backoff"));
    }
}
