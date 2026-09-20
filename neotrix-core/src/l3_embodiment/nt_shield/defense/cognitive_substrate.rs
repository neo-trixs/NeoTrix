//! Cognitive Substrate - 认知基底
//!
//! 持续感知 + 主动防御
//! 设计原则：从被动防御转为主动感知

use std::collections::HashMap;

/// 感知信号
#[derive(Debug, Clone)]
pub struct PerceptionSignal {
    pub source: String,
    pub signal_type: String,
    pub value: f64,
    pub context: HashMap<String, String>,
}

/// 感知结果
#[derive(Debug, Clone)]
pub struct PerceptionResult {
    pub awareness_score: f64,
    pub threats: Vec<Threat>,
    pub recommendations: Vec<String>,
    pub defense_mode: DefenseMode,
}

#[derive(Debug, Clone)]
pub struct Threat {
    pub name: String,
    pub severity: f64,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenseMode {
    /// 被动防御：等待攻击
    Passive,
    /// 主动防御：预测攻击
    Proactive,
    /// 自适应防御：根据威胁调整
    Adaptive,
}

/// 认知基底：持续感知 + 主动防御
pub struct CognitiveSubstrate {
    /// 历史信号
    signal_history: Vec<PerceptionSignal>,
    /// 当前防御模式
    defense_mode: DefenseMode,
    /// 感知窗口
    window_size: usize,
}

impl CognitiveSubstrate {
    pub fn new() -> Self {
        Self {
            signal_history: Vec::new(),
            defense_mode: DefenseMode::Passive,
            window_size: 100,
        }
    }

    /// 处理感知信号
    pub fn perceive(&mut self, signal: PerceptionSignal) -> PerceptionResult {
        self.signal_history.push(signal);

        // 保持窗口大小
        if self.signal_history.len() > self.window_size {
            self.signal_history.remove(0);
        }

        // 计算感知分数
        let awareness_score = self.calculate_awareness();

        // 检测威胁
        let threats = self.detect_threats();

        // 调整防御模式
        self.defense_mode = self.adjust_defense_mode(awareness_score, &threats);

        // 生成建议
        let recommendations = self.generate_recommendations(&threats);

        PerceptionResult {
            awareness_score,
            threats,
            recommendations,
            defense_mode: self.defense_mode,
        }
    }

    /// 计算感知分数
    fn calculate_awareness(&self) -> f64 {
        if self.signal_history.is_empty() {
            return 0.0;
        }

        let avg_value: f64 = self.signal_history
            .iter()
            .map(|s| s.value)
            .sum::<f64>() / self.signal_history.len() as f64;

        let unique_sources: std::collections::HashSet<_> =
            self.signal_history.iter().map(|s| s.source.clone()).collect();

        let source_diversity = unique_sources.len() as f64 / 10.0;

        (avg_value * 0.6 + source_diversity * 0.4).min(1.0)
    }

    /// 检测威胁
    fn detect_threats(&self) -> Vec<Threat> {
        let mut threats = Vec::new();

        // 检测高频攻击
        let mut source_counts: HashMap<String, u32> = HashMap::new();
        for signal in &self.signal_history {
            *source_counts.entry(signal.source.clone()).or_insert(0) += 1;
        }

        for (source, count) in &source_counts {
            if *count > 10 {
                threats.push(Threat {
                    name: "High frequency attack".to_string(),
                    severity: 0.8,
                    source: source.clone(),
                });
            }
        }

        threats
    }

    /// 调整防御模式
    fn adjust_defense_mode(&self, awareness: f64, threats: &[Threat]) -> DefenseMode {
        if threats.iter().any(|t| t.severity > 0.8) {
            DefenseMode::Adaptive
        } else if awareness > 0.5 {
            DefenseMode::Proactive
        } else {
            DefenseMode::Passive
        }
    }

    /// 生成建议
    fn generate_recommendations(&self, threats: &[Threat]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !threats.is_empty() {
            recommendations.push("Enable enhanced monitoring".to_string());
            recommendations.push("Review recent inputs".to_string());
        }

        if self.defense_mode == DefenseMode::Adaptive {
            recommendations.push("Switch to adaptive defense".to_string());
        }

        recommendations
    }
}

impl Default for CognitiveSubstrate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perceive_empty() {
        let mut substrate = CognitiveSubstrate::new();
        let signal = PerceptionSignal {
            source: "test".to_string(),
            signal_type: "input".to_string(),
            value: 0.5,
            context: HashMap::new(),
        };
        let result = substrate.perceive(signal);
        assert_eq!(result.defense_mode, DefenseMode::Passive);
    }

    #[test]
    fn test_perceive_threats() {
        let mut substrate = CognitiveSubstrate::new();
        // 添加多个信号触发威胁
        for _ in 0..15 {
            substrate.perceive(PerceptionSignal {
                source: "attacker".to_string(),
                signal_type: "input".to_string(),
                value: 0.9,
                context: HashMap::new(),
            });
        }
        let result = substrate.perceive(PerceptionSignal {
            source: "attacker".to_string(),
            signal_type: "input".to_string(),
            value: 0.9,
            context: HashMap::new(),
        });
        assert!(!result.threats.is_empty());
    }
}
