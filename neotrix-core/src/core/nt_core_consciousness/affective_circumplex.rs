/// Maps affective state to generation parameters
#[derive(Debug, Clone)]
pub struct AffectiveCircumplex {
    /// Temperature modulation range (min, max), default (0.3, 1.5)
    pub temp_range: (f64, f64),
    /// Top-p modulation range (min, max), default (0.8, 1.0)
    pub top_p_range: (f64, f64),
    /// Repetition penalty modulation range (min, max), default (1.0, 1.3)
    pub rep_penalty_range: (f64, f64),
}

impl Default for AffectiveCircumplex {
    fn default() -> Self {
        Self::new()
    }
}

impl AffectiveCircumplex {
    pub fn new() -> Self {
        Self {
            temp_range: (0.3, 1.5),
            top_p_range: (0.8, 1.0),
            rep_penalty_range: (1.0, 1.3),
        }
    }

    /// High valence + high arousal → higher temperature (more creative/diverse).
    pub fn modulate_temperature(&self, valence: f64, arousal: f64) -> f64 {
        let v = valence.clamp(-1.0, 1.0);
        let a = arousal.clamp(0.0, 1.0);
        let ratio = (v * 0.5 + 0.5) * 0.6 + a * 0.4;
        self.temp_range.0 + ratio * (self.temp_range.1 - self.temp_range.0)
    }

    /// Low confidence → lower top_p (more focused sampling).
    pub fn modulate_top_p(&self, confidence: f64) -> f64 {
        let c = confidence.clamp(0.0, 1.0);
        self.top_p_range.0 + c * (self.top_p_range.1 - self.top_p_range.0)
    }

    /// High curiosity → higher repetition penalty (avoid repeating).
    pub fn modulate_rep_penalty(&self, curiosity: f64) -> f64 {
        let c = curiosity.clamp(0.0, 1.0);
        self.rep_penalty_range.0 + c * (self.rep_penalty_range.1 - self.rep_penalty_range.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ranges() {
        let c = AffectiveCircumplex::new();
        assert_eq!(c.temp_range, (0.3, 1.5));
        assert_eq!(c.top_p_range, (0.8, 1.0));
        assert_eq!(c.rep_penalty_range, (1.0, 1.3));
    }

    #[test]
    fn test_modulate_temperature_high() {
        let c = AffectiveCircumplex::new();
        let t = c.modulate_temperature(1.0, 1.0);
        assert!(t >= c.temp_range.0 && t <= c.temp_range.1);
        assert!(t > c.temp_range.0);
    }

    #[test]
    fn test_modulate_temperature_low() {
        let c = AffectiveCircumplex::new();
        let t = c.modulate_temperature(-1.0, 0.0);
        assert!((t - c.temp_range.0).abs() < 0.01);
    }

    #[test]
    fn test_modulate_top_p_high_confidence() {
        let c = AffectiveCircumplex::new();
        let p = c.modulate_top_p(1.0);
        assert!((p - c.top_p_range.1).abs() < 0.01);
    }

    #[test]
    fn test_modulate_top_p_low_confidence() {
        let c = AffectiveCircumplex::new();
        let p = c.modulate_top_p(0.0);
        assert!((p - c.top_p_range.0).abs() < 0.01);
    }

    #[test]
    fn test_modulate_rep_penalty_high_curiosity() {
        let c = AffectiveCircumplex::new();
        let r = c.modulate_rep_penalty(1.0);
        assert!((r - c.rep_penalty_range.1).abs() < 0.01);
    }

    #[test]
    fn test_modulate_rep_penalty_low_curiosity() {
        let c = AffectiveCircumplex::new();
        let r = c.modulate_rep_penalty(0.0);
        assert!((r - c.rep_penalty_range.0).abs() < 0.01);
    }

    #[test]
    fn test_all_ranges_respected() {
        let c = AffectiveCircumplex::new();
        for v in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            for a in [0.0, 0.5, 1.0] {
                let t = c.modulate_temperature(v, a);
                assert!(t >= c.temp_range.0 - 0.001 && t <= c.temp_range.1 + 0.001);
            }
        }
    }
}
