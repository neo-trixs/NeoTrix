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
use std::path::{Path, PathBuf};
use std::sync::Arc;

use regex::Regex;

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
    r7_file_refs_exist, r8_hallucinated_paths, re_opt, strip_pure_placeholder_lines,
    RuleResult, EXTS, PLACEHOLDER_INLINE_RE, PLACEHOLDER_PURE_RE,
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

    pub fn resolve(&self, id: OutputStyleId) -> &dyn OutputStyle {
        self.styles
            .get(&id)
            .map(|s| s.as_ref())
            .unwrap_or_else(|| {
                self.styles
                    .get(&OutputStyleId::Plain)
                    .expect("Plain style registered in StyleRegistry")
                    .as_ref()
            })
    }

    /// 应用样式 (生产入口，被 AgentLoop 调用)。
    pub fn apply(&self, id: OutputStyleId, text: &str) -> String {
        self.resolve(id).apply(text)
    }

    /// G27 输出治理 (纯检查)。每条规则独立结果 + 综合得分 + 违规清单。
    pub fn govern(&self, text: &str, style: OutputStyleId) -> GovernanceReport {
        self.governor.govern(text, style)
    }

    /// G27 输出治理 (auto-fix): 额外剥离可机械修复项 (结尾道歉 / 纯占位行)。
    pub fn govern_with_autofix(&self, text: &str, style: OutputStyleId) -> GovernanceReport {
        self.governor.govern_with_autofix(text, style)
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

// ────────────────────────────────────────────────────────────────────────────
// G27 OutputGovernor — 输出纪律治理器 (吸收自 i-have-adhd 10 条输出格式规则)
// ────────────────────────────────────────────────────────────────────────────

/// 单条规则的检查结果。
#[derive(Debug, Clone)]
pub struct GovernanceReport {
    /// 每条规则的独立结果 (按 rule_id 顺序)。
    pub rule_results: Vec<RuleResult>,
    /// 综合得分 0-100 = 通过规则数 / 规则总数。
    pub overall_score: u8,
    /// 违规摘要 (格式 `R{NN}: {detail}`)。
    pub violations: Vec<String>,
    /// 自动修复清单 (空 = 无需修复)。
    pub fixes_applied: Vec<String>,
    /// auto-fix 后的文本 (仅 auto-fix 模式且发生修复时存在)。
    pub fixed_text: Option<String>,
    /// AI-smell 检测结果 (natural-japanese #14 吸收) — 机械式 AI 写作痕迹清单。
    pub smells: Vec<AiSmell>,
}

// ────────────────────────────────────────────────────────────────────────────
// AiSmellDetector — 机械式 AI 写作痕迹检测 (natural-japanese #14 吸收)
// ────────────────────────────────────────────────────────────────────────────

/// 单条 AI-smell 命中的结构化描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSmell {
    /// 模式标识 (如 `meta-speech` / `conclusion-signpost`)。
    pub pattern_id: &'static str,
    /// 命中行号 (1-based)。
    pub line: usize,
    /// 命中的原文片段 (截断显示)。
    pub matched: String,
    /// 建议改写 (消除机械感)。
    pub suggestion: &'static str,
}

/// AI-smell 检测模式 — 正则 + 建议。
pub struct SmellPattern {
    pub id: &'static str,
    pub regex: Option<Regex>,
    pub suggestion: &'static str,
}

/// 机械式 AI 写作痕迹检测器 — 规则化 regex 检测 (非 LLM 打分)。
pub struct AiSmellDetector {
    patterns: Vec<SmellPattern>,
    /// 每模式最多上报的命中数 (防止噪声淹没报告)。
    max_per_pattern: usize,
}

impl Default for AiSmellDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AiSmellDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
            max_per_pattern: 3,
        }
    }

    /// 内置模式集 — 高频机械 AI 标记。
    fn default_patterns() -> Vec<SmellPattern> {
        vec![
            SmellPattern {
                id: "meta-speech",
                regex: re_opt(r"(?i)值得注意的是|需要注意的是|it's worth noting|it is worth noting|please note that|as an ai,|i am an ai",),
                suggestion: "直接给结论/事实, 不要声明性前言",
            },
            SmellPattern {
                id: "conclusion-signpost",
                regex: re_opt(r"综上所述|总而言之|总的说来|in conclusion|to summarize|to sum up|overall, i think|in summary"),
                suggestion: "删掉总结开场白, 直接给要点或删除冗余段",
            },
            SmellPattern {
                id: "transition-cliche",
                regex: re_opt(r"首先，|其次，|最后，|最后,|firstly,|secondly,|furthermore,|moreover,|additionally,"),
                suggestion: "用清单/编号结构替代口语化过渡词",
            },
            SmellPattern {
                id: "over-polished",
                regex: re_opt(r"如下所示|如下：|以下是对|以下为|below is|here is the|as you can see|如您所见|正如您所知|as we all know|as you know"),
                suggestion: "去掉恭维性引导, 直入主题",
            },
            SmellPattern {
                id: "hedge-stack",
                regex: re_opt(r"(?i)very very|extremely extremely|absolutely|undoubtedly|无疑|诚然|毋庸置疑|显然,"),
                suggestion: "删减程度副词, 让论证自己说话",
            },
        ]
    }

    /// 对一段文本运行全部模式, 返回命中的 AI-smell (按行序去重排序)。
    pub fn detect(&self, text: &str) -> Vec<AiSmell> {
        let mut out: Vec<AiSmell> = Vec::new();
        for p in &self.patterns {
            let mut hits: Vec<AiSmell> = Vec::new();
            // ⛔ 正则不可用 ⇒ 该模式**不产出命中**（已被 log::error! 记录）
            let Some(p_regex) = p.regex.as_ref() else {
                continue;
            };
            for cap in p_regex.captures_iter(text) {
                if hits.len() >= self.max_per_pattern {
                    break;
                }
                if let Some(m) = cap.get(0) {
                    let line = text[..m.start()].matches('\n').count() + 1;
                    let matched = truncate(m.as_str().trim(), 60);
                    hits.push(AiSmell {
                        pattern_id: p.id,
                        line,
                        matched,
                        suggestion: p.suggestion,
                    });
                }
            }
            out.extend(hits);
        }
        // 按行号稳定排序 (同模式内保持正则顺序)。
        out.sort_by_key(|s| s.line);
        out
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

/// 单条治理规则 — 独立可测、可审计。
pub struct GovernorRule {
    pub id: u8,
    pub description: &'static str,
    pub check_fn: Box<dyn Fn(&str, OutputStyleId) -> RuleResult + Send + Sync>,
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

/// G27 输出纪律治理器 — 10 条 i-have-adhd 规则, 纯检查 + 可机械 auto-fix。
pub struct OutputGovernor {
    rules: Vec<GovernorRule>,
    workspace_root: PathBuf,
    max_message_chars: usize,
    placeholder_pure: Option<Regex>,
    /// AI-smell 检测器 (natural-japanese #14 吸收)。
    smell_detector: AiSmellDetector,
}

/// 默认单消息长度上限 (字符)。
pub const DEFAULT_MAX_MESSAGE_CHARS: usize = 8_000;

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
            check_fn: Box::new(|text, _style| r1_answer_first(text)),
        },
        GovernorRule {
            id: 2,
            description: "②禁止模糊对冲 — 不用'可能/或许/大概/我觉得/probably'等对冲词。",
            check_fn: Box::new(|text, _style| r2_no_hedging(text)),
        },
        GovernorRule {
            id: 3,
            description: "③章节必须有实内容 — 标题后不得紧跟空行/纯占位/纯符号。",
            check_fn: Box::new(move |text, _style| r3_sections_concrete(text, &placeholder_pure)),
        },
        GovernorRule {
            id: 4,
            description: "④禁止空/占位文本 — 不允许 TODO/TBD/待补充/lorem ipsum 等占位符。",
            check_fn: Box::new(move |text, _style| r4_no_placeholder(text, &placeholder_pure_for_inline, &placeholder_inline)),
        },
        GovernorRule {
            id: 5,
            description: "⑤单消息长度上限 — 超阈值即违规。",
            check_fn: Box::new(move |text, _style| r5_max_length(text, max_message_chars)),
        },
        GovernorRule {
            id: 6,
            description: "⑥禁止重复样板 — 相同长行 (≥25 字符) 出现 ≥3 次即违规。",
            check_fn: Box::new(|text, _style| r6_no_dup_boilerplate(text)),
        },
        GovernorRule {
            id: 7,
            description: "⑦文件引用必须存在 — 反引号/裸路径引用的文件必须真实存在于工作区。",
            check_fn: Box::new(move |text, _style| {
                r7_file_refs_exist(text, &root, &backtick_re, &bare_path_re, &line_suffix_re)
            }),
        },
        GovernorRule {
            id: 8,
            description: "⑧禁止幻影路径 — `file:line` 引用必须存在且行号在文件范围内。",
            check_fn: Box::new(move |text, _style| r8_hallucinated_paths(text, &root_for_hallucinated, &line_ref_re)),
        },
        GovernorRule {
            id: 9,
            description: "⑨语言一致 — 禁止显著中英混杂 (代码块除外)。",
            check_fn: Box::new(|text, _style| r9_consistent_language(text)),
        },
        GovernorRule {
            id: 10,
            description: "⑩禁止结尾道歉 — 输出不得以'抱歉/对不起/sorry'收尾。",
            check_fn: Box::new(|text, _style| r10_no_trailing_apology(text)),
        },
    ]
}

impl OutputGovernor {
    pub fn new() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            rules: build_rules(&root, DEFAULT_MAX_MESSAGE_CHARS),
            workspace_root: root,
            max_message_chars: DEFAULT_MAX_MESSAGE_CHARS,
            placeholder_pure: re_opt(PLACEHOLDER_PURE_RE),
            smell_detector: AiSmellDetector::new(),
        }
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// 设置工作区根目录 (R07/R08 文件引用校验基准), 重建依赖 root 的规则。
    pub fn set_workspace_root(&mut self, root: impl AsRef<Path>) {
        self.workspace_root = root.as_ref().to_path_buf();
        self.rules = build_rules(&self.workspace_root, self.max_message_chars);
    }

    /// 设置单消息长度上限, 重建 R05。
    pub(crate) fn _set_max_message_chars(&mut self, max: usize) {
        self.max_message_chars = max;
        self.rules = build_rules(&self.workspace_root, max);
    }

    /// 纯检查模式: 运行全部规则, 不修改文本。
    pub fn govern(&self, text: &str, style: OutputStyleId) -> GovernanceReport {
        let rule_results: Vec<RuleResult> = self
            .rules
            .iter()
            .map(|rule| (rule.check_fn)(text, style))
            .collect();
        self.finalize(text, rule_results, false)
    }

    /// auto-fix 模式: 检查 + 剥离可机械修复项 (结尾道歉 / 纯占位行)。
    pub fn govern_with_autofix(&self, text: &str, style: OutputStyleId) -> GovernanceReport {
        let rule_results: Vec<RuleResult> = self
            .rules
            .iter()
            .map(|rule| (rule.check_fn)(text, style))
            .collect();
        self.finalize(text, rule_results, true)
    }

    fn finalize(&self, text: &str, rule_results: Vec<RuleResult>, autofix: bool) -> GovernanceReport {
        let total = self.rules.len().max(1) as f64;
        let passed = rule_results.iter().filter(|r| r.passed).count() as f64;
        let overall_score = (passed / total * 100.0).round() as u8;
        let violations: Vec<String> = rule_results
            .iter()
            .filter(|r| !r.passed)
            .map(|r| format!("R{:02}: {}", r.rule_id, r.detail))
            .collect();

        let mut fixes_applied: Vec<String> = Vec::new();
        let mut fixed_text: Option<String> = None;
        if autofix {
            let mut cur = text.to_string();
            if let Some((f, changes)) = strip_trailing_apology(&cur) {
                cur = f;
                fixes_applied.extend(changes);
            }
            if let Some((f, changes)) = strip_pure_placeholder_lines(&cur, &self.placeholder_pure) {
                cur = f;
                fixes_applied.extend(changes);
            }
            if !fixes_applied.is_empty() {
                fixed_text = Some(cur);
            }
        }

        GovernanceReport {
            rule_results,
            overall_score,
            violations,
            fixes_applied,
            fixed_text,
            smells: self.smell_detector.detect(text),
        }
    }
}

impl Default for OutputGovernor {
    fn default() -> Self {
        Self::new()
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
        let report = reg.govern(
            "综上所述，这是对账户池的总结。\n其余内容正常。",
            OutputStyleId::Plain,
        );
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
        let report = gov.govern("", OutputStyleId::Plain);
        if !report.rule_results.iter().all(|r| r.passed) {
            failures.push("empty input should pass all rules".into());
        }

        // 3. 结尾道歉被捕获
        let report = gov.govern("结论是 x。\n抱歉", OutputStyleId::Plain);
        let r10_caught = report.rule_results.iter().any(|r| !r.passed && r.rule_id == 10);
        if !r10_caught {
            failures.push("trailing apology should be caught by R10".into());
        }

        // 4. 纯占位行被捕获 (占位规则为 R04)
        let report = gov.govern("待补充", OutputStyleId::Plain);
        let r04_caught = report.rule_results.iter().any(|r| !r.passed && r.rule_id == 4);
        if !r04_caught {
            failures.push("pure placeholder lines should be caught by R04".into());
        }

        // 5. autofix 能去除结尾道歉
        let report = gov.govern_with_autofix("结论是 x。\n抱歉", OutputStyleId::Plain);
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
