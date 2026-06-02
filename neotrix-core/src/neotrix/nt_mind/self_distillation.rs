/// JSD (Jensen-Shannon Divergence) between two probability distributions
pub fn js_divergence(p: &[f64], q: &[f64]) -> f64 {
    let mut kl_pm = 0.0;
    let mut kl_qm = 0.0;
    for i in 0..p.len().min(q.len()) {
        let m = (p[i] + q[i]) / 2.0;
        if p[i] > 0.0 && m > 0.0 { kl_pm += p[i] * (p[i] / m).ln(); }
        if q[i] > 0.0 && m > 0.0 { kl_qm += q[i] * (q[i] / m).ln(); }
    }
    0.5 * kl_pm + 0.5 * kl_qm
}

/// Token-level KL divergence with clipping to stabilize training
pub fn kl_divergence(p: &[f64], q: &[f64]) -> f64 {
    p.iter().zip(q.iter())
        .filter(|(&pi, _)| pi > 0.0)
        .map(|(&pi, &qi)| pi * (pi / qi.max(1e-10)).ln())
        .sum()
}

/// Configuration for on-policy self-distillation
pub struct DistillationConfig {
    pub beta: f64,
    pub token_kl_clip: f64,
    pub max_completion_length: usize,
    pub fixed_teacher: bool,
    pub reason_first: bool,
    pub learning_rate: f64,
}

impl Default for DistillationConfig {
    fn default() -> Self {
        Self {
            beta: 0.5,
            token_kl_clip: 0.05,
            max_completion_length: 1024,
            fixed_teacher: true,
            reason_first: false,
            learning_rate: 0.01,
        }
    }
}

/// Training state for on-policy self-distillation
pub struct SelfDistillationTrainer {
    pub config: DistillationConfig,
    pub student_logits: Vec<Vec<f64>>,
    pub teacher_logits: Vec<Vec<f64>>,
    pub step: u64,
    pub total_loss: f64,
    best_loss: f64,
    plateau_steps: u64,
}

impl SelfDistillationTrainer {
    pub fn new(config: DistillationConfig) -> Self {
        Self {
            config,
            student_logits: Vec::new(),
            teacher_logits: Vec::new(),
            step: 0,
            total_loss: 0.0,
            best_loss: f64::MAX,
            plateau_steps: 0,
        }
    }

    /// Feed a batch of token logits from student and teacher
    pub fn feed(&mut self, student: Vec<f64>, teacher: Vec<f64>) {
        self.student_logits.push(student);
        self.teacher_logits.push(teacher);
    }

    /// Compute JSD loss with token-level KL clipping
    pub fn compute_loss(&mut self) -> f64 {
        let mut total = 0.0;
        let mut count = 0;

        for (s, t) in self.student_logits.iter().zip(self.teacher_logits.iter()) {
            let raw_kl = kl_divergence(s, t);
            let clipped = raw_kl.min(self.config.token_kl_clip);
            total += clipped;
            count += 1;
        }

        if count > 0 {
            let avg = total / count as f64;
            self.total_loss = self.total_loss * 0.9 + avg * 0.1;
        }

        self.student_logits.clear();
        self.teacher_logits.clear();
        self.total_loss
    }

    /// Mix student and teacher distribution using JSD beta parameter
    pub fn mixed_distribution(student: &[f64], teacher: &[f64], beta: f64) -> Vec<f64> {
        student.iter().zip(teacher.iter())
            .map(|(&s, &t)| (1.0 - beta) * s + beta * t)
            .collect()
    }

    /// Check if training has plateaued (for early stopping)
    pub fn is_plateaued(&mut self, window: u64) -> bool {
        if self.total_loss < self.best_loss {
            self.best_loss = self.total_loss;
            self.plateau_steps = 0;
        } else {
            self.plateau_steps += 1;
        }
        self.plateau_steps >= window
    }

    /// Reset trainer state for a new distillation session
    pub fn reset(&mut self) {
        self.student_logits.clear();
        self.teacher_logits.clear();
        self.step = 0;
        self.total_loss = 0.0;
    }

    pub fn step_count(&self) -> u64 { self.step }
    pub fn avg_loss(&self) -> f64 { self.total_loss }

    /// Simulate a training step: student generates, teacher scores
    pub fn train_step(&mut self, student_output: &[f64], teacher_output: &[f64]) -> f64 {
        self.step += 1;
        let loss = js_divergence(student_output, teacher_output);
        let clipped = loss.min(self.config.token_kl_clip);
        self.total_loss = self.total_loss * 0.9 + clipped * 0.1;
        clipped
    }
}

/// Fixed-teacher strategy: teacher is frozen at step 0 (via LoRA adapter weights)
pub struct FixedTeacher {
    reference_outputs: Vec<Vec<f64>>,
    frozen: bool,
}

impl FixedTeacher {
    pub fn new() -> Self { Self { reference_outputs: Vec::new(), frozen: false } }
    pub fn freeze(&mut self, outputs: Vec<Vec<f64>>) { self.reference_outputs = outputs; self.frozen = true; }
    pub fn is_frozen(&self) -> bool { self.frozen }
    pub fn reference(&self) -> &[Vec<f64>] { &self.reference_outputs }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_divergence_identical() {
        let p = vec![0.5, 0.3, 0.2];
        let q = vec![0.5, 0.3, 0.2];
        let jsd = js_divergence(&p, &q);
        assert!((jsd - 0.0).abs() < 1e-12, "JSD of identical distributions should be 0, got {}", jsd);
    }

    #[test]
    fn test_js_divergence_different() {
        let p = vec![0.9, 0.05, 0.05];
        let q = vec![0.1, 0.8, 0.1];
        let jsd = js_divergence(&p, &q);
        assert!(jsd > 0.0, "JSD of different distributions should be > 0");
        assert!(jsd < 2.0, "JSD should be bounded for valid distributions");
    }

    #[test]
    fn test_kl_divergence_zero_when_identical() {
        let p = vec![0.2, 0.5, 0.3];
        let q = vec![0.2, 0.5, 0.3];
        let kl = kl_divergence(&p, &q);
        assert!((kl - 0.0).abs() < 1e-12, "KL of identical distributions should be 0, got {}", kl);
    }

    #[test]
    fn test_kl_divergence_positive() {
        let p = vec![0.8, 0.1, 0.1];
        let q = vec![0.3, 0.4, 0.3];
        let kl = kl_divergence(&p, &q);
        assert!(kl > 0.0, "KL of different distributions should be > 0");
    }

    #[test]
    fn test_token_kl_clipping() {
        let p = vec![0.99, 0.01, 0.0];
        let q = vec![0.01, 0.5, 0.49];
        let config = DistillationConfig { token_kl_clip: 0.02, ..Default::default() };
        let mut trainer = SelfDistillationTrainer::new(config);
        let loss = trainer.train_step(&p, &q);
        assert!(loss <= 0.02, "Clipped loss should not exceed token_kl_clip, got {}", loss);
    }

    #[test]
    fn test_feed_and_compute_loss() {
        let mut trainer = SelfDistillationTrainer::new(DistillationConfig::default());
        trainer.feed(vec![0.4, 0.6], vec![0.5, 0.5]);
        trainer.feed(vec![0.7, 0.3], vec![0.6, 0.4]);
        let loss = trainer.compute_loss();
        assert!(loss >= 0.0, "Loss should be non-negative, got {}", loss);
        assert!(trainer.student_logits.is_empty(), "Logits should be cleared after compute_loss");
        assert!(trainer.teacher_logits.is_empty(), "Teacher logits should be cleared after compute_loss");
    }

    #[test]
    fn test_plateau_detection() {
        let mut trainer = SelfDistillationTrainer::new(DistillationConfig::default());
        trainer.total_loss = 0.05;

        // First call: total_loss (0.05) < best_loss (f64::MAX) => not plateaued
        assert!(!trainer.is_plateaued(3));
        assert_eq!(trainer.plateau_steps, 0);

        // Simulate worsening: no improvement
        trainer.total_loss = 0.06;
        assert!(!trainer.is_plateaued(3));
        assert_eq!(trainer.plateau_steps, 1);

        trainer.total_loss = 0.07;
        assert!(!trainer.is_plateaued(3));
        assert_eq!(trainer.plateau_steps, 2);

        trainer.total_loss = 0.08;
        assert!(trainer.is_plateaued(3));
        assert_eq!(trainer.plateau_steps, 3);
    }

    #[test]
    fn test_mixed_distribution() {
        let student = vec![0.8, 0.1, 0.1];
        let teacher = vec![0.2, 0.6, 0.2];
        let beta = 0.7;
        let mixed = SelfDistillationTrainer::mixed_distribution(&student, &teacher, beta);
        assert_eq!(mixed.len(), 3);
        // (1-0.7)*0.8 + 0.7*0.2 = 0.24 + 0.14 = 0.38
        assert!((mixed[0] - 0.38).abs() < 1e-12);
        // (1-0.7)*0.1 + 0.7*0.6 = 0.03 + 0.42 = 0.45
        assert!((mixed[1] - 0.45).abs() < 1e-12);
        // (1-0.7)*0.1 + 0.7*0.2 = 0.03 + 0.14 = 0.17
        assert!((mixed[2] - 0.17).abs() < 1e-12);
    }

    #[test]
    fn test_fixed_teacher_freeze_isolation() {
        let mut teacher = FixedTeacher::new();
        assert!(!teacher.is_frozen());
        let outputs = vec![vec![0.5, 0.5], vec![0.3, 0.7]];
        teacher.freeze(outputs.clone());
        assert!(teacher.is_frozen());
        assert_eq!(teacher.reference(), &outputs);
    }

    #[test]
    fn test_trainer_reset() {
        let mut trainer = SelfDistillationTrainer::new(DistillationConfig::default());
        trainer.feed(vec![1.0, 0.0], vec![0.5, 0.5]);
        trainer.compute_loss();
        trainer.step = 5;
        trainer.reset();
        assert_eq!(trainer.step, 0);
        assert_eq!(trainer.total_loss, 0.0);
        assert!(trainer.student_logits.is_empty());
        assert!(trainer.teacher_logits.is_empty());
    }

    #[test]
    fn test_step_count_and_avg_loss() {
        let mut trainer = SelfDistillationTrainer::new(DistillationConfig::default());
        assert_eq!(trainer.step_count(), 0);
        assert_eq!(trainer.avg_loss(), 0.0);
        trainer.train_step(&[0.6, 0.4], &[0.5, 0.5]);
        assert_eq!(trainer.step_count(), 1);
        assert!(trainer.avg_loss() > 0.0);
    }
}
