//! # nt_core_graph_orch — StateGraph Orchestration
//!
//! LangGraph-style state graph with typed reducers and checkpointing.
//!
//! **Layer**: L0 Substrate (core infrastructure, foundation for L7 Capability)

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// Node identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeId(pub String);

/// Edge identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeId(pub String);

/// State reducer: updates state after a node executes
pub trait Reducer<S>: Send + Sync + Debug {
    fn reduce(&self, state: &mut S, node_id: &NodeId, output: &str) -> Result<(), String>;
}

/// Async handler for graph nodes
pub trait NodeHandler<S>: Send + Sync + Debug {
    fn handle(&self, state: &mut S, config: &RunConfig) -> Result<String, String>;
}

/// Run configuration
#[derive(Debug, Clone)]
pub struct RunConfig {
    pub max_steps: usize,
    pub checkpoint_interval: usize,
    pub metadata: HashMap<String, String>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self { max_steps: 100, checkpoint_interval: 10, metadata: HashMap::new() }
    }
}

/// A node in the state graph
#[derive(Clone)]
pub struct GraphNode<S> {
    pub id: NodeId,
    pub handler: Arc<dyn NodeHandler<S>>,
    pub metadata: HashMap<String, String>,
}

impl<S: Debug> Debug for GraphNode<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphNode").field("id", &self.id).field("metadata", &self.metadata).finish()
    }
}

/// Edge condition for routing
#[derive(Clone)]
pub enum EdgeCondition<S> {
    Always { target: NodeId },
    Conditional { target: NodeId, condition: Arc<dyn Fn(&S) -> bool + Send + Sync> },
}

impl<S: Debug> Debug for EdgeCondition<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeCondition::Always { target } => f.debug_struct("Always").field("target", target).finish(),
            EdgeCondition::Conditional { target, .. } => f.debug_struct("Conditional").field("target", target).finish(),
        }
    }
}

/// A directed edge in the graph
#[derive(Clone)]
pub struct GraphEdge<S> {
    pub id: EdgeId,
    pub source: NodeId,
    pub condition: EdgeCondition<S>,
}

impl<S: Debug> Debug for GraphEdge<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphEdge").field("id", &self.id).field("source", &self.source).field("condition", &self.condition).finish()
    }
}

/// Step record from execution
#[derive(Debug, Clone, Serialize)]
pub struct StepRecord {
    pub step: usize,
    pub node_id: NodeId,
    pub output: String,
    pub duration_ms: u64,
}

/// Execution result
#[derive(Debug, Clone)]
pub struct RunResult<S> {
    pub final_state: Option<S>,
    pub steps: Vec<StepRecord>,
    pub checkpoint_id: Option<String>,
    pub error: Option<String>,
}

/// Checkpoint at a point in execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint<S> {
    pub checkpoint_id: String,
    pub node_id: NodeId,
    pub state: S,
    pub step: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Checkpoint store trait
pub trait CheckpointStore<S>: Send + Sync {
    fn save(&mut self, checkpoint: &Checkpoint<S>) -> Result<(), String>;
    fn load(&self, checkpoint_id: &str) -> Option<Checkpoint<S>>;
    fn list(&self) -> Vec<String>;
    fn prune(&mut self, max_count: usize) -> Result<(), String>;
}

/// In-memory checkpoint store
#[derive(Debug, Clone)]
pub struct InMemoryCheckpointStore<S> {
    checkpoints: HashMap<String, Checkpoint<S>>,
}

impl<S> Default for InMemoryCheckpointStore<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> InMemoryCheckpointStore<S> {
    pub fn new() -> Self { Self { checkpoints: HashMap::new() } }
}

impl<S: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de>> CheckpointStore<S> for InMemoryCheckpointStore<S> {
    fn save(&mut self, checkpoint: &Checkpoint<S>) -> Result<(), String> {
        self.checkpoints.insert(checkpoint.checkpoint_id.clone(), checkpoint.clone());
        Ok(())
    }

    fn load(&self, checkpoint_id: &str) -> Option<Checkpoint<S>> {
        self.checkpoints.get(checkpoint_id).cloned()
    }

    fn list(&self) -> Vec<String> {
        self.checkpoints.keys().cloned().collect()
    }

    fn prune(&mut self, max_count: usize) -> Result<(), String> {
        while self.checkpoints.len() > max_count {
            if let Some(oldest) = self.checkpoints.keys().cloned().min() {
                self.checkpoints.remove(&oldest);
            }
        }
        Ok(())
    }
}

/// State graph definition
#[derive(Debug, Clone)]
pub struct StateGraph<S> {
    pub name: String,
    pub nodes: HashMap<NodeId, GraphNode<S>>,
    pub edges: Vec<GraphEdge<S>>,
    pub entry_point: Option<NodeId>,
    pub reducers: Vec<Arc<dyn Reducer<S>>>,
}

impl<S: Debug + Clone + Serialize> StateGraph<S> {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), nodes: HashMap::new(), edges: Vec::new(), entry_point: None, reducers: Vec::new() }
    }

    pub fn add_node(&mut self, node: GraphNode<S>) -> Result<(), String> {
        if self.nodes.contains_key(&node.id) {
            return Err(format!("Node {:?} already exists", node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn add_edge(&mut self, edge: GraphEdge<S>) -> Result<(), String> {
        if !self.nodes.contains_key(&edge.source) {
            return Err(format!("Source node {:?} not found", edge.source));
        }
        let target = match &edge.condition {
            EdgeCondition::Always { target } => target,
            EdgeCondition::Conditional { target, .. } => target,
        };
        if !self.nodes.contains_key(target) {
            return Err(format!("Target node {:?} not found", target));
        }
        self.edges.push(edge);
        Ok(())
    }

    pub fn set_entry_point(&mut self, id: NodeId) { self.entry_point = Some(id); }
    pub fn add_reducer(&mut self, reducer: Arc<dyn Reducer<S>>) { self.reducers.push(reducer); }

    /// Validate: detect cycles, unreachable nodes, missing entry
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.entry_point.is_none() {
            errors.push("No entry point set".into());
        }

        if self.nodes.is_empty() {
            errors.push("No nodes in graph".into());
        }

        if let Some(ref entry) = self.entry_point {
            if !self.nodes.contains_key(entry) {
                errors.push(format!("Entry point {:?} not found in nodes", entry));
            }
        }

        // Unreachable nodes: BFS from entry
        if let Some(ref entry) = self.entry_point {
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(entry.clone());
            while let Some(node) = queue.pop_front() {
                if !visited.insert(node.clone()) { continue; }
                for edge in &self.edges {
                    if edge.source == node {
                        match &edge.condition {
                            EdgeCondition::Always { target } | EdgeCondition::Conditional { target, .. } => {
                                queue.push_back(target.clone());
                            }
                        }
                    }
                }
            }
            for node_id in self.nodes.keys() {
                if !visited.contains(node_id) {
                    errors.push(format!("Unreachable node: {:?}", node_id));
                }
            }
        }

        // Cycle detection via DFS
        if let Some(ref entry) = self.entry_point {
            let mut white: HashSet<NodeId> = self.nodes.keys().cloned().collect();
            let mut gray = HashSet::new();
            let mut black = HashSet::new();

            fn visit<S>(
                node: &NodeId, edges: &[GraphEdge<S>], white: &mut HashSet<NodeId>,
                gray: &mut HashSet<NodeId>, black: &mut HashSet<NodeId>,
                path: &mut Vec<NodeId>, errors: &mut Vec<String>,
            ) {
                white.remove(node);
                gray.insert(node.clone());
                path.push(node.clone());

                for edge in edges {
                    if edge.source == *node {
                        let target = match &edge.condition {
                            EdgeCondition::Always { target } | EdgeCondition::Conditional { target, .. } => target,
                        };
                        if black.contains(target) { continue; }
                        if gray.contains(target) {
                            let cycle_start = path.iter().position(|n| n == target).unwrap_or(0);
                            let cycle: Vec<String> = path[cycle_start..].iter().map(|n| n.0.clone()).chain(std::iter::once(target.0.clone())).collect();
                            errors.push(format!("Cycle detected: {}", cycle.join(" → ")));
                            continue;
                        }
                        if white.contains(target) {
                            visit(target, edges, white, gray, black, path, errors);
                        }
                    }
                }

                path.pop();
                gray.remove(node);
                black.insert(node.clone());
            }

            let mut path = Vec::new();
            visit(entry, &self.edges, &mut white, &mut gray, &mut black, &mut path, &mut errors);
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// Compile: validate and return a compiled graph ready to run
    pub fn compile(&self) -> Result<CompiledGraph<S>, Vec<String>> {
        self.validate()?;
        Ok(CompiledGraph { graph: self.clone() })
    }
}

/// Validated graph ready to execute
#[derive(Debug, Clone)]
pub struct CompiledGraph<S> {
    pub graph: StateGraph<S>,
}

impl<S: Debug + Clone + Send + Sync + Serialize + for<'de> Deserialize<'de>> CompiledGraph<S> {
    /// Run the graph from initial state
    pub fn run(&self, mut state: S, config: &RunConfig) -> RunResult<S> {
        let entry = match self.graph.entry_point.as_ref() {
            Some(e) => e.clone(),
            None => return RunResult { final_state: None, steps: Vec::new(), checkpoint_id: None, error: Some("No entry point".into()) },
        };

        let mut current = entry;
        let mut steps = Vec::new();
        let mut store = InMemoryCheckpointStore::new();
        let mut cp_count = 0;

        for step in 0..config.max_steps {
            let node = match self.graph.nodes.get(&current) {
                Some(n) => n,
                None => return RunResult {
                    final_state: Some(state), steps, checkpoint_id: None,
                    error: Some(format!("Node {:?} not found", current)),
                },
            };

            let start = Instant::now();
            match node.handler.handle(&mut state, config) {
                Ok(output) => {
                    let duration = start.elapsed().as_millis() as u64;
                    steps.push(StepRecord { step, node_id: current.clone(), output: output.clone(), duration_ms: duration });

                    for reducer in &self.graph.reducers {
                        if let Err(e) = reducer.reduce(&mut state, &current, &output) {
                            return RunResult {
                                final_state: Some(state), steps, checkpoint_id: None,
                                error: Some(format!("Reducer error: {}", e)),
                            };
                        }
                    }

                    if step > 0 && step % config.checkpoint_interval == 0 {
                        let cp = Checkpoint {
                            checkpoint_id: format!("cp-{}-{}", self.graph.name, step),
                            node_id: current.clone(),
                            state: state.clone(),
                            step,
                            timestamp: chrono::Utc::now(),
                        };
                        let _ = store.save(&cp);
                        cp_count += 1;
                    }

                    let mut next_found = false;
                    for edge in &self.graph.edges {
                        if edge.source == current {
                            match &edge.condition {
                                EdgeCondition::Always { target } => {
                                    current = target.clone();
                                    next_found = true;
                                    break;
                                }
                                EdgeCondition::Conditional { target, condition } => {
                                    if (condition)(&state) {
                                        current = target.clone();
                                        next_found = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if !next_found { break; }
                }
                Err(e) => return RunResult {
                    final_state: Some(state), steps, checkpoint_id: None,
                    error: Some(e),
                },
            }
        }

        RunResult {
            final_state: Some(state), steps,
            checkpoint_id: if cp_count > 0 { Some(format!("cp-{}-last", self.graph.name)) } else { None },
            error: None,
        }
    }

    /// Run with checkpoint recovery: if a checkpoint exists, resume from there
    pub fn run_with_recovery(&self, state: S, config: &RunConfig, checkpoint_id: &str, store: &mut dyn CheckpointStore<S>) -> RunResult<S> {
        if let Some(cp) = store.load(checkpoint_id) {
            // Search for the node in the graph to resume from
            if self.graph.nodes.contains_key(&cp.node_id) {
                let mut partial_graph = self.graph.clone();
                partial_graph.entry_point = Some(cp.node_id);
                let compiled = CompiledGraph { graph: partial_graph };
                return compiled.run(cp.state, config);
            }
        }
        self.run(state, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        pub counter: i32,
        pub path: Vec<String>,
    }

    #[derive(Debug)]
    struct IncHandler;
    impl NodeHandler<TestState> for IncHandler {
        fn handle(&self, state: &mut TestState, _config: &RunConfig) -> Result<String, String> {
            state.counter += 1;
            state.path.push("inc".into());
            Ok("incremented".into())
        }
    }

    #[derive(Debug)]
    struct DecHandler;
    impl NodeHandler<TestState> for DecHandler {
        fn handle(&self, state: &mut TestState, _config: &RunConfig) -> Result<String, String> {
            state.counter -= 1;
            state.path.push("dec".into());
            Ok("decremented".into())
        }
    }

    #[derive(Debug)]
    struct NoopHandler;
    impl NodeHandler<TestState> for NoopHandler {
        fn handle(&self, state: &mut TestState, _config: &RunConfig) -> Result<String, String> {
            state.path.push("noop".into());
            Ok("done".into())
        }
    }

    #[derive(Debug)]
    struct AddReducer;
    impl Reducer<TestState> for AddReducer {
        fn reduce(&self, state: &mut TestState, _node_id: &NodeId, _output: &str) -> Result<(), String> {
            state.counter += 1;
            Ok(())
        }
    }

    #[test]
    fn test_linear_graph_executes_all_nodes() {
        let mut graph: StateGraph<TestState> = StateGraph::new("test_linear");
        let a = GraphNode { id: NodeId("A".into()), handler: Arc::new(IncHandler), metadata: HashMap::new() };
        let b = GraphNode { id: NodeId("B".into()), handler: Arc::new(IncHandler), metadata: HashMap::new() };
        let c = GraphNode { id: NodeId("C".into()), handler: Arc::new(IncHandler), metadata: HashMap::new() };
        graph.add_node(a).unwrap();
        graph.add_node(b).unwrap();
        graph.add_node(c).unwrap();
        graph.add_edge(GraphEdge { id: EdgeId("e1".into()), source: NodeId("A".into()), condition: EdgeCondition::Always { target: NodeId("B".into()) } }).unwrap();
        graph.add_edge(GraphEdge { id: EdgeId("e2".into()), source: NodeId("B".into()), condition: EdgeCondition::Always { target: NodeId("C".into()) } }).unwrap();
        graph.set_entry_point(NodeId("A".into()));

        let compiled = graph.compile().unwrap();
        let result = compiled.run(TestState { counter: 0, path: Vec::new() }, &RunConfig::default());
        assert!(result.error.is_none());
        assert_eq!(result.final_state.as_ref().unwrap().counter, 3);
        assert_eq!(result.steps.len(), 3);
    }

    #[test]
    fn test_conditional_branching() {
        let mut graph: StateGraph<TestState> = StateGraph::new("test_cond");
        let entry = GraphNode { id: NodeId("entry".into()), handler: Arc::new(NoopHandler), metadata: HashMap::new() };
        let inc = GraphNode { id: NodeId("inc".into()), handler: Arc::new(IncHandler), metadata: HashMap::new() };
        let dec = GraphNode { id: NodeId("dec".into()), handler: Arc::new(DecHandler), metadata: HashMap::new() };
        graph.add_node(entry).unwrap();
        graph.add_node(inc).unwrap();
        graph.add_node(dec).unwrap();
        graph.add_edge(GraphEdge {
            id: EdgeId("cond".into()), source: NodeId("entry".into()),
            condition: EdgeCondition::Conditional {
                target: NodeId("inc".into()),
                condition: Arc::new(|s: &TestState| s.counter >= 0),
            },
        }).unwrap();
        graph.add_edge(GraphEdge {
            id: EdgeId("cond2".into()), source: NodeId("entry".into()),
            condition: EdgeCondition::Conditional {
                target: NodeId("dec".into()),
                condition: Arc::new(|s: &TestState| s.counter < 0),
            },
        }).unwrap();
        graph.set_entry_point(NodeId("entry".into()));

        let compiled = graph.compile().unwrap();
        let result = compiled.run(TestState { counter: 5, path: Vec::new() }, &RunConfig::default());
        assert!(result.error.is_none());
        assert_eq!(result.final_state.as_ref().unwrap().counter, 6); // only inc path

        let result2 = compiled.run(TestState { counter: -1, path: Vec::new() }, &RunConfig::default());
        assert!(result2.error.is_none());
        assert_eq!(result2.final_state.as_ref().unwrap().counter, -2); // only dec path
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph: StateGraph<TestState> = StateGraph::new("test_cycle");
        let a = GraphNode { id: NodeId("A".into()), handler: Arc::new(NoopHandler), metadata: HashMap::new() };
        let b = GraphNode { id: NodeId("B".into()), handler: Arc::new(NoopHandler), metadata: HashMap::new() };
        graph.add_node(a).unwrap();
        graph.add_node(b).unwrap();
        graph.add_edge(GraphEdge { id: EdgeId("ab".into()), source: NodeId("A".into()), condition: EdgeCondition::Always { target: NodeId("B".into()) } }).unwrap();
        graph.add_edge(GraphEdge { id: EdgeId("ba".into()), source: NodeId("B".into()), condition: EdgeCondition::Always { target: NodeId("A".into()) } }).unwrap();
        graph.set_entry_point(NodeId("A".into()));

        let result = graph.compile();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("Cycle")));
    }

    #[test]
    fn test_unreachable_node_detected() {
        let mut graph: StateGraph<TestState> = StateGraph::new("test_unreachable");
        let a = GraphNode { id: NodeId("A".into()), handler: Arc::new(NoopHandler), metadata: HashMap::new() };
        let b = GraphNode { id: NodeId("B".into()), handler: Arc::new(NoopHandler), metadata: HashMap::new() };
        graph.add_node(a).unwrap();
        graph.add_node(b).unwrap();
        graph.set_entry_point(NodeId("A".into()));

        let result = graph.compile();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("Unreachable")));
    }

    #[test]
    fn test_checkpoint_save_load() {
        let mut store: InMemoryCheckpointStore<TestState> = InMemoryCheckpointStore::new();
        let cp = Checkpoint {
            checkpoint_id: "cp-1".into(),
            node_id: NodeId("A".into()),
            state: TestState { counter: 42, path: vec!["test".into()] },
            step: 1,
            timestamp: chrono::Utc::now(),
        };
        store.save(&cp).unwrap();
        let loaded = store.load("cp-1").unwrap();
        assert_eq!(loaded.state.counter, 42);
        assert_eq!(loaded.node_id, NodeId("A".into()));
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn test_reducer_updates_state() {
        let mut graph: StateGraph<TestState> = StateGraph::new("test_reducer");
        let a = GraphNode { id: NodeId("A".into()), handler: Arc::new(NoopHandler), metadata: HashMap::new() };
        let b = GraphNode { id: NodeId("B".into()), handler: Arc::new(NoopHandler), metadata: HashMap::new() };
        graph.add_node(a).unwrap();
        graph.add_node(b).unwrap();
        graph.add_edge(GraphEdge { id: EdgeId("ab".into()), source: NodeId("A".into()), condition: EdgeCondition::Always { target: NodeId("B".into()) } }).unwrap();
        graph.set_entry_point(NodeId("A".into()));
        graph.add_reducer(Arc::new(AddReducer));

        let compiled = graph.compile().unwrap();
        let result = compiled.run(TestState { counter: 0, path: Vec::new() }, &RunConfig::default());
        assert!(result.error.is_none());
        assert!(result.final_state.as_ref().unwrap().counter >= 2); // 2 nodes × +1 each
    }

    #[test]
    fn test_missing_entry_point() {
        let graph: StateGraph<TestState> = StateGraph::new("test_no_entry");
        let result = graph.compile();
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_recovery() {
        let mut graph: StateGraph<TestState> = StateGraph::new("test_recovery");
        let a = GraphNode { id: NodeId("A".into()), handler: Arc::new(IncHandler), metadata: HashMap::new() };
        let b = GraphNode { id: NodeId("B".into()), handler: Arc::new(IncHandler), metadata: HashMap::new() };
        graph.add_node(a).unwrap();
        graph.add_node(b).unwrap();
        graph.add_edge(GraphEdge { id: EdgeId("ab".into()), source: NodeId("A".into()), condition: EdgeCondition::Always { target: NodeId("B".into()) } }).unwrap();
        graph.set_entry_point(NodeId("A".into()));

        let compiled = graph.compile().unwrap();
        let mut store = InMemoryCheckpointStore::new();
        let cp = Checkpoint {
            checkpoint_id: "resume".into(),
            node_id: NodeId("B".into()),
            state: TestState { counter: 10, path: vec!["recovered".into()] },
            step: 1,
            timestamp: chrono::Utc::now(),
        };
        store.save(&cp).unwrap();

        let result = compiled.run_with_recovery(TestState { counter: 0, path: Vec::new() }, &RunConfig::default(), "resume", &mut store);
        assert!(result.error.is_none());
        assert_eq!(result.final_state.as_ref().unwrap().counter, 11);
    }
}
