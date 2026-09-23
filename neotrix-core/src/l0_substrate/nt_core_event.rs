use serde::{Deserialize, Serialize};

/// Core event enum — type-safe, no `dyn Any` downcasting.
/// Each variant carries typed payload directly.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreEvent {
    #[serde(rename = "task_submitted")]
    TaskSubmitted {
        task: String,
        task_type: String,
        priority: u32,
    },
    #[serde(rename = "agent_feedback")]
    AgentFeedback {
        agent_id: String,
        feedback: String,
        score: f64,
    },
    #[serde(rename = "global_halt")]
    GlobalHalt { reason: String, source: String },
    #[serde(rename = "external_reward")]
    ExternalReward { reward: f64, source: String },
    #[serde(rename = "goal_completed")]
    GoalCompleted {
        goal_id: String,
        goal: String,
        iterations: u64,
        score: f64,
    },
    #[serde(rename = "budget_exceeded")]
    BudgetExceeded {
        goal_id: String,
        budget_used: f64,
        max_budget: f64,
    },
    #[serde(rename = "agent_team")]
    AgentTeam {
        agent_id: String,
        action: String,
        timestamp: i64,
    },
    #[serde(rename = "system_error")]
    SystemError {
        component: String,
        error: String,
        severity: String,
    },
    #[serde(rename = "consciousness_critique")]
    ConsciousnessCritique {
        quality: f64,
        relevance: f64,
        consistency: f64,
        timestamp: i64,
    },
    #[serde(rename = "game_session_created")]
    GameSessionCreated {
        game_name: String,
        session_id: u64,
    },
    #[serde(rename = "game_episode_completed")]
    GameEpisodeCompleted {
        game_name: String,
        episode_id: u64,
        score: f64,
        turns: usize,
    },
    #[serde(rename = "game_training_update")]
    GameTrainingUpdate {
        game_name: String,
        iteration: usize,
        policy_loss: f64,
        win_rate: f64,
    },
    #[serde(rename = "game_consciousness_feedback")]
    GameConsciousnessFeedback {
        phi_delta: f64,
        emotion_label: String,
        attention_shift: String,
    },

    // ── NT-CORE (E8引导者) ──────────────────────────────────────────────
    #[serde(rename = "core_boot_started")]
    CoreBootStarted { phase: String },
    #[serde(rename = "core_consciousness_shift")]
    ConsciousnessShift {
        phi_before: f64,
        phi_after: f64,
        coherence: f64,
    },

    // ── NT-MIND (进化工匠) ─────────────────────────────────────────────
    #[serde(rename = "mind_seal_iteration")]
    MindSealIteration {
        cycle: u64,
        quality_delta: f64,
        status: String,
    },
    #[serde(rename = "mind_distillation")]
    MindDistillation {
        source_skill: String,
        output_crystal: String,
    },

    // ── NT-MEMORY (知识守护者) ─────────────────────────────────────────
    #[serde(rename = "memory_kb_write")]
    MemoryKbWrite {
        namespace: String,
        key: String,
        bytes: u64,
    },
    #[serde(rename = "memory_kb_query")]
    MemoryKbQuery {
        namespace: String,
        hit: bool,
        latency_ms: f64,
    },

    // ── NT-WORLD (虚空探索者) ───────────────────────────────────────────
    #[serde(rename = "world_crawl_completed")]
    WorldCrawlCompleted {
        source: String,
        pages: u32,
        errors: u32,
    },
    #[serde(rename = "world_fetch_error")]
    WorldFetchError {
        url: String,
        status_code: u16,
        retry: bool,
    },

    // ── NT-ACT (行动执行者) ────────────────────────────────────────────
    #[serde(rename = "act_tool_invocation")]
    ActToolInvocation {
        tool_name: String,
        success: bool,
        duration_ms: f64,
    },
    #[serde(rename = "act_goal_progress")]
    ActGoalProgress {
        goal_id: String,
        progress: f64,
        milestone: String,
    },

    // ── NT-IO (界面使徒) ──────────────────────────────────────────────
    #[serde(rename = "io_provider_switch")]
    IoProviderSwitch {
        from: String,
        to: String,
        reason: String,
    },
    #[serde(rename = "io_request_error")]
    IoRequestError {
        provider: String,
        error: String,
        retry_count: u32,
    },

    // ── NT-ACT / NT-MEDIA (下载进度) ────────────────────────────────────
    #[serde(rename = "download_progress")]
    DownloadProgress {
        url: String,
        status: String,
        downloaded: u64,
        total: Option<u64>,
        speed_bps: f64,
        output: String,
    },

    // ── NT-SHIELD (影卫) ──────────────────────────────────────────────
    #[serde(rename = "shield_intrusion_detected")]
    ShieldIntrusionDetected {
        source_ip: String,
        rule: String,
        severity: String,
    },
    #[serde(rename = "shield_audit_completed")]
    ShieldAuditCompleted {
        dimensions: u32,
        findings: u32,
        score: f64,
    },

    /// 领域事件 (trade/process 等)
    #[serde(rename = "domain_event")]
    DomainEvent {
        domain: String,
        event_type: String,
        payload: serde_json::Value,
    },

    // ── T28 实体生命周期 (E3) ───────────────────────────────────────────
    #[serde(rename = "workspace_switched")]
    WorkspaceSwitched {
        workspace_id: String,
        agent_count: usize,
        skill_count: usize,
    },
    #[serde(rename = "workspace_agent_added")]
    WorkspaceAgentAdded {
        workspace_id: String,
        agent_id: String,
    },
    #[serde(rename = "workspace_agent_removed")]
    WorkspaceAgentRemoved {
        workspace_id: String,
        agent_id: String,
    },
    #[serde(rename = "agent_status_changed")]
    AgentStatusChanged {
        agent_id: String,
        old_status: String,
        new_status: String,
    },
    #[serde(rename = "agent_task_delegated")]
    AgentTaskDelegated {
        from_agent: String,
        to_agent: String,
        task_id: String,
    },
    #[serde(rename = "skill_installed")]
    SkillInstalled {
        skill_id: String,
        workspace_id: String,
        version: u32,
    },
    #[serde(rename = "skill_executed")]
    SkillExecuted {
        skill_id: String,
        agent_id: String,
        success: bool,
        duration_ms: u64,
    },
    #[serde(rename = "skill_maturity_changed")]
    SkillMaturityChanged {
        skill_id: String,
        old_maturity: String,
        new_maturity: String,
    },
    #[serde(rename = "task_assigned")]
    TaskAssigned {
        task_id: String,
        agent_id: String,
        workspace_id: String,
    },
    #[serde(rename = "task_completed_v2")]
    TaskCompletedV2 {
        task_id: String,
        execution_id: String,
        success: bool,
        deliverables: Vec<String>,
    },
    #[serde(rename = "mcp_tool_called")]
    McpToolCalled {
        tool_name: String,
        server_name: String,
        agent_id: String,
        latency_ms: u64,
        success: bool,
    },
    #[serde(rename = "mcp_server_connected")]
    McpServerConnected {
        server_name: String,
        tool_count: usize,
    },
    #[serde(rename = "upgrade_available")]
    UpgradeAvailable {
        agent_id: String,
        installed_version: String,
        latest_version: String,
    },
}

// ── Backward-compatible type aliases ──────────────────────────────────────
pub type BusEvent = CoreEvent;

// ── Tests ────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_submitted() {
        let e = CoreEvent::TaskSubmitted {
            task: "t".into(),
            task_type: "g".into(),
            priority: 1,
        };
        assert_eq!(e.task(), "t");
    }

    #[test]
    fn test_agent_feedback() {
        let e = CoreEvent::AgentFeedback {
            agent_id: "a1".into(),
            feedback: "good".into(),
            score: 0.9,
        };
        assert_eq!(e.agent_id(), "a1");
    }

    #[test]
    fn test_global_halt() {
        let e = CoreEvent::GlobalHalt {
            reason: "err".into(),
            source: "test".into(),
        };
        assert_eq!(e.reason(), "err");
    }

    #[test]
    fn test_external_reward() {
        let e = CoreEvent::ExternalReward {
            reward: 1.0,
            source: "env".into(),
        };
        assert_eq!(e.reward(), 1.0);
    }

    #[test]
    fn test_goal_completed() {
        let e = CoreEvent::GoalCompleted {
            goal_id: "g1".into(),
            goal: "test".into(),
            iterations: 5,
            score: 0.8,
        };
        assert_eq!(e.goal_id(), "g1");
    }

    #[test]
    fn test_budget_exceeded() {
        let e = CoreEvent::BudgetExceeded {
            goal_id: "g1".into(),
            budget_used: 100.0,
            max_budget: 50.0,
        };
        assert!(e.budget_used() > e.max_budget());
    }

    #[test]
    fn test_agent_team() {
        let e = CoreEvent::AgentTeam {
            agent_id: "a1".into(),
            action: "join".into(),
            timestamp: 1000,
        };
        assert_eq!(e.action(), "join");
    }

    #[test]
    fn test_system_error() {
        let e = CoreEvent::SystemError {
            component: "db".into(),
            error: "timeout".into(),
            severity: "critical".into(),
        };
        assert_eq!(e.severity(), "critical");
    }

    #[test]
    fn test_consciousness_critique() {
        let e = CoreEvent::ConsciousnessCritique {
            quality: 0.7,
            relevance: 0.8,
            consistency: 0.9,
            timestamp: 1000,
        };
        assert!((e.quality() - 0.7).abs() < 0.01);
        assert!((e.relevance() - 0.8).abs() < 0.01);
        assert!((e.consistency() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_json_roundtrip() -> Result<(), String> {
        let e = CoreEvent::GoalCompleted {
            goal_id: "g1".into(),
            goal: "test".into(),
            iterations: 5,
            score: 0.8,
        };
        let json = serde_json::to_string(&e).unwrap();
        let parsed: CoreEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            CoreEvent::GoalCompleted {
                goal_id,
                goal: _,
                iterations,
                score: _,
            } => {
                assert_eq!(goal_id, "g1");
                assert_eq!(iterations, 5);
            }
            _ => return Err("wrong variant".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_workspace_switched_serde_roundtrip() -> Result<(), String> {
        let e = CoreEvent::WorkspaceSwitched {
            workspace_id: "ws1".into(),
            agent_count: 2,
            skill_count: 3,
        };
        let v = serde_json::to_value(&e).map_err(|e| e.to_string())?;
        match v.get("type").and_then(|t| t.as_str()) {
            Some("workspace_switched") => {}
            other => return Err(format!("wrong tag: {:?}", other)),
        }
        let parsed: CoreEvent = serde_json::from_value(v).map_err(|e| e.to_string())?;
        match parsed {
            CoreEvent::WorkspaceSwitched {
                workspace_id,
                agent_count,
                skill_count,
            } => {
                assert_eq!(workspace_id, "ws1");
                assert_eq!(agent_count, 2);
                assert_eq!(skill_count, 3);
            }
            _ => return Err("wrong variant".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_skill_executed_serde_roundtrip() -> Result<(), String> {
        let e = CoreEvent::SkillExecuted {
            skill_id: "sk1".into(),
            agent_id: "a1".into(),
            success: true,
            duration_ms: 120,
        };
        let v = serde_json::to_value(&e).map_err(|e| e.to_string())?;
        match v.get("type").and_then(|t| t.as_str()) {
            Some("skill_executed") => {}
            other => return Err(format!("wrong tag: {:?}", other)),
        }
        let parsed: CoreEvent = serde_json::from_value(v).map_err(|e| e.to_string())?;
        match parsed {
            CoreEvent::SkillExecuted {
                skill_id,
                agent_id,
                success,
                duration_ms,
            } => {
                assert_eq!(skill_id, "sk1");
                assert_eq!(agent_id, "a1");
                assert!(success);
                assert_eq!(duration_ms, 120);
            }
            _ => return Err("wrong variant".to_string()),
        }
        Ok(())
    }
}

// ── Accessor impls on enum ───────────────────────────────────────────────
// Avoid external pattern matching churn by providing field-level accessors.
impl CoreEvent {
    pub fn task(&self) -> &str {
        match self {
            Self::TaskSubmitted { task, .. } => task,
            _ => "",
        }
    }
    pub fn task_type(&self) -> &str {
        match self {
            Self::TaskSubmitted { task_type, .. } => task_type,
            _ => "",
        }
    }
    pub fn priority(&self) -> u32 {
        match self {
            Self::TaskSubmitted { priority, .. } => *priority,
            _ => 0,
        }
    }
    pub fn agent_id(&self) -> &str {
        match self {
            Self::AgentFeedback { agent_id, .. } => agent_id,
            Self::AgentTeam { agent_id, .. } => agent_id,
            _ => "",
        }
    }
    pub fn feedback(&self) -> &str {
        match self {
            Self::AgentFeedback { feedback, .. } => feedback,
            _ => "",
        }
    }
    pub fn score(&self) -> f64 {
        match self {
            Self::AgentFeedback { score, .. } => *score,
            _ => 0.0,
        }
    }
    pub fn reason(&self) -> &str {
        match self {
            Self::GlobalHalt { reason, .. } => reason,
            _ => "",
        }
    }
    pub fn source(&self) -> &str {
        match self {
            Self::GlobalHalt { source, .. } => source,
            Self::ExternalReward { source, .. } => source,
            _ => "",
        }
    }
    pub fn reward(&self) -> f64 {
        match self {
            Self::ExternalReward { reward, .. } => *reward,
            _ => 0.0,
        }
    }
    pub fn goal_id(&self) -> &str {
        match self {
            Self::GoalCompleted { goal_id, .. } => goal_id,
            Self::BudgetExceeded { goal_id, .. } => goal_id,
            _ => "",
        }
    }
    pub fn budget_used(&self) -> f64 {
        match self {
            Self::BudgetExceeded { budget_used, .. } => *budget_used,
            _ => 0.0,
        }
    }
    pub fn max_budget(&self) -> f64 {
        match self {
            Self::BudgetExceeded { max_budget, .. } => *max_budget,
            _ => 0.0,
        }
    }
    pub fn action(&self) -> &str {
        match self {
            Self::AgentTeam { action, .. } => action,
            _ => "",
        }
    }
    pub fn component(&self) -> &str {
        match self {
            Self::SystemError { component, .. } => component,
            _ => "",
        }
    }
    pub fn severity(&self) -> &str {
        match self {
            Self::SystemError { severity, .. } => severity,
            _ => "",
        }
    }
    pub fn quality(&self) -> f64 {
        match self {
            Self::ConsciousnessCritique { quality, .. } => *quality,
            _ => 0.0,
        }
    }
    pub fn relevance(&self) -> f64 {
        match self {
            Self::ConsciousnessCritique { relevance, .. } => *relevance,
            _ => 0.0,
        }
    }
    pub fn consistency(&self) -> f64 {
        match self {
            Self::ConsciousnessCritique { consistency, .. } => *consistency,
            _ => 0.0,
        }
    }

    /// Domain tag — identifies which NT-* domain originated the event.
    pub fn domain(&self) -> &str {
        match self {
            Self::CoreBootStarted { .. } | Self::ConsciousnessShift { .. } => "nt_core",
            Self::MindSealIteration { .. } | Self::MindDistillation { .. } => "nt_mind",
            Self::MemoryKbWrite { .. } | Self::MemoryKbQuery { .. } => "nt_memory",
            Self::WorldCrawlCompleted { .. } | Self::WorldFetchError { .. } => "nt_world",
            Self::ActToolInvocation { .. } | Self::ActGoalProgress { .. } | Self::DownloadProgress { .. } => "nt_act",
            Self::IoProviderSwitch { .. } | Self::IoRequestError { .. } => "nt_io",
            Self::ShieldIntrusionDetected { .. } | Self::ShieldAuditCompleted { .. } => "nt_shield",
            Self::TaskSubmitted { .. } | Self::AgentFeedback { .. } | Self::AgentTeam { .. } => "nt_act",
            Self::ExternalReward { .. } => "nt_world",
            Self::GoalCompleted { .. } | Self::BudgetExceeded { .. } => "nt_memory",
            Self::SystemError { .. } | Self::GlobalHalt { .. } => "nt_core",
            Self::ConsciousnessCritique { .. } => "nt_core",
            Self::GameSessionCreated { .. }
            | Self::GameEpisodeCompleted { .. }
            | Self::GameTrainingUpdate { .. }
            | Self::GameConsciousnessFeedback { .. } => "nt_mind",
            Self::DomainEvent { domain, .. } => domain,
            Self::WorkspaceSwitched { .. }
            | Self::WorkspaceAgentAdded { .. }
            | Self::WorkspaceAgentRemoved { .. }
            | Self::AgentStatusChanged { .. }
            | Self::AgentTaskDelegated { .. }
            | Self::SkillInstalled { .. }
            | Self::SkillExecuted { .. }
            | Self::SkillMaturityChanged { .. }
            | Self::TaskAssigned { .. }
            | Self::TaskCompletedV2 { .. }
            | Self::McpToolCalled { .. }
            | Self::McpServerConnected { .. }
            | Self::UpgradeAvailable { .. } => "nt_core",
        }
    }

    pub fn domain_field(&self) -> &str {
        match self {
            Self::DomainEvent { domain, .. } => domain,
            _ => "",
        }
    }
    pub fn event_type_field(&self) -> &str {
        match self {
            Self::DomainEvent { event_type, .. } => event_type,
            _ => "",
        }
    }
}
