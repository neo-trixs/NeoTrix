#![forbid(unsafe_code)]

//! Decay curve implementations (R-P121).
//!
//! Two standard curves:
//! - Ebbinghaus power-law forgetting curve (human-like, steep initial drop)
//! - Simple exponential decay (faster, commonly used in CS)

/// Ebbinghaus retention curve: `R = e^(-t / (s * half_life))`
/// where `s` is a stability factor derived from half-life.
///
/// Returns a value in [0.0, 1.0] representing retention.
pub fn ebbinghaus_retention(elapsed_hours: f64, half_life_hours: f64) -> f64 {
    if elapsed_hours <= 0.0 {
        return 1.0;
    }
    let half_life = half_life_hours.max(0.01);
    // Power-law approximation: R = 1 / (1 + t/c) where c is derived from half-life
    // At t = half_life: R = 0.5 → c = half_life
    let ratio = elapsed_hours / half_life;
    (1.0 / (1.0 + ratio)).clamp(0.0, 1.0)
}

/// Simple exponential decay: `R = e^(-lambda * t)`
///
/// Returns a value in [0.0, 1.0] representing retention.
pub fn exponential_decay(elapsed_hours: f64, lambda: f64) -> f64 {
    if elapsed_hours <= 0.0 {
        return 1.0;
    }
    (-lambda * elapsed_hours).exp().clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ebbinghaus_zero_time_returns_one() {
        assert!((ebbinghaus_retention(0.0, 24.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ebbinghaus_at_half_life() {
        let r = ebbinghaus_retention(24.0, 24.0);
        assert!((r - 0.5).abs() < 0.01, "retention at half-life = {r}");
    }

    #[test]
    fn ebbinghaus_monotonic_decrease() {
        let r1 = ebbinghaus_retention(10.0, 24.0);
        let r2 = ebbinghaus_retention(24.0, 24.0);
        let r3 = ebbinghaus_retention(100.0, 24.0);
        assert!(r1 > r2);
        assert!(r2 > r3);
    }

    #[test]
    fn ebbinghaus_bounded() {
        for t in [0.0, 1.0, 24.0, 168.0, 720.0] {
            let r = ebbinghaus_retention(t, 24.0);
            assert!((0.0..=1.0).contains(&r), "retention={r} at {t}h");
        }
    }

    #[test]
    fn exponential_zero_time_returns_one() {
        assert!((exponential_decay(0.0, 0.028) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn exponential_at_half_life() {
        let lambda = 0.693_147_180_559_945_3 / 24.0; // half-life = 24h
        let r = exponential_decay(24.0, lambda);
        assert!((r - 0.5).abs() < 1e-6, "retention at half-life = {r}");
    }

    #[test]
    fn exponential_monotonic_decrease() {
        let lambda = 0.03;
        let r1 = exponential_decay(10.0, lambda);
        let r2 = exponential_decay(24.0, lambda);
        let r3 = exponential_decay(100.0, lambda);
        assert!(r1 > r2);
        assert!(r2 > r3);
    }

    #[test]
    fn exponential_approaches_zero() {
        let r = exponential_decay(1000.0, 0.1);
        assert!(r < 1e-4);
    }

    #[test]
    fn exponential_clamped_below_one() {
        // Negative elapsed should return 1.0
        assert_eq!(exponential_decay(-5.0, 0.1), 1.0);
    }

    #[test]
    fn ebbinghaus_zero_half_life() {
        let r = ebbinghaus_retention(10.0, 0.0);
        assert!(r >= 0.0 && r <= 1.0);
    }

    #[test]
    fn exponential_zero_lambda_returns_one() {
        let r = exponential_decay(100.0, 0.0);
        assert!((r - 1.0).abs() < 1e-6);
    }
}
