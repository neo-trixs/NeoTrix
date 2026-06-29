#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlatformChannel {
    Telegram,
    Discord,
    Slack,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub platform: PlatformChannel,
    pub channel_id: String,
    pub content: String,
    pub sender: String,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_channel_variants() {
        let tg = PlatformChannel::Telegram;
        let dc = PlatformChannel::Discord;
        let sl = PlatformChannel::Slack;
        let cx = PlatformChannel::Custom("matrix".into());
        assert_eq!(tg, PlatformChannel::Telegram);
        assert_eq!(dc, PlatformChannel::Discord);
        assert_eq!(sl, PlatformChannel::Slack);
        assert_eq!(cx, PlatformChannel::Custom("matrix".into()));
    }

    #[test]
    fn test_message_construction() {
        let msg = Message {
            platform: PlatformChannel::Discord,
            channel_id: "12345".into(),
            content: "Hello from NeoTrix".into(),
            sender: "ai-agent".into(),
            timestamp: 1700000000,
        };
        assert_eq!(msg.sender, "ai-agent");
        assert_eq!(msg.timestamp, 1700000000);
    }

    #[test]
    fn test_custom_channel_inequality() {
        let a = PlatformChannel::Custom("a".into());
        let b = PlatformChannel::Custom("b".into());
        assert_ne!(a, b);
    }
}
