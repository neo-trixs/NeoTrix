//! Trade Event Bus — 外贸事件总线
//!
//! 提供发布/订阅模式的事件驱动架构，用于 trade 子系统间的解耦通信。
//! 支持同步与异步事件处理器，维护事件历史以供审计和回溯。

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================
// 1. 事件类型枚举 — EventType
// ============================================================

/// 事件类型分类，用于路由和过滤
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// 询盘事件
    Inquiry,
    /// 报价事件
    Quote,
    /// 订单事件
    Order,
    /// 生产事件
    Production,
    /// 物流事件
    Logistics,
    /// 财务事件
    Finance,
    /// 合规事件
    Compliance,
    /// 风险事件
    Risk,
    /// 系统事件
    System,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::Inquiry => write!(f, "Inquiry"),
            EventType::Quote => write!(f, "Quote"),
            EventType::Order => write!(f, "Order"),
            EventType::Production => write!(f, "Production"),
            EventType::Logistics => write!(f, "Logistics"),
            EventType::Finance => write!(f, "Finance"),
            EventType::Compliance => write!(f, "Compliance"),
            EventType::Risk => write!(f, "Risk"),
            EventType::System => write!(f, "System"),
        }
    }
}

// ============================================================
// 2. 外贸事件枚举 — TradeEvent
// ============================================================

/// 外贸业务事件，覆盖完整交易生命周期
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TradeEvent {
    /// 收到询盘
    InquiryReceived {
        buyer_id: String,
        product_ids: Vec<String>,
        message: String,
    },
    /// 生成报价
    QuoteGenerated {
        quote_id: String,
        buyer_id: String,
        total_amount: f64,
        currency: String,
    },
    /// 订单创建
    OrderCreated {
        order_id: String,
        buyer_id: String,
    },
    /// 订单确认
    OrderConfirmed {
        order_id: String,
        confirmed_by: String,
    },
    /// 生产启动
    ProductionStarted {
        order_id: String,
        production_line: String,
        estimated_completion: String,
    },
    /// 生产完成
    ProductionCompleted {
        order_id: String,
        actual_completion: String,
        quality_grade: String,
    },
    /// 质检结果
    InspectionResult {
        order_id: String,
        passed: bool,
        defects: Vec<String>,
    },
    /// 物流发运
    ShipmentDispatched {
        order_id: String,
        tracking_number: String,
        carrier: String,
        destination: String,
    },
    /// 物流到达
    ShipmentArrived {
        order_id: String,
        tracking_number: String,
        arrival_date: String,
    },
    /// 付款收到
    PaymentReceived {
        order_id: String,
        amount: f64,
        currency: String,
        payment_method: String,
    },
    /// 付款确认
    PaymentConfirmed {
        order_id: String,
        confirmed_by: String,
    },
    /// 退税申请
    TaxRefundApplied {
        order_id: String,
        refund_amount: f64,
        applied_date: String,
    },
    /// 退税到账
    TaxRefundCompleted {
        order_id: String,
        refund_amount: f64,
        received_date: String,
    },
    /// 合规审查
    ComplianceReview {
        order_id: String,
        reviewer: String,
        result: ComplianceReviewResult,
        notes: String,
    },
    /// 风险预警
    RiskAlert {
        order_id: String,
        risk_type: String,
        severity: RiskSeverity,
        description: String,
    },
    /// 谈判轮次
    NegotiationRound {
        order_id: String,
        round: u32,
        concession_item: String,
        original_value: f64,
        conceded_value: f64,
    },
    /// 合同签署
    ContractSigned {
        contract_id: String,
        order_id: String,
        signed_by: String,
        signed_date: String,
    },
    /// 纠纷发生
    DisputeRaised {
        order_id: String,
        dispute_type: String,
        description: String,
        raised_by: String,
    },
    /// 纠纷解决
    DisputeResolved {
        order_id: String,
        resolution: String,
        resolved_date: String,
    },
    /// 通用自定义事件
    Custom {
        event_name: String,
        data: HashMap<String, String>,
    },
}

impl TradeEvent {
    /// 获取事件对应的类型分类
    pub fn event_type(&self) -> EventType {
        match self {
            TradeEvent::InquiryReceived { .. } => EventType::Inquiry,
            TradeEvent::QuoteGenerated { .. } => EventType::Quote,
            TradeEvent::OrderCreated { .. }
            | TradeEvent::OrderConfirmed { .. } => EventType::Order,
            TradeEvent::ProductionStarted { .. }
            | TradeEvent::ProductionCompleted { .. } => EventType::Production,
            TradeEvent::InspectionResult { .. } => EventType::Production,
            TradeEvent::ShipmentDispatched { .. }
            | TradeEvent::ShipmentArrived { .. } => EventType::Logistics,
            TradeEvent::PaymentReceived { .. }
            | TradeEvent::PaymentConfirmed { .. } => EventType::Finance,
            TradeEvent::TaxRefundApplied { .. }
            | TradeEvent::TaxRefundCompleted { .. } => EventType::Finance,
            TradeEvent::ComplianceReview { .. } => EventType::Compliance,
            TradeEvent::RiskAlert { .. } => EventType::Risk,
            TradeEvent::NegotiationRound { .. } => EventType::Quote,
            TradeEvent::ContractSigned { .. } => EventType::Order,
            TradeEvent::DisputeRaised { .. }
            | TradeEvent::DisputeResolved { .. } => EventType::Risk,
            TradeEvent::Custom { .. } => EventType::System,
        }
    }

    /// 获取事件关联的订单 ID（如果有）
    pub fn order_id(&self) -> Option<&str> {
        match self {
            TradeEvent::InquiryReceived { .. } => None,
            TradeEvent::QuoteGenerated { .. } => None,
            TradeEvent::OrderCreated { order_id, .. }
            | TradeEvent::OrderConfirmed { order_id, .. }
            | TradeEvent::ProductionStarted { order_id, .. }
            | TradeEvent::ProductionCompleted { order_id, .. }
            | TradeEvent::InspectionResult { order_id, .. }
            | TradeEvent::ShipmentDispatched { order_id, .. }
            | TradeEvent::ShipmentArrived { order_id, .. }
            | TradeEvent::PaymentReceived { order_id, .. }
            | TradeEvent::PaymentConfirmed { order_id, .. }
            | TradeEvent::TaxRefundApplied { order_id, .. }
            | TradeEvent::TaxRefundCompleted { order_id, .. }
            | TradeEvent::ComplianceReview { order_id, .. }
            | TradeEvent::RiskAlert { order_id, .. }
            | TradeEvent::NegotiationRound { order_id, .. }
            | TradeEvent::DisputeRaised { order_id, .. }
            | TradeEvent::DisputeResolved { order_id, .. } => Some(order_id),
            TradeEvent::ContractSigned { order_id, .. } => Some(order_id),
            TradeEvent::Custom { .. } => None,
        }
    }
}

/// 合规审查结果
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceReviewResult {
    Approved,
    Rejected,
    Pending,
}

/// 风险严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

// ============================================================
// 3. 事件负载 — EventPayload
// ============================================================

/// 事件负载，封装事件元数据和具体内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    /// 事件唯一 ID
    pub id: String,
    /// 事件类型分类
    pub event_type: EventType,
    /// 事件发生时间
    pub timestamp: DateTime<Utc>,
    /// 具体事件内容
    pub event: TradeEvent,
    /// 可选的上下文标签
    pub tags: Vec<String>,
}

impl EventPayload {
    /// 创建新的事件负载
    pub fn new(event: TradeEvent) -> Self {
        let event_type = event.event_type();
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            event,
            tags: Vec::new(),
        }
    }

    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// 添加多个标签
    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }
}

// ============================================================
// 4. 事件处理器 trait — TradeEventHandler / AsyncTradeEventHandler
// ============================================================

/// 同步外贸事件处理器 trait
pub trait TradeEventHandler: Send + Sync {
    /// 处理事件
    fn handle(&self, payload: &EventPayload);

    /// 处理器名称，用于日志和调试
    fn name(&self) -> &str;
}

/// 异步外贸事件处理器 trait
#[async_trait::async_trait]
pub trait AsyncTradeEventHandler: Send + Sync {
    /// 异步处理事件
    async fn handle(&self, payload: &EventPayload);

    /// 处理器名称
    fn name(&self) -> &str;
}

// ============================================================
// 5. 订阅者类型 — Subscriber
// ============================================================

/// 订阅者包装，区分同步/异步处理器
enum Subscriber {
    Sync(Box<dyn TradeEventHandler>),
    Async(Box<dyn AsyncTradeEventHandler>),
}

impl Subscriber {
    fn name(&self) -> &str {
        match self {
            Subscriber::Sync(h) => h.name(),
            Subscriber::Async(h) => h.name(),
        }
    }
}

// ============================================================
// 6. 事件总线 — TradeEventBus
// ============================================================

/// 订阅者 ID，用于取消订阅
pub type SubscriberId = Uuid;

/// 订阅条目
struct Subscription {
    id: SubscriberId,
    subscriber: Subscriber,
}

/// 外贸事件总线，支持同步/异步事件发布/订阅
pub struct TradeEventBus {
    /// 订阅者映射：EventType → 订阅者列表
    subscribers: RwLock<HashMap<EventType, Vec<Subscription>>>,
    /// 全局订阅者（监听所有事件）
    global_subscribers: RwLock<Vec<Subscription>>,
    /// 事件历史记录
    event_history: RwLock<Vec<EventPayload>>,
    /// 事件历史容量上限
    max_history: usize,
}

impl TradeEventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(HashMap::new()),
            global_subscribers: RwLock::new(Vec::new()),
            event_history: RwLock::new(Vec::new()),
            max_history: 1000,
        }
    }

    /// 创建指定历史容量的事件总线
    pub fn with_history_capacity(capacity: usize) -> Self {
        Self {
            subscribers: RwLock::new(HashMap::new()),
            global_subscribers: RwLock::new(Vec::new()),
            event_history: RwLock::new(Vec::new()),
            max_history: capacity,
        }
    }

    /// 发布事件（同步处理器）
    pub async fn publish(&self, event: TradeEvent) -> EventPayload {
        let payload = EventPayload::new(event);
        self.dispatch_sync(&payload).await;
        self.record_history(payload.clone()).await;
        payload
    }

    /// 发布带标签的事件
    pub async fn publish_tagged(&self, event: TradeEvent, tags: Vec<String>) -> EventPayload {
        let payload = EventPayload::new(event).with_tags(tags);
        self.dispatch_sync(&payload).await;
        self.record_history(payload.clone()).await;
        payload
    }

    /// 异步发布事件（同时触发异步处理器）
    pub async fn publish_async(&self, event: TradeEvent) -> EventPayload {
        let payload = EventPayload::new(event);
        self.dispatch_all(&payload).await;
        self.record_history(payload.clone()).await;
        payload
    }

    /// 订阅特定事件类型（同步处理器）
    pub async fn subscribe(
        &self,
        event_type: EventType,
        handler: Box<dyn TradeEventHandler>,
    ) -> SubscriberId {
        let id = Uuid::new_v4();
        let sub = Subscription {
            id,
            subscriber: Subscriber::Sync(handler),
        };
        let mut subs = self.subscribers.write().await;
        subs.entry(event_type).or_default().push(sub);
        id
    }

    /// 订阅特定事件类型（异步处理器）
    pub async fn subscribe_async(
        &self,
        event_type: EventType,
        handler: Box<dyn AsyncTradeEventHandler>,
    ) -> SubscriberId {
        let id = Uuid::new_v4();
        let sub = Subscription {
            id,
            subscriber: Subscriber::Async(handler),
        };
        let mut subs = self.subscribers.write().await;
        subs.entry(event_type).or_default().push(sub);
        id
    }

    /// 订阅所有事件类型（全局同步处理器）
    pub async fn subscribe_all(&self, handler: Box<dyn TradeEventHandler>) -> SubscriberId {
        let id = Uuid::new_v4();
        let sub = Subscription {
            id,
            subscriber: Subscriber::Sync(handler),
        };
        let mut globals = self.global_subscribers.write().await;
        globals.push(sub);
        id
    }

    /// 订阅所有事件类型（全局异步处理器）
    pub async fn subscribe_all_async(
        &self,
        handler: Box<dyn AsyncTradeEventHandler>,
    ) -> SubscriberId {
        let id = Uuid::new_v4();
        let sub = Subscription {
            id,
            subscriber: Subscriber::Async(handler),
        };
        let mut globals = self.global_subscribers.write().await;
        globals.push(sub);
        id
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, id: SubscriberId) -> bool {
        // 从类型订阅中移除
        let mut subs = self.subscribers.write().await;
        for list in subs.values_mut() {
            let before = list.len();
            list.retain(|s| s.id != id);
            if list.len() < before {
                return true;
            }
        }

        // 从全局订阅中移除
        let mut globals = self.global_subscribers.write().await;
        let before = globals.len();
        globals.retain(|s| s.id != id);
        globals.len() < before
    }

    /// 取消某事件类型的所有订阅
    pub async fn unsubscribe_all_for_type(&self, event_type: EventType) -> usize {
        let mut subs = self.subscribers.write().await;
        match subs.remove(&event_type) {
            Some(list) => list.len(),
            None => 0,
        }
    }

    /// 获取事件历史
    pub async fn history(&self) -> Vec<EventPayload> {
        self.event_history.read().await.clone()
    }

    /// 获取特定类型的事件历史
    pub async fn history_for_type(&self, event_type: EventType) -> Vec<EventPayload> {
        self.event_history
            .read()
            .await
            .iter()
            .filter(|p| p.event_type == event_type)
            .cloned()
            .collect()
    }

    /// 获取特定订单的事件历史
    pub async fn history_for_order(&self, order_id: &str) -> Vec<EventPayload> {
        self.event_history
            .read()
            .await
            .iter()
            .filter(|p| {
                p.event
                    .order_id()
                    .map(|id| id == order_id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// 清空事件历史
    pub async fn clear_history(&self) {
        self.event_history.write().await.clear();
    }

    /// 获取当前订阅者统计
    pub async fn subscriber_counts(&self) -> HashMap<EventType, usize> {
        let subs = self.subscribers.read().await;
        subs.iter().map(|(k, v)| (*k, v.len())).collect()
    }

    /// 获取全局订阅者数量
    pub async fn global_subscriber_count(&self) -> usize {
        self.global_subscribers.read().await.len()
    }

    // ── 内部方法 ──

    /// 分发同步事件
    async fn dispatch_sync(&self, payload: &EventPayload) {
        // 类型订阅
        let subs = self.subscribers.read().await;
        if let Some(list) = subs.get(&payload.event_type) {
            for sub in list {
                if let Subscriber::Sync(handler) = &sub.subscriber {
                    handler.handle(payload);
                }
            }
        }

        // 全局订阅
        let globals = self.global_subscribers.read().await;
        for sub in globals.iter() {
            if let Subscriber::Sync(handler) = &sub.subscriber {
                handler.handle(payload);
            }
        }
    }

    /// 分发所有事件（包括异步处理器）
    async fn dispatch_all(&self, payload: &EventPayload) {
        // 类型订阅
        let subs = self.subscribers.read().await;
        if let Some(list) = subs.get(&payload.event_type) {
            for sub in list {
                match &sub.subscriber {
                    Subscriber::Sync(handler) => handler.handle(payload),
                    Subscriber::Async(handler) => handler.handle(payload).await,
                }
            }
        }

        // 全局订阅
        let globals = self.global_subscribers.read().await;
        for sub in globals.iter() {
            match &sub.subscriber {
                Subscriber::Sync(handler) => handler.handle(payload),
                Subscriber::Async(handler) => handler.handle(payload).await,
            }
        }
    }

    /// 记录事件历史
    async fn record_history(&self, payload: EventPayload) {
        let mut history = self.event_history.write().await;
        history.push(payload);
        while history.len() > self.max_history {
            history.remove(0);
        }
    }
}

impl Default for TradeEventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 7. 便捷句柄 — EventBusHandle
// ============================================================

/// 线程安全的事件总线句柄，可跨任务共享
#[derive(Clone)]
pub struct EventBusHandle {
    bus: Arc<TradeEventBus>,
}

impl EventBusHandle {
    /// 从 TradeEventBus 创建句柄
    pub fn new(bus: Arc<TradeEventBus>) -> Self {
        Self { bus }
    }

    /// 发布事件
    pub async fn publish(&self, event: TradeEvent) -> EventPayload {
        self.bus.publish(event).await
    }

    /// 异步发布事件
    pub async fn publish_async(&self, event: TradeEvent) -> EventPayload {
        self.bus.publish_async(event).await
    }

    /// 订阅（同步）
    pub async fn subscribe(
        &self,
        event_type: EventType,
        handler: Box<dyn TradeEventHandler>,
    ) -> SubscriberId {
        self.bus.subscribe(event_type, handler).await
    }

    /// 订阅（异步）
    pub async fn subscribe_async(
        &self,
        event_type: EventType,
        handler: Box<dyn AsyncTradeEventHandler>,
    ) -> SubscriberId {
        self.bus.subscribe_async(event_type, handler).await
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, id: SubscriberId) -> bool {
        self.bus.unsubscribe(id).await
    }

    /// 获取事件历史
    pub async fn history(&self) -> Vec<EventPayload> {
        self.bus.history().await
    }

    /// 获取订单事件历史
    pub async fn history_for_order(&self, order_id: &str) -> Vec<EventPayload> {
        self.bus.history_for_order(order_id).await
    }
}

// ============================================================
// 8. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestHandler {
        name: String,
        count: Arc<AtomicUsize>,
    }

    impl TradeEventHandler for TestHandler {
        fn handle(&self, _payload: &EventPayload) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        fn name(&self) -> &str {
            &self.name
        }
    }

    struct StructuredHandler {
        events: Arc<RwLock<Vec<String>>>,
    }

    impl TradeEventHandler for StructuredHandler {
        fn handle(&self, payload: &EventPayload) {
            let name = format!("{:?}", payload.event);
            let events = self.events.clone();
            tokio::spawn(async move {
                events.write().await.push(name);
            });
        }
        fn name(&self) -> &str {
            "StructuredHandler"
        }
    }

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = Arc::new(TradeEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));

        bus.subscribe(
            EventType::Order,
            Box::new(TestHandler {
                name: "order_handler".into(),
                count: counter.clone(),
            }),
        )
        .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "ORD-001".into(),
            buyer_id: "BUY-001".into(),
        })
        .await;

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let bus = Arc::new(TradeEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));

        let id = bus
            .subscribe(
                EventType::Order,
                Box::new(TestHandler {
                    name: "handler".into(),
                    count: counter.clone(),
                }),
            )
            .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "ORD-001".into(),
            buyer_id: "BUY-001".into(),
        })
        .await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        bus.unsubscribe(id).await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "ORD-002".into(),
            buyer_id: "BUY-002".into(),
        })
        .await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_event_history() {
        let bus = Arc::new(TradeEventBus::new());

        bus.publish(TradeEvent::InquiryReceived {
            buyer_id: "B1".into(),
            product_ids: vec!["P1".into()],
            message: "Hello".into(),
        })
        .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".into(),
            buyer_id: "B1".into(),
        })
        .await;

        let history = bus.history().await;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].event_type, EventType::Inquiry);
        assert_eq!(history[1].event_type, EventType::Order);
    }

    #[tokio::test]
    async fn test_history_for_order() {
        let bus = Arc::new(TradeEventBus::new());

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".into(),
            buyer_id: "B1".into(),
        })
        .await;

        bus.publish(TradeEvent::PaymentReceived {
            order_id: "O1".into(),
            amount: 1000.0,
            currency: "USD".into(),
            payment_method: "T/T".into(),
        })
        .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O2".into(),
            buyer_id: "B2".into(),
        })
        .await;

        let o1_history = bus.history_for_order("O1").await;
        assert_eq!(o1_history.len(), 2);

        let o2_history = bus.history_for_order("O2").await;
        assert_eq!(o2_history.len(), 1);
    }

    #[tokio::test]
    async fn test_global_subscriber() {
        let bus = Arc::new(TradeEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));

        bus.subscribe_all(Box::new(TestHandler {
            name: "global".into(),
            count: counter.clone(),
        }))
        .await;

        bus.publish(TradeEvent::InquiryReceived {
            buyer_id: "B1".into(),
            product_ids: vec![],
            message: "Hi".into(),
        })
        .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".into(),
            buyer_id: "B1".into(),
        })
        .await;

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_event_type_routing() {
        let bus = Arc::new(TradeEventBus::new());
        let order_count = Arc::new(AtomicUsize::new(0));
        let finance_count = Arc::new(AtomicUsize::new(0));

        bus.subscribe(
            EventType::Order,
            Box::new(TestHandler {
                name: "order".into(),
                count: order_count.clone(),
            }),
        )
        .await;

        bus.subscribe(
            EventType::Finance,
            Box::new(TestHandler {
                name: "finance".into(),
                count: finance_count.clone(),
            }),
        )
        .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".into(),
            buyer_id: "B1".into(),
        })
        .await;

        bus.publish(TradeEvent::PaymentReceived {
            order_id: "O1".into(),
            amount: 500.0,
            currency: "USD".into(),
            payment_method: "L/C".into(),
        })
        .await;

        assert_eq!(order_count.load(Ordering::SeqCst), 1);
        assert_eq!(finance_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_event_payload_tags() {
        let payload = EventPayload::new(TradeEvent::OrderCreated {
            order_id: "O1".into(),
            buyer_id: "B1".into(),
        })
        .with_tag("priority:high")
        .with_tags(vec!["region:asia".to_string(), "channel:direct".to_string()]);

        assert_eq!(payload.tags.len(), 3);
        assert!(payload.tags.contains(&"priority:high".to_string()));
    }

    #[tokio::test]
    async fn test_trade_event_metadata() {
        let event = TradeEvent::ShipmentDispatched {
            order_id: "O1".into(),
            tracking_number: "TRK-001".into(),
            carrier: "Maersk".into(),
            destination: "Rotterdam".into(),
        };

        assert_eq!(event.event_type(), EventType::Logistics);
        assert_eq!(event.order_id(), Some("O1"));

        let inquiry = TradeEvent::InquiryReceived {
            buyer_id: "B1".into(),
            product_ids: vec![],
            message: "Hi".into(),
        };
        assert_eq!(inquiry.event_type(), EventType::Inquiry);
        assert_eq!(inquiry.order_id(), None);
    }

    #[tokio::test]
    async fn test_history_capacity() {
        let bus = Arc::new(TradeEventBus::with_history_capacity(3));

        for i in 0..5 {
            bus.publish(TradeEvent::Custom {
                event_name: format!("event_{i}"),
                data: HashMap::new(),
            })
            .await;
        }

        let history = bus.history().await;
        assert_eq!(history.len(), 3);
        assert_eq!(
            history[0].event,
            TradeEvent::Custom {
                event_name: "event_2".into(),
                data: HashMap::new(),
            }
        );
    }

    #[tokio::test]
    async fn test_subscriber_counts() {
        let bus = Arc::new(TradeEventBus::new());

        bus.subscribe(
            EventType::Order,
            Box::new(TestHandler {
                name: "a".into(),
                count: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .await;

        bus.subscribe(
            EventType::Order,
            Box::new(TestHandler {
                name: "b".into(),
                count: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .await;

        bus.subscribe(
            EventType::Finance,
            Box::new(TestHandler {
                name: "c".into(),
                count: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .await;

        let counts = bus.subscriber_counts().await;
        assert_eq!(counts.get(&EventType::Order), Some(&2));
        assert_eq!(counts.get(&EventType::Finance), Some(&1));
    }

    #[tokio::test]
    async fn test_event_bus_handle() {
        let bus = Arc::new(TradeEventBus::new());
        let handle = EventBusHandle::new(bus);
        let counter = Arc::new(AtomicUsize::new(0));

        handle
            .subscribe(
                EventType::Order,
                Box::new(TestHandler {
                    name: "h".into(),
                    count: counter.clone(),
                }),
            )
            .await;

        handle
            .publish(TradeEvent::OrderCreated {
                order_id: "O1".into(),
                buyer_id: "B1".into(),
            })
            .await;

        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert_eq!(handle.history().await.len(), 1);
    }
}
