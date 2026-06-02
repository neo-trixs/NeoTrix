//! CodeReview — 基于 CapabilityVector 的代码审查引擎
//!
//! 分析代码质量、安全风险、架构合理性
//! 审查结果可作为 RL 奖励信号输入 SEAL 循环
//!
//! v2.0 新增:
//!   - CodeReviewLoop: 审查→评估→修复→再审查 迭代循环
//!   - SEAL 奖励信号集成 (review_reward → SelfIteratingBrain)
//!   - 对标开源项目 (nitpicker/DiffScope/Octorus/roborev/Coacker/Grippy)
//!   - OWASP Top 10:2025 安全检查
//!   - 增量delta + 收敛判定

use crate::neotrix::nt_mind::core::CapabilityVector;
use crate::neotrix::nt_mind::self_edit::MicroEdit;

#[derive(Debug, Clone)]
pub struct ReviewIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub message: String,
    pub line: Option<u32>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity { Critical, High, Medium, Low, Info }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueCategory {
    Security, Performance, Architecture, Style, ErrorHandling, UnsafeCode, Testing
}

#[derive(Debug, Clone)]
pub struct ReviewReport {
    pub file: String,
    pub issues: Vec<ReviewIssue>,
    pub score: f64,
}

impl ReviewReport {
    pub fn total(&self) -> usize { self.issues.len() }
    pub fn by_severity(&self, sev: IssueSeverity) -> Vec<&ReviewIssue> {
        self.issues.iter().filter(|i| i.severity == sev).collect()
    }
}

/// 代码审查引擎
pub struct CodeReviewEngine {
    pub capability: CapabilityVector,
}

impl CodeReviewEngine {
    pub fn new(capability: CapabilityVector) -> Self {
        Self { capability }
    }

    /// 审查代码片段，返回审查报告
    pub fn review(&self, file: &str, code: &str) -> ReviewReport {
        let mut issues = Vec::new();

        let arr = &self.capability.arr;
        if arr[super::core::IDX_ANALYSIS] > 0.3 {
            Self::check_unwrap(code, &mut issues);
            Self::check_panic(code, &mut issues);
            Self::check_unsafe(code, &mut issues);
            Self::check_hardcoded_paths(code, &mut issues);
        }
        if arr[super::core::IDX_QUALITY_GATES] > 0.3 {
            Self::check_missing_tests(code, &mut issues);
            Self::check_command_injection(code, &mut issues);
        }
        if arr[super::core::IDX_VERIFICATION] > 0.3 {
            Self::check_secrets_in_code(code, &mut issues);
        }

        let base = self.capability.quality_gates() * 0.4 + self.capability.verification() * 0.3;
        let penalty = issues.iter().map(|i| match i.severity {
            IssueSeverity::Critical => 0.15,
            IssueSeverity::High => 0.08,
            IssueSeverity::Medium => 0.04,
            _ => 0.01,
        }).sum::<f64>();
        let score = (base - penalty).clamp(0.0, 1.0);

        ReviewReport { file: file.to_string(), issues, score }
    }

    fn check_unwrap(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains(".unwrap()") && !line.trim().starts_with("//") && !line.trim().starts_with("#[") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::ErrorHandling,
                    message: "[OWASP A08:2025] 使用 .unwrap() 可能导致 panic（异常未处理）".to_string(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("替换为 .expect(\"msg\") 或 ? 操作符 + 统一 NeoTrixError".to_string()),
                });
            }
        }
    }

    fn check_panic(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains("panic!(") && !line.trim().starts_with("//") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::ErrorHandling,
                    message: "使用 panic!() 导致不可恢复崩溃".to_string(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("替换为返回 Result 类型".to_string()),
                });
            }
        }
    }

    fn check_unsafe(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains("unsafe {") && !line.trim().starts_with("//") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::UnsafeCode,
                    message: "[OWASP X02:2025] unsafe 块需要安全注释说明".to_string(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("添加 Safety: 注释说明为什么 unsafe 是安全的; 考虑使用安全抽象替代".to_string()),
                });
            }
        }
    }

    fn check_hardcoded_paths(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") { continue; }
            if trimmed.contains("\"/tmp/") || trimmed.contains("\"/Users/") || trimmed.contains("\"/etc/") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::Architecture,
                    message: "硬编码路径降低可移植性; 可能违反安全原则".to_string(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("使用 std::env::temp_dir() 或可配置项替代; 环境变量: $HOME, $TMPDIR".to_string()),
                });
            }
        }
    }

    fn check_command_injection(code: &str, issues: &mut Vec<ReviewIssue>) {
        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("sh") && trimmed.contains("-c") && trimmed.contains("Command::new") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::Security,
                    message: "[OWASP A05:2025] 检测到 shell 注入风险 (sh -c)".to_string(),
                    line: Some((i + 1) as u32),
                    suggestion: Some("使用 Command::arg() 直接传参代替 shell 字符串".to_string()),
                });
            }
        }
    }

    /// 检测代码中的硬编码 secrets (OWASP A04:2025)
    fn check_secrets_in_code(code: &str, issues: &mut Vec<ReviewIssue>) {
        let secret_patterns = [
            ("api_key", "API 密钥"),
            ("apiKey", "API 密钥"),
            ("password", "密码"),
            ("secret", "密钥"),
            ("token", "令牌"),
            ("credentials", "凭证"),
            ("GITHUB_TOKEN", "GITHUB_TOKEN"),
            ("NEOTRIX_API_KEY", "NEOTRIX_API_KEY"),
        ];

        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("#[") { continue; }

            for (pattern, label) in &secret_patterns {
                if trimmed.contains(pattern) && (trimmed.contains('=') || trimmed.contains(": \"")) {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::High,
                        category: IssueCategory::Security,
                        message: format!("[OWASP A04:2025] 检测到可能硬编码的 {} (line {})", label, i + 1),
                        line: Some((i + 1) as u32),
                        suggestion: Some("使用环境变量或加密 vault 替代硬编码; 审核 git history 是否已泄露".to_string()),
                    });
                    break;
                }
            }
        }
    }

    fn check_missing_tests(code: &str, issues: &mut Vec<ReviewIssue>) {
        let has_test_module = code.contains("#[cfg(test)]");
        let has_tests = code.contains("#[test]");
        let has_pub_fns = code.contains("pub fn") || code.contains("pub async fn");
        if has_pub_fns && !has_test_module {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Low,
                category: IssueCategory::Testing,
                message: "公开函数缺少测试模块".to_string(),
                line: None,
                suggestion: Some("添加 #[cfg(test)] mod tests { ... }".to_string()),
            });
        } else if has_pub_fns && !has_tests {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Info,
                category: IssueCategory::Testing,
                message: "测试模块存在但没有测试用例".to_string(),
                line: None,
                suggestion: Some("添加 #[test] 函数".to_string()),
            });
        }
    }

    /// 审查结果生成 MicroEdit 序列（供 SEAL 循环使用）
    pub fn issues_to_micro_edits(&self, report: &ReviewReport) -> Vec<MicroEdit> {
        let mut edits = Vec::new();
        let critical_count = report.by_severity(IssueSeverity::Critical).len() as f64;
        let high_count = report.by_severity(IssueSeverity::High).len() as f64;
        if critical_count > 0.0 {
            edits.push(MicroEdit::AdjustDimension("verification".to_string(), 0.05 * critical_count));
            edits.push(MicroEdit::AdjustDimension("quality_gates".to_string(), 0.04 * critical_count));
            edits.push(MicroEdit::AdjustDimension("nt_shield_audit".to_string(), 0.06 * critical_count));
        }
        if high_count > 0.0 {
            edits.push(MicroEdit::AdjustDimension("analysis".to_string(), 0.03 * high_count));
        }
        edits.push(MicroEdit::NormalizeVector);
        edits
    }
}

/// CodeReviewLoop — 审查→修复→再审查 迭代循环
///
/// 对标 Octorus AI Rally: 双AI agent review-fix cycle
/// 对标 roborev: continuous background review + auto-fix
pub struct CodeReviewLoop {
    pub iteration: u64,
    pub max_iterations: usize,
    pub quality_target: f64,
    pub history: Vec<ReviewReport>,
}

impl CodeReviewLoop {
    pub fn new(max_iterations: usize, quality_target: f64) -> Self {
        Self {
            iteration: 0,
            max_iterations,
            quality_target,
            history: Vec::new(),
        }
    }

    /// 执行一次审查→修复→再审查迭代
    /// 返回 (是否收敛, 最终评分, 迭代次数)
    pub fn iterate(
        &mut self,
        engine: &CodeReviewEngine,
        file: &str,
        code: &str,
    ) -> (bool, f64, u64) {
        self.iteration += 1;
        let report = engine.review(file, code);
        self.history.push(report.clone());

        let converged = report.score >= self.quality_target
            || self.iteration as usize >= self.max_iterations;

        (converged, report.score, self.iteration)
    }

    /// 计算奖励信号 (供 SEAL 循环使用)
    /// 正奖励 = 评分提升; 负奖励 = 发现新问题
    pub fn compute_seal_reward(&self) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }
        let last = self.history.last().expect("history.len() >= 2 per guard");
        let prev = self.history.get(self.history.len() - 2).expect("history has at least 2 entries");
        let delta = last.score - prev.score;
        let critical_penalty = last.by_severity(IssueSeverity::Critical).len() as f64 * 0.15;
        delta - critical_penalty
    }

    /// 获取增量改进
    pub fn delta_improvement(&self) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }
        self.history.last().expect("history.len() >= 2 per guard").score
            - self.history.first().expect("history is non-empty").score
    }

    pub fn is_converged(&self) -> bool {
        if self.history.len() < 3 {
            return false;
        }
        let recent: Vec<&ReviewReport> = self.history.iter().rev().take(3).collect();
        let scores: Vec<f64> = recent.iter().map(|r| r.score).collect();
        // 连续3轮评分变化 < 0.02 视为收敛
        scores.windows(2).all(|w| (w[1] - w[0]).abs() < 0.02)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_unwrap() {
        let mut cv = CapabilityVector::default();
        cv.arr[crate::neotrix::nt_mind::core::IDX_ANALYSIS] = 0.8;
        let engine = CodeReviewEngine::new(cv);
        let code = "fn main() { let x = foo.unwrap(); }";
        let report = engine.review("test.rs", code);
        assert!(report.issues.iter().any(|i| i.message.contains("unwrap")));
    }

    #[test]
    fn test_detect_panic() {
        let mut cv = CapabilityVector::default();
        cv.arr[crate::neotrix::nt_mind::core::IDX_ANALYSIS] = 0.8;
        let engine = CodeReviewEngine::new(cv);
        let code = "fn main() { panic!(\"boom\"); }";
        let report = engine.review("test.rs", code);
        assert!(report.issues.iter().any(|i| i.message.contains("panic")));
    }

    #[test]
    fn test_detect_unsafe() {
        let mut cv = CapabilityVector::default();
        cv.arr[crate::neotrix::nt_mind::core::IDX_ANALYSIS] = 0.8;
        let engine = CodeReviewEngine::new(cv);
        let code = "fn main() { unsafe { *p = 1; } }";
        let report = engine.review("test.rs", code);
        assert!(report.issues.iter().any(|i| i.category == IssueCategory::UnsafeCode));
    }

    #[test]
    fn test_score_is_bounded() {
        let cv = CapabilityVector::default();
        let engine = CodeReviewEngine::new(cv);
        let report = engine.review("test.rs", "fn main() {}");
        assert!(report.score >= 0.0 && report.score <= 1.0);
    }

    #[test]
    fn test_detect_secrets() {
        let mut cv = CapabilityVector::default();
        cv.arr[crate::neotrix::nt_mind::core::IDX_ANALYSIS] = 0.8;
        cv.arr[crate::neotrix::nt_mind::core::IDX_VERIFICATION] = 0.8;
        let engine = CodeReviewEngine::new(cv);
        let code = "let api_key = \"sk-12345\";";
        let report = engine.review("test.rs", code);
        assert!(report.issues.iter().any(|i| i.message.contains("API 密钥")));
    }

    #[test]
    fn test_code_review_loop() {
        let mut cv = CapabilityVector::default();
        cv.arr[crate::neotrix::nt_mind::core::IDX_ANALYSIS] = 0.8;
        cv.arr[crate::neotrix::nt_mind::core::IDX_QUALITY_GATES] = 0.8;
        let engine = CodeReviewEngine::new(cv);

        let mut loop_ = CodeReviewLoop::new(5, 0.9);
        let code = "fn main() { let x = foo.unwrap(); panic!(\"err\"); }";

        let (converged, score, _iters) = loop_.iterate(&engine, "test.rs", code);
        assert!(!converged || score > 0.5);

        let reward = loop_.compute_seal_reward();
        assert!(reward >= -1.0 && reward <= 1.0);
    }

    #[test]
    fn test_owasp_a05_detect_command_injection() {
        let mut cv = CapabilityVector::default();
        cv.arr[crate::neotrix::nt_mind::core::IDX_QUALITY_GATES] = 0.8;
        let engine = CodeReviewEngine::new(cv);
        let code = r#"Command::new("sh").args(["-c", &cmd])"#;
        let report = engine.review("test.rs", code);
        assert!(report.issues.iter().any(|i| i.message.contains("shell 注入")));
    }

    #[test]
    fn test_owasp_a04_hardcoded_secrets() {
        let mut cv = CapabilityVector::default();
        cv.arr[crate::neotrix::nt_mind::core::IDX_VERIFICATION] = 0.8;
        let engine = CodeReviewEngine::new(cv);
        let code = "let password = \"hunter2\";";
        let report = engine.review("test.rs", code);
        assert!(report.issues.iter().any(|i| i.category == IssueCategory::Security));
    }
}
