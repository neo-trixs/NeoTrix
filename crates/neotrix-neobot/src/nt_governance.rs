//! nt_governance — 输出纪律治理（G27 OutputGovernor）
//!
//! 2026-10-06 从 `neotrix-core/src/l1_action/nt_io/nt_io_output_style.rs`
//! 搬移而来。搬移理由（**不是重构洁癖，是接线**）：
//!
//! 治理器原本住在 `AgentLoop` 里，而 `AgentLoop` 经实测**零生产实例化**
//! （编译活、20 条测试全绿、接线为零）。于是：
//!   - 真实执行环 `nt_agent.rs::run_loop`（4 个生产入口）**零治理**：模型
//!     说什么就存什么、就发什么。
//!   - 而治理器本身**质量高于**它周边的骨架代码：10 条规则里有 2 条（R07/R08）
//!     真读文件系统校验文件引用与幻影路径，需要刻意的威胁建模，不是随手写的
//!     骨架；且已被 `l6_meta` 的 SelfTest 元门注册。
//!
//! ⇒ 这是一次**架构迁移未完成**：真实循环搬到了 neobot crate，治理没跟着搬。
//!    本文件把治理器搬到循环旁边，让它重新变得可达。
//!
//! ## 为什么搬到这里而不是让 neobot 依赖 neotrix-core
//!
//! `neotrix-core/Cargo.toml:106` 依赖 `neotrix-neobot`（core → neobot），
//! 反向依赖会造成**循环 crate 依赖**，Cargo 直接拒绝。
//!
//! ## 为什么不搬 `OutputStyleId` 那三个样式
//!
//! `AnswerFirstStyle` / `SpartanStyle` 是薄实现，`RundownStyle::apply` 的
//! 全部内容是 `out.push_str("\n\n---\n[rundown] 摘要待实现")` —— 接线等于
//! 告诉用户「摘要待实现」。本文件只搬**治理**，样式留在 core 待重做。
//!
//! ## 依赖
//!
//! 仅 `std` + `regex`，无任何 neotrix-core 内部设施 ⇒ 纯搬移。

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use regex::Regex;

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

/// 单条规则检查的完整治理报告。
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
    pub regex: Regex,
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
                regex: Regex::new(
                    r"(?i)值得注意的是|需要注意的是|it's worth noting|it is worth noting|please note that|as an ai,|i am an ai",
                )
                .expect("meta-speech 正则有效"),
                suggestion: "直接给结论/事实, 不要声明性前言",
            },
            SmellPattern {
                id: "conclusion-signpost",
                regex: Regex::new(r"综上所述|总而言之|总的说来|in conclusion|to summarize|to sum up|overall, i think|in summary").expect("conclusion-signpost 正则有效"),
                suggestion: "删掉总结开场白, 直接给要点或删除冗余段",
            },
            SmellPattern {
                id: "transition-cliche",
                regex: Regex::new(r"首先，|其次，|最后，|最后,|firstly,|secondly,|furthermore,|moreover,|additionally,").expect("transition-cliche 正则有效"),
                suggestion: "用清单/编号结构替代口语化过渡词",
            },
            SmellPattern {
                id: "over-polished",
                regex: Regex::new(r"如下所示|如下：|以下是对|以下为|below is|here is the|as you can see|如您所见|正如您所知|as we all know|as you know").expect("over-polished 正则有效"),
                suggestion: "去掉恭维性引导, 直入主题",
            },
            SmellPattern {
                id: "hedge-stack",
                regex: Regex::new(r"(?i)very very|extremely extremely|absolutely|undoubtedly|无疑|诚然|毋庸置疑|显然,").expect("hedge-stack 正则有效"),
                suggestion: "删减程度副词, 让论证自己说话",
            },
        ]
    }

    /// 对一段文本运行全部模式, 返回命中的 AI-smell (按行序去重排序)。
    pub fn detect(&self, text: &str) -> Vec<AiSmell> {
        let mut out: Vec<AiSmell> = Vec::new();
        for p in &self.patterns {
            let mut hits: Vec<AiSmell> = Vec::new();
            for cap in p.regex.captures_iter(text) {
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
    /// 规则检查函数。
    ///
    /// ⛔ 注意：原签名带一个 `style: OutputStyleId` 死参数，
    ///   10 条规则**无一**读它，`finalize` 也不读 —— 搬移时已去掉。
    ///   这正是治理器能独立于样式模块的前提（样式那三个 Style 是骨架）。
    pub check_fn: Box<dyn Fn(&str) -> RuleResult + Send + Sync>,
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
    /// AI-smell 检测器 (natural-japanese #14 吸收)。
    smell_detector: AiSmellDetector,
}

/// 默认单消息长度上限 (字符)。
pub const DEFAULT_MAX_MESSAGE_CHARS: usize = 8_000;

/// 构造正则，**失败时不 panic**（2026-10-07 去 unwrap/expect）。
///
/// # 为什么原来可以 expect，现在不行
///
/// `bare_path_re` / `line_ref_re` 是**运行时拼装**的
/// （`format!(... EXTS.join("|"))`），⛔ 不是编译期常量
/// ⇒ 任何人往 `EXTS` 里加一个 `.` / `+` / `(` 都会让
/// `OutputGovernor::new()` 在**启动期 panic**。
///
/// # 失败时怎么办
///
/// 记 `error!`（⛔ 不静默）并返回**永不匹配**的正则
/// ⇒ 相关治理规则不触发（能力降级但**可见**），而不是进程崩掉。
///
/// # 为什么不会被静默放过
///
/// `治理正则均可编译` 测试断言全部模式可编译
/// ⇒ 有人改坏 `EXTS` 时**CI 立刻红**，而不是等到启动才panic。
fn re_or_never_match(pat: &str) -> Regex {
    match Regex::new(pat) {
        Ok(r) => r,
        Err(e) => {
            log::error!("[governor] 正则无法编译，规则将不触发: {pat:?} ({e})");
            // ⚠️ `(?!)` **不行**：`regex` crate 不支持前瞻（实测 panic）⇒
            // 改用 `a^`（正则经典永不匹配式：无元字符，crate 必然接受）。
            // ⇒ 我第一版写的 `unreachable!()` **自身就是 panic 路径**，
            //   被本文件的 `坏模式降级为不匹配而不panic` 测试当场抓住。
            Regex::new("a^").expect("a^ 无元字符，regex crate 必然接受")
        }
    }
}

fn build_rules(root: &Path, max_message_chars: usize) -> Vec<GovernorRule> {
    let root = root.to_path_buf();
    let placeholder_pure = Arc::new(re_or_never_match(PLACEHOLDER_PURE_RE));
    let placeholder_inline = Arc::new(re_or_never_match(PLACEHOLDER_INLINE_RE));
    let placeholder_pure_for_inline = placeholder_pure.clone();
    let root_for_hallucinated = root.clone();
    let backtick_re = Arc::new(re_or_never_match(r"`([^`]+)`"));
    let bare_path_re = Arc::new(re_or_never_match(&format!(
        r"[\w.\-/]+\.(?:{})",
        EXTS.join("|")
    )));
    let line_ref_re = Arc::new(re_or_never_match(&format!(
        r"(?i)([A-Za-z0-9_.\-/]+\.(?:{})):(\d+)",
        EXTS.join("|")
    )));
    let line_suffix_re = Arc::new(re_or_never_match(r":\d+$"));

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

impl OutputGovernor {
    pub fn new() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            rules: build_rules(&root, DEFAULT_MAX_MESSAGE_CHARS),
            workspace_root: root,
            max_message_chars: DEFAULT_MAX_MESSAGE_CHARS,
            placeholder_pure: Regex::new(PLACEHOLDER_PURE_RE).expect("placeholder_pure 正则有效"),
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
    pub fn govern(&self, text: &str) -> GovernanceReport {
        let rule_results: Vec<RuleResult> =
            self.rules.iter().map(|rule| (rule.check_fn)(text)).collect();
        self.finalize(text, rule_results, false)
    }

    /// auto-fix 模式: 检查 + 剥离可机械修复项 (结尾道歉 / 纯占位行)。
    pub fn govern_with_autofix(&self, text: &str) -> GovernanceReport {
        let rule_results: Vec<RuleResult> =
            self.rules.iter().map(|rule| (rule.check_fn)(text)).collect();
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
mod governance_tests {
    use super::*;

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

    /// `govern()` 会把 AI-smell 一并带进报告（搬移后补的接线断言）。
    ///
    /// 原测试 `governed_report_includes_smells` 走的是 `OutputStyleRegistry::govern`，
    /// 那条路径依赖未搬移的样式模块 ⇒ 此处改为直接构造 `OutputGovernor`。
    /// 这条断言的作用：**防止搬移后 smells 只是编过而没真被 govern 调用**。
    #[test]
    fn govern_report_includes_smells() {
        let gov = OutputGovernor::new();
        let report = gov.govern("综上所述，这是对账户池的总结。\n其余内容正常。");
        assert!(!report.smells.is_empty(), "smells 应被 govern() 检出");
        assert!(report
            .smells
            .iter()
            .any(|s| s.pattern_id == "conclusion-signpost"));
    }
}

#[cfg(test)]
mod regex_validity_tests {
    use super::{EXTS, PLACEHOLDER_INLINE_RE, PLACEHOLDER_PURE_RE};
    use regex::Regex;

    /// ⭐ **把「启动期 panic」变成「CI 期失败」**（2026-10-07）。
    ///
    /// `bare_path_re` / `line_ref_re` 是**运行时拼装**的，⛔ 不是编译期常量
    /// ⇒ 任何人往 `EXTS` 里加一个正则元字符（`.` `+` `(` …），
    ///   生产代码只会「记 error + 规则不触发」（见 `re_or_never_match`），
    ///   **不会崩** —— 但那样治理能力就静默降级了。
    ///
    /// ⇒ 本测试断言**全部模式可编译** ⇒ 改坏 `EXTS` 时 CI 立刻红。
    #[test]
    fn 治理正则均可编译() {
        for pat in [
            PLACEHOLDER_PURE_RE,
            PLACEHOLDER_INLINE_RE,
            r"`([^`]+)`",
            r":\d+$",
            &format!(r"[\w.\-/]+\.(?:{})", EXTS.join("|")),
            &format!(
                r"(?i)([A-Za-z0-9_.\-/]+\.(?:{})):(\d+)",
                EXTS.join("|")
            ),
        ] {
            assert!(
                Regex::new(pat).is_ok(),
                "治理正则无法编译（会静默降级为不触发）: {pat:?}"
            );
        }
    }

    /// 反向证据：`re_or_never_match` 对**坏模式**不 panic，且返回永不匹配的正则。
    ///
    /// ⛔ 不断言它「有效」—— 断言「不 panic + 不匹配」才是契约。
    #[test]
    fn 坏模式降级为不匹配而不panic() {
        let r = super::re_or_never_match("([unclosed");
        assert!(!r.is_match("任何东西"), "降级后的正则必须永不匹配");
    }
}
