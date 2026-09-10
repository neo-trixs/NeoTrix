//! KB 共享领域类型 — 单一事实源在 neotrix-types::knowledge_access。
//!
//! core 层 (second_brain 等) 需要知识图谱节点/边类型枚举。
//! 这些是纯 serde 枚举 (无状态、无 l3 依赖)，由 neotrix-types 提供单一定义，
//! 此处 re-export 保持 `crate::core::nt_core_kb_types::*` 路径不变。

pub use neotrix_types::knowledge_access::{
    KnowledgeEdge, KnowledgeNode, NodeType, RelationType, TemporalValidity,
};

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// NT-CORE KB 类型核心自测: NodeType 枚举 as_str<->from_str 全变体往返 (卫生层 P0: 核心必须可自测)。
pub struct KbTypesSelfTest;

impl SelfTest for KbTypesSelfTest {
    fn name(&self) -> &str {
        "kb_types_core"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let all = [
            NodeType::Concept, NodeType::Paper, NodeType::Repository, NodeType::Person,
            NodeType::Event, NodeType::Source, NodeType::Tool, NodeType::Framework,
            NodeType::Algorithm, NodeType::Theory, NodeType::Method, NodeType::Dataset,
            NodeType::Benchmark, NodeType::Organization, NodeType::Book, NodeType::Course,
            NodeType::Article, NodeType::CodeSnippet, NodeType::Idea, NodeType::Question,
            NodeType::Insight, NodeType::HarnessProfile, NodeType::Image,
            NodeType::EvolutionPattern, NodeType::ConversationEvolution, NodeType::Textbook,
            NodeType::Resource, NodeType::External, NodeType::Summary, NodeType::Guide,
            NodeType::Skill, NodeType::Reference, NodeType::WikiPage, NodeType::ThinkingTrace,
            NodeType::SelfTestFailure, NodeType::EventRecord, NodeType::DetectionFinding,
            NodeType::GoalResult, NodeType::Session, NodeType::Filing,
        ];
        for v in all {
            if NodeType::from_str(v.as_str()) != v {
                failures.push(format!(
                    "kb_types_core: NodeType roundtrip failed for {}",
                    v.as_str()
                ));
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// 注册 KB 类型核心 SelfTest 到全局注册表 (T2)。
pub fn register_kb_types_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(KbTypesSelfTest));
}

#[cfg(test)]
mod selftest_tests {
    use super::*;

    #[test]
    fn test_kb_types_self_test_passes() {
        let t = super::KbTypesSelfTest;
        assert!(
            t.self_test().is_ok(),
            "KbTypesSelfTest failed: {:?}",
            t.self_test().err()
        );
    }
}
