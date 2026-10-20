# nt_core_graph_orch — Graph-Based Orchestration Engine

**Blind Spot**: No graph-based orchestration — E8 uses linear `reason()` → `reason_with_plan()` with sequential phases. No branching, no parallel execution, no checkpoint-based recovery.

**Sources**: LangGraph (34k★, Pregel-inspired state machine), Microsoft Agent Framework Workflow, Google's Pregel paper.

**Layer**: L4 Cognition (E8 sibling), depends on L0–L3 only.

---

## 1. Core Architecture

The graph engine wraps every E8 transition, SEAL stage, and tool call in a `StateGraph<S>` where `S` is the unified reasoning state. The entire NeoTrix reasoning loop becomes a single compiled graph.

### 1.1 Core Types

```rust
// neotrix-core/src/neotrix/l4_cognition_impl/nt_core_graph_orch/mod.rs

/// Generic state graph over state type S.
/// Inspired by LangGraph's StateGraph + Pregel superstep model.
pub struct StateGraph<S: Clone + Serialize + DeserializeOwned> {
    /// Named nodes — each is a computation step
    nodes: HashMap<String, GraphNode<S>>,
    /// Directed edges with optional condition functions
    edges: Vec<GraphEdge<S>>,
    /// Reducer functions keyed by state field name.
    /// LangGraph's key innovation: deterministic merge, not arbitrary mutation.
    reducers: HashMap<String, Box<dyn Reducer<S>>>,
    /// Persistence backend for crash recovery
    checkpoint_store: Box<dyn CheckpointStore>,
    /// Entry/exit points
    entry_point: String,
    exit_points: Vec<String>,
}

pub struct GraphNode<S> {
    pub name: String,
    pub handler: Box<dyn Fn(S) -> Result<S, GraphError>>,
    pub metadata: NodeMetadata,
}

pub struct GraphEdge<S> {
    pub from: String,
    pub to: String,
    /// If None, edge is unconditional (always taken after `from` completes)
    pub condition: Option<Box<dyn Fn(&StateSnapshot<S>) -> bool>>,
}

/// A reducer merges updates from multiple parallel nodes into shared state.
/// Signature: (current_value, update_value) -> merged_value
pub trait Reducer<S> {
    fn reduce(&self, current: &mut S, update: S);
}

/// Common reducer: append lists (e.g., messages += [new_msg])
pub struct AppendReducer;

impl<S: HasMessages> Reducer<S> for AppendReducer {
    fn reduce(&self, current: &mut S, update: S) {
        current.messages_mut().extend(update.into_messages());
    }
}

/// Common reducer: last-write-wins for scalar fields
pub struct LastWriteWinsReducer;

impl<S: HasStateFields> Reducer<S> for LastWriteWinsReducer {
    fn reduce(&self, current: &mut S, update: S) {
        for (key, value) in update.into_fields() {
            current.set_field(key, value); // always overwrites
        }
    }
}
```

### 1.2 Superstep Execution Model

Each iteration of the graph = one **superstep** (Pregel terminology).

```rust
pub struct SuperstepConfig {
    pub max_steps: usize,       // safety limit: fail after N supersteps
    pub sync_barrier: bool,     // if true, all nodes complete before next superstep
    pub timeout_ms: u64,        // per-superstep timeout
}

pub enum SuperstepEvent<S> {
    BeforeSuperstep { step: usize },
    NodeStarted { node: String, step: usize },
    NodeCompleted { node: String, step: usize, duration: Duration },
    CheckpointWritten { step: usize, size_bytes: usize },
    AfterSuperstep { step: usize, active_nodes: usize },
}

impl<S: Clone + Serialize + DeserializeOwned + 'static> StateGraph<S> {
    /// Execute the graph. Returns final state after all paths reach exit points.
    pub fn run(&self, initial_state: S, config: &SuperstepConfig) -> Result<S, GraphError> {
        let mut state = initial_state;
        let mut step = 0usize;

        // Load checkpoint if available
        if let Some(cp) = self.checkpoint_store.load_latest()? {
            state = bincode::deserialize(&cp.state)?;
            step = cp.step;
        }

        loop {
            step += 1;
            if step > config.max_steps {
                return Err(GraphError::SuperstepLimitExceeded(step));
            }

            // 1. Determine which nodes are eligible this superstep
            let eligible = self.find_eligibile_nodes(&state, step)?;
            if eligible.is_empty() {
                break; // all paths terminated
            }

            // 2. Execute all eligible nodes (parallel via rayon)
            let results: Vec<(String, Result<S, GraphError>)> = eligible.par_iter()
                .map(|node_name| {
                    let node = self.nodes.get(node_name).unwrap();
                    let result = (node.handler)(state.clone());
                    (node_name.clone(), result)
                })
                .collect();

            // 3. Apply reducers to merge results
            for (node_name, result) in &results {
                let update = result.as_ref().map_err(|e| e.clone())?;
                for (key, reducer) in &self.reducers {
                    reducer.reduce(&mut state, update.clone());
                }
            }

            // 4. Write checkpoint after every superstep
            let serialized = bincode::serialize(&state)?;
            self.checkpoint_store.write(&Checkpoint {
                step,
                state: serialized,
                timestamp: Instant::now(),
                node: eligible.join(","),
            })?;

            // 5. Check exit conditions
            if self.all_paths_terminated(&state) {
                break;
            }
        }

        Ok(state)
    }

    /// Find nodes whose all incoming edges' conditions are satisfied.
    /// On step 1, only entry_point is eligible.
    fn find_eligibile_nodes(&self, state: &S, step: usize) -> Result<Vec<String>, GraphError> {
        if step == 1 {
            return Ok(vec![self.entry_point.clone()]);
        }

        let snapshot = StateSnapshot::new(state, step);
        let mut eligible = Vec::new();

        for edge in &self.edges {
            let condition_met = match &edge.condition {
                Some(cond) => cond(&snapshot),
                None => true,
            };
            if condition_met && !eligible.contains(&edge.to) {
                eligible.push(edge.to.clone());
            }
        }

        Ok(eligible)
    }
}
```

### 1.3 Conditional Routing (LangGraph Pattern)

Conditions are pure functions over `StateSnapshot` — no side effects, deterministic.

```rust
pub struct StateSnapshot<S> {
    pub state: S,
    pub step: usize,
    pub previous_node: Option<String>,
}

// Factory helpers for common condition patterns
impl<S> StateSnapshot<S> {
    /// Route based on E8 hexagram mode
    pub fn e8_mode_is(&self, hex: ReasoningHexagram) -> bool
    where S: AsRef<FullReasoningState>
    {
        self.state.as_ref().current_mode == hex
    }

    /// Route based on error count threshold
    pub fn errors_exceed(&self, threshold: usize) -> bool
    where S: AsRef<FullReasoningState>
    {
        self.state.as_ref().error_count > threshold
    }

    /// Route based on specialist type
    pub fn specialist_is(&self, specialist: &str) -> bool
    where S: AsRef<FullReasoningState>
    {
        self.state.as_ref().active_specialist == specialist
    }
}
```

---

## 2. Checkpoint Store

SQLite-backed persistence enables crash recovery and time-travel debugging.

```rust
pub struct Checkpoint {
    pub step: usize,
    pub state: Vec<u8>,   // bincode-serialized
    pub timestamp: Instant,
    pub node: String,     // last completed node(s)
}

pub trait CheckpointStore: Send + Sync {
    fn write(&self, cp: &Checkpoint) -> Result<(), GraphError>;
    fn load_latest(&self) -> Result<Option<Checkpoint>, GraphError>;
    fn load_step(&self, step: usize) -> Result<Option<Checkpoint>, GraphError>;
    fn list_checkpoints(&self) -> Result<Vec<usize>, GraphError>; // step numbers
    fn prune_before(&self, step: usize) -> Result<usize, GraphError>; // count removed
}

pub struct SqliteCheckpointStore {
    conn: rusqlite::Connection,
}

impl SqliteCheckpointStore {
    pub fn open(path: &Path) -> Result<Self, GraphError> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS checkpoints (
                step INTEGER PRIMARY KEY,
                state BLOB NOT NULL,
                timestamp INTEGER NOT NULL,
                node TEXT NOT NULL
            );
            PRAGMA journal_mode=WAL;
        ")?;
        Ok(Self { conn })
    }
}

impl CheckpointStore for SqliteCheckpointStore {
    fn write(&self, cp: &Checkpoint) -> Result<(), GraphError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO checkpoints (step, state, timestamp, node) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                cp.step as i64,
                cp.state,
                cp.timestamp.elapsed().as_secs() as i64,
                cp.node,
            ],
        )?;
        Ok(())
    }

    fn load_latest(&self) -> Result<Option<Checkpoint>, GraphError> {
        let mut stmt = self.conn.prepare(
            "SELECT step, state, timestamp, node FROM checkpoints ORDER BY step DESC LIMIT 1"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Checkpoint {
                step: row.get::<_, i64>(0)? as usize,
                state: row.get(1)?,
                timestamp: Instant::now(), // approximate
                node: row.get(3)?,
            })
        })?;
        Ok(rows.into_iter().next().transpose()?)
    }
}
```

### Crash Recovery Algorithm

```
1. On startup, call checkpoint_store.load_latest()
2. If checkpoint exists:
   a. Deserialize state
   b. Re-run all supersteps from checkpoint.step + 1
   c. Deterministic reducers guarantee identical result
3. If no checkpoint:
   a. Start from fresh initial_state
4. Periodically prune checkpoints older than N supersteps (configurable)
```

---

## 3. E8 Integration

Each `GraphNode` wraps either an E8 transition or a SEAL stage.

```rust
// Wraps E8's reason() as a GraphNode
pub fn create_e8_node(name: &str, e8: Arc<Mutex<ReasoningEngine>>) -> GraphNode<FullReasoningState> {
    let e8_clone = e8.clone();
    let name_owned = name.to_string();
    GraphNode {
        name: name_owned,
        handler: Box::new(move |state: FullReasoningState| {
            let mut engine = e8_clone.lock().unwrap();
            let task = state.current_task.clone().unwrap_or_default();
            match engine.reason(&task) {
                Ok(response) => {
                    let mut new_state = state.clone();
                    new_state.last_response = Some(response);
                    new_state.error_count = 0;
                    Ok(new_state)
                }
                Err(e) => {
                    let mut new_state = state.clone();
                    new_state.error_count += 1;
                    new_state.last_error = Some(e.to_string());
                    Ok(new_state) // don't propagate error — let graph route handle it
                }
            }
        }),
        metadata: NodeMetadata {
            layer: Layer::Cognition,
            timeout_ms: 30_000,
            retry_count: 2,
            description: "E8 reasoning step".into(),
        },
    }
}

// Wraps a SEAL stage as a GraphNode
pub fn create_seal_node(stage: Box<dyn StageContract>) -> GraphNode<FullReasoningState> {
    GraphNode {
        name: stage.name().to_string(),
        handler: Box::new(move |state: FullReasoningState| {
            // SEAL stages operate on SelfIteratingBrain, not FullReasoningState
            // This wrapper translates: extract brain from state → run stage → write back
            // Implementation detail depends on how brain is stored in state
            todo!("SEAL stage wrapping")
        }),
        metadata: NodeMetadata {
            layer: Layer::Autonomic,
            timeout_ms: 60_000,
            retry_count: 1,
            description: format!("SEAL stage: {}", stage.name()),
        },
    }
}
```

### Example: Building the E8 Reasoning Graph

```rust
fn build_reasoning_graph(e8: Arc<Mutex<ReasoningEngine>>) -> StateGraph<FullReasoningState> {
    let mut graph = StateGraph::new(
        SqliteCheckpointStore::open(&PathBuf::from("~/.neotrix/graph_checkpoints.db")).unwrap()
    );

    // Nodes
    graph.add_node("entry", create_e8_node("entry", e8.clone()));
    graph.add_node("analyze", create_e8_node("analyze", e8.clone()));
    graph.add_node("plan", create_e8_node("plan", e8.clone()));
    graph.add_node("execute", create_e8_node("execute", e8.clone()));
    graph.add_node("verify", create_e8_node("verify", e8.clone()));
    graph.add_node("fallback", create_e8_node("fallback", e8.clone()));
    graph.add_node("finish", GraphNode {
        name: "finish".into(),
        handler: Box::new(|s| Ok(s)),
        metadata: NodeMetadata::exit(),
    });

    // Entry → Analyze unconditionally
    graph.add_edge(GraphEdge::unconditional("entry", "analyze"));

    // Analyze → Plan or Fallback (conditional on error)
    graph.add_edge(GraphEdge::conditional("analyze", "plan",
        |snap| snap.errors_exceed(0) == false));
    graph.add_edge(GraphEdge::conditional("analyze", "fallback",
        |snap| snap.errors_exceed(0)));

    // Plan → Execute unconditionally
    graph.add_edge(GraphEdge::unconditional("plan", "execute"));

    // Execute → Verify (always)
    graph.add_edge(GraphEdge::unconditional("execute", "verify"));

    // Verify → Finish or re-enter Plan (retry loop)
    graph.add_edge(GraphEdge::conditional("verify", "finish",
        |snap| snap.state.as_ref().error_count == 0));
    graph.add_edge(GraphEdge::conditional("verify", "plan",
        |snap| snap.state.as_ref().error_count > 0
            && snap.state.as_ref().error_count < 3));

    // Fallback → always finish (graceful degradation)
    graph.add_edge(GraphEdge::unconditional("fallback", "finish"));

    // Reducers
    graph.register_reducer("messages", Box::new(AppendReducer));
    graph.register_reducer("error_count", Box::new(LastWriteWinsReducer));

    graph.set_entry("entry");
    graph.add_exit("finish");

    graph
}
```

---

## 4. Handoff Pattern (OpenAI Agents SDK Pattern)

A sub-graph call suspends the parent graph and transfers control. The child graph runs to completion, then the parent resumes.

```rust
pub enum GraphEdge<S> {
    // ... normal edges ...
    /// Handoff: suspend parent, invoke child graph by name, resume with child's final state
    Handoff {
        from: String,
        child_graph: String,
        on_complete: Box<dyn Fn(S, S) -> S>, // merge child state into parent
    },
}

impl<S: Clone + Serialize + DeserializeOwned + 'static> StateGraph<S> {
    fn execute_node(&self, node_name: &str, state: S) -> Result<S, GraphError> {
        // Check for handoff edges from this node
        let handoffs: Vec<_> = self.edges.iter()
            .filter(|e| matches!(e, GraphEdge::Handoff { from, .. } if from == node_name))
            .collect();

        if handoffs.is_empty() {
            // Normal node execution
            let node = self.nodes.get(node_name).unwrap();
            return (node.handler)(state);
        }

        // Handoff: run child graph
        let mut child_state = state.clone();
        for handoff in &handoffs {
            if let GraphEdge::Handoff { child_graph, on_complete, .. } = handoff {
                let child = self.child_graphs.get(child_graph).unwrap();
                child_state = child.run(child_state, &self.superstep_config)?;
                state = on_complete(state, child_state);
            }
        }
        Ok(state)
    }
}
```

---

## 5. SEAL Pipeline Migration

The current 27-stage linear SEAL pipeline (`SelfIteratingBrain::step()` dispatches stages in order) becomes a `StateGraph<SealState>`.

### Stage → Node Mapping

| SEAL Stage | Graph Node | Parallel? | Condition |
|------------|-----------|-----------|-----------|
| Snapshot | `snapshot` | No | Always |
| AutonomyGate | `autonomy_gate` | No | Always |
| MemoryRetrieval | `memory_retrieval` | No | Always |
| GapAnalysis | `gap_analysis` | No | Always |
| SSMUpdate | `ssm_update` | No | Always |
| OpenSourceCompare | `open_source_compare` | Yes | `frequency % 10 == 0` |
| SelfEditGen → BoundedEdit → ApplyEdits | `self_edit_chain` | No | Ownership permits |
| RewardCalc | `reward_calc` | No | Always |
| ValidationGate | `validation_gate` | No | Always |
| GWTAbsob → Stats → Harness → TaskAffinity → KQ | `post_validation_chain` | No | Validation passed |
| RollbackDecision | `rollback` | No | Validation failed |
| RejectedFeedback | `rejected_feedback` | No | Rollback triggered |
| ChampionCompare → BankStorage → HypercubeOptimize | `storage_chain` | No | Always |
| E8Experiment | `e8_experiment` | Yes | `frequency % 5 == 0` |
| EpochSlowUpdate | `epoch_slow` | No | `frequency % 20 == 0` |
| SecurityScan | `security_scan` | No | `frequency % 1 == 0` (always) |
| SessionDistill → ConversationDistill | `distill_chain` | No | `frequency % 3 == 0` |
| AgingDiagnosis | `aging` | No | `frequency % 5 == 0` |
| EmbeddingRefresh | `embedding_refresh` | Yes | `frequency % 10 == 0` |

### Parallel Execution Groups

```rust
// These nodes run in parallel during the same superstep
fn build_seal_graph() -> StateGraph<SealState> {
    let mut g = StateGraph::new(InMemoryCheckpointStore::new());

    // Add all stages as nodes...

    // After reward_calc, fan out to parallel checks
    g.add_edge(unconditional("reward_calc", "validation_gate"));
    g.add_edge(unconditional("reward_calc", "open_source_compare"));
    g.add_edge(unconditional("reward_calc", "e8_experiment"));
    g.add_edge(unconditional("reward_calc", "security_scan"));

    // Sync barrier: all parallel branches must complete before continuing
    g.add_sync_barrier(vec!["validation_gate", "open_source_compare",
                            "e8_experiment", "security_scan"]);

    // After barrier, route based on validation result
    g.add_conditional("validation_gate", "post_validation_chain",
        |s| s.validation_passed);
    g.add_conditional("validation_gate", "rollback",
        |s| !s.validation_passed && s.rollback_eligible);

    g
}
```

---

## 6. MCP Tools for Graph Observability

Expose graph state as MCP tools for real-time debugging.

```mcp
# Tool: graph_get_state
# Returns current graph execution state as JSON
Tool: graph_get_state
  Arguments: {}
  Returns: {
    "step": 42,
    "current_nodes": ["verify"],
    "state_summary": { ... },
    "checkpoints": [10, 20, 30, 40, 42]
  }

# Tool: graph_step_back
# Roll state back to previous checkpoint (time travel)
Tool: graph_step_back
  Arguments: { "to_step": 40 }
  Returns: { "status": "ok", "restored_step": 40 }

# Tool: graph_visualize
# Generate Mermaid diagram of full graph
Tool: graph_visualize
  Arguments: { "format": "mermaid" }
  Returns: { "diagram": "graph TD\n  entry-->analyze\n  ..." }
```

---

## 7. State Schema Design Guidelines

Following LangGraph's TypedDict/Pydantic pattern, but in Rust:

```rust
/// Every state must implement this trait for graph orchestration
pub trait GraphState: Clone + Serialize + DeserializeOwned + Send + 'static {
    fn state_keys() -> Vec<String>;
    fn get_field(&self, key: &str) -> Option<serde_json::Value>;
    fn set_field(&mut self, key: &str, value: serde_json::Value);
}

/// FullReasoningState is the primary state for E8 graph
impl GraphState for FullReasoningState {
    fn state_keys() -> Vec<String> {
        vec![
            "current_task".into(),
            "current_mode".into(),
            "error_count".into(),
            "last_response".into(),
            "last_error".into(),
            "messages".into(),
            "specialist".into(),
            "reasoning_path".into(),
        ]
    }

    fn get_field(&self, key: &str) -> Option<serde_json::Value> {
        match key {
            "current_task" => Some(serde_json::json!(self.current_task)),
            "current_mode" => Some(serde_json::json!(self.current_mode)),
            "error_count" => Some(serde_json::json!(self.error_count)),
            "last_response" => Some(serde_json::json!(self.last_response)),
            "last_error" => Some(serde_json::json!(self.last_error)),
            "messages" => {
                let msgs: Vec<String> = self.message_history.iter()
                    .map(|m| format!("{:?}", m)).collect();
                Some(serde_json::json!(msgs))
            }
            _ => None,
        }
    }

    fn set_field(&mut self, key: &str, value: serde_json::Value) {
        match key {
            "error_count" => self.error_count = value.as_u64().unwrap_or(0) as usize,
            "last_response" => self.last_response = value.as_str().map(String::from),
            // ... etc
            _ => {}
        }
    }
}
```

---

## 8. Integration Points

| Module | Integration |
|--------|------------|
| `nt_core_e8` (L4) | `reason()` wrapped as `GraphNode` handler. E8 state = `GraphState` |
| `nt_core_gwt` (L5) | `SuperstepEvent` → GWT resonance (checkpoint written, error, handoff) |
| `nt_mind_seal` (L8) | 27-stage pipeline rebuilt as sub-graph; parallel execution groups |
| `nt_core_protocol` (L7) | Capability requests can fork sub-graphs via handoff pattern |
| `nt_io_mcp` (L1) | `graph_get_state`, `graph_step_back`, `graph_visualize` MCP tools |
| `nt_memory_kb` (L3) | Checkpoint store uses same SQLite infra; graph metadata stored as KnowledgeNodes |
| `nt_core_observer` (L9) | TurkeyScientist observes graph execution via `SuperstepEvent` stream |

---

## 9. Implementation Plan

### Phase 1: Core Types + StateGraph + Superstep Executor (3 days)
- `StateGraph<S>`, `GraphNode<S>`, `GraphEdge<S>`, `Reducer<S>` traits
- `SuperstepConfig`, `SuperstepEvent`
- `find_eligibile_nodes()` with conditional edge evaluation
- `run()` with parallel execution via `rayon::par_iter()`
- Unit tests: linear graph, branch graph, parallel graph, conditional routing

### Phase 2: Reducer Pattern + Checkpoint Store (2 days)
- `AppendReducer`, `LastWriteWinsReducer`, `SumReducer`, `MaxReducer`
- `SqliteCheckpointStore` with WAL mode
- Crash recovery: load + replay from checkpoint
- Time-travel: `load_step(n)` → restore and re-run
- Tests: crash during execution, recovery produces identical result, checkpoint pruning

### Phase 3: E8 Integration + SEAL Migration (3 days)
- `create_e8_node()` wrapper
- `build_reasoning_graph()` factory with conditional routing (error retry, specialist dispatch)
- SEAL 27-stage → sub-graph with parallel execution groups
- Handoff pattern for sub-graph composition
- Integration tests: E8 graph matches existing `reason()` output

### Phase 4: MCP Tools + Observability (2 days)
- `graph_get_state`, `graph_step_back`, `graph_visualize` MCP tools
- GWT event bridge (`SuperstepEvent` → `GlobalWorkspace::broadcast`)
- Graph visualization as Mermaid/Graphviz output
- CLI `/graph status`, `/graph step-back`, `/graph viz` commands
- Performance benchmarks: superstep overhead < 1ms for 20-node graph

---

## 10. Edge Cases & Safety

1. **Cyclic graphs**: `max_steps` prevents infinite loops. Default 100. Configurable.
2. **Node failure**: Handler returns `Result<S, GraphError>`. Graph routes to error handler nodes (like `fallback`) via conditional edges.
3. **Checkpoint corruption**: `bincode::deserialize` returns `Err` → fall back to previous checkpoint. If no valid checkpoint, start fresh.
4. **Parallel write conflicts**: Reducers resolve deterministically. No two nodes write to the same reducer key in the same superstep (enforced by `find_eligibile_nodes`).
5. **State explosion**: Checkpoints contain full serialized state. Implement `prune_before(step)` called after every 10 supersteps to keep only last 20 checkpoints.
6. **Deadlock detection**: If no node is eligible AND no exit is reached, graph returns `GraphError::Deadlock`.
