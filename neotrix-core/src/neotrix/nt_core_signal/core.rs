//! Signal 核心类型
//! 选择性状态向量 Ψ 和矩阵运算基础

use serde::{Deserialize, Serialize};
use std::fmt;
use chrono::Utc;

pub type Vector = Vec<f64>;
pub type Matrix = Vec<Vec<f64>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatrixError {
    DimensionMismatch { expected: usize, got: usize },
    EmptyMatrix,
    EmptyVector,
}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatrixError::DimensionMismatch { expected, got } => {
                write!(f, "Dimension mismatch: expected {}, got {}", expected, got)
            }
            MatrixError::EmptyMatrix => write!(f, "Matrix is empty"),
            MatrixError::EmptyVector => write!(f, "Vector is empty"),
        }
    }
}

impl std::error::Error for MatrixError {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectiveState {
    pub data: Vector,
    pub hidden: Vector,
    pub importance: Vector,
    pub timestamp: i64,
}

impl SelectiveState {
    pub fn new(dim: usize, hidden_dim: usize) -> Self {
        Self {
            data: vec![0.0; dim],
            hidden: vec![0.0; hidden_dim],
            importance: vec![0.0; dim],
            timestamp: 0,
        }
    }

    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// 选择性更新: 根据输入内容决定更新策略
    pub fn select_update(&mut self, input: &Vector, _operator: &super::select::SelectableOperator) {
        let dim = self.data.len().min(input.len());
        let selectivity = Self::compute_selectivity(input);
        for (i, input_val) in input.iter().take(dim).enumerate() {
            let gate = selectivity.get(i).copied().unwrap_or(0.5);
            self.data[i] = gate * input_val + (1.0 - gate) * self.data[i];
            self.importance[i] = gate * input_val.abs();
        }
    }

    fn compute_selectivity(input: &Vector) -> Vector {
        let sum: f64 = input.iter().map(|x| x.abs()).sum();
        if sum <= 0.0 {
            return vec![0.5; input.len()];
        }
        let exp: Vector = input.iter().map(|x| x.abs().exp()).collect();
        let exp_sum: f64 = exp.iter().sum();
        if exp_sum <= 0.0 {
            return vec![0.5; input.len()];
        }
        exp.iter().map(|x| x / exp_sum).collect()
    }

    /// 意识觉醒度
    pub fn awareness_score(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let energy: f64 = self.data.iter().map(|x| x.abs()).sum();
        let max_energy = self.data.len() as f64;
        (energy / max_energy).min(1.0)
    }

    /// 获取当前意识层级
    pub fn tier(&self) -> super::history::ConsciousnessTier {
        super::history::ConsciousnessTier::from_score(self.awareness_score())
    }

    /// 状态积分
    pub fn integrate(&mut self, new_input: &Vector, learning_rate: f64) {
        let dim = self.data.len().min(new_input.len());
        let one_minus_lr = 1.0 - learning_rate;
        for (data_i, input_i) in self.data.iter_mut().zip(new_input.iter()).take(dim) {
            *data_i = *data_i * one_minus_lr + input_i * learning_rate;
        }
        self.timestamp = Utc::now().timestamp();
    }

    /// 冥想 - 状态归零/重置
    pub fn meditate(&mut self) {
        for v in &mut self.data {
            *v *= 0.95;
        }
        self.importance = vec![0.0; self.importance.len()];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_core_signal::select::SelectableOperator;

    #[test]
    fn test_new_selective_state() {
        let state = SelectiveState::new(4, 8);
        assert_eq!(state.data.len(), 4);
        assert_eq!(state.hidden.len(), 8);
        assert_eq!(state.importance.len(), 4);
        assert_eq!(state.timestamp, 0);
        assert!(state.data.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_dim_and_edge_cases() {
        let state = SelectiveState::new(0, 0);
        assert_eq!(state.dim(), 0);
        assert_eq!(state.awareness_score(), 0.0);
        let state2 = SelectiveState::new(10, 5);
        assert_eq!(state2.dim(), 10);
        assert_eq!(state2.hidden.len(), 5);
    }

    #[test]
    fn test_integrate_updates_state() {
        let mut state = SelectiveState::new(3, 3);
        assert_eq!(state.timestamp, 0);
        state.integrate(&vec![1.0, 1.0, 1.0], 0.5);
        assert!(state.data.iter().all(|&x| (x - 0.5).abs() < 1e-10));
        assert!(state.timestamp > 0);
    }

    #[test]
    fn test_meditate_decays_data() {
        let mut state = SelectiveState::new(3, 3);
        state.integrate(&vec![1.0; 3], 1.0);
        state.meditate();
        assert!(state.data.iter().all(|&x| (x - 0.95).abs() < 1e-10));
        assert!(state.importance.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_select_update_with_operator() {
        let mut state = SelectiveState::new(3, 3);
        let op = SelectableOperator::new(3, 3);
        let input = vec![1.0, 0.0, 0.0];
        state.select_update(&input, &op);
        assert!(state.data[0] > 0.0);
        assert!(state.importance[0] > 0.0);
    }
}
