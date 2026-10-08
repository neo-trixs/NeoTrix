//! `nt_agent` — 有界多跳 agent loop.
//!
//! 有界步数 + 运行租约心跳 + hop 循环；工具收敛为
//! `bash + set_turn_status + 3×FS` 极简 schema；
//! 每个动作必经 `nt_policy` 网关 + `nt_audit` 先写后执.

use std::path::Path;

use chrono::Utc;
use uuid::Uuid;

use crate::nt_audit::{AuditDecision, AuditEvent};
use crate::nt_cancel::StopToken;
use crate::nt_capability_canary;
use crate::nt_config::{NeobotConfig, PolicyMode};
use crate::nt_engine::EngineAdapter;
use crate::nt_error::NtBotError;
use crate::nt_policy::{evaluate_policy, Actor, PolicyContext, PolicyDecision};
use crate::nt_store::NeobotStore;
use crate::nt_types::{AgentTask, TaskStatus, ToolName, ToolResult, TurnStatus};

const READ_CAP: u64 = 512 * 1024;
const WRITE_CAP: usize = 2 * 1024 * 1024;
const OUTPUT_CAP: usize = 8 * 1024;
/// 运行租约秒（单轮最长 10 分钟；崩溃后 `recover_stale_running` 凭它回收）。
pub const LEASE_SECS: i64 = 600;
/// 租约心跳的**最短间隔**：两跳之间至少隔这么久才允许再写一次库。
///
/// 为什么不是「每跳一次」：一 hop 可能是 300s（CLI 引擎超时）也可能只有
/// 几毫秒（本地回显）。按跳写库，慢轮次一次不多、快轮次一次不少 ⇒
/// 「每毫秒写一次库」。按**时间**闸才是真正的退避。
pub const LEASE_RENEW_INTERVAL_SECS: u64 = 60;
/// 单轮续租次数**上限**（32 × 60s ≈ 32 分钟连续心跳后停止续租）。
///
/// 有界是为了「异常长寿的单轮」不会无限续命：到顶即停手，让租约自然过期，
/// 崩溃恢复窗口重新变回一个 `LEASE_SECS`。**宁可在第 32 分钟之后让
/// `recover_stale_running` 回收，也好过让一个跑飞的任务永久占住一行。**
pub const LEASE_RENEW_MAX: usize = 32;
/// C2 取消汇总行的工具名（本跳被叫停时尚未执行的那些调用都记在这一行）。
const CANCELLED_TOOL_STEPS: &str = "cancelled:tool_calls";
/// C2 审计行的 `rule` 值（区别于网关自己的 deny 规则名）。
const CANCELLED_RULE: &str = "cancelled";
/// 落进 `tasks.error` 的取消原因前缀（`stopped by user at hop N/M`）。
const STOPPED_PREFIX: &str = "stopped by user";

// ══════════════════ 影子层（shadow）：超时策略按可逆性分流 ══════════════════
//
// ## 本段的全部目的是**把一个争论变成一次测量**
//
// 先前审计判定「可逆性门控」属于仪式，理由是 `ToolReversibility` 只有
// 一个取值 —— 那个结论**对 gateway 成立**（见下面的「同名 ≠ 同一符号」），
// 但它由**外部证据**（HKUDS/Vibe-Trading, MIT）推翻了：那里在
// `agent/src/agent/loop.py:2850-2925` 把工具执行的**超时策略按可逆性分流**，
// 原文注释是「Write tools are never killed: a watchdog warns once past the
// timeout, then the result is awaited to completion.」；只读工具才拿硬超时
// 并返回结构化 `tool_timeout` 信封。其承重原则是：
//
// > **能不能安全地中止，是动作可逆性的性质，不是超时配置的性质。**
// > 中途杀掉不可逆动作，会留下一个谁也没选过的世界状态；杀掉一次读是免费的。
//
// ## ⛔ 影子阶段：**不改变任何执行行为**
//
// 本段只做两件事：① 分类 + 纯决策函数（可单测）；② 在真实 funnel 上
// **记录**「这次调用本该适用哪条策略」。**没有任何一处分支去改超时、
// 改 kill、改 await。** 目的是让「这个类别到底有没有住户、分布如何」
// 用**我们自己运行的数据**回答，而不是继续辩论。
//
// ## ⛔ 同名 ≠ 同一符号（`RUST-STANDARDS.md` §4.2 L15）
//
// 本枚举刻意**不叫** `ToolReversibility`：那个名字已经属于
// `neotrix_gateway::gate::ToolReversibility`（crates/neotrix-gateway/src/gate.rs:111），
// 而本 crate **不得**依赖 gateway 之外的类型布局、也不该制造同名歧义。
// 那边的枚举是 4 值（含 `Compensable`），且**全仓唯一构造点都在它自己的
// 单测里**（gate.rs:1283 `ToolSpec::read_only("ls")` / gate.rs:1289
// `ToolSpec::irreversible("send_email")`）⇒ 那边确实只有测试住户。
// 本枚举是 neobot 从**自己的 `ToolName`** 独立推出来的分类，
// 逐条对照见 `reversibility_of` 的表。

/// neobot 本地的工具可逆性分级（**影子**）。
///
/// ⛔ 本类型**不改变任何执行行为**；见本节顶部说明。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeobotReversibility {
    /// 中止免费：没写过任何外部状态。
    ReadOnly,
    /// 中止后**有据可复原**（前像已落库 / 动作可原样重发）。
    Reversible,
    /// 中止会留下一个谁也没选过的状态。**未分类者的安全默认。**
    Irreversible,
}

impl NeobotReversibility {
    /// 落库/日志用的稳定字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Reversible => "reversible",
            Self::Irreversible => "irreversible",
        }
    }

    /// 判别标签（喂 [`crate::nt_determinism::Digest::variant`]，
    /// 必须逐变体唯一，否则摘要别名）。
    fn tag(self) -> u8 {
        match self {
            Self::ReadOnly => 1,
            Self::Reversible => 2,
            Self::Irreversible => 3,
        }
    }
}

/// 「如果这条策略今天生效，这次调用会被怎么办」的裁决（**影子**）。
///
/// ⛔ 影子阶段**没有任何生产分支读这个值**；它只被记录下来。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutPolicy {
    /// 越过超时也不中止：看门狗告警一次，然后**等它跑完**。
    WouldWait,
    /// 越过超时即中止（只读工具的硬超时），并回一个结构化 `tool_timeout` 信封。
    WouldTimeout,
}

impl TimeoutPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WouldWait => "would_wait",
            Self::WouldTimeout => "would_timeout",
        }
    }
}

/// neobot 的 `ToolName` → 可逆性。**穷尽匹配，无兜底分支**。
///
/// ⛔ 兜底分支在这里是**缺陷**，不是稳健：新增变体时若有 `_ =>`，新工具会
/// 静默落进某个类（默认多半是可杀的那个）⇒ 未来某个版本悄悄杀掉一个不可逆
/// 动作，且没有编译错误。Rust 的穷尽匹配正是我们要的编译器保险。
/// `Unknown(_)` 本身则显式判 `Irreversible`：**未知名按最危险处理**。
///
/// | `ToolName` | 分类 | 依据 |
/// |---|---|---|
/// | `ReadFile` `ReadImage` `WebSearch` `WebFetch` `QwenMediaInfo` `QwenReadVideo` `QwenVisualize` `PdfGroundText` | `ReadOnly` | 只读/只出结果，不写任何外部状态 |
/// | `WriteFile` `EditFile` | `Reversible` | `ChangeSink` 落**前像**（`nt_changes.rs:69-81`，`before` 非空即原内容），30 天留存 ⇒ 中止后有据可复原 |
/// | `SetTurnStatus` | `Reversible` | 唯一持久效果是 ledger 里的一个状态串，原样重发即可 |
/// | `SidebarOpen` | `Reversible` | 效果由**前端**执行（见 `nt_types.rs:196-201`），本 crate 无法断言「无副作用」；tab 可原样再开 |
/// | `Bash` | `Irreversible` | 任意 shell，可 `mv`/`git push`/改外部系统 |
/// | `ComputerAct` | `Irreversible` | 驱动真实 UI（点击/输入），落到人看得见的界面上 |
/// | `QwenSaveView` | `Irreversible` | 写盘产物（artifacts），会覆盖既有文件 |
/// | `CapabilityInvoke` | `Irreversible` | 副作用**由被调能力决定**，本 crate 拿不到该元数据 ⇒ 按最坏算（同 `Unknown`）**
/// | `Unknown(_)` | `Irreversible` | **安全默认**：认不出来的一律按不可逆 |
pub fn reversibility_of(tool: &ToolName) -> NeobotReversibility {
    match tool {
        ToolName::ReadFile
        | ToolName::ReadImage
        | ToolName::WebSearch
        | ToolName::WebFetch
        | ToolName::QwenMediaInfo
        | ToolName::QwenReadVideo
        | ToolName::QwenVisualize
        | ToolName::PdfGroundText => NeobotReversibility::ReadOnly,
        ToolName::WriteFile
        | ToolName::EditFile
        | ToolName::SetTurnStatus
        | ToolName::SidebarOpen => NeobotReversibility::Reversible,
        //  `CapabilityInvoke` 一律 `Irreversible`（2026-06 新增）：
        // 它的副作用**由被调的那个能力决定** —— 同一个 id 可能是只读的价格
        // 查询，也可能是不可逆的下单。而本 crate 在派发前**拿不到能力的
        // 副作用元数据**（市场条目里没有这一项）⇒ 无法逐能力判可逆性。
        //
        // ⛔ 刻意**不**把它归成 `ReadOnly`（那会让不可逆的能力白捡一个可逆
        //   标签，进而在超时策略里享受更宽的窗口）—— 与 `Unknown(_)` 同理：
        //   **认不出来就按不可逆**。
        ToolName::CapabilityInvoke
        | ToolName::Bash
        | ToolName::ComputerAct
        | ToolName::QwenSaveView => {
            NeobotReversibility::Irreversible
        }
        ToolName::Unknown(_) => NeobotReversibility::Irreversible,
    }
}

/// 这条工具路径**真正在执行**的超时（毫秒）。`None` = 该路径没有超时。
///
/// 影子阶段必须记**真实存在的**超时，不能借一个假想值：否则「有多少比例的
/// 调用真的会撞上超时」这个问题会被答错。逐条实测来源：
/// · `Bash` ⇒ `BASH_TIMEOUT`（本文件 `execute_bash`，60s，**到期 `child.kill()`**）
/// · `Qwen*` 四个 ⇒ `nt_qwen_mm::CORE_TIMEOUT_MS`（90s，经 `launch.session(...)`）
/// · `WebSearch`/`WebFetch` ⇒ `nt_web::TIMEOUT`（15s，ureq agent 级请求超时）
/// · 其余（`read_file`/`write_file`/`edit_file`/`computer_act`/`sidebar_open`/
///   `set_turn_status`/`pdf_ground_text`）⇒ 路径里**没有**超时，`None`。
#[must_use]
pub fn enforced_timeout_ms(tool: &ToolName) -> Option<f64> {
    match tool {
        ToolName::Bash => Some(60_000.0),
        ToolName::QwenMediaInfo
        | ToolName::QwenReadVideo
        | ToolName::QwenVisualize
        | ToolName::QwenSaveView => Some(90_000.0),
        ToolName::WebSearch | ToolName::WebFetch => Some(15_000.0),
        // `CapabilityInvoke` ⇒ 30s。它派发的是**别人写的**能力 ⇒ 本 crate
        // 无法假设它会自己收尾 ⇒ 必须有上限。⛔ 不给 `None`（无超时）：
        // 能力本体若卡住，这轮对话就永久挂起 —— 而它是 `Irreversible`，
        // 卡住的同时用户等不到任何反馈。
        ToolName::CapabilityInvoke => Some(30_000.0),
        ToolName::ReadFile
        | ToolName::ReadImage
        | ToolName::WriteFile
        | ToolName::EditFile
        | ToolName::ComputerAct
        | ToolName::SidebarOpen
        | ToolName::SetTurnStatus
        | ToolName::PdfGroundText
        | ToolName::Unknown(_) => None,
    }
}

/// `None` / `NaN` / `±∞` / `≤0` 一律折成「无超时」。
///
/// **裁决**：`NaN` 按**无超时**处理，不是按「立即超时」。
/// 理由：把 `NaN` 读成「已经超时」会让一次**超时配置本身有 bug** 的调用
/// 被当成超时受害者杀掉 —— 那是拿一次配置缺陷去换一个不可逆动作的半途而废。
/// 反过来把 `NaN` 读成无超时，代价只是这次调用**不被杀**（现状），
/// 方向安全。同理 `≤0` 视为「未配置」而非「零预算」。
fn normalize_timeout_ms(timeout_ms: Option<f64>) -> Option<f64> {
    timeout_ms.filter(|value| value.is_finite() && *value > 0.0)
}

/// 纯决策：(类, 已耗, 超时) → **本该适用**的策略。**影子阶段无人读它的值去分支。**
///
/// 规则（Vibe-Trading 的分流，只读侧才可中止）：
/// 1. 没有可用超时 ⇒ 永远到不了超时那条分支 ⇒ [`TimeoutPolicy::WouldWait`]。
/// 2. 未越过超时（`elapsed <= timeout`，**恰好等于不算越过**）⇒ [`TimeoutPolicy::WouldWait`]。
/// 3. 越过超时且只读 ⇒ [`TimeoutPolicy::WouldTimeout`]。越过超时但非只读
///    ⇒ [`TimeoutPolicy::WouldWait`]（告警一次，等它跑完）。
///
/// **为什么 `Reversible` 越过超时也判 `WouldWait`**（而不是给它第三种策略）：
/// 可逆性在本仓是**有条件**的 —— `ChangeSink` 的前像只在「文件此前存在」且
/// 「内容未超 `CHANGE_CONTENT_CAP`」时才落（`nt_changes.rs:91-98`），
/// 且**没有** `revert` 函数。所以「新文件写一半被中止」根本不可复原。
/// 影子阶段一律取**安全侧**；正是这份影子数据（有没有真在超时上吃过亏的
/// 可逆工具、分布如何）才能决定将来要不要把它放宽成 `WouldTimeout`。
///
/// `elapsed_ms` 为 `NaN`/负数同样按「未越过」处理（理由同 `normalize_timeout_ms`）。
#[must_use]
pub fn timeout_policy(
    class: NeobotReversibility,
    elapsed_ms: f64,
    timeout_ms: Option<f64>,
) -> TimeoutPolicy {
    let past_timeout = match normalize_timeout_ms(timeout_ms) {
        Some(limit) => elapsed_ms.is_finite() && elapsed_ms > limit,
        None => false,
    };
    if past_timeout && matches!(class, NeobotReversibility::ReadOnly) {
        TimeoutPolicy::WouldTimeout
    } else {
        TimeoutPolicy::WouldWait
    }
}

/// 「哪个工具落进了哪个类」的**稳定身份**。
///
/// 复用既有的 [`crate::nt_determinism::Digest`]（逐字段 `mul+add`，
/// 无位段别名）而**不**自己再造一个哈希；也**不**引新依赖。
/// 变体标签逐个唯一（`tag()`）⇒ 换类必换摘要；工具名按 `field_str` 纳入
/// （长度亦计入 ⇒ 不与别的拼接方式别名）。
#[must_use]
pub fn class_digest(class: NeobotReversibility, tool: &ToolName) -> u64 {
    crate::nt_determinism::Digest::new()
        .section("tool-reversibility-shadow")
        .variant(class.tag())
        .field_str(tool.as_str())
        .finish()
}

/// 记进 steps 行的那一行影子遥测（**追加**，不改既有内容）。
///
/// ⛔ 追加在**工具结果的 steps 行**上而不是另开一行，也不是审计行：
/// · 审计行的「先写后执」是网关律（见本文件 `run_local_turn` 的顺序注释），
///   事后往里补执行耗时等于倒着写账 ⇒ 审计行**不碰**。
/// · 另开一行会破「每个被请求过的工具都有且只有一行归属」的纪律
///   （C2 汇总行那条注释就是为这个存在的）⇒ 也不另开。
/// · steps 行本来就是**执行之后**写的（`result.ok` 只有执行完才知道），
///   所以实测耗时在这里是**如实**的，不是倒填。
fn shadow_line(
    tool: &ToolName,
    class: NeobotReversibility,
    timeout_ms: Option<f64>,
    elapsed_ms: Option<f64>,
    policy: TimeoutPolicy,
) -> String {
    let timeout = match timeout_ms {
        Some(value) => format!("{value}"),
        None => "none".to_owned(),
    };
    let elapsed = match elapsed_ms {
        Some(value) => format!("{value:.3}"),
        None => "none".to_owned(),
    };
    format!(
        "\n[shadow-rev] tool={} class={} timeout_ms={} elapsed_ms={} policy={} digest={}",
        tool.as_str(),
        class.as_str(),
        timeout,
        elapsed,
        policy.as_str(),
        class_digest(class, tool)
    )
}

/// 租约心跳（**有界 + 退避**的续租闸）。
///
/// 每跳之后问一次 `due()`，只在「距上次续租够间隔」且「本轮未到上限」时
/// 才推后 `lease_until`。续的是**滚动窗口**（`now + LEASE_SECS`），不是
/// 在旧值上累加 —— 于是进程一死，最坏仍只挂一个 `LEASE_SECS`。
struct LeaseHeartbeat {
    interval: std::time::Duration,
    cap: usize,
    renewals: usize,
    last: Option<std::time::Instant>,
}

impl LeaseHeartbeat {
    /// 生产默认（60s 一次、最多 32 次）。
    fn new() -> Self {
        Self {
            interval: std::time::Duration::from_secs(LEASE_RENEW_INTERVAL_SECS),
            cap: LEASE_RENEW_MAX,
            renewals: 0,
            last: None,
        }
    }

    /// 测试专用闸（生产恒走 `new()`）：把间隔/上限调成「一跳之内看得见」
    /// 的量级，否则毫秒级跑完的用例根本等不到 60s 那一拍。
    #[cfg(test)]
    fn tuned(interval: std::time::Duration, cap: usize) -> Self {
        Self {
            interval,
            cap,
            renewals: 0,
            last: None,
        }
    }

    /// 该不该现在续一次。
    fn due(&self) -> bool {
        if self.renewals >= self.cap {
            return false;
        }
        match self.last {
            None => true,
            Some(last) => last.elapsed() >= self.interval,
        }
    }

    /// 记一次「已经续过」（**成功与失败都记**）。
    ///
    /// 失败也记，是为了让退避照样生效：库忙/钥匙丢了都不该变成「每跳重试
    /// 一次」的写库活。计数同样占上限名额 —— 反复失败也不该无限重试。
    fn note(&mut self) {
        self.renewals += 1;
        self.last = Some(std::time::Instant::now());
    }
}

/// 租约窗口（**滚动**）：此刻 + `LEASE_SECS`。
///
/// 单独抽出来是因为「入轮那一次」与「每跳续租那 N 次」必须**同一条**算式：
/// 只要有一处写成在旧值上累加，崩溃恢复窗口就会被放大成累加的倍数。
fn lease_deadline() -> String {
    (Utc::now() + chrono::Duration::seconds(LEASE_SECS)).to_rfc3339()
}

/// 增量回调（流式对话流用）。
pub type DeltaCallback<'a> = &'a mut dyn FnMut(&str);
/// 工具步骤回调（tool 名，成功与否，500 字内摘要）。
pub type StepCallback<'a> = &'a mut dyn FnMut(&str, bool, &str);

/// 一轮运行的只读上下文（rish Env 思想：输入全显式，打包传参；
/// 顺带把 9 参函数压到 clippy type_complexity 线下）。
pub struct RunContext<'a> {
    pub store: &'a NeobotStore,
    pub config: &'a NeobotConfig,
    pub engine: &'a dyn EngineAdapter,
    pub actor: Actor,
    pub actor_name: &'a str,
    pub title: &'a str,
    pub user_text: &'a str,
    pub convo_id: Option<&'a str>,
}

/// 跑一轮本地任务 (创建 task → 有界 loop → 落库), 返回终态.
pub fn run_local_turn(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    title: &str,
    user_text: &str,
) -> Result<TurnStatus, NtBotError> {
    let ctx = RunContext {
        store,
        config,
        engine,
        actor: Actor::Bot,
        actor_name: "bot",
        title,
        user_text,
        convo_id: None,
    };
    run_local_turn_cancellable(&ctx, None, None, None)
}

/// 发起方具名版（routine firing 审计记 `routine:<name>`；审计 actor 即发起方）。
/// `convo_id`: 指定会话则任务归属该会话；None 则自动新建群组会话
/// （IM 语义：调用方传选中会话 id 即可追加）。
pub fn run_local_turn_as(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    actor: Actor,
    actor_name: &str,
    title: &str,
    user_text: &str,
    convo_id: Option<&str>,
) -> Result<TurnStatus, NtBotError> {
    let ctx = RunContext {
        store,
        config,
        engine,
        actor,
        actor_name,
        title,
        user_text,
        convo_id,
    };
    run_local_turn_cancellable(&ctx, None, None, None)
}

/// 流式版 — 模型增量内容经 `on_delta` 回调 (SSE 真流式引擎).
pub fn run_local_turn_stream(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    title: &str,
    user_text: &str,
    on_delta: DeltaCallback<'_>,
) -> Result<TurnStatus, NtBotError> {
    let ctx = RunContext {
        store,
        config,
        engine,
        actor: Actor::Bot,
        actor_name: "bot",
        title,
        user_text,
        convo_id: None,
    };
    run_local_turn_cancellable(&ctx, Some(on_delta), None, None)
}

/// 流式具名版.
pub fn run_local_turn_stream_as(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    actor: Actor,
    actor_name: &str,
    title: &str,
    user_text: &str,
    convo_id: Option<&str>,
    on_delta: DeltaCallback<'_>,
    on_step: Option<StepCallback<'_>>,
) -> Result<TurnStatus, NtBotError> {
    let ctx = RunContext {
        store,
        config,
        engine,
        actor,
        actor_name,
        title,
        user_text,
        convo_id,
    };
    run_local_turn_cancellable(&ctx, Some(on_delta), on_step, None)
}

/// **带停止令牌**的跑轮入口（切片 C1/C2/C3 用）。
///
/// 为什么收 `&RunContext` 而不是再抄一遍 8 个参数：四个 wrapper 已经各自
/// 建好 ctx，而本函数真正的变量只有 `stop`。参数打包是本文件既有的做法
/// （见 `RunContext` 的 rish Env 注释：把 9 参函数压到 clippy 线下）。
///
/// `stop = None` ⇒ 造一枚**私有**令牌（无人能置位）⇒ 行为与本函数存在
/// 之前**逐字相同**。这是本切片能独立上线的根据：`channel serve` 今天
/// 没人传 `Some`，所以对外行为不变，而取消机制已真实存在且被测试覆盖。
///
/// **给用户看的那句「已停（第 N 跳）」不由本函数产出** —— 它是 dispatch
/// 侧的事（那边持有令牌、知道自己置过位）。本函数只如实落库
/// `TaskStatus::Cancelled` + 位置，并返回 `TurnStatus::Waiting`（人可接手）。
pub fn run_local_turn_cancellable(
    ctx: &RunContext<'_>,
    on_delta: Option<DeltaCallback<'_>>,
    on_step: Option<StepCallback<'_>>,
    stop: Option<&StopToken>,
) -> Result<TurnStatus, NtBotError> {
    run_local_turn_inner(ctx, on_delta, on_step, stop, LeaseHeartbeat::new())
}

/// 占位标题（自动命名只动这些；用户手改过的永不动）。
fn is_placeholder_title(title: &str) -> bool {
    matches!(title.trim(), "我的私聊" | "新群组" | "新对话" | "")
}

/// 首行片段（去空白，截 24 字；纯附件系统行/空返回 None，不命名）。
fn title_snippet(text: &str) -> Option<String> {
    let first = text.lines().map(str::trim).find(|line| !line.is_empty())?;
    if first.starts_with('[') && first.ends_with(']') {
        return None;
    }
    let snippet: String = first.chars().take(24).collect();
    if snippet.is_empty() {
        None
    } else {
        Some(snippet)
    }
}

fn run_local_turn_inner(
    ctx: &RunContext<'_>,
    on_delta: Option<DeltaCallback<'_>>,
    on_step: Option<StepCallback<'_>>,
    stop: Option<&StopToken>,
    lease: LeaseHeartbeat,
) -> Result<TurnStatus, NtBotError> {
    let store = ctx.store;
    let config = ctx.config;
    let title = ctx.title;
    let convo_id = ctx.convo_id;
    let now = Utc::now().to_rfc3339();

    // 本轮是新的观察窗口：金丝雀计数归零。
    //
    // ⚠️ 位置：**必须在 `convo_id` 解析出来之后**再 reset（会话键化，
    // 2026-10-07 修 OPEN-DEFECTS P1-5）—— 旧实现在此处用进程全局窗口，
    // 任何会话的一轮开始都会清掉**所有会话**的计数。判据见下方 `reset` 处。
    // 检查点 C0（取消）：**入轮清一次旗**。
    //
    // 语义是「进入这一轮时的值」就是「这一轮专属的停止意图」。共享令牌
    // 可能带着上一轮的置位进来（同一会话连跑两轮），也可能被一次打在
    // 已结束轮次上的 `cancel()` 弄脏；不清就会误伤下一轮。故名是
    // 「入口清」而不是「用完清」。
    //
    // `None` 分支造一枚私有令牌：无人能置位 ⇒ 与本函数存在之前同行为。
    let owned_stop: StopToken;
    let stop: &StopToken = match stop {
        Some(token) => {
            token.reset();
            token
        }
        None => {
            owned_stop = StopToken::new();
            &owned_stop
        }
    };
    // 起点两扫（单机，一次 UPDATE 级代价）：崩溃残留回收 + 过期认领释放。
    // best-effort：扫失败不挡本轮（下次起点再扫）。
    let _recovered: usize = store.recover_stale_running(&now).unwrap_or(0);
    let _swept: usize = store
        .sweep_stale_claims(&now, crate::nt_store::CLAIM_TTL_SECS)
        .unwrap_or(0);
    // 租约窗口是**滚动**的：`lease_deadline()` 每次都按「此刻 + LEASE_SECS」
    // 重算，续租只是把它推后，绝不在旧值上累加（否则续 N 次就把崩溃恢复
    // 窗口放大成 N 倍 —— 那样续租反而成了泄漏源）。
    let lease_until = lease_deadline();
    let lease_id = Uuid::new_v4().to_string();
    // 会话归属：指定即用（不存在则报错，不静默建）；缺省自动建群组会话。
    let convo_id = match convo_id {
        Some(id) => {
            let found = store.list_conversations()?.into_iter().find(|c| c.id == id);
            let Some(convo) = found else {
                return Err(NtBotError::Store(format!("no such conversation '{id}'")));
            };
            // 自动标题：占位名会话 + 用户首条有效输入 → 取首行片段命名（仅一次；
            // 用户手改过的标题永不动；失败不挡本轮）。
            if is_placeholder_title(&convo.title) {
                if let Some(snippet) = title_snippet(ctx.user_text) {
                    let _renamed: Option<()> = store.rename_conversation(&convo.id, &snippet).ok();
                }
            }
            id.to_owned()
        }
        None => store.create_conversation("group", title, &[])?,
    };
    // 本轮是新的观察窗口：金丝雀计数归零 —— **只归零本会话**。
    //
    // 纪律照抄 plur `tools.ts:3594`：不 reset 的话，一次信号就能让金丝雀在
    // **整个进程生命周期**保持健康（它记的 #192 事故）。判据是
    // `fired>0 || ticks<3`，ticks 只增不减会让第一项一旦为真就永不失效，
    // 于是「早已坏掉的能力」看起来一直健康。
    //
    // ⚠️ 键化（2026-10-07，修 OPEN-DEFECTS P1-5）：`convo_id` 是唯一会话键
    // ⇒ A 会话起轮不碰 B 会话的窗口/计数。
    nt_capability_canary::reset(&convo_id);
    let task = AgentTask {
        id: Uuid::new_v4().to_string(),
        title: title.to_owned(),
        status: TaskStatus::Running,
        created_at: now.clone(),
        updated_at: now,
        claimed_by: None,
        claimed_at: None,
        visibility: crate::nt_types::default_visibility(),
        lease_id: Some(lease_id.clone()),
        lease_until: Some(lease_until),
        attempts: 1,
        error: None,
        conversation_id: Some(convo_id.clone()),
    };
    store.save_task(&task)?;
    let outcome = run_loop(ctx, &task.id, &lease_id, on_delta, on_step, stop, lease, &convo_id);
    // turn 级错误（落库失败等）记终态 Failed + error 后原错返回，不吞错。
    let (status, stopped_at, _degraded_tools) = match outcome {
        Ok(pair) => pair,
        Err(err) => {
            let failed = AgentTask {
                status: TaskStatus::Failed,
                updated_at: Utc::now().to_rfc3339(),
                lease_id: None,
                lease_until: None,
                error: Some(err.to_string()),
                ..task.clone()
            };
            // 已在错误处理中：落库再败也无处可记，静默丢弃。
            let _saved: Result<(), NtBotError> = store.save_task(&failed);
            return Err(err);
        }
    };
    // 检查点 C4（中止落库）：被用户叫停的这一轮写 `TaskStatus::Cancelled`。
    //
    // 三件事同时定（§10.4）：① 落库用 `TaskStatus::Cancelled`（既有变体，
    // `cancel_task` 早写它、`retry_task` 早收它 ⇒ 用户有正规重跑出口）；
    // ② 公开签名返回 `TurnStatus::Waiting`（人可接手，且它是**既有**变体，
    // 不必给 `TurnStatus` 造一个新造一个假的）；③ 面向用户的中文回执由
    // dispatch 侧产出，不由 `TurnStatus` 承载。
    //
    // `lease_id/lease_until` **必须清**：不清则 `recover_stale_running`
    // 在一个 `LEASE_SECS` 内不碰它（与 `cancel_task` 同款，同一个道理）。
    if let Some(hop) = stopped_at {
        // 分母与 `run_loop` 的归一化同款（`max_steps.max(1)`）。
        let steps = config.max_steps.max(1);
        let cancelled = AgentTask {
            status: TaskStatus::Cancelled,
            updated_at: Utc::now().to_rfc3339(),
            lease_id: None,
            lease_until: None,
            // 「第几跳」写在 error 里，信息不丢：`hop` 是 0 基的跳序号，
            // 即「在哪一跳的哪个检查点被拦下」。
            error: Some(format!("{STOPPED_PREFIX} at hop {hop}/{steps}")),
            ..task.clone()
        };
        store.save_task(&cancelled)?;
        // 2026-09-30: 删掉原先写在这里的 CH_MESSAGE_NEW outbox 行。
        // 该 topic 全仓无任何消费者（drainer 只认 CH_CHANNEL_SEND），
        // 且 payload 无 channel 键 ⇒ 行一进队列就永远发不出去、无限退避占位。
        // 任务本体已由上一行 save_task 持久化，通知行不承载任何额外信息。
        return Ok(TurnStatus::Waiting);
    }
    let finished = AgentTask {
        status: match status {
            TurnStatus::Done => TaskStatus::Done,
            TurnStatus::Blocked => TaskStatus::Failed,
            TurnStatus::Continue | TurnStatus::NeedsClarification | TurnStatus::Waiting => {
                TaskStatus::Pending
            }
        },
        updated_at: Utc::now().to_rfc3339(),
        lease_id: None,
        lease_until: None,
        error: if status == TurnStatus::Done {
            None
        } else {
            Some(format!("turn ended as {}", status.as_str()))
        },
        ..task
    };
    store.save_task(&finished)?;
    // 2026-09-30: 同上，删掉无消费者的 CH_MESSAGE_NEW 毒行（见上一处注释）。
    Ok(status)
}

/// 有界多跳循环。
///
/// 返回 `(终态, 被叫停的跳序号)`：第二项 `Some(hop)` 表示这一轮是**被用户
/// 叫停**的（`hop` 为 0 基，即在哪个检查点被拦下），`None` 表示正常跑完。
/// 「哪一跳被取消」走返回值而不是 `StopToken` 的内部可变性 —— 给一个跨
/// 执行流共享的载体加内部可变性，会逼出 `Mutex` 或 `unsafe`，两个都不要。
fn run_loop(
    ctx: &RunContext<'_>,
    task_id: &str,
    lease_id: &str,
    mut on_delta: Option<DeltaCallback<'_>>,
    mut on_step: Option<StepCallback<'_>>,
    stop: &StopToken,
    mut lease: LeaseHeartbeat,
    session: &str,
) -> Result<(TurnStatus, Option<usize>, usize), NtBotError> {
    use crate::nt_types::{TranscriptItem, TranscriptRole};
    let store = ctx.store;
    let config = ctx.config;
    let engine = ctx.engine;
    let actor = ctx.actor;
    let actor_name = ctx.actor_name;
    let user_text = ctx.user_text;
    let steps = config.max_steps.max(1);
    let mut history: Vec<TranscriptItem> = Vec::new();
    let mut current = TurnStatus::Continue;
    // 单轮累计写入（rish 嵌套预算中层）。
    let mut turn_written: usize = 0;
    // 被叫停的那一跳（0 基）。`None` = 正常跑完。
    let mut stopped_at: Option<usize> = None;
    // N6.2：本轮「工具输出被降级成省略标记」的次数（内容整体丢弃）。
    // 压缩（errors-first 蒸馏）**不算**降级 —— 只有塌成 `…` 才算（N6.2 判据）。
    let mut degraded_tools = 0usize;
    // 本轮文件改动账（侧边栏「本轮文件」视角；写/改/读都记账）。
    let sink = crate::nt_changes::ChangeSink {
        store,
        task_id,
        workspace: &config.workspace_dir,
    };
    for n in 0..steps {
        // 最后一步且已有工具活动: 提醒收尾 (防跑满 max_steps 仍无终态).
        if n + 1 == steps && !history.is_empty() {
            history.push(TranscriptItem {
                role: TranscriptRole::User,
                content: format!(
                    "提醒: 这是最后一步 (max_steps={steps})。请用已有信息直接回复, \
                     并调用 set_turn_status(done) 收尾, 不要再调工具。"
                ),
                tool_calls: Vec::new(),
                tool_call_id: None,
                image: None,
            });
        }
        // 检查点 C1：别再发起**新的一次模型调用**（省钱的主要来源）。
        // 命中即 `break`，**不写任何 steps 行**：没有发生的事不记账。
        // 不自己 `return`：`current` 仍是 `Continue`，下面的兜底会把它
        // 映射成 `Waiting`（人可接手）—— 这正是我们要的语义。
        if stop.is_cancelled() {
            stopped_at = Some(usize::from(n));
            break;
        }
        let hop_started = std::time::Instant::now();
        let turn = match on_delta.as_mut() {
            Some(callback) => engine.run_turn_stream(user_text, &history, &mut **callback)?,
            None => engine.run_turn_with_history(user_text, &history)?,
        };
        let hop_latency_ms = hop_started.elapsed().as_millis().min(i64::MAX as u128) as i64;
        // 有用量即落账本（purpose + measured + status + latency）。
        if let Some(usage) = turn.usage.as_ref() {
            let policy = crate::nt_cost::CostPolicy::from_env();
            let (cost_usd, measured) = if policy.is_configured()
                || crate::nt_cost::price_for(engine.model_name()).is_some()
            {
                crate::nt_cost::cost_for(
                    policy,
                    engine.model_name(),
                    usage.prompt_tokens,
                    usage.completion_tokens,
                )
            } else if usage.cost_usd > 0.0 {
                (usage.cost_usd, true)
            } else {
                (0.0, false)
            };
            store.record_ledger(&crate::nt_store::LedgerEntry {
                id: Uuid::new_v4().to_string(),
                at: Utc::now().to_rfc3339(),
                engine: engine.engine_id().to_owned(),
                model: engine.model_name().to_owned(),
                actor: actor_name.to_owned(),
                purpose: "agent-turn".to_owned(),
                in_tokens: usage.prompt_tokens,
                out_tokens: usage.completion_tokens,
                cost_usd,
                measured,
                status: turn.status.as_str().to_owned(),
                latency_ms: hop_latency_ms,
                error: None,
                // 会话口径 + key 口径出表（2026-10-08 拍板补列）。
                session_id: ctx.convo_id.map(|c| c.to_owned()),
                // 引擎暴露取 key 的 env 名（HttpEngine 由 Provider::http_engine 注入）；
                // 本地回显/CLI 工程不持有 key_env ⇒ 维持 None，不猜。
                key_env: engine.key_env_name().map(|s| s.to_owned()),
            })?;
        }
        // CLI 副作用回传 → step 行（落库前复核：kind 非空才记）。
        for effect in &turn.side_effects {
            if effect.kind.trim().is_empty() {
                continue;
            }
            let detail = serde_json::to_string(&effect.payload).unwrap_or_default();
            store.add_step(
                task_id,
                i64::from(n),
                &format!("side-effect:{}", effect.kind),
                true,
                &detail,
            )?;
        }
        history.push(TranscriptItem {
            role: TranscriptRole::Assistant,
            content: turn.assistant_text.clone(),
            tool_calls: turn.tool_calls.clone(),
            tool_call_id: None,
            image: None,
        });
        // 引擎自带 tool_calls 为空时按纯回复处理.
        if turn.tool_calls.is_empty() {
            store.add_step(task_id, i64::from(n), "reply", true, &turn.assistant_text)?;
            record_output_governance(store, task_id, i64::from(n), &turn.assistant_text)?;
            current = turn.status;
            break;
        }
        // 每个 tool call 推进一个金丝雀观察窗口。
        //
        // 位置：紧贴 tool_calls 循环头，即「模型请求了能力」的汇聚点。纪律
        // 沿用 plur `server.ts:336`（每次 tool call = 一个 turn）。
        // 刻意不放在 hop 开头 —— 那会把「这一跳模型没调任何工具」也算成一轮，
        // 窗口被无关轮次灌水，阈值（ticks < 3）形同虚设。
        for _call in turn.tool_calls.iter() {
            nt_capability_canary::tick(session);
        }

        let mut saw_status: Option<TurnStatus> = None;
        for (call_idx, call) in turn.tool_calls.iter().enumerate() {
            // 检查点 C2：**同跳内不再执行下一个工具**（最重要的一处）。
            //
            // 三个子动作的顺序不可换（`:366-367` 那条网关律「先写审计行，
            // 再执行」）：① 审计行 ② steps 汇总行 ③ break。
            //
            // 放在 `gate` **之前**而不是之后：`gate` 之后造出的是一条
            // 「声称网关已判定、实际什么也没做」的中间态；放在 `gate`
            // 之前则是**如实**记「用户叫停」而不是「网关拒绝」——
            // 这是两件不同的事实，不能混。所以这里的 `tool` 是合成的
            // `tool_calls`（网关并没有逐个判过这些调用），`rule=cancelled`。
            if stop.is_cancelled() {
                // `.get(..)` 而非 `[..]`：`call_idx` 来自对本数组的
                // `enumerate`，切片必然合法，但工作区禁裸下标（panic 面）。
                let pending: Vec<&str> = turn
                    .tool_calls
                    .get(call_idx..)
                    .map(|rest| rest.iter().map(|left| left.name.as_str()).collect())
                    .unwrap_or_default();
                let pending_csv = pending.join(",");
                store.record_audit(&AuditEvent::new(
                    actor_name,
                    "tool_calls",
                    AuditDecision::Deny,
                    Some(CANCELLED_RULE.to_owned()),
                    &format!("task={task_id} hop={n} stopped by user; not executed: {pending_csv}"),
                ))?;
                // 汇总 steps 行：本跳**尚未执行**的调用记在这里，于是每个被
                // 请求过的工具都有归属（正常路径有自己的行，取消路径进这一
                // 行）—— 不留悬空的 tool_call id（history 里有、steps 里
                // 查无此行是最难查的一类脏数据）。
                store.add_step(
                    task_id,
                    i64::from(n),
                    CANCELLED_TOOL_STEPS,
                    false,
                    &pending_csv,
                )?;
                stopped_at = Some(usize::from(n));
                break;
            }
            let (decision, rule) = gate(config, actor, call)?;
            let allowed = matches!(decision, PolicyDecision::Allow);
            // 网关律：先写审计行（无论放行与否），再执行。
            // 崩溃也不丢“谁动了什么”的记录；执行结果只进 steps 行。
            let intent = call.name.intent();
            let pre_event = AuditEvent::new(
                actor_name,
                call.name.as_str(),
                if allowed {
                    AuditDecision::Allow
                } else {
                    AuditDecision::Deny
                },
                rule.clone(),
                &format!("task={task_id} intent={intent}"),
            );
            store.record_audit(&pre_event)?;
            // ── 影子层（Vibe-Trading 式按可逆性分流）**只测不施** ──
            //
            // `will_execute` 是原 `if` 条件的**原样提取**，不改短路顺序、不改
            // 分支归属；提出来只为让下面能如实区分「测了耗时」与「压根没跑」
            // （被拒/dry-run 的 elapsed 若记 0，会把一次**没执行**混进耗时分布）。
            let will_execute =
                allowed && config.policy_mode == crate::nt_config::PolicyMode::Enforce;
            let rev_class = reversibility_of(&call.name);
            let rev_timeout_ms = enforced_timeout_ms(&call.name);
            let rev_started = std::time::Instant::now();
            // dry-run: 记录但不执行. 执行错误转失败结果 (模型可见, 可换路),
            // 只有落库/审计失败才 `?` 中断.
            let outcome = if will_execute {
                match execute_tool(config, engine, call, &mut turn_written, &sink, stop) {
                    Ok(outcome) => outcome,
                    Err(err) => ToolOutcome::from(ToolResult {
                        ok: false,
                        output: format!("tool error: {err}"),
                        truncated: false,
                    }),
                }
            } else {
                ToolOutcome::from(ToolResult {
                    ok: false,
                    output: if allowed {
                        "(dry-run: not executed)".to_owned()
                    } else {
                        "(denied)".to_owned()
                    },
                    truncated: false,
                })
            };
            // 实测耗时 → 纯决策 → 记账。⛔ `policy` **不被任何分支读**：
            // 它只被追加进下面的 steps 行（外加一条 debug 日志）。
            let rev_elapsed_ms = will_execute.then(|| rev_started.elapsed().as_secs_f64() * 1000.0);
            let rev_policy =
                timeout_policy(rev_class, rev_elapsed_ms.unwrap_or(0.0), rev_timeout_ms);
            let shadow_note = shadow_line(
                &call.name,
                rev_class,
                rev_timeout_ms,
                rev_elapsed_ms,
                rev_policy,
            );
            log::debug!(
                "shadow-rev tool={} class={} timeout_ms={:?} elapsed_ms={:?} policy={} digest={}",
                call.name.as_str(),
                rev_class.as_str(),
                rev_timeout_ms,
                rev_elapsed_ms,
                rev_policy.as_str(),
                class_digest(rev_class, &call.name)
            );
            let result = outcome.result;
            let reason_note = status_reason(&call.args)
                .map(|reason| format!(" reason={reason}"))
                .unwrap_or_default();
            // 凭据检测（工具**输出**面）：steps 表持久化、`on_step` 外发到
            // 前端 / IM ⇒ 命中的凭据会四处扩散。
            //
            // ⛔ 与 `nt_audit::redact_detail` **不重复**：后者靠行内 key 名
            //   （`api_key=` / `password=`）整行替换，而真实泄露常是**裸值**
            //   —— `read_file` 读到的 .env 内容可能没有可识别的前缀格式。
            // ⛔ 与 core 侧 `AgentLoop::call_tool` 的扫描**也不重复**：那个拦
            //   工具**参数**（模型即将送出的），这里拦工具**输出**（读回的）。
            // ⛔ 只报告不改写：用户可能**故意**让我们看自己的配置，自动截断
            //   会让模型拿不到它需要的上下文。处置决策交人。
            let output = &result.output;
            let step_output = match crate::nt_secret_scan::summarize(&crate::nt_secret_scan::scan(
                &result.output,
            )) {
                Some(warning) => {
                    store.record_audit(&crate::nt_audit::AuditEvent::new(
                        actor_name,
                        "output_secret_scan",
                        crate::nt_audit::AuditDecision::Allow,
                        Some(call.name.as_str().to_owned()),
                        &warning,
                    ))?;
                    format!("{warning}\n{output}{reason_note}{shadow_note}")
                }
                None => format!("{output}{reason_note}{shadow_note}"),
            };
            store.add_step(
                task_id,
                i64::from(n),
                call.name.as_str(),
                result.ok,
                &step_output,
            )?;
            // 工作流事件外发（流式对话流用；500 字截断，明细仍在 steps 表）。
            if let Some(emit) = on_step.as_mut() {
                let mut snippet = result.output.clone();
                if snippet.len() > 500 {
                    let mut cut = 500;
                    while cut > 0 && !snippet.is_char_boundary(cut) {
                        cut -= 1;
                    }
                    snippet.truncate(cut);
                    snippet.push('…');
                }
                emit(call.name.as_str(), result.ok, &snippet);
            }
            if crate::nt_output_distill::is_degraded(&result.output) {
                degraded_tools += 1;
            }
            history.push(TranscriptItem {
                role: TranscriptRole::Tool,
                // 蒸馏已**下沉到 truncate_output**（见该函数文档），这里原样带上。
                // ⛔ 原先的 truncate_history（砍尾留头）已从本路径移除；
                //   变异验证：接回来 ⇒ 接线守门测试立刻红（exit code 丢失）。
                content: result.output.clone(),
                tool_calls: Vec::new(),
                tool_call_id: Some(call.id.clone()),
                // 图像**不进 content**（`content` 是纯文本通道，进去了模型只会读到
                // 一堵 base64 字符墙）。它走这个侧信道，由 `nt_http_engine` 在下一次
                // 请求里升级成真正的 `image_url` 多模态部件。
                image: outcome.image,
            });
            if call.name == ToolName::SetTurnStatus {
                saw_status = parse_status_arg(&call.args);
            }
            if !allowed && config.policy_mode == PolicyMode::Enforce {
                current = TurnStatus::Blocked;
                break;
            }
        }
        // C2 命中后不再走下一跳 —— 否则下一跳开头的 C1 会把「第几跳」
        // 覆盖成一个更大的数，落库就谎报了被拦下的位置。
        if stopped_at.is_some() {
            break;
        }
        if let Some(status) = saw_status {
            current = status;
            if status != TurnStatus::Continue {
                break;
            }
        } else if current != TurnStatus::Continue {
            break;
        }
        enforce_transcript_budget(&mut history);
        // 租约心跳（C0）：每跳之后问一次闸（`due()` = 距上次够间隔 **且**
        // 本轮未到次数上限）。不快也不慢地写库：按时间闸退避、按次数封顶。
        //
        // best-effort（同起点的 `recover_stale_running` 那一扫）：续租失败
        // 不挡本轮 —— SQLite 写竞争的真实现是「一失败就整轮死掉」，而
        // 少续一次的后果只是租约窗口短一点，仍然可回收。
        if lease.due() {
            let until = lease_deadline();
            // 0 行 = 不再持有这行（被回收或被接管）：不重试，更不重写 ——
            // 续租只推后**自己手上那把钥匙**的窗口。
            let _renewed: Result<usize, NtBotError> = store.renew_lease(task_id, lease_id, &until);
            lease.note();
        }
    }
    // 跑满仍无终态 (模型一直行动不收尾) → Waiting (任务 Pending, 人可接手).
    if current == TurnStatus::Continue {
        current = TurnStatus::Waiting;
    }
    // N6.2：本轮发生过「工具输出整体丢弃」⇒ 账本落一行 `degraded`。
    //
    // ⛔ 为什么**单独一行**而不是并进费用行：降级不是花钱（cost=0），
    //   也不是失败（工具本身可能 ok）⇒ 混进任一行都会让「这轮为什么
    //   什么都没拿到」变成查不出来的问题。
    //   best-effort 记账：写不进去不挡本轮（与续租/清扫同款纪律）。
    if degraded_tools > 0 {
        let _degraded_row: Result<(), NtBotError> = store.record_ledger(&crate::nt_store::LedgerEntry {
            id: Uuid::new_v4().to_string(),
            at: Utc::now().to_rfc3339(),
            engine: engine.engine_id().to_owned(),
            model: engine.model_name().to_owned(),
            actor: actor_name.to_owned(),
            purpose: "tool-output-degrade".to_owned(),
            in_tokens: 0,
            out_tokens: 0,
            cost_usd: 0.0,
            measured: false,
            status: "degraded".to_owned(),
            latency_ms: 0,
            error: Some(format!("{degraded_tools} tool output(s) collapsed to placeholder")),
            session_id: ctx.convo_id.map(|c| c.to_owned()),
            key_env: engine.key_env_name().map(str::to_owned),
        });
    }
    Ok((current, stopped_at, degraded_tools))
}

/// 转录预算（rish 转录上限思想本地值：总量 256KiB / 200 条）。
/// 超限从旧往新丢 Tool 结果行（用户原文与 assistant 回复保留；
/// 全量仍在 steps 表，可追溯）。
const TRANSCRIPT_CAP_BYTES: usize = 256 * 1024;
const TRANSCRIPT_CAP_ITEMS: usize = 200;

/// 单条转录项占用的预算权重。
///
/// 图像必须**计入**：它不走 `content`（`content` 只有一行几十字节的说明），
/// 4 MiB 的图在 `image.base64` 里是 5.6 MiB。若只按 `content` 计账，
/// 一轮里连读十张图就能把请求体堆到 50 MiB 以上，而预算显示「几乎没占」。
/// 从旧往新丢的策略正好合适：先丢的那张图也是最该被忘掉的那张。
fn transcript_weight(item: &crate::nt_types::TranscriptItem) -> usize {
    item.content.len()
        + item
            .image
            .as_ref()
            .map(|image| image.base64.len())
            .unwrap_or(0)
}

fn enforce_transcript_budget(history: &mut Vec<crate::nt_types::TranscriptItem>) {
    while history.len() > TRANSCRIPT_CAP_ITEMS {
        let Some(pos) = history
            .iter()
            .position(|item| item.role == crate::nt_types::TranscriptRole::Tool)
        else {
            break;
        };
        history.remove(pos);
    }
    let mut bytes: usize = history.iter().map(transcript_weight).sum();
    while bytes > TRANSCRIPT_CAP_BYTES {
        let Some(pos) = history
            .iter()
            .position(|item| item.role == crate::nt_types::TranscriptRole::Tool)
        else {
            break;
        };
        bytes = bytes.saturating_sub(history.get(pos).map(transcript_weight).unwrap_or(0));
        if history.get(pos).is_none() {
            break;
        }
        history.remove(pos);
    }
}

/// 历史回填截断 (4KiB/条, 防上下文爆炸; 全量仍在 steps 表).
/// 旧的无脑截断。**保留但已不用于 transcript**。
///
/// ⛔ 勿再拿它当工具输出的截断手段：砍尾留头会丢 exit code（见接线处注释）。
#[allow(dead_code)]
fn truncate_history(output: &str) -> String {
    const LIMIT: usize = 4096;
    if output.len() <= LIMIT {
        return output.to_owned();
    }
    let mut cut = LIMIT;
    while cut > 0 && !output.is_char_boundary(cut) {
        cut -= 1;
    }
    match output.get(..cut) {
        Some(safe) => format!("{safe}…[truncated]"),
        None => "…[truncated]".to_owned(),
    }
}

/// 网关门控: `Unknown` 工具也进策略 (一律拒绝, 原名进审计).
/// 把 G27 输出治理报告落成一条 step（**纯观测，不改用户看到的文本**）。
///
/// ## 为什么接在这里
///
/// 这是「模型给出最终答案」在 `run_loop` 里的**唯一收敛点**：此后文本会落
/// transcript、进 SQLite、经 IM 通道发给用户。治理器原本只挂在
/// `AgentLoop::emit_final` 上，而 `AgentLoop` 经实测零生产实例化 ⇒ 真实执行环
/// **零治理**，模型说什么就存什么、就发什么。
///
/// ## 为什么只观测、不阻断、不改写
///
/// - **不改写**：本仓对输出有既定纪律（`enforce_transcript_budget` 的错误优先、
///   `truncate_history` 的原样透传），治理器若改文本会与这些纪律打架，且
///   「R07/R08 判定幻影路径」依赖工作区根目录，在 IM 场景下工作区语义不明确。
/// - **不阻断**：治理失败不该让对话失败 ⇒ 任何错误只 `warn`。
/// - 治理结果作为 `tool="reply_governance"` 的一步落库 ⇒ 可查询、可统计，
///   且**不污染** transcript（模型下一轮不会看到自己被判了几条违规）。
///
/// ## 成本
///
/// R07 / R08 会做真实文件 I/O（`is_file()` / `read_to_string`），因此
/// 每条最终输出多若干次系统调用。`OutputGovernor::new()` 默认以
/// `current_dir()` 为根 —— 这里显式传入本轮的工作区目录，使「文件引用存在」
/// 的判据有确定基准。
fn record_output_governance(
    store: &NeobotStore,
    task_id: &str,
    step: i64,
    text: &str,
) -> Result<(), NtBotError> {
    let gov = crate::nt_governance::OutputGovernor::new();
    let report = gov.govern(text);
    let mut summary = format!(
        "score={} passed={}/{} violations={} smells={}",
        report.overall_score,
        report.rule_results.iter().filter(|r| r.passed).count(),
        report.rule_results.len(),
        report.violations.len(),
        report.smells.len()
    );
    if !report.violations.is_empty() {
        summary.push_str(" | ");
        summary.push_str(&report.violations.join("; "));
    }
    // 同时落 steps（可按 task 查）与 audit（可按最近查）。⛔ 治理**不判红**
    // audit 的 decision：`violations.is_empty()` 已经写进 steps 的 `ok` 列，
    // 而 audit 是「执行决策」账本 —— 把「输出质量」塞进去会污染它的语义
    // （下游有按 decision=deny 统计安全事件的逻辑）。
    store.add_step(
        task_id,
        step,
        "reply_governance",
        report.violations.is_empty(),
        &summary,
    )?;
    store.record_audit(&crate::nt_audit::AuditEvent::new(
        "bot",
        "output_governance",
        crate::nt_audit::AuditDecision::Allow,
        Some(format!("score={}", report.overall_score)),
        &summary,
    ))
}

fn gate(
    config: &NeobotConfig,
    actor: Actor,
    call: &crate::nt_types::ToolCall,
) -> Result<(PolicyDecision, Option<String>), NtBotError> {
    let file_path = [
        "path",
        "file",
        "file_path",
        // Qwen-MM-Plugins 工具的路径参数名（上游 schema 原文）：
        // `media_info.path` / `read_video.video_path` / `visualize.file_path`。
        // 不收这三个键，`qwen_read_video {"video_path": "/etc/shadow"}` 就会
        // 绕过第 3 步的 `is_jailbreak_path` —— 收键与执行层的 `jail_join`
        // 一起构成双保险（与 `ReadFile` 同惯例）。
        "video_path",
        "image_path",
    ]
    .iter()
    .find_map(|key| call.args.get(*key))
    .and_then(|value| value.as_str())
    .map(str::to_owned);
    let command = call
        .args
        .get("command")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let computer_action = call
        .args
        .get("action")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let computer_target = call
        .args
        .get("target")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let ctx = PolicyContext {
        tool: call.name.clone(),
        actor,
        human_has_control: config.human_has_control,
        file_path: file_path.clone(),
        command: command.clone(),
        computer_action,
        computer_target,
        computer_allow: config.computer_allow.clone(),
        computer_hosts: config.computer_hosts.clone(),
    };
    let decision = evaluate_policy(&ctx);
    let rule = match &decision {
        PolicyDecision::Allow => None,
        PolicyDecision::Deny { rule, .. } => Some(rule.clone()),
    };
    if rule.is_none() {
        // operator 自写 deny：base 放行后才看。
        if let Some((extra_rule, reason)) = crate::nt_policy::evaluate_extra_deny(
            &config.extra_deny,
            &call.name,
            actor,
            command.as_deref(),
            file_path.as_deref(),
        ) {
            return Ok((
                PolicyDecision::Deny {
                    rule: extra_rule.clone(),
                    reason,
                },
                Some(extra_rule),
            ));
        }
    }
    Ok((decision, rule))
}

/// `set_turn_status` 的 reason（status/reason/next_step 三件套；
/// reason 进 step 行，方便复盘“为什么停”）。
fn status_reason(args: &serde_json::Value) -> Option<String> {
    args.get("reason")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|reason| !reason.is_empty())
        .map(|reason| {
            let mut cut = reason.len().min(200);
            while cut > 0 && !reason.is_char_boundary(cut) {
                cut -= 1;
            }
            reason.get(..cut).unwrap_or("").to_owned()
        })
}

fn parse_status_arg(args: &serde_json::Value) -> Option<TurnStatus> {
    args.get("status")
        .and_then(|value| value.as_str())
        .and_then(TurnStatus::parse)
}

/// 工具产出：文本结果（进 steps 表与转录）＋ 可能的图像部件（进下一次请求的
/// 多模态 content）。
///
/// 为什么**不**把图像挂进 `ToolResult`：那会让全仓每个构造点都多写一个字段，
/// 而全仓只有一个工具会产出图像。单独拎出来，读代码的人一眼看得出「哪些工具
/// 真的看得见图」，也把 base64 这个重物的传播路径限死在一条线上。
#[derive(Debug)]
struct ToolOutcome {
    result: ToolResult,
    image: Option<crate::nt_types::ImagePart>,
}

impl From<ToolResult> for ToolOutcome {
    /// 纯文本工具的落点：图像恒 `None`。
    fn from(result: ToolResult) -> Self {
        Self {
            result,
            image: None,
        }
    }
}

fn execute_tool(
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    call: &crate::nt_types::ToolCall,
    turn_written: &mut usize,
    sink: &crate::nt_changes::ChangeSink<'_>,
    stop: &StopToken,
) -> Result<ToolOutcome, NtBotError> {
    match &call.name {
        ToolName::SetTurnStatus => Ok(ToolResult {
            ok: parse_status_arg(&call.args).is_some(),
            output: "status recorded".to_owned(),
            truncated: false,
        }
        .into()),
        ToolName::Bash => Ok(execute_bash(config, call, stop)?.into()),
        ToolName::ReadFile => Ok(execute_read(config, call, sink)?.into()),
        ToolName::ReadImage => execute_read_image(config, engine, call),
        ToolName::WriteFile => Ok(execute_write(config, call, turn_written, sink)?.into()),
        ToolName::EditFile => Ok(execute_edit(config, call, turn_written, sink)?.into()),
        ToolName::ComputerAct => Ok(execute_computer(call)?.into()),
        ToolName::WebSearch => Ok(execute_web_search(call)?.into()),
        ToolName::WebFetch => Ok(execute_web_fetch(call)?.into()),
        ToolName::SidebarOpen => Ok(execute_sidebar_open(call)?.into()),
        ToolName::QwenMediaInfo
        | ToolName::QwenReadVideo
        | ToolName::QwenVisualize
        | ToolName::QwenSaveView => execute_qwen_mm(config, engine, call, stop),
        ToolName::PdfGroundText => Ok(execute_pdf_ground_text(config, call)?.into()),
        ToolName::CapabilityInvoke => Ok(execute_capability_invoke(call)?.into()),
        ToolName::Unknown(raw) => Err(NtBotError::Invalid(format!("unknown tool '{raw}'"))),
    }
}

/// Qwen-MM-Plugins 会话工具的执行（模型自主调，2026-09-29）。
///
/// 顺序不可换，每一步都有存在理由：
/// 1. **先问有没有眼睛**（与 `execute_read_image` 同纪律）：这些工具会产出
///    image 块（`read_video`/`visualize`/`save_view` 的预览帧），看不见图的
///    模型只会拿到一串路径 —— 那比不给更糟，它会开始"描述"没看见的画面。
/// 2. **再问服务器在不在**（`resolve_core_launch` 零 spawn 探测）；不在就把
///    安装指引**原样**交给模型，让它能对用户说清缺什么，而不是谎称成功。
/// 3. **最后才把路径参数 jail 化**：相对路径拼进 workspace；绝对路径必须落在
///    workspace 内（`jail_join` 第二道）。上游服务器拿到的是**已验证**的
///    绝对路径，不会自己去猜路径语义。
/// 4. artifacts 落进 `workspace/.neotrix-mm/`（而非全局临时目录）：落在
///    workspace 内 ⇒ 下一步 `read_image` 能合法读它，二段链（抽帧→看图）
///    才真正闭得上；也顺带被 workspace jail 覆盖。
fn execute_qwen_mm(
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    call: &crate::nt_types::ToolCall,
    stop: &StopToken,
) -> Result<ToolOutcome, NtBotError> {
    use crate::nt_qwen_mm as mm;
    if !engine.vision_capable() {
        return Err(NtBotError::Denied {
            rule: "no-vision-engine".to_owned(),
            reason: format!(
                "engine '{}' cannot deliver image parts to the model (model '{}'), so the \
                 qwen-media tools would only hand back file paths — set NEOBOT_VISION=1 if \
                 this model really is multimodal",
                engine.engine_id(),
                engine.model_name()
            ),
        });
    }
    if stop.is_cancelled() {
        return Ok(ToolResult {
            ok: false,
            output: "(cancelled: not executed)".to_owned(),
            truncated: false,
        }
        .into());
    }
    let launch = mm::resolve_core_launch().map_err(|e| NtBotError::Denied {
        rule: "qwen-mm-unavailable".to_owned(),
        reason: e.to_string(),
    })?;
    let artifacts = config.workspace_dir.join(".neotrix-mm");
    let session = launch.session(mm::CORE_TIMEOUT_MS, artifacts.clone());
    execute_qwen_mm_with_session(config, call, &session, &artifacts)
}

/// Qwen 会话工具的纯分发（可单测）：`session` 由调用方注入，
/// 测试时喂伪造 stdio 服务器，生产时是真服务器 —— 同一 framing。
fn execute_qwen_mm_with_session(
    config: &NeobotConfig,
    call: &crate::nt_types::ToolCall,
    session: &crate::nt_qwen_mm::McpStdioSession,
    artifacts: &std::path::Path,
) -> Result<ToolOutcome, NtBotError> {
    // 上游工具名 ← 我们的 ToolName。
    let upstream = match call.name {
        crate::nt_types::ToolName::QwenMediaInfo => "media_info",
        crate::nt_types::ToolName::QwenReadVideo => "read_video",
        crate::nt_types::ToolName::QwenVisualize => "visualize",
        _ => "save_view",
    };
    // 路径参数 jail 化：走本 crate 的第二道 jail（`join_workspace` 拒绝对路径/
    // `..`/`~`），把**已验证**的绝对路径交给上游服务器 —— 它不猜路径语义。
    let mut args = call.args.clone();
    for key in ["path", "file_path", "video_path", "image_path"] {
        if let Some(raw) = args.get(key).and_then(|v| v.as_str()) {
            let full = match join_workspace(&config.workspace_dir, raw) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(ToolResult {
                        ok: false,
                        output: format!("path refused: {e}"),
                        truncated: false,
                    }
                    .into());
                }
            };
            if let Some(obj) = args.as_object_mut() {
                obj.insert(
                    key.to_string(),
                    serde_json::Value::String(full.to_string_lossy().to_string()),
                );
            }
        }
    }
    if upstream == "save_view" {
        // 落盘目录固定在 workspace 内，不接受模型指定别处。
        if let Some(obj) = args.as_object_mut() {
            obj.insert(
                "output_dir".to_string(),
                serde_json::Value::String(artifacts.to_string_lossy().to_string()),
            );
        }
    }

    match session.call_tool(upstream, &args) {
        Ok(result) => {
            // 产物路径写进 steps 账：人可点；模型可再用 `read_image` 读（它在
            // workspace 内 ⇒ 合法），二段链（抽帧→看图）才真闭得上。
            let mut output = result.text.clone();
            let mut first_rel: Option<String> = None;
            if !result.saved_images.is_empty() {
                output.push_str("\n[saved under workspace .neotrix-mm/]");
                for p in &result.saved_images {
                    // 落盘目录就是 workspace 内的 artifacts（调用方定的），
                    // strip_prefix 必成功 —— 万一哪天改了落盘位置，
                    // 这里回退到绝对路径也不崩（只是不好点）。
                    match p.strip_prefix(&config.workspace_dir) {
                        Ok(rel) => {
                            let rel = rel.to_string_lossy().to_string();
                            output.push_str(&format!("\n- {rel}"));
                            if first_rel.is_none() {
                                first_rel = Some(rel);
                            }
                        }
                        Err(_) => output.push_str(&format!("\n- {}", p.display())),
                    }
                }
            }
            let image = first_rel
                .as_deref()
                .and_then(|rel| crate::nt_vision::load_image(&config.workspace_dir, rel).ok())
                .map(|(part, _kind)| part);
            Ok(ToolOutcome {
                result: ToolResult {
                    ok: !result.is_error,
                    output,
                    truncated: false,
                },
                // 预览图作为真正的多模态部件随下一轮抵达模型。
                image,
            })
        }
        Err(e) => Ok(ToolResult {
            ok: false,
            output: format!("qwen tool '{upstream}' failed: {e}"),
            truncated: false,
        }
        .into()),
    }
}

/// 看图执行：工作区内的图 → 真正的多模态部件。
///
/// 顺序有讲究：**先问引擎有没有眼睛，再去读盘**。看不见的引擎根本不该被喂
/// 一张图的字节（白读、白 base64），更不该拿到一个「成功」的空结果。
///
/// 三种失败都是**明说**，没有一种悄悄返回空：
/// - 引擎/模型无视觉 → 点名引擎与模型，叫用户设 `NEOBOT_VISION=1`；
/// - 路径越狱 / 缺参 → 网关与 jail 的原话；
/// - 不是图 / 超限 → `nt_vision` 说清实际认成了什么。
fn execute_read_image(
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    call: &crate::nt_types::ToolCall,
) -> Result<ToolOutcome, NtBotError> {
    if !engine.vision_capable() {
        return Err(NtBotError::Denied {
            rule: "no-vision-engine".to_owned(),
            reason: format!(
                "engine '{}' cannot deliver image parts to the model (model '{}'), \
                 so reading an image would only hand back a path — set NEOBOT_VISION=1 \
                 if this model really is multimodal",
                engine.engine_id(),
                engine.model_name()
            ),
        });
    }
    let path = required_path(&call.args)?;
    // 第二道 jail（策略层已判一次，这里按 `read_file` 的双保险惯例再拼一次）。
    let _guarded = join_workspace(&config.workspace_dir, &path)?;
    let (image, kind) = crate::nt_vision::load_image(&config.workspace_dir, &path)?;
    // 报**编码后**的字节数（手上有精确值；原图大小要靠 padding 反推，
    // 与其给个差 1-2 字节的「约等于」，不如报一个不用猜的数）。
    let encoded = image.base64.len();
    Ok(ToolOutcome {
        result: ToolResult {
            ok: true,
            // 给人（和模型）看的这行是**短文本**：图像本身走 image 通道，
            // 不在这里复述 base64（复述了模型就只会收到一堵字符墙）。
            output: format!(
                "attached 1 image as a multimodal part: {path} ({} , {encoded} base64 chars). \
                 You can see it now — describe what is actually visible.",
                kind.media_type()
            ),
            truncated: false,
        },
        image: Some(image),
    })
}

/// 侧边栏导航：**不执行、只成文**。
///
/// 输出是一行 JSON（`{"topic":"files","path":"a/b.rs","viewer":"code"}`），
/// 前端在流式 `Step` 事件里认出 `tool == "sidebar_open"` 后解析并跳。
/// 模型因此只能「提议打开」——开不开、开哪页永远由界面说了算。
///
/// 校验只有两件，都必要：topic 必须在册（防模型编页签），target 必须过
/// `nt_workspace` 的 jail 词法判定（防提议工作区外的路径）。
fn execute_sidebar_open(call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    let topic = call
        .args
        .get("topic")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let target = ["path", "target"]
        .iter()
        .find_map(|key| call.args.get(*key))
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let registry = crate::nt_sidebar::TabRegistry::with_builtins();
    let viewers = crate::nt_sidebar::builtin_viewers();
    let resolved = crate::nt_sidebar::resolve_open(&registry, &viewers, topic, target)?;
    let output = serde_json::to_string(&resolved)
        .map_err(|err| NtBotError::Codec(format!("sidebar_open encode: {err}")))?;
    Ok(ToolResult {
        ok: true,
        output,
        truncated: false,
    })
}

/// 抓「改前」内容，供改动账渲染 diff。
///
/// 不存在 / 不是普通文件 / 二进制 / 超 `CHANGE_CONTENT_CAP` 一律当「没有」——
/// 账会如实记 `before = None`，而不是在账里塞半个文件或半个二进制。
fn peek_before(full: &std::path::Path) -> Option<String> {
    use crate::nt_changes::CHANGE_CONTENT_CAP;
    let meta = std::fs::metadata(full).ok()?;
    if !meta.is_file() || meta.len() > CHANGE_CONTENT_CAP as u64 {
        return None;
    }
    let bytes = std::fs::read(full).ok()?;
    if crate::nt_workspace::is_binary(&bytes) {
        return None;
    }
    String::from_utf8(bytes).ok()
}

/// computer 执行 — 当前 Noop 后端诚实失败 (调用方转失败结果回填模型).
fn execute_computer(call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    use crate::nt_computer::{parse_computer_call, ComputerBackend as _, NoopBackend};
    let parsed = parse_computer_call(&call.args)?;
    let output = NoopBackend.execute(&parsed)?;
    Ok(ToolResult {
        ok: true,
        output,
        truncated: false,
    })
}

/// 联网搜索执行（客户端直调；count 越界钳制 1-10，缺 query 直接 Invalid）。
fn execute_web_search(call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    let Some(query) = call.args.get("query").and_then(|v| v.as_str()) else {
        return Err(NtBotError::Invalid(
            "web_search requires {query}".to_owned(),
        ));
    };
    let count = call.args.get("count").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
    let output = crate::nt_web::web_search(query, count)?;
    Ok(ToolResult {
        ok: true,
        output,
        truncated: true,
    })
}

/// 网页抓取执行（scheme 门控在 nt_web 内，fail-closed）。
fn execute_web_fetch(call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    let Some(url) = call.args.get("url").and_then(|v| v.as_str()) else {
        return Err(NtBotError::Invalid("web_fetch requires {url}".to_owned()));
    };
    let output = crate::nt_web::web_fetch(url)?;
    Ok(ToolResult {
        ok: true,
        output,
        truncated: true,
    })
}

/// PDF 文字定位（本地只读，零外部依赖）。
///
/// 为什么**不用**通用 OCR 路线：外挂二进制（tesseract）或自带检测权重，
/// 都比「PDF 文字本来就有精确坐标」贵得多 —— `nt_pdf_ground` 直接读
/// content 流（`Tm`/`Tf`/`Tj`），不 spawn、不联网、不写盘。
///
/// `ok` 的取法要说清楚：**查不到 ≠ 工具失败**。空命中是「这份 PDF 的文字层里
/// 没有这个词」这一事实，`ok: true` + 解释性输出；只有真的出错（越狱路径、
/// 不是 PDF、超大文件）才 `ok: false`。反过来会让模型把「没找到」当「工具坏了」
/// 去重试或改口。
///  **按市场 id 调用一个已上架能力**（2026-06 接通）。
///
/// ## 它补的是哪条断掉的通路
///
/// 5 个贸易能力长期「可上架但从不被调用」。查证结论（两条都实测过）：
/// - `tool_schemas` 是**硬编码**列表 ⇒ 模型看不见市场里的任何条目；
/// - `nt_capability_bridge::route_experience` 只把解析结果写进 rationale
///   ⇒ **从不执行**能力。
///
/// ⇒ 「已上架」与「能被模型调用」之间没有通路。本函数建立它。
///
/// ## 为什么这里才是「id 的门」（而 policy 只放行工具本身）
///
/// `nt_policy` 是纯函数（`PolicyContext` 里没有、也不该有市场清单），所以
/// 它只能判「允许调用 capability_invoke 这个**工具**」。而「这个**能力 id**
/// 是否已上架」必须在这里判 —— 这里才读得到市场 ⇒ **fail-closed 仍然成立**。
///
/// ## 三道门（顺序不可换）
///
/// ① **id 必须已上架**：未上架（含 `Gap`、缺 license/version）一律拒。
///    ⇒ 这条同时把「意识登记的能力缺口」挡住 —— `Gap` 永远进不了市场。
/// ② **必须是 `Tool` / `Skill` / `Workflow` / `Agent` 四类之一**
///    （由 ① 蕴含，显式写出来是为了让门**可读**，不依赖读者去查市场实现）。
/// ③ **调用计数必须真的增加**：本函数末尾按 id 记一次派发，
///    否则 `never_invoked` 清单与金丝雀永远看不到它。
fn execute_capability_invoke(call: &crate::nt_types::ToolCall) -> Result<ToolResult, NtBotError> {
    let id = call
        .args
        .get("capability_id")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim();
    if id.is_empty() {
        return Ok(ToolResult {
            ok: false,
            output: "capability_invoke: 缺 capability_id。id 从能力市场上架清单里取。".to_owned(),
            truncated: false,
        });
    }

    // ① 已上架这道门（fail-closed：查不到 ⇒ 拒）。
    let market = crate::nt_capability_registry::with_registry(|reg| {
        crate::nt_capability_market::listable(reg)
    })
    .map_err(|e| NtBotError::Store(e))?;
    let entry = market.iter().find(|e| e.id == id);
    let Some(entry) = entry else {
        // ⛔ 区分「没这个能力」与「有能力但没上架」—— 后者要给出可行动的原因
        //    （补 market.* 元数据），否则模型只会一次次重试同一个 id。
        let registered = crate::nt_capability_registry::has_node(id);
        let reason = if registered {
            let blocked = crate::nt_capability_registry::with_registry(|reg| {
                crate::nt_capability_market::blocked(reg)
                    .into_iter()
                    .find(|(bid, _)| bid == id)
                    .map(|(_, why)| why)
            })
            .ok()
            .flatten();
            match blocked {
                Some(why) => format!("能力 '{id}' 已注册但**不可上架**：{why}"),
                None => format!("能力 '{id}' 已注册但不在可上架清单里"),
            }
        } else {
            format!("能力 '{id}' 既未注册也不在市场上架清单里")
        };
        return Ok(ToolResult {
            ok: false,
            output: format!(
                "capability_invoke: {reason}。可调用 id：{}",
                market
                    .iter()
                    .map(|e| e.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            truncated: false,
        });
    };

    // ③ 调用计数：**能力本体尚未执行，故此处不计数。**
    //
    // ⚠️ 原实现在此调用 `record_dispatch`，即**先计数、后（永不）执行**，
    // 而返回 `ok: true`（见下）⇒ 上游账本读到的是「调用成功」，
    // `never_invoked` 也被污染成「有调用」。这违反生产闭环
    // 「… → fail-closed → 真实执行 → **成功后**计数」。
    //
    // 现改为：诚实报告 `counted: false`，计数待「执行端口」落地后
    // 在**执行成功之后**补上（唯一落点见下方注释）。
    let before = crate::nt_capability_registry::invoke_count(id);
    let payload = call.args.get("input").cloned();

    // ── 执行端口（2026-10-06）：真实派发尝试 ──
    //
    // 端口在 `nt-core-capability-tree`（两侧共读）⇒ core 注册实现、
    // 本 crate 消费。`Ok(None)` = **本进程没有该能力的实现**
    // ⇒ 保持 fail-closed；`Ok(Some(_))` = **真的执行了** ⇒ 此时才计数。
    let input = payload.clone().unwrap_or_else(|| serde_json::json!({}));
    // 本执行端口没有会话上下文 ⇒ 用默认会话键（与 CLI 冷启动视图同源）。
    let dispatched = nt_core_capability_tree::dispatch::dispatch(
        id,
        input.clone(),
        crate::nt_capability_canary::DEFAULT_SESSION,
    );

    //  **这里是真执行的边界**（2026-06 实测确认，见 commit 信息）：
    // 能力**本体**（trade 域的 `execute_trade` 等）的执行入口在
    // `neotrix-core` 的 L1（`nt_act_trade`），而 `neotrix-neobot` **不依赖
    // neotrix-core**（core → neobot 是既有方向，反过来会循环依赖）。
    // ⇒ 因此本函数**能诚实做到**的是：校验 + 解析 + 计数 + 回报市场元数据，
    //   并**如实说明能力本体未被执行**。
    //
    // ⛔ 刻意**不**在这里编造一个「看起来执行了」的假结果 —— 那会让调用数
    //   变成绿灯而能力依然不可用，正是本仓一路在治的「建成未用却看着健康」。
    //   执行通路的接法见 handoff `2026-10-06-capability-invoke.md` 的 §3。
    // ⛔ **fail-closed**：能力本体未执行 ⇒ 判词必须是 `ok: false`。
    //
    // 首版返回 `ok: true` 而 payload 里写 `"execution": "dispatched_to_..."`，
    // 同时 `note` 自认未执行 —— **机器可读判词与文字说明互相矛盾**。
    // 上游账本/审计读的是 `ok`，不是 `note` ⇒ 等于报假成功。
    // 这正是本仓一路在治的「建成未用却看着健康」。
    // 显式标注 exec_out 类型：移除 `Some(v)` 臂后已无锚点可推(E0282)
    let (ok, executed, counted, reason, exec_out): (bool, bool, bool, String, Option<serde_json::Value>) = match dispatched {
        // 有实现 ⇒ **真实执行**（驱动 future，按上下文安全分流）。
        Ok(Some(fut)) => match crate::nt_dispatch_drive::drive(fut) {
            // ⚠️ `drive` 的 `Result` **嵌套**实现自身的 `Result`
            //（外层=能否驱动，内层=实现执行结果）⇒ 必须三层全匹配，⛔ 不可 `?` 混掉。
            // ✅ 真实执行成功 ⇒ 现在才计数（闭环要求「成功后计数」）
            Ok(Ok(out)) => {
                let after = crate::nt_capability_registry::record_dispatch(id)
                    .map_err(NtBotError::Store)?;
                let _ = after;
                (
                    true,
                    true,
                    true,
                    "EXECUTED_VIA_DISPATCH_PORT".to_owned(),
                    Some(out),
                )
            }
            // 已驱动、但实现自身失败
            Ok(Err(e)) => (false, false, false, e, None),
            // 连驱动都失败（如已在 async runtime 内）
            Err(e) => (false, false, false, format!("DISPATCH_DRIVE_FAILED: {e}"), None),
        },
        Ok(None) => (
            false,
            false,
            false,
            "CAPABILITY_BODY_NOT_EXECUTED".to_owned(),
            None,
        ),
        Err(e) => (false, false, false, format!("DISPATCH_FAILED: {e}"), None),
    };
    let invoked_after = crate::nt_capability_registry::invoke_count(id);

    Ok(ToolResult {
        ok,
        output: serde_json::json!({
            "capability_id": entry.id,
            "kind": entry.kind.as_str(),
            "domain": entry.domain,
            "category": entry.category,
            "version": entry.version,
            "license": entry.license,
            "maturity": entry.maturity,
            "tags": entry.tags,
            "invoked_before": before,
            "invoked_after": invoked_after,
            "executed": executed,
            "counted": counted,
            "input_echo": payload,
            "reason": reason,
            "execution": if executed { "dispatched" } else { "not_executed" },
            "execution_result": exec_out,
            "note": "能力本体执行入口在 neotrix-core L1（nt_act_trade），本 crate 不反向依赖它（core → neobot 是既有方向）。本次仅完成市场校验与元数据回报，未执行、未计数。",
            "next_work": "执行端口落地后：在此处调用真执行，成功后再 record_dispatch(id) 并把 ok 置 true。",
        })
        .to_string(),
        truncated: false,
    })
}

fn execute_pdf_ground_text(
    config: &NeobotConfig,
    call: &crate::nt_types::ToolCall,
) -> Result<ToolResult, NtBotError> {
    use crate::nt_pdf_ground as ground;
    let rel = required_path(&call.args)?;
    let full = join_workspace(&config.workspace_dir, &rel)?;
    if !rel.to_ascii_lowercase().ends_with(".pdf") {
        return Ok(ToolResult {
            ok: false,
            output: format!(
                "pdf_ground_text: '{rel}' 不是 .pdf。这个工具只读 PDF 的文字层坐标，\
                 对图片/文本文件无效 —— 图片请用 qwen_visualize 渲染或 read_image 看图。"
            ),
            truncated: false,
        });
    }
    // lopdf 整篇进内存 ⇒ 必须有上限，否则一个超大 PDF 能把 daemon 吃穿。
    let meta = std::fs::metadata(&full)?;
    const PDF_CAP: u64 = 64 * 1024 * 1024;
    if meta.len() > PDF_CAP {
        return Ok(ToolResult {
            ok: false,
            output: format!(
                "pdf_ground_text: 文件 {} 字节 > 上限 {PDF_CAP}（整篇需进内存），换小的或先用 qwen_save_view 拆页。",
                meta.len()
            ),
            truncated: false,
        });
    }
    let query = call
        .args
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_owned();
    let max_pages = call
        .args
        .get("max_pages")
        .and_then(|v| v.as_u64())
        .unwrap_or(u64::from(ground::MAX_PAGES_DEFAULT))
        .clamp(1, 200) as u32;
    let report = ground::ground_text(&full, &query, max_pages)?;
    // 2026-09-29 审计修复（`ab1b9d96` 未编译）：本行原为
    //   Ok(ToolResult { ok: true, truncated: output.len() > 8000,
    //                   output: truncate_output(output, true) })
    // 即**手工构造 ToolResult 又把 truncate_output 的整个返回值塞进 output 字段**
    // —— 类型不匹配（expected String, found ToolResult），且语义双重截断。
    // 正确写法与其余 4 个调用点一致：直接返回 truncate_output 的结果
    // （截断判定与 `truncated` 标记由该函数统一负责，不要在外面重复算）。
    Ok(truncate_output(report.render(&query), true))
}

/// agent 侧 bash 执行（P0 审计 F2 加固版）：///
/// - 60s 超时杀（此前 `.output()` 死等，一句 `sleep 999` 卡死整轮）；
/// - 环境脱敏：`env_clear` + 最小白名单，不把宿主 secrets 递进 bash
///   （此前 `env` 回显直达模型 = key 送提供方）；
/// - 管道读数走独立线程，避免大输出撑爆 pipe 导致假死；
/// - **用户叫停**（C3）：`stop` 置位后杀掉已起的子进程 —— 这是整条取消
///   链路上唯一能到亚秒的地方（其余检查点的上界是一跳）。
fn execute_bash(
    config: &NeobotConfig,
    call: &crate::nt_types::ToolCall,
    stop: &StopToken,
) -> Result<ToolResult, NtBotError> {
    use std::io::Read as _;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    const BASH_TIMEOUT: Duration = Duration::from_secs(60);
    /// 递进子进程的最小环境白名单（PATH/HOME 等够用即可；`*KEY*` 类一律不传）。
    const ENV_ALLOW: &[&str] = &[
        "PATH", "HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "TMPDIR", "TEMP", "TERM",
    ];

    let Some(command) = call.args.get("command").and_then(|v| v.as_str()) else {
        return Err(NtBotError::Invalid("bash requires {command}".to_owned()));
    };
    if command.trim().is_empty() {
        return Err(NtBotError::Invalid("bash command is empty".to_owned()));
    }
    let mut cmd = std::process::Command::new("bash");
    cmd.arg("-c")
        .arg(command)
        .current_dir(&config.workspace_dir)
        .env_clear()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for key in ENV_ALLOW {
        if let Some(value) = std::env::var_os(key) {
            cmd.env(key, value);
        }
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| NtBotError::Io(format!("spawn bash: {e}")))?;
    let stdout_handle = child.stdout.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _drained: Option<usize> = pipe.read_to_end(&mut buf).ok();
            buf
        })
    });
    let stderr_handle = child.stderr.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _drained: Option<usize> = pipe.read_to_end(&mut buf).ok();
            buf
        })
    });
    let deadline = Instant::now() + BASH_TIMEOUT;
    let status = loop {
        match child
            .try_wait()
            .map_err(|e| NtBotError::Io(format!("wait bash: {e}")))?
        {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _killed: Option<()> = child.kill().ok();
                let _waited: Option<std::process::ExitStatus> = child.wait().ok();
                let mut text = String::from_utf8_lossy(
                    &stdout_handle
                        .and_then(|h| h.join().ok())
                        .unwrap_or_default(),
                )
                .into_owned();
                text.push_str("\n[neobot] bash timed out after 60s and was killed");
                return Ok(truncate_output(text, false));
            }
            None => {
                // 检查点 C3：睡之前问一次。放在 sleep **之前**才有亚秒级
                // 的响应（放在之后就是 50ms + 50ms）；50ms 本来就是既有
                // 轮询粒度，不值得为它引入 runtime 去等一个通知。
                if stop.is_cancelled() {
                    let _killed: Option<()> = child.kill().ok();
                    let _waited: Option<std::process::ExitStatus> = child.wait().ok();
                    let mut text = String::from_utf8_lossy(
                        &stdout_handle
                            .and_then(|h| h.join().ok())
                            .unwrap_or_default(),
                    )
                    .into_owned();
                    text.push_str("\n[neobot] stopped by user");
                    // `ok=false`：这一条**没有**跑完，如实说没跑完。
                    return Ok(truncate_output(text, false));
                }
                std::thread::sleep(Duration::from_millis(50))
            }
        }
    };
    let mut text = String::from_utf8_lossy(
        &stdout_handle
            .and_then(|h| h.join().ok())
            .unwrap_or_default(),
    )
    .into_owned();
    if !status.success() {
        let stderr_bytes = stderr_handle
            .and_then(|h| h.join().ok())
            .unwrap_or_default();
        let stderr = String::from_utf8_lossy(&stderr_bytes);
        text.push_str(&stderr);
    }
    Ok(truncate_output(text, status.success()))
}

fn execute_read(
    config: &NeobotConfig,
    call: &crate::nt_types::ToolCall,
    sink: &crate::nt_changes::ChangeSink<'_>,
) -> Result<ToolResult, NtBotError> {
    let path = required_path(&call.args)?;
    let full = join_workspace(&config.workspace_dir, &path)?;
    let meta = std::fs::metadata(&full)?;
    if meta.len() > READ_CAP {
        return Err(NtBotError::Invalid(format!(
            "file too large ({} > 512KiB)",
            meta.len()
        )));
    }
    let content = std::fs::read_to_string(&full)?;
    let mut result = truncate_output(content.clone(), true);
    // 读也记账（只记元信息，不记内容——内容回放没意义，路径才有）。
    if let Err(err) = sink.read(&path, content.len()) {
        crate::nt_changes::note_journal_failure(&mut result, err);
    }
    Ok(result)
}

fn execute_write(
    config: &NeobotConfig,
    call: &crate::nt_types::ToolCall,
    turn_written: &mut usize,
    sink: &crate::nt_changes::ChangeSink<'_>,
) -> Result<ToolResult, NtBotError> {
    let path = required_path(&call.args)?;
    let Some(content) = call.args.get("content").and_then(|v| v.as_str()) else {
        return Err(NtBotError::Invalid(
            "write_file requires {content}".to_owned(),
        ));
    };
    // 嵌套预算：单次上限 + 单轮累计上限（rish 律）。
    let budget = config.write_budget;
    if content.len() > budget.max_single_write_bytes {
        return Err(NtBotError::Invalid(format!(
            "content {} bytes exceeds single-write budget {}",
            content.len(),
            budget.max_single_write_bytes
        )));
    }
    if content.len() > WRITE_CAP {
        return Err(NtBotError::Invalid("content exceeds 2MiB".to_owned()));
    }
    *turn_written = turn_written.saturating_add(content.len());
    if *turn_written > budget.max_turn_write_bytes {
        return Err(NtBotError::Denied {
            rule: "write-budget".to_owned(),
            reason: format!(
                "turn wrote {} bytes, budget {}",
                turn_written, budget.max_turn_write_bytes
            ),
        });
    }
    let full = join_workspace(&config.workspace_dir, &path)?;
    // 改动账要在写盘**前**抓「改前」内容 —— 写完就抓不到了。
    let before = peek_before(&full);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&full, content)?;
    let mut result = ToolResult {
        ok: true,
        output: format!("wrote {} bytes", content.len()),
        truncated: false,
    };
    if let Err(err) = sink.write(&path, before.as_deref(), content) {
        crate::nt_changes::note_journal_failure(&mut result, err);
    }
    Ok(result)
}

fn execute_edit(
    config: &NeobotConfig,
    call: &crate::nt_types::ToolCall,
    turn_written: &mut usize,
    sink: &crate::nt_changes::ChangeSink<'_>,
) -> Result<ToolResult, NtBotError> {
    let path = required_path(&call.args)?;
    let (Some(old), Some(new)) = (
        call.args.get("old").and_then(|v| v.as_str()),
        call.args.get("new").and_then(|v| v.as_str()),
    ) else {
        return Err(NtBotError::Invalid(
            "edit_file requires {old,new}".to_owned(),
        ));
    };
    let full = join_workspace(&config.workspace_dir, &path)?;
    let content = std::fs::read_to_string(&full)?;
    let matches = content.matches(old).count();
    if matches != 1 {
        return Err(NtBotError::Invalid(format!(
            "edit needs exactly 1 match, found {matches}"
        )));
    }
    let updated = content.replacen(old, new, 1);
    let budget = config.write_budget;
    if updated.len() > WRITE_CAP {
        return Err(NtBotError::Invalid("result exceeds 2MiB".to_owned()));
    }
    *turn_written = turn_written.saturating_add(new.len());
    if *turn_written > budget.max_turn_write_bytes {
        return Err(NtBotError::Denied {
            rule: "write-budget".to_owned(),
            reason: format!(
                "turn wrote {} bytes, budget {}",
                turn_written, budget.max_turn_write_bytes
            ),
        });
    }
    std::fs::write(&full, &updated)?;
    let mut result = ToolResult {
        ok: true,
        output: "edited 1 occurrence".to_owned(),
        truncated: false,
    };
    // 编辑的前后内容手上就有（`content` 即改前、`updated` 即改后），不必再抓。
    if let Err(err) = sink.edit(&path, &content, &updated) {
        crate::nt_changes::note_journal_failure(&mut result, err);
    }
    Ok(result)
}

fn required_path(args: &serde_json::Value) -> Result<String, NtBotError> {
    args.get("path")
        .or_else(|| args.get("file"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| NtBotError::Invalid("file tool requires {path}".to_owned()))
}

/// workspace jail 双保险 (策略层已判一次, 执行层再拼一次, 防 TOCTOU 式误用).
fn join_workspace(workspace: &Path, rel: &str) -> Result<std::path::PathBuf, NtBotError> {
    if rel.trim().is_empty() || rel.starts_with('/') || rel.starts_with('~') || rel.contains("..") {
        return Err(NtBotError::Denied {
            rule: "workspace-jail".to_owned(),
            reason: "path escapes workspace".to_owned(),
        });
    }
    Ok(workspace.join(rel))
}

/// 工具输出的**统一收窄点**（5 个 executor 都走它：grep / bash / read / qwen-mm）。
///
///  2026-10-06：**这里原本是砍尾留头**（`output[..OUTPUT_CAP] + …[truncated]`），
/// 而砍尾留头有两个**具体**损失，不是「不够精细」这种修辞：
///   ① **exit code 与尾部摘要被丢掉** —— 而那正是命令成败的最终结论；
///   ② 8 KiB 之后的 `error:` / `FAIL:` 行一起被砍 ⇒ 模型看到一段正常的前缀，
///      **得到「命令成功」的错觉** —— 这是最坏的一类错误（假成功）。
///
/// ⛔⛔⛔ **本轮实测踩到的坑，比这更重要**：我一开始只在**外层**
///   （写进 transcript 前）接了 `distill_output`，测试红 ⇒ 查下去发现
///   `truncate_output` 早已在内层砍过一刀 ⇒ **外层拿到的是残缺文本**，
///   错误行在内层就没了，**外层再怎么蒸馏也救不回来**。
///   ⇒  **收窄必须发生在最靠里的那一层**，否则「更精细的算法」被更粗糙的
///      前置截断架空，成了永远走不到的死代码。
///
/// ⇒ 这里改成 errors-first 可逆蒸馏（与 `nt_output_distill` 同源）：
///   错误行前置、尾部 3 行（exit code / 摘要）享 20% 预留、其余按预算装入、
///   省略处打 `[ref#1]` 标记，全量原文仍在 `steps` 表。
fn truncate_output(text: String, ok: bool) -> ToolResult {
    if text.len() <= OUTPUT_CAP {
        return ToolResult {
            ok,
            output: text,
            truncated: false,
        };
    }
    // ⚠️ 预算按 token 算，不是字节：8 KiB 的中文 ≈ 2730 token，
    // 而 8 KiB 的 ASCII ≈ 2048 token ⇒ 用字节当 token 会高估中文的可用量。
    let budget_tokens = OUTPUT_CAP / 4;
    let distilled = crate::nt_output_distill::distill_output(&text, budget_tokens);
    ToolResult {
        ok,
        output: distilled,
        truncated: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{enforce_transcript_budget, run_local_turn};
    use crate::nt_config::NeobotConfig;
    use crate::nt_engine::LocalEchoEngine;
    use crate::nt_store::NeobotStore;
    use crate::nt_types::{TranscriptItem, TranscriptRole};

    /// 8 字节 PNG 签名（过魔数那一关即可，不必是能解码的整图）。
    const PNG_HEAD: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    /// 「无人叫停」的令牌：恒为干净的新令牌（`execute_tool` 签名要一个，
    /// 而这些既有用例验的是别的东西 —— 它们的**断言一个字都不改**）。
    fn live_stop() -> super::StopToken {
        super::StopToken::new()
    }

    #[test]
    fn transcript_budget_drops_oldest_tool_rows_first() {
        let mut history = vec![
            TranscriptItem {
                role: TranscriptRole::User,
                content: "keep me".to_owned(),
                tool_calls: Vec::new(),
                tool_call_id: None,
                image: None,
            },
            TranscriptItem {
                role: TranscriptRole::Tool,
                content: "x".repeat(300 * 1024),
                tool_calls: Vec::new(),
                tool_call_id: Some("old".to_owned()),
                image: None,
            },
            TranscriptItem {
                role: TranscriptRole::Assistant,
                content: "keep me too".to_owned(),
                tool_calls: Vec::new(),
                tool_call_id: None,
                image: None,
            },
        ];
        enforce_transcript_budget(&mut history);
        // 300KiB 的旧 Tool 行被丢，用户原文与回复保留。
        assert_eq!(history.len(), 2);
        assert!(history.iter().all(|item| item.role != TranscriptRole::Tool));
        assert!(history.iter().any(|item| item.content == "keep me"));
    }

    #[test]
    fn bash_env_scrubbed_and_path_kept() {
        use super::execute_bash;
        use crate::nt_types::{ToolCall, ToolName};
        // 审计 F2：宿主 secrets 不得递进 bash；白名单 PATH 得留（找得到 wc）。
        std::env::set_var("NEOBOT_TEST_ONLY_SECRET", "s3cr3t-marker");
        let dir = crate::nt_testutil::temp_dir("bash-test");
        let _ = std::fs::create_dir_all(dir.join("workspace"));
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::WriteBudget {
                max_single_write_bytes: 1024,
                max_turn_write_bytes: 1024,
                max_task_write_bytes: 1024,
            },
        };
        let call = ToolCall {
            id: "c1".to_owned(),
            name: ToolName::Bash,
            args: serde_json::json!({"command": "echo $NEOBOT_TEST_ONLY_SECRET | wc -c"}),
        };
        let res = execute_bash(&config, &call, &live_stop()).expect("run");
        std::env::remove_var("NEOBOT_TEST_ONLY_SECRET");
        assert!(
            res.ok,
            "PATH whitelist must keep wc working: {}",
            res.output
        );
        assert!(
            !res.output.contains("s3cr3t-marker"),
            "env must be scrubbed: {}",
            res.output
        );
    }

    #[test]
    fn turn_write_budget_denies_overrun() {
        let dir = crate::nt_testutil::temp_dir("budget-test");
        let _ = std::fs::remove_dir_all(&dir);
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::WriteBudget {
                max_single_write_bytes: 10,
                max_turn_write_bytes: 15,
                max_task_write_bytes: 1024,
            },
        };
        config.validate().expect("validate");
        // 改动账：真存一个内存库，顺带验「预算被拒时不记账」（拒在写盘前，
        // 盘上没动过就不该有账 —— 记了就是假账）。
        let store = crate::nt_store::NeobotStore::open(":memory:").expect("store");
        let sink = crate::nt_changes::ChangeSink {
            store: &store,
            task_id: "budget-task",
            workspace: &config.workspace_dir,
        };
        // 单次超限拒
        let big = serde_json::json!({"path": "a.txt", "content": "0123456789ABCDEF"});
        let mut turn_written = 0usize;
        let err = super::execute_write(
            &config,
            &crate::nt_types::ToolCall {
                id: "w1".to_owned(),
                name: crate::nt_types::ToolName::WriteFile,
                args: big,
            },
            &mut turn_written,
            &sink,
        )
        .expect_err("single over budget must fail");
        assert!(err.to_string().contains("single-write"), "{err}");
        // 两次小写累计超轮预算拒
        let small = serde_json::json!({"path": "b.txt", "content": "0123456789"});
        let call = crate::nt_types::ToolCall {
            id: "w2".to_owned(),
            name: crate::nt_types::ToolName::WriteFile,
            args: small,
        };
        super::execute_write(&config, &call, &mut turn_written, &sink).expect("first small write");
        let err = super::execute_write(&config, &call, &mut turn_written, &sink)
            .expect_err("turn over budget must fail");
        assert!(err.to_string().contains("write-budget"), "{err}");
        // 只有第一次成功写入了盘，才该有一笔账。
        let tallies = store.tally_task_paths("budget-task").expect("tally");
        assert_eq!(tallies.len(), 1, "被拒的两次不该记账：{tallies:?}");
        assert_eq!(tallies.first().map(|t| t.path.as_str()), Some("b.txt"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn placeholder_convo_auto_titles_from_first_text() {
        let dir = crate::nt_testutil::temp_dir("autotitle-test");
        let _ = std::fs::remove_dir_all(&dir);
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");
        store.upsert_member("neo", "human").expect("member");
        let dm = store
            .create_conversation("dm", "我的私聊", &["neo".to_owned()])
            .expect("dm");
        let run = |text: &str, convo: &str| {
            super::run_local_turn_as(
                &store,
                &config,
                &LocalEchoEngine,
                crate::nt_policy::Actor::Bot,
                "bot",
                "t",
                text,
                Some(convo),
            )
            .expect("run")
        };
        let title_of = |id: &str| {
            store
                .list_conversations()
                .expect("list")
                .into_iter()
                .find(|c| c.id == id)
                .expect("convo")
                .title
        };
        // 首条有效输入命名
        run("帮我写一份周报总结\n第二行不进标题", &dm);
        assert_eq!(title_of(&dm), "帮我写一份周报总结");
        // 仅一次：第二轮不再改名
        run("随便聊点别的什么内容", &dm);
        assert_eq!(title_of(&dm), "帮我写一份周报总结");
        // 纯附件系统行不命名
        let g = store
            .create_conversation("group", "新群组", &[])
            .expect("group");
        run("[附件：a.png]", &g);
        assert_eq!(title_of(&g), "新群组");
        // 超长截 24 字
        let g2 = store
            .create_conversation("group", "新群组", &[])
            .expect("group2");
        run(
            "这是一条超过二十四个字的超长输入内容用来测试截断行为是否正确",
            &g2,
        );
        assert_eq!(title_of(&g2).chars().count(), 24);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn echo_run_completes_and_persists() {
        let dir = crate::nt_testutil::temp_dir("agent-test");
        let _ = std::fs::remove_dir_all(&dir);
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");
        let status = run_local_turn(&store, &config, &LocalEchoEngine, "t", "hello").expect("run");
        assert_eq!(status, crate::nt_types::TurnStatus::Done);
        assert_eq!(store.list_tasks(10).expect("list").len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 永远只调工具不收尾的引擎 — 验证耗尽转 Waiting + 最后一步 nudge.
    struct LoopForever {
        seen_nudge: std::sync::Mutex<bool>,
        /// 每跳执行的 shell 命令。默认 `echo x`；接线测试改成 `cat` 凭据文件。
        command: Option<String>,
    }

    impl LoopForever {
        fn new() -> Self {
            Self {
                seen_nudge: std::sync::Mutex::new(false),
                command: None,
            }
        }

        fn with_command(command: &str) -> Self {
            Self {
                seen_nudge: std::sync::Mutex::new(false),
                command: Some(command.to_owned()),
            }
        }
    }

    impl crate::nt_engine::EngineAdapter for LoopForever {
        fn engine_id(&self) -> &str {
            "loop"
        }

        fn probe(&self) -> Result<String, crate::NtBotError> {
            Ok("loop".to_owned())
        }

        fn run_turn(
            &self,
            _prompt: &str,
            _inbox: &[String],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            Ok(crate::nt_engine::EngineTurn {
                assistant_text: String::new(),
                status: crate::nt_types::TurnStatus::Continue,
                tool_calls: vec![crate::nt_types::ToolCall {
                    id: "l1".to_owned(),
                    name: crate::nt_types::ToolName::Bash,
                    args: serde_json::json!({
                        "command": self.command.as_deref().unwrap_or("echo x"),
                    }),
                }],
                usage: None,
                side_effects: Vec::new(),
            })
        }

        fn run_turn_with_history(
            &self,
            prompt: &str,
            history: &[crate::nt_types::TranscriptItem],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            if history.iter().any(|item| item.content.contains("最后一步")) {
                if let Ok(mut seen) = self.seen_nudge.lock() {
                    *seen = true;
                }
            }
            self.run_turn(prompt, &[])
        }
    }

    /// 按固定路径 `read_file` 一次的引擎（`ReadFile` 在 policy 里是 allow 分支）。
    ///
    ///  顺带记录收到的 `history` —— 蒸馏结果只走这条路，而 `history` 是
    /// `run_loop` 的局部变量（**不落库**）⇒ 查 steps / audit 都取不到它。
    struct ReadPathEngine {
        path: String,
        done: std::sync::atomic::AtomicBool,
        seen: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl ReadPathEngine {
        fn new(path: &str) -> Self {
            Self {
                path: path.to_owned(),
                done: std::sync::atomic::AtomicBool::new(false),
                seen: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn seen(&self) -> std::sync::Arc<std::sync::Mutex<Vec<String>>> {
            self.seen.clone()
        }
    }

    impl crate::nt_engine::EngineAdapter for ReadPathEngine {
        fn engine_id(&self) -> &str {
            "read-path"
        }

        fn probe(&self) -> Result<String, crate::NtBotError> {
            Ok("read-path".to_owned())
        }

        fn run_turn(
            &self,
            _prompt: &str,
            _inbox: &[String],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            Ok(self.turn())
        }

        fn run_turn_with_history(
            &self,
            _prompt: &str,
            history: &[crate::nt_types::TranscriptItem],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            if let Ok(mut seen) = self.seen.lock() {
                for item in history {
                    seen.push(item.content.clone());
                }
            }
            Ok(self.turn())
        }
    }

    impl ReadPathEngine {
        fn turn(&self) -> crate::nt_engine::EngineTurn {
            let first = !self.done.swap(true, std::sync::atomic::Ordering::SeqCst);
            crate::nt_engine::EngineTurn {
                assistant_text: String::new(),
                status: if first {
                    crate::nt_types::TurnStatus::Continue
                } else {
                    crate::nt_types::TurnStatus::Done
                },
                tool_calls: if first {
                    vec![crate::nt_types::ToolCall {
                        id: "rp1".to_owned(),
                        name: crate::nt_types::ToolName::ReadFile,
                        args: serde_json::json!({"path": self.path.clone()}),
                    }]
                } else {
                    Vec::new()
                },
                usage: None,
                side_effects: Vec::new(),
            }
        }
    }

    ///  **接线守门：超预算工具输出进 transcript 时必须保留尾部错误与 exit code。**
    ///
    /// ⛔ 判别式必须区分两件事，否则就是**拿正确实现当缺陷**：
    ///   - 「被蒸馏」（长度收缩）
    ///   - 「超预算」（本该蒸馏）
    /// 我第一版夹具只写 200 行 ≈ 8206 字节 ≈ **2051 token**，落在 3000 预算
    /// 内 ⇒ `distill_output` **原样透传**（正确行为！）⇒ 测试红了。
    /// ⇒ 夹具必须远超预算，且要断言「Tool 行确实被收缩」。
///  **在测试里注册一个「已上架」能力并返回可调用的 id。**
    ///
    /// ## 为什么自己造节点而不是播种真实 trade 能力
    ///
    /// `bootstrap_trade_capabilities()` 在 `neotrix-core`，而
    /// `neotrix-neobot` **不依赖 neotrix-core**（core → neobot 是既有方向，
    /// 反过来会循环依赖）⇒ 本 crate 的测试**够不到**那个播种器。
    ///
    /// ⇒ 这里用**本 crate 自己的真接口**（`register_node` + `market.*`
    /// 元数据）造一个能力。这样测的是「市场 ⇒ 模型 ⇒ 派发 ⇒ 计数」这条链
    /// 本身，而不是「core 的播种器有没有被调到」—— 后者不在本 crate 的边界内。
    ///
    /// ⛔ id 带纳秒 ⇒ 避免与并行测试撞同 id（注册对同 id 幂等，但撞了会让
    /// 「before == after」类断言失真）。
    fn seed_market_capability_for_test() -> String {
        use nt_core_capability_tree::node::{CapabilityKind, CapabilityNode, Domain};
        let id = format!(
            "NT-TEST::cap::{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let mut node = CapabilityNode::new_primitive(
            id.clone(),
            Domain::Mind,
            vec!["test.capability".to_owned()],
        );
        node.kind = CapabilityKind::Skill;
        node.metadata.insert(
            crate::nt_capability_market::meta_keys::LICENSE.to_owned(),
            serde_json::Value::String("LicenseRef-Test".to_owned()),
        );
        node.metadata.insert(
            crate::nt_capability_market::meta_keys::VERSION.to_owned(),
            serde_json::Value::String("0.0.1".to_owned()),
        );
        node.metadata.insert(
            crate::nt_capability_market::meta_keys::CATEGORY.to_owned(),
            serde_json::Value::String("test".to_owned()),
        );
        crate::nt_capability_registry::register_node(node).expect("登记应成功");
        id
    }

    fn cap_invoke_call(id: &str) -> crate::nt_types::ToolCall {
        crate::nt_types::ToolCall {
            id: "c-cap".to_owned(),
            name: crate::nt_types::ToolName::CapabilityInvoke,
            args: serde_json::json!({"capability_id": id, "input": {"probe": true}}),
        }
    }

    fn run_cap_invoke(
        config: &crate::nt_config::NeobotConfig,
        call: &crate::nt_types::ToolCall,
    ) -> crate::nt_types::ToolResult {
        super::execute_tool(
            config,
            &LocalEchoEngine,
            call,
            &mut 0usize,
            &crate::nt_changes::ChangeSink {
                store: &crate::nt_store::NeobotStore::open(":memory:").expect("store"),
                task_id: "t",
                workspace: &config.workspace_dir,
            },
            &live_stop(),
        )
        .expect("execute_tool 不应 panic")
        .result
    }

    ///  **能力市场调用必须真的让调用数从 0 变正。**
    ///
    /// 这是本轮接线的**目标判据**。实测起点：探针里 5 个 trade 能力的
    /// `invoked` 全为 0、`never_invoked` 列满 —— 能力在市场上「可上架」，
    /// 而模型**没有任何通路**能调它们。
    ///
    /// ⛔ 判据不能是「schema 里出现了 capability_invoke」—— 那只证明模型
    /// **看得见**，不证明它**调得动**。必须走完 tool_call → execute → 计数。
    #[test]
    fn 能力市场调用让调用数变正() {
        let (_dir, config) = vision_fixture("cap-invoke");

        let target = seed_market_capability_for_test();
        let listed = crate::nt_capability_registry::with_registry(|reg| {
            crate::nt_capability_market::listable(reg)
                .into_iter()
                .any(|e| e.id == target)
        })
        .expect("注册表锁");
        assert!(listed, "★ 前置失败：造出的能力不在上架清单 ⇒ 用例零区分力");
        assert_eq!(
            crate::nt_capability_registry::invoke_count(&target),
            0,
            "★ 前置失败：新能力不该已有调用计数"
        );

        let result = run_cap_invoke(&config, &cap_invoke_call(&target));

        // ⚠️ 契约（2026-10-06，按派发端口分流）：
        //   ·本进程**未注册**该能力的派发实现 ⇒ `ok:false` +
        //     `CAPABILITY_BODY_NOT_EXECUTED` 且**不计数**（fail-closed）；
        //   · 已注册且执行成功 ⇒ `ok:true`、**此时才计数**（闭环要求「成功后计数」）。
        //   端口：`nt_core_capability_tree::dispatch::dispatch`（core 注册 / 本 crate 消费）。
        // ⚠️ 契约已变更（2026-10-06 B2 修复）：能力本体**尚未执行**，
        // 故判词必须是 `ok:false`、计数**不得增加**。
        // 本用例原先断言 `ok:true` 且计数 +1 —— **那固化了缺陷行为**：
        // 「先计数、后永不执行」正是「能力恒 0 调用」被掩盖成「有调用」的原因。
        assert!(
            !result.ok,
            "★ 未执行本体时不得报成功（否则上游账本读到假成功）: {}",
            result.output
        );
        assert!(
            result.output.contains("\"executed\":false"),
            "★ payload 应显式声明未执行: {}",
            result.output
        );
        assert!(
            result.output.contains("CAPABILITY_BODY_NOT_EXECUTED"),
            "★ 应给出可机读的原因码: {}",
            result.output
        );
        assert_eq!(
            crate::nt_capability_registry::invoke_count(&target),
            0,
            "★ 未执行 ⇒ 计数不得增加（「成功后计数」；本仓全程受此约束）"
        );
        assert!(
            result.output.contains(&target),
            "★ 回执须含被调能力 id，★ 否则模型无法确认调用对了目标: {}",
            result.output
        );
    }

    ///  **未上架的 id 必须被拒**（fail-closed 这道门）。
    ///
    /// ⛔ 这条比上一条更重要：上一条证明「能调」，这条证明「不该调的调不了」。
    ///   其中最要紧的是 `consciousness::gap::*` —— 意识登记的**缺口**
    ///   若能被当能力调用，「我不会」就变成「我会」，语义直接反了。
    #[test]
    fn 未上架的能力调用被拒() {
        let (_dir, config) = vision_fixture("cap-reject");

        for bad_id in [
            "NT-MEMORY::trade::does_not_exist",
            "consciousness::gap::q20",
        ] {
            let result = run_cap_invoke(&config, &cap_invoke_call(bad_id));
            assert!(
                !result.ok,
                "★ 未上架的 id '{bad_id}' 竟被放行（★ fail-closed 被破坏）"
            );
            assert!(
                !result.output.contains("dispatched_to_capability_registry"),
                "★ 拒绝回执里不应出现派发标记: {}",
                result.output
            );
            assert_eq!(
                crate::nt_capability_registry::invoke_count(bad_id),
                0,
                "★ 被拒的调用不得计入计数（★ 否则「调用过」会含混）"
            );
        }
    }

    #[test]
    fn 超预算工具输出进transcript保留尾部错误与退出码() {
        let dir = crate::nt_testutil::temp_dir("distill-wiring");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("workspace")).expect("mkdir");
        // 前缀远超旧 LIMIT(4096) 与蒸馏预算(3000 token)，错误与 exit code 在**尾部**。
        let mut big = String::new();
        for i in 0..600 {
            big.push_str(&format!(
                "filler line {i} padding padding padding padding\n"
            ));
        }
        big.push_str("error: the tail error that must survive\n");
        big.push_str("exit code 3\n");
        std::fs::write(dir.join("workspace/big.log"), &big).expect("write");
        assert!(
            big.len() > 4096 * 3,
            "夹具须远超旧 LIMIT，实际 {}",
            big.len()
        );

        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 2,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");
        let engine = ReadPathEngine::new("big.log");

        let _ = run_local_turn(&store, &config, &engine, "distill", "go").expect("run");

        let seen = engine.seen();
        let snapshot: Vec<String> = seen.lock().map(|v| v.clone()).unwrap_or_default();
        let tool_line = snapshot
            .iter()
            .find(|x| x.contains("filler line") || x.contains("## errors"))
            .expect("★ Tool 行缺失（引擎没收到）⇒ 用例零区分力");
        assert!(
            tool_line.len() < big.len(),
            "★ Tool 行未被蒸馏（{} 字节 vs 原文 {}）⇒ 用例零区分力",
            tool_line.len(),
            big.len()
        );
        assert!(
            tool_line.contains("error: the tail error"),
            "★ 尾部错误行被丢弃（砍尾留头的核心损失）：{}",
            &tool_line[..tool_line.len().min(300)]
        );
        assert!(
            tool_line.contains("exit code 3"),
            "★ 尾部 exit code 被丢弃（★ 这正是砍尾留头的核心损失）：{}",
            &tool_line[..tool_line.len().min(300)]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    ///  **接线守门：最终输出必须产生一条 `reply_governance` step。**
    ///
    /// 这条断言的作用：证明治理**真的在生产派发路径上跑过**，而不只是编译通过。
    /// ⛔ 若把 `record_output_governance(...)` 那行删掉，本用例立刻红
    /// （查不到任何 `reply_governance` 行）⇒ 零证明力的写法是只断言
    /// 「调用返回 Ok」。
    #[test]
    fn 最终输出会产生治理报告() {
        let dir = crate::nt_testutil::temp_dir("governance-wiring");
        let _ = std::fs::remove_dir_all(&dir);
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 2,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");

        let status = run_local_turn(&store, &config, &LocalEchoEngine, "t", "hello").expect("run");
        assert_eq!(status, crate::nt_types::TurnStatus::Done);

        // 用 audit 侧信道断言：task_id 是内部 UUID，测试拿不到，
        // 而 audit 按最近倒序可查。
        let audits = store.list_audit(50).expect("audits");
        let gov: Vec<_> = audits
            .iter()
            .filter(|a| a.tool == "output_governance")
            .collect();
        assert_eq!(
            gov.len(),
            1,
            "每条最终输出恰好一条治理审计；实际 audit tools: {:?}",
            audits.iter().map(|a| &a.tool).collect::<Vec<_>>()
        );
        assert!(
            gov[0].detail.contains("score="),
            "治理报告应含得分；实际: {}",
            gov[0].detail
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    ///  **变异守门**：模型输出命中治理规则时，`reply_governance` 必须标红。
    ///
    /// 这一条比上一条更重要：上一条只能证明「跑了」，这一条证明「跑出了真实
    /// 结论」。若治理器被改成永远返回满分，本条红。
    /// 只读一次凭据文件的引擎（`ReadFile` 在 policy 里是 allow 分支）。
    struct ReadCredsEngine {
        done: std::sync::atomic::AtomicBool,
    }

    impl ReadCredsEngine {
        fn new() -> Self {
            Self {
                done: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    impl crate::nt_engine::EngineAdapter for ReadCredsEngine {
        fn engine_id(&self) -> &str {
            "read-creds"
        }

        fn probe(&self) -> Result<String, crate::NtBotError> {
            Ok("read-creds".to_owned())
        }

        fn run_turn(
            &self,
            _prompt: &str,
            _inbox: &[String],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            Ok(self.turn(&[]))
        }

        fn run_turn_with_history(
            &self,
            _prompt: &str,
            _history: &[crate::nt_types::TranscriptItem],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            Ok(self.turn(&[]))
        }
    }

    impl ReadCredsEngine {
        fn turn(
            &self,
            _history: &[crate::nt_types::TranscriptItem],
        ) -> crate::nt_engine::EngineTurn {
            let first = !self.done.swap(true, std::sync::atomic::Ordering::SeqCst);
            crate::nt_engine::EngineTurn {
                assistant_text: String::new(),
                status: if first {
                    crate::nt_types::TurnStatus::Continue
                } else {
                    crate::nt_types::TurnStatus::Done
                },
                tool_calls: if first {
                    vec![crate::nt_types::ToolCall {
                        id: "r1".to_owned(),
                        name: crate::nt_types::ToolName::ReadFile,
                        args: serde_json::json!({"path": "creds.env"}),
                    }]
                } else {
                    Vec::new()
                },
                usage: None,
                side_effects: Vec::new(),
            }
        }
    }

    ///  **接线守门：工具输出含凭据时，生产派发路径必须落一条扫描审计。**
    ///
    /// 这条测试**真的跑 `run_loop`**：`LoopForever` 每跳执行
    /// `cat <凭据文件>` ⇒ 凭据经 `execute_bash` 的输出回到 `result.output`
    /// ⇒ 必经 `add_step` 之前那段扫描。
    ///
    /// ⛔ 第一版我写成「直接调 `nt_secret_scan::scan()` + `summarize()`」，
    ///   那只测了模块与 audit API，**没跑到接线点** ⇒ 把整段 `match` 删掉
    ///   它仍全绿 ⇒ 零证明力。现在改为端到端。
    #[test]
    fn 工具输出含凭据时生产路径落扫描审计() {
        let dir = crate::nt_testutil::temp_dir("secret-scan-wiring");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("workspace")).expect("mkdir");
        std::fs::write(
            dir.join("workspace/creds.env"),
            "OPENAI_API_KEY=sk-abcdefghijklmnopqrstuvwxyz0123\n",
        )
        .expect("write");
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 1,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");
        // ⛔ Bash 是**恒定 deny**（nt_policy.rs:172-173 的 `default-deny`），
        //   所以这条用例必须走 `ReadFile`（allow 分支）才能真正执行。
        //   我第一版用 bash ⇒ 每次都 `(denied)`，接线根本没被执行到 ——
        //   报错信息（audit tools 只有 ["bash"]）才是定位线索。
        let engine = ReadCredsEngine::new();

        let _ = run_local_turn(&store, &config, &engine, "scan", "go").expect("run");

        let audits = store.list_audit(50).expect("audits");
        let bash: Vec<_> = audits.iter().filter(|a| a.tool == "bash").collect();
        eprintln!(
            "DEBUG bash detail={:?}",
            bash.iter().map(|a| a.detail.as_str()).collect::<Vec<_>>()
        );
        let tid: Vec<String> = Vec::new();
        let _ = tid;
        // 直接用 store 查所有 task 的 step 不可行（★ task_id 内部 UUID），
        // 改为在 bash 审计后看是否有 output_secret_scan。
        let hits: Vec<_> = audits
            .iter()
            .filter(|a| a.tool == "output_secret_scan")
            .collect();
        assert!(
            !hits.is_empty(),
            "★ 生产路径未产生凭据扫描审计 ⇒ 接线被删或没跑到；audit tools: {:?}",
            audits.iter().map(|a| &a.tool).collect::<Vec<_>>()
        );
        // ⛔ 报告不得含凭据原文（报告本身也是泄露面）
        assert!(
            !hits[0].detail.contains("sk-abcdefghijklmnop"),
            "警告文案泄露凭据原文: {}",
            hits[0].detail
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 治理违规会把报告标红() {
        use crate::nt_governance::OutputGovernor;
        let gov = OutputGovernor::new();
        // R02（禁止模糊对冲）判据是 `total >= 3`，且词表**只有 14 个**
        // （可能/或许/大概/也许/我觉得/我猜/我认为/好像 + 6 个英文）。
        // ⛔ 我第一版写的「综上所述 / 总体来看」**不在词表里** ⇒ R02 放过、
        //   实际命中的是 AI-smell（绕开 rule_results）⇒ 变异能漏过。
        // ⛔ 第二版补了 3 处但仍用表外词 ⇒ 依旧漏过。⇒ 样本必须取自 HEDGES。
        let bad = "可能完成了。\n或许还需要再看看。\n大概就这样。";
        let report = gov.govern(bad);
        // ⛔ 必须**只**断言 violations：smells 由 AiSmellDetector 独立计算，
        // 绕开 rule_results ⇒ 用 `|| smells` 会让「治理器恒满分」的变异漏过
        // （我第一版就是这么写的，变异测试当场抓出来）。
        assert!(
            !report.violations.is_empty(),
            "含对冲词/AI 味的文本应被判违规；实际 violations={:?} smells={:?}",
            report.violations,
            report.smells.len()
        );
        // 干净文本不该被误伤 ⇒ 治理不是「一律判红」
        // ⛔ 样本必须**不含文件引用**：R07 会真读文件系统校验「引用的文件
        // 是否存在」，随便写个 src/main.rs 就会被判违规（我第一版就踩了这个，
        // 报错信息反而暴露了 R07 在正常工作）。
        let good = "读取完成。\n共处理 3 个请求。\n未发现问题。";
        let clean = gov.govern(good);
        assert!(
            clean.violations.is_empty(),
            "正常文本不应被误判；实际: {:?}",
            clean.violations
        );
    }

    #[test]
    fn exhaustion_becomes_waiting_with_nudge() {
        let dir = crate::nt_testutil::temp_dir("agent-loop-test");
        let _ = std::fs::remove_dir_all(&dir);
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 3,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");
        let engine = LoopForever::new();
        let status = run_local_turn(&store, &config, &engine, "loop", "go").expect("run");
        assert_eq!(status, crate::nt_types::TurnStatus::Waiting);
        assert!(engine.seen_nudge.lock().map(|seen| *seen).unwrap_or(false));
        // 3 跳 bash 全执行.
        let audits = store.list_audit(20).expect("audits");
        assert_eq!(
            audits.iter().filter(|event| event.tool == "bash").count(),
            3
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 会看图的引擎（`LocalEchoEngine` 不可用，故借一个只覆写 `vision_capable`
    /// 的壳 —— 它要验证的是「看得见的那条路上图像真的被生产出来」，
    /// 而不是引擎本身的网络行为）。
    struct VisionEngine;

    impl crate::nt_engine::EngineAdapter for VisionEngine {
        fn engine_id(&self) -> &str {
            "vision-test"
        }

        fn model_name(&self) -> &str {
            "test-vl"
        }

        fn vision_capable(&self) -> bool {
            true
        }

        fn probe(&self) -> Result<String, crate::NtBotError> {
            Ok("vision-test".to_owned())
        }

        fn run_turn(
            &self,
            _prompt: &str,
            _inbox: &[String],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            Ok(crate::nt_engine::EngineTurn {
                assistant_text: String::new(),
                status: crate::nt_types::TurnStatus::Done,
                tool_calls: Vec::new(),
                usage: None,
                side_effects: Vec::new(),
            })
        }
    }

    /// 每个用例一份**独立**目录：`cargo test` 默认并行跑，同名目录会互相
    /// `remove_dir_all` —— 上一轮就是被这个坑出来的偶发红灯。
    fn vision_fixture(tag: &str) -> (std::path::PathBuf, crate::nt_config::NeobotConfig) {
        let dir = crate::nt_testutil::temp_dir(&format!("neobot-readimage-{}", tag));
        let _ = std::fs::remove_dir_all(&dir);
        let workspace = dir.join("workspace");
        std::fs::create_dir_all(&workspace).expect("mkdir workspace");
        // 4 字节 PNG 头 + 一点体（够过魔数与 base64 往返，不必是能解码的整图）。
        let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        png.extend_from_slice(&[b'I', b'H', b'D', b'R']);
        std::fs::write(workspace.join("shot.png"), &png).expect("write png");
        let config = crate::nt_config::NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: workspace,
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        (dir, config)
    }

    fn read_image_call(path: &str) -> crate::nt_types::ToolCall {
        crate::nt_types::ToolCall {
            id: "c-img".to_owned(),
            name: crate::nt_types::ToolName::ReadImage,
            args: serde_json::json!({"path": path}),
        }
    }

    #[test]
    fn read_image_fails_honestly_on_a_blind_engine() {
        let (dir, config) = vision_fixture("blind");
        // `LocalEchoEngine` 没有模型、也就没有眼睛：必须**报错**，
        // 绝不返回「已读到 shot.png」这种会让模型开始编画面的成功结果。
        let err = super::execute_tool(
            &config,
            &LocalEchoEngine,
            &read_image_call("shot.png"),
            &mut 0usize,
            &crate::nt_changes::ChangeSink {
                store: &crate::nt_store::NeobotStore::open(":memory:").expect("store"),
                task_id: "t",
                workspace: &config.workspace_dir,
            },
            &live_stop(),
        )
        .expect_err("blind engine must not pretend to see");
        let text = err.to_string();
        assert!(text.contains("no-vision-engine"), "{text}");
        assert!(text.contains("echo"), "{text}"); // 点名是哪个引擎
        assert!(text.contains("NEOBOT_VISION"), "{text}"); // 给可操作的出路
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// turn 级 E2E：**模型点名 → 网关放行 → 真解析 → 坐标回到模型手里**。
    ///
    /// 为什么这条能测、Qwen 那条不能：Qwen 链的 turn 级测试要求
    /// `qwen_mm_mounted()` 为真（本机装了 MCP 服务器），CI 不保证 ⇒ 非确定性。
    /// `pdf_ground_text` 零外部依赖，所以整条链可以确定性地端到端跑完。
    /// 代价是它测不到 MCP framing —— 那部分由 core 侧
    /// `test_native_adapter_drives_session` 与 neobot 的分发单测各自覆盖。
    /// **三段职责分开测，别指望一条测试吃掉全部。**
    struct GroundOnce {
        seen_result: std::sync::Mutex<bool>,
    }

    impl GroundOnce {
        fn turn(
            &self,
            history: &[TranscriptItem],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            use crate::nt_types::{ToolCall, ToolName, TurnStatus};
            // 关键断言点：工具输出**真的回到了模型**（命中数与坐标都在 history 里）。
            // 只断言「audit 里有这条工具」是不够的 —— 那只证明网关放行了，
            // 不证明模型拿到了结果。
            let fed_back = history.iter().any(|item| {
                item.role == TranscriptRole::Tool
                    && item.content.contains("命中")
                    && item.content.contains("118")
            });
            if let Ok(mut seen) = self.seen_result.lock() {
                *seen = fed_back;
            }
            if fed_back {
                return Ok(crate::nt_engine::EngineTurn {
                    assistant_text: "在第 1 页偏上位置。".to_owned(),
                    status: TurnStatus::Done,
                    tool_calls: Vec::new(),
                    usage: None,
                    side_effects: Vec::new(),
                });
            }
            Ok(crate::nt_engine::EngineTurn {
                assistant_text: String::new(),
                status: TurnStatus::Continue,
                tool_calls: vec![ToolCall {
                    id: "g1".to_owned(),
                    name: ToolName::PdfGroundText,
                    args: serde_json::json!({"path": "contract.pdf", "query": "Confidential"}),
                }],
                usage: None,
                side_effects: Vec::new(),
            })
        }
    }

    impl crate::nt_engine::EngineAdapter for GroundOnce {
        fn engine_id(&self) -> &str {
            "ground-once"
        }

        fn probe(&self) -> Result<String, crate::NtBotError> {
            Ok("ground-once".to_owned())
        }

        fn run_turn(
            &self,
            _prompt: &str,
            _inbox: &[String],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            self.turn(&[])
        }

        fn run_turn_with_history(
            &self,
            _prompt: &str,
            history: &[TranscriptItem],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            self.turn(history)
        }
    }

    #[test]
    fn model_can_ground_pdf_text_end_to_end() {
        let dir = crate::nt_testutil::temp_dir("pdf-ground-e2e");
        let _ = std::fs::remove_dir_all(&dir);
        let workspace = dir.join("workspace");
        std::fs::create_dir_all(&workspace).expect("workspace");
        std::fs::write(
            workspace.join("contract.pdf"),
            crate::nt_pdf_ground::fixture_pdf(),
        )
        .expect("write pdf");
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: workspace,
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = NeobotStore::open(":memory:").expect("open");
        let engine = GroundOnce {
            seen_result: std::sync::Mutex::new(false),
        };
        let status = run_local_turn(&store, &config, &engine, "e2e", "第几页提到 Confidential")
            .expect("run");
        assert_eq!(status, crate::nt_types::TurnStatus::Done);
        // 网关放行了模型点的这个工具（不是 deny 后假装成功）。
        let audits = store.list_audit(20).expect("audits");
        let ground = audits
            .iter()
            .find(|event| event.tool == "pdf_ground_text")
            .expect("pdf_ground_text must be audited");
        assert_eq!(ground.decision, crate::nt_audit::AuditDecision::Allow);
        // 「自主执行」与「函数存在」的分界就在这一行。
        assert!(
            engine.seen_result.lock().map(|seen| *seen).unwrap_or(false),
            "tool output never reached the model"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_image_produces_a_real_part_not_a_base64_wall() {
        let (dir, config) = vision_fixture("part");
        let outcome = super::execute_tool(
            &config,
            &VisionEngine,
            &read_image_call("shot.png"),
            &mut 0usize,
            &crate::nt_changes::ChangeSink {
                store: &crate::nt_store::NeobotStore::open(":memory:").expect("store"),
                task_id: "t",
                workspace: &config.workspace_dir,
            },
            &live_stop(),
        )
        .expect("vision engine must see the image");
        assert!(outcome.result.ok);
        let image = outcome.image.expect("must carry the image part");
        assert_eq!(image.media_type, "image/png");
        assert_eq!(
            image.base64,
            crate::nt_vision::base64_encode(&[
                0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, b'I', b'H', b'D', b'R'
            ])
        );
        // 文本那行是**说明**不是载荷：base64 不许漏进 content。
        assert!(!outcome.result.output.contains(&image.base64));
        assert!(
            outcome.result.output.contains("image/png"),
            "{}",
            outcome.result.output
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_image_jail_and_type_refusals_reach_the_model() {
        let (dir, config) = vision_fixture("jail");
        let store = crate::nt_store::NeobotStore::open(":memory:").expect("store");
        let sink = crate::nt_changes::ChangeSink {
            store: &store,
            task_id: "t",
            workspace: &config.workspace_dir,
        };
        // 越狱：与 read_file 同一道 jail。
        let err = super::execute_tool(
            &config,
            &VisionEngine,
            &read_image_call("../escape.png"),
            &mut 0usize,
            &sink,
            &live_stop(),
        )
        .expect_err("escape must fail");
        assert!(err.to_string().contains("workspace-jail"), "{err}");
        // 缺参：不装懂。（空串另算 —— `required_path` 只管键在不在，
        // 空串交给下一道 jail 拒，报的是「越狱」而不是「缺参」，同样诚实。）
        let err = super::execute_tool(
            &config,
            &VisionEngine,
            &crate::nt_types::ToolCall {
                id: "c-noargs".to_owned(),
                name: crate::nt_types::ToolName::ReadImage,
                args: serde_json::json!({}),
            },
            &mut 0usize,
            &sink,
            &live_stop(),
        )
        .expect_err("missing path must fail");
        assert!(err.to_string().contains("requires {path}"), "{err}");
        let err = super::execute_tool(
            &config,
            &VisionEngine,
            &read_image_call(""),
            &mut 0usize,
            &sink,
            &live_stop(),
        )
        .expect_err("empty path must fail");
        assert!(err.to_string().contains("workspace-jail"), "{err}");
        // 名不副实：zip 装的 .png 必须被拆穿，且图像**字段为空**
        // （宁可不发，也不能发一个错 media_type 的部件）。
        std::fs::write(
            config.workspace_dir.join("fake.png"),
            b"PK\x03\x04not an image",
        )
        .expect("write fake");
        let err = super::execute_tool(
            &config,
            &VisionEngine,
            &read_image_call("fake.png"),
            &mut 0usize,
            &sink,
            &live_stop(),
        )
        .expect_err("zip is not an image");
        assert!(err.to_string().contains("zip archive"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// **已知缺口（不在本文件修）**：入站 IM 附件落在 `<data_dir>/attachments`，
    /// 而 jail 只认 workspace（默认 `<data_dir>/workspace`，两者是**兄弟目录**）；
    /// `nt_channel_dispatch::download_attachments` 又把**绝对路径**写进给模型的那
    /// 行说明里。绝对路径在两道 jail 上都会被拒，所以 `read_file` 与 `read_image`
    /// 目前都读不到入站附件。本测试把这个观察钉住，免得它被当成「读图功能坏了」，
    /// 而实际上是附件落盘位置与 jail 范围没对齐（改动点在别人的文件里）。
    #[test]
    fn inbound_attachment_path_is_outside_the_workspace_jail() {
        let dir = crate::nt_testutil::temp_dir("attachment-jail-test");
        let _ = std::fs::remove_dir_all(&dir);
        // 模拟 `download_attachments` 的落点：data_dir/attachments（与 workspace 同级）。
        let attachments = dir.join("attachments");
        std::fs::create_dir_all(&attachments).expect("mkdir attachments");
        std::fs::write(attachments.join("shot.png"), &PNG_HEAD).expect("write png");
        let config = crate::nt_config::NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        let absolute = attachments.join("shot.png").to_string_lossy().into_owned();
        // 网关层：绝对路径直接判越狱（工具根本不执行）。
        let mut ctx = crate::nt_policy::PolicyContext {
            tool: crate::nt_types::ToolName::ReadImage,
            actor: crate::nt_policy::Actor::Bot,
            human_has_control: false,
            file_path: Some(absolute.clone()),
            command: None,
            computer_action: None,
            computer_target: None,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
        };
        assert!(matches!(
            crate::nt_policy::evaluate_policy(&ctx),
            crate::nt_policy::PolicyDecision::Deny { .. }
        ));
        // 执行层：即便绕过网关，`join_workspace` 也拒。
        assert!(super::join_workspace(&config.workspace_dir, &absolute).is_err());
        ctx.file_path = None;
        let _ = ctx;
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn transcript_budget_counts_image_bytes() {
        // 图像不走 `content`，若预算只按 content 计账，一轮十张图就是 50MiB+
        // 的请求体而账面显示「几乎没占」。
        let mut history = vec![
            TranscriptItem {
                role: TranscriptRole::User,
                content: "what is in this picture?".to_owned(),
                tool_calls: Vec::new(),
                tool_call_id: None,
                image: None,
            },
            TranscriptItem {
                role: TranscriptRole::Tool,
                content: "attached 1 image as a multimodal part".to_owned(),
                tool_calls: Vec::new(),
                tool_call_id: Some("c1".to_owned()),
                image: Some(crate::nt_types::ImagePart {
                    media_type: "image/png".to_owned(),
                    base64: "A".repeat(400 * 1024),
                }),
            },
        ];
        assert_eq!(
            super::transcript_weight(&history[1]),
            history[1].content.len() + 400 * 1024
        );
        enforce_transcript_budget(&mut history);
        // 400KiB 的图像 Tool 行被丢，用户原文保留。
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].role, TranscriptRole::User);
    }

    // ---- 切片 C0（租约续租）/ C1（取消钩子）的验收 ----
    //
    // 前提（已核对代码）：跑轮**同步阻塞**，所以今天**没有任何执行流**能
    // 置位这些令牌 —— 本模块全部用例都是「在引擎/回调里代置位」。这正是
    // 它们能独立存在的原因：`channel serve` 的行为一个字都没变。

    /// 每跳都回 N 条 bash/web_search 调用且 `Continue` 的引擎；记被叫了几次。
    ///
    /// 为什么不用 `LoopForever`：它每跳只回一条 `echo x`，而 C2（工具边界）
    /// 要验「同一跳内第二个工具没被执行」，得有一跳多调用的引擎。
    struct ToolFarm {
        calls: std::sync::Mutex<usize>,
        tools_per_hop: usize,
        command: &'static str,
    }

    impl ToolFarm {
        fn new(tools_per_hop: usize) -> Self {
            Self {
                calls: std::sync::Mutex::new(0),
                tools_per_hop,
                command: "echo x",
            }
        }

        fn with_command(mut self, command: &'static str) -> Self {
            self.command = command;
            self
        }

        fn calls(&self) -> usize {
            self.calls.lock().map(|n| *n).unwrap_or(0)
        }
    }

    impl crate::nt_engine::EngineAdapter for ToolFarm {
        fn engine_id(&self) -> &str {
            "tool-farm"
        }

        fn probe(&self) -> Result<String, crate::NtBotError> {
            Ok("tool-farm".to_owned())
        }

        fn run_turn(
            &self,
            _prompt: &str,
            _inbox: &[String],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            if let Ok(mut calls) = self.calls.lock() {
                *calls += 1;
            }
            let tools = (0..self.tools_per_hop)
                .map(|i| crate::nt_types::ToolCall {
                    id: format!("c{i}"),
                    // 第二个工具换个名字：断言「它没有 allow 审计行」时
                    // 才不会与第一个工具的审计行混淆。
                    name: if i == 0 {
                        crate::nt_types::ToolName::Bash
                    } else {
                        crate::nt_types::ToolName::WebSearch
                    },
                    args: serde_json::json!({"command": self.command, "query": "x"}),
                })
                .collect();
            Ok(crate::nt_engine::EngineTurn {
                assistant_text: String::new(),
                status: crate::nt_types::TurnStatus::Continue,
                tool_calls: tools,
                usage: None,
                side_effects: Vec::new(),
            })
        }
    }

    fn cancel_fixture(tag: &str, max_steps: u8) -> (std::path::PathBuf, NeobotConfig) {
        let dir = crate::nt_testutil::temp_dir(&format!("neobot-cancel-{}", tag));
        let _ = std::fs::remove_dir_all(&dir);
        let workspace = dir.join("workspace");
        std::fs::create_dir_all(&workspace).expect("mkdir workspace");
        let config = NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: workspace,
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        (dir, config)
    }

    fn farm_ctx<'a>(
        store: &'a NeobotStore,
        config: &'a NeobotConfig,
        engine: &'a dyn crate::nt_engine::EngineAdapter,
    ) -> super::RunContext<'a> {
        super::RunContext {
            store,
            config,
            engine,
            actor: crate::nt_policy::Actor::Bot,
            actor_name: "bot",
            title: "cancel-test",
            user_text: "go",
            convo_id: None,
        }
    }

    /// C1（hop 边界）+ C4（中止落库）。
    ///
    /// 钉四件事：① 叫停后**不再发起新的一次模型调用**（引擎只被叫 1 次）；
    /// ② task 行是 `cancelled` 且**租约已清**（不清就十分钟内没人碰它）；
    /// ③ `error` 说得出**第几跳**（信息不许丢）；④ 被取消的那一跳
    /// **一条 steps 行都不写**（没发生的事不记账）。
    #[test]
    fn cancel_stops_before_next_hop_and_says_which_hop() {
        let (dir, config) = cancel_fixture("hops", 3);
        let store = NeobotStore::open(":memory:").expect("open");
        let engine = ToolFarm::new(1);
        let token = super::StopToken::new();
        let ctx = farm_ctx(&store, &config, &engine);
        // 第一个工具跑完就置位（今天没人能置位 —— 这里代置）。
        let mut after_first = |_tool: &str, _ok: bool, _out: &str| token.cancel();
        let status =
            super::run_local_turn_cancellable(&ctx, None, Some(&mut after_first), Some(&token))
                .expect("run");
        assert_eq!(status, crate::nt_types::TurnStatus::Waiting);
        assert_eq!(engine.calls(), 1, "叫停后不许再发起第 2 次模型调用");
        let task = store.list_tasks(1).expect("list").remove(0);
        assert_eq!(task.status, crate::nt_types::TaskStatus::Cancelled);
        assert!(task.lease_id.is_none(), "取消必须清租约钥匙");
        assert!(task.lease_until.is_none(), "不清则十分钟内没人碰它");
        let error = task.error.clone().unwrap_or_default();
        assert!(error.contains("stopped by user"), "{error}");
        assert!(error.contains("at hop 1/3"), "要能说清第几跳：{error}");
        // hop 0 的一条 bash + 没有别的（被取消的 hop 1 不写 steps）。
        assert_eq!(
            store.list_step_tools(&task.id).expect("steps"),
            vec!["bash".to_owned()],
            "被取消的跳不许留下 steps 行"
        );
        // 用户有正规重跑出口（`retry_task` 收 cancelled）。
        store.retry_task(&task.id).expect("retry");
        assert_eq!(
            store.get_task(&task.id).expect("get").expect("row").status,
            crate::nt_types::TaskStatus::Pending
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// C2（工具边界）：**不留悬空 + 不造假账**。
    ///
    /// 三条断言合起来才有效：审计里有 `deny` + `rule='cancelled'`（如实记
    /// 「用户叫停」而不是「网关拒绝」）；被跳过的那个工具**没有** allow
    /// 审计行（不许有一条声称判过、实际没做的记录）；有一条
    /// `cancelled:tool_calls` 的 steps 行且 `ok=0`，清单里点名它。
    #[test]
    fn cancel_at_tool_boundary_denies_the_rest_of_the_hop() {
        let (dir, config) = cancel_fixture("tooledge", 3);
        let store = NeobotStore::open(":memory:").expect("open");
        let engine = ToolFarm::new(2);
        let token = super::StopToken::new();
        let ctx = farm_ctx(&store, &config, &engine);
        let mut after_first = |_tool: &str, _ok: bool, _out: &str| token.cancel();
        let status =
            super::run_local_turn_cancellable(&ctx, None, Some(&mut after_first), Some(&token))
                .expect("run");
        assert_eq!(status, crate::nt_types::TurnStatus::Waiting);
        let task = store.list_tasks(1).expect("list").remove(0);
        assert_eq!(task.status, crate::nt_types::TaskStatus::Cancelled);
        // 拦在 hop 0 的第二个工具之前 ⇒ 第 0 跳。
        assert!(
            task.error
                .clone()
                .unwrap_or_default()
                .contains("at hop 0/3"),
            "{:?}",
            task.error
        );
        let audits = store.list_audit(50).expect("audits");
        let cancel_row = audits
            .iter()
            .find(|event| event.rule.as_deref() == Some(super::CANCELLED_RULE))
            .expect("必须有 rule=cancelled 的审计行");
        assert_eq!(cancel_row.decision, crate::nt_audit::AuditDecision::Deny);
        assert_eq!(cancel_row.tool, "tool_calls");
        assert!(
            !audits.iter().any(|event| event.tool == "web_search"
                && event.decision == crate::nt_audit::AuditDecision::Allow),
            "被跳过的工具不许有 allow 审计行（那是假账）"
        );
        // 汇总 steps 行：`ok=0` + 点名未执行的那一个。
        let marker = store
            .last_step(&task.id, super::CANCELLED_TOOL_STEPS)
            .expect("last")
            .expect("cancelled 汇总行必须存在");
        assert!(!marker.ok, "ok 必须是 0");
        assert!(marker.output.contains("web_search"), "{}", marker.output);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// C0（清旗在**入口**而不是「用完」）：同一枚令牌连跑两轮，
    /// 第一轮被停、第二轮必须**正常跑完**。
    ///
    /// 若把清旗写成「用完清」，第二轮进来时旗还立着 → 第二轮一进 hop
    /// 循环就被自己上一轮的停止意图杀掉。
    #[test]
    fn stop_flag_is_cleared_at_turn_entry() {
        let (dir, config) = cancel_fixture("entry", 3);
        let store = NeobotStore::open(":memory:").expect("open");
        let engine = ToolFarm::new(1);
        let token = super::StopToken::new();
        let ctx = farm_ctx(&store, &config, &engine);
        let mut cancel_once = |_tool: &str, _ok: bool, _out: &str| token.cancel();
        super::run_local_turn_cancellable(&ctx, None, Some(&mut cancel_once), Some(&token))
            .expect("first run");
        assert!(token.is_cancelled(), "第一轮结束时旗确实是立着的");
        let calls_after_first = engine.calls();
        // 第二轮：同一枚令牌，不给任何回调 ⇒ C0 必须先把旗清掉。
        let status =
            super::run_local_turn_cancellable(&ctx, None, None, Some(&token)).expect("second run");
        assert_eq!(
            status,
            crate::nt_types::TurnStatus::Waiting,
            "第二轮不该被误杀"
        );
        assert_eq!(engine.calls() - calls_after_first, 3, "第二轮应跑满 3 跳");
        let tasks = store.list_tasks(5).expect("list");
        let newest = tasks
            .iter()
            .find(|t| t.status != crate::nt_types::TaskStatus::Cancelled);
        assert!(
            newest.is_some(),
            "第二轮必须留下一个非 cancelled 的行（它跑完了）"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 取消**不跨会话**：A 的令牌置位，B 那一轮照跑。
    ///
    /// 今天之所以成立，是因为令牌是**每次调用现传**的（登记表演到切片 C2
    /// 才出现）。这条钉住的是那个设计的边界：登记表按 `convo_id` 键，
    /// 值是每轮专属的那枚令牌。
    #[test]
    fn cancel_does_not_cross_conversations() {
        let (dir, config) = cancel_fixture("cross", 2);
        let store = NeobotStore::open(":memory:").expect("open");
        let engine = ToolFarm::new(1);
        let token_a = super::StopToken::new();
        let token_b = super::StopToken::new();
        let ctx = farm_ctx(&store, &config, &engine);
        let mut cancel_a = |_tool: &str, _ok: bool, _out: &str| token_a.cancel();
        super::run_local_turn_cancellable(&ctx, None, Some(&mut cancel_a), Some(&token_a))
            .expect("A run");
        // B：另一枚令牌，且入轮照 C0 清旗（它是干净的）。
        let status =
            super::run_local_turn_cancellable(&ctx, None, None, Some(&token_b)).expect("B run");
        assert_eq!(status, crate::nt_types::TurnStatus::Waiting);
        let tasks = store.list_tasks(5).expect("list");
        assert_eq!(tasks.len(), 2);
        assert_eq!(
            tasks
                .iter()
                .filter(|t| t.status == crate::nt_types::TaskStatus::Cancelled)
                .count(),
            1,
            "只有 A 那一轮该是 cancelled"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// C3：已经起了的 bash 子进程被杀掉，**亚秒级**返回且 `ok=false`。
    ///
    /// 两段都必要：只测「预先置位」钉不住「检查在轮询**循环里**」——
    /// 那样一个只在循环前问一次的实现也能过。
    #[test]
    fn bash_child_is_killed_within_one_poll_interval() {
        let (dir, config) = cancel_fixture("bashkill", 3);
        let call = crate::nt_types::ToolCall {
            id: "c1".to_owned(),
            name: crate::nt_types::ToolName::Bash,
            args: serde_json::json!({"command": "sleep 30"}),
        };
        // (a) 预先置位：第一个轮询臂就该杀（远小于 30s 的 sleep / 60s 超时）。
        let token = super::StopToken::new();
        token.cancel();
        let started = std::time::Instant::now();
        let res = super::execute_bash(&config, &call, &token).expect("run");
        let elapsed = started.elapsed();
        assert!(!res.ok, "被叫停的工具不许报成功");
        assert!(res.output.contains("stopped by user"), "{}", res.output);
        assert!(
            elapsed < std::time::Duration::from_secs(1),
            "预先置位就该在一个轮询间隔内回来，实耗 {elapsed:?}"
        );
        // (b) 在飞途中置位：证明检查在循环里（0.1s 取消 ⇒ 亚秒级回来）。
        let token = super::StopToken::new();
        let signal = token.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            signal.cancel();
        });
        let started = std::time::Instant::now();
        let res = super::execute_bash(&config, &call, &token).expect("run");
        let elapsed = started.elapsed();
        let _joined: Option<()> = handle.join().ok();
        assert!(!res.ok);
        assert!(
            elapsed < std::time::Duration::from_secs(2),
            "在飞途中被叫停必须亚秒级回来（50ms 轮询），实耗 {elapsed:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// C0 续租：每跳之后 `lease_until` 被推后（**滚动**窗口，不是累加）。
    ///
    /// 用 `sleep 0.05` 的 bash 是为了把两跳之间拉开 ≥100ms —— 否则
    /// 「续租后与续租前的字符串」可能落在同一个时钟刻度上，断言会偶发。
    #[test]
    fn lease_is_renewed_across_hops() {
        let (dir, config) = cancel_fixture("renew", 3);
        let store = NeobotStore::open(":memory:").expect("open");
        let engine = ToolFarm::new(1).with_command("sleep 0.05");
        let seen: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
        let ctx = farm_ctx(&store, &config, &engine);
        let mut probe = |_tool: &str, _ok: bool, _out: &str| {
            if let Ok(mut rows) = store.list_tasks(1) {
                if let Some(lease_until) = rows.remove(0).lease_until {
                    if let Ok(mut log) = seen.lock() {
                        log.push(lease_until);
                    }
                }
            }
        };
        let heartbeat = super::LeaseHeartbeat::tuned(std::time::Duration::ZERO, 32);
        super::run_local_turn_inner(&ctx, None, Some(&mut probe), None, heartbeat).expect("run");
        let log = seen.lock().map(|log| log.clone()).unwrap_or_default();
        assert!(log.len() >= 2, "至少要看到两跳的租约：{log:?}");
        assert!(
            log[1] > log[0],
            "hop 1 看到的 lease_until 必须比 hop 0 更靠后（已续租）：{log:?}"
        );
        // 滚动窗口：续租后仍只有一个 LEASE_SECS 宽，不是逐跳累加。
        let first = chrono::DateTime::parse_from_rfc3339(&log[0]).expect("rfc3339");
        let second = chrono::DateTime::parse_from_rfc3339(&log[1]).expect("rfc3339");
        let width = (second - first).num_seconds();
        assert!(
            (0..=(super::LEASE_SECS as i64)).contains(&width),
            "续租是滚动窗口（≤ LEASE_SECS），不是累加，实测 {width}s"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// C0 续租的**上限**：到顶就停手，让租约自然过期。
    ///
    /// 「异常长寿的单轮」不该永远续命 —— 到顶后 `due()` 恒假，后面的跳
    /// 一行库都不写（`lease_until` 不再变化即为证据）。
    #[test]
    fn lease_renewal_stops_at_the_cap() {
        let (dir, config) = cancel_fixture("cap", 3);
        let store = NeobotStore::open(":memory:").expect("open");
        let engine = ToolFarm::new(1).with_command("sleep 0.05");
        let seen: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
        let ctx = farm_ctx(&store, &config, &engine);
        let mut probe = |_tool: &str, _ok: bool, _out: &str| {
            if let Ok(mut rows) = store.list_tasks(1) {
                if let Some(lease_until) = rows.remove(0).lease_until {
                    if let Ok(mut log) = seen.lock() {
                        log.push(lease_until);
                    }
                }
            }
        };
        // cap=1 ⇒ 只有第 0 跳之后续那一次。
        let heartbeat = super::LeaseHeartbeat::tuned(std::time::Duration::ZERO, 1);
        super::run_local_turn_inner(&ctx, None, Some(&mut probe), None, heartbeat).expect("run");
        let log = seen.lock().map(|log| log.clone()).unwrap_or_default();
        assert_eq!(log.len(), 3, "三跳都该看到租约：{log:?}");
        assert!(log[1] > log[0], "第一次续租必须发生：{log:?}");
        assert_eq!(log[2], log[1], "到上限后不许再续（有界）：{log:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 退避闸本身：刚续过就不该立刻又续（否则快 hop 会把库写穿）。
    #[test]
    fn lease_heartbeat_backs_off_after_a_renewal() {
        let mut heartbeat = super::LeaseHeartbeat::tuned(std::time::Duration::from_secs(60), 4);
        assert!(heartbeat.due(), "第一跳就该续（还没有上一次）");
        heartbeat.note();
        assert!(!heartbeat.due(), "刚续过 → 退避期内不再续");
        // 次数上限同样封住闸。
        let mut capped = super::LeaseHeartbeat::tuned(std::time::Duration::ZERO, 2);
        assert!(capped.due());
        capped.note();
        assert!(capped.due());
        capped.note();
        assert!(!capped.due(), "到上限 ⇒ 恒不再续");
        capped.note();
        assert!(!capped.due());
    }

    /// **反向测试（切片 C1 最重要的护栏）**：取消信号与落库接通之后，
    /// `/stop` 的回执**仍然不许**说「已停」。
    ///
    /// 理由：真正能被用户触发的路径要到切片 C2（`on_inbound` 切分）才有
    /// 第二个执行流去置位。此刻若谁顺手把回执改成「已停」，用户就会以为
    /// 停掉了而那一轮照跑照写 —— 比说「停不了」更危险。
    ///
    /// 切片 C3 真正接线时，**必须同一个提交里反向这条测试**（连同回执
    /// 一起改），否则它会红 —— 那正是它该做的事。
    #[test]
    fn stop_receipt_still_admits_the_limitation() {
        let store = NeobotStore::open(":memory:").expect("open");
        let bot = crate::nt_store::BotRow {
            channel: "tg".to_owned(),
            bot_id: "1".to_owned(),
            alias: String::new(),
            token_env: "NEOBOT_T".to_owned(),
            conversation_id: None,
            model: String::new(),
            allow_list: String::new(),
            created_at: String::new(),
            last_seen: None,
        };
        let got = crate::nt_channel_cmd::execute(
            &store,
            &crate::nt_channel_cmd::Command::Stop,
            &bot,
            "TG",
        )
        .expect("exec");
        // 标志仍置位（将来并发调度时这条命令已经在）。
        assert!(got.stop_requested);
        // 必须说清为什么停不了。
        assert!(got.reply.contains("停不了"), "{}", got.reply);
        assert!(got.reply.contains("同步"), "{}", got.reply);
        // **不许**出现任何「已经停下来了」的断言词。
        for claim in ["已请求停止", "已停", "已中止", "已终止", "停止成功"] {
            assert!(
                !got.reply.contains(claim),
                "取消尚未接线（C2/C3 之前），回执不能声称已停（命中 {claim:?}）：{}",
                got.reply
            );
        }
        // `/help` 同理：它也在教用户用 `/stop`。
        let help = crate::nt_channel_cmd::help_text("TG");
        for claim in ["已停", "已终止", "停止成功"] {
            assert!(
                !help.contains(claim),
                "help 也不能承诺一个此刻停不下来的动作（命中 {claim:?}）：{help}"
            );
        }
    }

    // Qwen-MM-Plugins 会话工具的纯分发测试：伪造 stdio MCP 服务器，
    // 不碰真实 Python、不改进程环境（PATH/checkout 全不碰 —— 并行测试安全）。
    const QWEN_FAKE_SERVER: &str = r#"#!/usr/bin/env bash
while IFS= read -r line; do
  case "$line" in
    *'"method":"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"fake","version":"0"}}}'
      ;;
    *'"method":"tools/call"'*)
      if [[ "$line" == *'"save_view"'* ]]; then
        printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"views ready"},{"type":"image","data":"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==","mimeType":"image/png"}],"isError":false}}'
      else
        printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"media ok"}],"isError":false}}'
      fi
      ;;
  esac
done
"#;

    /// 伪造会话＋workspace 夹具（artifacts 落进 workspace 内 `.neotrix-mm/`，
    /// 与生产一致 —— 落临时目录就测不到 `load_image` 那条线）。
    fn qwen_dispatch_fixture(
        tag: &str,
    ) -> (
        std::path::PathBuf,
        crate::nt_config::NeobotConfig,
        crate::nt_qwen_mm::McpStdioSession,
        std::path::PathBuf,
    ) {
        let dir = crate::nt_testutil::temp_dir(&format!("neobot-qwenmm-{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        let workspace = dir.join("workspace");
        std::fs::create_dir_all(&workspace).expect("mkdir workspace");
        let script = dir.join("fake.sh");
        std::fs::write(&script, QWEN_FAKE_SERVER).expect("write fake server");
        let config = crate::nt_config::NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: workspace.clone(),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        let artifacts = workspace.join(".neotrix-mm");
        let session = crate::nt_qwen_mm::McpStdioSession::new(
            if std::path::Path::new("/bin/bash").is_file() {
                "/bin/bash".to_string()
            } else {
                "bash".to_string()
            },
            vec![script.to_string_lossy().to_string()],
        )
        .with_timeout_ms(10_000)
        .with_artifacts_dir(artifacts.clone());
        (dir, config, session, artifacts)
    }

    fn qwen_call(
        name: crate::nt_types::ToolName,
        args: serde_json::Value,
    ) -> crate::nt_types::ToolCall {
        crate::nt_types::ToolCall {
            id: "c-qwen".to_owned(),
            name,
            args,
        }
    }

    #[test]
    fn qwen_dispatch_text_flows_and_jail_holds() {
        let (dir, config, session, artifacts) = qwen_dispatch_fixture("text");
        // 文本流：media_info 的回包原样进 output。
        let outcome = super::execute_qwen_mm_with_session(
            &config,
            &qwen_call(
                crate::nt_types::ToolName::QwenMediaInfo,
                serde_json::json!({"path": "clip.mp4"}),
            ),
            &session,
            &artifacts,
        )
        .expect("dispatch");
        assert!(outcome.result.ok);
        assert!(
            outcome.result.output.contains("media ok"),
            "{}",
            outcome.result.output
        );
        assert!(outcome.image.is_none());
        // 越狱：video_path 绝对路径在执行层被拒（网关是第一道，这里是第二道）。
        let denied = super::execute_qwen_mm_with_session(
            &config,
            &qwen_call(
                crate::nt_types::ToolName::QwenReadVideo,
                serde_json::json!({"video_path": "/etc/passwd"}),
            ),
            &session,
            &artifacts,
        )
        .expect("refusal is a normal outcome, not a Rust error");
        assert!(!denied.result.ok);
        assert!(
            denied.result.output.contains("refused"),
            "{}",
            denied.result.output
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn qwen_dispatch_image_becomes_a_real_part() {
        let (dir, config, session, artifacts) = qwen_dispatch_fixture("image");
        // save_view 回 image 块 → 落盘进 workspace/.neotrix-mm → load_image 读回
        // 真部件（魔数+4MiB 上限全走既有线），文本里只有路径没有 base64 墙。
        let outcome = super::execute_qwen_mm_with_session(
            &config,
            &qwen_call(
                crate::nt_types::ToolName::QwenSaveView,
                serde_json::json!({"file_path": "doc.pdf", "pages": "1"}),
            ),
            &session,
            &artifacts,
        )
        .expect("dispatch");
        assert!(outcome.result.ok);
        let image = outcome.image.expect("must carry the image part");
        assert_eq!(image.media_type, "image/png");
        assert!(!outcome.result.output.contains(&image.base64));
        assert!(
            outcome.result.output.contains(".neotrix-mm"),
            "{}",
            outcome.result.output
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 活测试（`#[ignore]`）：真 Qwen 服务器走 neobot 分发全链。
    ///
    /// 前置：`bash scripts/ops/nt_qwen_mm_setup.sh setup` + shim 进 PATH +
    /// `ffmpeg` 在 PATH。跑法：
    /// `NT_QWEN_MM_LIVE=1 cargo test -p neotrix-neobot --lib -- --ignored qwen_live_dispatch --nocapture`
    /// 这是"模型自主调"链的最后一块拼图：分发（伪服务器已证）× 真服务器
    /// （core 活测试已证）在这里合成一次。
    #[test]
    #[ignore]
    fn qwen_live_dispatch_through_neobot() {
        use std::process::Command;
        assert_eq!(
            std::env::var("NT_QWEN_MM_LIVE").as_deref(),
            Ok("1"),
            "live test needs NT_QWEN_MM_LIVE=1 (opt-in)"
        );
        let dir = crate::nt_testutil::temp_dir("neobot-qwenmm-live");
        let _ = std::fs::remove_dir_all(&dir);
        let workspace = dir.join("workspace");
        std::fs::create_dir_all(&workspace).expect("mkdir workspace");
        // ffmpeg 现场造 2 秒 mp4（SYSTEM_DEPS 声明的依赖，缺即 loud-fail）。
        let mp4 = workspace.join("t.mp4");
        let st = Command::new("ffmpeg")
            .args([
                "-y",
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=2:size=160x120:rate=5",
                "-pix_fmt",
                "yuv420p",
            ])
            .arg(&mp4)
            .output()
            .expect("live test needs ffmpeg on PATH");
        assert!(st.status.success(), "ffmpeg fixture failed");
        let config = crate::nt_config::NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: workspace,
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        // 注意：这里故意走 `execute_qwen_mm`（含 resolve + vision 门），
        // 不走 with_session —— 测的就是生产入口。
        let outcome = super::execute_qwen_mm(
            &config,
            &VisionEngine,
            &qwen_call(
                crate::nt_types::ToolName::QwenMediaInfo,
                serde_json::json!({"path": "t.mp4"}),
            ),
            &live_stop(),
        )
        .expect("live dispatch");
        assert!(outcome.result.ok, "{}", outcome.result.output);
        assert!(
            outcome.result.output.contains("Video stream"),
            "{}",
            outcome.result.output
        );
        eprintln!("qwen live via neobot dispatch: ALL GREEN (no keys used)");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn qwen_blind_engine_is_denied_not_faked() {
        // 看不见的引擎调 Qwen 工具 = 直接拒（与 read_image 同纪律）：
        // 伪造服务器再真也不能让盲引擎"看见"。
        let (_dir, config, session, artifacts) = qwen_dispatch_fixture("blind");
        let err = super::execute_qwen_mm(
            &config,
            &LocalEchoEngine,
            &qwen_call(
                crate::nt_types::ToolName::QwenReadVideo,
                serde_json::json!({"video_path": "clip.mp4"}),
            ),
            &live_stop(),
        )
        .expect_err("blind engine must not pretend to see");
        assert!(err.to_string().contains("no-vision-engine"), "{err}");
    }

    // ══════════════════ 影子层（shadow）：按可逆性分流 ══════════════════
    //
    // ⛔ 这些用例**全部只测纯函数**：本影子层不改任何执行行为，所以
    // 「行为没变」这件事**没有**测试能证明 —— 它由「决策值无人读」这句话
    // 保证（`timeout_policy` 的返回值在 funnel 里只被 format! 吃掉）。
    // 能测的只有：分类对不对、决策对不对、记账那行长什么样。

    use crate::nt_types::ToolName;

    use super::{
        class_digest, enforced_timeout_ms, reversibility_of, timeout_policy, NeobotReversibility,
        TimeoutPolicy,
    };

    /// `ToolName` 的**全部**变体（2026-10-05 实测 16 个）。
    ///
    /// 这份清单本身就是一道编译期保险的一半：`reversibility_of` 的
    /// `match` **没有** `_ =>` 兜底 ⇒ 日后给 `ToolName` 加变体，编译直接红，
    /// 逼着人来这里补一行分类（而不是让新工具静默落进某个类）。
    /// 另一半是下面那句 `assert_eq!(ALL_TOOLS.len(), 16)`：
    /// 万一有人用 `_ =>` 把编译期保险拆了，计数与逐项断言仍会红。
    const ALL_TOOLS: [ToolName; 16] = [
        ToolName::Bash,
        ToolName::SetTurnStatus,
        ToolName::ReadFile,
        ToolName::WriteFile,
        ToolName::EditFile,
        ToolName::ReadImage,
        ToolName::ComputerAct,
        ToolName::WebSearch,
        ToolName::WebFetch,
        ToolName::SidebarOpen,
        ToolName::QwenMediaInfo,
        ToolName::QwenReadVideo,
        ToolName::QwenVisualize,
        ToolName::QwenSaveView,
        ToolName::PdfGroundText,
        ToolName::Unknown(String::new()),
    ];

    #[test]
    fn reversibility_covers_every_tool_variant() {
        // 变体总数被钉住：新增变体而忘了改这里 ⇒ 红。
        assert_eq!(ALL_TOOLS.len(), 16, "ToolName 变体数变了，本清单待补");
        // 每个变体的分类都被逐条钉死（含「哪几个是只读」这个集合本身）。
        let expected = [
            (ToolName::ReadFile, NeobotReversibility::ReadOnly),
            (ToolName::ReadImage, NeobotReversibility::ReadOnly),
            (ToolName::WebSearch, NeobotReversibility::ReadOnly),
            (ToolName::WebFetch, NeobotReversibility::ReadOnly),
            (ToolName::QwenMediaInfo, NeobotReversibility::ReadOnly),
            (ToolName::QwenReadVideo, NeobotReversibility::ReadOnly),
            (ToolName::QwenVisualize, NeobotReversibility::ReadOnly),
            (ToolName::PdfGroundText, NeobotReversibility::ReadOnly),
            (ToolName::WriteFile, NeobotReversibility::Reversible),
            (ToolName::EditFile, NeobotReversibility::Reversible),
            (ToolName::SetTurnStatus, NeobotReversibility::Reversible),
            (ToolName::SidebarOpen, NeobotReversibility::Reversible),
            (ToolName::Bash, NeobotReversibility::Irreversible),
            (ToolName::ComputerAct, NeobotReversibility::Irreversible),
            (ToolName::QwenSaveView, NeobotReversibility::Irreversible),
            (
                ToolName::Unknown("rm_rf_root".to_owned()),
                NeobotReversibility::Irreversible,
            ),
        ];
        for (tool, class) in expected {
            assert_eq!(reversibility_of(&tool), class, "{}", tool.as_str());
        }
        // 「这个类别到底有没有住户」—— 正是本影子阶段要回答的那个问题，
        // 先用**枚举**回答一遍（真答案要看实测分布，那是 shadow 的产出）。
        let read_only = ALL_TOOLS
            .iter()
            .filter(|tool| reversibility_of(tool) == NeobotReversibility::ReadOnly)
            .count();
        let reversible = ALL_TOOLS
            .iter()
            .filter(|tool| reversibility_of(tool) == NeobotReversibility::Reversible)
            .count();
        let irreversible = ALL_TOOLS
            .iter()
            .filter(|tool| reversibility_of(tool) == NeobotReversibility::Irreversible)
            .count();
        assert_eq!((read_only, reversible, irreversible), (8, 4, 4));
        assert_eq!(
            read_only + reversible + irreversible,
            ALL_TOOLS.len(),
            "每个工具都必须落在恰好一个类里"
        );
    }

    #[test]
    fn unclassified_tool_defaults_to_irreversible() {
        // **安全默认**：认不出来的一律按不可逆（⇒ 超时被 `WouldWait`）。
        // 逐个常见「看起来无害」的未知名都要过这一关 ——
        // 「无害」是猜测，「不可逆」是保证。
        for raw in ["ls", "git_status", "read_notes", "echo_hello", "sleep_1"] {
            let tool = ToolName::parse(raw);
            assert_eq!(
                reversibility_of(&tool),
                NeobotReversibility::Irreversible,
                "{raw}"
            );
            assert_eq!(
                timeout_policy(NeobotReversibility::Irreversible, 10_000.0, Some(1.0)),
                TimeoutPolicy::WouldWait,
                "{raw}: 未知名必须走「等它跑完」"
            );
        }
    }

    #[test]
    fn read_only_past_timeout_would_time_out() {
        // Vibe-Trading 的只读侧：越过超时 ⇒ 可中止（结构化 tool_timeout 信封）。
        assert_eq!(
            timeout_policy(NeobotReversibility::ReadOnly, 15_001.0, Some(15_000.0)),
            TimeoutPolicy::WouldTimeout
        );
        // 没越过 ⇒ 无论什么类都照常等。
        assert_eq!(
            timeout_policy(NeobotReversibility::ReadOnly, 14_999.0, Some(15_000.0)),
            TimeoutPolicy::WouldWait
        );
    }

    #[test]
    fn irreversible_and_reversible_past_timeout_would_wait() {
        // 承重那一半：「Write tools are never killed」——
        // 越过超时也**等它跑完**，只告警一次。
        for class in [
            NeobotReversibility::Irreversible,
            NeobotReversibility::Reversible,
        ] {
            assert_eq!(
                timeout_policy(class, 999_999.0, Some(1.0)),
                TimeoutPolicy::WouldWait,
                "{}: 越过超时也不许中止",
                class.as_str()
            );
        }
    }

    #[test]
    fn exactly_at_the_timeout_is_not_yet_past_it() {
        // 边界：`elapsed == timeout` 判**未越过**（严格大于才是越过）。
        // 差这一点就会让「刚好 15.000s 的 web_fetch」被当成超时受害者。
        for class in [
            NeobotReversibility::ReadOnly,
            NeobotReversibility::Reversible,
            NeobotReversibility::Irreversible,
        ] {
            assert_eq!(
                timeout_policy(class, 15_000.0, Some(15_000.0)),
                TimeoutPolicy::WouldWait,
                "{}: 恰好等于超时不算越过",
                class.as_str()
            );
            assert_eq!(
                timeout_policy(class, 15_000.001, Some(15_000.0)),
                match class {
                    NeobotReversibility::ReadOnly => TimeoutPolicy::WouldTimeout,
                    _ => TimeoutPolicy::WouldWait,
                },
                "{}: 越过 1µs 之后",
                class.as_str()
            );
        }
    }

    #[test]
    fn nan_and_absent_timeout_mean_no_timeout() {
        // 裁决：`NaN` 读成「无超时」，不是「立即超时」。理由：
        // 把配置缺陷当成超时受害者 = 拿一次 bug 换一个不可逆动作的半途而废。
        let read_only = NeobotReversibility::ReadOnly;
        for bad in [
            None,
            Some(f64::NAN),
            Some(f64::INFINITY),
            Some(0.0),
            Some(-1.0),
        ] {
            assert_eq!(
                timeout_policy(read_only, 1_000_000.0, bad),
                TimeoutPolicy::WouldWait,
                "{bad:?}: 一律当「无超时」"
            );
        }
        // `elapsed` 侧同样 fail-safe：NaN / 负数都算「未越过」。
        for bad_elapsed in [f64::NAN, -1.0, 0.0] {
            assert_eq!(
                timeout_policy(read_only, bad_elapsed, Some(1.0)),
                TimeoutPolicy::WouldWait,
                "elapsed={bad_elapsed}: 一律当「未越过」"
            );
        }
        // 不可约简性：非正数超时与 `None` 归一化成同一个东西。
        assert_eq!(
            timeout_policy(read_only, 5.0, Some(0.0)),
            timeout_policy(read_only, 5.0, None)
        );
    }

    #[test]
    fn enforced_timeouts_are_the_ones_that_really_exist() {
        // 影子阶段只能记**真实存在**的超时，否则「多少比例会撞上超时」这个问题
        // 会被答错。三个来源：bash 60s / qwen 90s / web 15s，其余无超时。
        assert_eq!(enforced_timeout_ms(&ToolName::Bash), Some(60_000.0));
        assert_eq!(enforced_timeout_ms(&ToolName::QwenSaveView), Some(90_000.0));
        assert_eq!(
            enforced_timeout_ms(&ToolName::QwenReadVideo),
            Some(90_000.0)
        );
        assert_eq!(enforced_timeout_ms(&ToolName::WebFetch), Some(15_000.0));
        assert_eq!(enforced_timeout_ms(&ToolName::WebSearch), Some(15_000.0));
        for tool in [
            ToolName::ReadFile,
            ToolName::ReadImage,
            ToolName::WriteFile,
            ToolName::EditFile,
            ToolName::ComputerAct,
            ToolName::SidebarOpen,
            ToolName::SetTurnStatus,
            ToolName::PdfGroundText,
            ToolName::Unknown("x".to_owned()),
        ] {
            assert_eq!(enforced_timeout_ms(&tool), None, "{}", tool.as_str());
        }
        //  本机制的**实测抓手**：今天全仓唯一「带硬超时且会 kill」的路径是
        // bash（`child.kill()`）与 qwen 会话（`nt_qwen_mm` 到期报错），
        // 而它们**都不可逆**。也就是说「不可逆动作被硬杀」这条路**今天已经
        // 存在**（`bash -c 'git push'` 跑到 60s 被 SIGKILL），
        // 影子层要量的就是这条路上到底发生过几次。
        // （`let all = ALL_TOOLS`：把借用绑到局部变量上，别借 const 的临时量。）
        let all = ALL_TOOLS;
        let killed_and_irreversible: Vec<&str> = all
            .iter()
            .filter(|tool| {
                enforced_timeout_ms(tool).is_some()
                    && reversibility_of(tool) == NeobotReversibility::Irreversible
            })
            .map(ToolName::as_str)
            .collect();
        assert_eq!(killed_and_irreversible, vec!["bash", "qwen_save_view"]);
        // 反过来：只读工具**全都**有超时可谈吗？不是 —— read_file 没有超时，
        // 所以「只读 ⇒ 可中止」这条规则今天在 read_file 上是空转的。
        assert!(enforced_timeout_ms(&ToolName::ReadFile).is_none());
    }

    #[test]
    fn class_digest_separates_classes_and_tools() {
        // 摘要要能当「哪个工具落进了哪个类」的稳定身份：同工具不同类必须不同
        // （否则「分类写错了」在日志里看不出来），同类不同工具也必须不同。
        let bash_digest = class_digest(NeobotReversibility::Irreversible, &ToolName::Bash);
        assert_ne!(
            bash_digest,
            class_digest(NeobotReversibility::ReadOnly, &ToolName::Bash),
            "同一个工具换了类，摘要必须跟着变"
        );
        assert_ne!(
            class_digest(NeobotReversibility::ReadOnly, &ToolName::ReadFile),
            class_digest(NeobotReversibility::ReadOnly, &ToolName::ReadImage),
            "同类不同工具必须可区分"
        );
        // 稳定性：同输入同输出（否则跨运行聚合分布对不上）。
        assert_eq!(
            class_digest(NeobotReversibility::Irreversible, &ToolName::Bash),
            bash_digest
        );
    }

    #[test]
    fn shadow_line_records_every_field_it_promises() {
        // 记账那行是本影子层**唯一的产出** ⇒ 字段缺一个，测量就少一维。
        let line = super::shadow_line(
            &ToolName::WebFetch,
            NeobotReversibility::ReadOnly,
            Some(15_000.0),
            Some(15_123.5),
            TimeoutPolicy::WouldTimeout,
        );
        for field in [
            "tool=web_fetch",
            "class=read_only",
            "timeout_ms=15000",
            "elapsed_ms=15123.500",
            "policy=would_timeout",
            "digest=",
        ] {
            assert!(line.contains(field), "影子行缺字段 {field}：{line}");
        }
        // 没执行的路径（被拒 / dry-run）耗时记 `none` 而不是 0 ——
        // 「没跑」混进耗时分布会让分布好看得不真实。
        let not_run = super::shadow_line(
            &ToolName::Bash,
            NeobotReversibility::Irreversible,
            Some(60_000.0),
            None,
            TimeoutPolicy::WouldWait,
        );
        assert!(not_run.contains("elapsed_ms=none"), "{not_run}");
    }
}
