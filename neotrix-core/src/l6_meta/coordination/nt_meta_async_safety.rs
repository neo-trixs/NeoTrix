//! Async Safety Wrapper — 异步安全包装器
//!
//! 吸收 KB 经验:
//! - async 上下文内禁止直接调用阻塞网络 I/O
//! - 必须 tokio::task::spawn_blocking 包裹并 .await
//! - AtomicBool 开关门禁控制网络刷新
//! - 防止单测触发真实网络

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 异步安全包装器
pub struct AsyncSafetyWrapper {
    gate_states: HashMap<String, bool>,
    blocking_wrappers: Vec<BlockingWrapper>,
    config: AsyncSafetyConfig,
    stats: AsyncSafetyStats,
}

/// 异步安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncSafetyConfig {
    pub enable_spawn_blocking: bool,
    pub enable_gate_control: bool,
    pub default_gate_state: bool,
    pub timeout_ms: u64,
}

impl Default for AsyncSafetyConfig {
    fn default() -> Self {
        Self {
            enable_spawn_blocking: true,
            enable_gate_control: true,
            default_gate_state: false,
            timeout_ms: 30000,
        }
    }
}

/// 阻塞包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockingWrapper {
    pub wrapper_id: String,
    pub name: String,
    pub wrapper_type: WrapperType,
    pub gate_key: String,
    pub is_async: bool,
}

/// 包装器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WrapperType {
    NetworkIO,
    FileIO,
    DatabaseIO,
    BlockingOperation,
}

/// 异步安全统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncSafetyStats {
    pub total_wrappers: u64,
    pub active_gates: u64,
    pub blocked_operations: u64,
    pub successful_operations: u64,
    pub timeout_operations: u64,
}

/// 安全检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheckResult {
    pub safe: bool,
    pub violations: Vec<SafetyViolation>,
    pub recommendations: Vec<String>,
}

/// 安全违反
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyViolation {
    pub violation_type: String,
    pub message: String,
    pub severity: ViolationSeverity,
}

/// 违反严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

impl AsyncSafetyWrapper {
    /// 创建新的异步安全包装器
    pub fn new() -> Self {
        Self {
            gate_states: HashMap::new(),
            blocking_wrappers: Vec::new(),
            config: AsyncSafetyConfig::default(),
            stats: AsyncSafetyStats {
                total_wrappers: 0,
                active_gates: 0,
                blocked_operations: 0,
                successful_operations: 0,
                timeout_operations: 0,
            },
        }
    }

    /// 设置门禁状态
    pub(crate) fn _set_gate(&mut self, key: &str, state: bool) {
        self.gate_states.insert(key.to_string(), state);
        if state {
            self.stats.active_gates += 1;
        }
    }

    /// 检查门禁状态
    pub(crate) fn _check_gate(&self, key: &str) -> bool {
        self.gate_states.get(key).copied().unwrap_or(self.config.default_gate_state)
    }

    /// 检查异步安全
    pub fn check_safety(&self, operation: &str) -> SafetyCheckResult {
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();

        // 检查是否在异步上下文中使用阻塞操作
        if operation.contains("blocking") || operation.contains("reqwest::blocking") {
            violations.push(SafetyViolation {
                violation_type: "blocking_in_async".into(),
                message: "在异步上下文中使用了阻塞操作".into(),
                severity: ViolationSeverity::High,
            });
            recommendations.push("使用 tokio::task::spawn_blocking 包裹阻塞操作".into());
        }

        // 检查是否直接发起网络请求
        if operation.contains("reqwest::get") || operation.contains("reqwest::post") {
            violations.push(SafetyViolation {
                violation_type: "direct_network_in_async".into(),
                message: "在异步上下文中直接发起网络请求".into(),
                severity: ViolationSeverity::Medium,
            });
            recommendations.push("使用异步客户端 reqwest::Client".into());
        }

        SafetyCheckResult {
            safe: violations.is_empty(),
            violations,
            recommendations,
        }
    }

    /// 获取所有包装器
    pub fn wrappers(&self) -> &[BlockingWrapper] {
        &self.blocking_wrappers
    }

    /// 获取统计信息
    pub fn stats(&self) -> &AsyncSafetyStats {
        &self.stats
    }
}
