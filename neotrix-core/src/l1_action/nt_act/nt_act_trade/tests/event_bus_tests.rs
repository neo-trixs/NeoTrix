//! 事件总线测试
//!
//! 测试 event_bus.rs 中定义的事件类型、事件负载、事件处理器、事件总线等。
//! 使用 tokio 异步运行时进行测试。

#![forbid(unsafe_code)]

use super::super::event_bus::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::Inquiry.to_string(), "Inquiry");
        assert_eq!(EventType::Quote.to_string(), "Quote");
        assert_eq!(EventType::Order.to_string(), "Order");
        assert_eq!(EventType::Production.to_string(), "Production");
        assert_eq!(EventType::Logistics.to_string(), "Logistics");
        assert_eq!(EventType::Finance.to_string(), "Finance");
        assert_eq!(EventType::Compliance.to_string(), "Compliance");
        assert_eq!(EventType::Risk.to_string(), "Risk");
        assert_eq!(EventType::System.to_string(), "System");
    }

    #[test]
    fn test_trade_event_event_type() {
        let inquiry = TradeEvent::InquiryReceived {
            buyer_id: "B1".to_string(),
            product_ids: vec![],
            message: "Hi".to_string(),
        };
        assert_eq!(inquiry.event_type(), EventType::Inquiry);

        let quote = TradeEvent::QuoteGenerated {
            quote_id: "Q1".to_string(),
            buyer_id: "B1".to_string(),
            total_amount: 1000.0,
            currency: "USD".to_string(),
        };
        assert_eq!(quote.event_type(), EventType::Quote);

        let order = TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        };
        assert_eq!(order.event_type(), EventType::Order);
    }

    #[test]
    fn test_trade_event_order_id() {
        let inquiry = TradeEvent::InquiryReceived {
            buyer_id: "B1".to_string(),
            product_ids: vec![],
            message: "Hi".to_string(),
        };
        assert!(inquiry.order_id().is_none());

        let order = TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        };
        assert_eq!(order.order_id(), Some("O1"));

        let payment = TradeEvent::PaymentReceived {
            order_id: "O1".to_string(),
            amount: 1000.0,
            currency: "USD".to_string(),
            payment_method: "T/T".to_string(),
        };
        assert_eq!(payment.order_id(), Some("O1"));
    }

    #[test]
    fn test_event_payload_creation() {
        let event = TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        };
        let payload = EventPayload::new(event);

        assert!(!payload.id.is_empty());
        assert_eq!(payload.event_type, EventType::Order);
        assert!(payload.tags.is_empty());
    }

    #[test]
    fn test_event_payload_tags() {
        let event = TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        };
        let payload = EventPayload::new(event)
            .with_tag("priority:high")
            .with_tags(vec![
                "region:asia".to_string(),
                "channel:direct".to_string(),
            ]);

        assert_eq!(payload.tags.len(), 3);
        assert!(payload.tags.contains(&"priority:high".to_string()));
        assert!(payload.tags.contains(&"region:asia".to_string()));
        assert!(payload.tags.contains(&"channel:direct".to_string()));
    }

    #[test]
    fn test_risk_severity_ordering() {
        assert!(RiskSeverity::Low < RiskSeverity::Medium);
        assert!(RiskSeverity::Medium < RiskSeverity::High);
        assert!(RiskSeverity::High < RiskSeverity::Critical);
    }

    #[test]
    fn test_compliance_review_result() {
        let approved = ComplianceReviewResult::Approved;
        let rejected = ComplianceReviewResult::Rejected;
        let pending = ComplianceReviewResult::Pending;

        assert_ne!(approved, rejected);
        assert_ne!(approved, pending);
        assert_ne!(rejected, pending);
    }

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = Arc::new(TradeEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));

        bus.subscribe(
            EventType::Order,
            Box::new(TestHandler {
                name: "order_handler".to_string(),
                count: counter.clone(),
            }),
        )
        .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        })
        .await;

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_event_bus_unsubscribe() {
        let bus = Arc::new(TradeEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));

        let id = bus
            .subscribe(
                EventType::Order,
                Box::new(TestHandler {
                    name: "handler".to_string(),
                    count: counter.clone(),
                }),
            )
            .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        })
        .await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        bus.unsubscribe(id).await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O2".to_string(),
            buyer_id: "B2".to_string(),
        })
        .await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_event_bus_history() {
        let bus = Arc::new(TradeEventBus::new());

        bus.publish(TradeEvent::InquiryReceived {
            buyer_id: "B1".to_string(),
            product_ids: vec!["P1".to_string()],
            message: "Hello".to_string(),
        })
        .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        })
        .await;

        let history = bus.history().await;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].event_type, EventType::Inquiry);
        assert_eq!(history[1].event_type, EventType::Order);
    }

    #[tokio::test]
    async fn test_event_bus_history_for_order() {
        let bus = Arc::new(TradeEventBus::new());

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        })
        .await;

        bus.publish(TradeEvent::PaymentReceived {
            order_id: "O1".to_string(),
            amount: 1000.0,
            currency: "USD".to_string(),
            payment_method: "T/T".to_string(),
        })
        .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O2".to_string(),
            buyer_id: "B2".to_string(),
        })
        .await;

        let o1_history = bus.history_for_order("O1").await;
        assert_eq!(o1_history.len(), 2);

        let o2_history = bus.history_for_order("O2").await;
        assert_eq!(o2_history.len(), 1);
    }

    #[tokio::test]
    async fn test_event_bus_global_subscriber() {
        let bus = Arc::new(TradeEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));

        bus.subscribe_all(Box::new(TestHandler {
            name: "global".to_string(),
            count: counter.clone(),
        }))
        .await;

        bus.publish(TradeEvent::InquiryReceived {
            buyer_id: "B1".to_string(),
            product_ids: vec![],
            message: "Hi".to_string(),
        })
        .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        })
        .await;

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_event_bus_type_routing() {
        let bus = Arc::new(TradeEventBus::new());
        let order_count = Arc::new(AtomicUsize::new(0));
        let finance_count = Arc::new(AtomicUsize::new(0));

        bus.subscribe(
            EventType::Order,
            Box::new(TestHandler {
                name: "order".to_string(),
                count: order_count.clone(),
            }),
        )
        .await;

        bus.subscribe(
            EventType::Finance,
            Box::new(TestHandler {
                name: "finance".to_string(),
                count: finance_count.clone(),
            }),
        )
        .await;

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        })
        .await;

        bus.publish(TradeEvent::PaymentReceived {
            order_id: "O1".to_string(),
            amount: 500.0,
            currency: "USD".to_string(),
            payment_method: "L/C".to_string(),
        })
        .await;

        assert_eq!(order_count.load(Ordering::SeqCst), 1);
        assert_eq!(finance_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_event_bus_history_capacity() {
        let bus = Arc::new(TradeEventBus::with_history_capacity(3));

        for i in 0..5 {
            bus.publish(TradeEvent::Custom {
                event_name: format!("event_{}", i),
                data: HashMap::new(),
            })
            .await;
        }

        let history = bus.history().await;
        assert_eq!(history.len(), 3);
        assert_eq!(
            history[0].event,
            TradeEvent::Custom {
                event_name: "event_2".to_string(),
                data: HashMap::new(),
            }
        );
    }

    #[tokio::test]
    async fn test_event_bus_subscriber_counts() {
        let bus = Arc::new(TradeEventBus::new());

        bus.subscribe(
            EventType::Order,
            Box::new(TestHandler {
                name: "a".to_string(),
                count: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .await;

        bus.subscribe(
            EventType::Order,
            Box::new(TestHandler {
                name: "b".to_string(),
                count: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .await;

        bus.subscribe(
            EventType::Finance,
            Box::new(TestHandler {
                name: "c".to_string(),
                count: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .await;

        let counts = bus.subscriber_counts().await;
        assert_eq!(counts.get(&EventType::Order), Some(&2));
        assert_eq!(counts.get(&EventType::Finance), Some(&1));
    }

    #[tokio::test]
    async fn test_event_bus_clear_history() {
        let bus = Arc::new(TradeEventBus::new());

        bus.publish(TradeEvent::OrderCreated {
            order_id: "O1".to_string(),
            buyer_id: "B1".to_string(),
        })
        .await;

        assert_eq!(bus.history().await.len(), 1);

        bus.clear_history().await;
        assert_eq!(bus.history().await.len(), 0);
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
                    name: "h".to_string(),
                    count: counter.clone(),
                }),
            )
            .await;

        handle
            .publish(TradeEvent::OrderCreated {
                order_id: "O1".to_string(),
                buyer_id: "B1".to_string(),
            })
            .await;

        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert_eq!(handle.history().await.len(), 1);
    }

    #[tokio::test]
    async fn test_trade_event_serialization() {
        let events = vec![
            TradeEvent::InquiryReceived {
                buyer_id: "B1".to_string(),
                product_ids: vec!["P1".to_string()],
                message: "Hello".to_string(),
            },
            TradeEvent::QuoteGenerated {
                quote_id: "Q1".to_string(),
                buyer_id: "B1".to_string(),
                total_amount: 1000.0,
                currency: "USD".to_string(),
            },
            TradeEvent::OrderCreated {
                order_id: "O1".to_string(),
                buyer_id: "B1".to_string(),
            },
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            let deserialized: TradeEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.event_type(), event.event_type());
        }
    }
}
