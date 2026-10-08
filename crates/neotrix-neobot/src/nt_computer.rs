//! `nt_computer` — 受控 computer 动作.
//!
//! 受控动作表（11 声明 / 8 执行）+ 网关（resolve → policy →
//! 先写 audit → 再执行）。本地后端 trait 化：当前仅 `NoopBackend`
//! （诚实失败，不伪造截图/点击）；真浏览器后端按 `ComputerBackend`
//! 实现即插.

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;

/// 受控动作（执行子集）.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputerAction {
    Navigate,
    Click,
    Type,
    Key,
    Scroll,
    Screenshot,
    ReadFile,
    WriteFile,
    ListFiles,
}

impl ComputerAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Navigate => "navigate",
            Self::Click => "click",
            Self::Type => "type",
            Self::Key => "key",
            Self::Scroll => "scroll",
            Self::Screenshot => "screenshot",
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
            Self::ListFiles => "list_files",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "navigate" => Some(Self::Navigate),
            "click" => Some(Self::Click),
            "type" => Some(Self::Type),
            "key" => Some(Self::Key),
            "scroll" => Some(Self::Scroll),
            "screenshot" => Some(Self::Screenshot),
            "read_file" => Some(Self::ReadFile),
            "write_file" => Some(Self::WriteFile),
            "list_files" => Some(Self::ListFiles),
            _ => None,
        }
    }
}

/// 一次 computer 调用 (网关已审, 后端执行).
#[derive(Debug, Clone)]
pub struct ComputerCall {
    pub action: ComputerAction,
    pub target: String,
    pub text: String,
}

/// 解析 `computer_act` 参数.
pub fn parse_computer_call(args: &serde_json::Value) -> Result<ComputerCall, NtBotError> {
    let action_raw = args
        .get("action")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let Some(action) = ComputerAction::parse(action_raw) else {
        return Err(NtBotError::Invalid(format!("unknown computer action '{action_raw}'")));
    };
    Ok(ComputerCall {
        action,
        target: args
            .get("target")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_owned(),
        text: args
            .get("text")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_owned(),
    })
}

/// computer 后端接口 — 真实现 (浏览器/CDP) 按此 trait 接入.
pub trait ComputerBackend {
    fn backend_id(&self) -> &str;
    fn execute(&self, call: &ComputerCall) -> Result<String, NtBotError>;
}

/// 空后端 — 永远诚实失败, 不伪造任何屏幕/点击结果.
#[derive(Debug, Default)]
pub struct NoopBackend;

impl ComputerBackend for NoopBackend {
    fn backend_id(&self) -> &str {
        "noop"
    }

    fn execute(&self, call: &ComputerCall) -> Result<String, NtBotError> {
        Err(NtBotError::Engine {
            engine: "computer".to_owned(),
            reason: format!(
                "computer backend not configured; refused {} (target hidden)",
                call.action.as_str()
            ),
        })
    }
}

/// 动作回执三态（`computer-use` P7「防重放」吸收）。
///
/// **为什么要三态**：动作「发出去没有」不是两态。当回执是
/// `OutcomeUnknown`（已派发、效果未知）时，**任何自动重试都可能双击/双下单**
/// ⇒ 该态**必须**由上层决策（人），永不自动重试。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionOutcome {
    /// 已确认生效（后端回读过，或确定性 API 返回成功）。
    Applied,
    /// 已派发但效果未知 ⇒ **禁止自动重试**（P7 possibly_sent）。
    Unknown,
    /// 明确未生效（拒绝 / 参数错 / 连接失败）。
    Failed,
}

/// 上层处置建议（对应「Grok 错误即指令」的四档：换授权/换策略/停止/可重试）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryAdvice {
    /// 可原样重试（幂等失败：网络/连接层）。
    Retry,
    /// 需用户改授权/换目标再试（policy 拒绝、域禁区）。
    ChangeAuth,
    /// 应停止本轮（动作语义不支持、连续失败）。
    Stop,
    /// 效果未知 ⇒ 只能问人，禁止自动重试。
    AskHuman,
}

/// 一次动作的完整回执（动作面 → UI/账本）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ActionReceipt {
    pub action: String,
    /// 是否**已派发**给后端（决定是否可能重复生效）。
    pub sent: bool,
    pub outcome: ActionOutcome,
    pub advice: RetryAdvice,
    /// 人话原因（失败/未知时必填）。
    pub detail: String,
}

/// 把网关/后端错误映射成「三态 + 四档建议」。
///
/// 映射依据（`computer-use` P7 + 我方 `nt_cancel`/`tasks.outcome_unknown` 语义）：
/// - `Denied`（策略拒绝）⇒ **未派发** + `Failed` + `ChangeAuth`；
/// - `Invalid`（参数错）⇒ 未派发 + `Failed` + `Stop`；
/// - `Io`（传输层失败，**动作可能已发出**）⇒ `sent=true` + `Unknown` + `Retry`；
/// - `Engine{reason 含 refused/backend}` ⇒ 未派发 + `Failed` + `Stop`；
/// - 其它 ⇒ `Unknown` + `AskHuman`（**默认最保守**：宁可问人也不双击）。
#[must_use]
pub fn receipt_for_error(action: &str, err: &NtBotError) -> ActionReceipt {
    let (sent, outcome, advice, detail) = match err {
        NtBotError::Denied { rule, reason } => (
            false,
            ActionOutcome::Failed,
            RetryAdvice::ChangeAuth,
            format!("denied by '{rule}': {reason}"),
        ),
        NtBotError::Invalid(detail) => (false, ActionOutcome::Failed, RetryAdvice::Stop, detail.clone()),
        NtBotError::Io(detail) => (true, ActionOutcome::Unknown, RetryAdvice::Retry, detail.clone()),
        NtBotError::Engine { engine, reason } => {
            let refused = reason.contains("refused") || reason.contains("not configured");
            (
                !refused,
                if refused { ActionOutcome::Failed } else { ActionOutcome::Unknown },
                if refused { RetryAdvice::Stop } else { RetryAdvice::AskHuman },
                format!("{engine}: {reason}"),
            )
        }
        other => (
            true,
            ActionOutcome::Unknown,
            RetryAdvice::AskHuman,
            other.to_string(),
        ),
    };
    ActionReceipt {
        action: action.to_owned(),
        sent,
        outcome,
        advice,
        detail,
    }
}

/// **一次性控制租约**（`computer-use` P6 吸收；MiMo「1–20s 一次性租约」形状）。
///
/// 语义要点（三条，全部是事故驱动的）：
/// 1. **租约不自动续期**：到期即失效，需要人（或上层）重新授权 ——
///    「自动续期」等于把一次性授权变成永久授权，急停就失效了。
/// 2. **过期动作不回退成 Unknown**：租约过期发生在**派发之前**，属
///    `Failed` + `ChangeAuth`（重新授权即可），不是「可能已发出」。
/// 3. **上下界**：`1..=20` 秒。0 或 >20 直接拒收（0 = 永久授权的入口洞）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionLease {
    owner: String,
    expires_at_ms: i64,
}

/// 租约的秒数合法区间（MiMo 同款）。
pub const LEASE_MIN_SECS: i64 = 1;
/// 租约的秒数合法上界（MiMo 同款）。
pub const LEASE_MAX_SECS: i64 = 20;

impl ActionLease {
    /// 签发租约。秒数不在 `1..=20` ⇒ 拒收（`Invalid`），不静默夹取。
    pub fn grant(owner: &str, now_ms: i64, secs: i64) -> Result<Self, NtBotError> {
        if !(LEASE_MIN_SECS..=LEASE_MAX_SECS).contains(&secs) {
            return Err(NtBotError::Invalid(format!(
                "lease seconds must be {LEASE_MIN_SECS}..={LEASE_MAX_SECS}, got {secs}"
            )));
        }
        Ok(Self {
            owner: owner.to_owned(),
            expires_at_ms: now_ms + secs * 1000,
        })
    }

    /// 此刻是否仍持有控制权。
    #[must_use]
    pub fn is_valid(&self, now_ms: i64) -> bool {
        now_ms < self.expires_at_ms
    }

    /// 剩余毫秒（0 = 已过期）。
    #[must_use]
    pub fn remaining_ms(&self, now_ms: i64) -> i64 {
        (self.expires_at_ms - now_ms).max(0)
    }

    /// 持有者（审计面）。
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }
}

/// verify-after **回读三态**（`computer-use` P3/P7 吸收）。
///
/// 动作「成功返回」不等于「页面真的变了」；反之「读不到」也不等于「没变」。
/// ⇒ 三态必须分开，其中 `Drift` 是**停止信号**（继续动作只会越走越偏）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyOutcome {
    /// 回读与预期一致。
    Match,
    /// 回读与预期**不一致**（页面漂移/元素已不在）⇒ 停止并上报。
    Drift,
    /// 回读不可得（截图/DOM 读不到）⇒ 效果未知，问人。
    Unavailable,
}

/// verify-after 结果 → 处置建议。
///
/// - `Drift` ⇒ `Stop`（**不重试**：重试就是拿旧句柄继续点）；
/// - `Unavailable` ⇒ `AskHuman`（**不重试**：可能已生效）；
/// - `Match` ⇒ 无需补救（返回 `None` 表示「没有建议」）。
#[must_use]
pub fn verify_advice(outcome: VerifyOutcome) -> Option<RetryAdvice> {
    match outcome {
        VerifyOutcome::Match => None,
        VerifyOutcome::Drift => Some(RetryAdvice::Stop),
        VerifyOutcome::Unavailable => Some(RetryAdvice::AskHuman),
    }
}

/// **窄 CDP 传输口**（D 期：core ↔ neobot 的唯一接缝）。
///
/// 依赖方向是 **core → neobot**（core 依赖 neobot），所以
/// 「CDP 能力」必须由 neobot **先定义口**、core **后实现口** ——
/// 反过来做（neobot 依赖 core）会造环，也正是当初不写第二套浏览器栈的原因。
///
/// 口只有两个方法，是**刻意窄**的：CDP 的坑（target 生命周期、frame 漂移、
/// 连接复用、进程回收）全部留在实现侧；本 crate 只表达意图。
/// ⛔ 当前本仓**没有**注入 transport 的生产接线 ⇒ 默认仍走 [`NoopBackend`]
/// （诚实失败）；core 侧实现见 `docs/architecture/ABSORPTION-COMPUTER-USE-2026-10-08.md` §4-D。
pub trait CdpTransport {
    /// 执行一段 JS 并回传结果（同步，宿主负责超时）。
    fn evaluate(&self, script: &str) -> Result<String, NtBotError>;
    /// 当前页 URL（用于作用域/回读；实现拿不到就如实报 `None`）。
    fn current_url(&self) -> Option<String> {
        None
    }
}

/// 把动作编译成**确定性 JS**（不拼字符串进 eval 之外的通道）。
///
/// 返回 `(脚本, 该动作是否属于「写」动作)`。写动作 ⇒ 回执默认
/// `Unknown`（除非回读确认），因为「发出去没有」是三态。
fn compile_action(call: &ComputerCall) -> Result<(String, bool), NtBotError> {
    let q = |s: &str| -> String {
        serde_json::to_string(s).unwrap_or_else(|_| "\"".to_owned())
    };
    Ok(match call.action {
        ComputerAction::Navigate => {
            let url = q(&call.target);
            (format!("location.assign({url}); 'navigating'"), true)
        }
        ComputerAction::Click => {
            let sel = q(&call.target);
            (
                format!("(function(){{var e=document.querySelector({sel});if(!e)return 'no-element';e.click();return 'clicked';}})()"),
                true,
            )
        }
        ComputerAction::Type => {
            let sel = q(&call.target);
            let text = q(&call.text);
            (
                format!("(function(){{var e=document.querySelector({sel});if(!e)return 'no-element';e.focus();e.value={text};e.dispatchEvent(new Event('input',{{bubbles:true}}));return 'typed';}})()"),
                true,
            )
        }
        ComputerAction::Key => {
            let key = q(&call.text);
            (
                format!("(function(){{var e=new KeyboardEvent('keydown',{{key:{key},bubbles:true}}));document.activeElement&&document.activeElement.dispatchEvent(e);return 'keyed';}})()"),
                true,
            )
        }
        ComputerAction::Scroll => {
            let target = q(&call.target);
            (
                format!("(function(){{var e=document.querySelector({target})||document.scrollingElement;if(!e)return 'no-element';e.scrollTop=e.scrollHeight;return 'scrolled';}})()"),
                true,
            )
        }
        ComputerAction::Screenshot => ("'screenshot-not-supported-by-narrow-port'".to_owned(), false),
        // ⛔ 文件类动作**不属于**浏览器窄口：workspace 文件读写走 nt_workspace jail，
        //    在这里重做一遍就是造第二条无门路径（合规问题，不是能力问题）。
        ComputerAction::ReadFile | ComputerAction::WriteFile | ComputerAction::ListFiles => {
            return Err(NtBotError::Invalid(
                "narrow CDP port does not serve file actions; use nt_workspace".to_owned(),
            ));
        }
    })
}

/// CDP 后端：**只在宿主注入了 [`CdpTransport`] 时才存在**。
///
/// 与 [`NoopBackend`] 的区别不是「更聪明」，而是「真的接了线」——
/// 没有 transport 时**不允许**退化到 Noop 后假装能点。
#[derive(Debug, Clone)]
pub struct CdpBackend<T> {
    transport: T,
}

impl<T: CdpTransport> CdpBackend<T> {
    /// 用宿主 transport 构造。
    pub const fn new(transport: T) -> Self {
        Self { transport }
    }
}

impl<T: CdpTransport> ComputerBackend for CdpBackend<T> {
    fn backend_id(&self) -> &str {
        "cdp"
    }

    fn execute(&self, call: &ComputerCall) -> Result<String, NtBotError> {
        let (script, is_write) = compile_action(call)?;
        let out = self.transport.evaluate(&script)?;
        // 回读语义：写动作若回执不是明确成功词 ⇒ 效果未知（**不许重试**）。
        if is_write && !matches!(out.as_str(), "clicked" | "typed" | "navigating") {
            return Err(NtBotError::Io(format!(
                "cdp write returned non-confirmation: {}",
                crate::nt_computer::truncate_for_detail(&out)
            )));
        }
        // verify-after（**只有确认成功的写动作才回读**；读动作无须回读）：
        // 结果追加在输出尾部 `|verify=match|drift|unavailable`。
        // ⛔ 回读失败 ⇒ `unavailable`，**不**把动作改成失败（那会诱发重试）。
        let suffix = if is_write {
            match verify_after(&self.transport, call) {
                VerifyOutcome::Match => "|verify=match",
                VerifyOutcome::Drift => "|verify=drift",
                VerifyOutcome::Unavailable => "|verify=unavailable",
            }
        } else {
            ""
        };
        Ok(format!("{out}{suffix}"))
    }
}

/// verify-after 的**唯一执行点**：写动作之后回读目标是否还在。
///
/// 三态判据（`computer-use` P3）：`Match`=选择器仍命中；`Drift`=元素漂移⇒
/// 上层应 `Stop`；`Unavailable`=回读不可得⇒效果未知⇒`AskHuman`。
#[must_use]
pub fn verify_after<T: CdpTransport>(transport: &T, call: &ComputerCall) -> VerifyOutcome {
    if call.target.trim().is_empty() {
        return VerifyOutcome::Unavailable;
    }
    let probe = match serde_json::to_string(&call.target) {
        Ok(sel) => format!(
            "(function(){{try{{return document.querySelector({sel})?'present':'absent';}}catch(e){{return 'error';}}}})()"
        ),
        Err(_) => return VerifyOutcome::Unavailable,
    };
    match transport.evaluate(&probe) {
        Ok(v) if v.contains("present") => VerifyOutcome::Match,
        Ok(v) if v.contains("absent") => VerifyOutcome::Drift,
        _ => VerifyOutcome::Unavailable,
    }
}

/// 错误详情的定长截断（**不进 prompt 的原始 body**，防上下文炸）。
#[must_use]
pub fn truncate_for_detail(raw: &str) -> String {
    const LIMIT: usize = 120;
    raw.chars().take(LIMIT).collect()
}

/// 进程内**当前控制租约登记处**（P2：让 UI 能显示「还有几秒」）。
///
/// 语义边界（诚实声明）：
/// - 它**不是**跨进程/跨会话的授权系统，只是「本进程此刻谁持有控制权、到什么时候」；
/// - 不跨进程持久化 ⇒ 桌面/CLI 各自独立，**重启即失效**（这是有意的：租约要短命）；
/// - 真正的动作门控仍以 `ActionLease` 值为准，本登记处只是**同一份真相的可见面**。
static LEASE_REGISTRY: std::sync::OnceLock<std::sync::Mutex<Option<ActionLease>>> =
    std::sync::OnceLock::new();

fn lease_registry() -> &'static std::sync::Mutex<Option<ActionLease>> {
    LEASE_REGISTRY.get_or_init(|| std::sync::Mutex::new(None))
}

/// 现在的租约状态（给 UI/CLI 读）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LeaseStatus {
    pub active: bool,
    pub owner: String,
    /// 剩余毫秒（0 = 无租约或已过期）。
    pub remaining_ms: i64,
}

/// 签发并登记租约（越界秒数由 [`ActionLease::grant`] 拒收）。
pub fn register_lease(owner: &str, now_ms: i64, secs: i64) -> Result<LeaseStatus, NtBotError> {
    let lease = ActionLease::grant(owner, now_ms, secs)?;
    let remaining = lease.remaining_ms(now_ms);
    let Ok(mut slot) = lease_registry().lock() else {
        return Err(NtBotError::Store("lease registry lock poisoned".to_owned()));
    };
    *slot = Some(lease);
    Ok(LeaseStatus {
        active: true,
        owner: owner.to_owned(),
        remaining_ms: remaining,
    })
}

/// 撤销当前租约（急停的协议层另一半：翻令牌 + 撤租约）。
pub fn revoke_lease() {
    if let Ok(mut slot) = lease_registry().lock() {
        *slot = None;
    }
}

/// 读当前租约状态（过期即视为无租约，并顺手清位）。
pub fn lease_status(now_ms: i64) -> LeaseStatus {
    let Ok(mut slot) = lease_registry().lock() else {
        return LeaseStatus {
            active: false,
            owner: String::new(),
            remaining_ms: 0,
        };
    };
    match slot.as_ref() {
        Some(lease) if lease.is_valid(now_ms) => LeaseStatus {
            active: true,
            owner: lease.owner().to_owned(),
            remaining_ms: lease.remaining_ms(now_ms),
        },
        _ => {
            *slot = None;
            LeaseStatus {
                active: false,
                owner: String::new(),
                remaining_ms: 0,
            }
        }
    }
}

/// **元素句柄台账**（`computer-use` P3 吸收）。
///
/// 句柄失效是 CU/BU 的**第一大错误源**：模型拿着「42 号元素」去点，页面可能
/// 已经变了。本台账把三件事从「模型自觉」变成**协议层纪律**：
///
/// 1. **作用域显式**：每个 ref 绑定一个 `scope`（通常是 URL 规范化后的 host+
///    path 前缀）。作用域变了 ⇒ ref 失效，**不跨页复用**；
/// 2. **主动回收**：新一次观察（`begin_observation`）会把上一批未见过的 ref
///    全部作废 —— 「没被这一轮重新观察到」就等于**不存在**；
/// 3. **校验由知道真相的一侧做**：台账只回答「这个 ref 在当前观察里活着吗」，
///    **不**去猜描述是否还对（那是 observe 面的事）。
#[derive(Debug, Clone)]
pub struct ElementLedger {
    scope: String,
    observation: u64,
    refs: std::collections::BTreeMap<String, ElementRefEntry>,
}

#[derive(Debug, Clone)]
struct ElementRefEntry {
    selector: String,
    generation: u64,
}

/// 句柄解析结果（**三态**，不用 `Option` 压平）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefLookup {
    /// 命中当前观察内的活句柄。
    Live { selector: String },
    /// 句柄曾存在，但已被更新的观察淘汰（作用域相同）。
    Stale { selector: String },
    /// 从没见过（模型编的号，或另一页的号）。
    Unknown,
}

impl ElementLedger {
    /// 开一台账（作用域 = 规范化后的 target）。
    #[must_use]
    pub fn new(scope: &str) -> Self {
        Self {
            scope: scope.to_owned(),
            observation: 0,
            refs: std::collections::BTreeMap::new(),
        }
    }

    /// 作用域（当前绑定）。
    #[must_use]
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// 当前观察代号（每 `begin_observation` 递增一次）。
    #[must_use]
    pub fn observation(&self) -> u64 {
        self.observation
    }

    /// 开新一轮观察：**旧句柄全部作废**，返回被淘汰的数量。
    ///
    /// 这是「回收」纪律的执行点 —— 不依赖模型记得上一轮有什么。
    pub fn begin_observation(&mut self) -> u64 {
        self.observation = self.observation.saturating_add(1);
        let n = self.refs.len();
        self.refs.clear();
        self.observation
    }

    /// 切换作用域（跨页导航）：作废全部句柄。
    pub fn set_scope(&mut self, scope: &str) {
        if self.scope != scope {
            self.scope = scope.to_owned();
            self.begin_observation();
        }
    }

    /// 登记一次观察里看到的元素。
    pub fn observe(&mut self, handle: &str, selector: &str) {
        self.refs.insert(
            handle.to_owned(),
            ElementRefEntry {
                selector: selector.to_owned(),
                generation: self.observation,
            },
        );
    }

    /// 查句柄。
    #[must_use]
    pub fn lookup(&self, handle: &str) -> RefLookup {
        match self.refs.get(handle) {
            Some(entry) if entry.generation == self.observation => RefLookup::Live {
                selector: entry.selector.clone(),
            },
            Some(entry) => RefLookup::Stale {
                selector: entry.selector.clone(),
            },
            None => RefLookup::Unknown,
        }
    }

    /// 当前观察里的活句柄数。
    #[must_use]
    pub fn live_count(&self) -> usize {
        self.refs.len()
    }
}

/// 从 URL/类 URL 目标中提取 host (无 `url` 依赖的最小实现).
pub fn host_of(target: &str) -> Option<String> {
    let after_scheme = target.split_once("://").map(|(_, rest)| rest).unwrap_or(target);
    let host = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_end_matches('.');
    if host.is_empty() {
        return None;
    }
    Some(host.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{NoopBackend, ComputerBackend, host_of, parse_computer_call};

    #[test]
    fn parses_action() {
        let call = parse_computer_call(&serde_json::json!({
            "action": "navigate", "target": "https://example.com/a?b=1",
        }))
        .expect("parse");
        assert_eq!(call.action, super::ComputerAction::Navigate);
        assert_eq!(host_of(&call.target).as_deref(), Some("example.com"));
    }

    #[test]
    fn noop_never_executes() {
        let call = parse_computer_call(&serde_json::json!({"action": "click"})).expect("parse");
        assert!(NoopBackend.execute(&call).is_err());
    }

    #[test]
    fn unknown_action_rejected() {
        assert!(parse_computer_call(&serde_json::json!({"action": "rm"})).is_err());
    }

    #[test]
    fn receipt_denial_is_not_sent_and_asks_for_auth() {
        use crate::nt_error::NtBotError;
        use super::{receipt_for_error, ActionOutcome, RetryAdvice};
        let r = receipt_for_error(
            "click",
            &NtBotError::Denied { rule: "domain-ban".into(), reason: "target denied".into() },
        );
        assert!(!r.sent, "策略拒绝发生在派发之前");
        assert_eq!(r.outcome, ActionOutcome::Failed);
        assert_eq!(r.advice, RetryAdvice::ChangeAuth);
    }

    #[test]
    fn receipt_io_is_unknown_but_retryable() {
        use crate::nt_error::NtBotError;
        use super::{receipt_for_error, ActionOutcome, RetryAdvice};
        let r = receipt_for_error("type", &NtBotError::Io("connection reset".into()));
        assert!(r.sent, "传输层失败不能断言未派发");
        assert_eq!(r.outcome, ActionOutcome::Unknown);
        assert_eq!(r.advice, RetryAdvice::Retry, "幂等传输失败才可自动重试");
    }

    #[test]
    fn cdp_backend_writes_through_transport_and_refuses_fake_success() {
        use super::{CdpBackend, CdpTransport, ComputerBackend};
        use crate::nt_error::NtBotError;
        use std::cell::RefCell;

        struct Fake {
            calls: RefCell<Vec<String>>,
            reply: String,
        }
        impl CdpTransport for Fake {
            fn evaluate(&self, script: &str) -> Result<String, NtBotError> {
                self.calls.borrow_mut().push(script.to_owned());
                Ok(self.reply.clone())
            }
        }

        // 写动作 + 明确确认 ⇒ Ok
        let ok = CdpBackend::new(Fake { calls: RefCell::new(vec![]), reply: "clicked".into() });
        let call = parse_computer_call(&serde_json::json!({"action":"click","target":"#go"})).expect("parse");
        let out = ok.execute(&call).expect("exec");
        assert!(out.starts_with("clicked"), "{out}");
        assert!(out.contains("verify="), "写动作必须带回读三态: {out}");
        let logged = ok.transport.calls.borrow()[0].clone();
        assert!(logged.contains("querySelector(\"#go\")"), "选择器必须是 JSON 转义后的字面量: {logged}");

        // 写动作 + 非确认回执 ⇒ **Io 错**（上层映射成 Unknown ⇒ 禁止重试）
        let ambiguous = CdpBackend::new(Fake { calls: RefCell::new(vec![]), reply: "no-element".into() });
        let err = ambiguous.execute(&call).expect_err("no-element must not look like success");
        let receipt = super::receipt_for_error("click", &err);
        assert_eq!(receipt.outcome, super::ActionOutcome::Unknown, "非确认回执 ⇒ 效果未知");
        assert!(receipt.sent);
    }

    #[test]
    fn cdp_verify_after_appends_three_state_to_output() {
        use super::{CdpBackend, CdpTransport, ComputerBackend};
        use crate::nt_error::NtBotError;

        struct Probe {
            probe_reply: &'static str,
        }
        impl CdpTransport for Probe {
            fn evaluate(&self, script: &str) -> Result<String, NtBotError> {
                if script.contains("'present'") {
                    Ok(self.probe_reply.to_owned())
                } else {
                    Ok("clicked".to_owned())
                }
            }
        }

        let call = parse_computer_call(&serde_json::json!({"action":"click","target":"#go"})).expect("parse");
        for (reply, want) in [
            ("present", "|verify=match"),
            ("absent", "|verify=drift"),
            ("weird", "|verify=unavailable"),
        ] {
            let out = CdpBackend::new(Probe { probe_reply: reply })
                .execute(&call)
                .expect("exec");
            assert!(out.ends_with(want), "reply={reply} out={out}");
        }
        // 读动作不追加 verify 后缀
        let read = parse_computer_call(&serde_json::json!({"action":"screenshot"})).expect("parse");
        let out = CdpBackend::new(Probe { probe_reply: "present" })
            .execute(&read)
            .expect("exec");
        assert!(!out.contains("verify="), "{out}");
    }

    #[test]
    fn cdp_port_refuses_file_actions_instead_of_guessing() {
        use super::{CdpBackend, CdpTransport, ComputerBackend};
        use crate::nt_error::NtBotError;
        struct Never;
        impl CdpTransport for Never {
            fn evaluate(&self, _script: &str) -> Result<String, NtBotError> {
                panic!("file action must not reach the transport");
            }
        }
        let backend = CdpBackend::new(Never);
        for raw in ["read_file", "write_file", "list_files"] {
            let call = parse_computer_call(&serde_json::json!({"action": raw})).expect("parse");
            let err = backend.execute(&call).expect_err("file actions belong to nt_workspace");
            let receipt = super::receipt_for_error(raw, &err);
            assert!(!receipt.sent, "参数错 ⇒ 未派发");
            assert_eq!(receipt.advice, super::RetryAdvice::Stop);
        }
    }

    #[test]
    fn element_ledger_recycles_on_new_observation_and_scope_change() {
        use super::{ElementLedger, RefLookup};
        let mut ledger = ElementLedger::new("example.com/a");
        let first = ledger.begin_observation();
        ledger.observe("e1", "button.send");
        ledger.observe("e2", "input.to");
        assert_eq!(ledger.live_count(), 2);
        assert!(matches!(ledger.lookup("e1"), RefLookup::Live { .. }));

        // 新观察 ⇒ 上一轮没被重新观察到的句柄**全部作废**
        let second = ledger.begin_observation();
        assert_eq!(second, first + 1);
        assert!(matches!(ledger.lookup("e1"), RefLookup::Unknown));
        assert_eq!(ledger.live_count(), 0);

        // 编造的号永远 Unknown（不编 selector）
        assert_eq!(ledger.lookup("e99"), RefLookup::Unknown);

        // 换作用域 ⇒ 再次作废
        ledger.observe("e1", "a");
        ledger.set_scope("example.com/b");
        assert_eq!(ledger.live_count(), 0);
        assert_eq!(ledger.scope(), "example.com/b");
    }

    #[test]
    fn lease_registry_reports_countdown_and_revokes() {
        use super::{lease_status, register_lease, revoke_lease};
        let now = 2_000_000i64;
        revoke_lease();
        let st = register_lease("desktop", now, 5).expect("grant");
        assert!(st.active);
        assert_eq!(st.remaining_ms, 5000);
        assert_eq!(lease_status(now + 2500).remaining_ms, 2500);
        assert!(lease_status(now + 5000).active == false, "到点即失效并清位");
        register_lease("desktop", now, 5).expect("re-grant");
        revoke_lease();
        assert!(!lease_status(now).active, "撤销后立即无租约");
        assert!(register_lease("x", now, 0).is_err(), "0 秒仍拒收");
    }

    #[test]
    fn lease_bounds_are_enforced_and_never_auto_renew() {
        use super::{ActionLease, LEASE_MAX_SECS, LEASE_MIN_SECS};
        let now = 1_000_000i64;
        assert!(ActionLease::grant("me", now, 0).is_err(), "0 秒=永久授权的入口洞");
        assert!(ActionLease::grant("me", now, LEASE_MAX_SECS + 1).is_err());
        let lease = ActionLease::grant("me", now, LEASE_MAX_SECS).expect("grant");
        assert!(lease.is_valid(now));
        assert_eq!(lease.remaining_ms(now), LEASE_MAX_SECS * 1000);
        // 到点即失效，且**不**自动续期：同一时刻再问仍是无效。
        let after = now + LEASE_MAX_SECS * 1000;
        assert!(!lease.is_valid(after));
        assert_eq!(lease.remaining_ms(after), 0);
        assert_eq!(lease.owner(), "me");
        assert_eq!(LEASE_MIN_SECS, 1);
    }

    #[test]
    fn verify_drift_stops_and_unavailable_asks_human() {
        use super::{verify_advice, RetryAdvice, VerifyOutcome};
        assert_eq!(verify_advice(VerifyOutcome::Match), None);
        assert_eq!(verify_advice(VerifyOutcome::Drift), Some(RetryAdvice::Stop));
        assert_eq!(
            verify_advice(VerifyOutcome::Unavailable),
            Some(RetryAdvice::AskHuman),
            "读不到 ≠ 没生效 ⇒ 只能问人"
        );
    }

    #[test]
    fn receipt_backend_refused_is_failed_not_unknown() {
        use crate::nt_error::NtBotError;
        use super::{receipt_for_error, ActionOutcome, RetryAdvice};
        let call = parse_computer_call(&serde_json::json!({"action": "click"})).expect("parse");
        let err = NoopBackend.execute(&call).expect_err("noop refuses");
        let r = receipt_for_error(call.action.as_str(), &err);
        assert!(!r.sent);
        assert_eq!(r.outcome, ActionOutcome::Failed);
        assert_eq!(r.advice, RetryAdvice::Stop);
    }
}
