//! `nt_cmd_channels` — IM 渠道的 IPC 面（列渠道 / 挂机器人 / 探活 / 开关 / 单轮收取）.
//!
//! 脱敏律：`NeobotChannelBotItem` 只出 `token_env`（**变量名**），与既有
//! `NeobotProviderItem` 的 `key_env` 同一套口径。token 值由适配器在收发时
//! 现读环境变量，任何状态接口都不经手它。
//!
//! 单轮收取（`neobot_channel_poll_once`）刻意做成**手动触发**而不是常驻线程：
//! 桌面 App 关掉就不该有后台长轮询在偷偷连公网（local-first 的承诺）。
//! 想要持续在线的用户自己开一个终端跑 `neobot channel serve`。

use neotrix_neobot::{
    nt_channel::{self, AccessMode, ChannelRegistry},
    nt_channel_dispatch,
    nt_channel_telegram, nt_store::parse_allow_list,
};

use super::{load_config, open_store};

/// `EngineSelection` → 装箱的 `dyn EngineAdapter`。
///
/// 提成独立函数而不是内联：让 `on_inbound` 那处的借用关系一眼看得懂
/// （`engine` 活得比 `adapter` 久，混在一个作用域里容易踩借用检查）。
fn engine_from(selection: super::EngineSelection) -> Box<dyn neotrix_neobot::EngineAdapter> {
    use super::EngineSelection;
    match selection {
        EngineSelection::Echo => Box::new(neotrix_neobot::LocalEchoEngine),
        EngineSelection::Cli(engine) => Box::new(engine),
        EngineSelection::Opencode(engine) => Box::new(engine),
        EngineSelection::Http(engine) => Box::new(engine),
    }
}

/// 建一个只装了内置适配器的注册表。
///
/// 现在只有 Telegram 一个；每加一个渠道就在这里加一行，**不改**任何调用方。
fn registry() -> ChannelRegistry {
    let mut reg = ChannelRegistry::new();
    // 注册失败只可能是重复 id —— 那是**编程错误**（同一适配器登记两次），
    // 不是用户错误，故这里选择让它静默：新渠道少一个会在探活时立刻显形，
    // 而让启动路径 panic 只会让整个 App 打不开 —— 那是更糟的失败模式。
    drop(reg.register(Box::new(nt_channel_telegram::TelegramChannel::new(""))));
    reg
}

/// 渠道 DTO。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotChannelItem {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    /// `open` | `allow` | `dm_only`。
    pub access_mode: String,
    /// 轮询节拍（秒）。
    pub poll_secs: i64,
    /// 适配器在不在（编译进来的渠道恒在；`false` 表示只是留了条配置）。
    pub available: bool,
}

/// 机器人 DTO（**不含** token 值）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotChannelBotItem {
    pub channel: String,
    pub bot_id: String,
    pub alias: String,
    /// token 的环境变量名（不是值）。
    pub token_env: String,
    pub conversation_id: Option<String>,
    pub model: String,
    /// 已解析的白名单（数组，方便前端渲染）。
    pub allow_list: Vec<String>,
    /// 该机器人的独立访问模式（空 = 跟渠道缺省）。
    pub access_mode: String,
    pub last_seen: Option<String>,
}

/// 探活结果 DTO。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotChannelHealth {
    pub channel: String,
    pub ok: bool,
    pub detail: String,
    pub info: String,
    /// 连续失败次数（>= 3 基本就是网络或 token 的问题）。
    pub failures: u32,
}

/// 已注册渠道（编译期能用的）。
#[tauri::command]
pub fn neobot_channel_catalog() -> Vec<String> {
    registry().ids()
}

/// 渠道列表（含配置）。
#[tauri::command]
pub fn neobot_channels() -> Result<Vec<NeobotChannelItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let known = registry().ids();
    let mut rows = store
        .list_channels()
        .map_err(|err| err.to_string())?
        .into_iter()
        .map(|row| NeobotChannelItem {
            available: known.iter().any(|id| id == &row.id),
            id: row.id,
            title: row.title,
            enabled: row.enabled,
            access_mode: row.access_mode,
            poll_secs: row.poll_secs,
        })
        .collect::<Vec<_>>();
    // 编进来了但还没建过配置的渠道也要列出来，否则用户无处可点。
    for id in known {
        if !rows.iter().any(|row| row.id == *id) {
            rows.push(NeobotChannelItem {
                available: true,
                id: id.clone(),
                title: id.clone(),
                enabled: false,
                access_mode: "allow".to_owned(),
                poll_secs: 5,
            });
        }
    }
    Ok(rows)
}

/// 建 / 改一个渠道（`id` 认不出访问模式时归一到全拒，绝不变成全放行）。
#[tauri::command]
pub fn neobot_channel_upsert(
    id: String,
    title: String,
    access_mode: String,
    poll_secs: Option<i64>,
) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    // 先用解析结果回写规范值，保证存进去的一定是三个合法档之一。
    let normalised = AccessMode::parse(&access_mode).as_str();
    store
        .upsert_channel(&id, &title, normalised, poll_secs.unwrap_or(5))
        .map_err(|err| err.to_string())?;
    Ok(())
}

/// 启停一个渠道。
#[tauri::command]
pub fn neobot_channel_toggle(id: String, enabled: bool) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .set_channel_enabled(&id, enabled)
        .map_err(|err| err.to_string())?;
    Ok(())
}

/// 某渠道的机器人列表。
#[tauri::command]
pub fn neobot_channel_bots(channel: String) -> Result<Vec<NeobotChannelBotItem>, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let channel_row = store.get_channel(&channel).map_err(|err| err.to_string())?;
    let default_mode = channel_row
        .as_ref()
        .map(|row| row.access_mode.clone())
        .unwrap_or_else(|| AccessMode::Allow.as_str().to_owned());
    let rows = store
        .list_bots(&channel)
        .map_err(|err| err.to_string())?;
    Ok(rows
        .into_iter()
        .map(|bot| NeobotChannelBotItem {
            access_mode: default_mode.clone(),
            allow_list: parse_allow_list(&bot.allow_list),
            channel: bot.channel,
            bot_id: bot.bot_id,
            alias: bot.alias,
            token_env: bot.token_env,
            conversation_id: bot.conversation_id,
            model: bot.model,
            last_seen: bot.last_seen,
        })
        .collect())
}

/// 挂 / 改一个机器人的**绑定**（工作区会话 / 模型 / 白名单）。
///
/// 签名里刻意**没有** `alias`（别名有自己的命令，且 upsert 不覆盖别名）
/// 与 token **值**参数：命令签名里没有的口子，UI 就没法不小心用上。
/// 死参数比注释更会骗人，故直接不收。
#[tauri::command]
pub fn neobot_channel_bot_upsert(
    channel: String,
    bot_id: String,
    token_env: String,
    model: Option<String>,
    allow_list: Option<String>,
    conversation_id: Option<String>,
) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let existing = store
        .get_bot(&channel, &bot_id)
        .map_err(|err| err.to_string())?;
    let row = neotrix_neobot::BotRow {
        channel,
        bot_id,
        // upsert 不覆盖别名（沿用 store 的语义），所以这里回填原值。
        alias: existing
            .as_ref()
            .map(|bot| bot.alias.clone())
            .unwrap_or_default(),
        token_env: token_env.trim().to_owned(),
        conversation_id,
        model: model.unwrap_or_default().trim().to_owned(),
        allow_list: allow_list.unwrap_or_default(),
        created_at: String::new(),
        last_seen: None,
    };
    if let Some(previous) = existing {
        let mut merged = row;
        merged.alias = previous.alias;
        merged.created_at = previous.created_at;
        merged.last_seen = previous.last_seen;
        store.upsert_bot(&merged).map_err(|err| err.to_string())
    } else {
        store.upsert_bot(&row).map_err(|err| err.to_string())
    }
}

/// 改别名（空 = 清除，回落平台原名）。
#[tauri::command]
pub fn neobot_channel_bot_alias(
    channel: String,
    bot_id: String,
    alias: String,
) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .set_bot_alias(&channel, &bot_id, &alias)
        .map_err(|err| err.to_string())?;
    Ok(())
}

/// 摘掉一个机器人。
#[tauri::command]
pub fn neobot_channel_bot_remove(channel: String, bot_id: String) -> Result<(), String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    store
        .remove_bot(&channel, &bot_id)
        .map_err(|err| err.to_string())?;
    Ok(())
}

/// 探活（读环境变量里的 token 去问平台一次）。
#[tauri::command]
pub fn neobot_channel_probe(channel: String) -> Result<NeobotChannelHealth, String> {
    let mut reg = registry();
    let Some(adapter) = reg.get_mut(&channel) else {
        return Err(format!("未知渠道：{channel}"));
    };
    let health = adapter.probe();
    // 连续失败次数由适配器自己维护；`&mut Box<dyn …>` 要先降成 `&dyn`。
    let failures = adapter
        .as_ref()
        .consecutive_failures();
    Ok(NeobotChannelHealth {
        channel,
        ok: health.ok,
        detail: health.detail,
        info: health.info,
        failures,
    })
}

/// 手动跑一次收取 + 出站排空 + 补发。
///
/// 返回 `(收到几条, 忽略/拒绝几条, 发出几条, 补发几条)`。
#[tauri::command]
pub fn neobot_channel_poll_once(
    channel: String,
) -> Result<NeobotPollReport, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let row = store
        .get_channel(&channel)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("未知渠道：{channel}"))?;
    if !row.enabled {
        return Ok(NeobotPollReport::skipped("渠道已停用"));
    }
    let mut reg = registry();
    let bots = store.list_bots(&channel).map_err(|err| err.to_string())?;
    if bots.is_empty() {
        return Ok(NeobotPollReport::skipped("这个渠道下还没挂机器人"));
    }
    let mut report = NeobotPollReport {
        skipped: None,
        received: 0,
        ignored: 0,
        sent: 0,
        deferred: 0,
    };
    // 收取：每个机器人各拉一次（同渠道可挂多个 bot，各自独立）。
    for bot in &bots {
        let Some(adapter) = reg.get_mut(&channel) else {
            break;
        };
        let Ok(inbound) = adapter.poll() else {
            break;
        };
        for msg in inbound {
            report.received += 1;
            // 引擎与桌面**同一条**路由（同一份 `resolve_engine`）——
            // IM 侧不该有另一套「IM 专用模型」。
            let selection = super::resolve_engine(&config)?;
            let engine = engine_from(selection);
            match nt_channel_dispatch::on_inbound(
                &store,
                &config,
                engine.as_ref(),
                adapter.as_ref(),
                bot,
                &msg,
                None,
            ) {
                Ok(outcome) => {
                    if !matches!(outcome, nt_channel_dispatch::InboundOutcome::Turn { .. }) {
                        report.ignored += 1;
                    }
                }
                Err(err) => {
                    return Err(format!("跑轮失败：{err}"));
                }
            }
        }
    }
    // 出站 + 补发。
    let (sent, _failed) =
        nt_channel_dispatch::drain_outbox_once(&store, &mut reg).map_err(|err| err.to_string())?;
    report.sent = sent as i64;
    let (done, _gave_up) =
        nt_channel_dispatch::sweep_pending(&store, &mut reg).map_err(|err| err.to_string())?;
    report.deferred = done as i64;
    nt_channel_dispatch::upkeep_best_effort(&store);
    Ok(report)
}

/// 一次收取的结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotPollReport {
    /// 整轮被跳过时的原因（跑都没跑）。
    pub skipped: Option<String>,
    pub received: i64,
    /// 收到但没跑轮（重复 / 被闸门拒 / 是指令）。
    pub ignored: i64,
    pub sent: i64,
    /// 补发成功的条数。
    pub deferred: i64,
}

impl NeobotPollReport {
    fn skipped(reason: &str) -> Self {
        Self {
            skipped: Some(reason.to_owned()),
            received: 0,
            ignored: 0,
            sent: 0,
            deferred: 0,
        }
    }
}

/// 当前进程是否编译进了哪些渠道（doctor 展示用）。
#[tauri::command]
pub fn neobot_channel_status() -> Vec<NeobotChannelItem> {
    registry()
        .ids()
        .into_iter()
        .map(|id| NeobotChannelItem {
            title: id.clone(),
            id,
            enabled: true,
            access_mode: "allow".to_owned(),
            poll_secs: 5,
            available: true,
        })
        .collect()
}

// 让 `nt_channel` 的白名单解析在 IPC 层也有名字（前端要对齐口径）。
#[tauri::command]
pub fn neobot_channel_parse_allow(raw: String) -> Vec<String> {
    let _ = nt_channel::AccessMode::Allow;
    parse_allow_list(&raw)
}
