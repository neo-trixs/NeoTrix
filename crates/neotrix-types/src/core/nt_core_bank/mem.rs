use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::core::nt_core_edit::{MicroEdit, SelfEdit};
use crate::core::{RewardSource, TaskType};
use crate::core::nt_core_bank::{MemoryTier, MemoryLifecycle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum T3ViewType {
    Struct,
    Semantic,
    Reflect,
}

impl T3ViewType {
    pub fn all() -> &'static [T3ViewType] {
        &[T3ViewType::Struct, T3ViewType::Semantic, T3ViewType::Reflect]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T3Views {
    pub struct_view: Option<String>,
    pub semantic_view: Option<String>,
    pub reflect_view: Option<String>,
}

impl Default for T3Views {
    fn default() -> Self {
        Self::new()
    }
}

impl T3Views {
    pub fn new() -> Self {
        Self { struct_view: None, semantic_view: None, reflect_view: None }
    }

    pub fn get(&self, view_type: T3ViewType) -> Option<&str> {
        match view_type {
            T3ViewType::Struct => self.struct_view.as_deref(),
            T3ViewType::Semantic => self.semantic_view.as_deref(),
            T3ViewType::Reflect => self.reflect_view.as_deref(),
        }
    }

    pub fn from_memory(mem: &ReasoningMemory) -> Self {
        let struct_view = Self::generate_struct(mem);
        let semantic_view = Self::generate_semantic(mem);
        let reflect_view = Self::generate_reflect(mem);
        Self { struct_view, semantic_view, reflect_view }
    }

    fn generate_struct(mem: &ReasoningMemory) -> Option<String> {
        if mem.micro_edits.is_empty() { return None; }
        let mut steps: Vec<String> = Vec::new();
        for (i, edit) in mem.micro_edits.iter().enumerate() {
            steps.push(format!("  {}. {}", i + 1, edit.summary()));
        }
        Some(format!("Task: {}\nSteps:\n{}", mem.task_description, steps.join("\n")))
    }

    fn generate_semantic(mem: &ReasoningMemory) -> Option<String> {
        let outcome = if mem.reward > 0.7 { "successful" } else if mem.reward > 0.4 { "partial" } else { "failed" };
        let key_idea = mem.micro_edits.first().map(|e| format!(" key action: {}", e.summary())).unwrap_or_default();
        Some(format!("[{}] {} (reward={:.2}){}", outcome, mem.task_description, mem.reward, key_idea))
    }

    fn generate_reflect(mem: &ReasoningMemory) -> Option<String> {
        if mem.success { return None; }
        let mistake_desc = mem.micro_edits.last()
            .map(|e| format!("possible misstep: {}", e.summary()))
            .unwrap_or_else(|| "unknown failure pattern".to_string());
        Some(format!("FAILURE: {} — reward={:.2}\n  {}", mem.task_description, mem.reward, mistake_desc))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum MemorySource {
    #[default]
    User,
    Model,
    Tool,
    Consolidator,
    External,
    Unknown,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningMemory {
    pub id: String,
    pub task_description: String,
    pub task_type: TaskType,
    pub micro_edits: Vec<MicroEdit>,
    pub reward: f64,
    pub reward_source: RewardSource,
    pub success: bool,
    pub timestamp: i64,
    pub embedding: Option<Vec<f64>>,
    pub tier: MemoryTier,
    pub lifecycle: MemoryLifecycle,
    pub t3_views: T3Views,
    pub confidence: f64,
    pub source: MemorySource,
    pub last_used_at: i64,
    pub conflict_group: String,
    pub verification_time: i64,
}

impl ReasoningMemory {
    pub fn new(task: &str, task_type: TaskType, edits: &[MicroEdit], reward: f64) -> Self {
        let now = Utc::now().timestamp();
        let mut mem = Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_description: task.to_string(),
            task_type,
            micro_edits: edits.to_vec(),
            reward,
            reward_source: RewardSource::Internal,
            success: reward > 0.5,
            timestamp: now,
            embedding: None,
            tier: MemoryTier::Working,
            lifecycle: MemoryLifecycle::new(reward),
            t3_views: T3Views::new(),
            confidence: 1.0,
            source: MemorySource::User,
            last_used_at: 0,
            conflict_group: String::new(),
            verification_time: 0,
        };
        mem.t3_views = T3Views::from_memory(&mem);
        mem
    }

    pub fn with_external_reward(task: &str, task_type: TaskType, edits: &[MicroEdit], reward: f64) -> Self {
        let mut m = Self::new(task, task_type, edits, reward);
        m.reward_source = RewardSource::External;
        m
    }

    pub fn from_self_edit(task: &str, task_type: TaskType, edit: &SelfEdit, reward: f64) -> Self {
        let mut micro_edits = Vec::new();
        for dim in &edit.target_dimensions {
            micro_edits.push(MicroEdit::AdjustDimension(dim.clone(), edit.adjustment_magnitude));
        }
        micro_edits.push(MicroEdit::UpdateLearningRate(*edit.config_overrides.get("learning_rate").unwrap_or(&0.05)));
        micro_edits.push(MicroEdit::NormalizeVector);
        Self::new(task, task_type, &micro_edits, reward)
    }

    pub fn with_tier(mut self, tier: MemoryTier) -> Self {
        self.tier = tier;
        self
    }

    pub fn with_ttl(mut self, ttl_seconds: i64) -> Self {
        self.lifecycle.ttl_seconds = Some(ttl_seconds);
        self
    }

    pub fn touch(&mut self) {
        self.lifecycle.touch();
        self.timestamp = Utc::now().timestamp();
        self.last_used_at = self.timestamp;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    pub task_id: String,
    pub recent_memories: Vec<ReasoningMemory>,
    pub trend: String,
    pub last_reward: f64,
    pub avg_reward: f64,
    pub memory_count: usize,
}

impl TemporalContext {
    pub fn new(task_id: &str) -> Self {
        Self { task_id: task_id.to_string(), recent_memories: Vec::new(), trend: "stable".to_string(), last_reward: 0.0, avg_reward: 0.0, memory_count: 0 }
    }

    pub fn to_prompt_hint(&self) -> String {
        if self.recent_memories.is_empty() {
            return "  (no temporal context)".to_string();
        }
        format!(
            "Temporal context for '{}': {} memories, avg reward {:.2}, trend: {}",
            self.task_id, self.memory_count, self.avg_reward, self.trend
        )
    }
}
