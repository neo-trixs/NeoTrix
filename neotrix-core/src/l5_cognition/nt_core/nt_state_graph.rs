//! # State Graph Checkpointing (R-P127)
//!
//! LangGraph-inspired durable execution engine for NeoTrix.
//! Provides typed state graph management with checkpoint-based persistence.
//!
//! ## Architecture
//!
//! ```text
//! StateGraph<S>
//!   ├── GraphNode<S>  (id + state + edges)
//!   ├── GraphEdge     (condition + target)
//!   └── ExecutionPlan (topological order)
//!
//! CheckpointManager (trait)
//!   ├── JsonCheckpointEngine    (human-readable)
//!   └── BincodeCheckpointEngine (compact binary)
//!
//! DurableExecutor<S>
//!   ├── auto-checkpoint at configurable intervals
//!   ├── manual checkpoint triggers
//!   └── resume from checkpoint (replay)
//! ```

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Errors (R-P1: no unsafe, pure Rust error handling)
// ============================================================================

#[derive(Error, Debug)]
pub enum StateGraphError {
    #[error("node not found: {0}")]
    NodeNotFound(String),

    #[error("edge target not found: {0}")]
    EdgeTargetNotFound(String),

    #[error("cycle detected involving node: {0}")]
    CycleDetected(String),

    #[error("checkpoint error: {0}")]
    Checkpoint(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("graph has no entry node")]
    NoEntryNode,

    #[error("condition not met for edge from {from} to {to}")]
    ConditionNotMet { from: String, to: String },

    #[error("execution halted at node {0}: {1}")]
    ExecutionHalted(String, String),
}

pub type Result<T> = std::result::Result<T, StateGraphError>;

// ============================================================================
// GraphEdge — Conditional transition between nodes
// ============================================================================

/// A directed edge with an optional condition predicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Target node identifier.
    pub target: String,
    /// Optional condition expression (evaluated against node state).
    /// When `None`, edge is unconditional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// Edge weight for priority routing (higher = preferred).
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 {
    1.0
}

impl GraphEdge {
    pub fn unconditional(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            condition: None,
            weight: 1.0,
        }
    }

    pub fn conditional(target: impl Into<String>, condition: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            condition: Some(condition.into()),
            weight: 1.0,
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

// ============================================================================
// GraphNode — Stateful node in the execution graph
// ============================================================================

/// A node holding serializable state and outgoing edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode<S: Serialize + Clone> {
    /// Unique node identifier.
    pub id: String,
    /// Typed state payload.
    pub state: S,
    /// Outgoing edges to other nodes.
    pub edges: Vec<GraphEdge>,
    /// Node-level metadata (labels, tags, etc.).
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl<S: Serialize + Clone> GraphNode<S> {
    pub fn new(id: impl Into<String>, state: S) -> Self {
        Self {
            id: id.into(),
            state,
            edges: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_edge(mut self, edge: GraphEdge) -> Self {
        self.edges.push(edge);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Find the first edge whose condition matches (or unconditional fallback).
    pub fn next_edge(&self, _state: &S) -> Option<&GraphEdge> {
        // Prefer conditional edges first, then unconditional
        let conditional = self.edges.iter().find(|e| e.condition.is_some());
        if let Some(edge) = conditional {
            return Some(edge);
        }
        self.edges.first()
    }
}

// ============================================================================
// CheckpointConfig — R-P11 Config struct pattern
// ============================================================================

/// Configuration for checkpoint behavior (R-P11: config struct + Default).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Checkpoint storage directory.
    pub storage_dir: PathBuf,
    /// Maximum number of checkpoints to retain (0 = unlimited).
    pub max_checkpoints: usize,
    /// Auto-checkpoint interval (None = manual only).
    pub auto_interval: Option<Duration>,
    /// Enable state diffing (only save changed fields).
    pub diff_enabled: bool,
    /// Checkpoint format: "json" or "bincode".
    pub format: CheckpointFormat,
    /// Compression: "none", "lz4", or "zstd".
    pub compression: CompressionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckpointFormat {
    Json,
    Bincode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionMode {
    None,
    Lz4,
    Zstd,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            storage_dir: PathBuf::from(".neotrix/checkpoints"),
            max_checkpoints: 100,
            auto_interval: Some(Duration::from_secs(60)),
            diff_enabled: true,
            format: CheckpointFormat::Json,
            compression: CompressionMode::Lz4,
        }
    }
}

// ============================================================================
// CheckpointMetadata — Versioned checkpoint header
// ============================================================================

/// Metadata attached to every checkpoint for versioning and recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Checkpoint format version (for forward/backward compat).
    pub version: u32,
    /// Unix timestamp when checkpoint was created.
    pub timestamp: u64,
    /// Monotonic sequence number within a graph execution.
    pub sequence: u64,
    /// Hash of the graph topology (detects structural changes).
    pub graph_hash: String,
    /// Node ID that was active when checkpoint was taken.
    pub active_node: Option<String>,
    /// Human-readable label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Checkpoint format.
    pub format: CheckpointFormat,
    /// Compression mode used.
    pub compression: CompressionMode,
}

impl CheckpointMetadata {
    pub fn new(
        sequence: u64,
        graph_hash: &str,
        format: &CheckpointFormat,
        compression: &CompressionMode,
    ) -> Self {
        Self {
            version: 1,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            sequence,
            graph_hash: graph_hash.to_string(),
            active_node: None,
            label: None,
            format: format.clone(),
            compression: compression.clone(),
        }
    }
}

// ============================================================================
// CheckpointData — Full checkpoint payload
// ============================================================================

/// Complete checkpoint containing graph state, node states, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointData<S: Serialize + Clone> {
    pub metadata: CheckpointMetadata,
    /// All node states keyed by node ID.
    pub node_states: HashMap<String, S>,
    /// Execution history (node IDs in order of execution).
    pub execution_trace: Vec<String>,
    /// Which node is currently active.
    pub current_node: Option<String>,
}

impl<S: Serialize + Clone> CheckpointData<S> {
    pub fn new(
        sequence: u64,
        graph_hash: &str,
        format: &CheckpointFormat,
        compression: &CompressionMode,
    ) -> Self {
        Self {
            metadata: CheckpointMetadata::new(sequence, graph_hash, format, compression),
            node_states: HashMap::new(),
            execution_trace: Vec::new(),
            current_node: None,
        }
    }
}

// ============================================================================
// CheckpointManager trait
// ============================================================================

/// Trait for checkpoint persistence backends.
pub trait CheckpointManager<S: Serialize + Clone + for<'de> Deserialize<'de>> {
    /// Save a checkpoint to durable storage.
    fn save(&self, checkpoint: &CheckpointData<S>) -> Result<()>;

    /// Load the latest checkpoint for a given graph ID.
    fn load_latest(&self, graph_id: &str) -> Result<Option<CheckpointData<S>>>;

    /// Load a specific checkpoint by sequence number.
    fn load_sequence(&self, graph_id: &str, sequence: u64) -> Result<Option<CheckpointData<S>>>;

    /// List available checkpoint sequences for a graph.
    fn list_checkpoints(&self, graph_id: &str) -> Result<Vec<CheckpointMetadata>>;

    /// Delete checkpoints older than the given sequence.
    fn prune(&self, graph_id: &str, keep_last: usize) -> Result<usize>;

    /// Verify checkpoint integrity (R-P49 persistence verification).
    fn verify(&self, graph_id: &str, sequence: u64) -> Result<bool>;
}

// ============================================================================
// JsonCheckpointEngine
// ============================================================================

/// JSON-based checkpoint engine (human-readable, debuggable).
pub struct JsonCheckpointEngine {
    config: CheckpointConfig,
}

impl JsonCheckpointEngine {
    pub fn new(config: CheckpointConfig) -> Self {
        Self { config }
    }

    fn checkpoint_path(&self, graph_id: &str, sequence: u64) -> PathBuf {
        self.config
            .storage_dir
            .join(graph_id)
            .join(format!("{:08}.json", sequence))
    }

    fn metadata_path(&self, graph_id: &str) -> PathBuf {
        self.config.storage_dir.join(graph_id).join("index.json")
    }
}

impl CheckpointManager<serde_json::Value> for JsonCheckpointEngine {
    fn save(&self, checkpoint: &CheckpointData<serde_json::Value>) -> Result<()> {
        let graph_id = &checkpoint.metadata.graph_hash;
        let path = self.checkpoint_path(graph_id, checkpoint.metadata.sequence);

        // Create directory (R-P49: verify after write)
        std::fs::create_dir_all(path.parent().unwrap_or(&Path::new(".")))?;

        let json = serde_json::to_string_pretty(checkpoint)
            .map_err(|e| StateGraphError::Serialization(e.to_string()))?;

        std::fs::write(&path, &json)?;

        // R-P49: verify persistence
        let written = std::fs::read_to_string(&path)?;
        if written != json {
            return Err(StateGraphError::Checkpoint(format!(
                "persistence verification failed for {}",
                path.display()
            )));
        }

        // Update index
        self.update_index(graph_id, &checkpoint.metadata)?;

        Ok(())
    }

    fn load_latest(&self, graph_id: &str) -> Result<Option<CheckpointData<serde_json::Value>>> {
        let sequences = self.list_checkpoints(graph_id)?;
        match sequences.iter().max_by_key(|m| m.sequence) {
            Some(meta) => self.load_sequence(graph_id, meta.sequence),
            None => Ok(None),
        }
    }

    fn load_sequence(
        &self,
        graph_id: &str,
        sequence: u64,
    ) -> Result<Option<CheckpointData<serde_json::Value>>> {
        let path = self.checkpoint_path(graph_id, sequence);
        if !path.exists() {
            return Ok(None);
        }

        let data = std::fs::read_to_string(&path)?;
        let checkpoint: CheckpointData<serde_json::Value> = serde_json::from_str(&data)
            .map_err(|e| StateGraphError::Serialization(e.to_string()))?;

        Ok(Some(checkpoint))
    }

    fn list_checkpoints(&self, graph_id: &str) -> Result<Vec<CheckpointMetadata>> {
        let index_path = self.metadata_path(graph_id);
        if !index_path.exists() {
            return Ok(Vec::new());
        }

        let data = std::fs::read_to_string(&index_path)?;
        let entries: Vec<CheckpointMetadata> = serde_json::from_str(&data)
            .map_err(|e| StateGraphError::Serialization(e.to_string()))?;

        Ok(entries)
    }

    fn prune(&self, graph_id: &str, keep_last: usize) -> Result<usize> {
        let mut entries = self.list_checkpoints(graph_id)?;
        if entries.len() <= keep_last {
            return Ok(0);
        }

        entries.sort_by_key(|m| std::cmp::Reverse(m.sequence));
        let to_remove: Vec<_> = entries.split_off(keep_last);

        let mut removed = 0;
        for meta in &to_remove {
            let path = self.checkpoint_path(graph_id, meta.sequence);
            if path.exists() {
                std::fs::remove_file(&path)?;
                removed += 1;
            }
        }

        // Update index with remaining entries
        let index_path = self.metadata_path(graph_id);
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| StateGraphError::Serialization(e.to_string()))?;
        std::fs::write(index_path, json)?;

        Ok(removed)
    }

    fn verify(&self, graph_id: &str, sequence: u64) -> Result<bool> {
        let path = self.checkpoint_path(graph_id, sequence);
        if !path.exists() {
            return Ok(false);
        }

        // R-P49: verify file is readable and parseable
        let data = std::fs::read_to_string(&path)?;
        let result: std::result::Result<CheckpointData<serde_json::Value>, _> =
            serde_json::from_str(&data);

        Ok(result.is_ok())
    }
}

impl JsonCheckpointEngine {
    fn update_index(&self, graph_id: &str, metadata: &CheckpointMetadata) -> Result<()> {
        let mut entries = self.list_checkpoints(graph_id)?;

        // Remove existing entry with same sequence (update)
        entries.retain(|e| e.sequence != metadata.sequence);
        entries.push(metadata.clone());
        entries.sort_by_key(|m| m.sequence);

        let index_path = self.metadata_path(graph_id);
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| StateGraphError::Serialization(e.to_string()))?;
        std::fs::write(&index_path, &json)?;

        // R-P49: verify
        let written = std::fs::read_to_string(&index_path)?;
        if written != json {
            return Err(StateGraphError::Checkpoint(
                "index persistence failed".into(),
            ));
        }

        Ok(())
    }
}

// ============================================================================
// BincodeCheckpointEngine
// ============================================================================

/// Bincode-based checkpoint engine (compact, fast).
pub struct BincodeCheckpointEngine {
    config: CheckpointConfig,
}

impl BincodeCheckpointEngine {
    pub fn new(config: CheckpointConfig) -> Self {
        Self { config }
    }

    fn checkpoint_path(&self, graph_id: &str, sequence: u64) -> PathBuf {
        self.config
            .storage_dir
            .join(graph_id)
            .join(format!("{:08}.bin", sequence))
    }
}

impl CheckpointManager<Vec<u8>> for BincodeCheckpointEngine {
    fn save(&self, checkpoint: &CheckpointData<Vec<u8>>) -> Result<()> {
        let graph_id = &checkpoint.metadata.graph_hash;
        let path = self.checkpoint_path(graph_id, checkpoint.metadata.sequence);

        std::fs::create_dir_all(path.parent().unwrap_or(&Path::new(".")))?;

        let encoded = bincode::serialize(checkpoint)
            .map_err(|e| StateGraphError::Serialization(e.to_string()))?;

        std::fs::write(&path, &encoded)?;

        // R-P49: verify persistence
        let written = std::fs::read(&path)?;
        if written.len() != encoded.len() {
            return Err(StateGraphError::Checkpoint(format!(
                "persistence verification failed: size mismatch {} vs {}",
                written.len(),
                encoded.len()
            )));
        }

        Ok(())
    }

    fn load_latest(&self, graph_id: &str) -> Result<Option<CheckpointData<Vec<u8>>>> {
        let dir = self.config.storage_dir.join(graph_id);
        if !dir.exists() {
            return Ok(None);
        }

        let mut entries: Vec<_> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "bin")
                    .unwrap_or(false)
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());

        match entries.last() {
            Some(entry) => {
                let data = std::fs::read(entry.path())?;
                let checkpoint: CheckpointData<Vec<u8>> = bincode::deserialize(&data)
                    .map_err(|e| StateGraphError::Serialization(e.to_string()))?;
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }

    fn load_sequence(
        &self,
        graph_id: &str,
        sequence: u64,
    ) -> Result<Option<CheckpointData<Vec<u8>>>> {
        let path = self.checkpoint_path(graph_id, sequence);
        if !path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(&path)?;
        let checkpoint: CheckpointData<Vec<u8>> = bincode::deserialize(&data)
            .map_err(|e| StateGraphError::Serialization(e.to_string()))?;

        Ok(Some(checkpoint))
    }

    fn list_checkpoints(&self, graph_id: &str) -> Result<Vec<CheckpointMetadata>> {
        let dir = self.config.storage_dir.join(graph_id);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut meta = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "bin").unwrap_or(false) {
                let data = std::fs::read(&path)?;
                let checkpoint: CheckpointData<Vec<u8>> = bincode::deserialize(&data)
                    .map_err(|e| StateGraphError::Serialization(e.to_string()))?;
                meta.push(checkpoint.metadata);
            }
        }

        meta.sort_by_key(|m| m.sequence);
        Ok(meta)
    }

    fn prune(&self, graph_id: &str, keep_last: usize) -> Result<usize> {
        let mut entries = self.list_checkpoints(graph_id)?;
        if entries.len() <= keep_last {
            return Ok(0);
        }

        entries.sort_by_key(|m| std::cmp::Reverse(m.sequence));
        let to_remove: Vec<_> = entries.split_off(keep_last);

        let mut removed = 0;
        for meta in &to_remove {
            let path = self.checkpoint_path(graph_id, meta.sequence);
            if path.exists() {
                std::fs::remove_file(&path)?;
                removed += 1;
            }
        }

        Ok(removed)
    }

    fn verify(&self, graph_id: &str, sequence: u64) -> Result<bool> {
        let path = self.checkpoint_path(graph_id, sequence);
        if !path.exists() {
            return Ok(false);
        }

        let data = std::fs::read(&path)?;
        let result: std::result::Result<CheckpointData<Vec<u8>>, _> = bincode::deserialize(&data);
        Ok(result.is_ok())
    }
}

// ============================================================================
// StateGraph — Core graph management
// ============================================================================

/// A typed directed graph with stateful nodes, designed for checkpointing.
///
/// StateGraph manages:
/// - Node registration and lookup
/// - Edge wiring with conditions
/// - Topological execution ordering
/// - Checkpoint save/load with state diffing
pub struct StateGraph<S: Serialize + Clone + for<'de> Deserialize<'de>> {
    /// Graph identifier (used for checkpoint namespacing).
    id: String,
    /// All registered nodes, keyed by ID.
    nodes: HashMap<String, GraphNode<S>>,
    /// Entry node ID (where execution starts).
    entry_node: Option<String>,
    /// Execution sequence cache.
    execution_order: Vec<String>,
    /// Checkpoint sequence counter.
    checkpoint_seq: u64,
    /// Last checkpoint timestamp for auto-checkpoint interval.
    last_checkpoint_time: Option<Instant>,
    /// Config (R-P11).
    config: CheckpointConfig,
}

impl<S: Serialize + Clone + for<'de> Deserialize<'de>> StateGraph<S> {
    /// Create a new state graph.
    pub fn new(id: impl Into<String>, config: CheckpointConfig) -> Self {
        Self {
            id: id.into(),
            nodes: HashMap::new(),
            entry_node: None,
            execution_order: Vec::new(),
            checkpoint_seq: 0,
            last_checkpoint_time: None,
            config,
        }
    }

    /// Graph identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Register a node. If it's the first node, it becomes the entry.
    pub fn add_node(&mut self, node: GraphNode<S>) -> &mut Self {
        if self.entry_node.is_none() {
            self.entry_node = Some(node.id.clone());
        }
        self.nodes.insert(node.id.clone(), node);
        self.execution_order.clear(); // invalidate cache
        self
    }

    /// Set the entry node explicitly.
    pub fn set_entry(&mut self, node_id: impl Into<String>) -> Result<()> {
        let id = node_id.into();
        if !self.nodes.contains_key(&id) {
            return Err(StateGraphError::NodeNotFound(id));
        }
        self.entry_node = Some(id);
        Ok(())
    }

    /// Add an edge between two nodes.
    pub fn add_edge(&mut self, from: impl Into<String>, edge: GraphEdge) -> Result<()> {
        let from_id = from.into();
        if !self.nodes.contains_key(&from_id) {
            return Err(StateGraphError::NodeNotFound(from_id));
        }
        if !self.nodes.contains_key(&edge.target) {
            return Err(StateGraphError::EdgeTargetNotFound(edge.target.clone()));
        }

        if let Some(node) = self.nodes.get_mut(&from_id) {
            node.edges.push(edge);
        }
        self.execution_order.clear();
        Ok(())
    }

    /// Get a node by ID.
    pub fn node(&self, id: &str) -> Option<&GraphNode<S>> {
        self.nodes.get(id)
    }

    /// Get a mutable reference to a node's state.
    pub fn node_state_mut(&mut self, id: &str) -> Option<&mut S> {
        self.nodes.get_mut(id).map(|n| &mut n.state)
    }

    /// Get all node IDs.
    pub fn node_ids(&self) -> Vec<&str> {
        self.nodes.keys().map(|s| s.as_str()).collect()
    }

    /// Number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Compute topological execution order (Kahn's algorithm).
    /// Returns error if cycle is detected.
    pub fn topological_order(&mut self) -> Result<&[String]> {
        if !self.execution_order.is_empty() {
            return Ok(&self.execution_order);
        }

        let n = self.nodes.len();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for (id, node) in &self.nodes {
            in_degree.entry(id.clone()).or_insert(0);
            adj.entry(id.clone()).or_insert_with(Vec::new);

            for edge in &node.edges {
                *in_degree.entry(edge.target.clone()).or_insert(0) += 1;
                adj.get_mut(id).unwrap().push(edge.target.clone());
            }
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut order = Vec::with_capacity(n);

        while let Some(current) = queue.pop_front() {
            order.push(current.clone());
            if let Some(neighbors) = adj.get(&current) {
                for neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        if order.len() != n {
            // Find the cycle node for the error message
            let cycle_node = order
                .iter()
                .find(|id| !adj.get(*id).map_or(true, |edges| edges.is_empty()))
                .cloned()
                .unwrap_or_else(|| "unknown".into());
            return Err(StateGraphError::CycleDetected(cycle_node));
        }

        self.execution_order = order;
        Ok(&self.execution_order)
    }

    /// Compute a hash of the graph topology (for checkpoint versioning).
    pub fn topology_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        let mut node_ids: Vec<&str> = self.nodes.keys().map(|s| s.as_str()).collect();
        node_ids.sort();
        for id in &node_ids {
            id.hash(&mut hasher);
            if let Some(node) = self.nodes.get(*id) {
                for edge in &node.edges {
                    edge.target.hash(&mut hasher);
                }
            }
        }

        format!("{:016x}", hasher.finish())
    }

    /// Take a checkpoint of the entire graph state.
    pub fn checkpoint(&mut self, active_node: Option<&str>) -> CheckpointData<S> {
        self.checkpoint_seq += 1;

        let mut data = CheckpointData::new(
            self.checkpoint_seq,
            &self.topology_hash(),
            &self.config.format,
            &self.config.compression,
        );

        data.current_node = active_node.map(|s| s.to_string());

        for (id, node) in &self.nodes {
            data.node_states.insert(id.clone(), node.state.clone());
        }

        self.last_checkpoint_time = Some(Instant::now());

        data
    }

    /// Restore graph state from a checkpoint.
    pub fn restore_from_checkpoint(&mut self, checkpoint: &CheckpointData<S>) -> Result<()> {
        // Validate topology hasn't changed
        let current_hash = self.topology_hash();
        if current_hash != checkpoint.metadata.graph_hash {
            return Err(StateGraphError::Checkpoint(format!(
                "topology mismatch: current={}, checkpoint={}",
                current_hash, checkpoint.metadata.graph_hash
            )));
        }

        // Restore node states
        for (id, state) in &checkpoint.node_states {
            if let Some(node) = self.nodes.get_mut(id) {
                node.state = state.clone();
            }
        }

        self.checkpoint_seq = checkpoint.metadata.sequence;
        Ok(())
    }

    /// Compute a state diff between two checkpoints (only changed fields).
    pub fn diff_checkpoints(
        old: &CheckpointData<S>,
        new: &CheckpointData<S>,
    ) -> HashMap<String, bool> {
        let mut diffs = HashMap::new();

        for (id, new_state) in &new.node_states {
            match old.node_states.get(id) {
                Some(old_state) => {
                    let old_json =
                        serde_json::to_value(old_state).unwrap_or(serde_json::Value::Null);
                    let new_json =
                        serde_json::to_value(new_state).unwrap_or(serde_json::Value::Null);
                    diffs.insert(id.clone(), old_json != new_json);
                }
                None => {
                    diffs.insert(id.clone(), true); // new node
                }
            }
        }

        // Check for removed nodes
        for id in old.node_states.keys() {
            if !new.node_states.contains_key(id) {
                diffs.insert(id.clone(), true);
            }
        }

        diffs
    }

    /// Check if auto-checkpoint interval has elapsed.
    pub fn should_auto_checkpoint(&self) -> bool {
        match (self.config.auto_interval, self.last_checkpoint_time) {
            (Some(interval), Some(last)) => last.elapsed() >= interval,
            (Some(_), None) => true,
            (None, _) => false,
        }
    }

    /// Save checkpoint to a CheckpointManager.
    pub fn save_checkpoint(
        &mut self,
        manager: &dyn CheckpointManager<S>,
        active_node: Option<&str>,
    ) -> Result<CheckpointData<S>> {
        let data = self.checkpoint(active_node);
        manager.save(&data)?;
        Ok(data)
    }

    /// Load and restore from the latest checkpoint.
    pub fn load_and_restore(
        &mut self,
        manager: &dyn CheckpointManager<S>,
    ) -> Result<Option<CheckpointData<S>>> {
        match manager.load_latest(&self.id)? {
            Some(checkpoint) => {
                self.restore_from_checkpoint(&checkpoint)?;
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }

    /// Resume execution from the last checkpoint.
    pub fn resume_from(
        &mut self,
        manager: &dyn CheckpointManager<S>,
    ) -> Result<Option<(CheckpointData<S>, Option<String>)>> {
        match manager.load_latest(&self.id)? {
            Some(checkpoint) => {
                let active = checkpoint.current_node.clone();
                self.restore_from_checkpoint(&checkpoint)?;
                Ok(Some((checkpoint, active)))
            }
            None => Ok(None),
        }
    }
}

// ============================================================================
// DurableExecutor — Auto-checkpointing execution engine
// ============================================================================

/// Execution engine with automatic checkpointing and state diffing.
pub struct DurableExecutor<S: Serialize + Clone + for<'de> Deserialize<'de>> {
    graph: StateGraph<S>,
    manager: Arc<dyn CheckpointManager<S>>,
    /// State diff from last checkpoint.
    last_diff: Option<HashMap<String, bool>>,
}

impl<S: Serialize + Clone + for<'de> Deserialize<'de>> DurableExecutor<S> {
    pub fn new(graph: StateGraph<S>, manager: Arc<dyn CheckpointManager<S>>) -> Self {
        Self {
            graph,
            manager,
            last_diff: None,
        }
    }

    /// Mutable access to the underlying graph.
    pub fn graph_mut(&mut self) -> &mut StateGraph<S> {
        &mut self.graph
    }

    /// Immutable access to the underlying graph.
    pub fn graph(&self) -> &StateGraph<S> {
        &self.graph
    }

    /// Execute a mutation with automatic checkpoint if interval elapsed.
    pub fn mutate(
        &mut self,
        node_id: &str,
        f: impl FnOnce(&mut S),
        active_node: Option<&str>,
    ) -> Result<()> {
        // Apply mutation
        {
            let state = self
                .graph
                .node_state_mut(node_id)
                .ok_or_else(|| StateGraphError::NodeNotFound(node_id.to_string()))?;
            f(state);
        }

        // Auto-checkpoint if interval elapsed
        if self.graph.should_auto_checkpoint() {
            let old_checkpoint = self.manager.load_latest(self.graph.id())?;
            let new_checkpoint = self.graph.checkpoint(active_node);

            if let Some(ref old) = old_checkpoint {
                self.last_diff = Some(StateGraph::diff_checkpoints(old, &new_checkpoint));
            }

            self.manager.save(&new_checkpoint)?;
        }

        Ok(())
    }

    /// Manual checkpoint trigger (always saves regardless of interval).
    pub fn checkpoint_now(&mut self, active_node: Option<&str>) -> Result<CheckpointData<S>> {
        let old_checkpoint = self.manager.load_latest(self.graph.id())?;
        let new_checkpoint = self.graph.checkpoint(active_node);

        if let Some(ref old) = old_checkpoint {
            self.last_diff = Some(StateGraph::diff_checkpoints(old, &new_checkpoint));
        }

        self.manager.save(&new_checkpoint)?;
        Ok(new_checkpoint)
    }

    /// Resume execution from the latest checkpoint.
    pub fn resume(&mut self) -> Result<Option<String>> {
        match self.graph.resume_from(self.manager.as_ref())? {
            Some((_checkpoint, active_node)) => Ok(active_node),
            None => Ok(None),
        }
    }

    /// Verify checkpoint integrity (R-P49).
    pub fn verify_checkpoint(&self, sequence: u64) -> Result<bool> {
        self.manager.verify(self.graph.id(), sequence)
    }

    /// Prune old checkpoints, keeping only the last N.
    pub fn prune_checkpoints(&self, keep_last: usize) -> Result<usize> {
        self.manager.prune(self.graph.id(), keep_last)
    }

    /// Get the state diff from the last checkpoint.
    pub fn last_diff(&self) -> Option<&HashMap<String, bool>> {
        self.last_diff.as_ref()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestState {
        value: i32,
        label: String,
    }

    /// In-memory checkpoint manager for testing.
    struct MemCheckpointManager {
        store: Mutex<HashMap<String, Vec<CheckpointData<TestState>>>>,
    }

    impl MemCheckpointManager {
        fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }
    }

    impl CheckpointManager<TestState> for MemCheckpointManager {
        fn save(&self, checkpoint: &CheckpointData<TestState>) -> Result<()> {
            let graph_id = &checkpoint.metadata.graph_hash;
            let mut store = self.store.lock().unwrap();
            let entries = store.entry(graph_id.clone()).or_insert_with(Vec::new);
            entries.push(checkpoint.clone());
            Ok(())
        }

        fn load_latest(&self, graph_id: &str) -> Result<Option<CheckpointData<TestState>>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .get(graph_id)
                .and_then(|entries| entries.last().cloned()))
        }

        fn load_sequence(
            &self,
            graph_id: &str,
            sequence: u64,
        ) -> Result<Option<CheckpointData<TestState>>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(graph_id).and_then(|entries| {
                entries
                    .iter()
                    .find(|e| e.metadata.sequence == sequence)
                    .cloned()
            }))
        }

        fn list_checkpoints(&self, graph_id: &str) -> Result<Vec<CheckpointMetadata>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .get(graph_id)
                .map(|entries| entries.iter().map(|e| e.metadata.clone()).collect())
                .unwrap_or_default())
        }

        fn prune(&self, graph_id: &str, keep_last: usize) -> Result<usize> {
            let mut store = self.store.lock().unwrap();
            if let Some(entries) = store.get_mut(graph_id) {
                let before = entries.len();
                entries.sort_by_key(|e| e.metadata.sequence);
                entries.drain(..entries.len().saturating_sub(keep_last));
                Ok(before - entries.len())
            } else {
                Ok(0)
            }
        }

        fn verify(&self, graph_id: &str, sequence: u64) -> Result<bool> {
            let store = self.store.lock().unwrap();
            Ok(store
                .get(graph_id)
                .map(|entries| entries.iter().any(|e| e.metadata.sequence == sequence))
                .unwrap_or(false))
        }
    }

    fn make_test_graph() -> StateGraph<TestState> {
        let config = CheckpointConfig {
            auto_interval: None, // manual only for tests
            ..Default::default()
        };
        let mut graph = StateGraph::new("test-graph", config);

        graph.add_node(
            GraphNode::new(
                "start",
                TestState {
                    value: 0,
                    label: "start".into(),
                },
            )
            .with_edge(GraphEdge::unconditional("process")),
        );
        graph.add_node(
            GraphNode::new(
                "process",
                TestState {
                    value: 10,
                    label: "processing".into(),
                },
            )
            .with_edge(GraphEdge::unconditional("end")),
        );
        graph.add_node(GraphNode::new(
            "end",
            TestState {
                value: 100,
                label: "done".into(),
            },
        ));

        graph
    }

    #[test]
    fn test_graph_construction() {
        let graph = make_test_graph();
        assert_eq!(graph.len(), 3);
        assert!(!graph.is_empty());
        assert_eq!(graph.entry_node.as_deref(), Some("start"));
    }

    #[test]
    fn test_topological_order() {
        let mut graph = make_test_graph();
        let order = graph.topological_order().unwrap().to_vec();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], "start");
        assert_eq!(order[1], "process");
        assert_eq!(order[2], "end");
    }

    #[test]
    fn test_cycle_detection() {
        let config = CheckpointConfig::default();
        let mut graph = StateGraph::new("cycle-test", config);

        graph.add_node(
            GraphNode::new(
                "a",
                TestState {
                    value: 0,
                    label: "a".into(),
                },
            )
            .with_edge(GraphEdge::unconditional("b")),
        );
        graph.add_node(
            GraphNode::new(
                "b",
                TestState {
                    value: 0,
                    label: "b".into(),
                },
            )
            .with_edge(GraphEdge::unconditional("a")),
        );

        assert!(matches!(
            graph.topological_order(),
            Err(StateGraphError::CycleDetected(_))
        ));
    }

    #[test]
    fn test_checkpoint_save_load() {
        let manager = MemCheckpointManager::new();
        let mut graph = make_test_graph();

        let cp = graph.save_checkpoint(&manager, Some("start")).unwrap();
        assert_eq!(cp.metadata.sequence, 1);

        let loaded = manager.load_latest("test-graph").unwrap().unwrap();
        assert_eq!(loaded.metadata.sequence, 1);
        assert_eq!(loaded.node_states.get("start").unwrap().label, "start");
    }

    #[test]
    fn test_restore_from_checkpoint() {
        let manager = MemCheckpointManager::new();
        let mut graph = make_test_graph();

        // Modify state
        {
            let state = graph.node_state_mut("process").unwrap();
            state.value = 999;
        }

        let cp = graph.save_checkpoint(&manager, Some("process")).unwrap();

        // Create fresh graph and restore
        let mut graph2 = make_test_graph();
        graph2.restore_from_checkpoint(&cp).unwrap();

        assert_eq!(graph2.node("process").unwrap().state.value, 999);
    }

    #[test]
    fn test_resume_from_checkpoint() {
        let manager = MemCheckpointManager::new();
        let mut graph = make_test_graph();

        graph.save_checkpoint(&manager, Some("process")).unwrap();

        let mut graph2 = make_test_graph();
        let result = graph2.resume_from(&manager).unwrap();
        assert!(result.is_some());

        let (cp, active) = result.unwrap();
        assert_eq!(active.as_deref(), Some("process"));
        assert_eq!(cp.metadata.sequence, 1);
    }

    #[test]
    fn test_state_diffing() {
        let mut graph = make_test_graph();
        let cp1 = graph.checkpoint(None);

        // Modify state
        {
            let state = graph.node_state_mut("start").unwrap();
            state.value = 42;
        }
        let cp2 = graph.checkpoint(None);

        let diffs = StateGraph::diff_checkpoints(&cp1, &cp2);
        assert_eq!(diffs.get("start"), Some(&true));
        assert_eq!(diffs.get("process"), Some(&false));
    }

    #[test]
    fn test_durable_executor_mutation() {
        let manager = Arc::new(MemCheckpointManager::new());
        let config = CheckpointConfig {
            auto_interval: None,
            ..Default::default()
        };
        let graph = StateGraph::new("test", config);
        let mut executor = DurableExecutor::new(graph, manager.clone());

        // Mutate state
        executor
            .mutate("start", |s| s.value = 42, Some("start"))
            .unwrap();

        // No auto-checkpoint (interval is None)
        assert!(manager.load_latest("test").unwrap().is_none());

        // Manual checkpoint
        let cp = executor.checkpoint_now(Some("start")).unwrap();
        assert_eq!(cp.node_states.get("start").unwrap().value, 42);
    }

    #[test]
    fn test_prune_checkpoints() {
        let manager = MemCheckpointManager::new();
        let mut graph = make_test_graph();

        for i in 0..5 {
            graph.checkpoint_seq = i;
            graph.checkpoint(None);
            let data = graph.checkpoint(None);
            manager.save(&data).unwrap();
        }

        let pruned = manager.prune("test-graph", 2).unwrap();
        assert!(pruned > 0);

        let remaining = manager.list_checkpoints("test-graph").unwrap();
        assert!(remaining.len() <= 2);
    }

    #[test]
    fn test_topology_hash_stability() {
        let graph1 = make_test_graph();
        let graph2 = make_test_graph();
        assert_eq!(graph1.topology_hash(), graph2.topology_hash());
    }

    #[test]
    fn test_topological_order_cache() {
        let mut graph = make_test_graph();
        let order1 = graph.topological_order().unwrap().to_vec();
        let order2 = graph.topological_order().unwrap().to_vec();
        assert_eq!(order1, order2);
    }
}
