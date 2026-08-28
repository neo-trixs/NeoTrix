//! L6 / NT-ACT — thinkrail (github.com/JetBrains/thinkrail) 吸收节点 (C0)。
//!
//! 源: thinkrail — 思考/决策轨道 (thought/decision rail) 框架: 把 agent 的
//! 推理过程显式组织为"轨道", 区分思考轨与决策轨, 防止决策被未收敛的思考污染。
//! NeoTrix 视角: mini 模块, 轨道分类器 (C0: 编译 + 基础逻辑 + SelfTest T1)。

use crate::core::nt_core_self_test::SelfTest;

/// 轨道类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rail {
    Thought,
    Decision,
}

/// 思考/决策轨道分类器。
pub trait RailClassifier {
    /// 依据内容信号 (含 "?/maybe/consider" → 思考轨; 含 "do/set/choose" → 决策轨)
    /// 归类轨道。
    fn classify(&self, text: &str) -> Rail;
}

pub struct ThinkRail;

impl RailClassifier for ThinkRail {
    fn classify(&self, text: &str) -> Rail {
        let t = text.to_ascii_lowercase();
        if t.contains("do ") || t.contains("set ") || t.contains("choose ") {
            Rail::Decision
        } else {
            Rail::Thought
        }
    }
}

#[derive(Default)]
pub struct ThinkRailSelfTest;

impl SelfTest for ThinkRailSelfTest {
    fn name(&self) -> &str {
        "nt_act_thinkrail"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let r = ThinkRail;
        let mut errs = Vec::new();
        if r.classify("let's consider maybe?") != Rail::Thought {
            errs.push("thinkrail: uncertain text must be Thought rail".into());
        }
        if r.classify("we should do the action") != Rail::Decision {
            errs.push("thinkrail: imperative text must be Decision rail".into());
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_thought() {
        assert_eq!(ThinkRail.classify("maybe consider?"), Rail::Thought);
    }

    #[test]
    fn classifies_decision() {
        assert_eq!(ThinkRail.classify("do the thing"), Rail::Decision);
    }
}
