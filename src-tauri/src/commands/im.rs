//! IM Channel — 对接 DSH-IM 多渠道适配器模式
//!
//! 支持 9 个内置渠道：微信、飞书、钉钉、企业微信、QQ、Slack、Telegram、Discord、WhatsApp
//! 基于 DSH-IM 的 adapter 注册表模式，每个渠道独立配置和状态。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// IM 渠道类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChannelType {
    WeChat,
    Feishu,
    DingTalk,
    WeCom,
    QQ,
    Slack,
    Telegram,
    Discord,
    WhatsApp,
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WeChat => write!(f, "wechat"),
            Self::Feishu => write!(f, "feishu"),
            Self::DingTalk => write!(f, "dingtalk"),
            Self::WeCom => write!(f, "wecom"),
            Self::QQ => write!(f, "qq"),
            Self::Slack => write!(f, "slack"),
            Self::Telegram => write!(f, "telegram"),
            Self::Discord => write!(f, "discord"),
            Self::WhatsApp => write!(f, "whatsapp"),
        }
    }
}

impl ChannelType {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "wechat" | "微信" => Some(Self::WeChat),
            "feishu" | "飞书" => Some(Self::Feishu),
            "dingtalk" | "钉钉" => Some(Self::DingTalk),
            "wecom" | "企业微信" => Some(Self::WeCom),
            "qq" => Some(Self::QQ),
            "slack" => Some(Self::Slack),
            "telegram" => Some(Self::Telegram),
            "discord" => Some(Self::Discord),
            "whatsapp" => Some(Self::WhatsApp),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::WeChat => "微信",
            Self::Feishu => "飞书",
            Self::DingTalk => "钉钉",
            Self::WeCom => "企业微信",
            Self::QQ => "QQ",
            Self::Slack => "Slack",
            Self::Telegram => "Telegram",
            Self::Discord => "Discord",
            Self::WhatsApp => "WhatsApp",
        }
    }
}

/// 渠道连接状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChannelStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// 机器人配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub id: String,
    pub channel: ChannelType,
    pub name: String,
    pub credential_type: String,
    pub workspace: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
    pub created_at: u64,
}

/// 渠道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub channel: ChannelType,
    pub enabled: bool,
    pub bots: Vec<BotConfig>,
    pub context_enhancement: bool,
    pub proactive_delivery: bool,
}

/// IM 系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImStatus {
    pub channels: Vec<ChannelConfig>,
    pub total_bots: usize,
    pub connected_bots: usize,
    pub dsh_market_enabled: bool,
}

/// DSH 市场配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DshMarketConfig {
    pub enabled: bool,
    pub api_endpoint: String,
    pub auth_token: Option<String>,
    pub sync_enabled: bool,
    pub last_sync: Option<u64>,
}

impl Default for DshMarketConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_endpoint: "https://dshfind.com/api".into(),
            auth_token: None,
            sync_enabled: true,
            last_sync: None,
        }
    }
}

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
fn load_channels() -> Result<Vec<ChannelConfig>, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(default_channels());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("Read IM config: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse IM config: {e}"))
}

/// 保存渠道配置
fn save_channels(channels: &[ChannelConfig]) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Create dir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(channels).map_err(|e| format!("Serialize: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Write config: {e}"))
}

/// 加载 DSH 市场配置
fn load_dsh_market() -> Result<DshMarketConfig, String> {
    let path = dsh_market_path();
    if !path.exists() {
        return Ok(DshMarketConfig::default());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Read DSH market config: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse DSH market config: {e}"))
}

/// 保存 DSH 市场配置
fn save_dsh_market(config: &DshMarketConfig) -> Result<(), String> {
    let path = dsh_market_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Create dir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| format!("Serialize: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Write config: {e}"))
}

/// 默认渠道配置
fn default_channels() -> Vec<ChannelConfig> {
    vec![
        ChannelConfig {
            channel: ChannelType::WeChat,
            enabled: false,
            bots: vec![],
            context_enhancement: false,
            proactive_delivery: false,
        },
        ChannelConfig {
            channel: ChannelType::Feishu,
            enabled: false,
            bots: vec![],
            context_enhancement: false,
            proactive_delivery: false,
        },
        ChannelConfig {
            channel: ChannelType::DingTalk,
            enabled: false,
            bots: vec![],
            context_enhancement: false,
            proactive_delivery: false,
        },
        ChannelConfig {
            channel: ChannelType::WeCom,
            enabled: false,
            bots: vec![],
            context_enhancement: false,
            proactive_delivery: false,
        },
        ChannelConfig {
            channel: ChannelType::QQ,
            enabled: false,
            bots: vec![],
            context_enhancement: false,
            proactive_delivery: false,
        },
        ChannelConfig {
            channel: ChannelType::Slack,
            enabled: false,
            bots: vec![],
            context_enhancement: false,
            proactive_delivery: false,
        },
        ChannelConfig {
            channel: ChannelType::Telegram,
            enabled: false,
            bots: vec![],
            context_enhancement: false,
            proactive_delivery: false,
        },
        ChannelConfig {
            channel: ChannelType::Discord,
            enabled: false,
            bots: vec![],
            context_enhancement: false,
            proactive_delivery: false,
        },
        ChannelConfig {
            channel: ChannelType::WhatsApp,
            enabled: false,
            bots: vec![],
            context_enhancement: false,
            proactive_delivery: false,
        },
    ]
}

// ═══════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════

/// 获取 IM 系统状态
#[tauri::command]
pub async fn im_status() -> Result<ImStatus, String> {
    let channels = load_channels()?;
    let dsh_market = load_dsh_market()?;

    let total_bots: usize = channels.iter().map(|c| c.bots.len()).sum();
    let connected_bots = channels
        .iter()
        .filter(|c| c.enabled)
        .map(|c| c.bots.len())
        .sum();

    Ok(ImStatus {
        channels,
        total_bots,
        connected_bots,
        dsh_market_enabled: dsh_market.enabled,
    })
}

/// 获取所有渠道配置
#[tauri::command]
pub async fn im_list_channels() -> Result<Vec<ChannelConfig>, String> {
    load_channels()
}

/// 获取单个渠道配置
#[tauri::command]
pub async fn im_get_channel(channel: String) -> Result<ChannelConfig, String> {
    let channel_type =
        ChannelType::from_name(&channel).ok_or_else(|| format!("Unknown channel: {}", channel))?;
    let channels = load_channels()?;
    channels
        .into_iter()
        .find(|c| c.channel == channel_type)
        .ok_or_else(|| format!("Channel not found: {}", channel))
}

/// 启用/禁用渠道
#[tauri::command]
pub async fn im_toggle_channel(channel: String, enabled: bool) -> Result<ChannelConfig, String> {
    let channel_type =
        ChannelType::from_name(&channel).ok_or_else(|| format!("Unknown channel: {}", channel))?;
    let mut channels = load_channels()?;

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
        save_channels(&channels)?;
        Ok(result)
    } else {
        Err(format!("Channel not found: {}", channel))
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
) -> Result<BotConfig, String> {
    let channel_type =
        ChannelType::from_name(&channel).ok_or_else(|| format!("Unknown channel: {}", channel))?;
    let mut channels = load_channels()?;

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
            result = ch.bots.last().unwrap().clone();
            found = true;
            break;
        }
    }

    if found {
        save_channels(&channels)?;
        Ok(result)
    } else {
        Err(format!("Channel not found: {}", channel))
    }
}

/// 删除机器人
#[tauri::command]
pub async fn im_remove_bot(channel: String, bot_id: String) -> Result<bool, String> {
    let channel_type =
        ChannelType::from_name(&channel).ok_or_else(|| format!("Unknown channel: {}", channel))?;
    let mut channels = load_channels()?;

    for ch in &mut channels {
        if ch.channel == channel_type {
            let original_len = ch.bots.len();
            ch.bots.retain(|b| b.id != bot_id);
            if ch.bots.len() < original_len {
                save_channels(&channels)?;
                return Ok(true);
            }
            return Ok(false);
        }
    }

    Ok(false)
}

/// 更新机器人配置
#[tauri::command]
pub async fn im_update_bot(
    channel: String,
    bot_id: String,
    name: Option<String>,
    workspace: Option<String>,
    model: Option<String>,
) -> Result<BotConfig, String> {
    let channel_type =
        ChannelType::from_name(&channel).ok_or_else(|| format!("Unknown channel: {}", channel))?;
    let mut channels = load_channels()?;

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
        save_channels(&channels)?;
        Ok(result)
    } else {
        Err(format!("Bot not found: {} / {}", channel, bot_id))
    }
}

/// 设置上下文增强
#[tauri::command]
pub async fn im_set_context_enhancement(
    channel: String,
    enabled: bool,
) -> Result<ChannelConfig, String> {
    let channel_type =
        ChannelType::from_name(&channel).ok_or_else(|| format!("Unknown channel: {}", channel))?;
    let mut channels = load_channels()?;

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
        save_channels(&channels)?;
        Ok(result)
    } else {
        Err(format!("Channel not found: {}", channel))
    }
}

/// 设置主动投递
#[tauri::command]
pub async fn im_set_proactive_delivery(
    channel: String,
    enabled: bool,
) -> Result<ChannelConfig, String> {
    let channel_type =
        ChannelType::from_name(&channel).ok_or_else(|| format!("Unknown channel: {}", channel))?;
    let mut channels = load_channels()?;

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
        save_channels(&channels)?;
        Ok(result)
    } else {
        Err(format!("Channel not found: {}", channel))
    }
}

// ═══════════════════════════════════════════════
// DSH 市场模式 Commands
// ═══════════════════════════════════════════════

/// 获取 DSH 市场配置
#[tauri::command]
pub async fn im_dsh_market_status() -> Result<DshMarketConfig, String> {
    load_dsh_market()
}

/// 启用/禁用 DSH 市场
#[tauri::command]
pub async fn im_dsh_market_toggle(enabled: bool) -> Result<DshMarketConfig, String> {
    let mut config = load_dsh_market()?;
    config.enabled = enabled;
    save_dsh_market(&config)?;
    Ok(config)
}

/// 更新 DSH 市场配置
#[tauri::command]
pub async fn im_dsh_market_config(
    api_endpoint: Option<String>,
    auth_token: Option<String>,
    sync_enabled: Option<bool>,
) -> Result<DshMarketConfig, String> {
    let mut config = load_dsh_market()?;
    if let Some(ep) = api_endpoint {
        config.api_endpoint = ep;
    }
    if let Some(token) = auth_token {
        config.auth_token = Some(token);
    }
    if let Some(sync) = sync_enabled {
        config.sync_enabled = sync;
    }
    save_dsh_market(&config)?;
    Ok(config)
}

/// 从 DSH 市场同步插件
#[tauri::command]
pub async fn im_dsh_market_sync() -> Result<HashMap<String, String>, String> {
    let config = load_dsh_market()?;
    if !config.enabled {
        return Err("DSH market is not enabled".into());
    }

    // TODO: 实现实际的 DSH 市场 API 调用
    // 目前返回模拟数据
    let mut plugins = HashMap::new();
    plugins.insert("dsh-im-core".into(), "1.0.0".into());
    plugins.insert("dsh-im-wechat".into(), "1.0.0".into());
    plugins.insert("dsh-im-feishu".into(), "1.0.0".into());
    plugins.insert("dsh-im-telegram".into(), "1.0.0".into());

    Ok(plugins)
}
