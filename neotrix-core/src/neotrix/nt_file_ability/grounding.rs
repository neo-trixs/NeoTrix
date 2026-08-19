//! doc7 吸收: Grounding 精确值校验。
//! 来源: github.com/magicrew/doc7 internal/extract/grounding.go + grounding_numeric.go
//! (MIT, absorbed 2026-08-13, cycle 1101)。
//!
//! 公理 (Grounding 是精确值保险): 视觉理解管线可能幻读数字/代码/ID。从嵌入文本层
//! 提取关键 token (≥3 位数字或含小数/百分号/货币符号, 以及大写字母+数字标识符),
//! 与 VLM 输出比对, 缺失则标记为 ungrounded — 由调用方决定二次校正 (遵循 R-P36:
//! grounding 结果必须进入行为, 而非仅日志)。本模块是纯算法, 无 LLM 依赖。

use serde::{Deserialize, Serialize};

/// 关键标识符模式: 大写字母+数字+_/- (如 "Attention-12", "B2")
const CRITICAL_IDENTIFIER_RE: &str = r"[A-Z]{2,}[A-Z0-9_-]*\d[A-Z0-9_-]*";

/// 关键数字 token 模式: 可选正负/百分号前缀, 数字带千分位逗号与可选小数
const NUMERIC_TOKEN_RE: &str = r"(?:\([0-9][0-9,]*(?:\.[0-9]+)?%?\)|[+\-−－△]?[0-9][0-9,]*(?:\.[0-9]+)?%?)";

/// 多段版本号模式 (如 2.5.1, 3.14.159) — doc7 单小数段正则的增强
const VERSION_TOKEN_RE: &str = r"[0-9]+\.[0-9]+(?:\.[0-9]+)+";

/// 数字 token 判定: ≥3 位数字, 或含小数/百分号/货币符号
pub(super) fn is_critical_numeric_token(value: &str) -> bool {
    let trimmed = value.trim().trim_matches(|c| c == '(' || c == ')');
    let digits = trimmed.chars().filter(|c| c.is_ascii_digit()).count();
    digits >= 3 || trimmed.chars().any(|c| matches!(c, '.' | '%' | '$' | '€' | '£' | '¥'))
}

/// 数值 token 归一化: 空格/unicode 减号/括号归一, 便于跨来源比对
pub(super) fn normalize_numeric_token(value: &str) -> String {
    let mut v = value.trim().trim_end_matches(',').to_string();
    v = v.replace(['−', '－'], "-").replace('△', "-").replace(' ', "");
    if v.starts_with("△") {
        v = format!("-{}", &v["△".len()..]);
    }
    if (v.starts_with('(') && v.ends_with(')')) || (v.starts_with('（') && v.ends_with('）')) {
        v = v[1..v.len() - 1].to_string();
    }
    v
}

/// 紧凑化文本: 去除空白/Markdown 标记/unicode 减号, 用于存在性比对
pub(super) fn compact_numeric_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => continue,
            '−' | '－' | '△' => out.push('-'),
            '\\' | '$' | '{' | '}' | '^' | '_' | '*' | '`' => {} // Markdown/LaTeX 标记
            _ => out.push(ch),
        }
    }
    out
}

/// 判断文本是否为数学行 (含 LaTeX 标记 → grounding 跳过, 防止错改公式)
fn is_math_line(value: &str) -> bool {
    value.contains('$') || value.contains('^') || value.contains('{') || value.contains("\\")
}

/// 关键数字 token 提取 (带 1-based 位置, 与 doc7 一致用于行定位)
fn critical_numeric_tokens(value: &str) -> Vec<(usize, String)> {
    let version_re = regex::Regex::new(VERSION_TOKEN_RE).expect("VERSION_TOKEN_RE 有效");
    let re = regex::Regex::new(NUMERIC_TOKEN_RE).expect("NUMERIC_TOKEN_RE 有效");
    let mut tokens = Vec::new();
    let mut covered: Vec<(usize, usize)> = Vec::new();
    for (pos, cap) in version_re.captures_iter(value).enumerate() {
        let m = cap.get(0).expect("group 0");
        covered.push((m.start(), m.end()));
        let raw = &value[m.start()..m.end()];
        if is_critical_numeric_token(raw) {
            tokens.push((pos + 1, raw.to_string()));
        }
    }
    for (pos, cap) in re.captures_iter(value).enumerate() {
        let m = cap.get(0).expect("group 0");
        if covered.iter().any(|(s, e)| *s <= m.start() && m.end() <= *e) {
            continue;
        }
        let raw = &value[m.start()..m.end()];
        let trimmed = raw.trim_start_matches(|c: char| {
            c.is_whitespace() || matches!(c, '+' | '-' | '−' | '－' | '△' | '(')
        });
        let trimmed = trimmed.trim_end_matches(|c: char| c.is_whitespace() || c == ')');
        if is_critical_numeric_token(trimmed) {
            tokens.push((pos + 1, trimmed.to_string()));
        }
    }
    tokens
}

/// 关键标识符提取 (大写字母开头且含数字)
fn critical_identifiers(value: &str) -> Vec<String> {
    regex::Regex::new(CRITICAL_IDENTIFIER_RE)
        .expect("CRITICAL_IDENTIFIER_RE 有效")
        .find_iter(value)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// 数值序列是否一致 (source vs content 归一化后按 token 计数比对)
fn numeric_sequences_equal(source: &str, content: &str) -> bool {
    use std::collections::HashMap;
    let mut left: HashMap<String, usize> = HashMap::new();
    for (_, t) in critical_numeric_tokens(source) {
        *left.entry(normalize_numeric_token(&t)).or_insert(0) += 1;
    }
    let mut right: HashMap<String, usize> = HashMap::new();
    for (_, t) in critical_numeric_tokens(content) {
        *right.entry(normalize_numeric_token(&t)).or_insert(0) += 1;
    }
    left == right
}

/// grounding 检查结果
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GroundingReport {
    pub checked: bool,
    pub missing_numeric: Vec<String>,
    pub missing_identifiers: Vec<String>,
    pub math_guard_skipped: usize,
    pub ungrounded: Vec<String>,
    pub sequence_ok: bool,
    /// 可靠性评分 (0.0 ~ 1.0) — 公理二 (grounding 是精确值保险) 的量化表达。
    /// 1.0 = 源文本所有关键 token 均保真输出; 越低表示 VLM 输出与源越偏离。
    /// 计算: 未接地 token 占关键 token 比例的反向 (R-P79 生产接地)。
    pub reliability_score: f64,
}

/// 执行 grounding 检查: 源文本层 token vs VLM 输出内容。
/// 返回缺失 token 清单; 调用方决定二次校正或标记失败 (R-P79 接线)。
pub fn ground_missing_tokens(source_text: &str, content: &str) -> GroundingReport {
    let mut report = GroundingReport {
        checked: !source_text.trim().is_empty(),
        ..Default::default()
    };
    if !report.checked {
        // 空源无关键 token 可校验 → 视为保真 (公理二: 无风险则保险成立)
        report.reliability_score = 1.0;
        return report;
    }

    // 关键数字: 源存在而输出缺失
    let compact_content = compact_numeric_text(content);
    let content_tokens: std::collections::HashSet<String> = critical_numeric_tokens(content)
        .iter()
        .map(|(_, t)| normalize_numeric_token(t))
        .collect();
    let mut seen = std::collections::HashSet::new();
    for (_, t) in critical_numeric_tokens(source_text) {
        let norm = normalize_numeric_token(&t);
        if !seen.insert(norm.clone()) {
            continue;
        }
        let missing = !content_tokens.contains(&norm) && !compact_content.contains(&compact_numeric_text(&t));
        if missing {
            // math 行保护: 输出缺失数字所在行若含 LaTeX → 判定为公式, 跳过
            let line = content.lines().find(|l| compact_numeric_text(l).contains(&compact_numeric_text(&t)));
            if let Some(line) = line {
                if is_math_line(line) {
                    report.math_guard_skipped += 1;
                    continue;
                }
            }
            report.missing_numeric.push(t.clone());
            report.ungrounded.push(t);
        }
    }

    // 关键标识符
    seen.clear();
    for id in critical_identifiers(source_text) {
        if !seen.insert(id.clone()) {
            continue;
        }
        if !compact_content.contains(&compact_numeric_text(&id)) {
            report.missing_identifiers.push(id.clone());
            report.ungrounded.push(id);
        }
    }

    report.sequence_ok = numeric_sequences_equal(source_text, content);
    // 公理二量化: 可靠性评分 = 1 - (未接地 token / 源关键 token 总数)
    // math_guard_skipped 视为合理豁免 (公式行), 不计入未接地。
    let total_critical = critical_numeric_tokens(source_text)
        .iter()
        .map(|(_, t)| normalize_numeric_token(t))
        .collect::<std::collections::HashSet<_>>()
        .len()
        + critical_identifiers(source_text)
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
    if total_critical > 0 {
        report.reliability_score = 1.0 - (report.ungrounded.len() as f64 / total_critical as f64);
    } else {
        report.reliability_score = 1.0; // 无关键 token → 默认保真
    }
    report.reliability_score = report.reliability_score.clamp(0.0, 1.0);
    report
}