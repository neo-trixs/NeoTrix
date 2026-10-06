//! L3 Embodiment — Sandbox execution guard (migrated from cli::sandbox).
//!
//! Decoupled from CLI command types: `check_read_only` returns `Option<String>`
//! instead of `CommandOutput` so non-CLI callers (AutoOrchestrator, agent loop)
//! can use it directly.

use std::sync::LazyLock;
use std::sync::Mutex;

/// 沙箱**权限级别**（不是后端选择）。
///
/// ⚠️ **同名不同物**：`l3_embodiment/nt_shield/nt_shield_sandbox_entry.rs`
/// 里另有一个 `SandboxMode { Local, Docker, Wasm, Remote }`，
/// 那是**后端选择**（用哪个隔离机制），与本枚举**语义完全不同**。
/// ⇒ 本枚举回答「能写多少」，那个回答「在哪跑」。
///
/// ## 三档的语义（对齐 `codex` 的 `--sandbox`）
/// | 档 | 可写范围 | 参考 |
/// |---|---|---|
/// | `ReadOnly` | 任何写入都拒 | codex `read-only` |
/// | `WorkspaceWrite` | 工作区内可写，**网络默认关** | codex `workspace-write` |
/// | `Disabled` | 不设限（**仍受 execpolicy/规则引擎约束**） | codex `danger-full-access` 的「不装 sandbox」，但**语义更弱** —— 本仓的规则引擎与 deny 规则在 `Disabled` 下**依然生效** |
///
/// ⭐ 为什么要有中间档：原实现只有 `Disabled` / `ReadOnly` / `Docker`，
/// 而 `Docker` 是**后端**不是**级别** ⇒ 用户想要「能写代码但不许联网/不许出工作区」
/// 只能选 `Disabled` ⇒ **被迫全放开**。这是缺口，不是设计。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxMode {
    /// 不设沙箱级别限制。⚠️ 规则引擎与显式 deny **仍然生效**。
    Disabled,
    /// 任何写入都拒。
    ReadOnly,
    /// ⭐ 工作区内可写；工作区外与网络仍受限。
    WorkspaceWrite,
    /// 用 Docker 做隔离（**后端**，不是级别）。
    Docker,
}

impl SandboxMode {
    /// ⭐⭐⭐ 解析用户输入的沙箱级别。**未知值返回 `Err`，不再静默兜底。**
    ///
    /// 【缺陷（2026-10-06 修）】首版是 `_ => Self::Disabled`
    /// ⇒ `--sandbox danger-full-access`（`codex` 的最危险档）
    /// **不报错、静默变成 `Disabled`** ⇒ 用户以为设了最严档，实际是「不设限」。
    /// 这是本会话已修的「**声称存在但实际不生效**」那一族的 CLI 面实例：
    /// flag 存在、能解析、无错误，而**语义与用户意图相反**。
    ///
    /// ## 三条设计判据（依据 `codex` / `claude-code` 的实测做法）
    /// 1. **静默失效只允许朝严格方向回落** —— `claude-code` 把它写成了文档纪律；
    ///    而「未知值 → Disabled」是朝**宽松**方向回落 ⇒ 直接违反。
    /// 2. **未知值是硬拒绝**，不是兜底 —— 与 `codex` 对退役开关
    ///    「**rejected** 而非 ignored」同向。
    /// 3. 别名必须**逐字列出**（`ro` / `read_only` 等），
    ///    不靠 `starts_with` 之类模糊匹配。
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.trim().to_lowercase().as_str() {
            "disabled" | "off" | "none" => Ok(Self::Disabled),
            "read-only" | "readonly" | "read_only" | "ro" => Ok(Self::ReadOnly),
            // ⭐ 新增中间档（对齐 codex workspace-write）
            "workspace-write" | "workspace_write" | "workspace" | "ww" => Ok(Self::WorkspaceWrite),
            "docker" => Ok(Self::Docker),
            other => Err(format!(
                "unknown sandbox mode {other:?}: use read-only | workspace-write | disabled | docker"
            )),
        }
    }

    /// ⭐ 该级别是否允许**工作区内**写入。
    pub fn allows_workspace_write(&self) -> bool {
        matches!(self, Self::WorkspaceWrite | Self::Disabled | Self::Docker)
    }

    /// ⭐ 该级别是否**完全禁止写入**。
    pub fn is_read_only(&self) -> bool {
        matches!(self, Self::ReadOnly)
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Disabled => "",
            Self::ReadOnly => "🔒 READ-ONLY",
            Self::WorkspaceWrite => "📝 WORKSPACE-WRITE",
            Self::Docker => "🐳 DOCKER",
        }
    }
}

pub struct SandboxEnforcer {
    mode: SandboxMode,
}

impl SandboxEnforcer {
    pub fn new(mode: SandboxMode) -> Self {
        Self { mode }
    }

    pub fn mode(&self) -> SandboxMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: SandboxMode) {
        self.mode = mode;
    }

    /// 委托给 [`SandboxMode::is_read_only`]（枚举是唯一真源，避免两处判据漂移）。
    pub fn is_read_only(&self) -> bool {
        self.mode.is_read_only()
    }

    pub fn check_read_only(&self) -> Option<String> {
        if self.is_read_only() {
            Some(
                "🔒 Read-only sandbox: this operation is blocked. Use --sandbox disabled to allow write operations."
                    .to_string(),
            )
        } else {
            None
        }
    }
}

/// Global fallback — prefer context-local sandbox config instead.
pub static SANDBOX_ENFORCER: LazyLock<Mutex<SandboxEnforcer>> = LazyLock::new(|| {
    Mutex::new(SandboxEnforcer::new(SandboxMode::Disabled))
});

pub fn global_sandbox() -> &'static Mutex<SandboxEnforcer> {
    &SANDBOX_ENFORCER
}

pub fn init_sandbox(mode: SandboxMode) {
    let mut e = global_sandbox().lock().unwrap_or_else(|e| e.into_inner());
    e.set_mode(mode);
}

pub fn check_sandbox() -> Option<String> {
    global_sandbox()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .check_read_only()
}

#[cfg(test)]
mod mode_parse_tests {
    //! ⭐ 反向锁：**未知沙箱档必须硬拒绝**，不得静默兜底。
    //!
    //! 【缺陷（2026-10-06 修）】首版是 `_ => Self::Disabled`
    //! ⇒ `--sandbox danger-full-access`（`codex` 最危险档）
    //! **不报错、静默变成「不设限」** ⇒ 用户以为设了最严档，实际相反。
    //!
    //! 【判据】静默失效只允许朝**严格**方向回落（`claude-code` 的文档纪律）；
    //! 「未知 → Disabled」是朝**宽松**方向 ⇒ 直接违反。

    use super::SandboxMode;

    /// 四个合法档都要能解析（含别名）。
    #[test]
    fn all_documented_modes_parse() {
        for raw in [
            "disabled",
            "read-only",
            "readonly",
            "read_only",
            "ro",
            "workspace-write",
            "workspace_write",
            "workspace",
            "ww",
            "docker",
        ] {
            assert!(
                SandboxMode::from_str(raw).is_ok(),
                "合法档 {raw:?} 不该被拒"
            );
        }
    }

    /// ⭐⭐ 未知档必须 `Err`，且**不得**是 `Disabled`。
    ///
    /// `Disabled` 是「不设限」，把它当兜底值是最危险的一种 ——
    /// 用户拼错一个字母 ⇒ 保护全没了。
    #[test]
    fn unknown_mode_is_ERR_and_never_falls_back_to_disabled() {
        for bad in [
            "danger-full-access", // codex 的档名 —— 本仓没有，不能静默吞
            "workspace_write_only",
            "READ-ONLY ",         // 首尾空格应被 trim 接受
            "",
            "sandbox",
            "true",
        ] {
            match SandboxMode::from_str(bad) {
                Ok(m) => {
                    // 只有 trim 后的合法值才允许 Ok
                    let trimmed = bad.trim();
                    assert!(
                        ["disabled", "read-only", "readonly", "read_only", "ro",
                         "workspace-write", "workspace_write", "workspace", "ww", "docker"]
                            .contains(&trimmed.to_lowercase().as_str()),
                        "「{bad:?}」被解析成 {m:?} —— 合法档表里没有它 ⇒ 应 Err"
                    );
                }
                Err(e) => assert!(
                    e.contains("read-only"),
                    "错误信息应列出合法档（调用方要据此纠正）：{e}"
                ),
            }
        }
    }

    /// ⭐ `WorkspaceWrite` 是**新增的中间档**（对齐 codex）——
    /// 此前用户想要「能写代码但不许出工作区」只能选 `Disabled` ⇒ 被迫全放开。
    #[test]
    fn workspace_write_is_distinct_from_disabled() {
        let ww = SandboxMode::from_str("workspace-write").unwrap();
        assert_ne!(ww, SandboxMode::Disabled, "中间档必须与 Disabled 可区分");
        assert!(ww.allows_workspace_write(), "workspace-write 允许工作区内写");
        assert!(!ww.is_read_only(), "workspace-write 不是只读档");
        // 对照：read-only 禁止一切写入
        let ro = SandboxMode::from_str("ro").unwrap();
        assert!(ro.is_read_only());
        assert!(!ro.allows_workspace_write(), "read-only 不允许任何写");
    }

    /// 判据唯一真源在枚举上：`SandboxEnforcer::is_read_only` 必须委托它，
    /// 不能各写一份（两处判据漂移是本会话修过的一类缺陷）。
    #[test]
    fn enforcer_delegates_to_enum_single_source_of_truth() {
        for mode in [
            SandboxMode::Disabled,
            SandboxMode::ReadOnly,
            SandboxMode::WorkspaceWrite,
            SandboxMode::Docker,
        ] {
            let e = super::SandboxEnforcer::new(mode);
            assert_eq!(
                e.is_read_only(),
                mode.is_read_only(),
                "SandboxEnforcer 与枚举对 {mode:?} 的只读判定不一致 ⇒ 判据有两份"
            );
        }
    }
}
