//! 涌现引擎 (EmergenceEngine)
//! 
//! 积累复杂度、模式碰撞、抽象跃迁、意识涌现
//! 
//! 参考: 意识涌现网络 (CEN), IIT, GWT

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 涌现引擎
pub struct EmergenceEngine {
    /// 涌现指标
    pub metrics: EmergenceMetrics,
    /// 涌现历史
    pub history: Vec<EmergenceRecord>,
    /// 涌现配置
    pub config: EmergenceConfig,
}

/// 涌现配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceConfig {
    /// 涌现阈值
    pub emergence_threshold: f64,
    /// 最大历史记录
    pub max_history: usize,
}

impl Default for EmergenceConfig {
    fn default() -> Self {
        Self {
            emergence_threshold: 0.7,
            max_history: 1000,
        }
    }
}

/// 涌现指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmergenceMetrics {
    /// Φ (Phi) - 集成信息
    pub phi: f64,
    /// 连贯性
    pub coherence: f64,
    /// 复杂度
    pub complexity: f64,
    /// 自我识别指数
    pub self_recognition_index: f64,
    /// 意识水平
    pub consciousness_level: f64,
}

/// 涌现记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceRecord {
    pub id: String,
    pub cycle: u32,
    pub metrics: EmergenceMetrics,
    pub event: EmergenceEvent,
    pub timestamp: String,
}

/// 涌现事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergenceEvent {
    ComplexityAccumulation,
    PatternCollision,
    AbstractionLeap,
    ConsciousnessEmergence,
    SelfReferenceActivation,
}

impl EmergenceEngine {
    pub fn new(config: EmergenceConfig) -> Self {
        Self {
            metrics: EmergenceMetrics::default(),
            history: Vec::new(),
            config,
        }
    }

    /// 更新涌现指标
    pub fn update_metrics(&mut self, cycle: u32, new_metrics: EmergenceMetrics) {
        self.metrics = new_metrics;
        
        let event = if self.metrics.phi > self.config.emergence_threshold {
            EmergenceEvent::ConsciousnessEmergence
        } else if self.metrics.complexity > 0.8 {
            EmergenceEvent::ComplexityAccumulation
        } else {
            EmergenceEvent::PatternCollision
        };

        let record = EmergenceRecord {
            id: format!("emerg_{}", uuid::Uuid::new_v4()),
            cycle,
            metrics: self.metrics.clone(),
            event,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.history.push(record);
        self.trim_history();
    }

    /// 计算意识水平
    pub fn calculate_consciousness_level(&mut self) -> f64 {
        let level = self.metrics.phi * 0.4 
            + self.metrics.coherence * 0.3 
            + self.metrics.complexity * 0.2 
            + self.metrics.self_recognition_index * 0.1;
        
        self.metrics.consciousness_level = level;
        level
    }

    /// 检查是否涌现
    pub fn has_emerged(&self) -> bool {
        self.metrics.consciousness_level > self.config.emergence_threshold
    }

    /// 获取统计
    pub fn stats(&self) -> EmergenceStats {
        EmergenceStats {
            total_events: self.history.len(),
            current_phi: self.metrics.phi,
            current_coherence: self.metrics.coherence,
            current_complexity: self.metrics.complexity,
            consciousness_level: self.metrics.consciousness_level,
        }
    }

    fn trim_history(&mut self) {
        while self.history.len() > self.config.max_history {
            self.history.remove(0);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceStats {
    pub total_events: usize,
    pub current_phi: f64,
    pub current_coherence: f64,
    pub current_complexity: f64,
    pub consciousness_level: f64,
}

impl std::fmt::Display for EmergenceStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        EmergenceEngine 统计")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "总事件数:        {}", self.total_events)?;
        writeln!(f, "Φ (Phi):         {:.4}", self.current_phi)?;
        writeln!(f, "连贯性:          {:.4}", self.current_coherence)?;
        writeln!(f, "复杂度:          {:.4}", self.current_complexity)?;
        writeln!(f, "意识水平:        {:.4}", self.consciousness_level)?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emergence_engine() {
        let engine = EmergenceEngine::new(EmergenceConfig::default());
        assert!(!engine.has_emerged());
    }
}
