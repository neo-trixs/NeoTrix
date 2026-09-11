//! 自进化执行结构图 (Procedural Graph)
//!
//! 基于 arXiv 2609.09153 (Procedural Graphs) 的技能执行图框架。
//! 技能节点构成有向图，边表示执行依赖关系。
//! LLM refiner 可根据成功/失败轨迹编辑图拓扑，
//! 被拒绝的编辑保留为反模式，防止重复犯错。

use std::collections::HashMap;

/// 技能执行节点
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkillNode {
    /// 唯一标识符
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能类型：Tool / LLM / Hybrid
    pub skill_type: SkillType,
    /// 输入 JSON schema
    pub input_schema: String,
    /// 输出 JSON schema
    pub output_schema: String,
    /// 累计成功次数
    pub success_count: u32,
    /// 累计失败次数
    pub failure_count: u32,
}

/// 技能类型枚举
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillType {
    /// 纯工具调用
    Tool,
    /// 纯 LLM 推理
    LLM,
    /// 工具 + LLM 混合
    Hybrid,
}

/// 执行依赖边
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExecutionEdge {
    /// 源技能 ID
    pub from: String,
    /// 目标技能 ID
    pub to: String,
    /// 边类型
    pub edge_type: EdgeType,
}

/// 边类型枚举
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeType {
    /// 必须先执行的前置依赖
    Prerequisite,
    /// 可选前置
    Optional,
    /// 失败时的替代路径
    Fallback,
}

/// 执行轨迹 — 记录一次完整的技能图执行
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    /// 轨迹唯一 ID
    pub trace_id: String,
    /// 执行步骤序列
    pub steps: Vec<TraceStep>,
    /// 整体是否成功
    pub success: bool,
    /// 总消耗 token 数
    pub total_tokens: u32,
    /// 总延迟（毫秒）
    pub total_latency_ms: u64,
}

/// 单个执行步骤
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TraceStep {
    /// 执行的技能 ID
    pub skill_id: String,
    /// 输入 token 数
    pub input_tokens: u32,
    /// 输出 token 数
    pub output_tokens: u32,
    /// 延迟（毫秒）
    pub latency_ms: u64,
    /// 是否成功
    pub success: bool,
    /// 错误信息（成功时为 None）
    pub error: Option<String>,
}

/// 图拓扑编辑操作
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum GraphEdit {
    /// 添加节点
    AddNode(SkillNode),
    /// 删除节点
    RemoveNode(String),
    /// 添加边
    AddEdge(ExecutionEdge),
    /// 删除边（from, to）
    RemoveEdge(String, String),
    /// 重连边（from, old_to, new_to）
    RewireEdge(String, String, String),
}

/// 被拒绝的编辑 — 保留为反模式
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RejectedEdit {
    /// 被拒绝的编辑
    pub edit: GraphEdit,
    /// 拒绝原因
    pub reason: String,
    /// 累计被拒绝次数
    pub rejection_count: u32,
}

/// 图统计信息
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GraphStats {
    /// 节点数
    pub node_count: usize,
    /// 边数
    pub edge_count: usize,
    /// 轨迹数
    pub trace_count: usize,
    /// 被拒绝编辑数
    pub rejected_edit_count: usize,
    /// 平均成功率
    pub avg_success_rate: f64,
}

/// 自进化执行结构图
///
/// 技能节点构成有向无环图（DAG），边表示执行依赖。
/// LLM refiner 可根据执行轨迹提议拓扑编辑，
/// 被拒绝的编辑保留为反模式，防止系统重复犯错。
///
/// # 核心流程
///
/// 1. **构建图** — `add_node` + `add_edge` 建立技能拓扑
/// 2. **执行排序** — `topological_sort` 返回合法执行顺序
/// 3. **记录轨迹** — `record_trace` 记录每次执行的详细步骤
/// 4. **提议编辑** — `propose_edit` 让 LLM refiner 提议拓扑变更
/// 5. **审核决策** — `approve_edit` / `reject_edit` 控制进化方向
/// 6. **反模式** — 被拒绝的编辑自动积累为反模式知识
#[allow(dead_code)]
pub struct ProceduralGraph {
    /// 技能节点（ID → 节点）
    nodes: HashMap<String, SkillNode>,
    /// 执行依赖边列表
    edges: Vec<ExecutionEdge>,
    /// 执行轨迹历史
    traces: Vec<ExecutionTrace>,
    /// 被拒绝的编辑（反模式）
    rejected_edits: Vec<RejectedEdit>,
    /// 下一个节点 ID 计数器
    next_node_id: u32,
}

impl ProceduralGraph {
    /// 创建空的执行结构图
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            traces: Vec::new(),
            rejected_edits: Vec::new(),
            next_node_id: 1,
        }
    }

    /// 添加技能节点，返回自动生成的节点 ID
    pub fn add_node(
        &mut self,
        name: &str,
        skill_type: SkillType,
        input_schema: &str,
        output_schema: &str,
    ) -> String {
        let id = format!("skill_{}", self.next_node_id);
        self.next_node_id += 1;
        let node = SkillNode {
            id: id.clone(),
            name: name.to_string(),
            skill_type,
            input_schema: input_schema.to_string(),
            output_schema: output_schema.to_string(),
            success_count: 0,
            failure_count: 0,
        };
        self.nodes.insert(id.clone(), node);
        id
    }

    /// 添加执行依赖边
    ///
    /// # 错误
    /// - 源或目标节点不存在
    /// - 添加后会形成循环依赖
    pub fn add_edge(&mut self, from: &str, to: &str, edge_type: EdgeType) -> Result<(), String> {
        if !self.nodes.contains_key(from) {
            return Err(format!("源节点不存在: {}", from));
        }
        if !self.nodes.contains_key(to) {
            return Err(format!("目标节点不存在: {}", to));
        }
        if from == to {
            return Err("不允许自环".to_string());
        }

        // 检查是否已存在相同的边
        let duplicate = self.edges.iter().any(|e| e.from == from && e.to == to);
        if duplicate {
            return Err(format!("边已存在: {} → {}", from, to));
        }

        let edge = ExecutionEdge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type,
        };
        self.edges.push(edge);

        // 检查是否会形成循环
        if self.has_cycle() {
            // 回滚：移除刚添加的边
            self.edges.pop();
            return Err(format!("添加该边会形成循环依赖: {} → {}", from, to));
        }

        Ok(())
    }

    /// 记录一次完整的执行轨迹
    pub fn record_trace(&mut self, trace: ExecutionTrace) {
        // 根据轨迹结果更新节点统计
        for step in &trace.steps {
            if let Some(node) = self.nodes.get_mut(&step.skill_id) {
                if step.success {
                    node.success_count += 1;
                } else {
                    node.failure_count += 1;
                }
            }
        }
        self.traces.push(trace);
    }

    /// 获取指定技能的成功率 (0.0 ~ 1.0)
    ///
    /// 无执行记录时返回 0.0
    pub fn success_rate(&self, skill_id: &str) -> f64 {
        match self.nodes.get(skill_id) {
            Some(node) => {
                let total = node.success_count + node.failure_count;
                if total == 0 {
                    0.0
                } else {
                    node.success_count as f64 / total as f64
                }
            }
            None => 0.0,
        }
    }

    /// 拓扑排序 — 返回从根节点到叶节点的合法执行顺序
    ///
    /// # 错误
    /// 图中存在循环依赖时返回错误
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        if self.has_cycle() {
            return Err("图中存在循环依赖，无法进行拓扑排序".to_string());
        }

        // 计算每个节点的入度
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for node_id in self.nodes.keys() {
            in_degree.entry(node_id.as_str()).or_insert(0);
        }
        for edge in &self.edges {
            *in_degree.entry(&edge.to).or_insert(0) += 1;
        }

        // Kahn 算法
        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();
        queue.sort(); // 确定性排序

        let mut result = Vec::new();
        while !queue.is_empty() {
            let node_id = queue.remove(0);
            result.push(node_id.to_string());

            // 找到所有从 node_id 出发的边，减少目标节点的入度
            let mut neighbors: Vec<&str> = self
                .edges
                .iter()
                .filter(|e| e.from == node_id)
                .map(|e| e.to.as_str())
                .collect();
            neighbors.sort();

            for neighbor in neighbors {
                let degree = in_degree.get_mut(neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push(neighbor);
                    queue.sort();
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err("拓扑排序失败：图中存在循环依赖".to_string());
        }

        Ok(result)
    }

    /// 提议拓扑编辑（由 LLM refiner 调用）
    ///
    /// 编辑会被暂存，需要通过 `approve_edit` 或 `reject_edit` 最终决策。
    /// 基本合法性检查（节点存在性、循环检测）在此阶段执行。
    pub fn propose_edit(&mut self, edit: GraphEdit) -> Result<(), RejectedEdit> {
        // 先提取验证信息，再用 owned edit 构造错误
        let validation = match &edit {
            GraphEdit::AddNode(node) => {
                if self.nodes.contains_key(&node.id) {
                    Err(format!("节点 {} 已存在", node.id))
                } else {
                    Ok(())
                }
            }
            GraphEdit::RemoveNode(node_id) => {
                if !self.nodes.contains_key(node_id) {
                    Err(format!("节点 {} 不存在", node_id))
                } else {
                    let has_deps = self
                        .edges
                        .iter()
                        .any(|e| &e.from == node_id || &e.to == node_id);
                    if has_deps {
                        Err(format!("节点 {} 仍有执行依赖，无法删除", node_id))
                    } else {
                        Ok(())
                    }
                }
            }
            GraphEdit::AddEdge(edge) => {
                if !self.nodes.contains_key(&edge.from) {
                    Err(format!("源节点 {} 不存在", edge.from))
                } else if !self.nodes.contains_key(&edge.to) {
                    Err(format!("目标节点 {} 不存在", edge.to))
                } else if edge.from == edge.to {
                    Err("不允许自环".to_string())
                } else {
                    // 临时添加边检查循环
                    let test_edge = ExecutionEdge {
                        from: edge.from.clone(),
                        to: edge.to.clone(),
                        edge_type: edge.edge_type.clone(),
                    };
                    self.edges.push(test_edge);
                    let cycle = self.has_cycle();
                    self.edges.pop();
                    if cycle {
                        Err(format!(
                            "添加该边会形成循环依赖: {} → {}",
                            edge.from, edge.to
                        ))
                    } else {
                        Ok(())
                    }
                }
            }
            GraphEdit::RemoveEdge(from, to) => {
                let exists = self.edges.iter().any(|e| &e.from == from && &e.to == to);
                if !exists {
                    Err(format!("边 {} → {} 不存在", from, to))
                } else {
                    Ok(())
                }
            }
            GraphEdit::RewireEdge(from, old_to, new_to) => {
                let exists = self
                    .edges
                    .iter()
                    .any(|e| &e.from == from && &e.to == old_to);
                if !exists {
                    Err(format!("边 {} → {} 不存在", from, old_to))
                } else if !self.nodes.contains_key(new_to) {
                    Err(format!("新目标节点 {} 不存在", new_to))
                } else if from == new_to {
                    Err("不允许自环".to_string())
                } else {
                    // 临时修改边检查循环
                    let idx = self
                        .edges
                        .iter()
                        .position(|e| &e.from == from && &e.to == old_to)
                        .unwrap();
                    let old_edge = self.edges[idx].clone();
                    self.edges[idx].to = new_to.clone();
                    let cycle = self.has_cycle();
                    self.edges[idx] = old_edge;
                    if cycle {
                        Err(format!("重连后会形成循环依赖: {} → {}", from, new_to))
                    } else {
                        Ok(())
                    }
                }
            }
        };

        match validation {
            Ok(()) => Ok(()),
            Err(reason) => Err(RejectedEdit {
                edit,
                reason,
                rejection_count: 1,
            }),
        }
    }

    /// 批准编辑，将变更应用到图拓扑
    pub fn approve_edit(&mut self, edit: GraphEdit) {
        match edit {
            GraphEdit::AddNode(node) => {
                self.nodes.insert(node.id.clone(), node);
            }
            GraphEdit::RemoveNode(node_id) => {
                self.nodes.remove(&node_id);
                self.edges.retain(|e| e.from != node_id && e.to != node_id);
            }
            GraphEdit::AddEdge(edge) => {
                self.edges.push(edge);
            }
            GraphEdit::RemoveEdge(from, to) => {
                self.edges.retain(|e| !(e.from == from && e.to == to));
            }
            GraphEdit::RewireEdge(from, old_to, new_to) => {
                if let Some(edge) = self
                    .edges
                    .iter_mut()
                    .find(|e| e.from == from && e.to == old_to)
                {
                    edge.to = new_to;
                }
            }
        }
    }

    /// 拒绝编辑，记录为反模式
    pub fn reject_edit(&mut self, edit: GraphEdit, reason: &str) {
        // 查找是否已有相同的反模式
        let existing = self.rejected_edits.iter_mut().find(|re| {
            std::mem::discriminant(&re.edit) == std::mem::discriminant(&edit) && re.reason == reason
        });

        if let Some(rejected) = existing {
            rejected.rejection_count += 1;
        } else {
            self.rejected_edits.push(RejectedEdit {
                edit,
                reason: reason.to_string(),
                rejection_count: 1,
            });
        }
    }

    /// 检测图中是否存在循环依赖（DFS 染色法）
    pub fn has_cycle(&self) -> bool {
        #[derive(Clone, Copy, PartialEq)]
        enum Color {
            White,
            Gray,
            Black,
        }

        let mut color: HashMap<&str, Color> = self
            .nodes
            .keys()
            .map(|id| (id.as_str(), Color::White))
            .collect();

        fn has_cycle_dfs<'a>(
            node: &'a str,
            edges: &'a [ExecutionEdge],
            color: &mut HashMap<&'a str, Color>,
        ) -> bool {
            color.insert(node, Color::Gray);

            for edge in edges.iter().filter(|e| e.from == node) {
                match color.get(edge.to.as_str()) {
                    Some(Color::Gray) => return true,
                    Some(Color::Black) => continue,
                    _ => {
                        if has_cycle_dfs(&edge.to, edges, color) {
                            return true;
                        }
                    }
                }
            }

            color.insert(node, Color::Black);
            false
        }

        for node_id in self.nodes.keys() {
            if matches!(color.get(node_id.as_str()), Some(Color::White)) {
                if has_cycle_dfs(node_id, &self.edges, &mut color) {
                    return true;
                }
            }
        }

        false
    }

    /// 获取所有反模式（被拒绝的编辑）
    pub fn anti_patterns(&self) -> &[RejectedEdit] {
        &self.rejected_edits
    }

    /// 获取图统计信息
    pub fn stats(&self) -> GraphStats {
        let total_success: u32 = self.nodes.values().map(|n| n.success_count).sum();
        let total_exec: u32 = self
            .nodes
            .values()
            .map(|n| n.success_count + n.failure_count)
            .sum();

        let avg_success_rate = if total_exec == 0 {
            0.0
        } else {
            total_success as f64 / total_exec as f64
        };

        GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            trace_count: self.traces.len(),
            rejected_edit_count: self.rejected_edits.len(),
            avg_success_rate,
        }
    }

    /// 获取节点引用
    pub fn get_node(&self, id: &str) -> Option<&SkillNode> {
        self.nodes.get(id)
    }

    /// 获取所有边的引用
    pub fn edges(&self) -> &[ExecutionEdge] {
        &self.edges
    }

    /// 获取所有轨迹的引用
    pub fn traces(&self) -> &[ExecutionTrace] {
        &self.traces
    }

    /// 获取节点总数
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 获取边总数
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// 查找所有入度为 0 的根节点（无前置依赖）
    pub fn root_nodes(&self) -> Vec<&SkillNode> {
        let has_incoming: std::collections::HashSet<&str> =
            self.edges.iter().map(|e| e.to.as_str()).collect();
        self.nodes
            .values()
            .filter(|n| !has_incoming.contains(n.id.as_str()))
            .collect()
    }

    /// 查找所有出度为 0 的叶节点（无后续依赖）
    pub fn leaf_nodes(&self) -> Vec<&SkillNode> {
        let has_outgoing: std::collections::HashSet<&str> =
            self.edges.iter().map(|e| e.from.as_str()).collect();
        self.nodes
            .values()
            .filter(|n| !has_outgoing.contains(n.id.as_str()))
            .collect()
    }

    /// 查找指定节点的所有直接后继
    pub fn successors(&self, node_id: &str) -> Vec<&SkillNode> {
        self.edges
            .iter()
            .filter(|e| e.from == node_id)
            .filter_map(|e| self.nodes.get(&e.to))
            .collect()
    }

    /// 查找指定节点的所有直接前驱
    pub fn predecessors(&self, node_id: &str) -> Vec<&SkillNode> {
        self.edges
            .iter()
            .filter(|e| e.to == node_id)
            .filter_map(|e| self.nodes.get(&e.from))
            .collect()
    }

    /// 查找指定节点的所有 Fallback 替代路径
    pub fn fallbacks(&self, node_id: &str) -> Vec<&SkillNode> {
        self.edges
            .iter()
            .filter(|e| e.from == node_id && e.edge_type == EdgeType::Fallback)
            .filter_map(|e| self.nodes.get(&e.to))
            .collect()
    }

    /// 按成功率排序的节点列表（降序）
    pub fn nodes_by_success_rate(&self) -> Vec<(&SkillNode, f64)> {
        let mut pairs: Vec<(&SkillNode, f64)> = self
            .nodes
            .values()
            .map(|n| (n, self.success_rate(&n.id)))
            .collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        pairs
    }
}

impl Default for ProceduralGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph() -> ProceduralGraph {
        let mut g = ProceduralGraph::new();
        let a = g.add_node("fetch", SkillType::Tool, "{}", "{}");
        let b = g.add_node("parse", SkillType::LLM, "{}", "{}");
        let c = g.add_node("summarize", SkillType::Hybrid, "{}", "{}");
        g.add_edge(&a, &b, EdgeType::Prerequisite).unwrap();
        g.add_edge(&b, &c, EdgeType::Prerequisite).unwrap();
        g
    }

    #[test]
    fn test_basic_graph() {
        let g = make_graph();
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn test_topological_sort() {
        let g = make_graph();
        let order = g.topological_sort().unwrap();
        assert_eq!(order.len(), 3);
        // fetch 必须在 parse 之前，parse 必须在 summarize 之前
        let fetch_idx = order
            .iter()
            .position(|id| g.get_node(id).unwrap().name == "fetch")
            .unwrap();
        let parse_idx = order
            .iter()
            .position(|id| g.get_node(id).unwrap().name == "parse")
            .unwrap();
        let summary_idx = order
            .iter()
            .position(|id| g.get_node(id).unwrap().name == "summarize")
            .unwrap();
        assert!(fetch_idx < parse_idx);
        assert!(parse_idx < summary_idx);
    }

    #[test]
    fn test_cycle_detection() {
        let mut g = ProceduralGraph::new();
        let a = g.add_node("a", SkillType::Tool, "{}", "{}");
        let b = g.add_node("b", SkillType::Tool, "{}", "{}");
        let c = g.add_node("c", SkillType::Tool, "{}", "{}");
        g.add_edge(&a, &b, EdgeType::Prerequisite).unwrap();
        g.add_edge(&b, &c, EdgeType::Prerequisite).unwrap();
        // 尝试添加形成循环的边
        let result = g.add_edge(&c, &a, EdgeType::Prerequisite);
        assert!(result.is_err());
    }

    #[test]
    fn test_self_loop_rejected() {
        let mut g = ProceduralGraph::new();
        let a = g.add_node("a", SkillType::Tool, "{}", "{}");
        let result = g.add_edge(&a, &a, EdgeType::Prerequisite);
        assert!(result.is_err());
    }

    #[test]
    fn test_record_trace_updates_stats() {
        let mut g = make_graph();
        let nodes: Vec<String> = g.topological_sort().unwrap();

        let trace = ExecutionTrace {
            trace_id: "t1".to_string(),
            steps: vec![
                TraceStep {
                    skill_id: nodes[0].clone(),
                    input_tokens: 100,
                    output_tokens: 50,
                    latency_ms: 200,
                    success: true,
                    error: None,
                },
                TraceStep {
                    skill_id: nodes[1].clone(),
                    input_tokens: 50,
                    output_tokens: 80,
                    latency_ms: 300,
                    success: false,
                    error: Some("parse error".to_string()),
                },
            ],
            success: false,
            total_tokens: 280,
            total_latency_ms: 500,
        };
        g.record_trace(trace);

        let first_id = &nodes[0];
        assert_eq!(g.get_node(first_id).unwrap().success_count, 1);
        let second_id = &nodes[1];
        assert_eq!(g.get_node(second_id).unwrap().failure_count, 1);
    }

    #[test]
    fn test_success_rate() {
        let mut g = ProceduralGraph::new();
        let id = g.add_node("test", SkillType::Tool, "{}", "{}");
        assert_eq!(g.success_rate(&id), 0.0);

        g.record_trace(ExecutionTrace {
            trace_id: "t1".to_string(),
            steps: vec![TraceStep {
                skill_id: id.clone(),
                input_tokens: 10,
                output_tokens: 10,
                latency_ms: 10,
                success: true,
                error: None,
            }],
            success: true,
            total_tokens: 20,
            total_latency_ms: 10,
        });
        assert_eq!(g.success_rate(&id), 1.0);
    }

    #[test]
    fn test_reject_edit_creates_anti_pattern() {
        let mut g = make_graph();
        let edit = GraphEdit::RemoveNode("skill_1".to_string());
        let reason = "核心节点不可删除";

        g.reject_edit(edit.clone(), reason);
        assert_eq!(g.anti_patterns().len(), 1);
        assert_eq!(g.anti_patterns()[0].rejection_count, 1);

        // 重复拒绝同一编辑应递增计数
        g.reject_edit(edit, reason);
        assert_eq!(g.anti_patterns().len(), 1);
        assert_eq!(g.anti_patterns()[0].rejection_count, 2);
    }

    #[test]
    fn test_propose_edit_validation() {
        let mut g = make_graph();

        // 添加已存在的节点
        let dup_node = SkillNode {
            id: "skill_1".to_string(),
            name: "dup".to_string(),
            skill_type: SkillType::Tool,
            input_schema: "{}".to_string(),
            output_schema: "{}".to_string(),
            success_count: 0,
            failure_count: 0,
        };
        let result = g.propose_edit(GraphEdit::AddNode(dup_node));
        assert!(result.is_err());

        // 删除不存在的节点
        let result = g.propose_edit(GraphEdit::RemoveNode("nonexistent".to_string()));
        assert!(result.is_err());

        // 添加到不存在的节点的边
        let edge = ExecutionEdge {
            from: "skill_1".to_string(),
            to: "nonexistent".to_string(),
            edge_type: EdgeType::Prerequisite,
        };
        let result = g.propose_edit(GraphEdit::AddEdge(edge));
        assert!(result.is_err());
    }

    #[test]
    fn test_approve_edit_add_node() {
        let mut g = make_graph();
        let new_node = SkillNode {
            id: "skill_new".to_string(),
            name: "new_skill".to_string(),
            skill_type: SkillType::Hybrid,
            input_schema: "{}".to_string(),
            output_schema: "{}".to_string(),
            success_count: 0,
            failure_count: 0,
        };
        g.approve_edit(GraphEdit::AddNode(new_node));
        assert_eq!(g.node_count(), 4);
        assert!(g.get_node("skill_new").is_some());
    }

    #[test]
    fn test_approve_edit_remove_node() {
        let mut g = make_graph();
        // skill_3 是叶节点，无依赖，可以安全删除
        g.approve_edit(GraphEdit::RemoveNode("skill_3".to_string()));
        assert_eq!(g.node_count(), 2);
        assert!(g.get_node("skill_3").is_none());
    }

    #[test]
    fn test_root_and_leaf_nodes() {
        let g = make_graph();
        let roots = g.root_nodes();
        let leaves = g.leaf_nodes();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "fetch");
        assert_eq!(leaves.len(), 1);
        assert_eq!(leaves[0].name, "summarize");
    }

    #[test]
    fn test_fallback_edges() {
        let mut g = make_graph();
        // 添加 fallback 边
        let fallback_id = g.add_node("fallback_parse", SkillType::Tool, "{}", "{}");
        let skill_1 = "skill_1".to_string();
        g.add_edge(&skill_1, &fallback_id, EdgeType::Fallback)
            .unwrap();

        let fallbacks = g.fallbacks(&skill_1);
        assert_eq!(fallbacks.len(), 1);
        assert_eq!(fallbacks[0].name, "fallback_parse");
    }

    #[test]
    fn test_stats() {
        let g = make_graph();
        let stats = g.stats();
        assert_eq!(stats.node_count, 3);
        assert_eq!(stats.edge_count, 2);
        assert_eq!(stats.trace_count, 0);
        assert_eq!(stats.rejected_edit_count, 0);
    }

    #[test]
    fn test_default() {
        let g = ProceduralGraph::default();
        assert_eq!(g.node_count(), 0);
    }

    #[test]
    fn test_nodes_by_success_rate() {
        let mut g = ProceduralGraph::new();
        let a = g.add_node("high", SkillType::Tool, "{}", "{}");
        let b = g.add_node("low", SkillType::Tool, "{}", "{}");

        // a: 9/10 = 0.9
        for _ in 0..9 {
            g.record_trace(ExecutionTrace {
                trace_id: format!("t_{}_s", a),
                steps: vec![TraceStep {
                    skill_id: a.clone(),
                    input_tokens: 0,
                    output_tokens: 0,
                    latency_ms: 0,
                    success: true,
                    error: None,
                }],
                success: true,
                total_tokens: 0,
                total_latency_ms: 0,
            });
        }
        g.record_trace(ExecutionTrace {
            trace_id: format!("t_{}_f", a),
            steps: vec![TraceStep {
                skill_id: a.clone(),
                input_tokens: 0,
                output_tokens: 0,
                latency_ms: 0,
                success: false,
                error: None,
            }],
            success: false,
            total_tokens: 0,
            total_latency_ms: 0,
        });

        // b: 1/10 = 0.1
        for _ in 0..9 {
            g.record_trace(ExecutionTrace {
                trace_id: format!("t_{}_f", b),
                steps: vec![TraceStep {
                    skill_id: b.clone(),
                    input_tokens: 0,
                    output_tokens: 0,
                    latency_ms: 0,
                    success: false,
                    error: None,
                }],
                success: false,
                total_tokens: 0,
                total_latency_ms: 0,
            });
        }
        g.record_trace(ExecutionTrace {
            trace_id: format!("t_{}_s", b),
            steps: vec![TraceStep {
                skill_id: b.clone(),
                input_tokens: 0,
                output_tokens: 0,
                latency_ms: 0,
                success: true,
                error: None,
            }],
            success: true,
            total_tokens: 0,
            total_latency_ms: 0,
        });

        let ranked = g.nodes_by_success_rate();
        assert_eq!(ranked[0].0.name, "high");
        assert_eq!(ranked[1].0.name, "low");
    }

    #[test]
    fn test_rewire_edge() {
        let mut g = make_graph();

        // skill_1 → skill_2 → skill_3
        // 将 skill_2 → skill_3 重连为 skill_2 → skill_1 (会形成循环)
        let result = g.propose_edit(GraphEdit::RewireEdge(
            "skill_2".to_string(),
            "skill_3".to_string(),
            "skill_1".to_string(),
        ));
        assert!(result.is_err());
    }
}
