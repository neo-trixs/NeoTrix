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

// ─── trait 层（2026-09-30 补齐）────────────────────────────────────────
// 为什么补：本模块的 `config.rs` / `salience.rs` / `mod.rs` 都 `use` 了
// `DecayCurve` / `ExponentialDecay` / `EbbinghausDecay`，而这些类型
// **全仓都不存在** ⇒ 6 个编译错误，模块被 `nt_memory/mod.rs` 整簇注释，
// 734 行 + 若干测试从此不参与编译。
//
// 成因（已查实）：一次**未完成的 API 迁移**——自由函数形态
// （`ebbinghaus_retention` / `exponential_decay`）已经写好并有测试，
// `mod.rs` 也已改为 re-export 自由函数，但 `config.rs` / `salience.rs`
// 仍在用旧的 trait 形态。
//
// ⇒ 处置：**补齐 trait 层，并让自由函数作为 trait 的默认实现**，
// 两种调用形态共存。这样不必回退任何一边，且旧调用点立即可用。
//
// ⚠️ **单位约定**（这是实现时最容易搞错的一点）：
// · `salience.rs` 调 `self.decay.factor(age_days)` —— 传的是**天**；
// · 本文件的自由函数收的是**小时**（`elapsed_hours`）。
// ⇒ trait 层负责换算（天 × 24），不把单位的坑留给调用方。

/// 保持率曲线：输入**天**，输出 `[0,1]` 的保持因子。
pub trait DecayCurve {
    /// `elapsed_days` 距今多少天，返回该时刻的保持因子。
    fn factor(&self, elapsed_days: f64) -> f64;

    /// 曲线的名字（用于诊断/日志）。
    fn curve_name(&self) -> &'static str;
}

/// 指数衰减 `R = e^(-λ·t)`。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExponentialDecay {
    /// 衰减常数 λ（按**天**计）。
    pub lambda: f64,
}

impl Default for ExponentialDecay {
    /// 默认半衰期 **7 天** ⇒ λ = ln2 / 7。
    fn default() -> Self {
        Self::from_half_life(7.0)
    }
}

impl ExponentialDecay {
    /// 由**半衰期（天）**构造：λ = ln2 / half_life_days。
    pub fn from_half_life(half_life_days: f64) -> Self {
        let hl = half_life_days.max(1e-9);
        Self { lambda: std::f64::consts::LN_2 / hl }
    }

    /// **逆运算**：由 λ 还原半衰期（天）= ln2 / λ。
    ///
    /// ⚠️ 这是 `from_half_life` 的逆运算，首版**漏了**它 ——
    /// `config.rs` 的测试 `presets_have_expected_half_life` 会调
    /// `calc.decay.half_life()`，因方法不存在导致 **lib test 编译失败**
    /// （而 `cargo check --lib` 通过，因为测试代码只在 test 目标编译 —— 
    /// 与本会话已记录的「只跑 check 抓不到测试代码错误」同一教训）。
    pub fn half_life(&self) -> f64 {
        if self.lambda <= 0.0 {
            return f64::INFINITY;
        }
        std::f64::consts::LN_2 / self.lambda
    }
}

impl DecayCurve for ExponentialDecay {
    fn factor(&self, elapsed_days: f64) -> f64 {
        // 天 → 小时，复用既有自由函数（单一实现，避免两处公式漂移）。
        exponential_decay(elapsed_days * 24.0, self.lambda / 24.0)
    }
    fn curve_name(&self) -> &'static str {
        "exponential"
    }
}

/// Ebbinghaus 幂律近似 `R = 1/(1+t/c)`。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EbbinghausDecay {
    /// 有效半衰期（天）。
    pub half_life_days: f64,
}

impl Default for EbbinghausDecay {
    fn default() -> Self {
        Self { half_life_days: 7.0 }
    }
}

impl EbbinghausDecay {
    pub fn new(half_life_days: f64) -> Self {
        Self { half_life_days: half_life_days.max(1e-9) }
    }
}

impl DecayCurve for EbbinghausDecay {
    fn factor(&self, elapsed_days: f64) -> f64 {
        // 天 → 小时，复用既有自由函数。
        ebbinghaus_retention(elapsed_days * 24.0, self.half_life_days * 24.0)
    }
    fn curve_name(&self) -> &'static str {
        "ebbinghaus"
    }
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
