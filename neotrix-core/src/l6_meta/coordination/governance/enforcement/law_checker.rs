//! NT-LAWS: Agent 行为不变量强制执行器
//!
//! 基于 Bend LAWS.bend 启发，在 Agent 执行前检查不变量。
//! 违反 law 会阻止执行并记录审计日志。
//!
//! # 核心思想
//!
//! ```text
//! Agent 想执行 action
//!        ↓
//!   check_laws(action, laws)
//!        ↓
//!   通过 → 执行
//!   违反 → 阻止 + 审计日志
//! ```

use std::fmt;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Law 错误：违反不变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawViolation {
    pub law_name: String,
    pub description: String,
    pub action_type: String,
    pub timestamp: DateTime<Utc>,
}

impl fmt::Display for LawViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Law '{}' violated: {} (action: {})",
            self.law_name, self.description, self.action_type
        )
    }
}

impl std::error::Error for LawViolation {}

/// Law 验证结果
#[derive(Debug)]
pub enum LawCheckResult {
    /// 通过，可以执行
    Passed,
    /// 违反，阻止执行
    Violated(LawViolation),
}

/// Law trait：定义不变量
///
/// # 实现示例
///
/// ```ignore
/// struct MemoryNeverLeaks;
///
/// impl Law for MemoryNeverLeaks {
///     fn name(&self) -> &str { "memory_never_leaks" }
///     fn description(&self) -> &str { "Agent 不能将敏感数据写入非授权存储" }
///     fn check(&self, action: &dyn std::any::Any) -> LawCheckResult {
///         // 检查逻辑
///         LawCheckResult::Passed
///     }
/// }
/// ```
pub trait Law: Send + Sync {
    /// law 名称
    fn name(&self) -> &str;

    /// law 描述
    fn description(&self) -> &str;

    /// 检查 action 是否违反此 law
    fn check(&self, action: &dyn std::any::Any) -> LawCheckResult;

    /// 严重级别（可选）
    fn severity(&self) -> LawSeverity {
        LawSeverity::High
    }
}

/// Law 严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LawSeverity {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// Law 检查器
pub struct LawChecker {
    laws: Vec<Arc<dyn Law>>,
    violations: Vec<LawViolation>,
}

impl LawChecker {
    /// 创建新的 LawChecker
    pub fn new() -> Self {
        Self {
            laws: Vec::new(),
            violations: Vec::new(),
        }
    }

    /// 注册 law
    pub fn register(&mut self, law: Arc<dyn Law>) {
        self.laws.push(law);
    }

    /// 批量注册 laws
    pub fn register_all(&mut self, laws: Vec<Arc<dyn Law>>) {
        self.laws.extend(laws);
    }

    /// 检查 action 是否违反任何 law
    pub fn check_action(&mut self, action: &dyn std::any::Any, _action_type: &str) -> LawCheckResult {
        for law in &self.laws {
            match law.check(action) {
                LawCheckResult::Passed => continue,
                LawCheckResult::Violated(violation) => {
                    self.violations.push(violation.clone());
                    return LawCheckResult::Violated(violation);
                }
            }
        }
        LawCheckResult::Passed
    }

    /// 批量检查多个 actions
    pub fn check_actions(
        &mut self,
        actions: &[(&dyn std::any::Any, &str)],
    ) -> Vec<LawCheckResult> {
        actions
            .iter()
            .map(|(action, action_type)| self.check_action(*action, action_type))
            .collect()
    }

    /// 获取所有 violations
    pub fn violations(&self) -> &[LawViolation] {
        &self.violations
    }

    /// 清除 violations
    pub fn clear_violations(&mut self) {
        self.violations.clear();
    }

    /// 获取注册的 law 数量
    pub fn law_count(&self) -> usize {
        self.laws.len()
    }
}

impl Default for LawChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// 安全不变量 Law：检查是否泄漏敏感数据
pub struct MemoryNeverLeaks;

impl Law for MemoryNeverLeaks {
    fn name(&self) -> &str {
        "memory_never_leaks"
    }

    fn description(&self) -> &str {
        "Agent 不能将敏感数据写入非授权存储"
    }

    fn check(&self, action: &dyn std::any::Any) -> LawCheckResult {
        // TODO: 实现实际检查逻辑
        // 目前默认通过，需要与 AgentAction 类型集成
        let _ = action;
        LawCheckResult::Passed
    }

    fn severity(&self) -> LawSeverity {
        LawSeverity::Critical
    }
}

/// 成本不变量 Law：检查是否超预算
pub struct CostStaysBelow {
    pub max_cost: f64,
}

impl Law for CostStaysBelow {
    fn name(&self) -> &str {
        "cost_stays_below"
    }

    fn description(&self) -> &str {
        "单次会话总成本不超过预算"
    }

    fn check(&self, action: &dyn std::any::Any) -> LawCheckResult {
        // TODO: 实现实际检查逻辑
        // 需要从 action 中提取成本信息
        let _ = action;
        LawCheckResult::Passed
    }

    fn severity(&self) -> LawSeverity {
        LawSeverity::High
    }
}

/// 交接不变量 Law：检查是否包含 context
pub struct HandoffIncludesContext;

impl Law for HandoffIncludesContext {
    fn name(&self) -> &str {
        "handoff_includes_context"
    }

    fn description(&self) -> &str {
        "Agent 交接时必须包含完整上下文"
    }

    fn check(&self, action: &dyn std::any::Any) -> LawCheckResult {
        // TODO: 实现实际检查逻辑
        // 需要与 Handoff 类型集成
        let _ = action;
        LawCheckResult::Passed
    }

    fn severity(&self) -> LawSeverity {
        LawSeverity::High
    }
}

/// 创建默认的 laws
pub fn default_laws() -> Vec<Arc<dyn Law>> {
    vec![
        Arc::new(MemoryNeverLeaks),
        Arc::new(CostStaysBelow { max_cost: 1.0 }),
        Arc::new(HandoffIncludesContext),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_law_checker_new() {
        let checker = LawChecker::new();
        assert_eq!(checker.law_count(), 0);
        assert!(checker.violations().is_empty());
    }

    #[test]
    fn test_law_checker_register() {
        let mut checker = LawChecker::new();
        checker.register(Arc::new(MemoryNeverLeaks));
        assert_eq!(checker.law_count(), 1);
    }

    #[test]
    fn test_law_checker_check_action_passes() {
        let mut checker = LawChecker::new();
        checker.register(Arc::new(MemoryNeverLeaks));
        
        let action = 42; // 临时测试数据
        let result = checker.check_action(&action, "test");
        
        matches!(result, LawCheckResult::Passed);
        assert!(checker.violations().is_empty());
    }

    #[test]
    fn test_default_laws() {
        let laws = default_laws();
        assert_eq!(laws.len(), 3);
    }

    #[test]
    fn test_memory_never_leaks_law() {
        let law = MemoryNeverLeaks;
        assert_eq!(law.name(), "memory_never_leaks");
        assert_eq!(law.severity(), LawSeverity::Critical);
    }

    #[test]
    fn test_cost_stays_below_law() {
        let law = CostStaysBelow { max_cost: 1.0 };
        assert_eq!(law.name(), "cost_stays_below");
        assert_eq!(law.severity(), LawSeverity::High);
    }
}
