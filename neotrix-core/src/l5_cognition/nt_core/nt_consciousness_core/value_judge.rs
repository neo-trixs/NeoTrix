//! 价值判断器 (ValueJudge)
//! 
//! 评估价值、权衡利弊、伦理判断


use serde::{Deserialize, Serialize};

/// 价值判断器
pub struct ValueJudge {
    /// 价值观
    pub values: Vec<Value>,
    /// 判断历史
    pub history: Vec<JudgmentRecord>,
}

/// 价值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub id: String,
    pub name: String,
    pub description: String,
    pub weight: f64,
    pub value_type: _ValueType,
}

/// 价值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum _ValueType {
    Ethical,
    Practical,
    Aesthetic,
    Social,
    Personal,
}

/// 判断记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentRecord {
    pub id: String,
    pub cycle: u32,
    pub input: String,
    pub judgment: String,
    pub score: f64,
    pub timestamp: String,
}

impl ValueJudge {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            history: Vec::new(),
        }
    }

    /// 评估价值
    pub fn evaluate(&mut self, cycle: u32, option: &str) -> f64 {
        let score = 0.5 + (option.len() as f64 * 0.01).min(0.5);
        
        let record = JudgmentRecord {
            id: format!("judge_{}", uuid::Uuid::new_v4()),
            cycle,
            input: option.to_string(),
            judgment: format!("Score: {}", score),
            score,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.history.push(record);
        score
    }

    /// 权衡利弊
    pub(crate) fn _weigh_pros_cons(&self, pros: &[String], cons: &[String]) -> f64 {
        let pros_score: f64 = pros.iter().map(|p| p.len() as f64 * 0.1).sum();
        let cons_score: f64 = cons.iter().map(|c| c.len() as f64 * 0.1).sum();
        
        if pros_score + cons_score > 0.0 {
            pros_score / (pros_score + cons_score)
        } else {
            0.5
        }
    }

    /// 获取统计
    pub fn stats(&self) -> _ValueStats {
        _ValueStats {
            total_values: self.values.len(),
            total_judgments: self.history.len(),
            avg_score: if self.history.is_empty() {
                0.0
            } else {
                self.history.iter().map(|r| r.score).sum::<f64>() / self.history.len() as f64
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _ValueStats {
    pub total_values: usize,
    pub total_judgments: usize,
    pub avg_score: f64,
}

impl std::fmt::Display for _ValueStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ValueJudge: {} values, {} judgments, avg score {:.4}",
            self.total_values, self.total_judgments, self.avg_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_judge() {
        let mut judge = ValueJudge::new();
        let score = judge.evaluate(0, "test option");
        assert!(score > 0.0);
    }
}
