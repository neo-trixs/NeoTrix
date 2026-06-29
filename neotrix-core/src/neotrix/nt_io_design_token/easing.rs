use super::physics::{SpringParams, SpringSimulation};

#[derive(Debug, Clone)]
pub enum EasingCurve {
    CubicBezier { x1: f64, y1: f64, x2: f64, y2: f64 },
    Spring(SpringParams),
    Linear,
}

impl EasingCurve {
    pub fn sample(&self, t: f64) -> f64 {
        match self {
            EasingCurve::CubicBezier { x1, y1, x2, y2 } => {
                sample_cubic_bezier(t, *x1, *y1, *x2, *y2)
            }
            EasingCurve::Spring(params) => {
                let mut sim = SpringSimulation::new(params.clone());
                let _ = sim.simulate(1.0, 0.008);
                sim.position_at(t)
            }
            EasingCurve::Linear => t,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            EasingCurve::CubicBezier { .. } => "cubic-bezier",
            EasingCurve::Spring(_) => "spring",
            EasingCurve::Linear => "linear",
        }
    }
}

fn sample_cubic_bezier(t: f64, _x1: f64, y1: f64, _x2: f64, y2: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let output = 0.0 * mt3 + 3.0 * y1 * mt2 * t + 3.0 * y2 * mt * t2 + 1.0 * t3;
    output.clamp(0.0, 1.0)
}

impl Default for EasingCurve {
    fn default() -> Self {
        EasingCurve::CubicBezier {
            x1: 0.4,
            y1: 0.0,
            x2: 0.2,
            y2: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EntranceType {
    FadeIn,
    Rise,
    BlurClear,
    ScaleIn,
    SlideIn { from_x: f64, from_y: f64 },
}

#[derive(Debug, Clone)]
pub struct EntranceAnimation {
    pub entrance_type: EntranceType,
    pub easing: EasingCurve,
    pub duration_ms: u32,
    pub delay_ms: u32,
    pub rise_distance: f64,
    pub blur_radius: f64,
}

impl Default for EntranceAnimation {
    fn default() -> Self {
        EntranceAnimation {
            entrance_type: EntranceType::FadeIn,
            easing: EasingCurve::CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 0.2,
                y2: 1.0,
            },
            duration_ms: 300,
            delay_ms: 0,
            rise_distance: 16.0,
            blur_radius: 8.0,
        }
    }
}

impl EntranceAnimation {
    pub fn fade_rise_blur() -> Self {
        EntranceAnimation {
            entrance_type: EntranceType::FadeIn,
            easing: EasingCurve::CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 0.2,
                y2: 1.0,
            },
            duration_ms: 350,
            delay_ms: 0,
            rise_distance: 24.0,
            blur_radius: 10.0,
        }
    }

    pub fn compound_entrance() -> Vec<EntranceAnimation> {
        vec![
            EntranceAnimation {
                entrance_type: EntranceType::BlurClear,
                easing: EasingCurve::CubicBezier {
                    x1: 0.0,
                    y1: 0.0,
                    x2: 0.2,
                    y2: 1.0,
                },
                duration_ms: 400,
                delay_ms: 0,
                rise_distance: 0.0,
                blur_radius: 12.0,
            },
            EntranceAnimation {
                entrance_type: EntranceType::Rise,
                easing: EasingCurve::CubicBezier {
                    x1: 0.34,
                    y1: 1.56,
                    x2: 0.64,
                    y2: 1.0,
                },
                duration_ms: 350,
                delay_ms: 80,
                rise_distance: 20.0,
                blur_radius: 0.0,
            },
            EntranceAnimation {
                entrance_type: EntranceType::FadeIn,
                easing: EasingCurve::CubicBezier {
                    x1: 0.0,
                    y1: 0.0,
                    x2: 0.2,
                    y2: 1.0,
                },
                duration_ms: 250,
                delay_ms: 50,
                rise_distance: 0.0,
                blur_radius: 0.0,
            },
        ]
    }

    pub fn press_state() -> EntranceAnimation {
        EntranceAnimation {
            entrance_type: EntranceType::ScaleIn,
            easing: EasingCurve::Spring(SpringParams::snappy()),
            duration_ms: 100,
            delay_ms: 0,
            rise_distance: 0.0,
            blur_radius: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubic_bezier_range() {
        let curve = EasingCurve::CubicBezier {
            x1: 0.4,
            y1: 0.0,
            x2: 0.2,
            y2: 1.0,
        };
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let s = curve.sample(t);
            assert!((0.0..=1.0).contains(&s), "t={} sample={}", t, s);
        }
    }

    #[test]
    fn test_spring_easing_overshoot() {
        let curve = EasingCurve::Spring(SpringParams::expressive());
        let s = curve.sample(0.5);
        assert!(s >= 0.0 && s <= 1.5);
    }

    #[test]
    fn test_linear_is_identity() {
        let curve = EasingCurve::Linear;
        assert_eq!(curve.sample(0.0), 0.0);
        assert_eq!(curve.sample(0.5), 0.5);
        assert_eq!(curve.sample(1.0), 1.0);
    }

    #[test]
    fn test_compound_entrance_order() {
        let steps = EntranceAnimation::compound_entrance();
        assert_eq!(steps.len(), 3);
        assert!(steps[1].delay_ms > steps[0].delay_ms);
    }

    #[test]
    fn test_press_state_duration() {
        let press = EntranceAnimation::press_state();
        assert_eq!(press.duration_ms, 100);
    }
}
