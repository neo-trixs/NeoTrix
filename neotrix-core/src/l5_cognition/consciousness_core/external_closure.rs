//! 外部缺口闭环 — 知识获取 + 试错求解
//!
//! 当意识核心发现内部知识不足以完成任务时，通过此模块
//! 调用外部知识源（论文/GitHub/技术文档）获取信息基础。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use super::dispatch::ConsciousTask;

/// 外部知识缺口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    pub description: String,
    pub domain: String,
    pub priority: u8,
}

/// 外部知识获取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalKnowledge {
    pub source: String,
    pub content: String,
    pub confidence: f64,
    pub url: Option<String>,
}

/// 外部缺口闭环配置
#[derive(Debug, Clone, Default)]
pub struct ExternalClosureConfig {
    pub max_attempts: u32,
    pub timeout_secs: u64,
}

/// 外部缺口闭环报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalClosureReport {
    pub solved: bool,
    pub attempts: u32,
    pub output: String,
}

/// 求解执行器 trait
#[async_trait]
pub trait SolutionExecutor: Send + Sync {
    async fn execute(&self, task: &ConsciousTask) -> Result<String, String>;
}

/// 外部缺口闭环引擎
pub struct ExternalClosureEngine {
    max_attempts: u32,
}

impl Default for ExternalClosureEngine {
    fn default() -> Self {
        Self { max_attempts: 5 }
    }
}

impl ExternalClosureEngine {
    pub fn new(max_attempts: u32) -> Self {
        Self { max_attempts }
    }

    /// 尝试通过外部知识源填补缺口
    pub fn attempt_closure(&self, gap: &KnowledgeGap) -> Result<ExternalKnowledge, ClosureError> {
        // Stub: 实际实现需要接入 NT-WORLD crawl 能力
        Ok(ExternalKnowledge {
            source: "stub".to_string(),
            content: format!("External knowledge for: {}", gap.description),
            confidence: 0.5,
            url: None,
        })
    }

    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

/// 关闭外部缺口 (stub)
pub fn close_external_gap(
    _kb: &crate::l5_cognition::kb_facade::KnowledgeBase,
    task: &ConsciousTask,
    _executor: &dyn SolutionExecutor,
    _config: &ExternalClosureConfig,
) -> ExternalClosureReport {
    ExternalClosureReport {
        solved: false,
        attempts: 0,
        output: format!("Stub: external gap closure for '{}'", task.summary),
    }
}

/// 运行外部缺口闭环 (stub)
pub fn run_external_closure(
    task: &ConsciousTask,
    _executor: &dyn SolutionExecutor,
    _config: &ExternalClosureConfig,
    _sources: &[&str],
) -> ExternalClosureReport {
    ExternalClosureReport {
        solved: false,
        attempts: 0,
        output: format!("Stub: external closure for '{}'", task.summary),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClosureError {
    NoSourceAvailable,
    Timeout,
    AllAttemptsExhausted,
}
