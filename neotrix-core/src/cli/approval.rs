use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalMode {
    Suggest,
    AutoEdit,
    FullAuto,
}

impl ApprovalMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "suggest" | "Suggest" => Some(Self::Suggest),
            "auto-edit" | "auto_edit" | "AutoEdit" => Some(Self::AutoEdit),
            "full-auto" | "full_auto" | "FullAuto" | "yolo" => Some(Self::FullAuto),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    FileWrite {
        path: String,
        content_preview: String,
    },
    FileCreate {
        path: String,
    },
    FileEdit {
        path: String,
        diff: String,
    },
    ShellCommand {
        command: String,
    },
    GitOperation {
        description: String,
    },
}

#[derive(Debug, Clone)]
pub struct PendingAction {
    pub id: String,
    pub action_type: ActionType,
    pub description: String,
    pub created_at: Instant,
}

pub struct ApprovalEngine {
    mode: ApprovalMode,
    pending: Vec<PendingAction>,
    next_id: u64,
}

impl ApprovalEngine {
    pub fn new(mode: ApprovalMode) -> Self {
        Self {
            mode,
            pending: Vec::new(),
            next_id: 0,
        }
    }

    pub fn mode(&self) -> ApprovalMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ApprovalMode) {
        self.mode = mode;
    }

    /// Check whether a given action type requires user approval under current mode.
    /// Respects active permission profile (deny overrides everything).
    pub fn require_approval(&self, action: &ActionType) -> bool {
        // Profile deny takes precedence over everything
        let action_key = crate::cli::permission_profiles::action_type_to_key(action);
        if crate::cli::permission_profiles::is_action_denied(action_key) {
            return true; // action is blocked — require approval to inform user
        }
        // Profile allow overrides mode (no approval needed)
        if crate::cli::permission_profiles::is_action_allowed(action_key) {
            return false;
        }
        match self.mode {
            ApprovalMode::Suggest => true,
            ApprovalMode::AutoEdit => {
                matches!(
                    action,
                    ActionType::ShellCommand { .. } | ActionType::GitOperation { .. }
                )
            }
            ApprovalMode::FullAuto => false,
        }
    }

    /// Submit a new action for approval. Returns the pending action.
    pub fn submit(&mut self, action: ActionType) -> PendingAction {
        let id = format!("a{:04}", self.next_id);
        self.next_id += 1;
        let description = describe_action(&action);
        let pa = PendingAction {
            id,
            action_type: action,
            description,
            created_at: Instant::now(),
        };
        self.pending.push(pa.clone());
        pa
    }

    pub fn approve(&mut self, id: &str) -> Result<(), String> {
        let idx = self
            .pending
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| format!("No pending action with id '{}'", id))?;
        self.pending.remove(idx);
        Ok(())
    }

    pub fn deny(&mut self, id: &str) -> Result<(), String> {
        let idx = self
            .pending
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| format!("No pending action with id '{}'", id))?;
        self.pending.remove(idx);
        Ok(())
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn summary(&self) -> String {
        let mode_str = match self.mode {
            ApprovalMode::Suggest => "Suggest",
            ApprovalMode::AutoEdit => "AutoEdit",
            ApprovalMode::FullAuto => "FullAuto",
        };
        format!("Mode: {} | Pending: {}", mode_str, self.pending.len())
    }

    pub fn pending_actions(&self) -> &[PendingAction] {
        &self.pending
    }

    pub fn approve_all(&mut self) -> usize {
        let count = self.pending.len();
        self.pending.clear();
        count
    }

    pub fn deny_all(&mut self) -> usize {
        let count = self.pending.len();
        self.pending.clear();
        count
    }
}

fn describe_action(action: &ActionType) -> String {
    match action {
        ActionType::FileWrite {
            path,
            content_preview,
        } => {
            let preview = if content_preview.len() > 60 {
                format!("{}…", &content_preview[..57])
            } else {
                content_preview.clone()
            };
            format!("📝 Write {}: {}", path, preview)
        }
        ActionType::FileCreate { path } => format!("📄 Create {}", path),
        ActionType::FileEdit { path, diff } => {
            let d = if diff.len() > 60 {
                format!("{}…", &diff[..57])
            } else {
                diff.clone()
            };
            format!("✏️ Edit {}: {}", path, d)
        }
        ActionType::ShellCommand { command } => format!("💻 Run: {}", command),
        ActionType::GitOperation { description } => format!("🔧 Git: {}", description),
    }
}

/// Global approval engine, lazily initialized.
pub static APPROVAL_ENGINE: OnceLock<Mutex<ApprovalEngine>> = OnceLock::new();

pub fn global_approval() -> &'static Mutex<ApprovalEngine> {
    APPROVAL_ENGINE.get_or_init(|| Mutex::new(ApprovalEngine::new(ApprovalMode::Suggest)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_suggest_requires_all() {
        let engine = ApprovalEngine::new(ApprovalMode::Suggest);
        assert!(engine.require_approval(&ActionType::FileWrite {
            path: "x".into(),
            content_preview: "".into()
        }));
        assert!(engine.require_approval(&ActionType::FileCreate { path: "x".into() }));
        assert!(engine.require_approval(&ActionType::FileEdit {
            path: "x".into(),
            diff: "".into()
        }));
        assert!(engine.require_approval(&ActionType::ShellCommand {
            command: "ls".into()
        }));
        assert!(engine.require_approval(&ActionType::GitOperation {
            description: "commit".into()
        }));
    }

    #[test]
    fn test_auto_edit_approves_files() {
        let engine = ApprovalEngine::new(ApprovalMode::AutoEdit);
        assert!(!engine.require_approval(&ActionType::FileWrite {
            path: "x".into(),
            content_preview: "".into()
        }));
        assert!(!engine.require_approval(&ActionType::FileCreate { path: "x".into() }));
        assert!(!engine.require_approval(&ActionType::FileEdit {
            path: "x".into(),
            diff: "".into()
        }));
        assert!(engine.require_approval(&ActionType::ShellCommand {
            command: "ls".into()
        }));
        assert!(engine.require_approval(&ActionType::GitOperation {
            description: "commit".into()
        }));
    }

    #[test]
    fn test_full_auto_requires_nothing() {
        let engine = ApprovalEngine::new(ApprovalMode::FullAuto);
        assert!(!engine.require_approval(&ActionType::FileWrite {
            path: "x".into(),
            content_preview: "".into()
        }));
        assert!(!engine.require_approval(&ActionType::FileCreate { path: "x".into() }));
        assert!(!engine.require_approval(&ActionType::FileEdit {
            path: "x".into(),
            diff: "".into()
        }));
        assert!(!engine.require_approval(&ActionType::ShellCommand {
            command: "ls".into()
        }));
        assert!(!engine.require_approval(&ActionType::GitOperation {
            description: "commit".into()
        }));
    }

    #[test]
    fn test_submit_approve_deny_cycle() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = engine.submit(ActionType::FileWrite {
            path: "/tmp/test".into(),
            content_preview: "hello".into(),
        });
        assert_eq!(engine.pending_count(), 1);
        assert!(engine.approve(&pa.id).is_ok());
        assert_eq!(engine.pending_count(), 0);

        let pa2 = engine.submit(ActionType::FileCreate {
            path: "/tmp/test2".into(),
        });
        assert!(engine.deny(&pa2.id).is_ok());
        assert_eq!(engine.pending_count(), 0);
    }

    #[test]
    fn test_approve_unknown_id() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        assert!(engine.approve("nonexistent").is_err());
    }

    #[test]
    fn test_approve_all() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        engine.submit(ActionType::FileWrite {
            path: "a".into(),
            content_preview: "".into(),
        });
        engine.submit(ActionType::FileCreate { path: "b".into() });
        assert_eq!(engine.approve_all(), 2);
        assert_eq!(engine.pending_count(), 0);
    }

    #[test]
    fn test_mode_from_str() {
        assert_eq!(
            ApprovalMode::from_str("suggest"),
            Some(ApprovalMode::Suggest)
        );
        assert_eq!(
            ApprovalMode::from_str("auto-edit"),
            Some(ApprovalMode::AutoEdit)
        );
        assert_eq!(
            ApprovalMode::from_str("full-auto"),
            Some(ApprovalMode::FullAuto)
        );
        assert_eq!(ApprovalMode::from_str("yolo"), Some(ApprovalMode::FullAuto));
        assert_eq!(ApprovalMode::from_str("unknown"), None);
    }

    #[serial]
    #[test]
    fn test_global_engine() {
        let engine = global_approval();
        let mut e = engine.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(e.mode(), ApprovalMode::Suggest);
        e.set_mode(ApprovalMode::FullAuto);
        assert_eq!(e.mode(), ApprovalMode::FullAuto);
        e.set_mode(ApprovalMode::Suggest);
    }

    #[test]
    fn test_summary() {
        let engine = ApprovalEngine::new(ApprovalMode::Suggest);
        assert_eq!(engine.summary(), "Mode: Suggest | Pending: 0");
    }

    #[test]
    fn test_pending_actions_list() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        engine.submit(ActionType::FileWrite {
            path: "x.txt".into(),
            content_preview: "data".into(),
        });
        let list = engine.pending_actions();
        assert_eq!(list.len(), 1);
        assert!(list[0].description.contains("Write"));
    }
}
