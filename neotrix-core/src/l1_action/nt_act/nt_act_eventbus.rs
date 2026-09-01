//! Unified EventBus — 跨层通信基础设施
//!
//! 吸收 KB 经验:
//! - 跨层事件路由 (L1↔L2↔L3↔L4↔L5↔L6)
//! - 发布/订阅模式
//! - 事件过滤/转换
//! - 死信队列
//! - 事件溯源

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

/// 统一事件总线 — 跨层通信核心
pub struct EventBus {
    subscribers: HashMap<String, Vec<Box<dyn Fn(&Event) + Send + Sync>>>,
    event_log: Vec<Event>,
    dead_letter_queue: Vec<Event>,
    filters: Vec<EventFilter>,
    max_log_size: usize,
    stats: EventBusStats,
}

/// 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub event_type: EventType,
    pub source_layer: String,
    pub target_layer: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub priority: EventPriority,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // L1 Action
    ToolCall,
    ToolResult,
    ActionTrigger,
    // L2 Perception
    SensoryInput,
    PerceptionUpdate,
    WorldModelChange,
    // L3 Embodiment
    SecurityAlert,
    PhysicalStateChange,
    SafetyViolation,
    // L4 Emotion
    EmotionDetected,
    EmotionRegulated,
    PersonaSwitch,
    // L5 Cognition
    ReasoningTask,
    DecisionMade,
    KnowledgeUpdate,
    // L6 Meta
    MetaReflection,
    SelfHealTrigger,
    EvolutionCycle,
    // Cross-layer
    CrossLayerSync,
    SystemHealth,
    ConfigChange,
}

/// 事件优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// 事件过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    pub filter_type: FilterType,
    pub pattern: String,
    pub action: FilterAction,
}

/// 过滤器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterType {
    EventType,
    SourceLayer,
    TargetLayer,
    Priority,
    Custom,
}

/// 过滤动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterAction {
    Allow,
    Deny,
    Transform,
    Log,
}

/// 事件总线统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusStats {
    pub total_events: u64,
    pub delivered_events: u64,
    pub dead_letters: u64,
    pub filtered_events: u64,
    pub avg_latency_ms: f64,
}

/// 订阅句柄
pub struct SubscriptionHandle {
    id: String,
    event_type: String,
    layer: String,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
            event_log: Vec::new(),
            dead_letter_queue: Vec::new(),
            filters: Vec::new(),
            max_log_size: 10000,
            stats: EventBusStats {
                total_events: 0,
                delivered_events: 0,
                dead_letters: 0,
                filtered_events: 0,
                avg_latency_ms: 0.0,
            },
        }
    }

    /// 发布事件
    pub fn publish(&mut self, event: Event) -> Result<(), String> {
        self.stats.total_events += 1;

        // 应用过滤器
        for filter in &self.filters {
            match self.apply_filter(filter, &event) {
                FilterAction::Deny => {
                    self.stats.filtered_events += 1;
                    return Ok(());
                }
                FilterAction::Log => {
                    self.event_log.push(event.clone());
                }
                _ => {}
            }
        }

        // 记录事件
        if self.event_log.len() >= self.max_log_size {
            self.event_log.remove(0);
        }
        self.event_log.push(event.clone());

        // 分发给订阅者
        let key = format!("{}:{}", event.event_type.to_string(), event.target_layer.as_deref().unwrap_or("all"));
        if let Some(handlers) = self.subscribers.get(&key) {
            for handler in handlers {
                handler(&event);
                self.stats.delivered_events += 1;
            }
        }

        // 也检查通配符订阅
        let wildcard_key = format!("{}:all", event.event_type.to_string());
        if let Some(handlers) = self.subscribers.get(&wildcard_key) {
            for handler in handlers {
                handler(&event);
                self.stats.delivered_events += 1;
            }
        }

        Ok(())
    }

    /// 订阅事件
    pub fn subscribe<F>(
        &mut self,
        event_type: &str,
        layer: &str,
        handler: F,
    ) -> SubscriptionHandle
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        let key = format!("{}:{}", event_type, layer);
        self.subscribers
            .entry(key)
            .or_insert_with(Vec::new)
            .push(Box::new(handler));

        SubscriptionHandle {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            layer: layer.to_string(),
        }
    }

    /// 添加过滤器
    pub fn add_filter(&mut self, filter: EventFilter) {
        self.filters.push(filter);
    }

    /// 应用过滤器
    fn apply_filter(&self, filter: &EventFilter, event: &Event) -> FilterAction {
        match filter.filter_type {
            FilterType::EventType => {
                if event.event_type.to_string() == filter.pattern {
                    filter.action.clone()
                } else {
                    FilterAction::Allow
                }
            }
            FilterType::SourceLayer => {
                if event.source_layer == filter.pattern {
                    filter.action.clone()
                } else {
                    FilterAction::Allow
                }
            }
            FilterType::TargetLayer => {
                if event.target_layer.as_deref() == Some(&filter.pattern) {
                    filter.action.clone()
                } else {
                    FilterAction::Allow
                }
            }
            _ => FilterAction::Allow,
        }
    }

    /// 获取事件日志
    pub fn get_event_log(&self) -> &[Event] {
        &self.event_log
    }

    /// 获取死信队列
    pub fn get_dead_letters(&self) -> &[Event] {
        &self.dead_letter_queue
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &EventBusStats {
        &self.stats
    }

    /// 清理旧事件
    pub fn cleanup(&mut self, max_age: chrono::Duration) {
        let cutoff = chrono::Utc::now() - max_age;
        self.event_log.retain(|e| e.timestamp > cutoff);
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::ToolCall => write!(f, "tool_call"),
            EventType::ToolResult => write!(f, "tool_result"),
            EventType::ActionTrigger => write!(f, "action_trigger"),
            EventType::SensoryInput => write!(f, "sensory_input"),
            EventType::PerceptionUpdate => write!(f, "perception_update"),
            EventType::WorldModelChange => write!(f, "world_model_change"),
            EventType::SecurityAlert => write!(f, "security_alert"),
            EventType::PhysicalStateChange => write!(f, "physical_state_change"),
            EventType::SafetyViolation => write!(f, "safety_violation"),
            EventType::EmotionDetected => write!(f, "emotion_detected"),
            EventType::EmotionRegulated => write!(f, "emotion_regulated"),
            EventType::PersonaSwitch => write!(f, "persona_switch"),
            EventType::ReasoningTask => write!(f, "reasoning_task"),
            EventType::DecisionMade => write!(f, "decision_made"),
            EventType::KnowledgeUpdate => write!(f, "knowledge_update"),
            EventType::MetaReflection => write!(f, "meta_reflection"),
            EventType::SelfHealTrigger => write!(f, "self_heal_trigger"),
            EventType::EvolutionCycle => write!(f, "evolution_cycle"),
            EventType::CrossLayerSync => write!(f, "cross_layer_sync"),
            EventType::SystemHealth => write!(f, "system_health"),
            EventType::ConfigChange => write!(f, "config_change"),
        }
    }
}
