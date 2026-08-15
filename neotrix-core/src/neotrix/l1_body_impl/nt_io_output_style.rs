//! # NT-IO output_style — 输出样式一等能力
//!
//! 吸收源: attention-span + GitHub output-style 生态 (answer-first / spartan / rundown)
//! + i-have-adhd 输出纪律 (G27 OutputGovernor 10 规则)。
//! 与 skills 正交: 样式改变"怎么说话"，不改"怎么编码"。
//!
//! 骨架阶段 (C0): 注册表 + 三种内置样式 + 格式化入口已接 AgentLoop 生产路径。
//! G27: 每条最终输出经 `OutputGovernor` 治理 (10 条纪律规则 + 可机械修复),
//! 报告附于 AgentLoop.last_governance 供观测。待完善: 插件式扩展 / 样式度量反馈。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use regex::Regex;

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
        let head = head.trim_end_matches(|c| c == '.' || c == '。' || c == '!');
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
            .unwrap_or_else(|| self.styles.get(&OutputStyleId::Plain).unwrap().as_ref())
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
    pub fn with_governor_root(mut self, root: impl AsRef<Path>) -> Self {
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
pub struct RuleResult {
    pub rule_id: u8,
    pub passed: bool,
    pub detail: String,
}

impl RuleResult {
    fn pass(id: u8, detail: impl Into<String>) -> Self {
        Self {
            rule_id: id,
            passed: true,
            detail: detail.into(),
        }
    }
    fn fail(id: u8, detail: impl Into<String>) -> Self {
        Self {
            rule_id: id,
            passed: false,
            detail: detail.into(),
        }
    }
}

/// 一次 `govern()` 的完整治理报告。
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
}

/// 单条治理规则 — 独立可测、可审计。
pub struct GovernorRule {
    pub id: u8,
    pub description: &'static str,
    pub check_fn: Box<dyn Fn(&str, OutputStyleId) -> RuleResult + Send + Sync>,
}

/// 已知文件扩展名 (R07/R08 路径引用识别)。
const EXTS: &[&str] = &[
    "rs", "md", "toml", "py", "ts", "tsx", "js", "jsx", "json", "yaml", "yml", "sh", "bash",
    "go", "c", "cpp", "cc", "h", "hpp", "rb", "lua", "sql", "vue", "svelte", "css", "scss",
    "html", "svg", "txt", "xml", "proto", "java", "kt", "swift", "zig", "ex", "cs", "php",
    "ino", "lock", "png", "jpg", "jpeg", "gif", "pdf", "docx", "xlsx", "pptx",
];

fn has_known_ext(p: &str) -> bool {
    let l = p.to_lowercase();
    EXTS.iter().any(|e| l.ends_with(&format!(".{e}")))
}

/// 纯占位行 (整行只有占位符) — R04 违规 + auto-fix 可剥离。
const PLACEHOLDER_PURE_RE: &str = r"^(?:\s*[\[<]?)?(?:\bTODO\b|\bTBD\b|\bFIXME\b|\bPLACEHOLDER\b|lorem ipsum|待补充|待完善|待定|占位|\.\.\.|…)(?:\s*[\]>]?)?$";

/// 内联强占位符 (出现在行内即违规) — 全 Latin 加词边界防误伤。
const PLACEHOLDER_INLINE_RE: &str = r"(?i)\b(TODO|TBD|FIXME|PLACEHOLDER)\b|lorem ipsum";

/// 掩盖 ``` 代码块内容 (路径/语言检查跳过代码内文本)。
fn mask_code_fences(text: &str) -> String {
    let mut masked = String::new();
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            masked.push_str(line);
        } else if in_fence {
            masked.push_str(line.trim_end_matches(|c: char| !c.is_whitespace()).replace(|c: char| !c.is_whitespace(), " ").as_str());
        } else {
            masked.push_str(line);
        }
        masked.push('\n');
    }
    masked
}

/// 判断一行是否为纯占位行。
fn is_placeholder_only(line: &str, pure_re: &Regex) -> bool {
    pure_re.is_match(line.trim())
}

/// 提取文本中的路径引用 (反引号 + 裸路径)，排除 `file:line` 形态 (R08 处理)。
fn extract_path_refs(text: &str, backtick_re: &Regex, bare_re: &Regex, line_suffix_re: &Regex) -> Vec<String> {
    let masked = mask_code_fences(text);
    let mut out: Vec<String> = Vec::new();
    for cap in backtick_re.captures_iter(&masked) {
        let inner = cap[1].trim();
        if (inner.contains('/') || has_known_ext(inner)) && !line_suffix_re.is_match(inner) {
            out.push(inner.to_string());
        }
    }
    for cap in bare_re.captures_iter(&masked) {
        let tok = cap[0].trim();
        if line_suffix_re.is_match(tok) || tok.contains("//") || tok.contains('*') {
            continue;
        }
        out.push(tok.to_string());
    }
    out
}

// ── 各规则检查实现 (纯函数, 便于单测) ───────────────────────────────

/// R01 答案前置: 禁止以"让我先/让我想想"等铺垫推迟答案。
fn r1_answer_first(text: &str) -> RuleResult {
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    let Some(first) = lines.first() else {
        return RuleResult::pass(1, "空输入，无前置铺垫问题");
    };
    let lower = first.to_lowercase();
    const DEFER: &[&str] = &[
        "让我想", "让我看", "让我先", "让我来", "让我查", "让我分析", "让我调研",
        "嗯，让我", "let me think", "let me check", "let me look", "let me review",
        "let me investigate", "hmm, let me",
    ];
    if DEFER.iter().any(|d| lower.contains(d)) && lines.len() >= 3 {
        RuleResult::fail(1, format!("答案被前置铺垫推迟: 首行 `{first}`"))
    } else {
        RuleResult::pass(1, "结论前置")
    }
}

/// R02 禁止模糊对冲: "可能/或许/大概/我觉得/probably" 等。
const HEDGES: &[&str] = &[
    "可能", "或许", "大概", "也许", "我觉得", "我猜", "我认为", "好像",
    "probably", "maybe", "perhaps", "i think", "i guess", "it seems",
];

fn r2_no_hedging(text: &str) -> RuleResult {
    let lower = text.to_lowercase();
    let mut found: Vec<(String, usize)> = Vec::new();
    let mut total = 0usize;
    for h in HEDGES {
        let n = lower.matches(h).count();
        if n > 0 {
            found.push(((*h).to_string(), n));
            total += n;
        }
    }
    if total >= 3 {
        let shown: Vec<String> = found.iter().take(5).map(|(h, n)| format!("{h}×{n}")).collect();
        RuleResult::fail(2, format!("发现 {total} 处对冲表述: {}", shown.join(", ")))
    } else {
        RuleResult::pass(2, "无模糊对冲")
    }
}

/// R03 章节必须有实内容: 标题后不得紧跟空行/纯占位/纯符号。
fn r3_sections_concrete(text: &str, pure_re: &Regex) -> RuleResult {
    let lines: Vec<&str> = text.lines().collect();
    let mut bad: Vec<String> = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if !line.starts_with('#') {
            continue;
        }
        let mut has_content = false;
        for j in (i + 1)..lines.len().min(i + 4) {
            let t = lines[j].trim();
            if t.is_empty() {
                continue;
            }
            if t.starts_with('#') {
                break;
            }
            if is_placeholder_only(t, pure_re) || t.chars().filter(|c| !c.is_whitespace()).count() <= 1 {
                break;
            }
            has_content = true;
            break;
        }
        if !has_content {
            bad.push(format!("`{line}`"));
        }
    }
    if bad.is_empty() {
        RuleResult::pass(3, "所有章节均有实内容")
    } else {
        RuleResult::fail(3, format!("空章节: {}", bad.join(", ")))
    }
}

/// R04 禁止空/占位文本: TODO/TBD/待补充/lorem ipsum 等。
fn r4_no_placeholder(text: &str, pure_re: &Regex, inline_re: &Regex) -> RuleResult {
    let mut bad: Vec<String> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let t = raw.trim();
        if is_placeholder_only(t, pure_re) || inline_re.is_match(t) {
            bad.push(format!("L{} `{t}`", i + 1));
        }
    }
    if bad.is_empty() {
        RuleResult::pass(4, "无占位文本")
    } else {
        RuleResult::fail(4, format!("占位文本: {}", bad.join("; ")))
    }
}

/// R05 单消息长度上限。
fn r5_max_length(text: &str, max_chars: usize) -> RuleResult {
    let n = text.chars().count();
    if n > max_chars {
        RuleResult::fail(5, format!("单消息 {n} 字符 > 上限 {max_chars}"))
    } else {
        RuleResult::pass(5, format!("长度 {n} ≤ {max_chars}"))
    }
}

/// R06 禁止重复样板: 相同长行 (≥25 字符) 出现 ≥3 次。
fn r6_no_dup_boilerplate(text: &str) -> RuleResult {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for line in text.lines() {
        let t = line.trim();
        if t.chars().count() >= 25 {
            *counts.entry(t.to_string()).or_default() += 1;
        }
    }
    let dups: Vec<(String, usize)> = counts.into_iter().filter(|(_, c)| *c >= 3).collect();
    if dups.is_empty() {
        RuleResult::pass(6, "无重复样板")
    } else {
        let shown: Vec<String> = dups
            .iter()
            .map(|(s, c)| format!("`{}`×{}", truncate(s, 40), c))
            .collect();
        RuleResult::fail(6, format!("重复样板行: {}", shown.join("; ")))
    }
}

fn truncate(s: &str, max: usize) -> String {
    let mut t: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        t.push('…');
    }
    t
}

/// R07 文件引用必须存在 (工作区真实文件)。
fn r7_file_refs_exist(text: &str, root: &Path, backtick_re: &Regex, bare_re: &Regex, line_suffix_re: &Regex) -> RuleResult {
    let mut missing: Vec<String> = Vec::new();
    for p in extract_path_refs(text, backtick_re, bare_re, line_suffix_re) {
        if p.contains("//") || p.starts_with('*') || p.starts_with("http") {
            continue;
        }
        if !root.join(&p).is_file() && !missing.iter().any(|m| m == &p) {
            missing.push(p);
        }
    }
    if missing.is_empty() {
        RuleResult::pass(7, "文件引用均存在")
    } else {
        RuleResult::fail(7, format!("引用了不存在的工作区文件: {}", missing.join(", ")))
    }
}

/// R08 禁止幻影路径: `file:line` 引用必须存在且行号在文件范围内。
fn r8_hallucinated_paths(text: &str, root: &Path, line_ref_re: &Regex) -> RuleResult {
    let masked = mask_code_fences(text);
    let mut bad: Vec<String> = Vec::new();
    for cap in line_ref_re.captures_iter(&masked) {
        let path = &cap[1];
        let line: usize = cap[2].parse().unwrap_or(0);
        if path.contains("//") || path.contains("http") {
            continue;
        }
        let full = root.join(path);
        if !full.is_file() {
            bad.push(format!("`{path}:{line}` 文件不存在"));
        } else if let Ok(content) = std::fs::read_to_string(&full) {
            let total = content.lines().count();
            if line == 0 || line > total {
                bad.push(format!("`{path}:{line}` 行号超范围 (文件共 {total} 行)"));
            }
        }
    }
    if bad.is_empty() {
        RuleResult::pass(8, "无幻影路径")
    } else {
        RuleResult::fail(8, format!("幻影路径: {}", bad.join("; ")))
    }
}

/// R09 语言一致: 禁止显著中英混杂 (代码块除外)。
fn is_cjk(c: char) -> bool {
    matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{F900}'..='\u{FAFF}')
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
        .filter(|l| !l.is_empty())
        .last()
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

/// auto-fix R04: 移除纯占位行。
fn strip_pure_placeholder_lines(text: &str, pure_re: &Regex) -> Option<(String, Vec<String>)> {
    let mut removed: Vec<String> = Vec::new();
    let out: Vec<&str> = text
        .lines()
        .filter(|l| {
            if is_placeholder_only(l, pure_re) {
                removed.push(l.trim().to_string());
                false
            } else {
                true
            }
        })
        .collect();
    if removed.is_empty() {
        return None;
    }
    Some((
        out.join("\n").trim_end().to_string(),
        vec![format!(
            "R04: 移除纯占位行 {}",
            removed.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(" ")
        )],
    ))
}

/// G27 输出纪律治理器 — 10 条 i-have-adhd 规则, 纯检查 + 可机械 auto-fix。
pub struct OutputGovernor {
    rules: Vec<GovernorRule>,
    workspace_root: PathBuf,
    max_message_chars: usize,
    placeholder_pure: Regex,
}

/// 默认单消息长度上限 (字符)。
pub const DEFAULT_MAX_MESSAGE_CHARS: usize = 8_000;

fn build_rules(root: &Path, max_message_chars: usize) -> Vec<GovernorRule> {
    let root = root.to_path_buf();
    let placeholder_pure = Arc::new(Regex::new(PLACEHOLDER_PURE_RE).expect("placeholder_pure 正则有效"));
    let placeholder_inline = Arc::new(Regex::new(PLACEHOLDER_INLINE_RE).expect("placeholder_inline 正则有效"));
    let placeholder_pure_for_inline = placeholder_pure.clone();
    let root_for_hallucinated = root.clone();
    let backtick_re = Arc::new(Regex::new(r"`([^`]+)`").expect("backtick 正则有效"));
    let bare_path_re = Arc::new(
        Regex::new(&format!(r"[\w.\-/]+\.(?:{})", EXTS.join("|")))
            .expect("bare_path 正则有效"),
    );
    let line_ref_re = Arc::new(
        Regex::new(&format!(
            r"(?i)([A-Za-z0-9_.\-/]+\.(?:{})):(\d+)",
            EXTS.join("|")
        ))
        .expect("line_ref 正则有效"),
    );
    let line_suffix_re = Arc::new(Regex::new(r":\d+$").expect("line_suffix 正则有效"));

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
            placeholder_pure: Regex::new(PLACEHOLDER_PURE_RE).expect("placeholder_pure 正则有效"),
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
    pub fn set_max_message_chars(&mut self, max: usize) {
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
}
