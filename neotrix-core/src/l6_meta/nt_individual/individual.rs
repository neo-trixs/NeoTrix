//! NtIndividual — 个体（一等公民）.
//!
//! 组装公式：个体 = 基础能力（AgentCard id 引用）
//!          × 业务场景（persona specialty/system_prompt）
//!          × 专业性（constitution + memory 指针）
//!
//! 进化 = absorb_session 持续接入经验分支；宪法变更须人审
//! （Supervised 及以下自治级别）。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::super::nt_agent_identity::{AgentPersona, AutonomyLevel};
use super::constitution::Constitution;
use super::judgment::{judge, ActionKind, Judgment, JudgmentCtx};
use super::memory::MemoryLink;

/// 个体级错误（公开 API 禁止 Result<T, String>）
#[derive(Debug, Error)]
pub enum IndividualError {
    #[error("judgment denied by {rule}: {reason}")]
    Denied { rule: String, reason: String },
    #[error("needs human: {rule}: {reason}")]
    NeedHuman { rule: String, reason: String },
    #[error("unknown action: {0}")]
    UnknownAction(String),
}

/// NeoTrix 个体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtIndividual {
    /// 身份（复用 l6 nt_agent_identity）
    pub persona: AgentPersona,
    /// 引用的 AgentCard id（如 "preset-trade-agent"）
    pub card_id: String,
    /// 行为宪法
    pub constitution: Constitution,
    /// 记忆链接（指针）
    pub memory: MemoryLink,
}

impl NtIndividual {
    pub fn new(persona: AgentPersona, card_id: &str, constitution: Constitution) -> Self {
        Self {
            persona,
            card_id: card_id.to_string(),
            constitution,
            memory: MemoryLink::new(),
        }
    }

    /// 行动前裁决（个体判断力的唯一入口）
    pub fn decide(&self, action: &ActionKind, ctx: &JudgmentCtx) -> Result<(), IndividualError> {
        match judge(&self.constitution, action, ctx) {
            Judgment::Allow => Ok(()),
            Judgment::Deny { rule, reason } => Err(IndividualError::Denied { rule, reason }),
            Judgment::NeedHuman { rule, reason } => Err(IndividualError::NeedHuman { rule, reason }),
        }
    }

    /// 进化：接入一次会话的经验指针（只增记忆，不改宪法）
    pub fn evolve(&mut self, session_id: &str, branches: u64) {
        self.memory.absorb_session(session_id, branches);
        self.persona.session_count += 1;
    }

    /// 宪法修订：Supervised 及以下必须人审（调用方负责弹确认，这里只判）
    pub fn may_revise_constitution(&self) -> bool {
        matches!(self.persona.autonomy, AutonomyLevel::Autonomous)
            && self.persona.autonomy.allows_tools()
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::nt_agent_identity::AgentStatus;
    use super::*;
    use std::collections::HashMap;

    fn persona() -> AgentPersona {
        AgentPersona {
            id: "ind-test".to_string(),
            name: "测试个体".to_string(),
            avatar: "🤖".to_string(),
            specialty: "trade_inquiry".to_string(),
            personality: "严谨".to_string(),
            autonomy: AutonomyLevel::Supervised,
            cost_budget: 10.0,
            status: AgentStatus::Available,
            provider: None,
            model: None,
            system_prompt: None,
            tags: vec![],
            created_at: 0,
            updated_at: 0,
            session_count: 0,
            total_cost: 0.0,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn individual_denies_unverified_fetch() {
        let ind = NtIndividual::new(persona(), "preset-trade-agent", Constitution::wsd_default());
        let r = ind.decide(&ActionKind::FetchUnverifiedEndpoint, &JudgmentCtx::default());
        assert!(matches!(r, Err(IndividualError::Denied { .. })));
    }

    #[test]
    fn evolve_increments_memory_and_sessions() {
        let mut ind =
            NtIndividual::new(persona(), "preset-trade-agent", Constitution::wsd_default());
        ind.evolve("sess_1", 13);
        ind.evolve("sess_1", 13);
        assert_eq!(ind.memory.session_count(), 1);
        assert_eq!(ind.persona.session_count, 2);
    }

    #[test]
    fn supervised_cannot_self_revise_constitution() {
        let ind = NtIndividual::new(persona(), "preset-trade-agent", Constitution::wsd_default());
        assert!(!ind.may_revise_constitution());
    }
}
