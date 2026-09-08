//! Excalidraw 图渲染模块 (NT-IO)
//!
//! 吸收源: github.com/coleam00/excalidraw-diagram-skill
//! 成熟度: C1 (unit-tested stub, 无外部渲染服务集成)
//!
//! 核心能力: 将结构化图描述 (节点 + 连线) 转换为 Excalidraw scene JSON,
//! 供后续渲染/导出。本 stub 负责场景骨架生成与基础校验。

use crate::core::nt_core_self_test::SelfTest;

/// 图节点。
#[derive(Debug, Clone, PartialEq)]
pub struct DiagramNode {
    pub id: String,
    pub label: String,
}

/// 图连线。
#[derive(Debug, Clone, PartialEq)]
pub struct DiagramEdge {
    pub from: String,
    pub to: String,
}

/// 图渲染器 trait — 把图规格转换为可序列化的场景表示。
pub trait DiagramRenderer: Send + Sync {
    /// 渲染场景: 返回元素数量 (节点+连线), 或 None 当规格非法。
    fn render_scene(&self, nodes: &[DiagramNode], edges: &[DiagramEdge]) -> Option<usize>;
    /// 校验规格: 所有 edge 端点必须存在于节点集合。
    fn is_valid(&self, nodes: &[DiagramNode], edges: &[DiagramEdge]) -> bool;
}

/// 默认实现: 骨架计数式渲染 + 端点一致性校验。
#[derive(Default)]
pub struct ExcalidrawRenderer;

impl DiagramRenderer for ExcalidrawRenderer {
    fn render_scene(&self, nodes: &[DiagramNode], edges: &[DiagramEdge]) -> Option<usize> {
        if !self.is_valid(nodes, edges) {
            return None;
        }
        Some(nodes.len() + edges.len())
    }

    fn is_valid(&self, nodes: &[DiagramNode], edges: &[DiagramEdge]) -> bool {
        let ids: std::collections::HashSet<&str> =
            nodes.iter().map(|n| n.id.as_str()).collect();
        edges.iter().all(|e| ids.contains(e.from.as_str()) && ids.contains(e.to.as_str()))
    }
}

/// T1 SelfTest: 验证渲染器存在并正确校验端点。
#[derive(Default)]
pub struct ExcalidrawSelfTest;

impl SelfTest for ExcalidrawSelfTest {
    fn name(&self) -> &str {
        "nt_io_excalidraw"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let r = ExcalidrawRenderer;
        let nodes = vec![
            DiagramNode { id: "a".into(), label: "A".into() },
            DiagramNode { id: "b".into(), label: "B".into() },
        ];
        let edges = vec![DiagramEdge { from: "a".into(), to: "b".into() }];
        match r.render_scene(&nodes, &edges) {
            Some(n) if n == 3 => Ok(()),
            _ => Err(vec!["nt_io_excalidraw: valid scene failed to render".into()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (Vec<DiagramNode>, Vec<DiagramEdge>) {
        (
            vec![
                DiagramNode { id: "a".into(), label: "A".into() },
                DiagramNode { id: "b".into(), label: "B".into() },
            ],
            vec![DiagramEdge { from: "a".into(), to: "b".into() }],
        )
    }

    #[test]
    fn test_render_valid_scene() {
        let (n, e) = sample();
        assert_eq!(ExcalidrawRenderer.render_scene(&n, &e), Some(3));
    }

    #[test]
    fn test_invalid_edge_rejected() {
        let (n, _) = sample();
        let bad = vec![DiagramEdge { from: "a".into(), to: "z".into() }];
        assert!(!ExcalidrawRenderer.is_valid(&n, &bad));
        assert_eq!(ExcalidrawRenderer.render_scene(&n, &bad), None);
    }

    #[test]
    fn test_empty_scene_renders_zero() {
        let r = ExcalidrawRenderer;
        assert_eq!(r.render_scene(&[], &[]), Some(0));
    }
}
