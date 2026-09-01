//! HarnessScaffold — 推理时 harness 脚手架 (AI4AI 强到弱能力迁移)
//!
//! 概念: harness = 确定性脚手架, 把弱模型的非稳定推理卸载为可验证代码。
//!   1. FormatEnforcer  — 严格答案格式强制 (如 `<answer>...</answer>`)
//!   2. DeterministicCode — 把可确定步骤卸载为代码块
//!   3. Decomposition   — 任务分解为编号子步骤
//!   4. Verifier        — 规则校验, 失败则覆盖弱模型结论 (overrule)
//!
//! 纯确定性代码: 无网络 / 无 tokio / 无文件 IO。

use std::collections::HashMap;

/// Harness 类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessKind {
    /// 严格答案格式强制 (把输出包装进 `<answer>...</answer>`)
    FormatEnforcer,
    /// 把可确定步骤卸载为代码块
    DeterministicCode,
    /// 任务分解为编号子步骤
    Decomposition,
    /// 规则校验, 失败覆盖弱模型结论
    Verifier,
}

/// 规则谓词 — 枚举形式, 便于 Debug/Clone/序列化
#[derive(Debug, Clone)]
pub enum CheckRule {
    /// 恰好一个 `<answer>...</answer>` 包裹, 且开闭顺序正确
    SingleAnswer,
    /// 文本必须以给定前缀开始
    StartsWith(String),
    /// 文本必须包含给定子串
    Contains(String),
    /// 文本不得包含给定子串
    Forbids(String),
    /// 字符长度必须在 [min, max] 内
    LengthInRange(usize, usize),
    /// 自定义函数指针谓词
    Custom(fn(&str) -> bool),
}

impl CheckRule {
    pub fn check(&self, text: &str) -> bool {
        match self {
            CheckRule::SingleAnswer => {
                let opens = text.matches("<answer>").count();
                let closes = text.matches("</answer>").count();
                opens == 1
                    && closes == 1
                    && text.find("<answer>").zip(text.find("</answer>"))
                        .map(|(o, c)| o < c)
                        .unwrap_or(false)
            }
            CheckRule::StartsWith(prefix) => text.trim_start().starts_with(prefix.as_str()),
            CheckRule::Contains(sub) => text.contains(sub.as_str()),
            CheckRule::Forbids(sub) => !text.contains(sub.as_str()),
            CheckRule::LengthInRange(min, max) => {
                let len = text.chars().count();
                len >= *min && len <= *max
            }
            CheckRule::Custom(f) => f(text),
        }
    }
}

/// 单条检查
#[derive(Debug, Clone)]
pub struct HarnessCheck {
    pub id: String,
    pub description: String,
    pub rule: CheckRule,
}

impl HarnessCheck {
    pub fn new(id: impl Into<String>, description: impl Into<String>, rule: CheckRule) -> Self {
        Self { id: id.into(), description: description.into(), rule }
    }
}

/// Harness 规格
#[derive(Debug, Clone)]
pub struct HarnessSpec {
    pub kind: HarnessKind,
    pub name: String,
    pub checks: Vec<HarnessCheck>,
    pub builder_effort: f64,
}

impl HarnessSpec {
    pub fn new(kind: HarnessKind, name: impl Into<String>, builder_effort: f64) -> Self {
        Self {
            kind,
            name: name.into(),
            checks: Vec::new(),
            builder_effort: builder_effort.clamp(0.0, 1.0),
        }
    }

    pub fn with_check(mut self, check: HarnessCheck) -> Self {
        self.checks.push(check);
        self
    }
}

/// Harness 判定
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessVerdict {
    /// 全部检查通过
    WeakPass,
    /// 存在检查失败 (非 Verifier 或 Verifier 通过)
    WeakFail,
    /// Verifier 校验失败, 覆盖弱模型结论
    VerifierOverrule,
}

/// Harness 应用结果
#[derive(Debug, Clone, PartialEq)]
pub struct HarnessResult {
    pub formatted: String,
    pub passed_checks: usize,
    pub total_checks: usize,
    pub verdict: HarnessVerdict,
}

/// 单条 effort band 统计
#[derive(Debug, Clone, Default)]
pub struct EffortStats {
    pub attempts: u32,
    pub successes: u32,
}

impl EffortStats {
    pub fn success_rate(&self) -> Option<f64> {
        if self.attempts == 0 {
            None
        } else {
            Some(self.successes as f64 / self.attempts as f64)
        }
    }
}

/// 推理时 harness 脚手架 — 确定性验证 + 格式强制
#[derive(Debug, Clone, Default)]
pub struct HarnessScaffold {
    specs: Vec<HarnessSpec>,
    efforts: HashMap<u32, EffortStats>,
}

/// 把 effort [0,1] 映射到 band 0-9 (用于成功率分组)
pub fn effort_band(effort: f64) -> u32 {
    (effort.clamp(0.0, 1.0) * 10.0) as u32
}

impl HarnessScaffold {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_spec(&mut self, spec: HarnessSpec) {
        self.specs.push(spec);
    }

    pub fn specs(&self) -> &[HarnessSpec] {
        &self.specs
    }

    pub fn spec_by_name(&self, name: &str) -> Option<&HarnessSpec> {
        self.specs.iter().find(|s| s.name == name)
    }

    /// 应用单个 spec: 格式强制 + 规则检查
    pub fn apply(&self, spec: &HarnessSpec, weak_output: &str) -> HarnessResult {
        let formatted = enforce_format(&spec.kind, weak_output);
        let total = spec.checks.len();
        let passed = spec.checks.iter().filter(|c| c.rule.check(&formatted)).count();
        let verdict = verdict_for(&spec.kind, passed, total);
        HarnessResult { formatted, passed_checks: passed, total_checks: total, verdict }
    }

    /// 按名字查找 spec 并应用 (找不到则返回 None)
    pub fn apply_named(&self, name: &str, weak_output: &str) -> Option<HarnessResult> {
        self.spec_by_name(name).map(|s| self.apply(s, weak_output))
    }

    /// 默认应用第一个 spec (单 harness 场景)
    pub fn apply_default(&self, weak_output: &str) -> Option<HarnessResult> {
        self.specs.first().map(|s| self.apply(s, weak_output))
    }

    /// 记录一次 builder effort 尝试
    pub fn record_effort(&mut self, effort: f64, success: bool) {
        let band = effort_band(effort);
        let stats = self.efforts.entry(band).or_default();
        stats.attempts += 1;
        if success {
            stats.successes += 1;
        }
    }

    /// 查询某 effort 的成功率
    pub fn success_rate(&self, effort: f64) -> Option<f64> {
        self.efforts.get(&effort_band(effort)).and_then(|s| s.success_rate())
    }

    /// 某 band 的尝试统计
    pub fn band_stats(&self, band: u32) -> Option<&EffortStats> {
        self.efforts.get(&band)
    }

    /// 各 band 汇总 (band 升序)
    pub fn effort_bands(&self) -> Vec<(u32, &EffortStats)> {
        let mut bands: Vec<(u32, &EffortStats)> = self.efforts.iter().map(|(b, s)| (*b, s)).collect();
        bands.sort_by_key(|(b, _)| *b);
        bands
    }

    /// 记录一次基于 spec.builder_effort 的自动尝试
    pub fn apply_and_record(&mut self, name: &str, weak_output: &str) -> Option<HarnessResult> {
        let spec = self.spec_by_name(name)?;
        let result = self.apply(spec, weak_output);
        let success = matches!(result.verdict, HarnessVerdict::WeakPass);
        self.record_effort(spec.builder_effort, success);
        Some(result)
    }
}

impl crate::core::nt_core_self_test::SelfTest for HarnessScaffold {
    fn name(&self) -> &str {
        "nt_act_orchestrator_harness_scaffold"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let spec = HarnessSpec::new(HarnessKind::FormatEnforcer, "answer", 0.2)
            .with_check(HarnessCheck::new("ans", "must be a single answer", CheckRule::SingleAnswer));
        let scaffold = HarnessScaffold::new();
        let ok = scaffold.apply(&spec, "  the answer is 42  ");
        if ok.verdict != HarnessVerdict::WeakPass {
            failures.push("FormatEnforcer should WeakPass a wrappable answer".into());
        }
        if !ok.formatted.contains("<answer>") {
            failures.push("FormatEnforcer should wrap output in <answer>".into());
        }
        let bad = scaffold.apply(&spec, "42\n</answer>\n<answer>");
        if bad.verdict != HarnessVerdict::WeakFail {
            failures.push("malformed answer should WeakFail".into());
        }
        let ver = HarnessSpec::new(HarnessKind::Verifier, "verify", 0.9)
            .with_check(HarnessCheck::new("forbid", "no junk", CheckRule::Forbids("junk".into())));
        let over = scaffold.apply(&ver, "the plan contains junk here");
        if over.verdict != HarnessVerdict::VerifierOverrule {
            failures.push("Verifier failure should overrule".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// 按 kind 做确定性格式强制
fn enforce_format(kind: &HarnessKind, text: &str) -> String {
    let trimmed = text.trim();
    match kind {
        HarnessKind::FormatEnforcer => {
            if trimmed.contains("<answer>") && trimmed.contains("</answer>") {
                trimmed.to_string()
            } else {
                format!("<answer>{}</answer>", trimmed)
            }
        }
        HarnessKind::DeterministicCode => {
            if trimmed.starts_with("```") && trimmed.ends_with("```") {
                trimmed.to_string()
            } else {
                format!("```\n{}\n```", trimmed)
            }
        }
        HarnessKind::Decomposition => {
            let lines: Vec<&str> = trimmed.lines().collect();
            let already_numbered = lines.iter().any(|l| {
                let t = l.trim();
                let first = t.split('.').next().unwrap_or("");
                first.trim().parse::<u32>().is_ok()
            });
            if already_numbered || trimmed.contains("Step ") || trimmed.contains("步骤 ") {
                trimmed.to_string()
            } else {
                lines
                    .iter()
                    .enumerate()
                    .map(|(i, l)| format!("{}. {}", i + 1, l.trim()))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        HarnessKind::Verifier => trimmed.to_string(),
    }
}

fn verdict_for(kind: &HarnessKind, passed: usize, total: usize) -> HarnessVerdict {
    let all_pass = passed == total;
    match kind {
        HarnessKind::Verifier if !all_pass => HarnessVerdict::VerifierOverrule,
        _ if all_pass => HarnessVerdict::WeakPass,
        _ => HarnessVerdict::WeakFail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer_spec() -> HarnessSpec {
        HarnessSpec::new(HarnessKind::FormatEnforcer, "answer", 0.2)
            .with_check(HarnessCheck::new("ans", "single answer", CheckRule::SingleAnswer))
    }

    #[test]
    fn format_enforcer_wraps_and_passes() {
        let scaffold = HarnessScaffold::new();
        let r = scaffold.apply(&answer_spec(), "  42 is the answer  ");
        assert_eq!(r.verdict, HarnessVerdict::WeakPass);
        assert_eq!(r.passed_checks, 1);
        assert_eq!(r.total_checks, 1);
        assert_eq!(r.formatted, "<answer>42 is the answer</answer>");
    }

    #[test]
    fn format_enforcer_keeps_existing_answer_markers() {
        let scaffold = HarnessScaffold::new();
        let r = scaffold.apply(&answer_spec(), "<answer>42</answer>");
        assert_eq!(r.formatted, "<answer>42</answer>");
        assert_eq!(r.verdict, HarnessVerdict::WeakPass);
    }

    #[test]
    fn malformed_answer_fails() {
        let scaffold = HarnessScaffold::new();
        let r = scaffold.apply(&answer_spec(), "</answer>\n<answer>");
        assert_eq!(r.verdict, HarnessVerdict::WeakFail);
        assert_eq!(r.passed_checks, 0);
    }

    #[test]
    fn verifier_overrules_weak_output() {
        let scaffold = HarnessScaffold::new();
        let ver = HarnessSpec::new(HarnessKind::Verifier, "verify", 0.9)
            .with_check(HarnessCheck::new("forbid", "no junk", CheckRule::Forbids("junk".into())));
        let r = scaffold.apply(&ver, "the plan contains junk here");
        assert_eq!(r.verdict, HarnessVerdict::VerifierOverrule);
        assert_eq!(r.passed_checks, 0);
    }

    #[test]
    fn verifier_pass_keeps_weak_pass() {
        let scaffold = HarnessScaffold::new();
        let ver = HarnessSpec::new(HarnessKind::Verifier, "verify", 0.9)
            .with_check(HarnessCheck::new("forbid", "no junk", CheckRule::Forbids("junk".into())));
        let r = scaffold.apply(&ver, "clean plan here");
        assert_eq!(r.verdict, HarnessVerdict::WeakPass);
        assert_eq!(r.passed_checks, 1);
    }

    #[test]
    fn decomposition_numbers_steps() {
        let scaffold = HarnessScaffold::new();
        let dec = HarnessSpec::new(HarnessKind::Decomposition, "decomp", 0.5)
            .with_check(HarnessCheck::new("num", "numbered steps", CheckRule::Contains("1.".into())));
        let r = scaffold.apply(&dec, "parse input\nvalidate\nemit output");
        assert!(r.formatted.contains("1. parse input"));
        assert_eq!(r.verdict, HarnessVerdict::WeakPass);
    }

    #[test]
    fn deterministic_code_wraps_in_fence() {
        let scaffold = HarnessScaffold::new();
        let det = HarnessSpec::new(HarnessKind::DeterministicCode, "code", 0.8)
            .with_check(HarnessCheck::new("fence", "fenced", CheckRule::StartsWith("```".into())));
        let r = scaffold.apply(&det, "fn main() {}");
        assert!(r.formatted.starts_with("```"));
        assert!(r.formatted.ends_with("```"));
        assert_eq!(r.verdict, HarnessVerdict::WeakPass);
    }

    #[test]
    fn partial_checks_weak_fail() {
        let scaffold = HarnessScaffold::new();
        let spec = HarnessSpec::new(HarnessKind::FormatEnforcer, "multi", 0.3)
            .with_check(HarnessCheck::new("ans", "single answer", CheckRule::SingleAnswer))
            .with_check(HarnessCheck::new("len", "length", CheckRule::LengthInRange(0, 5)));
        let r = scaffold.apply(&spec, "a very long answer that exceeds the window");
        assert_eq!(r.passed_checks, 1);
        assert_eq!(r.total_checks, 2);
        assert_eq!(r.verdict, HarnessVerdict::WeakFail);
    }

    #[test]
    fn custom_predicate_rule() {
        let scaffold = HarnessScaffold::new();
        let spec = HarnessSpec::new(HarnessKind::Verifier, "custom", 0.4)
            .with_check(HarnessCheck::new(
                "even",
                "length even",
                CheckRule::Custom(|t: &str| t.chars().count() % 2 == 0),
            ));
        assert_eq!(scaffold.apply(&spec, "abcd").verdict, HarnessVerdict::WeakPass);
        assert_eq!(scaffold.apply(&spec, "abc").verdict, HarnessVerdict::VerifierOverrule);
    }

    #[test]
    fn effort_bands_track_success_rate() {
        let mut scaffold = HarnessScaffold::new();
        for _ in 0..4 {
            scaffold.record_effort(0.1, true);
        }
        scaffold.record_effort(0.1, false);
        assert_eq!(scaffold.success_rate(0.15), Some(0.8));
        scaffold.record_effort(0.95, false);
        assert_eq!(scaffold.success_rate(0.99), Some(0.0));
        assert!(scaffold.success_rate(0.5).is_none());
    }

    #[test]
    fn higher_effort_band_correlates_with_success() {
        let mut scaffold = HarnessScaffold::new();
        scaffold.record_effort(0.1, false);
        scaffold.record_effort(0.1, false);
        scaffold.record_effort(0.9, true);
        scaffold.record_effort(0.9, true);
        let low = scaffold.success_rate(0.1).unwrap();
        let high = scaffold.success_rate(0.9).unwrap();
        assert!(high > low, "higher effort band should have higher success rate");
        let bands = scaffold.effort_bands();
        assert_eq!(bands.len(), 2);
        assert_eq!(bands[0].0, 1);
        assert_eq!(bands[1].0, 9);
    }

    #[test]
    fn apply_named_and_apply_and_record() {
        let mut scaffold = HarnessScaffold::new();
        let spec = answer_spec();
        scaffold.add_spec(spec.clone());
        assert!(scaffold.apply_named("answer", "42").is_some());
        assert!(scaffold.apply_named("missing", "42").is_none());
        assert!(scaffold.apply_default("42").is_some());
        let r = scaffold.apply_and_record("answer", "42").unwrap();
        assert_eq!(r.verdict, HarnessVerdict::WeakPass);
        assert_eq!(scaffold.success_rate(0.2), Some(1.0));
    }

    #[test]
    fn effort_band_mapping_clamps() {
        assert_eq!(effort_band(0.0), 0);
        assert_eq!(effort_band(0.5), 5);
        assert_eq!(effort_band(1.0), 10);
        assert_eq!(effort_band(2.0), 10);
    }
}
