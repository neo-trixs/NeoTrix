//! 确定性仲裁器 — LEGIO 启发
//!
//! LEGIO 论文启发: 模块输出经确定性规则仲裁
//! 无随机性: 规则按优先级排序，条件匹配即决策

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ctm::Chunk;

/// 仲裁决策
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ArbitrationDecision {
    /// 执行: chunk 进入 Workspace
    GO,
    /// 重新框架化: chunk 需要重构后重试
    REFRAME,
    /// 拒绝: chunk 被丢弃
    NO_GO,
}

impl std::fmt::Display for ArbitrationDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GO => write!(f, "GO"),
            Self::REFRAME => write!(f, "REFRAME"),
            Self::NO_GO => write!(f, "NO_GO"),
        }
    }
}

/// 仲裁条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArbitrationCondition {
    /// 安全违规: chunk 包含危险内容
    SafetyViolation,
    /// 置信度高于阈值
    ConfidenceAbove(f64),
    /// 置信度低于阈值
    ConfidenceBelow(f64),
    /// 域限制: 仅允许特定源模块
    DomainRestricted(Vec<String>),
    /// 频率限制: 超过每分钟最大次数
    RateLimitExceeded,
    /// 来源模块白名单
    SourceWhitelist(Vec<String>),
}

/// 仲裁规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationRule {
    pub name: String,
    pub condition: ArbitrationCondition,
    pub decision: ArbitrationDecision,
    /// 优先级: 数值越大越先检查
    pub priority: u32,
}

/// 频率限制器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiter {
    pub max_per_minute: u32,
    pub current_count: u32,
    pub window_start: u64,
}

impl RateLimiter {
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            max_per_minute,
            current_count: 0,
            window_start: 0,
        }
    }

    /// 检查并消耗一次配额
    pub fn check_and_consume(&mut self, now: u64) -> bool {
        let window = 60; // 60 秒窗口
        if now - self.window_start >= window {
            // 新窗口
            self.window_start = now;
            self.current_count = 0;
        }

        if self.current_count >= self.max_per_minute {
            return false;
        }

        self.current_count += 1;
        true
    }
}

/// 仲裁记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationRecord {
    pub chunk_summary: String,
    pub source_module: String,
    pub decision: ArbitrationDecision,
    pub reason: String,
    pub timestamp: u64,
}

/// 仲裁统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationStats {
    pub total_decisions: usize,
    pub go_count: usize,
    pub reframe_count: usize,
    pub no_go_count: usize,
    pub go_rate: f64,
    pub decisions_by_reason: HashMap<String, usize>,
}

/// 确定性仲裁器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicArbitrator {
    /// 仲裁规则，按优先级降序排列
    pub rules: Vec<ArbitrationRule>,
    /// 默认置信度阈值
    pub go_threshold: f64,
    /// REFRAME 阈值
    pub reframe_threshold: f64,
    /// 频率限制器
    pub rate_limiter: RateLimiter,
    /// 决策日志
    pub decision_log: Vec<ArbitrationRecord>,
    #[serde(skip)]
    counter: u64,
}

impl DeterministicArbitrator {
    pub fn new() -> Self {
        let mut rules = Vec::new();

        // 内置安全规则 (最高优先级)
        rules.push(ArbitrationRule {
            name: "safety_block".into(),
            condition: ArbitrationCondition::SafetyViolation,
            decision: ArbitrationDecision::NO_GO,
            priority: 1000,
        });

        // 内置频率限制
        rules.push(ArbitrationRule {
            name: "rate_limit".into(),
            condition: ArbitrationCondition::RateLimitExceeded,
            decision: ArbitrationDecision::NO_GO,
            priority: 900,
        });

        // 按优先级排序
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        Self {
            rules,
            go_threshold: 0.7,
            reframe_threshold: 0.4,
            rate_limiter: RateLimiter::new(60),
            decision_log: Vec::new(),
            counter: 0,
        }
    }

    /// 确定性仲裁 — 无随机性
    pub fn arbitrate(&mut self, chunk: &Chunk) -> ArbitrationDecision {
        self.counter += 1;
        let now = self.counter;

        // 按优先级遍历规则
        for rule in &self.rules {
            let matched = match &rule.condition {
                ArbitrationCondition::SafetyViolation => {
                    // 检查 chunk 元数据中的安全标记
                    chunk
                        .metadata
                        .get("safety_violation")
                        .map(|v| v == "true")
                        .unwrap_or(false)
                }
                ArbitrationCondition::ConfidenceAbove(threshold) => chunk.score > *threshold,
                ArbitrationCondition::ConfidenceBelow(threshold) => chunk.score < *threshold,
                ArbitrationCondition::DomainRestricted(allowed) => {
                    !allowed.contains(&chunk.source_module)
                }
                ArbitrationCondition::RateLimitExceeded => {
                    !self.rate_limiter.check_and_consume(now)
                }
                ArbitrationCondition::SourceWhitelist(whitelist) => {
                    !whitelist.contains(&chunk.source_module)
                }
            };

            if matched {
                let record = ArbitrationRecord {
                    chunk_summary: chunk.content.chars().take(80).collect(),
                    source_module: chunk.source_module.clone(),
                    decision: rule.decision.clone(),
                    reason: rule.name.clone(),
                    timestamp: now,
                };
                self.decision_log.push(record);
                return rule.decision.clone();
            }
        }

        // 无规则匹配: 按置信度阈值判定
        let decision = if chunk.score >= self.go_threshold {
            ArbitrationDecision::GO
        } else if chunk.score >= self.reframe_threshold {
            ArbitrationDecision::REFRAME
        } else {
            ArbitrationDecision::NO_GO
        };

        let record = ArbitrationRecord {
            chunk_summary: chunk.content.chars().take(80).collect(),
            source_module: chunk.source_module.clone(),
            decision: decision.clone(),
            reason: format!(
                "threshold: score={:.2} (go={}, reframe={})",
                chunk.score, self.go_threshold, self.reframe_threshold
            ),
            timestamp: now,
        };
        self.decision_log.push(record);

        decision
    }

    /// 添加自定义规则
    pub fn add_rule(&mut self, rule: ArbitrationRule) {
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// 获取决策统计
    pub fn stats(&self) -> ArbitrationStats {
        let total = self.decision_log.len();
        let go_count = self
            .decision_log
            .iter()
            .filter(|r| r.decision == ArbitrationDecision::GO)
            .count();
        let reframe_count = self
            .decision_log
            .iter()
            .filter(|r| r.decision == ArbitrationDecision::REFRAME)
            .count();
        let no_go_count = self
            .decision_log
            .iter()
            .filter(|r| r.decision == ArbitrationDecision::NO_GO)
            .count();

        let mut decisions_by_reason: HashMap<String, usize> = HashMap::new();
        for record in &self.decision_log {
            *decisions_by_reason
                .entry(record.reason.clone())
                .or_insert(0) += 1;
        }

        ArbitrationStats {
            total_decisions: total,
            go_count,
            reframe_count,
            no_go_count,
            go_rate: if total > 0 {
                go_count as f64 / total as f64
            } else {
                0.0
            },
            decisions_by_reason,
        }
    }

    /// 获取最近 N 条决策
    pub fn recent_decisions(&self, n: usize) -> Vec<&ArbitrationRecord> {
        self.decision_log.iter().rev().take(n).collect()
    }
}
