//! L6 / NT-ACT — hermeskill (github.com/theopitori/hermeskill) 吸收节点 (C1)。
//!
//! 源: hermeskill — Hermes 技能节点框架: 把 agent 能力模块化组织为可组合/可调
//! 用的技能节点 (skill node)。NeoTrix 视角: 技能节点注册表 + 调用路由 trait。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 技能节点。
#[derive(Debug, Clone)]
pub struct SkillNode {
    pub id: String,
    pub capability: String,
    pub enabled: bool,
}

/// 技能节点注册表/路由器。
pub trait SkillRegistry {
    fn register(&mut self, node: SkillNode);
    fn resolve(&self, capability: &str) -> Option<&SkillNode>;
    fn enabled_count(&self) -> usize;
}

pub struct HermesSkillRegistry {
    nodes: HashMap<String, SkillNode>,
}

impl Default for HermesSkillRegistry {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }
}

impl SkillRegistry for HermesSkillRegistry {
    fn register(&mut self, node: SkillNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    fn resolve(&self, capability: &str) -> Option<&SkillNode> {
        self.nodes.values().find(|n| n.capability == capability && n.enabled)
    }

    fn enabled_count(&self) -> usize {
        self.nodes.values().filter(|n| n.enabled).count()
    }
}

#[derive(Default)]
pub struct HermesSkillSelfTest;

impl SelfTest for HermesSkillSelfTest {
    fn name(&self) -> &str {
        "nt_act_hermeskill"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut reg = HermesSkillRegistry::default();
        reg.register(SkillNode {
            id: "s1".into(),
            capability: "search".into(),
            enabled: true,
        });
        reg.register(SkillNode {
            id: "s2".into(),
            capability: "parse".into(),
            enabled: false,
        });
        let mut errs = Vec::new();
        if reg.enabled_count() != 1 {
            errs.push(format!("hermeskill: enabled_count {}, expected 1", reg.enabled_count()));
        }
        if reg.resolve("search").map(|n| n.id.as_str()) != Some("s1") {
            errs.push("hermeskill: must resolve enabled search node".into());
        }
        if reg.resolve("parse").is_some() {
            errs.push("hermeskill: disabled node must not resolve".into());
        }
        if reg.resolve("missing").is_some() {
            errs.push("hermeskill: unknown capability must not resolve".into());
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
    fn register_and_resolve() {
        let mut reg = HermesSkillRegistry::default();
        reg.register(SkillNode {
            id: "a".into(),
            capability: "x".into(),
            enabled: true,
        });
        assert_eq!(reg.resolve("x").unwrap().id, "a");
    }

    #[test]
    fn disabled_not_resolved() {
        let mut reg = HermesSkillRegistry::default();
        reg.register(SkillNode {
            id: "a".into(),
            capability: "x".into(),
            enabled: false,
        });
        assert!(reg.resolve("x").is_none());
    }

    #[test]
    fn enabled_count() {
        let mut reg = HermesSkillRegistry::default();
        reg.register(SkillNode { id: "a".into(), capability: "x".into(), enabled: true });
        reg.register(SkillNode { id: "b".into(), capability: "y".into(), enabled: false });
        assert_eq!(reg.enabled_count(), 1);
    }
}
