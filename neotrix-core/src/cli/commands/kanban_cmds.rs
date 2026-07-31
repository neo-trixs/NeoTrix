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
}

// ─── KanbanBoard ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanBoard {
    pub project: String,
    pub items: Vec<WorkItem>,
    pub wip_limits: HashMap<WorkItemPhase, u8>,
}

impl KanbanBoard {
    pub fn new(project: &str) -> Self {
        let mut wip_limits = HashMap::new();
        wip_limits.insert(WorkItemPhase::Running, 3);
        wip_limits.insert(WorkItemPhase::Review, 3);
        Self { project: project.to_string(), items: Vec::new(), wip_limits }
    }

    pub fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn next_id(&self) -> String {
        format!("task-{}", self.items.len() + 1)
    }

    pub fn add_item(&mut self, title: &str, description: &str, priority: u8) -> String {
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
                    "Item: {}\n  Title: {}\n  Description: {}\n  Phase: {}\n  Priority: {}\n  Assignee: {}\n  Dependencies: {}\n    deps: {}\n    depended_by: {}\n    chain: {}\n  Can start: {ready}\n  Tags: {}\n  Milestone: {}\n  Blocked reason: {}\n  Created: {}\n  Updated: {}\n  Results: {}",
                    item.id,
                    item.title,
                    item.description,
                    item.phase,
                    item.priority,
                    item.assignee.as_deref().unwrap_or("unassigned"),
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
}

impl CliCommand for BoardCmd {
    fn name(&self) -> &str { "/board" }
    fn aliases(&self) -> Vec<&str> { vec!["/b"] }
    fn description(&self) -> &str {
        "Kanban board: /board list | create <spec> | move <id> [--to <phase>] [--force] | view <id> | dependency <id> add/remove <dep> | block/unblock <id> | assign <id> <agent> | priority <id> <level> | wip <phase> <limit> | ready | save [path] | load [path]"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");

        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            let b = board().lock().unwrap_or_else(|e| e.into_inner());
            return CommandOutput::ok(&format!(
                "Kanban Board: {}\n  Tasks: {}\n\nCommands:\n  create <spec>       — new task\n  list                 — show all\n  move <id> [--force] [--to <phase>]  — advance\n  view <id>            — task detail\n  dependency <id> add/remove <dep>\n  block <id> <reason>\n  unblock <id>\n  assign <id> <agent>\n  priority <id> <level>\n  wip <phase> <limit>\n  ready                — show ready items\n  save [path]          — persist to JSON\n  load [path]          — load from JSON",
                b.project, b.items.len()
            ));
        }

        match args[0].as_str() {
            "create" | "new" => {
                let spec = args[1..].iter()
                    .filter(|a| *a != "--json")
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");
                if spec.is_empty() {
                    return CommandOutput::err("Usage: /board create <task spec>");
                }
                let mut b = board().lock().unwrap_or_else(|e| e.into_inner());
                let task_id = b.add_item(&spec, &spec, 3);
                let msg = format!("Created task {task_id}: {spec}");
                if want_json {
                    return CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "task_id": task_id, "spec": spec, "phase": "Backlog"
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
            _ => CommandOutput::err(&format!("Unknown subcommand: {}. Try: create, list, move, view, dependency, block, unblock, assign, priority, wip, ready, save, load", args[0])),
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
}
