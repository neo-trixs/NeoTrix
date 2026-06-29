use crate::core::nt_core_hex::optimal_starting_mode;
use crate::core::nt_core_hex::ReasoningHexagram;
use rand::Rng;
use std::collections::VecDeque;

const TRAJECTORY_CAPACITY: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub struct ContinuousHexParam {
    pub abstraction: f64,
    pub scope: f64,
    pub method: f64,
    pub depth: f64,
    pub mode: f64,
    pub stance: f64,
}

impl ContinuousHexParam {
    pub fn new() -> Self {
        Self {
            abstraction: 0.5,
            scope: 0.5,
            method: 0.5,
            depth: 0.5,
            mode: 0.5,
            stance: 0.5,
        }
    }

    pub fn from_hexagram(h: ReasoningHexagram) -> Self {
        Self {
            abstraction: h.abstraction() as f64,
            scope: h.scope() as f64,
            method: h.method() as f64,
            depth: h.depth() as f64,
            mode: h.reasoning_mode() as f64,
            stance: h.stance() as f64,
        }
    }

    pub fn to_hexagram(&self) -> ReasoningHexagram {
        let bits = (if self.abstraction >= 0.5 { 1u8 << 5 } else { 0 })
            | (if self.scope >= 0.5 { 1u8 << 4 } else { 0 })
            | (if self.method >= 0.5 { 1u8 << 3 } else { 0 })
            | (if self.depth >= 0.5 { 1u8 << 2 } else { 0 })
            | (if self.mode >= 0.5 { 1u8 << 1 } else { 0 })
            | (if self.stance >= 0.5 { 1u8 } else { 0 });
        ReasoningHexagram(bits)
    }

    pub fn interpolate(&self, other: &Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            abstraction: self.abstraction * (1.0 - t) + other.abstraction * t,
            scope: self.scope * (1.0 - t) + other.scope * t,
            method: self.method * (1.0 - t) + other.method * t,
            depth: self.depth * (1.0 - t) + other.depth * t,
            mode: self.mode * (1.0 - t) + other.mode * t,
            stance: self.stance * (1.0 - t) + other.stance * t,
        }
    }

    pub fn random(entropy: f64) -> Self {
        let mut rng = rand::thread_rng();
        let e = entropy.clamp(0.0, 1.0);
        let mut lerp = |center: f64| -> f64 {
            let offset = rng.gen::<f64>() * 2.0 - 1.0;
            (center * (1.0 - e) + (center + offset * e)).clamp(0.0, 1.0)
        };
        Self {
            abstraction: lerp(0.5),
            scope: lerp(0.5),
            method: lerp(0.5),
            depth: lerp(0.5),
            mode: lerp(0.5),
            stance: lerp(0.5),
        }
    }
}

pub struct VariationalReasoningMode {
    pub base: ContinuousHexParam,
    pub variance: ContinuousHexParam,
    pub temperature: f64,
    pub trajectory: VecDeque<ContinuousHexParam>,
}

impl VariationalReasoningMode {
    pub fn new(base: ReasoningHexagram) -> Self {
        let param = ContinuousHexParam::from_hexagram(base);
        let mut trajectory = VecDeque::with_capacity(TRAJECTORY_CAPACITY);
        trajectory.push_back(param.clone());
        Self {
            base: param,
            variance: ContinuousHexParam::new(),
            temperature: 1.0,
            trajectory,
        }
    }

    pub fn sample(&self) -> ReasoningHexagram {
        let mut rng = rand::thread_rng();
        let mut gaussian = || -> f64 {
            let u: f64 = rng.gen();
            let v: f64 = rng.gen();
            (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
        };

        let mut perturb = |center: f64, var: f64| -> f64 {
            (center + gaussian() * var * self.temperature).clamp(0.0, 1.0)
        };

        let perturbed = ContinuousHexParam {
            abstraction: perturb(self.base.abstraction, self.variance.abstraction),
            scope: perturb(self.base.scope, self.variance.scope),
            method: perturb(self.base.method, self.variance.method),
            depth: perturb(self.base.depth, self.variance.depth),
            mode: perturb(self.base.mode, self.variance.mode),
            stance: perturb(self.base.stance, self.variance.stance),
        };

        perturbed.to_hexagram()
    }

    pub fn update_from_quality(&mut self, quality: f64) {
        let q = quality.clamp(0.0, 1.0);
        let factor = if q >= 0.7 {
            0.9
        } else if q <= 0.3 {
            1.15
        } else {
            1.0 - (q - 0.3) * 0.4
        };
        let clamp = |v: f64| v.clamp(0.0, 1.0);
        self.variance.abstraction = clamp(self.variance.abstraction * factor);
        self.variance.scope = clamp(self.variance.scope * factor);
        self.variance.method = clamp(self.variance.method * factor);
        self.variance.depth = clamp(self.variance.depth * factor);
        self.variance.mode = clamp(self.variance.mode * factor);
        self.variance.stance = clamp(self.variance.stance * factor);
    }

    pub fn trajectory_entropy(&self) -> f64 {
        if self.trajectory.len() < 2 {
            return 0.0;
        }
        let mut counts: Vec<u32> = vec![0u32; 64];
        for param in &self.trajectory {
            let h = param.to_hexagram();
            counts[h.0 as usize] += 1;
        }
        let total = self.trajectory.len() as f64;
        let mut entropy = 0.0f64;
        for &c in &counts {
            if c > 0 {
                let p = c as f64 / total;
                entropy -= p * p.log2();
            }
        }
        let max_entropy = (total.min(64.0)).log2();
        if max_entropy > 0.0 {
            (entropy / max_entropy).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

pub fn optimal_continuous_starting_mode(task: &str) -> ContinuousHexParam {
    ContinuousHexParam::from_hexagram(optimal_starting_mode(task))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hexagram_roundtrip() {
        for bits in 0..64u8 {
            let h = ReasoningHexagram(bits);
            let param = ContinuousHexParam::from_hexagram(h);
            let h2 = param.to_hexagram();
            assert_eq!(h, h2, "roundtrip failed for hexagram {}", bits);
        }
    }

    #[test]
    fn test_interpolation() {
        let low = ContinuousHexParam {
            abstraction: 0.0,
            scope: 0.0,
            method: 0.0,
            depth: 0.0,
            mode: 0.0,
            stance: 0.0,
        };
        let high = ContinuousHexParam {
            abstraction: 1.0,
            scope: 1.0,
            method: 1.0,
            depth: 1.0,
            mode: 1.0,
            stance: 1.0,
        };

        let t0 = low.interpolate(&high, 0.0);
        assert!((t0.abstraction - 0.0).abs() < 1e-10);
        assert!((t0.scope - 0.0).abs() < 1e-10);
        assert!((t0.method - 0.0).abs() < 1e-10);

        let t05 = low.interpolate(&high, 0.5);
        assert!((t05.abstraction - 0.5).abs() < 1e-10);

        let t1 = low.interpolate(&high, 1.0);
        assert!((t1.abstraction - 1.0).abs() < 1e-10);
        assert!((t1.stance - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sample_produces_valid_hexagram() {
        let h = ReasoningHexagram(0);
        let vrm = VariationalReasoningMode::new(h);
        for _ in 0..100 {
            let sampled = vrm.sample();
            assert!(
                sampled.0 < 64,
                "sampled hexagram out of range: {}",
                sampled.0
            );
        }
    }

    #[test]
    fn test_variance_updates_with_quality() {
        let h = ReasoningHexagram(42);
        let mut vrm = VariationalReasoningMode::new(h);
        vrm.variance.abstraction = 0.5;
        vrm.variance.scope = 0.5;
        vrm.variance.method = 0.5;
        vrm.variance.depth = 0.5;
        vrm.variance.mode = 0.5;
        vrm.variance.stance = 0.5;

        vrm.update_from_quality(0.9);
        assert!(
            vrm.variance.abstraction < 0.5,
            "high quality should reduce variance"
        );

        let low = vrm.variance.abstraction;
        vrm.update_from_quality(0.1);
        assert!(
            vrm.variance.abstraction > low,
            "low quality should increase variance"
        );
    }

    #[test]
    fn test_trajectory_tracks_recent_states() {
        let h = ReasoningHexagram(0);
        let mut vrm = VariationalReasoningMode::new(h);
        assert_eq!(vrm.trajectory.len(), 1);

        let distinct = [
            ReasoningHexagram(0),
            ReasoningHexagram(1),
            ReasoningHexagram(2),
            ReasoningHexagram(3),
        ];
        for &hex in &distinct {
            let param = ContinuousHexParam::from_hexagram(hex);
            vrm.trajectory.push_back(param);
        }
        assert_eq!(vrm.trajectory.len(), 5);

        let entropy = vrm.trajectory_entropy();
        assert!(entropy > 0.0, "entropy should be > 0 with distinct states");
        assert!(entropy <= 1.0, "entropy should be <= 1.0");
    }

    #[test]
    fn test_continuous_hex_param_new_default() {
        let p = ContinuousHexParam::new();
        assert!((p.abstraction - 0.5).abs() < 1e-10);
        assert!((p.scope - 0.5).abs() < 1e-10);
        assert!((p.method - 0.5).abs() < 1e-10);
        assert!((p.depth - 0.5).abs() < 1e-10);
        assert!((p.mode - 0.5).abs() < 1e-10);
        assert!((p.stance - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_optimal_continuous_starting_mode_is_valid() {
        let param = optimal_continuous_starting_mode("fix this crash bug");
        let hex = param.to_hexagram();
        assert!(hex.0 < 64);
    }
}
