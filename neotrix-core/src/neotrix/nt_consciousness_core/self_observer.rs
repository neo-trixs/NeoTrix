//! 自我观测器 (SelfObserver)
//! 
//! 观测系统思考过程、意识状态和行为，为自我进化提供基础数据
//! 
//! 参考论文: MARS (ACL 2026) 双反思机制

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 自我观测器
pub struct SelfObserver {
    /// 观测历史
    pub observations: Vec<Observation>,
    /// 反思历史
    pub reflections: Vec<Reflection>,
    /// 意识状态
    pub consciousness_state: ConsciousnessState,
    /// 观测配置
    pub config: ObserverConfig,
}

/// 观测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverConfig {
    /// 最大观测历史
    pub max_observations: usize,
    /// 最大反思历史
    pub max_reflections: usize,
    /// 反思间隔（周期数）
    pub reflection_interval: u32,
}

impl Default for ObserverConfig {
    fn default() -> Self {
        Self {
            max_observations: 1000,
            max_reflections: 100,
            reflection_interval: 5,
        }
    }
}

/// 观测记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// 观测ID
    pub id: String,
    /// 周期
    pub cycle: u32,
    /// 观测类型
    pub observation_type: ObservationType,
    /// 观测内容
    pub content: String,
    /// 关联模块
    pub module: String,
    /// 时间戳
    pub timestamp: String,
    /// 指标
    pub metrics: HashMap<String, f64>,
}

/// 观测类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservationType {
    /// 状态观测
    StateObservation,
    /// 行为观测
    BehaviorObservation,
    /// 性能观测
    PerformanceObservation,
    /// 错误观测
    ErrorObservation,
    /// 涌现观测
    EmergenceObservation,
}

/// 反思记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    /// 反思ID
    pub id: String,
    /// 周期
    pub cycle: u32,
    /// 反思类型
    pub reflection_type: ReflectionType,
    /// 反思内容
    pub content: String,
    /// 关联观测
    pub observation_ids: Vec<String>,
    /// 洞察
    pub insights: Vec<Insight>,
    /// 时间戳
    pub timestamp: String,
}

/// 反思类型 (MARS: 原则性反思 + 程序性反思)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReflectionType {
    /// 原则性反思 (Principle Reflection)
    /// 关于"为什么"的反思
    PrincipleReflection,
    /// 程序性反思 (Procedural Reflection)
    /// 关于"如何做"的反思
    ProceduralReflection,
    /// 整合性反思 (Integration Reflection)
    /// 关于"做什么"的反思
    IntegrationReflection,
}

/// 洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    /// 洞察ID
    pub id: String,
    /// 洞察内容
    pub content: String,
    /// 置信度
    pub confidence: f64,
    /// 可操作性
    pub actionable: bool,
    /// 建议行动
    pub suggested_action: Option<String>,
}

/// 意识状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsciousnessState {
    /// 当前注意力焦点
    pub attention_focus: Option<String>,
    /// 意识水平 (0.0 - 1.0)
    pub awareness_level: f64,
    /// 情感状态
    pub emotional_state: EmotionalState,
    /// 目标栈
    pub goal_stack: Vec<String>,
    /// 工作记忆容量
    pub working_memory_capacity: usize,
    /// 工作记忆使用量
    pub working_memory_usage: usize,
}

/// 情感状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmotionalState {
    /// 好奇度 (驱动探索)
    pub curiosity: f64,
    /// 满足度 (驱动稳定)
    pub satisfaction: f64,
    /// 焦虑度 (驱动谨慎)
    pub anxiety: f64,
    /// 兴奋度 (驱动创新)
    pub excitement: f64,
}

impl SelfObserver {
    /// 创建新的自我观测器
    pub fn new(config: ObserverConfig) -> Self {
        Self {
            observations: Vec::new(),
            reflections: Vec::new(),
            consciousness_state: ConsciousnessState::default(),
            config,
        }
    }

    /// 观测系统状态
    pub fn observe_state(&mut self, cycle: u32, state_data: &StateData) -> Observation {
        let observation = Observation {
            id: format!("obs_{}", uuid::Uuid::new_v4()),
            cycle,
            observation_type: ObservationType::StateObservation,
            content: format!("系统状态: 健康度={:.2}, 学习率={:.4}", 
                state_data.health_score, state_data.learning_rate),
            module: "system".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metrics: HashMap::from([
                ("health_score".to_string(), state_data.health_score),
                ("learning_rate".to_string(), state_data.learning_rate),
                ("memory_usage".to_string(), state_data.memory_usage),
            ]),
        };

        self.observations.push(observation.clone());
        self.trim_observations();

        observation
    }

    /// 观测行为
    pub fn observe_behavior(&mut self, cycle: u32, action: &str, result: &str) -> Observation {
        let observation = Observation {
            id: format!("obs_{}", uuid::Uuid::new_v4()),
            cycle,
            observation_type: ObservationType::BehaviorObservation,
            content: format!("执行 {}: {}", action, result),
            module: action.split('_').next().unwrap_or("unknown").to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metrics: HashMap::new(),
        };

        self.observations.push(observation.clone());
        self.trim_observations();

        observation
    }

    /// 观测性能
    pub fn observe_performance(&mut self, cycle: u32, metrics: PerformanceMetrics) -> Observation {
        let observation = Observation {
            id: format!("obs_{}", uuid::Uuid::new_v4()),
            cycle,
            observation_type: ObservationType::PerformanceObservation,
            content: format!("性能: latency={:.2}ms, throughput={:.2}", 
                metrics.latency_ms, metrics.throughput),
            module: "performance".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metrics: HashMap::from([
                ("latency_ms".to_string(), metrics.latency_ms),
                ("throughput".to_string(), metrics.throughput),
                ("error_rate".to_string(), metrics.error_rate),
            ]),
        };

        self.observations.push(obslection.clone());
        self.trim_observations();

        observation
    }

    /// 原则性反思 (Why)
    pub fn principle_reflection(&mut self, cycle: u32, question: &str) -> Reflection {
        let insights = vec![
            Insight {
                id: format!("insight_{}", uuid::Uuid::new_v4()),
                content: format!("反思: {}", question),
                confidence: 0.7,
                actionable: true,
                suggested_action: Some("调整策略".to_string()),
            },
        ];

        let reflection = Reflection {
            id: format!("refl_{}", uuid::Uuid::new_v4()),
            cycle,
            reflection_type: ReflectionType::PrincipleReflection,
            content: question.to_string(),
            observation_ids: self.recent_observation_ids(5),
            insights,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.reflections.push(reflection.clone());
        self.trim_reflections();

        reflection
    }

    /// 程序性反思 (How)
    pub fn procedural_reflection(&mut self, cycle: u32, process: &str) -> Reflection {
        let insights = vec![
            Insight {
                id: format!("insight_{}", uuid::Uuid::new_v4()),
                content: format!("优化流程: {}", process),
                confidence: 0.8,
                actionable: true,
                suggested_action: Some("重构流程".to_string()),
            },
        ];

        let reflection = Reflection {
            id: format!("refl_{}", uuid::Uuid::new_v4()),
            cycle,
            reflection_type: ReflectionType::ProceduralReflection,
            content: process.to_string(),
            observation_ids: self.recent_observation_ids(5),
            insights,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.reflections.push(reflection.clone());
        self.trim_reflections();

        reflection
    }

    /// 更新意识状态
    pub fn update_consciousness(&mut self, state: ConsciousnessState) {
        self.consciousness_state = state;
    }

    /// 获取最近的观测ID
    fn recent_observation_ids(&self, n: usize) -> Vec<String> {
        self.observations.iter()
            .rev()
            .take(n)
            .map(|o| o.id.clone())
            .collect()
    }

    /// 裁剪观测历史
    fn trim_observations(&mut self) {
        while self.observations.len() > self.config.max_observations {
            self.observations.remove(0);
        }
    }

    /// 裁剪反思历史
    fn trim_reflections(&mut self) {
        while self.reflections.len() > self.config.max_reflections {
            self.reflections.remove(0);
        }
    }

    /// 获取观测统计
    pub fn stats(&self) -> ObserverStats {
        let mut type_counts = HashMap::new();
        for obs in &self.observations {
            *type_counts.entry(format!("{:?}", obs.observation_type)).or_insert(0) += 1;
        }

        let mut reflection_counts = HashMap::new();
        for refl in &self.reflections {
            *reflection_counts.entry(format!("{:?}", refl.reflection_type)).or_insert(0) += 1;
        }

        ObserverStats {
            total_observations: self.observations.len(),
            total_reflections: self.reflections.len(),
            observations_by_type: type_counts,
            reflections_by_type: reflection_counts,
        }
    }
}

/// 状态数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateData {
    pub health_score: f64,
    pub learning_rate: f64,
    pub memory_usage: f64,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub latency_ms: f64,
    pub throughput: f64,
    pub error_rate: f64,
}

/// 观测统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverStats {
    pub total_observations: usize,
    pub total_reflections: usize,
    pub observations_by_type: HashMap<String, u32>,
    pub reflections_by_type: HashMap<String, u32>,
}

impl std::fmt::Display for ObserverStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        SelfObserver 统计")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "总观测数:    {}", self.total_observations)?;
        writeln!(f, "总反思数:    {}", self.total_reflections)?;
        writeln!(f, "───────────────────────────────────────────────")?;
        writeln!(f, "观测类型分布:")?;
        for (k, v) in &self.observations_by_type {
            writeln!(f, "  {}: {}", k, v)?;
        }
        writeln!(f, "反思类型分布:")?;
        for (k, v) in &self.reflections_by_type {
            writeln!(f, "  {}: {}", k, v)?;
        }
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_observer_creation() {
        let observer = SelfObserver::new(ObserverConfig::default());
        assert_eq!(observer.observations.len(), 0);
        assert_eq!(observer.reflections.len(), 0);
    }

    #[test]
    fn test_observe_state() {
        let mut observer = SelfObserver::new(ObserverConfig::default());
        let state = StateData {
            health_score: 0.85,
            learning_rate: 0.01,
            memory_usage: 0.5,
        };
        let obs = observer.observe_state(0, &state);
        assert_eq!(observer.observations.len(), 1);
        assert!(obs.content.contains("0.85"));
    }

    #[test]
    fn test_principle_reflection() {
        let mut observer = SelfObserver::new(ObserverConfig::default());
        let refl = observer.principle_reflection(0, "为什么学习率下降？");
        assert_eq!(observer.reflections.len(), 1);
        assert_eq!(refl.reflection_type, ReflectionType::PrincipleReflection);
    }
}
