use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSpace {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub project_root: Option<PathBuf>,
    pub description: String,
    pub tags: Vec<String>,
    pub memory_count: u32,
    pub goal_count: u32,
    pub skill_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkSpaceManager {
    pub workspaces: Vec<WorkSpace>,
    pub active_id: Option<String>,
}

impl WorkSpaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: Vec::new(),
            active_id: None,
        }
    }

    pub fn create(
        &mut self,
        name: &str,
        project_root: Option<PathBuf>,
        description: &str,
    ) -> WorkSpace {
        let id = format!("ws-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let now = chrono::Utc::now();
        let ws = WorkSpace {
            id: id.clone(),
            name: name.to_string(),
            created_at: now,
            last_active: now,
            project_root,
            description: description.to_string(),
            tags: Vec::new(),
            memory_count: 0,
            goal_count: 0,
            skill_count: 0,
        };
        self.active_id = Some(id);
        self.workspaces.push(ws.clone());
        ws
    }

    pub fn list(&self) -> &[WorkSpace] {
        &self.workspaces
    }

    pub fn switch(&mut self, id: &str) -> Result<(), String> {
        if self.workspaces.iter().any(|w| w.id == id) {
            self.active_id = Some(id.to_string());
            if let Some(ws) = self.workspaces.iter_mut().find(|w| w.id == id) {
                ws.last_active = chrono::Utc::now();
            }
            Ok(())
        } else {
            Err(format!("WorkSpace not found: {}", id))
        }
    }

    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let pos = self
            .workspaces
            .iter()
            .position(|w| w.id == id)
            .ok_or_else(|| format!("WorkSpace not found: {}", id))?;
        self.workspaces.remove(pos);
        if self.active_id.as_deref() == Some(id) {
            self.active_id = self.workspaces.first().map(|w| w.id.clone());
        }
        Ok(())
    }

    pub fn active(&self) -> Option<&WorkSpace> {
        self.active_id
            .as_ref()
            .and_then(|id| self.workspaces.iter().find(|w| w.id == *id))
    }

    pub fn get(&self, id: &str) -> Option<&WorkSpace> {
        self.workspaces.iter().find(|w| w.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut WorkSpace> {
        self.workspaces.iter_mut().find(|w| w.id == id)
    }

    pub fn rename(&mut self, id: &str, new_name: &str) -> Result<(), String> {
        let ws = self
            .get_mut(id)
            .ok_or_else(|| format!("WorkSpace not found: {}", id))?;
        ws.name = new_name.to_string();
        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        crate::l5_cognition::nt_core_state::save("workspaces", &self.to_json()?)
    }

    /// Phase 2 KB 直写: 可注入连接变体 (测试用内存连接)。
    pub fn save_with(&self, conn: &rusqlite::Connection) -> Result<(), String> {
        crate::l5_cognition::nt_core_state::save_with(conn, "workspaces", &self.to_json()?)
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))
    }

    pub fn load() -> Self {
        crate::l5_cognition::nt_core_state::load("workspaces")
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Phase 2 KB 直写: 可注入连接变体 (测试用内存连接)。
    pub fn load_with(conn: &rusqlite::Connection) -> Self {
        crate::l5_cognition::nt_core_state::load_with(conn, "workspaces")
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn scope_root(&self, id: &str) -> Option<PathBuf> {
        self.get(id).and_then(|ws| ws.project_root.clone())
    }
}

impl Default for WorkSpaceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global workspace manager fallback — prefer `CliContext.workspace` instead.
pub static WORKSPACE_MANAGER: LazyLock<Mutex<WorkSpaceManager>> =
    LazyLock::new(|| Mutex::new(WorkSpaceManager::load()));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_manager_new_is_empty() {
        let mgr = WorkSpaceManager::new();
        assert!(mgr.list().is_empty());
        assert!(mgr.active().is_none());
    }

    #[test]
    fn workspace_manager_create() {
        let mut mgr = WorkSpaceManager::new();
        let ws = mgr.create("test", None, "a test workspace");
        assert_eq!(ws.name, "test");
        assert_eq!(ws.description, "a test workspace");
        assert!(mgr.active().is_some());
        assert_eq!(mgr.list().len(), 1);
    }

    #[test]
    fn workspace_manager_switch() {
        let mut mgr = WorkSpaceManager::new();
        let ws1 = mgr.create("ws1", None, "first");
        let _ws2 = mgr.create("ws2", None, "second");
        assert_eq!(mgr.active().unwrap().name, "ws2");
        mgr.switch(&ws1.id).unwrap();
        assert_eq!(mgr.active().unwrap().name, "ws1");
    }

    #[test]
    fn workspace_manager_switch_nonexistent() {
        let mut mgr = WorkSpaceManager::new();
        assert!(mgr.switch("nope").is_err());
    }

    #[test]
    fn workspace_manager_delete() {
        let mut mgr = WorkSpaceManager::new();
        let ws = mgr.create("ws1", None, "desc");
        mgr.delete(&ws.id).unwrap();
        assert!(mgr.list().is_empty());
        assert!(mgr.active().is_none());
    }

    #[test]
    fn workspace_manager_delete_nonexistent() {
        let mut mgr = WorkSpaceManager::new();
        assert!(mgr.delete("nope").is_err());
    }

    #[test]
    fn workspace_manager_get() {
        let mut mgr = WorkSpaceManager::new();
        let ws = mgr.create("ws1", None, "desc");
        assert!(mgr.get(&ws.id).is_some());
        assert!(mgr.get("nope").is_none());
    }

    #[test]
    fn workspace_manager_rename() {
        let mut mgr = WorkSpaceManager::new();
        let ws = mgr.create("old", None, "desc");
        mgr.rename(&ws.id, "new").unwrap();
        assert_eq!(mgr.get(&ws.id).unwrap().name, "new");
    }

    #[test]
    fn workspace_manager_rename_nonexistent() {
        let mut mgr = WorkSpaceManager::new();
        assert!(mgr.rename("nope", "new").is_err());
    }

    #[test]
    fn workspace_manager_scope_root() {
        let mut mgr = WorkSpaceManager::new();
        assert!(mgr.scope_root("nope").is_none());
        let ws = mgr.create("ws1", Some(PathBuf::from("/tmp")), "desc");
        assert_eq!(mgr.scope_root(&ws.id).unwrap(), PathBuf::from("/tmp"));
    }

    #[test]
    fn workspace_manager_default() {
        let mgr = WorkSpaceManager::default();
        assert!(mgr.list().is_empty());
    }

    #[test]
    fn workspace_serde_roundtrip() {
        let mut mgr = WorkSpaceManager::new();
        mgr.create("ws", Some(PathBuf::from("/p")), "desc");
        let json = mgr.to_json().unwrap();
        let back: WorkSpaceManager = serde_json::from_str(&json).unwrap();
        assert_eq!(back.list().len(), 1);
        assert_eq!(back.list()[0].name, "ws");
    }
}
