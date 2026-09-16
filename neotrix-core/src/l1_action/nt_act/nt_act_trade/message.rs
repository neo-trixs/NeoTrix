//! Trade Message Protocol — 多 Agent 通信协议
//!
//! 定义 NeoTrix trade 子系统中 Agent 间通信的标准消息格式。
//! 支持任务分配/响应、数据流转、命令分发、事件通知四大类消息。
//! 所有消息通过 `MessageHeader` 携带路由元数据，通过 `TradeMessage` 统一分发。

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// 1. 消息优先级 — MessagePriority
// ============================================================

/// 消息优先级，影响路由调度顺序
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

// ============================================================
// 2. 消息头 — MessageHeader
// ============================================================

/// 消息路由元数据，每个 `TradeMessage` 必须携带
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// 消息唯一 ID
    pub id: Uuid,
    /// 关联 ID（用于 request-response 链路追踪）
    pub correlation_id: Option<Uuid>,
    /// 发送方 Agent 标识
    pub source: String,
    /// 接收方 Agent 标识（`"*"` 表示广播）
    pub destination: String,
    /// 消息创建时间戳
    pub timestamp: DateTime<Utc>,
    /// 消息优先级
    pub priority: MessagePriority,
    /// 生存时间（秒），超时后接收方可丢弃
    pub ttl_secs: u64,
}

impl MessageHeader {
    /// 创建新消息头（自动生成 ID 和时间戳）
    pub fn new(source: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            correlation_id: None,
            source: source.into(),
            destination: destination.into(),
            timestamp: Utc::now(),
            priority: MessagePriority::default(),
            ttl_secs: 300,
        }
    }

    /// 设置关联 ID
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置 TTL
    pub fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }

    /// 检查消息是否已过期
    pub fn is_expired(&self) -> bool {
        Utc::now()
            .signed_duration_since(self.timestamp)
            .num_seconds() as u64
            > self.ttl_secs
    }
}

// ============================================================
// 3. 任务状态 — TaskStatus
// ============================================================

/// 任务执行状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// ============================================================
// 4. 任务消息体
// ============================================================

/// 任务请求 — 发送方请求接收方执行特定任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    /// 任务类型标识（如 `"extract_customers"`, `"analyze_data"`）
    pub task_type: String,
    /// 任务负载（JSON 格式，由任务类型决定结构）
    pub payload: serde_json::Value,
    /// 执行超时（秒），0 表示无超时
    pub timeout_secs: u64,
    /// 任务标签（用于分组或过滤）
    pub tags: Vec<String>,
}

/// 任务响应 — 接收方返回任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    /// 原始请求的消息 ID（关联到 `MessageHeader.id`）
    pub request_id: Uuid,
    /// 任务当前状态
    pub status: TaskStatus,
    /// 成功时的结果负载
    pub result: Option<serde_json::Value>,
    /// 失败时的错误信息
    pub error: Option<String>,
    /// 已耗时（秒）
    pub elapsed_secs: f64,
}

/// 任务进度更新 — 长时间运行任务的中间状态推送
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    /// 原始请求的消息 ID
    pub request_id: Uuid,
    /// 进度百分比 (0.0 ~ 100.0)
    pub percent: f64,
    /// 当前阶段描述
    pub stage: String,
    /// 可选的中间结果
    pub partial_result: Option<serde_json::Value>,
}

/// 任务错误 — 任务执行过程中的不可恢复错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskError {
    /// 原始请求的消息 ID
    pub request_id: Uuid,
    /// 错误码（机器可读）
    pub code: String,
    /// 错误描述（人类可读）
    pub message: String,
    /// 是否可重试
    pub retryable: bool,
    /// 错误堆栈（调试用）
    pub stack_trace: Option<String>,
}

// ============================================================
// 5. 数据消息体
// ============================================================

/// 数据提取完成 — 从外部源提取数据后的通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExtracted {
    /// 数据源标识（如平台名、文件路径）
    pub source: String,
    /// 提取的记录数
    pub record_count: u64,
    /// 数据 schema 版本
    pub schema_version: String,
    /// 数据负载引用（实际数据可通过 KB 查询获取）
    pub data_ref: String,
    /// 提取耗时（毫秒）
    pub extraction_time_ms: u64,
}

/// 数据归一化完成 — 数据清洗/标准化后的通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataNormalized {
    /// 原始数据源
    pub source: String,
    /// 归一化后的记录数
    pub record_count: u64,
    /// 丢弃的无效记录数
    pub dropped_count: u64,
    /// 归一化后的数据引用
    pub data_ref: String,
    /// 应用的转换规则列表
    pub transformations: Vec<String>,
}

/// 数据持久化完成 — 数据写入存储后的通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStored {
    /// 存储目标（如 `"kb.products"`, `"sqlite.orders"`）
    pub target: String,
    /// 写入的记录数
    pub record_count: u64,
    /// 存储事务 ID（用于回滚追踪）
    pub transaction_id: Option<Uuid>,
    /// 写入耗时（毫秒）
    pub write_time_ms: u64,
}

// ============================================================
// 6. 命令消息体
// ============================================================

/// 提取客户数据命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractCustomers {
    /// 目标平台列表（如 `["alibaba", "made_in_china"]`）
    pub platforms: Vec<String>,
    /// 筛选条件
    pub filters: serde_json::Value,
    /// 最大提取数量
    pub limit: u64,
    /// 是否增量提取（仅新增/变更）
    pub incremental: bool,
}

/// 分析客户数据命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeCustomers {
    /// 待分析的客户 ID 列表
    pub customer_ids: Vec<String>,
    /// 分析维度（如 `["grading", "segmentation", "churn_risk"]`）
    pub dimensions: Vec<String>,
    /// 分析时间范围起始
    pub since: Option<DateTime<Utc>>,
    /// 分析时间范围结束
    pub until: Option<DateTime<Utc>>,
}

/// 生成报告命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReport {
    /// 报告类型（如 `"monthly_sales"`, `"customer_analysis"`）
    pub report_type: String,
    /// 报告参数
    pub params: serde_json::Value,
    /// 输出格式
    pub output_format: ReportFormat,
    /// 接收方（接收报告文件引用）
    pub recipient: String,
}

/// 报告输出格式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Pdf,
    Xlsx,
    Html,
    Json,
    Markdown,
}

// ============================================================
// 7. 事件消息体
// ============================================================

/// 同步启动事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStarted {
    /// 同步任务 ID
    pub sync_id: Uuid,
    /// 数据源列表
    pub sources: Vec<String>,
    /// 预估总记录数
    pub estimated_records: Option<u64>,
}

/// 同步完成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCompleted {
    /// 同步任务 ID
    pub sync_id: Uuid,
    /// 成功提取的记录数
    pub extracted_count: u64,
    /// 成功写入的记录数
    pub stored_count: u64,
    /// 失败的记录数
    pub failed_count: u64,
    /// 总耗时（秒）
    pub total_secs: f64,
    /// 失败详情（如有）
    pub errors: Vec<String>,
}

/// 告警触发事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggered {
    /// 告警类型（如 `"price_anomaly"`, `"sync_failure"`）
    pub alert_type: String,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 告警描述
    pub message: String,
    /// 触发条件描述
    pub trigger_condition: String,
    /// 关联的数据引用
    pub data_ref: Option<String>,
}

/// 告警严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Critical = 3,
}

// ============================================================
// 8. 核心消息枚举 — TradeMessage
// ============================================================

/// Trade 子系统 Agent 间通信的统一消息类型
///
/// 所有跨 Agent 通信必须通过此枚举进行序列化/反序列化。
/// 消息路由依赖 `MessageHeader.destination` 字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeMessage {
    // ── 任务消息 ──
    /// 任务请求
    TaskRequest(TaskRequest),
    /// 任务响应
    TaskResponse(TaskResponse),
    /// 任务进度更新
    TaskProgress(TaskProgress),
    /// 任务错误
    TaskError(TaskError),

    // ── 数据消息 ──
    /// 数据提取完成
    DataExtracted(DataExtracted),
    /// 数据归一化完成
    DataNormalized(DataNormalized),
    /// 数据持久化完成
    DataStored(DataStored),

    // ── 命令消息 ──
    /// 提取客户数据
    ExtractCustomers(ExtractCustomers),
    /// 分析客户数据
    AnalyzeCustomers(AnalyzeCustomers),
    /// 生成报告
    GenerateReport(GenerateReport),

    // ── 事件消息 ──
    /// 同步启动
    SyncStarted(SyncStarted),
    /// 同步完成
    SyncCompleted(SyncCompleted),
    /// 告警触发
    AlertTriggered(AlertTriggered),
}

impl TradeMessage {
    /// 获取消息类型的字符串标签（用于日志和路由匹配）
    pub fn type_tag(&self) -> &'static str {
        match self {
            Self::TaskRequest(_) => "task.request",
            Self::TaskResponse(_) => "task.response",
            Self::TaskProgress(_) => "task.progress",
            Self::TaskError(_) => "task.error",
            Self::DataExtracted(_) => "data.extracted",
            Self::DataNormalized(_) => "data.normalized",
            Self::DataStored(_) => "data.stored",
            Self::ExtractCustomers(_) => "command.extract_customers",
            Self::AnalyzeCustomers(_) => "command.analyze_customers",
            Self::GenerateReport(_) => "command.generate_report",
            Self::SyncStarted(_) => "event.sync_started",
            Self::SyncCompleted(_) => "event.sync_completed",
            Self::AlertTriggered(_) => "event.alert_triggered",
        }
    }

    /// 检查消息是否为任务类消息
    pub fn is_task_message(&self) -> bool {
        matches!(
            self,
            Self::TaskRequest(_)
                | Self::TaskResponse(_)
                | Self::TaskProgress(_)
                | Self::TaskError(_)
        )
    }

    /// 检查消息是否为数据类消息
    pub fn is_data_message(&self) -> bool {
        matches!(
            self,
            Self::DataExtracted(_) | Self::DataNormalized(_) | Self::DataStored(_)
        )
    }

    /// 检查消息是否为命令类消息
    pub fn is_command_message(&self) -> bool {
        matches!(
            self,
            Self::ExtractCustomers(_)
                | Self::AnalyzeCustomers(_)
                | Self::GenerateReport(_)
        )
    }

    /// 检查消息是否为事件类消息
    pub fn is_event_message(&self) -> bool {
        matches!(
            self,
            Self::SyncStarted(_) | Self::SyncCompleted(_) | Self::AlertTriggered(_)
        )
    }
}

// ============================================================
// 9. 信封 — Envelope（Header + Body 打包）
// ============================================================

/// 消息信封 — 将 `MessageHeader` 和 `TradeMessage` 打包为可传输单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// 消息头（路由元数据）
    pub header: MessageHeader,
    /// 消息体
    pub body: TradeMessage,
}

impl Envelope {
    /// 创建新信封
    pub fn new(header: MessageHeader, body: TradeMessage) -> Self {
        Self { header, body }
    }

    /// 便捷构造：从源/目标/消息体直接创建
    pub fn create(
        source: impl Into<String>,
        destination: impl Into<String>,
        body: TradeMessage,
    ) -> Self {
        Self {
            header: MessageHeader::new(source, destination),
            body,
        }
    }

    /// 设置关联 ID（用于 request-response 链路）
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.header.correlation_id = Some(id);
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.header.priority = priority;
        self
    }

    /// 设置 TTL
    pub fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.header.ttl_secs = ttl_secs;
        self
    }

    /// 检查信封是否已过期
    pub fn is_expired(&self) -> bool {
        self.header.is_expired()
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// ============================================================
// 10. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── MessageHeader ──

    #[test]
    fn test_message_header_creation() {
        let header = MessageHeader::new("agent_a", "agent_b");
        assert_eq!(header.source, "agent_a");
        assert_eq!(header.destination, "agent_b");
        assert_eq!(header.priority, MessagePriority::Normal);
        assert_eq!(header.ttl_secs, 300);
        assert!(header.correlation_id.is_none());
    }

    #[test]
    fn test_message_header_builder() {
        let header = MessageHeader::new("src", "dst")
            .with_correlation_id(Uuid::new_v4())
            .with_priority(MessagePriority::High)
            .with_ttl(60);

        assert!(header.correlation_id.is_some());
        assert_eq!(header.priority, MessagePriority::High);
        assert_eq!(header.ttl_secs, 60);
    }

    #[test]
    fn test_message_header_expiry() {
        let header = MessageHeader {
            id: Uuid::new_v4(),
            correlation_id: None,
            source: "src".into(),
            destination: "dst".into(),
            timestamp: Utc::now() - chrono::Duration::seconds(10),
            priority: MessagePriority::Normal,
            ttl_secs: 5,
        };
        assert!(header.is_expired());

        let header = MessageHeader {
            ttl_secs: 60,
            ..header
        };
        assert!(!header.is_expired());
    }

    // ── TradeMessage ──

    #[test]
    fn test_trade_message_type_tags() {
        let msg = TradeMessage::TaskRequest(TaskRequest {
            task_type: "test".into(),
            payload: serde_json::json!({}),
            timeout_secs: 30,
            tags: vec![],
        });
        assert_eq!(msg.type_tag(), "task.request");
        assert!(msg.is_task_message());
        assert!(!msg.is_data_message());
        assert!(!msg.is_command_message());
        assert!(!msg.is_event_message());

        let msg = TradeMessage::DataExtracted(DataExtracted {
            source: "alibaba".into(),
            record_count: 100,
            schema_version: "1.0".into(),
            data_ref: "kb.extracted.001".into(),
            extraction_time_ms: 500,
        });
        assert_eq!(msg.type_tag(), "data.extracted");
        assert!(msg.is_data_message());

        let msg = TradeMessage::ExtractCustomers(ExtractCustomers {
            platforms: vec!["alibaba".into()],
            filters: serde_json::json!({}),
            limit: 100,
            incremental: false,
        });
        assert_eq!(msg.type_tag(), "command.extract_customers");
        assert!(msg.is_command_message());

        let msg = TradeMessage::SyncStarted(SyncStarted {
            sync_id: Uuid::new_v4(),
            sources: vec!["platform_a".into()],
            estimated_records: Some(1000),
        });
        assert_eq!(msg.type_tag(), "event.sync_started");
        assert!(msg.is_event_message());
    }

    // ── Envelope serialization round-trip ──

    #[test]
    fn test_envelope_json_roundtrip() {
        let envelope = Envelope::create(
            "agent_extractor",
            "agent_analyzer",
            TradeMessage::TaskRequest(TaskRequest {
                task_type: "analyze_customers".into(),
                payload: serde_json::json!({"customer_ids": ["C001", "C002"]}),
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
                assert_eq!(req.tags, vec!["priority:high"]);
            }
            _ => panic!("Expected TaskRequest"),
        }
    }

    #[test]
    fn test_envelope_correlation_roundtrip() {
        let request = Envelope::create(
            "orchestrator",
            "extractor",
            TradeMessage::ExtractCustomers(ExtractCustomers {
                platforms: vec!["alibaba".into(), "globalsources".into()],
                filters: serde_json::json!({"region": "asia"}),
                limit: 500,
                incremental: true,
            }),
        );

        // 模拟响应：携带 correlation_id 链接到原始请求
        let response = Envelope::create(
            "extractor",
            "orchestrator",
            TradeMessage::TaskResponse(TaskResponse {
                request_id: request.header.id,
                status: TaskStatus::Completed,
                result: Some(serde_json::json!({"extracted": 42})),
                error: None,
                elapsed_secs: 12.5,
            }),
        )
        .with_correlation_id(request.header.id);

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

    // ── 各消息体序列化 ──

    #[test]
    fn test_task_progress_serialization() {
        let msg = TradeMessage::TaskProgress(TaskProgress {
            request_id: Uuid::new_v4(),
            percent: 67.5,
            stage: "extracting customers".into(),
            partial_result: Some(serde_json::json!({"count": 30})),
        });
        let json = serde_json::to_string(&msg).unwrap();
        let restored: TradeMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.type_tag(), "task.progress");
    }

    #[test]
    fn test_task_error_serialization() {
        let msg = TradeMessage::TaskError(TaskError {
            request_id: Uuid::new_v4(),
            code: "EXTRACTION_TIMEOUT".into(),
            message: "Platform response exceeded 30s timeout".into(),
            retryable: true,
            stack_trace: None,
        });
        let json = serde_json::to_string(&msg).unwrap();
        let restored: TradeMessage = serde_json::from_str(&json).unwrap();
        match &restored {
            TradeMessage::TaskError(e) => {
                assert_eq!(e.code, "EXTRACTION_TIMEOUT");
                assert!(e.retryable);
            }
            _ => panic!("Expected TaskError"),
        }
    }

    #[test]
    fn test_data_messages_serialization() {
        let msgs = vec![
            TradeMessage::DataNormalized(DataNormalized {
                source: "alibaba".into(),
                record_count: 95,
                dropped_count: 5,
                data_ref: "kb.normalized.001".into(),
                transformations: vec!["dedup".into(), "normalize_phone".into()],
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
    fn test_command_messages_serialization() {
        let msgs = vec![
            TradeMessage::AnalyzeCustomers(AnalyzeCustomers {
                customer_ids: vec!["C001".into(), "C002".into()],
                dimensions: vec!["grading".into(), "segmentation".into()],
                since: None,
                until: None,
            }),
            TradeMessage::GenerateReport(GenerateReport {
                report_type: "monthly_sales".into(),
                params: serde_json::json!({"month": "2026-09"}),
                output_format: ReportFormat::Pdf,
                recipient: "manager@company.com".into(),
            }),
        ];

        for msg in msgs {
            let json = serde_json::to_string(&msg).unwrap();
            let restored: TradeMessage = serde_json::from_str(&json).unwrap();
            assert_eq!(restored.type_tag(), msg.type_tag());
        }
    }

    #[test]
    fn test_event_messages_serialization() {
        let msgs = vec![
            TradeMessage::SyncCompleted(SyncCompleted {
                sync_id: Uuid::new_v4(),
                extracted_count: 1000,
                stored_count: 980,
                failed_count: 20,
                total_secs: 45.2,
                errors: vec!["timeout on record 501".into()],
            }),
            TradeMessage::AlertTriggered(AlertTriggered {
                alert_type: "price_anomaly".into(),
                severity: AlertSeverity::Warning,
                message: "Valve price 40% below market average".into(),
                trigger_condition: "unit_price < market_avg * 0.6".into(),
                data_ref: Some("kb.prices.P001".into()),
            }),
        ];

        for msg in msgs {
            let json = serde_json::to_string(&msg).unwrap();
            let restored: TradeMessage = serde_json::from_str(&json).unwrap();
            assert_eq!(restored.type_tag(), msg.type_tag());
        }
    }

    // ── MessagePriority ordering ──

    #[test]
    fn test_priority_ordering() {
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }
}
