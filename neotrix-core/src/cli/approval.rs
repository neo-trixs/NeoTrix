use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::sync::Mutex;
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
    FileWrite { path: String, content_preview: String },
    FileCreate { path: String },
    FileEdit { path: String, diff: String },
    ShellCommand { command: String },
    GitOperation { description: String },
    /// 未映射到具体类别的工具调用（AgentLoop 工具审批兜底）。
    Other { tool: String, args: String },
}

#[derive(Debug, Clone)]
pub struct PendingAction {
    pub id: String,
    pub action_type: ActionType,
    pub description: String,
    /// W2.1 poka-yoke 披露门 (batch3 2026-08-26, 源: rainmanjam/poka-yoke):
    /// 此动作将关闭的可能性。审批 UI/回调可见; 空表 = 未披露 (Detection 级)。
    pub forecloses: Vec<String>,
    pub created_at: Instant,
}

/// Andon 严重度梯子 (poka-yoke 三级信号塔)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureSeverity {
    /// 错误不可能发生 = 硬拒绝层 (SecurityInspector Deny 层负责)
    Control,
    /// 随发生即宣告 = 审批请求携带 forecloses 披露
    Warning,
    /// 事后才被发现 = 高影响动作但无披露
    Detection,
}

impl PendingAction {
    pub fn disclosure_severity(&self) -> DisclosureSeverity {
        if self.forecloses.is_empty() {
            DisclosureSeverity::Detection
        } else {
            DisclosureSeverity::Warning
        }
    }
}

/// W2.1 核心: 从动作推断其关闭的可能性 — 把 "agent 很少主动说出 fix 关闭了什么"
/// 机械化到审批门本身。确定性模式匹配, 无 LLM。
pub fn infer_foreclosures(action: &ActionType) -> Vec<String> {
    let mut out = Vec::new();
    match action {
        ActionType::ShellCommand { command } => {
            let c = command.to_lowercase();
            if c.contains("rm -rf") || c.contains("rm -fr") || c.contains("rmdir") {
                out.push("目标路径数据不可恢复".into());
            }
            if c.contains("git push") && (c.contains("--force") || c.contains("-f")) {
                out.push("远端历史被覆盖, 协作者本地分叉失效".into());
            }
            if c.contains("drop table") || c.contains("drop database") || c.contains("truncate table") {
                out.push("数据库结构/数据即刻丢失".into());
            }
            if c.contains("dd ") && (c.contains("of=/dev/") || c.contains("oflag")) || c.contains("mkfs") {
                out.push("目标块设备全盘覆写".into());
            }
            if c.contains("chmod -r 777") || c.contains("chmod 777 /") {
                out.push("权限边界永久放开, 审计链失效".into());
            }
            if c.contains("systemctl stop") || c.contains("service stop") || c.contains("reboot")
                || c.contains("shutdown") || c.contains("pkill") || c.contains("killall")
            {
                out.push("运行中服务/进程即时中断".into());
            }
        }
        ActionType::GitOperation { description } => {
            let d = description.to_lowercase();
            if d.contains("--force") || d.contains("push -f") {
                out.push("远端历史被覆盖, 协作者本地分叉失效".into());
            }
            if d.contains("reset --hard") || d.contains("checkout -- .") || d.contains("clean -fd") {
                out.push("未提交工作区改动不可恢复".into());
            }
        }
        _ => {}
    }
    out
}

pub struct ApprovalEngine {
    mode: ApprovalMode,
    pending: Vec<PendingAction>,
    next_id: u64,
}

impl ApprovalEngine {
    pub fn new(mode: ApprovalMode) -> Self {
        Self { mode, pending: Vec::new(), next_id: 0 }
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
                // AutoEdit 白名单: 文件类免审批; 命令/git/未分类工具 (Other) 需审批
                matches!(
                    action,
                    ActionType::ShellCommand { .. }
                        | ActionType::GitOperation { .. }
                        | ActionType::Other { .. }
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
        // W2.1 披露门: 审批门替 agent 说出 fix 关闭了什么 (Warning 级 andon)
        let forecloses = infer_foreclosures(&action);
        let description = if forecloses.is_empty() {
            description
        } else {
            format!("{}\n⚠ 此操作将关闭: {}", description, forecloses.join("; "))
        };
        if !forecloses.is_empty() {
            log::warn!(
                "poka-yoke disclosure: {} → forecloses {:?}",
                id, forecloses
            );
        }
        let pa = PendingAction {
            id,
            action_type: action,
            description,
            forecloses,
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
        ActionType::FileWrite { path, content_preview } => {
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
        ActionType::Other { tool, args } => format!("🔧 Tool {}: {}", tool, args),
    }
}

/// Global approval engine, lazily initialized.
// TODO: inject via DI — pass &ApprovalEngine through CLI command chain instead
pub static APPROVAL_ENGINE: LazyLock<Mutex<ApprovalEngine>> = LazyLock::new(|| {
    Mutex::new(ApprovalEngine::new(ApprovalMode::Suggest))
});

pub fn global_approval() -> &'static Mutex<ApprovalEngine> {
    &APPROVAL_ENGINE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_requires_all() {
        let engine = ApprovalEngine::new(ApprovalMode::Suggest);
        assert!(engine.require_approval(&ActionType::FileWrite { path: "x".into(), content_preview: "".into() }));
        assert!(engine.require_approval(&ActionType::FileCreate { path: "x".into() }));
        assert!(engine.require_approval(&ActionType::FileEdit { path: "x".into(), diff: "".into() }));
        assert!(engine.require_approval(&ActionType::ShellCommand { command: "ls".into() }));
        assert!(engine.require_approval(&ActionType::GitOperation { description: "commit".into() }));
    }

    #[test]
    fn test_auto_edit_approves_files() {
        let engine = ApprovalEngine::new(ApprovalMode::AutoEdit);
        assert!(!engine.require_approval(&ActionType::FileWrite { path: "x".into(), content_preview: "".into() }));
        assert!(!engine.require_approval(&ActionType::FileCreate { path: "x".into() }));
        assert!(!engine.require_approval(&ActionType::FileEdit { path: "x".into(), diff: "".into() }));
        assert!(engine.require_approval(&ActionType::ShellCommand { command: "ls".into() }));
        assert!(engine.require_approval(&ActionType::GitOperation { description: "commit".into() }));
    }

    #[test]
    fn test_full_auto_requires_nothing() {
        let engine = ApprovalEngine::new(ApprovalMode::FullAuto);
        assert!(!engine.require_approval(&ActionType::FileWrite { path: "x".into(), content_preview: "".into() }));
        assert!(!engine.require_approval(&ActionType::FileCreate { path: "x".into() }));
        assert!(!engine.require_approval(&ActionType::FileEdit { path: "x".into(), diff: "".into() }));
        assert!(!engine.require_approval(&ActionType::ShellCommand { command: "ls".into() }));
        assert!(!engine.require_approval(&ActionType::GitOperation { description: "commit".into() }));
    }

    #[test]
    fn test_submit_approve_deny_cycle() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = engine.submit(ActionType::FileWrite { path: "/tmp/test".into(), content_preview: "hello".into() });
        assert_eq!(engine.pending_count(), 1);
        assert!(engine.approve(&pa.id).is_ok());
        assert_eq!(engine.pending_count(), 0);

        let pa2 = engine.submit(ActionType::FileCreate { path: "/tmp/test2".into() });
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
        engine.submit(ActionType::FileWrite { path: "a".into(), content_preview: "".into() });
        engine.submit(ActionType::FileCreate { path: "b".into() });
        assert_eq!(engine.approve_all(), 2);
        assert_eq!(engine.pending_count(), 0);
    }

    #[test]
    fn test_mode_from_str() {
        assert_eq!(ApprovalMode::from_str("suggest"), Some(ApprovalMode::Suggest));
        assert_eq!(ApprovalMode::from_str("auto-edit"), Some(ApprovalMode::AutoEdit));
        assert_eq!(ApprovalMode::from_str("full-auto"), Some(ApprovalMode::FullAuto));
        assert_eq!(ApprovalMode::from_str("yolo"), Some(ApprovalMode::FullAuto));
        assert_eq!(ApprovalMode::from_str("unknown"), None);
    }

    #[test]
    fn test_global_engine() {
        let engine = global_approval();
        let mut e = engine.lock().unwrap();
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
        engine.submit(ActionType::FileWrite { path: "x.txt".into(), content_preview: "data".into() });
        let list = engine.pending_actions();
        assert_eq!(list.len(), 1);
        assert!(list[0].description.contains("Write"));
    }

    #[test]
    fn test_other_action_type() {
        // Other 兜底：Suggest 下需审批，AutoEdit 下也需审批（非文件/非命令白名单）。
        let engine = ApprovalEngine::new(ApprovalMode::Suggest);
        assert!(engine.require_approval(&ActionType::Other { tool: "web_search".into(), args: "q=rust".into() }));
        let engine = ApprovalEngine::new(ApprovalMode::AutoEdit);
        assert!(engine.require_approval(&ActionType::Other { tool: "web_search".into(), args: "q=rust".into() }));
        let engine = ApprovalEngine::new(ApprovalMode::FullAuto);
        assert!(!engine.require_approval(&ActionType::Other { tool: "web_search".into(), args: "q=rust".into() }));
        // describe_action 不 panic 且包含工具名。
        let pa = ApprovalEngine::new(ApprovalMode::Suggest)
            .submit(ActionType::Other { tool: "web_search".into(), args: "q=rust".into() });
        assert!(pa.description.contains("web_search"));
    }

    // ── W2.1 (batch3 2026-08-26, rainmanjam/poka-yoke 吸收) 披露门验收 ──

    #[test]
    fn test_infer_foreclosures_destructive_shell() {
        let f = infer_foreclosures(&ActionType::ShellCommand {
            command: "rm -rf ./build && echo done".into(),
        });
        assert!(f.iter().any(|s| s.contains("不可恢复")), "{f:?}");

        let f = infer_foreclosures(&ActionType::ShellCommand {
            command: "git push --force origin main".into(),
        });
        assert!(f.iter().any(|s| s.contains("远端历史")), "{f:?}");
    }

    #[test]
    fn test_infer_foreclosures_git_reset_hard() {
        let f = infer_foreclosures(&ActionType::GitOperation {
            description: "reset --hard to v1".into(),
        });
        assert!(f.iter().any(|s| s.contains("工作区改动")), "{f:?}");
    }

    #[test]
    fn test_benign_action_no_disclosure() {
        assert!(infer_foreclosures(&ActionType::ShellCommand { command: "ls -la".into() }).is_empty());
        assert!(infer_foreclosures(&ActionType::FileEdit { path: "a.rs".into(), diff: "-old\n+new".into() }).is_empty());
    }

    #[test]
    fn test_submit_attaches_disclosure_and_severity() {
        let mut engine = ApprovalEngine::new(ApprovalMode::Suggest);
        let pa = engine.submit(ActionType::ShellCommand { command: "rm -rf /tmp/x".into() });
        assert_eq!(pa.disclosure_severity(), DisclosureSeverity::Warning);
        assert!(!pa.forecloses.is_empty());
        assert!(pa.description.contains("此操作将关闭"), "{}", pa.description);

        let benign = engine.submit(ActionType::ShellCommand { command: "ls".into() });
        assert_eq!(benign.disclosure_severity(), DisclosureSeverity::Detection);
    }
}
