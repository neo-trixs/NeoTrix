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
    use std::sync::Arc;

    #[test]
    fn test_full_guardian_pipeline() {
        // 1. 创建统一健康状态
        let health = Arc::new(HealthState::new());

        // 2. 执行系统扫描
        let findings = scan_system_health(&health);
        health.update_batch(findings.clone());

        // 3. 看门狗检查
        let mut registry = WatchdogRegistry::new();
        registry.register(Box::new(BuildCheck));
        let _watchdog_findings = registry.run_all();

        // 4. 修复编排 —— 机器无关的不变量验证
        // 2026-09-28 修正: 原断言 `results.len() <= 1` 依赖**运行机器** ——
        // 上面 scan_system_health 扫的是真实内存/磁盘, 于是本测试的结果取决于
        // 跑它的那台机器当时健康与否; 内存吃紧时多个组件降级, 修复数自然 > 1。
        // 这不是被测代码的错, 是测试把宿主环境当成了前置条件。
        // 改为用**只含健康组件**的独立 HealthState 验证真正的不变量:
        // 健康组件不该被修复。这完全不依赖宿主状态。
        let healthy_only = Arc::new(HealthState::new());
        healthy_only.update_batch(vec![ComponentHealth {
            name: "deterministic_probe".into(),
            level: HealthLevel::Healthy,
            message: "显式注入的健康组件".into(),
            metrics: std::collections::HashMap::new(),
            checked_at_ms: 0,
        }]);
        let mut repair_probe = RepairOrchestrator::new(healthy_only);
        let probe_results = repair_probe.repair_cycle();
        assert!(
            probe_results.is_empty(),
            "健康组件不该触发修复, 实际修了 {:?}",
            probe_results.iter().map(|r| &r.message).collect::<Vec<_>>()
        );

        // 真实扫描那条路径仍然要能跑通(它是管道的一部分), 但不断言其修复数
        let mut repair = RepairOrchestrator::new(health.clone());
        let _results = repair.repair_cycle();

        // 5. 熔断器
        let cb = CircuitBreaker::with_defaults();
        let _ = cb.call(|| Ok::<_, String>("test"));

        // 6. Supervisor 配置
        let config = SupervisorConfig::default();

        // 验证
        assert!(health.global_level() != HealthLevel::Unknown || findings.is_empty());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(config.max_restarts, 10);
    }
}
