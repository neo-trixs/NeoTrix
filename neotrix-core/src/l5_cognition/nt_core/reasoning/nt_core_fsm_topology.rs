//! FSM Behavior Topology Engine — FSM行为拓扑引擎
//!
//! 吸收 KB 经验:
//! - arXiv:2608.23670 Digital Automata
//! - FSM 行为模型集成到 GWT attention routing
//! - 失败预测能力
//! - 目标预测

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// FSM 行为拓扑引擎
pub struct _FSMBehaviorTopologyEngine {
    states: Vec<_FSMState>,
    transitions: Vec<_FSMTransition>,
    predictions: Vec<_BehaviorPrediction>,
    #[allow(dead_code)]
    config: _FSMConfig,
    stats: _FSMStats,
}

/// FSM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _FSMConfig {
    pub max_states: usize,
    pub max_transitions: usize,
    pub prediction_horizon: u32,
    pub enable_failure_prediction: bool,
    pub enable_goal_prediction: bool,
}

impl Default for _FSMConfig {
    fn default() -> Self {
        Self {
            max_states: 100,
            max_transitions: 500,
            prediction_horizon: 10,
            enable_failure_prediction: true,
            enable_goal_prediction: true,
        }
    }
}

/// FSM 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _FSMState {
    pub state_id: String,
    pub name: String,
    pub state_type: _StateType,
    pub metrics: _StateMetrics,
    pub metadata: HashMap<String, String>,
}

/// 状态类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum _StateType {
    Initial,
    Normal,
    Warning,
    Critical,
    Terminal,
}

/// 状态指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _StateMetrics {
    pub visit_count: u64,
    pub avg_duration: f64,
    pub success_rate: f64,
    pub failure_rate: f64,
}

/// FSM 转换
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _FSMTransition {
    pub transition_id: String,
    pub from_state: String,
    pub to_state: String,
    pub trigger: String,
    pub probability: f64,
    pub conditions: Vec<String>,
}

/// 行为预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _BehaviorPrediction {
    pub prediction_id: String,
    pub current_state: String,
    pub predicted_states: Vec<String>,
    pub confidence: f64,
    pub prediction_type: _PredictionType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 预测类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum _PredictionType {
    NextState,
    Failure,
    Goal,
    Recovery,
}

/// FSM 统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _FSMStats {
    pub total_states: u64,
    pub total_transitions: u64,
    pub total_predictions: u64,
    pub successful_predictions: u64,
    pub failed_predictions: u64,
    pub avg_prediction_accuracy: f64,
}

impl _FSMBehaviorTopologyEngine {
    /// 创建新的 FSM 行为拓扑引擎
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            transitions: Vec::new(),
            predictions: Vec::new(),
            config: _FSMConfig::default(),
            stats: _FSMStats {
                total_states: 0,
                total_transitions: 0,
                total_predictions: 0,
                successful_predictions: 0,
                failed_predictions: 0,
                avg_prediction_accuracy: 0.0,
            },
        }
    }

    /// 添加状态
    pub fn add_state(&mut self, state: _FSMState) {
        self.states.push(state);
        self.stats.total_states += 1;
    }

    /// 添加转换
    pub fn add_transition(&mut self, transition: _FSMTransition) {
        self.transitions.push(transition);
        self.stats.total_transitions += 1;
    }

    /// 预测下一个状态
    pub fn predict_next_state(&mut self, current_state: &str) -> Option<_BehaviorPrediction> {
        let possible_transitions: Vec<&_FSMTransition> = self.transitions
            .iter()
            .filter(|t| t.from_state == current_state)
            .collect();

        if possible_transitions.is_empty() {
            return None;
        }

        // 选择概率最高的转换
        let best_transition = possible_transitions
            .iter()
            .max_by(|a, b| a.probability.partial_cmp(&b.probability).unwrap())?;

        let prediction = _BehaviorPrediction {
            prediction_id: uuid::Uuid::new_v4().to_string(),
            current_state: current_state.to_string(),
            predicted_states: vec![best_transition.to_state.clone()],
            confidence: best_transition.probability,
            prediction_type: _PredictionType::NextState,
            timestamp: chrono::Utc::now(),
        };

        self.predictions.push(prediction.clone());
        self.stats.total_predictions += 1;

        Some(prediction)
    }

    /// 预测失败
    pub fn predict_failure(&self, current_state: &str) -> Option<f64> {
        // 查找从当前状态到失败状态的转换
        let failure_transitions: Vec<&_FSMTransition> = self.transitions
            .iter()
            .filter(|t| {
                t.from_state == current_state && 
                self.states.iter().any(|s| s.state_id == t.to_state && s.state_type == _StateType::Critical)
            })
            .collect();

        if failure_transitions.is_empty() {
            return Some(0.0);
        }

        let failure_probability: f64 = failure_transitions
            .iter()
            .map(|t| t.probability)
            .sum();

        Some(failure_probability.min(1.0))
    }

    /// 获取所有状态
    pub fn states(&self) -> &[_FSMState] {
        &self.states
    }

    /// 获取所有转换
    pub fn transitions(&self) -> &[_FSMTransition] {
        &self.transitions
    }

    /// 获取统计信息
    pub fn stats(&self) -> &_FSMStats {
        &self.stats
    }
}
