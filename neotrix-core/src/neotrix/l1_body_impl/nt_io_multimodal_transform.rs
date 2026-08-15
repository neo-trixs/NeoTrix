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
