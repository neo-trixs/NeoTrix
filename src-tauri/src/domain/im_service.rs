//! IM Domain Service
//!
//! 核心 IM 配置和状态管理逻辑，供 commands/im.rs 和 domain/plugins/im.rs 共用。

use anyhow::{Context, Result as AnyhowResult};
use crate::atomic_io;
use crate::config::AppConfig;
use crate::domain::plugins::im::{
    BotConfig, ChannelConfig, ChannelType, DshMarketConfig, ImStatus, ALL_CHANNEL_TYPES,
};
use crate::domain::serde_json;
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════
// 路径
// ═══════════════════════════════════════════════

pub fn config_path() -> PathBuf {
    AppConfig::base_dir()
        .unwrap_or_default()
        .join("im_channels.json")
}

pub fn dsh_market_path() -> PathBuf {
    AppConfig::base_dir()
        .unwrap_or_default()
        .join("dsh_market.json")
}

// ═══════════════════════════════════════════════
// 渠道配置 I/O
// ═══════════════════════════════════════════════

pub fn load_channels(base_dir: &Path) -> AnyhowResult<Vec<ChannelConfig>> {
    let path = base_dir.join("im_channels.json");
    if !path.exists() {
        return Ok(default_channels());
    }
    let content = String::from_utf8(atomic_io::read_with_fallback(&path).context("Read IM config")?)?;
    serde_json::from_str(&content).context("Parse IM config")
}

pub fn save_channels(base_dir: &Path, channels: &[ChannelConfig]) -> AnyhowResult<()> {
    let path = base_dir.join("im_channels.json");
    atomic_io::ensure_parent_dir(&path).context("Create dir")?;
    atomic_io::write_json_atomic(&path, channels).context("Write config")
}

// ═══════════════════════════════════════════════
// DSH 市场 I/O
// ═══════════════════════════════════════════════

pub fn load_dsh_market(base_dir: &Path) -> AnyhowResult<DshMarketConfig> {
    let path = base_dir.join("dsh_market.json");
    if !path.exists() {
        return Ok(DshMarketConfig::default());
    }
    let content = String::from_utf8(
        atomic_io::read_with_fallback(&path).context("Read DSH market config")?,
    )?;
    serde_json::from_str(&content).context("Parse DSH market config")
}

pub fn save_dsh_market(base_dir: &Path, config: &DshMarketConfig) -> AnyhowResult<()> {
    let path = base_dir.join("dsh_market.json");
    atomic_io::ensure_parent_dir(&path).context("Create dir")?;
    atomic_io::write_json_atomic(&path, config).context("Write config")
}

// ═══════════════════════════════════════════════
// 默认值
// ═══════════════════════════════════════════════

pub fn default_channels() -> Vec<ChannelConfig> {
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
// 状态查询
// ═══════════════════════════════════════════════

pub fn get_status(base_dir: &Path) -> AnyhowResult<ImStatus> {
    let channels = load_channels(base_dir)?;
    let dsh_market = load_dsh_market(base_dir)?;

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

// ═══════════════════════════════════════════════
// 渠道操作
// ═══════════════════════════════════════════════

pub fn toggle_channel(
    base_dir: &Path,
    channel: &str,
    enabled: bool,
) -> AnyhowResult<ChannelConfig> {
    let channel_type = ChannelType::from_name(channel)
        .ok_or_else(|| anyhow::anyhow!("Unknown channel: {}", channel))?;
    let mut channels = load_channels(base_dir)?;

    for ch in &mut channels {
        if ch.channel == channel_type {
            ch.enabled = enabled;
            let result = ch.clone();
            save_channels(base_dir, &channels)?;
            return Ok(result);
        }
    }

    Err(anyhow::anyhow!("Channel not found: {}", channel))
}

// ═══════════════════════════════════════════════
// 机器人操作
// ═══════════════════════════════════════════════

pub fn add_bot(
    base_dir: &Path,
    channel: &str,
    name: &str,
    cred_type: &str,
    ws_url: &str,
    model: &str,
) -> AnyhowResult<BotConfig> {
    let channel_type = ChannelType::from_name(channel)
        .ok_or_else(|| anyhow::anyhow!("Unknown channel: {}", channel))?;
    let mut channels = load_channels(base_dir)?;

    let bot_id = format!(
        "bot-{}-{}",
        channel_type,
        &uuid::Uuid::new_v4().to_string()[..8]
    );

    let bot = BotConfig {
        id: bot_id,
        channel: channel_type.clone(),
        name: name.to_string(),
        credential_type: cred_type.to_string(),
        workspace: Some(ws_url.to_string()),
        model: Some(model.to_string()),
        enabled: true,
        created_at: chrono::Utc::now().timestamp() as u64,
        response_mode: Default::default(),
        whitelist: Default::default(),
    };

    for ch in &mut channels {
        if ch.channel == channel_type {
            ch.bots.push(bot.clone());
            save_channels(base_dir, &channels)?;
            return Ok(bot);
        }
    }

    Err(anyhow::anyhow!("Channel not found: {}", channel))
}

pub fn remove_bot(base_dir: &Path, channel: &str, bot_id: &str) -> AnyhowResult<bool> {
    let channel_type = ChannelType::from_name(channel)
        .ok_or_else(|| anyhow::anyhow!("Unknown channel: {}", channel))?;
    let mut channels = load_channels(base_dir)?;

    for ch in &mut channels {
        if ch.channel == channel_type {
            let original_len = ch.bots.len();
            ch.bots.retain(|b| b.id != bot_id);
            if ch.bots.len() < original_len {
                save_channels(base_dir, &channels)?;
                return Ok(true);
            }
            return Ok(false);
        }
    }

    Ok(false)
}

pub fn update_bot(
    base_dir: &Path,
    channel: &str,
    bot_id: &str,
    name: Option<&str>,
    ws_url: Option<&str>,
    model: Option<&str>,
) -> AnyhowResult<BotConfig> {
    let channel_type = ChannelType::from_name(channel)
        .ok_or_else(|| anyhow::anyhow!("Unknown channel: {}", channel))?;
    let mut channels = load_channels(base_dir)?;

    for ch in &mut channels {
        if ch.channel == channel_type {
            for bot in &mut ch.bots {
                if bot.id == bot_id {
                    if let Some(n) = name {
                        bot.name = n.to_string();
                    }
                    if let Some(w) = ws_url {
                        bot.workspace = Some(w.to_string());
                    }
                    if let Some(m) = model {
                        bot.model = Some(m.to_string());
                    }
                    let result = bot.clone();
                    save_channels(base_dir, &channels)?;
                    return Ok(result);
                }
            }
        }
    }

    Err(anyhow::anyhow!("Bot not found: {} / {}", channel, bot_id))
}

// ═══════════════════════════════════════════════
// 功能开关
// ═══════════════════════════════════════════════

pub fn set_context_enhancement(
    base_dir: &Path,
    channel: &str,
    enabled: bool,
) -> AnyhowResult<ChannelConfig> {
    let channel_type = ChannelType::from_name(channel)
        .ok_or_else(|| anyhow::anyhow!("Unknown channel: {}", channel))?;
    let mut channels = load_channels(base_dir)?;

    for ch in &mut channels {
        if ch.channel == channel_type {
            ch.context_enhancement = enabled;
            let result = ch.clone();
            save_channels(base_dir, &channels)?;
            return Ok(result);
        }
    }

    Err(anyhow::anyhow!("Channel not found: {}", channel))
}

pub fn set_proactive_delivery(
    base_dir: &Path,
    channel: &str,
    enabled: bool,
) -> AnyhowResult<ChannelConfig> {
    let channel_type = ChannelType::from_name(channel)
        .ok_or_else(|| anyhow::anyhow!("Unknown channel: {}", channel))?;
    let mut channels = load_channels(base_dir)?;

    for ch in &mut channels {
        if ch.channel == channel_type {
            ch.proactive_delivery = enabled;
            let result = ch.clone();
            save_channels(base_dir, &channels)?;
            return Ok(result);
        }
    }

    Err(anyhow::anyhow!("Channel not found: {}", channel))
}

// ═══════════════════════════════════════════════
// DSH 市场操作
// ═══════════════════════════════════════════════

pub fn dsh_market_toggle(base_dir: &Path, enabled: bool) -> AnyhowResult<DshMarketConfig> {
    let mut config = load_dsh_market(base_dir)?;
    config.enabled = enabled;
    save_dsh_market(base_dir, &config)?;
    Ok(config)
}

pub fn dsh_market_config_update(
    base_dir: &Path,
    api_endpoint: Option<&str>,
    auth_token: Option<&str>,
    sync_enabled: Option<bool>,
) -> AnyhowResult<DshMarketConfig> {
    let mut config = load_dsh_market(base_dir)?;
    if let Some(ep) = api_endpoint {
        config.api_endpoint = ep.to_string();
    }
    if let Some(token) = auth_token {
        config.auth_token = Some(token.to_string());
    }
    if let Some(sync) = sync_enabled {
        config.sync_enabled = sync;
    }
    save_dsh_market(base_dir, &config)?;
    Ok(config)
}

pub fn dsh_market_sync(
    base_dir: &Path,
) -> AnyhowResult<std::collections::HashMap<String, String>> {
    let config = load_dsh_market(base_dir)?;
    if !config.enabled {
        return Err(anyhow::anyhow!("DSH market is not enabled"));
    }

    // TODO: 实际 DSH 市场 API 调用
    let mut plugins = std::collections::HashMap::new();
    plugins.insert("dsh-im-core".into(), "1.0.0".into());
    plugins.insert("dsh-im-wechat".into(), "1.0.0".into());
    plugins.insert("dsh-im-feishu".into(), "1.0.0".into());
    plugins.insert("dsh-im-telegram".into(), "1.0.0".into());

    Ok(plugins)
}
