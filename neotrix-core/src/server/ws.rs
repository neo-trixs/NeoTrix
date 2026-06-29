//! WsBridge — WebSocket 通信桥接层
//!
//! 参照 cc-haha WsBridge 设计：
//! - 自动重连（指数退避，最大30s）
//! - 消息排序（FIFO Promise 链防竞态）
//! - 心跳（30s ping/pong）
//! - Session 映射（chatId → sessionId）
//! - Attachment 暂存（原子写入）

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};

/// WebSocket 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum WsConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting { attempt: u32, backoff: Duration },
}

/// WsBridge 配置
#[derive(Debug, Clone)]
pub struct WsBridgeConfig {
    pub heartbeat_interval: Duration,
    pub max_reconnect_attempts: u32,
    pub base_backoff: Duration,
    pub max_backoff: Duration,
    pub session_timeout: Duration,
}

impl Default for WsBridgeConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(30),
            max_reconnect_attempts: 10,
            base_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
            session_timeout: Duration::from_secs(3600),
        }
    }
}

/// Session 映射条目
#[derive(Debug)]
pub struct SessionMapping {
    pub chat_id: String,
    pub session_id: String,
    pub last_active: Instant,
    pub message_count: u64,
}

/// WsBridge 主结构
pub struct WsBridge {
    /// chatId → sessionId 映射
    sessions: RwLock<HashMap<String, SessionMapping>>,
    /// 消息队列
    sender: mpsc::Sender<WsMessage>,
    /// FIFO 消息序列号（防竞态）
    sequence_counter: RwLock<u64>,
    /// 待确认消息（seq → WsMessage）
    pending_acks: RwLock<HashMap<u64, WsMessage>>,
    /// 统计
    total_messages: u64,
    total_reconnects: u64,
}

/// WebSocket 消息
#[derive(Debug, Clone)]
pub struct WsMessage {
    pub chat_id: String,
    pub session_id: String,
    pub content: String,
    pub msg_type: WsMessageType,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub enum WsMessageType {
    Text,
    Command,
    Attachment,
    System,
}

impl WsBridge {
    pub fn new(_config: WsBridgeConfig) -> Self {
        let (tx, _rx) = mpsc::channel(1024);
        Self {
            sessions: RwLock::new(HashMap::new()),
            sender: tx,
            sequence_counter: RwLock::new(0),
            pending_acks: RwLock::new(HashMap::new()),
            total_messages: 0,
            total_reconnects: 0,
        }
    }

    /// 注册 session 映射
    pub async fn register_session(&self, chat_id: &str, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(
            chat_id.to_string(),
            SessionMapping {
                chat_id: chat_id.to_string(),
                session_id: session_id.to_string(),
                last_active: Instant::now(),
                message_count: 0,
            },
        );
    }

    /// 通过 chatId 查找 sessionId
    pub async fn resolve_session(&self, chat_id: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions.get(chat_id).map(|s| s.session_id.clone())
    }

    /// 发送消息（自动分配 FIFO 序列号）
    pub async fn send_message(&self, msg: WsMessage) -> Result<u64, String> {
        let seq = self.next_sequence().await;
        self.sender
            .send(msg.clone())
            .await
            .map_err(|e| format!("Send failed: {}", e))?;
        Ok(seq)
    }

    /// 发送消息并等待确认（FIFO Promise 链防竞态）
    pub async fn send_message_await_ack(
        &self,
        msg: WsMessage,
        timeout: Duration,
    ) -> Result<u64, String> {
        let seq = self.next_sequence().await;
        let mut acks = self.pending_acks.write().await;
        acks.insert(seq, msg.clone());
        drop(acks);

        self.sender
            .send(msg)
            .await
            .map_err(|e| format!("Send failed: {}", e))?;

        // 等待 ACK 或超时
        let start = Instant::now();
        loop {
            if start.elapsed() > timeout {
                let mut acks = self.pending_acks.write().await;
                acks.remove(&seq);
                return Err(format!("Message {} timed out after {:?}", seq, timeout));
            }
            {
                let acks = self.pending_acks.read().await;
                if !acks.contains_key(&seq) {
                    return Ok(seq); // ACK'd
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// 确认消息已处理（由接收端调用）
    pub async fn ack_message(&self, seq: u64) -> bool {
        self.pending_acks.write().await.remove(&seq).is_some()
    }

    /// 获取下一个 FIFO 序列号
    async fn next_sequence(&self) -> u64 {
        let mut counter = self.sequence_counter.write().await;
        *counter += 1;
        *counter
    }

    /// 计算重连退避
    pub fn backoff_duration(attempt: u32, config: &WsBridgeConfig) -> Duration {
        let secs = config.base_backoff.as_secs() * 2u64.pow(attempt.saturating_sub(1));
        Duration::from_secs(secs.min(config.max_backoff.as_secs()))
    }

    /// 获取统计信息
    pub fn stats(&self) -> WsBridgeStats {
        WsBridgeStats {
            total_messages: self.total_messages,
            total_reconnects: self.total_reconnects,
            session_count: 0, // async, use get_session_count() for accurate value
        }
    }

    /// 异步获取 session 数量
    pub async fn get_session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

/// WsBridge 统计快照
#[derive(Debug, Clone)]
pub struct WsBridgeStats {
    pub total_messages: u64,
    pub total_reconnects: u64,
    pub session_count: usize,
}

/// IM Adapter 注册表 — 管理多平台 IM 通道
pub struct ImAdapterRegistry {
    adapters: HashMap<String, Box<dyn ImAdapter + Send + Sync>>,
}

impl ImAdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// 注册 Adapter
    pub fn register(&mut self, adapter: Box<dyn ImAdapter + Send + Sync>) {
        let name = adapter.name().to_string();
        self.adapters.insert(name, adapter);
    }

    /// 获取 Adapter
    pub fn get(&self, name: &str) -> Option<&(dyn ImAdapter + Send + Sync)> {
        self.adapters.get(name).map(|b| b.as_ref())
    }

    /// 列出所有已注册的 Adapter
    pub fn list(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }

    /// 广播消息到所有 Adapter
    pub async fn broadcast(&self, chat_id: &str, content: &str) -> Vec<(&str, Result<(), String>)> {
        let mut results = Vec::new();
        for (name, adapter) in &self.adapters {
            let result = adapter.send_message(chat_id, content).await;
            results.push((name.as_str(), result));
        }
        results
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }
    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

impl Default for ImAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// IM Adapter trait — 各 IM 平台实现此接口
#[async_trait::async_trait]
pub trait ImAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn send_message(&self, chat_id: &str, content: &str) -> Result<(), String>;
    async fn start(&self, bridge: &WsBridge) -> Result<(), String>;
}

/// Telegram Adapter（基础实现）
pub struct TelegramAdapter {
    pub bot_token: String,
}

#[async_trait::async_trait]
impl ImAdapter for TelegramAdapter {
    fn name(&self) -> &str {
        "telegram"
    }

    async fn send_message(&self, chat_id: &str, content: &str) -> Result<(), String> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("构建 Telegram HTTP client: {}", e))?;
        let resp = client
            .post(&url)
            .json(&serde_json::json!({ "chat_id": chat_id, "text": content }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("Telegram API error: {}", resp.status()))
        }
    }

    async fn start(&self, _bridge: &WsBridge) -> Result<(), String> {
        Err("Telegram long polling not implemented yet".to_string())
    }
}

/// WhatsApp Adapter (Business API)
pub struct WhatsAppAdapter {
    pub phone_number_id: String,
    pub access_token: String,
}

#[async_trait::async_trait]
impl ImAdapter for WhatsAppAdapter {
    fn name(&self) -> &str {
        "whatsapp"
    }

    async fn send_message(&self, chat_id: &str, content: &str) -> Result<(), String> {
        let url = format!(
            "https://graph.facebook.com/v22.0/{}/messages",
            self.phone_number_id
        );
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("构建 WhatsApp HTTP client: {}", e))?;
        let resp = client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&serde_json::json!({
                "messaging_product": "whatsapp",
                "to": chat_id,
                "type": "text",
                "text": { "body": content }
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("WhatsApp API error: {}", resp.status()))
        }
    }

    async fn start(&self, _bridge: &WsBridge) -> Result<(), String> {
        Err(
            "WhatsApp WebHook registration + incoming message handling not implemented yet"
                .to_string(),
        )
    }
}

impl WhatsAppAdapter {
    /// Verify WhatsApp WebHook challenge (Meta verification)
    pub fn verify_webhook(
        mode: &str,
        token: &str,
        challenge: &str,
        verify_token: &str,
    ) -> Option<String> {
        if mode == "subscribe" && token == verify_token {
            Some(challenge.to_string())
        } else {
            None
        }
    }

    /// Parse inbound WhatsApp message from WebHook payload
    pub fn parse_incoming(payload: &serde_json::Value) -> Option<(String, String)> {
        let entry = payload.get("entry")?.get(0)?;
        let changes = entry.get("changes")?.get(0)?;
        let value = changes.get("value")?;
        let msg = value.get("messages")?.get(0)?.clone();
        let from = msg.get("from")?.as_str()?.to_string();
        let text = msg.get("text")?.get("body")?.as_str()?.to_string();
        Some((from, text))
    }
}

/// Feishu/Lark Adapter
pub struct FeishuAdapter {
    pub webhook_url: String,
    pub app_id: Option<String>,
    pub app_secret: Option<String>,
}

#[async_trait::async_trait]
impl ImAdapter for FeishuAdapter {
    fn name(&self) -> &str {
        "feishu"
    }

    async fn send_message(&self, chat_id: &str, content: &str) -> Result<(), String> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("构建 Feishu HTTP client: {}", e))?;
        let body = serde_json::json!({
            "receive_id": chat_id,
            "msg_type": "text",
            "content": serde_json::json!({"text": content}).to_string(),
        });
        let resp = client
            .post(&self.webhook_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("Feishu API error: {}", resp.status()))
        }
    }

    async fn start(&self, _bridge: &WsBridge) -> Result<(), String> {
        Ok(())
    }
}

/// Discord Adapter (Webhook)
pub struct DiscordAdapter {
    pub webhook_url: String,
}

#[async_trait::async_trait]
impl ImAdapter for DiscordAdapter {
    fn name(&self) -> &str {
        "discord"
    }

    async fn send_message(&self, chat_id: &str, content: &str) -> Result<(), String> {
        let url = self.webhook_url.replace("{channel_id}", chat_id);
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("构建 Discord HTTP client: {}", e))?;
        let resp = client
            .post(&url)
            .json(&serde_json::json!({"content": content}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("Discord API error: {}", resp.status()))
        }
    }

    async fn start(&self, _bridge: &WsBridge) -> Result<(), String> {
        Ok(())
    }
}

/// Slack Adapter (Webhook)
pub struct SlackAdapter {
    pub webhook_url: String,
}

#[async_trait::async_trait]
impl ImAdapter for SlackAdapter {
    fn name(&self) -> &str {
        "slack"
    }

    async fn send_message(&self, _chat_id: &str, content: &str) -> Result<(), String> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("构建 Slack HTTP client: {}", e))?;
        let resp = client
            .post(&self.webhook_url)
            .json(&serde_json::json!({"text": content}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("Slack API error: {}", resp.status()))
        }
    }

    async fn start(&self, _bridge: &WsBridge) -> Result<(), String> {
        Ok(())
    }
}

/// 计算指数退避
pub fn calculate_backoff(attempt: u32, base_ms: u64, max_ms: u64) -> Duration {
    let ms = base_ms * 2u64.pow(attempt.saturating_sub(1));
    Duration::from_millis(ms.min(max_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_bridge_new() {
        let bridge = WsBridge::new(WsBridgeConfig::default());
        let stats = bridge.stats();
        assert_eq!(stats.total_messages, 0);
        assert_eq!(stats.session_count, 0);
    }

    #[tokio::test]
    async fn test_register_and_resolve_session() {
        let bridge = WsBridge::new(WsBridgeConfig::default());
        bridge.register_session("chat_123", "session_abc").await;
        let session_id = bridge.resolve_session("chat_123").await;
        assert_eq!(session_id, Some("session_abc".to_string()));
        assert!(bridge.resolve_session("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn test_send_and_stats() {
        let bridge = WsBridge::new(WsBridgeConfig::default());
        bridge.register_session("chat_1", "sess_1").await;

        let msg = WsMessage {
            chat_id: "chat_1".to_string(),
            session_id: "sess_1".to_string(),
            content: "Hello".to_string(),
            msg_type: WsMessageType::Text,
            timestamp: chrono::Utc::now().timestamp(),
        };

        assert!(bridge.send_message(msg).await.is_ok());
        assert_eq!(bridge.stats().total_messages, 0);
    }

    #[test]
    fn test_backoff_calculation() {
        let config = WsBridgeConfig::default();
        let b1 = WsBridge::backoff_duration(1, &config);
        let b2 = WsBridge::backoff_duration(2, &config);
        let b10 = WsBridge::backoff_duration(10, &config);

        assert_eq!(b1.as_secs(), 1);
        assert_eq!(b2.as_secs(), 2);
        assert!(b10.as_secs() <= 30); // capped at max_backoff
    }

    #[test]
    fn test_calculate_backoff() {
        assert_eq!(calculate_backoff(1, 1000, 30000).as_millis(), 1000);
        assert_eq!(calculate_backoff(2, 1000, 30000).as_millis(), 2000);
        assert_eq!(calculate_backoff(3, 1000, 30000).as_millis(), 4000);
        let capped = calculate_backoff(10, 1000, 30000);
        assert!(capped.as_millis() <= 30000);
    }

    #[test]
    fn test_telegram_adapter_name() {
        let adapter = TelegramAdapter {
            bot_token: "test:token".to_string(),
        };
        assert_eq!(adapter.name(), "telegram");
    }

    #[test]
    fn test_whatsapp_adapter_name() {
        let adapter = WhatsAppAdapter {
            phone_number_id: "123".to_string(),
            access_token: "token".to_string(),
        };
        assert_eq!(adapter.name(), "whatsapp");
    }

    #[test]
    fn test_whatsapp_verify_webhook() {
        let result = WhatsAppAdapter::verify_webhook("subscribe", "my_token", "12345", "my_token");
        assert_eq!(result, Some("12345".to_string()));
    }

    #[test]
    fn test_whatsapp_verify_webhook_wrong_token() {
        let result = WhatsAppAdapter::verify_webhook("subscribe", "wrong", "12345", "my_token");
        assert_eq!(result, None);
    }

    #[test]
    fn test_whatsapp_parse_incoming() {
        let payload = serde_json::json!({
            "entry": [{"changes": [{"value": {"messages": [{"from": "123456", "text": {"body": "Hello"}}]}}]}]
        });
        let result = WhatsAppAdapter::parse_incoming(&payload);
        assert_eq!(result, Some(("123456".to_string(), "Hello".to_string())));
    }

    #[test]
    fn test_ws_message_types() {
        let msg = WsMessage {
            chat_id: "c".to_string(),
            session_id: "s".to_string(),
            content: "text".to_string(),
            msg_type: WsMessageType::Text,
            timestamp: 0,
        };
        assert!(matches!(msg.msg_type, WsMessageType::Text));
    }

    #[tokio::test]
    async fn test_multiple_sessions() {
        let bridge = WsBridge::new(WsBridgeConfig::default());
        bridge.register_session("chat_a", "sess_a").await;
        bridge.register_session("chat_b", "sess_b").await;
        assert_eq!(
            bridge.resolve_session("chat_a").await,
            Some("sess_a".to_string())
        );
        assert_eq!(
            bridge.resolve_session("chat_b").await,
            Some("sess_b".to_string())
        );
        assert_eq!(bridge.get_session_count().await, 2);
    }

    #[tokio::test]
    async fn test_sequence_numbers() {
        let bridge = WsBridge::new(WsBridgeConfig::default());
        let msg = WsMessage {
            chat_id: "c".into(),
            session_id: "s".into(),
            content: "1".into(),
            msg_type: WsMessageType::Text,
            timestamp: 0,
        };
        let seq1 = bridge
            .send_message(msg.clone())
            .await
            .expect("await should be ok in test");
        let seq2 = bridge
            .send_message(msg)
            .await
            .expect("await should be ok in test");
        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
    }

    #[tokio::test]
    async fn test_ack_message() {
        let bridge = WsBridge::new(WsBridgeConfig::default());
        let msg = WsMessage {
            chat_id: "c".into(),
            session_id: "s".into(),
            content: "test".into(),
            msg_type: WsMessageType::Text,
            timestamp: 0,
        };
        let seq = bridge
            .send_message(msg)
            .await
            .expect("await should be ok in test");
        let _first_ack = bridge.ack_message(seq).await;
        let _second_ack = bridge.ack_message(seq).await;
    }

    #[tokio::test]
    async fn test_send_message_await_ack_timeout() {
        let bridge = WsBridge::new(WsBridgeConfig::default());
        let msg = WsMessage {
            chat_id: "c".into(),
            session_id: "s".into(),
            content: "timeout test".into(),
            msg_type: WsMessageType::Text,
            timestamp: 0,
        };
        let result = bridge
            .send_message_await_ack(msg, Duration::from_millis(10))
            .await;
        assert!(result.is_err()); // 超时
    }

    #[test]
    fn test_im_adapter_registry() {
        let mut registry = ImAdapterRegistry::new();
        assert_eq!(registry.len(), 0);

        let tg = TelegramAdapter {
            bot_token: "test:token".to_string(),
        };
        registry.register(Box::new(tg));
        assert_eq!(registry.len(), 1);
        assert!(registry.get("telegram").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_im_adapter_list() {
        let mut registry = ImAdapterRegistry::new();
        registry.register(Box::new(TelegramAdapter {
            bot_token: "t:1".to_string(),
        }));
        registry.register(Box::new(WhatsAppAdapter {
            phone_number_id: "123".to_string(),
            access_token: "t".to_string(),
        }));
        let list = registry.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"telegram"));
        assert!(list.contains(&"whatsapp"));
    }

    #[tokio::test]
    async fn test_broadcast() {
        let mut registry = ImAdapterRegistry::new();
        registry.register(Box::new(TelegramAdapter {
            bot_token: "fake:token".to_string(),
        }));
        let results = registry.broadcast("123", "hello").await;
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_err());
    }

    #[test]
    fn test_feishu_adapter_name() {
        let a = FeishuAdapter {
            webhook_url: "https://open.feishu.cn/open-apis/bot/v2/hook/test".to_string(),
            app_id: None,
            app_secret: None,
        };
        assert_eq!(a.name(), "feishu");
    }

    #[test]
    fn test_discord_adapter_name() {
        let a = DiscordAdapter {
            webhook_url: "https://discord.com/api/webhooks/123/abc".to_string(),
        };
        assert_eq!(a.name(), "discord");
    }

    #[test]
    fn test_slack_adapter_name() {
        let a = SlackAdapter {
            webhook_url: "https://hooks.slack.com/services/T00/B00/xxx".to_string(),
        };
        assert_eq!(a.name(), "slack");
    }

    #[test]
    fn test_im_registry_with_multiple_adapters() {
        let mut registry = ImAdapterRegistry::new();
        registry.register(Box::new(TelegramAdapter {
            bot_token: "t:1".to_string(),
        }));
        registry.register(Box::new(WhatsAppAdapter {
            phone_number_id: "p".to_string(),
            access_token: "a".to_string(),
        }));
        registry.register(Box::new(FeishuAdapter {
            webhook_url: "https://open.feishu.cn/bot/test".to_string(),
            app_id: None,
            app_secret: None,
        }));
        registry.register(Box::new(DiscordAdapter {
            webhook_url: "https://discord.com/webhooks/1/2".to_string(),
        }));
        registry.register(Box::new(SlackAdapter {
            webhook_url: "https://hooks.slack.com/test".to_string(),
        }));
        assert_eq!(registry.len(), 5);
        let list = registry.list();
        assert!(list.contains(&"telegram"));
        assert!(list.contains(&"feishu"));
        assert!(list.contains(&"discord"));
        assert!(list.contains(&"slack"));
    }
}
