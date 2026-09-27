//! `nt_routine` — 定时例行（单机版）.
//!
//! 设计点（只取可单机落地的部分）：
//! - firing 帧（frame/read 二函数，单声明双读）。
//!   指令原文是 schedule 口吻（“每 15 分钟…”），裸发给模型会被读成
//!   “要不要建 schedule”，帧负责声明三件事：正在 firing、本轮干活、不许管 routines。
//! - 15min 地板（模型哄不住地板）。
//! - 10 连败自停（此处记 error 列 + 审计）。
//! - 最多 20 个启用例行、指令 2000 字上限。
//!
//! 不取：cron 表达式（无 cron 依赖，用 interval_secs）、channel 通知、
//! 多租户 owner（owner 为自由串，单机座位）。

use chrono::Utc;

use crate::nt_config::NeobotConfig;
use crate::nt_engine::EngineAdapter;
use crate::nt_error::NtBotError;
use crate::nt_store::NeobotStore;
use crate::nt_types::TurnStatus;

/// 例行最短间隔秒（15min 地板）。
pub const MIN_INTERVAL_SECS: i64 = 15 * 60;
/// 连败自停上限。
pub const FATIGUE_LIMIT: i64 = 10;
/// 同时启用的例行上限。
pub const MAX_ENABLED_ROUTINES: i64 = 20;
/// 指令上限（字符数）。
pub const MAX_INSTRUCTION_CHARS: usize = 2000;

/// 例行行（`routines` 表 1:1）。
#[derive(Debug, Clone)]
pub struct Routine {
    pub name: String,
    pub interval_secs: i64,
    pub instruction: String,
    pub owner: String,
    pub failures: i64,
    pub disabled: i64,
    pub next_run_at: i64,
    pub last_run_at: i64,
    pub last_error: Option<String>,
}

/// firing 帧（三句中文声明：正在 firing、本轮干活、不许管 routines）。
pub const FIRING_FRAME: [&str; 3] = [
    "你的一个 routine 正在按计划 firing，这就是那次 firing。",
    "在本轮内执行下面的指令：现在就干活，然后说清楚发生了什么。",
    "除非指令本身要求，否则不要创建、列出或修改任何 routine。",
];

/// 拼 firing 提示词（只包新消息；历史回放不再包，防帧叠帧）。
pub fn frame_firing(instruction: &str) -> String {
    format!("{}\n\n{instruction}", FIRING_FRAME.join("\n"))
}

/// 识别 firing 文本并剥帧（展示层/标题器用；非 firing 返回 None）。
/// 空指令的完整帧仍是 firing（返回 Some("")），不撒谎。
/// 整帧前缀匹配：只引用首句的调试文本不算 firing。
pub fn read_firing(text: &str) -> Option<String> {
    let prefix = format!("{}\n\n", FIRING_FRAME.join("\n"));
    text.strip_prefix(&prefix).map(str::to_owned)
}

/// 跑一次到期 firing：取例行 → 帧包装 → 走一轮本地 turn → 落结果。
/// 返回 `(TurnStatus, 是否本次致禁用)`。
pub fn fire_routine(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    name: &str,
    now_epoch: i64,
) -> Result<(TurnStatus, bool), NtBotError> {
    let routine = store
        .list_routines()?
        .into_iter()
        .find(|r| r.name == name)
        .ok_or_else(|| NtBotError::Store(format!("no such routine '{name}'")))?;
    if routine.disabled != 0 {
        return Err(NtBotError::Store(format!("routine '{name}' is switched off")));
    }
    let framed = frame_firing(&routine.instruction);
    let title = format!("routine: {name}");
    // 例行归属稳定会话（同名 routine 共用 `routine:<name>` 群组，不刷屏）。
    let convo_title = format!("routine:{name}");
    let convo_id = match store
        .list_conversations()?
        .into_iter()
        .find(|c| c.title == convo_title)
    {
        Some(c) => c.id,
        None => store.create_conversation("group", &convo_title, &[])?,
    };
    let status = crate::nt_agent::run_local_turn_as(
        store,
        config,
        engine,
        crate::nt_policy::Actor::Routine,
        &routine_actor(name),
        &title,
        &framed,
        Some(&convo_id),
    )?;
    let ok = status == TurnStatus::Done;
    let error = if ok {
        None
    } else {
        Some(format!("turn ended as {}", status.as_str()))
    };
    store.record_routine_run(name, now_epoch, ok, error.as_deref())?;
    let after = store
        .list_routines()?
        .into_iter()
        .find(|r| r.name == name)
        .ok_or_else(|| NtBotError::Store(format!("routine '{name}' vanished")))?;
    Ok((status, after.disabled != 0))
}

/// sweep 一次：取出到期例行逐个 fire，返回 `(fired, disabled_now)` 明细。
/// engine 失败（spawn 不起等）记一次失败，不中断后续例行。
pub fn sweep_routines(
    store: &NeobotStore,
    config: &NeobotConfig,
    engine: &dyn EngineAdapter,
    now_epoch: i64,
) -> Result<Vec<(String, TurnStatus, bool)>, NtBotError> {
    let mut out = Vec::new();
    for routine in store.due_routines(now_epoch)? {
        match fire_routine(store, config, engine, &routine.name, now_epoch) {
            Ok((status, disabled)) => out.push((routine.name, status, disabled)),
            Err(err) => {
                let msg = err.to_string();
                // engine 起不来也记一次失败（疲劳律照转），不中断后续例行。
                let _recorded =
                    store.record_routine_run(&routine.name, now_epoch, false, Some(msg.as_str()));
                out.push((routine.name, TurnStatus::Blocked, false));
            }
        }
    }
    Ok(out)
}

/// 当前 epoch 秒（CLI/桌面层用；store 层保持无时钟）。
pub fn now_epoch() -> i64 {
    Utc::now().timestamp()
}

/// 本轮审计 actor 名（例行身份可查）：`routine:<name>`。
pub fn routine_actor(name: &str) -> String {
    format!("routine:{name}")
}

#[cfg(test)]
mod tests {
    use super::{frame_firing, read_firing};

    #[test]
    fn firing_frame_roundtrip() {
        let framed = frame_firing("check inbox");
        assert!(framed.contains("firing"));
        assert_eq!(read_firing(&framed).as_deref(), Some("check inbox"));
        // 非 firing 文本 → None
        assert_eq!(read_firing("check inbox"), None);
        // 只引用首句 → 仍是 None（整帧前缀才算）
        assert_eq!(read_firing("你的一个 routine 正在按计划 firing"), None);
        // 空指令整帧 → Some("")（是 firing，不撒谎）
        assert_eq!(read_firing(&frame_firing("")).as_deref(), Some(""));
    }
}
