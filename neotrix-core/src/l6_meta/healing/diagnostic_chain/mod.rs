//! # DIAGNOSTIC-CHAIN — 自愈诊断链 (nt_repair 子模块)
//!
//! 信号采集 → 诊断分析 → 修复动作 → 闭环验证 的完整自愈管线。
//!
//! - `health_signal.rs` — 健康信号: 组件指标的原子数据单元
//! - `diagnostician.rs` — 诊断引擎: 信号 → 根因 + 置信度 + 建议动作
//! - `repair_action.rs` — 修复动作: Restart / Rollback / Degrade / Alert / SelfHeal
//! - `healing_loop.rs` — 闭环调度: monitor → diagnose → repair → verify, 含重试上限
//! - `config.rs` — 治疗配置: 检查间隔 / 最大重试 / 自动修复开关

#![forbid(unsafe_code)]

pub mod config;
pub mod diagnostician;
pub mod healing_loop;
pub mod health_signal;
pub mod repair_action;

pub use config::HealingConfig;
pub use diagnostician::{DiagnosticResult, Diagnostician};
pub use healing_loop::HealingLoop;
pub use health_signal::{HealthSignal, Severity};
pub use repair_action::RepairAction;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_re_exports() {
        // 验证所有公开类型可通过 diagnostic_chain:: 访问
        let _sig = HealthSignal::new("test_component", "cpu_usage", 0.95, 1000, Severity::Warning);
        let _cfg = HealingConfig::default();
    }

    #[test]
    fn test_diagnostician_to_repair_action_pipeline() {
        let signals = vec![
            HealthSignal::critical("db", "error_rate", 0.95, 1000),
            HealthSignal::critical("db", "latency_ms", 5000.0, 1000),
        ];
        let diagnostic = Diagnostician::analyze(&signals).unwrap();
        let action = repair_action::select_action(&diagnostic.source_component, "critical");
        let result = repair_action::execute_repair(&action).unwrap();
        assert!(result.contains("db"));
    }

    #[test]
    fn test_healing_config_drives_healing_loop() {
        let config = HealingConfig::aggressive();
        assert_eq!(config.max_repair_attempts, 5);

        let mut loop_inst = HealingLoop::new(config);
        let signals = vec![HealthSignal::critical("svc", "error", 0.9, 1000)];
        let action = loop_inst.run_cycle(&signals);
        assert!(action.is_some());
        assert_eq!(loop_inst.attempt, 1);
    }

    #[test]
    fn test_health_signal_to_diagnostic_result_contains_source() {
        let signals = vec![HealthSignal::warning("cache", "hit_rate", 0.2, 1000)];
        let result = Diagnostician::analyze(&signals).unwrap();
        assert_eq!(result.source_component, "cache");
    }

    #[test]
    fn test_repair_action_variants_all_work() {
        let actions = vec![
            RepairAction::restart("comp"),
            RepairAction::rollback("v1.0"),
            RepairAction::degrade("fallback"),
            RepairAction::alert("check this"),
            RepairAction::self_heal("scripts/fix.sh"),
        ];
        for action in &actions {
            let result = repair_action::execute_repair(action);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_config_preset_drives_behavior() {
        let diag_only = HealingConfig::diagnostic_only();
        let mut loop_inst = HealingLoop::new(diag_only);
        let signals = vec![HealthSignal::critical("db", "err", 0.9, 1000)];

        // With auto_heal disabled, attempt counter should NOT increment
        let action = loop_inst.run_cycle(&signals);
        assert!(action.is_some());
        assert_eq!(loop_inst.attempt, 0);
    }
}
