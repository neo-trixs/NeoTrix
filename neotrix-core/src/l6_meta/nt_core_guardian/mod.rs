//! # NT-CORE-GUARDIAN — 统一自守护模块
//!
//! ⛔ **2026-10-07 文档修正：原文「融合 NeoTrix 8 个分散的自管理机制」是错的。**
//! 下面这张表**列 13 行**，且 4 个路径不存在、4 个存在但**从未被融合**。
//! ⇒ 三处不符逐一列出后，再谈"融合到"。
//!
//! ✅ **2026-10-08 最终裁决**: 本模块**不用于生产守护路径**，仅用于 test/demo 验证。
//! 不强制在生产代码中 `pub mod` 此模块。
//!
//! | 原始模块（路径已逐一核实） | 融合到 | 实际状态 |
//! |---------|--------|---------|
//! | `l6_meta/healing/self_healing/health_monitor.rs` | `health.rs` | ⚠️ **未融合**（仍为独立活实现） |
//! | `l1_facade::self_audit` | `health.rs` | ⚠️ 未融合 |
//! | `l6_meta/healing/nt_mind_consciousness_monitor.rs` | `health.rs` | ⚠️ 未融合 |
//! | `l6_meta/healing/self_healing/auto_repair.rs` | `repair.rs` | ⚠️ **未融合**（仍为独立活实现） |
//! | `l6_meta/healing/diagnostic_chain/` | `repair.rs` | ⚠️ 未融合 |
//! | `l6_meta/healing/nt_repair_self_heal.rs` | `repair.rs` | ⚠️ **未融合**（仍为独立活实现） |
//! | `l6_meta/healing/self_healing/circuit_breaker.rs` | `circuit_breaker.rs` | ⚠️ **未融合** —— 见下方假融合声明 |
//! | `l1_action/nt_infra_breaker.rs` | `circuit_breaker.rs` | ⚠️ **未融合**（仍为独立活实现） |
//! | `l0_substrate/nt_core_schema_watchdog.rs` | `watchdog.rs` | ⛔ 原表写 `schema_watchdog.rs`，**真实文件名带 `nt_core_` 前缀** |
//! | `l6_meta/coordination/nt_meta_build_watchdog.rs` | `watchdog.rs` | ⛔ 原表写 `coordination/build_watchdog.rs`，**真实文件名带 `nt_meta_` 前缀** |
//! | `foundation/guardian.rs` | `watchdog.rs` (KbGuardCheck) | ⚠️ `KbGuardCheck` **全仓只出现在本模块内** ⇒ 该行两个方向都无从证伪 |
//! | `daemon_monitor.rs` | `supervisor.rs` | ⛔ **该文件不存在** |
//! | `entry/mod.rs` supervisor | `supervisor.rs` | ⛔ **该文件不存在** |
//!
//! ## ⛔ 三条未决问题（不要把本模块当"已完成的融合"）
//!
//! 1. **零消费者**：全仓 `--glob '*.rs'` 实测 `nt_core_guardian` 的**唯一**命中是
//!    `l6_meta/mod.rs:17` 的 `pub mod nt_core_guardian;` ⇒ **导出 ≠ 调用**。
//! 2. **已建 + 已测 + 未接线**：本模块有 `test_full_guardian_pipeline`，
//!    故按本仓既有裁决（对照 `CLAIMED-BUT-NOT-ENFORCED-2026-10-05.md` §2 对
//!    `nt_shield_ztnet` 的处置）**不删**，只标注接线待决。
//! 3. ⛔ **本模块的 `circuit_breaker.rs` 自称「单一实现（融合 self_healing +
//!    nt_infra_breaker）」** —— 实测**两个源都仍是独立活实现**
//!    （`healing/self_healing/circuit_breaker.rs`、`l1_action/nt_infra_breaker.rs`）
//!    ⇒ 那是**假融合声明**，与本表同型。
//!
//! **架构原则（原作者声明）**:
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
