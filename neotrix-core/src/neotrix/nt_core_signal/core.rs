//! Signal 核心类型
//! 选择性状态向量 Ψ 和矩阵运算基础

pub type Vector = Vec<f64>;
pub type Matrix = Vec<Vec<f64>>;

pub use crate::core::nt_core_ssm::MatrixError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_ssm::SelectableOperator;
    use crate::core::nt_core_ssm::SelectiveState;

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
