use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum LossDimension {
    PredictionError,
    CalibrationLoss,
    NegentropyDecay,
    CoherenceDrop,
    CScoreDecline,
}

#[derive(Debug, Clone)]
pub struct LossSample {
    pub cycle: u64,
    pub timestamp: u64,
    pub dimension: LossDimension,
    pub value: f64,
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub struct CompositeLoss {
    pub prediction_error: f64,
    pub calibration_loss: f64,
    pub negentropy_decay: f64,
    pub coherence_drop: f64,
    pub c_score_decline: f64,
    pub total: f64,
}

pub struct LossFunction {
    pub samples: VecDeque<LossSample>,
    pub window_size: usize,
    pub prediction_ema: f64,
    pub calibration_ema: f64,
    pub composite: CompositeLoss,
    pub last_negentropy: f64,
    pub last_coherence: f64,
    pub last_c_score: f64,
}

impl Default for LossFunction {
    fn default() -> Self {
        Self::new(100)
    }
}

impl LossFunction {
    pub fn new(window_size: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(window_size),
            window_size,
            prediction_ema: 0.0,
            calibration_ema: 0.0,
            composite: CompositeLoss {
                prediction_error: 0.0,
                calibration_loss: 0.0,
                negentropy_decay: 0.0,
                coherence_drop: 0.0,
                c_score_decline: 0.0,
                total: 0.0,
            },
            last_negentropy: 0.0,
            last_coherence: 0.0,
            last_c_score: 0.0,
        }
    }

    fn push_sample(&mut self, dimension: LossDimension, value: f64, weight: f64) {
        if self.samples.len() >= self.window_size {
            self.samples.pop_front();
        }
        self.samples.push_back(LossSample {
            cycle: 0,
            timestamp: 0,
            dimension,
            value,
            weight,
        });
    }

    pub fn record_prediction_error(&mut self, error: f64) {
        self.push_sample(LossDimension::PredictionError, error, 0.3);
        self.prediction_ema = self.prediction_ema * 0.9 + error * 0.1;
    }

    pub fn record_calibration_error(&mut self, ece: f64) {
        self.push_sample(LossDimension::CalibrationLoss, ece, 0.25);
        self.calibration_ema = self.calibration_ema * 0.9 + ece * 0.1;
    }

    pub fn record_negentropy_change(&mut self, delta: f64) {
        if delta < 0.0 {
            let loss = delta.abs();
            self.push_sample(LossDimension::NegentropyDecay, loss, 0.2);
        }
    }

    pub fn record_coherence_change(&mut self, delta: f64) {
        if delta < 0.0 {
            let loss = delta.abs();
            self.push_sample(LossDimension::CoherenceDrop, loss, 0.15);
        }
    }

    pub fn record_c_score_change(&mut self, delta: f64) {
        if delta < 0.0 {
            let loss = delta.abs();
            self.push_sample(LossDimension::CScoreDecline, loss, 0.1);
        }
    }

    pub fn compute(&mut self) -> CompositeLoss {
        let prediction_weight = 0.3;
        let calibration_weight = 0.25;
        let negentropy_weight = 0.2;
        let coherence_weight = 0.15;
        let c_score_weight = 0.1;

        let ne = self.composite.negentropy_decay;
        let co = self.composite.coherence_drop;
        let cs = self.composite.c_score_decline;

        let total = self.prediction_ema * prediction_weight
            + self.calibration_ema * calibration_weight
            + ne * negentropy_weight
            + co * coherence_weight
            + cs * c_score_weight;

        self.composite = CompositeLoss {
            prediction_error: self.prediction_ema,
            calibration_loss: self.calibration_ema,
            negentropy_decay: ne,
            coherence_drop: co,
            c_score_decline: cs,
            total,
        };
        self.composite.clone()
    }

    pub fn stats(&self) -> LossStats {
        LossStats {
            samples: self.samples.len(),
            prediction_ema: self.prediction_ema,
            calibration_ema: self.calibration_ema,
            total_loss: self.composite.total,
        }
    }

    pub fn reset(&mut self) {
        self.samples.clear();
        self.prediction_ema = 0.0;
        self.calibration_ema = 0.0;
        self.composite = CompositeLoss {
            prediction_error: 0.0,
            calibration_loss: 0.0,
            negentropy_decay: 0.0,
            coherence_drop: 0.0,
            c_score_decline: 0.0,
            total: 0.0,
        };
        self.last_negentropy = 0.0;
        self.last_coherence = 0.0;
        self.last_c_score = 0.0;
    }
}

pub struct LossStats {
    pub samples: usize,
    pub prediction_ema: f64,
    pub calibration_ema: f64,
    pub total_loss: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_prediction_error_updates_ema() {
        let mut lf = LossFunction::new(10);
        lf.record_prediction_error(0.5);
        assert!((lf.prediction_ema - 0.05).abs() < 1e-6);
        lf.record_prediction_error(1.0);
        assert!((lf.prediction_ema - 0.145).abs() < 1e-6);
    }

    #[test]
    fn test_compute_returns_non_zero_composite() {
        let mut lf = LossFunction::new(10);
        lf.record_prediction_error(0.8);
        lf.record_calibration_error(0.3);
        lf.record_negentropy_change(-0.2);
        lf.record_coherence_change(-0.1);
        lf.record_c_score_change(-0.05);
        let composite = lf.compute();
        assert!(composite.total > 0.0);
        assert!((composite.prediction_error - 0.08).abs() < 1e-6);
    }

    #[test]
    fn test_reset_clears_all_state() {
        let mut lf = LossFunction::new(10);
        lf.record_prediction_error(0.5);
        lf.record_calibration_error(0.2);
        lf.compute();
        assert!(lf.composite.total > 0.0);
        lf.reset();
        assert_eq!(lf.samples.len(), 0);
        assert_eq!(lf.prediction_ema, 0.0);
        assert_eq!(lf.calibration_ema, 0.0);
        assert_eq!(lf.composite.total, 0.0);
    }

    #[test]
    fn test_multiple_dimensions_contribute_to_total() {
        let mut lf = LossFunction::new(10);
        lf.record_prediction_error(1.0);
        lf.record_calibration_error(1.0);
        lf.record_negentropy_change(-1.0);
        lf.record_coherence_change(-1.0);
        lf.record_c_score_change(-1.0);
        let composite = lf.compute();
        // prediction_ema = 0.1 (after one step with α=0.1)
        // calibration_ema = 0.1
        // negentropy_decay = 1.0, coherence_drop = 1.0, c_score_decline = 1.0
        // total = 0.1*0.3 + 0.1*0.25 + 1.0*0.2 + 1.0*0.15 + 1.0*0.1 = 0.03+0.025+0.2+0.15+0.1 = 0.505
        assert!((composite.total - 0.505).abs() < 1e-6);
    }

    #[test]
    fn test_stats_returns_correct_values() {
        let mut lf = LossFunction::new(10);
        lf.record_prediction_error(0.3);
        lf.record_calibration_error(0.1);
        lf.compute();
        let s = lf.stats();
        assert_eq!(s.samples, 2);
        assert!((s.total_loss - lf.composite.total).abs() < 1e-6);
    }

    #[test]
    fn test_positive_deltas_do_not_record_loss() {
        let mut lf = LossFunction::new(10);
        lf.record_negentropy_change(0.5);
        assert_eq!(lf.composite.negentropy_decay, 0.0);
        lf.record_coherence_change(0.3);
        assert_eq!(lf.composite.coherence_drop, 0.0);
        lf.record_c_score_change(0.2);
        assert_eq!(lf.composite.c_score_decline, 0.0);
    }
}
