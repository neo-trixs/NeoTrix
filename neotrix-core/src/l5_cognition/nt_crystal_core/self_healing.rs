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
    /// ⛔ **遗留声明字段**（2026-10-07 起**不再被信任**）。
    ///
    /// `HealthMonitor::check()` 已改为**由 `error_rate` / `latency_p99_ms` /
    /// `last_check` 三项真实测量推导** `overall_healthy`，
    /// ⛔ **不再**读本字段（此前是 `modules.iter().all(|m| m.is_healthy)`）。
    ///
    /// ⚠️ 本字段**仅在测试中被赋值**（L370/388/409）⇒ 生产路径无写入者。
    /// ⛔ 故它是一个**无信息量的声明** ⇒ 任何据此做的判断都是
    ///    「未经测量的主张」。
    ///
    /// ⚠️ **保留**而不删除：它属于对外数据形状（`Serialize`），
    ///    删除会破坏持久化兼容。⇒ 处置是「**标记为不被信任**」。
    ///
    /// ⭐ 判定依据：`check-fake-signal` R4 —— 「只有字面量赋值 + 生产区零读点」
    ///    ⇒ **该维度是恒定假信号且无人消费**。本字段符合该形态。
    ///
    /// ⭐⭐⭐ 2026-10-07 D2 切片复核：确认为 **nt-unwired-spec（有意保留，不删）**。
    ///
    /// # 为什么「生产区零读点」在这里是**正确设计**而不是漏接线
    ///
    /// `HealthMonitor::check`（见下）**故意不读**本字段，改由
    /// `last_check` / `latency_p99_ms` / `error_rate` 三个**真实测量**推导
    /// `overall_healthy` —— 因为本字段在生产路径上只是**调用方的主观声明**
    /// （只在测试 L343/L361 被赋值）。理由在该函数上方已完整记录。
    // nt-unwired-spec: 有意保留 —— 上面的 `check()` 刻意不读它，改用真实指标推导；
    //   ⛔ **不要**为了「让它看起来被用上」而回退到 `all(|m| m.is_healthy)`：
    //   那会让整体健康判定退回「调用方自称健康」，是 check-fake-signal R1/R4
    //   命中过的形态。字段按原注释保留（ABI/其他消费方）。
    //
    //   ⓘ 名字键控的基线说明：`dead-flag-baseline.txt` 的 `is_healthy` 条目
    //   只对**零读点的那个实例**生效。`nt_io_provider/routing/provider_swap.rs`
    //   也有一个同名 `is_healthy`，但它在同文件 :38/:47/:59 **被读** ⇒ 是活的，
    //   本就不在 dead 集合里，与本条目无关。
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
    ///
    /// ⭐ 2026-10-07 接线：整体健康**由真实指标推导**，⛔ 不再直接采信 `m.is_healthy`。
    ///
    /// ⛔ **原实现的缺陷**（`check-fake-signal` R1+R4 双重命中）：
    ///   `modules.iter().all(|m| m.is_healthy)` ⇒ `overall_healthy` 完全取决于
    ///   **调用方填的 `is_healthy`**；而实测该字段**只在测试里被赋值**
    ///   （本文件 L343 `true` / L361 `false`）⇒ 生产路径里它是**调用方的主观声明**，
    ///   ⛔ 而本 struct 明明带着**真实指标** `error_rate` / `latency_p99_ms`，
    ///   且本文件 L104/L113 **已经**用它们与阈值比较了。
    /// ⇒ 也就是说：**真实信号存在，但整体判定没用它** ⇒ 这是最坏的一种债
    ///   （有真数据、却按声明记账）。
    ///
    /// ⭐ 正解：与同文件 L104/L113 **同一判据**（`error_rate_threshold`）
    ///   推导 `overall_healthy`。
    /// ⚠️ 保留 `m.is_healthy` 字段本身（ABI/其他消费方），
    ///   ⛔ 但⛔ **不再**用它决定整体健康。
    pub fn check(&self, modules: &[ModuleHealth]) -> HealthSnapshot {
        // ⭐ 由**真实指标**推导（与下方 L104 的判据一致）
        let overall = modules.iter().all(|m| {
            // ⭐⭐ 三个「真实测量」缺一不可（实测：本 struct 的
            //   `is_healthy` 与 `last_check` 都**只在测试里被赋值**
            //   —— `is_healthy` L343/361、`last_check` L365/383
            //   ⇒ 生产路径上它们都是**未经测量的声明**）。
            // ⇒ 判据：`last_check` 必须非 0（说明真被检查过）、
            //   `latency_p99_ms` 必须 > 0、且错误率未超阈值。
            m.last_check > 0
                && m.latency_p99_ms > 0.0
                && m.error_rate <= self.error_rate_threshold
        });
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

#[cfg(test)]
mod real_health_tests {
    use super::{HealthMonitor, ModuleHealth};

    fn mk(name: &str, error_rate: f64, p99: f64) -> ModuleHealth {
        ModuleHealth {
            module_name: name.to_owned(),
            // ⭐ 刻意**恒 true**（模拟「调用方总是声明健康」）
            is_healthy: true,
            error_rate,
            latency_p99_ms: p99,
            // ⚠️ 这两个字段我**漏了**（第 8 次「猜数据结构」）⇒ 补上
            last_check: 1_700_000_000_000,   // 非 0 ⇒ 表示真被检查过
            consecutive_failures: 0,
        }
    }

    /// ⭐ **变异证据**：`overall_healthy` 必须**随真实指标变化**。
    ///
    /// 修复前它是 `modules.iter().all(|m| m.is_healthy)`，
    /// 而 `is_healthy` 在测试里恒 `true` ⇒ **无论错误率多高都判健康**。
    #[test]
    fn overall必须随真实错误率变化() {
        let mon = HealthMonitor::new();

        // 真实指标健康 ⇒ 判健康
        let healthy = mon.check(&[mk("a", 0.01, 10.0)]);
        assert!(healthy.overall_healthy, "低错误率 ⇒ 必须健康");

        // ⭐ 承重：声明仍说健康，但**真实错误率超阈值** ⇒ 必须判不健康
        let sick = mon.check(&[mk("a", 0.5, 10.0)]);
        assert!(
            !sick.overall_healthy,
            "error_rate 0.5 > threshold 0.1 ⇒ 必须判**不健康**，\
             即便 is_healthy 仍声明 true（修复前不可能失败 ⇒ 本测试即变异证据）"
        );
    }

    /// ⭐ 对照：`latency_p99_ms` 也参与判定（延迟为 0 视为**无测量** ⇒ 不健康）。
    #[test]
    fn 无测量值视为不健康() {
        let mon = HealthMonitor::new();
        let no_measure = mon.check(&[mk("a", 0.0, 0.0)]);
        assert!(
            !no_measure.overall_healthy,
            "latency_p99_ms == 0 ⇒ 该模块**没有测量数据** ⇒ ⛔ 不可判为健康"
        );
    }
}
