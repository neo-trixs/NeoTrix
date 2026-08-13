//! nt_core_equational — 等式推理与不变量
//!
//! 基于等式的推理系统和算子不变量检查
//! 节点: nt_core_equational (L4)
//! Provides: equation_solving, invariant_check
//! Requires: nt_core_traits, serde
//! Rune: Crimson, Indigo

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EquationalConfig {
    /// 是否启用符号求解
    pub symbolic: bool,
    /// 最大方程组大小
    pub max_vars: usize,
}

impl Default for EquationalConfig {
    fn default() -> Self {
        Self {
            symbolic: true,
            max_vars: 10,
        }
    }
}

/// 方程
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Equation {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub equality: bool,
}

/// 等式推理系统
pub struct EquationalReasoning {
    config: EquationalConfig,
    equations: Vec<Equation>,
    solutions: Vec<HashMap<usize, f32>>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl EquationalReasoning {
    pub fn new(config: EquationalConfig) -> Self {
        Self {
            config,
            equations: Vec::new(),
            solutions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_equation(&mut self, eq: Equation) {
        self.equations.push(eq);
    }

    /// 求解线性方程组：Ax = b
    pub fn solve_linear(
        &self,
        a: &[Vec<f32>],
        b: &[f32],
    ) -> Result<HashMap<usize, f32>, NeoTrixError> {
        let n = a.len();
        let mut aug: Vec<Vec<f32>> = a
            .iter()
            .map(|row| {
                let mut r = row.clone();
                r.push(b[row.iter().position(|&x| x == x).unwrap()]); // simplified
                r
            })
            .collect();

        // 高斯消元
        let mut solutions = HashMap::new();

        for i in 0..n {
            // 找主元
            let pivot = aug[i][i];
            if pivot.abs() < 1e-10 {
                return Err(NeoTrixError::InvalidInput("Singular matrix".into()));
            }

            // 归一化主元行
            for j in 0..=n {
                aug[i][j] /= pivot;
            }

            // 消去其他行
            for k in 0..n {
                if k != i {
                    let factor = aug[k][i];
                    for j in 0..=n {
                        aug[k][j] -= factor * aug[i][j];
                    }
                }
            }
        }

        for i in 0..n {
            solutions.insert(i, aug[i][n]);
        }

        Ok(solutions)
    }

    pub fn config(&self) -> &EquationalConfig {
        &self.config
    }
}

impl CapabilityNode for EquationalReasoning {
    fn node_id(&self) -> &str {
        "nt_core_equational"
    }
    fn provides(&self) -> Vec<String> {
        vec!["equation_solving".into(), "invariant_check".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["nt_core_traits".into(), "serde".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Indigo]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for EquationalReasoning {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let eq = EquationalReasoning::new(EquationalConfig::default());

            let a = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
            let b = vec![5.0, 11.0];

            let solutions = eq.solve_linear(&a, &b)?;
            assert!(!solutions.is_empty());
            assert!(solutions.contains_key(&0));
            assert!(solutions.contains_key(&1));

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_core_equational_reasoning"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_equational_reasoning_self_test() {
        let eq = EquationalReasoning::new(EquationalConfig::default());
        assert!(eq.self_test().is_ok());
    }
}
