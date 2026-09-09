//! Resonance Complexity Engine — 共振复杂度引擎
//!
//! 基于 Resonance Complexity Theory (RCT):
//! - 意识从振荡神经活动的稳定干涉模式中涌现
//! - 模式必须超过复杂度、相干性、增益和分形维度的临界阈值
//! - 时空吸引子编码主观意识
//! - 嵌套干涉晶格: 慢振荡提供高频模式稳定的时间支架

use serde::{Deserialize, Serialize};

/// 共振复杂度引擎
pub struct ResonanceComplexityEngine {
    oscillators: Vec<Oscillator>,
    interference_patterns: Vec<InterferencePattern>,
    attractors: Vec<ResonanceAttractor>,
    config: ResonanceConfig,
    stats: ResonanceStats,
}

/// 共振配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceConfig {
    pub min_complexity_index: f64,
    pub min_coherence: f64,
    pub min_gain: f64,
    pub min_fractal_dimension: f64,
    pub min_dwell_time_ms: u64,
    pub enable_nested_hierarchy: bool,
}

impl Default for ResonanceConfig {
    fn default() -> Self {
        Self {
            min_complexity_index: 0.7,
            min_coherence: 0.6,
            min_gain: 1.5,
            min_fractal_dimension: 1.5,
            min_dwell_time_ms: 100,
            enable_nested_hierarchy: true,
        }
    }
}

/// 振荡器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oscillator {
    pub oscillator_id: String,
    pub frequency_hz: f64,
    pub amplitude: f64,
    pub phase: f64,
    pub oscillator_type: OscillatorType,
    pub coupling_strength: f64,
}

/// 振荡器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OscillatorType {
    Delta,      // 0.5-4 Hz
    Theta,      // 4-8 Hz
    Alpha,      // 8-13 Hz
    Beta,       // 13-30 Hz
    Gamma,      // 30-100 Hz
    HighGamma,  // 100+ Hz
    UltraSlow,  // 0.01-0.1 Hz
}

/// 干涉模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterferencePattern {
    pub pattern_id: String,
    pub oscillators: Vec<String>,
    pub pattern_type: PatternType,
    pub spatial_coherence: f64,
    pub temporal_stability: f64,
    pub constructive: bool,
}

/// 模式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    Constructive,   // 相长干涉
    Destructive,    // 相消干涉
    Standing,       // 驻波
    Traveling,      // 行波
}

/// 共振吸引子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceAttractor {
    pub attractor_id: String,
    pub name: String,
    pub complexity_index: f64,
    pub coherence: f64,
    pub gain: f64,
    pub fractal_dimension: f64,
    pub dwell_time_ms: u64,
    pub stable: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 共振统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceStats {
    pub total_oscillators: u64,
    pub total_patterns: u64,
    pub total_attractors: u64,
    pub stable_attractors: u64,
    pub avg_complexity_index: f64,
    pub avg_coherence: f64,
    pub consciousness_level: f64,
}

/// 复杂度指数 (CI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityIndex {
    pub fractal_dimension: f64,
    pub spatial_coherence: f64,
    pub signal_gain: f64,
    pub dwell_time: f64,
    pub composite_score: f64,
}

impl ResonanceComplexityEngine {
    /// 创建新的共振复杂度引擎
    pub fn new() -> Self {
        Self {
            oscillators: Vec::new(),
            interference_patterns: Vec::new(),
            attractors: Vec::new(),
            config: ResonanceConfig::default(),
            stats: ResonanceStats {
                total_oscillators: 0,
                total_patterns: 0,
                total_attractors: 0,
                stable_attractors: 0,
                avg_complexity_index: 0.0,
                avg_coherence: 0.0,
                consciousness_level: 0.0,
            },
        }
    }

    /// 添加振荡器
    pub fn add_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
        self.stats.total_oscillators += 1;
    }

    /// 创建干涉模式
    pub fn create_interference_pattern(&mut self, oscillator_ids: Vec<String>) -> InterferencePattern {
        let pattern = InterferencePattern {
            pattern_id: uuid::Uuid::new_v4().to_string(),
            oscillators: oscillator_ids,
            pattern_type: PatternType::Constructive,
            spatial_coherence: 0.0,
            temporal_stability: 0.0,
            constructive: true,
        };

        self.interference_patterns.push(pattern.clone());
        self.stats.total_patterns += 1;

        pattern
    }

    /// 计算复杂度指数
    pub fn calculate_complexity_index(&self) -> ComplexityIndex {
        let d = 1.8; // 分形维度
        let c = 0.75; // 空间相干性
        let g = 2.0; // 信号增益
        let tau = 150.0; // 驻留时间 (ms)

        let composite: f64 = (d as f64 * c as f64 * g as f64 * tau as f64).sqrt() / 100.0_f64;

        ComplexityIndex {
            fractal_dimension: d,
            spatial_coherence: c,
            signal_gain: g,
            dwell_time: tau,
            composite_score: composite,
        }
    }

    /// 评估意识阈值
    pub fn evaluate_consciousness_threshold(&self) -> f64 {
        let ci = self.calculate_complexity_index();

        let mut score = 0.0;

        // 复杂度指数评估
        if ci.composite_score >= self.config.min_complexity_index {
            score += 0.25;
        }

        // 相干性评估
        if ci.spatial_coherence >= self.config.min_coherence {
            score += 0.25;
        }

        // 增益评估
        if ci.signal_gain >= self.config.min_gain {
            score += 0.25;
        }

        // 分形维度评估
        if ci.fractal_dimension >= self.config.min_fractal_dimension {
            score += 0.25;
        }

        score
    }

    /// 创建共振吸引子
    pub fn create_attractor(&mut self, name: &str) -> ResonanceAttractor {
        let ci = self.calculate_complexity_index();

        let attractor = ResonanceAttractor {
            attractor_id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            complexity_index: ci.composite_score,
            coherence: ci.spatial_coherence,
            gain: ci.signal_gain,
            fractal_dimension: ci.fractal_dimension,
            dwell_time_ms: ci.dwell_time as u64,
            stable: ci.composite_score >= self.config.min_complexity_index,
            timestamp: chrono::Utc::now(),
        };

        self.attractors.push(attractor.clone());
        self.stats.total_attractors += 1;

        if attractor.stable {
            self.stats.stable_attractors += 1;
        }

        attractor
    }

    /// 获取所有振荡器
    pub fn oscillators(&self) -> &[Oscillator] {
        &self.oscillators
    }

    /// 获取所有干涉模式
    pub fn patterns(&self) -> &[InterferencePattern] {
        &self.interference_patterns
    }

    /// 获取所有吸引子
    pub fn attractors(&self) -> &[ResonanceAttractor] {
        &self.attractors
    }

    /// 获取统计信息
    pub fn stats(&self) -> &ResonanceStats {
        &self.stats
    }
}
