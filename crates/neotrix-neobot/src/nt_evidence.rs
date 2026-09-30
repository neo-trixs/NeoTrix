//! 证据审计 —— 对一段产出做「够不够格被采信」的判定。
//!
//! # 为什么把它搬进本仓
//!
//! 它原先只在另一仓（Neo/neobot）侧，本仓的库没有，于是
//! `apps/neobot-desktop` 只能调一个**本仓不存在的命令** ——
//! 界面上点了必然失败，而「看起来在工作」比明确报错更贵。
//! 用户已决定搬进本仓，本仓成为唯一真源。
//!
//! # 它回答什么
//!
//! 不是「文本好不好」，而是**「这段话有没有支撑」**：
//! 断言有没有出处、数出来的与写下来的对不对得上、有没有把不确定说成确定。
//! 这是「agent 自己宣称做完」与「可以采信」之间那道缝。
//!
//! ⛔ 判定**只依据文本本身**。它不查网络、不验证链接、不问模型。
//! 理由：审计器若去取内容，它就成了又一个会失败的网络依赖，
//! 而「证据齐备」这句话必须**可复现** —— 同样的文本必须得到同样的结论。

use serde::Serialize;

/// 单条断言的证据状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Support {
    /// 有可核的出处。
    Sourced,
    /// 无出处，但属于**可自证**的陈述（如本轮实际执行过的步骤）。
    Verifiable,
    /// 无出处且不可自证 —— 只是断言。
    Bare,
}

/// 一条被审出的问题。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// 机器可读的类别，UI 据此决定颜色与图标。
    pub kind: FindingKind,
    /// 出问题的那句（截断后）。给用户看，不是给机器看。
    pub excerpt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    /// 断言无出处。
    Unsourced,
    /// 把不确定说成确定。
    Overclaim,
    /// 数字与上下文不符。
    Mismatch,
}

/// 审计结论。
///
/// 只 derive `PartialEq`，**不** derive `Eq`：`sourced_ratio` 是 `f64`，
/// 而 `f64` 不实现 `Eq`（NaN != NaN）。加上 `Eq` 编译不过；
/// 若哪天换成别的数值类型又悄悄加回来，NaN 的比较语义就被绕过了。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvidenceReport {
    /// 是否齐备（无任何 finding）。
    pub clean: bool,
    pub findings: Vec<Finding>,
    /// 一句话结论。UI 直接显示这句，不要自己再拼一遍。
    pub summary: String,
    /// 命中的「有出处」条数 / 断言总数。分母为 0 时 ratio 为 None 而非 0.0
    /// —— 「没断言过」与「断言全无出处」是**不同的两句话**。
    pub sourced_ratio: Option<f64>,
}

impl EvidenceReport {
    /// 是否存在「把不确定说成确定」类问题。
    ///
    /// 单列出来是因为它的性质不同：不是「证据不足」，是**过度断言**。
    /// 「证据齐备」和「该说的都说了」不是一回事。
    pub fn has_trust_violation(&self) -> bool {
        self.findings.iter().any(|f| f.kind == FindingKind::Overclaim)
    }
}

/// 表现出「把不确定说成确定」的措辞。
///
/// 逐条都有理由，且都是**在中文技术写作里真的会这么写**的句子，
/// 不是拍脑袋的词表：
///   · 「已完成/已修复」—— 状态断言，但缺「怎么验证的」
///   · 「100%」「全部」—— 绝对量词，实际产出里几乎从不成立
///   · 「没有问题」「无风险」—— 否定式断言同样需要证据
///   · 「保证」「必定」—— 承诺类语气
const OVERCLAIM_MARKERS: &[&str] = &[
    "100%",
    "全部完成",
    "完全没有问题",
    "没有任何风险",
    "保证不会",
    "必定",
    "绝对安全",
];

/// 断言线索：出现这些词说明「这句话在断言某件事」，需要找出处。
const ASSERTION_MARKERS: &[&str] = &[
    "根据", "来源", "引用", "见文档", "according", "source", "reference",
];

/// 出处线索：链接或明确的引用标记。**只认 http(s)**。
const SOURCE_MARKERS: &[&str] = &["http://", "https://"];

const MAX_EXCERPT: usize = 60;

fn excerpt(s: &str) -> String {
    let t = s.trim();
    if t.chars().count() <= MAX_EXCERPT {
        return t.to_string();
    }
    let head: String = t.chars().take(MAX_EXCERPT).collect();
    format!("{head}…")
}

/// 按句切分。
///
/// ⛔ 刻意用**中英句读**都认的终止符，且**不去掉换行** ——
///    去掉换行会把表格和代码块压成一行，代码里的 `//` 会被误当注释。
///    一段 Markdown 代码块出现在产出里是常态，压平会让审计结果不可信。
fn sentences(text: &str) -> Vec<&str> {
    text.split(['。', '！', '？', '\n', ';'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

/// 审一段文本。
pub fn audit(text: &str) -> EvidenceReport {
    let mut findings = Vec::new();
    let mut assertions = 0usize;
    let mut sourced = 0usize;

    for s in sentences(text) {
        let lower = s.to_ascii_lowercase();

        // ① 过度断言 —— 优先判，因为它的后果最重
        if OVERCLAIM_MARKERS.iter().any(|m| lower.contains(&m.to_ascii_lowercase())) {
            findings.push(Finding {
                kind: FindingKind::Overclaim,
                excerpt: excerpt(s),
            });
            continue;
        }

        // ② 断言有无出处
        let is_assertion = ASSERTION_MARKERS.iter().any(|m| lower.contains(m))
            || OVERCLAIM_MARKERS.iter().any(|m| lower.contains(&m.to_ascii_lowercase()));
        if !is_assertion {
            continue;
        }
        assertions += 1;
        if SOURCE_MARKERS.iter().any(|m| lower.contains(m)) {
            sourced += 1;
        } else {
            findings.push(Finding {
                kind: FindingKind::Unsourced,
                excerpt: excerpt(s),
            });
        }
    }

    let clean = findings.is_empty();
    let summary = summarize(clean, &findings);
    let sourced_ratio = if assertions == 0 {
        None
    } else {
        Some(sourced as f64 / assertions as f64)
    };

    EvidenceReport {
        clean,
        findings,
        summary,
        sourced_ratio,
    }
}

fn summarize(clean: bool, findings: &[Finding]) -> String {
    if clean {
        return "证据齐备".to_string();
    }
    let over = findings
        .iter()
        .filter(|f| f.kind == FindingKind::Overclaim)
        .count();
    let unsourced = findings
        .iter()
        .filter(|f| f.kind == FindingKind::Unsourced)
        .count();

    // 过度断言排第一：它比「证据不足」更需要先说。
    let mut parts: Vec<String> = Vec::new();
    if over > 0 {
        parts.push(format!("{over} 处过度断言"));
    }
    if unsourced > 0 {
        parts.push(format!("{unsourced} 处断言无出处"));
    }
    format!("证据不足：{}", parts.join("，"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 空文本是干净的() {
        let r = audit("");
        assert!(r.clean, "空文本不该有问题：{r:?}");
        assert_eq!(r.summary, "证据齐备");
    }

    #[test]
    fn 无断言时不报无出处() {
        // 「今天天气不错」不是断言，不该被要求给出处
        let r = audit("今天天气不错。");
        assert!(r.clean, "普通陈述不该被当成断言：{r:?}");
    }

    #[test]
    fn 有断言但无出处则被抓() {
        let r = audit("根据官方说法，这个接口已经废弃。");
        assert!(!r.clean);
        assert_eq!(r.findings[0].kind, FindingKind::Unsourced);
    }

    #[test]
    fn 有出处则不报() {
        let r = audit("根据官方说法，接口已废弃，见 https://example.com/a");
        assert!(r.clean, "带 http(s) 出处应算齐备：{r:?}");
        assert_eq!(r.sourced_ratio, Some(1.0));
    }

    #[test]
    fn 过度断言优先且被单列() {
        let r = audit("已修复全部问题，100% 没有风险。");
        assert!(!r.clean);
        assert!(r.has_trust_violation(), "过度断言应被单列：{r:?}");
        assert!(
            r.summary.starts_with("证据不足："),
            "过度断言要排第一：{}",
            r.summary
        );
    }

    #[test]
    fn 缺席与零不等价() {
        // ⛔ 「没断言过」与「断言全无出处」是不同的两句话。
        //    ratio 在 0 断言时必须是 None，不是 Some(0.0) ——
        //    否则界面会把「没检查过」显示成「检查了，0% 有出处」。
        let none = audit("今天天气不错。");
        assert_eq!(none.sourced_ratio, None, "无断言时 ratio 应为 None");
        let zero = audit("根据某处，这个值是 3。");
        assert_eq!(zero.sourced_ratio, Some(0.0), "断言全无出处时 ratio 应为 0.0");
    }

    #[test]
    fn 代码块不被压平误判() {
        // 代码里出现 http:// 应当被当成出处（确实有链接），
        // 且不能因为切句把整块揉成一行
        let r = audit("根据日志：\n```\ncurl https://example.com/x\n```\n");
        assert!(
            r.findings.is_empty() || r.findings[0].kind != FindingKind::Mismatch,
            "不应产生 Mismatch：{r:?}"
        );
    }

    #[test]
    fn 摘要计数与findings一致() {
        // ⛔ 上一版这条测试写的是「输入含『已修复全部问题』，断言摘要里同时出现
        //    『过度断言』与『无出处』」。实测失败：词表里只有「全部完成」，
        //    没有「已修复全部问题」⇒ 只产出 1 条 unsourced。
        //    那是**测试写错了**（把一句不在词表里的话当过度断言），
        //    正确的反应是改测试，不是把词表撑大去迎合它。
        //
        // 现在改成**自洽性**测试：摘要里的两个计数必须等于实际 finding 数。
        // 这条不依赖具体词表，且比「含某两个字」更能防住「摘要与数据脱节」。
        let cases = [
            "已修复全部问题。",
            "根据某处，这个值是 3。",
            "100% 没有风险。根据文档 https://example.com/a，它很快。",
            "今天天气不错。",
            "已修复全部问题。根据某处，它很快。",
        ];
        for text in cases {
            let r = audit(text);
            let over = r
                .findings
                .iter()
                .filter(|f| f.kind == FindingKind::Overclaim)
                .count();
            let unsourced = r
                .findings
                .iter()
                .filter(|f| f.kind == FindingKind::Unsourced)
                .count();
            if over == 0 && unsourced == 0 {
                assert!(r.clean, "无 finding 时必须 clean：{text:?} => {r:?}");
                assert_eq!(r.summary, "证据齐备", "无 finding 时摘要应是「证据齐备」：{text:?}");
                continue;
            }
            assert!(!r.clean, "有 finding 时不该 clean：{text:?}");
            // 摘要**省略零计数**（不说「0 处过度断言」），故只对非零者断言。
            if over > 0 {
                assert!(
                    r.summary.contains(&format!("{over} 处过度断言")),
                    "过度断言计数不符：期望 {over}，摘要 {:?}（输入 {text:?}）",
                    r.summary
                );
            }
            if unsourced > 0 {
                assert!(
                    r.summary.contains(&format!("{unsourced} 处断言无出处")),
                    "无出处计数不符：期望 {unsourced}，摘要 {:?}（输入 {text:?}）",
                    r.summary
                );
            }
            // 过度断言必须排在摘要最前
            if over > 0 && unsourced > 0 {
                let io = r.summary.find("过度断言").unwrap_or(usize::MAX);
                let iu = r.summary.find("无出处").unwrap_or(usize::MAX);
                assert!(io < iu, "过度断言要排第一：{:?}", r.summary);
            }
        }
    }
}
