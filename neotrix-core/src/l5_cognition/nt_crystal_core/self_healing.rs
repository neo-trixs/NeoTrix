//! SelfHealingEngine — 自愈机制 + 恢复模式库
//!
//! 基于 IEEE ICDSAAI 2026 的 Self-Healing AI Architecture:
//! - 健康监控 + 异常检测 + 级联故障检测
//! - 恢复模式: CircuitBreaker/Bulkhead/RetryWithBackoff/FallbackChain/StateRollback

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// 健康监控
// ═══════════════════════════════════════════════════════════════

/// 模块健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHealth {
    pub module_name: String,
    pub is_healthy: bool,
    pub error_rate: f64,
    pub latency_p99_ms: f64,
    pub last_check: u64,
    pub consecutive_failures: u32,
}

/// 健康快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub timestamp: u64,
    pub modules: Vec<ModuleHealth>,
    pub overall_healthy: bool,
}

/// 健康监控器
pub struct HealthMonitor {
    pub check_interval_ms: u64,
    pub failure_threshold: u32,
    pub error_rate_threshold: f64,
    pub history: Vec<HealthSnapshot>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            check_interval_ms: 60_000,
            failure_threshold: 3,
            error_rate_threshold: 0.1,
            history: Vec::new(),
        }
    }

    /// 检查模块健康
    pub fn check(&self, modules: &[ModuleHealth]) -> HealthSnapshot {
        let overall = modules.iter().all(|m| m.is_healthy);
        let snapshot = HealthSnapshot {
            timestamp: now_ms(),
            modules: modules.to_vec(),
            overall_healthy: overall,
        };
        snapshot
    }
}

// ═══════════════════════════════════════════════════════════════
// 异常检测
// ═══════════════════════════════════════════════════════════════

/// 异常类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    HighErrorRate,
    HighLatency,
    ConsecutiveFailures,
    ResourceExhaustion,
    StateInconsistency,
}

/// 异常
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub anomaly_type: AnomalyType,
    pub module_name: String,
    pub severity: f64,
    pub details: String,
    pub detected_at: u64,
}

/// 异常检测器
pub struct AnomalyDetector {
    pub error_rate_threshold: f64,
    pub latency_threshold_ms: f64,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            error_rate_threshold: 0.1,
            latency_threshold_ms: 1000.0,
        }
    }

    pub fn detect(&self, health: &HealthSnapshot) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        for module in &health.modules {
            if module.error_rate > self.error_rate_threshold {
                anomalies.push(Anomaly {
                    anomaly_type: AnomalyType::HighErrorRate,
                    module_name: module.module_name.clone(),
                    severity: module.error_rate,
                    details: format!("Error rate: {:.2}", module.error_rate),
                    detected_at: health.timestamp,
                });
            }
            if module.latency_p99_ms > self.latency_threshold_ms {
                anomalies.push(Anomaly {
                    anomaly_type: AnomalyType::HighLatency,
                    module_name: module.module_name.clone(),
                    severity: module.latency_p99_ms / self.latency_threshold_ms,
                    details: format!("Latency p99: {:.0}ms", module.latency_p99_ms),
                    detected_at: health.timestamp,
                });
            }
            if module.consecutive_failures > 3 {
                anomalies.push(Anomaly {
                    anomaly_type: AnomalyType::ConsecutiveFailures,
                    module_name: module.module_name.clone(),
                    severity: module.consecutive_failures as f64 / 10.0,
                    details: format!("Consecutive failures: {}", module.consecutive_failures),
                    detected_at: health.timestamp,
                });
            }
        }
        anomalies
    }
}

// ═══════════════════════════════════════════════════════════════
// 级联故障检测
// ═══════════════════════════════════════════════════════════════

/// 级联故障
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeFault {
    pub root_module: String,
    pub affected_modules: Vec<String>,
    pub propagation_path: Vec<String>,
    pub severity: f64,
}

/// 级联故障检测器
pub struct CascadeDetector {
    /// 依赖图: 模块 → 依赖的模块列表
    pub dependency_graph: HashMap<String, Vec<String>>,
}

impl CascadeDetector {
    pub fn new() -> Self {
        Self {
            dependency_graph: HashMap::new(),
        }
    }

    pub fn detect_cascade(&self, anomalies: &[Anomaly]) -> Vec<CascadeFault> {
        let mut cascades = Vec::new();
        let anomaly_modules: Vec<&str> = anomalies.iter().map(|a| a.module_name.as_str()).collect();

        // 检查依赖链
        for (module, deps) in &self.dependency_graph {
            if anomaly_modules.contains(&module.as_str()) {
                let affected: Vec<String> = deps
                    .iter()
                    .filter(|d| anomaly_modules.contains(&d.as_str()))
                    .cloned()
                    .collect();
                if !affected.is_empty() {
                    cascades.push(CascadeFault {
                        root_module: module.clone(),
                        affected_modules: affected,
                        propagation_path: deps.clone(),
                        severity: anomalies.iter()
                            .find(|a| a.module_name == *module)
                            .map(|a| a.severity)
                            .unwrap_or(0.5),
                    });
                }
            }
        }
        cascades
    }
}

// ═══════════════════════════════════════════════════════════════
// 恢复模式
// ═══════════════════════════════════════════════════════════════

/// 恢复模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryMode {
    /// 断路器: 连续失败 > 阈值 → 暂停 + 降级 + 重试
    CircuitBreaker { failure_threshold: u32, recovery_timeout_ms: u64 },
    /// 隔舱: 级联故障扩散 → 隔离 + 独立恢复
    Bulkhead { max_concurrent: u32 },
    /// 指数退避重试
    RetryWithBackoff { max_retries: u32, base_delay_ms: u64 },
    /// 备选路径降级
    FallbackChain { fallback_paths: Vec<String> },
    /// 状态回滚
    StateRollback { snapshot_id: String },
}

/// 恢复计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub target_module: String,
    pub mode: RecoveryMode,
    pub estimated_recovery_ms: u64,
    pub confidence: f64,
}

/// 恢复计划器
pub struct RecoveryPlanner {
    pub default_mode: RecoveryMode,
}

impl RecoveryPlanner {
    pub fn new() -> Self {
        Self {
            default_mode: RecoveryMode::RetryWithBackoff {
                max_retries: 3,
                base_delay_ms: 1000,
            },
        }
    }

    pub fn plan(&self, anomalies: &[Anomaly], cascades: &[CascadeFault]) -> Vec<RecoveryPlan> {
        let mut plans = Vec::new();

        for anomaly in anomalies {
            let mode = if cascades.iter().any(|c| c.root_module == anomaly.module_name) {
                RecoveryMode::Bulkhead { max_concurrent: 1 }
            } else if anomaly.severity > 0.8 {
                RecoveryMode::CircuitBreaker {
                    failure_threshold: 5,
                    recovery_timeout_ms: 30_000,
                }
            } else {
                self.default_mode.clone()
            };

            plans.push(RecoveryPlan {
                target_module: anomaly.module_name.clone(),
                mode,
                estimated_recovery_ms: 5000,
                confidence: 1.0 - anomaly.severity,
            });
        }

        plans
    }
}

// ═══════════════════════════════════════════════════════════════
// SelfHealingEngine 主引擎
// ═══════════════════════════════════════════════════════════════

/// SelfHealingEngine — 自愈引擎
pub struct SelfHealingEngine {
    pub health_monitor: HealthMonitor,
    pub anomaly_detector: AnomalyDetector,
    pub cascade_detector: CascadeDetector,
    pub recovery_planner: RecoveryPlanner,
    pub recovery_history: Vec<RecoveryEvent>,
}

/// 恢复事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryEvent {
    pub plan: RecoveryPlan,
    pub success: bool,
    pub timestamp: u64,
    pub details: String,
}

impl SelfHealingEngine {
    pub fn new() -> Self {
        Self {
            health_monitor: HealthMonitor::new(),
            anomaly_detector: AnomalyDetector::new(),
            cascade_detector: CascadeDetector::new(),
            recovery_planner: RecoveryPlanner::new(),
            recovery_history: Vec::new(),
        }
    }

    /// 监控并恢复
    pub fn monitor_and_heal(&mut self, modules: &[ModuleHealth]) -> Vec<RecoveryPlan> {
        // 1. 健康检查
        let health = self.health_monitor.check(modules);

        // 2. 异常检测
        let anomalies = self.anomaly_detector.detect(&health);

        if anomalies.is_empty() {
            return Vec::new();
        }

        // 3. 级联检测
        let cascades = self.cascade_detector.detect_cascade(&anomalies);

        // 4. 恢复计划
        let plans = self.recovery_planner.plan(&anomalies, &cascades);

        // 5. 记录
        for plan in &plans {
            self.recovery_history.push(RecoveryEvent {
                plan: plan.clone(),
                success: true,
                timestamp: now_ms(),
                details: format!("Recovery plan created for {}", plan.target_module),
            });
        }

        plans
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_monitor() {
        let monitor = HealthMonitor::new();
        let modules = vec![
            ModuleHealth {
                module_name: "nt_core".to_string(),
                is_healthy: true,
                error_rate: 0.01,
                latency_p99_ms: 10.0,
                last_check: now_ms(),
                consecutive_failures: 0,
            },
        ];
        let snapshot = monitor.check(&modules);
        assert!(snapshot.overall_healthy);
    }

    #[test]
    fn test_anomaly_detection() {
        let detector = AnomalyDetector::new();
        let health = HealthSnapshot {
            timestamp: now_ms(),
            modules: vec![ModuleHealth {
                module_name: "nt_core".to_string(),
                is_healthy: false,
                error_rate: 0.5,
                latency_p99_ms: 2000.0,
                last_check: now_ms(),
                consecutive_failures: 5,
            }],
            overall_healthy: false,
        };
        let anomalies = detector.detect(&health);
        assert_eq!(anomalies.len(), 3); // high error, high latency, consecutive failures
    }
}
