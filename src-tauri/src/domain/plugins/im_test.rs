//! IM Plugin integration tests

use super::im::{ChannelType, ImPlugin, ResponseMode};
use crate::domain::app_handle::set_app_handle;
use crate::domain::{DomainError, DomainPlugin, DomainRegistry};
use std::sync::Arc;
use tokio::sync::RwLock;

fn make_plugin() -> ImPlugin {
    let registry = Arc::new(RwLock::new(DomainRegistry::new()));
    ImPlugin::new(registry)
}

fn call_sync(plugin: &ImPlugin, action: &str, args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
    tokio::runtime::Runtime::new().unwrap().block_on(plugin.call(action, args))
}

#[test]
fn test_im_plugin_name_and_actions() {
    let plugin = make_plugin();
    assert_eq!(plugin.name(), "im");
    assert!(!plugin.description().is_empty());
    let actions = plugin.actions();
    assert!(actions.len() >= 5, "Expected at least 5 actions, got {}", actions.len());

    let names: Vec<&str> = actions.iter().map(|a| a.name.as_str()).collect();
    assert!(names.contains(&"status"));
    assert!(names.contains(&"list_channels"));
    assert!(names.contains(&"add_bot"));
    assert!(names.contains(&"should_respond"));
    assert!(names.contains(&"get_channel"));
}

#[test]
fn test_status_action() {
    let plugin = make_plugin();
    let result = call_sync(&plugin, "status", serde_json::json!({}));
    assert!(result.is_ok(), "status call failed: {:?}", result.err());

    let val = result.unwrap();
    assert!(val.get("channels").is_some(), "Missing 'channels' in status response");
    assert!(val.get("total_bots").is_some(), "Missing 'total_bots' in status response");
    assert!(val.get("connected_bots").is_some(), "Missing 'connected_bots' in status response");
    assert!(val.get("dsh_market_enabled").is_some(), "Missing 'dsh_market_enabled' in status response");

    // Default channels should be 9 (WeChat, Feishu, DingTalk, WeCom, QQ, Slack, Telegram, Discord, WhatsApp)
    let channels = val["channels"].as_array().unwrap();
    assert_eq!(channels.len(), 9, "Expected 9 default channels");
}

#[test]
fn test_list_channels_action() {
    let plugin = make_plugin();
    let result = call_sync(&plugin, "list_channels", serde_json::json!({}));
    assert!(result.is_ok(), "list_channels call failed: {:?}", result.err());

    let channels = result.unwrap().as_array().unwrap().clone();
    assert_eq!(channels.len(), 9);

    // Verify all channel types are present
    let channel_names: Vec<String> = channels
        .iter()
        .map(|c| c["channel"].as_str().unwrap().to_string())
        .collect();
    assert!(channel_names.contains(&"wechat".to_string()));
    assert!(channel_names.contains(&"feishu".to_string()));
    assert!(channel_names.contains(&"dingtalk".to_string()));
    assert!(channel_names.contains(&"wecom".to_string()));
    assert!(channel_names.contains(&"qq".to_string()));
    assert!(channel_names.contains(&"slack".to_string()));
    assert!(channel_names.contains(&"telegram".to_string()));
    assert!(channel_names.contains(&"discord".to_string()));
    assert!(channel_names.contains(&"whatsapp".to_string()));
}

#[test]
fn test_add_bot_action() {
    let plugin = make_plugin();

    // Add a bot to WeChat
    let result = call_sync(&plugin, 
        "add_bot",
        serde_json::json!({
            "channel": "wechat",
            "name": "TestBot",
            "credential_type": "token",
            "workspace": "test-workspace",
            "model": "gpt-4"
        }),
    );
    assert!(result.is_ok(), "add_bot call failed: {:?}", result.err());

    let bot = result.unwrap();
    assert_eq!(bot["channel"], "wechat");
    assert_eq!(bot["name"], "TestBot");
    assert_eq!(bot["credential_type"], "token");
    assert_eq!(bot["workspace"], "test-workspace");
    assert_eq!(bot["model"], "gpt-4");
    assert_eq!(bot["enabled"], true);
    assert!(bot["id"].as_str().unwrap().starts_with("bot-wechat-"));

    // Verify bot was added to channel
    let status = call_sync(&plugin, "status", serde_json::json!({})).unwrap();
    let total_bots = status["total_bots"].as_u64().unwrap();
    assert_eq!(total_bots, 1, "Expected 1 total bot after add_bot");
}

#[test]
fn test_add_bot_invalid_channel() {
    let plugin = make_plugin();
    let result = call_sync(&plugin, 
        "add_bot",
        serde_json::json!({
            "channel": "nonexistent",
            "name": "TestBot"
        }),
    );
    assert!(result.is_err(), "Expected error for invalid channel");
    let err = result.err().unwrap();
    assert_eq!(err.code, "INVALID_CHANNEL");
}

#[test]
fn test_should_respond_group_invite() {
    let plugin = make_plugin();

    // Add a bot first
    let bot_result = call_sync(&plugin, 
        "add_bot",
        serde_json::json!({
            "channel": "slack",
            "name": "MyBot"
        }),
    )
    .unwrap();
    let bot_id = bot_result["id"].as_str().unwrap().to_string();

    // GroupInvite mode: only responds when @mentioned
    let result = call_sync(&plugin, 
        "should_respond",
        serde_json::json!({
            "channel": "slack",
            "bot_id": bot_id,
            "chat_id": "general",
            "sender_id": "user123",
            "text": "Hello everyone!"
        }),
    )
    .unwrap();
    assert_eq!(result, serde_json::json!(false), "Should NOT respond without @mention");

    let result = call_sync(&plugin, 
        "should_respond",
        serde_json::json!({
            "channel": "slack",
            "bot_id": bot_id,
            "chat_id": "general",
            "sender_id": "user123",
            "text": "Hey @MyBot, can you help?"
        }),
    )
    .unwrap();
    assert_eq!(result, serde_json::json!(true), "Should respond when @mentioned");
}

#[test]
fn test_should_respond_group_keyword() {
    let plugin = make_plugin();

    // Add a bot and set to GroupKeyword mode
    let bot_result = call_sync(&plugin, 
        "add_bot",
        serde_json::json!({
            "channel": "telegram",
            "name": "HelperBot"
        }),
    )
    .unwrap();
    let bot_id = bot_result["id"].as_str().unwrap().to_string();

    call_sync(&plugin, 
        "set_response_mode",
        serde_json::json!({
            "channel": "telegram",
            "bot_id": bot_id,
            "mode": "group_keyword",
            "keyword": "help"
        }),
    )
    .unwrap();

    // Without keyword
    let result = call_sync(&plugin, 
        "should_respond",
        serde_json::json!({
            "channel": "telegram",
            "bot_id": bot_id,
            "chat_id": "group1",
            "sender_id": "user1",
            "text": "Nice weather today"
        }),
    )
    .unwrap();
    assert_eq!(result, serde_json::json!(false));

    // With keyword
    let result = call_sync(&plugin, 
        "should_respond",
        serde_json::json!({
            "channel": "telegram",
            "bot_id": bot_id,
            "chat_id": "group1",
            "sender_id": "user1",
            "text": "I need help with something"
        }),
    )
    .unwrap();
    assert_eq!(result, serde_json::json!(true));
}

#[test]
fn test_should_respond_private_mode() {
    let plugin = make_plugin();

    // Add a bot and set to Private mode
    let bot_result = call_sync(&plugin, 
        "add_bot",
        serde_json::json!({
            "channel": "discord",
            "name": "PrivateBot"
        }),
    )
    .unwrap();
    let bot_id = bot_result["id"].as_str().unwrap().to_string();

    call_sync(&plugin, 
        "set_response_mode",
        serde_json::json!({
            "channel": "discord",
            "bot_id": bot_id,
            "mode": "private"
        }),
    )
    .unwrap();

    // Private mode with empty whitelist = respond to all
    let result = call_sync(&plugin, 
        "should_respond",
        serde_json::json!({
            "channel": "discord",
            "bot_id": bot_id,
            "chat_id": "dm1",
            "sender_id": "anyone",
            "text": "hi"
        }),
    )
    .unwrap();
    assert_eq!(result, serde_json::json!(true));

    // Add a whitelist user
    call_sync(&plugin, 
        "add_whitelist",
        serde_json::json!({
            "channel": "discord",
            "bot_id": bot_id,
            "user_id": "allowed_user"
        }),
    )
    .unwrap();

    // Whitelisted user
    let result = call_sync(&plugin, 
        "should_respond",
        serde_json::json!({
            "channel": "discord",
            "bot_id": bot_id,
            "chat_id": "dm1",
            "sender_id": "allowed_user",
            "text": "hi"
        }),
    )
    .unwrap();
    assert_eq!(result, serde_json::json!(true));

    // Non-whitelisted user
    let result = call_sync(&plugin, 
        "should_respond",
        serde_json::json!({
            "channel": "discord",
            "bot_id": bot_id,
            "chat_id": "dm1",
            "sender_id": "blocked_user",
            "text": "hi"
        }),
    )
    .unwrap();
    assert_eq!(result, serde_json::json!(false));
}

#[test]
fn test_should_respond_group_all() {
    let plugin = make_plugin();

    let bot_result = call_sync(&plugin, 
        "add_bot",
        serde_json::json!({
            "channel": "feishu",
            "name": "EchoBot"
        }),
    )
    .unwrap();
    let bot_id = bot_result["id"].as_str().unwrap().to_string();

    call_sync(&plugin, 
        "set_response_mode",
        serde_json::json!({
            "channel": "feishu",
            "bot_id": bot_id,
            "mode": "group_all"
        }),
    )
    .unwrap();

    // GroupAll mode: responds to every message
    let result = call_sync(&plugin, 
        "should_respond",
        serde_json::json!({
            "channel": "feishu",
            "bot_id": bot_id,
            "chat_id": "group1",
            "sender_id": "user1",
            "text": "anything at all"
        }),
    )
    .unwrap();
    assert_eq!(result, serde_json::json!(true));
}

#[test]
fn test_should_respond_missing_bot() {
    let plugin = make_plugin();
    let result = call_sync(&plugin, 
        "should_respond",
        serde_json::json!({
            "channel": "slack",
            "bot_id": "nonexistent-bot",
            "chat_id": "general",
            "sender_id": "user1",
            "text": "hello"
        }),
    );
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().code, "NOT_FOUND");
}

#[test]
fn test_channel_type_parsing() {
    assert_eq!(ChannelType::from_name("wechat"), Some(ChannelType::WeChat));
    assert_eq!(ChannelType::from_name("微信"), Some(ChannelType::WeChat));
    assert_eq!(ChannelType::from_name("feishu"), Some(ChannelType::Feishu));
    assert_eq!(ChannelType::from_name("飞书"), Some(ChannelType::Feishu));
    assert_eq!(ChannelType::from_name("dingtalk"), Some(ChannelType::DingTalk));
    assert_eq!(ChannelType::from_name("钉钉"), Some(ChannelType::DingTalk));
    assert_eq!(ChannelType::from_name("slack"), Some(ChannelType::Slack));
    assert_eq!(ChannelType::from_name("telegram"), Some(ChannelType::Telegram));
    assert_eq!(ChannelType::from_name("discord"), Some(ChannelType::Discord));
    assert_eq!(ChannelType::from_name("whatsapp"), Some(ChannelType::WhatsApp));
    assert_eq!(ChannelType::from_name("unknown"), None);
    assert_eq!(ChannelType::from_name(""), None);
}

#[test]
fn test_channel_type_display() {
    assert_eq!(ChannelType::WeChat.to_string(), "wechat");
    assert_eq!(ChannelType::Feishu.to_string(), "feishu");
    assert_eq!(ChannelType::DingTalk.to_string(), "dingtalk");
    assert_eq!(ChannelType::Slack.to_string(), "slack");
    assert_eq!(ChannelType::Telegram.to_string(), "telegram");
    assert_eq!(ChannelType::Discord.to_string(), "discord");
    assert_eq!(ChannelType::WhatsApp.to_string(), "whatsapp");
}

#[test]
fn test_session_id_roundtrip() {
    use crate::domain::plugins::im::SessionChannelPrefix;

    let prefix = SessionChannelPrefix {
        channel: ChannelType::Slack,
        bot_id: "bot-001".to_string(),
        chat_id: "C1234".to_string(),
    };
    let session_id = prefix.to_session_id();
    assert_eq!(session_id, "slack:bot-001:C1234");

    let parsed = SessionChannelPrefix::from_session_id(&session_id).unwrap();
    assert_eq!(parsed.channel, ChannelType::Slack);
    assert_eq!(parsed.bot_id, "bot-001");
    assert_eq!(parsed.chat_id, "C1234");

    // Invalid session id
    assert!(SessionChannelPrefix::from_session_id("invalid").is_none());
    assert!(SessionChannelPrefix::from_session_id("unknown:bot:chat").is_none());
}

#[test]
fn test_get_channel_action() {
    let plugin = make_plugin();

    let result = call_sync(&plugin, 
        "get_channel",
        serde_json::json!({ "channel": "slack" }),
    );
    assert!(result.is_ok(), "get_channel failed: {:?}", result.err());
    let ch = result.unwrap();
    assert_eq!(ch["channel"], "slack");
    assert_eq!(ch["enabled"], false);

    // Invalid channel
    let result = call_sync(&plugin, 
        "get_channel",
        serde_json::json!({ "channel": "nope" }),
    );
    assert!(result.is_err());
}

#[test]
fn test_remove_bot() {
    let plugin = make_plugin();

    // Add then remove
    let bot = call_sync(&plugin, 
        "add_bot",
        serde_json::json!({ "channel": "whatsapp", "name": "TempBot" }),
    )
    .unwrap();
    let bot_id = bot["id"].as_str().unwrap();

    let removed = call_sync(&plugin, 
        "remove_bot",
        serde_json::json!({ "channel": "whatsapp", "bot_id": bot_id }),
    )
    .unwrap();
    assert_eq!(removed, serde_json::json!(true));

    // Verify removed
    let status = call_sync(&plugin, "status", serde_json::json!({})).unwrap();
    assert_eq!(status["total_bots"], 0);

    // Remove again should return false
    let removed = call_sync(&plugin, 
        "remove_bot",
        serde_json::json!({ "channel": "whatsapp", "bot_id": bot_id }),
    )
    .unwrap();
    assert_eq!(removed, serde_json::json!(false));
}
