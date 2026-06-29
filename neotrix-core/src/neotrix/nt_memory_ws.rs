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
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        let path = home.join(".neotrix").join("workspaces.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
        }
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, json).map_err(|e| format!("Write error: {}", e))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("Rename error: {}", e))?;
        Ok(())
    }

    pub fn load() -> Self {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return Self::new(),
        };
        let path = home.join(".neotrix").join("workspaces.json");
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
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

pub static WORKSPACE_MANAGER: LazyLock<Mutex<WorkSpaceManager>> =
    LazyLock::new(|| Mutex::new(WorkSpaceManager::load()));

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[test]
    fn test_workspace_create_and_get() {
        let mut mgr = WorkSpaceManager::new();
        let ws = mgr.create("test", None, "test workspace");
        assert_eq!(ws.name, "test");
        assert_eq!(ws.description, "test workspace");
        assert!(mgr.active().is_some());
        let got = mgr.get(&ws.id);
        assert!(got.is_some());
    }

    #[test]
    fn test_workspace_delete_and_switch() {
        let mut mgr = WorkSpaceManager::new();
        let ws1 = mgr.create("alpha", None, "");
        let ws2 = mgr.create("beta", None, "");
        assert!(mgr.switch(&ws1.id).is_ok());
        assert!(mgr.delete(&ws2.id).is_ok());
        assert!(mgr.get(&ws2.id).is_none());
        assert!(mgr.switch("nonexistent").is_err());
    }
}
