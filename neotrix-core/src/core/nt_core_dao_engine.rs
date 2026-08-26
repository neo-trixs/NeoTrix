//! 符号回归引擎。
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BinaryOp { Add, Sub, Mul, Div }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExprNode {
    Const(f64),
    Var(String),
    Binary(BinaryOp, Box<ExprNode>, Box<ExprNode>),
}

impl ExprNode {
    pub fn eval(&self, vars: &HashMap<String, f64>) -> Result<f64, String> {
        match self {
            ExprNode::Const(v) => Ok(*v),
            ExprNode::Var(n) => vars.get(n).copied().ok_or_else(|| format!("var not bound: {n}")),
            ExprNode::Binary(op, l, r) => {
                let lv = l.eval(vars)?;
                let rv = r.eval(vars)?;
                Ok(match op {
                    BinaryOp::Add => lv + rv,
                    BinaryOp::Sub => lv - rv,
                    BinaryOp::Mul => lv * rv,
                    BinaryOp::Div => if rv == 0.0 { return Err("div0".into()); } else { lv / rv },
                })
            }
        }
    }
}

impl std::fmt::Display for ExprNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExprNode::Const(v) => write!(f, "{}", v),
            ExprNode::Var(s) => write!(f, "{}", s),
            ExprNode::Binary(op, l, r) => {
                let s = match op { BinaryOp::Add=>"+", BinaryOp::Sub=>"-", BinaryOp::Mul=>"*", BinaryOp::Div=>"/" };
                write!(f, "({} {} {})", l, s, r)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoEngineConfig { pub population_size: usize, pub max_generations: usize }
impl Default for DaoEngineConfig { fn default() -> Self { Self { population_size: 100, max_generations: 50 } } }

pub struct DaoEngine { pub config: DaoEngineConfig, pub best_expr: Option<String> }
impl DaoEngine {
    pub fn new(c: DaoEngineConfig) -> Self { Self { config: c, best_expr: None } }
    pub fn fit_linear(&mut self, x: &[f64], y: &[f64]) -> Result<String, String> {
        if x.len() != y.len() || x.is_empty() { return Err("mismatch".into()); }
        let n = x.len() as f64;
        let sx: f64 = x.iter().sum();
        let sy: f64 = y.iter().sum();
        let sxx: f64 = x.iter().map(|v| v*v).sum();
        let sxy: f64 = x.iter().zip(y.iter()).map(|(a,b)| a*b).sum();
        let d = n * sxx - sx * sx;
        if d == 0.0 { return Err("singular".into()); }
        let a = (n * sxy - sx * sy) / d;
        let b = (sy - a * sx) / n;
        self.best_expr = Some(format!("({} * x + {})", a, b));
        Ok(self.best_expr.clone().unwrap())
    }

    /// 二次拟合 y = ax² + bx + c (最小二乘)
    pub fn fit_quadratic(&mut self, x: &[f64], y: &[f64]) -> Result<String, String> {
        if x.len() < 3 { return Err("need >= 3 points".into()); }
        // 正规方程组求解 3x3 线性系统
        let n = x.len() as f64;
        let mut sums = [0.0f64; 5]; // Σ1,Σx,Σx²,Σx³,Σx⁴
        for &xi in x {
            let xi2 = xi * xi;
            sums[0] += 1.0; sums[1] += xi; sums[2] += xi2;
            sums[3] += xi2 * xi; sums[4] += xi2 * xi2;
        }
        let mut rhs = [0.0f64; 3]; // Σy,Σxy,Σx²y
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            let xi2 = xi * xi;
            rhs[0] += yi; rhs[1] += xi * yi; rhs[2] += xi2 * yi;
        }

        // Cramer's rule 解 [n,Σx,Σx²][Σx,Σx²,Σx³][Σx²,Σx³,Σx⁴] × [c,b,a] = [Σy,Σxy,Σx²y]
        let det = n*(sums[2]*sums[4]-sums[3]*sums[3]) - sums[1]*(sums[1]*sums[4]-sums[3]*sums[2]) + sums[2]*(sums[1]*sums[3]-sums[2]*sums[2]);
        if det.abs() < 1e-10 { return Err("singular matrix".into()); }

        let det_c = rhs[0]*(sums[2]*sums[4]-sums[3]*sums[3]) - rhs[1]*(sums[1]*sums[4]-sums[3]*sums[2]) + rhs[2]*(sums[1]*sums[3]-sums[2]*sums[2]);
        let det_b = n*(rhs[1]*sums[4]-sums[3]*rhs[2]) - sums[1]*(rhs[0]*sums[4]-rhs[2]*sums[2]) + sums[2]*(rhs[0]*sums[3]-rhs[1]*sums[1]);
        let det_a = n*(sums[2]*rhs[2]-rhs[1]*sums[2]) - sums[1]*(sums[1]*rhs[2]-rhs[1]*sums[1]) + rhs[2]*(sums[1]*sums[2]-sums[2]*sums[1]);

        let c_coef = det_c / det;
        let b_coef = det_b / det;
        let a_coef = det_a / det;

        self.best_expr = Some(format!("({:.4}x² + {:.4}x + {:.4})", a_coef, b_coef, c_coef));
        Ok(self.best_expr.clone().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fit_quadratic() {
        let mut e = DaoEngine::new(DaoEngineConfig::default());
        let x: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
        let y: Vec<f64> = vec![1.0, 4.0, 9.0, 16.0]; // y = x²
        let result = e.fit_quadratic(&x, &y);
        assert!(result.is_ok(), "quadratic fit should succeed: {:?}", result.err());
    }

    #[test]
    fn test_fit_linear() {
        let mut e = DaoEngine::new(DaoEngineConfig::default());
        let x: Vec<f64> = (0..5).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| 2.0 * v + 1.0).collect();
        assert!(e.fit_linear(&x, &y).unwrap().contains("x"));
    }
}
