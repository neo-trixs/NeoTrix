//! IM Channel — 对接 DSH-IM 多渠道适配器模式
//!
//! 支持 9 个内置渠道：微信、飞书、钉钉、企业微信、QQ、Slack、Telegram、Discord、WhatsApp
//! 基于 DSH-IM 的 adapter 注册表模式，每个渠道独立配置和状态。

use crate::config::AppConfig;
use crate::domain::im_service;
use crate::domain::plugins::im::{
    BotConfig, ChannelConfig, ChannelType, DshMarketConfig, ImStatus,
};
use crate::ipc::{self, IpcResponse};
use std::collections::HashMap;

fn base_dir() -> std::path::PathBuf {
    AppConfig::base_dir().unwrap_or_default()
}

// ═══════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════

/// 获取 IM 系统状态
#[tauri::command]
pub async fn im_status() -> IpcResponse<ImStatus> {
    match im_service::get_status(&base_dir()) {
        Ok(status) => ipc::ok(status),
        Err(e) => ipc::err("IM_LOAD_FAILED", format!("{e}")),
    }
}

/// 获取所有渠道配置
#[tauri::command]
pub async fn im_list_channels() -> IpcResponse<Vec<ChannelConfig>> {
    match im_service::load_channels(&base_dir()) {
        Ok(channels) => ipc::ok(channels),
        Err(e) => ipc::err("IM_LOAD_FAILED", format!("{e}")),
    }
}

/// 获取单个渠道配置
#[tauri::command]
pub async fn im_get_channel(channel: String) -> IpcResponse<ChannelConfig> {
    let channel_type = match ChannelType::from_name(&channel) {
        Some(ct) => ct,
        None => {
            return ipc::err(
                "IM_UNKNOWN_CHANNEL",
                &format!("Unknown channel: {}", channel),
            )
        }
    };
    match im_service::load_channels(&base_dir()) {
        Ok(channels) => match channels.into_iter().find(|c| c.channel == channel_type) {
            Some(ch) => ipc::ok(ch),
            None => ipc::err(
                "IM_CHANNEL_NOT_FOUND",
                &format!("Channel not found: {}", channel),
            ),
        },
        Err(e) => ipc::err("IM_LOAD_FAILED", e.to_string()),
    }
}

/// 启用/禁用渠道
#[tauri::command]
pub async fn im_toggle_channel(channel: String, enabled: bool) -> IpcResponse<ChannelConfig> {
    match im_service::toggle_channel(&base_dir(), &channel, enabled) {
        Ok(ch) => ipc::ok(ch),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Unknown channel") {
                ipc::err("IM_UNKNOWN_CHANNEL", &msg)
            } else if msg.contains("Channel not found") {
                ipc::err("IM_CHANNEL_NOT_FOUND", &msg)
            } else {
                ipc::err("IM_SAVE_FAILED", &msg)
            }
        }
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
    match im_service::add_bot(
        &base_dir(),
        &channel,
        &name,
        &credential_type,
        workspace.as_deref().unwrap_or(""),
        model.as_deref().unwrap_or(""),
    ) {
        Ok(bot) => ipc::ok(bot),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Unknown channel") {
                ipc::err("IM_UNKNOWN_CHANNEL", &msg)
            } else {
                ipc::err("IM_SAVE_FAILED", &msg)
            }
        }
    }
}

/// 删除机器人
#[tauri::command]
pub async fn im_remove_bot(channel: String, bot_id: String) -> IpcResponse<bool> {
    match im_service::remove_bot(&base_dir(), &channel, &bot_id) {
        Ok(removed) => ipc::ok(removed),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Unknown channel") {
                ipc::err("IM_UNKNOWN_CHANNEL", &msg)
            } else {
                ipc::err("IM_SAVE_FAILED", &msg)
            }
        }
    }
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
    match im_service::update_bot(
        &base_dir(),
        &channel,
        &bot_id,
        name.as_deref(),
        workspace.as_deref(),
        model.as_deref(),
    ) {
        Ok(bot) => ipc::ok(bot),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Unknown channel") {
                ipc::err("IM_UNKNOWN_CHANNEL", &msg)
            } else if msg.contains("Bot not found") {
                ipc::err("IM_BOT_NOT_FOUND", &msg)
            } else {
                ipc::err("IM_SAVE_FAILED", &msg)
            }
        }
    }
}

/// 设置上下文增强
#[tauri::command]
pub async fn im_set_context_enhancement(
    channel: String,
    enabled: bool,
) -> IpcResponse<ChannelConfig> {
    match im_service::set_context_enhancement(&base_dir(), &channel, enabled) {
        Ok(ch) => ipc::ok(ch),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Unknown channel") {
                ipc::err("IM_UNKNOWN_CHANNEL", &msg)
            } else if msg.contains("Channel not found") {
                ipc::err("IM_CHANNEL_NOT_FOUND", &msg)
            } else {
                ipc::err("IM_SAVE_FAILED", &msg)
            }
        }
    }
}

/// 设置主动投递
#[tauri::command]
pub async fn im_set_proactive_delivery(
    channel: String,
    enabled: bool,
) -> IpcResponse<ChannelConfig> {
    match im_service::set_proactive_delivery(&base_dir(), &channel, enabled) {
        Ok(ch) => ipc::ok(ch),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Unknown channel") {
                ipc::err("IM_UNKNOWN_CHANNEL", &msg)
            } else if msg.contains("Channel not found") {
                ipc::err("IM_CHANNEL_NOT_FOUND", &msg)
            } else {
                ipc::err("IM_SAVE_FAILED", &msg)
            }
        }
    }
}

// ═══════════════════════════════════════════════
// DSH 市场模式 Commands
// ═══════════════════════════════════════════════

/// 获取 DSH 市场配置
#[tauri::command]
pub async fn im_dsh_market_status() -> IpcResponse<DshMarketConfig> {
    match im_service::load_dsh_market(&base_dir()) {
        Ok(config) => ipc::ok(config),
        Err(e) => ipc::err("IM_LOAD_FAILED", format!("{e}")),
    }
}

/// 启用/禁用 DSH 市场
#[tauri::command]
pub async fn im_dsh_market_toggle(enabled: bool) -> IpcResponse<DshMarketConfig> {
    match im_service::dsh_market_toggle(&base_dir(), enabled) {
        Ok(config) => ipc::ok(config),
        Err(e) => ipc::err("IM_SAVE_FAILED", format!("{e}")),
    }
}

/// 更新 DSH 市场配置
#[tauri::command]
pub async fn im_dsh_market_config(
    api_endpoint: Option<String>,
    auth_token: Option<String>,
    sync_enabled: Option<bool>,
) -> IpcResponse<DshMarketConfig> {
    match im_service::dsh_market_config_update(
        &base_dir(),
        api_endpoint.as_deref(),
        auth_token.as_deref(),
        sync_enabled,
    ) {
        Ok(config) => ipc::ok(config),
        Err(e) => ipc::err("IM_SAVE_FAILED", format!("{e}")),
    }
}

/// 从 DSH 市场同步插件
#[tauri::command]
pub async fn im_dsh_market_sync() -> IpcResponse<HashMap<String, String>> {
    match im_service::dsh_market_sync(&base_dir()) {
        Ok(plugins) => ipc::ok(plugins),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not enabled") {
                ipc::err("IM_DSH_DISABLED", &msg)
            } else {
                ipc::err("IM_LOAD_FAILED", &msg)
            }
        }
    }
}
