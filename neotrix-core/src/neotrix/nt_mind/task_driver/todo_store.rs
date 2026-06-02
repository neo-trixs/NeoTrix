use std::collections::{HashMap, HashSet};
use std::path::Path;

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
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("读取 TODO.yml 失败: {}", e))?;
        let raw: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| format!("解析 TODO.yml 失败: {}", e))?;

        let meta = parse_meta(&raw["meta"])?;
        let items = parse_items(&raw["items"])?;

        Ok(Self { meta, items, path: path_str })
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

fn parse_meta(v: &serde_yaml::Value) -> Result<TodoMeta, String> {
    Ok(TodoMeta {
        conflicts: v["conflicts"].as_u64().unwrap_or(0),
        generated_at: v["generated_at"].as_str().unwrap_or("").to_string(),
        total_items: v["total_items"].as_u64().unwrap_or(0),
        v1_completed: v["v1_completed"].as_u64().unwrap_or(0),
        session_distilled: v["session_distilled"].as_bool().unwrap_or(false),
        total_tests: v["total_tests"].as_u64().unwrap_or(0),
        compile_default: v["compile_default"].as_str().unwrap_or("").to_string(),
        compile_full: v["compile_full"].as_str().unwrap_or("").to_string(),
        p0_ready_not_started: v["p0_ready_not_started"].as_u64().unwrap_or(0),
    })
}

fn parse_items(v: &serde_yaml::Value) -> Result<Vec<TodoItem>, String> {
    let arr = v.as_sequence().ok_or("items 不是数组")?;
    let mut items = Vec::with_capacity(arr.len());
    for (i, item) in arr.iter().enumerate() {
        let status = match item["status"].as_str() {
            Some("done") => ItemStatus::Done,
            Some("pending") => ItemStatus::Pending,
            Some("in_progress") => ItemStatus::InProgress,
            Some("blocked") => ItemStatus::Blocked,
            Some(other) => return Err(format!("items[{}]: 未知状态 '{}'", i, other)),
            None => return Err(format!("items[{}]: 缺少 status", i)),
        };
        let priority = match item["priority"].as_str() {
            Some("high") => ItemPriority::High,
            Some("medium") => ItemPriority::Medium,
            Some("low") => ItemPriority::Low,
            Some(other) => return Err(format!("items[{}]: 未知优先级 '{}'", i, other)),
            None => ItemPriority::Medium,
        };
        let files = match &item["files"] {
            serde_yaml::Value::Sequence(seq) => {
                seq.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            }
            _ => Vec::new(),
        };
        let depends_on = match &item["depends_on"] {
            serde_yaml::Value::Sequence(seq) => {
                seq.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            }
            _ => Vec::new(),
        };
        items.push(TodoItem {
            id: item["id"].as_str().unwrap_or("").to_string(),
            title: item["title"].as_str().unwrap_or("").to_string(),
            status,
            priority,
            created: item["created"].as_str().unwrap_or("").to_string(),
            updated: item["updated"].as_str().unwrap_or("").to_string(),
            session: item["session"].as_str().map(String::from),
            files,
            depends_on,
            notes: item["notes"].as_str().unwrap_or("").to_string(),
        });
    }
    Ok(items)
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
