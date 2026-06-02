use serde::{Deserialize, Serialize};

/// Supported push channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PushChannelType {
    Telegram,
    Discord,
    WeCom,
    Feishu,
    DingTalk,
    Email,
    Webhook,
}

impl PushChannelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PushChannelType::Telegram => "telegram",
            PushChannelType::Discord => "discord",
            PushChannelType::WeCom => "wecom",
            PushChannelType::Feishu => "feishu",
            PushChannelType::DingTalk => "dingtalk",
            PushChannelType::Email => "email",
            PushChannelType::Webhook => "webhook",
        }
    }
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub channel_type: PushChannelType,
    pub webhook_url: Option<String>,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub smtp_config: Option<SmtpConfig>,
    pub enabled: bool,
}

/// SMTP email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub to_address: String,
}

/// Push nt_io_notify message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushMessage {
    pub title: String,
    pub body: String,
    pub source: Option<String>,
    pub url: Option<String>,
    pub priority: PushPriority,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PushPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Push result
#[derive(Debug, Clone)]
pub struct PushResult {
    pub success: bool,
    pub channel: PushChannelType,
    pub error: Option<String>,
    pub delivered_at: u64,
}

/// Push nt_io_notify manager
pub struct PushManager {
    pub channels: Vec<ChannelConfig>,
    pub history: Vec<PushResult>,
    pub max_history: usize,
}

impl PushManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            channels: Vec::new(),
            history: Vec::new(),
            max_history,
        }
    }

    pub fn add_channel(&mut self, config: ChannelConfig) {
        self.channels.push(config);
    }

    pub fn remove_channel(&mut self, channel_type: &PushChannelType) {
        self.channels.retain(|c| c.channel_type != *channel_type);
    }

    pub fn push(&mut self, _message: &PushMessage) -> Vec<PushResult> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let results: Vec<PushResult> = self
            .channels
            .iter()
            .filter(|c| c.enabled)
            .map(|channel| {
                // In production, this would make actual API calls
                PushResult {
                    success: true,
                    channel: channel.channel_type,
                    error: None,
                    delivered_at: now,
                }
            })
            .collect();
        self.history.extend(results.clone());
        if self.history.len() > self.max_history {
            self.history.drain(0..self.history.len() - self.max_history);
        }
        results
    }

    pub fn test_channel(&self, channel_type: &PushChannelType) -> PushResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        PushResult {
            success: true,
            channel: *channel_type,
            error: None,
            delivered_at: now,
        }
    }

    pub fn enabled_count(&self) -> usize {
        self.channels.iter().filter(|c| c.enabled).count()
    }

    pub fn recent_results(&self, n: usize) -> Vec<&PushResult> {
        self.history.iter().rev().take(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_manager() {
        let mut pm = PushManager::new(100);
        assert_eq!(pm.enabled_count(), 0);
        pm.add_channel(ChannelConfig {
            channel_type: PushChannelType::Telegram,
            webhook_url: None,
            bot_token: Some("test:token".into()),
            chat_id: Some("123".into()),
            smtp_config: None,
            enabled: true,
        });
        assert_eq!(pm.enabled_count(), 1);
    }

    #[test]
    fn test_push_message() {
        let msg = PushMessage {
            title: "Test Alert".into(),
            body: "This is a test".into(),
            source: Some("weibo".into()),
            url: None,
            priority: PushPriority::Normal,
            timestamp: 1000,
        };
        assert_eq!(msg.title, "Test Alert");
    }

    #[test]
    fn test_push() {
        let mut pm = PushManager::new(10);
        pm.add_channel(ChannelConfig {
            channel_type: PushChannelType::Discord,
            webhook_url: Some("https://discord.com/api/webhooks/test".into()),
            bot_token: None,
            chat_id: None,
            smtp_config: None,
            enabled: true,
        });
        let msg = PushMessage {
            title: "Hot Topic".into(),
            body: "Something trending".into(),
            source: None,
            url: None,
            priority: PushPriority::High,
            timestamp: 1000,
        };
        let results = pm.push(&msg);
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(pm.history.len(), 1);
    }

    #[test]
    fn test_test_channel() {
        let pm = PushManager::new(10);
        let result = pm.test_channel(&PushChannelType::Telegram);
        assert!(result.success);
    }

    #[test]
    fn test_channel_type_as_str() {
        assert_eq!(PushChannelType::Telegram.as_str(), "telegram");
        assert_eq!(PushChannelType::WeCom.as_str(), "wecom");
    }
}
