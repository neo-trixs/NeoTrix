//! 技能链配置 — ChainConfig
//!
//! 控制链执行行为: 超时、重试、自动推进
//! R-P124: 配置集中管理，支持 Default trait

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::skill_step::SkillStep;

/// 技能链执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// 每个步骤的超时时间(秒)，key 为步骤名称
    pub step_timeouts: HashMap<String, u64>,
    /// 每步最大重试次数
    pub max_retries_per_step: u32,
    /// 是否在步骤成功后自动推进到下一步
    pub auto_advance: bool,
    /// 链整体超时(秒)，None 表示无限制
    pub chain_timeout: Option<u64>,
    /// 失败时是否回滚已完成的步骤
    pub rollback_on_failure: bool,
}

impl Default for ChainConfig {
    fn default() -> Self {
        let mut step_timeouts = HashMap::new();
        step_timeouts.insert("Brainstorm".into(), 300);
        step_timeouts.insert("Plan".into(), 120);
        step_timeouts.insert("TDD".into(), 600);
        step_timeouts.insert("Review".into(), 180);

        Self {
            step_timeouts,
            max_retries_per_step: 3,
            auto_advance: true,
            chain_timeout: None,
            rollback_on_failure: false,
        }
    }
}

impl ChainConfig {
    /// 创建最小配置
    pub fn minimal() -> Self {
        Self {
            step_timeouts: HashMap::new(),
            max_retries_per_step: 1,
            auto_advance: true,
            chain_timeout: None,
            rollback_on_failure: false,
        }
    }

    /// 获取指定步骤的超时时间(秒)，未配置则返回默认值
    ///
    /// # ⚠️ 当前**未被执行器读取**（2026-09-28 实测）
    ///
    /// `ChainExecutor::execute_chain` 与 `StepExecutor::execute_step` 都是**同步**
    /// 的（`Box<dyn StepExecutor>`，无 async/await），链里**没有任何超时机制**。
    /// 全仓 `step_timeouts` / `timeout_for_step` 的引用只出现在本文件（定义 + 测试）。
    ///
    /// ⇒ **这张表目前是死数据**：TDD 的 600s、Plan 的 120s 等**从未生效**。
    /// 不要以为配了它就有超时保护。
    ///
    /// 补齐它是**功能新增**而非修 bug，需要先定方案（同步侧只能用
    /// 「另起线程 + 限时 join」，涉及线程回收与 panic 传播；或把执行器整体改 async）。
    /// 台账：`docs/architecture/OPEN-TASKS-2026-09-28.md` §3.1 R-1。
    pub fn timeout_for_step(&self, step: &SkillStep) -> u64 {
        self.step_timeouts
            .get(step.name())
            .copied()
            .unwrap_or(300)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = ChainConfig::default();
        assert_eq!(cfg.max_retries_per_step, 3);
        assert!(cfg.auto_advance);
        assert!(!cfg.rollback_on_failure);
        assert!(cfg.chain_timeout.is_none());
        assert!(cfg.step_timeouts.contains_key("Brainstorm"));
        assert!(cfg.step_timeouts.contains_key("Plan"));
        // 2026-09-27: 键名跟随生产契约 —— SkillStep::Tdd.name() == "TDD",
        // 原断言钉的是错键 "Tdd", 正是该表对 TDD 整体失效的原因。
        assert!(cfg.step_timeouts.contains_key("TDD"));
        assert!(cfg.step_timeouts.contains_key("Review"));
    }

    #[test]
    fn test_minimal_config() {
        let cfg = ChainConfig::minimal();
        assert_eq!(cfg.max_retries_per_step, 1);
        assert!(cfg.auto_advance);
        assert!(cfg.step_timeouts.is_empty());
    }

    #[test]
    fn test_timeout_for_step_fallback() {
        let cfg = ChainConfig::minimal();
        let step = SkillStep::Brainstorm { topic: "x".into() };
        // 未配置 → 默认300
        assert_eq!(cfg.timeout_for_step(&step), 300);
    }

    #[test]
    fn test_timeout_for_step_custom() {
        let mut cfg = ChainConfig::default();
        cfg.step_timeouts.insert("TDD".into(), 1200); // 键名随 SkillStep::Tdd.name()
        let step = SkillStep::Tdd { feature: "auth".into() };
        assert_eq!(cfg.timeout_for_step(&step), 1200);
    }

    #[test]
    fn test_serde_roundtrip() {
        let cfg = ChainConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: ChainConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.max_retries_per_step, back.max_retries_per_step);
        assert_eq!(cfg.auto_advance, back.auto_advance);
    }

    #[test]
    fn test_default_config_full_fields() {
        let cfg = ChainConfig::default();
        assert_eq!(cfg.max_retries_per_step, 3);
        assert!(cfg.auto_advance);
        assert!(!cfg.rollback_on_failure);
        assert!(cfg.chain_timeout.is_none());
        assert_eq!(cfg.step_timeouts.len(), 4);
    }

    #[test]
    fn test_minimal_config_full_fields() {
        let cfg = ChainConfig::minimal();
        assert_eq!(cfg.max_retries_per_step, 1);
        assert!(cfg.auto_advance);
        assert!(!cfg.rollback_on_failure);
        assert!(cfg.chain_timeout.is_none());
        assert!(cfg.step_timeouts.is_empty());
    }

    #[test]
    fn test_timeout_for_plan_step() {
        let cfg = ChainConfig::default();
        let step = SkillStep::Plan { goal: "design".into() };
        assert_eq!(cfg.timeout_for_step(&step), 120);
    }

    #[test]
    fn test_timeout_for_tdd_step() {
        let cfg = ChainConfig::default();
        let step = SkillStep::Tdd { feature: "impl".into() };
        assert_eq!(cfg.timeout_for_step(&step), 600);
    }

    #[test]
    fn test_timeout_for_review_step() {
        let cfg = ChainConfig::default();
        let step = SkillStep::Review { code: "fn test(){}".into() };
        assert_eq!(cfg.timeout_for_step(&step), 180);
    }

    #[test]
    fn test_config_custom_chain_timeout() {
        let mut cfg = ChainConfig::default();
        cfg.chain_timeout = Some(3600);
        assert_eq!(cfg.chain_timeout, Some(3600));
    }

    #[test]
    fn test_config_rollback_on_failure() {
        let mut cfg = ChainConfig::default();
        cfg.rollback_on_failure = true;
        assert!(cfg.rollback_on_failure);
    }

    #[test]
    fn test_config_debug_format() {
        let cfg = ChainConfig::default();
        let debug = format!("{:?}", cfg);
        assert!(debug.contains("ChainConfig"));
    }
}
