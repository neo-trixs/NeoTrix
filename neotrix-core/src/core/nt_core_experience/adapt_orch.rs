use std::collections::{HashMap, HashSet, VecDeque};

use crate::neotrix::nt_act_orchestrator::process::ProcessType;
use crate::neotrix::nt_act_orchestrator::state_graph::{ArtifactNode, ArtifactType, StateGraph};
use crate::neotrix::nt_act_orchestrator::topology_router::TopologyRouter;

pub type HandlerId = String;
pub const VSA_ROUTER_TOP_K: usize = 8;

// ── VSA MoE Router (Kimi-VL inspired) ──

/// A VSA-based handler profile encoding what cognitive state a handler is relevant for.
/// Each profile is a deterministic hash of the handler's name, description, and task type.
#[derive(Debug, Clone)]
pub struct HandlerProfile {
    pub name: String,
    pub cognitive_vsa: Vec<u8>,
    pub task_type: String,
}

#[derive(Debug, Clone)]
pub struct VsaMoERouter {
    /// Handler VSA profiles indexed by handler name
    pub profiles: HashMap<String, HandlerProfile>,
    /// Number of top handlers to select per routing call
    pub top_k: usize,
    /// VSA dimension
    pub vsa_dim: usize,
    /// Routing call tracking
    pub total_routes: u64,
    pub last_routed_handlers: Vec<String>,
}

impl VsaMoERouter {
    pub fn new(vsa_dim: usize) -> Self {
        Self {
            profiles: HashMap::new(),
            top_k: VSA_ROUTER_TOP_K,
            vsa_dim,
            total_routes: 0,
            last_routed_handlers: Vec::new(),
        }
    }

    /// Generate a deterministic VSA profile vector from handler metadata.
    /// Uses a seeded hash so the same handler always maps to the same profile.
    fn generate_profile_vsa(name: &str, task_type: &str, vsa_dim: usize) -> Vec<u8> {
        use std::hash::{DefaultHasher, Hasher};
        let mut hasher = DefaultHasher::new();
        hasher.write(name.as_bytes());
        hasher.write(task_type.as_bytes());
        hasher.write(&[0xAB, 0xCD, 0xEF]);
        let seed = hasher.finish();
        let mut vsa = vec![0u8; vsa_dim];
        let mut rng = seed;
        for byte in vsa.iter_mut() {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *byte = (rng >> 40) as u8;
        }
        vsa
    }

    /// Register a handler with a cognitive profile.
    pub fn register(&mut self, name: &str, task_type: &str) {
        let vsa = Self::generate_profile_vsa(name, task_type, self.vsa_dim);
        self.profiles.insert(
            name.to_string(),
            HandlerProfile {
                name: name.to_string(),
                cognitive_vsa: vsa,
                task_type: task_type.to_string(),
            },
        );
    }

    /// Compute VSA cosine similarity between two vectors.
    fn vsa_cosine_similarity(a: &[u8], b: &[u8]) -> f64 {
        let dot: u64 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (*x as u64) * (*y as u64))
            .sum();
        let na: f64 = (a.iter().map(|x| (*x as u64) * (*x as u64)).sum::<u64>() as f64).sqrt();
        let nb: f64 = (b.iter().map(|x| (*x as u64) * (*x as u64)).sum::<u64>() as f64).sqrt();
        if na == 0.0 || nb == 0.0 {
            0.0
        } else {
            dot as f64 / (na * nb)
        }
    }

    /// Route to top-K handlers based on VSA similarity between cognitive state
    /// and handler profiles. Returns (handlers, similarities).
    pub fn route(&mut self, cognitive_state: &[u8]) -> Vec<(String, f64)> {
        self.total_routes += 1;
        if cognitive_state.is_empty() || self.profiles.is_empty() {
            self.last_routed_handlers = self.profiles.keys().take(self.top_k).cloned().collect();
            return self
                .last_routed_handlers
                .iter()
                .map(|n| (n.clone(), 0.5))
                .collect();
        }

        let mut scored: Vec<(String, f64)> = self
            .profiles
            .iter()
            .map(|(name, profile)| {
                let sim = Self::vsa_cosine_similarity(cognitive_state, &profile.cognitive_vsa);
                (name.clone(), sim)
            })
            .collect();

        // Sort by similarity descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_k_scored: Vec<(String, f64)> = scored.into_iter().take(self.top_k).collect();
        self.last_routed_handlers = top_k_scored.iter().map(|(n, _)| n.clone()).collect();
        top_k_scored
    }

    /// Register all known CI handlers with their task types.
    pub fn register_all_ci_handlers(&mut self) {
        let handler_types: &[(&str, &str)] = &[
            ("source_cognition", "input"),
            ("vsa_input", "input"),
            ("temporal_attention", "attention"),
            ("cross_modal", "alignment"),
            ("value_system", "reasoning"),
            ("volition", "decision"),
            ("inner_critic", "evaluation"),
            ("specious_present", "temporal"),
            ("narrative_self", "identity"),
            ("valence_axis", "affect"),
            ("drive_selector", "motivation"),
            ("memory_lattice", "memory"),
            ("memory_palace", "memory"),
            ("memory_sync", "memory"),
            ("memory_reflector", "reflection"),
            ("vsa_vocabulary", "knowledge"),
            ("cognitive_load", "regulation"),
            ("default_mode", "rest"),
            ("stream_buffer", "attention"),
            ("first_person", "identity"),
            ("awakening", "meta"),
            ("dream_consolidator", "consolidation"),
            ("meta_cognition", "meta"),
            ("calibration", "evaluation"),
            ("policy_repair", "regulation"),
            ("working_memory", "memory"),
            ("evosc", "evolution"),
            ("open_skill", "skill"),
            ("skill_dag", "skill"),
            ("skill_trend", "skill"),
            ("exploratory_gap", "curiosity"),
            ("signal_pattern", "pattern"),
            ("resonance", "pattern"),
            ("emergent_property", "reasoning"),
            ("concept_drift", "pattern"),
            ("reflexivity", "meta"),
            ("cognitive_diversity", "meta"),
            ("adaptive_rate", "regulation"),
            ("ne_evaluator", "language"),
            ("ne_loader", "language"),
            ("self_evolution", "evolution"),
            ("meta_agent", "meta"),
            ("news_radar", "external"),
            ("voice_synthesis", "external"),
            ("html_presentation", "external"),
            ("introspection", "meta"),
            ("adapt_orch", "meta"),
            ("sparse_vsa_attn", "attention"),
        ];
        for (name, task_type) in handler_types {
            self.register(name, task_type);
        }
    }

    pub fn stats(&self) -> String {
        format!(
            "vsa_moe:{}_profiles,{}_routes,last_k={}",
            self.profiles.len(),
            self.total_routes,
            self.last_routed_handlers.len(),
        )
    }
}

pub const CONTEXT_GATHER: &str = "CONTEXT_GATHER";
pub const DECISION_COMPRESS: &str = "DECISION_COMPRESS";
pub const EXPERIENCE_REFLECT: &str = "EXPERIENCE_REFLECT";
pub const SKILL_ACCUMULATE: &str = "SKILL_ACCUMULATE";
pub const CURRICULUM_GENERATE: &str = "CURRICULUM_GENERATE";
pub const DGMH_WRITEBACK: &str = "DGMH_WRITEBACK";
pub const SRCC_UPDATE: &str = "SRCC_UPDATE";
pub const SLEEP_CONSOLIDATE: &str = "SLEEP_CONSOLIDATE";
pub const GOAL_EXECUTE: &str = "GOAL_EXECUTE";
pub const NEUROMODULATE: &str = "NEUROMODULATE";
pub const ADVERSARIAL_ARENA: &str = "ADVERSARIAL_ARENA";
pub const META_REFLECT: &str = "META_REFLECT";
pub const E8_GEOMETRY: &str = "E8_GEOMETRY";
pub const VSA_CLEANUP: &str = "VSA_CLEANUP";
pub const NARRATIVE_TICK: &str = "NARRATIVE_TICK";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EdgeWeight {
    Required,
    Preferred,
    Optional,
}

#[derive(Debug, Clone)]
pub struct DagNode {
    pub id: HandlerId,
    pub avg_latency_ms: f64,
    pub call_count: u64,
    pub avg_contribution: f64,
}

#[derive(Debug, Clone)]
pub struct DagEdge {
    pub from: HandlerId,
    pub to: HandlerId,
    pub weight: EdgeWeight,
}

/// A topological layer of handlers that can run in parallel.
/// All handlers in a layer have no cross-dependencies.
pub type HandlerLayer = Vec<HandlerId>;

#[derive(Debug, Clone)]
pub struct AdaptOrch {
    pub nodes: HashMap<HandlerId, DagNode>,
    pub edges: Vec<DagEdge>,
    pub max_nodes: usize,
    pub energy_budget: f64,
    pub last_schedule: Vec<HandlerId>,
    /// Handler dependency graph: name -> list of dependencies (must run first)
    pub handler_graph: HashMap<String, Vec<String>>,
    /// Topological layers for parallel dispatch.
    /// Layer 0 runs first, then Layer 1, etc. All handlers in a layer are independent.
    pub execution_order: Vec<Vec<String>>,
    /// Cognitive load threshold for skipping cold layers (0.0 = never skip)
    pub cold_skip_threshold: f64,
    /// VSA-based MoE router (Kimi-VL inspired)
    pub vsa_router: VsaMoERouter,
    /// Whether VSA routing is active
    pub vsa_routing_enabled: bool,
    /// Topology router for DAG-based process type selection
    pub topology_router: Option<TopologyRouter>,
    /// Cached result of the last topology analysis
    pub selected_topology: Option<ProcessType>,
}

impl AdaptOrch {
    pub fn new() -> Self {
        let vsa_dim = 64;
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            max_nodes: 32,
            energy_budget: 1.0,
            last_schedule: Vec::new(),
            handler_graph: HashMap::new(),
            execution_order: Vec::new(),
            cold_skip_threshold: 0.8,
            vsa_router: VsaMoERouter::new(vsa_dim),
            vsa_routing_enabled: false,
            topology_router: None,
            selected_topology: None,
        }
    }

    pub fn with_max_nodes(max_nodes: usize) -> Self {
        let vsa_dim = 64;
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            max_nodes,
            energy_budget: 1.0,
            last_schedule: Vec::new(),
            handler_graph: HashMap::new(),
            execution_order: Vec::new(),
            cold_skip_threshold: 0.8,
            vsa_router: VsaMoERouter::new(vsa_dim),
            vsa_routing_enabled: false,
            topology_router: None,
            selected_topology: None,
        }
    }

    pub fn register_handler(&mut self, id: HandlerId) {
        if self.nodes.len() >= self.max_nodes {
            return;
        }
        self.nodes.entry(id.clone()).or_insert(DagNode {
            id,
            avg_latency_ms: 0.0,
            call_count: 0,
            avg_contribution: 0.0,
        });
    }

    pub fn add_edge(&mut self, from: HandlerId, to: HandlerId, weight: EdgeWeight) {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return;
        }
        if from == to {
            return;
        }
        if self.edges.iter().any(|e| e.from == from && e.to == to) {
            return;
        }
        if self.would_create_cycle(&from, &to) {
            return;
        }
        self.edges.push(DagEdge { from, to, weight });
    }

    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue: VecDeque<&str> = VecDeque::new();
        queue.push_back(to);
        while let Some(current) = queue.pop_front() {
            if current == from {
                return true;
            }
            if !visited.insert(current) {
                continue;
            }
            for edge in &self.edges {
                if edge.from == current {
                    queue.push_back(&edge.to);
                }
            }
        }
        false
    }

    pub fn record_latency(&mut self, id: &str, latency_ms: f64) {
        self.update_latency(&id.to_string(), latency_ms);
    }

    pub fn update_latency(&mut self, id: &HandlerId, latency_ms: f64) {
        if let Some(node) = self.nodes.get_mut(id) {
            if node.call_count == 0 {
                node.avg_latency_ms = latency_ms;
            } else {
                let alpha = 0.3;
                node.avg_latency_ms = alpha * latency_ms + (1.0 - alpha) * node.avg_latency_ms;
            }
        }
    }

    pub fn update_contribution(&mut self, id: &HandlerId, contribution: f64) {
        if let Some(node) = self.nodes.get_mut(id) {
            if node.call_count == 0 {
                node.avg_contribution = contribution;
            } else {
                let alpha = 0.3;
                node.avg_contribution =
                    alpha * contribution + (1.0 - alpha) * node.avg_contribution;
            }
            node.call_count += 1;
        }
    }

    fn active_edge_weight(&self, load: f64) -> EdgeWeight {
        if load > 0.8 {
            EdgeWeight::Required
        } else if load > 0.5 {
            EdgeWeight::Preferred
        } else {
            EdgeWeight::Optional
        }
    }

    pub fn schedule(&self, cognitive_load: f64) -> Vec<HandlerId> {
        let min_weight = self.active_edge_weight(cognitive_load);

        let reachable_from_roots = self.reachable_via(&min_weight);

        let mut candidates: Vec<&DagNode> = reachable_from_roots
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect();

        if cognitive_load > 0.7 {
            candidates = candidates
                .into_iter()
                .filter(|n| {
                    let cost_ratio = if n.avg_latency_ms <= 0.0 {
                        n.avg_contribution
                    } else {
                        n.avg_contribution / n.avg_latency_ms
                    };
                    cost_ratio > 0.0
                })
                .collect();
        }

        candidates.sort_by(|a, b| {
            let ratio_a = if a.avg_latency_ms <= 0.0 {
                a.avg_contribution
            } else {
                a.avg_contribution / a.avg_latency_ms
            };
            let ratio_b = if b.avg_latency_ms <= 0.0 {
                b.avg_contribution
            } else {
                b.avg_contribution / b.avg_latency_ms
            };
            ratio_b
                .partial_cmp(&ratio_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let sorted_ids: Vec<HandlerId> = candidates.into_iter().map(|n| n.id.clone()).collect();

        if self.energy_budget < 0.3 {
            sorted_ids
        } else if self.energy_budget < 0.7 {
            sorted_ids
        } else {
            sorted_ids
        }
    }

    fn reachable_via(&self, min_weight: &EdgeWeight) -> Vec<HandlerId> {
        let children: HashMap<&str, Vec<&str>> = self.build_child_map(min_weight);
        let parents: HashMap<&str, Vec<&str>> = self.build_parent_map(min_weight);

        let roots: Vec<&str> = self
            .nodes
            .keys()
            .filter(|id| !parents.contains_key(id.as_str()))
            .map(|s| s.as_str())
            .collect();

        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue: VecDeque<&str> = VecDeque::new();
        for root in &roots {
            queue.push_back(root);
        }
        while let Some(current) = queue.pop_front() {
            if !visited.insert(current) {
                continue;
            }
            if let Some(children) = children.get(current) {
                for child in children {
                    queue.push_back(child);
                }
            }
        }

        let mut result: Vec<HandlerId> = visited.into_iter().map(|s| s.to_string()).collect();
        result.sort();
        result
    }

    fn build_child_map(&self, min_weight: &EdgeWeight) -> HashMap<&str, Vec<&str>> {
        let mut map: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            if Self::weight_satisfies(&edge.weight, min_weight) {
                map.entry(edge.from.as_str())
                    .or_default()
                    .push(edge.to.as_str());
            }
        }
        map
    }

    fn build_parent_map(&self, min_weight: &EdgeWeight) -> HashMap<&str, Vec<&str>> {
        let mut map: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            if Self::weight_satisfies(&edge.weight, min_weight) {
                map.entry(edge.to.as_str())
                    .or_default()
                    .push(edge.from.as_str());
            }
        }
        map
    }

    fn weight_satisfies(edge_w: &EdgeWeight, min_w: &EdgeWeight) -> bool {
        match (edge_w, min_w) {
            (EdgeWeight::Required, _) => true,
            (EdgeWeight::Preferred, EdgeWeight::Required) => false,
            (EdgeWeight::Preferred, _) => true,
            (EdgeWeight::Optional, EdgeWeight::Optional) => true,
            (EdgeWeight::Optional, _) => false,
        }
    }

    pub fn cost_benefit_report(&self) -> Vec<(String, f64, f64, f64)> {
        let mut report: Vec<(String, f64, f64, f64)> = self
            .nodes
            .values()
            .map(|n| {
                let cpms = if n.avg_latency_ms <= 0.0 {
                    n.avg_contribution
                } else {
                    n.avg_contribution / n.avg_latency_ms
                };
                (n.id.clone(), n.avg_latency_ms, n.avg_contribution, cpms)
            })
            .collect();
        report.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        report
    }

    pub fn build_core_graph() -> Self {
        let mut orch = Self::new();

        let handlers = [
            CONTEXT_GATHER,
            DECISION_COMPRESS,
            EXPERIENCE_REFLECT,
            SKILL_ACCUMULATE,
            CURRICULUM_GENERATE,
            DGMH_WRITEBACK,
            SRCC_UPDATE,
            SLEEP_CONSOLIDATE,
            GOAL_EXECUTE,
            NEUROMODULATE,
            ADVERSARIAL_ARENA,
            META_REFLECT,
            E8_GEOMETRY,
            VSA_CLEANUP,
            NARRATIVE_TICK,
        ];

        for h in &handlers {
            orch.register_handler(h.to_string());
        }

        orch.add_edge(
            CONTEXT_GATHER.to_string(),
            DECISION_COMPRESS.to_string(),
            EdgeWeight::Required,
        );
        orch.add_edge(
            DECISION_COMPRESS.to_string(),
            EXPERIENCE_REFLECT.to_string(),
            EdgeWeight::Required,
        );
        orch.add_edge(
            EXPERIENCE_REFLECT.to_string(),
            SKILL_ACCUMULATE.to_string(),
            EdgeWeight::Required,
        );

        orch.add_edge(
            SKILL_ACCUMULATE.to_string(),
            CURRICULUM_GENERATE.to_string(),
            EdgeWeight::Preferred,
        );
        orch.add_edge(
            EXPERIENCE_REFLECT.to_string(),
            META_REFLECT.to_string(),
            EdgeWeight::Preferred,
        );

        orch.add_edge(
            DGMH_WRITEBACK.to_string(),
            VSA_CLEANUP.to_string(),
            EdgeWeight::Optional,
        );

        orch
    }

    pub fn set_energy_budget(&mut self, budget: f64) {
        self.energy_budget = budget.clamp(0.0, 1.0);
    }

    /// Build topological layers from Hot/Warm/Cold tiers.
    /// Layer 0: Hot handlers (no dependencies, always run)
    /// Layer 1: Warm handlers (depend on at least 1 hot handler)
    /// Layer 2: Cold handlers (depend on at least 1 warm handler)
    pub fn build_from_handler_graph(
        &mut self,
        hot_handlers: &[String],
        warm_handlers: &[String],
        cold_handlers: &[String],
    ) {
        self.execution_order.clear();
        self.handler_graph.clear();

        // Register all handlers as nodes
        for h in hot_handlers
            .iter()
            .chain(warm_handlers.iter())
            .chain(cold_handlers.iter())
        {
            self.register_handler(h.clone());
        }

        // Build dependency graph:
        // Hot → no dependencies
        // Warm → depends on all hot handlers
        // Cold → depends on all warm handlers
        for h in hot_handlers {
            self.handler_graph.entry(h.clone()).or_default();
        }
        for h in warm_handlers {
            self.handler_graph
                .entry(h.clone())
                .or_insert_with(|| hot_handlers.to_vec());
        }
        for h in cold_handlers {
            self.handler_graph
                .entry(h.clone())
                .or_insert_with(|| warm_handlers.to_vec());
        }

        // Build DAG edges from dependency graph (clone to avoid borrow conflict)
        let edge_plan: Vec<(String, String)> = self
            .handler_graph
            .iter()
            .flat_map(|(handler, deps)| deps.iter().map(move |dep| (dep.clone(), handler.clone())))
            .collect();
        for (from, to) in &edge_plan {
            self.add_edge(from.clone(), to.clone(), EdgeWeight::Required);
        }

        // Build topological layers
        if !hot_handlers.is_empty() {
            self.execution_order.push(hot_handlers.to_vec());
        }
        if !warm_handlers.is_empty() {
            self.execution_order.push(warm_handlers.to_vec());
        }
        if !cold_handlers.is_empty() {
            self.execution_order.push(cold_handlers.to_vec());
        }
    }

    /// Return topological layers for dispatch. Under high cognitive load,
    /// cold layers are skipped.
    pub fn topological_layers(&self, cognitive_load: f64) -> &[Vec<String>] {
        if cognitive_load > self.cold_skip_threshold && self.execution_order.len() > 2 {
            // Under high load, only execute hot + warm layers
            &self.execution_order[..self.execution_order.len().min(2)]
        } else if cognitive_load > 0.7 && self.execution_order.len() > 1 {
            // Under moderate load, only execute hot layer
            &self.execution_order[..1]
        } else {
            &self.execution_order
        }
    }

    /// Number of dependencies for a given handler.
    pub fn handler_dependency_count(&self, name: &str) -> usize {
        self.handler_graph
            .get(name)
            .map(|deps| deps.len())
            .unwrap_or(0)
    }

    /// Lazily compute and cache the topology from the current DAG.
    /// Returns the selected ProcessType.
    pub fn compute_topology(&mut self) -> ProcessType {
        let _ = self.topology_router.get_or_insert(TopologyRouter);
        let graph = self.build_state_graph();
        let pt = TopologyRouter::analyze_and_select(&graph, None);
        self.selected_topology = Some(pt);
        pt
    }

    /// Build a StateGraph snapshot from the internal DAG for topology analysis.
    fn build_state_graph(&self) -> StateGraph {
        let mut g = StateGraph::new();
        for (id, _) in &self.nodes {
            g.add_node(ArtifactNode::new(id, ArtifactType::Task, "handler"));
        }
        for edge in &self.edges {
            g.add_edge(&edge.from, &edge.to);
        }
        g
    }

    /// Report on current execution order structure.
    pub fn layer_report(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for (i, layer) in self.execution_order.iter().enumerate() {
            parts.push(format!("L{}:{}", i, layer.len()));
        }
        format!(
            "dag:{}_layers,{}_handlers,{}_edges|{}",
            self.execution_order.len(),
            self.nodes.len(),
            self.edges.len(),
            parts.join(","),
        )
    }
}

impl Default for AdaptOrch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_handler() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("test_handler".to_string());
        assert_eq!(orch.nodes.len(), 1);
        assert!(orch.nodes.contains_key("test_handler"));
    }

    #[test]
    fn test_register_duplicate_handler() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("dup".to_string());
        orch.register_handler("dup".to_string());
        assert_eq!(orch.nodes.len(), 1);
    }

    #[test]
    fn test_update_latency_first_call() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("h".to_string());
        orch.update_latency(&"h".to_string(), 15.0);
        assert!((orch.nodes["h"].avg_latency_ms - 15.0).abs() < 1e-9);
    }

    #[test]
    fn test_update_latency_running_avg() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("h".to_string());
        orch.update_latency(&"h".to_string(), 10.0);
        orch.update_latency(&"h".to_string(), 20.0);
        let expected = 0.3 * 20.0 + 0.7 * 10.0;
        assert!((orch.nodes["h"].avg_latency_ms - expected).abs() < 1e-9);
    }

    #[test]
    fn test_update_contribution_running_avg() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("h".to_string());
        orch.update_contribution(&"h".to_string(), 0.5);
        orch.update_contribution(&"h".to_string(), 1.0);
        let expected = 0.3 * 1.0 + 0.7 * 0.5;
        assert!((orch.nodes["h"].avg_contribution - expected).abs() < 1e-9);
        assert_eq!(orch.nodes["h"].call_count, 2);
    }

    #[test]
    fn test_add_edge_creates_dependency() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("a".to_string());
        orch.register_handler("b".to_string());
        orch.add_edge("a".to_string(), "b".to_string(), EdgeWeight::Required);
        assert_eq!(orch.edges.len(), 1);
        assert_eq!(orch.edges[0].from, "a");
        assert_eq!(orch.edges[0].to, "b");
        assert_eq!(orch.edges[0].weight, EdgeWeight::Required);
    }

    #[test]
    fn test_add_self_edge_ignored() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("a".to_string());
        orch.add_edge("a".to_string(), "a".to_string(), EdgeWeight::Required);
        assert_eq!(orch.edges.len(), 0);
    }

    #[test]
    fn test_add_cycle_edge_ignored() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("a".to_string());
        orch.register_handler("b".to_string());
        orch.add_edge("a".to_string(), "b".to_string(), EdgeWeight::Required);
        orch.add_edge("b".to_string(), "a".to_string(), EdgeWeight::Required);
        assert_eq!(orch.edges.len(), 1);
    }

    #[test]
    fn test_schedule_under_low_load_keeps_all() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("a".to_string());
        orch.register_handler("b".to_string());
        orch.register_handler("c".to_string());
        orch.add_edge("a".to_string(), "b".to_string(), EdgeWeight::Required);
        let sched = orch.schedule(0.3);
        assert!(sched.contains(&"a".to_string()));
        assert!(sched.contains(&"b".to_string()));
        assert!(sched.contains(&"c".to_string()));
    }

    #[test]
    fn test_schedule_under_high_load_filters_by_weight() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("root".to_string());
        orch.register_handler("required_child".to_string());
        orch.register_handler("optional_child".to_string());
        orch.add_edge(
            "root".to_string(),
            "required_child".to_string(),
            EdgeWeight::Required,
        );
        orch.add_edge(
            "root".to_string(),
            "optional_child".to_string(),
            EdgeWeight::Optional,
        );
        let sched = orch.schedule(0.9);
        assert!(sched.contains(&"root".to_string()));
        assert!(sched.contains(&"required_child".to_string()));
        assert!(!sched.contains(&"optional_child".to_string()));
    }

    #[test]
    fn test_schedule_respects_topological_order() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("a".to_string());
        orch.register_handler("b".to_string());
        orch.register_handler("c".to_string());
        orch.add_edge("a".to_string(), "b".to_string(), EdgeWeight::Required);
        orch.add_edge("b".to_string(), "c".to_string(), EdgeWeight::Required);
        let sched = orch.schedule(0.5);
        let pos_a = sched.iter().position(|x| x == "a").unwrap();
        let pos_b = sched.iter().position(|x| x == "b").unwrap();
        let pos_c = sched.iter().position(|x| x == "c").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_cost_benefit_report_format() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("a".to_string());
        orch.register_handler("b".to_string());
        orch.update_contribution(&"a".to_string(), 0.8);
        orch.update_latency(&"a".to_string(), 10.0);
        orch.update_contribution(&"b".to_string(), 0.2);
        orch.update_latency(&"b".to_string(), 5.0);
        let report = orch.cost_benefit_report();
        assert_eq!(report.len(), 2);
        for (name, latency, contrib, cpms) in &report {
            assert!(!name.is_empty());
            assert!(*latency >= 0.0);
            assert!(*contrib >= 0.0);
            assert!(cpms >= &0.0);
        }
        assert!(report[0].3 >= report[1].3);
    }

    #[test]
    fn test_build_core_graph_has_all_handlers() {
        let orch = AdaptOrch::build_core_graph();
        assert_eq!(orch.nodes.len(), 15);
        assert!(orch.nodes.contains_key(CONTEXT_GATHER));
        assert!(orch.nodes.contains_key(DECISION_COMPRESS));
        assert!(orch.nodes.contains_key(EXPERIENCE_REFLECT));
        assert!(orch.nodes.contains_key(SKILL_ACCUMULATE));
        assert!(orch.nodes.contains_key(CURRICULUM_GENERATE));
        assert!(orch.nodes.contains_key(DGMH_WRITEBACK));
        assert!(orch.nodes.contains_key(SRCC_UPDATE));
        assert!(orch.nodes.contains_key(SLEEP_CONSOLIDATE));
        assert!(orch.nodes.contains_key(GOAL_EXECUTE));
        assert!(orch.nodes.contains_key(NEUROMODULATE));
        assert!(orch.nodes.contains_key(ADVERSARIAL_ARENA));
        assert!(orch.nodes.contains_key(META_REFLECT));
        assert!(orch.nodes.contains_key(E8_GEOMETRY));
        assert!(orch.nodes.contains_key(VSA_CLEANUP));
        assert!(orch.nodes.contains_key(NARRATIVE_TICK));
    }

    #[test]
    fn test_build_core_graph_edges() {
        let orch = AdaptOrch::build_core_graph();
        assert!(!orch.edges.is_empty());
        let required_count = orch
            .edges
            .iter()
            .filter(|e| matches!(e.weight, EdgeWeight::Required))
            .count();
        let preferred_count = orch
            .edges
            .iter()
            .filter(|e| matches!(e.weight, EdgeWeight::Preferred))
            .count();
        let optional_count = orch
            .edges
            .iter()
            .filter(|e| matches!(e.weight, EdgeWeight::Optional))
            .count();
        assert_eq!(required_count, 3);
        assert_eq!(preferred_count, 2);
        assert_eq!(optional_count, 1);
    }

    #[test]
    fn test_empty_graph_scheduling() {
        let orch = AdaptOrch::new();
        let sched = orch.schedule(0.5);
        assert!(sched.is_empty());
    }

    #[test]
    fn test_single_node_graph() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("solo".to_string());
        let sched = orch.schedule(0.5);
        assert_eq!(sched.len(), 1);
        assert_eq!(sched[0], "solo");
    }

    #[test]
    fn test_energy_budget_below_03() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("a".to_string());
        orch.register_handler("b".to_string());
        orch.add_edge("a".to_string(), "b".to_string(), EdgeWeight::Required);
        orch.set_energy_budget(0.2);
        let sched = orch.schedule(0.5);
        assert_eq!(sched.len(), 2);
    }

    #[test]
    fn test_set_energy_budget_clamps() {
        let mut orch = AdaptOrch::new();
        orch.set_energy_budget(-0.5);
        assert!((orch.energy_budget - 0.0).abs() < 1e-9);
        orch.set_energy_budget(1.5);
        assert!((orch.energy_budget - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_schedule_sorts_by_contribution_per_latency() {
        let mut orch = AdaptOrch::new();
        orch.register_handler("high_roi".to_string());
        orch.register_handler("low_roi".to_string());
        orch.update_contribution(&"high_roi".to_string(), 1.0);
        orch.update_latency(&"high_roi".to_string(), 1.0);
        orch.update_contribution(&"low_roi".to_string(), 0.1);
        orch.update_latency(&"low_roi".to_string(), 10.0);
        let sched = orch.schedule(0.5);
        let pos_high = sched.iter().position(|x| x == "high_roi").unwrap();
        let pos_low = sched.iter().position(|x| x == "low_roi").unwrap();
        assert!(pos_high < pos_low);
    }

    #[test]
    fn test_max_nodes_cap() {
        let mut orch = AdaptOrch::with_max_nodes(3);
        for i in 0..5 {
            orch.register_handler(format!("h{}", i));
        }
        assert_eq!(orch.nodes.len(), 3);
    }
}
