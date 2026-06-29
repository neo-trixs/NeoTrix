use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArtifactType {
    Proposal,
    Spec,
    Design,
    Task,
    Code,
    Test,
    Review,
    Prd,
    Experiment,
    UxReport,
    JourneyMap,
    CaseStudy,
}

impl ArtifactType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Proposal => "proposal",
            Self::Spec => "spec",
            Self::Design => "design",
            Self::Task => "task",
            Self::Code => "code",
            Self::Test => "test",
            Self::Review => "review",
            Self::Prd => "prd",
            Self::Experiment => "experiment",
            Self::UxReport => "ux_report",
            Self::JourneyMap => "journey_map",
            Self::CaseStudy => "case_study",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactState {
    Pending,
    Ready,
    InProgress,
    Done,
    Blocked(String),
}

#[derive(Debug, Clone)]
pub struct ArtifactNode {
    pub id: String,
    pub artifact_type: ArtifactType,
    pub description: String,
    pub state: ArtifactState,
}

impl ArtifactNode {
    pub fn new(id: &str, artifact_type: ArtifactType, description: &str) -> Self {
        Self {
            id: id.to_string(),
            artifact_type,
            description: description.to_string(),
            state: ArtifactState::Pending,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DagEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone)]
pub struct StateGraph {
    pub nodes: HashMap<String, ArtifactNode>,
    pub edges: Vec<DagEdge>,
}

impl Default for StateGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl StateGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: ArtifactNode) {
        self.nodes.entry(node.id.clone()).or_insert(node);
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        if self.nodes.contains_key(from) && self.nodes.contains_key(to) {
            self.edges.push(DagEdge {
                from: from.to_string(),
                to: to.to_string(),
            });
        }
    }

    pub fn node(&self, id: &str) -> Option<&ArtifactNode> {
        self.nodes.get(id)
    }

    pub fn node_mut(&mut self, id: &str) -> Option<&mut ArtifactNode> {
        self.nodes.get_mut(id)
    }

    /// Kahn's algorithm for topological sort
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for id in self.nodes.keys() {
            in_degree.entry(id.as_str()).or_insert(0);
        }
        for edge in &self.edges {
            *in_degree.entry(&edge.to).or_insert(0) += 1;
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted = Vec::new();
        while let Some(node) = queue.pop_front() {
            sorted.push(node.to_string());
            for edge in &self.edges {
                if edge.from == node {
                    if let Some(deg) = in_degree.get_mut(edge.to.as_str()) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(&edge.to);
                        }
                    }
                }
            }
        }

        if sorted.len() != self.nodes.len() {
            let missing: Vec<String> = self
                .nodes
                .keys()
                .filter(|id| !sorted.contains(id))
                .cloned()
                .collect();
            return Err(format!("Cycle detected in nodes: {:?}", missing));
        }

        Ok(sorted)
    }

    /// 当前就绪（所有前置依赖已完成）的节点
    pub fn ready_nodes(&self) -> Vec<&ArtifactNode> {
        let mut ready = Vec::new();
        for node in self.nodes.values() {
            if node.state == ArtifactState::Ready {
                ready.push(node);
                continue;
            }
            if node.state != ArtifactState::Pending {
                continue;
            }
            let all_deps_done = self.edges.iter().filter(|e| e.to == node.id).all(|e| {
                self.nodes
                    .get(&e.from)
                    .map(|n| n.state == ArtifactState::Done)
                    .unwrap_or(false)
            });
            if all_deps_done {
                ready.push(node);
            }
        }
        ready
    }

    /// 标记节点完成，自动更新下游节点状态
    pub fn mark_done(&mut self, id: &str) -> Result<(), String> {
        let node = self
            .nodes
            .get_mut(id)
            .ok_or_else(|| format!("Node '{}' not found", id))?;
        node.state = ArtifactState::Done;

        let to_update: Vec<String> = self
            .edges
            .iter()
            .filter(|e| e.from == id)
            .map(|e| e.to.clone())
            .collect();

        for dep_id in &to_update {
            let from_ids: Vec<String> = self
                .edges
                .iter()
                .filter(|e| e.to == *dep_id)
                .map(|e| e.from.clone())
                .collect();
            let all_deps_done = from_ids.iter().all(|fid| {
                self.nodes
                    .get(fid)
                    .map(|n| n.state == ArtifactState::Done)
                    .unwrap_or(false)
            });
            if let Some(dep) = self.nodes.get_mut(dep_id) {
                if dep.state == ArtifactState::Pending && all_deps_done {
                    dep.state = ArtifactState::Ready;
                }
            }
        }
        Ok(())
    }

    /// 从目标描述自动构建任务 DAG
    pub fn build_plan(&mut self, goal: &str, task_count: usize) {
        let goal_slug = goal
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .take(20)
            .collect::<String>()
            .trim()
            .replace(' ', "_");

        self.add_node(ArtifactNode::new(
            &format!("{}.proposal", goal_slug),
            ArtifactType::Proposal,
            &format!("Proposal: {}", goal),
        ));
        self.add_node(ArtifactNode::new(
            &format!("{}.spec", goal_slug),
            ArtifactType::Spec,
            "Define system behavior spec",
        ));
        self.add_edge(
            &format!("{}.proposal", goal_slug),
            &format!("{}.spec", goal_slug),
        );

        for i in 0..task_count {
            let task_id = format!("{}.task_{}", goal_slug, i);
            self.add_node(ArtifactNode::new(
                &task_id,
                ArtifactType::Task,
                &format!("Task #{}", i),
            ));
            self.add_edge(&format!("{}.spec", goal_slug), &task_id);
        }

        let review_id = format!("{}.review", goal_slug);
        self.add_node(ArtifactNode::new(
            &review_id,
            ArtifactType::Review,
            "Review all outputs",
        ));
        for i in 0..task_count {
            self.add_edge(&format!("{}.task_{}", goal_slug, i), &review_id);
        }

        if let Some(proposal) = self.nodes.get_mut(&format!("{}.proposal", goal_slug)) {
            proposal.state = ArtifactState::Ready;
        }
    }

    /// Build a PM-specific DAG plan with appropriate topology
    pub fn build_pm_plan(&mut self, goal: &str, _task_count: usize) {
        let research = ArtifactNode::new(
            "research",
            ArtifactType::Prd,
            &format!("Research: {}", goal),
        );
        let draft = ArtifactNode::new("draft", ArtifactType::Prd, &format!("Draft: {}", goal));
        let review =
            ArtifactNode::new("review", ArtifactType::Review, &format!("Review: {}", goal));
        let refine = ArtifactNode::new("refine", ArtifactType::Prd, &format!("Refine: {}", goal));

        self.add_node(research);
        self.add_node(draft);
        self.add_node(review);
        self.add_node(refine);

        // Research → Draft → Review → Refine
        self.add_edge("research", "draft");
        self.add_edge("draft", "review");
        self.add_edge("review", "refine");

        // Mark root node as ready
        if let Some(root) = self.nodes.get_mut("research") {
            root.state = ArtifactState::Ready;
        }
    }

    pub fn summary(&self) -> String {
        let total = self.nodes.len();
        let done = self
            .nodes
            .values()
            .filter(|n| n.state == ArtifactState::Done)
            .count();
        let ready = self
            .nodes
            .values()
            .filter(|n| n.state == ArtifactState::Ready)
            .count();
        let in_progress = self
            .nodes
            .values()
            .filter(|n| n.state == ArtifactState::InProgress)
            .count();
        format!(
            "DAG: {}/{} done, {} ready, {} in-progress",
            done, total, ready, in_progress
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph_empty() {
        let g = StateGraph::new();
        assert!(g.nodes.is_empty());
        assert!(g.edges.is_empty());
    }

    #[test]
    fn test_add_node_and_edge() {
        let mut g = StateGraph::new();
        g.add_node(ArtifactNode::new("a", ArtifactType::Proposal, "Proposal"));
        g.add_node(ArtifactNode::new("b", ArtifactType::Task, "Task"));
        g.add_edge("a", "b");
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.edges.len(), 1);
    }

    #[test]
    fn test_topological_sort_simple() {
        let mut g = StateGraph::new();
        g.add_node(ArtifactNode::new("a", ArtifactType::Proposal, ""));
        g.add_node(ArtifactNode::new("b", ArtifactType::Spec, ""));
        g.add_node(ArtifactNode::new("c", ArtifactType::Task, ""));
        g.add_edge("a", "b");
        g.add_edge("b", "c");
        let sorted = g.topological_sort().expect("valid DAG");
        assert_eq!(sorted, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_topological_sort_diamond() {
        let mut g = StateGraph::new();
        g.add_node(ArtifactNode::new("root", ArtifactType::Proposal, ""));
        g.add_node(ArtifactNode::new("left", ArtifactType::Task, ""));
        g.add_node(ArtifactNode::new("right", ArtifactType::Task, ""));
        g.add_node(ArtifactNode::new("merge", ArtifactType::Review, ""));
        g.add_edge("root", "left");
        g.add_edge("root", "right");
        g.add_edge("left", "merge");
        g.add_edge("right", "merge");
        let sorted = g.topological_sort().expect("valid DAG");
        assert_eq!(sorted.len(), 4);
        assert_eq!(sorted[0], "root");
        assert_eq!(sorted[3], "merge");
        assert!(sorted[1] == "left" || sorted[1] == "right");
    }

    #[test]
    fn test_cycle_detection() {
        let mut g = StateGraph::new();
        g.add_node(ArtifactNode::new("a", ArtifactType::Task, ""));
        g.add_node(ArtifactNode::new("b", ArtifactType::Task, ""));
        g.add_edge("a", "b");
        g.add_edge("b", "a");
        assert!(g.topological_sort().is_err());
    }

    #[test]
    fn test_mark_done_unlocks_dependents() {
        let mut g = StateGraph::new();
        g.add_node(ArtifactNode::new("a", ArtifactType::Proposal, ""));
        g.add_node(ArtifactNode::new("b", ArtifactType::Task, ""));
        g.add_node(ArtifactNode::new("c", ArtifactType::Review, ""));
        g.add_edge("a", "b");
        g.add_edge("b", "c");

        g.node_mut("a").expect("node 'a' exists").state = ArtifactState::Ready;
        assert!(g.ready_nodes().iter().any(|n| n.id == "a"));

        g.mark_done("a").expect("mark_done should succeed");
        assert_eq!(
            g.node("a").expect("node 'a' exists").state,
            ArtifactState::Done
        );
        assert_eq!(
            g.node("b").expect("node 'b' exists").state,
            ArtifactState::Ready
        );

        g.mark_done("b").expect("mark_done should succeed");
        assert_eq!(
            g.node("c").expect("node 'c' exists").state,
            ArtifactState::Ready
        );
    }

    #[test]
    fn test_build_plan_creates_correct_structure() {
        let mut g = StateGraph::new();
        g.build_plan("test task", 3);
        let sorted = g.topological_sort().expect("valid DAG");
        assert_eq!(sorted.len(), 6); // 1 proposal + 1 spec + 3 tasks + 1 review

        let first = &sorted[0];
        assert!(first.contains("proposal"));
        let last = &sorted[5];
        assert!(last.contains("review"));
    }

    #[test]
    fn test_ready_nodes() {
        let mut g = StateGraph::new();
        g.build_plan("test", 2);
        let ready = g.ready_nodes();
        assert_eq!(ready.len(), 1);
        assert!(ready[0].id.contains("proposal"));
    }

    #[test]
    fn test_summary_format() {
        let g = StateGraph::new();
        let s = g.summary();
        assert!(s.contains("done"));
    }

    #[test]
    fn test_build_pm_plan() {
        let mut graph = StateGraph::new();
        graph.build_pm_plan("Create PRD", 4);
        assert!(graph.node("research").is_some());
        assert!(graph.node("draft").is_some());
        assert!(graph.node("review").is_some());
        assert!(graph.node("refine").is_some());
        let research = graph.node("research").expect("node 'research' exists");
        assert_eq!(research.state, ArtifactState::Ready);
    }

    #[test]
    fn test_build_pm_plan_topological_order() {
        let mut graph = StateGraph::new();
        graph.build_pm_plan("UX Audit", 4);
        let sorted = graph.topological_sort().expect("valid DAG");
        assert_eq!(sorted.len(), 4);
        assert_eq!(sorted[0], "research");
        assert_eq!(sorted[3], "refine");
    }

    #[test]
    fn test_pm_artifacts_mark_done() {
        let mut graph = StateGraph::new();
        graph.build_pm_plan("Experiment design", 4);
        graph
            .mark_done("research")
            .expect("mark_done should succeed");
        assert_eq!(
            graph.node("draft").expect("node 'draft' exists").state,
            ArtifactState::Ready
        );
        graph.mark_done("draft").expect("mark_done should succeed");
        assert_eq!(
            graph.node("review").expect("node 'review' exists").state,
            ArtifactState::Ready
        );
        graph.mark_done("review").expect("mark_done should succeed");
        assert_eq!(
            graph.node("refine").expect("node 'refine' exists").state,
            ArtifactState::Ready
        );
    }
}
