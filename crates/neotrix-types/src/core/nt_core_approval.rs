//! L0 正典审批类型（T06a）：五实体唯一事实源。
//!
//! `ApprovalMode` / `ActionType` / `PendingAction` 此前散落在 L6
//! `nt_approval.rs`；现收敛至此，L6 仅 `pub use` 重导出＋引擎实现。
//! `ApproveGate`（T06b）是引擎不知情的 L0 trait：L3 经回调调用，
//! L6 `ApprovalEngine` 实现并注入（T06c），L3 永不直引 L6。

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 审批模式：建议→自动编辑→全自动（yolo 为 FullAuto 别名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalMode {
    Suggest,
    AutoEdit,
    FullAuto,
}

impl ApprovalMode {
    /// ⭐⭐⭐ 解析审批模式。**未知值返回 `Err`**，不再返回 `None` 让调用方兜底。
    ///
    /// 【缺陷（2026-10-06 修）】首版返回 `Option`，
    /// 而**唯一的调用方**（`nt_permission_profiles::plan_profile_switch`）
    /// 当时写的是 `.and_then(|s| ApprovalMode::from_str(s))`
    /// ⇒ 档位里写错一个字母 ⇒ `None` ⇒ **该档的审批模式覆盖被静默忽略**
    /// ⇒ 切换后模式与用户预期不符，且**无任何错误**。
    ///
    /// 判据与 `nt_sandbox::SandboxMode::from_str` 相同：
    /// **静默失效只允许朝严格方向回落**；而「拼错 → 忽略覆盖」朝的是宽松方向
    /// （`developer` 档的 `auto-edit` 若被拼错 ⇒ 停在 `Suggest`，
    /// 看似更严，实际是**用户要的自动批准没生效且没人知道**）。
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "suggest" | "manual" => Ok(Self::Suggest),
            "auto-edit" | "auto_edit" | "autoedit" => Ok(Self::AutoEdit),
            "full-auto" | "full_auto" | "fullauto" | "yolo" => Ok(Self::FullAuto),
            other => Err(format!(
                "unknown approval mode {other:?}: use suggest | auto-edit | full-auto"
            )),
        }
    }

    /// 严格程度的序数（越大越松）：`Suggest`=0 < `AutoEdit`=1 < `FullAuto`=2。
    ///
    /// 供「本次切换是否**放宽**审批」这类判据使用
    /// （见 `nt_permission_profiles::ProfileSwitchPlan::loosens_approval`）。
    pub fn strictness_rank(self) -> u8 {
        match self {
            Self::Suggest => 0,
            Self::AutoEdit => 1,
            Self::FullAuto => 2,
        }
    }

    /// 人类可读的档位名（用于 CLI 输出与审计行）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggest => "suggest",
            Self::AutoEdit => "auto-edit",
            Self::FullAuto => "full-auto",
        }
    }
}

/// 需审批的动作类型（与各域同名 `ActionType` 无关，此为审批域正典）。
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

#[derive(Debug, Clone)]
pub struct PendingAction {
    pub id: String,
    pub action_type: ActionType,
    pub description: String,
    /// W2.1 poka-yoke 披露门：此动作将关闭的可能性；空表 = 未披露 (Detection 级)。
    pub forecloses: Vec<String>,
    pub created_at: Instant,
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

/// T06b：审批门 trait——引擎不知情，L3 只依赖此 trait（经回调），L6 实现并注入。
pub trait ApproveGate: Send + Sync {
    /// 当前模式下该动作是否需要审批。
    fn needs_approval(&self, action: &ActionType) -> bool;
    /// 当前审批模式。
    fn approval_mode(&self) -> ApprovalMode;
}
