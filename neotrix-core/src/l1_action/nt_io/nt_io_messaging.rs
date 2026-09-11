//! L1 CAT-1 通信 — 统一消息接口
//!
//! 实现统一架构: L1Capability + MessagingProvider trait
//! 类别: CapabilityCategory::Communication
//! 进化: C0→C1→C2→C3→C4→C5→C6
//!
//! Provider 可插拔: WhatsApp / Email / SMS / Telegram / WeChat / Slack
//! 模板引擎: 渲染 {{variable}} 占位符
//! 会话管理: 按渠道+联系人聚合消息

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::l1_action::traits::{
    L1Capability, MessagingProvider, CapabilityCategory, ConstellationLevel,
    CapabilityHealth, CapabilityStats, CapabilityError,
    Message, MessageStatus,
};

// ════════════════════════════════════════════════════════════════
// 类型定义 (types.rs 模式)
// ════════════════════════════════════════════════════════════════

/// 消息通道类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    WhatsApp,
    Email,
    Sms,
    Telegram,
    WeChat,
    Slack,
}

/// 消息方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

/// 附件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub mime_type: String,
    pub data: Vec<u8>,
    pub size_bytes: usize,
}

/// 扩展消息 (继承 traits::Message 的基础字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedMessage {
    pub base: Message,
    pub direction: MessageDirection,
    pub subject: Option<String>,
    pub attachments: Vec<Attachment>,
    pub template_id: Option<String>,
    pub template_vars: HashMap<String, String>,
    pub reply_to: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// 会话线程
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub channel: Channel,
    pub participants: Vec<String>,
    pub contact_id: Option<String>,
    pub lead_id: Option<String>,
    pub messages: Vec<ExtendedMessage>,
    pub status: ConversationStatus,
    pub context: HashMap<String, String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationStatus {
    Active,
    WaitingReply,
    Closed,
    Spam,
}

// ════════════════════════════════════════════════════════════════
// 模板引擎
// ════════════════════════════════════════════════════════════════

/// 消息模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTemplate {
    pub id: String,
    pub name: String,
    pub channel: Channel,
    pub language: String,
    pub subject: Option<String>,
    pub body: String,
    pub variables: Vec<TemplateVariable>,
    pub category: TemplateCategory,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub var_type: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateCategory {
    Greeting,
    FollowUp,
    Proposal,
    Reminder,
    ThankYou,
    Complaint,
    Custom,
}

impl MessageTemplate {
    pub fn render(&self, vars: &HashMap<String, String>) -> Result<String, String> {
        let mut result = self.body.clone();
        for var in &self.variables {
            let value = vars.get(&var.name)
                .or(var.default.as_ref())
                .filter(|_| var.required)
                .ok_or_else(|| format!("Missing required variable: {}", var.name))?;
            result = result.replace(&format!("{{{{{}}}}}", var.name), value);
        }
        Ok(result)
    }
}

// ════════════════════════════════════════════════════════════════
// Provider 实现 — 统一 trait 实现
// ════════════════════════════════════════════════════════════════

/// WhatsApp Business API Provider
pub struct WhatsAppProvider {
    id: String,
    pub api_url: String,
    pub access_token: String,
    pub phone_number_id: String,
    pub business_account_id: String,
    pub templates: HashMap<String, MessageTemplate>,
    stats: CapabilityStats,
}

impl WhatsAppProvider {
    pub fn new(api_url: &str, access_token: &str, phone_number_id: &str, business_account_id: &str) -> Self {
        Self {
            id: format!("messaging.whatsapp.{}", phone_number_id),
            api_url: api_url.to_string(),
            access_token: access_token.to_string(),
            phone_number_id: phone_number_id.to_string(),
            business_account_id: business_account_id.to_string(),
            templates: HashMap::new(),
            stats: CapabilityStats::default(),
        }
    }

    pub fn register_template(&mut self, template: MessageTemplate) {
        self.templates.insert(template.id.clone(), template);
    }
}

impl L1Capability for WhatsAppProvider {
    fn capability_id(&self) -> &str { &self.id }
    fn category(&self) -> CapabilityCategory { CapabilityCategory::Communication }
    fn constellation(&self) -> ConstellationLevel { ConstellationLevel::C1UnitTest }
    fn health_check(&self) -> CapabilityHealth {
        CapabilityHealth {
            healthy: !self.access_token.is_empty(),
            latency_ms: None,
            error_rate: 0.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            message: None,
        }
    }
    fn description(&self) -> &str { "WhatsApp Business API messaging provider" }
    fn stats(&self) -> CapabilityStats { self.stats.clone() }
}

impl MessagingProvider for WhatsAppProvider {
    fn send(&self, _msg: &Message) -> Result<String, CapabilityError> {
        let msg_id = format!("wa_{}", uuid::Uuid::new_v4());
        // 实际实现: POST {api_url}/{phone_number_id}/messages
        Ok(msg_id)
    }

    fn receive(&self, _since: Option<u64>) -> Result<Vec<Message>, CapabilityError> {
        Ok(Vec::new())
    }

    fn get_status(&self, _id: &str) -> Result<MessageStatus, CapabilityError> {
        Ok(MessageStatus::Sent)
    }
}

/// Email Provider (SMTP)
pub struct EmailProvider {
    id: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub from_name: String,
    pub templates: HashMap<String, MessageTemplate>,
    stats: CapabilityStats,
}

impl EmailProvider {
    pub fn new(smtp_host: &str, smtp_port: u16, username: &str, password: &str, from: &str) -> Self {
        Self {
            id: format!("messaging.email.{}", smtp_host),
            smtp_host: smtp_host.to_string(),
            smtp_port,
            username: username.to_string(),
            password: password.to_string(),
            from_address: from.to_string(),
            from_name: String::new(),
            templates: HashMap::new(),
            stats: CapabilityStats::default(),
        }
    }
}

impl L1Capability for EmailProvider {
    fn capability_id(&self) -> &str { &self.id }
    fn category(&self) -> CapabilityCategory { CapabilityCategory::Communication }
    fn constellation(&self) -> ConstellationLevel { ConstellationLevel::C1UnitTest }
    fn health_check(&self) -> CapabilityHealth {
        CapabilityHealth {
            healthy: !self.smtp_host.is_empty(),
            latency_ms: None,
            error_rate: 0.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            message: None,
        }
    }
    fn description(&self) -> &str { "SMTP email provider" }
    fn stats(&self) -> CapabilityStats { self.stats.clone() }
}

impl MessagingProvider for EmailProvider {
    fn send(&self, _msg: &Message) -> Result<String, CapabilityError> {
        let msg_id = format!("email_{}", uuid::Uuid::new_v4());
        // 实际实现: SMTP send
        Ok(msg_id)
    }

    fn receive(&self, _since: Option<u64>) -> Result<Vec<Message>, CapabilityError> {
        Ok(Vec::new())
    }

    fn get_status(&self, _id: &str) -> Result<MessageStatus, CapabilityError> {
        Ok(MessageStatus::Sent)
    }
}

// ════════════════════════════════════════════════════════════════
// Registry — 能力注册中心
// ════════════════════════════════════════════════════════════════

/// 消息能力注册中心
pub struct MessagingRegistry {
    providers: Vec<Box<dyn MessagingProvider>>,
    #[allow(dead_code)]
    by_channel: HashMap<Channel, Vec<usize>>,
    templates: HashMap<String, MessageTemplate>,
}

impl Default for MessagingRegistry {
    fn default() -> Self { Self::new() }
}

impl MessagingRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            by_channel: HashMap::new(),
            templates: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn MessagingProvider>) {
        // 注册时无法直接获取 channel，通过 capability_id 推断
        let _idx = self.providers.len();
        self.providers.push(provider);
        // by_channel 在 route 时动态填充
    }

    pub fn register_template(&mut self, template: MessageTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    pub fn get(&self, id: &str) -> Option<&dyn MessagingProvider> {
        self.providers.iter().find(|p| p.capability_id() == id).map(|p| p.as_ref())
    }

    pub fn health_check_all(&self) -> Vec<(String, CapabilityHealth)> {
        self.providers.iter()
            .map(|p| (p.capability_id().to_string(), p.health_check()))
            .collect()
    }

    pub fn optimal(&self) -> Option<&dyn MessagingProvider> {
        self.providers.iter()
            .filter(|p| p.health_check().healthy)
            .max_by(|a, b| {
                let a_score = 1.0 - a.health_check().error_rate;
                let b_score = 1.0 - b.health_check().error_rate;
                a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.as_ref())
    }

    pub fn templates(&self) -> &HashMap<String, MessageTemplate> {
        &self.templates
    }
}

// ════════════════════════════════════════════════════════════════
// Router — 智能路由
// ════════════════════════════════════════════════════════════════

/// 消息路由器 — 按渠道选择最佳 Provider
pub struct MessagingRouter {
    registry: MessagingRegistry,
}

impl MessagingRouter {
    pub fn new(registry: MessagingRegistry) -> Self {
        Self { registry }
    }

    /// 路由到最佳 Provider
    pub fn route(&self, _channel: Channel) -> Option<&dyn MessagingProvider> {
        // 优先按渠道匹配，fallback 到 optimal
        self.registry.optimal()
    }

    /// 发送消息
    pub fn send(&self, msg: &Message) -> Result<String, CapabilityError> {
        let provider = self.registry.optimal()
            .ok_or_else(|| CapabilityError::NotAvailable("No messaging provider".into()))?;
        provider.send(msg)
    }

    /// 获取模板
    pub fn template(&self, id: &str) -> Option<&MessageTemplate> {
        self.registry.templates().get(id)
    }

    /// 发送模板消息
    pub fn send_template(
        &self,
        channel: Channel,
        template_id: &str,
        to: &str,
        vars: &HashMap<String, String>,
    ) -> Result<String, CapabilityError> {
        let template = self.template(template_id)
            .ok_or_else(|| CapabilityError::NotAvailable(format!("Template {} not found", template_id)))?;
        let body = template.render(vars).map_err(|e| CapabilityError::InvalidInput(e))?;
        let msg = Message {
            id: uuid::Uuid::new_v4().to_string(),
            channel: format!("{:?}", channel),
            from: String::new(),
            to: to.to_string(),
            body,
            status: MessageStatus::Queued,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        self.send(&msg)
    }
}

// ════════════════════════════════════════════════════════════════
// Bridge — L1↔L5 桥接
// ════════════════════════════════════════════════════════════════

/// 消息能力桥接 — L5 编排器通过此调用 L1 消息能力
pub struct MessagingBridge {
    router: MessagingRouter,
}

impl MessagingBridge {
    pub fn new(router: MessagingRouter) -> Self {
        Self { router }
    }

    pub fn send(&self, msg: &Message) -> Result<String, CapabilityError> {
        self.router.send(msg)
    }

    pub fn send_template(
        &self,
        channel: Channel,
        template_id: &str,
        to: &str,
        vars: &HashMap<String, String>,
    ) -> Result<String, CapabilityError> {
        self.router.send_template(channel, template_id, to, vars)
    }
}

// ════════════════════════════════════════════════════════════════
// 预置外贸模板
// ════════════════════════════════════════════════════════════════

pub fn trade_templates() -> Vec<MessageTemplate> {
    vec![
        MessageTemplate {
            id: "whatsapp_greeting".into(),
            name: "WhatsApp 问候".into(),
            channel: Channel::WhatsApp,
            language: "en".into(),
            subject: None,
            body: "Hello {{name}}! I'm {{sender_name}} from {{company}}. I noticed you're interested in {{product_category}}. How can I help you today?".into(),
            variables: vec![
                TemplateVariable { name: "name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "sender_name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "company".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "product_category".into(), var_type: "string".into(), required: true, default: None },
            ],
            category: TemplateCategory::Greeting,
            tags: vec!["trade".into(), "whatsapp".into()],
        },
        MessageTemplate {
            id: "email_followup".into(),
            name: "邮件跟进".into(),
            channel: Channel::Email,
            language: "en".into(),
            subject: Some("Following up on {{product_name}} inquiry".into()),
            body: "Dear {{name}},\n\nThank you for your interest in {{product_name}}. I wanted to follow up on my previous message.\n\nWe can offer:\n- MOQ: {{moq}}\n- Lead time: {{lead_time}}\n- Price range: {{price_range}}\n\nPlease let me know if you'd like to proceed.\n\nBest regards,\n{{sender_name}}".into(),
            variables: vec![
                TemplateVariable { name: "name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "product_name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "moq".into(), var_type: "string".into(), required: false, default: Some("negotiable".into()) },
                TemplateVariable { name: "lead_time".into(), var_type: "string".into(), required: false, default: Some("15-20 days".into()) },
                TemplateVariable { name: "price_range".into(), var_type: "string".into(), required: false, default: None },
                TemplateVariable { name: "sender_name".into(), var_type: "string".into(), required: true, default: None },
            ],
            category: TemplateCategory::FollowUp,
            tags: vec!["trade".into(), "email".into()],
        },
        MessageTemplate {
            id: "email_proposal".into(),
            name: "报价邮件".into(),
            channel: Channel::Email,
            language: "en".into(),
            subject: Some("Quotation for {{product_name}} - {{company}}".into()),
            body: "Dear {{name}},\n\nPlease find below our quotation:\n\n{{quotation_table}}\n\nTerms:\n- Payment: {{payment_terms}}\n- Delivery: {{delivery_terms}}\n- Validity: {{validity_days}} days\n\nBest regards,\n{{sender_name}}".into(),
            variables: vec![
                TemplateVariable { name: "name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "product_name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "quotation_table".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "payment_terms".into(), var_type: "string".into(), required: false, default: Some("T/T 30% deposit, 70% before shipment".into()) },
                TemplateVariable { name: "delivery_terms".into(), var_type: "string".into(), required: false, default: Some("FOB Shanghai".into()) },
                TemplateVariable { name: "validity_days".into(), var_type: "string".into(), required: false, default: Some("15".into()) },
                TemplateVariable { name: "company".into(), var_type: "string".into(), required: false, default: None },
                TemplateVariable { name: "sender_name".into(), var_type: "string".into(), required: true, default: None },
            ],
            category: TemplateCategory::Proposal,
            tags: vec!["trade".into(), "email".into(), "quotation".into()],
        },
        MessageTemplate {
            id: "whatsapp_order_update".into(),
            name: "WhatsApp 订单更新".into(),
            channel: Channel::WhatsApp,
            language: "en".into(),
            subject: None,
            body: "Hi {{name}}! Your order {{order_id}} status update:\n\n📦 Status: {{status}}\n📅 ETA: {{eta}}\n\n{{details}}\n\nAny questions? Just reply here!".into(),
            variables: vec![
                TemplateVariable { name: "name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "order_id".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "status".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "eta".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "details".into(), var_type: "string".into(), required: false, default: None },
            ],
            category: TemplateCategory::Custom,
            tags: vec!["trade".into(), "whatsapp".into()],
        },
    ]
}

// ════════════════════════════════════════════════════════════════
// 测试
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whatsapp_provider_trait_impl() {
        let p = WhatsAppProvider::new("https://api.whatsapp.com", "token", "123", "biz");
        assert_eq!(p.category(), CapabilityCategory::Communication);
        assert_eq!(p.constellation(), ConstellationLevel::C1UnitTest);
        assert!(p.health_check().healthy);
        assert!(p.capability_id().starts_with("messaging.whatsapp"));
    }

    #[test]
    fn test_email_provider_trait_impl() {
        let p = EmailProvider::new("smtp.gmail.com", 587, "user", "pass", "user@gmail.com");
        assert_eq!(p.category(), CapabilityCategory::Communication);
        assert!(p.health_check().healthy);
    }

    #[test]
    fn test_registry_register_and_health() {
        let mut reg = MessagingRegistry::new();
        let p = WhatsAppProvider::new("https://api.whatsapp.com", "token", "123", "biz");
        reg.register(Box::new(p));
        assert_eq!(reg.health_check_all().len(), 1);
        assert!(reg.optimal().is_some());
    }

    #[test]
    fn test_template_render() {
        let template = trade_templates().into_iter().next().unwrap();
        let mut vars = HashMap::new();
        vars.insert("name".into(), "John".into());
        vars.insert("sender_name".into(), "Alice".into());
        vars.insert("company".into(), "ACME".into());
        vars.insert("product_category".into(), "Machinery".into());
        let result = template.render(&vars).unwrap();
        assert!(result.contains("John"));
        assert!(result.contains("ACME"));
    }

    #[test]
    fn test_template_missing_var() {
        let template = MessageTemplate {
            id: "test".into(),
            name: "Test".into(),
            channel: Channel::Email,
            language: "en".into(),
            subject: None,
            body: "Hello {{name}}.".into(),
            variables: vec![TemplateVariable { name: "name".into(), var_type: "string".into(), required: true, default: None }],
            category: TemplateCategory::Custom,
            tags: vec![],
        };
        assert!(template.render(&HashMap::new()).is_err());
    }

    #[test]
    fn test_trade_templates_count() {
        assert!(trade_templates().len() >= 4);
    }
}
