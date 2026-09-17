//! Trade Engine Traits — 统一 Engine 契约层
//!
//! 定义所有外贸 Engine 的公共 trait、类型枚举、健康状态和性能指标。
//! 支持多态调用 (dyn TradeEngine) + 动态注册表 (TradeEngineRegistry)。
//!
//! 设计原则:
//! - 零 unsafe (#\[forbid(unsafe_code\)])
//! - trait 要求 Send + Sync，可安全跨线程持有 Arc<dyn TradeEngine>
//! - 健康检查和指标采集为异步，适配生产环境 IO

#![forbid(unsafe_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// 1. Engine 类型枚举
// ============================================================

/// Engine 类型分类 — 覆盖外贸全链路
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineType {
    /// 客户关系管理
    Crm,
    /// 邮件集成
    Email,
    /// 销售管道
    Pipeline,
    /// 单证管理
    Document,
    /// 任务/日历
    Task,
    /// 数据看板
    Dashboard,
    /// 供应商评估
    SupplierMgmt,
    /// 财务合规
    Finance,
    /// 生产跟踪
    Production,
    /// 物流管理
    Logistics,
}

impl std::fmt::Display for EngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Crm => write!(f, "CRM"),
            Self::Email => write!(f, "Email"),
            Self::Pipeline => write!(f, "Pipeline"),
            Self::Document => write!(f, "Document"),
            Self::Task => write!(f, "Task"),
            Self::Dashboard => write!(f, "Dashboard"),
            Self::SupplierMgmt => write!(f, "SupplierMgmt"),
            Self::Finance => write!(f, "Finance"),
            Self::Production => write!(f, "Production"),
            Self::Logistics => write!(f, "Logistics"),
        }
    }
}

// ============================================================
// 2. 健康状态
// ============================================================

/// Engine 健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    /// 是否健康
    pub healthy: bool,
    /// 状态消息
    pub message: String,
    /// 最近检查时间 (epoch seconds)
    pub last_check: u64,
    /// 运行时间 (秒)
    pub uptime_secs: u64,
}

impl EngineStatus {
    /// 构造健康状态
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            healthy: true,
            message: message.into(),
            last_check: now_epoch(),
            uptime_secs: 0,
        }
    }

    /// 构造不健康状态
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            healthy: false,
            message: message.into(),
            last_check: now_epoch(),
            uptime_secs: 0,
        }
    }
}

// ============================================================
// 3. 性能指标
// ============================================================

/// Engine 性能指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineMetrics {
    /// 总请求次数
    pub total_requests: u64,
    /// 成功请求次数
    pub successful_requests: u64,
    /// 失败请求次数
    pub failed_requests: u64,
    /// 平均响应时间 (毫秒)
    pub avg_response_time_ms: f64,
    /// 最近活动时间 (epoch seconds)
    pub last_activity: Option<u64>,
}

impl EngineMetrics {
    /// 记录一次请求
    pub fn record_request(&mut self, success: bool, elapsed_ms: f64) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
        // 滚动平均
        let n = self.total_requests as f64;
        self.avg_response_time_ms =
            self.avg_response_time_ms * ((n - 1.0) / n) + elapsed_ms / n;
        self.last_activity = Some(now_epoch());
    }

    /// 成功率 (0.0 - 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 1.0;
        }
        self.successful_requests as f64 / self.total_requests as f64
    }
}

// ============================================================
// 4. Engine 信息 (注册表用)
// ============================================================

/// 引擎注册信息 (不持有 Engine 实例)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    /// 引擎 ID
    pub engine_id: String,
    /// 引擎类型
    pub engine_type: EngineType,
    /// 引擎显示名称
    pub engine_name: String,
}

// ============================================================
// 5. TradeEngine trait
// ============================================================

/// 所有外贸 Engine 的公共契约
#[async_trait]
pub trait TradeEngine: Send + Sync {
    /// 引擎唯一 ID
    fn engine_id(&self) -> &str;

    /// 引擎类型
    fn engine_type(&self) -> EngineType;

    /// 引擎显示名称
    fn engine_name(&self) -> &str;

    /// 健康检查
    async fn health_check(&self) -> EngineStatus;

    /// 获取性能指标
    fn metrics(&self) -> EngineMetrics;

    /// 初始化引擎 (加载数据、连接外部服务)
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 关闭引擎 (释放资源)
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

// ============================================================
// 6. TradeEngineRegistry
// ============================================================

/// Engine 注册表 — 动态管理 Engine 实例，支持按 ID/类型查找
pub struct TradeEngineRegistry {
    engines: HashMap<String, Arc<dyn TradeEngine>>,
    type_index: HashMap<EngineType, Vec<String>>,
}

impl Default for TradeEngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeEngineRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// 注册 Engine (相同 ID 覆盖旧实例)
    pub fn register(&mut self, engine: Arc<dyn TradeEngine>) {
        let id = engine.engine_id().to_string();
        let etype = engine.engine_type();
        // 覆盖时清理旧类型索引
        if let Some(old) = self.engines.get(&id) {
            let old_type = old.engine_type();
            if let Some(ids) = self.type_index.get_mut(&old_type) {
                ids.retain(|x| x != &id);
            }
        }
        self.engines.insert(id.clone(), engine);
        self.type_index.entry(etype).or_default().push(id);
    }

    /// 按 ID 获取 Engine
    pub fn get(&self, engine_id: &str) -> Option<Arc<dyn TradeEngine>> {
        self.engines.get(engine_id).cloned()
    }

    /// 按类型获取所有 Engine
    pub fn get_by_type(&self, engine_type: EngineType) -> Vec<Arc<dyn TradeEngine>> {
        self.type_index
            .get(&engine_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.engines.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 列出所有已注册引擎信息
    pub fn list_engines(&self) -> Vec<EngineInfo> {
        self.engines
            .values()
            .map(|e| EngineInfo {
                engine_id: e.engine_id().to_string(),
                engine_type: e.engine_type(),
                engine_name: e.engine_name().to_string(),
            })
            .collect()
    }

    /// 已注册引擎数量
    pub fn len(&self) -> usize {
        self.engines.len()
    }

    /// 注册表是否为空
    pub fn is_empty(&self) -> bool {
        self.engines.is_empty()
    }

    /// 对所有引擎执行健康检查
    pub async fn health_check_all(&self) -> HashMap<String, EngineStatus> {
        let mut results = HashMap::new();
        for (id, engine) in &self.engines {
            let status = engine.health_check().await;
            results.insert(id.clone(), status);
        }
        results
    }

    /// 聚合所有引擎指标
    pub fn aggregate_metrics(&self) -> EngineMetrics {
        let mut agg = EngineMetrics::default();
        for engine in self.engines.values() {
            let m = engine.metrics();
            agg.total_requests += m.total_requests;
            agg.successful_requests += m.successful_requests;
            agg.failed_requests += m.failed_requests;
            if let Some(act) = m.last_activity {
                agg.last_activity = Some(match agg.last_activity {
                    Some(prev) => prev.max(act),
                    None => act,
                });
            }
        }
        // 重新计算平均响应时间
        if agg.total_requests > 0 {
            let mut weighted_sum = 0.0;
            for engine in self.engines.values() {
                let m = engine.metrics();
                weighted_sum += m.avg_response_time_ms * m.total_requests as f64;
            }
            agg.avg_response_time_ms = weighted_sum / agg.total_requests as f64;
        }
        agg
    }
}

// ============================================================
// 7. 辅助函数
// ============================================================

/// 当前 epoch 秒数
fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ============================================================
// 8. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── Mock Engine 用于测试 ──

    struct MockEngine {
        id: String,
        name: String,
        etype: EngineType,
        metrics: EngineMetrics,
    }

    impl MockEngine {
        fn new(id: &str, name: &str, etype: EngineType) -> Self {
            Self {
                id: id.into(),
                name: name.into(),
                etype,
                metrics: EngineMetrics::default(),
            }
        }
    }

    #[async_trait]
    impl TradeEngine for MockEngine {
        fn engine_id(&self) -> &str {
            &self.id
        }
        fn engine_type(&self) -> EngineType {
            self.etype
        }
        fn engine_name(&self) -> &str {
            &self.name
        }
        async fn health_check(&self) -> EngineStatus {
            EngineStatus::healthy("ok")
        }
        fn metrics(&self) -> EngineMetrics {
            self.metrics.clone()
        }
        async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_registry_register_and_get() {
        let mut reg = TradeEngineRegistry::new();
        assert!(reg.is_empty());

        let engine = Arc::new(MockEngine::new("crm-1", "CRM Engine", EngineType::Crm));
        reg.register(engine);
        assert_eq!(reg.len(), 1);

        let found = reg.get("crm-1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().engine_name(), "CRM Engine");
    }

    #[tokio::test]
    async fn test_registry_get_by_type() {
        let mut reg = TradeEngineRegistry::new();
        reg.register(Arc::new(MockEngine::new("crm-1", "CRM", EngineType::Crm)));
        reg.register(Arc::new(MockEngine::new("email-1", "Email", EngineType::Email)));
        reg.register(Arc::new(MockEngine::new("crm-2", "CRM 2", EngineType::Crm)));

        let crm_engines = reg.get_by_type(EngineType::Crm);
        assert_eq!(crm_engines.len(), 2);

        let email_engines = reg.get_by_type(EngineType::Email);
        assert_eq!(email_engines.len(), 1);

        let empty = reg.get_by_type(EngineType::Logistics);
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn test_registry_overwrite() {
        let mut reg = TradeEngineRegistry::new();
        reg.register(Arc::new(MockEngine::new("crm-1", "V1", EngineType::Crm)));
        reg.register(Arc::new(MockEngine::new("crm-1", "V2", EngineType::Crm)));
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.get("crm-1").unwrap().engine_name(), "V2");
        // 类型索引不应有重复
        assert_eq!(reg.get_by_type(EngineType::Crm).len(), 1);
    }

    #[tokio::test]
    async fn test_registry_list_engines() {
        let mut reg = TradeEngineRegistry::new();
        reg.register(Arc::new(MockEngine::new("a", "A", EngineType::Crm)));
        reg.register(Arc::new(MockEngine::new("b", "B", EngineType::Email)));

        let list = reg.list_engines();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|e| e.engine_id == "a"));
        assert!(list.iter().any(|e| e.engine_id == "b"));
    }

    #[tokio::test]
    async fn test_health_check_all() {
        let mut reg = TradeEngineRegistry::new();
        reg.register(Arc::new(MockEngine::new("a", "A", EngineType::Crm)));
        reg.register(Arc::new(MockEngine::new("b", "B", EngineType::Email)));

        let statuses = reg.health_check_all().await;
        assert_eq!(statuses.len(), 2);
        assert!(statuses.get("a").unwrap().healthy);
        assert!(statuses.get("b").unwrap().healthy);
    }

    #[test]
    fn test_engine_metrics_record() {
        let mut m = EngineMetrics::default();
        m.record_request(true, 10.0);
        m.record_request(true, 20.0);
        m.record_request(false, 5.0);
        assert_eq!(m.total_requests, 3);
        assert_eq!(m.successful_requests, 2);
        assert_eq!(m.failed_requests, 1);
        assert!((m.success_rate() - 2.0 / 3.0).abs() < 0.001);
        assert!(m.avg_response_time_ms > 0.0);
    }

    #[test]
    fn test_aggregate_metrics() {
        let mut reg = TradeEngineRegistry::new();
        let mut e1 = MockEngine::new("a", "A", EngineType::Crm);
        e1.metrics.record_request(true, 10.0);
        e1.metrics.record_request(true, 20.0);
        reg.register(Arc::new(e1));

        let mut e2 = MockEngine::new("b", "B", EngineType::Email);
        e2.metrics.record_request(true, 30.0);
        reg.register(Arc::new(e2));

        let agg = reg.aggregate_metrics();
        assert_eq!(agg.total_requests, 3);
        assert_eq!(agg.successful_requests, 3);
        assert_eq!(agg.failed_requests, 0);
    }

    #[test]
    fn test_engine_type_display() {
        assert_eq!(EngineType::Crm.to_string(), "CRM");
        assert_eq!(EngineType::Email.to_string(), "Email");
        assert_eq!(EngineType::Logistics.to_string(), "Logistics");
    }

    #[test]
    fn test_engine_status_constructors() {
        let h = EngineStatus::healthy("running");
        assert!(h.healthy);
        assert_eq!(h.message, "running");

        let u = EngineStatus::unhealthy("timeout");
        assert!(!u.healthy);
        assert_eq!(u.message, "timeout");
    }
}
