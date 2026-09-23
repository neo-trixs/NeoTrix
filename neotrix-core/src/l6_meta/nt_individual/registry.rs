//! Individual registry — 注册个体（分身挂靠的锚点）.
//!
//! 设计立场：个体可移植运行（身体独立），但身份/记忆/治理向
//! NeoTrix 注册（灵魂挂靠）。本注册表是挂靠点：
//! - 注册/查询/按专长选择个体
//! - 会话结算（session 计数 + cost + 记忆指针接入）
//! - KB 持久化钩子（serde 进出，KV 写入由调用方持 KB 句柄完成）
//!
//! 公开 API 统一用 [`RegistryError`]（禁 Result<T, String>）。

#![forbid(unsafe_code)]

use std::collections::HashMap;
use thiserror::Error;

use super::individual::NtIndividual;

/// 注册表错误
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("individual '{0}' already registered")]
    Duplicate(String),
    #[error("individual '{0}' not found")]
    NotFound(String),
    #[error("serialization failed: {0}")]
    Serde(String),
}

/// 个体注册表（内存 + KB 钩子）
#[derive(Debug, Default)]
pub struct IndividualRegistry {
    individuals: HashMap<String, NtIndividual>,
}

impl IndividualRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册个体（重复 id 拒绝）
    pub fn register(&mut self, ind: NtIndividual) -> Result<(), RegistryError> {
        let id = ind.persona.id.clone();
        if self.individuals.contains_key(&id) {
            return Err(RegistryError::Duplicate(id));
        }
        self.individuals.insert(id, ind);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<&NtIndividual, RegistryError> {
        self.individuals
            .get(id)
            .ok_or_else(|| RegistryError::NotFound(id.to_string()))
    }

    pub fn get_mut(&mut self, id: &str) -> Result<&mut NtIndividual, RegistryError> {
        self.individuals
            .get_mut(id)
            .ok_or_else(|| RegistryError::NotFound(id.to_string()))
    }

    /// 按专长选可用个体（挂靠调度的最小实现）
    pub fn select_best(&self, specialty: &str) -> Option<&NtIndividual> {
        use super::super::nt_agent_identity::AgentStatus;
        self.individuals
            .values()
            .filter(|i| {
                i.persona.status == AgentStatus::Available
                    && i.persona.autonomy.allows_tools()
                    && (i.persona.specialty == specialty
                        || i.persona.tags.iter().any(|t| t == specialty))
            })
            .next()
    }

    /// 会话结算：计数 + cost + 记忆指针接入（进化入口）
    pub fn record_session(
        &mut self,
        id: &str,
        session_id: &str,
        branches: u64,
        cost: f64,
    ) -> Result<(), RegistryError> {
        let ind = self.get_mut(id)?;
        ind.evolve(session_id, branches);
        ind.persona.total_cost += cost;
        Ok(())
    }

    /// KB 持久化钩子：个体 → JSON（调用方写入 `agent_individual` namespace）
    pub fn to_kb_value(&self, id: &str) -> Result<String, RegistryError> {
        let ind = self.get(id)?;
        serde_json::to_string(ind).map_err(|e| RegistryError::Serde(e.to_string()))
    }

    /// KB 恢复钩子：JSON → 个体并注册
    pub fn from_kb_value(&mut self, value: &str) -> Result<String, RegistryError> {
        let ind: NtIndividual =
            serde_json::from_str(value).map_err(|e| RegistryError::Serde(e.to_string()))?;
        let id = ind.persona.id.clone();
        self.register(ind)?;
        Ok(id)
    }

    pub fn len(&self) -> usize {
        self.individuals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l6_meta::nt_individual::presets::wsd_trade_individual;

    #[test]
    fn register_rejects_duplicates() {
        let mut reg = IndividualRegistry::new();
        reg.register(wsd_trade_individual()).expect("first");
        let err = reg.register(wsd_trade_individual()).expect_err("dup");
        assert!(matches!(err, RegistryError::Duplicate(_)));
    }

    #[test]
    fn select_best_matches_specialty() {
        let mut reg = IndividualRegistry::new();
        reg.register(wsd_trade_individual()).expect("reg");
        let hit = reg.select_best("trade_inquiry");
        assert!(hit.is_some());
        assert!(reg.select_best("nope").is_none());
    }

    #[test]
    fn record_session_evolves_memory() {
        let mut reg = IndividualRegistry::new();
        reg.register(wsd_trade_individual()).expect("reg");
        reg.record_session("ind-wsd-trade-001", "sess_x", 5, 0.2)
            .expect("record");
        let ind = reg.get("ind-wsd-trade-001").expect("get");
        assert_eq!(ind.memory.branches_absorbed, 46 + 5);
        assert!((ind.persona.total_cost - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn kb_roundtrip() {
        let mut reg = IndividualRegistry::new();
        reg.register(wsd_trade_individual()).expect("reg");
        let v = reg.to_kb_value("ind-wsd-trade-001").expect("ser");
        let mut reg2 = IndividualRegistry::new();
        let id = reg2.from_kb_value(&v).expect("de");
        assert_eq!(id, "ind-wsd-trade-001");
        assert_eq!(reg2.len(), 1);
    }
}
