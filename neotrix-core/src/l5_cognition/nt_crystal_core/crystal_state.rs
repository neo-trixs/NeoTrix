//! CrystalState — 统一状态空间
//!
//! 所有子系统的单一事实源。晶体不是连接模块的中心，晶体是系统本身。
//! 能力/Agent/子系统是晶体的内在属性，不是外部模块。
//!
//! 设计文档: docs/1-DESIGN/unified-crystal-architecture.md
//! KB 命名空间五实体键约定: workspaces/agent_cards/skill_candidates/scheduled_tasks/mcp_servers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::consciousness::CrystalConsciousness;
use super::evolution::GrowthPhase;
use super::experience::CrystalExperience;
use super::identity::CrystalIdentity;
use super::knowledge::CrystalKnowledge;
use super::evolution::CrystalEvolution;

/// save() 并发 tmp 序列号 (仿 `super::CrystalCore::save` 的 pid+seq 隔离)
static STATE_SAVE_TMP_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════
// Core Types — 晶体统一状态空间
// ═══════════════════════════════════════════════════════════════

/// 晶体的统一状态空间 — 所有子系统的单一事实源
///
/// CrystalState 是 Unified Crystal Architecture 的核心。
/// 它不是"连接模块的中心"，而是"系统本身"。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalState {
    // === 核心身份 ===
    /// 晶体身份 (不可变)
    pub identity: CrystalIdentity,
    /// 内部时钟
    pub tick: u64,

    // === 四层核心 ===
    /// 知识层 — 缓慢进化 (理论/因果/矛盾)
    pub knowledge: CrystalKnowledge,
    /// 经验层 — 快速积累 (情境/教训/方案)
    pub experience: CrystalExperience,
    /// 进化层 — 实时变化 (生长周期/能力评分)
    pub evolution: CrystalEvolution,

    // === 感知与注意力 ===
    /// 注意力焦点
    pub attention_focus: Option<String>,
    /// 当前目标
    pub current_goal: Option<String>,

    // === 能力坐标系 ===
    /// 能力评分 (名称 → 0.0-1.0)
    pub capabilities: HashMap<String, f64>,
    /// 活跃能力投影 (Agent 在任务空间的临时实例)
    pub projections: Vec<AgentProjection>,

    // === 执行追踪 ===
    /// 执行历史
    pub execution_trace: Vec<ExecutionRecord>,

    // === 五实体投影 (E2: CrystalState 为单一事实源) ===
    /// Workspace 投影 (Option: 单 workspace 阶段)
    #[serde(default)]
    pub workspace: Option<WorkspaceProjection>,
    /// Agent 投影 (常驻目录, 与临时 projections 并存)
    #[serde(default)]
    pub agents: Vec<AgentProjection>,
    /// Skill 投影
    #[serde(default)]
    pub skills: Vec<SkillProjection>,
    /// Task 投影
    #[serde(default)]
    pub tasks: Vec<TaskProjection>,
    /// Tool 投影
    #[serde(default)]
    pub tools: Vec<ToolProjection>,
}

/// Agent 在任务空间的临时投影
///
/// Agent 不是独立引擎，是晶体意识在任务空间的临时投影。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProjection {
    /// 投影 ID
    #[serde(default)]
    pub id: String,
    /// 任务描述
    #[serde(default)]
    pub task: String,
    /// 投影的能力子集
    #[serde(default)]
    pub capabilities_used: Vec<String>,
    /// 置信度
    #[serde(default)]
    pub confidence: f64,
    /// 创建时间
    #[serde(default)]
    pub created_at: u64,
    /// 所属 workspace (E2: String 保持零上层依赖)
    #[serde(default)]
    pub workspace_id: Option<String>,
    /// 运行状态 (E2: String 而非跨层枚举)
    #[serde(default)]
    pub status: String,
    /// 已安装技能
    #[serde(default)]
    pub installed_skills: Vec<String>,
    /// MCP 权限
    #[serde(default)]
    pub mcp_permissions: Vec<String>,
}

/// 执行记录 — 晶体的神经系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// 时间戳
    pub tick: u64,
    /// 操作类型
    pub action: String,
    /// 输入摘要
    pub input: String,
    /// 输出摘要
    pub output: String,
    /// 成功与否
    pub success: bool,
    /// 耗时 (ms)
    pub duration_ms: u64,
}

// ═══════════════════════════════════════════════════════════════
// E2 Projections — 五实体投影 (CrystalState 为单一事实源)
// ═══════════════════════════════════════════════════════════════

/// Workspace 投影
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceProjection {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub root: String,
    #[serde(default)]
    pub agent_ids: Vec<String>,
    #[serde(default)]
    pub skill_ids: Vec<String>,
    #[serde(default)]
    pub tool_names: Vec<String>,
    #[serde(default)]
    pub shared_memory_keys: Vec<String>,
}

/// Skill 投影
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProjection {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub source_knowledge_ids: Vec<String>,
    #[serde(default)]
    pub source_experience_ids: Vec<String>,
    #[serde(default)]
    pub effectiveness: f64,
    #[serde(default)]
    pub use_count: usize,
    #[serde(default)]
    pub maturity: String,
}

/// Task 投影 (复用本文件 ExecutionRecord 作为 history 项)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProjection {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub instruction: String,
    #[serde(default)]
    pub executor_agent_id: Option<String>,
    #[serde(default)]
    pub skill_ids: Vec<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub history: Vec<ExecutionRecord>,
}

/// Tool 投影
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProjection {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub server_name: String,
    #[serde(default)]
    pub capability_tag: String,
    #[serde(default)]
    pub risk_level: String,
    #[serde(default)]
    pub usage_count: u64,
    #[serde(default)]
    pub avg_latency_ms: f64,
}

// ═══════════════════════════════════════════════════════════════
// AgentDirectory — 只读聚合外观 (V3.1·S1 设计修正版)
// 只读投影 (单一事实源), 不直接引用四注册表类型 (避免反向耦合)。
// 注册表→投影的 feeding 是 E2 后续接线事项。
// ═══════════════════════════════════════════════════════════════

/// Agent 目录条目 (只读投影项, source 标数据来源)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDirEntry {
    #[serde(default)]
    pub agent_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub source: String,
}

/// Agent 目录 — 只读聚合外观
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentDirectory {
    #[serde(default)]
    pub entries: Vec<AgentDirEntry>,
}

impl AgentDirectory {
    /// 从 Agent 投影切片构建只读目录 (返回 Self 以支持链式只读查询;
    /// 如需裸 Vec 可用 `.entries` / `.into_entries()`).
    pub fn from_projections(projections: &[AgentProjection]) -> Self {
        let entries = projections
            .iter()
            .map(|p| AgentDirEntry {
                agent_id: p.id.clone(),
                // AgentProjection 无独立 name, 以 id 为名保持可追溯
                name: p.id.clone(),
                capabilities: p.capabilities_used.clone(),
                status: p.status.clone(),
                source: "projection".to_string(),
            })
            .collect();
        Self { entries }
    }

    /// 消费为裸条目 Vec (对应蓝图 `-> Vec<AgentDirEntry>` 的取数形态)
    pub fn into_entries(self) -> Vec<AgentDirEntry> {
        self.entries
    }

    /// 只读访问条目切片
    pub fn entries(&self) -> &[AgentDirEntry] {
        &self.entries
    }

    /// 目录大小
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 按能力子串过滤 (大小写敏感, `contains` 语义)
    pub fn find_by_capability_substr(&self, kw: &str) -> Vec<&AgentDirEntry> {
        self.entries
            .iter()
            .filter(|e| e.capabilities.iter().any(|c| c.contains(kw)))
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════
// Implementation — 统一状态操作
// ═══════════════════════════════════════════════════════════════

impl CrystalState {
    /// 创建新的晶体状态
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut capabilities = HashMap::new();
        capabilities.insert("memory_capacity".into(), 0.0);
        capabilities.insert("reasoning_depth".into(), 0.0);
        capabilities.insert("pattern_recognition".into(), 0.0);
        capabilities.insert("self_awareness".into(), 0.0);
        capabilities.insert("creativity".into(), 0.0);

        Self {
            identity: CrystalIdentity::new(name),
            tick: 0,
            knowledge: CrystalKnowledge::new(),
            experience: CrystalExperience::new(),
            evolution: CrystalEvolution::new(),
            attention_focus: None,
            current_goal: None,
            capabilities,
            projections: Vec::new(),
            execution_trace: Vec::new(),
            workspace: None,
            agents: Vec::new(),
            skills: Vec::new(),
            tasks: Vec::new(),
            tools: Vec::new(),
        }
    }

    /// 获取当前 tick
    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    /// 推进 tick
    pub fn advance_tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    /// 记忆 — 统一入口
    pub fn remember(&mut self, content: impl Into<String>, domain: impl Into<String>) {
        self.experience.record_episode(
            content,
            "remember",      // action
            "recorded",      // result
            "self-observe",  // reflection
            domain,
            1.0,             // quality
        );
    }

    /// 记录执行
    pub fn record_execution(&mut self, action: String, input: String, output: String, success: bool, duration_ms: u64) {
        self.tick += 1;
        self.execution_trace.push(ExecutionRecord {
            tick: self.tick,
            action,
            input,
            output,
            success,
            duration_ms,
        });
    }

    /// 创建 Agent 投影
    pub fn project_agent(&mut self, task: String, capabilities: Vec<String>) -> AgentProjection {
        let id = format!("agent-{}", self.tick);
        let projection = AgentProjection {
            id: id.clone(),
            task,
            capabilities_used: capabilities,
            confidence: 0.5,
            created_at: self.tick,
            workspace_id: None,
            status: "active".to_string(),
            installed_skills: Vec::new(),
            mcp_permissions: Vec::new(),
        };
        self.projections.push(projection.clone());
        projection
    }

    /// 获取整体能力评分
    pub fn overall_capability(&self) -> f64 {
        if self.capabilities.is_empty() {
            return 0.0;
        }
        self.capabilities.values().sum::<f64>() / self.capabilities.len() as f64
    }

    /// 获取状态摘要
    pub fn summary(&self) -> CrystalStateSummary {
        CrystalStateSummary {
            name: self.identity.name.clone(),
            tick: self.tick,
            knowledge_count: self.knowledge.theories.len() + self.knowledge.patterns.len(),
            experience_count: self.experience.episodes.len(),
            evolution_phase: self.evolution.current_phase.clone(),
            capability_score: self.overall_capability(),
            active_projections: self.projections.len(),
            execution_count: self.execution_trace.len(),
            active_tasks: self
                .tasks
                .iter()
                .filter(|t| {
                    !matches!(
                        t.status.as_str(),
                        "completed" | "done" | "cancelled" | "failed" | "closed"
                    )
                })
                .count(),
        }
    }

    /// 从 CrystalCore 构建
    pub fn from_core(core: super::CrystalCore) -> Self {
        let mut state = Self::new(&core.identity.name);
        state.knowledge = core.knowledge;
        state.experience = core.experience;
        state.evolution = core.evolution;
        state
    }

    /// 从 CrystalConsciousness 构建
    pub fn from_consciousness(cc: CrystalConsciousness) -> Self {
        let mut state = Self::new(&cc.identity.name);
        state.capabilities = cc.capabilities;
        state.attention_focus = cc.state.attention_focus;
        state.current_goal = cc.state.current_goal;
        // 迁移记忆
        for (_, memory) in cc.memories {
            state.remember(&memory.content, &memory.domain);
        }
        state
    }

    /// 保存到磁盘 (R-P0-2: tmp+rename 原子写，旧态轮转为 `.bak` 快照)
    ///
    /// 路径与 `crystal.json` 同目录 (`crystal_root()`)，文件名为 `crystal_state.json`。
    /// 并发安全：tmp 文件名带 pid+自增序列，多线程同时 save 不抢同一个 tmp；
    /// `rename` 本身原子，最后落盘者胜，绝不出现半截文件。
    pub fn save(&self) -> Result<(), String> {
        let root = super::crystal_root();
        std::fs::create_dir_all(&root).map_err(|e| format!("Failed to create crystal dir: {e}"))?;
        let path = root.join("crystal_state.json");
        let tmp = root.join(format!(
            "crystal_state.json.tmp.{}-{}",
            std::process::id(),
            STATE_SAVE_TMP_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize crystal state: {e}"))?;
        std::fs::write(&tmp, data)
            .map_err(|e| format!("Failed to write crystal state tmp: {e}"))?;
        if path.exists() {
            // 五代轮转（T25b）：先平移旧备份，再把当前态转正为 .bak
            Self::rotate_backups(&root, "crystal_state.json", 5);
            match std::fs::rename(&path, root.join("crystal_state.json.bak")) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("Failed to rotate crystal state backup: {e}")),
            }
        }
        std::fs::rename(&tmp, &path).map_err(|e| format!("Failed to commit crystal state: {e}"))
    }

    /// 备份轮转（T25b：与 `crystal.json` 五代轮转对等；`mod.rs` 后续可复用本函数）。
    ///
    /// `.bak` 为第一代（`load` 兼容链不变），另存 `.bak.1`…`.bak.{generations-2}`；
    /// 超限最旧代删除。调用方在轮转后自行把当前文件 rename 为 `.bak`.
    fn rotate_backups(root: &std::path::Path, stem: &str, generations: usize) {
        if generations < 2 {
            return;
        }
        let mut chain: Vec<std::path::PathBuf> = vec![root.join(format!("{stem}.bak"))];
        for i in 1..generations - 1 {
            chain.push(root.join(format!("{stem}.bak.{i}")));
        }
        // 先删超限最旧代（chain 之外一代）
        let _ = std::fs::remove_file(root.join(format!("{stem}.bak.{}", generations - 1)));
        // 自旧向新平移：chain[k] ← chain[k-1]
        // 2026-09-30: rename 失败原先被吞。平移落盘一半会让代际链不一致
        // （新代缺失、旧代残留），而加载侧只认主文件+legacy .bak ——
        // 平移失败 ⇒ 崩溃恢复可能读到过期代。只补可观测性，不改平移逻辑。
        for k in (1..chain.len()).rev() {
            if chain[k - 1].exists() {
                if let Err(e) = std::fs::rename(&chain[k - 1], &chain[k]) {
                    log::error!(
                        "[crystal-state] 代际平移失败 {} -> {}: {} —— 代际链可能不一致",
                        chain[k - 1].display(),
                        chain[k].display(),
                        e
                    );
                }
            }
        }
    }
    /// 从磁盘加载；缺省/损坏一律返回 `Self::new("restored")`，不抛错
    ///
    /// 候选链：`crystal_state.json` → `crystal_state.json.bak` (legacy 兼容)；
    /// 全失败即视为首跑/损坏，返回默认恢复态。
    pub fn load() -> Self {
        let root = super::crystal_root();
        let candidates = [
            root.join("crystal_state.json"),
            root.join("crystal_state.json.bak"),
        ];
        for cand in &candidates {
            if let Ok(data) = std::fs::read_to_string(cand) {
                if let Ok(state) = serde_json::from_str::<Self>(&data) {
                    return state;
                }
            }
        }
        Self::new("restored")
    }
}

/// 状态摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalStateSummary {
    pub name: String,
    pub tick: u64,
    pub knowledge_count: usize,
    pub experience_count: usize,
    pub evolution_phase: GrowthPhase,
    pub capability_score: f64,
    pub active_projections: usize,
    pub execution_count: usize,
    #[serde(default)]
    pub active_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t25b_backup_rotation_five_generations() {
        // T25b：隔离目录演练轮转（不碰真实 crystal_root）
        let root =
            std::env::temp_dir().join(format!("nt_crystal_state_t25b_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("test temp dir");
        let stem = "crystal_state.json";
        // 预置：当前＋.bak（旧内容），跑两轮 save 语义（轮转＋转正）
        std::fs::write(root.join(stem), "v0").expect("seed current");
        std::fs::write(root.join(format!("{stem}.bak")), "v-1").expect("seed bak");
        let promote = |ver: &str| {
            CrystalState::rotate_backups(&root, stem, 5);
            std::fs::rename(root.join(stem), root.join(format!("{stem}.bak"))).expect("promote");
            std::fs::write(root.join(stem), ver).expect("new current");
        };
        promote("v1");
        promote("v2");
        let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap_or_default();
        assert_eq!(read(stem), "v2");
        assert_eq!(read(&format!("{stem}.bak")), "v1");
        assert_eq!(read(&format!("{stem}.bak.1")), "v0");
        assert_eq!(read(&format!("{stem}.bak.2")), "v-1");
        assert!(!root.join(format!("{stem}.bak.3")).exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_crystal_state_new() {
        let state = CrystalState::new("TestCrystal");
        assert_eq!(state.identity.name, "TestCrystal");
        assert_eq!(state.current_tick(), 0);
        assert!(state.overall_capability() >= 0.0);
    }

    #[test]
    fn test_crystal_state_remember() {
        let mut state = CrystalState::new("TestCrystal");
        state.remember("Test fact", "engineering");
        assert_eq!(state.experience.episodes.len(), 1);
    }

    #[test]
    fn test_crystal_state_project_agent() {
        let mut state = CrystalState::new("TestCrystal");
        let agent = state.project_agent("test task".into(), vec!["reasoning".into()]);
        assert!(!agent.id.is_empty());
        assert_eq!(state.projections.len(), 1);
    }

    #[test]
    fn test_crystal_state_summary() {
        let state = CrystalState::new("TestCrystal");
        let summary = state.summary();
        assert_eq!(summary.name, "TestCrystal");
        assert_eq!(summary.tick, 0);
    }

    #[test]
    fn test_t22_projections_serde_roundtrip_and_legacy_snapshot() {
        let mut state = CrystalState::new("T22Crystal");
        // 新字段默认为空
        assert!(state.workspace.is_none());
        assert!(state.agents.is_empty());
        assert!(state.skills.is_empty());
        assert!(state.tasks.is_empty());
        assert!(state.tools.is_empty());
        assert_eq!(state.summary().active_tasks, 0);

        // 构造投影数据
        let agent = state.project_agent("t22 task".into(), vec!["reasoning".into()]);
        assert_eq!(agent.status, "active");
        state.agents.push(agent);
        state.workspace = Some(WorkspaceProjection {
            id: "ws-1".to_string(),
            name: "ws".to_string(),
            root: "/tmp/ws".to_string(),
            agent_ids: vec!["agent-0".to_string()],
            skill_ids: Vec::new(),
            tool_names: Vec::new(),
            shared_memory_keys: Vec::new(),
        });
        state.skills.push(SkillProjection {
            id: "sk-1".to_string(),
            name: "rust-review".to_string(),
            triggers: vec!["review".to_string()],
            source_knowledge_ids: Vec::new(),
            source_experience_ids: Vec::new(),
            effectiveness: 0.8,
            use_count: 2,
            maturity: "pilot".to_string(),
        });
        state.tasks.push(TaskProjection {
            id: "task-1".to_string(),
            name: "t1".to_string(),
            instruction: "do it".to_string(),
            executor_agent_id: Some("agent-0".to_string()),
            skill_ids: vec!["sk-1".to_string()],
            status: "active".to_string(),
            history: Vec::new(),
        });
        state.tools.push(ToolProjection {
            name: "read".to_string(),
            server_name: "fs".to_string(),
            capability_tag: "io".to_string(),
            risk_level: "low".to_string(),
            usage_count: 1,
            avg_latency_ms: 2.5,
        });
        assert_eq!(state.summary().active_tasks, 1);

        // serde 往返
        let value = serde_json::to_value(&state);
        assert!(value.is_ok());
        if let Ok(v) = value {
            let back = serde_json::from_value::<CrystalState>(v);
            assert!(back.is_ok());
            if let Ok(s2) = back {
                assert_eq!(s2.agents.len(), 1);
                assert_eq!(s2.skills.len(), 1);
                assert_eq!(s2.tasks.len(), 1);
                assert_eq!(s2.tools.len(), 1);
                assert!(s2.workspace.is_some());
                assert_eq!(s2.summary().active_tasks, 1);
            }
        }

        // 旧快照 (无新键) 仍可反序列化, 新字段取默认空值
        let legacy = serde_json::to_value(&state);
        assert!(legacy.is_ok());
        if let Ok(mut v) = legacy {
            if let Some(obj) = v.as_object_mut() {
                obj.remove("workspace");
                obj.remove("agents");
                obj.remove("skills");
                obj.remove("tasks");
                obj.remove("tools");
                // 旧 projections 条目同样无 AgentProjection 新键
                if let Some(projs) = obj.get_mut("projections").and_then(|p| p.as_array_mut()) {
                    for p in projs.iter_mut() {
                        if let Some(po) = p.as_object_mut() {
                            po.remove("workspace_id");
                            po.remove("status");
                            po.remove("installed_skills");
                            po.remove("mcp_permissions");
                        }
                    }
                }
            }
            let back = serde_json::from_value::<CrystalState>(v);
            assert!(back.is_ok());
            if let Ok(s3) = back {
                assert!(s3.workspace.is_none());
                assert!(s3.agents.is_empty());
                assert!(s3.skills.is_empty());
                assert!(s3.tasks.is_empty());
                assert!(s3.tools.is_empty());
                assert_eq!(s3.summary().active_tasks, 0);
            }
        }
    }

    #[test]
    fn test_t22_agent_directory_filter() {
        let mut state = CrystalState::new("T22Dir");
        let mut a1 = state.project_agent("t1".into(), vec!["rust-review".into()]);
        a1.id = "a1".to_string();
        let mut a2 = state.project_agent("t2".into(), vec!["python-lint".into()]);
        a2.id = "a2".to_string();
        let projs = vec![a1, a2];
        let dir = AgentDirectory::from_projections(&projs);
        assert_eq!(dir.len(), 2);
        assert!(!dir.is_empty());
        let hits = dir.find_by_capability_substr("rust");
        assert_eq!(hits.len(), 1);
        if let Some(first) = hits.first() {
            assert_eq!(first.agent_id, "a1");
            assert_eq!(first.source, "projection");
        }
        let miss = dir.find_by_capability_substr("go-compiler-xyz");
        assert!(miss.is_empty());
    }

    #[test]
    fn test_t25_state_save_load_roundtrip_isolated() {
        // T25 注：save/load 硬编码 crystal_root() (~/.neotrix/crystal_core)，
        // 为不写真实家目录，本单测做“序列化往返＋路径构造断言”，不触碰 live root；
        // 覆盖的正是 save/load 共用的 serde 路径与投影字段。
        let mut state = CrystalState::new("T25Crystal");
        let agent = state.project_agent("t25 task".into(), vec!["reasoning".into()]);
        state.agents.push(agent);
        state.workspace = Some(WorkspaceProjection {
            id: "ws-t25".to_string(),
            name: "ws".to_string(),
            root: "/tmp/ws-t25".to_string(),
            agent_ids: vec!["agent-0".to_string()],
            skill_ids: vec!["sk-t25".to_string()],
            tool_names: vec!["read".to_string()],
            shared_memory_keys: Vec::new(),
        });
        state.skills.push(SkillProjection {
            id: "sk-t25".to_string(),
            name: "rust-review".to_string(),
            triggers: vec!["review".to_string()],
            source_knowledge_ids: Vec::new(),
            source_experience_ids: Vec::new(),
            effectiveness: 0.9,
            use_count: 3,
            maturity: "pilot".to_string(),
        });
        state.tasks.push(TaskProjection {
            id: "task-t25".to_string(),
            name: "t25".to_string(),
            instruction: "persist me".to_string(),
            executor_agent_id: Some("agent-0".to_string()),
            skill_ids: vec!["sk-t25".to_string()],
            status: "active".to_string(),
            history: Vec::new(),
        });
        state.tools.push(ToolProjection {
            name: "read".to_string(),
            server_name: "fs".to_string(),
            capability_tag: "io".to_string(),
            risk_level: "low".to_string(),
            usage_count: 7,
            avg_latency_ms: 1.5,
        });

        // 路径构造断言：与 crystal.json 同目录，文件名为 crystal_state.json
        let live = super::super::crystal_root().join("crystal_state.json");
        assert!(live.ends_with("crystal_state.json"));
        assert_eq!(
            live.parent().map(|p| p.to_path_buf()),
            Some(super::super::crystal_root())
        );

        // temp_dir 隔离的序列化往返 (save 的 serialize / load 的 deserialize 同路径)
        let dir = std::env::temp_dir().join(format!("nt_crystal_state_t25_{}", std::process::id()));
        assert!(std::fs::create_dir_all(&dir).is_ok());
        let tmp_path = dir.join("crystal_state.json");
        let data = serde_json::to_string_pretty(&state);
        assert!(data.is_ok());
        if let Ok(json) = data {
            assert!(std::fs::write(&tmp_path, &json).is_ok());
            let back_text = std::fs::read_to_string(&tmp_path);
            assert!(back_text.is_ok());
            if let Ok(text) = back_text {
                let back = serde_json::from_str::<CrystalState>(&text);
                assert!(back.is_ok());
                if let Ok(s2) = back {
                    assert!(s2.workspace.is_some());
                    assert_eq!(s2.agents.len(), 1);
                    assert_eq!(s2.skills.len(), 1);
                    assert_eq!(s2.tasks.len(), 1);
                    assert_eq!(s2.tools.len(), 1);
                    assert_eq!(s2.skills[0].id, "sk-t25");
                    assert_eq!(s2.tasks[0].status, "active");
                    assert_eq!(s2.tools[0].name, "read");
                }
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
