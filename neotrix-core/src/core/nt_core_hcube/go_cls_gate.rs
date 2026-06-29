#[derive(Debug, Clone)]
pub struct GoClsGate {
    pub transfer_threshold: f64,
    pub generalization_score: f64,
    pub memorization_score: f64,
}

impl GoClsGate {
    pub fn new() -> Self {
        Self {
            transfer_threshold: 0.6,
            generalization_score: 0.0,
            memorization_score: 0.0,
        }
    }
    pub fn should_transfer(&self) -> bool {
        self.generalization_score > self.transfer_threshold
    }
    pub fn record(&mut self, gen_gain: f64, mem_gain: f64) {
        self.generalization_score = self.generalization_score * 0.9 + gen_gain * 0.1;
        self.memorization_score = self.memorization_score * 0.9 + mem_gain * 0.1;
    }
}
