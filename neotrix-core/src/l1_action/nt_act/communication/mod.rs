//! 通用通信能力模块
//!
//! 提供跨域复用的通信渠道、交互类型、联系人信息等基础类型。
//! 外贸Agent、客服Agent、社交Agent等均可复用此模块。

#![forbid(unsafe_code)]

pub mod channel;
pub mod contact;
pub mod interaction;
pub mod social_media;

pub use channel::CommunicationChannel;
pub use contact::{ContactInfo, ContactMethod};
pub use interaction::{Attachment, InteractionDirection, InteractionRecord, InteractionType};
pub use social_media::{SocialMediaCategory, SocialMediaPlatform};
