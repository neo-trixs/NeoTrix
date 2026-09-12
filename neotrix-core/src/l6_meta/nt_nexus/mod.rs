//! NT-NEXUS — 跨会话记忆枢纽
//!
//! 连接不同会话的经验, 维护知识图谱, 桥接会话间断点

use std::collections::HashMap;

// 从 L1 nt_act_autonomy 迁移过来的模块
pub mod cross_session_memory;

pub use cross_session_memory::CrossSessionMemory;

/// 跨会话记忆核心
pub struct NexusCore {
    /// 会话间连接图
    session_graph: HashMap<String, Vec<String>>,
    /// 经验索引
    experience_index: HashMap<String, Vec<ExperienceRef>>,
}

/// 经验引用
#[derive(Debug, Clone)]
pub struct ExperienceRef {
    pub session_id: String,
    pub cycle: u32,
    pub summary: String,
    pub domain: String,
    pub timestamp: u64,
}

impl NexusCore {
    pub fn new() -> Self {
        Self {
            session_graph: HashMap::new(),
            experience_index: HashMap::new(),
        }
    }

    /// 记录会话间连接
    pub fn link_sessions(&mut self, from: &str, to: &str) {
        self.session_graph
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
    }

    /// 索引经验
    pub fn index_experience(&mut self, exp: ExperienceRef) {
        self.experience_index
            .entry(exp.domain.clone())
            .or_default()
            .push(exp);
    }

    /// 查询相关经验
    pub fn query(&self, domain: &str, keyword: &str) -> Vec<&ExperienceRef> {
        self.experience_index
            .get(domain)
            .map(|exps| exps.iter().filter(|e| e.summary.contains(keyword)).collect())
            .unwrap_or_default()
    }

    /// 获取会话链
    pub fn session_chain(&self, session_id: &str) -> Vec<&str> {
        self.session_graph
            .get(session_id)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
}

impl Default for NexusCore {
    fn default() -> Self {
        Self::new()
    }
}
