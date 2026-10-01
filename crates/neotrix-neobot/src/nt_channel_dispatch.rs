//! `nt_channel_dispatch` — 渠道消息的进出调度。
//!
//! **入站链路**（`on_inbound`）：
//! `poll` → 逐条：去重 → 访问闸门 → 斜杠指令 → （不是指令才）落到会话并跑轮
//!
//! **出站链路**（`drain_outbox_once`）：
//! outbox → 按渠道分派 → 发 → 成功删 / 失败退避
//!
//! **补发链路**（`sweep_pending`）：
//! 超时之后原任务跑完了 → 把结果补回原聊天；到顶不再自动重试
//!
//! 降级律：附件发不出去**不降级**，整条算失败退回 outbox。
//! 早先这里写的是「降为纯文字并如实回报」—— 但出站这一层**没有**「回报」的
//! 通道（`drain_outbox_once` 只回两个计数），所谓「如实回报」实际就是
//! 静默丢文件：用户收到一段文字，以为文件也到了。故宁可让消息带着退避
//! 重发一轮，也不能让「已发送」与事实不符。真正需要降级的地方是**入站**
//! （那里能把「没取到」写进给模型的正文，见 [`download_attachments`]）。
//!
//! 附件载体律：outbox 的 payload 里 `attachments` 是**本地路径数组**，
//! 与 `OutboundMessage.attachments` 同一口径。空数组与缺字段等价（老 payload
//! 没有这个键也照发）。空正文**且**无附件才算没东西可发 —— 只有附件
//! （一张图零解释）是完全合法的出站。
//!
//! 编辑载体律：payload 里 `edit_of`（**可选键**）是要改的那条**平台**消息
//! id，缺省等价于 `None`/发新消息。老 payload 里没有这个键，必须照发不误
//! —— 把「缺这个键」判成错误会让历史队列整片退避，那是加个可选键就能引发的
//! 雪崩。写这个键的入口是 [`enqueue_outbound_editing`]，读的是
//! [`payload_edit_of`]。
//!
//! **「编辑」在本模块的边界**：排进队列 / 直发两处都只负责**把意图带到位**，
//! 能不能真改成是渠道的事（Telegram 只让 bot 改自己发过的消息）。改不成时
//! 渠道退回发新消息并**记账**，本模块不假装它编辑过 —— 也拿不到那个记账
//! （`ChannelAdapter::send` 只能回一个消息 id），这是 trait 签名的已知盲区。
//!
//! ## 停止信号（`/stop`）：管道已接通，**兑现仍缺并发**
//!
//! 停一轮需要三样东西同时到位：① 一枚跨执行流可见的令牌（[`StopToken`]，
//! `nt_cancel.rs`）；② 一张**按会话索引**的表，让「命令入口」能找到「跑轮」
//! 手上那一枚（`RunSlot` 表 + RAII 注销）；③ 令牌真的被递进跑轮
//! （[`RunContext`] + `run_local_turn_cancellable`，四个检查点在 `nt_agent`）。
//! **本切片把 ②③ 接上了**，于是 `/stop` 不再置一个没人看的旗，而是去翻
//! 跑轮**正握着**的那一枚。
//!
//! **但 IM 上它此刻仍兑现不了，缺的是并发调度** —— 这一条不许被任何注释、
//! 文案或测试名说反了：
//!
//! - `channel serve` 是**同步阻塞**的：一条消息的跑轮不返回，下一条消息
//!   **根本不会被 poll 到**（见 `nt_channel_serve::run_once`）。
//! - 于是用户按下的 `/stop` 躺在平台队列里，等**这一轮跑完**才被读出来；
//!   读出来时 [`RunRegistration`] 早已析构、登记表已空 ⇒ 如实答案是
//!   「**没有可停的轮次**」，而不是「已停」。
//! - 要让它真能中途停，得把调度改成并发的（线程池 / async，跑轮与收取解耦）。
//!   那是架构级一步，不是本切片能顺手做的。
//!
//! 桌面侧为什么能真停：它的 `/stop` 是**另一次** IPC 调用，而上一轮还跑在
//! `spawn_blocking` 里 ⇒ 天然并发（`apps/neobot-desktop/…/nt_cmd_run.rs`）。
//! 本模块是桌面的**同构**实现（同一个 `StopToken`、同一套登记表口径、
//! 同样「找不到就如实说、绝不静默成功」），差别只在调度形状，不在取消机制。
//!
//! **信号送达 ≠ 轮次已停**：这两个是不同的事实，本模块用 [`StopState`] 的
//! 三个变体分别说，不许互相顶替（见该类型的注释）。

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use chrono::Utc;

use crate::nt_agent::{RunContext, run_local_turn_cancellable};
use crate::nt_cancel::StopToken;
use crate::nt_channel::{
    AccessMode, ChannelAdapter, InboundMessage, OutboundMessage, admits, admit_reason, dedup_key,
};
use crate::nt_channel_cmd;
use crate::nt_config::NeobotConfig;
use crate::nt_error::NtBotError;
use crate::nt_store::{NeobotStore, parse_allow_list};
use crate::nt_types::TaskStatus;

/// 一条入站消息的处理结果（供测试与上层日志断言）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundOutcome {
    /// 见过重复，丢弃。
    Duplicate,
    /// 没过访问闸门，丢弃（带人话理由）。
    Rejected { reason: String },
    /// 是斜杠指令，已执行并回执（不跑轮）。
    Handled { command: String, reply: String },
    /// 已落成任务并跑轮。
    Turn { task_id: String, status: String },
}

// ═══ 停止信号：登记表 + 三态 ═══
//
// ## 为什么需要一张表
//
// `StopToken` 是**跨执行流的信号载体**（`Arc<AtomicBool>`，克隆共享一面旗）。
// 跑轮那侧把它**传进去**，停止那侧要能**找到它**并翻它 —— 二者在 IM 上是
// 两次不同的 `on_inbound` 调用（一条消息一次），中间隔着这张表。没有它，
// `/stop` 就只能置一个**没人读**的本地变量（这正是被删掉的 `stop_flag`
// 形参干的事：它装得下一个 `bool`，装不下一枚 `StopToken`，于是永远传
// `None`，也永远停不了任何东西 —— 详见 [`on_inbound`] 的签名注释）。
//
// ## 为什么按 `convo_id` 索引（不是 `task_id`）
//
// 与桌面侧同一条理由（`apps/neobot-desktop/…/nt_cmd_run.rs`）：任务行是
// `run_local_turn_inner` 在函数**内部**建的，跑轮期间没有任何回调把 id 递
// 出来。而 `/stop` 到达时我们手里只有「这个机器人绑在哪个会话」—— 那正是
// 这一轮跑在里面的会话。按 `task_id` 索引**做不到**，不是「还没做」。
//
// ## 并发安全
//
// - 表是 `Mutex<HashMap>`，**只在毫秒级读改写期间持锁**：不跨跑轮、
//   不跨任何 IO。跑轮本体拿的是令牌的**克隆**，不持锁跑完全程。
// - **同键覆盖 + 世代号**：新登记的直接盖掉旧的；守卫析构时**只在世代号
//   仍是自己**时删，否则先发起的旧轮次收尾会把新一轮的令牌**误删**
//   （那会让新一轮变得「停不了」而没人察觉）。
// - 锁毒（poisoned）时**如实说锁坏了**，绝不 panic —— 残留一格令牌的后果
//   只是「停止报找不到」，而 panic 会掀翻整轮。

/// 登记表的一格：一枚令牌 + 一个世代号。
struct RunSlot {
    token: StopToken,
    seq: u64,
}

static RUN_REGISTRY: OnceLock<Mutex<HashMap<String, RunSlot>>> = OnceLock::new();
static RUN_SEQ: AtomicU64 = AtomicU64::new(0);

fn run_registry() -> &'static Mutex<HashMap<String, RunSlot>> {
    RUN_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 登记键：会话 id。**没有会话就不可取消**（如实不登记，而不是编一个键）。
fn run_key(convo_id: Option<&str>) -> Option<String> {
    convo_id
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
}

/// RAII 登记守卫：析构时按世代号注销。
///
/// 刻意**不**无条件 `remove(key)`：那会让先发起的旧轮次在收尾时删掉新一轮
/// 的令牌（新一轮于是「停不了」，而没人知道为什么）。
struct RunRegistration {
    key: Option<String>,
    seq: u64,
}

impl Drop for RunRegistration {
    fn drop(&mut self) {
        let Some(key) = self.key.as_ref() else {
            return;
        };
        // 锁毒：登记期间若被毒化，注销失败也不该把 panic 传染出去 ——
        // 残留一格令牌的后果只是「停止报找不到」，而 panic 会掀翻整轮。
        let Ok(mut table) = run_registry().lock() else {
            return;
        };
        if table.get(key).is_some_and(|slot| slot.seq == self.seq) {
            table.remove(key);
        }
    }
}

/// 为这一轮登记一枚可被 `/stop` 翻转的令牌，返回守卫（**必须**活到跑轮收尾）。
///
/// 登记点刻意在**建跑轮上下文之前**：那一刻起（含「任务已建、轮次在跑」那一段）
/// 到收尾，整段都有一枚可被翻的令牌 —— 早于它登记会留下一段「明明在跑、
/// 停止却说找不到」的窗口。
fn register_run(convo_id: Option<&str>, token: &StopToken) -> RunRegistration {
    let seq = RUN_SEQ.fetch_add(1, Ordering::Relaxed);
    let key = run_key(convo_id);
    if let Some(key) = key.as_ref() {
        if let Ok(mut table) = run_registry().lock() {
            table.insert(
                key.clone(),
                RunSlot {
                    token: token.clone(),
                    seq,
                },
            );
        }
    }
    RunRegistration { key, seq }
}

/// 取消请求的**诚实三态**。三者互不顶替，尤其**不许**把「已请求」说成「已停」。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopState {
    /// **没有可停的轮次**（已经结束、或还没开始、或压根没绑定会话）。
    NothingToStop { reason: String },
    /// 停止信号**已经递到**正在跑的那一轮（`token.cancel()` 返回了）。
    ///
    /// 这**不是**「那一轮一定停住了」：它可能在信号到达前就跑完了。
    /// 「真停住了」的判据是 [`StopState::Stopped`]，由 [`turn_stop_state`]
    /// 回**库**核对（看那一轮任务自己的终态，不看令牌）。
    Signalled { convo_id: String },
    /// 这一轮**真的**以 `TaskStatus::Cancelled` 落了库。
    ///
    /// `hop` 是「第几跳/共几跳」，从库里的 `error` 解析；**认不出就是
    /// `None`**，届时文案明说「第几跳没记下来」，绝不编一个数字。
    Stopped { hop: Option<(usize, usize)> },
}

impl StopState {
    /// 给用户看的那一句（IM 纯文本直发，**不放 markdown 记号** ——
    /// 与 `nt_channel_cmd::help_text` 的纯文本律同一条）。
    pub fn text(&self) -> String {
        match self {
            // 「没得停」只说没得停：一个「已停」类断言词都不许出现
            // （用户是扫一眼找结论的，那几个字一出现，他读到的就是「停了」）。
            StopState::NothingToStop { reason } => reason.clone(),
            StopState::Signalled { .. } => concat!(
                "已把停止信号递到正在跑的那一轮：它会在下一个取消检查点（跳边界 / 工具边界 / 睡之前）收手，\n",
                "但正在跑的那一次调用不会被掐断。\n",
                "这不等于它已经停下：停没停、停在哪一跳，都由那一轮自己的回执说。"
            )
            .to_owned(),
            StopState::Stopped { hop: Some((hop, steps)) } => format!(
                "已停：这一轮在第 {hop}/{steps} 跳被叫停（库里记为 cancelled，可以重跑再来一次）。"
            ),
            StopState::Stopped { hop: None } => concat!(
                "已停：这一轮被叫停了（库里记为 cancelled；第几跳没记下来，我不猜）。\n",
                "可以重跑再来一次。"
            )
            .to_owned(),
        }
    }
}

/// 翻掉某一轮正在跑的令牌，返回**这一趟实际发生的事**。
///
/// 「没有可停的轮次」是**正常答案的一种**，不是错误 —— 它被
/// [`StopState::NothingToStop`] 如实说出来，**绝不**退化成
/// `StopState::Signalled`（「假装递到了」比「停不了」坏得多）。
///
/// 今天在生产 IM 上它**恒**落在 `NothingToStop`：同步调度下 `/stop` 被读到
/// 时上一轮早已结束、登记已注销（见模块头「兑现仍缺并发」）。这不是本函数
/// 的缺陷，是调度形状的限制；要真兑现得先有第二个执行流。
pub fn signal_run_cancel(convo_id: Option<&str>) -> StopState {
    let Some(key) = run_key(convo_id) else {
        return StopState::NothingToStop {
            reason: "这个机器人还没绑定会话，无从知道该停哪一轮。".to_owned(),
        };
    };
    let Ok(mut table) = run_registry().lock() else {
        return StopState::NothingToStop {
            reason: "停止登记表锁坏了，这一轮停不了。".to_owned(),
        };
    };
    let Some(slot) = table.get_mut(&key) else {
        return StopState::NothingToStop {
            // 「刚查过」三个字是**要说的**：它告诉用户我们真的去查了，
            // 而不是「这个功能不支持」。没得停与不support 是两件事。
            reason: "刚查过：这个会话下没有正在跑的轮次，没有可停的轮次。".to_owned(),
        };
    };
    slot.token.cancel();
    StopState::Signalled { convo_id: key }
}

/// 本轮任务 id 认领用的窗口大小（与 `list_convo_tasks` 的 `created_at DESC`
/// 排序配套：新建的那条必然在最新一端）。
const TASK_WINDOW: i64 = 200;

/// 会话内最近若干条任务的 id 集（跑轮**之前**取一次当基线）。
fn convo_task_ids(store: &NeobotStore, convo_id: &str) -> HashSet<String> {
    store
        .list_convo_tasks(convo_id, TASK_WINDOW)
        .unwrap_or_default()
        .into_iter()
        .map(|task| task.id)
        .collect()
}

/// 认领**本轮自己的** task id（不是「库里最新的那个」）。
///
/// `run_local_turn_inner` 建的 id 只存在于函数内部、没有任何回调递得出来
/// （见「为什么按 convo_id 索引」），所以唯一能确证的口径是**集合差**：
/// 跑轮前后各读一次本会话的 id 集，本轮**新建**的那一条就是本轮的。
///
/// 收窄条件：**恰好一个**新 id。两个以上 = 有并发的别的跑轮，认不出 ⇒
/// 返回空串（下游据此只说较弱的话，不拿一个「大概是它」的 id 去宣称
/// 「这一轮已停」—— 那会把别人的停说成自己的）。
fn claim_turn_task_id(
    store: &NeobotStore,
    convo_id: &str,
    before: &HashSet<String>,
) -> String {
    let fresh: Vec<String> = store
        .list_convo_tasks(convo_id, TASK_WINDOW)
        .unwrap_or_default()
        .into_iter()
        .map(|task| task.id)
        .filter(|id| !before.contains(id))
        .collect();
    match fresh.as_slice() {
        [only] => (*only).clone(),
        _ => String::new(),
    }
}

/// `nt_agent` 落库时写的停止前缀（`STOPPED_PREFIX`，那边是私有常量）。
///
/// 这里是**字符串耦合**，故解析必须**fail-safe**：认不出前缀就当
/// 「第几跳没记下来」（`hop = None`），绝不猜一个数字。若哪天 `nt_agent`
/// 改了这条文案，本函数退化到 `None` 而不是报一个错的跳数。
const STOPPED_MARK: &str = "stopped by user";
/// `nt_agent` 的 hop 记法：`… at hop {hop}/{steps}`。
const HOP_MARK: &str = " at hop ";

/// 从任务的 `error` 里解析「第几跳/共几跳」（认不出回 `None`）。
fn stopped_hop(error: Option<&str>) -> Option<(usize, usize)> {
    let error = error?;
    if !error.contains(STOPPED_MARK) {
        return None;
    }
    let (_, tail) = error.split_once(HOP_MARK)?;
    let (hop, steps) = tail.trim().split_once('/')?;
    Some((hop.trim().parse().ok()?, steps.trim().parse().ok()?))
}

/// 本轮**真的**被叫停了吗 —— **回库核对**，不看令牌。
///
/// 理由与桌面侧同款：停止信号送达 ≠ 这一轮被停掉（它可能在信号到达前就跑完
/// 了）。判据必须落在**这一轮任务自己的终态**上（`TaskStatus::Cancelled`），
/// 而「哪一跳」从同一行的 `error` 里取。
///
/// `task_id` 为空（认不出本轮那条，见 `claim_turn_task_id`）⇒ 回 `None`：
/// 那时**连较弱的话都不说**，而不是拿「会话里最新那条」去猜。
pub fn turn_stop_state(store: &NeobotStore, task_id: &str) -> Option<StopState> {
    if task_id.trim().is_empty() {
        return None;
    }
    let task = store.get_task(task_id).ok().flatten()?;
    if task.status != TaskStatus::Cancelled {
        return None;
    }
    Some(StopState::Stopped {
        hop: stopped_hop(task.error.as_deref()),
    })
}

/// 跑一轮入站消息。
///
/// 需要 `engine` 来真的跑模型。
///
/// ## 为什么**没有** `stop_flag` 形参（2026-09-28 切片 C4 的明确决断）
///
/// 早先的最后一个形参是 `stop_flag: Option<&mut bool>`，而**所有调用方一律传
/// `None`**（`channel serve` 与桌面 `neobot_channel_poll_once` 都是）。它接不了
/// [`StopToken`]（`Arc<AtomicBool>`，克隆共享一面旗，`&mut bool` 与它不同型），
/// 于是它**在结构上就不可能**把停止信号递进跑轮 —— 只能置一个没人读的本地球。
/// 留着一个「永远传 `None`」的死形参，正是「测试充分但生产死代码」的翻版
/// （本仓库已经因此吃过亏：它还有一条测试在断言那个本地球被置起来，
/// 而生产路径上那个球根本不存在）。
///
/// **决断：删掉，不保留。** 替代物是本模块内部那张按 `convo_id` 索引的
/// 登记表（见上面的「停止信号：登记表 + 三态」）：`/stop` 去翻跑轮**正握着**
/// 的那一枚，跑轮把它当 `Option<&StopToken>` 递进 `run_local_turn_cancellable`。
/// 两个执行流之间的交接由表承担，不再由形参承担。
///
/// **这条改动波及 `on_inbound` 的每个调用点**（本切片之外的、需由其属主跟上的
/// 见交付报告；本文件内的调用点已全部跟上，测试的**断言内容一字未改**）。
pub fn on_inbound(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn crate::nt_engine::EngineAdapter,
    adapter: &dyn ChannelAdapter,
    bot: &crate::nt_store::BotRow,
    msg: &InboundMessage,
) -> Result<InboundOutcome, NtBotError> {
    // 1) 去重（长轮询重发必须幂等）。
    // 去重键必须带 chat：平台 message_id 按 chat 各自编号（见 `dedup_key`）。
    if !store.mark_seen(&dedup_key(
        bot.channel.as_str(),
        &msg.chat,
        &msg.message_id,
    ))? {
        return Ok(InboundOutcome::Duplicate);
    }
    // 2) 访问闸门。**在指令之前** —— 陌生人发 /new 不该有任何效果。
    let channel_row = store.get_channel(&bot.channel)?;
    let access = channel_row
        .as_ref()
        .map(|row| AccessMode::parse(&row.access_mode))
        .unwrap_or(AccessMode::Allow);
    let allow = parse_allow_list(&bot.allow_list);
    if !admits(access, &allow, msg.is_dm, &msg.sender) {
        return Ok(InboundOutcome::Rejected {
            reason: admit_reason(access, &allow, msg.is_dm, &msg.sender),
        });
    }
    store.touch_bot(&bot.channel, &bot.bot_id)?;

    // 3) 斜杠指令：不跑轮。
    if let Some(command) = nt_channel_cmd::parse(&msg.text) {
        let outcome = nt_channel_cmd::execute(store, &command, bot, adapter.display_name())?;
        // `/stop`：**去翻跑轮正握着的那一枚令牌**（不是置一个没人读的本地球）。
        // 键是机器人当前绑定的会话 —— 那正是上一轮跑在里面/该跑进去的会话。
        //
        // 回执是「本趟发生的事」+ core 那段共享文案（`nt_channel_cmd::local_reply`
        // 所有，**不在本切片所有权内，一个字都不改**）：本模块能保证的是前者
        // 如实（三态之一），core 那段在生产 IM 上仍字面为真（同步调度 ⇒
        // 恒落在「没有可停的轮次」）。若哪天调度真并发了，core 那段「停不了」
        // 就会变陈旧措辞 —— 那是 core 那段的属主要决断的事，不是这里。
        let reply = if outcome.stop_requested {
            let state = signal_run_cancel(bot.conversation_id.as_deref());
            format!("{}\n{}", state.text(), outcome.reply)
        } else {
            outcome.reply
        };
        // 回执发不出去也不该卡住整条链路：主回复已经写进会话与账本了。
        drop(adapter.send(&OutboundMessage {
            chat: msg.chat.clone(),
            text: reply.clone(),
            attachments: Vec::new(),
            edit_of: None,
        }));
        return Ok(InboundOutcome::Handled {
            command: command.as_str().to_owned(),
            reply,
        });
    }

    // 4) 落到绑定会话（没绑就现建一个 dm）。
    let convo_id = match bot.conversation_id.clone() {
        Some(id) => id,
        None => {
            let id = store.create_conversation(
                if msg.is_dm { "dm" } else { "group" },
                &default_convo_title(&msg.sender_name, &msg.sender),
                &[],
            )?;
            let mut updated = bot.clone();
            updated.conversation_id = Some(id.clone());
            store.upsert_bot(&updated)?;
            id
        }
    };

    // 4.5) 附件先落地（平台给的只是「凭据」，得取到本机才有意义）。
    // 顺序有讲究：先下载、后建会话、再登记 —— 登记要挂会话 id。
    let (saved, mut notes) = download_attachments(config, adapter, msg);

    // 会话落定后再登记附件。
    notes.extend(record_attachments(store, &convo_id, &saved));
    let user_text = if notes.is_empty() {
        msg.text.clone()
    } else if msg.text.trim().is_empty() {
        notes.join("\n")
    } else {
        format!("{}\n\n{}", msg.text, notes.join("\n"))
    };

    // 5) 跑轮（走与桌面**同一条**跑轮内层 `run_local_turn_cancellable`：
    //    同一网关、同一账本、同一组取消检查点 —— 差别只有 `stop` 那一参）。
    let actor = crate::nt_policy::Actor::Person;
    let actor_name = format!("{}:{}", bot.channel, msg.sender);
    let title = default_convo_title(&msg.sender_name, &msg.sender);
    // 本轮任务 id 的基线：**必须在跑轮之前**取（跑轮会新建本轮任务）。
    let tasks_before = convo_task_ids(store, &convo_id);
    let ctx = RunContext {
        store,
        config,
        engine,
        actor,
        actor_name: &actor_name,
        title: &title,
        user_text: &user_text,
        convo_id: Some(&convo_id),
    };
    // 停止信号接线：登记一枚本轮专属的令牌 → 把**同一枚**（克隆，共享一面旗）
    // 递进跑轮 → 轮次结束当场注销。此后 `/stop` 会如实报「没有可停的轮次」。
    //
    // 登记表在**生产 IM 上几乎总是空的**，因为同步调度下 `/stop` 永远排在
    // 这一轮**之后**才被读到（见模块头）。这不影响接线的正确性：链路是
    // 「命令入口 → 登记表 → 跑轮」三段都真，缺的是**第二个执行流**。
    let stop = StopToken::new();
    let registration = register_run(Some(&convo_id), &stop);
    let status = run_local_turn_cancellable(&ctx, None, None, Some(&stop))?;
    // 这一轮真的结束了才注销（含上面 `?` 早退：守卫随栈帧析构，没有漏注销的路径）。
    drop(registration);

    // 6) 结果回渠道。跑轮本身已经把答案写进 `steps`/会话，这里发的是
    //    「这一轮的状态 + 末次回复」。
    //
    //    「停没停」**回库核对**（`turn_stop_state`），不看令牌：信号送达与轮次
    //    结束是两件事，只有这一轮任务自己的终态能证明后者。
    let task_id = claim_turn_task_id(store, &convo_id, &tasks_before);
    let stopped = turn_stop_state(store, &task_id);
    let reply = match stopped {
        // 被叫停的这一轮：说实话「停在哪一跳」。早先这里发的是
        // `status_text(Waiting)` = 「已行动，等外部条件」—— 对一个被用户叫停
        // 的轮次来说那是**反的**（它啥也没「行动」到底就收手了）。
        Some(state) => state.text(),
        None => last_reply_of(store, &convo_id).unwrap_or_else(|| status_text(&status)),
    };
    // 同上：结果已落库，这一步只是把它递到渠道上。
    drop(adapter.send(&OutboundMessage {
        chat: msg.chat.clone(),
        text: reply,
        attachments: Vec::new(),
        edit_of: None,
    }));
    Ok(InboundOutcome::Turn {
        task_id,
        status: status.as_str().to_owned(),
    })
}

/// 把入站附件从平台取到 `<data_dir>/attachments`。
///
/// 返回 `(落地的路径, 给模型看的说明行)`：
/// 成功的是「已存到 …，读它用 read_file」，失败的是「<名字> 没取到：<原因>」。
/// 两者都**如实**说 —— 模型不知道有附件，就既不会去读、也不会告诉用户失败。
///
/// 只下载、**不登记**：登记要挂到会话上，而此刻会话可能还没建（首条消息）。
/// 登记由 [`record_attachments`] 在会话落定之后做。
pub fn download_attachments(
    config: &NeobotConfig,
    adapter: &dyn ChannelAdapter,
    msg: &InboundMessage,
) -> (Vec<SavedAttachment>, Vec<String>) {
    if msg.attachments.is_empty() {
        return (Vec::new(), Vec::new());
    }
    // 附件必须落在**工作区之内**，否则模型读不到它。
    //
    // 早先落 `<data_dir>/attachments`，而 jail 只放行 `workspace_dir`
    // （= `<data_dir>/workspace`）—— 两者是**兄弟目录**。于是给模型的那条
    // 「读它用 read_file，path=…」指向一个网关会当场拒掉的绝对路径
    // （`is_jailbreak_path` 拒 `/` 开头，`join_workspace` 再拒一次），
    // **附件等于收了但永远读不到**。
    //
    // 选「落进工作区」而不是「给 jail 加第二根白名单根」：后者扩大了牢笼，
    // 前者什么都不放宽，还顺手让附件能和其它文件一样被 `find` 搜到。
    let dir = config.workspace_dir.join("attachments");
    let mut saved = Vec::new();
    let mut notes = Vec::new();
    for att in &msg.attachments {
        match adapter.fetch_attachment(&att.id, &dir) {
            Ok(path) => {
                let path_text = path.to_string_lossy().into_owned();
                // 给模型的是**工作区相对路径**：网关的 `is_jailbreak_path`
                // 拒一切以 `/` 开头的绝对路径，所以绝对路径必然被拒。
                let rel = crate::nt_workspace::rel_of(&config.workspace_dir, &path);
                let size = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
                let name = att.name.clone();
                // 挑工具看**平台给的原名**，而不是落盘后的名字：后者可能已被
                // 适配器改过（`{id}.txt` 之类），据此判断会把图片错指给
                // `read_file`（读二进制必然失败）。原名没有扩展名时才回落看落盘名。
                //
                // 判据是「名字声称是图」，不验字节 —— `read_image` 自己会按魔数
                // 嗅探并诚实拒绝假图，所以这里指错的后果是它**说实话**，
                // 而不是把二进制当文本糊给模型。
                let looks_image = crate::nt_workspace::is_image_name(&att.name)
                    || crate::nt_workspace::is_image_name(&rel);
                let how = if looks_image { "read_image" } else { "read_file" };
                notes.push(format!(
                    "（附件 {name} 已存到工作区的 {rel}；读它用 {how}，path={rel}）"
                ));
                saved.push(SavedAttachment {
                    name,
                    kind: classify_name(&att.name),
                    path: path_text,
                    size: i64::try_from(size).unwrap_or(i64::MAX),
                });
            }
            Err(err) => notes.push(format!("（附件 {} 没取到：{err}）", att.name)),
        }
    }
    (saved, notes)
}

/// 附件落地后的登记信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedAttachment {
    pub name: String,
    /// `image` | `video` | `text` | `file`。
    pub kind: &'static str,
    pub path: String,
    pub size: i64,
}

/// 把已落地的附件登记到会话上（会话落定之后才做）。
///
/// 登记失败只如实回一行，不掀翻整条消息 —— 文件已经在磁盘上了。
pub fn record_attachments(
    store: &NeobotStore,
    convo_id: &str,
    saved: &[SavedAttachment],
) -> Vec<String> {
    let mut notes = Vec::new();
    for att in saved {
        match store.add_attachment(convo_id, att.kind, &att.name, &att.path, att.size) {
            Ok(_) => {}
            Err(err) => notes.push(format!("（附件 {} 登记失败：{err}）", att.name)),
        }
    }
    notes
}

/// 附件类型（按扩展名；与 `nt_store::classify_attachment` 同一口径的轻量版）。
fn classify_name(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    match lower.rsplit('.').next().unwrap_or("") {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "heic" => "image",
        "mp4" | "mov" | "webm" | "mkv" | "m4v" => "video",
        "txt" | "md" | "json" | "csv" | "log" | "rs" | "ts" | "js" | "py" => "text",
        _ => "file",
    }
}

/// 新会话的缺省标题。
pub fn default_convo_title(sender_name: &str, sender: &str) -> String {
    let who = if sender_name.trim().is_empty() {
        sender.trim()
    } else {
        sender_name.trim()
    };
    let title = if who.is_empty() { "IM 会话" } else { who };
    title.chars().take(40).collect()
}

/// 该会话最后一轮的助手回复（从 `steps` 的 `reply` 行取）。
pub fn last_reply_of(store: &NeobotStore, convo_id: &str) -> Option<String> {
    let tasks = store.list_convo_tasks(convo_id, 1).ok()?;
    let task = tasks.first()?;
    store
        .last_step_output(&task.id, "reply")
        .ok()
        .flatten()
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn status_text(status: &crate::nt_types::TurnStatus) -> String {
    match status {
        crate::nt_types::TurnStatus::Done => "办完了（没有可贴的正文）。".to_owned(),
        crate::nt_types::TurnStatus::NeedsClarification => "需要你补一句才能继续。".to_owned(),
        crate::nt_types::TurnStatus::Blocked => "这一步被策略网关挡住了（fail-closed）。".to_owned(),
        crate::nt_types::TurnStatus::Waiting => "已行动，等外部条件。".to_owned(),
        crate::nt_types::TurnStatus::Continue => "还没收尾。".to_owned(),
    }
}

/// 从 outbox 的 payload 里取本地附件路径（**缺字段当空**）。
///
/// 只收字符串、只收非空串；别的形状（对象 / 数字 / 空串）**丢掉**而不是报错 ——
/// 一个坏附件名不该把整条消息（含正文）一起害掉。丢掉之后并不会变成
/// 「假装发成了」：渠道侧压根没收到附件就报失败，消息退回 outbox 重试。
fn payload_attachments(parsed: &serde_json::Value) -> Vec<String> {
    parsed
        .get("attachments")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// 从 outbox 的 payload 里取要编辑的**平台消息 id**（**缺字段当没有**）。
///
/// 键名 `edit_of`，与 [`OutboundMessage::edit_of`] 同名同口径。
///
/// 与 [`payload_attachments`] 同一套「坏值丢掉而不是报错」的口径，
/// **理由比附件那条更硬**：这个键是**可选**的，早于它的 payload 一条都没有。
/// 要是把「没这个键」当成错误，那些 payload 会被整条判失败、全部退避重试
/// —— 用户看到的是队列雪崩，而触发它的只是我们加了个可选键。
/// 所以：缺字段 / `null` / 非字符串 / 空串 **一律等价于 `None`**
/// （发新消息，也就是接入前的行为）。
fn payload_edit_of(parsed: &serde_json::Value) -> Option<String> {
    parsed
        .get("edit_of")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
}

/// 出站：把 outbox 里排队的消息按渠道发出去。
///
/// 返回 `(成功条数, 失败条数)`。失败的消息**留在 outbox**（走退避），
/// 不丢 —— 丢了等于用户白等了一轮模型。
pub fn drain_outbox_once(
    store: &NeobotStore,
    adapters: &mut crate::nt_channel::ChannelRegistry,
) -> Result<(usize, usize), NtBotError> {
    let now = Utc::now().to_rfc3339();
    let rows = store.drain_outbox(20, &now)?;
    let (mut sent, mut failed) = (0usize, 0usize);
    for (id, topic, payload) in rows {
        // 只有 CH_CHANNEL_SEND 有发送方（本函数）。别的 topic（历史遗留的
        // CH_MESSAGE_NEW 通知行等）本 drainer 永远发不出去；payload 不可变，
        // 故永远不会变可发。退回重试只会让它永久占位 → 删 + 留痕。
        if topic != "CH_CHANNEL_SEND" {
            eprintln!("neobot: outbox 丢弃无发送方的行 id={id} topic={topic}");
            drop(store.drop_poison_outbox_row(&id));
            failed += 1;
            continue;
        }
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&payload) else {
            // 解析失败的行永远解析不了（payload 不可变）。
            // 原先退避重试，但重试不是恢复 —— 它无限占用 drain 预算。
            // 删 + 留痕（证据在 stderr，不静默吞）。
            eprintln!("neobot: outbox 丢弃坏 payload 行 id={id}");
            drop(store.drop_poison_outbox_row(&id));
            failed += 1;
            continue;
        };
        let Some(channel) = parsed.get("channel").and_then(|v| v.as_str()) else {
            // 同上：缺 channel 的行永远不可路由（payload 不可变）。
            // 注意与「渠道未注册」区分 —— 后者是暂时的（用户可能稍后启用渠道），
            // 仍退避重试（见 outbox_drain_does_not_drop_unknown_channel）。
            eprintln!("neobot: outbox 丢弃缺 channel 行 id={id}");
            drop(store.drop_poison_outbox_row(&id));
            failed += 1;
            continue;
        };
        let chat = parsed
            .get("chat")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned();
        let text = parsed
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned();
        let attachments = payload_attachments(&parsed);
        let edit_of = payload_edit_of(&parsed);
        if text.trim().is_empty() && attachments.is_empty() {
            // 没正文也没附件 = 没东西可发。**只有附件不行**：一张图零解释是
            // 完全合法的出站，早先那个「空正文就丢」的判据会把它吃掉。
            drop(store.fail_outbox(&id, &retry_at(&now)));
            failed += 1;
            continue;
        }
        let Some(adapter) = adapters.get_mut(channel) else {
            // 渠道没注册：退回重试（用户可能只是还没启那个渠道）。
            drop(store.fail_outbox(&id, &retry_at(&now)));
            failed += 1;
            continue;
        };
        // 渠道自称不能发附件时，**不**替它把附件扔掉只发正文 —— 那就是静默降级。
        // 退回重试：等渠道支持了（或 payload 改了）自然就通。
        if !attachments.is_empty() && !adapter.supports_attachments() {
            drop(store.fail_outbox(&id, &retry_at(&now)));
            failed += 1;
            continue;
        }
        match adapter.send(&OutboundMessage {
            chat,
            text,
            attachments,
            edit_of,
        }) {
            Ok(_) => {
                drop(store.fail_outbox(&id, &retry_at(&now)));
                sent += 1;
            }
            Err(_) => {
                drop(store.fail_outbox(&id, &retry_at(&now)));
                failed += 1;
            }
        }
    }
    Ok((sent, failed))
}

/// 退避到 `now + 60s`（与 outbox 的 `available_at` 语义一致）。
fn retry_at(now: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(now) {
        Ok(parsed) => (parsed + chrono::Duration::seconds(60))
            .to_rfc3339(),
        Err(_) => now.to_owned(),
    }
}

/// 排一条渠道出站消息进 outbox（供上层用）。
pub fn enqueue_outbound(
    store: &NeobotStore,
    channel: &str,
    chat: &str,
    text: &str,
) -> Result<String, NtBotError> {
    enqueue_outbound_with_attachments(store, channel, chat, text, &[])
}

/// 同上，但带**本地附件路径**（模型 → 用户方向）。
///
/// 单列一个函数而不是给 [`enqueue_outbound`] 加参数：既有调用方一个都不用改，
/// 也不会出现「传了空 vec 还得多占一个参数位」。
///
/// 附件路径会原样进 payload 并被 [`drain_outbox_once`] 送到渠道 —— 这一段以前
/// 是断的（payload 里的附件被静默丢弃），附件律要求它端到端是通的。
pub fn enqueue_outbound_with_attachments(
    store: &NeobotStore,
    channel: &str,
    chat: &str,
    text: &str,
    attachments: &[String],
) -> Result<String, NtBotError> {
    enqueue_outbound_full(store, channel, chat, text, attachments, None)
}

/// 排一条**要编辑原消息**的出站（payload 里带 `edit_of`）。
///
/// `edit_of` = 平台那条要改的消息 id（Telegram 是 `message_id`）。
/// 渠道侧会先试编辑，编不成就退回发新消息并记账（见
/// `nt_channel_telegram::SendOutcome`）—— 这里只管把意图排进队列。
///
/// 排出来的 `edit_of` 是**谁写谁负责**的事：给一个不是本 bot 发的
/// message id，平台会拒（`message to edit not found`），降级成发新消息。
/// 想避免那次多余请求，就该传本 bot 自己发过的那条 id。
pub fn enqueue_outbound_editing(
    store: &NeobotStore,
    channel: &str,
    chat: &str,
    text: &str,
    edit_of: &str,
) -> Result<String, NtBotError> {
    enqueue_outbound_full(store, channel, chat, text, &[], Some(edit_of))
}

/// 排队的共同实现：**没有编辑目标时 payload 一个字节都不多写**。
///
/// `edit_of` 用「有值才插键」而不是恒写 `null`：老 payload 与新 payload
/// 长得一模一样，靠 [`payload_edit_of`] 的缺省语义吃下全部历史队列。
fn enqueue_outbound_full(
    store: &NeobotStore,
    channel: &str,
    chat: &str,
    text: &str,
    attachments: &[String],
    edit_of: Option<&str>,
) -> Result<String, NtBotError> {
    let id = uuid::Uuid::new_v4().to_string();
    let mut payload = serde_json::json!({
        "channel": channel,
        "chat": chat,
        "text": text,
        "attachments": attachments,
    });
    // 用 `as_object_mut` 而不是 `payload["edit_of"] = …`：本 crate 开着
    // `-W clippy::indexing_slicing`，而 `Value` 的下标插入在 key 类型不对时
    // 会**panic**（`&str` 键插到非对象上就是这个下场）。这里显式只认对象。
    if let Some(target) = edit_of.map(str::trim).filter(|id| !id.is_empty()) {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(
                "edit_of".to_owned(),
                serde_json::Value::String(target.to_owned()),
            );
        }
    }
    store.enqueue_outbox(&id, "CH_CHANNEL_SEND", &payload.to_string())?;
    Ok(id)
}

/// 补发一轮：把某任务的结果补回它来的聊天。
///
/// `text` 为空 = 那个任务没有可补的正文（如被网关挡住），此时**不补发**——
/// 补一句「失败了」比让用户对着空白等待要好，但也不能把内部细节抖出去。
/// 返回是否真的补发了。
pub fn deliver_result(
    store: &NeobotStore,
    adapter: &dyn ChannelAdapter,
    channel: &str,
    bot_id: &str,
    chat: &str,
    origin_message: &str,
    task_id: &str,
    text: &str,
) -> Result<bool, NtBotError> {
    if text.trim().is_empty() {
        return Ok(false);
    }
    match adapter.send(&OutboundMessage {
        chat: chat.to_owned(),
        text: text.to_owned(),
        attachments: Vec::new(),
        edit_of: Some(origin_message.to_owned()),
    }) {
        Ok(_) => {
            store.touch_bot(channel, bot_id)?;
            Ok(true)
        }
        Err(err) => {
            // 发不出去就记一条待补发（而不是丢）。
            //
            // ⛔ id 是效果键（任务 + 内容），不是 uuid：同一任务同一文本
            // 重试只留一行 —— 否则扫补发时会把同一条结果发两遍（用户收到
            // 两条 identical 回复，而那是重试抖动，不是两个结果）。
            // 已在队里（重试撞上上次）不重复记，但仍返回 Err ——
            // 这次确实没发出去，调用方有权知道。
            let key = crate::nt_effect_key::effect_key(
                &format!("task/{task_id}"),
                "deliver_result",
                &serde_json::json!({
                    "channel": channel,
                    "chat": chat,
                    "origin_message": origin_message,
                    "text": text,
                }),
            )
            .map_err(crate::nt_error::NtBotError::Invalid)?;
            if !store.has_pending_delivery(&key)? {
                store.enqueue_delivery(&crate::nt_store::PendingDelivery {
                    id: key,
                    channel: channel.to_owned(),
                    bot_id: bot_id.to_owned(),
                    chat: chat.to_owned(),
                    origin_message: origin_message.to_owned(),
                    text: text.to_owned(),
                    task_id: task_id.to_owned(),
                    attempts: 0,
                    created_at: Utc::now().to_rfc3339(),
                })?;
            }
            Err(err)
        }
    }
}

/// 扫一遍待补发队列。返回 `(补成了几条, 放弃了几条)`。
///
/// 到顶的自动放弃（`due_deliveries` 已过滤）—— 平台返回不确定时无限重发
/// 会把同一条结果发好几遍，那比不发更糟。
pub fn sweep_pending(
    store: &NeobotStore,
    adapters: &mut crate::nt_channel::ChannelRegistry,
) -> Result<(usize, usize), NtBotError> {
    let (mut done, mut gave_up) = (0usize, 0usize);
    for item in store.due_deliveries(20)? {
        let Some(adapter) = adapters.get(&item.channel) else {
            // 渠道还没注册：记一次尝试，退避，别放弃（用户可能待会儿就启了）。
            store.fail_delivery(&item.id)?;
            continue;
        };
        match adapter.send(&OutboundMessage {
            chat: item.chat.clone(),
            text: item.text.clone(),
            attachments: Vec::new(),
            edit_of: Some(item.origin_message.clone()),
        }) {
            Ok(_) => {
                store.complete_delivery(&item.id)?;
                drop(store.touch_bot(&item.channel, &item.bot_id));
                done += 1;
            }
            Err(_) => {
                let attempts = store.fail_delivery(&item.id)?;
                if attempts >= crate::nt_store::MAX_SEND_ATTEMPTS {
                    gave_up += 1;
                }
            }
        }
    }
    Ok((done, gave_up))
}

/// 渠道维护（去重表 / 待补发表按留存期切掉）。
pub fn upkeep_best_effort(store: &NeobotStore) {
    let seen_cutoff = (Utc::now() - chrono::Duration::days(
        crate::nt_store::SEEN_RETENTION_DAYS,
    ))
    .to_rfc3339();
    drop(store.prune_seen(&seen_cutoff));
    let pending_cutoff = (Utc::now() - chrono::Duration::days(
        crate::nt_store::PENDING_RETENTION_DAYS,
    ))
    .to_rfc3339();
    drop(store.prune_deliveries(&pending_cutoff));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nt_store::BotRow;

    struct FakeChannel {
        sent: std::cell::RefCell<Vec<OutboundMessage>>,
        fail: bool,
    }

    impl ChannelAdapter for FakeChannel {
        fn channel_id(&self) -> &str {
            "fake"
        }
        fn display_name(&self) -> &str {
            "假渠道"
        }
        fn token_env(&self) -> &str {
            "NEOBOT_FAKE_TOKEN"
        }
        fn probe(&self) -> crate::nt_channel::ChannelHealth {
            crate::nt_channel::ChannelHealth { ok: true, detail: String::new(), info: String::new() }
        }
        fn poll(&mut self) -> Result<Vec<InboundMessage>, NtBotError> {
            Ok(Vec::new())
        }
        fn send(&self, out: &OutboundMessage) -> Result<String, NtBotError> {
            if self.fail {
                return Err(NtBotError::Io("boom".to_owned()));
            }
            self.sent.borrow_mut().push(out.clone());
            Ok("m1".to_owned())
        }
    }

    fn store(case: &str) -> NeobotStore {
        NeobotStore::open(":memory:").unwrap_or_else(|err| panic!("{case}: {err}"))
    }

    fn bot() -> BotRow {
        BotRow {
            channel: "fake".to_owned(),
            bot_id: "b1".to_owned(),
            alias: String::new(),
            token_env: "NEOBOT_FAKE_TOKEN".to_owned(),
            conversation_id: None,
            model: String::new(),
            allow_list: String::new(),
            created_at: String::new(),
            last_seen: None,
        }
    }

    fn msg(id: &str, text: &str, is_dm: bool) -> InboundMessage {
        InboundMessage {
            chat: "42".to_owned(),
            is_dm,
            sender: "alice".to_owned(),
            sender_name: "Alice".to_owned(),
            text: text.to_owned(),
            message_id: id.to_owned(),
            attachments: Vec::new(),
            reply_to: None,
        }
    }

    fn config(case: &str) -> NeobotConfig {
        let dir = crate::nt_testutil::temp_dir(&format!("neobot-dispatch-test-{}", case));
        drop(std::fs::remove_dir_all(&dir));
        NeobotConfig {
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
        }
    }

    fn open_channel(store: &NeobotStore, mode: &str) {
        store.upsert_channel("fake", "假渠道", mode, 5).expect("channel");
    }

    #[test]
    fn duplicate_message_is_dropped() {
        let st = store("dup");
        let cfg = config("dup");
        open_channel(&st, "open");
        let mut b = bot();
        b.conversation_id = Some(st.create_conversation("dm", "A", &[]).expect("c"));
        st.upsert_bot(&b).expect("bot");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        let first = on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "你好", true))
            .expect("first");
        assert!(matches!(first, InboundOutcome::Turn { .. }));
        let second = on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "你好", true))
            .expect("second");
        assert_eq!(second, InboundOutcome::Duplicate, "同一条只能处理一次");
        assert_eq!(ch.sent.borrow().len(), 1, "重复不该再发一次回复");
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    #[test]
    fn access_gate_blocks_before_commands() {
        let st = store("gate");
        let cfg = config("gate");
        open_channel(&st, "allow"); // 空名单 = 全拒
        let b = bot();
        st.upsert_bot(&b).expect("bot");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        // 未放行来源发 /new：必须**连指令都不执行**（不重置别人的会话）。
        let got = on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "/new", true))
            .expect("run");
        assert!(matches!(got, InboundOutcome::Rejected { .. }));
        assert!(ch.sent.borrow().is_empty(), "被拒的消息不该有回执");
        assert!(st.get_bot("fake", "b1").expect("get").expect("some").conversation_id.is_none());
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    #[test]
    fn allow_list_admits_listed_sender() {
        let st = store("allow");
        let cfg = config("allow");
        open_channel(&st, "allow");
        let mut b = bot();
        b.allow_list = "alice".to_owned();
        b.conversation_id = Some(st.create_conversation("dm", "A", &[]).expect("c"));
        st.upsert_bot(&b).expect("bot");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        let got = on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "在吗", true))
            .expect("run");
        assert!(matches!(got, InboundOutcome::Turn { .. }), "{got:?}");
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    #[test]
    fn dm_only_ignores_group_messages() {
        let st = store("dm");
        let cfg = config("dm");
        open_channel(&st, "dm_only");
        let b = bot();
        st.upsert_bot(&b).expect("bot");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        let got = on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "在吗", false))
            .expect("run");
        assert!(matches!(got, InboundOutcome::Rejected { .. }), "{got:?}");
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    #[test]
    fn slash_command_does_not_run_a_turn() {
        let st = store("cmd");
        let cfg = config("cmd");
        open_channel(&st, "open");
        let b = bot();
        st.upsert_bot(&b).expect("bot");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        let before = st.list_tasks(50).expect("tasks").len();
        let got = on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "/new 新的", true))
            .expect("run");
        assert_eq!(
            got,
            InboundOutcome::Handled { command: "new".to_owned(), reply: got.reply() },
            "{got:?}"
        );
        // 指令不建任务。
        assert_eq!(st.list_tasks(50).expect("tasks").len(), before, "指令不该跑轮");
        // 但确实建了新会话并改了绑定。
        let after = st.get_bot("fake", "b1").expect("get").expect("some");
        assert!(after.conversation_id.is_some());
        // 回执发回渠道。
        assert_eq!(ch.sent.borrow().len(), 1);
        assert!(ch.sent.borrow()[0].text.contains("新的"));
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    /// `/stop` 把**跑轮正握着的那一枚**令牌翻掉。
    ///
    /// 本切片之前，这条测试断言的是一个**没人读的本地球**（`stop_flag` 形参），
    /// 生产路径上那个球根本不存在（所有调用方都传 `None`）。现在断言的是
    /// 登记表里那枚**跑轮真的握着**的 `StopToken` —— **断言文案逐字未改**，
    /// 被换掉的只是「外部停止信号」的可观测量。
    #[test]
    fn stop_command_sets_the_stop_flag() {
        let st = store("stop");
        let cfg = config("stop");
        open_channel(&st, "open");
        let mut b = bot();
        let convo = st.create_conversation("dm", "A", &[]).expect("convo");
        b.conversation_id = Some(convo.clone());
        st.upsert_bot(&b).expect("bot");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        // 代「正在跑的那一轮」：它已把这枚令牌登记在**自己那个会话**名下。
        let live = StopToken::new();
        let _registration = register_run(Some(&convo), &live);
        let got = on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "/stop", true)).expect("run");
        assert!(matches!(got, InboundOutcome::Handled { .. }));
        assert!(live.is_cancelled(), "/stop 必须把外部停止信号置起来");
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    #[test]
    fn first_message_creates_and_binds_a_conversation() {
        let st = store("bind");
        let cfg = config("bind");
        open_channel(&st, "open");
        let b = bot();
        st.upsert_bot(&b).expect("bot");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "第一条", true))
            .expect("run");
        let after = st.get_bot("fake", "b1").expect("get").expect("some");
        let convo = after.conversation_id.expect("should be bound");
        let row = st.get_conversation(&convo).expect("get").expect("some");
        assert_eq!(row.kind, "dm");
        assert_eq!(row.title, "Alice");
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    #[test]
    fn group_message_creates_a_group_conversation() {
        let st = store("grp");
        let cfg = config("grp");
        open_channel(&st, "open");
        let b = bot();
        st.upsert_bot(&b).expect("bot");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "群里说的", false))
            .expect("run");
        let after = st.get_bot("fake", "b1").expect("get").expect("some");
        let convo = after.conversation_id.expect("bound");
        assert_eq!(st.get_conversation(&convo).expect("get").expect("some").kind, "group");
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    // ---- 出站 ----

    #[test]
    fn outbox_drain_sends_and_clears() {
        let st = store("drain");
        let mut reg = crate::nt_channel::ChannelRegistry::new();
        reg.register(Box::new(FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false }))
            .expect("register");
        enqueue_outbound(&st, "fake", "42", "发我出去").expect("enqueue");
        let (sent, failed) = drain_outbox_once(&st, &mut reg).expect("drain");
        assert_eq!((sent, failed), (1, 0));
    }

    #[test]
    fn outbox_drain_keeps_failed_messages_with_backoff() {
        let st = store("faildrain");
        let mut reg = crate::nt_channel::ChannelRegistry::new();
        reg.register(Box::new(FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: true }))
            .expect("register");
        enqueue_outbound(&st, "fake", "42", "会失败的").expect("enqueue");
        let (sent, failed) = drain_outbox_once(&st, &mut reg).expect("drain");
        assert_eq!(sent, 0);
        assert_eq!(failed, 1);
        // 关键：消息**没被丢掉**（还在队列里，带退避）。
        let pending = st.drain_outbox(10, &chrono::Utc::now().to_rfc3339()).expect("drain");
        assert!(pending.is_empty(), "退避期内不该再可取（说明 available_at 已推后）");
        // 换个将来的时间点就能取回 → 证明它还在。
        let later = st
            .drain_outbox(10, &(chrono::Utc::now() + chrono::Duration::seconds(120)).to_rfc3339())
            .expect("drain later");
        assert_eq!(later.len(), 1, "失败的消息必须还在队列里等重试");
    }

    #[test]
    fn outbox_drain_does_not_drop_unknown_channel() {
        let st = store("unknown");
        let mut reg = crate::nt_channel::ChannelRegistry::new();
        enqueue_outbound(&st, "nope", "42", "无家可归").expect("enqueue");
        let (sent, failed) = drain_outbox_once(&st, &mut reg).expect("drain");
        assert_eq!((sent, failed), (0, 1));
        let later = st
            .drain_outbox(10, &(chrono::Utc::now() + chrono::Duration::seconds(120)).to_rfc3339())
            .expect("drain later");
        assert_eq!(later.len(), 1, "渠道没注册不该丢消息");
    }

    #[test]
    fn outbox_drain_drops_row_without_channel() {
        // 缺 channel 的行永远不可路由（payload 不可变），重试不是恢复。
        // 2026-09-30 前它是无限退避占位（nt_agent 的 CH_MESSAGE_NEW 毒行即此类）。
        let st = store("nochannel");
        let mut reg = crate::nt_channel::ChannelRegistry::new();
        st.enqueue_outbox("nc1", "CH_CHANNEL_SEND", r#"{"text":"没渠道"}"#)
            .expect("enqueue");
        let (sent, failed) = drain_outbox_once(&st, &mut reg).expect("drain");
        assert_eq!((sent, failed), (0, 1));
        let later = st
            .drain_outbox(10, &(chrono::Utc::now() + chrono::Duration::seconds(120)).to_rfc3339())
            .expect("drain later");
        assert!(later.is_empty(), "缺 channel 的行必须被删，不能无限退避占位");
    }

    #[test]
    fn outbox_drain_drops_unparseable_payload() {
        // 解析失败的行永远解析不了。删 + 留痕，而不是无限退避。
        let st = store("badpayload");
        let mut reg = crate::nt_channel::ChannelRegistry::new();
        st.enqueue_outbox("bp1", "CH_CHANNEL_SEND", "这不是 json{")
            .expect("enqueue");
        let (sent, failed) = drain_outbox_once(&st, &mut reg).expect("drain");
        assert_eq!((sent, failed), (0, 1));
        let later = st
            .drain_outbox(10, &(chrono::Utc::now() + chrono::Duration::seconds(120)).to_rfc3339())
            .expect("drain later");
        assert!(later.is_empty(), "坏 payload 行必须被删，不能无限退避占位");
    }

    #[test]
    fn outbox_drain_drops_row_with_unknown_topic() {
        // 非 CH_CHANNEL_SEND 的 topic 没有发送方（drainer 只认发送 topic）。
        // 留着既发不出去，prune 又只删 claimed=1 —— 不删就是永久堆积。
        let st = store("badtopic");
        let mut reg = crate::nt_channel::ChannelRegistry::new();
        st.enqueue_outbox("bt1", "CH_MESSAGE_NEW", r#"{"task_id":"t1","status":"done"}"#)
            .expect("enqueue");
        let (sent, failed) = drain_outbox_once(&st, &mut reg).expect("drain");
        assert_eq!((sent, failed), (0, 1));
        let later = st
            .drain_outbox(10, &(chrono::Utc::now() + chrono::Duration::seconds(120)).to_rfc3339())
            .expect("drain later");
        assert!(later.is_empty(), "无发送方的 topic 行必须被删");
    }

    // ---- 出站附件（model → user）----

    /// 会记下收到的 `OutboundMessage`、并如实自报支不支持附件的渠道。
    ///
    /// 发送记录同时写进 `self.sent` 和表外的 [`SENT_LOG`]：前者让
    /// `&SendChannel` 的测试能查；后者是必需的 —— `ChannelRegistry` 只暴露
    /// `&dyn ChannelAdapter`，断言 `drain_outbox_once` **真的把附件传下去了**
    /// 就只能从注册表外面看。
    struct SendChannel {
        supports: bool,
    }

    impl ChannelAdapter for SendChannel {
        fn channel_id(&self) -> &str { "fake" }
        fn display_name(&self) -> &str { "假渠道" }
        fn token_env(&self) -> &str { "NEOBOT_FAKE_TOKEN" }
        fn probe(&self) -> crate::nt_channel::ChannelHealth {
            crate::nt_channel::ChannelHealth { ok: true, detail: String::new(), info: String::new() }
        }
        fn poll(&mut self) -> Result<Vec<InboundMessage>, NtBotError> { Ok(Vec::new()) }
        fn supports_attachments(&self) -> bool { self.supports }
        fn send(&self, out: &OutboundMessage) -> Result<String, NtBotError> {
            // 与 Telegram 侧同一个不变式：不支持就别假装收下了。
            if !out.attachments.is_empty() && !self.supports {
                return Err(NtBotError::Invalid("this channel cannot send files".to_owned()));
            }
            SENT_LOG.with(|log| log.borrow_mut().push(out.clone()));
            Ok("m1".to_owned())
        }
    }

    thread_local! {
        static SENT_LOG: std::cell::RefCell<Vec<OutboundMessage>> =
            const { std::cell::RefCell::new(Vec::new()) };
    }

    fn send_chan(supports: bool) -> SendChannel {
        SendChannel { supports }
    }

    fn drain_one(store: &NeobotStore) -> Result<(usize, usize), NtBotError> {
        let mut reg = crate::nt_channel::ChannelRegistry::new();
        reg.register(Box::new(send_chan(true))).expect("register");
        drain_outbox_once(store, &mut reg)
    }

    #[test]
    fn outbox_drain_passes_attachment_paths_through_to_the_channel() {
        // 端到端的关键一环：payload 里的 attachments 以前被 `Vec::new()` 顶掉了，
        // 于是模型产出的文件路径**静默消失** —— 用户收到文字，以为文件也到了。
        let st = store("attdrain");
        enqueue_outbound_with_attachments(
            &st,
            "fake",
            "42",
            "两个文件",
            &["/tmp/a.txt".to_owned(), "/tmp/b.png".to_owned()],
        )
        .expect("enqueue");
        SENT_LOG.with(|log| log.borrow_mut().clear());
        let (sent, failed) = drain_one(&st).expect("drain");
        assert_eq!((sent, failed), (1, 0));
        let recorded = SENT_LOG.with(|log| log.borrow().last().cloned());
        let Some(recorded) = recorded else {
            panic!("渠道一次都没被 send 过");
        };
        assert_eq!(
            recorded.attachments,
            vec!["/tmp/a.txt".to_owned(), "/tmp/b.png".to_owned()],
            "附件路径必须原样送到渠道（不能被清空）"
        );
        assert_eq!(recorded.text, "两个文件");
        assert_eq!(recorded.chat, "42");
    }

    #[test]
    fn outbox_drain_leaves_text_only_payloads_untouched() {
        // 老 payload 没有 attachments 键（`enqueue_outbound` 就还是那样排的），
        // 必须照发不误。
        let st = store("noattkey");
        enqueue_outbound(&st, "fake", "42", "纯文字").expect("enqueue");
        SENT_LOG.with(|log| log.borrow_mut().clear());
        let (sent, failed) = drain_one(&st).expect("drain");
        assert_eq!((sent, failed), (1, 0));
        let recorded = SENT_LOG.with(|log| log.borrow().last().cloned());
        assert_eq!(
            recorded.map(|m| m.attachments),
            Some(Vec::new()),
            "老 payload（没有 attachments 键）不该被当成有附件"
        );
    }

    #[test]
    fn outbox_drain_sends_an_attachment_only_message_with_empty_text() {
        // 一张图零解释是完全合法的出站。早先「空正文就丢」的判据会把它吃掉。
        let st = store("attonly");
        enqueue_outbound_with_attachments(&st, "fake", "42", "   ", &["/tmp/only.png".to_owned()])
            .expect("enqueue");
        let (sent, failed) = drain_one(&st).expect("drain");
        assert_eq!((sent, failed), (1, 0), "只有附件不该被判成空消息");
    }

    #[test]
    fn outbox_drain_refuses_to_drop_files_for_a_channel_without_support() {
        // 降级律的落点：渠道说不能发，就**不**替它只发正文。
        // 那样用户会以为文件也到了 —— 比报错坏得多。
        let st = store("nosupport-out");
        let mut reg = crate::nt_channel::ChannelRegistry::new();
        reg.register(Box::new(send_chan(false))).expect("register");
        enqueue_outbound_with_attachments(&st, "fake", "42", "看这个", &["/tmp/a.txt".to_owned()])
            .expect("enqueue");
        SENT_LOG.with(|log| log.borrow_mut().clear());
        let (sent, failed) = drain_outbox_once(&st, &mut reg).expect("drain");
        assert_eq!((sent, failed), (0, 1), "渠道不支持附件不该算成功");
        assert!(
            SENT_LOG.with(|log| log.borrow().is_empty()),
            "一个字都不该发出去（不能只发正文）"
        );
        // 关键：消息还在队列里等重试（而不是被当成已发送丢掉）。
        let later = st
            .drain_outbox(10, &(chrono::Utc::now() + chrono::Duration::seconds(120)).to_rfc3339())
            .expect("drain later");
        assert_eq!(later.len(), 1, "退避后仍应能取回");
    }

    #[test]
    fn payload_attachments_only_accepts_non_empty_strings() {
        let parsed: serde_json::Value = serde_json::from_str(
            r#"{"attachments":["/tmp/a.txt","  ","",42,null,{"p":1},["/tmp/b.txt"]]}"#,
        )
        .expect("json");
        // 只收字符串与路径；数组/对象/数字/空串一律丢掉（一个坏附件名
        // 不该把整条消息连正文一起害掉）。
        assert_eq!(payload_attachments(&parsed), vec!["/tmp/a.txt".to_owned()]);
        // 缺字段 / 不是数组 = 没有附件（老 payload 照发）。
        let none: serde_json::Value = serde_json::from_str(r#"{"text":"hi"}"#).expect("json");
        assert!(payload_attachments(&none).is_empty());
        let wrong: serde_json::Value =
            serde_json::from_str(r#"{"attachments":"/tmp/a.txt"}"#).expect("json");
        assert!(payload_attachments(&wrong).is_empty());
    }

    // ---- 编辑原消息（`edit_of` 走队列）----

    #[test]
    fn payload_edit_of_treats_a_missing_key_as_no_edit() {
        // **防队列雪崩的那条**：早于 `edit_of` 的 payload 一条都没有这个键。
        // 缺字段必须等价于 `None`，否则历史 payload 会被整片判失败。
        let legacy: serde_json::Value =
            serde_json::from_str(r#"{"channel":"telegram","chat":"42","text":"hi","attachments":[]}"#)
                .expect("json");
        assert_eq!(payload_edit_of(&legacy), None, "老 payload 必须照发");
        // 同一套「坏值丢掉而不是报错」的口径（与 attachments 一致）。
        for bad in [
            r#"{"edit_of":null}"#,
            r#"{"edit_of":""}"#,
            r#"{"edit_of":"   "}"#,
            r#"{"edit_of":555}"#,
            r#"{"edit_of":{"id":1}}"#,
            r#"{"edit_of":["555"]}"#,
            r#"{"edit_of":true}"#,
        ] {
            let parsed: serde_json::Value = serde_json::from_str(bad).expect("json");
            assert_eq!(payload_edit_of(&parsed), None, "{bad} 不该当编辑目标");
        }
        // 正常值（两侧空格忽略）。
        let good: serde_json::Value = serde_json::from_str(r#"{"edit_of":" 555 "}"#).expect("json");
        assert_eq!(payload_edit_of(&good), Some("555".to_owned()));
    }

    #[test]
    fn outbox_drain_passes_edit_of_through_to_the_channel() {
        // 队列这一段以前是断的：payload 里的编辑意图压根到不了适配器。
        let st = store("editdrain");
        enqueue_outbound_editing(&st, "fake", "42", "补发结果", "555").expect("enqueue");
        SENT_LOG.with(|log| log.borrow_mut().clear());
        let (sent, failed) = drain_one(&st).expect("drain");
        assert_eq!((sent, failed), (1, 0));
        let recorded = SENT_LOG.with(|log| log.borrow().last().cloned());
        let Some(recorded) = recorded else {
            panic!("渠道一次都没被 send 过");
        };
        assert_eq!(
            recorded.edit_of.as_deref(),
            Some("555"),
            "payload 的 edit_of 必须原样送到渠道（不能被 None 顶掉）"
        );
    }

    #[test]
    fn 补发重试同一任务同一文本只留一行() {
        // ⛔ 超时重试/调用方重入时，同一结果不能进两行 ——
        // 否则扫补发会把同一条发两遍。id 是效果键（任务 + 内容），不是 uuid。
        let st = store("dedup-delivery");
        let fail = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: true };
        for _ in 0..2 {
            assert!(
                deliver_result(&st, &fail, "fake", "b1", "42", "111", "t1", "结果正文").is_err(),
                "发送失败必须返回 Err（这次确实没发出去）"
            );
        }
        let rows = st.due_deliveries(100).expect("读补发队列");
        assert_eq!(rows.len(), 1, "同一意图重试只留一行，实际：{}", rows.len());
        assert_eq!(rows[0].task_id, "t1");
    }

    #[test]
    fn 补发文本变了就是新意图() {
        // 同一任务、不同文本 = 两次不同的事，必须各留一行。
        // 反例：若按 task_id 去重，编辑后的重发会被吞掉。
        let st = store("dedup-delivery-text");
        let fail = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: true };
        assert!(deliver_result(&st, &fail, "fake", "b1", "42", "111", "t1", "第一版").is_err());
        assert!(deliver_result(&st, &fail, "fake", "b1", "42", "111", "t1", "第二版").is_err());
        let rows = st.due_deliveries(100).expect("读补发队列");
        assert_eq!(rows.len(), 2, "文本不同必须各留一行，实际：{}", rows.len());
    }

    #[test]
    fn outbox_drain_of_a_legacy_payload_still_sends_normally() {
        // 防雪崩的端到端那一半：老 payload（无 `edit_of` 键）照发，且
        // **不带**编辑意图 —— 不能因为缺键就判失败。
        let st = store("legacyedit");
        enqueue_outbound(&st, "fake", "42", "纯文字").expect("enqueue");
        SENT_LOG.with(|log| log.borrow_mut().clear());
        let (sent, failed) = drain_one(&st).expect("drain");
        assert_eq!((sent, failed), (1, 0), "老 payload 必须照发不误");
        let recorded = SENT_LOG.with(|log| log.borrow().last().cloned());
        assert_eq!(
            recorded.map(|m| m.edit_of),
            Some(None),
            "缺 edit_of 键 = 不编辑，而不是失败"
        );
    }

    #[test]
    fn enqueue_writes_the_edit_of_key_only_when_asked_to() {
        let st = store("editkey");
        let plain = enqueue_outbound(&st, "fake", "42", "普通").expect("enqueue");
        let editing = enqueue_outbound_editing(&st, "fake", "42", "补发", "555").expect("enqueue");
        // 空目标不该写出一个空的 `edit_of`（那会让渠道去编辑一条 id 为空的消息）。
        let blank = enqueue_outbound_editing(&st, "fake", "42", "补发", "   ").expect("enqueue");
        let rows = st
            .drain_outbox(10, &(chrono::Utc::now() + chrono::Duration::seconds(120)).to_rfc3339())
            .expect("drain");
        let payload_of = |id: &str| {
            rows.iter()
                .find(|(row_id, _, _)| row_id == &id)
                .map(|(_, _, payload)| payload.clone())
                .unwrap_or_default()
        };
        let plain_payload: serde_json::Value =
            serde_json::from_str(&payload_of(&plain)).expect("json");
        assert!(
            plain_payload.get("edit_of").is_none(),
            "无编辑意图时不该多写这个键：{plain_payload}"
        );
        let editing_payload: serde_json::Value =
            serde_json::from_str(&payload_of(&editing)).expect("json");
        assert_eq!(editing_payload.get("edit_of").and_then(|v| v.as_str()), Some("555"));
        let blank_payload: serde_json::Value =
            serde_json::from_str(&payload_of(&blank)).expect("json");
        assert!(
            blank_payload.get("edit_of").is_none(),
            "空 edit_of 不该进 payload：{blank_payload}"
        );
    }

    #[test]
    fn direct_delivery_paths_still_ask_the_channel_to_edit() {
        // 直发 / 补发两段本来就填了 `edit_of` —— 锁住这个约定，
        // 渠道侧就是拿它去走 `editMessageText`（改不成就降级发新消息并记账）。
        let st = store("editdirect");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let ok = deliver_result(&st, &ch, "fake", "b1", "42", "m1", "t1", "结果在这").expect("ok");
        assert!(ok);
        assert_eq!(ch.sent.borrow()[0].edit_of.as_deref(), Some("m1"));
    }

    // ---- 停止信号（切片 C4：IM 侧接线）----
    //
    // **三态的诚实边界**（「已请求」≠「已停」）：
    // - `NothingToStop`：没得停。**禁用词表一个都不许命中**。
    // - `Signalled`：已把信号递到在飞轮次。**同样不许**说「已停」——
    //   那一刻轮次可能已经跑完了。
    // - `Stopped`：回库核对到 `TaskStatus::Cancelled`，并报「第几跳」。

    /// 「已经停下来了」类断言词的**禁用词表**。
    ///
    /// 用户是在 IM 里**扫一眼**找结论的：这几个字一旦出现，他读到的结论就是
    /// 「停了」。故「没得停」与「已请求停止」两句都不许命中任何一个。
    const BANNED_STOP_CLAIMS: [&str; 6] =
        ["已停止", "已停", "已中止", "已终止", "停止成功", "已打断"];

    /// 造一个「机器人已绑定到某会话」的夹具（`/stop` 按**绑定会话**查登记）。
    fn bound_bot(st: &NeobotStore, case: &str) -> (BotRow, String) {
        let convo = st.create_conversation("dm", case, &[]).expect("convo");
        let mut b = bot();
        b.conversation_id = Some(convo.clone());
        st.upsert_bot(&b).expect("bot");
        (b, convo)
    }

    /// `/stop` 命中**且有在飞轮次** ⇒ 令牌真被翻掉，且回执**只**声称
    /// 「递到了信号」。
    ///
    /// 反向断言是这条的重点：把「已递信号」写成「已停」就是撒谎 —— 令牌被翻
    /// 只说明**信号送达**，那一轮停不停、停在第几跳，只有它自己的终态能说。
    #[test]
    fn stop_with_a_live_turn_flips_the_token_and_only_claims_the_signal() {
        let st = store("liveturn");
        let cfg = config("liveturn");
        open_channel(&st, "open");
        let (b, convo) = bound_bot(&st, "liveturn");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        let live = StopToken::new();
        let _registration = register_run(Some(&convo), &live);
        let got = on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "/stop", true)).expect("run");
        // ① 可观测量：登记表里那枚令牌**真的**被翻掉了（不是「返回了 Ok」）。
        assert!(live.is_cancelled(), "在飞轮次的令牌必须被翻掉");
        // ② 文案：只说「递到了」，并把「谁来说停没停」指给那一轮自己。
        let receipt = ch.sent.borrow()[0].text.clone();
        assert!(receipt.contains("已把停止信号递到正在跑的那一轮"), "{receipt}");
        for claim in BANNED_STOP_CLAIMS {
            assert!(
                !receipt.contains(claim),
                "信号送达 ≠ 轮次已停，回执不能命中 {claim:?}：{receipt}"
            );
        }
        // 返回值与发出去的那句逐字相同（不许一个说 A 一个说 B）。
        assert_eq!(got.reply(), receipt, "返回值与实际发出的回执必须一致");
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    /// `/stop` 命中但**没有在飞轮次** ⇒ 如实说「没得停」。
    ///
    /// **这正是生产 IM 今天会走的分支**：同步调度下 `/stop` 被读到时上一轮
    /// 早已结束、登记已注销。它是最该被钉死的一条 —— 一旦有人把回执改成
    /// 「已停」，用户就会以为停掉了，而实际什么都没发生。
    #[test]
    fn stop_without_a_live_turn_reports_nothing_to_stop() {
        let st = store("nothing");
        let cfg = config("nothing");
        open_channel(&st, "open");
        let (b, convo) = bound_bot(&st, "nothing");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        // **没有任何登记**：轮次不在跑。
        let got = on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "/stop", true)).expect("run");
        assert!(matches!(got, InboundOutcome::Handled { .. }));
        // ① 登记表这一层如实报「没得停」（不是「递到了」）。
        match signal_run_cancel(Some(&convo)) {
            StopState::NothingToStop { reason } => {
                assert!(reason.contains("刚查过"), "要说清是查过之后没有：{reason}");
                assert!(reason.contains("没有可停的轮次"), "要说清是没得停：{reason}")
            }
            other => panic!("没有在飞轮次时不该是 {other:?}"),
        }
        // ② 回执必须说「没得停」，且一个禁用词都不许出现。
        let sent = ch.sent.borrow();
        assert_eq!(sent.len(), 1, "指令必须回一句");
        let receipt = sent[0].text.clone();
        assert!(receipt.contains("没有可停的轮次"), "如实说没得停：{receipt}");
        for claim in BANNED_STOP_CLAIMS {
            assert!(
                !receipt.contains(claim),
                "没得停时回执绝不能命中 {claim:?}：{receipt}"
            );
        }
        assert_eq!(got.reply(), receipt, "返回值与实际发出的回执必须一致");
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    /// 停止信号**不跨会话**：A 名下登记着在飞轮次，B 会话发来的 `/stop`
    /// 既不该报「已递信号」，也不该动 A 的令牌。
    ///
    /// 与 `nt_agent` 的 `cancel_does_not_cross_conversations` 同一条律的
    /// **登记表那一半**（那边验的是令牌本身，这边验的是键）。
    #[test]
    fn stop_reaches_only_the_conversation_the_turn_runs_in() {
        let st = store("crossonly");
        let cfg = config("crossonly");
        open_channel(&st, "open");
        let convo_a = st.create_conversation("dm", "A", &[]).expect("A");
        let (b, convo_b) = bound_bot(&st, "B");
        assert_ne!(convo_a, convo_b);
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        let live = StopToken::new();
        let _registration = register_run(Some(&convo_a), &live);
        // 机器人绑在 B，它收到的 `/stop` 只该查 B。
        on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "/stop", true)).expect("run");
        assert!(!live.is_cancelled(), "A 名下的令牌不该被 B 的 /stop 翻掉");
        let receipt = ch.sent.borrow()[0].text.clone();
        assert!(receipt.contains("没有可停的轮次"), "如实说没得停：{receipt}");
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    /// 一个「跑轮途中被叫停」的引擎：每次模型调用都去**翻登记表里那枚令牌**
    /// —— 代「第二个执行流在跑轮途中把 `/stop` 递了过来」。
    ///
    /// 为什么必须**每次**都翻而不是只翻一次：单次调用的引擎一返回就收手
    /// （`tool_calls` 为空 ⇒ 那一轮立刻 `break`），根本走不到任何取消检查点。
    /// 它回一条 `bash` 调用，跳循环才继续，于是 C2（工具边界）当场拦下。
    struct SelfStoppingEngine {
        convo: String,
        calls: std::sync::Mutex<usize>,
    }

    impl SelfStoppingEngine {
        fn new(convo: &str) -> Self {
            Self {
                convo: convo.to_owned(),
                calls: std::sync::Mutex::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.lock().map(|n| *n).unwrap_or(0)
        }
    }

    impl crate::nt_engine::EngineAdapter for SelfStoppingEngine {
        fn engine_id(&self) -> &str {
            "self-stopping"
        }

        fn probe(&self) -> Result<String, crate::NtBotError> {
            Ok("self-stopping ready".to_owned())
        }

        fn run_turn(
            &self,
            _prompt: &str,
            _inbox: &[String],
        ) -> Result<crate::nt_engine::EngineTurn, crate::NtBotError> {
            if let Ok(mut calls) = self.calls.lock() {
                *calls += 1;
            }
            // 走**生产同一条路**（按 convo 查登记 → 翻旗），不是去摸私有字段。
            match signal_run_cancel(Some(&self.convo)) {
                StopState::Signalled { .. } => {}
                other => {
                    return Err(crate::NtBotError::Engine {
                        engine: self.engine_id().to_owned(),
                        reason: format!("跑轮途中叫停没递到：{other:?}"),
                    });
                }
            }
            Ok(crate::nt_engine::EngineTurn {
                assistant_text: String::new(),
                status: crate::nt_types::TurnStatus::Continue,
                tool_calls: vec![crate::nt_types::ToolCall {
                    id: "c0".to_owned(),
                    name: crate::nt_types::ToolName::Bash,
                    args: serde_json::json!({"command": "echo hi"}),
                }],
                usage: None,
                side_effects: Vec::new(),
            })
        }
    }

    /// 端到端：IM 跑轮途中被叫停 ⇒ 库里落 `TaskStatus::Cancelled` 并带
    /// 「第几跳」，回执说的是「已停（第 N/M 跳）」而不是「已行动，等外部条件」。
    ///
    /// 这一条是「信号送达 → 跑轮真停」整条链的**唯一**证明：令牌由
    /// `on_inbound` 登记、也由 `on_inbound` 递进跑轮，测试只负责在跑轮途中
    /// 走公开接口翻它 —— 三个环节少任何一个，这三条断言都会红。
    #[test]
    fn a_stopped_inbound_turn_lands_cancelled_with_its_hop() {
        let st = store("stoprun");
        let cfg = config("stoprun");
        open_channel(&st, "open");
        let (b, convo) = bound_bot(&st, "stoprun");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = SelfStoppingEngine::new(&convo);
        let task_id = match on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "跑一下", true))
            .expect("run")
        {
            InboundOutcome::Turn { task_id, .. } => task_id,
            other => panic!("普通消息该走跑轮：{other:?}"),
        };
        assert!(!task_id.is_empty(), "认领不到本轮任务 id 就不该声称停在哪一跳");
        // ① 落库：cancelled + 说得出第几跳（`max_steps` = 2，被拦在第 0 跳）。
        let task = st.get_task(&task_id).expect("get").expect("row");
        assert_eq!(task.status, TaskStatus::Cancelled, "{task:?}");
        let error = task.error.clone().unwrap_or_default();
        assert!(error.contains("stopped by user"), "{error}");
        assert!(error.contains("at hop 0/2"), "要能说清第几跳：{error}");
        // ② 叫停后不许再发起第 2 次模型调用。
        assert_eq!(engine.calls(), 1, "叫停后不该再有下一次模型调用");
        // ③ 回执说的是「已停（第 0/2 跳）」—— 不是「已行动，等外部条件」。
        let receipt = ch.sent.borrow()[0].text.clone();
        assert!(receipt.contains("已停"), "{receipt}");
        assert!(receipt.contains("第 0/2 跳"), "要说清停在哪一跳：{receipt}");
        assert!(
            !receipt.contains("已行动，等外部条件"),
            "被叫停的轮次不该被说成「已行动」：{receipt}"
        );
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    /// 注销律：轮次结束后登记当场清掉（不泄漏），再调取消**如实报没得停**。
    ///
    /// 漏注销的后果不是「多占一格内存」，而是**下一轮跑起来时 `/stop` 去翻了
    /// 一枚上一轮的令牌** —— 用户以为停了下一轮，其实停的是空气。
    #[test]
    fn the_registration_is_gone_once_the_turn_ends() {
        let st = store("dereg");
        let cfg = config("dereg");
        open_channel(&st, "open");
        let (b, convo) = bound_bot(&st, "dereg");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "第一条", true)).expect("run");
        // 轮次已结束 ⇒ 登记表里不该再有它。
        match signal_run_cancel(Some(&convo)) {
            StopState::NothingToStop { .. } => {}
            other => panic!("轮次结束后不该还有可停的登记：{other:?}"),
        }
        assert!(
            run_registry()
                .lock()
                .map(|table| !table.contains_key(&convo))
                .unwrap_or(false),
            "登记表不该残留本会话（否则会翻到空气）"
        );
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    /// 跑轮失败（`?` 早退）也不能漏注销 —— 否则那枚令牌会一直挂着，
    /// 让之后的 `/stop` 报「已递信号」而实际什么都没在跑。
    #[test]
    fn a_failing_turn_also_drops_its_registration() {
        let st = store("faildereg");
        let cfg = config("faildereg");
        open_channel(&st, "open");
        let (b, convo) = bound_bot(&st, "faildereg");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        // 引擎在跑轮途中报「叫停没递到」⇒ 跑轮以 Err 结束（`on_inbound` 走 `?`）。
        let engine = SelfStoppingEngine::new("别的会话");
        let failed = on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "跑一下", true));
        assert!(failed.is_err(), "跑轮失败该如实报错");
        match signal_run_cancel(Some(&convo)) {
            StopState::NothingToStop { .. } => {}
            other => panic!("跑轮早退后不该残留登记：{other:?}"),
        }
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    /// 跑完的轮次**没有**被停过 ⇒ 回执是正文，且一个禁用词都不许出现。
    ///
    /// 「已停」那一支是新加的，最容易出的错是它在**没被停**的轮次上也响 ——
    /// 那会让用户看到一句根本没发生的事。
    #[test]
    fn an_unstopped_turn_is_never_reported_as_stopped() {
        let st = store("unstopped");
        let cfg = config("unstopped");
        open_channel(&st, "open");
        let (b, convo) = bound_bot(&st, "unstopped");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let engine = crate::nt_engine::LocalEchoEngine;
        let task_id = match on_inbound(&st, &cfg, &engine, &ch, &b, &msg("m1", "在吗", true))
            .expect("run")
        {
            InboundOutcome::Turn { task_id, .. } => task_id,
            other => panic!("普通消息该走跑轮：{other:?}"),
        };
        assert!(turn_stop_state(&st, &task_id).is_none(), "没被停就不该报停");
        // 轮次已收尾 ⇒ 它也**不是**一个取消目标（否则会翻到一枚空令牌）。
        assert!(
            matches!(
                signal_run_cancel(Some(&convo)),
                StopState::NothingToStop { .. }
            ),
            "跑完的轮次不该还挂在登记表上"
        );
        let receipt = ch.sent.borrow()[0].text.clone();
        for claim in BANNED_STOP_CLAIMS {
            assert!(!receipt.contains(claim), "正常轮次的回执命中 {claim:?}：{receipt}");
        }
        // 认不出本轮任务 id 时**连较弱的话都不说**（不许拿「最新那条」去猜）。
        assert!(turn_stop_state(&st, "").is_none());
        drop(std::fs::remove_dir_all(&cfg.data_dir));
    }

    // ---- 附件 ----

    struct AttChannel {
        sent: std::cell::RefCell<Vec<OutboundMessage>>,
    }

    impl ChannelAdapter for AttChannel {
        fn channel_id(&self) -> &str { "att" }
        fn display_name(&self) -> &str { "附件渠道" }
        fn token_env(&self) -> &str { "NEOBOT_ATT" }
        fn probe(&self) -> crate::nt_channel::ChannelHealth {
            crate::nt_channel::ChannelHealth { ok: true, detail: String::new(), info: String::new() }
        }
        fn poll(&mut self) -> Result<Vec<InboundMessage>, NtBotError> { Ok(Vec::new()) }
        fn send(&self, out: &OutboundMessage) -> Result<String, NtBotError> {
            self.sent.borrow_mut().push(out.clone());
            Ok("m1".to_owned())
        }
        fn fetch_attachment(
            &self,
            id: &str,
            dest: &std::path::Path,
        ) -> Result<std::path::PathBuf, NtBotError> {
            if id == "bad" {
                return Err(NtBotError::Io("平台说没有这个文件".to_owned()));
            }
            std::fs::create_dir_all(dest).map_err(|e| NtBotError::Io(e.to_string()))?;
            let path = dest.join(format!("{id}.txt"));
            std::fs::write(&path, "file body").map_err(|e| NtBotError::Io(e.to_string()))?;
            Ok(path)
        }
    }

    fn att_msg() -> InboundMessage {
        let mut m = msg("att1", "看下这个", true);
        m.attachments = vec![
            crate::nt_channel::InboundAttachment {
                id: "good".to_owned(),
                name: "report.md".to_owned(),
                size: 9,
            },
            crate::nt_channel::InboundAttachment {
                id: "bad".to_owned(),
                name: "lost.png".to_owned(),
                size: 0,
            },
        ];
        m
    }

    #[test]
    fn download_attachments_saves_ok_and_reports_failures() {
        let cfg = config("att");
        let ch = AttChannel { sent: std::cell::RefCell::new(Vec::new()) };
        let (saved, notes) = download_attachments(&cfg, &ch, &att_msg());
        // 好的落了盘，坏的如实报出来。
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].name, "report.md");
        assert_eq!(saved[0].kind, "text");
        assert!(std::path::Path::new(&saved[0].path).is_file());
        assert_eq!(notes.len(), 2, "成功与失败各一行");
        assert!(notes[0].contains("read_file"), "要告诉模型怎么读：{}", notes[0]);
        assert!(notes[1].contains("没取到"), "失败要如实说：{}", notes[1]);
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn record_attachments_registers_against_the_convo() {
        let st = store("attrec");
        let convo = st.create_conversation("dm", "A", &[]).expect("convo");
        let saved = vec![SavedAttachment {
            name: "a.txt".to_owned(),
            kind: "text",
            path: "/tmp/a.txt".to_owned(),
            size: 5,
        }];
        let notes = record_attachments(&st, &convo, &saved);
        assert!(notes.is_empty(), "{notes:?}");
        let rows = st.list_attachments(&convo).expect("list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "a.txt");
        assert_eq!(rows[0].kind, "text");
    }

    #[test]
    fn attachment_reaches_the_model_as_a_readable_path() {
        let st = store("attturn");
        let cfg = config("attturn");
        open_channel(&st, "open");
        let b = bot();
        st.upsert_bot(&b).expect("bot");
        let ch = AttChannel { sent: std::cell::RefCell::new(Vec::new()) };
        let engine = crate::nt_engine::LocalEchoEngine;
        let got = on_inbound(&st, &cfg, &engine, &ch, &b, &att_msg()).expect("run");
        assert!(matches!(got, InboundOutcome::Turn { .. }), "{got:?}");
        // 附件已登记到**新建的**会话上（不是空串）。
        let bound = st.get_bot("fake", "b1").expect("get").expect("some")
            .conversation_id
            .expect("bound");
        let rows = st.list_attachments(&bound).expect("list");
        assert_eq!(rows.len(), 1, "附件应挂在真实会话上");
        // 且模型那一轮真的读到了那个文件。
        let reply = last_reply_of(&st, &bound).unwrap_or_default();
        assert!(reply.contains("report.md"), "模型应提到附件名：{reply}");
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn inbound_attachment_lands_inside_the_jail_and_is_reachable() {
        // 回归锁：附件曾落在 `<data_dir>/attachments`，而 jail 只放行
        // `<data_dir>/workspace` —— 兄弟目录，于是**收了但模型永远读不到**。
        // 这条测试按「模型真的会怎么用」来验：拿 note 里给的相对路径，
        // 走一遍真实的 jail + read_file。
        let st = store("reachable");
        let cfg = config("reachable");
        let ch = AttChannel { sent: std::cell::RefCell::new(Vec::new()) };
        let mut m = msg("att1", "看下这个", true);
        m.attachments = vec![crate::nt_channel::InboundAttachment {
            id: "good".to_owned(),
            name: "notes.md".to_owned(),
            size: 9,
        }];
        let (saved, notes) = download_attachments(&cfg, &ch, &m);
        assert_eq!(saved.len(), 1);
        let on_disk = &saved[0].path;
        // 1) 物理位置在工作区之内（不是兄弟目录）。
        let real = std::fs::canonicalize(&cfg.workspace_dir).expect("ws");
        let file = std::fs::canonicalize(on_disk).expect("file");
        assert!(file.starts_with(&real), "附件落在 jail 之外：{on_disk}");

        // 2) note 给的是**相对路径**，且能通过网关的越狱判定。
        let note = notes.first().expect("note").clone();
        let rel = note
            .rsplit_once("path=")
            .map(|(_, tail)| tail.trim_end_matches('）').to_owned())
            .expect("note 里应有 path=");
        assert!(!rel.starts_with('/'), "给了绝对路径，网关必拒：{rel}");
        assert!(!crate::nt_policy::PolicyContext {
            tool: crate::nt_types::ToolName::ReadFile,
            actor: crate::nt_policy::Actor::Bot,
            human_has_control: false,
            file_path: Some(rel.clone()),
            command: None,
            computer_action: None,
            computer_target: None,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
        }
        .file_path
        .map(|p| p.starts_with('/') || p.contains(".."))
        .unwrap_or(false), "相对路径不该含越狱片段：{rel}");

        // 3) 用这个相对路径真的能读到内容。
        let joined = crate::nt_workspace::jail_join(&cfg.workspace_dir, &rel).expect("jail");
        assert_eq!(
            std::fs::read_to_string(&joined).ok().as_deref(),
            Some("file body"),
            "模型按 note 给的路径应当读得到"
        );
        let _ = st;
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn image_attachments_are_pointed_at_read_image() {
        let cfg = config("imgtool");
        let ch = AttChannel { sent: std::cell::RefCell::new(Vec::new()) };
        let mut m = msg("att2", "看图", true);
        m.attachments = vec![crate::nt_channel::InboundAttachment {
            id: "good".to_owned(),
            name: "shot.png".to_owned(),
            size: 9,
        }];
        let (_saved, notes) = download_attachments(&cfg, &ch, &m);
        let note = notes.first().expect("note");
        assert!(note.contains("read_image"), "图片应指向 read_image：{note}");
        assert!(!note.contains("read_file"), "图片不该指向 read_file（读不出二进制）：{note}");
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn channels_without_attachment_support_fail_loudly() {
        // 默认实现必须**报错** —— 静默返回空会让用户以为文件收到了。
        let cfg = config("nosupport");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        let (saved, notes) = download_attachments(&cfg, &ch, &att_msg());
        assert!(saved.is_empty());
        assert_eq!(notes.len(), 2);
        assert!(notes.iter().all(|n| n.contains("没取到")), "{notes:?}");
        assert!(
            notes.iter().any(|n| n.contains("does not support")),
            "要说清是渠道不支持：{notes:?}"
        );
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    // ---- 补发 ----

    #[test]
    fn deliver_result_sends_directly_when_channel_works() {
        let st = store("deliver");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        st.upsert_channel("fake", "假", "open", 5).expect("ch");
        let b = bot();
        st.upsert_bot(&b).expect("bot");
        let ok = deliver_result(&st, &ch, "fake", "b1", "42", "m1", "t1", "结果在这")
            .expect("deliver");
        assert!(ok);
        assert_eq!(ch.sent.borrow().len(), 1);
        assert_eq!(ch.sent.borrow()[0].edit_of.as_deref(), Some("m1"));
        // 直接发成了就不该留待补发。
        assert!(st.due_deliveries(10).expect("due").is_empty());
    }

    #[test]
    fn deliver_result_queues_for_retry_when_send_fails() {
        let st = store("queue");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: true };
        st.upsert_channel("fake", "假", "open", 5).expect("ch");
        st.upsert_bot(&bot()).expect("bot");
        let failed = deliver_result(&st, &ch, "fake", "b1", "42", "m1", "t1", "结果在这");
        assert!(failed.is_err(), "发不出去要如实报错");
        // 但消息进了待补发，不是丢了。
        let due = st.due_deliveries(10).expect("due");
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].text, "结果在这");
        assert_eq!(due[0].origin_message, "m1");
    }

    #[test]
    fn deliver_result_declines_empty_text() {
        let st = store("empty");
        let ch = FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false };
        st.upsert_channel("fake", "假", "open", 5).expect("ch");
        st.upsert_bot(&bot()).expect("bot");
        assert!(!deliver_result(&st, &ch, "fake", "b1", "42", "m1", "t1", "   ")
            .expect("deliver"));
        assert!(ch.sent.borrow().is_empty());
    }

    #[test]
    fn sweep_pending_gives_up_after_max_attempts() {
        let st = store("sweep");
        let mut reg = crate::nt_channel::ChannelRegistry::new();
        reg.register(Box::new(FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: true }))
            .expect("register");
        st.enqueue_delivery(&crate::nt_store::PendingDelivery {
            id: "d1".to_owned(),
            channel: "fake".to_owned(),
            bot_id: "b1".to_owned(),
            chat: "42".to_owned(),
            origin_message: "m1".to_owned(),
            text: "迟到的结果".to_owned(),
            task_id: "t1".to_owned(),
            attempts: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .expect("enqueue");
        let mut gave_up_total = 0;
        for _ in 0..5 {
            let (_done, gave_up) = sweep_pending(&st, &mut reg).expect("sweep");
            gave_up_total += gave_up;
        }
        assert_eq!(gave_up_total, 1, "到顶后只该放弃一次，不该反复放弃");
        assert!(st.due_deliveries(10).expect("due").is_empty());
    }

    #[test]
    fn sweep_pending_delivers_once_channel_recovers() {
        let st = store("recover");
        let mut reg = crate::nt_channel::ChannelRegistry::new();
        st.enqueue_delivery(&crate::nt_store::PendingDelivery {
            id: "d1".to_owned(),
            channel: "fake".to_owned(),
            bot_id: "b1".to_owned(),
            chat: "42".to_owned(),
            origin_message: "m1".to_owned(),
            text: "迟到的结果".to_owned(),
            task_id: "t1".to_owned(),
            attempts: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .expect("enqueue");
        // 渠道还没注册 → 只记尝试、不放弃。
        let (done, gave_up) = sweep_pending(&st, &mut reg).expect("sweep");
        assert_eq!((done, gave_up), (0, 0), "渠道没注册不该算放弃");
        assert_eq!(st.due_deliveries(10).expect("due").len(), 1);
        // 渠道注册回来了。
        reg.register(Box::new(FakeChannel { sent: std::cell::RefCell::new(Vec::new()), fail: false }))
            .expect("register");
        let (done, _) = sweep_pending(&st, &mut reg).expect("sweep");
        assert_eq!(done, 1);
        assert!(st.due_deliveries(10).expect("due").is_empty());
    }

    #[test]
    fn title_falls_back_to_sender_id() {
        assert_eq!(default_convo_title("Alice", "alice"), "Alice");
        assert_eq!(default_convo_title("", "bob"), "bob");
        assert_eq!(default_convo_title("  ", "  "), "IM 会话");
        // 超长截断到 40 字符。
        let long = "x".repeat(100);
        assert_eq!(default_convo_title(&long, "s").chars().count(), 40);
    }
}

/// 测试辅助：把 `Handled` 的 reply 拿出来（避免在断言里写两遍长结构）。
#[cfg(test)]
trait ReplyOf {
    fn reply(&self) -> String;
}

#[cfg(test)]
impl ReplyOf for InboundOutcome {
    fn reply(&self) -> String {
        match self {
            InboundOutcome::Handled { reply, .. } => reply.clone(),
            _ => String::new(),
        }
    }
}
