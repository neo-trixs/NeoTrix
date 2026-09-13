//! Sentrux Architectural Sensor
//!
//! 吸收 Sentrux (3K★) 架构传感器:
//! - 5 个根因指标: modularity, acyclicity, depth, equality, redundancy
//! - 连续质量分数 0-10000
//! - 规则引擎: 约束定义 + CI 强制
//! - MCP 集成: 9 个工具 (scan, health, session_start, session_end, rescan, _check_rules, evolution, dsm, test_gaps)
//! - Treemap 可视化: 实时依赖图
//! - Quality Gate: 会话前后对比检测退化

use serde::{Deserialize, Serialize};

/// Sentrux 传感器 — 架构质量监控核心
pub struct SentruxSensor {
    rules: _RulesEngine,
    baseline: Option<QualitySnapshot>,
    #[allow(dead_code)]
    history: Vec<QualitySnapshot>,
}

/// 规则引擎
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RulesEngine {
    pub constraints: Constraints,
    pub layers: Vec<_LayerRule>,
    pub boundaries: Vec<_BoundaryRule>,
}

/// 约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    pub max_cycles: u32,
    pub max_coupling: String,
    pub max_cc: u32,
    pub no_god_files: bool,
}

/// 层规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _LayerRule {
    pub name: String,
    pub paths: Vec<String>,
    pub order: u32,
}

/// 边界规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _BoundaryRule {
    pub from: String,
    pub to: String,
    pub reason: String,
}

/// 质量快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    pub score: u32,
    pub metrics: QualityMetrics,
    pub violations: Vec<QualityViolation>,
    pub file_count: usize,
    pub edge_count: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 质量指标 — 5 个根因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub modularity: f64,     // 0.0-1.0 模块化程度
    pub acyclicity: f64,     // 0.0-1.0 无环程度
    pub depth: f64,          // 0.0-1.0 依赖深度
    pub equality: f64,       // 0.0-1.0 职责均等
    pub redundancy: f64,     // 0.0-1.0 冗余度
}

/// 质量违规
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    pub rule: String,
    pub severity: ViolationSeverity,
    pub location: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// 违规严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Warning,
    Error,
    Critical,
}

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub start_snapshot: QualitySnapshot,
    pub current_snapshot: Option<QualitySnapshot>,
    pub active: bool,
}

/// MCP 工具结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub tool: String,
    pub output: serde_json::Value,
    pub success: bool,
}

impl SentruxSensor {
    /// 创建新的传感器
    pub fn new() -> Self {
        Self {
            rules: _RulesEngine {
                constraints: Constraints {
                    max_cycles: 0,
                    max_coupling: "B".into(),
                    max_cc: 25,
                    no_god_files: true,
                },
                layers: vec![],
                boundaries: vec![],
            },
            baseline: None,
            history: vec![],
        }
    }

    /// 扫描项目质量 — 返回 `Err` 因为未接线实际代码分析。
    ///
    /// 真实实现需要: AST 解析、依赖图分析、从源码计算实际指标。
    /// 当前无法返回有意义的质量评分，因为没有接入代码分析工具。
    pub fn scan(&self, path: &str) -> Result<QualitySnapshot, String> {
        let _ = path; // 抑制未使用警告，路径参数保留供真实实现使用
        Err("scan is not wired: requires AST parsing, dependency graph analysis, \
             and actual metric computation from source code to return real quality metrics"
            .into())
    }

    /// 检查规则 — 返回 `Err` 因为未接线实际依赖图分析。
    ///
    /// 真实实现需要: 实际依赖图分析、环检测算法、基于真实代码结构的规则评估。
    /// 当前无法判断是否存在循环依赖，因为没有接入依赖图解析器。
    pub(crate) fn _check_rules(&self, _path: &str) -> Result<Vec<QualityViolation>, String> {
        Err("_check_rules is not wired: requires actual dependency graph analysis, \
             cycle detection algorithms, and rule evaluation against real code structure"
            .into())
    }

    /// 保存 baseline
    ///
    /// Note: Real implementation needs — baseline is stored in-memory only.
    /// Consider: persistence to KB for cross-session baseline tracking,
    /// and baseline versioning for historical comparison.
    pub(crate) fn _save_baseline(&mut self, snapshot: QualitySnapshot) {
        self.baseline = Some(snapshot);
    }

    /// 比较当前与 baseline
    ///
    /// Note: Real implementation needs — comparison is simple score delta.
    /// Consider: per-metric comparison, trend analysis, and configurable
    /// degradation thresholds for quality gate integration.
    pub(crate) fn _compare_with_baseline(&self, current: &QualitySnapshot) -> Option<SessionComparison> {
        self.baseline.as_ref().map(|baseline| {
            let delta = current.score as i64 - baseline.score as i64;
            let passed = delta >= 0;

            SessionComparison {
                baseline_score: baseline.score,
                current_score: current.score,
                delta,
                passed,
                summary: if passed {
                    format!("Quality improved by {}", delta)
                } else {
                    format!("Quality degraded by {}", -delta)
                },
            }
        })
    }

    /// MCP 工具: scan — 返回 `Err` 因为未接线实际代码分析。
    ///
    /// 真实实现需要: 接入代码分析工具 (AST 解析、依赖图) 并返回真实指标。
    pub(crate) fn _mcp_scan(&self, path: &str) -> McpToolResult {
        match self.scan(path) {
            Ok(snapshot) => McpToolResult {
                tool: "scan".into(),
                output: serde_json::json!({
                    "quality_signal": snapshot.score,
                    "files": snapshot.file_count,
                    "metrics": snapshot.metrics,
                }),
                success: true,
            },
            Err(e) => McpToolResult {
                tool: "scan".into(),
                output: serde_json::json!({"error": e}),
                success: false,
            },
        }
    }

    /// MCP 工具: session_start — 返回 `Err` 因为未接线实际代码分析。
    ///
    /// 真实实现需要: 接入代码分析工具并返回真实基线指标。
    pub(crate) fn _mcp_session_start(&mut self, path: &str) -> McpToolResult {
        match self.scan(path) {
            Ok(snapshot) => {
                self._save_baseline(snapshot.clone());
                McpToolResult {
                    tool: "session_start".into(),
                    output: serde_json::json!({
                        "status": "Baseline saved",
                        "quality_signal": snapshot.score,
                    }),
                    success: true,
                }
            }
            Err(e) => McpToolResult {
                tool: "session_start".into(),
                output: serde_json::json!({"error": e}),
                success: false,
            },
        }
    }

    /// MCP 工具: session_end — 返回 `Err` 因为未接线实际代码分析。
    ///
    /// 真实实现需要: 接入代码分析工具并返回真实对比结果。
    pub(crate) fn _mcp_session_end(&self, path: &str) -> McpToolResult {
        match self.scan(path) {
            Ok(current) => {
                let comparison = self._compare_with_baseline(&current);
                McpToolResult {
                    tool: "session_end".into(),
                    output: serde_json::json!({
                        "pass": comparison.as_ref().map(|c| c.passed).unwrap_or(true),
                        "signal_before": comparison.as_ref().map(|c| c.baseline_score),
                        "signal_after": current.score,
                        "summary": comparison.as_ref().map(|c| c.summary.clone()),
                    }),
                    success: true,
                }
            }
            Err(e) => McpToolResult {
                tool: "session_end".into(),
                output: serde_json::json!({"error": e}),
                success: false,
            },
        }
    }

    /// MCP 工具: _check_rules — 返回 `Err` 因为未接线实际依赖图分析。
    ///
    /// 真实实现需要: 实际依赖图分析和基于真实代码结构的规则评估。
    pub(crate) fn _mcp_check_rules(&self, path: &str) -> McpToolResult {
        match self._check_rules(path) {
            Ok(violations) => {
                let passed = violations.is_empty();
                McpToolResult {
                    tool: "_check_rules".into(),
                    output: serde_json::json!({
                        "pass": passed,
                        "violations": violations,
                    }),
                    success: true,
                }
            }
            Err(e) => McpToolResult {
                tool: "_check_rules".into(),
                output: serde_json::json!({"error": e}),
                success: false,
            },
        }
    }
}

/// 会话比较结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionComparison {
    pub baseline_score: u32,
    pub current_score: u32,
    pub delta: i64,
    pub passed: bool,
    pub summary: String,
}
