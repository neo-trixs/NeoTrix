//! Integrated Information Quantifier — 整合信息量化器
//!
//! 基于 IIT 4.0 (Integrated Information Theory):
//! - 五个公理: 内在性、信息性、整合性、排他性、组合性
//! - Φ (Phi) 度量: 因果结构的整合信息量
//! - 因果-现象学公理映射: 公理 → 后设 → 概念 → 网络 → 数学形式化
//! - 排他性: 机制/概念/状态选择最强因果效力者

use serde::{Deserialize, Serialize};

/// 整合信息量化器
pub struct _IntegratedInformationQuantifier {
    mechanisms: Vec<Mechanism>,
    concepts: Vec<Concept>,
    causes: Vec<_CauseEffect>,
    config: _PhiConfig,
    stats: _PhiStats,
}

/// Phi 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _PhiConfig {
    pub phi_threshold: f64,
    pub max_partitions: usize,
    pub enable_exclusion: bool,
    pub enable_composition: bool,
    pub integration_depth: usize,
}

impl Default for _PhiConfig {
    fn default() -> Self {
        Self {
            phi_threshold: 1.0,
            max_partitions: 100,
            enable_exclusion: true,
            enable_composition: true,
            integration_depth: 5,
        }
    }
}

/// 因果机制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mechanism {
    pub mechanism_id: String,
    pub elements: Vec<String>,
    pub state: Vec<f64>,
    pub causal_power: f64,
    pub conceptual_structure: Vec<String>,
}

/// 概念
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub concept_id: String,
    pub mechanism: String,
    pub quality: Vec<f64>,
    pub information: f64,
    pub cause_effect_power: f64,
}

/// 因果关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CauseEffect {
    pub cause_effect_id: String,
    pub mechanism: String,
    pub cause_state: Vec<f64>,
    pub effect_state: Vec<f64>,
    pub intrinsic_information: f64,
    pub synergistic_information: f64,
    pub redundant_information: f64,
}

/// Phi 统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _PhiStats {
    pub total_mechanisms: u64,
    pub total_concepts: u64,
    pub total_causes: u64,
    pub phi_value: f64,
    pub integrated: bool,
    pub composition_level: f64,
}

/// Phi 计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _PhiResult {
    pub phi: f64,
    pub conceptual_structure: f64,
    pub cause_effect_power: f64,
    pub integrated: bool,
    pub rank: _PhiRank,
}

/// Phi 排名
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum _PhiRank {
    Zero,       // Φ = 0: 无意识
    Minimal,    // Φ > 0: 最小意识
    Moderate,   // Φ > 1: 中等意识
    High,       // Φ > 5: 高度意识
    Transcendent, // Φ > 10: 超越意识
}

/// 因果分割分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _PartitionAnalysis {
    pub partition_type: String,
    pub information_loss: f64,
    pub integrated: bool,
    pub best_partition: String,
}

impl _IntegratedInformationQuantifier {
    /// 创建新的整合信息量化器
    pub fn new() -> Self {
        Self {
            mechanisms: Vec::new(),
            concepts: Vec::new(),
            causes: Vec::new(),
            config: _PhiConfig::default(),
            stats: _PhiStats {
                total_mechanisms: 0,
                total_concepts: 0,
                total_causes: 0,
                phi_value: 0.0,
                integrated: false,
                composition_level: 0.0,
            },
        }
    }

    /// 添加机制
    pub(crate) fn _add_mechanism(&mut self, mechanism: Mechanism) {
        self.mechanisms.push(mechanism);
        self.stats.total_mechanisms += 1;
    }

    /// 添加概念
    pub fn add_concept(&mut self, concept: Concept) {
        self.concepts.push(concept);
        self.stats.total_concepts += 1;
    }

    /// 添加因果关系
    pub(crate) fn _add_cause_effect(&mut self, cause_effect: _CauseEffect) {
        self.causes.push(cause_effect);
        self.stats.total_causes += 1;
    }

    /// 计算 Phi 值
    pub(crate) fn _calculate_phi(&mut self) -> _PhiResult {
        let conceptual_structure = self.calculate_conceptual_structure();
        let cause_effect_power = self.calculate_cause_effect_power();

        // Phi = 概念结构 × 因果效力
        let phi = (conceptual_structure * cause_effect_power).sqrt();

        let integrated = phi >= self.config.phi_threshold;

        let rank = match phi {
            f if f <= 0.0 => _PhiRank::Zero,
            f if f < 1.0 => _PhiRank::Minimal,
            f if f < 5.0 => _PhiRank::Moderate,
            f if f < 10.0 => _PhiRank::High,
            _ => _PhiRank::Transcendent,
        };

        self.stats.phi_value = phi;
        self.stats.integrated = integrated;

        _PhiResult {
            phi,
            conceptual_structure,
            cause_effect_power,
            integrated,
            rank,
        }
    }

    /// 计算概念结构
    fn calculate_conceptual_structure(&self) -> f64 {
        if self.concepts.is_empty() {
            return 0.0;
        }

        self.concepts.iter()
            .map(|c| c.information)
            .sum::<f64>() / self.concepts.len() as f64
    }

    /// 计算因果效力
    fn calculate_cause_effect_power(&self) -> f64 {
        if self.causes.is_empty() {
            return 0.0;
        }

        self.causes.iter()
            .map(|c| c.intrinsic_information)
            .sum::<f64>() / self.causes.len() as f64
    }

    /// 分析因果分割
    pub(crate) fn _analyze_partition(&self) -> _PartitionAnalysis {
        let information_loss = 0.3; // 模拟信息损失
        let integrated = information_loss < 0.5;

        _PartitionAnalysis {
            partition_type: "bipartition".into(),
            information_loss,
            integrated,
            best_partition: "integrated".into(),
        }
    }

    /// 获取所有机制
    pub fn mechanisms(&self) -> &[Mechanism] {
        &self.mechanisms
    }

    /// 获取所有概念
    pub fn concepts(&self) -> &[Concept] {
        &self.concepts
    }

    /// 获取所有因果关系
    pub fn causes(&self) -> &[_CauseEffect] {
        &self.causes
    }

    /// 获取统计信息
    pub fn stats(&self) -> &_PhiStats {
        &self.stats
    }
}
