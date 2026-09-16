/// NT-CORE Heartbeat — 统一系统健康聚合器
///
/// 聚合所有子系统的健康状态，提供统一的健康检查接口。
/// 支持时间衰减：超过 TTL 的健康报告自动降级为 Unknown。
/// 生成 SystemHealthSnapshot 供 GWT 注意力调制使用。

use std::collections::HashMap;
use std::time::{Duration, Instant};

use neotrix_types::shared::HealthStatus;

/// 默认 TTL：5 分钟内未更新的组件降级为 Unknown
const DEFAULT_TTL: Duration = Duration::from_secs(300);

/// WHALE metrics — tracked in SystemHealthSnapshot for GWT weighting
#[derive(Debug, Clone)]
pub struct WhaleMetrics {
    /// Current WHALE optimization phase (weight_update or harness_search)
    pub current_phase: String,
    /// Duration of the current phase in milliseconds
    pub phase_duration_ms: u64,
    /// Recent improvement rate (0.0 to 1.0)
    pub improvement_rate: f64,
    /// Ratio of harness_search cycles to weight_update cycles
    pub harness_weight_ratio: f64,
    /// Number of adaptive phase switches performed
    pub adaptive_switch_count: u64,
}

impl Default for WhaleMetrics {
    fn default() -> Self {
        Self {
            current_phase: "weight_update".to_string(),
            phase_duration_ms: 0,
            improvement_rate: 0.0,
            harness_weight_ratio: 0.0,
            adaptive_switch_count: 0,
        }
    }
}

/// 健康报告
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 组件健康状态（带时间戳）
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_updated: Instant,
}

/// 系统健康快照 — 供 GWT 注意力调制使用
///
/// 每个字段对应一个子系统的健康分数 [0.0, 1.0]。
/// GWT 用这些分数调制对应专家的激活权重：
/// - 健康分数高 → 专家权重增加（更多注意力）
/// - 健康分数低 → 专家权重降低（减少注意力）
#[derive(Debug, Clone)]
pub struct SystemHealthSnapshot {
    /// 编译健康度（cargo check/test 通过率）
    pub compilation: f64,
    /// 测试健康度（单元测试/集成测试通过率）
    pub testing: f64,
    /// KB 健康度（查询成功率/索引完整性）
    pub kb_health: f64,
    /// EventBus 健康度（事件投递成功率）
    pub eventbus: f64,
    /// 模块健康度（各模块 SelfTest 通过率）
    pub modules: f64,
    /// 整体健康度（加权平均）
    pub overall: f64,
    /// 快照时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SystemHealthSnapshot {
    /// 从 HealthReport 生成快照，计算各维度健康分数
    pub fn from_report(report: &HealthReport) -> Self {
        let mut compilation = 0.0;
        let mut testing = 0.0;
        let mut kb_health = 0.0;
        let mut eventbus = 0.0;
        let mut modules = 0.0;
        let mut module_count = 0u32;

        for (name, component) in &report.components {
            let score = match component.status {
                HealthStatus::Healthy => 1.0,
                HealthStatus::Degraded => 0.5,
                HealthStatus::Unhealthy => 0.0,
                HealthStatus::Unknown => 0.3, // 未知状态给中间分
            };

            // 按名称分类到各维度
            if name.contains("compile") || name.contains("build") {
                compilation = score;
            } else if name.contains("test") || name.contains("self_test") {
                testing = score;
            } else if name.contains("kb") || name.contains("memory") {
                kb_health = score;
            } else if name.contains("event") || name.contains("bus") {
                eventbus = score;
            } else {
                modules += score;
                module_count += 1;
            }
        }

        // 如果没有模块数据，用整体状态填充
        if module_count == 0 {
            modules = match report.status {
                HealthStatus::Healthy => 1.0,
                HealthStatus::Degraded => 0.5,
                HealthStatus::Unhealthy => 0.0,
                HealthStatus::Unknown => 0.3,
            };
        } else {
            modules /= module_count as f64;
        }

        // 加权平均：编译 25%, 测试 25%, KB 20%, EventBus 15%, 模块 15%
        let overall = compilation * 0.25
            + testing * 0.25
            + kb_health * 0.20
            + eventbus * 0.15
            + modules * 0.15;

        Self {
            compilation,
            testing,
            kb_health,
            eventbus,
            modules,
            overall,
            timestamp: report.timestamp,
        }
    }

    /// 转换为 GWT 专家权重调整向量
    ///
    /// 返回 (专家名, 权重调整量) 列表。
    /// 正值 = 增加注意力，负值 = 减少注意力。
    ///
    /// **当前映射局限**: 7 个 NT-* 域只有 5 个健康维度，部分域共享同一分数。
    /// nt_act/nt_world/nt_shield 共用 modules 分数 — 真实实现需为每个域
    /// 维护独立健康信号（如 nt_act 需要工具调用成功率，nt_world 需要爬取成功率）。
    pub fn to_gwt_weights(&self) -> Vec<(String, f64)> {
        vec![
            ("nt_core".into(), self.compilation - 0.5),
            ("nt_mind".into(), self.testing - 0.5),
            ("nt_memory".into(), self.kb_health - 0.5),
            ("nt_io".into(), self.eventbus - 0.5),
            // STUB: 以下三域共享 modules 分数 — 不反映各自独立健康状态
            // 真实实现需每域独立信号源 (nt_act: 工具调用率, nt_world: 爬取成功率, nt_shield: 安全事件率)
            ("nt_act".into(), self.modules - 0.5),
            ("nt_world".into(), self.modules - 0.5),
            ("nt_shield".into(), self.modules - 0.5),
        ]
    }

    /// Add WHALE metrics to the health snapshot
    ///
    /// This injects WHALE-specific signals into the GWT weight calculation,
    /// allowing the attention router to factor in optimization phase health.
    /// Harness optimization can substitute for model optimization,
    /// validating NeoTrix's focus on GWT routing optimization over model scaling.
    pub fn with_whale_metrics(mut self, whale: &WhaleMetrics) -> Self {
        self.compilation = self.compilation.max(whale.improvement_rate);
        self.overall = ((self.compilation * 0.25
            + self.testing * 0.25
            + self.kb_health * 0.20
            + self.eventbus * 0.15
            + self.modules * 0.15)
            .min(1.0))
            .max(0.0);
        self
    }
}

/// 健康聚合器
pub struct HeartbeatAggregator {
    components: HashMap<String, ComponentHealth>,
    /// 组件报告的 TTL（超过此时间未更新则降级）
    ttl: Duration,
}

impl Default for HeartbeatAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl HeartbeatAggregator {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            ttl: DEFAULT_TTL,
        }
    }

    /// 设置自定义 TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// 记录组件健康状态
    pub fn record(&mut self, name: &str, status: HealthStatus, message: Option<String>) {
        self.components.insert(
            name.to_string(),
            ComponentHealth {
                name: name.to_string(),
                status,
                message,
                last_updated: Instant::now(),
            },
        );
    }

    /// 应用时间衰减：超过 TTL 的组件降级为 Unknown
    fn apply_decay(&self, component: &ComponentHealth) -> HealthStatus {
        if component.last_updated.elapsed() > self.ttl {
            HealthStatus::Unknown
        } else {
            component.status.clone()
        }
    }

    /// 生成健康报告（带时间衰减）
    pub fn report(&self) -> HealthReport {
        let mut overall_status = HealthStatus::Healthy;
        let mut components = HashMap::new();

        for (name, component) in &self.components {
            let effective_status = self.apply_decay(component);
            if effective_status == HealthStatus::Unhealthy {
                overall_status = HealthStatus::Unhealthy;
            } else if effective_status == HealthStatus::Degraded
                && overall_status != HealthStatus::Unhealthy
            {
                overall_status = HealthStatus::Degraded;
            } else if effective_status == HealthStatus::Unknown
                && overall_status == HealthStatus::Healthy
            {
                overall_status = HealthStatus::Unknown;
            }

            components.insert(
                name.clone(),
                ComponentHealth {
                    name: component.name.clone(),
                    status: effective_status,
                    message: component.message.clone(),
                    last_updated: component.last_updated,
                },
            );
        }

        HealthReport {
            status: overall_status,
            components,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 生成 SystemHealthSnapshot 供 GWT 使用
    pub fn snapshot(&self) -> SystemHealthSnapshot {
        let report = self.report();
        SystemHealthSnapshot::from_report(&report)
    }

    /// 获取 GWT 权重调整建议
    pub fn gwt_weights(&self) -> Vec<(String, f64)> {
        self.snapshot().to_gwt_weights()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_basic() {
        let mut hb = HeartbeatAggregator::new();
        hb.record("nt_core", HealthStatus::Healthy, None);
        hb.record("nt_memory", HealthStatus::Degraded, Some("slow queries".into()));

        let report = hb.report();
        assert_eq!(report.status, HealthStatus::Degraded);
        assert_eq!(report.components.len(), 2);
    }

    #[test]
    fn test_time_decay() {
        let mut hb = HeartbeatAggregator::new()
            .with_ttl(Duration::from_millis(1));

        hb.record("nt_core", HealthStatus::Healthy, None);

        let report = hb.report();
        assert_eq!(report.status, HealthStatus::Healthy);

        std::thread::sleep(Duration::from_millis(5));

        let report = hb.report();
        assert_eq!(report.status, HealthStatus::Unknown);
    }

    #[test]
    fn test_system_health_snapshot() {
        let mut hb = HeartbeatAggregator::new();
        hb.record("compile_check", HealthStatus::Healthy, None);
        hb.record("self_test", HealthStatus::Healthy, None);
        hb.record("kb_query", HealthStatus::Degraded, None);

        let snapshot = hb.snapshot();
        assert!(snapshot.compilation > 0.9, "compilation should be healthy");
        assert!(snapshot.testing > 0.9, "testing should be healthy");
        assert!(snapshot.kb_health < 0.6, "kb should be degraded");
        assert!(snapshot.overall > 0.0 && snapshot.overall <= 1.0);
    }

    #[test]
    fn test_gwt_weights_produce_vector() {
        let mut hb = HeartbeatAggregator::new();
        hb.record("nt_core", HealthStatus::Healthy, None);
        hb.record("nt_memory", HealthStatus::Unhealthy, None);

        let weights = hb.gwt_weights();
        assert_eq!(weights.len(), 7, "should produce 7 domain weights");

        let core_weight = weights.iter().find(|(name, _)| name == "nt_core").unwrap();
        assert!(core_weight.1 > 0.0, "healthy core should increase attention");

        let mem_weight = weights.iter().find(|(name, _)| name == "nt_memory").unwrap();
        assert!(mem_weight.1 < 0.0, "unhealthy memory should decrease attention");
    }

    #[test]
    fn test_whale_metrics_default() {
        let whale = WhaleMetrics::default();
        assert_eq!(whale.current_phase, "weight_update");
        assert_eq!(whale.improvement_rate, 0.0);
        assert_eq!(whale.adaptive_switch_count, 0);
    }

    #[test]
    fn test_with_whale_metrics() {
        let snapshot = SystemHealthSnapshot::from_report(&HealthReport {
            status: HealthStatus::Healthy,
            components: HashMap::new(),
            timestamp: chrono::Utc::now(),
        });
        let whale = WhaleMetrics {
            current_phase: "harness_search".to_string(),
            phase_duration_ms: 5000,
            improvement_rate: 0.8,
            harness_weight_ratio: 0.4,
            adaptive_switch_count: 3,
        };
        let enhanced = snapshot.with_whale_metrics(&whale);
        assert!(enhanced.compilation >= 0.8, "compilation should reflect whale improvement");
    }
}