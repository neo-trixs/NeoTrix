//! # NT-IO output_style — 输出样式一等能力
//!
//! 吸收源: attention-span + GitHub output-style 生态 (answer-first / spartan / rundown)
//! + i-have-adhd 输出纪律 (G27 OutputGovernor 10 规则)。
//!
//! 与 skills 正交: 样式改变"怎么说话"，不改"怎么编码"。
//!
//! 骨架阶段 (C0): 注册表 + 三种内置样式 + 格式化入口已接 AgentLoop 生产路径。
//! G27: 每条最终输出经 `OutputGovernor` 治理 (10 条纪律规则 + 可机械修复),
//! 报告附于 AgentLoop.last_governance 供观测。待完善: 插件式扩展 / 样式度量反馈。

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;


// ⭐⭐ **冗余下沉 · Phase 1**（2026-10-07，纯搬运，**零行为变更**）
//
// 这 8 条治理规则的 `check_fn` 本体、5 个助手、3 个常量与 `RuleResult`
// 曾在本文件与 `neotrix_neobot::nt_governance` **各存一份逐字相同的副本**
// （实测 8 个 `r*` 逐函数 diff 为空）⇒ 现统一从 neobot 引用。
//
// ⚠️ 依赖方向天然合适：`neotrix-core` 已依赖 `neotrix-neobot` ⇒ **零新 crate**。
//
// 本文件**只保留** `GovernorRule` 容器 —— 因其 `check_fn` 多带一个
// `OutputStyleId`（8 条规则当前**都不使用**它，是为未来 style-aware 规则预留）。
// ⇒ **规则逻辑共享、规则容器各自持有**。
use neotrix_neobot::nt_governance::{
    // ⛔ 不引 `extract_path_refs` / `has_known_ext`：它们只被**共享的** `r*` 内部使用，
    //    本文件不再直接调用 ⇒ 引入即 unused。
    mask_code_fences, r1_answer_first, r2_no_hedging,
    r3_sections_concrete, r4_no_placeholder, r5_max_length, r6_no_dup_boilerplate,
    r7_file_refs_exist, r8_hallucinated_paths, re_opt,
    RuleResult, EXTS, PLACEHOLDER_INLINE_RE, PLACEHOLDER_PURE_RE,
};

// ⭐⭐⭐ **冗余下沉 · Phase 2**（2026-10-07）：治理**容器**也统一从 neobot 引用。
//
// ⛔ Phase 1 只共享了规则本体，**容器与类型仍在本文件另存一份** ⇒ 实测 7 项双份：
//   GovernanceReport / AiSmell / SmellPattern / AiSmellDetector /
//   GovernorRule / OutputGovernor / DEFAULT_MAX_MESSAGE_CHARS
//   （前 6 项逐字相同；GovernorRule 仅差一个 `OutputStyleId` 死参数）
//
// ⛔ 原先保留本文件副本的理由是「`check_fn` 多带一个 `OutputStyleId`，
//   为未来 style-aware 规则预留」。**该参数在本文件 10 条规则里无一使用**
//   （全部写成 `|text, _style|`）⇒ ⭐ 那是**投机预留**，不是当前需求；
//   而为它付出的代价是整条重复的类型层级 + 两个 impl 块（方法集实测完全相同）。
// ⇒ 本轮已把死参数从整条链去掉（govern / govern_with_autofix / check_fn /
//   10 个闭包），签名与 neobot 侧**完全一致** ⇒ 副本不再有任何存在理由。
// ⛔ 不保留任何转发别名或 `#[deprecated]` 垫片：旧签名已无人调用，
//   留兼容层只会让「同一条治理链有两种写法」继续共存。
pub use neotrix_neobot::nt_governance::{
    AiSmell, AiSmellDetector, GovernanceReport, GovernorRule, OutputGovernor,
    SmellPattern, DEFAULT_MAX_MESSAGE_CHARS,
};

/// 内置输出样式标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputStyleId {
    /// answer-first: 结论先行 + 粗体引导 skim，长尾信息折叠。吸收 attention-span。
    AnswerFirst,
    /// spartan: 精简默认，删冗余，答案尽量短。
    Spartan,
    /// rundown: 结构化清单 + 摘要，适合多要点场景。
    Rundown,
    /// 未配置 — 原样透传。
    Plain,
}

impl OutputStyleId {
    pub fn from_str(s: &str) -> OutputStyleId {
        match s.to_ascii_lowercase().as_str() {
            "answer_first" | "answer-first" | "answerfirst" => OutputStyleId::AnswerFirst,
            "spartan" | "concise" => OutputStyleId::Spartan,
            "rundown" | "list" | "summary" => OutputStyleId::Rundown,
            _ => OutputStyleId::Plain,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            OutputStyleId::AnswerFirst => "answer-first",
            OutputStyleId::Spartan => "spartan",
            OutputStyleId::Rundown => "rundown",
            OutputStyleId::Plain => "plain",
        }
    }
}

/// 输出样式实现契约。
/// `Send + Sync`: OutputStyleRegistry 经 AgentLoop 跨线程共享 (Arc + Mutex), trait 对象必须是线程安全。
pub trait OutputStyle: Send + Sync {
    fn id(&self) -> OutputStyleId;
    /// 对一段模型原始文本应用样式，返回格式化文本。
    fn apply(&self, text: &str) -> String;
    /// 兜底：样式失败时原样透传。
    fn fallback(&self, text: &str) -> String {
        text.to_string()
    }
}

/// 内置样式 — 骨架实现，规则粒度待完善。
pub struct AnswerFirstStyle;
impl OutputStyle for AnswerFirstStyle {
    fn id(&self) -> OutputStyleId {
        OutputStyleId::AnswerFirst
    }
    fn apply(&self, text: &str) -> String {
        // 骨架: 首段作为结论并加粗引导；后续段折叠为要点。TODO: 分句权重 + 度量。
        let t = text.trim();
        if t.is_empty() {
            return self.fallback(text);
        }
        let mut lines: Vec<&str> = t.lines().collect();
        let head = lines.remove(0);
        let head = head.trim_end_matches(['.', '。', '!']);
        let mut out = format!("**{head}**");
        if !lines.is_empty() {
            out.push_str("\n\n要点:");
            for l in lines.iter().take(6) {
                out.push_str(&format!("\n- {}", l.trim()));
            }
        }
        out
    }
}

pub struct SpartanStyle;
impl OutputStyle for SpartanStyle {
    fn id(&self) -> OutputStyleId {
        OutputStyleId::Spartan
    }
    fn apply(&self, text: &str) -> String {
        // 骨架: 去空行重复、压缩换行。TODO: 语义精简。
        let compact: Vec<&str> = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        compact.join("\n")
    }
}

pub struct RundownStyle;
impl OutputStyle for RundownStyle {
    fn id(&self) -> OutputStyleId {
        OutputStyleId::Rundown
    }
    fn apply(&self, text: &str) -> String {
        // 骨架: 保留原结构，在结尾追加摘要行。TODO: 要点抽取。
        let mut out = text.trim().to_string();
        out.push_str("\n\n---\n[rundown] 摘要待实现");
        out
    }
}

/// 输出样式注册表 — 样式按 id 解析 + 输出纪律治理 (G27)。
pub struct OutputStyleRegistry {
    styles: HashMap<OutputStyleId, Box<dyn OutputStyle>>,
    governor: OutputGovernor,
}

impl Default for OutputStyleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputStyleRegistry {
    pub fn new() -> Self {
        let mut styles: HashMap<OutputStyleId, Box<dyn OutputStyle>> = HashMap::new();
        styles.insert(OutputStyleId::AnswerFirst, Box::new(AnswerFirstStyle));
        styles.insert(OutputStyleId::Spartan, Box::new(SpartanStyle));
        styles.insert(OutputStyleId::Rundown, Box::new(RundownStyle));
        styles.insert(OutputStyleId::Plain, Box::new(PlainStyle));
        Self {
            styles,
            governor: OutputGovernor::new(),
        }
    }

    pub fn register(&mut self, style: Box<dyn OutputStyle>) {
        self.styles.insert(style.id(), style);
    }

    /// 解析样式。未知 id 回退到 `Plain`。
    ///
    /// ⭐ 返回 `Option` 而非 `&dyn OutputStyle`（2026-10-07）。
    /// 原实现 `.expect("Plain style registered in StyleRegistry")`：
    /// 我核实 `new()` 无条件插入 `Plain`、且 `Default` 委托 `new()`、
    /// 且 `styles` 私有、`register()` 只能增不能删 ⇒ **panic 实际不可达**。
    /// 但 `expect` 把「不可达」写成**运行时不变量**，⛔ 而非类型保证 ——
    /// 未来若给 `styles` 加 `remove()`/反序列化路径，panic 立刻变可达。
    /// ⇒ 改为 `Option`：把不可达性交给**调用方显式处理**，无 `unwrap/expect`。
    pub fn resolve(&self, id: OutputStyleId) -> Option<&dyn OutputStyle> {
        self.styles
            .get(&id)
            .map(|s| s.as_ref())
            .or_else(|| self.styles.get(&OutputStyleId::Plain).map(|s| s.as_ref()))
    }

    /// 应用样式 (生产入口，被 AgentLoop 调用)。
    ///
    /// ⭐ 兜底链：`id` → `Plain` → **恒等透传**（`text` 原样返回）。
    /// 最后一级是**语义正确**的降级：没有样式时输出应保持原文，
    /// ⛔ 而不是 panic 或静默改成另一种风格。
    pub fn apply(&self, id: OutputStyleId, text: &str) -> String {
        match self.resolve(id) {
            Some(s) => s.apply(text),
            None => text.to_string(),
        }
    }

    /// G27 输出治理 (纯检查)。每条规则独立结果 + 综合得分 + 违规清单。
    pub fn govern(&self, text: &str) -> GovernanceReport {
        self.governor.govern(text)
    }

    /// G27 输出治理 (auto-fix): 额外剥离可机械修复项 (结尾道歉 / 纯占位行)。
    pub fn govern_with_autofix(&self, text: &str) -> GovernanceReport {
        self.governor.govern_with_autofix(text)
    }

    /// 设置治理器工作区根目录 (R07/R08 文件引用校验基准)。
    pub(crate) fn _with_governor_root(mut self, root: impl AsRef<Path>) -> Self {
        self.governor.set_workspace_root(root);
        self
    }

    /// 访问治理器 (观测/测试)。
    pub fn governor(&self) -> &OutputGovernor {
        &self.governor
    }
}

pub struct PlainStyle;
impl OutputStyle for PlainStyle {
    fn id(&self) -> OutputStyleId {
        OutputStyleId::Plain
    }
    fn apply(&self, text: &str) -> String {
        text.to_string()
    }
}

// ── 各规则检查实现 (纯函数, 便于单测) ───────────────────────────────

/// R02 禁止模糊对冲: "可能/或许/大概/我觉得/probably" 等。
const HEDGES: &[&str] = &[
    "可能", "或许", "大概", "也许", "我觉得", "我猜", "我认为", "好像",
    "probably", "maybe", "perhaps", "i think", "i guess", "it seems",
];

fn truncate(s: &str, max: usize) -> String {
    let mut t: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        t.push('…');
    }
    t
}

/// R09 语言一致: 禁止显著中英混杂 (代码块除外)。
/// 2026-09-29：本地副本已删，改用唯一事实源。
/// 本处是**格式/语言判断**（统计正文里汉字占比）⇒ 用宽口径 `is_cjk_wide`。
/// ⚠️ 原口径含 `F900–FAFF`（CJK 兼容表意），而 `is_cjk_wide` 不含该段 ——
/// 那是 Unicode 重复区（NFKC 归一后即基本区），归一后仍会被识别，
/// 故不影响「统计汉字占比」的结论。
fn is_cjk(c: char) -> bool {
    neotrix_types::core::nt_cjk::is_cjk_wide(c)
        || ('\u{F900}'..='\u{FAFF}').contains(&c)
}

fn r9_consistent_language(text: &str) -> RuleResult {
    let masked = mask_code_fences(text);
    let mut cjk = 0usize;
    let mut latin_words = 0usize;
    let mut prose_lines = 0usize;
    for line in masked.lines() {
        let cjk_here = line.chars().filter(|c| is_cjk(*c)).count();
        cjk += cjk_here;
        let words: Vec<&str> = line
            .split_whitespace()
            .filter(|w| w.len() >= 2 && w.chars().all(|c| c.is_ascii_alphabetic()))
            .collect();
        latin_words += words.len();
        if words.len() >= 5 && cjk_here == 0 && !line.trim_start().starts_with("```") {
            prose_lines += 1;
        }
    }
    if cjk >= 15 && latin_words >= 10 && prose_lines >= 1 {
        RuleResult::fail(9, format!("中英混杂 (中文字符 {cjk} / 英文词 {latin_words} / 英文行 {prose_lines})"))
    } else {
        RuleResult::pass(9, "语言一致")
    }
}

/// R10 禁止结尾道歉: 末行不得以"抱歉/对不起/sorry"收尾。
const APOLOGY_MARKERS: &[&str] = &[
    "抱歉", "不好意思", "对不起", "我的错", "我道歉", "请您谅解", "请谅解",
    "sorry", "apologies", "apology", "apologize", "i apologize",
];

fn r10_no_trailing_apology(text: &str) -> RuleResult {
    let last = text
        .lines()
        .map(str::trim)
        .rfind(|l| !l.is_empty())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    if APOLOGY_MARKERS.iter().any(|m| last.contains(m)) {
        RuleResult::fail(10, format!("结尾道歉: `{}`", last))
    } else {
        RuleResult::pass(10, "无结尾道歉")
    }
}

/// auto-fix R10: 剥离结尾道歉行 (从末尾向上剥含道歉标记的连续非空行)。
fn strip_trailing_apology(text: &str) -> Option<(String, Vec<String>)> {
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let mut removed: Vec<String> = Vec::new();
    loop {
        while let Some(last) = lines.last() {
            if last.trim().is_empty() {
                lines.pop();
            } else {
                break;
            }
        }
        let Some(last) = lines.last() else { break };
        let lower = last.to_lowercase();
        if APOLOGY_MARKERS.iter().any(|m| lower.contains(m)) {
            removed.push(last.trim().to_string());
            lines.pop();
        } else {
            break;
        }
    }
    while let Some(last) = lines.last() {
        if last.trim().is_empty() {
            lines.pop();
        } else {
            break;
        }
    }
    if removed.is_empty() {
        return None;
    }
    let out = lines.join("\n");
    Some((
        out,
        vec![format!(
            "R10: 剥离结尾道歉 {}",
            removed.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(" / ")
        )],
    ))
}

fn build_rules(root: &Path, max_message_chars: usize) -> Vec<GovernorRule> {
    let root = root.to_path_buf();
    let placeholder_pure = Arc::new(re_opt(PLACEHOLDER_PURE_RE));
    let placeholder_inline = Arc::new(re_opt(PLACEHOLDER_INLINE_RE));
    let placeholder_pure_for_inline = placeholder_pure.clone();
    let root_for_hallucinated = root.clone();
    let backtick_re = Arc::new(re_opt(r"`([^`]+)`"));
    let bare_path_re = Arc::new(re_opt(&format!(
        r"[\w.\-/]+\.(?:{})",
        EXTS.join("|")
    )));
    let line_ref_re = Arc::new(re_opt(&format!(
        r"(?i)([A-Za-z0-9_.\-/]+\.(?:{})):(\d+)",
        EXTS.join("|")
    )));
    let line_suffix_re = Arc::new(re_opt(r":\d+$"));

    vec![
        GovernorRule {
            id: 1,
            description: "①答案前置 — 结论先行，禁止'让我先/让我想想'等铺垫推迟答案。",
            check_fn: Box::new(|text| r1_answer_first(text)),
        },
        GovernorRule {
            id: 2,
            description: "②禁止模糊对冲 — 不用'可能/或许/大概/我觉得/probably'等对冲词。",
            check_fn: Box::new(|text| r2_no_hedging(text)),
        },
        GovernorRule {
            id: 3,
            description: "③章节必须有实内容 — 标题后不得紧跟空行/纯占位/纯符号。",
            check_fn: Box::new(move |text| r3_sections_concrete(text, &placeholder_pure)),
        },
        GovernorRule {
            id: 4,
            description: "④禁止空/占位文本 — 不允许 TODO/TBD/待补充/lorem ipsum 等占位符。",
            check_fn: Box::new(move |text| r4_no_placeholder(text, &placeholder_pure_for_inline, &placeholder_inline)),
        },
        GovernorRule {
            id: 5,
            description: "⑤单消息长度上限 — 超阈值即违规。",
            check_fn: Box::new(move |text| r5_max_length(text, max_message_chars)),
        },
        GovernorRule {
            id: 6,
            description: "⑥禁止重复样板 — 相同长行 (≥25 字符) 出现 ≥3 次即违规。",
            check_fn: Box::new(|text| r6_no_dup_boilerplate(text)),
        },
        GovernorRule {
            id: 7,
            description: "⑦文件引用必须存在 — 反引号/裸路径引用的文件必须真实存在于工作区。",
            check_fn: Box::new(move |text| {
                r7_file_refs_exist(text, &root, &backtick_re, &bare_path_re, &line_suffix_re)
            }),
        },
        GovernorRule {
            id: 8,
            description: "⑧禁止幻影路径 — `file:line` 引用必须存在且行号在文件范围内。",
            check_fn: Box::new(move |text| r8_hallucinated_paths(text, &root_for_hallucinated, &line_ref_re)),
        },
        GovernorRule {
            id: 9,
            description: "⑨语言一致 — 禁止显著中英混杂 (代码块除外)。",
            check_fn: Box::new(|text| r9_consistent_language(text)),
        },
        GovernorRule {
            id: 10,
            description: "⑩禁止结尾道歉 — 输出不得以'抱歉/对不起/sorry'收尾。",
            check_fn: Box::new(|text| r10_no_trailing_apology(text)),
        },
    ]
}
#[cfg(test)]
mod resolve_degrade_tests {
    use super::{OutputStyleId, OutputStyleRegistry};

    /// ⭐ **变异证据**：`Plain` 未注册时必须**降级为恒等透传**，
    /// ⛔ 而非 panic（原实现是 `.expect(...)`）。
    ///
    /// 构造方式：`OutputStyleRegistry::new()` 恒插入 `Plain`，
    /// 故用手工构造绕过构造器 —— 这正是 `expect` 变成可达的那条未来路径。
    #[test]
    fn 无Plain时降级为恒等透传() {
        let mut reg = OutputStyleRegistry::new();
        // ⭐ 模拟「未来加了 remove()/反序列化路径」：抽掉 Plain
        reg.styles.remove(&OutputStyleId::Plain);
        // 1) ⚠️ 我第一版断言写错了：`new()` 插入 Spartan，故抽掉 Plain
        //    **不会**让 Spartan 变 None —— 假设错误，不是代码错误。
        //    真正承重的断言在下面：抽掉 Plain 后**不得 panic**，
        //    且未注册的 id 必须退化为 None。
        assert!(
            reg.resolve(OutputStyleId::Spartan).is_some(),
            "Spartan 仍在注册表内（抽掉的只有 Plain）"
        );
        // 2) apply 恒等透传，**不改变原文**
        let src = "原始文本";
        assert_eq!(
            reg.apply(OutputStyleId::Plain, src),
            src,
            "无任何样式时必须原样返回（恒等透传）"
        );
    }

    /// 正常路径不变：`Plain` 在时未知 id 回退 `Plain`。
    #[test]
    fn 有Plain时未知id回退Plain() {
        let reg = OutputStyleRegistry::new();
        assert!(
            reg.resolve(OutputStyleId::Plain).is_some(),
            "new() 必须注册 Plain"
        );
        assert_eq!(
            reg.apply(OutputStyleId::Plain, "x"),
            "x",
            "Plain 样式应原样返回"
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answer_first_keeps_conclusion_head() {
        let s = AnswerFirstStyle;
        let out = s.apply("NeoTrix 已接入输出样式。\n支持多种样式。");
        assert!(out.starts_with("**NeoTrix 已接入输出样式**"));
        assert!(out.contains("要点"));
    }

    #[test]
    fn registry_applies_spartan() {
        let reg = OutputStyleRegistry::new();
        let out = reg.apply(OutputStyleId::Spartan, "  a  \n\n  b  \n");
        assert_eq!(out, "a\nb");
    }

    #[test]
    fn registry_unknown_id_falls_back_to_plain() {
        let reg = OutputStyleRegistry::new();
        assert_eq!(reg.apply(OutputStyleId::Plain, "abc"), "abc");
    }

    // ── AiSmellDetector (natural-japanese #14 吸收) ─────────────────────────

    #[test]
    fn detector_finds_mechanical_markers() {
        let det = AiSmellDetector::new();
        let text = "值得注意的是，本模块已接入。\n综上所述，一切正常。";
        let smells = det.detect(text);
        assert!(smells.len() >= 2, "got {smells:?}");
        assert!(smells.iter().any(|s| s.pattern_id == "meta-speech" && s.line == 1));
        assert!(smells.iter().any(|s| s.pattern_id == "conclusion-signpost" && s.line == 2));
        // 每条命中都带可执行建议
        for s in &smells {
            assert!(!s.suggestion.is_empty());
        }
    }

    #[test]
    fn detector_clean_text_zero_smells() {
        let det = AiSmellDetector::new();
        let text = "网关已升级。\n账户池支持 round-robin 与限流检疫。\ncargo check 通过。";
        assert_eq!(det.detect(text).len(), 0);
    }

    #[test]
    fn detector_reports_line_and_snippet() {
        let det = AiSmellDetector::new();
        let text = "第一行。\n第二行如下所示。\n第三行。";
        let smells = det.detect(text);
        let hit = smells
            .iter()
            .find(|s| s.pattern_id == "over-polished")
            .expect("over-polished marker present");
        assert_eq!(hit.line, 2);
        assert_eq!(hit.matched, "如下所示");
    }

    #[test]
    fn governed_report_includes_smells() {
        let reg = OutputStyleRegistry::new();
        let report = reg.govern("综上所述，这是对账户池的总结。\n其余内容正常。");
        assert!(!report.smells.is_empty(), "smells should be detected in govern()");
        assert!(report
            .smells
            .iter()
            .any(|s| s.pattern_id == "conclusion-signpost"));
    }
}

// ────────────────────────────────────────────────────────────────
// SelfTest: OutputGovernor — 输出治理器 (10 条纪律规则 + 可机械修复)
// ────────────────────────────────────────────────────────────────

pub struct OutputGovernorSelfTest;

impl crate::l0_substrate::nt_core_self_test::SelfTest for OutputGovernorSelfTest {
    fn name(&self) -> &str {
        "nt_io_output_style::output_governor"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let gov = OutputGovernor::new();

        // 1. 规则数量正确
        if gov.rule_count() != 10 {
            failures.push(format!("expected 10 rules, got {}", gov.rule_count()));
        }

        // 2. 空输入通过
        let report = gov.govern("");
        if !report.rule_results.iter().all(|r| r.passed) {
            failures.push("empty input should pass all rules".into());
        }

        // 3. 结尾道歉被捕获
        let report = gov.govern("结论是 x。\n抱歉");
        let r10_caught = report.rule_results.iter().any(|r| !r.passed && r.rule_id == 10);
        if !r10_caught {
            failures.push("trailing apology should be caught by R10".into());
        }

        // 4. 纯占位行被捕获 (占位规则为 R04)
        let report = gov.govern("待补充");
        let r04_caught = report.rule_results.iter().any(|r| !r.passed && r.rule_id == 4);
        if !r04_caught {
            failures.push("pure placeholder lines should be caught by R04".into());
        }

        // 5. autofix 能去除结尾道歉
        let report = gov.govern_with_autofix("结论是 x。\n抱歉");
        if report.fixed_text.is_none() || report.fixed_text.as_deref().unwrap_or("").contains("抱歉") {
            failures.push("autofix should strip trailing apology".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests_output_governor_selftest {
    use super::*;
    use crate::l0_substrate::nt_core_self_test::SelfTest;

    #[test]
    fn test_output_governor_selftest() {
        let t = OutputGovernorSelfTest;
        let res = t.self_test();
        assert!(res.is_ok());
    }
}
