//! # NT-IO multimodal_transform — 多模态→文本降维预处理阶段
//!
//! 吸收源: hqman/pi-deepseek-vision — 在 AgentLoop 进入 text-only 目标模型前，
//! 将消息中的 ImageContent 替换为"编号标记 + 纯文本分析"，目标模型始终不接触
//! 图像。vision 模型只做感知/分析，推理模型保持 text-only。
//!
//! 骨架阶段 (C0): 图片标记检测/替换 + 可插拔 VisionAnalyzer 已接 AgentLoop
//! 生产路径; 待完善: 真 vision 后端接入 / toolResult 图片批量变换 / 顺序保真
//! 与多图批处理。
//!
//! # Diagram/Chart Rendering (G17) — 图表渲染吸收
//!
//! 吸收源: pretty-mermaid-skills + diagram-design — Mermaid→ASCII 渲染、
//! 27 种视觉类型分类、语义/布局解耦。KB-落盘/CLI 显示路径的可读化产出:
//! `render_diagram(source)` 入口 (text-based source 语法)。
//! 零外部渲染依赖: 结构化 `DiagramModel` + box-drawing ASCII 渲染 + Mermaid 文本生成。
//! 语义模型 (节点/边/类型) 与布局表现 (ASCII 框线 / Mermaid 文本) 分离。

use std::collections::HashMap;
use std::fmt;

/// 视觉分析器契约 — 骨架仅提供占位实现，真实后端 (本地视觉模型/多模态 API)
/// 实现此 trait 后注入。
pub trait VisionAnalyzer: Send + Sync {
    /// 对单张图片产出纯文本分析。
    fn analyze(&self, image_id: usize, marker: &str) -> String;
    /// 意图感知分析 (吸收自 Anionex/agent-vision-toolkit): 把当前任务意图
    /// 传给视觉层, 使其产出贴合当前目标的观察 (而非通用描述)。默认委托
    /// [`analyze`](VisionAnalyzer::analyze), 后端可选择覆盖以利用意图。
    fn analyze_with_intent(&self, image_id: usize, marker: &str, _intent: &str) -> String {
        self.analyze(image_id, marker)
    }
    /// 后端是否可用 (不可用则原样透传图片标记，避免丢信息)。
    fn is_available(&self) -> bool {
        true
    }
}

/// 骨架占位分析器 — 用图片元数据生成占位文本，供管线联通测试。
pub struct PlaceholderAnalyzer {
    pub prefix: String,
}

impl Default for PlaceholderAnalyzer {
    fn default() -> Self {
        Self {
            prefix: "image".to_string(),
        }
    }
}

impl VisionAnalyzer for PlaceholderAnalyzer {
    fn analyze(&self, image_id: usize, marker: &str) -> String {
        format!("[{} #{image_id}: {}]", self.prefix, marker)
    }
}

/// 变换配置。
#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub enabled: bool,
    /// 目标模型列表 (空 = 全部模型均 text-only 处理)。
    pub target_models: Vec<String>,
    /// 变换后标记前缀。
    pub marker_prefix: String,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target_models: vec![
                "deepseek-v4-flash".to_string(),
                "deepseek-v4-pro".to_string(),
            ],
            marker_prefix: "IMG".to_string(),
        }
    }
}

/// 单条消息变换结果。
#[derive(Debug, Clone)]
pub struct Transformed {
    pub text: String,
    /// 检出并替换的图片数。
    pub images_replaced: usize,
    /// 未替换的图片标记 (analyzer 不可用时)。
    pub images_passthrough: usize,
}

impl fmt::Display for Transformed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (replaced={}, passthrough={})",
            self.text, self.images_replaced, self.images_passthrough
        )
    }
}

/// 多模态预处理阶段。
pub struct MultimodalTransform {
    pub config: TransformConfig,
    analyzer: Box<dyn VisionAnalyzer>,
}

impl MultimodalTransform {
    pub fn new(config: TransformConfig, analyzer: Box<dyn VisionAnalyzer>) -> Self {
        Self { config, analyzer }
    }

    pub fn with_placeholder(config: TransformConfig) -> Self {
        Self::new(config, Box::new(PlaceholderAnalyzer::default()))
    }

    /// 检测目标模型是否应做 text-only 变换。
    pub fn applies_to(&self, model: &str) -> bool {
        if !self.config.enabled {
            return false;
        }
        if self.config.target_models.is_empty() {
            return true;
        }
        self.config.target_models.iter().any(|m| m == model)
    }

    /// 变换单条消息文本: 图片标记 → `[PREFIX_n: analysis]`。
    pub fn transform(&self, content: &str) -> Transformed {
        self.transform_with_intent(content, "")
    }

    /// 意图感知变换 (modlens + agent-vision-toolkit 吸收):
    /// 与 [`transform`](MultimodalTransform::transform) 相同, 但把当前任务意图
    /// 传入视觉分析器, 使图片分析贴合当前目标。意图为空时退化为通用分析。
    pub fn transform_with_intent(&self, content: &str, intent: &str) -> Transformed {
        if !self.config.enabled {
            return Transformed {
                text: content.to_string(),
                images_replaced: 0,
                images_passthrough: 0,
            };
        }
        let mut replaced = 0usize;
        let mut passthrough = 0usize;
        let available = self.analyzer.is_available();
        // 骨架: 匹配 markdown 图片 `![...](...)` 与显式 `[Image: n]`。
        let mut out = String::with_capacity(content.len());
        let mut rest = content;
        loop {
            let start_md = rest.find("![");
            let start_explicit = rest.find("[Image:");
            let start = match (start_md, start_explicit) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            match start {
                None => {
                    out.push_str(rest);
                    break;
                }
                Some(idx) => {
                    out.push_str(&rest[..idx]);
                    let (marker, consumed) = if rest[idx..].starts_with("![") {
                        self.consume_markdown(&rest[idx..])
                    } else {
                        self.consume_explicit(&rest[idx..])
                    };
                    if available {
                        replaced += 1;
                        out.push_str(&format!(
                            "[{}: {}]",
                            self.config.marker_prefix,
                            self.analyzer.analyze_with_intent(replaced, &marker, intent)
                        ));
                    } else {
                        passthrough += 1;
                        out.push_str(&marker);
                    }
                    rest = &rest[idx + consumed..];
                }
            }
        }
        Transformed {
            text: out,
            images_replaced: replaced,
            images_passthrough: passthrough,
        }
    }

    fn consume_markdown(&self, s: &str) -> (String, usize) {
        // `![alt](url)` — 返回 (alt, 消耗字节数)。
        let after = &s[2..];
        let alt_end = after.find(']').unwrap_or(0);
        let alt = &after[..alt_end];
        let rest = &after[alt_end + 1..];
        let (url, consumed_url) = if rest.starts_with('(') {
            let end = rest.find(')').unwrap_or(rest.len());
            (&rest[1..end], end + 1)
        } else {
            ("", 0)
        };
        (format!("{alt} [{url}]"), 2 + alt_end + 1 + consumed_url)
    }

    fn consume_explicit(&self, s: &str) -> (String, usize) {
        // `[Image: n]`
        let after = &s[1..];
        let end = after.find(']').unwrap_or(after.len());
        (after[..end].to_string(), 1 + end + 1)
    }

    /// 变换用户输入 (AgentLoop 生产接线点)。
    pub fn transform_input(&self, user_input: &str) -> String {
        self.transform(user_input).text
    }

    /// 意图感知变换用户输入 (AgentLoop 生产接线点, agent-vision-toolkit 吸收)。
    pub fn transform_input_with_intent(&self, user_input: &str, intent: &str) -> String {
        self.transform_with_intent(user_input, intent).text
    }
}

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

    #[test]
    fn markdown_image_replaced_with_marker() {
        let t = MultimodalTransform::with_placeholder(TransformConfig::default());
        let r = t.transform("看这张图 ![diagram](data:image/png;base64,xxx) 分析它");
        assert_eq!(r.images_replaced, 1);
        assert!(r
            .text
            .contains("[IMG: [image #1: diagram [data:image/png;base64,xxx]]"));
        assert!(!r.text.contains("![diagram"));
    }

    #[test]
    fn explicit_marker_replaced() {
        let t = MultimodalTransform::with_placeholder(TransformConfig::default());
        let r = t.transform("[Image: 1] 请描述");
        assert_eq!(r.images_replaced, 1);
        assert!(r.text.contains("[IMG:"));
    }

    #[test]
    fn no_image_passthrough_unchanged() {
        let t = MultimodalTransform::with_placeholder(TransformConfig::default());
        let r = t.transform("纯文本，无图片");
        assert_eq!(r.images_replaced, 0);
        assert_eq!(r.text, "纯文本，无图片");
    }

    #[test]
    fn disabled_config_returns_input() {
        let mut cfg = TransformConfig::default();
        cfg.enabled = false;
        let t = MultimodalTransform::with_placeholder(cfg);
        let r = t.transform("![a](b) 保留");
        assert_eq!(r.images_replaced, 0);
        assert_eq!(r.text, "![a](b) 保留");
    }

    #[test]
    fn applies_to_target_model_only() {
        let mut cfg = TransformConfig::default();
        cfg.target_models = vec!["deepseek-v4-flash".into()];
        let t = MultimodalTransform::with_placeholder(cfg);
        assert!(t.applies_to("deepseek-v4-flash"));
        assert!(!t.applies_to("qwen2.5:7b"));
    }

    #[test]
    fn intent_forwarded_to_analyzer() {
        // 意图感知 (agent-vision-toolkit 吸收): 覆盖 analyze_with_intent 的
        // 后端应收到当前任务意图。
        struct IntentAnalyzer {
            received: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
        }
        impl VisionAnalyzer for IntentAnalyzer {
            fn analyze(&self, _id: usize, marker: &str) -> String {
                format!("base:{}", marker)
            }
            fn analyze_with_intent(&self, id: usize, marker: &str, intent: &str) -> String {
                self.received.lock().unwrap().push(intent.to_string());
                format!("intent[{}]:{}:{}", id, marker, intent)
            }
        }
        let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let t = MultimodalTransform::new(
            TransformConfig::default(),
            Box::new(IntentAnalyzer { received: received.clone() }),
        );
        let r = t.transform_with_intent("![图](url) 看这个", "提取登录按钮坐标");
        assert_eq!(r.images_replaced, 1);
        let got = received.lock().unwrap();
        assert_eq!(got[0], "提取登录按钮坐标");
        assert!(r.text.contains("intent[1]"));
    }

    #[test]
    fn intent_empty_falls_back_to_base() {
        // 默认 analyze_with_intent 委托 analyze (placeholder 不感知意图)。
        let t = MultimodalTransform::with_placeholder(TransformConfig::default());
        let r = t.transform_with_intent("![a](b)", "");
        assert_eq!(r.images_replaced, 1);
    }

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

// ────────────────────────────────────────────────────────────────────────────
// P18 vision_preprocess — VLM 多图联合分析预处理 (Plugin-Deepseek-Vision 吸收)
// 将多图 payload 替换为联合分析文本; 幂等缓存 + fail-closed (不静默降级)。
// ────────────────────────────────────────────────────────────────────────────

/// 图像引用 — `hash` 由 id+size_bytes 经 djb2 确定性计算。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageRef {
    pub id: String,
    pub size_bytes: usize,
    pub hash: String,
}

/// djb2 确定性哈希 (id 字节 + size_bytes 小端字节) → 8 位 hex。
pub fn djb2_hash(id: &str, size_bytes: usize) -> String {
    let mut h: u32 = 5381;
    for b in id.bytes() {
        h = h.wrapping_mul(33).wrapping_add(u32::from(b));
    }
    for b in size_bytes.to_le_bytes() {
        h = h.wrapping_mul(33).wrapping_add(u32::from(b));
    }
    format!("{h:08x}")
}

impl ImageRef {
    pub fn new(id: impl Into<String>, size_bytes: usize) -> Self {
        let id = id.into();
        let hash = djb2_hash(&id, size_bytes);
        Self { id, size_bytes, hash }
    }
}

/// 多图输入 — 同批图片 + 可选说明提示。
#[derive(Debug, Clone)]
pub struct VisionInput {
    pub images: Vec<ImageRef>,
    pub caption_hint: Option<String>,
}

/// 预处理输出文本。
#[derive(Debug, Clone)]
pub struct VisionText {
    pub text: String,
    pub image_count: usize,
    pub cache_hit: bool,
}

/// hash 集确定性 key (排序后 join, 前缀数量防碰撞)。
fn hash_set_key(hashes: &[String]) -> String {
    let mut sorted: Vec<&String> = hashes.iter().collect();
    sorted.sort();
    let joined = sorted.iter().map(|h| h.as_str()).collect::<Vec<_>>().join(",");
    format!("{}:{}", sorted.len(), joined)
}

/// 幂等缓存: hash 集 → 联合分析文本。同 hash 集不重复调用 analyze。
#[derive(Debug, Clone, Default)]
pub struct VisionCache {
    entries: HashMap<String, String>,
}

impl VisionCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_cached(&self, hashes: &[String]) -> bool {
        self.entries.contains_key(&hash_set_key(hashes))
    }

    pub fn cache_len(&self) -> usize {
        self.entries.len()
    }
}

/// VLM 多图联合分析预处理 (fail-closed)。
pub struct VisionPreprocessor {
    pub cache: VisionCache,
    invocations: usize,
}

impl Default for VisionPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl VisionPreprocessor {
    pub fn new() -> Self {
        Self {
            cache: VisionCache::new(),
            invocations: 0,
        }
    }

    /// analyze 实际被调用的次数 (幂等性证明)。
    pub fn invocations(&self) -> usize {
        self.invocations
    }

    /// fail-closed: 空图集或缺 hash → Err, 绝不静默降级。
    fn fail_closed(reason: impl Into<String>) -> Result<VisionText, String> {
        Err(format!("vision preprocess fail-closed: {}", reason.into()))
    }

    /// 联合分析: 每唯一 hash 集执行一次 analyze; 命中缓存直接返回缓存文本。
    pub fn preprocess(
        &mut self,
        input: VisionInput,
        analyze: fn(&[ImageRef]) -> String,
    ) -> Result<VisionText, String> {
        if input.images.is_empty() {
            return Self::fail_closed("empty image set");
        }
        for img in &input.images {
            if img.hash.is_empty() {
                return Self::fail_closed(format!("image '{}' missing content hash", img.id));
            }
        }
        let hashes: Vec<String> = input.images.iter().map(|i| i.hash.clone()).collect();
        let key = hash_set_key(&hashes);
        if let Some(cached) = self.cache.entries.get(&key) {
            return Ok(VisionText {
                text: cached.clone(),
                image_count: input.images.len(),
                cache_hit: true,
            });
        }
        let analysis = analyze(&input.images);
        self.invocations += 1;
        let mut text = format!("[joint-analysis] {analysis}");
        if let Some(hint) = &input.caption_hint {
            text.push_str(&format!("\n[caption] {hint}"));
        }
        let result = VisionText {
            text: text.clone(),
            image_count: input.images.len(),
            cache_hit: false,
        };
        self.cache.entries.insert(key, text);
        Ok(result)
    }
}

/// SelfTest (T1): "nt_io_multimodal_vision_preprocess" — 幂等 + fail-closed 自检。
impl crate::core::nt_core_self_test::SelfTest for VisionPreprocessor {
    fn name(&self) -> &str {
        "nt_io_multimodal_vision_preprocess"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut p = VisionPreprocessor::new();
        let join: fn(&[ImageRef]) -> String = |imgs| {
            imgs.iter().map(|i| i.id.clone()).collect::<Vec<_>>().join("+")
        };
        let input = VisionInput {
            images: vec![ImageRef::new("a.png", 100)],
            caption_hint: None,
        };
        match p.preprocess(input.clone(), join) {
            Ok(t) => {
                if t.image_count != 1 || t.cache_hit {
                    failures.push("first preprocess must be a fresh joint-analysis".into());
                }
            }
            Err(e) => failures.push(format!("preprocess failed: {e}")),
        }
        if let Err(e) = p.preprocess(input, join) {
            failures.push(format!("second preprocess failed: {e}"));
        }
        if p.invocations() != 1 {
            failures.push("joint analysis invoked more than once for same hash set".into());
        }
        let empty = VisionInput {
            images: vec![],
            caption_hint: None,
        };
        if p.preprocess(empty, join).is_ok() {
            failures.push("empty image set must fail closed".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// P19 cpu_tts — CPU TTS 流水线 (pocket-tts 吸收: 快速语音加载)
// 纯确定性管线, 无真实音频 IO / 无 tokio。
// ────────────────────────────────────────────────────────────────────────────

/// 语音状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceState {
    pub name: String,
    pub model_path: String,
    pub sample_rate: u32,
    pub load_ms: u64,
}

/// TTS 错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TtsError {
    UnknownVoice(String),
    EmptyText,
}

impl fmt::Display for TtsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TtsError::UnknownVoice(v) => write!(f, "unknown voice: {v}"),
            TtsError::EmptyText => write!(f, "empty text"),
        }
    }
}

/// 语音加载器 — 预加载语音即时返回 (load_ms=0); 其余按 cost map 模拟
/// safetensors 秒级加载。已加载语音入缓存, 后续加载即时。
pub struct VoiceLoader {
    preloaded: Vec<VoiceState>,
    loading_cost_ms: HashMap<String, u64>,
}

impl VoiceLoader {
    pub fn new(preloaded: Vec<VoiceState>, loading_cost_ms: HashMap<String, u64>) -> Self {
        Self {
            preloaded,
            loading_cost_ms,
        }
    }

    pub fn empty() -> Self {
        Self {
            preloaded: Vec::new(),
            loading_cost_ms: HashMap::new(),
        }
    }

    pub fn cache_len(&self) -> usize {
        self.preloaded.len()
    }

    pub fn load(&mut self, name: &str) -> Result<VoiceState, TtsError> {
        if let Some(v) = self.preloaded.iter().find(|v| v.name == name) {
            let mut v = v.clone();
            v.load_ms = 0; // 已驻留内存 → 加载即时
            return Ok(v);
        }
        if let Some(cost) = self.loading_cost_ms.get(name) {
            let v = VoiceState {
                name: name.to_string(),
                model_path: format!("models/{name}.safetensors"),
                sample_rate: 24_000,
                load_ms: *cost,
            };
            self.preloaded.push(v.clone());
            return Ok(v);
        }
        Err(TtsError::UnknownVoice(name.to_string()))
    }
}

/// TTS 请求。
#[derive(Debug, Clone)]
pub struct TtsRequest {
    pub text: String,
    pub voice: String,
    pub speed: f64,
}

/// TTS 合成结果 (纯确定性, 无真实音频 IO)。
#[derive(Debug, Clone)]
pub struct TtsResult {
    pub samples_len: usize,
    pub estimated_ms: u64,
    pub voice: VoiceState,
}

/// CPU TTS 引擎。
pub struct CpuTtsEngine {
    pub loader: VoiceLoader,
}

impl CpuTtsEngine {
    pub fn new(loader: VoiceLoader) -> Self {
        Self { loader }
    }

    pub fn synthesize(&mut self, req: TtsRequest) -> Result<TtsResult, TtsError> {
        if req.text.trim().is_empty() {
            return Err(TtsError::EmptyText);
        }
        let voice = self.loader.load(&req.voice)?;
        // speed-normalized factor (ms 尺度): speed=1.0 → 1000, speed=2.0 → 500。
        let factor = (1000.0 / req.speed.max(0.1)) as u64;
        let samples_len =
            ((req.text.chars().count() as u64) * (voice.sample_rate as u64) * factor / 10_000) as usize;
        let playback_ms = (samples_len as u64) * 1000 / (voice.sample_rate as u64);
        let estimated_ms = playback_ms + voice.load_ms;
        Ok(TtsResult {
            samples_len,
            estimated_ms,
            voice,
        })
    }

    /// 实时性: 合成耗时 ≤ 音频播放时长 (RTF ≤ 1.0×)。
    pub fn is_realtime(&self, result: &TtsResult) -> bool {
        let playback_ms = (result.samples_len as u64) * 1000 / (result.voice.sample_rate as u64);
        result.estimated_ms <= playback_ms
    }
}

/// SelfTest (T1): "nt_io_multimodal_cpu_tts" — 快速语音加载 + 实时性自检。
impl crate::core::nt_core_self_test::SelfTest for CpuTtsEngine {
    fn name(&self) -> &str {
        "nt_io_multimodal_cpu_tts"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let voice = VoiceState {
            name: "selftest".into(),
            model_path: "models/selftest.safetensors".into(),
            sample_rate: 24_000,
            load_ms: 0,
        };
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![voice], HashMap::new()));
        let req = TtsRequest {
            text: "selftest".into(),
            voice: "selftest".into(),
            speed: 1.0,
        };
        match engine.synthesize(req.clone()) {
            Ok(r) => {
                if !engine.is_realtime(&r) {
                    failures.push("preloaded voice must synthesize in realtime".into());
                }
                if let Err(e) = engine.synthesize(req) {
                    failures.push(format!("synthesize failed: {e}"));
                }
            }
            Err(e) => failures.push(format!("synthesize failed: {e}")),
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod multimodal_fusion_tests {
    use super::*;

    fn joint_analyzer(imgs: &[ImageRef]) -> String {
        imgs.iter().map(|i| i.id.as_str()).collect::<Vec<_>>().join("+")
    }

    fn input_for(ids: &[&str]) -> VisionInput {
        VisionInput {
            images: ids.iter().map(|id| ImageRef::new(*id, 1024)).collect(),
            caption_hint: None,
        }
    }

    fn make_voice(name: &str) -> VoiceState {
        VoiceState {
            name: name.to_string(),
            model_path: format!("models/{name}.safetensors"),
            sample_rate: 24_000,
            load_ms: 0,
        }
    }

    // ── P18 vision_preprocess ─────────────────────────────────

    #[test]
    fn vision_pre_analyze_invoked_once_per_hash_set() {
        let mut p = VisionPreprocessor::new();
        let out = p.preprocess(input_for(&["a.png", "b.png"]), joint_analyzer).expect("ok");
        assert!(!out.cache_hit);
        assert!(out.text.contains("[joint-analysis] a.png+b.png"));
        assert_eq!(p.invocations(), 1);
        assert_eq!(p.cache.cache_len(), 1);
    }

    #[test]
    fn vision_pre_cache_hit_skips_analyze() {
        let mut p = VisionPreprocessor::new();
        p.preprocess(input_for(&["a.png"]), joint_analyzer).expect("ok");
        let hash = ImageRef::new("a.png", 1024).hash;
        assert!(p.cache.is_cached(&[hash]));
        let out = p.preprocess(input_for(&["a.png"]), joint_analyzer).expect("ok");
        assert!(out.cache_hit, "same hash set must be served from cache");
        assert_eq!(p.invocations(), 1, "analyze must not be re-invoked");
        assert_eq!(p.cache.cache_len(), 1);
    }

    #[test]
    fn vision_pre_different_hash_set_reanalyzes() {
        let mut p = VisionPreprocessor::new();
        p.preprocess(input_for(&["a.png"]), joint_analyzer).expect("ok");
        p.preprocess(input_for(&["b.png"]), joint_analyzer).expect("ok");
        assert_eq!(p.invocations(), 2);
        assert_eq!(p.cache.cache_len(), 2);
    }

    #[test]
    fn vision_pre_empty_images_fail_closed() {
        let mut p = VisionPreprocessor::new();
        let err = p
            .preprocess(VisionInput { images: vec![], caption_hint: None }, joint_analyzer)
            .expect_err("empty set must fail closed");
        assert!(err.contains("fail-closed"));
        assert_eq!(p.invocations(), 0);
    }

    #[test]
    fn vision_pre_missing_hash_fail_closed() {
        let mut p = VisionPreprocessor::new();
        let input = VisionInput {
            images: vec![ImageRef {
                id: "x".into(),
                size_bytes: 1,
                hash: String::new(),
            }],
            caption_hint: None,
        };
        assert!(p.preprocess(input, joint_analyzer).is_err());
        assert_eq!(p.invocations(), 0);
    }

    #[test]
    fn vision_pre_caption_hint_included_in_text() {
        let mut p = VisionPreprocessor::new();
        let input = VisionInput {
            images: vec![ImageRef::new("a.png", 10)],
            caption_hint: Some("这是支付页".into()),
        };
        let out = p.preprocess(input, joint_analyzer).expect("ok");
        assert!(out.text.contains("[caption] 这是支付页"));
        assert_eq!(out.image_count, 1);
    }

    #[test]
    fn vision_pre_djb2_hash_deterministic() {
        assert_eq!(djb2_hash("a.png", 100), djb2_hash("a.png", 100));
        assert_ne!(djb2_hash("a.png", 100), djb2_hash("b.png", 100));
        assert_ne!(djb2_hash("a.png", 100), djb2_hash("a.png", 200));
        let img = ImageRef::new("a.png", 100);
        assert!(!img.hash.is_empty());
    }

    #[test]
    fn vision_pre_selftest_name_matches() {
        use crate::core::nt_core_self_test::SelfTest;
        let p = VisionPreprocessor::new();
        assert_eq!(p.name(), "nt_io_multimodal_vision_preprocess");
        assert!(p.self_test().is_ok());
    }

    // ── P19 cpu_tts ───────────────────────────────────────────

    #[test]
    fn cpu_tts_preloaded_voice_loads_instantly() {
        let mut loader = VoiceLoader::new(vec![make_voice("en")], std::collections::HashMap::new());
        let v = loader.load("en").expect("preloaded");
        assert_eq!(v.load_ms, 0);
        assert_eq!(loader.cache_len(), 1);
    }

    #[test]
    fn cpu_tts_cold_voice_load_cost_then_cached() {
        let mut cost = std::collections::HashMap::new();
        cost.insert("zh".to_string(), 2500);
        let mut loader = VoiceLoader::new(vec![], cost);
        let first = loader.load("zh").expect("cold load");
        assert_eq!(first.load_ms, 2500, "cold load pays safetensors cost");
        assert_eq!(loader.cache_len(), 1);
        let second = loader.load("zh").expect("cached load");
        assert_eq!(second.load_ms, 0, "cached voice loads instantly");
    }

    #[test]
    fn cpu_tts_unknown_voice_is_err() {
        let mut engine = CpuTtsEngine::new(VoiceLoader::empty());
        let req = TtsRequest {
            text: "hi".into(),
            voice: "ghost".into(),
            speed: 1.0,
        };
        assert_eq!(
            engine.synthesize(req).expect_err("must err"),
            TtsError::UnknownVoice("ghost".into())
        );
    }

    #[test]
    fn cpu_tts_empty_text_is_err() {
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![make_voice("en")], std::collections::HashMap::new()));
        let req = TtsRequest {
            text: String::new(),
            voice: "en".into(),
            speed: 1.0,
        };
        assert_eq!(engine.synthesize(req).expect_err("must err"), TtsError::EmptyText);
    }

    #[test]
    fn cpu_tts_samples_determinism() {
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![make_voice("en")], std::collections::HashMap::new()));
        let req = TtsRequest {
            text: "hello world".into(),
            voice: "en".into(),
            speed: 1.0,
        };
        let a = engine.synthesize(req.clone()).expect("ok");
        let b = engine.synthesize(req).expect("ok");
        assert_eq!(a.samples_len, b.samples_len, "samples must be deterministic");
        // chars(11) × 24000 × 1000 / 10000 = 26400
        assert_eq!(a.samples_len, 11 * 24_000 * 1000 / 10_000);
    }

    #[test]
    fn cpu_tts_speed_scales_samples() {
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![make_voice("en")], std::collections::HashMap::new()));
        let normal = engine
            .synthesize(TtsRequest { text: "hello world".into(), voice: "en".into(), speed: 1.0 })
            .expect("ok");
        let fast = engine
            .synthesize(TtsRequest { text: "hello world".into(), voice: "en".into(), speed: 2.0 })
            .expect("ok");
        assert_eq!(fast.samples_len, normal.samples_len / 2);
    }

    #[test]
    fn cpu_tts_realtime_check() {
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![make_voice("en")], std::collections::HashMap::new()));
        let r = engine
            .synthesize(TtsRequest { text: "hello world".into(), voice: "en".into(), speed: 1.0 })
            .expect("ok");
        assert!(engine.is_realtime(&r), "preloaded voice must be realtime");
    }

    #[test]
    fn cpu_tts_cold_load_not_realtime_then_realtime() {
        let mut cost = std::collections::HashMap::new();
        cost.insert("slow".to_string(), 5000);
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![], cost));
        let req = TtsRequest {
            text: "hello".into(),
            voice: "slow".into(),
            speed: 1.0,
        };
        let first = engine.synthesize(req.clone()).expect("ok");
        assert!(!engine.is_realtime(&first), "cold-load cost must exceed playback");
        let second = engine.synthesize(req).expect("ok");
        assert!(engine.is_realtime(&second), "cached voice becomes realtime");
    }

    #[test]
    fn cpu_tts_selftest_name_matches() {
        use crate::core::nt_core_self_test::SelfTest;
        let e = CpuTtsEngine::new(VoiceLoader::empty());
        assert_eq!(e.name(), "nt_io_multimodal_cpu_tts");
        assert!(e.self_test().is_ok());
    }
}

// ────────────────────────────────────────────────────────────────────────────
// P13 unified_face — 人脸分析统一 API (uniface 吸收: detection/recognition/
// tracking/gaze 四任务统一接口)。确定性骨架实现, 无真实视觉后端。
// ────────────────────────────────────────────────────────────────────────────

/// 人脸分析四任务。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaceTask {
    Detection,
    Recognition,
    Tracking,
    Gaze,
}

impl FaceTask {
    pub fn label(self) -> &'static str {
        match self {
            FaceTask::Detection => "detection",
            FaceTask::Recognition => "recognition",
            FaceTask::Tracking => "tracking",
            FaceTask::Gaze => "gaze",
        }
    }
}

/// 单张人脸框 (归一化坐标 [0,1] 内)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaceBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub confidence: f64,
}

/// 人脸分析结果 — 四任务共享统一返回。
#[derive(Debug, Clone)]
pub struct FaceResult {
    pub task: FaceTask,
    pub boxes: Vec<FaceBox>,
    pub identity: Option<String>,
    pub gaze_vector: Option<(f64, f64, f64)>,
}

/// 人脸分析统一引擎 (确定性骨架)。
pub struct UnifiedFace {
    pub max_detections: usize,
    pub min_confidence: f64,
}

impl Default for UnifiedFace {
    fn default() -> Self {
        Self {
            max_detections: 10,
            min_confidence: 0.5,
        }
    }
}

impl UnifiedFace {
    pub fn new(max_detections: usize, min_confidence: f64) -> Self {
        Self {
            max_detections,
            min_confidence,
        }
    }

    /// detection: 确定性伪随机坐标 (i*31%1000/1000 派生), 置信度 =
    /// min_confidence + (i%5)/10, 低于阈值者过滤。
    pub fn detect(&self, face_count: usize) -> FaceResult {
        let count = face_count.min(self.max_detections);
        let mut boxes = Vec::with_capacity(count);
        for i in 0..count {
            let confidence =
                (self.min_confidence + (i % 5) as f64 / 10.0).max(0.0).min(1.0);
            if confidence < self.min_confidence {
                continue;
            }
            boxes.push(FaceBox {
                x: (i.wrapping_mul(31) % 1000) as f64 / 1000.0,
                y: (i.wrapping_mul(131) % 1000) as f64 / 1000.0,
                w: (i.wrapping_mul(47) % 1000) as f64 / 1000.0,
                h: (i.wrapping_mul(17) % 1000) as f64 / 1000.0,
                confidence,
            });
        }
        FaceResult {
            task: FaceTask::Detection,
            boxes,
            identity: None,
            gaze_vector: None,
        }
    }

    /// recognition: 1 个框 + 身份。
    pub fn recognize(&self, identity: &str) -> FaceResult {
        FaceResult {
            task: FaceTask::Recognition,
            boxes: vec![FaceBox {
                x: 0.0,
                y: 0.0,
                w: 0.5,
                h: 0.5,
                confidence: self.min_confidence.max(0.0).min(1.0),
            }],
            identity: Some(identity.to_string()),
            gaze_vector: None,
        }
    }

    /// tracking: 每帧 detect, 身份无关 (identity=None), 返回帧序列。
    pub fn track(&self, frames: usize) -> Vec<FaceResult> {
        let mut out = Vec::with_capacity(frames);
        for _ in 0..frames {
            let mut r = self.detect(self.max_detections);
            r.task = FaceTask::Tracking;
            r.identity = None;
            out.push(r);
        }
        out
    }

    /// gaze: 注视向量。
    pub fn gaze(&self) -> FaceResult {
        FaceResult {
            task: FaceTask::Gaze,
            boxes: Vec::new(),
            identity: None,
            gaze_vector: Some((0.0, 0.5, 1.0)),
        }
    }

    /// 统一分派入口。Detection/Tracking 的 arg 解析失败用默认 3。
    pub fn run(&self, task: FaceTask, arg: &str) -> FaceResult {
        match task {
            FaceTask::Detection => self.detect(arg.parse::<usize>().unwrap_or(3)),
            FaceTask::Recognition => self.recognize(arg),
            FaceTask::Tracking => {
                let frames = self.track(arg.parse::<usize>().unwrap_or(3));
                frames.into_iter().next_back().unwrap_or_else(|| FaceResult {
                    task: FaceTask::Tracking,
                    boxes: Vec::new(),
                    identity: None,
                    gaze_vector: None,
                })
            }
            FaceTask::Gaze => self.gaze(),
        }
    }
}

/// SelfTest (T1): "nt_io_multimodal_unified_face" — 四任务统一接口自检。
impl crate::core::nt_core_self_test::SelfTest for UnifiedFace {
    fn name(&self) -> &str {
        "nt_io_multimodal_unified_face"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let f = UnifiedFace::default();
        if f.detect(3).boxes.len() != 3 {
            failures.push("detect must yield face_count boxes".into());
        }
        if f.recognize("alice").identity.as_deref() != Some("alice") {
            failures.push("recognize must carry identity".into());
        }
        if f.track(2).len() != 2 {
            failures.push("track must yield frame count".into());
        }
        if f.gaze().gaze_vector != Some((0.0, 0.5, 1.0)) {
            failures.push("gaze must carry gaze vector".into());
        }
        if f.run(FaceTask::Gaze, "").gaze_vector.is_none() {
            failures.push("run must dispatch gaze".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod unified_face_tests {
    use super::*;

    #[test]
    fn unified_face_detect_filters_low_confidence() {
        let f = UnifiedFace::new(10, 1.5);
        let r = f.detect(10);
        assert!(r.boxes.is_empty(), "below-threshold boxes must be filtered");
        let d = UnifiedFace::default();
        let ok = d.detect(5);
        assert_eq!(ok.boxes.len(), 5, "default threshold retains all boxes");
        assert!(ok.boxes.iter().all(|b| b.confidence >= d.min_confidence));
    }

    #[test]
    fn unified_face_detect_respects_max_detections() {
        let f = UnifiedFace::new(3, 0.5);
        assert_eq!(f.detect(10).boxes.len(), 3, "capped by max_detections");
        assert_eq!(f.detect(2).boxes.len(), 2);
    }

    #[test]
    fn unified_face_recognize_carries_identity() {
        let f = UnifiedFace::new(10, 0.5);
        let r = f.recognize("alice");
        assert_eq!(r.task, FaceTask::Recognition);
        assert_eq!(r.boxes.len(), 1);
        assert_eq!(r.identity.as_deref(), Some("alice"));
        assert!(r.boxes[0].confidence >= f.min_confidence);
    }

    #[test]
    fn unified_face_track_returns_frame_count() {
        let f = UnifiedFace::new(10, 0.5);
        let frames = f.track(4);
        assert_eq!(frames.len(), 4);
        for fr in &frames {
            assert_eq!(fr.task, FaceTask::Tracking);
            assert!(fr.identity.is_none());
            assert!(!fr.boxes.is_empty());
        }
    }

    #[test]
    fn unified_face_gaze_carries_vector() {
        let f = UnifiedFace::default();
        let r = f.gaze();
        assert_eq!(r.task, FaceTask::Gaze);
        assert_eq!(r.gaze_vector, Some((0.0, 0.5, 1.0)));
    }

    #[test]
    fn unified_face_run_dispatches_all_tasks() {
        let f = UnifiedFace::default();
        assert_eq!(f.run(FaceTask::Detection, "5").boxes.len(), 5);
        assert_eq!(
            f.run(FaceTask::Detection, "bad").boxes.len(),
            3,
            "parse failure falls back to default"
        );
        assert_eq!(
            f.run(FaceTask::Recognition, "bob").identity.as_deref(),
            Some("bob")
        );
        assert_eq!(
            f.run(FaceTask::Tracking, "2").boxes.len(),
            10,
            "last frame of 2-frame track"
        );
        assert_eq!(
            f.run(FaceTask::Tracking, "oops").boxes.len(),
            10,
            "tracking parse fallback to 3 frames"
        );
        assert_eq!(
            f.run(FaceTask::Gaze, "x").gaze_vector,
            Some((0.0, 0.5, 1.0))
        );
    }

    #[test]
    fn unified_face_labels_distinct() {
        let labels: Vec<&str> = [
            FaceTask::Detection,
            FaceTask::Recognition,
            FaceTask::Tracking,
            FaceTask::Gaze,
        ]
        .iter()
        .map(|t| t.label())
        .collect();
        let mut set = std::collections::HashSet::new();
        for l in &labels {
            assert!(set.insert(*l), "duplicate label {l}");
        }
        assert_eq!(labels.len(), 4);
        assert_eq!(FaceTask::Detection.label(), "detection");
    }

    #[test]
    fn unified_face_selftest_name_matches() {
        use crate::core::nt_core_self_test::SelfTest;
        let f = UnifiedFace::default();
        assert_eq!(f.name(), "nt_io_multimodal_unified_face");
        assert!(f.self_test().is_ok());
    }
}
