//! Trade Email Integration — 外贸邮件集成模块
//!
//! 对标 TMS 平台的邮件能力：
//! - 邮件模板管理 (跟进/报价/催款/节日问候)
//! - 邮件追踪 (打开/点击/回复检测)
//! - 邮件-客户自动关联
//! - AI 辅助回复草稿生成
//! - 批量发送 + 频率控制

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 1. 邮件模板
// ============================================================

/// 邮件模板类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmailTemplateType {
    /// 首次跟进
    FirstFollowUp,
    /// 报价发送
    QuotationSend,
    /// 报价跟进
    QuotationFollowUp,
    /// 催款提醒
    PaymentReminder,
    /// 发货通知
    ShipmentNotification,
    /// 节日问候
    HolidayGreeting,
    /// 样品寄送
    SampleShipment,
    /// 售后回访
    AfterSales,
    /// 自定义
    Custom(String),
}

/// 邮件模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: String,
    pub name: String,
    pub template_type: EmailTemplateType,
    pub subject: String,
    pub body_html: String,
    pub body_text: String,
    /// 占位符变量 (如 {{customer_name}}, {{product_name}})
    pub variables: Vec<String>,
    /// 语言 (en, zh, es, etc.)
    pub language: String,
    pub created_at: u64,
    pub updated_at: u64,
}

// ============================================================
// 2. 邮件记录
// ============================================================

/// 邮件发送状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmailStatus {
    /// 草稿
    Draft,
    /// 已发送
    Sent,
    /// 已送达
    Delivered,
    /// 已打开
    Opened,
    /// 已点击链接
    Clicked,
    /// 已回复
    Replied,
    /// 发送失败
    Failed,
    /// 退回
    Bounced,
}

/// 邮件记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRecord {
    pub id: String,
    /// 关联客户 ID
    pub customer_id: Option<String>,
    /// 关联联系人 ID
    pub contact_id: Option<String>,
    /// 收件人邮箱
    pub to: Vec<String>,
    /// 抄送
    pub cc: Vec<String>,
    /// 密送
    pub bcc: Vec<String>,
    /// 发件人
    pub from: String,
    /// 主题
    pub subject: String,
    /// 正文 (HTML)
    pub body_html: String,
    /// 正文 (纯文本)
    pub body_text: String,
    /// 使用的模板 ID
    pub template_id: Option<String>,
    /// 状态
    pub status: EmailStatus,
    /// 追踪数据
    pub tracking: EmailTracking,
    /// 附件列表
    pub attachments: Vec<EmailAttachment>,
    /// 发送时间
    pub sent_at: Option<u64>,
    /// 创建时间
    pub created_at: u64,
}

/// 邮件追踪
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmailTracking {
    /// 打开次数
    pub open_count: u32,
    /// 首次打开时间
    pub first_opened_at: Option<u64>,
    /// 最后打开时间
    pub last_opened_at: Option<u64>,
    /// 点击次数
    pub click_count: u32,
    /// 点击的链接
    pub clicked_links: Vec<String>,
    /// 是否已回复
    pub replied: bool,
    /// 回复时间
    pub replied_at: Option<u64>,
}

/// 邮件附件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub filename: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub path: String,
}

// ============================================================
// 3. 发送配置
// ============================================================

/// 批量发送配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkSendConfig {
    /// 每日发送上限
    pub daily_limit: u32,
    /// 每小时发送上限
    pub hourly_limit: u32,
    /// 发送间隔 (秒)
    pub interval_seconds: u32,
    /// 随机延迟范围 (秒)
    pub random_delay_range: (u32, u32),
    /// 时区
    pub timezone: String,
    /// 静默时段 (如 22:00-08:00 不发)
    pub quiet_hours: Option<(u32, u32)>,
}

impl Default for BulkSendConfig {
    fn default() -> Self {
        Self {
            daily_limit: 100,
            hourly_limit: 20,
            interval_seconds: 30,
            random_delay_range: (5, 15),
            timezone: "UTC".into(),
            quiet_hours: Some((22, 8)),
        }
    }
}

// ============================================================
// 4. 邮件引擎
// ============================================================

/// 邮件集成引擎
pub struct TradeEmailEngine {
    /// 邮件模板库
    templates: HashMap<String, EmailTemplate>,
    /// 邮件记录
    records: Vec<EmailRecord>,
    /// 发送配置
    config: BulkSendConfig,
    /// 每日发送计数
    daily_sent: u32,
    /// 最后发送时间
    last_send_at: Option<u64>,
}

impl Default for TradeEmailEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeEmailEngine {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            records: Vec::new(),
            config: BulkSendConfig::default(),
            daily_sent: 0,
            last_send_at: None,
        }
    }

    /// 创建邮件模板
    pub fn create_template(&mut self, template: EmailTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// 获取模板
    pub fn get_template(&self, template_id: &str) -> Option<&EmailTemplate> {
        self.templates.get(template_id)
    }

    /// 渲染模板 (替换占位符)
    pub fn render_template(
        &self,
        template_id: &str,
        variables: &HashMap<String, String>,
    ) -> Result<(String, String), String> {
        let template = self
            .templates
            .get(template_id)
            .ok_or_else(|| format!("Template {} not found", template_id))?;
        let mut subject = template.subject.clone();
        let mut body = template.body_html.clone();
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            subject = subject.replace(&placeholder, value);
            body = body.replace(&placeholder, value);
        }
        Ok((subject, body))
    }

    /// 创建邮件草稿
    pub fn create_draft(
        &mut self,
        to: Vec<String>,
        subject: String,
        body_html: String,
        body_text: String,
        customer_id: Option<String>,
        contact_id: Option<String>,
    ) -> EmailRecord {
        let now = self.current_timestamp();
        let record = EmailRecord {
            id: uuid::Uuid::new_v4().to_string(),
            customer_id,
            contact_id,
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            from: String::new(),
            subject,
            body_html,
            body_text,
            template_id: None,
            status: EmailStatus::Draft,
            tracking: EmailTracking::default(),
            attachments: Vec::new(),
            sent_at: None,
            created_at: now,
        };
        self.records.push(record.clone());
        record
    }

    /// 检查是否可以发送 (频率控制)
    pub fn can_send(&self) -> bool {
        if self.daily_sent >= self.config.daily_limit {
            return false;
        }
        if let Some(last) = self.last_send_at {
            let now = self.current_timestamp();
            if now - last < self.config.interval_seconds as u64 {
                return false;
            }
        }
        true
    }

    /// 记录邮件已发送
    pub fn mark_sent(&mut self, email_id: &str) -> Result<(), String> {
        let now = self.current_timestamp();
        let record = self
            .records
            .iter_mut()
            .find(|r| r.id == email_id)
            .ok_or_else(|| format!("Email {} not found", email_id))?;
        record.status = EmailStatus::Sent;
        record.sent_at = Some(now);
        self.daily_sent += 1;
        self.last_send_at = Some(now);
        Ok(())
    }

    /// 记录邮件打开
    pub fn track_open(&mut self, email_id: &str) -> Result<(), String> {
        let now = self.current_timestamp();
        let record = self
            .records
            .iter_mut()
            .find(|r| r.id == email_id)
            .ok_or_else(|| format!("Email {} not found", email_id))?;
        record.tracking.open_count += 1;
        if record.tracking.first_opened_at.is_none() {
            record.tracking.first_opened_at = Some(now);
        }
        record.tracking.last_opened_at = Some(now);
        if record.status == EmailStatus::Sent {
            record.status = EmailStatus::Opened;
        }
        Ok(())
    }

    /// 记录邮件回复
    pub fn track_reply(&mut self, email_id: &str) -> Result<(), String> {
        let now = self.current_timestamp();
        let record = self
            .records
            .iter_mut()
            .find(|r| r.id == email_id)
            .ok_or_else(|| format!("Email {} not found", email_id))?;
        record.tracking.replied = true;
        record.tracking.replied_at = Some(now);
        record.status = EmailStatus::Replied;
        Ok(())
    }

    /// 获取邮件统计
    pub fn summary(&self) -> EmailSummary {
        let total = self.records.len() as u64;
        let sent = self.records.iter().filter(|r| r.status != EmailStatus::Draft).count() as u64;
        let opened = self.records.iter().filter(|r| r.tracking.open_count > 0).count() as u64;
        let replied = self.records.iter().filter(|r| r.tracking.replied).count() as u64;
        EmailSummary {
            total_emails: total,
            sent,
            opened,
            replied,
            open_rate: if sent > 0 { opened as f64 / sent as f64 } else { 0.0 },
            reply_rate: if sent > 0 { replied as f64 / sent as f64 } else { 0.0 },
        }
    }

    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// 邮件统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSummary {
    pub total_emails: u64,
    pub sent: u64,
    pub opened: u64,
    pub replied: u64,
    pub open_rate: f64,
    pub reply_rate: f64,
}

// ============================================================
// 5. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_template() {
        let mut engine = TradeEmailEngine::new();
        let template = EmailTemplate {
            id: "T-001".into(),
            name: "First Follow Up".into(),
            template_type: EmailTemplateType::FirstFollowUp,
            subject: "Following up on our discussion, {{customer_name}}".into(),
            body_html: "<p>Dear {{customer_name}},</p><p>Regarding {{product_name}}...</p>".into(),
            body_text: "Dear {{customer_name}},\nRegarding {{product_name}}...".into(),
            variables: vec!["customer_name".into(), "product_name".into()],
            language: "en".into(),
            created_at: 0,
            updated_at: 0,
        };
        engine.create_template(template);
        assert!(engine.get_template("T-001").is_some());
    }

    #[test]
    fn test_render_template() {
        let mut engine = TradeEmailEngine::new();
        engine.create_template(EmailTemplate {
            id: "T-001".into(),
            name: "Test".into(),
            template_type: EmailTemplateType::FirstFollowUp,
            subject: "Hello {{name}}".into(),
            body_html: "<p>Hi {{name}}</p>".into(),
            body_text: "Hi {{name}}".into(),
            variables: vec!["name".into()],
            language: "en".into(),
            created_at: 0,
            updated_at: 0,
        });
        let mut vars = HashMap::new();
        vars.insert("name".into(), "John".into());
        let (subject, body) = engine.render_template("T-001", &vars).unwrap();
        assert_eq!(subject, "Hello John");
        assert_eq!(body, "<p>Hi John</p>");
    }

    #[test]
    fn test_send_tracking() {
        let mut engine = TradeEmailEngine::new();
        let email = engine.create_draft(
            vec!["test@example.com".into()],
            "Test".into(),
            "<p>Hi</p>".into(),
            "Hi".into(),
            None,
            None,
        );
        assert!(engine.can_send());
        engine.mark_sent(&email.id).unwrap();
        engine.track_open(&email.id).unwrap();
        engine.track_reply(&email.id).unwrap();
        let summary = engine.summary();
        assert_eq!(summary.total_emails, 1);
        assert_eq!(summary.replied, 1);
    }
}
