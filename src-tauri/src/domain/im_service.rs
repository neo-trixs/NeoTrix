//! IM Domain Service
//!
//! 核心 IM 配置和状态管理逻辑，供 commands/im.rs 和 domain/plugins/im.rs 共用。

use crate::atomic_io;
use crate::config::AppConfig;
use crate::domain::plugins::im::{
    BotConfig, ChannelConfig, ChannelType, DshMarketConfig, ImStatus, ALL_CHANNEL_TYPES,
};
use crate::domain::serde_json;
use anyhow::{Context, Result as AnyhowResult};
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
    let content =
        String::from_utf8(atomic_io::read_with_fallback(&path).context("Read IM config")?)?;
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
    let content =
        String::from_utf8(atomic_io::read_with_fallback(&path).context("Read DSH market config")?)?;
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

pub fn dsh_market_sync(base_dir: &Path) -> AnyhowResult<std::collections::HashMap<String, String>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ── default_channels ─────────────────────────────

    #[test]
    fn default_channels_count_matches_all_types() {
        let channels = default_channels();
        assert_eq!(channels.len(), ALL_CHANNEL_TYPES.len());
    }

    #[test]
    fn default_channels_all_disabled() {
        let channels = default_channels();
        for ch in &channels {
            assert!(!ch.enabled, "Channel {} should be disabled", ch.channel);
        }
    }

    #[test]
    fn default_channels_no_bots() {
        let channels = default_channels();
        for ch in &channels {
            assert!(ch.bots.is_empty());
        }
    }

    #[test]
    fn default_channels_no_enhancements() {
        let channels = default_channels();
        for ch in &channels {
            assert!(!ch.context_enhancement);
            assert!(!ch.proactive_delivery);
        }
    }

    #[test]
    fn default_channels_contains_each_type() {
        let channels = default_channels();
        for ch_type in ALL_CHANNEL_TYPES {
            assert!(
                channels.iter().any(|c| c.channel == *ch_type),
                "Missing channel type: {ch_type}"
            );
        }
    }

    // ── load_channels ────────────────────────────────

    #[test]
    fn load_channels_missing_file_returns_defaults() {
        let dir = tempdir().unwrap();
        let channels = load_channels(dir.path()).unwrap();
        assert_eq!(channels.len(), ALL_CHANNEL_TYPES.len());
    }

    #[test]
    fn load_channels_valid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("im_channels.json");
        let channels = default_channels();
        atomic_io::write_json_atomic(&path, &channels).unwrap();
        let loaded = load_channels(dir.path()).unwrap();
        assert_eq!(loaded.len(), channels.len());
    }

    // ── save_channels ────────────────────────────────

    #[test]
    fn save_and_load_channels_roundtrip() {
        let dir = tempdir().unwrap();
        let mut channels = default_channels();
        channels[0].enabled = true;
        save_channels(dir.path(), &channels).unwrap();
        let loaded = load_channels(dir.path()).unwrap();
        assert!(loaded[0].enabled);
    }

    // ── toggle_channel ───────────────────────────────

    #[test]
    fn toggle_channel_enable() {
        let dir = tempdir().unwrap();
        let ch = toggle_channel(dir.path(), "wechat", true).unwrap();
        assert!(ch.enabled);
    }

    #[test]
    fn toggle_channel_disable() {
        let dir = tempdir().unwrap();
        toggle_channel(dir.path(), "wechat", true).unwrap();
        let ch = toggle_channel(dir.path(), "wechat", false).unwrap();
        assert!(!ch.enabled);
    }

    #[test]
    fn toggle_channel_nonexistent_returns_error() {
        let dir = tempdir().unwrap();
        let result = toggle_channel(dir.path(), "bogus_channel", true);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unknown channel"));
    }

    // ── add_bot ──────────────────────────────────────

    #[test]
    fn add_bot_valid_channel() {
        let dir = tempdir().unwrap();
        let bot = add_bot(
            dir.path(),
            "telegram",
            "test-bot",
            "token",
            "ws://localhost:8080",
            "gpt-4",
        )
        .unwrap();
        assert_eq!(bot.name, "test-bot");
        assert_eq!(bot.channel.to_string(), "telegram");
        assert!(bot.enabled);
        assert!(bot.id.starts_with("bot-telegram-"));
    }

    #[test]
    fn add_bot_invalid_channel() {
        let dir = tempdir().unwrap();
        let result = add_bot(
            dir.path(),
            "bogus_channel",
            "test-bot",
            "token",
            "ws://localhost",
            "gpt-4",
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unknown channel"));
    }

    #[test]
    fn add_bot_empty_name() {
        let dir = tempdir().unwrap();
        let bot = add_bot(
            dir.path(),
            "slack",
            "",
            "token",
            "ws://localhost",
            "gpt-4",
        )
        .unwrap();
        assert!(bot.name.is_empty());
    }

    #[test]
    fn add_bot_persists_to_file() {
        let dir = tempdir().unwrap();
        add_bot(
            dir.path(),
            "discord",
            "persist-bot",
            "token",
            "ws://localhost",
            "gpt-4",
        )
        .unwrap();
        let channels = load_channels(dir.path()).unwrap();
        let discord_ch = channels.iter().find(|c| c.channel == ChannelType::Discord).unwrap();
        assert_eq!(discord_ch.bots.len(), 1);
        assert_eq!(discord_ch.bots[0].name, "persist-bot");
    }

    // ── remove_bot ───────────────────────────────────

    #[test]
    fn remove_bot_existing() {
        let dir = tempdir().unwrap();
        let bot = add_bot(
            dir.path(),
            "slack",
            "rm-bot",
            "token",
            "ws://localhost",
            "gpt-4",
        )
        .unwrap();
        let removed = remove_bot(dir.path(), "slack", &bot.id).unwrap();
        assert!(removed);
    }

    #[test]
    fn remove_bot_nonexistent() {
        let dir = tempdir().unwrap();
        let removed = remove_bot(dir.path(), "slack", "bot-slack-nobody").unwrap();
        assert!(!removed);
    }

    #[test]
    fn remove_bot_invalid_channel() {
        let dir = tempdir().unwrap();
        let result = remove_bot(dir.path(), "invalid", "bot-id");
        assert!(result.is_err());
    }

    // ── update_bot ───────────────────────────────────

    #[test]
    fn update_bot_name() {
        let dir = tempdir().unwrap();
        let bot = add_bot(
            dir.path(),
            "wechat",
            "old-name",
            "token",
            "ws://localhost",
            "gpt-4",
        )
        .unwrap();
        let updated = update_bot(dir.path(), "wechat", &bot.id, Some("new-name"), None, None).unwrap();
        assert_eq!(updated.name, "new-name");
    }

    #[test]
    fn update_bot_not_found() {
        let dir = tempdir().unwrap();
        let result = update_bot(dir.path(), "wechat", "nonexistent-id", Some("x"), None, None);
        assert!(result.is_err());
    }

    // ── set_context_enhancement / set_proactive_delivery ──

    #[test]
    fn set_context_enhancement() {
        let dir = tempdir().unwrap();
        let ch = set_context_enhancement(dir.path(), "feishu", true).unwrap();
        assert!(ch.context_enhancement);
    }

    #[test]
    fn set_proactive_delivery() {
        let dir = tempdir().unwrap();
        let ch = set_proactive_delivery(dir.path(), "dingtalk", true).unwrap();
        assert!(ch.proactive_delivery);
    }

    #[test]
    fn set_feature_nonexistent_channel() {
        let dir = tempdir().unwrap();
        assert!(set_context_enhancement(dir.path(), "bogus", true).is_err());
        assert!(set_proactive_delivery(dir.path(), "bogus", true).is_err());
    }

    // ── dsh_market ───────────────────────────────────

    #[test]
    fn load_dsh_market_missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let config = load_dsh_market(dir.path()).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.api_endpoint, "https://dshfind.com/api");
    }

    #[test]
    fn dsh_market_toggle() {
        let dir = tempdir().unwrap();
        let config = dsh_market_toggle(dir.path(), true).unwrap();
        assert!(config.enabled);
    }

    #[test]
    fn dsh_market_config_update() {
        let dir = tempdir().unwrap();
        let config = dsh_market_config_update(
            dir.path(),
            Some("https://custom.api"),
            Some("tok_abc"),
            Some(false),
        )
        .unwrap();
        assert_eq!(config.api_endpoint, "https://custom.api");
        assert_eq!(config.auth_token.as_deref(), Some("tok_abc"));
        assert!(!config.sync_enabled);
    }

    #[test]
    fn dsh_market_sync_disabled_returns_error() {
        let dir = tempdir().unwrap();
        let result = dsh_market_sync(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }

    #[test]
    fn dsh_market_sync_enabled_returns_plugins() {
        let dir = tempdir().unwrap();
        dsh_market_toggle(dir.path(), true).unwrap();
        let plugins = dsh_market_sync(dir.path()).unwrap();
        assert!(plugins.contains_key("dsh-im-core"));
        assert!(plugins.contains_key("dsh-im-wechat"));
        assert!(plugins.contains_key("dsh-im-feishu"));
        assert!(plugins.contains_key("dsh-im-telegram"));
        assert_eq!(plugins.len(), 4);
    }

    // ── get_status ───────────────────────────────────

    #[test]
    fn get_status_empty() {
        let dir = tempdir().unwrap();
        let status = get_status(dir.path()).unwrap();
        assert_eq!(status.total_bots, 0);
        assert_eq!(status.connected_bots, 0);
        assert!(!status.dsh_market_enabled);
    }

    #[test]
    fn get_status_counts_bots() {
        let dir = tempdir().unwrap();
        add_bot(dir.path(), "telegram", "b1", "token", "ws://l", "gpt-4").unwrap();
        toggle_channel(dir.path(), "telegram", true).unwrap();
        let status = get_status(dir.path()).unwrap();
        assert_eq!(status.total_bots, 1);
        assert_eq!(status.connected_bots, 1);
    }
}
