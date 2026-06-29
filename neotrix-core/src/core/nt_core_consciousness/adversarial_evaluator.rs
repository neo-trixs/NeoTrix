use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone)]
pub enum JudgeModel {
    SameModel,
    VSAJudge { threshold: f64 },
    ExternalModel { model_id: String },
}

impl Default for JudgeModel {
    fn default() -> Self {
        JudgeModel::SameModel
    }
}

#[derive(Debug, Clone)]
pub struct AdversarialVerdict {
    pub passed: bool,
    pub judge_score: f64,
    pub cross_divergence: f64,
    pub confidence: f64,
    pub reasons: Vec<String>,
    pub pressure_level: PressureLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureLevel {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct AdversarialEvaluator {
    pub judge_model: JudgeModel,
    pub divergence_threshold: f64,
    pub pressure_testing: bool,
    pub current_pressure: PressureLevel,
    pub verdict_history: Vec<AdversarialVerdict>,
    max_history: usize,
    pressure_ramp_success: u64,
    pressure_decay_failure: u64,
}

impl AdversarialEvaluator {
    pub fn new(judge_model: JudgeModel) -> Self {
        Self {
            judge_model,
            divergence_threshold: 0.3,
            pressure_testing: false,
            current_pressure: PressureLevel::None,
            verdict_history: Vec::new(),
            max_history: 100,
            pressure_ramp_success: 3,
            pressure_decay_failure: 2,
        }
    }

    pub fn evaluate(
        &mut self,
        generator_output: &[u8],
        context: &[u8],
        output_text: &str,
    ) -> AdversarialVerdict {
        let _ = output_text;
        let (judge_score, confidence, mut reasons) = match &self.judge_model {
            JudgeModel::SameModel => {
                let sim = QuantizedVSA::similarity(generator_output, context);
                let conf = 0.5 + sim * 0.5;
                (sim, conf, vec![format!("same_model:sim={:.3}", sim)])
            }
            JudgeModel::VSAJudge { threshold } => {
                let sim = QuantizedVSA::similarity(generator_output, context);
                let conf = 0.7 + sim * 0.3;
                let mut r = vec![format!("vsa_judge:sim={:.3}", sim)];
                if sim < *threshold {
                    r.push("below vsa threshold".to_string());
                }
                (sim, conf, r)
            }
            JudgeModel::ExternalModel { model_id } => {
                let sim = QuantizedVSA::similarity(generator_output, context);
                let conf = 0.5;
                (
                    sim,
                    conf,
                    vec![format!("external_model:{}_sim={:.3}", model_id, sim)],
                )
            }
        };

        let cross_divergence = Self::compute_cross_divergence(generator_output, context);

        let passed = match self.current_pressure {
            PressureLevel::None => {
                judge_score >= 0.3 && cross_divergence < self.divergence_threshold
            }
            PressureLevel::Low => {
                judge_score >= 0.4 && cross_divergence < self.divergence_threshold
            }
            PressureLevel::Medium => {
                judge_score >= 0.6 && cross_divergence < self.divergence_threshold * 0.7
            }
            PressureLevel::High => {
                judge_score >= 0.8 && cross_divergence < self.divergence_threshold * 0.5
            }
        };

        if !passed {
            reasons.push(format!("cross_divergence={:.3}", cross_divergence));
        }

        let verdict = AdversarialVerdict {
            passed,
            judge_score,
            cross_divergence,
            confidence,
            reasons,
            pressure_level: self.current_pressure,
        };

        self.verdict_history.push(verdict.clone());
        if self.verdict_history.len() > self.max_history {
            self.verdict_history.remove(0);
        }

        if self.pressure_testing {
            self.step_pressure(&verdict);
        }

        verdict
    }

    pub fn compute_cross_divergence(generator_output: &[u8], judge_output: &[u8]) -> f64 {
        let sim = QuantizedVSA::similarity(generator_output, judge_output);
        1.0 - sim
    }

    pub fn enable_pressure_testing(&mut self) {
        self.pressure_testing = true;
    }

    pub fn disable_pressure_testing(&mut self) {
        self.pressure_testing = false;
    }

    pub fn step_pressure(&mut self, last_verdict: &AdversarialVerdict) {
        if last_verdict.passed {
            let consecutive_passes = self
                .verdict_history
                .iter()
                .rev()
                .take_while(|v| v.passed)
                .count() as u64;
            if consecutive_passes >= self.pressure_ramp_success {
                self.current_pressure = match self.current_pressure {
                    PressureLevel::None => PressureLevel::Low,
                    PressureLevel::Low => PressureLevel::Medium,
                    PressureLevel::Medium => PressureLevel::High,
                    PressureLevel::High => PressureLevel::High,
                };
            }
        } else {
            let consecutive_fails = self
                .verdict_history
                .iter()
                .rev()
                .take_while(|v| !v.passed)
                .count() as u64;
            if consecutive_fails >= self.pressure_decay_failure {
                self.current_pressure = match self.current_pressure {
                    PressureLevel::None => PressureLevel::None,
                    PressureLevel::Low => PressureLevel::None,
                    PressureLevel::Medium => PressureLevel::Low,
                    PressureLevel::High => PressureLevel::Medium,
                };
            }
        }
    }

    pub fn pass_rate(&self) -> f64 {
        if self.verdict_history.is_empty() {
            return 1.0;
        }
        let passed = self.verdict_history.iter().filter(|v| v.passed).count();
        passed as f64 / self.verdict_history.len() as f64
    }

    pub fn average_divergence(&self) -> f64 {
        if self.verdict_history.is_empty() {
            return 0.0;
        }
        let sum: f64 = self
            .verdict_history
            .iter()
            .map(|v| v.cross_divergence)
            .sum();
        sum / self.verdict_history.len() as f64
    }

    pub fn report(&self) -> String {
        format!(
            "AdversarialEvaluator: pass_rate={:.3} avg_div={:.3} pressure={:?} judge={:?} history={}",
            self.pass_rate(),
            self.average_divergence(),
            self.current_pressure,
            self.judge_model,
            self.verdict_history.len(),
        )
    }

    pub fn switch_judge_model(&mut self, model: JudgeModel) {
        self.judge_model = model;
        self.verdict_history.clear();
        self.current_pressure = PressureLevel::None;
    }
}
