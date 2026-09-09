//! Field Resonance Coordinator — 场共振协调器
//!
//! 基于 Field Resonance Theory:
//! - 电磁场可以在 0.74 mV/mm 阈值下 entrain 神经脉冲时序
//! - 5000 倍速度优势: ephaptic 场传播 50 km/s vs 脉冲 10-100 m/s
//! - 跨频率耦合: 相位-振幅耦合、频率-频率耦合
//! - 嵌套振荡层级: 超慢波动 → delta → theta → alpha → beta → gamma

use serde::{Deserialize, Serialize};

/// 场共振协调器
pub struct FieldResonanceCoordinator {
    field_components: Vec<FieldComponent>,
    coupling_mechanisms: Vec<CouplingMechanism>,
    coordination_states: Vec<CoordinationState>,
    config: FieldCoordinationConfig,
    stats: FieldCoordinationStats,
}

/// 场协调配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCoordinationConfig {
    pub propagation_speed_km_s: f64,
    pub min_entrainment_threshold: f64,
    pub enable_cross_frequency_coupling: bool,
    pub enable_nested_hierarchy: bool,
    pub integration_window_ms: f64,
}

impl Default for FieldCoordinationConfig {
    fn default() -> Self {
        Self {
            propagation_speed_km_s: 50.0,
            min_entrainment_threshold: 0.74,
            enable_cross_frequency_coupling: true,
            enable_nested_hierarchy: true,
            integration_window_ms: 3.0,
        }
    }
}

/// 场组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldComponent {
    pub component_id: String,
    pub component_type: FieldType,
    pub spatial_extent: f64,
    pub temporal_dynamics: f64,
    pub coherence_level: f64,
    pub active: bool,
}

/// 场类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Electromagnetic,
    Ephaptic,
    Ionic,
    Quantum,
}

/// 耦合机制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingMechanism {
    pub mechanism_id: String,
    pub coupling_type: CouplingType,
    pub source_frequency: f64,
    pub target_frequency: f64,
    pub coupling_strength: f64,
    pub phase_relationship: f64,
}

/// 耦合类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CouplingType {
    PhaseAmplitude,     // 相位-振幅耦合
    FrequencyFrequency, // 频率-频率耦合
    PhasePhase,         // 相位-相位耦合
    AmplitudeAmplitude, // 振幅-振幅耦合
}

/// 协调状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationState {
    pub state_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub field_coherence: f64,
    pub coupling_efficiency: f64,
    pub integration_level: f64,
    pub consciousness_markers: Vec<String>,
}

/// 场协调统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCoordinationStats {
    pub total_components: u64,
    pub active_components: u64,
    pub total_couplings: u64,
    pub avg_coherence: f64,
    pub avg_coupling_efficiency: f64,
    pub integration_speed_ms: f64,
}

/// 全脑整合结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WholeBrainIntegration {
    pub integration_achieved: bool,
    pub integration_time_ms: f64,
    pub coherence_level: f64,
    pub active_couplings: u64,
    pub consciousness_emergent: bool,
}

impl FieldResonanceCoordinator {
    /// 创建新的场共振协调器
    pub fn new() -> Self {
        Self {
            field_components: Vec::new(),
            coupling_mechanisms: Vec::new(),
            coordination_states: Vec::new(),
            config: FieldCoordinationConfig::default(),
            stats: FieldCoordinationStats {
                total_components: 0,
                active_components: 0,
                total_couplings: 0,
                avg_coherence: 0.0,
                avg_coupling_efficiency: 0.0,
                integration_speed_ms: 0.0,
            },
        }
    }

    /// 添加场组件
    pub fn add_field_component(&mut self, component: FieldComponent) {
        self.field_components.push(component);
        self.stats.total_components += 1;
    }

    /// 添加耦合机制
    pub fn add_coupling_mechanism(&mut self, mechanism: CouplingMechanism) {
        self.coupling_mechanisms.push(mechanism);
        self.stats.total_couplings += 1;
    }

    /// 计算全脑整合时间
    pub fn calculate_integration_time(&self, brain_volume: f64) -> f64 {
        // 50 km/s 传播速度，3ms 内整合全脑
        let distance = brain_volume.cbrt(); // 立方根近似直径
        (distance / self.config.propagation_speed_km_s) * 1000.0 // 转换为 ms
    }

    /// 评估场协调状态
    pub fn evaluate_coordination(&self) -> CoordinationState {
        let active_components = self.field_components.iter().filter(|c| c.active).count();
        let total_components = self.field_components.len();

        let field_coherence = if total_components > 0 {
            self.field_components.iter()
                .map(|c| c.coherence_level)
                .sum::<f64>() / total_components as f64
        } else {
            0.0
        };

        let coupling_efficiency = if !self.coupling_mechanisms.is_empty() {
            self.coupling_mechanisms.iter()
                .map(|m| m.coupling_strength)
                .sum::<f64>() / self.coupling_mechanisms.len() as f64
        } else {
            0.0
        };

        let mut consciousness_markers = Vec::new();
        if field_coherence > 0.7 {
            consciousness_markers.push("high_coherence".into());
        }
        if coupling_efficiency > 0.6 {
            consciousness_markers.push("strong_coupling".into());
        }
        if active_components as f64 / total_components as f64 > 0.8 {
            consciousness_markers.push("widespread_activation".into());
        }

        CoordinationState {
            state_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            field_coherence,
            coupling_efficiency,
            integration_level: (field_coherence + coupling_efficiency) / 2.0,
            consciousness_markers,
        }
    }

    /// 执行全脑整合
    pub fn integrate_whole_brain(&mut self) -> WholeBrainIntegration {
        let state = self.evaluate_coordination();
        let integration_time = self.calculate_integration_time(1400.0); // 1400 cm³ 大脑体积

        let consciousness_emergent = state.field_coherence > 0.7 &&
                                   state.coupling_efficiency > 0.6 &&
                                   integration_time < 5.0;

        self.coordination_states.push(state.clone());

        WholeBrainIntegration {
            integration_achieved: integration_time < 5.0,
            integration_time_ms: integration_time,
            coherence_level: state.field_coherence,
            active_couplings: self.stats.total_couplings,
            consciousness_emergent,
        }
    }

    /// 获取所有场组件
    pub fn field_components(&self) -> &[FieldComponent] {
        &self.field_components
    }

    /// 获取所有耦合机制
    pub fn coupling_mechanisms(&self) -> &[CouplingMechanism] {
        &self.coupling_mechanisms
    }

    /// 获取统计信息
    pub fn stats(&self) -> &FieldCoordinationStats {
        &self.stats
    }
}
