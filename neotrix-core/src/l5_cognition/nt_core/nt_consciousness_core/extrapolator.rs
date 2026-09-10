//! 外推器 (Extrapolator)
//! 
//! 趋势外推、模式外推、创新外推

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 外推器
pub struct Extrapolator {
    /// 历史数据点
    pub data_points: Vec<DataPoint>,
    /// 外推历史
    pub history: Vec<ExtrapolationRecord>,
}

/// 数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: u64,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

/// 外推记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrapolationRecord {
    pub id: String,
    pub cycle: u32,
    pub extrapolation_type: ExtrapolationType,
    pub input_points: Vec<DataPoint>,
    pub output: ExtrapolationResult,
    pub timestamp: String,
}

/// 外推类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtrapolationType {
    Linear,
    Polynomial,
    Exponential,
    Pattern,
    Creative,
}

/// 外推结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrapolationResult {
    pub predicted_value: f64,
    pub confidence: f64,
    pub trend: TrendDirection,
    pub next_points: Vec<DataPoint>,
}

/// 趋势方向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
    Unknown,
}

impl Extrapolator {
    pub fn new() -> Self {
        Self {
            data_points: Vec::new(),
            history: Vec::new(),
        }
    }

    /// 添加数据点
    pub fn add_data_point(&mut self, point: DataPoint) {
        self.data_points.push(point);
    }

    /// 线性外推
    pub fn linear_extrapolate(&mut self, cycle: u32, steps: u32) -> ExtrapolationResult {
        let result = if self.data_points.len() >= 2 {
            let n = self.data_points.len() as f64;
            let sum_x: f64 = (0..self.data_points.len() as u64).sum::<u64>() as f64;
            let sum_y: f64 = self.data_points.iter().map(|p| p.value).sum();
            let sum_xy: f64 = self.data_points.iter().enumerate()
                .map(|(i, p)| i as f64 * p.value).sum();
            let sum_x2: f64 = (0..self.data_points.len() as u64)
                .map(|i| i as f64 * i as f64).sum::<f64>();

            let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
            let intercept = (sum_y - slope * sum_x) / n;

            let predicted = slope * (self.data_points.len() as f64 + steps as f64) + intercept;
            
            ExtrapolationResult {
                predicted_value: predicted,
                confidence: 0.8,
                trend: if slope > 0.01 { TrendDirection::Increasing }
                       else if slope < -0.01 { TrendDirection::Decreasing }
                       else { TrendDirection::Stable },
                next_points: vec![],
            }
        } else {
            ExtrapolationResult {
                predicted_value: 0.0,
                confidence: 0.3,
                trend: TrendDirection::Unknown,
                next_points: vec![],
            }
        };

        // 记录
        let record = ExtrapolationRecord {
            id: format!("ext_{}", uuid::Uuid::new_v4()),
            cycle,
            extrapolation_type: ExtrapolationType::Linear,
            input_points: self.data_points.clone(),
            output: result.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        self.history.push(record);
        result
    }

    /// 获取统计
    pub fn stats(&self) -> ExtrapolatorStats {
        ExtrapolatorStats {
            total_data_points: self.data_points.len(),
            total_extrapolations: self.history.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrapolatorStats {
    pub total_data_points: usize,
    pub total_extrapolations: usize,
}

impl std::fmt::Display for ExtrapolatorStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Extrapolator: {} data points, {} extrapolations",
            self.total_data_points, self.total_extrapolations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extrapolator() {
        let mut ext = Extrapolator::new();
        assert_eq!(ext.data_points.len(), 0);
    }
}
