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
    #[serde(default)]
    pub kind: WorkspaceKind,
    #[serde(default)]
    pub agent_ids: Vec<String>,
    #[serde(default)]
    pub skill_ids: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerBinding>,
    #[serde(default)]
    pub shared_memory_keys: Vec<String>,
    #[serde(default)]
    pub config: WorkspaceEntityConfig,
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub llm_response_language: String,
}

/// Workspace 运行形态：本地 / 远端 / 沙箱执行器。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceKind {
    Local,
    Remote {
        #[serde(default)]
        url: String,
    },
    Sandbox,
}

impl Default for WorkspaceKind {
    fn default() -> Self {
        Self::Local
    }
}

/// MCP 服务器绑定（E1.1 Workspace.mcp_servers 条目＋S7.1 三开关＋ERP 门）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpServerBinding {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub switchable: bool,
    #[serde(default)]
    pub selectable: bool,
    #[serde(default)]
    pub open_flag: bool,
    #[serde(default)]
    pub requires_erp: bool,
}

/// 文件门禁白名单（S7.1：preview/upload）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileGates {
    #[serde(default)]
    pub preview_exts: Vec<String>,
    #[serde(default)]
    pub upload_exts: Vec<String>,
}

/// E1.1 Workspace 实体配置（含 S7.1 governance/file_gates/menus）。
///
/// 注：`WorkspaceConfig` 已被 `l1_action::nt_act::nt_act_workspace_isolator`
/// （隔离配置，语义不同）占用，故本类型后缀改名为 `WorkspaceEntityConfig`。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceEntityConfig {
    #[serde(default)]
    pub model_tier_preference: Option<String>,
    #[serde(default)]
    pub max_concurrent_agents: u32,
    #[serde(default)]
    pub auto_save: bool,
    #[serde(default)]
    pub governance_level: String,
    #[serde(default)]
    pub file_gates: FileGates,
    #[serde(default)]
    pub menus: Vec<String>,
    #[serde(default)]
    pub feature_flags: Vec<String>,
    #[serde(default)]
    pub memory_offline_extract_enabled: bool,
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
            kind: WorkspaceKind::default(),
            agent_ids: Vec::new(),
            skill_ids: Vec::new(),
            mcp_servers: Vec::new(),
            shared_memory_keys: Vec::new(),
            config: WorkspaceEntityConfig::default(),
            locale: String::new(),
            llm_response_language: String::new(),
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
        crate::l0_substrate::nt_core_state::save("workspaces", &self.to_json()?)
    }

    /// Phase 2 KB 直写: 可注入连接变体 (测试用内存连接)。
    pub fn save_with(&self, conn: &rusqlite::Connection) -> Result<(), String> {
        crate::l0_substrate::nt_core_state::save_with(conn, "workspaces", &self.to_json()?)
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))
    }

    pub fn load() -> Self {
        crate::l0_substrate::nt_core_state::load("workspaces")
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Phase 2 KB 直写: 可注入连接变体 (测试用内存连接)。
    pub fn load_with(conn: &rusqlite::Connection) -> Self {
        crate::l0_substrate::nt_core_state::load_with(conn, "workspaces")
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

    #[test]
    fn workspace_full_construct_serde_roundtrip_with_defaults() {
        let now = chrono::Utc::now();
        let ws = WorkSpace {
            id: "ws-full-1".to_string(),
            name: "full".to_string(),
            created_at: now,
            last_active: now,
            project_root: Some(PathBuf::from("/repo")),
            description: "full construct".to_string(),
            tags: vec!["t1".to_string()],
            memory_count: 2,
            goal_count: 3,
            skill_count: 4,
            kind: WorkspaceKind::Remote {
                url: "https://example.invalid/ws".to_string(),
            },
            agent_ids: vec!["a1".to_string()],
            skill_ids: vec!["s1".to_string()],
            mcp_servers: vec![McpServerBinding {
                name: "erp".to_string(),
                command: "npx".to_string(),
                args: vec!["-y".to_string()],
                enabled: true,
                switchable: true,
                selectable: true,
                open_flag: true,
                requires_erp: true,
            }],
            shared_memory_keys: vec!["mem-k1".to_string()],
            config: WorkspaceEntityConfig {
                model_tier_preference: Some("expert".to_string()),
                max_concurrent_agents: 4,
                auto_save: true,
                governance_level: "enforce".to_string(),
                file_gates: FileGates {
                    preview_exts: vec!["md".to_string()],
                    upload_exts: vec!["pdf".to_string()],
                },
                menus: vec!["ceo".to_string()],
                feature_flags: vec!["imageGen".to_string()],
                memory_offline_extract_enabled: true,
            },
            locale: "zh-CN".to_string(),
            llm_response_language: "zh".to_string(),
        };
        let json = serde_json::to_string(&ws).unwrap();
        let back: WorkSpace = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "ws-full-1");
        assert!(matches!(
            back.kind,
            WorkspaceKind::Remote { ref url } if url == "https://example.invalid/ws"
        ));
        assert_eq!(back.mcp_servers.len(), 1);
        assert!(back.mcp_servers[0].requires_erp);
        assert_eq!(back.config.model_tier_preference.as_deref(), Some("expert"));
        assert_eq!(back.locale, "zh-CN");

        // 旧快照兼容：缺新键 → 全部回退默认值。
        let legacy = serde_json::json!({
            "id": "ws-legacy",
            "name": "legacy",
            "created_at": now.to_rfc3339(),
            "last_active": now.to_rfc3339(),
            "project_root": null,
            "description": "old snapshot",
            "tags": [],
            "memory_count": 0,
            "goal_count": 0,
            "skill_count": 0
        });
        let legacy_ws: WorkSpace = serde_json::from_value(legacy).unwrap();
        assert!(matches!(legacy_ws.kind, WorkspaceKind::Local));
        assert!(legacy_ws.agent_ids.is_empty());
        assert!(legacy_ws.skill_ids.is_empty());
        assert!(legacy_ws.mcp_servers.is_empty());
        assert!(legacy_ws.shared_memory_keys.is_empty());
        assert!(legacy_ws.config.model_tier_preference.is_none());
        assert!(legacy_ws.locale.is_empty());
        assert!(legacy_ws.llm_response_language.is_empty());
    }
}
