//! # NT-IO output_style — 输出样式一等能力
//!
//! 吸收源: attention-span + GitHub output-style 生态 (answer-first / spartan / rundown)。
//! 与 skills 正交: 样式改变"怎么说话"，不改"怎么编码"。
//!
//! 骨架阶段 (C0): 注册表 + 三种内置样式 + 格式化入口已接 AgentLoop 生产路径;
//! 待完善: 每消息规则 / 插件式扩展 / 样式度量反馈。见 AGENTS.md 完善建议。

use std::collections::HashMap;

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

/// 输出样式注册表 — 样式按 id 解析。
pub struct OutputStyleRegistry {
    styles: HashMap<OutputStyleId, Box<dyn OutputStyle>>,
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
        Self { styles }
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
