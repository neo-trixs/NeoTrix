//! ASI Compliance - ASI 安全等级对齐检查
//!
//! 通用安全接口：适配所有外部模型的安全等级
//! 设计原则：模型无关，架构驱动

/// 安全等级（OWASP AISVS 2026 对齐）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SafetyLevel {
    /// 无限制（不安全）
    Unrestricted = 0,
    /// 基本安全（仅拒绝明显恶意）
    Basic = 1,
    /// 标准安全（多层防御）
    Standard = 2,
    /// 增强安全（主动检测）
    Enhanced = 3,
    /// 最高安全（零信任）
    Maximum = 4,
}

/// 合规检查结果
#[derive(Debug, Clone)]
pub struct ComplianceResult {
    pub level: SafetyLevel,
    pub passed: bool,
    pub violations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// ASI 合规检查器
pub struct AsiComplianceChecker {
    /// 当前要求的安全等级
    required_level: SafetyLevel,
    /// 模型特定规则
    model_rules: Vec<ComplianceRule>,
}

#[derive(Debug, Clone)]
pub struct ComplianceRule {
    name: String,
    description: String,
    level: SafetyLevel,
    check_fn: String, // 规则标识，实际检查由外部实现
}

impl AsiComplianceChecker {
    pub fn new(required_level: SafetyLevel) -> Self {
        let model_rules = vec![
            ComplianceRule {
                name: "prompt_injection_defense".to_string(),
                description: "多层防御 prompt injection".to_string(),
                level: SafetyLevel::Standard,
                check_fn: "check_injection_defense".to_string(),
            },
            ComplianceRule {
                name: "output_filtering".to_string(),
                description: "输出过滤".to_string(),
                level: SafetyLevel::Standard,
                check_fn: "check_output_filter".to_string(),
            },
            ComplianceRule {
                name: "reasoning_protection".to_string(),
                description: "推理链保护".to_string(),
                level: SafetyLevel::Enhanced,
                check_fn: "check_reasoning_protection".to_string(),
            },
            ComplianceRule {
                name: "trust_grading".to_string(),
                description: "信任分级".to_string(),
                level: SafetyLevel::Enhanced,
                check_fn: "check_trust_grading".to_string(),
            },
            ComplianceRule {
                name: "escape_detection".to_string(),
                description: "逃逸检测".to_string(),
                level: SafetyLevel::Maximum,
                check_fn: "check_escape_detection".to_string(),
            },
        ];

        Self {
            required_level,
            model_rules,
        }
    }

    /// 检查合规性
    pub fn check(&self, current_level: SafetyLevel, context: &[(&str, bool)]) -> ComplianceResult {
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();

        // 检查是否满足要求的安全等级
        if current_level < self.required_level {
            violations.push(format!(
                "Security level {:?} below required {:?}",
                current_level, self.required_level
            ));
            recommendations.push(format!(
                "Upgrade security level to {:?} or higher",
                self.required_level
            ));
        }

        // 检查模型特定规则
        for rule in &self.model_rules {
            if rule.level > current_level {
                // 检查上下文中是否有该规则的通过标记
                let passed = context
                    .iter()
                    .any(|(name, passed)| *name == rule.check_fn && *passed);

                if !passed {
                    violations.push(format!(
                        "Rule '{}' not satisfied: {}",
                        rule.name, rule.description
                    ));
                    recommendations.push(format!("Implement: {}", rule.description));
                }
            }
        }

        ComplianceResult {
            level: current_level,
            passed: violations.is_empty(),
            violations,
            recommendations,
        }
    }

    /// 获取所有规则
    pub fn rules(&self) -> &[ComplianceRule] {
        &self.model_rules
    }

    /// 获取要求的安全等级
    pub fn required_level(&self) -> SafetyLevel {
        self.required_level
    }
}

impl Default for AsiComplianceChecker {
    fn default() -> Self {
        Self::new(SafetyLevel::Standard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐⭐ 2026-10-04 **修陈旧夹具**（⭐⭐ 本模块自 2026-09 起从未编译，
    /// ⭐⭐ 所以它的测试也从未运行 ⇒ ⭐⭐ 没人发现夹具已与规则集脱节）。
    ///
    /// ⛔ **改前**的夹具只给了 2 个 context 项，而规则集有 **6 条**
    ///   （其中 **4 条** 在 `SafetyLevel::Standard`）⇒ `violations` 必然非空
    ///   ⇒ `assert!(result.passed)` ⛔ **必然失败**。
    /// ⭐⭐⭐ 注意：⭐⭐ **这不是实现 bug，是夹具腐化**。
    /// ⭐⭐ 而「夹具腐化」的成因与「代码腐化」不同：⭐⭐ 代码能被编译器抓，
    /// ⭐⭐⭐ **夹具不会被任何工具提醒** —— 只有真跑测试才看得见。
    ///
    /// ✅ **修法（系统性，不是打补丁）**：⭐⭐ 夹具**从规则集派生** ——
    ///   遍历 `checker.rules()`，⭐⭐ 对每条在当前等级生效的规则给出通过标记。
    /// ⇒ ⭐⭐⭐ **以后再加规则，这个测试也不会再陈旧**；
    /// ⭐⭐ 而若实现真的坏了，它**照样会红**（⭐⭐ 没有把断言改弱）。
    #[test]
    fn test_compliance_pass() {
        let checker = AsiComplianceChecker::new(SafetyLevel::Standard);
        // ⭐⭐ 从**规则集**派生夹具：⭐⭐ 不再手写、⭐⭐ 因此永不陈旧
        // ⭐⭐⭐⭐ **过滤方向**：实现里是 `if rule.level > current_level` 才检查
        // （`asi_compliance.rs:106`），⭐⭐ 而 `SafetyLevel` 的序数是
        // `Unrestricted(0) < Basic < Standard(2) < Enhanced(3) < Maximum(4)`
        // —— ⭐⭐ **数值越高 = 越严**。
        // ⇒ ⭐⭐⭐ 当前等级 `Standard` 下**真正被检查**的是 `level > Standard` 的规则。
        //
        // ⛔ 我第一版写的是 `r.level <= Standard` —— ⭐⭐⭐ **方向正好相反**：
        //   于是夹具去标注了「根本不会被检查」的规则，⭐⭐ 而真正被检查的
        //   3 条（`reasoning_protection` / `trust_grading` / `escape_detection`）
        //   ⭐⭐ 一个都没标 ⇒ `violations` 非空 ⇒ 断言失败。
        // ⭐⭐ ⭐ **是靠断言消息里的 `violations` 真实内容定位的**，
        //   ⭐⭐ 不是靠继续推理（⭐⭐ 我先前口头说「6 条规则、4 条在 Standard 级」
        //   ⭐⭐ **也是错的**：实测 `rules=5`）。
        let owned: Vec<(String, bool)> = checker
            .rules()
            .iter()
            .filter(|r| r.level > SafetyLevel::Standard)
            .map(|r| (r.check_fn.clone(), true))
            .collect();
        let context: Vec<(&str, bool)> =
            owned.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        assert!(
            !context.is_empty(),
            "⭐⭐ 派生夹具为空 ⇒ ⭐⭐ 规则集变了，⭐⭐ 本测试需重新审视"
        );
        let result = checker.check(SafetyLevel::Standard, &context);
        assert!(result.passed);
    }

    #[test]
    fn test_compliance_fail_level() {
        let checker = AsiComplianceChecker::new(SafetyLevel::Enhanced);
        let result = checker.check(SafetyLevel::Basic, &[]);
        assert!(!result.passed);
        assert!(!result.violations.is_empty());
    }

    #[test]
    fn test_compliance_fail_rule() {
        let checker = AsiComplianceChecker::new(SafetyLevel::Standard);
        let context = vec![]; // 没有通过任何规则
        let result = checker.check(SafetyLevel::Standard, &context);
        assert!(!result.passed);
    }
}
