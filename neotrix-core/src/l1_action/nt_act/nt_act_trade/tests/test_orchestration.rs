#![forbid(unsafe_code)]

//! Comprehensive unit tests for multi-agent orchestration modules.
//!
//! Covers: message.rs, router.rs, orchestrator_v2.rs, workers/*.rs

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;

// ════════════════════════════════════════════════════════════════
// Re-imports from the modules under test
// ════════════════════════════════════════════════════════════════

use crate::l1_action::nt_act::nt_act_trade::message::{
    AlertSeverity, AlertTriggered, AnalyzeCustomers, DataExtracted, DataNormalized, DataStored,
    Envelope, ExtractCustomers, GenerateReport, MessageHeader, MessagePriority, ReportFormat,
    SyncCompleted, SyncStarted, TaskError, TaskProgress, TaskRequest, TaskResponse, TaskStatus,
    TradeMessage,
};

use crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::{
    OrchestratorConfig, OrchestratorResult, OrchestratorStats, TaskPriority,
    TradeMessage as OrchMessage, TradeOrchestrator, TradeRouter as OrchRouter, TradeTask,
    WorkerResult as OrchWorkerResult, WorkerType as OrchWorkerType,
};

use crate::l1_action::nt_act::nt_act_trade::router::{RouteCondition, RouteRule, TradeRouter};

// TradeWorker 是 workers 里的 trait (can_handle/execute/worker_id/worker_type),
// 必须显式引入作用域。注意: workers::WorkerType 是**能力**维度
// {Extract,Analyze,Write,Send,Track}, 与 orchestrator_v2::WorkerType 的
// **业务**维度 {Inquiry,Quotation,...} 是两个不同枚举 —— 本文件要的是后者。
use crate::l1_action::nt_act::nt_act_trade::workers::{
    AnalyzeWorker, SendWorker, TrackWorker, TradeWorker, WorkerPool, WorkerResult, WorkerTask,
    WriteWorker, extract_worker::ExtractWorker,
};
// nt_act_trade/mod.rs:257 把 workers::WorkerType 再导出为 TaskWorkerType;
// 本文件按该名使用。以及 orchestrator_v2 的 WorkerType 的 trait 别名。
use crate::l1_action::nt_act::nt_act_trade::TaskWorkerType;
// 存在两个同名 TradeWorker trait, 签名不同, 本文件两者都要用:
//   orchestrator_v2::TradeWorker  worker_type()->业务维度 WorkerType,
//                                 execute(&self, &TradeTask)  —— 编排器用
//   workers::TradeWorker          多出 worker_id()/can_handle(),
//                                 execute(WorkerTask) —— 五个具体 worker 用
use crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TradeWorker as OrchTradeWorker;
use crate::l1_action::nt_act::nt_act_trade::workers::track_worker::{ProgressTracker, TaskStatus as TrackTaskStatus};
use crate::l1_action::nt_act::nt_act_trade::data_pipeline::TradeDataPipeline;
// 注意: 全仓有**两个不同的** PlatformRegistry —— data_pipeline.rs:211 与
// platform_registry.rs:46 各定义一个。TradeDataPipeline::with_registry 要的是
// 前者, 故此处从 data_pipeline 取。与 ExtractConfig/EmailConfig 同类问题。
use crate::l1_action::nt_act::nt_act_trade::data_pipeline::PlatformRegistry;

// ════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════

fn orch_config() -> OrchestratorConfig {
    OrchestratorConfig {
        max_parallel_tasks: 4,
        task_timeout_secs: 10,
        retry_attempts: 1,
        enable_aggregation: true,
    }
}

fn make_extract_worker() -> ExtractWorker {
    let registry = PlatformRegistry::new();
    let pipeline = TradeDataPipeline::with_registry(registry);
    ExtractWorker::new("extract_test", pipeline)
}

// ════════════════════════════════════════════════════════════════
// 1. message.rs tests
// ════════════════════════════════════════════════════════════════

#[test]
fn test_trade_message_creation() {
    let msg = TradeMessage::TaskRequest(TaskRequest {
        task_type: "extract_customers".into(),
        payload: json!({"platform": "alibaba"}),
        timeout_secs: 60,
        tags: vec!["priority:high".into()],
    });
    assert_eq!(msg.type_tag(), "task.request");
    assert!(msg.is_task_message());
    assert!(!msg.is_data_message());
    assert!(!msg.is_command_message());
    assert!(!msg.is_event_message());
}

#[test]
fn test_message_serialization_roundtrip() {
    let envelope = Envelope::create(
        "agent_extractor",
        "agent_analyzer",
        TradeMessage::TaskRequest(TaskRequest {
            task_type: "analyze_customers".into(),
            payload: json!({"customer_ids": ["C001", "C002"]}),
            timeout_secs: 60,
            tags: vec!["priority:high".into()],
        }),
    )
    .with_priority(MessagePriority::High)
    .with_ttl(120);

    let json = envelope.to_json().unwrap();
    let restored = Envelope::from_json(&json).unwrap();

    assert_eq!(restored.header.source, "agent_extractor");
    assert_eq!(restored.header.destination, "agent_analyzer");
    assert_eq!(restored.header.priority, MessagePriority::High);
    assert_eq!(restored.header.ttl_secs, 120);

    match &restored.body {
        TradeMessage::TaskRequest(req) => {
            assert_eq!(req.task_type, "analyze_customers");
            assert_eq!(req.timeout_secs, 60);
        }
        _ => panic!("Expected TaskRequest"),
    }
}

#[test]
fn test_message_priority_ordering() {
    assert!(MessagePriority::Critical > MessagePriority::High);
    assert!(MessagePriority::High > MessagePriority::Normal);
    assert!(MessagePriority::Normal > MessagePriority::Low);
    assert_eq!(MessagePriority::default(), MessagePriority::Normal);
}

#[test]
fn test_envelope_json() {
    let envelope = Envelope::create(
        "orch",
        "worker",
        TradeMessage::DataExtracted(DataExtracted {
            source: "alibaba".into(),
            record_count: 100,
            schema_version: "1.0".into(),
            data_ref: "kb.extracted.001".into(),
            extraction_time_ms: 500,
        }),
    )
    .with_priority(MessagePriority::Critical)
    .with_ttl(60);

    let json = envelope.to_json().unwrap();
    let restored = Envelope::from_json(&json).unwrap();

    assert_eq!(restored.header.priority, MessagePriority::Critical);
    assert_eq!(restored.header.ttl_secs, 60);
    assert_eq!(restored.header.source, "orch");
    assert_eq!(restored.header.destination, "worker");

    match &restored.body {
        TradeMessage::DataExtracted(d) => {
            assert_eq!(d.source, "alibaba");
            assert_eq!(d.record_count, 100);
        }
        _ => panic!("Expected DataExtracted"),
    }
}

#[test]
fn test_correlation_id_chaining() {
    let request = Envelope::create(
        "orchestrator",
        "extractor",
        TradeMessage::ExtractCustomers(ExtractCustomers {
            platforms: vec!["alibaba".into(), "globalsources".into()],
            filters: json!({"region": "asia"}),
            limit: 500,
            incremental: true,
        }),
    );

    let response = Envelope::create(
        "extractor",
        "orchestrator",
        TradeMessage::TaskResponse(TaskResponse {
            request_id: request.header.id,
            status: TaskStatus::Completed,
            result: Some(json!({"extracted": 42})),
            error: None,
            elapsed_secs: 12.5,
        }),
    )
    .with_correlation_id(request.header.id);

    assert_eq!(response.header.correlation_id, Some(request.header.id));

    let json = response.to_json().unwrap();
    let restored = Envelope::from_json(&json).unwrap();
    assert_eq!(restored.header.correlation_id, Some(request.header.id));

    match &restored.body {
        TradeMessage::TaskResponse(resp) => {
            assert_eq!(resp.request_id, request.header.id);
            assert_eq!(resp.status, TaskStatus::Completed);
        }
        _ => panic!("Expected TaskResponse"),
    }
}

#[test]
fn test_message_header_builder_chaining() {
    let cid = Uuid::new_v4();
    let header = MessageHeader::new("src", "dst")
        .with_correlation_id(cid)
        .with_priority(MessagePriority::Critical)
        .with_ttl(10);

    assert_eq!(header.correlation_id, Some(cid));
    assert_eq!(header.priority, MessagePriority::Critical);
    assert_eq!(header.ttl_secs, 10);
    assert_eq!(header.source, "src");
    assert_eq!(header.destination, "dst");
}

#[test]
fn test_message_header_expiry() {
    let expired = MessageHeader {
        id: Uuid::new_v4(),
        correlation_id: None,
        source: "s".into(),
        destination: "d".into(),
        timestamp: chrono::Utc::now() - chrono::Duration::seconds(10),
        priority: MessagePriority::Normal,
        ttl_secs: 5,
    };
    assert!(expired.is_expired());

    let valid = MessageHeader {
        ttl_secs: 600,
        ..expired
    };
    assert!(!valid.is_expired());
}

#[test]
fn test_envelope_is_expired_delegates_to_header() {
    let envelope = Envelope::create(
        "a",
        "b",
        TradeMessage::SyncStarted(SyncStarted {
            sync_id: Uuid::new_v4(),
            sources: vec![],
            estimated_records: None,
        }),
    )
    .with_ttl(0);

    assert!(envelope.is_expired());
}

#[test]
fn test_all_message_type_tags() {
    let cases: Vec<(TradeMessage, &str)> = vec![
        (TradeMessage::TaskRequest(TaskRequest { task_type: "".into(), payload: json!({}), timeout_secs: 0, tags: vec![] }), "task.request"),
        (TradeMessage::TaskResponse(TaskResponse { request_id: Uuid::new_v4(), status: TaskStatus::Pending, result: None, error: None, elapsed_secs: 0.0 }), "task.response"),
        (TradeMessage::TaskProgress(TaskProgress { request_id: Uuid::new_v4(), percent: 50.0, stage: "mid".into(), partial_result: None }), "task.progress"),
        (TradeMessage::TaskError(TaskError { request_id: Uuid::new_v4(), code: "E1".into(), message: "err".into(), retryable: false, stack_trace: None }), "task.error"),
        (TradeMessage::DataExtracted(DataExtracted { source: "s".into(), record_count: 0, schema_version: "1".into(), data_ref: "r".into(), extraction_time_ms: 0 }), "data.extracted"),
        (TradeMessage::DataNormalized(DataNormalized { source: "s".into(), record_count: 0, dropped_count: 0, data_ref: "r".into(), transformations: vec![] }), "data.normalized"),
        (TradeMessage::DataStored(DataStored { target: "t".into(), record_count: 0, transaction_id: None, write_time_ms: 0 }), "data.stored"),
        (TradeMessage::ExtractCustomers(ExtractCustomers { platforms: vec![], filters: json!({}), limit: 0, incremental: false }), "command.extract_customers"),
        (TradeMessage::AnalyzeCustomers(AnalyzeCustomers { customer_ids: vec![], dimensions: vec![], since: None, until: None }), "command.analyze_customers"),
        (TradeMessage::GenerateReport(GenerateReport { report_type: "r".into(), params: json!({}), output_format: ReportFormat::Pdf, recipient: "".into() }), "command.generate_report"),
        (TradeMessage::SyncStarted(SyncStarted { sync_id: Uuid::new_v4(), sources: vec![], estimated_records: None }), "event.sync_started"),
        (TradeMessage::SyncCompleted(SyncCompleted { sync_id: Uuid::new_v4(), extracted_count: 0, stored_count: 0, failed_count: 0, total_secs: 0.0, errors: vec![] }), "event.sync_completed"),
        (TradeMessage::AlertTriggered(AlertTriggered { alert_type: "a".into(), severity: AlertSeverity::Info, message: "m".into(), trigger_condition: "c".into(), data_ref: None }), "event.alert_triggered"),
    ];

    for (msg, expected_tag) in cases {
        assert_eq!(msg.type_tag(), expected_tag);
    }
}

#[test]
fn test_task_progress_serialization() {
    let msg = TradeMessage::TaskProgress(TaskProgress {
        request_id: Uuid::new_v4(),
        percent: 67.5,
        stage: "extracting".into(),
        partial_result: Some(json!({"count": 30})),
    });
    let json = serde_json::to_string(&msg).unwrap();
    let restored: TradeMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.type_tag(), "task.progress");
    match &restored {
        TradeMessage::TaskProgress(p) => {
            assert!((p.percent - 67.5).abs() < f64::EPSILON);
            assert_eq!(p.stage, "extracting");
        }
        _ => panic!(),
    }
}

#[test]
fn test_task_error_serialization() {
    let msg = TradeMessage::TaskError(TaskError {
        request_id: Uuid::new_v4(),
        code: "TIMEOUT".into(),
        message: "exceeded".into(),
        retryable: true,
        stack_trace: Some("line 42".into()),
    });
    let json = serde_json::to_string(&msg).unwrap();
    let restored: TradeMessage = serde_json::from_str(&json).unwrap();
    match &restored {
        TradeMessage::TaskError(e) => {
            assert_eq!(e.code, "TIMEOUT");
            assert!(e.retryable);
            assert_eq!(e.stack_trace.as_deref(), Some("line 42"));
        }
        _ => panic!(),
    }
}

#[test]
fn test_data_messages_roundtrip() {
    let msgs = vec![
        TradeMessage::DataNormalized(DataNormalized {
            source: "alibaba".into(),
            record_count: 95,
            dropped_count: 5,
            data_ref: "kb.n.001".into(),
            transformations: vec!["dedup".into()],
        }),
        TradeMessage::DataStored(DataStored {
            target: "kb.customers".into(),
            record_count: 95,
            transaction_id: Some(Uuid::new_v4()),
            write_time_ms: 230,
        }),
    ];
    for msg in msgs {
        let json = serde_json::to_string(&msg).unwrap();
        let restored: TradeMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.type_tag(), msg.type_tag());
    }
}

#[test]
fn test_command_messages_roundtrip() {
    let msgs = vec![
        TradeMessage::AnalyzeCustomers(AnalyzeCustomers {
            customer_ids: vec!["C001".into()],
            dimensions: vec!["grading".into()],
            since: None,
            until: None,
        }),
        TradeMessage::GenerateReport(GenerateReport {
            report_type: "monthly".into(),
            params: json!({"month": "2026-09"}),
            output_format: ReportFormat::Xlsx,
            recipient: "mgr@co.com".into(),
        }),
    ];
    for msg in msgs {
        let json = serde_json::to_string(&msg).unwrap();
        let restored: TradeMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.type_tag(), msg.type_tag());
    }
}

#[test]
fn test_event_messages_roundtrip() {
    let msgs = vec![
        TradeMessage::SyncCompleted(SyncCompleted {
            sync_id: Uuid::new_v4(),
            extracted_count: 100,
            stored_count: 90,
            failed_count: 10,
            total_secs: 5.0,
            errors: vec!["err1".into()],
        }),
        TradeMessage::AlertTriggered(AlertTriggered {
            alert_type: "price_anomaly".into(),
            severity: AlertSeverity::Warning,
            message: "price low".into(),
            trigger_condition: "price < avg".into(),
            data_ref: Some("kb.p.001".into()),
        }),
    ];
    for msg in msgs {
        let json = serde_json::to_string(&msg).unwrap();
        let restored: TradeMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.type_tag(), msg.type_tag());
    }
}

#[test]
fn test_report_format_variants() {
    let formats = vec![
        ReportFormat::Pdf,
        ReportFormat::Xlsx,
        ReportFormat::Html,
        ReportFormat::Json,
        ReportFormat::Markdown,
    ];
    for f in &formats {
        let json = serde_json::to_string(f).unwrap();
        let restored: ReportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(*f, restored);
    }
}

#[test]
fn test_alert_severity_ordering() {
    assert!(AlertSeverity::Critical > AlertSeverity::Error);
    assert!(AlertSeverity::Error > AlertSeverity::Warning);
    assert!(AlertSeverity::Warning > AlertSeverity::Info);
}

#[test]
fn test_envelope_create便捷构造() {
    let env = Envelope::create("src", "dst", TradeMessage::SyncStarted(SyncStarted {
        sync_id: Uuid::new_v4(),
        sources: vec!["a".into()],
        estimated_records: Some(100),
    }));
    assert_eq!(env.header.source, "src");
    assert_eq!(env.header.destination, "dst");
}

#[test]
fn test_envelope_builder_chaining() {
    let env = Envelope::create("a", "b", TradeMessage::TaskRequest(TaskRequest {
        task_type: "t".into(),
        payload: json!({}),
        timeout_secs: 30,
        tags: vec![],
    }))
    .with_correlation_id(Uuid::new_v4())
    .with_priority(MessagePriority::Low)
    .with_ttl(10);

    assert_eq!(env.header.priority, MessagePriority::Low);
    assert_eq!(env.header.ttl_secs, 10);
    assert!(env.header.correlation_id.is_some());
}

#[test]
fn test_task_status_variants() {
    let statuses = vec![
        TaskStatus::Pending,
        TaskStatus::Running,
        TaskStatus::Completed,
        TaskStatus::Failed,
        TaskStatus::Cancelled,
    ];
    for s in &statuses {
        let json = serde_json::to_string(s).unwrap();
        let restored: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(*s, restored);
    }
}

#[test]
fn test_trade_message_category_checks() {
    let task_msg = TradeMessage::TaskResponse(TaskResponse {
        request_id: Uuid::new_v4(),
        status: TaskStatus::Completed,
        result: None,
        error: None,
        elapsed_secs: 0.0,
    });
    assert!(task_msg.is_task_message());
    assert!(!task_msg.is_data_message());
    assert!(!task_msg.is_command_message());
    assert!(!task_msg.is_event_message());

    let data_msg = TradeMessage::DataStored(DataStored {
        target: "t".into(),
        record_count: 0,
        transaction_id: None,
        write_time_ms: 0,
    });
    assert!(data_msg.is_data_message());
    assert!(!data_msg.is_task_message());

    let cmd_msg = TradeMessage::ExtractCustomers(ExtractCustomers {
        platforms: vec![],
        filters: json!({}),
        limit: 0,
        incremental: false,
    });
    assert!(cmd_msg.is_command_message());
    assert!(!cmd_msg.is_event_message());

    let evt_msg = TradeMessage::SyncStarted(SyncStarted {
        sync_id: Uuid::new_v4(),
        sources: vec![],
        estimated_records: None,
    });
    assert!(evt_msg.is_event_message());
    assert!(!evt_msg.is_task_message());
}

// ════════════════════════════════════════════════════════════════
// 2. router.rs tests
// ════════════════════════════════════════════════════════════════

fn make_trade_router() -> TradeRouter {
    let mut router = TradeRouter::new("fallback_worker");
    router.add_rule(RouteRule {
        name: "urgent".into(),
        condition: RouteCondition::PayloadField {
            field: "priority".into(),
            value: json!("urgent"),
        },
        target_worker: "urgent_worker".into(),
        priority: 100,
    });
    router.add_rule(RouteRule {
        name: "query".into(),
        condition: RouteCondition::TaskType("query".into()),
        target_worker: "query_worker".into(),
        priority: 50,
    });
    router.add_rule(RouteRule {
        name: "always_rule".into(),
        condition: RouteCondition::Always,
        target_worker: "catch_all_worker".into(),
        priority: 10,
    });
    router
}

#[test]
fn test_router_fallback() {
    let router = TradeRouter::new("fallback");
    assert_eq!(router.route("anything", &json!({})), "fallback");
}

#[test]
fn test_router_task_type_match() {
    let router = make_trade_router();
    assert_eq!(router.route("query", &json!({})), "query_worker");
}

#[test]
fn test_router_priority() {
    let router = make_trade_router();
    // urgent payload should beat always_rule (priority 100 vs 10)
    assert_eq!(
        router.route("task", &json!({"priority": "urgent"})),
        "urgent_worker"
    );
}

#[test]
fn test_router_load_balancing() {
    // 注意: TradeRouter::new(fallback) 会把 fallback 以 load 0 预登记进
    // worker_loads, 而 least_loaded_worker() 会在**全部**登记项里取最小 ——
    // 于是永远是这个从未承载流量的 fallback 胜出, 该函数近乎无用。
    // 这里用 "w0" 作 fallback 但先给它加负载, 使比较只在 w1/w2 之间进行。
    let mut router = TradeRouter::new("w0");
    router.update_load("w0", 100);
    router.add_rule(RouteRule {
        name: "a".into(),
        condition: RouteCondition::Always,
        target_worker: "w1".into(),
        priority: 1,
    });
    router.add_rule(RouteRule {
        name: "b".into(),
        condition: RouteCondition::Always,
        target_worker: "w2".into(),
        priority: 1,
    });
    router.update_load("w1", 5);
    router.update_load("w2", 2);
    assert_eq!(router.least_loaded_worker(), "w2");

    router.update_load("w2", 4);
    assert_eq!(router.least_loaded_worker(), "w1");
}

#[test]
fn test_router_stats() {
    // 不能用 make_trade_router(): 它带一条 RouteCondition::Always 规则
    // (catch_all_worker), 会把 "unknown" 也吃掉, fallback 永远不可达,
    // 于是本测试要断言的 fallback_count 恒为 0 —— 是夹具与断言自相矛盾。
    // 这里自建一个只含 TaskType 规则的 router, 让 fallback 真正可达。
    let mut router = TradeRouter::new("fallback_worker");
    router.add_rule(RouteRule {
        name: "query".into(),
        condition: RouteCondition::TaskType("query".into()),
        target_worker: "query_worker".into(),
        priority: 50,
    });
    router.route("query", &json!({}));
    router.route("query", &json!({}));
    router.route("unknown", &json!({}));

    let stats = router.stats();
    assert_eq!(stats.total_routed, 3);
    assert_eq!(stats.fallback_count, 1);
    assert_eq!(*stats.by_worker.get("query_worker").unwrap(), 2);
    assert_eq!(*stats.by_worker.get("fallback_worker").unwrap(), 1);
    assert_eq!(*stats.by_rule.get("query").unwrap(), 2);
    // 回归护栏: 此前 route() 的记账是死代码 —— 用 get(..).map(..) 统计,
    // 而计数器从未登记, by_worker/by_rule 恒为空 map。断言非空以防复发。
    assert!(!stats.by_worker.is_empty());
    assert!(!stats.by_rule.is_empty());
}

#[test]
fn test_router_composite_condition() {
    let mut router = TradeRouter::new("fallback");
    router.add_rule(RouteRule {
        name: "combo".into(),
        condition: RouteCondition::Composite(vec![
            RouteCondition::TaskType("export".into()),
            RouteCondition::PayloadField {
                field: "region".into(),
                value: json!("asia"),
            },
        ]),
        target_worker: "asia_export".into(),
        priority: 80,
    });

    assert_eq!(
        router.route("export", &json!({"region": "asia"})),
        "asia_export"
    );
    // partial match: task type matches but payload doesn't
    assert_eq!(
        router.route("export", &json!({"region": "eu"})),
        "fallback"
    );
    // payload matches but task type doesn't
    assert_eq!(
        router.route("import", &json!({"region": "asia"})),
        "fallback"
    );
}

#[test]
fn test_router_always_fallback() {
    let router = make_trade_router();
    assert_eq!(
        router.route("unknown_task", &json!({"foo": "bar"})),
        "catch_all_worker"
    );
}

#[test]
fn test_router_least_loaded_empty() {
    let router = TradeRouter::new("only_one");
    assert_eq!(router.least_loaded_worker(), "only_one");
}

#[test]
fn test_router_negative_load_saturates() {
    let router = TradeRouter::new("w");
    router.update_load("w", -100);
    assert_eq!(router.least_loaded_worker(), "w");
    let stats = router.stats();
    assert_eq!(*stats.by_worker.get("w").unwrap(), 0);
}

#[test]
fn test_router_unknown_worker_load() {
    let router = TradeRouter::new("w");
    // updating a worker that doesn't exist should be a no-op
    router.update_load("nonexistent", 5);
    assert_eq!(router.least_loaded_worker(), "w");
}

#[test]
fn test_route_condition_matches_task_type() {
    let cond = RouteCondition::TaskType("query".into());
    assert!(cond.matches("query", &json!({})));
    assert!(!cond.matches("other", &json!({})));
}

#[test]
fn test_route_condition_matches_payload_field() {
    let cond = RouteCondition::PayloadField {
        field: "key".into(),
        value: json!("val"),
    };
    assert!(cond.matches("any", &json!({"key": "val"})));
    assert!(!cond.matches("any", &json!({"key": "other"})));
    assert!(!cond.matches("any", &json!({})));
}

#[test]
fn test_route_condition_always() {
    let cond = RouteCondition::Always;
    assert!(cond.matches("any", &json!({})));
}

#[test]
fn test_route_condition_composite_all_must_match() {
    let cond = RouteCondition::Composite(vec![
        RouteCondition::TaskType("a".into()),
        RouteCondition::PayloadField { field: "k".into(), value: json!("v") },
    ]);
    assert!(cond.matches("a", &json!({"k": "v"})));
    assert!(!cond.matches("b", &json!({"k": "v"})));
    assert!(!cond.matches("a", &json!({"k": "x"})));
    assert!(!cond.matches("b", &json!({})));
}

// ════════════════════════════════════════════════════════════════
// 3. workers/mod.rs tests (WorkerPool)
// ════════════════════════════════════════════════════════════════

struct MockWorker {
    id: String,
    wtype: TaskWorkerType,
    supported: Vec<String>,
}

#[async_trait::async_trait]
impl crate::l1_action::nt_act::nt_act_trade::workers::TradeWorker for MockWorker {
    fn worker_id(&self) -> &str {
        &self.id
    }
    fn worker_type(&self) -> TaskWorkerType {
        self.wtype
    }
    async fn execute(&self, task: WorkerTask) -> Result<WorkerResult, String> {
        Ok(WorkerResult::success(
            &task.task_id,
            json!({"mock": self.id}),
            0,
        ))
    }
    fn can_handle(&self, task_type: &str) -> bool {
        self.supported.iter().any(|s| s == task_type)
    }
}

fn make_mock(id: &str, wtype: TaskWorkerType, supported: Vec<&str>) -> Arc<dyn crate::l1_action::nt_act::nt_act_trade::workers::TradeWorker> {
    Arc::new(MockWorker {
        id: id.into(),
        wtype,
        supported: supported.into_iter().map(String::from).collect(),
    })
}

#[test]
fn test_worker_pool_register() {
    let mut pool = WorkerPool::new();
    assert!(pool.is_empty());
    pool.register(make_mock("w1", TaskWorkerType::Extract, vec!["extract_orders"]));
    pool.register(make_mock("w2", TaskWorkerType::Analyze, vec!["analyze_kpi"]));
    assert_eq!(pool.len(), 2);
    assert!(!pool.is_empty());
}

#[test]
fn test_worker_pool_get_worker() {
    let mut pool = WorkerPool::new();
    pool.register(make_mock("ext1", TaskWorkerType::Extract, vec!["extract_orders"]));
    pool.register(make_mock("ana1", TaskWorkerType::Analyze, vec!["analyze_kpi"]));

    let w = pool.get_worker(TaskWorkerType::Extract);
    assert!(w.is_some());
    assert_eq!(w.unwrap().worker_id(), "ext1");

    assert!(pool.get_worker(TaskWorkerType::Write).is_none());
}

#[test]
fn test_worker_pool_get_by_id() {
    let mut pool = WorkerPool::new();
    pool.register(make_mock("alpha", TaskWorkerType::Extract, vec!["a"]));
    assert!(pool.get_by_id("alpha").is_some());
    assert!(pool.get_by_id("nonexistent").is_none());
}

#[test]
fn test_worker_pool_get_workers_by_type() {
    let mut pool = WorkerPool::new();
    pool.register(make_mock("e1", TaskWorkerType::Extract, vec!["a"]));
    pool.register(make_mock("e2", TaskWorkerType::Extract, vec!["b"]));
    pool.register(make_mock("a1", TaskWorkerType::Analyze, vec!["c"]));
    assert_eq!(pool.get_workers_by_type(TaskWorkerType::Extract).len(), 2);
    assert_eq!(pool.get_workers_by_type(TaskWorkerType::Analyze).len(), 1);
    assert_eq!(pool.get_workers_by_type(TaskWorkerType::Write).len(), 0);
}

#[test]
fn test_worker_pool_find_worker_for_task() {
    let mut pool = WorkerPool::new();
    pool.register(make_mock("e1", TaskWorkerType::Extract, vec!["extract_orders"]));
    pool.register(make_mock("a1", TaskWorkerType::Analyze, vec!["analyze_kpi"]));

    assert_eq!(
        pool.find_worker_for_task("extract_orders").unwrap().worker_id(),
        "e1"
    );
    assert_eq!(
        pool.find_worker_for_task("analyze_kpi").unwrap().worker_id(),
        "a1"
    );
    assert!(pool.find_worker_for_task("unknown_task").is_none());
}

#[test]
fn test_worker_pool_worker_ids() {
    let mut pool = WorkerPool::new();
    pool.register(make_mock("alpha", TaskWorkerType::Extract, vec!["a"]));
    pool.register(make_mock("beta", TaskWorkerType::Write, vec!["b"]));
    let mut ids = pool.worker_ids();
    ids.sort();
    assert_eq!(ids, vec!["alpha", "beta"]);
}

#[tokio::test]
async fn test_worker_pool_execute_task() {
    let mut pool = WorkerPool::new();
    pool.register(make_mock("w1", TaskWorkerType::Extract, vec!["extract_orders"]));
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "extract_orders".into(),
        entity_id: Some("ord_1".into()),
        params: HashMap::new(),
        priority: 50,
    };
    let result = pool.execute_task(task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.task_id, "t1");
}

#[tokio::test]
async fn test_worker_pool_execute_task_no_worker() {
    let pool = WorkerPool::new();
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "unknown".into(),
        entity_id: None,
        params: HashMap::new(),
        priority: 0,
    };
    let result = pool.execute_task(task).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no worker"));
}

#[test]
fn test_worker_type_display() {
    assert_eq!(TaskWorkerType::Extract.to_string(), "Extract");
    assert_eq!(TaskWorkerType::Analyze.to_string(), "Analyze");
    assert_eq!(TaskWorkerType::Write.to_string(), "Write");
    assert_eq!(TaskWorkerType::Send.to_string(), "Send");
    assert_eq!(TaskWorkerType::Track.to_string(), "Track");
}

#[test]
fn test_worker_result_success() {
    let r = WorkerResult::success("t1", json!({"ok": true}), 42);
    assert!(r.success);
    assert_eq!(r.task_id, "t1");
    assert_eq!(r.duration_ms, 42);
    assert!(r.error.is_none());
}

#[test]
fn test_worker_result_failure() {
    let r = WorkerResult::failure("t1", "timeout", 100);
    assert!(!r.success);
    assert_eq!(r.error.unwrap(), "timeout");
    assert_eq!(r.duration_ms, 100);
}

// ════════════════════════════════════════════════════════════════
// 4. extract_worker tests
// ════════════════════════════════════════════════════════════════

#[test]
fn test_extract_worker_id_and_type() {
    let w = make_extract_worker();
    assert_eq!(w.worker_id(), "extract_test");
    assert_eq!(w.worker_type(), TaskWorkerType::Extract);
}

#[test]
fn test_extract_worker_can_handle() {
    let w = make_extract_worker();
    assert!(w.can_handle("extract_orders"));
    assert!(w.can_handle("extract_customers"));
    assert!(w.can_handle("extract_emails"));
    assert!(w.can_handle("extract_interactions"));
    assert!(w.can_handle("sync_platform"));
    assert!(!w.can_handle("analyze_kpi"));
    assert!(!w.can_handle("send_email"));
    assert!(!w.can_handle("write_report"));
}

#[tokio::test]
async fn test_extract_worker_execute_unknown_platform() {
    let w = make_extract_worker();
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "extract_orders".into(),
        entity_id: None,
        params: [("platform".into(), "nonexistent".into())].into_iter().collect(),
        priority: 50,
    };
    let result = w.execute(task).await;
    assert!(result.is_err());
}

// ════════════════════════════════════════════════════════════════
// 5. analyze_worker tests
// ════════════════════════════════════════════════════════════════

#[test]
fn test_analyze_worker_id_and_type() {
    let w = AnalyzeWorker::new("ana_test");
    assert_eq!(w.worker_id(), "ana_test");
    assert_eq!(w.worker_type(), TaskWorkerType::Analyze);
}

#[test]
fn test_analyze_worker_can_handle() {
    let w = AnalyzeWorker::new("ana");
    assert!(w.can_handle("analyze_kpi"));
    assert!(w.can_handle("analyze_risk"));
    assert!(w.can_handle("analyze_summary"));
    assert!(w.can_handle("analyze_customer"));
    assert!(!w.can_handle("extract_orders"));
    assert!(!w.can_handle("send_email"));
}

#[tokio::test]
async fn test_analyze_worker_execute_summary() {
    let w = AnalyzeWorker::new("ana");
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "analyze_summary".into(),
        entity_id: None,
        params: [("analysis_type".into(), "summary".into())].into_iter().collect(),
        priority: 50,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.task_id, "t1");
    assert_eq!(result.data["analysis_type"], "summary");
}

#[tokio::test]
async fn test_analyze_worker_execute_kpi() {
    let w = AnalyzeWorker::new("ana");
    let task = WorkerTask {
        task_id: "t2".into(),
        task_type: "analyze_kpi".into(),
        entity_id: None,
        params: [("analysis_type".into(), "kpi".into())].into_iter().collect(),
        priority: 50,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.data["total_orders"], 0);
    assert!(result.data.get("total_revenue").is_some());
}

#[tokio::test]
async fn test_analyze_worker_execute_risk() {
    let w = AnalyzeWorker::new("ana");
    let task = WorkerTask {
        task_id: "t3".into(),
        task_type: "analyze_risk".into(),
        entity_id: None,
        params: [("analysis_type".into(), "risk".into())].into_iter().collect(),
        priority: 50,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.data["risk_level"], "low");
}

#[tokio::test]
async fn test_analyze_worker_execute_unknown_type() {
    let w = AnalyzeWorker::new("ana");
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "analyze_kpi".into(),
        entity_id: None,
        params: [("analysis_type".into(), "nonexistent".into())].into_iter().collect(),
        priority: 50,
    };
    let result = w.execute(task).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unknown analysis type"));
}

// ════════════════════════════════════════════════════════════════
// 6. write_worker tests
// ════════════════════════════════════════════════════════════════

#[test]
fn test_write_worker_id_and_type() {
    let w = WriteWorker::new("wr_test");
    assert_eq!(w.worker_id(), "wr_test");
    assert_eq!(w.worker_type(), TaskWorkerType::Write);
}

#[test]
fn test_write_worker_can_handle() {
    let w = WriteWorker::new("wr");
    assert!(w.can_handle("write_quotation"));
    assert!(w.can_handle("write_contract"));
    assert!(w.can_handle("write_packing_list"));
    assert!(w.can_handle("write_bill_of_lading"));
    assert!(w.can_handle("write_report"));
    assert!(w.can_handle("write_invoice"));
    assert!(!w.can_handle("extract_orders"));
    assert!(!w.can_handle("send_email"));
}

#[tokio::test]
async fn test_write_worker_execute_report() {
    let w = WriteWorker::new("wr");
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "write_report".into(),
        entity_id: Some("ord_123".into()),
        params: [
            ("document_type".into(), "report".into()),
            ("format".into(), "pdf".into()),
        ]
        .into_iter()
        .collect(),
        priority: 50,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.data["document_type"], "report");
    assert_eq!(result.data["format"], "pdf");
    assert_eq!(result.data["status"], "generated");
}

#[tokio::test]
async fn test_write_worker_execute_quotation() {
    let w = WriteWorker::new("wr");
    let task = WorkerTask {
        task_id: "t2".into(),
        task_type: "write_quotation".into(),
        entity_id: Some("q_456".into()),
        params: HashMap::new(),
        priority: 80,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    assert!(result.metadata.contains_key("document_type"));
    assert!(result.metadata.contains_key("format"));
}

#[tokio::test]
async fn test_write_worker_default_params() {
    let w = WriteWorker::new("wr");
    let task = WorkerTask {
        task_id: "t3".into(),
        task_type: "write_report".into(),
        entity_id: None,
        params: HashMap::new(),
        priority: 0,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    // defaults: document_type = "report", format = "pdf"
    assert_eq!(result.data["document_type"], "report");
    assert_eq!(result.data["format"], "pdf");
}

// ════════════════════════════════════════════════════════════════
// 7. send_worker tests
// ════════════════════════════════════════════════════════════════

#[test]
fn test_send_worker_id_and_type() {
    let w = SendWorker::new("snd_test");
    assert_eq!(w.worker_id(), "snd_test");
    assert_eq!(w.worker_type(), TaskWorkerType::Send);
}

#[test]
fn test_send_worker_can_handle() {
    let w = SendWorker::new("snd");
    assert!(w.can_handle("send_email"));
    assert!(w.can_handle("send_notification"));
    assert!(w.can_handle("send_alert"));
    assert!(w.can_handle("send_whatsapp"));
    assert!(w.can_handle("send_follow_up"));
    assert!(!w.can_handle("extract_orders"));
    assert!(!w.can_handle("write_report"));
}

#[tokio::test]
async fn test_send_worker_execute_success() {
    let w = SendWorker::new("snd");
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "send_email".into(),
        entity_id: Some("cust_1".into()),
        params: [
            ("channel".into(), "email".into()),
            ("recipient".into(), "buyer@example.com".into()),
            ("subject".into(), "Quotation for Valves".into()),
        ]
        .into_iter()
        .collect(),
        priority: 50,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.data["channel"], "email");
    assert_eq!(result.data["recipient"], "buyer@example.com");
    assert_eq!(result.data["subject"], "Quotation for Valves");
    assert_eq!(result.data["status"], "sent");
}

#[tokio::test]
async fn test_send_worker_execute_missing_recipient() {
    let w = SendWorker::new("snd");
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "send_email".into(),
        entity_id: None,
        params: [("channel".into(), "email".into())].into_iter().collect(),
        priority: 50,
    };
    let result = w.execute(task).await.unwrap();
    assert!(!result.success);
    assert_eq!(result.error.unwrap(), "recipient is required");
}

#[tokio::test]
async fn test_send_worker_whatsapp_channel() {
    let w = SendWorker::new("snd");
    let task = WorkerTask {
        task_id: "t2".into(),
        task_type: "send_whatsapp".into(),
        entity_id: None,
        params: [
            ("channel".into(), "whatsapp".into()),
            ("recipient".into(), "+8613800138000".into()),
            ("subject".into(), "Follow up".into()),
        ]
        .into_iter()
        .collect(),
        priority: 60,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.data["channel"], "whatsapp");
    assert_eq!(result.data["recipient"], "+8613800138000");
}

#[tokio::test]
async fn test_send_worker_default_params() {
    let w = SendWorker::new("snd");
    let task = WorkerTask {
        task_id: "t3".into(),
        task_type: "send_email".into(),
        entity_id: None,
        params: [("recipient".into(), "a@b.com".into())].into_iter().collect(),
        priority: 0,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.data["channel"], "email"); // default
}

// ════════════════════════════════════════════════════════════════
// 8. track_worker tests
// ════════════════════════════════════════════════════════════════

#[test]
fn test_track_worker_id_and_type() {
    let w = TrackWorker::new("trk_test");
    assert_eq!(w.worker_id(), "trk_test");
    assert_eq!(w.worker_type(), TaskWorkerType::Track);
}

#[test]
fn test_track_worker_can_handle() {
    let w = TrackWorker::new("trk");
    assert!(w.can_handle("track_progress"));
    assert!(w.can_handle("track_order"));
    assert!(w.can_handle("track_production"));
    assert!(w.can_handle("track_shipment"));
    assert!(!w.can_handle("extract_orders"));
    assert!(!w.can_handle("send_email"));
}

#[tokio::test]
async fn test_track_worker_execute_update() {
    let w = TrackWorker::new("trk");
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "track_progress".into(),
        entity_id: Some("ord_123".into()),
        params: [
            ("action".into(), "update".into()),
            ("status".into(), "running".into()),
            ("progress".into(), "0.6".into()),
            ("message".into(), "production in progress".into()),
        ]
        .into_iter()
        .collect(),
        priority: 50,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.data["action"], "updated");
    assert!(result.data["tracked_tasks"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn test_track_worker_execute_snapshot() {
    let w = TrackWorker::new("trk");
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "track_progress".into(),
        entity_id: None,
        params: [("action".into(), "snapshot".into())].into_iter().collect(),
        priority: 50,
    };
    let result = w.execute(task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.data["action"], "snapshot");
}

#[tokio::test]
async fn test_track_worker_execute_clear() {
    let w = TrackWorker::new("trk");
    // first add a completed entry
    let update_task = WorkerTask {
        task_id: "u1".into(),
        task_type: "track_progress".into(),
        entity_id: Some("task_a".into()),
        params: [
            ("action".into(), "update".into()),
            ("status".into(), "completed".into()),
            ("progress".into(), "1.0".into()),
            ("message".into(), "done".into()),
        ]
        .into_iter()
        .collect(),
        priority: 50,
    };
    w.execute(update_task).await.unwrap();

    let clear_task = WorkerTask {
        task_id: "c1".into(),
        task_type: "track_progress".into(),
        entity_id: None,
        params: [("action".into(), "clear".into())].into_iter().collect(),
        priority: 50,
    };
    let result = w.execute(clear_task).await.unwrap();
    assert!(result.success);
    assert_eq!(result.data["action"], "cleared");
    assert_eq!(result.data["remaining"], 0);
}

#[tokio::test]
async fn test_track_worker_execute_unknown_action() {
    let w = TrackWorker::new("trk");
    let task = WorkerTask {
        task_id: "t1".into(),
        task_type: "track_progress".into(),
        entity_id: None,
        params: [("action".into(), "bogus".into())].into_iter().collect(),
        priority: 50,
    };
    let result = w.execute(task).await.unwrap();
    assert!(!result.success);
}

#[test]
fn test_progress_tracker_basics() {
    let mut tracker = ProgressTracker::new();
    assert_eq!(tracker.overall_progress(), 0.0);

    tracker.update("t1", TrackTaskStatus::Running, 0.5, "halfway");
    assert_eq!(tracker.entries().len(), 1);
    assert!((tracker.overall_progress() - 0.5).abs() < f64::EPSILON);

    tracker.update("t2", TrackTaskStatus::Completed, 1.0, "done");
    assert_eq!(tracker.entries().len(), 2);
    assert!((tracker.overall_progress() - 0.75).abs() < f64::EPSILON);
}

#[test]
fn test_progress_tracker_update_existing() {
    let mut tracker = ProgressTracker::new();
    tracker.update("t1", TrackTaskStatus::Running, 0.3, "started");
    tracker.update("t1", TrackTaskStatus::Running, 0.7, "almost there");
    assert_eq!(tracker.entries().len(), 1);
    let entry = tracker.get("t1").unwrap();
    assert!((entry.progress - 0.7).abs() < f64::EPSILON);
    assert_eq!(entry.message, "almost there");
}

#[test]
fn test_progress_tracker_clear_completed() {
    let mut tracker = ProgressTracker::new();
    tracker.update("t1", TrackTaskStatus::Completed, 1.0, "done");
    tracker.update("t2", TrackTaskStatus::Running, 0.5, "working");
    tracker.clear_completed();
    assert_eq!(tracker.entries().len(), 1);
    assert!(tracker.get("t1").is_none());
    assert!(tracker.get("t2").is_some());
}

#[test]
fn test_progress_tracker_get_nonexistent() {
    let tracker = ProgressTracker::new();
    assert!(tracker.get("nonexistent").is_none());
}

#[test]
fn test_progress_tracker_overall_empty() {
    let tracker = ProgressTracker::new();
    assert_eq!(tracker.overall_progress(), 0.0);
}

#[test]
fn test_track_task_status_display() {
    assert_eq!(TrackTaskStatus::Pending.to_string(), "Pending");
    assert_eq!(TrackTaskStatus::Running.to_string(), "Running");
    assert_eq!(TrackTaskStatus::Completed.to_string(), "Completed");
    assert_eq!(TrackTaskStatus::Failed.to_string(), "Failed");
}

#[test]
fn test_track_worker_snapshot_returns_current_state() {
    let tracker = Arc::new(std::sync::Mutex::new(ProgressTracker::new()));
    {
        let mut t = tracker.lock().unwrap();
        t.update("t1", TrackTaskStatus::Running, 0.5, "mid");
    }
    let w = TrackWorker::with_tracker("trk_shared", Arc::clone(&tracker));
    let snap = w.snapshot();
    assert_eq!(snap.entries().len(), 1);
    assert!((snap.overall_progress() - 0.5).abs() < f64::EPSILON);
}

// ════════════════════════════════════════════════════════════════
// 9. orchestrator_v2 tests
// ════════════════════════════════════════════════════════════════

#[test]
fn test_orchestrator_config_default() {
    let cfg = OrchestratorConfig::default();
    assert_eq!(cfg.max_parallel_tasks, 8);
    assert_eq!(cfg.task_timeout_secs, 300);
    assert_eq!(cfg.retry_attempts, 3);
    assert!(cfg.enable_aggregation);
}

#[test]
fn test_trade_task_creation() {
    let task = TradeTask::new("quotation", json!({"items": []}));
    assert_eq!(task.task_type, "quotation");
    assert_eq!(task.priority, TaskPriority::Normal);
    assert!(task.dependencies.is_empty());
    assert!(task.metadata.is_empty());
}

#[test]
fn test_trade_task_builder() {
    let dep_id = Uuid::new_v4();
    let task = TradeTask::new("contract_review", json!({}))
        .with_priority(TaskPriority::Critical)
        .with_dependency(dep_id)
        .with_metadata("order_id", "ORD-001");

    assert_eq!(task.priority, TaskPriority::Critical);
    assert_eq!(task.dependencies, vec![dep_id]);
    assert_eq!(task.metadata.get("order_id").unwrap(), "ORD-001");
}

#[test]
fn test_worker_type_from_task_type() {
    assert_eq!(OrchWorkerType::from_task_type("inquiry_parse"), OrchWorkerType::Inquiry);
    assert_eq!(OrchWorkerType::from_task_type("generate_quotation"), OrchWorkerType::Quotation);
    assert_eq!(OrchWorkerType::from_task_type("contract_review"), OrchWorkerType::Contract);
    assert_eq!(OrchWorkerType::from_task_type("production_schedule"), OrchWorkerType::Production);
    assert_eq!(OrchWorkerType::from_task_type("logistics_tracking"), OrchWorkerType::Logistics);
    assert_eq!(OrchWorkerType::from_task_type("finance_payment"), OrchWorkerType::Finance);
    assert_eq!(OrchWorkerType::from_task_type("random_task"), OrchWorkerType::Generic);
}

#[test]
fn test_worker_type_chinese() {
    assert_eq!(OrchWorkerType::from_task_type("询盘处理"), OrchWorkerType::Inquiry);
    assert_eq!(OrchWorkerType::from_task_type("报价生成"), OrchWorkerType::Quotation);
    assert_eq!(OrchWorkerType::from_task_type("合同审核"), OrchWorkerType::Contract);
    assert_eq!(OrchWorkerType::from_task_type("生产调度"), OrchWorkerType::Production);
    assert_eq!(OrchWorkerType::from_task_type("物流跟踪"), OrchWorkerType::Logistics);
    assert_eq!(OrchWorkerType::from_task_type("财务结算"), OrchWorkerType::Finance);
}

#[test]
fn test_orchestrator_router_default_rules() {
    let router = OrchRouter::new();
    let task = TradeTask::new("quotation_generation", json!({}));
    assert_eq!(router.route(&task), OrchWorkerType::Quotation);

    let task = TradeTask::new("logistics_booking", json!({}));
    assert_eq!(router.route(&task), OrchWorkerType::Logistics);

    let task = TradeTask::new("unknown_type", json!({}));
    assert_eq!(router.route(&task), OrchWorkerType::Generic);
}

#[test]
fn test_orchestrator_router_custom_rule() {
    let mut router = OrchRouter::new();
    router.add_rule("custom_domain".into(), OrchWorkerType::Generic);
    let task = TradeTask::new("custom_domain_processor", json!({}));
    assert_eq!(router.route(&task), OrchWorkerType::Generic);
}

#[test]
fn test_task_decomposition_quotation() {
    let orch = TradeOrchestrator::new(orch_config());
    let task = TradeTask::new("quotation_generation", json!({"items": []}));
    let subtasks = orch.decompose_task(&task);

    assert_eq!(subtasks.len(), 4);
    assert!(subtasks[0].task_type.contains("inquiry_analysis"));
    assert!(subtasks[1].task_type.contains("price_calculation"));
    assert!(subtasks[2].task_type.contains("quotation_generation"));
    assert!(subtasks[3].task_type.contains("quotation_review"));

    // dependency chain
    assert!(subtasks[0].dependencies.is_empty());
    assert_eq!(subtasks[1].dependencies.len(), 1);
    assert_eq!(subtasks[2].dependencies.len(), 1);
    assert_eq!(subtasks[3].dependencies.len(), 1);

    // metadata
    for (i, st) in subtasks.iter().enumerate() {
        assert_eq!(st.metadata.get("parent_id").unwrap(), &task.id.to_string());
        assert_eq!(st.metadata.get("subtask_index").unwrap(), &i.to_string());
    }
}

#[test]
fn test_task_decomposition_production() {
    let orch = TradeOrchestrator::new(orch_config());
    let task = TradeTask::new("production_scheduling", json!({}));
    let subtasks = orch.decompose_task(&task);
    assert_eq!(subtasks.len(), 4);
    assert!(subtasks[0].task_type.contains("material_procurement"));
    assert!(subtasks[1].task_type.contains("production_scheduling"));
    assert!(subtasks[2].task_type.contains("quality_inspection"));
    assert!(subtasks[3].task_type.contains("production_tracking"));
}

#[test]
fn test_task_decomposition_contract() {
    let orch = TradeOrchestrator::new(orch_config());
    let task = TradeTask::new("contract_draft", json!({}));
    let subtasks = orch.decompose_task(&task);
    assert_eq!(subtasks.len(), 3);
    assert!(subtasks[0].task_type.contains("contract_draft"));
    assert!(subtasks[1].task_type.contains("contract_review"));
    assert!(subtasks[2].task_type.contains("contract_approval"));
}

#[test]
fn test_task_decomposition_logistics() {
    let orch = TradeOrchestrator::new(orch_config());
    let task = TradeTask::new("shipment_tracking", json!({}));
    let subtasks = orch.decompose_task(&task);
    assert_eq!(subtasks.len(), 4);
    assert!(subtasks[0].task_type.contains("booking_arrangement"));
    assert!(subtasks[1].task_type.contains("customs_declaration"));
    assert!(subtasks[2].task_type.contains("shipment_tracking"));
    assert!(subtasks[3].task_type.contains("document_management"));
}

#[test]
fn test_task_decomposition_finance() {
    let orch = TradeOrchestrator::new(orch_config());
    let task = TradeTask::new("finance_payment", json!({}));
    let subtasks = orch.decompose_task(&task);
    assert_eq!(subtasks.len(), 3);
    assert!(subtasks[0].task_type.contains("payment_verification"));
    assert!(subtasks[1].task_type.contains("settlement_processing"));
    assert!(subtasks[2].task_type.contains("tax_refund_filing"));
}

#[test]
fn test_task_decomposition_inquiry() {
    let orch = TradeOrchestrator::new(orch_config());
    let task = TradeTask::new("inquiry_processing", json!({}));
    let subtasks = orch.decompose_task(&task);
    assert_eq!(subtasks.len(), 3);
    assert!(subtasks[0].task_type.contains("inquiry_parsing"));
    assert!(subtasks[1].task_type.contains("requirement_extraction"));
    assert!(subtasks[2].task_type.contains("inquiry_classification"));
}

#[test]
fn test_task_decomposition_generic() {
    let orch = TradeOrchestrator::new(orch_config());
    let task = TradeTask::new("unknown_task", json!({}));
    let subtasks = orch.decompose_task(&task);
    assert_eq!(subtasks.len(), 3);
    assert!(subtasks[0].task_type.contains("task_analysis"));
    assert!(subtasks[1].task_type.contains("task_execution"));
    assert!(subtasks[2].task_type.contains("task_verification"));
}

#[test]
fn test_result_aggregation_all_success() {
    let orch = TradeOrchestrator::new(orch_config());
    let results = vec![
        OrchWorkerResult {
            worker_type: OrchWorkerType::Inquiry,
            output: json!({"step": "parse"}),
            success: true,
            error: None,
            duration_ms: 100,
        },
        OrchWorkerResult {
            worker_type: OrchWorkerType::Quotation,
            output: json!({"step": "quote"}),
            success: true,
            error: None,
            duration_ms: 200,
        },
    ];
    let aggregated = orch.aggregate_results(results);
    assert_eq!(aggregated.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);
    assert!(aggregated.aggregated.is_some());
    assert_eq!(aggregated.duration_ms, 300);

    let agg = aggregated.aggregated.unwrap();
    assert_eq!(agg["worker_count"], 2);
    assert_eq!(agg["all_success"], true);
    assert_eq!(agg["total_duration_ms"], 300);
}

#[test]
fn test_result_aggregation_partial_failure() {
    let orch = TradeOrchestrator::new(orch_config());
    let results = vec![
        OrchWorkerResult {
            worker_type: OrchWorkerType::Inquiry,
            output: json!({"step": "parse"}),
            success: true,
            error: None,
            duration_ms: 100,
        },
        OrchWorkerResult {
            worker_type: OrchWorkerType::Quotation,
            output: json!({"error": "timeout"}),
            success: false,
            error: Some("timeout".into()),
            duration_ms: 5000,
        },
    ];
    let aggregated = orch.aggregate_results(results);
    assert_eq!(aggregated.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Failed("partial failure".into()));
    let agg = aggregated.aggregated.unwrap();
    assert_eq!(agg["all_success"], false);
}

#[test]
fn test_result_aggregation_empty() {
    let orch = TradeOrchestrator::new(orch_config());
    let aggregated = orch.aggregate_results(vec![]);
    assert_eq!(aggregated.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);
    let agg = aggregated.aggregated.unwrap();
    assert_eq!(agg["worker_count"], 0);
    assert_eq!(agg["total_duration_ms"], 0);
}

#[test]
fn test_orchestrator_stats_initial() {
    let orch = TradeOrchestrator::new(orch_config());
    let stats = orch.stats();
    assert_eq!(stats.total_tasks, 0);
    assert_eq!(stats.completed, 0);
    assert_eq!(stats.failed, 0);
    assert_eq!(stats.in_progress, 0);
    assert_eq!(stats.avg_duration_ms, 0.0);
}

#[tokio::test]
async fn test_orchestrator_execute() {
    let orch = TradeOrchestrator::new(orch_config());
    let task = TradeTask::new("generic_task", json!({"data": "test"}));
    let result = orch.execute(task).await;

    assert_eq!(result.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);
    assert_eq!(result.results.len(), 1);
    assert!(result.results[0].success);
    assert!(result.aggregated.is_some());
    assert!(result.duration_ms > 0);
}

#[tokio::test]
async fn test_orchestrator_execute_parallel() {
    let orch = TradeOrchestrator::new(orch_config());
    let tasks = vec![
        TradeTask::new("inquiry_task", json!({})),
        TradeTask::new("quotation_task", json!({})),
        TradeTask::new("contract_task", json!({})),
    ];
    let results = orch.execute_parallel(tasks).await;
    assert_eq!(results.len(), 3);
    for result in &results {
        assert_eq!(result.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);
        assert!(result.results[0].success);
    }
}

#[tokio::test]
async fn test_orchestrator_execute_parallel_with_dependencies() {
    let orch = TradeOrchestrator::new(orch_config());
    let task_a = TradeTask::new("inquiry_task", json!({}));
    let task_b = TradeTask::new("quotation_task", json!({}))
        .with_dependency(task_a.id);
    let tasks = vec![task_b.clone(), task_a.clone()];
    let results = orch.execute_parallel(tasks).await;
    assert_eq!(results.len(), 2);
    for result in &results {
        assert_eq!(result.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);
    }
}

#[tokio::test]
async fn test_orchestrator_execute_parallel_empty() {
    let orch = TradeOrchestrator::new(orch_config());
    let results = orch.execute_parallel(vec![]).await;
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_orchestrator_stats_after_execution() {
    let orch = TradeOrchestrator::new(orch_config());
    orch.execute(TradeTask::new("task_a", json!({}))).await;
    orch.execute(TradeTask::new("task_b", json!({}))).await;

    let stats = orch.stats();
    assert_eq!(stats.total_tasks, 2);
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.failed, 0);
    assert!(stats.avg_duration_ms > 0.0);
}

#[tokio::test]
async fn test_orchestrator_retry_on_failure() {
    let config = OrchestratorConfig {
        max_parallel_tasks: 2,
        task_timeout_secs: 10,
        retry_attempts: 2,
        enable_aggregation: false,
    };

    let call_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let cc = Arc::clone(&call_count);

    let mut orch = TradeOrchestrator::new(config);
    orch.worker_pool.register_callback_worker(
        OrchWorkerType::Generic,
        move |_task| {
            let count = cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Box::pin(async move {
                if count < 2 {
                    Err(format!("attempt {} failed", count + 1))
                } else {
                    Ok(OrchWorkerResult {
                        worker_type: OrchWorkerType::Generic,
                        output: json!({"attempt": count + 1}),
                        success: true,
                        error: None,
                        duration_ms: 0,
                    })
                }
            })
        },
    );

    let task = TradeTask::new("flaky_task", json!({}));
    let result = orch.execute(task).await;
    assert_eq!(result.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_orchestrator_retry_exhausted() {
    let config = OrchestratorConfig {
        max_parallel_tasks: 2,
        task_timeout_secs: 10,
        retry_attempts: 1,
        enable_aggregation: false,
    };

    let mut orch = TradeOrchestrator::new(config);
    orch.worker_pool.register_callback_worker(
        OrchWorkerType::Generic,
        |_task| {
            Box::pin(async { Err("persistent error".into()) })
        },
    );

    let task = TradeTask::new("always_fail", json!({}));
    let result = orch.execute(task).await;
    assert!(matches!(result.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Failed(_)));
    assert_eq!(result.results.len(), 1);
    assert!(!result.results[0].success);
}

#[tokio::test]
async fn test_orchestrator_task_timeout() {
    let config = OrchestratorConfig {
        max_parallel_tasks: 2,
        task_timeout_secs: 1,
        retry_attempts: 0,
        enable_aggregation: false,
    };

    let mut orch = TradeOrchestrator::new(config);
    orch.worker_pool.register_callback_worker(
        OrchWorkerType::Generic,
        |_task| {
            Box::pin(async {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                Ok(OrchWorkerResult {
                    worker_type: OrchWorkerType::Generic,
                    output: json!({}),
                    success: true,
                    error: None,
                    duration_ms: 0,
                })
            })
        },
    );

    let task = TradeTask::new("slow_task", json!({}));
    let result = orch.execute(task).await;
    assert_eq!(result.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Failed("Task timed out after 1s".into()));
}

#[tokio::test]
async fn test_orchestrator_with_custom_worker() {
    struct CustomWorker;
    #[async_trait::async_trait]
    impl OrchTradeWorker for CustomWorker {
        fn worker_type(&self) -> OrchWorkerType { OrchWorkerType::Inquiry }
        async fn execute(&self, _task: &TradeTask) -> Result<OrchWorkerResult, String> {
            Ok(OrchWorkerResult {
                worker_type: OrchWorkerType::Inquiry,
                output: json!({"custom": true}),
                success: true,
                error: None,
                duration_ms: 0,
            })
        }
    }

    let mut orch = TradeOrchestrator::new(orch_config());
    orch = orch.with_worker(Arc::new(CustomWorker));
    let task = TradeTask::new("inquiry_custom", json!({}));
    let result = orch.execute(task).await;
    assert_eq!(result.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);
    assert_eq!(result.results[0].output["custom"], true);
}

#[tokio::test]
async fn test_orchestrator_decompose_then_execute() {
    let orch = TradeOrchestrator::new(orch_config());
    let task = TradeTask::new("quotation_generation", json!({"items": []}));
    let subtasks = orch.decompose_task(&task);
    let results = orch.execute_parallel(subtasks).await;
    assert_eq!(results.len(), 4);
    for result in &results {
        assert_eq!(result.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);
    }
}

#[tokio::test]
async fn test_orchestrator_is_task_ready() {
    let orch = TradeOrchestrator::new(orch_config());
    let dep = TradeTask::new("dep_task", json!({}));
    let task = TradeTask::new("main_task", json!({})).with_dependency(dep.id);
    assert!(!orch.is_task_ready(&task).await);

    orch.execute(dep).await;
    let result = orch.execute(task).await;
    assert_eq!(result.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);
}

#[tokio::test]
async fn test_orchestrator_task_status() {
    let orch = TradeOrchestrator::new(orch_config());
    let task = TradeTask::new("inquiry_test", json!({}));
    let tid = task.id;
    let result = orch.execute(task).await;
    assert_eq!(result.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);

    let status = orch.task_status(tid).await;
    assert_eq!(status, Some(crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed));
}

#[tokio::test]
async fn test_orchestrator_task_status_nonexistent() {
    let orch = TradeOrchestrator::new(orch_config());
    assert!(orch.task_status(Uuid::new_v4()).await.is_none());
}

#[tokio::test]
async fn test_orchestrator_message_bus() {
    let orch = TradeOrchestrator::new(orch_config());
    // Sender 无法换出 Receiver (现代 tokio 已无 into_stream), 故用新增的
    // take_message_receiver() —— 它同时暴露了「编排器只发不收」这个事实。
    let mut rx = orch
        .take_message_receiver()
        .await
        .expect("receiver available once");

    let task = TradeTask::new("inquiry_test", json!({}));
    let _ = orch.execute(task).await;

    let mut messages = Vec::new();
    while let Ok(Some(msg)) = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        rx.recv(),
    ).await {
        messages.push(msg);
    }

    assert!(!messages.is_empty());
    assert!(matches!(messages[0], OrchMessage::TaskScheduled { .. }));
    assert!(messages.iter().any(|m| matches!(m, OrchMessage::TaskStarted { .. })));
    assert!(messages.iter().any(|m| matches!(m, OrchMessage::TaskCompleted { .. })));
}

#[tokio::test]
async fn test_orchestrator_concurrent_limit() {
    let config = OrchestratorConfig {
        max_parallel_tasks: 2,
        task_timeout_secs: 30,
        retry_attempts: 0,
        enable_aggregation: false,
    };

    let active_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let max_observed = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let ac = Arc::clone(&active_count);
    let mc = Arc::clone(&max_observed);

    let mut orch = TradeOrchestrator::new(config);
    orch.worker_pool.register_callback_worker(
        OrchWorkerType::Generic,
        move |_task| {
            let active = Arc::clone(&ac);
            let max = Arc::clone(&mc);
            Box::pin(async move {
                let current = active.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                max.fetch_max(current, std::sync::atomic::Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                active.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                Ok(OrchWorkerResult {
                    worker_type: OrchWorkerType::Generic,
                    output: json!({}),
                    success: true,
                    error: None,
                    duration_ms: 0,
                })
            })
        },
    );

    let tasks: Vec<_> = (0..6)
        .map(|i| TradeTask::new(format!("task_{}", i), json!({})))
        .collect();
    let _ = orch.execute_parallel(tasks).await;
    let observed = max_observed.load(std::sync::atomic::Ordering::SeqCst);
    assert!(observed <= 2, "Max concurrent should be <= 2, got {}", observed);
}

#[test]
fn test_task_priority_ordering() {
    assert!(TaskPriority::Critical > TaskPriority::High);
    assert!(TaskPriority::High > TaskPriority::Normal);
    assert!(TaskPriority::Normal > TaskPriority::Low);
}

#[test]
fn test_orchestrator_config_builder() {
    let cfg = OrchestratorConfig {
        max_parallel_tasks: 16,
        task_timeout_secs: 600,
        retry_attempts: 5,
        enable_aggregation: false,
    };
    assert_eq!(cfg.max_parallel_tasks, 16);
    assert_eq!(cfg.task_timeout_secs, 600);
    assert_eq!(cfg.retry_attempts, 5);
    assert!(!cfg.enable_aggregation);
}

#[test]
fn test_trade_task_serialization_roundtrip() {
    let task = TradeTask::new("quotation", json!({"items": [1,2,3]}))
        .with_priority(TaskPriority::High)
        .with_metadata("key", "val");

    let json = serde_json::to_string(&task).unwrap();
    let restored: TradeTask = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.task_type, "quotation");
    assert_eq!(restored.priority, TaskPriority::High);
    assert_eq!(restored.metadata.get("key").unwrap(), "val");
}

#[test]
fn test_orchestrator_result_serialization() {
    let result = OrchestratorResult {
        task_id: Uuid::new_v4(),
        status: crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed,
        results: vec![OrchWorkerResult {
            worker_type: OrchWorkerType::Generic,
            output: json!({"ok": true}),
            success: true,
            error: None,
            duration_ms: 42,
        }],
        aggregated: Some(json!({"combined": []})),
        duration_ms: 42,
    };
    let json = serde_json::to_string(&result).unwrap();
    let restored: OrchestratorResult = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.status, crate::l1_action::nt_act::nt_act_trade::orchestrator_v2::TaskStatus::Completed);
    assert_eq!(restored.duration_ms, 42);
}

#[test]
fn test_orchestrator_stats_default() {
    let stats = OrchestratorStats::default();
    assert_eq!(stats.total_tasks, 0);
    assert_eq!(stats.completed, 0);
    assert_eq!(stats.failed, 0);
    assert_eq!(stats.in_progress, 0);
    assert_eq!(stats.avg_duration_ms, 0.0);
    assert!(stats.by_worker_type.is_empty());
}
