//! NT-IO multimodal_transform — 文本模态: 图表语义模型与渲染 (`nt_text_transform`).
//!
//! G17 吸收 (pretty-mermaid-skills + diagram-design): 27 种视觉类型分类、
//! 语义/布局解耦 (`DiagramModel` + ASCII 框线 / Mermaid 文本)。零外部渲染依赖。

// ────────────────────────────────────────────────────────────────────────────
// Diagram/Chart Rendering (G17) — 语义模型与布局表现解耦
// ────────────────────────────────────────────────────────────────────────────

/// 27 种视觉类型分类 (吸收自 pretty-mermaid-skills / diagram-design)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum VisualType {
    #[default]
    Flowchart,
    Sequence,
    Class,
    State,
    EntityRelation,
    Gantt,
    Pie,
    Quadrant,
    Architecture,
    Network,
    Timeline,
    Radar,
    Sankey,
    Mindmap,
    GitGraph,
    ERDiagram,
    BarChart,
    LineChart,
    Scatter,
    Histogram,
    Heatmap,
    TreeMap,
    Bubble,
    DecisionTree,
    VenDiagram,
    C4Model,
    Other,
}

impl VisualType {
    /// 全量 27 类 (供分类/校验/测试)。
    pub const ALL: [VisualType; 27] = [
        VisualType::Flowchart,
        VisualType::Sequence,
        VisualType::Class,
        VisualType::State,
        VisualType::EntityRelation,
        VisualType::Gantt,
        VisualType::Pie,
        VisualType::Quadrant,
        VisualType::Architecture,
        VisualType::Network,
        VisualType::Timeline,
        VisualType::Radar,
        VisualType::Sankey,
        VisualType::Mindmap,
        VisualType::GitGraph,
        VisualType::ERDiagram,
        VisualType::BarChart,
        VisualType::LineChart,
        VisualType::Scatter,
        VisualType::Histogram,
        VisualType::Heatmap,
        VisualType::TreeMap,
        VisualType::Bubble,
        VisualType::DecisionTree,
        VisualType::VenDiagram,
        VisualType::C4Model,
        VisualType::Other,
    ];

    pub fn name(self) -> &'static str {
        match self {
            VisualType::Flowchart => "flowchart",
            VisualType::Sequence => "sequence",
            VisualType::Class => "class",
            VisualType::State => "state",
            VisualType::EntityRelation => "entity_relation",
            VisualType::Gantt => "gantt",
            VisualType::Pie => "pie",
            VisualType::Quadrant => "quadrant",
            VisualType::Architecture => "architecture",
            VisualType::Network => "network",
            VisualType::Timeline => "timeline",
            VisualType::Radar => "radar",
            VisualType::Sankey => "sankey",
            VisualType::Mindmap => "mindmap",
            VisualType::GitGraph => "git_graph",
            VisualType::ERDiagram => "er_diagram",
            VisualType::BarChart => "bar_chart",
            VisualType::LineChart => "line_chart",
            VisualType::Scatter => "scatter",
            VisualType::Histogram => "histogram",
            VisualType::Heatmap => "heatmap",
            VisualType::TreeMap => "tree_map",
            VisualType::Bubble => "bubble",
            VisualType::DecisionTree => "decision_tree",
            VisualType::VenDiagram => "venn_diagram",
            VisualType::C4Model => "c4_model",
            VisualType::Other => "other",
        }
    }

    /// 从源文本关键词分类。语义先行: 命中即返回, 兜底 Other。
    pub fn classify(source: &str) -> VisualType {
        let s = source.to_lowercase();
        // 精确 Mermaid 类型头 (如 `flowchart TD` / `sequenceDiagram`)。
        if s.contains("sequencediagram") || s.contains("sequence diagram") {
            return VisualType::Sequence;
        }
        if s.contains("classdiagram") {
            return VisualType::Class;
        }
        if s.contains("statediagram") || s.contains("state diagram") {
            return VisualType::State;
        }
        if s.contains("gantt") {
            return VisualType::Gantt;
        }
        if s.contains("pie ") || s.starts_with("pie\n") || s.contains("piechart") {
            return VisualType::Pie;
        }
        if s.contains("erdiagram") || s.contains("entity relation") {
            return VisualType::ERDiagram;
        }
        if s.contains("mindmap") || s.contains("mind map") {
            return VisualType::Mindmap;
        }
        if s.contains("gitgraph") || s.contains("git graph") {
            return VisualType::GitGraph;
        }
        if s.contains("sankey") {
            return VisualType::Sankey;
        }
        if s.contains("radar") {
            return VisualType::Radar;
        }
        if s.contains("timeline") {
            return VisualType::Timeline;
        }
        if s.contains("heatmap") {
            return VisualType::Heatmap;
        }
        if s.contains("histogram") {
            return VisualType::Histogram;
        }
        if s.contains("scatter") {
            return VisualType::Scatter;
        }
        if s.contains("bubble chart") || s.contains("bubblechart") {
            return VisualType::Bubble;
        }
        if s.contains("tree map") || s.contains("treemap") {
            return VisualType::TreeMap;
        }
        if s.contains("bar chart") || s.contains("barchart") {
            return VisualType::BarChart;
        }
        if s.contains("line chart") || s.contains("linechart") {
            return VisualType::LineChart;
        }
        if s.contains("venn") || s.contains("ven diagram") {
            return VisualType::VenDiagram;
        }
        if s.contains("decision tree") {
            return VisualType::DecisionTree;
        }
        if s.contains("c4 model") || s.contains("c4model") {
            return VisualType::C4Model;
        }
        if s.contains("quadrant") {
            return VisualType::Quadrant;
        }
        if s.contains("architecture") || s.contains("arch diagram") {
            return VisualType::Architecture;
        }
        if s.contains("network") || s.contains("topology") {
            return VisualType::Network;
        }
        if s.contains("flowchart") || s.contains("flow chart") || s.contains("->") {
            return VisualType::Flowchart;
        }
        VisualType::Other
    }
}

/// 图节点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramNode {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
}

/// 节点形状。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Process,
    Decision,
    Terminator,
    Data,
    Subprocess,
}

impl NodeKind {
    fn shape(self) -> (char, char) {
        // (左右框符) — Process 用方框, Decision 用尖括号, 数据用双线。
        match self {
            NodeKind::Process => ('[', ']'),
            NodeKind::Decision => ('<', '>'),
            NodeKind::Terminator => ('(', ')'),
            NodeKind::Data => ('{', '}'),
            NodeKind::Subprocess => ('(', ')'),
        }
    }
}

/// 有向边。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// 语义图模型 — 与布局表现解耦。
#[derive(Debug, Clone, Default)]
pub struct DiagramModel {
    pub title: Option<String>,
    pub vtype: VisualType,
    pub nodes: Vec<DiagramNode>,
    pub edges: Vec<DiagramEdge>,
}

impl DiagramModel {
    pub fn new(vtype: VisualType) -> Self {
        Self {
            title: None,
            vtype,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn add_node(&mut self, node: DiagramNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: DiagramEdge) {
        self.edges.push(edge);
    }

    pub fn node(&self, id: &str) -> Option<&DiagramNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// 出边邻接 (供遍历/布局)。
    pub fn out_edges(&self, id: &str) -> Vec<&DiagramEdge> {
        self.edges.iter().filter(|e| e.from == id).collect()
    }
}

/// 渲染选项。
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// 是否输出标题横幅。
    pub with_title: bool,
    /// 框线宽度 (字符数)。
    pub box_width: usize,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            with_title: true,
            box_width: 40,
        }
    }
}

/// 从 text-based source 语法解析语义模型。
/// 语法 (每行):
///   `title: <文本>`
///   `<id>: <标签>`   → 节点 (Process)
///   `<id>:<kind>: <标签>` → 节点 (kind ∈ process|decision|terminator|data|subprocess)
///   `<from> -> <to>` 或 `<from> -> <to> [: <label>]` → 边
pub fn parse_diagram(source: &str) -> DiagramModel {
    let vtype = VisualType::classify(source);
    let mut model = DiagramModel::new(vtype);
    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("title:") {
            model.title = Some(rest.trim().to_string());
            continue;
        }
        // 边: `a -> b` / `a -> b: label`
        if let Some(arrow) = line.find("->") {
            let from = line[..arrow].trim().to_string();
            let after = line[arrow + 2..].trim();
            let (to, label) = match after.find(':') {
                Some(idx) => (after[..idx].trim().to_string(), Some(after[idx + 1..].trim().to_string())),
                None => (after.to_string(), None),
            };
            if !from.is_empty() && !to.is_empty() {
                model.add_edge(DiagramEdge { from, to, label });
            }
            continue;
        }
        // 节点: `id[:kind]: label`
        let (id, rest) = match line.find(':') {
            Some(idx) => (line[..idx].trim().to_string(), line[idx + 1..].trim()),
            None => {
                // 裸标签 → 以标签为 id
                model.add_node(DiagramNode {
                    id: line.to_string(),
                    label: line.to_string(),
                    kind: NodeKind::Process,
                });
                continue;
            }
        };
        let (kind, label) = match rest.find(':') {
            Some(idx) => {
                let kind = match rest[..idx].trim() {
                    "decision" => NodeKind::Decision,
                    "terminator" => NodeKind::Terminator,
                    "data" => NodeKind::Data,
                    "subprocess" => NodeKind::Subprocess,
                    _ => NodeKind::Process,
                };
                (kind, rest[idx + 1..].trim().to_string())
            }
            None => (NodeKind::Process, rest.to_string()),
        };
        model.add_node(DiagramNode { id, label, kind });
    }
    model
}

/// 渲染主入口: 解析 → ASCII 框线渲染。失败 (空图) 返回 None。
pub fn render_diagram(source: &str) -> Option<String> {
    let model = parse_diagram(source);
    if model.nodes.is_empty() && model.edges.is_empty() {
        return None;
    }
    Some(render_ascii(&model, &RenderOptions::default()))
}

/// Box-drawing ASCII 渲染 — 节点框 + 边 (带标签)。
pub fn render_ascii(model: &DiagramModel, opts: &RenderOptions) -> String {
    let mut out = String::new();
    if opts.with_title {
        if let Some(title) = &model.title {
            out.push_str(&format!("# {title} [{}]\n", model.vtype.name()));
        } else {
            out.push_str(&format!("# diagram [{}]\n", model.vtype.name()));
        }
    }
    // 画布宽度 = max(框线宽度, 内容)。简化: 每节点单行框。
    let width = opts.box_width.max(8);
    let render_box = |label: &str, kind: NodeKind| -> String {
        let (l, r) = kind.shape();
        let label = if label.is_empty() { "(?)" } else { label };
        let inner = width.saturating_sub(2).max(label.len());
        let pad = inner.saturating_sub(label.chars().count());
        let left_pad = pad / 2;
        let right_pad = pad - left_pad;
        format!(
            "{l}{0}{1}{2}{r}",
            " ".repeat(left_pad),
            label,
            " ".repeat(right_pad),
        )
    };
    let top = format!("{}{}{}", "┌", "─".repeat(width.saturating_sub(2)), "┐");
    let bottom = format!("{}{}{}", "└", "─".repeat(width.saturating_sub(2)), "┘");
    for node in &model.nodes {
        out.push_str(&top);
        out.push('\n');
        out.push_str(&render_box(&node.label, node.kind));
        out.push('\n');
        out.push_str(&bottom);
        out.push('\n');
    }
    for edge in &model.edges {
        let from = model.node(&edge.from).map(|n| n.id.clone()).unwrap_or_else(|| edge.from.clone());
        let to = model.node(&edge.to).map(|n| n.id.clone()).unwrap_or_else(|| edge.to.clone());
        if let Some(label) = &edge.label {
            out.push_str(&format!("{from} ──({label})──▶ {to}\n"));
        } else {
            out.push_str(&format!("{from} ──────▶ {to}\n"));
        }
    }
    out
}

/// 从语义模型生成 Mermaid 文本 (flowchart)。
pub fn to_mermaid(model: &DiagramModel) -> String {
    let mut out = String::new();
    out.push_str("flowchart TD\n");
    for node in &model.nodes {
        let shape = match node.kind {
            NodeKind::Process => format!("[{}]", node.label),
            NodeKind::Decision => format!("{{{}}}", node.label),
            NodeKind::Terminator => format!("(({}))", node.label),
            NodeKind::Data => format!("[{}]", node.label),
            NodeKind::Subprocess => format!("[[{}]]", node.label),
        };
        out.push_str(&format!("    {} {shape}\n", node.id));
    }
    for edge in &model.edges {
        let label = edge.label.as_ref().map(|l| format!("|{l}|")).unwrap_or_default();
        out.push_str(&format!("    {} --{label}--> {}\n", edge.from, edge.to));
    }
    out
}

/// 渲染为 Mermaid 文本的便捷入口 (KB 落盘 / CLI 展示)。
pub fn render_mermaid(source: &str) -> Option<String> {
    let model = parse_diagram(source);
    if model.nodes.is_empty() && model.edges.is_empty() {
        return None;
    }
    Some(to_mermaid(&model))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── G17 Diagram rendering ───────────────────────────────────────────

    #[test]
    fn visual_type_classify_flowchart_and_sequence() {
        assert_eq!(VisualType::classify("flowchart TD\nA -> B"), VisualType::Flowchart);
        assert_eq!(VisualType::classify("sequenceDiagram\nA->>B: hi"), VisualType::Sequence);
    }

    #[test]
    fn visual_type_classify_chart_family() {
        assert_eq!(VisualType::classify("gantt\n title"), VisualType::Gantt);
        assert_eq!(VisualType::classify("pie chart 标题"), VisualType::Pie);
        assert_eq!(VisualType::classify("radar chart"), VisualType::Radar);
        assert_eq!(VisualType::classify("network topology"), VisualType::Network);
        assert_eq!(VisualType::classify("something unknown here"), VisualType::Other);
    }

    #[test]
    fn all_types_have_unique_names() {
        let mut names = std::collections::HashSet::new();
        for t in VisualType::ALL {
            assert!(names.insert(t.name()), "duplicate name for {t:?}");
        }
        assert_eq!(names.len(), 27);
    }

    #[test]
    fn parse_diagram_nodes_edges_title() {
        let src = "title: 支付流程\n\
                   A: 下单\n\
                   B: 支付\n\
                   C:decision: 校验通过?\n\
                   A -> B: 去支付\n\
                   B -> C\n";
        let m = parse_diagram(src);
        assert_eq!(m.title.as_deref(), Some("支付流程"));
        assert_eq!(m.nodes.len(), 3);
        assert_eq!(m.edges.len(), 2);
        assert_eq!(m.node("C").unwrap().kind, NodeKind::Decision);
        assert_eq!(m.edges[0].label.as_deref(), Some("去支付"));
    }

    #[test]
    fn render_diagram_produces_box_drawing() {
        let src = "title: demo\nA: 起点\nB: 终点\nA -> B\n";
        let out = render_diagram(src).expect("renders");
        assert!(out.contains("┌"));
        assert!(out.contains("┐"));
        assert!(out.contains("└"));
        assert!(out.contains("▶"));
        assert!(out.contains("# demo [flowchart]"));
    }

    #[test]
    fn render_diagram_empty_returns_none() {
        assert!(render_diagram("").is_none());
        assert!(render_diagram("# just a comment").is_none());
    }

    #[test]
    fn render_mermaid_roundtrip() {
        let src = "title: flow\nA: 开始\nB: 处理\nA -> B: 数据\n";
        let mermaid = render_mermaid(src).expect("mermaid");
        assert!(mermaid.starts_with("flowchart TD"));
        assert!(mermaid.contains("A --|数据|--> B"));
        assert!(mermaid.contains("B [处理]"));
    }

    #[test]
    fn node_kind_shapes_distinct() {
        assert_ne!(NodeKind::Process.shape(), NodeKind::Decision.shape());
        assert_ne!(NodeKind::Data.shape(), NodeKind::Terminator.shape());
    }
}
