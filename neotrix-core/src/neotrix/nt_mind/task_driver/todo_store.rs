use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TodoStore {
    pub meta: TodoMeta,
    pub items: Vec<TodoItem>,
    path: String,
}

#[derive(Debug, Clone)]
pub struct TodoMeta {
    pub conflicts: u64,
    pub generated_at: String,
    pub total_items: u64,
    pub v1_completed: u64,
    pub session_distilled: bool,
    pub total_tests: u64,
    pub compile_default: String,
    pub compile_full: String,
    pub p0_ready_not_started: u64,
}

#[derive(Debug, Clone)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub status: ItemStatus,
    pub priority: ItemPriority,
    pub created: String,
    pub updated: String,
    pub session: Option<String>,
    pub files: Vec<String>,
    pub depends_on: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemPriority {
    High = 0,
    Medium = 1,
    Low = 2,
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub items: Vec<TodoItem>,
    pub by_id: HashMap<String, usize>,
    pub dependents: HashMap<String, Vec<String>>,
}

impl TodoStore {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("读取 TODO.yml 失败: {}", e))?;
        let mut store = TodoStore::new(path.to_string());
        let mut in_items = false;
        let mut current_item: Option<TodoItem> = None;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "items:" {
                in_items = true;
                continue;
            }
            if in_items {
                if trimmed.starts_with("- id:") {
                    if let Some(item) = current_item.take() {
                        store.items.push(item);
                    }
                    current_item = Some(TodoItem {
                        id: trimmed.trim_start_matches("- id:").trim().trim_matches('\'').to_string(),
                        title: String::new(),
                        status: ItemStatus::Pending,
                        priority: ItemPriority::Medium,
                        created: String::new(),
                        updated: String::new(),
                        session: None,
                        files: Vec::new(),
                        depends_on: Vec::new(),
                        notes: String::new(),
                    });
                } else if let Some(ref mut item) = current_item {
                    if let Some(val) = trimmed.strip_prefix("title:") {
                        item.title = val.trim().to_string();
                    } else if let Some(val) = trimmed.strip_prefix("status:") {
                        item.status = match val.trim() {
                            "done" => ItemStatus::Done,
                            "in_progress" => ItemStatus::InProgress,
                            "blocked" => ItemStatus::Blocked,
                            _ => ItemStatus::Pending,
                        };
                    } else if let Some(val) = trimmed.strip_prefix("priority:") {
                        item.priority = match val.trim() {
                            "high" => ItemPriority::High,
                            "low" => ItemPriority::Low,
                            _ => ItemPriority::Medium,
                        };
                    } else if let Some(val) = trimmed.strip_prefix("notes:") {
                        item.notes = val.trim().to_string();
                    }
                }
            } else if let Some(val) = trimmed.strip_prefix("generated_at:") {
                store.meta.generated_at = val.trim().trim_matches('\'').to_string();
            } else if let Some(val) = trimmed.strip_prefix("total_items:") {
                store.meta.total_items = val.trim().parse().unwrap_or(0);
            }
        }
        if let Some(item) = current_item.take() {
            store.items.push(item);
        }
        Ok(store)
    }

    pub fn new(path: String) -> Self {
        Self {
            meta: TodoMeta {
                conflicts: 0,
                generated_at: String::new(),
                total_items: 0,
                v1_completed: 0,
                session_distilled: false,
                total_tests: 0,
                compile_default: String::new(),
                compile_full: String::new(),
                p0_ready_not_started: 0,
            },
            items: Vec::new(),
            path,
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let content = self.to_yaml_string();
        std::fs::write(&self.path, &content)
            .map_err(|e| format!("写入 TODO.yml 失败: {}", e))
    }

    pub fn to_yaml_string(&self) -> String {
        let mut s = String::new();
        s.push_str("conflicts: []\n");
        s.push_str("meta:\n");
        s.push_str(&format!("  conflicts: {}\n", self.meta.conflicts));
        s.push_str(&format!("  generated_at: '{}'\n", self.meta.generated_at));
        s.push_str(&format!("  total_items: {}\n", self.meta.total_items));
        s.push_str("  v2_phase: all_done\n");
        s.push_str(&format!("  v1_completed: {}\n", self.meta.v1_completed));
        s.push_str("  v2_r1_completed: 5\n");
        s.push_str("  v2_r1_total: 5\n");
        s.push_str(&format!("  session_distilled: {}\n", self.meta.session_distilled));
        s.push_str(&format!("  total_tests: {}\n", self.meta.total_tests));
        s.push_str(&format!("  compile_default: '{}'\n", self.meta.compile_default));
        s.push_str(&format!("  compile_full: '{}'\n", self.meta.compile_full));
        s.push_str(&format!("  external_analysed: {}\n", count_done_external(&self.items)));
        s.push_str(&format!("  fusion_docs: {}\n", count_done_fusion(&self.items)));
        s.push_str(&format!("  p0_ready_not_started: {}\n", self.meta.p0_ready_not_started));
        s.push('\n');
        s.push_str("items:\n");
        for item in &self.items {
            s.push_str(&item.to_yaml());
        }
        s
    }

    pub fn pending_items(&self) -> Vec<&TodoItem> {
        self.items.iter().filter(|i| i.status == ItemStatus::Pending).collect()
    }

    pub fn get(&self, id: &str) -> Option<&TodoItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut TodoItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    pub fn update_status(&mut self, id: &str, status: ItemStatus) -> Result<(), String> {
        let item = self.get_mut(id).ok_or_else(|| format!("ID {} 未找到", id))?;
        item.status = status;
        item.updated = chrono::Utc::now().format("%Y-%m-%d").to_string();
        Ok(())
    }

    pub fn build_dependency_graph(&self) -> DependencyGraph {
        let mut by_id = HashMap::new();
        for (idx, item) in self.items.iter().enumerate() {
            by_id.insert(item.id.clone(), idx);
        }
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
        for item in &self.items {
            for dep in &item.depends_on {
                dependents.entry(dep.clone()).or_default().push(item.id.clone());
            }
        }
        DependencyGraph { items: self.items.clone(), by_id, dependents }
    }
}

impl TodoItem {
    pub fn is_ready(&self, done_ids: &HashSet<String>) -> bool {
        if self.status != ItemStatus::Pending && self.status != ItemStatus::InProgress {
            return false;
        }
        self.depends_on.iter().all(|d| done_ids.contains(d))
    }

    pub fn priority_rank(&self) -> u8 {
        match self.priority {
            ItemPriority::High => 0,
            ItemPriority::Medium => 1,
            ItemPriority::Low => 2,
        }
    }

    fn to_yaml(&self) -> String {
        let status_str = match self.status {
            ItemStatus::Pending => "pending",
            ItemStatus::InProgress => "in_progress",
            ItemStatus::Done => "done",
            ItemStatus::Blocked => "blocked",
        };
        let priority_str = match self.priority {
            ItemPriority::High => "high",
            ItemPriority::Medium => "medium",
            ItemPriority::Low => "low",
        };
        let mut s = format!("  - id: {}\n", self.id);
        s.push_str(&format!("    title: {}\n", self.title));
        s.push_str(&format!("    status: {}\n", status_str));
        s.push_str(&format!("    priority: {}\n", priority_str));
        s.push_str(&format!("    created: '{}'\n", self.created));
        s.push_str(&format!("    updated: '{}'\n", self.updated));
        if let Some(ref session) = self.session {
            s.push_str(&format!("    session: {}\n", session));
        } else {
            s.push_str("    session: ''\n");
        }
        if !self.files.is_empty() {
            if self.files.len() == 1 {
                s.push_str(&format!("    files: [{}]\n", self.files[0]));
            } else {
                s.push_str("    files:\n");
                for f in &self.files {
                    s.push_str(&format!("      - {}\n", f));
                }
            }
        } else {
            s.push_str("    files: []\n");
        }
        if self.depends_on.len() == 1 {
            s.push_str(&format!("    depends_on: [{}]\n", self.depends_on[0]));
        } else if self.depends_on.len() > 1 {
            s.push_str("    depends_on:\n");
            for d in &self.depends_on {
                s.push_str(&format!("      - {}\n", d));
            }
        } else {
            s.push_str("    depends_on: []\n");
        }
        s.push_str(&format!("    notes: {}\n", self.notes));
        s
    }
}

impl DependencyGraph {
    pub fn ready_pending(&self, done_ids: &HashSet<String>) -> Vec<&TodoItem> {
        let mut ready: Vec<&TodoItem> = self.items.iter()
            .filter(|i| i.status == ItemStatus::Pending)
            .filter(|i| i.depends_on.iter().all(|d| done_ids.contains(d)))
            .collect();
        ready.sort_by_key(|i| i.priority_rank());
        ready
    }

    pub fn topological_order(&self) -> Vec<&TodoItem> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for item in &self.items {
            in_degree.entry(&item.id).or_insert(0);
            for dep in &item.depends_on {
                adj.entry(dep).or_default().push(&item.id);
                *in_degree.entry(&item.id).or_insert(0) += 1;
            }
        }
        let mut queue: Vec<&str> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| *id)
            .collect();
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        while let Some(id) = queue.pop() {
            if visited.contains(id) { continue; }
            visited.insert(id);
            if let Some(idx) = self.by_id.get(id) {
                result.push(&self.items[*idx]);
            }
            if let Some(neighbors) = adj.get(id) {
                for n in neighbors {
                    if let Some(deg) = in_degree.get_mut(n) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(n);
                        }
                    }
                }
            }
        }
        result
    }
}



fn count_done_external(items: &[TodoItem]) -> u64 {
    items.iter().filter(|i| i.id.starts_with("S19-") && i.status == ItemStatus::Done).count() as u64
}

fn count_done_fusion(items: &[TodoItem]) -> u64 {
    items.iter().filter(|i| i.id.starts_with("S19-") && i.status == ItemStatus::Done).count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_priority_ordering() {
        assert!(ItemPriority::High < ItemPriority::Medium);
        assert!(ItemPriority::Medium < ItemPriority::Low);
    }

    #[test]
    fn test_is_ready_no_deps() {
        let item = TodoItem {
            id: "test-1".into(), title: "Test".into(), status: ItemStatus::Pending,
            priority: ItemPriority::High, created: "2026-01-01".into(),
            updated: "2026-01-01".into(), session: None,
            files: vec![], depends_on: vec![], notes: "".into(),
        };
        let done = HashSet::new();
        assert!(item.is_ready(&done));
    }

    #[test]
    fn test_is_ready_with_deps() {
        let item = TodoItem {
            id: "test-2".into(), title: "Test".into(), status: ItemStatus::Pending,
            priority: ItemPriority::Medium, created: "2026-01-01".into(),
            updated: "2026-01-01".into(), session: None,
            files: vec![], depends_on: vec!["dep-1".into()], notes: "".into(),
        };
        let mut done = HashSet::new();
        assert!(!item.is_ready(&done));
        done.insert("dep-1".into());
        assert!(item.is_ready(&done));
    }

    #[test]
    fn test_dependency_graph_topological() {
        let items = vec![
            TodoItem { id: "a".into(), status: ItemStatus::Done, depends_on: vec![], ..basic_item("a") },
            TodoItem { id: "b".into(), status: ItemStatus::Pending, depends_on: vec!["a".into()], ..basic_item("b") },
            TodoItem { id: "c".into(), status: ItemStatus::Pending, depends_on: vec!["b".into()], ..basic_item("c") },
        ];
        let mut by_id = HashMap::new();
        for (i, item) in items.iter().enumerate() { by_id.insert(item.id.clone(), i); }
        let graph = DependencyGraph { items, by_id, dependents: HashMap::new() };
        let order = graph.topological_order();
        assert!(!order.is_empty());
        let ids: Vec<&str> = order.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"b"));
        assert!(ids.contains(&"c"));
    }

    fn basic_item(id: &str) -> TodoItem {
        TodoItem {
            id: id.into(), title: id.into(), status: ItemStatus::Pending,
            priority: ItemPriority::Medium, created: "2026-01-01".into(),
            updated: "2026-01-01".into(), session: None,
            files: vec![], depends_on: vec![], notes: "".into(),
        }
    }
}
