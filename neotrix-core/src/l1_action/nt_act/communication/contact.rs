//! 联系人信息
//!
//! 定义通用的联系人信息结构，支持所有需要联系人管理的Agent。

#![forbid(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::social_media::SocialMediaPlatform;

/// 联系人信息 — 跨域通用
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContactInfo {
    /// 电话号码
    pub phone: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 社交媒体账号 (平台 → 账号)
    pub social: HashMap<SocialMediaPlatform, String>,
    /// 地址
    pub address: Option<String>,
    /// 时区
    pub timezone: Option<String>,
    /// 语言偏好
    pub language: Option<String>,
}

impl ContactInfo {
    /// 创建空联系人
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置电话
    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    /// 设置邮箱
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// 添加社交媒体账号
    pub fn with_social(mut self, platform: SocialMediaPlatform, account: impl Into<String>) -> Self {
        self.social.insert(platform, account.into());
        self
    }

    /// 获取社交媒体账号
    pub fn get_social(&self, platform: &SocialMediaPlatform) -> Option<&str> {
        self.social.get(platform).map(|s| s.as_str())
    }

    /// 是否有联系方式
    pub fn has_contact(&self) -> bool {
        self.phone.is_some() || self.email.is_some() || !self.social.is_empty()
    }

    /// 获取主要联系方式
    pub fn primary_contact(&self) -> Option<ContactMethod> {
        if self.email.is_some() {
            Some(ContactMethod::Email)
        } else if self.phone.is_some() {
            Some(ContactMethod::Phone)
        } else if let Some((platform, _)) = self.social.iter().next() {
            Some(ContactMethod::Social(platform.clone()))
        } else {
            None
        }
    }
}

/// 联系方式类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContactMethod {
    Email,
    Phone,
    Social(SocialMediaPlatform),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_info() {
        let contact = ContactInfo::new()
            .with_phone("+1234567890")
            .with_email("test@example.com")
            .with_social(SocialMediaPlatform::WhatsApp, "+1234567890");

        assert!(contact.has_contact());
        assert_eq!(contact.phone, Some("+1234567890".into()));
        assert_eq!(contact.email, Some("test@example.com".into()));
        assert_eq!(
            contact.get_social(&SocialMediaPlatform::WhatsApp),
            Some("+1234567890")
        );
    }

    #[test]
    fn test_primary_contact() {
        let contact1 = ContactInfo::new().with_email("test@example.com");
        assert_eq!(contact1.primary_contact(), Some(ContactMethod::Email));

        let contact2 = ContactInfo::new().with_phone("+1234567890");
        assert_eq!(contact2.primary_contact(), Some(ContactMethod::Phone));

        let contact3 = ContactInfo::new()
            .with_social(SocialMediaPlatform::LinkedIn, "profile");
        assert_eq!(
            contact3.primary_contact(),
            Some(ContactMethod::Social(SocialMediaPlatform::LinkedIn))
        );
    }
}
