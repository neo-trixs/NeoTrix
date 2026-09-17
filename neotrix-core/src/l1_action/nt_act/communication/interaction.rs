//! 交互类型和记录
//!
//! 定义通用的交互类型和交互记录结构，支持所有需要交互管理的Agent。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::channel::CommunicationChannel;

/// 交互类型 — 跨域通用
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionType {
    /// 邮件
    Email,
    /// 电话
    Phone,
    /// 即时消息
    InstantMessage,
    /// 面对面会议
    Meeting,
    /// 视频会议
    VideoCall,
    /// 网站表单提交
    WebsiteForm,
    /// 网站在线聊天
    WebsiteChat,
    /// 社交媒体互动
    SocialMedia,
    /// 短信
    SMS,
    /// 系统自动记录
    System,
    /// 其他
    Other(String),
}

impl InteractionType {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "email" | "Email" | "邮件" => Some(Self::Email),
            "phone" | "Phone" | "电话" => Some(Self::Phone),
            "im" | "IM" | "即时消息" => Some(Self::InstantMessage),
            "meeting" | "Meeting" | "会议" => Some(Self::Meeting),
            "video" | "Video" | "视频会议" => Some(Self::VideoCall),
            "form" | "Form" | "表单" => Some(Self::WebsiteForm),
            "chat" | "Chat" | "在线聊天" => Some(Self::WebsiteChat),
            "social" | "Social" | "社交媒体" => Some(Self::SocialMedia),
            "sms" | "SMS" | "短信" => Some(Self::SMS),
            "system" | "System" | "系统" => Some(Self::System),
            _ => Some(Self::Other(s.to_string())),
        }
    }

    /// 转换为通信渠道
    pub fn to_channel(&self) -> CommunicationChannel {
        match self {
            Self::Email => CommunicationChannel::Email,
            Self::Phone => CommunicationChannel::Phone,
            Self::InstantMessage => CommunicationChannel::InstantMessage,
            Self::Meeting => CommunicationChannel::Meeting,
            Self::VideoCall => CommunicationChannel::VideoCall,
            Self::WebsiteForm => CommunicationChannel::WebsiteForm,
            Self::WebsiteChat => CommunicationChannel::WebsiteChat,
            Self::SocialMedia => CommunicationChannel::Social,
            Self::SMS => CommunicationChannel::SMS,
            Self::System => CommunicationChannel::Other("System".into()),
            Self::Other(s) => CommunicationChannel::Other(s.clone()),
        }
    }

    /// 是否为数字渠道
    pub fn is_digital(&self) -> bool {
        self.to_channel().is_digital()
    }
}

impl std::fmt::Display for InteractionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "Email"),
            Self::Phone => write!(f, "Phone"),
            Self::InstantMessage => write!(f, "InstantMessage"),
            Self::Meeting => write!(f, "Meeting"),
            Self::VideoCall => write!(f, "VideoCall"),
            Self::WebsiteForm => write!(f, "WebsiteForm"),
            Self::WebsiteChat => write!(f, "WebsiteChat"),
            Self::SocialMedia => write!(f, "SocialMedia"),
            Self::SMS => write!(f, "SMS"),
            Self::System => write!(f, "System"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

/// 交互记录 — 跨域通用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    /// 记录 ID
    pub id: String,
    /// 交互类型
    pub interaction_type: InteractionType,
    /// 通信渠道
    pub channel: CommunicationChannel,
    /// 方向 (inbound/outbound)
    pub direction: InteractionDirection,
    /// 摘要
    pub summary: String,
    /// 详细内容
    pub content: Option<String>,
    /// 关联联系人 ID
    pub contact_id: Option<String>,
    /// 关联实体 ID (订单/商机/工单等)
    pub entity_id: Option<String>,
    /// 执行人 ID
    pub operator_id: String,
    /// 时间戳 (epoch seconds)
    pub timestamp: u64,
    /// 附件列表
    pub attachments: Vec<Attachment>,
    /// 标签
    pub tags: Vec<String>,
}

/// 交互方向
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionDirection {
    /// 入站 (客户发起)
    Inbound,
    /// 出站 (我方发起)
    Outbound,
    /// 双向 (对话)
    Bidirectional,
    /// 系统自动
    System,
    /// 其他 (自定义方向)
    Other(String),
}

impl std::fmt::Display for InteractionDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inbound => write!(f, "inbound"),
            Self::Outbound => write!(f, "outbound"),
            Self::Bidirectional => write!(f, "bidirectional"),
            Self::System => write!(f, "system"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

/// 附件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// 文件名
    pub filename: String,
    /// 文件类型
    pub content_type: String,
    /// 文件大小 (bytes)
    pub size: u64,
    /// 文件路径或URL
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_type_from_str() {
        assert_eq!(
            InteractionType::from_str("Email"),
            Some(InteractionType::Email)
        );
        assert_eq!(
            InteractionType::from_str("电话"),
            Some(InteractionType::Phone)
        );
        assert_eq!(
            InteractionType::from_str("WhatsApp"),
            Some(InteractionType::Other("WhatsApp".into()))
        );
    }

    #[test]
    fn test_interaction_type_to_channel() {
        assert_eq!(
            InteractionType::Email.to_channel(),
            CommunicationChannel::Email
        );
        assert_eq!(
            InteractionType::Phone.to_channel(),
            CommunicationChannel::Phone
        );
        assert_eq!(
            InteractionType::Meeting.to_channel(),
            CommunicationChannel::Meeting
        );
    }

    #[test]
    fn test_interaction_record() {
        let record = InteractionRecord {
            id: "123".into(),
            interaction_type: InteractionType::Email,
            channel: CommunicationChannel::Email,
            direction: InteractionDirection::Outbound,
            summary: "Sent follow-up email".into(),
            content: None,
            contact_id: Some("contact-1".into()),
            entity_id: None,
            operator_id: "user-1".into(),
            timestamp: 1234567890,
            attachments: vec![],
            tags: vec!["follow-up".into()],
        };

        assert_eq!(record.interaction_type, InteractionType::Email);
        assert_eq!(record.direction, InteractionDirection::Outbound);
    }
}
