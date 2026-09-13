//! Golden Ratio Frequency Architecture — 黄金比例频率架构
//!
//! 基于 Schumann-anchored golden ratio organization:
//! - 神经振荡遵循 φ^n 架构: f(n) = f0 × φ^n
//! - 基础频率 f0 ≈ 7.6 Hz (Schumann 共振)
//! - 黄金比例提供最大抗模式锁定能力
//! - Fibonacci 介导的跨频率耦合路径
//! - 分离 (独立并行处理) 与 整合 (灵活受控通信) 的平衡

use serde::{Deserialize, Serialize};

/// 黄金比例频率架构
pub struct _GoldenRatioFrequencyArchitecture {
    fundamental_frequency: f64,
    phi: f64,
    _frequency_bands: Vec<_GoldenBand>,
    _fibonacci_couplings: Vec<_FibonacciCoupling>,
    config: _GoldenRatioConfig,
    stats: _GoldenRatioStats,
}

/// 黄金比例配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _GoldenRatioConfig {
    pub base_frequency_hz: f64,
    pub phi: f64,
    pub max_harmonics: usize,
    pub enable_fibonacci_coupling: bool,
    pub enable_noble_positions: bool,
}

impl Default for _GoldenRatioConfig {
    fn default() -> Self {
        Self {
            base_frequency_hz: 7.6,
            phi: 1.618033988749895,
            max_harmonics: 10,
            enable_fibonacci_coupling: true,
            enable_noble_positions: true,
        }
    }
}

/// 黄金比例频带
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _GoldenBand {
    pub band_id: String,
    pub n: i32,
    pub frequency_hz: f64,
    pub band_type: _BandType,
    pub stability: f64,
    pub fibonacci_index: Option<usize>,
    pub noble_position: bool,
}

/// 频带类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum _BandType {
    UltraSlow,
    Delta,
    Theta,
    Alpha,
    Beta,
    Gamma,
    HighGamma,
}

/// Fibonacci 耦合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _FibonacciCoupling {
    pub coupling_id: String,
    pub source_band: String,
    pub target_band: String,
    pub fibonacci_ratio: f64,
    pub coupling_strength: f64,
    pub phase_locking: bool,
}

/// 黄金比例统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _GoldenRatioStats {
    pub total_bands: u64,
    pub noble_positions: u64,
    pub _fibonacci_couplings: u64,
    pub avg_stability: f64,
    pub frequency_coverage: f64,
    pub segregation_integration_balance: f64,
}

/// 共振对齐结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ResonanceAlignment {
    pub aligned: bool,
    pub alignment_score: f64,
    pub active_bands: Vec<String>,
    pub coupling_efficiency: f64,
    pub consciousness_potential: f64,
}

impl _GoldenRatioFrequencyArchitecture {
    /// 创建新的黄金比例频率架构
    pub fn new() -> Self {
        let mut arch = Self {
            fundamental_frequency: 7.6,
            phi: 1.618033988749895,
            _frequency_bands: Vec::new(),
            _fibonacci_couplings: Vec::new(),
            config: _GoldenRatioConfig::default(),
            stats: _GoldenRatioStats {
                total_bands: 0,
                noble_positions: 0,
                _fibonacci_couplings: 0,
                avg_stability: 0.0,
                frequency_coverage: 0.0,
                segregation_integration_balance: 0.0,
            },
        };
        arch.initialize_golden_bands();
        arch
    }

    /// 初始化黄金比例频带
    ///
    /// Note: Generates `max_harmonics` bands at f(n) = f0 × φ^n frequencies.
    /// Each band is classified into EEG-like categories (Delta/Theta/Alpha/Beta/Gamma)
    /// based on frequency ranges. Noble positions (fractional ≈ 0.382 or 0.618) get
    /// higher stability (0.9 vs 0.7) — these correspond to maximally irrational ratios
    /// that resist phase-locking, enabling independent parallel processing.
    fn initialize_golden_bands(&mut self) {
        for n in 0..self.config.max_harmonics {
            let frequency = self.fundamental_frequency * self.phi.powi(n as i32);

            let band_type = match frequency {
                f if f < 0.5 => _BandType::UltraSlow,
                f if f < 4.0 => _BandType::Delta,
                f if f < 8.0 => _BandType::Theta,
                f if f < 13.0 => _BandType::Alpha,
                f if f < 30.0 => _BandType::Beta,
                f if f < 100.0 => _BandType::Gamma,
                _ => _BandType::HighGamma,
            };

            let fibonacci_index = Self::closest_fibonacci(n);
            let noble_position = Self::is_noble_position(n as f64);

            let band = _GoldenBand {
                band_id: uuid::Uuid::new_v4().to_string(),
                n: n as i32,
                frequency_hz: frequency,
                band_type,
                stability: if noble_position { 0.9 } else { 0.7 },
                fibonacci_index,
                noble_position,
            };

            self._frequency_bands.push(band);
            self.stats.total_bands += 1;

            if noble_position {
                self.stats.noble_positions += 1;
            }
        }
    }

    /// 查找最接近的 Fibonacci 数
    ///
    /// Note: Returns the index of `n` in the Fibonacci sequence [1,1,2,3,5,8,13,21,34,55,89].
    /// Returns `None` if `n` is not a Fibonacci number. Used to tag bands that align with
    /// Fibonacci indices for cross-frequency coupling path selection.
    fn closest_fibonacci(n: usize) -> Option<usize> {
        let fibs = [1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89];
        fibs.iter().position(|&f| f == n)
    }

    /// 判断是否为 Noble 位置
    fn is_noble_position(n: f64) -> bool {
        let fractional = n.fract();
        (fractional - 0.618).abs() < 0.05 || (fractional - 0.382).abs() < 0.05
    }

    /// 创建 Fibonacci 耦合
    ///
    /// Note: Creates a phase-locking coupling between two frequency bands.
    /// The coupling strength (0.8) and phase_locking (true) are hardcoded defaults.
    /// Real implementation needs:
    /// - Adaptive coupling strength based on frequency ratio proximity to Fibonacci ratios
    /// - Phase-locking value (PLV) computation from actual signal data
    /// - Support for higher-order couplings (3+ band interactions)
    pub(crate) fn _create_fibonacci_coupling(&mut self, source_idx: usize, target_idx: usize) -> _FibonacciCoupling {
        let source = &self._frequency_bands[source_idx];
        let target = &self._frequency_bands[target_idx];

        let ratio = target.frequency_hz / source.frequency_hz;
        let closest_fib = Self::find_closest_fibonacci_ratio(ratio);

        let coupling = _FibonacciCoupling {
            coupling_id: uuid::Uuid::new_v4().to_string(),
            source_band: source.band_id.clone(),
            target_band: target.band_id.clone(),
            fibonacci_ratio: closest_fib,
            coupling_strength: 0.8,
            phase_locking: true,
        };

        self._fibonacci_couplings.push(coupling.clone());
        self.stats._fibonacci_couplings += 1;

        coupling
    }

    /// 查找最接近的 Fibonacci 比率
    ///
    /// Note: Returns the Fibonacci ratio closest to the input ratio.
    /// The ratio table [1.0, 1.0, 2.0, 1.5, 1.667, ...] approximates F(n+1)/F(n)
    /// converging to φ. Used for coupling classification — bands with Fibonacci ratios
    /// have stronger cross-frequency coupling in neural models.
    fn find_closest_fibonacci_ratio(ratio: f64) -> f64 {
        let fib_ratios = [1.0, 1.0, 2.0, 1.5, 1.667, 1.6, 1.625, 1.615, 1.619, 1.618];
        fib_ratios.iter()
            .min_by(|a, b| (**a - ratio).abs().partial_cmp(&(**b - ratio).abs()).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
            .unwrap_or(1.618)
    }

    /// 评估共振对齐
    ///
    /// Note: Computes alignment_score as mean band stability, coupling_efficiency as
    /// mean coupling strength. consciousness_potential is their average. The `aligned`
    /// flag fires when alignment_score > 0.7. Real implementation needs:
    /// - Weighted stability (noble-position bands weighted higher)
    /// - Phase coherence metric (not just amplitude stability)
    /// - Dynamic threshold based on system load
    pub(crate) fn _evaluate_alignment(&self) -> _ResonanceAlignment {
        let active_bands: Vec<String> = self._frequency_bands.iter()
            .filter(|b| b.stability > 0.7)
            .map(|b| b.band_id.clone())
            .collect();

        let alignment_score = if !self._frequency_bands.is_empty() {
            self._frequency_bands.iter()
                .map(|b| b.stability)
                .sum::<f64>() / self._frequency_bands.len() as f64
        } else {
            0.0
        };

        let coupling_efficiency = if !self._fibonacci_couplings.is_empty() {
            self._fibonacci_couplings.iter()
                .map(|c| c.coupling_strength)
                .sum::<f64>() / self._fibonacci_couplings.len() as f64
        } else {
            0.0
        };

        _ResonanceAlignment {
            aligned: alignment_score > 0.7,
            alignment_score,
            active_bands,
            coupling_efficiency,
            consciousness_potential: (alignment_score + coupling_efficiency) / 2.0,
        }
    }

    /// 获取所有频带
    pub(crate) fn _frequency_bands(&self) -> &[_GoldenBand] {
        &self._frequency_bands
    }

    /// 获取所有 Fibonacci 耦合
    pub(crate) fn _fibonacci_couplings(&self) -> &[_FibonacciCoupling] {
        &self._fibonacci_couplings
    }

    /// 获取统计信息
    pub fn stats(&self) -> &_GoldenRatioStats {
        &self.stats
    }
}
