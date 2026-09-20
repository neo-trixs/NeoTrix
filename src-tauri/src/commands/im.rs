//! IM Channel — 对接 DSH-IM 多渠道适配器模式
//!
//! 支持 9 个内置渠道：微信、飞书、钉钉、企业微信、QQ、Slack、Telegram、Discord、WhatsApp
//! 基于 DSH-IM 的 adapter 注册表模式，每个渠道独立配置和状态。

use anyhow::{Context, Result as AnyhowResult};
use crate::atomic_io;
use crate::domain::plugins::im::{
    BotConfig, ChannelConfig, ChannelType, DshMarketConfig, ImStatus, ALL_CHANNEL_TYPES,
};
use crate::domain::serde_json;
use crate::ipc::{self, IpcResponse};
use std::collections::HashMap;
use std::path::PathBuf;

/// 配置文件路径
fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("im_channels.json")
}

/// DSH 市场配置路径
fn dsh_market_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("dsh_market.json")
}

/// 加载渠道配置
fn load_channels() -> AnyhowResult<Vec<ChannelConfig>> {
    let path = config_path();
    if !path.exists() {
        return Ok(default_channels());
    }
    let content = std::fs::read_to_string(&path).context("Read IM config")?;
    serde_json::from_str(&content).context("Parse IM config")
}

/// 保存渠道配置
fn save_channels(channels: &[ChannelConfig]) -> AnyhowResult<()> {
    let path = config_path();
    atomic_io::ensure_parent_dir(&path).context("Create dir")?;
    atomic_io::write_json_atomic(&path, channels).context("Write config")
}

/// 加载 DSH 市场配置
fn load_dsh_market() -> AnyhowResult<DshMarketConfig> {
    let path = dsh_market_path();
    if !path.exists() {
        return Ok(DshMarketConfig::default());
    }
    let content = std::fs::read_to_string(&path).context("Read DSH market config")?;
    serde_json::from_str(&content).context("Parse DSH market config")
}

/// 保存 DSH 市场配置
fn save_dsh_market(config: &DshMarketConfig) -> AnyhowResult<()> {
    let path = dsh_market_path();
    atomic_io::ensure_parent_dir(&path).context("Create dir")?;
    atomic_io::write_json_atomic(&path, config).context("Write config")
}

/// 默认渠道配置（单一事实源：ALL_CHANNEL_TYPES）
fn default_channels() -> Vec<ChannelConfig> {
    ALL_CHANNEL_TYPES
        .iter()
        .map(|ch| ChannelConfig {
            channel: ch.clone(),
            enabled: false,
            bots: vec![],
            context_enhancement: false,
            proactive_delivery: false,
        })
        .collect()
}

// ═══════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════

/// 获取 IM 系统状态
#[tauri::command]
pub async fn im_status() -> IpcResponse<ImStatus> {
    let channels = match load_channels() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", format!("{e}")),
    };
    let dsh_market = match load_dsh_market() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", format!("{e}")),
    };

    let total_bots: usize = channels.iter().map(|c| c.bots.len()).sum();
    let connected_bots = channels
        .iter()
        .filter(|c| c.enabled)
        .map(|c| c.bots.len())
        .sum();

    ipc::ok(ImStatus {
        channels,
        total_bots,
        connected_bots,
        dsh_market_enabled: dsh_market.enabled,
    })
}

/// 获取所有渠道配置
#[tauri::command]
pub async fn im_list_channels() -> IpcResponse<Vec<ChannelConfig>> {
    match load_channels() {
        Ok(channels) => ipc::ok(channels),
        Err(e) => ipc::err("IM_LOAD_FAILED", format!("{e}")),
    }
}

/// 获取单个渠道配置
#[tauri::command]
pub async fn im_get_channel(channel: String) -> IpcResponse<ChannelConfig> {
    let channel_type = match ChannelType::from_name(&channel) {
        Some(ct) => ct,
        None => return ipc::err("IM_UNKNOWN_CHANNEL", &format!("Unknown channel: {}", channel)),
    };
    let channels = match load_channels() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", &e),
    };
    match channels.into_iter().find(|c| c.channel == channel_type) {
        Some(ch) => ipc::ok(ch),
        None => ipc::err("IM_CHANNEL_NOT_FOUND", &format!("Channel not found: {}", channel)),
    }
}

/// 启用/禁用渠道
#[tauri::command]
pub async fn im_toggle_channel(channel: String, enabled: bool) -> IpcResponse<ChannelConfig> {
    let channel_type = match ChannelType::from_name(&channel) {
        Some(ct) => ct,
        None => return ipc::err("IM_UNKNOWN_CHANNEL", &format!("Unknown channel: {}", channel)),
    };
    let mut channels = match load_channels() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", &e),
    };

    let mut found = false;
    let mut result = ChannelConfig {
        channel: channel_type.clone(),
        enabled: false,
        bots: vec![],
        context_enhancement: false,
        proactive_delivery: false,
    };

    for ch in &mut channels {
        if ch.channel == channel_type {
            ch.enabled = enabled;
            result = ch.clone();
            found = true;
            break;
        }
    }

    if found {
        if let Err(e) = save_channels(&channels) {
            return ipc::err("IM_SAVE_FAILED", format!("{e}"));
        }
        ipc::ok(result)
    } else {
        ipc::err("IM_CHANNEL_NOT_FOUND", &format!("Channel not found: {}", channel))
    }
}

/// 添加机器人
#[tauri::command]
pub async fn im_add_bot(
    channel: String,
    name: String,
    credential_type: String,
    workspace: Option<String>,
    model: Option<String>,
) -> IpcResponse<BotConfig> {
    let channel_type = match ChannelType::from_name(&channel) {
        Some(ct) => ct,
        None => return ipc::err("IM_UNKNOWN_CHANNEL", &format!("Unknown channel: {}", channel)),
    };
    let mut channels = match load_channels() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", &e),
    };

    let bot_id = format!(
        "bot-{}-{}",
        channel_type,
        &uuid::Uuid::new_v4().to_string()[..8]
    );

    let bot = BotConfig {
        id: bot_id,
        channel: channel_type.clone(),
        name,
        credential_type,
        workspace,
        model,
        enabled: true,
        created_at: chrono::Utc::now().timestamp() as u64,
    };

    let mut found = false;
    let mut result = bot.clone();

    for ch in &mut channels {
        if ch.channel == channel_type {
            ch.bots.push(bot);
            result = match ch.bots.last() {
                Some(b) => b.clone(),
                None => return ipc::err("IM_INTERNAL", "empty bots after push"),
            };
            found = true;
            break;
        }
    }

    if found {
        if let Err(e) = save_channels(&channels) {
            return ipc::err("IM_SAVE_FAILED", format!("{e}"));
        }
        ipc::ok(result)
    } else {
        ipc::err("IM_CHANNEL_NOT_FOUND", &format!("Channel not found: {}", channel))
    }
}

/// 删除机器人
#[tauri::command]
pub async fn im_remove_bot(channel: String, bot_id: String) -> IpcResponse<bool> {
    let channel_type = match ChannelType::from_name(&channel) {
        Some(ct) => ct,
        None => return ipc::err("IM_UNKNOWN_CHANNEL", &format!("Unknown channel: {}", channel)),
    };
    let mut channels = match load_channels() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", &e),
    };

    for ch in &mut channels {
        if ch.channel == channel_type {
            let original_len = ch.bots.len();
            ch.bots.retain(|b| b.id != bot_id);
            if ch.bots.len() < original_len {
                if let Err(e) = save_channels(&channels) {
                    return ipc::err("IM_SAVE_FAILED", format!("{e}"));
                }
                return ipc::ok(true);
            }
            return ipc::ok(false);
        }
    }

    ipc::ok(false)
}

/// 更新机器人配置
#[tauri::command]
pub async fn im_update_bot(
    channel: String,
    bot_id: String,
    name: Option<String>,
    workspace: Option<String>,
    model: Option<String>,
) -> IpcResponse<BotConfig> {
    let channel_type = match ChannelType::from_name(&channel) {
        Some(ct) => ct,
        None => return ipc::err("IM_UNKNOWN_CHANNEL", &format!("Unknown channel: {}", channel)),
    };
    let mut channels = match load_channels() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", &e),
    };

    let mut found = false;
    let mut result = BotConfig {
        id: bot_id.clone(),
        channel: channel_type.clone(),
        name: String::new(),
        credential_type: String::new(),
        workspace: None,
        model: None,
        enabled: false,
        created_at: 0,
    };

    for ch in &mut channels {
        if ch.channel == channel_type {
            for bot in &mut ch.bots {
                if bot.id == bot_id {
                    if let Some(ref n) = name {
                        bot.name = n.clone();
                    }
                    if let Some(ref w) = workspace {
                        bot.workspace = Some(w.clone());
                    }
                    if let Some(ref m) = model {
                        bot.model = Some(m.clone());
                    }
                    result = bot.clone();
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
    }

    if found {
        if let Err(e) = save_channels(&channels) {
            return ipc::err("IM_SAVE_FAILED", format!("{e}"));
        }
        ipc::ok(result)
    } else {
        ipc::err("IM_BOT_NOT_FOUND", &format!("Bot not found: {} / {}", channel, bot_id))
    }
}

/// 设置上下文增强
#[tauri::command]
pub async fn im_set_context_enhancement(
    channel: String,
    enabled: bool,
) -> IpcResponse<ChannelConfig> {
    let channel_type = match ChannelType::from_name(&channel) {
        Some(ct) => ct,
        None => return ipc::err("IM_UNKNOWN_CHANNEL", &format!("Unknown channel: {}", channel)),
    };
    let mut channels = match load_channels() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", &e),
    };

    let mut found = false;
    let mut result = ChannelConfig {
        channel: channel_type.clone(),
        enabled: false,
        bots: vec![],
        context_enhancement: false,
        proactive_delivery: false,
    };

    for ch in &mut channels {
        if ch.channel == channel_type {
            ch.context_enhancement = enabled;
            result = ch.clone();
            found = true;
            break;
        }
    }

    if found {
        if let Err(e) = save_channels(&channels) {
            return ipc::err("IM_SAVE_FAILED", format!("{e}"));
        }
        ipc::ok(result)
    } else {
        ipc::err("IM_CHANNEL_NOT_FOUND", &format!("Channel not found: {}", channel))
    }
}

/// 设置主动投递
#[tauri::command]
pub async fn im_set_proactive_delivery(
    channel: String,
    enabled: bool,
) -> IpcResponse<ChannelConfig> {
    let channel_type = match ChannelType::from_name(&channel) {
        Some(ct) => ct,
        None => return ipc::err("IM_UNKNOWN_CHANNEL", &format!("Unknown channel: {}", channel)),
    };
    let mut channels = match load_channels() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", &e),
    };

    let mut found = false;
    let mut result = ChannelConfig {
        channel: channel_type.clone(),
        enabled: false,
        bots: vec![],
        context_enhancement: false,
        proactive_delivery: false,
    };

    for ch in &mut channels {
        if ch.channel == channel_type {
            ch.proactive_delivery = enabled;
            result = ch.clone();
            found = true;
            break;
        }
    }

    if found {
        if let Err(e) = save_channels(&channels) {
            return ipc::err("IM_SAVE_FAILED", format!("{e}"));
        }
        ipc::ok(result)
    } else {
        ipc::err("IM_CHANNEL_NOT_FOUND", &format!("Channel not found: {}", channel))
    }
}

// ═══════════════════════════════════════════════
// DSH 市场模式 Commands
// ═══════════════════════════════════════════════

/// 获取 DSH 市场配置
#[tauri::command]
pub async fn im_dsh_market_status() -> IpcResponse<DshMarketConfig> {
    match load_dsh_market() {
        Ok(config) => ipc::ok(config),
        Err(e) => ipc::err("IM_LOAD_FAILED", format!("{e}")),
    }
}

/// 启用/禁用 DSH 市场
#[tauri::command]
pub async fn im_dsh_market_toggle(enabled: bool) -> IpcResponse<DshMarketConfig> {
    let mut config = match load_dsh_market() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", &e),
    };
    config.enabled = enabled;
    if let Err(e) = save_dsh_market(&config) {
        return ipc::err("IM_SAVE_FAILED", format!("{e}"));
    }
    ipc::ok(config)
}

/// 更新 DSH 市场配置
#[tauri::command]
pub async fn im_dsh_market_config(
    api_endpoint: Option<String>,
    auth_token: Option<String>,
    sync_enabled: Option<bool>,
) -> IpcResponse<DshMarketConfig> {
    let mut config = match load_dsh_market() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", &e),
    };
    if let Some(ep) = api_endpoint {
        config.api_endpoint = ep;
    }
    if let Some(token) = auth_token {
        config.auth_token = Some(token);
    }
    if let Some(sync) = sync_enabled {
        config.sync_enabled = sync;
    }
    if let Err(e) = save_dsh_market(&config) {
        return ipc::err("IM_SAVE_FAILED", format!("{e}"));
    }
    ipc::ok(config)
}

/// 从 DSH 市场同步插件
#[tauri::command]
pub async fn im_dsh_market_sync() -> IpcResponse<HashMap<String, String>> {
    let config = match load_dsh_market() {
        Ok(c) => c,
        Err(e) => return ipc::err("IM_LOAD_FAILED", &e),
    };
    if !config.enabled {
        return ipc::err("IM_DSH_DISABLED", "DSH market is not enabled");
    }

    // TODO: 实现实际的 DSH 市场 API 调用
    // 目前返回模拟数据
    let mut plugins = HashMap::new();
    plugins.insert("dsh-im-core".into(), "1.0.0".into());
    plugins.insert("dsh-im-wechat".into(), "1.0.0".into());
    plugins.insert("dsh-im-feishu".into(), "1.0.0".into());
    plugins.insert("dsh-im-telegram".into(), "1.0.0".into());

    ipc::ok(plugins)
}
