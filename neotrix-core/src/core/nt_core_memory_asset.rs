//! NT-MEMORY 四态记忆资产模型 — 吸收 `TencentCloud/TencentDB-Agent-Memory`。
//!
//! 将知识节点分类为四种可复用记忆资产 (R-P42 强化现有 `KnowledgeNode`, 不新建平行存储):
//! `ChatMemory` / `Skill` / `LlmWiki` / `CodeGraph`。分类优先级: metadata 显式覆盖 →
//! node_type 启发式。为 KB 提供团队级记忆中枢的资产视图, 支撑跨会话持久与技能结晶。

use crate::core::nt_core_kb_types::KnowledgeNode;
use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};
use serde::{Deserialize, Serialize};

/// 记忆资产四态 — 团队级记忆中枢的可复用资产类别。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryAssetKind {
    /// 对话记忆 — 由会话蒸馏得到的可复用上下文
    ChatMemory,
    /// 技能 — 可执行的技能/工具封装
    Skill,
    /// LLM 维基 — 由文档/知识蒸馏得到的百科式条目
    LlmWiki,
    /// 代码图 — 代码库结构/依赖的知识图谱切片
    CodeGraph,
}

impl MemoryAssetKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryAssetKind::ChatMemory => "chat_memory",
            MemoryAssetKind::Skill => "skill",
            MemoryAssetKind::LlmWiki => "llm_wiki",
            MemoryAssetKind::CodeGraph => "code_graph",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "chat_memory" | "chatmemory" => Some(MemoryAssetKind::ChatMemory),
            "skill" => Some(MemoryAssetKind::Skill),
            "llm_wiki" | "llmwiki" | "wiki" => Some(MemoryAssetKind::LlmWiki),
            "code_graph" | "codegraph" => Some(MemoryAssetKind::CodeGraph),
            _ => None,
        }
    }

    /// 依据节点 metadata 覆盖 + node_type 启发式分类; 无法归类返回 `None`。
    pub fn classify(node: &KnowledgeNode) -> Option<Self> {
        // 1. metadata 显式覆盖 (最高优先)
        if let Some(meta) = &node.metadata {
            if let Some(v) = meta.get("asset_kind").and_then(|v| v.as_str()) {
                if let Some(k) = Self::from_str(v) {
                    return Some(k);
                }
            }
        }
        // 2. node_type 启发式
        match node.node_type {
            crate::core::nt_core_kb_types::NodeType::Skill => Some(MemoryAssetKind::Skill),
            crate::core::nt_core_kb_types::NodeType::WikiPage
            | crate::core::nt_core_kb_types::NodeType::Guide
            | crate::core::nt_core_kb_types::NodeType::Textbook
            | crate::core::nt_core_kb_types::NodeType::Article => Some(MemoryAssetKind::LlmWiki),
            crate::core::nt_core_kb_types::NodeType::CodeSnippet
            | crate::core::nt_core_kb_types::NodeType::Repository => Some(MemoryAssetKind::CodeGraph),
            crate::core::nt_core_kb_types::NodeType::ConversationEvolution
            | crate::core::nt_core_kb_types::NodeType::Session => Some(MemoryAssetKind::ChatMemory),
            _ => None,
        }
    }
}

/// NT-MEMORY 四态资产分类核心自测 (卫生层 P0: 核心必须可自测)。
pub struct MemoryAssetSelfTest;

impl SelfTest for MemoryAssetSelfTest {
    fn name(&self) -> &str {
        "memory_asset_kind"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let all = [
            MemoryAssetKind::ChatMemory,
            MemoryAssetKind::Skill,
            MemoryAssetKind::LlmWiki,
            MemoryAssetKind::CodeGraph,
        ];
        for v in all {
            if MemoryAssetKind::from_str(v.as_str()) != Some(v) {
                failures.push(format!("memory_asset_kind: roundtrip failed for {}", v.as_str()));
            }
        }
        // 分类启发式校验
        let skill = KnowledgeNode {
            id: "s1".into(),
            node_type: crate::core::nt_core_kb_types::NodeType::Skill,
            title: "t".into(),
            summary: None,
            content: None,
            url: None,
            domain: None,
            language: "en".into(),
            confidence: 1.0,
            importance: 1.0,
            recall_weight: 1.0,
            created_at: 0,
            updated_at: 0,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        };
        if MemoryAssetKind::classify(&skill) != Some(MemoryAssetKind::Skill) {
            failures.push("memory_asset_kind: Skill classify failed".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// 注册四态资产 SelfTest 到全局注册表 (T2)。
pub fn register_memory_asset_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(MemoryAssetSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_kb_types::{KnowledgeNode, NodeType};

    fn base(node_type: NodeType) -> KnowledgeNode {
        KnowledgeNode {
            id: "x".into(),
            node_type,
            title: "t".into(),
            summary: None,
            content: None,
            url: None,
            domain: None,
            language: "en".into(),
            confidence: 1.0,
            importance: 1.0,
            recall_weight: 1.0,
            created_at: 0,
            updated_at: 0,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        }
    }

    #[test]
    fn test_asset_kind_roundtrip() {
        for v in [
            MemoryAssetKind::ChatMemory,
            MemoryAssetKind::Skill,
            MemoryAssetKind::LlmWiki,
            MemoryAssetKind::CodeGraph,
        ] {
            assert_eq!(MemoryAssetKind::from_str(v.as_str()), Some(v));
        }
    }

    #[test]
    fn test_classify_by_node_type() {
        assert_eq!(MemoryAssetKind::classify(&base(NodeType::Skill)), Some(MemoryAssetKind::Skill));
        assert_eq!(MemoryAssetKind::classify(&base(NodeType::WikiPage)), Some(MemoryAssetKind::LlmWiki));
        assert_eq!(
            MemoryAssetKind::classify(&base(NodeType::Repository)),
            Some(MemoryAssetKind::CodeGraph)
        );
        assert_eq!(
            MemoryAssetKind::classify(&base(NodeType::ConversationEvolution)),
            Some(MemoryAssetKind::ChatMemory)
        );
        // 无显式/启发式覆盖 → None
        assert_eq!(MemoryAssetKind::classify(&base(NodeType::Concept)), None);
    }

    #[test]
    fn test_classify_metadata_override() {
        let mut n = base(NodeType::Concept);
        n.metadata = Some(serde_json::json!({ "asset_kind": "code_graph" }));
        assert_eq!(MemoryAssetKind::classify(&n), Some(MemoryAssetKind::CodeGraph));
    }
}
