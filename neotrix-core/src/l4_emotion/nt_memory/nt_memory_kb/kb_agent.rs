//! kb_agent — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。

use std::collections::HashMap;

use super::KnowledgeBase;
use super::nt_memory_agent_driven;
use super::nt_memory_unify;
use super::shared_utils;
use super::nt_memory_agent_session::{AgentSession, AgentSessionEntry, AgentSessionManager};

impl KnowledgeBase {
    pub fn agent_memory_insert(&self, content: &str) -> uuid::Uuid {
        self.agent_memory
            .write()
            .map_err(|e| format!("Lock: {}", e))
            .map(|mut m| m.insert(content))
            .unwrap_or(uuid::Uuid::nil())
    }

    pub fn agent_memory_search(
        &self,
        query: &str,
    ) -> Vec<(nt_memory_agent_driven::AgentMemoryEntry, f64)> {
        self.agent_memory
            .read()
            .map(|m| {
                m.search_all(query)
                    .into_iter()
                    .map(|(e, s)| (e.clone(), s))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn agent_memory_consolidate(&self) -> usize {
        self.agent_memory
            .write()
            .map(|mut m| m.consolidate())
            .unwrap_or(0)
    }

    pub fn agent_memory_self_edit(
        &self,
        entry_id: &uuid::Uuid,
        new_content: &str,
    ) -> Result<uuid::Uuid, String> {
        let mut mem = self
            .agent_memory
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        mem.self_edit(entry_id, new_content)
    }

    pub fn agent_memory_stats(&self) -> nt_memory_agent_driven::MemoryStats {
        self.agent_memory
            .read()
            .map(|m| m.stats())
            .unwrap_or(nt_memory_agent_driven::MemoryStats {
                core_count: 0,
                working_count: 0,
                archival_count: 0,
                total: 0,
            })
    }

    pub fn save_agent_memory(&self) -> Result<(), String> {
        let mem = self
            .agent_memory
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        shared_utils::save_kv_state(self, "agent_memory", "state", &*mem)
    }

    pub fn load_agent_memory(&self) -> Result<(), String> {
        let loaded: Option<nt_memory_agent_driven::AgentMemory> =
            shared_utils::load_kv_state(self, "agent_memory", "state")?;
        if let Some(data) = loaded {
            let mut mem = self
                .agent_memory
                .write()
                .map_err(|e| format!("Lock: {}", e))?;
            *mem = data;
        }
        Ok(())
    }

    // ── RouteLearner 持久化 (P1) ──
    // 派单路由学习者的行为统计存 kv_store `route_learner` namespace, 跨会话存活 —
    // 让"派单从结果里学"不止在单次运行内生效, 重启后继续累积证据。

    pub fn save_route_learner(&self, json: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::kv_set(&conn, "route_learner", "state", json)
    }

    pub fn load_route_learner(&self) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::kv_get(&conn, "route_learner", "state")
    }

    // ── DispatchTopology 持久化 (P3, MANTA 跨轮 playbook) ──
    // 派单拓扑 (域→档案边) 的修复履历存 kv_store `dispatch_topology` namespace,
    // 跨会话存活 — 让"组织自进化"的经验跨运行传输 (MANTA cross-run playbook)。

    pub fn save_dispatch_topology(&self, json: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::kv_set(&conn, "dispatch_topology", "state", json)
    }

    pub fn load_dispatch_topology(&self) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::kv_get(&conn, "dispatch_topology", "state")
    }

    // ── CoEvolutionLoop 持久化 (P4, MAGE 四子图共进化) ──
    // 共进化知识图谱 (capability/task/experience/environment 四子图) 与任务级搜索 bandit
    // 存 kv_store `coevolution` namespace, 跨会话存活 — 让"同一 reward 驱动图+bandit 共进化"
    // 的成果跨运行传输, 重启后继续在既有图谱上累积 (append-only 不重头再来)。

    pub fn save_coevo(&self, json: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::kv_set(&conn, "coevolution", "state", json)
    }

    pub fn load_coevo(&self) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_unify::kv_get(&conn, "coevolution", "state")
    }

    // ── Agent Session Management (SQLite-backed) ──

    pub fn agent_session_begin(&self, agent_id: &str, label: &str) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        AgentSessionManager::begin_session(&conn, agent_id, label)
            .map_err(|e| format!("Session begin: {}", e))
    }

    pub fn agent_session_end(&self, session_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        AgentSessionManager::end_session(&conn, session_id)
            .map_err(|e| format!("Session end: {}", e))
    }

    pub fn agent_memory_store(
        &self,
        agent_id: &str,
        session_id: &str,
        content: &str,
        tier: &str,
        metadata: HashMap<String, String>,
        embedding: Option<&[f32]>,
    ) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        AgentSessionManager::store(
            &conn, agent_id, session_id, content, tier, metadata, embedding,
        )
        .map_err(|e| format!("Memory store: {}", e))
    }

    pub fn agent_memory_recall(
        &self,
        agent_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<AgentSessionEntry>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        AgentSessionManager::recall_by_agent(&conn, agent_id, query, limit)
            .map_err(|e| format!("Memory recall: {}", e))
    }

    pub fn agent_memory_recall_session(
        &self,
        session_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<AgentSessionEntry>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        AgentSessionManager::recall_by_session(&conn, session_id, query, limit)
            .map_err(|e| format!("Session recall: {}", e))
    }

    pub fn agent_memory_recall_similar(
        &self,
        agent_id: &str,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(AgentSessionEntry, f64)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        AgentSessionManager::recall_similar(&conn, agent_id, query_embedding, limit)
            .map_err(|e| format!("Similar recall: {}", e))
    }

    pub fn agent_session_list(&self, agent_id: &str) -> Result<Vec<AgentSession>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        AgentSessionManager::list_sessions(&conn, agent_id)
            .map_err(|e| format!("List sessions: {}", e))
    }

    // ── SVAF Gate ──
}
