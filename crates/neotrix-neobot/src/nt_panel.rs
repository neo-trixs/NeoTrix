//! 决策面板 —— 骨架侧的应答契约。
//!
//! # 为什么这个类型要在**库里**而不是只在前端
//!
//! 前端已经有一份同构的 `DecisionPanel`（`frontend/src/ui/block-model.ts`），
//! 且已实现了版本校验。**那份校验只能保护界面，不能保护状态。**
//! 前端校验发生在「用户的浏览器里」—— 真正的授权判定不能建立在
//! 一个调用方可以改的检查上。
//!
//! 更实际的理由：**过期作答会静默污染状态**。
//! 骨架在 v1 候选集上工作，用户在界面上选了一个 v2 才有的选项；
//! 若不拦，这个选择会写进会话，而界面上**看不出任何异常**。
//! 所以这条不变量必须在骨架这侧再兜一次，且要有测试。
//!
//! # 与前端类型的关系
//!
//! 两份定义**手工保持一致**（跨语言，无法共享类型）。`neobot_panel_answer`
//! 的返回体会告诉前端「你这个作答被拒了以及为什么」，前端据此提示重选。

use serde::{Deserialize, Serialize};

/// 一条出处。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub title: String,
    /// 必须是 http(s)。**入库前校验**：`javascript:` 到了渲染层
    /// 就是可执行注入点，而渲染层是本仓之外的东西。
    pub url: String,
}

/// 面板里的一个候选。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelOption {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub details: Vec<String>,
    #[serde(default)]
    pub sources: Vec<Source>,
}

/// 决策面板。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Panel {
    pub id: String,
    pub thread_id: String,
    pub turn_id: String,
    /// 候选集版本。骨架每换一批候选就 +1。
    /// 这是**整套机制的关键字段**：没有它，「用户选的那个」与
    /// 「我给的那批」就无法对上，而界面看不出任何异常。
    pub candidate_set_version: u64,
    pub kind: PanelKind,
    pub title: String,
    pub options: Vec<PanelOption>,
    #[serde(default)]
    pub mode: Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanelKind {
    /// 需要用户澄清。
    Clarification,
    /// 让用户在几个候选里比较后选一个。
    Comparison,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// 演示数据。界面必须区别显示，不能冒充真实候选。
    Sample,
    /// 真实候选。
    Live,
}

impl Default for Mode {
    /// 缺省是 **Sample**，不是 Live。
    ///
    /// ⛔ 骨架若忘了带 `mode`，缺省成 Live 就等于「在没有证据的情况下
    ///    声称这是真实候选」—— 与 dsh-market 那条「`undefined` = 没扫过，
    ///    不能说成『没检出』」是同一个错误的两个版本。
    ///    缺省成 Sample 最多让真面板被错标成演示（用户看得到警示），
    ///    缺省成 Live 则让演示面板冒充真实（用户看不出来）。后者的代价大得多。
    fn default() -> Self {
        Mode::Sample
    }
}

/// 界面回传的作答。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    pub panel_id: String,
    pub option_id: String,
    /// 界面**看到**的候选集版本。
    ///
    /// ⛔ 这不是「作答时的版本」的官方记录，而是「用户以为自己选的是哪一批」。
    /// 骨架拿它和自己当前的版本比，对不上就说明用户选的是**过期**那一批。
    pub candidate_set_version: u64,
}

/// 作答被拒的理由。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reject {
    /// 面板不存在。常见于界面拿着旧面板作答（会话已切走）。
    NoSuchPanel,
    /// 候选集已更新 —— 用户选的是上一批。
    Stale { current: u64, answered: u64 },
    /// 选项不在当前候选集里。
    NoSuchOption,
    /// 面板本身不合法（骨架的 bug），此时不能接受任何作答。
    PanelInvalid(String),
}

/// ⚠️ 必须 `Serialize`：它是 Tauri 命令的**返回类型**，
///    少这个 derive 的报错是 `blocking_kind` 不存在 —— 编译器的提示指向一个
///    完全无关的方法名，很容易把人带偏（实际是「不能被当作返回值序列化」。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AnswerOutcome {
    Accepted,
    Rejected(Reject),
}

/// 校验一次作答。
///
/// ⛔ **过期一律拒**，不静默接受。理由：那会用旧选择覆盖新候选集，
///    而界面上看不出任何异常 —— 属于「看起来在工作、实际在损坏数据」。
pub fn validate_answer(panel: &Panel, answer: &Answer) -> AnswerOutcome {
    if answer.panel_id != panel.id {
        return AnswerOutcome::Rejected(Reject::NoSuchPanel);
    }
    // 面板自身不合法 ⇒ 任何作答都不能接受。先报面板的问题，
    // 否则用户会看到「你选的选项不存在」而不知道真正的错在骨架。
    if let Err(why) = validate_panel(panel) {
        return AnswerOutcome::Rejected(Reject::PanelInvalid(why));
    }
    if answer.candidate_set_version != panel.candidate_set_version {
        return AnswerOutcome::Rejected(Reject::Stale {
            current: panel.candidate_set_version,
            answered: answer.candidate_set_version,
        });
    }
    if !panel.options.iter().any(|o| o.id == answer.option_id) {
        return AnswerOutcome::Rejected(Reject::NoSuchOption);
    }
    AnswerOutcome::Accepted
}

/// 校验面板本身。
pub fn validate_panel(panel: &Panel) -> Result<(), String> {
    if panel.id.trim().is_empty() {
        return Err("面板 id 不能为空".to_owned());
    }
    if panel.candidate_set_version < 1 {
        return Err("candidate_set_version 必须 ≥ 1".to_owned());
    }
    if panel.options.is_empty() {
        return Err("决策面板至少要一个选项".to_owned());
    }
    if panel.options.len() > 12 {
        return Err("决策面板选项超过 12 个".to_owned());
    }
    let mut seen = std::collections::HashSet::new();
    for o in &panel.options {
        if !seen.insert(o.id.as_str()) {
            return Err(format!("选项 id 重复：{}", o.id));
        }
        for s in &o.sources {
            if !(s.url.starts_with("http://") || s.url.starts_with("https://")) {
                return Err(format!("选项「{}」的出处必须是 http(s)：{}", o.label, s.url));
            }
        }
        // 比较型要求每个选项都有出处 ——
        // 否则「比较」只是在比几句没有依据的断言。
        if panel.kind == PanelKind::Comparison && o.sources.is_empty() {
            return Err(format!("比较型面板的选项「{}」必须给出处", o.label));
        }
    }
    if panel.kind == PanelKind::Comparison && panel.options.len() > 3 {
        return Err("比较型面板最多 3 个选项".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(t: &str) -> Source {
        Source { title: t.to_owned(), url: format!("https://example.com/{t}") }
    }

    fn comparison() -> Panel {
        Panel {
            id: "p1".to_owned(),
            thread_id: "t1".to_owned(),
            turn_id: "tu1".to_owned(),
            candidate_set_version: 1,
            kind: PanelKind::Comparison,
            title: "选哪个".to_owned(),
            mode: Mode::Live,
            options: vec![
                PanelOption {
                    id: "a".to_owned(),
                    label: "A".to_owned(),
                    details: vec![],
                    sources: vec![src("a")],
                },
                PanelOption {
                    id: "b".to_owned(),
                    label: "B".to_owned(),
                    details: vec![],
                    sources: vec![src("b")],
                },
            ],
        }
    }

    fn answer(panel_id: &str, option_id: &str, v: u64) -> Answer {
        Answer {
            panel_id: panel_id.to_owned(),
            option_id: option_id.to_owned(),
            candidate_set_version: v,
        }
    }

    #[test]
    fn 同版本作答被接受() {
        let p = comparison();
        assert_eq!(
            validate_answer(&p, &answer("p1", "a", 1)),
            AnswerOutcome::Accepted
        );
    }

    #[test]
    fn 过期作答被拒() {
        // ⛔ 这是整套机制存在的理由：用户选的是**上一批**候选。
        //    若接受，新候选集会带着一个来自旧集合的选择继续走，
        //    而界面上**看不出任何异常**。
        let p = comparison();
        assert_eq!(
            validate_answer(&p, &answer("p1", "a", 0)),
            AnswerOutcome::Rejected(Reject::Stale { current: 1, answered: 0 })
        );
    }

    #[test]
    fn 换批之后旧选择不再有效() {
        let mut p = comparison();
        p.candidate_set_version = 2;   // 骨架换了一批候选
        assert_eq!(
            validate_answer(&p, &answer("p1", "a", 1)),
            AnswerOutcome::Rejected(Reject::Stale { current: 2, answered: 1 })
        );
    }

    #[test]
    fn 面板不存在被拒() {
        let p = comparison();
        assert_eq!(
            validate_answer(&p, &answer("p-其他", "a", 1)),
            AnswerOutcome::Rejected(Reject::NoSuchPanel)
        );
    }

    #[test]
    fn 不存在的选项被拒() {
        let p = comparison();
        assert_eq!(
            validate_answer(&p, &answer("p1", "zzz", 1)),
            AnswerOutcome::Rejected(Reject::NoSuchOption)
        );
    }

    #[test]
    fn 面板本身不合法时先报面板() {
        // ⛔ 先报 NoSuchOption 会让用户以为「选项不见了」，
        //    而真正的错在骨架下了一个非法面板。
        let mut p = comparison();
        p.options.clear();
        assert!(matches!(
            validate_answer(&p, &answer("p1", "a", 1)),
            AnswerOutcome::Rejected(Reject::PanelInvalid(_))
        ));
    }

    #[test]
    fn 比较型选项必须给出处() {
        let mut p = comparison();
        p.options[0].sources.clear();
        let err = validate_panel(&p).expect_err("比较型缺出处必须被拒");
        assert!(err.contains("必须给出处"), "{err}");
    }

    #[test]
    fn 非http出处被拒() {
        let mut p = comparison();
        p.options[0].sources[0].url = "javascript:alert(1)".to_owned();
        let err = validate_panel(&p).expect_err("非 http 出处必须被拒");
        assert!(err.contains("http(s)"), "{err}");
    }

    #[test]
    fn 比较型最多三项() {
        let mut p = comparison();
        p.options.push(PanelOption {
            id: "c".to_owned(),
            label: "C".to_owned(),
            details: vec![],
            sources: vec![src("c")],
        });
        p.options.push(PanelOption {
            id: "d".to_owned(),
            label: "D".to_owned(),
            details: vec![],
            sources: vec![src("d")],
        });
        assert!(validate_panel(&p).is_err(), "比较型超过 3 项必须被拒");
    }

    #[test]
    fn 澄清型不要求出处() {
        // 澄清型是「问问题」，不是「比方案」，不强制出处。
        let mut p = comparison();
        p.kind = PanelKind::Clarification;
        p.options[0].sources.clear();
        assert!(validate_panel(&p).is_ok(), "澄清型不该要求出处");
    }

    #[test]
    fn 版本号必须至少为1() {
        let mut p = comparison();
        p.candidate_set_version = 0;
        assert!(validate_panel(&p).is_err());
    }

    #[test]
    fn 重复选项id被拒() {
        let mut p = comparison();
        p.options[1].id = "a".to_owned();
        let err = validate_panel(&p).expect_err("重复 id 必须被拒");
        assert!(err.contains("重复"), "{err}");
    }
}
