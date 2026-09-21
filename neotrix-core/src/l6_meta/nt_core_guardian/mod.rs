//! # NT-CORE-GUARDIAN — 统一自守护模块
//!
//! 融合 NeoTrix 8 个分散的自管理机制到单一模块:
//!
//! | 原始模块 | 融合到 | 职责 |
//! |---------|--------|------|
//! | `self_healing/health_monitor.rs` | `health.rs` | 组件健康采集 |
//! | `l1_facade::self_audit` | `health.rs` | 系统级健康扫描 |
//! | `healing/nt_mind_consciousness_monitor.rs` | `health.rs` | 意识层监控 |
//! | `self_healing/auto_repair.rs` | `repair.rs` | 故障→修复动作 |
//! | `diagnostic_chain/` | `repair.rs` | 诊断→修复闭环 |
//! | `nt_repair_self_heal.rs` | `repair.rs` | SelfTest 自愈 |
//! | `self_healing/circuit_breaker.rs` | `circuit_breaker.rs` | 熔断器 |
//! | `l1_action/nt_infra_breaker.rs` | `circuit_breaker.rs` | 基础熔断 |
//! | `l0_substrate/schema_watchdog.rs` | `watchdog.rs` | Schema 漂移检测 |
//! | `coordination/build_watchdog.rs` | `watchdog.rs` | 构建状态监控 |
//! | `foundation/guardian.rs` | `watchdog.rs` (KbGuardCheck) | KB 守卫 |
//! | `daemon_monitor.rs` | `supervisor.rs` | 进程管理 |
//! | `entry/mod.rs` supervisor | `supervisor.rs` | 进程级自守护 |
//!
//! 架构原则:
//! - 单一事实源: 所有健康状态汇总到 HealthState
//! - 分层消费: supervisor 读 HealthState 决定重启, repair 写 HealthState 记录修复
//! - 零外部依赖: 不依赖 launchd/systemd, 自包含 PID + 心跳 + 恢复日志
//! - R-P1 合规: forbid(unsafe_code), 无 unwrap()

#![forbid(unsafe_code)]

pub mod heartbeat;
pub mod health;
pub mod watchdog;
pub mod repair;
pub mod circuit_breaker;
pub mod supervisor;

pub use heartbeat::{HeartbeatState, spawn_heartbeat, spawn_health_writer};
pub use health::{HealthState, HealthLevel, ComponentHealth, scan_system_health};
pub use watchdog::{WatchdogRegistry, WatchdogCheck, WatchdogFinding, FindingSeverity, SchemaCheck, KbGuardCheck, BuildCheck};
pub use repair::{RepairOrchestrator, RepairAction, RepairResult};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState, CircuitError};
pub use supervisor::{SupervisorConfig, SupervisorEvent, backoff_with_jitter, write_pid_file, cleanup_pid_file, log_recovery, setup_heartbeat};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_guardian_pipeline() {
        // 1. 创建统一健康状态
        let health = Arc::new(HealthState::new());

        // 2. 执行系统扫描
        let findings = scan_system_health(&health);
        health.update_batch(findings);

        // 3. 看门狗检查
        let mut registry = WatchdogRegistry::new();
        registry.register(Box::new(BuildCheck));
        let watchdog_findings = registry.run_all();

        // 4. 修复编排
        let mut repair = RepairOrchestrator::new(health.clone());
        let results = repair.repair_cycle();

        // 5. 熔断器
        let cb = CircuitBreaker::with_defaults();
        let _ = cb.call(|| Ok::<_, String>("test"));

        // 6. Supervisor 配置
        let config = SupervisorConfig::default();

        // 验证
        assert!(health.global_level() != HealthLevel::Unknown || findings.is_empty());
        assert!(results.len() <= 1); // 大部分情况无需修复
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(config.max_restarts, 10);
    }
}
