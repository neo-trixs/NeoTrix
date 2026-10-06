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
    /// 解析审批模式。**未知值返回 `Err`**，不再返回 `None` 让调用方兜底。
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

/// 动作的**内容指纹**（2026-10-06）。
///
/// ## 为什么需要它（真实缺陷，不是前瞻设计）
/// `ApprovalEngine::approve(id)` **只凭 id** 批准，而执行发生在**稍后**、
/// 在**别处**（`execute_tool`）。⇒ 从「批准」到「执行」之间，
/// 动作内容**没有任何一步被复核**。
/// 若这期间动作被替换（队列重渲染 / 子代理中转 / handoff 转述），
/// 那次批准会被**原样用在另一份内容上**。
///
/// 外部同源判据：
/// · 一份公开的系统提示纪律：*「确认只覆盖用户当时看到的那份具体内容；
///   内容若在批准后改变，先出示新版本」* —— 与本仓
///   `uber/ADR` 吸收的 provenance 纪律同族。
/// · `uber/ADR`（Apache-2.0）：用摘要而非内容来证明「是这一份」。
///
/// ## 判据设计
/// · 只覆盖**会改变执行结果**的字段（路径 / 内容 / diff / 命令 / 描述 / 参数），
///   **不含** `id` 与时间戳 —— 否则同一动作两次提交会算出不同指纹，
///   「同一动作」就无从判定。
/// · **短摘要**（12 位 hex）足够：它要证明的是「变了没有」，不是密码学防碰撞。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActionFingerprint(pub String);

impl ActionFingerprint {
    /// 计算动作的内容指纹。
    pub fn of(action: &ActionType) -> Self {
        use sha2::Digest;
        // ⛔ 字段顺序是判据的一部分：改它会让历史指纹全部失配。
        //   故此处**只用显式列举**，不用 `{:?}`（derive 输出会随代码变动而变）。
        let mut h = sha2::Sha256::new();
        match action {
            ActionType::FileWrite { path, content_preview } => {
                h.update(b"FileWrite\0");
                h.update(path.as_bytes());
                h.update(b"\0");
                h.update(content_preview.as_bytes());
            }
            ActionType::FileCreate { path } => {
                h.update(b"FileCreate\0");
                h.update(path.as_bytes());
            }
            ActionType::FileEdit { path, diff } => {
                h.update(b"FileEdit\0");
                h.update(path.as_bytes());
                h.update(b"\0");
                h.update(diff.as_bytes());
            }
            ActionType::ShellCommand { command } => {
                h.update(b"ShellCommand\0");
                h.update(command.as_bytes());
            }
            ActionType::GitOperation { description } => {
                h.update(b"GitOperation\0");
                h.update(description.as_bytes());
            }
            ActionType::Other { tool, args } => {
                h.update(b"Other\0");
                h.update(tool.as_bytes());
                h.update(b"\0");
                h.update(args.as_bytes());
            }
        }
        ActionFingerprint(hex::encode(h.finalize())[..12].to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 内容是否仍与批准时一致。
    pub fn matches(&self, action: &ActionType) -> bool {
        *self == ActionFingerprint::of(action)
    }
}

impl std::fmt::Display for ActionFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct PendingAction {
    pub id: String,
    pub action_type: ActionType,
    pub description: String,
    /// 提交时的**内容指纹**（2026-10-06）。
    ///
    /// ⛔ 它与 `description` 里那段 `sha256:` 字符串**刻意重复** ——
    /// 那段是给人看的，这字段是给**执行前复核**用的。
    /// 合成一个会迫使执行侧去解析人类可读串（脆），或反过来让人从
    /// 展示文本反推是否可信（更脆）。
    pub content_fingerprint: ActionFingerprint,
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
