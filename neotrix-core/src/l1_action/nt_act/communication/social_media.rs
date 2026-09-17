//! 社交媒体平台枚举
//!
//! 定义通用的社交媒体平台类型，支持所有需要社媒能力的Agent。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 社交媒体平台 — 跨域通用
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocialMediaPlatform {
    /// WhatsApp
    WhatsApp,
    /// 微信
    WeChat,
    /// LinkedIn
    LinkedIn,
    /// Facebook
    Facebook,
    /// Twitter/X
    Twitter,
    /// Instagram
    Instagram,
    /// Telegram
    Telegram,
    /// Line
    Line,
    /// Viber
    Viber,
    /// Slack
    Slack,
    /// Discord
    Discord,
    /// 其他
    Other(String),
}

impl SocialMediaPlatform {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "whatsapp" | "WhatsApp" => Some(Self::WhatsApp),
            "wechat" | "WeChat" | "微信" => Some(Self::WeChat),
            "linkedin" | "LinkedIn" => Some(Self::LinkedIn),
            "facebook" | "Facebook" => Some(Self::Facebook),
            "twitter" | "Twitter" | "x" | "X" => Some(Self::Twitter),
            "instagram" | "Instagram" => Some(Self::Instagram),
            "telegram" | "Telegram" => Some(Self::Telegram),
            "line" | "Line" => Some(Self::Line),
            "viber" | "Viber" => Some(Self::Viber),
            "slack" | "Slack" => Some(Self::Slack),
            "discord" | "Discord" => Some(Self::Discord),
            _ => Some(Self::Other(s.to_string())),
        }
    }

    /// 是否支持消息功能
    pub fn supports_messaging(&self) -> bool {
        matches!(
            self,
            Self::WhatsApp
                | Self::WeChat
                | Self::Telegram
                | Self::Line
                | Self::Viber
                | Self::Slack
                | Self::Discord
        )
    }

    /// 是否支持商业账号
    pub fn supports_business(&self) -> bool {
        matches!(
            self,
            Self::WhatsApp | Self::WeChat | Self::LinkedIn | Self::Facebook | Self::Instagram
        )
    }

    /// 是否支持广告投放
    pub fn supports_advertising(&self) -> bool {
        matches!(
            self,
            Self::Facebook | Self::LinkedIn | Self::Twitter | Self::Instagram
        )
    }

    /// 获取平台类别
    pub fn category(&self) -> SocialMediaCategory {
        match self {
            Self::WhatsApp | Self::WeChat | Self::Telegram | Self::Line | Self::Viber => {
                SocialMediaCategory::InstantMessage
            }
            Self::LinkedIn => SocialMediaCategory::Professional,
            Self::Facebook | Self::Instagram | Self::Twitter => SocialMediaCategory::Social,
            Self::Slack | Self::Discord => SocialMediaCategory::Community,
            _ => SocialMediaCategory::Other,
        }
    }
}

impl std::fmt::Display for SocialMediaPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WhatsApp => write!(f, "WhatsApp"),
            Self::WeChat => write!(f, "WeChat"),
            Self::LinkedIn => write!(f, "LinkedIn"),
            Self::Facebook => write!(f, "Facebook"),
            Self::Twitter => write!(f, "Twitter"),
            Self::Instagram => write!(f, "Instagram"),
            Self::Telegram => write!(f, "Telegram"),
            Self::Line => write!(f, "Line"),
            Self::Viber => write!(f, "Viber"),
            Self::Slack => write!(f, "Slack"),
            Self::Discord => write!(f, "Discord"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

/// 社交媒体类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocialMediaCategory {
    /// 即时通讯
    InstantMessage,
    /// 职业社交
    Professional,
    /// 社交网络
    Social,
    /// 社区/论坛
    Community,
    /// 其他
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_from_str() {
        assert_eq!(
            SocialMediaPlatform::from_str("WhatsApp"),
            Some(SocialMediaPlatform::WhatsApp)
        );
        assert_eq!(
            SocialMediaPlatform::from_str("微信"),
            Some(SocialMediaPlatform::WeChat)
        );
        assert_eq!(
            SocialMediaPlatform::from_str("LinkedIn"),
            Some(SocialMediaPlatform::LinkedIn)
        );
    }

    #[test]
    fn test_platform_capabilities() {
        assert!(SocialMediaPlatform::WhatsApp.supports_messaging());
        assert!(SocialMediaPlatform::WhatsApp.supports_business());
        assert!(!SocialMediaPlatform::WhatsApp.supports_advertising());

        assert!(SocialMediaPlatform::Facebook.supports_advertising());
        assert!(SocialMediaPlatform::LinkedIn.supports_advertising());
    }

    #[test]
    fn test_platform_category() {
        assert_eq!(
            SocialMediaPlatform::WhatsApp.category(),
            SocialMediaCategory::InstantMessage
        );
        assert_eq!(
            SocialMediaPlatform::LinkedIn.category(),
            SocialMediaCategory::Professional
        );
        assert_eq!(
            SocialMediaPlatform::Facebook.category(),
            SocialMediaCategory::Social
        );
    }
}
