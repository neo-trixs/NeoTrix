//! NT-NEXUS — 跨会话记忆枢纽
//!
//! 连接不同会话的经验, 维护知识图谱, 桥接会话间断点

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;

// 从 L1 nt_act_autonomy 迁移过来的模块
pub mod cross_session_memory;

pub use cross_session_memory::CrossSessionMemory;

/// 跨会话记忆核心
pub struct NexusCore {
    /// 会话间连接图
    session_graph: HashMap<String, Vec<String>>,
    /// 经验索引
    experience_index: HashMap<String, Vec<ExperienceRef>>,
    /// KB 持久化句柄 (可选)
    kb: Option<KnowledgeBase>,
}

/// 经验引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceRef {
    pub session_id: String,
    pub cycle: u32,
    pub summary: String,
    pub domain: String,
    pub timestamp: u64,
}

/// 会话图持久化载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionGraphPayload {
    graph: HashMap<String, Vec<String>>,
}

/// 经验索引持久化载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExperienceIndexPayload {
    index: HashMap<String, Vec<ExperienceRef>>,
}

/// KB namespace 常量
const NEXUS_NS: &str = "nexus";
const KEY_SESSION_GRAPH: &str = "session_graph";
const KEY_EXPERIENCE_INDEX: &str = "experience_index";

impl NexusCore {
    /// 创建空实例 (无 KB)
    pub fn new() -> Self {
        Self {
            session_graph: HashMap::new(),
            experience_index: HashMap::new(),
            kb: None,
        }
    }

    /// 绑定 KB 并尝试加载已持久化的数据
    pub fn with_kb(kb: KnowledgeBase) -> Self {
        let mut core = Self {
            session_graph: HashMap::new(),
            experience_index: HashMap::new(),
            kb: Some(kb),
        };
        // 从 KB 加载已有数据, 加载失败不影响构造 (降级为空状态)
        if let Err(e) = core.load_from_kb() {
            log::warn!("[NexusCore] 首次加载 KB 失败, 使用空状态: {}", e);
        }
        core
    }

    /// 运行时绑定 KB (延迟接线场景)
    pub fn set_kb(&mut self, kb: KnowledgeBase) {
        self.kb = Some(kb);
        if let Err(e) = self.load_from_kb() {
            log::warn!("[NexusCore] 绑定 KB 后加载失败: {}", e);
        }
    }

    /// 将当前状态持久化到 KB
    ///
    /// 存储策略: 以 JSON 序列化 session_graph / experience_index,
    /// 分别写入 `kv_store nexus` namespace 的两个 key。
    pub fn save_to_kb(&self) -> Result<(), String> {
        let kb = self.kb.as_ref().ok_or("NexusCore 未绑定 KB")?;

        // 序列化会话连接图
        let graph_payload = SessionGraphPayload {
            graph: self.session_graph.clone(),
        };
        let graph_json =
            serde_json::to_string(&graph_payload).map_err(|e| format!("序列化 session_graph 失败: {}", e))?;
        kb.kv_set(NEXUS_NS, KEY_SESSION_GRAPH, &graph_json)?;

        // 序列化经验索引
        let index_payload = ExperienceIndexPayload {
            index: self.experience_index.clone(),
        };
        let index_json =
            serde_json::to_string(&index_payload).map_err(|e| format!("序列化 experience_index 失败: {}", e))?;
        kb.kv_set(NEXUS_NS, KEY_EXPERIENCE_INDEX, &index_json)?;

        log::debug!(
            "[NexusCore] 已持久化到 KB: {} 个会话节点, {} 个经验域",
            self.session_graph.len(),
            self.experience_index.len(),
        );
        Ok(())
    }

    /// 从 KB 加载数据到内存
    ///
    /// KB 不可用或数据不存在时静默返回 Ok, 不覆盖内存状态。
    pub fn load_from_kb(&mut self) -> Result<(), String> {
        let kb = self.kb.as_ref().ok_or("NexusCore 未绑定 KB")?;

        // 加载会话连接图
        if let Some(graph_json) = kb.kv_get(NEXUS_NS, KEY_SESSION_GRAPH)? {
            let loaded: SessionGraphPayload =
                serde_json::from_str(&graph_json).map_err(|e| format!("反序列化 session_graph 失败: {}", e))?;
            self.session_graph = loaded.graph;
        }

        // 加载经验索引
        if let Some(index_json) = kb.kv_get(NEXUS_NS, KEY_EXPERIENCE_INDEX)? {
            let loaded: ExperienceIndexPayload =
                serde_json::from_str(&index_json).map_err(|e| format!("反序列化 experience_index 失败: {}", e))?;
            self.experience_index = loaded.index;
        }

        log::debug!(
            "[NexusCore] 已从 KB 加载: {} 个会话节点, {} 个经验域",
            self.session_graph.len(),
            self.experience_index.len(),
        );
        Ok(())
    }

    /// 记录会话间连接 (写后自动持久化)
    pub fn link_sessions(&mut self, from: &str, to: &str) {
        self.session_graph
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
        // 写后自动同步 KB (best-effort)
        if let Err(e) = self.save_to_kb() {
            log::debug!("[NexusCore] link_sessions 持久化跳过: {}", e);
        }
    }

    /// 索引经验 (写后自动持久化)
    pub fn index_experience(&mut self, exp: ExperienceRef) {
        self.experience_index
            .entry(exp.domain.clone())
            .or_default()
            .push(exp);
        // 写后自动同步 KB (best-effort)
        if let Err(e) = self.save_to_kb() {
            log::debug!("[NexusCore] index_experience 持久化跳过: {}", e);
        }
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

    /// 当前是否绑定了 KB
    pub fn has_kb(&self) -> bool {
        self.kb.is_some()
    }

    /// 获取内存中的会话连接数
    pub fn session_count(&self) -> usize {
        self.session_graph.len()
    }

    /// 获取内存中的经验索引条目总数
    pub fn experience_count(&self) -> usize {
        self.experience_index.values().map(|v| v.len()).sum()
    }
}

impl Default for NexusCore {
    fn default() -> Self {
        Self::new()
    }
}
