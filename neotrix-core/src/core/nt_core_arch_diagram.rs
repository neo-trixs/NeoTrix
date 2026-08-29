//! NT-CORE 可验证架构图 (吸收 `tt-a1i/archify` + `cathrynlavery/diagram-design`):
//! 从节点/边生成 self-contained SVG, 标签清晰、确定性可校验。R-P42 强化现有架构表示,
//! 不新建平行渲染器 — 仅提供架构图原语供 Seed Graph / 意识树消费。

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// 架构图节点: 标签 + 所属层 (0..=9, NeoTrix 9 层架构)。
#[derive(Debug, Clone)]
pub struct ArchNode {
    pub label: String,
    pub layer: u8,
}

/// 生成 self-contained SVG 架构图: 按 layer 纵向分层堆叠节点盒, 边以竖线表示。
/// 输出确定性 (同输入同输出), 标签明文可见 → 可校验 (archify "verifiable")。
pub fn render_arch_diagram(nodes: &[ArchNode], edges: &[(usize, usize)]) -> String {
    let width: u32 = 760;
    let row_h: u32 = 46;
    let height: u32 = (nodes.len() as u32) * row_h + 20;
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
        width, height, width, height
    );
    for (i, n) in nodes.iter().enumerate() {
        let y = 10 + i as u32 * row_h;
        svg.push_str(&format!(
            "<g class=\"layer-{}\"><rect x=\"20\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"6\" fill=\"#1b1b2f\" stroke=\"#8b7fff\"/>\
<text x=\"30\" y=\"{}\" fill=\"#e8e8ff\" font-family=\"monospace\" font-size=\"14\">{:?}</text></g>",
            n.layer,
            y,
            width - 40,
            row_h - 8,
            y + row_h / 2,
            n.label
        ));
    }
    for (a, b) in edges {
        let ya = 10 + *a as u32 * row_h + row_h / 2;
        let yb = 10 + *b as u32 * row_h + row_h / 2;
        svg.push_str(&format!(
            "<line x1=\"40\" y1=\"{}\" x2=\"40\" y2=\"{}\" stroke=\"#555\" />",
            ya, yb
        ));
    }
    svg.push_str("</svg>");
    svg
}

/// NT-CORE 可验证架构图自测 (卫生层 P0)。
pub struct ArchDiagramSelfTest;

impl SelfTest for ArchDiagramSelfTest {
    fn name(&self) -> &str {
        "nt_core_arch_diagram"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let nodes = vec![
            ArchNode { label: "L0 Substrate".into(), layer: 0 },
            ArchNode { label: "L1 Body".into(), layer: 1 },
            ArchNode { label: "L9 Transcendent".into(), layer: 9 },
        ];
        let svg = render_arch_diagram(&nodes, &[(0, 1), (1, 2)]);
        for expected in ["<svg", "L0 Substrate", "L1 Body", "L9 Transcendent"] {
            if !svg.contains(expected) {
                return Err(vec![format!("arch diagram missing {}", expected)]);
            }
        }
        if svg.matches("<line").count() != 2 {
            return Err(vec!["arch diagram edge count wrong".into()]);
        }
        Ok(())
    }
}

/// 注册架构图 SelfTest 到全局注册表 (T2)。
pub fn register_arch_diagram_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(ArchDiagramSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_diagram_self_test_passes() {
        assert!(
            ArchDiagramSelfTest.self_test().is_ok(),
            "arch diagram self_test failed: {:?}",
            ArchDiagramSelfTest.self_test().err()
        );
    }
}
