use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CuriosityRecord {
    pub prediction_error: f64,
    pub novelty: f64,
    pub curiosity_drive: f64,
    pub timestamp: u64,
}

pub struct CuriosityEngine {
    history: VecDeque<CuriosityRecord>,
    max_history: usize,
    curiosity_temperature: f64,
    exploration_rate: f64,
    total_curiosity: f64,
}

impl CuriosityEngine {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(256),
            max_history: 256,
            curiosity_temperature: 1.0,
            exploration_rate: 0.1,
            total_curiosity: 0.0,
        }
    }

    pub fn observe(&mut self, prediction_error: f64, novelty: f64, timestamp: u64) {
        let curiosity_drive = self.compute_curiosity(prediction_error, novelty);
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(CuriosityRecord {
            prediction_error,
            novelty,
            curiosity_drive,
            timestamp,
        });
        self.total_curiosity += curiosity_drive;
    }

    pub fn compute_curiosity(&self, prediction_error: f64, novelty: f64) -> f64 {
        let raw = prediction_error * novelty * self.curiosity_temperature;
        let decay = 1.0 / (1.0 + self.history.len() as f64 * 0.01);
        (raw * decay).clamp(0.0, 1.0)
    }

    pub fn free_energy(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let recent: Vec<_> = self.history.iter().rev().take(30).collect();
        let mean_pe: f64 = recent.iter().map(|r| r.prediction_error).sum::<f64>() / recent.len() as f64;
        let entropy: f64 = recent.iter()
            .map(|r| {
                let p = r.novelty.clamp(1e-10, 1.0 - 1e-10);
                -p * p.ln() - (1.0 - p) * (1.0 - p).ln()
            })
            .sum::<f64>() / recent.len() as f64;
        (mean_pe + entropy * 0.5).clamp(0.0, 2.0)
    }

    pub fn exploration_urge(&self) -> f64 {
        let fe = self.free_energy();
        let boredom = if self.history.len() > 20 {
            let recent: Vec<_> = self.history.iter().rev().take(20).collect();
            let mean_cd: f64 = recent.iter().map(|r| r.curiosity_drive).sum::<f64>() / 20.0;
            1.0 - mean_cd
        } else {
            1.0
        };
        ((fe * 0.6 + boredom * 0.4) * self.exploration_rate).clamp(0.0, 1.0)
    }

    pub fn set_temperature(&mut self, t: f64) {
        self.curiosity_temperature = t.clamp(0.1, 5.0);
    }

    pub fn set_exploration_rate(&mut self, rate: f64) {
        self.exploration_rate = rate.clamp(0.0, 1.0);
    }

    pub fn total_curiosity(&self) -> f64 {
        self.total_curiosity
    }

    pub fn recent_curiosity(&self) -> f64 {
        self.history.back().map(|r| r.curiosity_drive).unwrap_or(0.0)
    }
}

impl Default for CuriosityEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curiosity_initial_state() {
        let c = CuriosityEngine::new();
        assert!((c.free_energy() - 0.0).abs() < 1e-6);
        let urge = c.exploration_urge();
        assert!(urge >= 0.0 && urge <= 1.0);
    }

    #[test]
    fn test_curiosity_accumulates() {
        let mut c = CuriosityEngine::new();
        for i in 0..10 {
            c.observe(0.8, 0.5, i as u64);
        }
        assert!(c.total_curiosity() > 0.0);
        assert!(c.free_energy() > 0.0);
    }

    #[test]
    fn test_curiosity_drive_clamped() {
        let c = CuriosityEngine::new();
        let drive = c.compute_curiosity(100.0, 100.0);
        assert!(drive <= 1.0);
        assert!(drive >= 0.0);
    }
}
