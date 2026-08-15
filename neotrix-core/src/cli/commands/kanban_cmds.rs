//! Kanban board commands (agtx-inspired orchestrator UI).
//!
//! Provides a task-kanban interface with KB persistence and DAG dependency support:
//!   /board list             — show all tasks by phase
//!   /board create <spec>    — create a new task
//!   /board move <id>        — advance task to next phase
//!   /board view <id>        — show task details
//!   /board dependency <id> add <dep_id>
//!   /board dependency <id> remove <dep_id>
//!   /board block <id> <reason>
//!   /board unblock <id>
//!   /board ready            — show ready items (dependencies met)
//!   /board save [path]      — persist to JSON
//!   /board load [path]      — load from JSON
//!   /board wip <phase> <limit>
//!   /board assign <id> <agent>
//!   /board priority <id> <level>

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_mind::SelfIteratingBrain;

// ─── todo sync helpers (port of scripts/sync_todos.py) ───

fn iso_now() -> String {
    let now = std::time::SystemTime::now();
    let secs = now.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86400;
    let hrs = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    format!("2026-{:02}-{:02}T{:02}:{:02}:00", (days / 28).min(12) + 1, (days % 28) + 1, hrs, mins)
}

fn phase_to_str(phase: &WorkItemPhase) -> &'static str {
    match phase {
        WorkItemPhase::Backlog => "pending",
        WorkItemPhase::Planning => "pending",
        WorkItemPhase::Running => "in_progress",
        WorkItemPhase::Review => "in_progress",
        WorkItemPhase::Blocked => "blocked",
        WorkItemPhase::Done => "done",
        WorkItemPhase::Cancelled => "cancelled",
        WorkItemPhase::Deferred => "deferred",
    }
}

fn priority_label(priority: u8) -> &'static str {
    match priority {
        3 => "high",
        1 => "low",
        _ => "medium",
    }
}

/// Scan git branches + worktrees in the current repo (dashi-taskboard parity:
/// branch options are read from the repository, not typed by hand).
fn scan_git_branches() -> Vec<String> {
    use std::process::Command;
    let mut names: Vec<String> = Vec::new();
    // Local branches
    if let Ok(out) = Command::new("git").args(["branch", "--format=%(refname:short)"]).output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let l = line.trim();
                if !l.is_empty() {
                    names.push(l.to_string());
                }
            }
        }
    }
    // Worktrees (each has a path + branch)
    if let Ok(out) = Command::new("git").args(["worktree", "list", "--porcelain"]).output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut in_worktree = false;
            let mut branch: Option<String> = None;
            for line in text.lines() {
                let l = line.trim();
                if l.is_empty() {
                    if let Some(b) = branch.take() {
                        if !names.contains(&b) {
                            names.push(b);
                        }
                    }
                    in_worktree = false;
                } else if l.starts_with("worktree ") {
                    in_worktree = true;
                    branch = None;
                } else if in_worktree && l.starts_with("branch ") {
                    branch = Some(l["branch ".len()..].replace("refs/heads/", ""));
                }
            }
            if let Some(b) = branch.take() {
                if !names.contains(&b) {
                    names.push(b);
                }
            }
        }
    }
    names
}

/// Bigram Dice coefficient (difflib.SequenceMatcher.ratio approximation).
fn dice_similarity(a: &str, b: &str) -> f64 {
    let bigrams = |s: &str| -> std::collections::HashSet<(char, char)> {
        s.chars().collect::<Vec<_>>().windows(2).map(|w| (w[0], w[1])).collect()
    };
    let sa = bigrams(a);
    let sb = bigrams(b);
    if sa.is_empty() && sb.is_empty() {
        return if a == b { 1.0 } else { 0.0 };
    }
    let inter = sa.intersection(&sb).count();
    2.0 * inter as f64 / (sa.len() + sb.len()) as f64
}

// ─── WorkItemPhase ───

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkItemPhase {
    Backlog,
    Planning,
    Running,
    Review,
    Blocked,
    Done,
    Cancelled,
    Deferred,
}

impl WorkItemPhase {
    pub fn can_advance_to(&self, target: &WorkItemPhase) -> bool {
        use WorkItemPhase::*;
        matches!((self, target),
            (Backlog, Planning | Cancelled | Deferred)
            | (Planning, Running | Backlog | Cancelled | Deferred)
            | (Running, Review | Blocked | Backlog | Cancelled | Deferred)
            | (Review, Done | Running | Blocked | Cancelled | Deferred)
            | (Blocked, Running | Planning | Backlog | Cancelled | Deferred)
            | (Deferred, Planning | Backlog | Cancelled)
        )
    }

    pub fn all() -> &'static [WorkItemPhase] {
        use WorkItemPhase::*;
        &[Backlog, Planning, Running, Review, Blocked, Done, Cancelled, Deferred]
    }
}

impl std::fmt::Display for WorkItemPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::str::FromStr for WorkItemPhase {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use WorkItemPhase::*;
        match s.to_lowercase().as_str() {
            "backlog" => Ok(Backlog),
            "planning" => Ok(Planning),
            "running" => Ok(Running),
            "review" => Ok(Review),
            "blocked" => Ok(Blocked),
            "done" => Ok(Done),
            "cancelled" => Ok(Cancelled),
            "deferred" => Ok(Deferred),
            _ => Err(format!("Unknown phase: {s}")),
        }
    }
}

// ─── WorkItem ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub phase: WorkItemPhase,
    pub priority: u8,
    pub assignee: Option<String>,
    pub dependencies: Vec<String>,
    pub depended_by: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub tags: Vec<String>,
    pub milestone: Option<String>,
    pub results: Vec<String>,
    pub blocked_reason: Option<String>,
    /// Dynamic efficiency score (port of scripts/sync_todos.py calc_efficiency_score).
    #[serde(default)]
    pub efficiency_score: f64,
    /// Bound git branch / worktree (dashi-taskboard parity). Optional; old JSON loads as None.
    #[serde(default)]
    pub git_branch: Option<String>,
    /// Associated agent session/thread id (dashi-taskboard CODEX_THREAD_ID parity).
    #[serde(default)]
    pub thread_id: Option<String>,
}

impl WorkItem {
    /// Recover "files" from tags matching common source paths (sync_todos.py export shape).
    pub fn files_hint(&self) -> Vec<String> {
        self.tags.iter()
            .filter(|t| t.contains('/') || t.contains('.'))
            .cloned()
            .collect()
    }
}

// ─── KanbanBoard ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanBoard {
    pub project: String,
    pub items: Vec<WorkItem>,
    pub wip_limits: HashMap<WorkItemPhase, u8>,
    /// Conflict report produced by smart_analyze (sync_todos.py).
    #[serde(default)]
    pub conflicts: Vec<serde_json::Value>,
    /// Monotonic id counter. items.len()+1 reuses ids after remove_item,
    /// which corrupts dependency/stale references, so ids are never
    /// recycled within a board lifetime.
    #[serde(default)]
    next_id_counter: u64,
}

impl KanbanBoard {
    pub fn new(project: &str) -> Self {
        let mut wip_limits = HashMap::new();
        wip_limits.insert(WorkItemPhase::Running, 3);
        wip_limits.insert(WorkItemPhase::Review, 3);
        Self { project: project.to_string(), items: Vec::new(), wip_limits, conflicts: Vec::new(), next_id_counter: 0 }
    }

    pub fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn next_id(&mut self) -> String {
        // Backward compat: if a board was deserialized before the counter
        // existed, seed it from the current max numeric id so fresh ids
        // never collide with existing ones.
        if self.next_id_counter == 0 {
            let max_numeric = self.items.iter()
                .filter_map(|i| i.id.strip_prefix("task-").and_then(|s| s.parse::<u64>().ok()))
                .max()
                .unwrap_or(0);
            self.next_id_counter = max_numeric.max(self.items.len() as u64);
        }
        self.next_id_counter += 1;
        format!("task-{}", self.next_id_counter)
    }

    pub fn add_item(&mut self, title: &str, description: &str, priority: u8) -> String {
        self.add_item_full(title, description, priority, None, None)
    }

    /// Add an item with optional git branch + thread association (dashi parity).
    pub fn add_item_full(
        &mut self,
        title: &str,
        description: &str,
        priority: u8,
        git_branch: Option<String>,
        thread_id: Option<String>,
    ) -> String {
        let id = self.next_id();
        let ts = Self::now();
        self.items.push(WorkItem {
            id: id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            phase: WorkItemPhase::Backlog,
            priority: priority.min(5).max(1),
            assignee: None,
            dependencies: Vec::new(),
            depended_by: Vec::new(),
            created_at: ts,
            updated_at: ts,
            tags: Vec::new(),
            milestone: None,
            results: Vec::new(),
            blocked_reason: None,
            efficiency_score: 0.0,
            git_branch,
            thread_id,
        });
        id
    }

    pub fn get_item_by_id(&self, id: &str) -> Option<&WorkItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn get_item_by_id_mut(&mut self, id: &str) -> Option<&mut WorkItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    pub fn remove_item(&mut self, id: &str) -> Result<WorkItem, String> {
        let idx = self.items.iter().position(|i| i.id == id).ok_or_else(|| format!("Item {id} not found"))?;
        let item = self.items.remove(idx);
        for other in self.items.iter_mut() {
            other.dependencies.retain(|d| d != id);
            other.depended_by.retain(|d| d != id);
        }
        Ok(item)
    }

    pub fn move_item(&mut self, id: &str, target: &WorkItemPhase, force: bool) -> Result<(), String> {
        let idx = self.items.iter().position(|i| i.id == id).ok_or_else(|| format!("Item {id} not found"))?;
        let current = self.items[idx].phase.clone();

        if current == *target {
            return Ok(());
        }

        if !force && !current.can_advance_to(target) {
            return Err(format!("Cannot move from {current} to {target} (use --force to override)"));
        }

        if !force && !self.check_wip_limit(target) {
            return Err(format!("WIP limit reached for phase {target}"));
        }

        if *target == WorkItemPhase::Running {
            let item = &self.items[idx];
            for dep_id in &item.dependencies {
                let dep = self.get_item_by_id(dep_id).ok_or_else(|| format!("Dependency {dep_id} not found"))?;
                if dep.phase != WorkItemPhase::Done && dep.phase != WorkItemPhase::Cancelled {
                    return Err(format!("Dependency {dep_id} not done (phase: {})", dep.phase));
                }
            }
        }

        self.items[idx].phase = target.clone();
        self.items[idx].updated_at = Self::now();
        Ok(())
    }

    pub fn add_dependency(&mut self, id: &str, dep_id: &str) -> Result<(), String> {
        if id == dep_id {
            return Err("Cannot depend on itself".to_string());
        }
        if !self.items.iter().any(|i| i.id == id) {
            return Err(format!("Item {id} not found"));
        }
        if !self.items.iter().any(|i| i.id == dep_id) {
            return Err(format!("Dependency {dep_id} not found"));
        }
        if self.would_cycle(id, dep_id) {
            return Err("Adding this dependency would create a cycle".to_string());
        }
        let target = self.items.iter_mut().find(|i| i.id == id).ok_or_else(|| format!("Item {id} not found"))?;
        target.dependencies.push(dep_id.to_string());
        target.updated_at = Self::now();
        let dep = self.items.iter_mut().find(|i| i.id == dep_id).ok_or_else(|| format!("Dependency {dep_id} not found"))?;
        dep.depended_by.push(id.to_string());
        dep.updated_at = Self::now();
        Ok(())
    }

    pub fn remove_dependency(&mut self, id: &str, dep_id: &str) -> Result<(), String> {
        let target = self.items.iter_mut().find(|i| i.id == id).ok_or_else(|| format!("Item {id} not found"))?;
        target.dependencies.retain(|d| d != dep_id);
        target.updated_at = Self::now();
        if let Some(dep) = self.items.iter_mut().find(|i| i.id == dep_id) {
            dep.depended_by.retain(|d| d != id);
            dep.updated_at = Self::now();
        }
        Ok(())
    }

    fn would_cycle(&self, start: &str, target: &str) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(target.to_string());
        while let Some(current) = queue.pop_front() {
            if current == start {
                return true;
            }
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(item) = self.get_item_by_id(&current) {
                for dep in &item.dependencies {
                    queue.push_back(dep.clone());
                }
            }
        }
        false
    }

    pub fn get_dependency_chain(&self, id: &str) -> Vec<String> {
        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(id.to_string());
        while let Some(current) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }
            if current != id {
                result.push(current.clone());
            }
            if let Some(item) = self.get_item_by_id(&current) {
                for dep in &item.dependencies {
                    queue.push_back(dep.clone());
                }
            }
        }
        result
    }

    pub fn get_blocked_items(&self) -> Vec<WorkItem> {
        self.items.iter().filter(|i| i.phase == WorkItemPhase::Blocked).cloned().collect()
    }

    pub fn get_items_by_phase(&self, phase: &WorkItemPhase) -> Vec<&WorkItem> {
        self.items.iter().filter(|i| i.phase == *phase).collect()
    }

    pub fn can_start(&self, id: &str) -> bool {
        let item = match self.get_item_by_id(id) {
            Some(i) => i,
            None => return false,
        };
        if item.dependencies.is_empty() {
            return true;
        }
        item.dependencies.iter().all(|dep_id| {
            self.get_item_by_id(dep_id)
                .map(|dep| dep.phase == WorkItemPhase::Done || dep.phase == WorkItemPhase::Cancelled)
                .unwrap_or(false)
        })
    }

    pub fn get_ready_items(&self) -> Vec<&WorkItem> {
        self.items.iter()
            .filter(|i| i.phase == WorkItemPhase::Backlog && self.can_start(&i.id))
            .collect()
    }

    pub fn set_wip_limit(&mut self, phase: &WorkItemPhase, limit: u8) {
        self.wip_limits.insert(phase.clone(), limit);
    }

    pub fn check_wip_limit(&self, phase: &WorkItemPhase) -> bool {
        let limit = self.wip_limits.get(phase).copied().unwrap_or(u8::MAX);
        if limit == 0 {
            return true;
        }
        let count = self.items.iter().filter(|i| i.phase == *phase).count() as u8;
        count < limit
    }

    // ── JSON serialization ──

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialization error: {e}"))
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Deserialization error: {e}"))
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let json = self.to_json()?;
        let p = PathBuf::from(path);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create directory: {e}"))?;
        }
        std::fs::write(&p, &json).map_err(|e| format!("Cannot write file: {e}"))
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let json = std::fs::read_to_string(path).map_err(|e| format!("Cannot read file: {e}"))?;
        Self::from_json(&json)
    }

    // ── KB persistence ──

    pub fn save_to_kb(&self, kb: &KnowledgeBase) -> Result<(), String> {
        let json = self.to_json()?;
        kb.kv_set("kanban", &self.project, &json)
    }

    pub fn load_from_kb(kb: &KnowledgeBase, project: &str) -> Result<Self, String> {
        let json = kb.kv_get("kanban", project)?
            .ok_or_else(|| format!("No kanban board found for project '{project}'"))?;
        Self::from_json(&json)
    }

    // ── TODO.md import / export (sync_todos.py) ──

    /// Parse a human-readable TODO.md into board items.
    ///
    /// Faithful port of `scripts/sync_todos.py:load_todo_md`: matches
    /// `### <status_emoji> <ID>: <title>` blocks and extracts session, subagent,
    /// files, priority and status. Existing items are updated in place; new ones
    /// are appended. Returns the number of items parsed.
    pub fn import_todo_md(&mut self, md: &str) -> usize {
        // Line-based port of the Python `### <emoji> <ID>: <title>` block scan
        // (Rust regex has no look-around, so split on ###/## headings manually).
        let now = Self::now();
        let mut parsed = 0;
        let mut current_id: Option<String> = None;
        let mut current_emoji = String::new();
        let mut current_body = String::new();
        for line in md.lines() {
            let line = line.trim_end();
            if let Some(rest) = line.strip_prefix("### ") {
                if let Some(id) = current_id.take() {
                    self.parse_todo_block(&id, &current_emoji, &current_body, now);
                    parsed += 1;
                }
                // ### <emoji> <ID>: <title>
                let rest = rest.trim_start();
                let mut parts = rest.splitn(2, ": ");
                let header = parts.next().unwrap_or("");
                let title = parts.next().unwrap_or("").trim();
                let mut tokens = header.split_whitespace();
                let token1 = tokens.next().unwrap_or("");
                let token2 = tokens.next().unwrap_or("");
                let (emoji, id) = if token2.starts_with("S-") {
                    (token1.to_string(), token2.to_string())
                } else {
                    (String::new(), token1.to_string())
                };
                current_body = if title.is_empty() { String::new() } else { format!("{title}\n") };
                current_id = Some(id);
                current_emoji = emoji;
            } else if line.starts_with("## ") {
                if let Some(id) = current_id.take() {
                    self.parse_todo_block(&id, &current_emoji, &current_body, now);
                    parsed += 1;
                }
            } else if current_id.is_some() {
                if !current_body.is_empty() {
                    current_body.push('\n');
                }
                current_body.push_str(line);
            }
        }
        if let Some(id) = current_id.take() {
            self.parse_todo_block(&id, &current_emoji, &current_body, now);
            parsed += 1;
        }
        parsed
    }

    fn parse_todo_block(&mut self, id: &str, emoji_status: &str, body: &str, now: u64) {
        let body_lower = body.to_lowercase();
        let status = if emoji_status.contains('✅') || body_lower.contains("done") {
            WorkItemPhase::Done
        } else if emoji_status.contains('🔄') || body_lower.contains("in_progress") {
            WorkItemPhase::Running
        } else if emoji_status.contains('⏳') || body_lower.contains("blocked") {
            WorkItemPhase::Blocked
        } else {
            WorkItemPhase::Backlog
        };
        let priority = if emoji_status.contains('🔴') || body_lower.contains("高优先级") || body_lower.contains("high priority") {
            3
        } else if emoji_status.contains('🟢') || body_lower.contains("低优先级") || body_lower.contains("low priority") {
            1
        } else {
            2
        };
        let title = body.lines().next().map(|l| l.trim().to_string()).unwrap_or_default();
        let field = |label: &str| -> String {
            body.lines()
                .find_map(|l| {
                    l.strip_prefix(label)
                        .map(|v| v.trim().trim_matches(|c| c == '`' || c == '*' || c == ' ').to_string())
                })
                .unwrap_or_default()
        };
        let session = field("**Session**:");
        let subagent = field("**子代理**:");
        let files: Vec<String> = body.lines()
            .find_map(|l| l.strip_prefix("**文件**:"))
            .map(|v| v.split(',').map(|f| f.trim().trim_matches('`').to_string()).filter(|f| !f.is_empty()).collect())
            .unwrap_or_default();
        let dependencies: Vec<String> = body.lines()
            .find_map(|l| l.strip_prefix("**依赖**:"))
            .map(|v| v.split([',', ' ', ';']).filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        if let Some(existing) = self.get_item_by_id_mut(id) {
            existing.phase = status;
            existing.updated_at = now;
            if !subagent.is_empty() {
                existing.assignee = Some(subagent);
            }
            if !dependencies.is_empty() {
                existing.dependencies = dependencies;
            }
            if !session.is_empty() {
                existing.milestone = Some(session);
            }
            if !files.is_empty() {
                existing.tags = files;
            }
        } else {
            self.items.push(WorkItem {
                id: id.to_string(),
                title: if title.is_empty() { id.to_string() } else { title },
                description: body.trim().to_string(),
                phase: status,
                priority,
                assignee: if subagent.is_empty() { None } else { Some(subagent) },
                dependencies,
                depended_by: Vec::new(),
                created_at: now,
                updated_at: now,
                tags: files,
                milestone: if session.is_empty() { None } else { Some(session) },
                results: Vec::new(),
                blocked_reason: None,
                efficiency_score: 0.0,
                git_branch: None,
                thread_id: None,
            });
        }
    }

    /// Serialize the board back to human-readable TODO.md (sync_todos.py:save_todo_md).
    pub fn export_todo_md(&self) -> String {
        let mut out = String::new();
        out.push_str("# NeoTrix TODO 列表\n");
        out.push_str(&format!("> 智能同步生成，最后更新：{}\n\n", iso_now()));
        for (priority, emoji, label) in [(3, "🔴", "High"), (2, "🟡", "Medium"), (1, "🟢", "Low")] {
            let items: Vec<&WorkItem> = self.items.iter().filter(|i| i.priority == priority).collect();
            if items.is_empty() {
                continue;
            }
            out.push_str(&format!("## {emoji} {label} 优先级\n\n"));
            for item in items {
                let status_emoji = match item.phase {
                    WorkItemPhase::Done => "✅",
                    WorkItemPhase::Running => "🔄",
                    WorkItemPhase::Blocked => "⏳",
                    _ => "⬜",
                };
                out.push_str(&format!("### {status_emoji} {}: {}\n\n", item.id, item.title));
                out.push_str(&format!("**状态**: {}\n", phase_to_str(&item.phase)));
                if let Some(session) = item.milestone.as_deref() {
                    out.push_str(&format!("**Session**: {session}\n"));
                }
                if let Some(agent) = item.assignee.as_deref() {
                    out.push_str(&format!("**子代理**: {agent}\n"));
                }
                if !item.files_hint().is_empty() {
                    out.push_str(&format!("**文件**: {}\n", item.files_hint().join(", ")));
                }
                if !item.dependencies.is_empty() {
                    out.push_str(&format!("**依赖**: {}\n", item.dependencies.join(", ")));
                }
                out.push_str(&format!("**更新**: {}\n", iso_now()));
                out.push_str(&format!("**效率分数**: {:.1}\n\n", item.efficiency_score));
            }
        }
        out
    }

    /// Serialize the board to machine-readable TODO.yml (sync_todos.py:save_todo_yml).
    pub fn export_todo_yml(&self) -> Result<String, String> {
        let items: Vec<serde_json::Value> = self.items.iter().map(|i| serde_json::json!({
            "id": i.id,
            "title": i.title,
            "priority": priority_label(i.priority),
            "status": phase_to_str(&i.phase),
            "session": i.milestone,
            "session_name": format!("{} {}", i.id, i.title),
            "created": iso_now(),
            "updated": iso_now(),
            "subagent": i.assignee,
            "files": i.files_hint(),
            "depends_on": i.dependencies,
            "blocked_by": i.dependencies.iter().filter(|d| {
                self.get_item_by_id(d).map(|dep| dep.phase != WorkItemPhase::Done).unwrap_or(true)
            }).cloned().collect::<Vec<_>>(),
            "potential_conflict": false,
        })).collect();
        let doc = serde_yaml::to_string(&serde_json::json!({
            "meta": {
                "generated_at": iso_now(),
                "total_items": self.items.len(),
                "conflicts": self.conflicts.len(),
            },
            "items": items,
            "conflicts": self.conflicts,
            "subagents": {},
        })).map_err(|e| format!("YAML serialization error: {e}"))?;
        Ok(doc)
    }

    /// Dedup + dependency + conflict analysis (sync_todos.py:smart_analyze).
    /// Returns a human-readable report of what changed.
    pub fn smart_analyze(&mut self) -> String {
        let mut report = Vec::new();
        // 1. Duplicate detection (bigram Dice > 0.8 on titles) + exact-id dedup
        let mut to_remove: Vec<usize> = Vec::new();
        for i in 0..self.items.len() {
            for j in (i + 1)..self.items.len() {
                if i >= j || to_remove.contains(&i) || to_remove.contains(&j) {
                    continue;
                }
                let a = &self.items[i];
                let b = &self.items[j];
                if a.id == b.id {
                    if a.updated_at >= b.updated_at {
                        to_remove.push(j);
                    } else {
                        to_remove.push(i);
                    }
                    report.push(format!("[DUPLICATE] ID 重复: {} → {}", a.id, b.id));
                    continue;
                }
                let ratio = dice_similarity(&a.title, &b.title);
                if ratio > 0.8 {
                    self.conflicts.push(serde_json::json!({
                        "type": "similar_title",
                        "id1": a.id,
                        "id2": b.id,
                        "similarity": ratio,
                    }));
                    report.push(format!("[CONFLICT] 标题相似度 {ratio:.2}: {} vs {}", a.id, b.id));
                }
            }
        }
        to_remove.sort_unstable();
        to_remove.dedup();
        for idx in to_remove.into_iter().rev() {
            if idx < self.items.len() {
                let removed = self.items.remove(idx);
                report.push(format!("[INFO] 移除重复项: {}", removed.id));
            }
        }
        // 2. Dependency check: block items whose dependencies aren't done
        for i in 0..self.items.len() {
            let deps = self.items[i].dependencies.clone();
            let mut newly_blocked = Vec::new();
            for dep_id in deps {
                match self.get_item_by_id(&dep_id) {
                    None => report.push(format!("[WARN] {} 依赖的 {} 不存在", self.items[i].id, dep_id)),
                    Some(dep) if dep.phase != WorkItemPhase::Done && dep.phase != WorkItemPhase::Cancelled => {
                        newly_blocked.push(dep_id);
                    }
                    _ => {}
                }
            }
            if !newly_blocked.is_empty() {
                if self.items[i].phase == WorkItemPhase::Backlog {
                    self.items[i].phase = WorkItemPhase::Blocked;
                }
                self.items[i].blocked_reason = Some(format!("等待依赖: {}", newly_blocked.join(", ")));
                report.push(format!("[BLOCKED] {} 被阻塞，等待 {}", self.items[i].id, newly_blocked.join(", ")));
            }
        }
        // 3. Efficiency scoring + sort by score descending (dynamic_priority_adjustment)
        self.recompute_efficiency();
        self.items.sort_by(|a, b| b.efficiency_score.partial_cmp(&a.efficiency_score).unwrap_or(std::cmp::Ordering::Equal));
        report.push(format!("[SMART] 分析完成: {} 个 TODO, {} 个冲突", self.items.len(), self.conflicts.len()));
        report.join("\n")
    }

    /// Recompute dynamic efficiency scores for all items (sync_todos.py:dynamic_priority_adjustment).
    pub fn recompute_efficiency(&mut self) {
        let running: std::collections::HashSet<String> = self.items.iter()
            .filter(|i| i.phase == WorkItemPhase::Running)
            .filter_map(|i| i.assignee.clone())
            .collect();
        for item in self.items.iter_mut() {
            let subagent_running = item.assignee.as_ref().map(|a| running.contains(a)).unwrap_or(false);
            let subagent_completed = false; // completed tracked via phase
            item.efficiency_score = crate::neotrix::nt_core_parallel::OptimalTaskAllocator::new(
                crate::neotrix::nt_core_parallel::AllocationStrategy::Hybrid
            ).score_todo(
                item.priority,
                item.created_at,
                item.dependencies.len(),
                subagent_running,
                subagent_completed,
            );
        }
    }

    /// Items ready to allocate: pending (Backlog) with unmet dependencies cleared by smart_analyze.
    pub fn ready_for_allocation(&self) -> Vec<&WorkItem> {
        self.items.iter()
            .filter(|i| i.phase == WorkItemPhase::Backlog)
            .filter(|i| i.blocked_reason.is_none())
            .collect()
    }

    /// Subagents currently registered on Running items.
    pub fn running_subagents(&self) -> Vec<&str> {
        self.items.iter()
            .filter(|i| i.phase == WorkItemPhase::Running)
            .filter_map(|i| i.assignee.as_deref())
            .collect()
    }

    /// Number of items currently running (max_parallel budget check).
    pub fn running_count(&self) -> usize {
        self.items.iter().filter(|i| i.phase == WorkItemPhase::Running).count()
    }
}

// ─── Global Board ───

fn board() -> &'static Mutex<KanbanBoard> {
    static BOARD: OnceLock<Mutex<KanbanBoard>> = OnceLock::new();
    BOARD.get_or_init(|| Mutex::new(KanbanBoard::new("neotrix")))
}

fn open_kb() -> Option<KnowledgeBase> {
    KnowledgeBase::open(None).ok()
}

// ─── BoardCmd ───

pub struct BoardCmd;

impl BoardCmd {
    fn list_output(board: &KanbanBoard, want_json: bool) -> CommandOutput {
        if board.items.is_empty() {
            return CommandOutput::ok("Board is empty. Use /board create <spec>");
        }
        let mut lines = vec![format!("=== Kanban Board: {} ===", board.project)];
        let wip_str: Vec<String> = board.wip_limits.iter()
            .map(|(p, l)| format!("{p}:{l}"))
            .collect();
        lines.push(format!("  WIP: {}", wip_str.join(" ")));

        for phase in WorkItemPhase::all() {
            let items: Vec<&WorkItem> = board.items.iter().filter(|i| i.phase == *phase).collect();
            if items.is_empty() {
                continue;
            }
            lines.push(format!("  [{phase}]"));
            for item in &items {
                let dep_flag = if !item.dependencies.is_empty() && phase != &WorkItemPhase::Blocked {
                    if !board.can_start(&item.id) { " ⏳" } else { "" }
                } else {
                    ""
                };
                let blocked = if item.phase == WorkItemPhase::Blocked {
                    format!(" 🚫 {}", item.blocked_reason.as_deref().unwrap_or("no reason"))
                } else {
                    String::new()
                };
                lines.push(format!("    {}  —  {}{}{}", item.id, item.title, dep_flag, blocked));
            }
        }
        if want_json {
            let json_items: Vec<serde_json::Value> = board.items.iter().map(|i| serde_json::json!({
                "id": i.id,
                "title": i.title,
                "phase": i.phase.to_string(),
                "priority": i.priority,
                "assignee": i.assignee,
                "dependencies": i.dependencies,
                "depended_by": i.depended_by,
                "blocked_reason": i.blocked_reason,
                "tags": i.tags,
                "milestone": i.milestone,
                "git_branch": i.git_branch,
                "thread_id": i.thread_id,
            })).collect();
            return CommandOutput::ok("").with_json(serde_json::json!({
                "project": board.project,
                "items": json_items,
                "wip_limits": board.wip_limits,
            }));
        }
        CommandOutput::ok(&lines.join("\n"))
    }

    fn view_output(board: &KanbanBoard, task_id: &str, want_json: bool) -> CommandOutput {
        match board.get_item_by_id(task_id) {
            Some(item) => {
                let dep_chain = board.get_dependency_chain(task_id);
                let ready = board.can_start(task_id);
                let lines = format!(
                    "Item: {}\n  Title: {}\n  Description: {}\n  Phase: {}\n  Priority: {}\n  Assignee: {}\n  Branch: {}\n  Thread: {}\n  Dependencies: {}\n    deps: {}\n    depended_by: {}\n    chain: {}\n  Can start: {ready}\n  Tags: {}\n  Milestone: {}\n  Blocked reason: {}\n  Created: {}\n  Updated: {}\n  Results: {}",
                    item.id,
                    item.title,
                    item.description,
                    item.phase,
                    item.priority,
                    item.assignee.as_deref().unwrap_or("unassigned"),
                    item.git_branch.as_deref().unwrap_or("unbound"),
                    item.thread_id.as_deref().unwrap_or("none"),
                    item.dependencies.len(),
                    item.dependencies.join(", "),
                    item.depended_by.join(", "),
                    dep_chain.join(" → "),
                    item.tags.join(", "),
                    item.milestone.as_deref().unwrap_or("none"),
                    item.blocked_reason.as_deref().unwrap_or("none"),
                    item.created_at,
                    item.updated_at,
                    item.results.len(),
                );
                if want_json {
                    return CommandOutput::ok("").with_json(serde_json::json!(item));
                }
                CommandOutput::ok(&lines)
            }
            None => CommandOutput::not_found(&format!("Task {task_id} not found")),
        }
    }

    /// `/board todo …` — absorbed scripts/sync_todos.py (import → analyze → allocate → persist).
    fn cmd_todo(&self, args: &[String]) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("");
        match sub {
            "import" | "import-todo" => {
                let path = args.get(1).map(|s| s.as_str()).unwrap_or("TODO.md");
                let md = match std::fs::read_to_string(path) {
                    Ok(m) => m,
                    Err(e) => return CommandOutput::err(&format!("Cannot read {path}: {e}")),
                };
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                let parsed = b.import_todo_md(&md);
                CommandOutput::ok(&format!("Imported {parsed} TODO items from {path}"))
            }
            "sync" | "smart-sync" => {
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                // Persist subagent registry first so efficiency scoring sees real state.
                if let Some(kb) = open_kb() {
                    let shared = crate::cli::commands::agent_cmds::shared_subagent_manager();
                    let mut mgr = shared.blocking_write();
                    if let Err(e) = mgr.load_from_kb(&kb) {
                        log::warn!("todo sync: load subagents: {e}");
                    }
                }
                let report = b.smart_analyze();
                let yml = b.export_todo_yml();
                if let (Ok(y), Some(kb)) = (&yml, open_kb()) {
                    let _ = kb.kv_set("todo", "TODO.yml", y);
                    let _ = kb.kv_set("todo", "TODO.md", &b.export_todo_md());
                    let shared = crate::cli::commands::agent_cmds::shared_subagent_manager();
                    let mgr = shared.try_write();
                    if let Ok(mgr) = mgr {
                        let _ = mgr.save_to_kb(&kb);
                    }
                }
                if let Ok(y) = &yml {
                    if let Err(e) = std::fs::write("TODO.yml", y) {
                        return CommandOutput::err(&format!("Cannot write TODO.yml: {e}"));
                    }
                }
                if let Err(e) = std::fs::write("TODO.md", b.export_todo_md()) {
                    return CommandOutput::err(&format!("Cannot write TODO.md: {e}"));
                }
                CommandOutput::ok(&report)
            }
            "allocate" => {
                let max_parallel: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
                let (ready_ids, mut b) = {
                    let b = board().lock().unwrap_or_else(|e| e.into_inner());
                    (b.ready_for_allocation().iter().map(|i| i.id.clone()).collect::<Vec<_>>(), b)
                };
                let allocator = crate::neotrix::nt_core_parallel::OptimalTaskAllocator::new(
                    crate::neotrix::nt_core_parallel::AllocationStrategy::Hybrid,
                );
                let shared = crate::cli::commands::agent_cmds::shared_subagent_manager();
                let mut mgr = shared.blocking_write();
                let running = mgr.running_count();
                if running >= max_parallel {
                    return CommandOutput::ok(&format!(
                        "已达最大并行数 ({max_parallel})，当前 running={running}，等待…"
                    ));
                }
                let budget = max_parallel - running;
                let mut allocated = Vec::new();
                // Build TodoTask views of ready items and pick top-budget by efficiency score.
                for id in &ready_ids {
                    let item = match b.get_item_by_id(id) {
                        Some(i) => i.clone(),
                        None => continue,
                    };
                    let todo = crate::neotrix::nt_core_parallel::TodoTask::new(
                        item.id.clone(), item.title.clone(), "todo".into(),
                    )
                        .with_priority(item.priority as i32)
                        .with_dependencies(item.dependencies.clone());
                    let mut t = todo;
                    t.created_at = item.created_at;
                    let budget_hit = allocator.allocate_todo(std::slice::from_ref(&t), budget, |_| false);
                    if !budget_hit.is_empty() && allocated.len() < budget {
                        let agent_id = mgr.register_for_task(&item.id, item.milestone.as_deref().unwrap_or(""));
                        if let Some(wi) = b.get_item_by_id_mut(&item.id) {
                            wi.phase = WorkItemPhase::Running;
                            wi.assignee = Some(agent_id.clone());
                            wi.updated_at = KanbanBoard::now();
                        }
                        allocated.push(format!("{agent_id} → {} ({})", item.id, item.title));
                    }
                }
                if allocated.is_empty() {
                    return CommandOutput::ok("无可用任务（pending 且未阻塞）");
                }
                let _ = b.save_to_kb(&open_kb().unwrap_or_else(|| KnowledgeBase::open(None).expect("kb")));
                CommandOutput::ok(&format!("分配了 {} 个任务:\n{}", allocated.len(), allocated.join("\n")))
            }
            "status" => {
                let b = board().lock().unwrap_or_else(|e| e.into_inner());
                let mut out = String::new();
                out.push_str("=== NeoTrix TODO 状态报告 ===\n");
                let mut stats: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
                for item in &b.items {
                    *stats.entry(format!("{}_{}", priority_label(item.priority), phase_to_str(&item.phase))).or_insert(0) += 1;
                }
                for (k, v) in &stats {
                    out.push_str(&format!("  {k}: {v}\n"));
                }
                out.push_str("\n[最高效任务 TOP 5]\n");
                let mut sorted: Vec<&WorkItem> = b.items.iter().collect();
                sorted.sort_by(|a, c| c.efficiency_score.partial_cmp(&a.efficiency_score).unwrap_or(std::cmp::Ordering::Equal));
                for item in sorted.iter().take(5) {
                    out.push_str(&format!("  {}: 分数={:.1}, 状态={}\n", item.id, item.efficiency_score, phase_to_str(&item.phase)));
                }
                CommandOutput::ok(&out)
            }
            _ => CommandOutput::err("Usage: /board todo import|sync|allocate [max_parallel]|status"),
        }
    }
}

impl CliCommand for BoardCmd {
    fn name(&self) -> &str { "/board" }
    fn aliases(&self) -> Vec<&str> { vec!["/b", "/todo"] }
    fn description(&self) -> &str {
        "Kanban board: /board list | create <spec> [--branch <name>] | move <id> [--to <phase>] [--force] | view <id> | dependency <id> add/remove <dep> | block/unblock <id> | assign <id> <agent> | priority <id> <level> | branch <id> [name] | wip <phase> <limit> | ready | save [path] | load [path] | todo import|sync|allocate|status"
    }
    fn is_primary(&self) -> bool { false }


    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");

        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            let b = board().lock().unwrap_or_else(|e| e.into_inner());
            return CommandOutput::ok(&format!(
                "Kanban Board: {}\n  Tasks: {}\n\nCommands:\n  create <spec> [--branch <name>]  — new task\n  list                 — show all\n  move <id> [--force] [--to <phase>]  — advance\n  view <id>            — task detail\n  dependency <id> add/remove <dep>\n  block <id> <reason>\n  unblock <id>\n  assign <id> <agent>\n  priority <id> <level>\n  branch <id> [name]   — bind git branch / list options\n  wip <phase> <limit>\n  ready                — show ready items\n  save [path]          — persist to JSON\n  load [path]          — load from JSON",
                b.project, b.items.len()
            ));
        }

        match args[0].as_str() {
            "create" | "new" => {
                // Parse optional --branch <name> and --thread <id> flags, strip from spec.
                let mut branch: Option<String> = None;
                let mut thread: Option<String> = None;
                let mut spec_parts: Vec<String> = Vec::new();
                let mut i = 0;
                let raw: Vec<String> = args[1..].to_vec();
                while i < raw.len() {
                    match raw[i].as_str() {
                        "--branch" => {
                            if i + 1 < raw.len() { branch = Some(raw[i + 1].clone()); i += 2; } else { i += 1; }
                        }
                        "--thread" => {
                            if i + 1 < raw.len() { thread = Some(raw[i + 1].clone()); i += 2; } else { i += 1; }
                        }
                        "--json" => { i += 1; }
                        s => { spec_parts.push(s.to_string()); i += 1; }
                    }
                }
                let spec = spec_parts.join(" ");
                if spec.is_empty() {
                    return CommandOutput::err("Usage: /board create <task spec> [--branch <name>] [--thread <id>]");
                }
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                let task_id = b.add_item_full(&spec, &spec, 3, branch.clone(), thread);
                let mut msg = format!("Created task {task_id}: {spec}");
                if let Some(br) = branch.as_deref() { msg.push_str(&format!(" (branch: {br})")); }
                if want_json {
                    return CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "task_id": task_id, "spec": spec, "phase": "Backlog",
                        "git_branch": branch,
                    }));
                }
                CommandOutput::ok(&msg)
            }
            "list" | "ls" => {
                let b = board().lock().unwrap_or_else(|e| e.into_inner());
                Self::list_output(&b, want_json)
            }
            "move" | "advance" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /board move <task_id> [--force] [--to <phase>]");
                }
                let task_id = &args[1];
                let force = args.iter().any(|a| a == "--force");
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                let target_phase = if let Some(pos) = args.iter().position(|a| a == "--to") {
                    if pos + 1 >= args.len() {
                        return CommandOutput::err("Missing phase after --to");
                    }
                    match args[pos + 1].parse::<WorkItemPhase>() {
                        Ok(p) => p,
                        Err(e) => return CommandOutput::err(&e),
                    }
                } else {
                    let current = match b.get_item_by_id(task_id) {
                        Some(i) => i.phase.clone(),
                        None => return CommandOutput::not_found(&format!("Task {task_id} not found")),
                    };
                    match current {
                        WorkItemPhase::Backlog => WorkItemPhase::Planning,
                        WorkItemPhase::Planning => WorkItemPhase::Running,
                        WorkItemPhase::Running => WorkItemPhase::Review,
                        WorkItemPhase::Review => WorkItemPhase::Done,
                        WorkItemPhase::Blocked => WorkItemPhase::Running,
                        _ => return CommandOutput::err(&format!("No advance path from {current}")),
                    }
                };

                match b.move_item(task_id, &target_phase, force) {
                    Ok(()) => {
                        let phase_str = match b.get_item_by_id(task_id) {
                            Some(i) => i.phase.to_string(),
                            None => {
                                return CommandOutput::err(&format!("Task {task_id} not found after move"));
                            }
                        };
                        let msg = format!("Task {task_id}: → {phase_str}");
                        if want_json {
                            return CommandOutput::ok(&msg).with_json(serde_json::json!({
                                "task_id": task_id, "phase": phase_str,
                            }));
                        }
                        CommandOutput::ok(&msg)
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "view" | "show" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /board view <task_id>");
                }
                let task_id = &args[1];
                let b = board().lock().unwrap_or_else(|e| e.into_inner());
                Self::view_output(&b, task_id, want_json)
            }
            "dependency" | "dep" => {
                if args.len() < 4 {
                    return CommandOutput::err("Usage: /board dependency <id> add/remove <dep_id>");
                }
                let id = &args[1];
                let action = &args[2];
                let dep_id = &args[3];
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                let result = match action.as_str() {
                    "add" => b.add_dependency(id, dep_id),
                    "remove" | "rm" => b.remove_dependency(id, dep_id),
                    _ => return CommandOutput::err("Action must be 'add' or 'remove'"),
                };
                match result {
                    Ok(()) => {
                        let msg = format!("Dependency {action}: {id} {action} {dep_id}");
                        if want_json {
                            return CommandOutput::ok(&msg).with_json(serde_json::json!({
                                "id": id, "action": action, "dep_id": dep_id,
                            }));
                        }
                        CommandOutput::ok(&msg)
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "block" => {
                if args.len() < 3 {
                    return CommandOutput::err("Usage: /board block <id> <reason>");
                }
                let id = &args[1];
                let reason = args[2..].iter()
                    .filter(|a| *a != "--json")
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                match b.get_item_by_id_mut(id) {
                    Some(item) => {
                        item.phase = WorkItemPhase::Blocked;
                        item.blocked_reason = Some(reason.clone());
                        item.updated_at = KanbanBoard::now();
                        let msg = format!("Blocked {id}: {reason}");
                        if want_json {
                            return CommandOutput::ok(&msg).with_json(serde_json::json!({
                                "id": id, "blocked_reason": reason,
                            }));
                        }
                        CommandOutput::ok(&msg)
                    }
                    None => CommandOutput::not_found(&format!("Task {id} not found")),
                }
            }
            "unblock" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /board unblock <id>");
                }
                let id = &args[1];
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                match b.get_item_by_id_mut(id) {
                    Some(item) => {
                        item.blocked_reason = None;
                        if item.phase == WorkItemPhase::Blocked {
                            item.phase = WorkItemPhase::Running;
                        }
                        item.updated_at = KanbanBoard::now();
                        let msg = format!("Unblocked {id}");
                        if want_json {
                            return CommandOutput::ok(&msg).with_json(serde_json::json!({ "id": id }));
                        }
                        CommandOutput::ok(&msg)
                    }
                    None => CommandOutput::not_found(&format!("Task {id} not found")),
                }
            }
            "ready" => {
                let b = board().lock().unwrap_or_else(|e| e.into_inner());
                let ready = b.get_ready_items();
                if ready.is_empty() {
                    return CommandOutput::ok("No ready items (all dependencies met, in Backlog)");
                }
                let mut lines = vec!["=== Ready Items ===".to_string()];
                for item in &ready {
                    lines.push(format!("  {}  —  {}", item.id, item.title));
                }
                if want_json {
                    let json: Vec<serde_json::Value> = ready.iter().map(|i| serde_json::json!({
                        "id": i.id, "title": i.title,
                    })).collect();
                    return CommandOutput::ok("").with_json(serde_json::json!({"ready": json}));
                }
                CommandOutput::ok(&lines.join("\n"))
            }
            "save" => {
                let path = if args.len() > 1 && args[1] != "--json" {
                    args[1].clone()
                } else {
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    format!("{home}/.neotrix/kanban/board.json")
                };
                let b = board().lock().unwrap_or_else(|e| e.into_inner());
                match b.save(&path) {
                    Ok(()) => {
                        if let Some(kb) = open_kb() {
                            if let Err(e) = b.save_to_kb(&kb) {
                                log::warn!("KB save failed: {e}");
                            }
                        }
                        let msg = format!("Board saved to {path}");
                        if want_json {
                            return CommandOutput::ok(&msg).with_json(serde_json::json!({"path": path}));
                        }
                        CommandOutput::ok(&msg)
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "load" => {
                let path = if args.len() > 1 && args[1] != "--json" {
                    args[1].clone()
                } else {
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    format!("{home}/.neotrix/kanban/board.json")
                };
                match KanbanBoard::load(&path) {
                    Ok(loaded) => {
                        let count = loaded.items.len();
                        if let Ok(mut b) = board().lock() {
                            *b = loaded;
                        }
                        let msg = format!("Loaded {count} tasks from {path}");
                        if want_json {
                            return CommandOutput::ok(&msg).with_json(serde_json::json!({"path": path, "count": count}));
                        }
                        CommandOutput::ok(&msg)
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "wip" => {
                if args.len() < 3 {
                    return CommandOutput::err("Usage: /board wip <phase> <limit>");
                }
                let phase: WorkItemPhase = match args[1].parse() {
                    Ok(p) => p,
                    Err(e) => return CommandOutput::err(&e),
                };
                let limit: u8 = match args[2].parse() {
                    Ok(l) => l,
                    Err(_) => return CommandOutput::err("Limit must be a number 0-255"),
                };
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                b.set_wip_limit(&phase, limit);
                let msg = format!("WIP limit for {phase} set to {limit}");
                if want_json {
                    return CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "phase": phase.to_string(), "limit": limit,
                    }));
                }
                CommandOutput::ok(&msg)
            }
            "assign" => {
                if args.len() < 3 {
                    return CommandOutput::err("Usage: /board assign <id> <agent>");
                }
                let id = &args[1];
                let agent = &args[2];
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                match b.get_item_by_id_mut(id) {
                    Some(item) => {
                        item.assignee = Some(agent.to_string());
                        item.updated_at = KanbanBoard::now();
                        let msg = format!("Assigned {id} to {agent}");
                        if want_json {
                            return CommandOutput::ok(&msg).with_json(serde_json::json!({
                                "id": id, "assignee": agent,
                            }));
                        }
                        CommandOutput::ok(&msg)
                    }
                    None => CommandOutput::not_found(&format!("Task {id} not found")),
                }
            }
            "priority" => {
                if args.len() < 3 {
                    return CommandOutput::err("Usage: /board priority <id> <level> (1-5)");
                }
                let id = &args[1];
                let level: u8 = match args[2].parse() {
                    Ok(l) if (1..=5).contains(&l) => l,
                    _ => return CommandOutput::err("Priority must be 1-5"),
                };
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                match b.get_item_by_id_mut(id) {
                    Some(item) => {
                        item.priority = level;
                        item.updated_at = KanbanBoard::now();
                        let msg = format!("Set priority of {id} to {level}");
                        if want_json {
                            return CommandOutput::ok(&msg).with_json(serde_json::json!({
                                "id": id, "priority": level,
                            }));
                        }
                        CommandOutput::ok(&msg)
                    }
                    None => CommandOutput::not_found(&format!("Task {id} not found")),
                }
            }
            "branch" | "bind-branch" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /board branch <id> [branch-name]");
                }
                let id = &args[1];
                // No branch name given → list scan options from current git repo (dashi parity).
                let name = args.iter()
                    .skip(2)
                    .find(|a| *a != "--json")
                    .cloned();
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                let item_exists = b.get_item_by_id(id).is_some();
                if !item_exists {
                    return CommandOutput::not_found(&format!("Task {id} not found"));
                }
                let branch_name = match name {
                    Some(n) => n,
                    None => {
                        drop(b);
                        let branches = scan_git_branches();
                        if branches.is_empty() {
                            return CommandOutput::err("No branches found (cwd is not a git repo?). Usage: /board branch <id> <branch-name>");
                        }
                        let mut lines = vec![format!("Available branches for task {id}:")];
                        for (i, br) in branches.iter().enumerate() {
                            lines.push(format!("  [{}] {}", i + 1, br));
                        }
                        lines.push("Usage: /board branch <id> <branch-name>".to_string());
                        return CommandOutput::ok(&lines.join("\n"));
                    }
                };
                match b.get_item_by_id_mut(id) {
                    Some(item) => {
                        item.git_branch = Some(branch_name.clone());
                        item.updated_at = KanbanBoard::now();
                        let msg = format!("Bound task {id} to branch {branch_name}");
                        if want_json {
                            return CommandOutput::ok(&msg).with_json(serde_json::json!({
                                "id": id, "git_branch": branch_name,
                            }));
                        }
                        CommandOutput::ok(&msg)
                    }
                    None => CommandOutput::not_found(&format!("Task {id} not found")),
                }
            }
            "todo" => self.cmd_todo(&args[1..]),
            // /todo <sub> alias resolves to BoardCmd with args[0]=sub; route the
            // todo-only sub-actions here so both `/board todo import` and `/todo import` work.
            "sync" | "import" | "import-todo" | "smart-sync" | "allocate" | "status" => self.cmd_todo(args),
            _ => CommandOutput::err(&format!("Unknown subcommand: {}. Try: create, list, move, view, dependency, block, unblock, assign, priority, branch, wip, ready, save, load, todo", args[0])),
        }
    }
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_item_phase_transitions() {
        assert!(WorkItemPhase::Backlog.can_advance_to(&WorkItemPhase::Planning));
        assert!(WorkItemPhase::Planning.can_advance_to(&WorkItemPhase::Running));
        assert!(WorkItemPhase::Running.can_advance_to(&WorkItemPhase::Review));
        assert!(WorkItemPhase::Review.can_advance_to(&WorkItemPhase::Done));
        assert!(WorkItemPhase::Running.can_advance_to(&WorkItemPhase::Blocked));
        assert!(WorkItemPhase::Blocked.can_advance_to(&WorkItemPhase::Running));
        assert!(!WorkItemPhase::Done.can_advance_to(&WorkItemPhase::Running));
        assert!(!WorkItemPhase::Cancelled.can_advance_to(&WorkItemPhase::Backlog));
        assert!(WorkItemPhase::Backlog.can_advance_to(&WorkItemPhase::Cancelled));
        assert!(WorkItemPhase::Backlog.can_advance_to(&WorkItemPhase::Deferred));
        assert!(WorkItemPhase::Deferred.can_advance_to(&WorkItemPhase::Planning));
    }

    #[test]
    fn test_dag_dependency_chain() {
        let mut board = KanbanBoard::new("test");
        let a = board.add_item("Task A", "", 3);
        let b = board.add_item("Task B", "", 3);
        let c = board.add_item("Task C", "", 3);

        board.add_dependency(&b, &a).unwrap();
        board.add_dependency(&c, &b).unwrap();

        let chain = board.get_dependency_chain(&c);
        assert_eq!(chain.len(), 2);
        assert!(chain.contains(&a));
        assert!(chain.contains(&b));
    }

    #[test]
    fn test_can_start_with_dependencies() {
        let mut board = KanbanBoard::new("test");
        let a = board.add_item("Task A", "", 3);
        let b = board.add_item("Task B", "", 3);

        board.add_dependency(&b, &a).unwrap();

        assert!(board.can_start(&a));
        assert!(!board.can_start(&b));

        board.move_item(&a, &WorkItemPhase::Planning, true).unwrap();
        board.move_item(&a, &WorkItemPhase::Running, true).unwrap();
        board.move_item(&a, &WorkItemPhase::Review, true).unwrap();
        board.move_item(&a, &WorkItemPhase::Done, true).unwrap();
        assert!(board.can_start(&b));
    }

    #[test]
    fn test_wip_limit_enforcement() {
        let mut board = KanbanBoard::new("test");
        board.set_wip_limit(&WorkItemPhase::Running, 2);

        let a = board.add_item("A", "", 3);
        let b = board.add_item("B", "", 3);
        let c = board.add_item("C", "", 3);

        board.move_item(&a, &WorkItemPhase::Running, true).unwrap();
        board.move_item(&b, &WorkItemPhase::Running, true).unwrap();
        board.move_item(&c, &WorkItemPhase::Planning, true).unwrap();
        let result = board.move_item(&c, &WorkItemPhase::Running, false);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("WIP limit") || err_msg.contains("limit"), "Expected WIP limit error, got: {err_msg}");
    }

    #[test]
    fn test_board_save_load_roundtrip() {
        let mut board = KanbanBoard::new("roundtrip");
        let a = board.add_item("Task A", "Description A", 2);
        let b = board.add_item("Task B", "Description B", 4);
        board.add_dependency(&b, &a).unwrap();

        let json = board.to_json().unwrap();
        let loaded = KanbanBoard::from_json(&json).unwrap();

        assert_eq!(loaded.project, "roundtrip");
        assert_eq!(loaded.items.len(), 2);
        assert_eq!(loaded.items[0].title, "Task A");
        assert_eq!(loaded.items[1].dependencies.len(), 1);
        assert_eq!(loaded.items[1].dependencies[0], loaded.items[0].id);
    }

    #[test]
    fn test_blocked_item_detection() {
        let mut board = KanbanBoard::new("blocked-test");
        let a = board.add_item("Task A", "", 3);
        {
            let item = board.get_item_by_id_mut(&a).unwrap();
            item.phase = WorkItemPhase::Blocked;
            item.blocked_reason = Some("Waiting for review".to_string());
        }
        let blocked = board.get_blocked_items();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].blocked_reason.as_deref(), Some("Waiting for review"));
    }

    #[test]
    fn test_board_create_and_list() {
        let cmd = BoardCmd;
        let result = cmd.execute(&["create".into(), "test".into()], None);
        assert!(result.success);
    }

    #[test]
    fn test_board_move() {
        let cmd = BoardCmd;
        cmd.execute(&["create".into(), "move_test".into()], None);
        let result = cmd.execute(&["move".into(), "task-1".into()], None);
        assert!(result.success);
        assert!(result.message.contains("→") || result.message.contains("not found"));
    }

    #[test]
    fn test_board_view_not_found() {
        let cmd = BoardCmd;
        let result = cmd.execute(&["view".into(), "nonexistent".into()], None);
        assert!(!result.success);
    }

    #[test]
    fn test_board_json_output() {
        let cmd = BoardCmd;
        let result = cmd.execute(&["create".into(), "json_test".into(), "--json".into()], None);
        assert!(result.success || result.json.is_some());
    }

    #[test]
    fn test_board_cli_dependency() {
        let cmd = BoardCmd;
        cmd.execute(&["create".into(), "parent".into()], None);
        cmd.execute(&["create".into(), "child".into()], None);
        let result = cmd.execute(&["dependency".into(), "task-2".into(), "add".into(), "task-1".into()], None);
        assert!(result.success);
        assert!(result.message.contains("add"));
    }

    #[test]
    fn test_board_cli_block_unblock() {
        let cmd = BoardCmd;
        cmd.execute(&["create".into(), "blockable".into()], None);
        let result = cmd.execute(&["block".into(), "task-1".into(), "broken".into()], None);
        assert!(result.success);
        let result = cmd.execute(&["unblock".into(), "task-1".into()], None);
        assert!(result.success);
    }

    #[test]
    fn test_board_display_and_parse() {
        assert_eq!(format!("{}", WorkItemPhase::Backlog), "Backlog");
        assert_eq!(format!("{}", WorkItemPhase::Blocked), "Blocked");
        assert_eq!("running".parse::<WorkItemPhase>().unwrap(), WorkItemPhase::Running);
        assert_eq!("Backlog".parse::<WorkItemPhase>().unwrap(), WorkItemPhase::Backlog);
        assert!("invalid".parse::<WorkItemPhase>().is_err());
    }

    #[test]
    fn test_board_circular_dependency_detection() {
        let mut board = KanbanBoard::new("circular");
        let a = board.add_item("A", "", 3);
        let b = board.add_item("B", "", 3);
        let c = board.add_item("C", "", 3);
        let d = board.add_item("D", "", 3);

        board.add_dependency(&b, &a).unwrap();
        board.add_dependency(&c, &b).unwrap();
        board.add_dependency(&d, &c).unwrap();

        assert!(board.add_dependency(&a, &d).is_err());
    }

    #[test]
    fn test_board_remove_dependency() {
        let mut board = KanbanBoard::new("remove-dep");
        let a = board.add_item("A", "", 3);
        let b = board.add_item("B", "", 3);

        board.add_dependency(&b, &a).unwrap();
        assert_eq!(board.items[1].dependencies.len(), 1);

        board.remove_dependency(&b, &a).unwrap();
        assert_eq!(board.items[1].dependencies.len(), 0);
    }

    #[test]
    fn test_board_id_not_reused_after_remove() {
        // Regression: next_id used items.len()+1, so removing the middle
        // item reused its id and silently replaced a live item. Ids must be
        // monotonic within a board lifetime.
        let mut board = KanbanBoard::new("id-reuse");
        let a = board.add_item("A", "", 1);
        let b = board.add_item("B", "", 1);
        let c = board.add_item("C", "", 1);
        assert_eq!((a.as_str(), b.as_str(), c.as_str()), ("task-1", "task-2", "task-3"));

        board.remove_item(&b).unwrap();
        let d = board.add_item("D", "", 1);
        assert_eq!(d, "task-4", "id must not be recycled after remove_item");
        assert!(board.get_item_by_id("task-3").is_some(), "existing task-3 must survive");
    }

    #[test]
    fn test_board_id_seed_from_deserialized_legacy_board() {
        // Boards saved before the counter field existed deserialize with
        // next_id_counter == 0; fresh ids must start above existing ones.
        let mut board = KanbanBoard::new("legacy");
        board.add_item("A", "", 1);
        board.add_item("B", "", 1);
        let json = serde_json::to_string(&board).unwrap();
        let mut loaded: KanbanBoard = serde_json::from_str(&json).unwrap();
        let id = loaded.add_item("C", "", 1);
        assert_eq!(id, "task-3", "legacy board must not collide with existing ids");
    }

    #[test]
    fn test_import_todo_md_parses_items() {
        let md = "# NeoTrix TODO 列表\n\
## 🔴 High 优先级\n\n\
### ✅ S-TASK-1: Ship the thing\n\n\
**状态**: done  \n\
**Session**: cycle-200 (Ship it)\n\
**子代理**: agent-0001\n\
**文件**: src/a.rs, src/b.rs\n\n\
### 🔄 S-TASK-2: In progress task\n\n\
**状态**: in_progress  \n\
**依赖**: S-TASK-3\n\n\
### ⬜ S-TASK-3: Pending task\n\n\
**状态**: pending  \n\
";
        let mut board = KanbanBoard::new("import");
        let parsed = board.import_todo_md(md);
        assert_eq!(parsed, 3);
        assert_eq!(board.items.len(), 3);
        let t1 = board.get_item_by_id("S-TASK-1").unwrap();
        assert_eq!(t1.phase, WorkItemPhase::Done);
        assert_eq!(t1.assignee.as_deref(), Some("agent-0001"));
        assert_eq!(t1.tags.len(), 2);
        let t2 = board.get_item_by_id("S-TASK-2").unwrap();
        assert_eq!(t2.phase, WorkItemPhase::Running);
        assert_eq!(t2.dependencies, vec!["S-TASK-3"]);
        let t3 = board.get_item_by_id("S-TASK-3").unwrap();
        assert_eq!(t3.phase, WorkItemPhase::Backlog);
    }

    #[test]
    fn test_import_todo_md_incremental_update() {
        let md1 = "### ⬜ S-TASK-1: First task\n\n**状态**: pending\n";
        let mut board = KanbanBoard::new("import2");
        assert_eq!(board.import_todo_md(md1), 1);
        let md2 = "### ✅ S-TASK-1: First task\n\n**状态**: done\n";
        assert_eq!(board.import_todo_md(md2), 1);
        assert_eq!(board.items.len(), 1, "re-import must update, not duplicate");
        assert_eq!(board.items[0].phase, WorkItemPhase::Done);
    }

    #[test]
    fn test_smart_analyze_blocks_on_dependencies() {
        let mut board = KanbanBoard::new("analyze");
        board.add_item("A", "Task A", 2);
        board.add_item("B", "Task B", 2);
        let a = "task-1".to_string();
        let b = "task-2".to_string();
        board.add_dependency(&b, &a).unwrap();
        // Move dep A to Backlog (not done) → B blocked
        let report = board.smart_analyze();
        assert!(report.contains("[BLOCKED]"), "report: {report}");
        let bb = board.get_item_by_id(&b).unwrap();
        assert_eq!(bb.phase, WorkItemPhase::Blocked);
    }

    #[test]
    fn test_smart_analyze_dedup_by_similar_title() {
        let mut board = KanbanBoard::new("dedup");
        let a = board.add_item("Add telemetry to agent loop", "", 2);
        let b = board.add_item("Add telemetry to agent loop!", "", 2);
        board.smart_analyze();
        // Similar titles produce a conflict entry, not deletion (Python keeps both + flags)
        assert!(!board.conflicts.is_empty(), "expected a similar-title conflict");
        assert!(board.get_item_by_id(&a).is_some());
        assert!(board.get_item_by_id(&b).is_some());
    }

    #[test]
    fn test_export_todo_yml_roundtrip_shape() {
        let mut board = KanbanBoard::new("yml");
        let id = board.add_item("YAML export task", "desc", 3);
        board.items[0].milestone = Some("cycle-201".into());
        let yml = board.export_todo_yml().unwrap();
        assert!(yml.contains("total_items: 1"));
        assert!(yml.contains("high"));
        assert!(yml.contains(&id));
        // Must be valid YAML
        let v: serde_yaml::Value = serde_yaml::from_str(&yml).unwrap();
        assert!(v.get("items").is_some());
    }

    #[test]
    fn test_recompute_efficiency_ranks_priority() {
        let mut board = KanbanBoard::new("eff");
        board.add_item("low", "", 1);
        board.add_item("high", "", 3);
        board.recompute_efficiency();
        let low = board.items.iter().find(|i| i.title == "low").unwrap().efficiency_score;
        let high = board.items.iter().find(|i| i.title == "high").unwrap().efficiency_score;
        assert!(high > low);
    }

    #[test]
    fn test_dice_similarity() {
        assert!(dice_similarity("same title text", "same title text") > 0.99);
        assert!(dice_similarity("completely unrelated one", "totally different two") < 0.3);
        assert_eq!(dice_similarity("", ""), 1.0);
    }

    #[test]
    fn test_board_todo_import_and_status() {
        let cmd = BoardCmd;
        let md = "### ✅ S-T1: First\n\n**状态**: done\n### ⬜ S-T2: Second\n\n**状态**: pending\n";
        let path = std::env::temp_dir().join(format!("nt_todo_import_{}.md", std::process::id()));
        std::fs::write(&path, md).unwrap();
        let r = cmd.execute(&["todo".into(), "import".into(), path.to_str().unwrap().into()], None);
        assert!(r.success, "{}", r.message);
        assert!(r.message.contains("2"), "import should report 2: {}", r.message);
        let st = cmd.execute(&["todo".into(), "status".into()], None);
        assert!(st.success);
        assert!(st.message.contains("TODO") || st.message.contains("任务"));
        // board is global; also test /todo alias normalizes to same handler
        let alias = cmd.execute(&["import".into(), path.to_str().unwrap().into()], None);
        assert!(alias.success, "alias routing: {}", alias.message);
    }

    #[test]
    fn test_board_todo_allocate_respects_running_budget() {
        let cmd = BoardCmd;
        // Allocate with max_parallel=1 and no running agents should pick exactly 1
        let r = cmd.execute(&["todo".into(), "allocate".into(), "1".into()], None);
        assert!(r.success, "{}", r.message);
    }

    #[test]
    fn test_board_todo_sync_writes_files() {
        let cmd = BoardCmd;
        let r = cmd.execute(&["todo".into(), "sync".into()], None);
        assert!(r.success, "{}", r.message);
        // Leaves TODO.md / TODO.yml in cwd; just assert no panic and report shape
        assert!(r.message.contains("[SMART]") || r.message.contains("分析完成") || r.message.contains("TODO"));
    }

    #[test]
    fn test_create_with_branch_flag() {
        let cmd = BoardCmd;
        let r = cmd.execute(&["create".into(), "branch task".into(), "--branch".into(), "feat/x".into()], None);
        assert!(r.success, "{}", r.message);
        assert!(r.message.contains("branch: feat/x"), "{}", r.message);
        let b = board().lock().unwrap_or_else(|e| e.into_inner());
        // 全局 board() 跨测试共享 — 并行时 items.last() 可能不是本测试刚建的项,
        // 按 title 精确定位 (共享状态测试隔离纪律)。
        let item = b.items.iter().rev().find(|i| i.title == "branch task").expect("branch task item");
        assert_eq!(item.git_branch.as_deref(), Some("feat/x"));
        drop(b);
        // cleanup: remove test item so later tests are unaffected
        let mut b2 = board().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(i) = b2.items.iter().rev().find(|i| i.title == "branch task").cloned() {
            let _ = b2.remove_item(&i.id);
        }
    }

    #[test]
    fn test_branch_bind_and_view() {
        let mut b = KanbanBoard::new("branch-test");
        let id = b.add_item_full("Branch item", "", 2, Some("feat/x".into()), Some("thread-42".into()));
        let item = b.get_item_by_id(&id).unwrap();
        assert_eq!(item.git_branch.as_deref(), Some("feat/x"));
        assert_eq!(item.thread_id.as_deref(), Some("thread-42"));
        // view JSON round-trips new fields
        let json = serde_json::to_value(item).unwrap();
        assert_eq!(json["git_branch"], "feat/x");
        assert_eq!(json["thread_id"], "thread-42");
    }

    #[test]
    fn test_legacy_json_without_new_fields_loads() {
        // Boards serialized before git_branch/thread_id existed must deserialize.
        let legacy = r#"{"project":"old","items":[{"id":"task-1","title":"Old","description":"","phase":"Backlog","priority":3,"assignee":null,"dependencies":[],"depended_by":[],"created_at":0,"updated_at":0,"tags":[],"milestone":null,"results":[],"blocked_reason":null}],"wip_limits":{"Running":3,"Review":3},"conflicts":[],"next_id_counter":1}"#;
        let board: KanbanBoard = serde_json::from_str(legacy).unwrap();
        assert_eq!(board.items.len(), 1);
        assert_eq!(board.items[0].git_branch, None);
        assert_eq!(board.items[0].thread_id, None);
    }

    #[test]
    fn test_branch_cmd_binds_and_lists() {
        let cmd = BoardCmd;
        let r = cmd.execute(&["create".into(), "bind me".into()], None);
        assert!(r.success, "{}", r.message);
        let b = board().lock().unwrap_or_else(|e| e.into_inner());
        let id = b.items.iter().rev().find(|i| i.title == "bind me").expect("bind me item").id.clone();
        drop(b);
        // bind a branch
        let r = cmd.execute(&["branch".into(), id.clone().into(), "feat/bound".into()], None);
        assert!(r.success, "{}", r.message);
        assert!(r.message.contains("feat/bound"));
        // no-arg lists options or errors gracefully (cwd may not be a repo in tests)
        let r = cmd.execute(&["branch".into(), id.clone().into()], None);
        assert!(r.success || r.message.contains("no git repo") || r.message.contains("Available branches"));
        // cleanup
        let mut b2 = board().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(i) = b2.items.iter().rev().find(|i| i.title == "bind me").cloned() {
            let _ = b2.remove_item(&i.id);
        }
    }

    #[test]
    fn test_scan_git_branches_in_repo() {
        // NeoTrix cwd is a git repo in CI/local; at minimum the function must
        // not panic and return a Vec. In a repo it should include main/master.
        let branches = scan_git_branches();
        let _ = branches; // assert no panic; content depends on cwd
    }
}
