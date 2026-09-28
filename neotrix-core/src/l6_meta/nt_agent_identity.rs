//! Agent Identity System — Agent 一等公民身份管理
//!
//! 每个 agent 有持久化 persona (名字/头像/专长/性格/自治级别/成本预算)，
//! 跨会话一致性；KB `agent_identity` namespace 持久化。
//!
//! 设计启发: Cumora (agent-as-teammate) + Munder Difflin (GOD agent routing)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Persona ────────────────────────────────────────────────────────────────

/// Agent persona — 一等公民身份定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPersona {
    /// Unique identifier (UUID)
    pub id: String,
    /// Display name (e.g. "研究员", "编码员", "审查员")
    pub name: String,
    /// Avatar URL or emoji
    pub avatar: String,
    /// Specialty domain (e.g. "code_generation", "research", "review")
    pub specialty: String,
    /// Personality description (e.g. "严谨、注重细节、偏好安全方案")
    pub personality: String,
    /// Autonomy level — how much the agent can do without human approval
    pub autonomy: AutonomyLevel,
    /// Cost budget per session (in USD)
    pub cost_budget: f64,
    /// Current status
    pub status: AgentStatus,
    /// LLM provider preference (e.g. "openai", "anthropic", "ollama")
    pub provider: Option<String>,
    /// Model preference (e.g. "gpt-4o", "claude-sonnet-4-20250514")
    pub model: Option<String>,
    /// Custom system prompt prefix
    pub system_prompt: Option<String>,
    /// Tags for search/filter
    pub tags: Vec<String>,
    /// When this persona was created
    pub created_at: u64,
    /// When this persona was last updated
    pub updated_at: u64,
    /// Session count (how many sessions this agent has participated in)
    pub session_count: u64,
    /// Total cost consumed across all sessions
    pub total_cost: f64,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Autonomy level — controls how much the agent can do without human approval
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AutonomyLevel {
    /// Read-only, no side effects
    ReadOnly,
    /// Can read + write to allowed paths only
    Constrained,
    /// Can read + write + execute with confirmation
    Supervised,
    /// Full autonomy (within cost budget)
    Autonomous,
}

impl AutonomyLevel {
    /// Whether this level allows tool execution
    pub fn allows_tools(&self) -> bool {
        !matches!(self, AutonomyLevel::ReadOnly)
    }

    /// Whether this level allows write operations
    pub fn allows_writes(&self) -> bool {
        matches!(
            self,
            AutonomyLevel::Constrained | AutonomyLevel::Supervised | AutonomyLevel::Autonomous
        )
    }

    /// Whether this level requires human confirmation before writes
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, AutonomyLevel::Supervised)
    }

    /// Numeric score for comparison (higher = more autonomous)
    pub fn score(&self) -> u8 {
        match self {
            AutonomyLevel::ReadOnly => 0,
            AutonomyLevel::Constrained => 1,
            AutonomyLevel::Supervised => 2,
            AutonomyLevel::Autonomous => 3,
        }
    }
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    /// Available for assignment
    Available,
    /// Currently processing a task
    Busy,
    /// Temporarily unavailable (e.g. rate-limited, cooling down)
    Unavailable,
    /// Disabled by user
    Disabled,
}

// ─── Registry ───────────────────────────────────────────────────────────────

/// In-memory agent persona registry with KB persistence
pub struct AgentIdentityRegistry {
    personas: HashMap<String, AgentPersona>,
}

impl AgentIdentityRegistry {
    pub fn new() -> Self {
        Self {
            personas: HashMap::new(),
        }
    }

    /// Register a new persona
    pub fn register(&mut self, persona: AgentPersona) -> Result<(), String> {
        if self.personas.contains_key(&persona.id) {
            return Err(format!("Agent persona '{}' already exists", persona.id));
        }
        self.personas.insert(persona.id.clone(), persona);
        Ok(())
    }

    /// Get a persona by ID
    pub fn get(&self, id: &str) -> Option<&AgentPersona> {
        self.personas.get(id)
    }

    /// Get a mutable persona by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut AgentPersona> {
        self.personas.get_mut(id)
    }

    /// List all personas
    pub fn list(&self) -> Vec<&AgentPersona> {
        self.personas.values().collect()
    }

    /// List personas by status
    pub fn list_available(&self) -> Vec<&AgentPersona> {
        self.personas
            .values()
            .filter(|p| p.status == AgentStatus::Available)
            .collect()
    }

    /// List personas by specialty
    pub fn list_by_specialty(&self, specialty: &str) -> Vec<&AgentPersona> {
        self.personas
            .values()
            .filter(|p| p.specialty == specialty)
            .collect()
    }

    /// Update persona status
    pub fn set_status(&mut self, id: &str, status: AgentStatus) -> Result<(), String> {
        let persona = self
            .personas
            .get_mut(id)
            .ok_or_else(|| format!("Agent persona '{}' not found", id))?;
        persona.status = status;
        persona.updated_at = now_secs();
        Ok(())
    }

    /// Record a session completion (increment count, add cost)
    pub fn record_session(&mut self, id: &str, cost: f64) -> Result<(), String> {
        let persona = self
            .personas
            .get_mut(id)
            .ok_or_else(|| format!("Agent persona '{}' not found", id))?;
        persona.session_count += 1;
        persona.total_cost += cost;
        persona.updated_at = now_secs();
        Ok(())
    }

    /// Remove a persona
    pub fn remove(&mut self, id: &str) -> Option<AgentPersona> {
        self.personas.remove(id)
    }

    /// Select best agent for a task based on specialty match + availability + cost
    pub fn select_best(&self, task_specialty: &str) -> Option<&AgentPersona> {
        let mut candidates: Vec<&AgentPersona> = self
            .personas
            .values()
            .filter(|p| {
                p.status == AgentStatus::Available
                    && p.autonomy.allows_tools()
                    && (p.specialty == task_specialty || p.tags.contains(&task_specialty.to_string()))
            })
            .collect();

        // Sort by: specialty exact match first, then by cost (cheapest first), then by session count (least experienced first)
        candidates.sort_by(|a, b| {
            let a_exact = if a.specialty == task_specialty { 0 } else { 1 };
            let b_exact = if b.specialty == task_specialty { 0 } else { 1 };
            a_exact
                .cmp(&b_exact)
                .then_with(|| a.cost_budget.partial_cmp(&b.cost_budget).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| b.session_count.cmp(&a.session_count)) // prefer less experienced
        });

        candidates.into_iter().next()
    }

    /// Serialize all personas to JSON for KB persistence
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.personas).map_err(|e| e.to_string())
    }

    /// Deserialize personas from JSON
    pub fn from_json(json: &str) -> Result<Self, String> {
        let personas: HashMap<String, AgentPersona> =
            serde_json::from_str(json).map_err(|e| e.to_string())?;
        Ok(Self { personas })
    }
}

// ─── Preset Personas (Agent Gallery) ────────────────────────────────────────

impl AgentPersona {
    /// Create a preset persona from a gallery template
    pub fn from_preset(preset: AgentPreset) -> Self {
        let now = now_secs();
        match preset {
            AgentPreset::Coder => Self {
                id: uuid_v4(),
                name: "编码员".to_string(),
                avatar: "💻".to_string(),
                specialty: "code_generation".to_string(),
                personality: "严谨、高效、偏好简洁实现".to_string(),
                autonomy: AutonomyLevel::Constrained,
                cost_budget: 5.0,
                status: AgentStatus::Available,
                provider: None,
                model: None,
                system_prompt: Some("你是一个专业的编码助手。优先编写简洁、安全的代码。".to_string()),
                tags: vec!["code".to_string(), "rust".to_string(), "typescript".to_string()],
                created_at: now,
                updated_at: now,
                session_count: 0,
                total_cost: 0.0,
                metadata: HashMap::new(),
            },
            AgentPreset::Reviewer => Self {
                id: uuid_v4(),
                name: "审查员".to_string(),
                avatar: "🔍".to_string(),
                specialty: "code_review".to_string(),
                personality: "细致、批判性思维、关注安全和架构".to_string(),
                autonomy: AutonomyLevel::ReadOnly,
                cost_budget: 3.0,
                status: AgentStatus::Available,
                provider: None,
                model: None,
                system_prompt: Some("你是一个代码审查专家。关注安全漏洞、架构问题和代码质量。".to_string()),
                tags: vec!["review".to_string(), "security".to_string(), "architecture".to_string()],
                created_at: now,
                updated_at: now,
                session_count: 0,
                total_cost: 0.0,
                metadata: HashMap::new(),
            },
            AgentPreset::Researcher => Self {
                id: uuid_v4(),
                name: "研究员".to_string(),
                avatar: "🔬".to_string(),
                specialty: "research".to_string(),
                personality: "好奇心强、全面、引用可靠来源".to_string(),
                autonomy: AutonomyLevel::ReadOnly,
                cost_budget: 8.0,
                status: AgentStatus::Available,
                provider: None,
                model: None,
                system_prompt: Some("你是一个研究助手。深入调研，引用可靠来源，避免猜测。".to_string()),
                tags: vec!["research".to_string(), "analysis".to_string(), "writing".to_string()],
                created_at: now,
                updated_at: now,
                session_count: 0,
                total_cost: 0.0,
                metadata: HashMap::new(),
            },
            AgentPreset::Debugger => Self {
                id: uuid_v4(),
                name: "调试员".to_string(),
                avatar: "🐛".to_string(),
                specialty: "debugging".to_string(),
                personality: "系统化、耐心、先复现再修复".to_string(),
                autonomy: AutonomyLevel::Supervised,
                cost_budget: 4.0,
                status: AgentStatus::Available,
                provider: None,
                model: None,
                system_prompt: Some("你是一个调试专家。系统化排查：复现→定位→修复→验证。".to_string()),
                tags: vec!["debug".to_string(), "diagnostic".to_string(), "troubleshoot".to_string()],
                created_at: now,
                updated_at: now,
                session_count: 0,
                total_cost: 0.0,
                metadata: HashMap::new(),
            },
            AgentPreset::Architect => Self {
                id: uuid_v4(),
                name: "架构师".to_string(),
                avatar: "🏗️".to_string(),
                specialty: "architecture".to_string(),
                personality: "全局视野、权衡利弊、文档驱动".to_string(),
                autonomy: AutonomyLevel::Supervised,
                cost_budget: 10.0,
                status: AgentStatus::Available,
                provider: None,
                model: None,
                system_prompt: Some("你是一个系统架构师。设计模块边界、数据流、API 契约。".to_string()),
                tags: vec!["architecture".to_string(), "design".to_string(), "system".to_string()],
                created_at: now,
                updated_at: now,
                session_count: 0,
                total_cost: 0.0,
                metadata: HashMap::new(),
            },
            AgentPreset::Writer => Self {
                id: uuid_v4(),
                name: "写作者".to_string(),
                avatar: "✍️".to_string(),
                specialty: "documentation".to_string(),
                personality: "清晰、结构化、面向读者".to_string(),
                autonomy: AutonomyLevel::Constrained,
                cost_budget: 3.0,
                status: AgentStatus::Available,
                provider: None,
                model: None,
                system_prompt: Some("你是一个技术写作者。创建清晰、结构化的文档和注释。".to_string()),
                tags: vec!["docs".to_string(), "writing".to_string(), "tutorial".to_string()],
                created_at: now,
                updated_at: now,
                session_count: 0,
                total_cost: 0.0,
                metadata: HashMap::new(),
            },
        }
    }
}

/// Preset agent gallery templates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentPreset {
    Coder,
    Reviewer,
    Researcher,
    Debugger,
    Architect,
    Writer,
}

impl AgentPreset {
    /// All available presets
    pub fn all() -> Vec<Self> {
        vec![
            Self::Coder,
            Self::Reviewer,
            Self::Researcher,
            Self::Debugger,
            Self::Architect,
            Self::Writer,
        ]
    }

    /// Display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Coder => "编码员",
            Self::Reviewer => "审查员",
            Self::Researcher => "研究员",
            Self::Debugger => "调试员",
            Self::Architect => "架构师",
            Self::Writer => "写作者",
        }
    }

    /// Description
    pub fn description(&self) -> &str {
        match self {
            Self::Coder => "高效的编码助手，编写简洁安全的代码",
            Self::Reviewer => "细致的代码审查专家，关注安全和架构",
            Self::Researcher => "全面的研究助手，引用可靠来源",
            Self::Debugger => "系统化调试专家，先复现再修复",
            Self::Architect => "全局架构师，设计模块边界和数据流",
            Self::Writer => "技术写作者，创建清晰的文档",
        }
    }
}

// ─── Cost Gate ──────────────────────────────────────────────────────────────

/// Spend gate — 人类审批阈值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendGate {
    /// Per-session cost limit (USD) — agent cannot exceed without human approval
    pub session_limit: f64,
    /// Per-day cost limit (USD)
    pub daily_limit: f64,
    /// Per-agent lifetime cost limit (USD)
    pub lifetime_limit: f64,
    /// Whether to auto-disable agent when budget exceeded
    pub auto_disable: bool,
}

impl Default for SpendGate {
    fn default() -> Self {
        Self {
            session_limit: 10.0,
            daily_limit: 50.0,
            lifetime_limit: 1000.0,
            auto_disable: true,
        }
    }
}

impl SpendGate {
    /// Check if a proposed cost is within budget
    pub fn check(&self, proposed: f64, session_spent: f64, daily_spent: f64, lifetime_spent: f64) -> SpendVerdict {
        let total_session = session_spent + proposed;
        let total_daily = daily_spent + proposed;
        let total_lifetime = lifetime_spent + proposed;

        if total_session > self.session_limit {
            return SpendVerdict::ExceedsSessionLimit {
                proposed,
                limit: self.session_limit,
                current: session_spent,
            };
        }
        if total_daily > self.daily_limit {
            return SpendVerdict::ExceedsDailyLimit {
                proposed,
                limit: self.daily_limit,
                current: daily_spent,
            };
        }
        if total_lifetime > self.lifetime_limit {
            return SpendVerdict::ExceedsLifetimeLimit {
                proposed,
                limit: self.lifetime_limit,
                current: lifetime_spent,
            };
        }
        SpendVerdict::Allowed {
            remaining_session: self.session_limit - total_session,
            remaining_daily: self.daily_limit - total_daily,
        }
    }
}

/// Spend gate verdict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpendVerdict {
    Allowed {
        remaining_session: f64,
        remaining_daily: f64,
    },
    ExceedsSessionLimit {
        proposed: f64,
        limit: f64,
        current: f64,
    },
    ExceedsDailyLimit {
        proposed: f64,
        limit: f64,
        current: f64,
    },
    ExceedsLifetimeLimit {
        proposed: f64,
        limit: f64,
        current: f64,
    },
}

impl SpendVerdict {
    pub fn is_allowed(&self) -> bool {
        matches!(self, SpendVerdict::Allowed { .. })
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

pub use crate::l0_substrate::nt_core_time::now_secs;

fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autonomy_levels() {
        assert!(!AutonomyLevel::ReadOnly.allows_tools());
        assert!(AutonomyLevel::Constrained.allows_tools());
        assert!(AutonomyLevel::Constrained.allows_writes());
        assert!(AutonomyLevel::Supervised.requires_confirmation());
        assert!(!AutonomyLevel::Autonomous.requires_confirmation());
    }

    #[test]
    fn test_registry_crud() {
        let mut reg = AgentIdentityRegistry::new();
        let persona = AgentPersona::from_preset(AgentPreset::Coder);
        let id = persona.id.clone();

        assert!(reg.register(persona).is_ok());
        assert!(reg.get(&id).is_some());
        assert_eq!(reg.list().len(), 1);

        assert!(reg.set_status(&id, AgentStatus::Busy).is_ok());
        assert_eq!(reg.get(&id).unwrap().status, AgentStatus::Busy);

        assert!(reg.record_session(&id, 0.5).is_ok());
        assert_eq!(reg.get(&id).unwrap().session_count, 1);
        assert_eq!(reg.get(&id).unwrap().total_cost, 0.5);

        assert!(reg.remove(&id).is_some());
        assert!(reg.get(&id).is_none());
    }

    #[test]
    fn test_registry_duplicate_rejected() {
        let mut reg = AgentIdentityRegistry::new();
        let p1 = AgentPersona::from_preset(AgentPreset::Coder);
        let p2 = AgentPersona::from_preset(AgentPreset::Reviewer);
        // Force same ID
        let mut p2 = p2;
        p2.id = p1.id.clone();

        assert!(reg.register(p1).is_ok());
        assert!(reg.register(p2).is_err());
    }

    #[test]
    fn test_select_best() {
        let mut reg = AgentIdentityRegistry::new();

        let mut p1 = AgentPersona::from_preset(AgentPreset::Coder);
        p1.cost_budget = 5.0;
        p1.status = AgentStatus::Available;

        let mut p2 = AgentPersona::from_preset(AgentPreset::Coder);
        p2.cost_budget = 2.0;
        p2.status = AgentStatus::Available;

        let mut p3 = AgentPersona::from_preset(AgentPreset::Reviewer);
        p3.status = AgentStatus::Available;

        reg.register(p1).unwrap();
        reg.register(p2).unwrap();
        reg.register(p3).unwrap();

        // Should select cheapest coder
        let best = reg.select_best("code_generation").unwrap();
        assert_eq!(best.cost_budget, 2.0);

        // Should not select reviewer for code task
        let best = reg.select_best("code_generation").unwrap();
        assert_ne!(best.specialty, "code_review");
    }

    #[test]
    fn test_preset_gallery() {
        let presets = AgentPreset::all();
        assert_eq!(presets.len(), 6);

        for preset in presets {
            let persona = AgentPersona::from_preset(preset);
            assert!(!persona.id.is_empty());
            assert!(!persona.name.is_empty());
            assert!(!persona.avatar.is_empty());
        }
    }

    #[test]
    fn test_spend_gate() {
        let gate = SpendGate {
            session_limit: 10.0,
            daily_limit: 50.0,
            lifetime_limit: 1000.0,
            auto_disable: true,
        };

        // Under limit
        let v = gate.check(5.0, 0.0, 0.0, 0.0);
        assert!(v.is_allowed());

        // Over session limit
        let v = gate.check(5.0, 8.0, 0.0, 0.0);
        assert!(!v.is_allowed());

        // Over daily limit
        let v = gate.check(5.0, 0.0, 48.0, 0.0);
        assert!(!v.is_allowed());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut reg = AgentIdentityRegistry::new();
        reg.register(AgentPersona::from_preset(AgentPreset::Coder))
            .unwrap();
        reg.register(AgentPersona::from_preset(AgentPreset::Researcher))
            .unwrap();

        let json = reg.to_json().unwrap();
        let reg2 = AgentIdentityRegistry::from_json(&json).unwrap();

        assert_eq!(reg.list().len(), reg2.list().len());
    }
}
