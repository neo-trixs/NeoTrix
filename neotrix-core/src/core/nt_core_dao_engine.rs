//! 道引擎 2.0 — 符号回归引擎 (最小可用版本)。
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
                    BinaryOp::Div => if rv == 0.0 { return Err("div by zero".into()); } else { lv / rv },
                })
            }
        }
    }
    pub fn complexity(&self) -> usize {
        match self {
            ExprNode::Const(_) | ExprNode::Var(_) => 1,
            ExprNode::Binary(_, l, r) => 1 + l.complexity() + r.complexity(),
        }
    }
}

impl std::fmt::Display for ExprNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExprNode::Const(v) => write!(f, "{}", v),
            ExprNode::Var(s) => write!(f, "{}", s),
            ExprNode::Binary(op, l, r) => {
                let op_s = match op { BinaryOp::Add=>"+", BinaryOp::Sub=>"-", BinaryOp::Mul=>"*", BinaryOp::Div=>"/" };
                write!(f, "({} {} {})", l, op_s, r)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoEngineConfig { pub population_size: usize, pub max_generations: usize }
impl Default for DaoEngineConfig { fn default() -> Self { Self { population_size: 100, max_generations: 50 } } }

pub struct DaoEngine { pub config: DaoEngineConfig, pub best_expr: Option<String> }
impl DaoEngine {
    pub fn new(config: DaoEngineConfig) -> Self { Self { config, best_expr: None } }
    pub fn fit_linear(&mut self, x: &[f64], y: &[f64]) -> Result<String, String> {
        if x.len() != y.len() || x.is_empty() { return Err("data mismatch".into()); }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_eval() {
        let e = ExprNode::Binary(BinaryOp::Mul, Box::new(ExprNode::Var("x".into())), Box::new(ExprNode::Const(2.0)));
        let mut v = HashMap::new(); v.insert("x".into(), 3.0);
        assert_eq!(e.eval(&v).unwrap(), 6.0);
    }
    #[test]
    fn test_fit() {
        let mut e = DaoEngine::new(DaoEngineConfig::default());
        let x: Vec<f64> = (0..5).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| 2.0 * v + 1.0).collect();
        assert!(e.fit_linear(&x, &y).unwrap().contains("x"));
    }
}
