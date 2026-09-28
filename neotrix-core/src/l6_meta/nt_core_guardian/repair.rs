//! # REPAIR — 统一修复编排 (融合 auto_repair + diagnostic_chain + self_heal)
//!
//! 融合:
//! - `self_healing/auto_repair.rs` — 故障→修复动作映射
//! - `diagnostic_chain/` — 信号→诊断→修复→验证闭环
//! - `nt_repair_self_heal.rs` — SelfTest 驱动的自愈
//!
//! 单一入口: RepairOrchestrator 消费 HealthState 的 findings,
//! 执行修复动作, 记录经验, 广播事件。

#![forbid(unsafe_code)]

use super::health::{HealthLevel, HealthState};
use std::sync::Arc;

/// 修复动作
#[derive(Debug, Clone)]
pub enum RepairAction {
    /// 清理缓存
    ClearCache { reason: String },
    /// 重启组件 (经 EventBus)
    RestartComponent { component: String, reason: String },
    /// 降级运行
    Degrade { fallback: String },
    /// 告警 (注入意识监控)
    Alert { message: String },
    /// 从备份恢复
    RestoreFromBackup { component: String },
    /// 无需修复
    NoOp,
}

/// 修复结果
#[derive(Debug, Clone)]
pub struct RepairResult {
    pub action: RepairAction,
    pub success: bool,
    pub message: String,
}

/// 修复编排器
pub struct RepairOrchestrator {
    health: Arc<HealthState>,
    repairs_done: u64,
    last_repairs: Vec<RepairResult>,
}

impl RepairOrchestrator {
    pub fn new(health: Arc<HealthState>) -> Self {
        Self {
            health,
            repairs_done: 0,
            last_repairs: Vec::new(),
        }
    }

    /// 执行一轮修复: 读取 HealthState, 选择修复动作, 执行
    pub fn repair_cycle(&mut self) -> Vec<RepairResult> {
        let components = self.health.get_all_components();
        let mut results = Vec::new();

        for comp in &components {
            if comp.level == HealthLevel::Healthy || comp.level == HealthLevel::Unknown {
                continue;
            }

            let action = self.select_action(comp);
            let result = self.execute_action(&action);
            self.repairs_done += 1;
            results.push(result);
        }

        self.last_repairs = results.clone();
        results
    }

    /// 根据组件状态选择修复动作
    fn select_action(&self, comp: &super::health::ComponentHealth) -> RepairAction {
        match comp.level {
            HealthLevel::Critical => {
                // Critical: 尝试恢复/重启
                if comp.name == "kb" {
                    RepairAction::RestoreFromBackup {
                        component: comp.name.clone(),
                    }
                } else if comp.name == "disk" || comp.name == "memory" {
                    RepairAction::ClearCache {
                        reason: comp.message.clone(),
                    }
                } else {
                    RepairAction::RestartComponent {
                        component: comp.name.clone(),
                        reason: comp.message.clone(),
                    }
                }
            }
            HealthLevel::Degraded => {
                // Degraded: 清理/告警
                if comp.name == "disk" {
                    RepairAction::ClearCache {
                        reason: comp.message.clone(),
                    }
                } else {
                    RepairAction::Alert {
                        message: comp.message.clone(),
                    }
                }
            }
            _ => RepairAction::NoOp,
        }
    }

    /// 执行修复动作
    fn execute_action(&self, action: &RepairAction) -> RepairResult {
        match action {
            RepairAction::ClearCache { reason } => {
                // 2026-09-27 安全修复 (高危): 原实现执行 `cargo clean` ——
                // 会删掉整个 target/ (含 deps 活指纹), 一次自愈动作即可让全量重编
                // 数小时, 且违反本仓硬规则 R-BUILD-5。改为只删 target/<profile>/incremental
                // (纯派生产物, 可再生), 并如实上报实际结果, 不再无脑报 success。
                log::info!("[repair] clearing incremental build cache: {}", reason);
                let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
                let mut cleared: Vec<String> = Vec::new();
                let mut failed: Vec<String> = Vec::new();
                for profile in ["debug", "release"] {
                    let inc = std::path::Path::new(&target_dir).join(profile).join("incremental");
                    if !inc.exists() {
                        continue;
                    }
                    match std::fs::remove_dir_all(&inc) {
                        Ok(()) => cleared.push(inc.display().to_string()),
                        Err(e) => failed.push(format!("{}: {}", inc.display(), e)),
                    }
                }
                let ok = failed.is_empty();
                if !ok {
                    log::warn!("[repair] incremental cache clear partial: {:?}", failed);
                }
                RepairResult {
                    action: action.clone(),
                    success: ok,
                    message: if cleared.is_empty() && ok {
                        format!("no incremental cache to clear: {}", reason)
                    } else {
                        format!(
                            "incremental cache cleared ({} dir(s), deps preserved): {}",
                            cleared.len(),
                            reason
                        )
                    },
                }
            }
            RepairAction::RestartComponent { component, reason } => {
                log::warn!("[repair] restart signal for {}: {}", component, reason);
                // 通过日志广播, 由 EventBus 消费
                RepairResult {
                    action: action.clone(),
                    success: true,
                    message: format!("restart signal emitted for {}", component),
                }
            }
            RepairAction::RestoreFromBackup { component } => {
                log::warn!("[repair] restoring {} from backup", component);
                // 委托给 KbGuard
                let guard = crate::l5_cognition::nt_mind::foundation::guardian::KbGuard::default();
                match guard.guard() {
                    report if report.restored => RepairResult {
                        action: action.clone(),
                        success: true,
                        message: format!("{} restored from backup", component),
                    },
                    _ => RepairResult {
                        action: action.clone(),
                        success: false,
                        message: format!("{} restore failed", component),
                    },
                }
            }
            RepairAction::Alert { message } => {
                log::warn!("[repair] alert: {}", message);
                RepairResult {
                    action: action.clone(),
                    success: true,
                    message: format!("alert: {}", message),
                }
            }
            RepairAction::Degrade { fallback } => {
                log::warn!("[repair] degrading to: {}", fallback);
                RepairResult {
                    action: action.clone(),
                    success: true,
                    message: format!("degraded to {}", fallback),
                }
            }
            RepairAction::NoOp => RepairResult {
                action: action.clone(),
                success: true,
                message: "no repair needed".into(),
            },
        }
    }

    /// 获取修复统计
    pub fn stats(&self) -> (u64, Vec<RepairResult>) {
        (self.repairs_done, self.last_repairs.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::super::health::ComponentHealth;
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_no_repair_for_healthy() {
        let health = Arc::new(HealthState::new());
        health.update_component(ComponentHealth {
            name: "test".into(),
            level: HealthLevel::Healthy,
            message: "ok".into(),
            metrics: HashMap::new(),
            checked_at_ms: 0,
        });
        let mut orch = RepairOrchestrator::new(health);
        let results = orch.repair_cycle();
        assert!(results.is_empty());
    }

    #[test]
    fn test_repair_for_critical() {
        let health = Arc::new(HealthState::new());
        health.update_component(ComponentHealth {
            name: "disk".into(),
            level: HealthLevel::Critical,
            message: "disk full".into(),
            metrics: HashMap::new(),
            checked_at_ms: 0,
        });
        let mut orch = RepairOrchestrator::new(health);
        let results = orch.repair_cycle();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }
}
