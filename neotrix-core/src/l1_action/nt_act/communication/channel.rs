//! 通信渠道枚举
//!
//! 定义通用的通信渠道类型，支持所有需要通信能力的Agent。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 通信渠道 — 跨域通用
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommunicationChannel {
    /// 邮件
    Email,
    /// 电话
    Phone,
    /// 社交媒体 (通过 SocialMediaPlatform 指定具体平台)
    Social,
    /// 即时通讯 (WhatsApp/WeChat/Telegram等)
    InstantMessage,
    /// 面对面会议
    Meeting,
    /// 视频会议
    VideoCall,
    /// 网站表单
    WebsiteForm,
    /// 网站在线聊天
    WebsiteChat,
    /// 传真
    Fax,
    /// 短信
    SMS,
    /// 其他
    Other(String),
}

impl CommunicationChannel {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "email" | "Email" | "邮件" => Some(Self::Email),
            "phone" | "Phone" | "电话" => Some(Self::Phone),
            "social" | "Social" | "社交媒体" => Some(Self::Social),
            "im" | "IM" | "即时通讯" => Some(Self::InstantMessage),
            "meeting" | "Meeting" | "会议" => Some(Self::Meeting),
            "video" | "Video" | "视频会议" => Some(Self::VideoCall),
            "form" | "Form" | "表单" => Some(Self::WebsiteForm),
            "chat" | "Chat" | "在线聊天" => Some(Self::WebsiteChat),
            "fax" | "Fax" | "传真" => Some(Self::Fax),
            "sms" | "SMS" | "短信" => Some(Self::SMS),
            _ => Some(Self::Other(s.to_string())),
        }
    }

    /// 是否为数字渠道 (非线下)
    pub fn is_digital(&self) -> bool {
        matches!(
            self,
            Self::Email
                | Self::Phone
                | Self::Social
                | Self::InstantMessage
                | Self::VideoCall
                | Self::WebsiteForm
                | Self::WebsiteChat
                | Self::SMS
        )
    }

    /// 是否支持异步通信
    pub fn is_asynchronous(&self) -> bool {
        matches!(
            self,
            Self::Email | Self::Social | Self::InstantMessage | Self::SMS | Self::Fax
        )
    }

    /// 是否支持实时通信
    pub fn is_realtime(&self) -> bool {
        matches!(
            self,
            Self::Phone | Self::Meeting | Self::VideoCall | Self::WebsiteChat
        )
    }
}

impl std::fmt::Display for CommunicationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "Email"),
            Self::Phone => write!(f, "Phone"),
            Self::Social => write!(f, "Social"),
            Self::InstantMessage => write!(f, "InstantMessage"),
            Self::Meeting => write!(f, "Meeting"),
            Self::VideoCall => write!(f, "VideoCall"),
            Self::WebsiteForm => write!(f, "WebsiteForm"),
            Self::WebsiteChat => write!(f, "WebsiteChat"),
            Self::Fax => write!(f, "Fax"),
            Self::SMS => write!(f, "SMS"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_from_str() {
        assert_eq!(
            CommunicationChannel::from_str("Email"),
            Some(CommunicationChannel::Email)
        );
        assert_eq!(
            CommunicationChannel::from_str("电话"),
            Some(CommunicationChannel::Phone)
        );
        assert_eq!(
            CommunicationChannel::from_str("WhatsApp"),
            Some(CommunicationChannel::Other("WhatsApp".into()))
        );
    }

    #[test]
    fn test_channel_properties() {
        assert!(CommunicationChannel::Email.is_digital());
        assert!(CommunicationChannel::Email.is_asynchronous());
        assert!(!CommunicationChannel::Email.is_realtime());

        assert!(CommunicationChannel::Phone.is_digital());
        assert!(!CommunicationChannel::Phone.is_asynchronous());
        assert!(CommunicationChannel::Phone.is_realtime());

        assert!(!CommunicationChannel::Meeting.is_digital());
        assert!(!CommunicationChannel::Meeting.is_asynchronous());
        assert!(CommunicationChannel::Meeting.is_realtime());
    }
}
