//! Dispatcher config — 调度器配置（纯搬移，行为零变更）。

use super::nt_dispatcher_policy::{CONFIDENCE_FLOOR, clamp_confidence};
use serde::{Deserialize, Serialize};

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatcherConfig {
    /// 任务拆解激进度 (0.0-1.0)
    pub decomposition_aggression: f64,
    /// 最大子任务数
    pub max_sub_tasks: usize,
    /// 是否启用 CoT 生成
    pub enable_cot: bool,
    /// 是否启用验证器
    pub enable_verifier: bool,
    /// 并发执行子任务数
    pub max_concurrent_tasks: usize,
    /// 子任务超时（秒）
    pub sub_task_timeout_secs: u64,
    /// 是否隐藏内部实现细节（用户视角）
    pub hide_internal_details: bool,
    /// 置信阈值 τ（D-5；默认 0.65；V-3 只许调严，钳位 [0.65, 1.0]）。
    pub confidence_threshold: f64,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            decomposition_aggression: 0.5,
            max_sub_tasks: 10,
            enable_cot: true,
            enable_verifier: true,
            max_concurrent_tasks: 3,
            sub_task_timeout_secs: 60,
            hide_internal_details: true,
            confidence_threshold: CONFIDENCE_FLOOR,
        }
    }
}

impl DispatcherConfig {
    /// 从环境变量加载配置（NEOTRIX_DISPATCH_* 前缀），未设置时回退默认值。
    /// 支持: NEOTRIX_DISPATCH_AGGRESSION, NEOTRIX_DISPATCH_MAX_SUBTASKS,
    ///       NEOTRIX_DISPATCH_COT, NEOTRIX_DISPATCH_VERIFIER,
    ///       NEOTRIX_DISPATCH_CONCURRENCY, NEOTRIX_DISPATCH_TIMEOUT,
    ///       NEOTRIX_DISPATCH_HIDE_INTERNAL, NEOTRIX_DISPATCH_CONFIDENCE
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_AGGRESSION") {
            if let Ok(f) = v.parse::<f64>() {
                cfg.decomposition_aggression = f.clamp(0.0, 1.0);
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_MAX_SUBTASKS") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.max_sub_tasks = n.max(1);
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_COT") {
            cfg.enable_cot = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_VERIFIER") {
            cfg.enable_verifier = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_CONCURRENCY") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.max_concurrent_tasks = n.max(1);
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_TIMEOUT") {
            if let Ok(n) = v.parse::<u64>() {
                cfg.sub_task_timeout_secs = n.max(1);
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_HIDE_INTERNAL") {
            cfg.hide_internal_details = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_CONFIDENCE") {
            if let Ok(f) = v.parse::<f64>() {
                cfg.confidence_threshold = clamp_confidence(f);
            }
        }
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_core_task_dispatcher::nt_dispatcher_policy::CONFIDENCE_FLOOR;

    #[test]
    fn test_dispatcher_config_default() {
        let config = DispatcherConfig::default();
        assert_eq!(config.decomposition_aggression, 0.5);
        assert_eq!(config.max_sub_tasks, 10);
        assert!(config.enable_cot);
        assert!(config.enable_verifier);
        assert_eq!(config.confidence_threshold, CONFIDENCE_FLOOR);
    }
}
