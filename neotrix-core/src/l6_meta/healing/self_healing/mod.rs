//! # SELF-HEALING — 自愈检测与自动修复 (l6_meta healing 子模块)
//!
//! 四层自愈管线:
//!   health_monitor  — 健康状态采集: 组件级指标监控 + HealthStatus 语义
//!   auto_repair     — 自动修复: 故障检测 → 修复动作 → 结果反馈
//!   circuit_breaker — 断路器: 失败熔断 → 半开探测 → 恢复关闭
//!   resilience      — 弹性执行: 重试 + 断路器 + 超时 的组合门面
//!
//! R-P139 合规: forbid(unsafe_code), 无外部依赖, 仅 std。

#![forbid(unsafe_code)]

pub mod auto_repair;
pub mod circuit_breaker;
pub mod health_monitor;
pub mod resilience;

pub use auto_repair::{AutoRepair, Failure, FailureSeverity, RepairAction, RepairResult};
pub use circuit_breaker::CircuitBreaker;
pub use health_monitor::{ComponentMonitor, HealthStatus, HealthStatusKind};
pub use resilience::{ResilienceManager, ResilienceResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_re_exports() {
        let monitor = ComponentMonitor::new();
        let _ = monitor.get_all_status();

        let cb = CircuitBreaker::new(3, std::time::Duration::from_secs(30));
        let _ = cb.state();

        let rm = ResilienceManager::new();
        let _ = rm.execute(|| Ok::<_, String>("ok".to_string()));
    }

    #[test]
    fn test_health_monitor_to_auto_repair_pipeline() {
        let monitor = ComponentMonitor::new();
        let repair = AutoRepair::new(monitor);
        let failures = repair.detect_failures();
        for f in &failures {
            let result = repair.repair(f);
            assert!(result.success);
        }
    }

    #[test]
    fn test_circuit_breaker_with_auto_repair() {
        let cb = CircuitBreaker::new(3, std::time::Duration::from_millis(50));
        let monitor = ComponentMonitor::new();
        let repair = AutoRepair::new(monitor);

        // Simulate failures that trip the circuit breaker
        for i in 0..3 {
            let f = Failure::new(format!("comp_{}", i), FailureSeverity::High, "fail");
            let _ = cb.call(|| {
                repair.repair(&f);
                Err::<String, _>("simulated")
            });
        }
        assert_eq!(cb.state(), circuit_breaker::CircuitState::Open);
    }

    #[test]
    fn test_resilience_manager_with_health_monitor() {
        let rm = ResilienceManager::new().with_retry(3, std::time::Duration::from_millis(10));
        let monitor = ComponentMonitor::new();

        let result = rm.execute(|| {
            let status = monitor.check_component("memory");
            if status.status == health_monitor::HealthStatusKind::Critical {
                Err("critical")
            } else {
                Ok(status.component.clone())
            }
        });
        // Should succeed since system is healthy
        assert!(result.success);
    }

    #[test]
    fn test_failure_severity_affects_repair_action() {
        let monitor = ComponentMonitor::new();
        let repair = AutoRepair::new(monitor);

        let f_low = Failure::new("net", FailureSeverity::Low, "minor");
        let f_med = Failure::new("net", FailureSeverity::Medium, "degraded");
        let f_high = Failure::new("net", FailureSeverity::High, "down");
        let f_fatal = Failure::new("net", FailureSeverity::Fatal, "panic");

        assert!(matches!(
            repair.select_action(&f_low),
            RepairAction::ClearCache { .. }
        ));
        assert!(matches!(
            repair.select_action(&f_med),
            RepairAction::RebalanceLoad { .. }
        ));
        assert!(matches!(
            repair.select_action(&f_high),
            RepairAction::RestartComponent { .. }
        ));
        assert!(matches!(
            repair.select_action(&f_fatal),
            RepairAction::RestartComponent { .. }
        ));
    }
}
