//! L1 Unified Messaging — WhatsApp + Email + SMS 统一接口
//!
//! 通用能力: 任何域(外贸/客服/营销/内部协作)都可调用
//! 设计原则: Provider trait 抽象 → 多后端可插拔, 模板引擎 + 附件 + 会话状态

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════
// 消息通道抽象
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

/// 消息状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Draft,
    Queued,
    Sending,
    Sent,
    Delivered,
    Read,
    Failed,
    Bounced,
}

/// 附件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub mime_type: String,
    pub data: Vec<u8>,
    pub size_bytes: usize,
}

/// 统一消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel: Channel,
    pub direction: MessageDirection,
    pub from: String,
    pub to: String,
    pub subject: Option<String>,
    pub body: String,
    pub attachments: Vec<Attachment>,
    pub template_id: Option<String>,
    pub template_vars: HashMap<String, String>,
    pub status: MessageStatus,
    pub timestamp: u64,
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
    pub messages: Vec<Message>,
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
    /// 渲染模板: 将 {{variable}} 替换为实际值
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
// Provider trait — 多后端可插拔
// ════════════════════════════════════════════════════════════════

/// 消息发送 Provider trait
pub trait MessageProvider: Send + Sync {
    fn channel(&self) -> Channel;
    fn provider_name(&self) -> &str;
    fn is_available(&self) -> bool;

    /// 发送消息
    fn send(&self, message: &Message) -> Result<String, String>;

    /// 获取消息状态
    fn get_status(&self, message_id: &str) -> Result<MessageStatus, String>;

    /// 接收新消息 (轮询或 webhook 回调)
    fn receive(&self, since: Option<u64>) -> Result<Vec<Message>, String>;

    /// 获取会话历史
    fn get_conversation(&self, conversation_id: &str) -> Result<Option<Conversation>, String>;

    /// 发送模板消息
    fn send_template(
        &self,
        template: &MessageTemplate,
        to: &str,
        vars: &HashMap<String, String>,
    ) -> Result<String, String> {
        let body = template.render(vars)?;
        let msg = Message {
            id: uuid::Uuid::new_v4().to_string(),
            channel: self.channel(),
            direction: MessageDirection::Outbound,
            from: String::new(),
            to: to.to_string(),
            subject: template.subject.clone(),
            body,
            attachments: Vec::new(),
            template_id: Some(template.id.clone()),
            template_vars: vars.clone(),
            status: MessageStatus::Queued,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            reply_to: None,
            metadata: HashMap::new(),
        };
        self.send(&msg)
    }
}

// ════════════════════════════════════════════════════════════════
// WhatsApp Provider (抽象层)
// ════════════════════════════════════════════════════════════════

/// WhatsApp Business API Provider
pub struct WhatsAppProvider {
    pub api_url: String,
    pub access_token: String,
    pub phone_number_id: String,
    pub business_account_id: String,
    pub templates: HashMap<String, MessageTemplate>,
}

impl WhatsAppProvider {
    pub fn new(api_url: &str, access_token: &str, phone_number_id: &str, business_account_id: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            access_token: access_token.to_string(),
            phone_number_id: phone_number_id.to_string(),
            business_account_id: business_account_id.to_string(),
            templates: HashMap::new(),
        }
    }

    pub fn register_template(&mut self, template: MessageTemplate) {
        self.templates.insert(template.id.clone(), template);
    }
}

impl MessageProvider for WhatsAppProvider {
    fn channel(&self) -> Channel { Channel::WhatsApp }
    fn provider_name(&self) -> &str { "whatsapp_business" }
    fn is_available(&self) -> bool { !self.access_token.is_empty() }

    fn send(&self, message: &Message) -> Result<String, String> {
        // 实际实现调用 WhatsApp Business API
        // POST {api_url}/{phone_number_id}/messages
        let msg_id = format!("wa_{}", uuid::Uuid::new_v4());
        Ok(msg_id)
    }

    fn get_status(&self, _message_id: &str) -> Result<MessageStatus, String> {
        Ok(MessageStatus::Sent)
    }

    fn receive(&self, _since: Option<u64>) -> Result<Vec<Message>, String> {
        Ok(Vec::new())
    }

    fn get_conversation(&self, _conversation_id: &str) -> Result<Option<Conversation>, String> {
        Ok(None)
    }
}

// ════════════════════════════════════════════════════════════════
// Email Provider (抽象层)
// ════════════════════════════════════════════════════════════════

/// Email Provider (SMTP / API)
pub struct EmailProvider {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub from_name: String,
    pub templates: HashMap<String, MessageTemplate>,
}

impl EmailProvider {
    pub fn new(smtp_host: &str, smtp_port: u16, username: &str, password: &str, from: &str) -> Self {
        Self {
            smtp_host: smtp_host.to_string(),
            smtp_port,
            username: username.to_string(),
            password: password.to_string(),
            from_address: from.to_string(),
            from_name: String::new(),
            templates: HashMap::new(),
        }
    }
}

impl MessageProvider for EmailProvider {
    fn channel(&self) -> Channel { Channel::Email }
    fn provider_name(&self) -> &str { "smtp" }
    fn is_available(&self) -> bool { !self.smtp_host.is_empty() }

    fn send(&self, message: &Message) -> Result<String, String> {
        let msg_id = format!("email_{}", uuid::Uuid::new_v4());
        Ok(msg_id)
    }

    fn get_status(&self, _message_id: &str) -> Result<MessageStatus, String> {
        Ok(MessageStatus::Sent)
    }

    fn receive(&self, _since: Option<u64>) -> Result<Vec<Message>, String> {
        Ok(Vec::new())
    }

    fn get_conversation(&self, _conversation_id: &str) -> Result<Option<Conversation>, String> {
        Ok(None)
    }
}

// ════════════════════════════════════════════════════════════════
// 统一消息总线
// ════════════════════════════════════════════════════════════════

/// 统一消息管理器 — 路由、模板、会话管理
pub struct MessagingBus {
    providers: HashMap<Channel, Box<dyn MessageProvider>>,
    templates: HashMap<String, MessageTemplate>,
    conversations: HashMap<String, Conversation>,
    inbound_queue: Vec<Message>,
}

impl Default for MessagingBus {
    fn default() -> Self {
        Self::new()
    }
}

impl MessagingBus {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            templates: HashMap::new(),
            conversations: HashMap::new(),
            inbound_queue: Vec::new(),
        }
    }

    /// 注册消息 Provider
    pub fn register_provider(&mut self, provider: Box<dyn MessageProvider>) {
        let channel = provider.channel();
        self.providers.insert(channel, provider);
    }

    /// 注册消息模板
    pub fn register_template(&mut self, template: MessageTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// 发送消息 (自动路由到正确的 Provider)
    pub fn send(&self, message: &Message) -> Result<String, String> {
        let provider = self.providers.get(&message.channel)
            .ok_or_else(|| format!("No provider for channel {:?}", message.channel))?;
        provider.send(message)
    }

    /// 发送模板消息
    pub fn send_template(
        &self,
        channel: Channel,
        template_id: &str,
        to: &str,
        vars: &HashMap<String, String>,
    ) -> Result<String, String> {
        let template = self.templates.get(template_id)
            .ok_or_else(|| format!("Template {} not found", template_id))?;
        let provider = self.providers.get(&channel)
            .ok_or_else(|| format!("No provider for channel {:?}", channel))?;
        provider.send_template(template, to, vars)
    }

    /// 批量发送 (不同渠道)
    pub fn broadcast(
        &self,
        recipients: &[(Channel, &str)], // (channel, address)
        template_id: &str,
        vars: &HashMap<String, String>,
    ) -> Vec<Result<String, String>> {
        recipients.iter().map(|(channel, addr)| {
            self.send_template(*channel, template_id, addr, vars)
        }).collect()
    }

    /// 拉取所有渠道的新消息
    pub fn poll_all(&mut self, since: Option<u64>) -> Vec<Message> {
        let mut all = Vec::new();
        for provider in self.providers.values() {
            if let Ok(msgs) = provider.receive(since) {
                all.extend(msgs);
            }
        }
        all
    }

    /// 获取或创建会话
    pub fn get_or_create_conversation(
        &mut self,
        channel: Channel,
        participant: &str,
    ) -> &Conversation {
        let conv_id = format!("{:?}_{}", channel, participant);
        self.conversations.entry(conv_id.clone()).or_insert_with(|| {
            Conversation {
                id: conv_id,
                channel,
                participants: vec![participant.to_string()],
                contact_id: None,
                lead_id: None,
                messages: Vec::new(),
                status: ConversationStatus::Active,
                context: HashMap::new(),
                created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                updated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            }
        });
        self.conversations.get(&conv_id).unwrap()
    }

    /// 列出所有模板
    pub fn list_templates(&self) -> Vec<&MessageTemplate> {
        self.templates.values().collect()
    }

    /// 按渠道列出模板
    pub fn templates_by_channel(&self, channel: Channel) -> Vec<&MessageTemplate> {
        self.templates.values().filter(|t| t.channel == channel).collect()
    }
}

// ════════════════════════════════════════════════════════════════
// 预置外贸模板库
// ════════════════════════════════════════════════════════════════

/// 生成外贸常用消息模板
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
            body: "Dear {{name}},\n\nThank you for your interest in {{product_name}}. I wanted to follow up on my previous message regarding your inquiry.\n\nWe can offer:\n- MOQ: {{moq}}\n- Lead time: {{lead_time}}\n- Price range: {{price_range}}\n\nPlease let me know if you'd like to proceed with a sample order.\n\nBest regards,\n{{sender_name}}".into(),
            variables: vec![
                TemplateVariable { name: "name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "product_name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "moq".into(), var_type: "string".into(), required: false, default: Some(" negotiable".into()) },
                TemplateVariable { name: "lead_time".into(), var_type: "string".into(), required: false, default: Some("15-20 days".into()) },
                TemplateVariable { name: "price_range".into(), var_type: "string".into(), required: false, default: None },
                TemplateVariable { name: "sender_name".into(), var_type: "string".into(), required: true, default: None },
            ],
            category: TemplateCategory::FollowUp,
            tags: vec!["trade".into(), "email".into(), "followup".into()],
        },
        MessageTemplate {
            id: "email_proposal".into(),
            name: "报价邮件".into(),
            channel: Channel::Email,
            language: "en".into(),
            subject: Some("Quotation for {{product_name}} - {{company}}".into()),
            body: "Dear {{name}},\n\nPlease find below our quotation for {{product_name}}:\n\n{{quotation_table}}\n\nTerms:\n- Payment: {{payment_terms}}\n- Delivery: {{delivery_terms}}\n- Validity: {{validity_days}} days\n\nLooking forward to your reply.\n\nBest regards,\n{{sender_name}}".into(),
            variables: vec![
                TemplateVariable { name: "name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "product_name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "quotation_table".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "payment_terms".into(), var_type: "string".into(), required: false, default: Some("T/T 30% deposit, 70% before shipment".into()) },
                TemplateVariable { name: "delivery_terms".into(), var_type: "string".into(), required: false, default: Some("FOB Shanghai".into()) },
                TemplateVariable { name: "validity_days".into(), var_type: "string".into(), required: false, default: Some("15".into()) },
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
            tags: vec!["trade".into(), "whatsapp".into(), "order".into()],
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
    fn test_template_render() {
        let template = MessageTemplate {
            id: "test".into(),
            name: "Test".into(),
            channel: Channel::WhatsApp,
            language: "en".into(),
            subject: None,
            body: "Hello {{name}}, your order {{order_id}} is {{status}}.".into(),
            variables: vec![
                TemplateVariable { name: "name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "order_id".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "status".into(), var_type: "string".into(), required: true, default: None },
            ],
            category: TemplateCategory::Custom,
            tags: vec![],
        };

        let mut vars = HashMap::new();
        vars.insert("name".into(), "John".into());
        vars.insert("order_id".into(), "ORD-001".into());
        vars.insert("status".into(), "shipped".into());

        let result = template.render(&vars).unwrap();
        assert_eq!(result, "Hello John, your order ORD-001 is shipped.");
    }

    #[test]
    fn test_template_missing_required_var() {
        let template = MessageTemplate {
            id: "test".into(),
            name: "Test".into(),
            channel: Channel::Email,
            language: "en".into(),
            subject: None,
            body: "Hello {{name}}.".into(),
            variables: vec![
                TemplateVariable { name: "name".into(), var_type: "string".into(), required: true, default: None },
            ],
            category: TemplateCategory::Custom,
            tags: vec![],
        };

        let vars = HashMap::new();
        assert!(template.render(&vars).is_err());
    }

    #[test]
    fn test_template_default_value() {
        let template = MessageTemplate {
            id: "test".into(),
            name: "Test".into(),
            channel: Channel::Email,
            language: "en".into(),
            subject: None,
            body: "Hello {{name}}, your order will arrive in {{eta}}.".into(),
            variables: vec![
                TemplateVariable { name: "name".into(), var_type: "string".into(), required: true, default: None },
                TemplateVariable { name: "eta".into(), var_type: "string".into(), required: false, default: Some("15 days".into()) },
            ],
            category: TemplateCategory::Custom,
            tags: vec![],
        };

        let mut vars = HashMap::new();
        vars.insert("name".into(), "John".into());
        let result = template.render(&vars).unwrap();
        assert!(result.contains("15 days"));
    }

    #[test]
    fn test_messaging_bus_send() {
        let mut bus = MessagingBus::new();
        let provider = WhatsAppProvider::new("https://api.whatsapp.com", "token", "123", "biz");
        bus.register_provider(Box::new(provider));

        let template = trade_templates().into_iter().next().unwrap();
        bus.register_template(template);

        let mut vars = HashMap::new();
        vars.insert("name".into(), "John".into());
        vars.insert("sender_name".into(), "Alice".into());
        vars.insert("company".into(), "ACME".into());
        vars.insert("product_category".into(), "Machinery".into());

        let result = bus.send_template(Channel::WhatsApp, "whatsapp_greeting", "+1234567890", &vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trade_templates_count() {
        let templates = trade_templates();
        assert!(templates.len() >= 4, "Should have at least 4 trade templates");
    }
}
