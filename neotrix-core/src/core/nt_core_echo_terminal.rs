//! ECHO: Terminal Agents Learn World Models for Free
//!
//! Implements the core insight from Shrivastava et al. (arXiv 2605.24517):
//! CLI terminal output (stdout, stderr, exit codes, file diffs) is a dense
//! supervision signal that standard agent RL discards. ECHO adds a complementary
//! cross-entropy loss on environment-observation tokens, reusing the same forward
//! pass as GRPO without additional rollouts.
//!
//! In NeoTrix, this module:
//! 1. Captures terminal observations from CLI tool execution
//! 2. Encodes them as VSA-like feature vectors
//! 3. Provides PRM-compatible scoring of terminal outcomes
//! 4. Generates ECHO-style training targets for policy improvement
//!
//! Layer: L2 (Perception) → L4 (Cognition) — terminal observation as learning signal

use std::collections::VecDeque;
use std::time::Instant;

/// A single terminal observation from tool/command execution
#[derive(Debug, Clone)]
pub struct TerminalObservation {
    pub timestamp: Instant,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub file_changes: Vec<FileChange>,
    pub success: bool,
}

impl TerminalObservation {
    pub fn is_empty(&self) -> bool {
        self.stdout.is_empty() && self.stderr.is_empty()
    }

    /// Signal strength: how much information this observation carries
    pub fn signal_strength(&self) -> f64 {
        let mut score = 0.0;
        if !self.stdout.is_empty() {
            score += 0.3 * (self.stdout.len() as f64 / 1000.0).min(1.0);
        }
        if !self.stderr.is_empty() {
            score += 0.4; // stderr is a stronger signal
        }
        if self.exit_code != 0 {
            score += 0.3;
        }
        if self.success {
            score += 0.2;
        }
        score.min(1.0)
    }
}

/// A file change detected during terminal observation
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub lines_added: usize,
    pub lines_removed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

/// ECHO feature vector extracted from terminal observation
#[derive(Debug, Clone)]
pub struct EchoFeatures {
    pub stdout_entropy: f64,
    pub stderr_entropy: f64,
    pub error_density: f64,
    pub action_success_rate: f64,
    pub file_change_count: usize,
    pub lines_changed: usize,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub signal_strength: f64,
}

impl EchoFeatures {
    pub fn extract(obs: &TerminalObservation) -> Self {
        Self {
            stdout_entropy: Self::shannon_entropy(&obs.stdout),
            stderr_entropy: Self::shannon_entropy(&obs.stderr),
            error_density: if obs.stderr.is_empty() { 0.0 }
                else { (obs.stderr.len() as f64 / obs.stdout.len().max(1) as f64).min(1.0) },
            action_success_rate: if obs.success { 1.0 } else { 0.0 },
            file_change_count: obs.file_changes.len(),
            lines_changed: obs.file_changes.iter().map(|f| f.lines_added + f.lines_removed).sum(),
            exit_code: obs.exit_code,
            duration_ms: obs.duration_ms,
            signal_strength: obs.signal_strength(),
        }
    }

    /// Shannon entropy of a string (byte-level)
    fn shannon_entropy(data: &str) -> f64 {
        if data.is_empty() { return 0.0; }
        let bytes = data.as_bytes();
        let len = bytes.len() as f64;
        let mut counts = [0usize; 256];
        for &b in bytes {
            counts[b as usize] = counts[b as usize].saturating_add(1);
        }
        let mut entropy = 0.0;
        for &count in counts.iter() {
            if count == 0 { continue; }
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
        entropy
    }

    /// Composite signal score for PRM integration
    pub fn composite_score(&self) -> f64 {
        let mut score = 0.0;
        score += self.action_success_rate * 0.35;
        score += (1.0 - self.error_density) * 0.25;
        score += (1.0 - (self.exit_code != 0) as i32 as f64) * 0.20;
        score += (self.signal_strength) * 0.10;
        score += (1.0 - (self.duration_ms as f64 / 30000.0).min(1.0)) * 0.10;
        score.max(0.0).min(1.0)
    }

    /// ECHO cross-entropy target: predict terminal observation from action
    pub fn echo_target(&self) -> f64 {
        self.composite_score()
    }
}

/// ECHO trajectory: sequence of action-observation pairs
#[derive(Debug, Clone)]
pub struct EchoTrajectory {
    pub actions: Vec<String>,
    pub observations: Vec<TerminalObservation>,
    pub features: Vec<EchoFeatures>,
    pub outcome_score: f64,
    pub trajectory_id: String,
}

impl EchoTrajectory {
    pub fn new(trajectory_id: &str) -> Self {
        Self {
            actions: Vec::new(),
            observations: Vec::new(),
            features: Vec::new(),
            outcome_score: 0.0,
            trajectory_id: trajectory_id.to_string(),
        }
    }

    pub fn record_step(&mut self, action: &str, obs: TerminalObservation) {
        self.actions.push(action.to_string());
        let features = EchoFeatures::extract(&obs);
        self.features.push(features);
        self.observations.push(obs);
    }

    /// Compute ECHO score: average of composite + cross-entropy signals
    pub fn compute_echo_score(&self) -> f64 {
        if self.features.is_empty() { return 0.0; }
        let scores: Vec<f64> = self.features.iter()
            .map(|f| f.composite_score())
            .collect();
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let decay = 0.95_f64.powi(self.features.len() as i32);
        mean * (1.0 - decay) + scores.last().copied().unwrap_or(0.0) * decay
    }

    /// Dense supervision targets: one per step (ECHO core idea)
    pub fn echo_targets(&self) -> Vec<f64> {
        self.features.iter().map(|f| f.echo_target()).collect()
    }

    /// Terminal prediction loss: mean squared error between predicted and actual
    pub fn prediction_loss(&self, predicted_scores: &[f64]) -> f64 {
        let targets = self.echo_targets();
        let n = targets.len().min(predicted_scores.len());
        if n == 0 { return 0.0; }
        let mut loss = 0.0;
        for i in 0..n {
            let diff = targets[i] - predicted_scores[i];
            loss += diff * diff;
        }
        loss / n as f64
    }
}

/// ECHO controller — manages terminal observation capture and signal processing
#[derive(Debug, Clone)]
pub struct EchoController {
    pub history: VecDeque<EchoTrajectory>,
    pub max_history: usize,
    pub enable_dense_supervision: bool,
    pub enable_verifier_free: bool,
    pub observation_window: usize,
}

impl Default for EchoController {
    fn default() -> Self {
        Self {
            history: VecDeque::new(),
            max_history: 100,
            enable_dense_supervision: true,
            enable_verifier_free: false,
            observation_window: 10,
        }
    }
}

impl EchoController {
    pub fn new(max_history: usize) -> Self {
        Self { max_history, ..Default::default() }
    }

    /// Record a terminal observation and add to current trajectory
    pub fn record_observation(
        &mut self,
        trajectory_id: &str,
        action: &str,
        obs: TerminalObservation,
    ) {
        let existing = self.history.iter_mut()
            .find(|t: &&mut EchoTrajectory| t.trajectory_id == trajectory_id);
        if let Some(traj) = existing {
            traj.record_step(action, obs);
        } else {
            let mut traj = EchoTrajectory::new(trajectory_id);
            traj.record_step(action, obs);
            self.history.push_back(traj);
            while self.history.len() > self.max_history {
                self.history.pop_front();
            }
        }
    }

    /// Get an ECHO training target for a given trajectory step
    pub fn echo_signal(&self, trajectory_id: &str) -> Option<Vec<f64>> {
        self.history.iter()
            .find(|t| t.trajectory_id == trajectory_id)
            .map(|t| t.echo_targets())
    }

    /// Compute terminal prediction loss for model improvement
    pub fn terminal_prediction_loss(
        &self,
        trajectory_id: &str,
        predicted: &[f64],
    ) -> Option<f64> {
        self.history.iter()
            .find(|t| t.trajectory_id == trajectory_id)
            .map(|t| t.prediction_loss(predicted))
    }

    /// PRM-compatible reward signal from terminal observations
    /// Used to augment the sparse outcome-level reward with dense per-step signals
    pub fn prm_augmented_reward(&self, trajectory_id: &str) -> Option<Vec<f64>> {
        self.history.iter()
            .find(|t| t.trajectory_id == trajectory_id)
            .map(|t| {
                let base = t.echo_targets();
                let final_score = t.compute_echo_score();
                // Blend per-step scores with final outcome
                base.iter().map(|s| s * 0.7 + final_score * 0.3).collect()
            })
    }

    /// Verifier-free self-improvement signal (ECHO §4.3)
    /// When enable_verifier_free is true, the environment prediction loss alone
    /// enables improvement on unseen OOD tasks
    pub fn verifier_free_signal(&self, trajectory_id: &str) -> Option<f64> {
        if !self.enable_verifier_free { return None; }
        self.history.iter()
            .find(|t| t.trajectory_id == trajectory_id)
            .map(|t| {
                let echo_targets = t.echo_targets();
                if echo_targets.is_empty() { return 0.0; }
                let mean = echo_targets.iter().sum::<f64>() / echo_targets.len() as f64;
                let variance = echo_targets.iter()
                    .map(|s| (s - mean).powi(2))
                    .sum::<f64>() / echo_targets.len() as f64;
                // Higher variance = more learning potential
                (mean * 0.6 + variance.sqrt() * 0.4).max(0.0).min(1.0)
            })
    }

    /// Clear old trajectories, keep the most recent n
    pub fn truncate(&mut self, keep: usize) {
        while self.history.len() > keep {
            self.history.pop_front();
        }
    }

    pub fn total_trajectories(&self) -> usize { self.history.len() }
    pub fn total_observations(&self) -> usize {
        self.history.iter().map(|t| t.observations.len()).sum()
    }

    /// Generate a terminal observation from command output
    pub fn observe_command(
        command: &str,
        stdout: &str,
        stderr: &str,
        exit_code: i32,
        duration_ms: u64,
        success: bool,
    ) -> TerminalObservation {
        TerminalObservation {
            timestamp: Instant::now(),
            command: command.to_string(),
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            exit_code,
            duration_ms,
            file_changes: Vec::new(),
            success,
        }
    }

    /// Batch ECHO scoring: compute average signal strength across recent trajectories
    pub fn batch_signal_quality(&self) -> EchoBatchReport {
        let mut report = EchoBatchReport::default();
        for traj in &self.history {
            report.total_trajectories += 1;
            report.total_observations += traj.observations.len();
            let echo_score = traj.compute_echo_score();
            report.avg_echo_score += echo_score;
            report.avg_signal_strength += traj.features.iter()
                .map(|f| f.signal_strength).sum::<f64>() / traj.features.len().max(1) as f64;
            if traj.observations.iter().any(|o| o.exit_code != 0) {
                report.error_trajectories += 1;
            }
        }
        let n = self.history.len().max(1);
        report.avg_echo_score /= n as f64;
        report.avg_signal_strength /= n as f64;
        report.dense_supervision_enabled = self.enable_dense_supervision;
        report
    }
}

/// Batch report for ECHO signal quality monitoring
#[derive(Debug, Clone, Default)]
pub struct EchoBatchReport {
    pub total_trajectories: usize,
    pub total_observations: usize,
    pub avg_echo_score: f64,
    pub avg_signal_strength: f64,
    pub error_trajectories: usize,
    pub dense_supervision_enabled: bool,
}

impl EchoBatchReport {
    pub fn signal_coverage(&self) -> f64 {
        if self.total_observations == 0 { return 0.0; }
        self.total_trajectories as f64 / self.total_observations as f64
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_trajectories == 0 { return 0.0; }
        self.error_trajectories as f64 / self.total_trajectories as f64
    }
}

/// Adapter: ECHO signal → PRM scoring bridge
/// Enables ECHO terminal observations to be used as PRM training data
#[derive(Debug, Clone)]
pub struct EchoPrmBridge {
    pub echo: EchoController,
    pub signal_buffer: VecDeque<(String, f64)>,
    pub max_buffer: usize,
    pub total_signals_count: usize,
}

impl Default for EchoPrmBridge {
    fn default() -> Self {
        Self {
            echo: EchoController::default(),
            signal_buffer: VecDeque::new(),
            max_buffer: 1000,
            total_signals_count: 0,
        }
    }
}

impl EchoPrmBridge {
    pub fn new() -> Self { Self::default() }

    /// Record an observation and generate PRM-augmented reward
    pub fn record_and_reward(
        &mut self,
        trajectory_id: &str,
        action: &str,
        obs: TerminalObservation,
    ) -> f64 {
        self.echo.record_observation(trajectory_id, action, obs);
        let reward = self.echo.prm_augmented_reward(trajectory_id)
            .and_then(|r| r.last().copied())
            .unwrap_or(0.0);
        self.total_signals_count += 1;
        self.signal_buffer.push_back((trajectory_id.to_string(), reward));
        while self.signal_buffer.len() > self.max_buffer {
            self.signal_buffer.pop_front();
        }
        reward
    }

    /// Get the average recent ECHO reward
    pub fn avg_recent_reward(&self, n: usize) -> f64 {
        let recent: Vec<&f64> = self.signal_buffer.iter().rev()
            .take(n).map(|(_, r)| r).collect();
        if recent.is_empty() { return 0.0; }
        recent.iter().copied().sum::<f64>() / recent.len() as f64
    }

    pub fn total_signals(&self) -> usize { self.total_signals_count }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_obs(cmd: &str, stdout: &str, stderr: &str, code: i32, success: bool) -> TerminalObservation {
        EchoController::observe_command(cmd, stdout, stderr, code, 100, success)
    }

    #[test]
    fn test_terminal_observation_creation() {
        let obs = make_obs("ls -la", "total 42\n-rw-r--r-- 1 file.rs", "", 0, true);
        assert_eq!(obs.command, "ls -la");
        assert!(!obs.stdout.is_empty());
        assert!(obs.stderr.is_empty());
        assert_eq!(obs.exit_code, 0);
    }

    #[test]
    fn test_signal_strength_empty() {
        let obs = make_obs("true", "", "", 0, true);
        assert!(obs.signal_strength() > 0.0); // success gives base signal
    }

    #[test]
    fn test_signal_strength_error() {
        let err_obs = make_obs("bad", "", "error: not found", 1, false);
        let ok_obs = make_obs("good", "output", "", 0, true);
        assert!(err_obs.signal_strength() > ok_obs.signal_strength());
    }

    #[test]
    fn test_echo_features_extraction() {
        let obs = make_obs("test", "hello world", "warning: x", 0, true);
        let features = EchoFeatures::extract(&obs);
        assert!(features.stdout_entropy > 0.0);
        assert!(features.error_density >= 0.0);
        assert_eq!(features.exit_code, 0);
    }

    #[test]
    fn test_echo_features_composite_score() {
        let good = make_obs("ok", "output", "", 0, true);
        let bad = make_obs("fail", "", "error", 1, false);
        let good_score = EchoFeatures::extract(&good).composite_score();
        let bad_score = EchoFeatures::extract(&bad).composite_score();
        assert!(good_score > bad_score);
    }

    #[test]
    fn test_shannon_entropy() {
        let e = EchoFeatures::shannon_entropy("aaaa");
        assert!((e - 0.0).abs() < 0.01);
        let e2 = EchoFeatures::shannon_entropy("abcd");
        assert!(e2 > 1.5);
    }

    #[test]
    fn test_echo_trajectory_recording() {
        let mut traj = EchoTrajectory::new("test-1");
        traj.record_step("ls", make_obs("ls", "files", "", 0, true));
        traj.record_step("gcc", make_obs("gcc", "", "error", 1, false));
        assert_eq!(traj.actions.len(), 2);
        assert_eq!(traj.observations.len(), 2);
        assert_eq!(traj.features.len(), 2);
    }

    #[test]
    fn test_echo_trajectory_compute_score() {
        let mut traj = EchoTrajectory::new("test-2");
        traj.record_step("ok", make_obs("ok", "success", "", 0, true));
        let score = traj.compute_echo_score();
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_echo_targets_dense_supervision() {
        let mut traj = EchoTrajectory::new("test-3");
        traj.record_step("a", make_obs("a", "out", "", 0, true));
        traj.record_step("b", make_obs("b", "", "err", 1, false));
        let targets = traj.echo_targets();
        assert_eq!(targets.len(), 2);
        assert!(targets[0] > targets[1]);
    }

    #[test]
    fn test_prediction_loss_perfect() {
        let mut traj = EchoTrajectory::new("test-4");
        traj.record_step("ok", make_obs("ok", "good", "", 0, true));
        let targets = traj.echo_targets();
        let loss = traj.prediction_loss(&targets);
        assert!(loss < 0.01);
    }

    #[test]
    fn test_prediction_loss_wrong() {
        let mut traj = EchoTrajectory::new("test-5");
        traj.record_step("ok", make_obs("ok", "good", "", 0, true));
        let loss = traj.prediction_loss(&[0.0]);
        assert!(loss > 0.1);
    }

    #[test]
    fn test_echo_controller_record_and_retrieve() {
        let mut ctrl = EchoController::default();
        ctrl.record_observation("t1", "ls", make_obs("ls", "out", "", 0, true));
        ctrl.record_observation("t1", "gcc", make_obs("gcc", "", "err", 1, false));
        assert_eq!(ctrl.total_trajectories(), 1);
        assert_eq!(ctrl.total_observations(), 2);
        let signal = ctrl.echo_signal("t1");
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().len(), 2);
    }

    #[test]
    fn test_echo_controller_multiple_trajectories() {
        let mut ctrl = EchoController::default();
        ctrl.record_observation("t1", "a", make_obs("a", "", "", 0, true));
        ctrl.record_observation("t2", "b", make_obs("b", "", "", 0, true));
        assert_eq!(ctrl.total_trajectories(), 2);
    }

    #[test]
    fn test_echo_controller_history_limit() {
        let mut ctrl = EchoController::new(3);
        for i in 0..10 {
            ctrl.record_observation(&format!("t{}", i), "x", make_obs("x", "", "", 0, true));
        }
        assert!(ctrl.total_trajectories() <= 3);
    }

    #[test]
    fn test_prm_augmented_reward() {
        let mut ctrl = EchoController::default();
        ctrl.record_observation("t1", "ok", make_obs("ok", "good", "", 0, true));
        let reward = ctrl.prm_augmented_reward("t1");
        assert!(reward.is_some());
        assert!(!reward.unwrap().is_empty());
    }

    #[test]
    fn test_echo_prm_bridge() {
        let mut bridge = EchoPrmBridge::new();
        let reward = bridge.record_and_reward("t1", "ls",
            make_obs("ls", "files", "", 0, true));
        assert!(reward > 0.0);
        assert_eq!(bridge.total_signals(), 1);
    }

    #[test]
    fn test_echo_prm_bridge_avg_reward() {
        let mut bridge = EchoPrmBridge::new();
        bridge.record_and_reward("t1", "a", make_obs("a", "ok", "", 0, true));
        bridge.record_and_reward("t2", "b", make_obs("b", "", "err", 1, false));
        let avg = bridge.avg_recent_reward(2);
        assert!(avg > 0.0);
    }

    #[test]
    fn test_verifier_free_signal() {
        let mut ctrl = EchoController::default();
        ctrl.enable_verifier_free = true;
        ctrl.record_observation("t1", "a", make_obs("a", "out1", "", 0, true));
        ctrl.record_observation("t1", "b", make_obs("b", "", "err", 1, false));
        let signal = ctrl.verifier_free_signal("t1");
        assert!(signal.is_some());
        assert!(signal.unwrap() > 0.0);
    }

    #[test]
    fn test_verifier_free_signal_disabled() {
        let ctrl = EchoController::default();
        assert!(ctrl.verifier_free_signal("t1").is_none());
    }

    #[test]
    fn test_batch_report() {
        let mut ctrl = EchoController::default();
        ctrl.record_observation("t1", "a", make_obs("a", "out", "", 0, true));
        ctrl.record_observation("t2", "b", make_obs("b", "", "err", 1, false));
        let report = ctrl.batch_signal_quality();
        assert_eq!(report.total_trajectories, 2);
        assert!(report.avg_echo_score > 0.0);
        assert_eq!(report.error_trajectories, 1);
    }

    #[test]
    fn test_batch_report_signal_coverage() {
        let mut ctrl = EchoController::default();
        ctrl.record_observation("t1", "a", make_obs("a", "x", "", 0, true));
        ctrl.record_observation("t1", "b", make_obs("b", "y", "", 0, true));
        let report = ctrl.batch_signal_quality();
        assert!(report.signal_coverage() > 0.0);
    }

    #[test]
    fn test_terminal_prediction_loss() {
        let mut ctrl = EchoController::default();
        ctrl.record_observation("t1", "a", make_obs("a", "out", "", 0, true));
        let loss = ctrl.terminal_prediction_loss("t1", &[0.5]);
        assert!(loss.is_some());
    }

    #[test]
    fn test_echo_controller_truncate() {
        let mut ctrl = EchoController::default();
        for i in 0..5 {
            ctrl.record_observation(&format!("t{}", i), "x", make_obs("x", "", "", 0, true));
        }
        assert_eq!(ctrl.total_trajectories(), 5);
        ctrl.truncate(2);
        assert_eq!(ctrl.total_trajectories(), 2);
    }

    #[test]
    fn test_file_change_tracking() {
        let obs = TerminalObservation {
            timestamp: Instant::now(),
            command: "git diff".into(),
            stdout: "+++ b/src/main.rs".into(),
            stderr: String::new(),
            exit_code: 0,
            duration_ms: 50,
            file_changes: vec![FileChange {
                path: "src/main.rs".into(),
                change_type: ChangeType::Modified,
                lines_added: 5,
                lines_removed: 3,
            }],
            success: true,
        };
        let features = EchoFeatures::extract(&obs);
        assert_eq!(features.file_change_count, 1);
        assert_eq!(features.lines_changed, 8);
    }

    #[test]
    fn test_echo_features_exit_code_penalty() {
        let ok = EchoFeatures::extract(&make_obs("ok", "out", "", 0, true));
        let fail = EchoFeatures::extract(&make_obs("fail", "", "err", 1, false));
        assert!(ok.composite_score() > fail.composite_score());
    }

    #[test]
    fn test_echo_controller_batch_signal_quality_error_rate() {
        let mut ctrl = EchoController::default();
        ctrl.record_observation("t1", "a", make_obs("a", "", "", 0, true));
        ctrl.record_observation("t2", "b", make_obs("b", "", "err", 1, false));
        let report = ctrl.batch_signal_quality();
        assert!((report.error_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_echo_prm_bridge_signal_buffer_limit() {
        let mut bridge = EchoPrmBridge {
            max_buffer: 5,
            ..EchoPrmBridge::default()
        };
        for i in 0..10 {
            bridge.record_and_reward(&format!("t{}", i), "x",
                make_obs("x", "", "", 0, true));
        }
        assert_eq!(bridge.total_signals(), 10);
        assert_eq!(bridge.signal_buffer.len(), 5);
    }
}
