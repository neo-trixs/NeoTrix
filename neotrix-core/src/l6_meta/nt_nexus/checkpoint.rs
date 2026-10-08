// Checkpoint Provenance — Atlas-inspired session lineage tracking
//
// A Checkpoint captures the full state of a session at a point in time:
// - Session metadata (agent, timestamp, parent checkpoint)
// - Decision trace (what was decided and why)
// - Tool call history (what actions were taken)
// - State diff (what changed since parent checkpoint)
// - Causal links (which previous checkpoints influenced this one)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ⚠️ `NexusCore` 曾在此 import 但**从未被使用** —— 文件里只有两处提到它，
//    且都在**文档注释**中（"same pattern as NexusCore" /
//    "cross-referencing with NexusCore"）⇒ 编译器判定为 unused import。
// ⇒ 接入时删掉它。`ExperienceRef` 保留（下面真用到）。
use super::ExperienceRef;
use crate::l5_cognition::l1_facade::KnowledgeBase;

// ─── Domain Types ───

/// A decision made during a session checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub description: String,
    pub reasoning: String,
    pub confidence: f64,
    pub predecessor: Option<String>,
}

/// Record of a tool invocation at a checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub args_hash: String,
    pub result_summary: String,
    pub duration_ms: u64,
    pub success: bool,
}

/// Snapshot of what changed since the parent checkpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

/// A point-in-time snapshot of session state with full provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub session_id: String,
    pub agent_id: String,
    pub parent_id: Option<String>,
    pub timestamp: u64,
    pub decisions: Vec<Decision>,
    pub tool_calls: Vec<ToolCallRecord>,
    pub state_diff: StateDiff,
    pub causal_links: Vec<String>,
}

/// Checkpoint summary for chain navigation (avoids loading full checkpoint bodies).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointSummary {
    pub id: String,
    pub session_id: String,
    pub agent_id: String,
    pub parent_id: Option<String>,
    pub timestamp: u64,
    pub decision_count: usize,
    pub tool_call_count: usize,
    pub causal_links: Vec<String>,
}

impl From<&Checkpoint> for CheckpointSummary {
    fn from(cp: &Checkpoint) -> Self {
        Self {
            id: cp.id.clone(),
            session_id: cp.session_id.clone(),
            agent_id: cp.agent_id.clone(),
            parent_id: cp.parent_id.clone(),
            timestamp: cp.timestamp,
            decision_count: cp.decisions.len(),
            tool_call_count: cp.tool_calls.len(),
            causal_links: cp.causal_links.clone(),
        }
    }
}

// ─── Checkpoint Chain Navigation ───

/// A navigable chain of checkpoints following parent lineage.
pub struct CheckpointChain {
    summaries: Vec<CheckpointSummary>,
}

impl CheckpointChain {
    /// Build a chain from a leaf checkpoint tracing backwards through parent links.
    pub fn new(summaries: Vec<CheckpointSummary>) -> Self {
        Self { summaries }
    }

    /// Number of checkpoints in the chain.
    pub fn len(&self) -> usize {
        self.summaries.len()
    }

    /// Whether the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.summaries.is_empty()
    }

    /// Iterate from leaf to root (most recent first).
    pub fn iter(&self) -> impl Iterator<Item = &CheckpointSummary> {
        self.summaries.iter()
    }

    /// Get the leaf (most recent) checkpoint.
    pub fn leaf(&self) -> Option<&CheckpointSummary> {
        self.summaries.first()
    }

    /// Get the root (oldest) checkpoint.
    pub fn root(&self) -> Option<&CheckpointSummary> {
        self.summaries.last()
    }

    /// Total decisions across the chain.
    pub fn total_decisions(&self) -> usize {
        self.summaries.iter().map(|s| s.decision_count).sum()
    }

    /// Total tool calls across the chain.
    pub fn total_tool_calls(&self) -> usize {
        self.summaries.iter().map(|s| s.tool_call_count).sum()
    }
}

// ─── Checkpoint Store ───

/// KB namespace for checkpoint persistence.
const CHECKPOINT_NS: &str = "nexus_checkpoint";
/// Key for the checkpoint index (list of all checkpoint IDs).
const KEY_INDEX: &str = "__index__";

/// Persistent store for checkpoints backed by KB kv_store.
pub struct CheckpointStore {
    /// In-memory checkpoint index (id → summary).
    index: HashMap<String, CheckpointSummary>,
    /// KB handle (optional, same pattern as NexusCore).
    kb: Option<KnowledgeBase>,
}

impl CheckpointStore {
    /// Create an empty store (no KB).
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            kb: None,
        }
    }

    /// Bind a KB and load persisted checkpoints.
    pub fn with_kb(kb: KnowledgeBase) -> Self {
        let mut store = Self {
            index: HashMap::new(),
            kb: Some(kb),
        };
        if let Err(e) = store.load_index() {
            log::warn!("[CheckpointStore] 首次加载 KB 失败, 使用空状态: {}", e);
        }
        store
    }

    /// Bind KB at runtime (deferred wiring).
    pub fn set_kb(&mut self, kb: KnowledgeBase) {
        self.kb = Some(kb);
        if let Err(e) = self.load_index() {
            log::warn!("[CheckpointStore] 绑定 KB 后加载失败: {}", e);
        }
    }

    /// Generate a checkpoint ID from current timestamp.
    pub fn generate_id(prefix: &str) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;
        format!("{}_{}", prefix, ts)
    }

    /// Save a checkpoint to the store (index + full body).
    pub fn save(&mut self, checkpoint: Checkpoint) -> Result<(), String> {
        let kb = self.kb.as_ref().ok_or("CheckpointStore 未绑定 KB")?;
        let id = checkpoint.id.clone();
        let summary = CheckpointSummary::from(&checkpoint);

        // Persist full checkpoint body
        let body_json = serde_json::to_string(&checkpoint)
            .map_err(|e| format!("序列化 checkpoint 失败: {}", e))?;
        kb.kv_set(CHECKPOINT_NS, &id, &body_json)?;

        // Update in-memory index
        self.index.insert(id, summary);

        // Persist index
        self.save_index()?;

        log::debug!(
            "[CheckpointStore] 已保存 checkpoint: {} (总计 {})",
            checkpoint.id,
            self.index.len(),
        );
        Ok(())
    }

    /// Load a full checkpoint by ID.
    pub fn load(&self, id: &str) -> Result<Option<Checkpoint>, String> {
        let kb = self.kb.as_ref().ok_or("CheckpointStore 未绑定 KB")?;
        match kb.kv_get(CHECKPOINT_NS, id)? {
            Some(json) => {
                let cp: Checkpoint = serde_json::from_str(&json)
                    .map_err(|e| format!("反序列化 checkpoint 失败: {}", e))?;
                Ok(Some(cp))
            }
            None => Ok(None),
        }
    }

    /// List all checkpoint summaries, ordered by timestamp descending.
    pub fn list(&self) -> Vec<&CheckpointSummary> {
        let mut entries: Vec<&CheckpointSummary> = self.index.values().collect();
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries
    }

    /// Compute the state diff between two checkpoints (a → b).
    ///
    /// Returns a synthetic diff describing what changed from checkpoint `a` to `b`:
    /// - Files added in `b` but not in `a`
    /// - Files removed in `b` but present in `a`
    /// - Files modified (present in both but in different state_diff.modified lists)
    pub fn diff(&self, a_id: &str, b_id: &str) -> Result<StateDiff, String> {
        let a = self
            .load(a_id)?
            .ok_or_else(|| format!("checkpoint {} 不存在", a_id))?;
        let b = self
            .load(b_id)?
            .ok_or_else(|| format!("checkpoint {} 不存在", b_id))?;

        let a_modified: std::collections::HashSet<&str> =
            a.state_diff.modified.iter().map(|s| s.as_str()).collect();
        let b_modified: std::collections::HashSet<&str> =
            b.state_diff.modified.iter().map(|s| s.as_str()).collect();

        let a_added: std::collections::HashSet<&str> =
            a.state_diff.added.iter().map(|s| s.as_str()).collect();
        let b_added: std::collections::HashSet<&str> =
            b.state_diff.added.iter().map(|s| s.as_str()).collect();

        let a_removed: std::collections::HashSet<&str> =
            a.state_diff.removed.iter().map(|s| s.as_str()).collect();
        let b_removed: std::collections::HashSet<&str> =
            b.state_diff.removed.iter().map(|s| s.as_str()).collect();

        // Files in b.added but not in a.added (new additions relative to a)
        let added: Vec<String> = b_added
            .difference(&a_added)
            .map(|s| s.to_string())
            .collect();

        // Files in b.removed but not in a.removed (new removals relative to a)
        let removed: Vec<String> = b_removed
            .difference(&a_removed)
            .map(|s| s.to_string())
            .collect();

        // Files modified in b but not in a (or modified differently)
        let modified: Vec<String> = b_modified
            .symmetric_difference(&a_modified)
            .map(|s| s.to_string())
            .collect();

        Ok(StateDiff {
            added,
            removed,
            modified,
        })
    }

    /// Trace back from a checkpoint through its parent chain.
    ///
    /// Returns checkpoints ordered from most recent to oldest (leaf → root).
    pub fn trace_back(&self, start_id: &str) -> Result<CheckpointChain, String> {
        let mut chain = Vec::new();
        let mut current_id = Some(start_id.to_string());
        let mut visited = std::collections::HashSet::new();

        while let Some(id) = current_id {
            if !visited.insert(id.clone()) {
                break; // cycle guard
            }
            let summary = self
                .index
                .get(&id)
                .cloned()
                .ok_or_else(|| format!("checkpoint {} 不在索引中", id))?;
            current_id = summary.parent_id.clone();
            chain.push(summary);
        }

        Ok(CheckpointChain::new(chain))
    }

    /// Delete a checkpoint and remove it from the index.
    pub fn delete(&mut self, id: &str) -> Result<bool, String> {
        let kb = self.kb.as_ref().ok_or("CheckpointStore 未绑定 KB")?;
        if self.index.remove(id).is_some() {
            kb.kv_delete(CHECKPOINT_NS, id)?;
            self.save_index()?;
            log::debug!("[CheckpointStore] 已删除 checkpoint: {}", id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Number of checkpoints in the index.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Get a summary by ID.
    pub fn summary(&self, id: &str) -> Option<&CheckpointSummary> {
        self.index.get(id)
    }

    /// Find checkpoints by session ID.
    pub fn by_session(&self, session_id: &str) -> Vec<&CheckpointSummary> {
        let mut found: Vec<&CheckpointSummary> = self
            .index
            .values()
            .filter(|s| s.session_id == session_id)
            .collect();
        found.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        found
    }

    /// Find checkpoints by agent ID.
    pub fn by_agent(&self, agent_id: &str) -> Vec<&CheckpointSummary> {
        let mut found: Vec<&CheckpointSummary> = self
            .index
            .values()
            .filter(|s| s.agent_id == agent_id)
            .collect();
        found.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        found
    }

    /// Convert checkpoints to ExperienceRef for cross-referencing with NexusCore.
    pub fn to_experience_refs(&self, domain: &str) -> Vec<ExperienceRef> {
        self.index
            .values()
            .map(|s| ExperienceRef {
                session_id: s.session_id.clone(),
                cycle: s.decision_count as u32,
                summary: format!(
                    "checkpoint:{}:decisions:{}:tools:{}",
                    s.id, s.decision_count, s.tool_call_count
                ),
                domain: domain.to_string(),
                timestamp: s.timestamp,
            })
            .collect()
    }

    // ─── Internal persistence helpers ───

    fn save_index(&self) -> Result<(), String> {
        let kb = self.kb.as_ref().ok_or("CheckpointStore 未绑定 KB")?;
        let index_json =
            serde_json::to_string(&self.index).map_err(|e| format!("序列化索引失败: {}", e))?;
        kb.kv_set(CHECKPOINT_NS, KEY_INDEX, &index_json)
    }

    fn load_index(&mut self) -> Result<(), String> {
        let kb = self.kb.as_ref().ok_or("CheckpointStore 未绑定 KB")?;
        if let Some(index_json) = kb.kv_get(CHECKPOINT_NS, KEY_INDEX)? {
            let loaded: HashMap<String, CheckpointSummary> = serde_json::from_str(&index_json)
                .map_err(|e| format!("反序列化索引失败: {}", e))?;
            self.index = loaded;
        }
        log::debug!(
            "[CheckpointStore] 已从 KB 加载索引: {} 个 checkpoint",
            self.index.len(),
        );
        Ok(())
    }
}

impl Default for CheckpointStore {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Builder Helpers ───

impl Checkpoint {
    /// Create a new checkpoint with a generated ID and current timestamp.
    pub fn new(session_id: &str, agent_id: &str) -> Self {
        let id = CheckpointStore::generate_id("cp");
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id,
            session_id: session_id.to_string(),
            agent_id: agent_id.to_string(),
            parent_id: None,
            timestamp,
            decisions: Vec::new(),
            tool_calls: Vec::new(),
            state_diff: StateDiff::default(),
            causal_links: Vec::new(),
        }
    }

    /// Set the parent checkpoint (for lineage).
    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.parent_id = Some(parent_id.to_string());
        self
    }

    /// Add a decision to this checkpoint.
    pub fn with_decision(mut self, decision: Decision) -> Self {
        self.decisions.push(decision);
        self
    }

    /// Add a tool call record.
    pub fn with_tool_call(mut self, record: ToolCallRecord) -> Self {
        self.tool_calls.push(record);
        self
    }

    /// Set the state diff.
    pub fn with_state_diff(mut self, diff: StateDiff) -> Self {
        self.state_diff = diff;
        self
    }

    /// Add a causal link to a previous checkpoint.
    pub fn with_causal_link(mut self, checkpoint_id: &str) -> Self {
        self.causal_links.push(checkpoint_id.to_string());
        self
    }

    /// Create a Decision with a generated ID.
    pub fn make_decision(
        description: &str,
        reasoning: &str,
        confidence: f64,
        predecessor: Option<String>,
    ) -> Decision {
        let id = CheckpointStore::generate_id("dec");
        Decision {
            id,
            description: description.to_string(),
            reasoning: reasoning.to_string(),
            confidence,
            predecessor,
        }
    }

    /// Create a ToolCallRecord.
    pub fn make_tool_call(
        tool_name: &str,
        args_hash: &str,
        result_summary: &str,
        duration_ms: u64,
        success: bool,
    ) -> ToolCallRecord {
        ToolCallRecord {
            tool_name: tool_name.to_string(),
            args_hash: args_hash.to_string(),
            result_summary: result_summary.to_string(),
            duration_ms,
            success,
        }
    }
}

/// 从 `tool_hook` 闭包到 `ToolCallRecord` 的适配器：返回一个
/// `(hook_closure, shared_buffer)` 对。调用方把 hook 传给
/// `AgentLoop::with_tool_hook`，稍后从 buffer 取出记录 `make_tool_call`
/// 落进 Checkpoint —— 完成 L1→L6 的 checkpoint 接线（依赖倒置：
/// L1 只持有闭包，L6 提供构造方）。
pub fn checkpoint_tool_hook() -> (
    std::sync::Arc<dyn Fn(&str, bool) + Send + Sync>,
    std::sync::Arc<std::sync::Mutex<Vec<ToolCallRecord>>>,
) {
    let buf: std::sync::Arc<std::sync::Mutex<Vec<ToolCallRecord>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let buf2 = buf.clone();
    let hook = std::sync::Arc::new(move |name: &str, success: bool| {
        if let Ok(mut v) = buf2.lock() {
            v.push(ToolCallRecord {
                tool_name: name.to_string(),
                args_hash: String::new(),
                result_summary: String::new(),
                duration_ms: 0,
                success,
            });
        }
    });
    (hook, buf)
}

/// Resume 判据（LongHorizon-Harness 式 plan→act→verify→checkpoint→recover）：
/// 续跑时**跳过**已成功记录的工具调用，**重跑**失败/未记录的。
/// 纯函数，不读写任何外部状态（与 `nt_crawl_sources` 的 pending/completed
/// 位同型：at-least-once，宁可重跑不漏跑）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeVerdict {
    /// 可跳过的工具调用名（已成功记录）。
    pub skip: Vec<String>,
    /// 必须重跑的工具调用名（失败记录）。
    pub rerun: Vec<String>,
}

/// 从 checkpoint 推出 resume 动作。
#[must_use]
pub fn resume_verdict(cp: &Checkpoint) -> ResumeVerdict {
    let mut skip = Vec::new();
    let mut rerun = Vec::new();
    for tc in &cp.tool_calls {
        if tc.success {
            skip.push(tc.tool_name.clone());
        } else {
            rerun.push(tc.tool_name.clone());
        }
    }
    ResumeVerdict { skip, rerun }
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_new_generates_id() {
        let cp = Checkpoint::new("session_1", "agent_alpha");
        assert!(cp.id.starts_with("cp_"));
        assert_eq!(cp.session_id, "session_1");
        assert_eq!(cp.agent_id, "agent_alpha");
        assert!(cp.parent_id.is_none());
        assert!(cp.decisions.is_empty());
        assert!(cp.tool_calls.is_empty());
    }

    #[test]
    fn test_checkpoint_builder_chain() {
        let dec1 = Checkpoint::make_decision("do X", "because Y", 0.9, None);
        let dec2 = Checkpoint::make_decision("do Z", "because W", 0.8, Some(dec1.id.clone()));
        let tool = Checkpoint::make_tool_call("grep", "abc123", "found 5 matches", 42, true);

        let cp = Checkpoint::new("s1", "a1")
            .with_parent("cp_0")
            .with_decision(dec1)
            .with_decision(dec2)
            .with_tool_call(tool)
            .with_state_diff(StateDiff {
                added: vec!["src/new.rs".into()],
                removed: vec!["src/old.rs".into()],
                modified: vec!["src/main.rs".into()],
            })
            .with_causal_link("cp_prior");

        assert_eq!(cp.decisions.len(), 2);
        assert_eq!(cp.tool_calls.len(), 1);
        assert_eq!(cp.state_diff.added.len(), 1);
        assert!(cp.causal_links.contains(&"cp_prior".to_string()));
    }

    #[test]
    fn test_checkpoint_chain_navigation() {
        let summaries = vec![
            CheckpointSummary {
                id: "cp_3".into(),
                session_id: "s1".into(),
                agent_id: "a1".into(),
                parent_id: Some("cp_2".into()),
                timestamp: 300,
                decision_count: 2,
                tool_call_count: 1,
                causal_links: vec![],
            },
            CheckpointSummary {
                id: "cp_2".into(),
                session_id: "s1".into(),
                agent_id: "a1".into(),
                parent_id: Some("cp_1".into()),
                timestamp: 200,
                decision_count: 1,
                tool_call_count: 0,
                causal_links: vec![],
            },
            CheckpointSummary {
                id: "cp_1".into(),
                session_id: "s1".into(),
                agent_id: "a1".into(),
                parent_id: None,
                timestamp: 100,
                decision_count: 3,
                tool_call_count: 2,
                causal_links: vec![],
            },
        ];

        let chain = CheckpointChain::new(summaries);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.leaf().unwrap().id, "cp_3");
        assert_eq!(chain.root().unwrap().id, "cp_1");
        assert_eq!(chain.total_decisions(), 6);
        assert_eq!(chain.total_tool_calls(), 3);
    }

    #[test]
    fn test_diff_computation() {
        let mut a = Checkpoint::new("s", "a");
        a.state_diff = StateDiff {
            added: vec!["a.rs".into(), "b.rs".into()],
            removed: vec!["x.rs".into()],
            modified: vec!["m.rs".into()],
        };

        let mut b = Checkpoint::new("s", "a");
        b.state_diff = StateDiff {
            added: vec!["a.rs".into(), "c.rs".into()],
            removed: vec!["x.rs".into(), "y.rs".into()],
            modified: vec!["n.rs".into()],
        };

        // Simulate diff logic inline (same as CheckpointStore::diff)
        let a_added: std::collections::HashSet<&str> =
            a.state_diff.added.iter().map(|s| s.as_str()).collect();
        let b_added: std::collections::HashSet<&str> =
            b.state_diff.added.iter().map(|s| s.as_str()).collect();
        let a_removed: std::collections::HashSet<&str> =
            a.state_diff.removed.iter().map(|s| s.as_str()).collect();
        let b_removed: std::collections::HashSet<&str> =
            b.state_diff.removed.iter().map(|s| s.as_str()).collect();
        let a_modified: std::collections::HashSet<&str> =
            a.state_diff.modified.iter().map(|s| s.as_str()).collect();
        let b_modified: std::collections::HashSet<&str> =
            b.state_diff.modified.iter().map(|s| s.as_str()).collect();

        let added: Vec<&str> = b_added.difference(&a_added).copied().collect();
        let removed: Vec<&str> = b_removed.difference(&a_removed).copied().collect();
        let modified: Vec<&str> = b_modified
            .symmetric_difference(&a_modified)
            .copied()
            .collect();

        assert!(added.contains(&"c.rs"));
        assert!(!added.contains(&"a.rs")); // already in a
        assert!(removed.contains(&"y.rs"));
        assert!(modified.contains(&"m.rs"));
        assert!(modified.contains(&"n.rs"));
        assert!(!modified.contains(&"a.rs"));
    }

    #[test]
    fn test_decision_predecessor_chain() {
        let d1 = Checkpoint::make_decision("first", "reason1", 1.0, None);
        let d2 = Checkpoint::make_decision("second", "reason2", 0.9, Some(d1.id.clone()));
        let d3 = Checkpoint::make_decision("third", "reason3", 0.8, Some(d2.id.clone()));

        assert!(d1.predecessor.is_none());
        assert_eq!(d2.predecessor.as_deref(), Some(d1.id.as_str()));
        assert_eq!(d3.predecessor.as_deref(), Some(d2.id.as_str()));
    }

    #[test]
    fn test_to_experience_refs() {
        let mut store = CheckpointStore::new();
        let cp = Checkpoint::new("s1", "a1")
            .with_decision(Checkpoint::make_decision("d1", "r1", 1.0, None));
        // Manually insert into index (without KB)
        store
            .index
            .insert(cp.id.clone(), CheckpointSummary::from(&cp));

        let refs = store.to_experience_refs("nexus");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].session_id, "s1");
        assert_eq!(refs[0].domain, "nexus");
        assert!(refs[0].summary.contains("decisions:1"));
    }

    #[test]
    fn resume_verdict_splits_success_and_failure() {
        let mut cp = Checkpoint::new("s1", "a1");
        cp.tool_calls
            .push(Checkpoint::make_tool_call("search", "h1", "ok", 10, true));
        cp.tool_calls
            .push(Checkpoint::make_tool_call("delete", "h2", "err", 5, false));
        let v = resume_verdict(&cp);
        assert_eq!(v.skip, vec!["search".to_string()]);
        assert_eq!(v.rerun, vec!["delete".to_string()]);
    }

    #[test]
    fn checkpoint_tool_hook_collects_and_feeds_resume_verdict() {
        let (hook, buf) = checkpoint_tool_hook();
        hook("search", true);
        hook("delete", false);
        let records = std::mem::take(&mut *buf.lock().unwrap());
        assert_eq!(records.len(), 2);
        let mut cp = Checkpoint::new("s1", "a1");
        for r in records {
            cp.tool_calls.push(Checkpoint::make_tool_call(
                &r.tool_name, &r.args_hash, &r.result_summary, r.duration_ms, r.success,
            ));
        }
        let v = resume_verdict(&cp);
        assert_eq!(v.skip, vec!["search".to_string()]);
        assert_eq!(v.rerun, vec!["delete".to_string()]);
    }

    #[test]
    fn resume_verdict_empty_checkpoint_reruns_nothing() {
        let cp = Checkpoint::new("s1", "a1");
        let v = resume_verdict(&cp);
        assert!(v.skip.is_empty());
        assert!(v.rerun.is_empty());
    }

    #[test]
    fn test_generate_id_format() {
        let id = CheckpointStore::generate_id("cp");
        assert!(id.starts_with("cp_"));
        let parts: Vec<&str> = id.split('_').collect();
        assert_eq!(parts.len(), 2);
        let _: u64 = parts[1].parse().expect("timestamp should be numeric");
    }

    #[test]
    fn test_store_empty_without_kb() {
        let store = CheckpointStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert!(store.list().is_empty());
    }

    #[test]
    fn test_summary_from_checkpoint() {
        let mut cp = Checkpoint::new("s1", "a1");
        cp.decisions
            .push(Checkpoint::make_decision("d1", "r1", 1.0, None));
        cp.tool_calls
            .push(Checkpoint::make_tool_call("t1", "h1", "ok", 10, true));

        let summary = CheckpointSummary::from(&cp);
        assert_eq!(summary.id, cp.id);
        assert_eq!(summary.decision_count, 1);
        assert_eq!(summary.tool_call_count, 1);
    }
}
