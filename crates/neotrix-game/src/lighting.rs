// 光照系统 — 吸收 Godot 2D Lighting

use macroquad::prelude::{Color, WHITE};

#[derive(Debug, Clone)]
pub struct Light2D {
    pub x: f32,
    pub y: f32,
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
}

impl Light2D {
    pub fn point(x: f32, y: f32, color: Color, intensity: f32, radius: f32) -> Self {
        Self { x, y, color, intensity, radius }
    }
}

#[derive(Debug, Clone)]
pub struct DayNightCycle {
    pub time_hours: f32,
    pub speed: f32,
    pub ambient_color: Color,
    pub ambient_intensity: f32,
}

impl DayNightCycle {
    pub fn new() -> Self {
        Self { time_hours: 8.0, speed: 0.5, ambient_color: WHITE, ambient_intensity: 1.0 }
    }

    pub fn update(&mut self, dt: f32) {
        self.time_hours += self.speed * dt;
        if self.time_hours >= 24.0 { self.time_hours -= 24.0; }
        self.compute_ambient();
    }

    fn compute_ambient(&mut self) {
        let h = self.time_hours;
        let (r, g, b, intensity) = if h >= 6.0 && h < 8.0 {
            let t = (h - 6.0) / 2.0;
            (lerp(0.2, 1.0, t), lerp(0.1, 0.9, t), lerp(0.3, 0.8, t), lerp(0.4, 1.0, t))
        } else if h >= 8.0 && h < 17.0 {
            (1.0, 0.95, 0.9, 1.0)
        } else if h >= 17.0 && h < 19.0 {
            let t = (h - 17.0) / 2.0;
            (lerp(1.0, 0.3, t), lerp(0.9, 0.15, t), lerp(0.8, 0.4, t), lerp(1.0, 0.5, t))
        } else {
            (0.1, 0.1, 0.25, 0.35)
        };
        self.ambient_color = Color::new(r, g, b, 1.0);
        self.ambient_intensity = intensity;
    }

    pub fn time_label(&self) -> String {
        let h = self.time_hours as u32;
        let m = ((self.time_hours - h as f32) * 60.0) as u32;
        format!("{:02}:{:02}", h, m)
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

/// 剧情实景天空梯度（纯函数）：时辰 →（天顶色， 地平线色），0–1 RGB。
/// 夜靛 / 晓橙 / 昼蓝 / 昏绛四段关键帧线性插值，供 LVSS 天空层用。
pub fn sky_gradient(h: f32) -> ((f32, f32, f32), (f32, f32, f32)) {
    let h = h.rem_euclid(24.0);
    // (时刻, 顶, 平)
    const KEYS: [(f32, (f32, f32, f32), (f32, f32, f32)); 6] = [
        (0.0, (0.03, 0.04, 0.12), (0.08, 0.08, 0.20)),
        (5.0, (0.05, 0.06, 0.18), (0.25, 0.12, 0.28)),
        (7.5, (0.35, 0.45, 0.75), (0.95, 0.55, 0.35)),
        (10.0, (0.25, 0.50, 0.90), (0.65, 0.85, 0.95)),
        (17.5, (0.20, 0.30, 0.65), (0.90, 0.45, 0.30)),
        (20.0, (0.04, 0.05, 0.14), (0.20, 0.08, 0.22)),
    ];
    let mut i = 0;
    while i + 1 < KEYS.len() && h >= KEYS[i + 1].0 {
        i += 1;
    }
    let (h0, t0, g0) = KEYS[i];
    let (h1, t1, g1) = if i + 1 < KEYS.len() { KEYS[i + 1] } else { KEYS[0] };
    let span = if h1 > h0 { h1 - h0 } else { 24.0 - h0 + h1 };
    let t = if span <= 0.0 { 0.0 } else { ((h - h0).rem_euclid(24.0) / span).clamp(0.0, 1.0) };
    let mix = |a: (f32, f32, f32), b: (f32, f32, f32)| {
        (lerp(a.0, b.0, t), lerp(a.1, b.1, t), lerp(a.2, b.2, t))
    };
    (mix(t0, t1), mix(g0, g1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sky_keys_sane() {
        // 正午天顶蓝于地平线？实为天顶深蓝、地平浅蓝
        let (top, hor) = sky_gradient(12.0);
        assert!(top.2 > hor.2 || top.0 < hor.0);
        // 子夜深
        let (nt, _) = sky_gradient(0.0);
        assert!(nt.0 < 0.1 && nt.1 < 0.1);
        // 24h 周期闭合
        assert_eq!(sky_gradient(24.0), sky_gradient(0.0));
        // 渐变连续（晓昏之间单调不过冲）
        let (a, _) = sky_gradient(6.0);
        let (b, _) = sky_gradient(9.0);
        assert!(b.0 >= a.0 && b.2 >= a.2);
    }
}
