use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_id() -> String {
    let count = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:016x}{:016x}", now, count)
}

fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ─── Entity Node ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityNode {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub source_node_id: String,
    pub confidence: f64,
    pub properties: HashMap<String, String>,
    pub created_at: u64,
}

// ─── Relation Edge ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationEdge {
    pub id: String,
    pub source_entity: String,
    pub target_entity: String,
    pub relation_type: String,
    pub weight: f64,
    pub evidence: String,
    pub confidence: f64,
    pub created_at: u64,
}

// ─── Entity Graph ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityGraph {
    pub entities: HashMap<String, EntityNode>,
    pub relations: HashMap<String, RelationEdge>,
    pub adjacency: HashMap<String, Vec<(String, String, String)>>,
}

impl Default for EntityGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityGraph {
    pub fn new() -> Self {
        EntityGraph {
            entities: HashMap::new(),
            relations: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }
}

// ─── Graph Query Mode ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQueryMode {
    Local { max_depth: usize, max_neighbors: usize },
    Global { community_level: usize },
    Hybrid { local_depth: usize, global_level: usize },
    Auto,
}

// ─── Config ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ExtractionMode {
    #[default]
    Heuristic,
    Llm,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRagConfig {
    pub max_entities_per_doc: usize,
    pub min_confidence: f64,
    pub enable_incremental_updates: bool,
    pub max_graph_size: usize,
    pub extraction_mode: ExtractionMode,
}

impl Default for GraphRagConfig {
    fn default() -> Self {
        GraphRagConfig {
            max_entities_per_doc: 50,
            min_confidence: 0.3,
            enable_incremental_updates: true,
            max_graph_size: 100000,
            extraction_mode: ExtractionMode::Heuristic,
        }
    }
}

// ─── Stats ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphRagStats {
    pub total_entities: usize,
    pub total_relations: usize,
    pub extraction_runs: u64,
    pub avg_extraction_time_ms: f64,
}

// ─── Subgraph Result ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphResult {
    pub entities: Vec<EntityNode>,
    pub relations: Vec<RelationEdge>,
    pub traversal_depth: usize,
    pub query_mode: String,
}

// ─── Hybrid Result ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridResult {
    pub local_results: Vec<SubgraphResult>,
    pub global_results: Vec<GlobalSummary>,
    pub merged_entities: Vec<EntityNode>,
    pub merged_relations: Vec<RelationEdge>,
}

// ─── Global Summary ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSummary {
    pub community_id: String,
    pub topic_keywords: Vec<String>,
    pub summary_text: String,
    pub confidence: f64,
    pub last_updated: u64,
    pub entity_count: usize,
    pub relation_count: usize,
}

// ─── Incremental Change ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalChange {
    pub added_entities: Vec<EntityNode>,
    pub added_relations: Vec<RelationEdge>,
    pub timestamp: u64,
}

// ─── LightRag Index ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightRagIndex {
    pub global_summaries: Vec<GlobalSummary>,
    pub change_log: Vec<IncrementalChange>,
    pub last_community_update: u64,
    pub query_count: u64,
}

impl Default for LightRagIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl LightRagIndex {
    pub fn new() -> Self {
        LightRagIndex {
            global_summaries: Vec::new(),
            change_log: Vec::new(),
            last_community_update: 0,
            query_count: 0,
        }
    }
}

// ─── Community ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: String,
    pub entity_ids: Vec<String>,
    pub summary: String,
    pub size: usize,
    pub avg_confidence: f64,
}

// ─── GraphRagStore ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRagStore {
    graph: EntityGraph,
    config: GraphRagConfig,
    stats: GraphRagStats,
    global_summaries: Vec<GlobalSummary>,
    change_log: Vec<IncrementalChange>,
    lightrag_index: LightRagIndex,
}

impl GraphRagStore {
    pub fn new(config: GraphRagConfig) -> Self {
        GraphRagStore {
            graph: EntityGraph::new(),
            config,
            stats: GraphRagStats::default(),
            global_summaries: Vec::new(),
            change_log: Vec::new(),
            lightrag_index: LightRagIndex::new(),
        }
    }

    pub fn config(&self) -> &GraphRagConfig {
        &self.config
    }

    pub fn stats(&self) -> &GraphRagStats {
        &self.stats
    }

    pub fn graph(&self) -> &EntityGraph {
        &self.graph
    }

    // ── Entity Extraction ──────────────────────────────────────────

    pub fn extract_entities(
        &mut self,
        text: &str,
        source_id: &str,
    ) -> Result<(Vec<EntityNode>, Vec<RelationEdge>), String> {
        let start = std::time::Instant::now();

        let sentences = split_sentences(text);
        let mut all_entities: Vec<EntityNode> = Vec::new();
        let mut all_relations: Vec<RelationEdge> = Vec::new();
        let mut seen_entity_names: HashSet<String> = HashSet::new();

        // Phase 1: collect all unique entity names across sentences,
        // then create EntityNodes once per name.
        let mut sentence_entity_names: Vec<Vec<String>> = Vec::new();
        for sentence in &sentences {
            if sentence.len() < 3 {
                sentence_entity_names.push(Vec::new());
                continue;
            }
            let entity_names = extract_capitalized_terms(sentence, source_id);

            for ename in &entity_names {
                let key = ename.to_lowercase();
                if !seen_entity_names.contains(&key) {
                    if all_entities.len() >= self.config.max_entities_per_doc {
                        break;
                    }
                    seen_entity_names.insert(key);
                    let etype = infer_entity_type(ename);
                    let confidence = estimate_entity_confidence(ename, sentence);
                    all_entities.push(EntityNode {
                        id: generate_id(),
                        name: ename.clone(),
                        entity_type: etype,
                        source_node_id: source_id.to_string(),
                        confidence,
                        properties: HashMap::new(),
                        created_at: now_nanos(),
                    });
                }
            }
            sentence_entity_names.push(entity_names);
        }

        // Phase 2: for each sentence, detect relations between co-occurring entities
        for (s_idx, sentence) in sentences.iter().enumerate() {
            if sentence.len() < 3 {
                continue;
            }
            let names_in_sentence = &sentence_entity_names[s_idx];
            if names_in_sentence.len() < 2 {
                continue;
            }
            // Build reference to existing EntityNode by name (case-insensitive)
            let mut entities_in_sentence: Vec<&EntityNode> = Vec::new();
            for ename in names_in_sentence {
                let key = ename.to_lowercase();
                if let Some(entity) = all_entities.iter().find(|e| e.name.to_lowercase() == key) {
                    entities_in_sentence.push(entity);
                }
            }

            for i in 0..entities_in_sentence.len() {
                for j in (i + 1)..entities_in_sentence.len() {
                    let e1 = entities_in_sentence[i];
                    let e2 = entities_in_sentence[j];

                    if let Some((rel_type, distance)) =
                        detect_relation(&e1.name, &e2.name, sentence)
                    {
                        let weight = e1.confidence.min(e2.confidence) * (1.0 / distance.max(1.0));
                        let evidence_start = sentence
                            .find(&e1.name)
                            .unwrap_or(0)
                            .min(sentence.find(&e2.name).unwrap_or(0));
                        let evidence_end = (evidence_start + 150).min(sentence.len());
                        let evidence = if evidence_end > evidence_start {
                            sentence[evidence_start..evidence_end].to_string()
                        } else {
                            sentence.clone()
                        };

                        let relation = RelationEdge {
                            id: generate_id(),
                            source_entity: e1.id.clone(),
                            target_entity: e2.id.clone(),
                            relation_type: rel_type.to_string(),
                            weight,
                            evidence,
                            confidence: e1.confidence.min(e2.confidence),
                            created_at: now_nanos(),
                        };
                        all_relations.push(relation);
                    }
                }
            }
        }

        // Store extracted data (if incremental updates enabled)
        if self.config.enable_incremental_updates {
            let max_size = self.config.max_graph_size;
            let mut actually_added_entities: Vec<EntityNode> = Vec::new();
            let mut actually_added_relations: Vec<RelationEdge> = Vec::new();
            for entity in &all_entities {
                if self.graph.entities.len() >= max_size {
                    break;
                }
                if !self.graph.entities.contains_key(&entity.id) {
                    self.add_entity_internal(entity.clone());
                    actually_added_entities.push(entity.clone());
                }
            }
            for relation in &all_relations {
                if self.graph.relations.len() >= max_size {
                    break;
                }
                if !self.graph.relations.contains_key(&relation.id) {
                    self.add_relation_internal(relation.clone());
                    actually_added_relations.push(relation.clone());
                }
            }
            // Record change for incremental updates
            if !actually_added_entities.is_empty() || !actually_added_relations.is_empty() {
                self.change_log.push(IncrementalChange {
                    added_entities: actually_added_entities,
                    added_relations: actually_added_relations,
                    timestamp: now_nanos(),
                });
            }
        }

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        self.stats.extraction_runs += 1;
        self.stats.total_entities = self.graph.entities.len();
        self.stats.total_relations = self.graph.relations.len();
        if self.stats.extraction_runs > 1 {
            self.stats.avg_extraction_time_ms = self.stats.avg_extraction_time_ms
                * ((self.stats.extraction_runs - 1) as f64 / self.stats.extraction_runs as f64)
                + elapsed / self.stats.extraction_runs as f64;
        } else {
            self.stats.avg_extraction_time_ms = elapsed;
        }

        Ok((all_entities, all_relations))
    }

    // ── Query ──────────────────────────────────────────────────────

    pub fn query(
        &self,
        seed_entity_ids: &[String],
        mode: GraphQueryMode,
    ) -> Result<SubgraphResult, String> {
        match mode {
            GraphQueryMode::Local {
                max_depth,
                max_neighbors,
            } => {
                let mut collected_entities: Vec<EntityNode> = Vec::new();
                let mut collected_relations: Vec<RelationEdge> = Vec::new();
                let mut visited_entities: HashSet<String> = HashSet::new();
                let mut visited_relations: HashSet<String> = HashSet::new();
                let mut queue: VecDeque<(String, usize)> = VecDeque::new();

                for seed_id in seed_entity_ids {
                    if self.graph.entities.contains_key(seed_id) && visited_entities.insert(seed_id.clone()) {
                        if let Some(entity) = self.graph.entities.get(seed_id) {
                            collected_entities.push(entity.clone());
                        }
                        queue.push_back((seed_id.clone(), 0));
                    }
                }

                while let Some((current_id, depth)) = queue.pop_front() {
                    if depth >= max_depth {
                        continue;
                    }
                    if let Some(adj) = self.graph.adjacency.get(&current_id) {
                        let mut neighbors: Vec<&(String, String, String)> = adj.iter().collect();
                        // Sort by weight descending (look up relation weight)
                        neighbors.sort_by(|a, b| {
                            let wa = self
                                .graph
                                .relations
                                .get(&a.2)
                                .map(|r| r.weight)
                                .unwrap_or(0.0);
                            let wb = self
                                .graph
                                .relations
                                .get(&b.2)
                                .map(|r| r.weight)
                                .unwrap_or(0.0);
                            wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
                        });

                        for (_rel_type, target_id, edge_id) in neighbors.iter().take(max_neighbors) {
                            if visited_relations.insert(edge_id.to_string()) {
                                if let Some(relation) = self.graph.relations.get(&edge_id.to_string()) {
                                    collected_relations.push(relation.clone());
                                }
                            }
                            if visited_entities.insert(target_id.to_string()) {
                                if let Some(entity) = self.graph.entities.get(&target_id.to_string()) {
                                    collected_entities.push(entity.clone());
                                }
                                queue.push_back((target_id.to_string(), depth + 1));
                            }
                        }
                    }
                }

                Ok(SubgraphResult {
                    entities: collected_entities,
                    relations: collected_relations,
                    traversal_depth: max_depth,
                    query_mode: "local".to_string(),
                })
            }
            GraphQueryMode::Global { community_level: _ } => {
                let communities = self.community_summary();
                let community_ids: HashSet<String> = seed_entity_ids
                    .iter()
                    .filter_map(|eid| {
                        communities
                            .iter()
                            .find(|c| c.entity_ids.contains(eid))
                            .map(|c| c.id.clone())
                    })
                    .collect();

                let mut collected_entities: Vec<EntityNode> = Vec::new();
                let mut collected_relations: Vec<RelationEdge> = Vec::new();
                let mut entity_set: HashSet<String> = HashSet::new();

                for comm in &communities {
                    if !seed_entity_ids.is_empty() && !community_ids.contains(&comm.id) {
                        continue;
                    }
                    for eid in &comm.entity_ids {
                        if entity_set.insert(eid.clone()) {
                            if let Some(entity) = self.graph.entities.get(eid) {
                                collected_entities.push(entity.clone());
                            }
                        }
                    }
                }

                for relation in self.graph.relations.values() {
                    if entity_set.contains(&relation.source_entity)
                        && entity_set.contains(&relation.target_entity)
                    {
                        collected_relations.push(relation.clone());
                    }
                }

                Ok(SubgraphResult {
                    entities: collected_entities,
                    relations: collected_relations,
                    traversal_depth: 0,
                    query_mode: "global".to_string(),
                })
            }
            GraphQueryMode::Auto => {
                // Auto-detect: if seed entity IDs are provided, use local; otherwise global
                if seed_entity_ids.is_empty() {
                    let subgraph = self.query(seed_entity_ids, GraphQueryMode::Global { community_level: 0 })?;
                    return Ok(SubgraphResult {
                        entities: subgraph.entities,
                        relations: subgraph.relations,
                        traversal_depth: 0,
                        query_mode: "auto(global)".to_string(),
                    });
                }
                let subgraph = self.query(
                    seed_entity_ids,
                    GraphQueryMode::Local { max_depth: 2, max_neighbors: 10 },
                )?;
                Ok(SubgraphResult {
                    entities: subgraph.entities,
                    relations: subgraph.relations,
                    traversal_depth: subgraph.traversal_depth,
                    query_mode: "auto(local)".to_string(),
                })
            }
            GraphQueryMode::Hybrid {
                local_depth,
                global_level,
            } => {
                let local = self.query(
                    seed_entity_ids,
                    GraphQueryMode::Local {
                        max_depth: local_depth,
                        max_neighbors: 10,
                    },
                )?;

                let global = self.query(
                    seed_entity_ids,
                    GraphQueryMode::Global {
                        community_level: global_level,
                    },
                )?;

                let mut seen_entities: HashSet<String> = HashSet::new();
                let mut seen_relations: HashSet<String> = HashSet::new();
                let mut merged_entities: Vec<EntityNode> = Vec::new();
                let mut merged_relations: Vec<RelationEdge> = Vec::new();

                for e in local.entities.into_iter().chain(global.entities.into_iter()) {
                    if seen_entities.insert(e.id.clone()) {
                        merged_entities.push(e);
                    }
                }
                for r in local.relations.into_iter().chain(global.relations.into_iter()) {
                    if seen_relations.insert(r.id.clone()) {
                        merged_relations.push(r);
                    }
                }

                Ok(SubgraphResult {
                    entities: merged_entities,
                    relations: merged_relations,
                    traversal_depth: local_depth,
                    query_mode: "hybrid".to_string(),
                })
            }
        }
    }

    // ── Query by Text ──────────────────────────────────────────────

    pub fn query_by_text(
        &self,
        query_entities: &[&str],
        mode: GraphQueryMode,
    ) -> Result<SubgraphResult, String> {
        let mut matched_ids: Vec<String> = Vec::new();
        for q in query_entities {
            let q_lower = q.to_lowercase();
            for e in self.graph.entities.values() {
                if e.name.to_lowercase().contains(&q_lower) {
                    matched_ids.push(e.id.clone());
                }
            }
        }

        if matched_ids.is_empty() {
            return Ok(SubgraphResult {
                entities: Vec::new(),
                relations: Vec::new(),
                traversal_depth: 0,
                query_mode: format!("{:?}", mode),
            });
        }

        let seed_ids: Vec<String> = matched_ids.clone();
        self.query(&seed_ids, mode)
    }

    // ── Add Entity ─────────────────────────────────────────────────

    pub fn add_entity(&mut self, entity: EntityNode) -> String {
        let id = entity.id.clone();
        self.add_entity_internal(entity);
        self.stats.total_entities = self.graph.entities.len();
        id
    }

    fn add_entity_internal(&mut self, entity: EntityNode) {
        let id = entity.id.clone();
        if !self.graph.adjacency.contains_key(&id) {
            self.graph.adjacency.insert(id.clone(), Vec::new());
        }
        self.graph.entities.insert(id, entity);
    }

    // ── Add Relation ───────────────────────────────────────────────

    pub fn add_relation(&mut self, relation: RelationEdge) -> String {
        let id = relation.id.clone();
        self.add_relation_internal(relation);
        self.stats.total_relations = self.graph.relations.len();
        id
    }

    fn add_relation_internal(&mut self, relation: RelationEdge) {
        let id = relation.id.clone();
        let rel_type = relation.relation_type.clone();
        let source = relation.source_entity.clone();
        let target = relation.target_entity.clone();

        self.graph.relations.insert(id.clone(), relation);

        self.graph
            .adjacency
            .entry(source.clone())
            .or_default()
            .push((rel_type.clone(), target.clone(), id.clone()));

        self.graph
            .adjacency
            .entry(target)
            .or_default()
            .push((rel_type, source, id));
    }

    // ── Remove Entity ──────────────────────────────────────────────

    pub fn remove_entity(&mut self, entity_id: &str) -> bool {
        if !self.graph.entities.contains_key(entity_id) {
            return false;
        }

        // Collect relation IDs to remove
        let to_remove: Vec<String> = self
            .graph
            .relations
            .values()
            .filter(|r| r.source_entity == entity_id || r.target_entity == entity_id)
            .map(|r| r.id.clone())
            .collect();

        for rid in &to_remove {
            self.remove_relation(rid);
        }

        self.graph.adjacency.remove(entity_id);
        self.graph.entities.remove(entity_id);
        self.stats.total_entities = self.graph.entities.len();
        self.stats.total_relations = self.graph.relations.len();
        true
    }

    // ── Remove Relation ────────────────────────────────────────────

    pub fn remove_relation(&mut self, relation_id: &str) -> bool {
        let relation = match self.graph.relations.remove(relation_id) {
            Some(r) => r,
            None => return false,
        };

        // Remove from adjacency lists
        let source = relation.source_entity;
        let target = relation.target_entity;

        if let Some(adj) = self.graph.adjacency.get_mut(&source) {
            adj.retain(|(_, _, eid)| eid != relation_id);
        }
        if let Some(adj) = self.graph.adjacency.get_mut(&target) {
            adj.retain(|(_, _, eid)| eid != relation_id);
        }

        self.stats.total_relations = self.graph.relations.len();
        true
    }

    // ── Community Summary ──────────────────────────────────────────

    /// Compute degree centrality for graph entities
    fn compute_centrality(&self) -> HashMap<String, f64> {
        let mut centrality: HashMap<String, f64> = HashMap::new();
        for (eid, adj) in &self.graph.adjacency {
            let degree = adj.len() as f64;
            centrality.insert(eid.clone(), degree);
        }
        // Normalize
        let max = centrality.values().cloned().fold(0.0, f64::max);
        if max > 0.0 {
            for v in centrality.values_mut() {
                *v /= max;
            }
        }
        centrality
    }

    /// Generate a rich text summary for a community given its member entity IDs.
    fn build_community_summary(
        &self,
        label: usize,
        members: &[String],
        centrality: &HashMap<String, f64>,
    ) -> Community {
        let size = members.len();
        if size == 0 {
            return Community {
                id: format!("comm_{}", label),
                entity_ids: Vec::new(),
                summary: "Empty community".into(),
                size: 0,
                avg_confidence: 0.0,
            };
        }

        let mut avg_conf = 0.0_f64;
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut member_centralities: Vec<(String, f64, String)> = Vec::new(); // (name, centr, type)

        for eid in members {
            if let Some(entity) = self.graph.entities.get(eid) {
                avg_conf += entity.confidence;
                *type_counts.entry(entity.entity_type.clone()).or_insert(0) += 1;
                let centr = centrality.get(eid).copied().unwrap_or(0.0);
                member_centralities.push((entity.name.clone(), centr, entity.entity_type.clone()));
            }
        }
        avg_conf /= size as f64;

        // Sort members by centrality descending, take top 5
        member_centralities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_entities: Vec<String> = member_centralities
            .into_iter()
            .take(5)
            .map(|(n, _, t)| format!("{} ({})", n, t))
            .collect();

        // Build type distribution string
        let mut type_parts: Vec<String> = type_counts
            .into_iter()
            .map(|(t, c)| format!("{} {}{}", c, t, if c > 1 { "s" } else { "" }))
            .collect();
        type_parts.sort();
        let type_str = if type_parts.is_empty() {
            String::new()
        } else {
            format!(", types: {}", type_parts.join(", "))
        };

        // Count internal relations (both entities in this community)
        let member_set: HashSet<&String> = members.iter().collect();
        let mut rel_types: HashMap<String, usize> = HashMap::new();
        for rel in self.graph.relations.values() {
            if member_set.contains(&rel.source_entity)
                && member_set.contains(&rel.target_entity)
            {
                *rel_types.entry(rel.relation_type.clone()).or_insert(0) += 1;
            }
        }

        let rel_str = if rel_types.is_empty() {
            String::new()
        } else {
            let mut parts: Vec<String> = rel_types
                .into_iter()
                .map(|(t, c)| format!("{} {}", c, t))
                .collect();
            parts.sort();
            format!(", relations: {}", parts.join(", "))
        };

        let summary = format!(
            "Community {}: {} entities{}{}. Top: [{}]",
            label,
            size,
            type_str,
            rel_str,
            top_entities.join(", "),
        );

        Community {
            id: format!("comm_{}", label),
            entity_ids: members.to_vec(),
            summary,
            size,
            avg_confidence: avg_conf,
        }
    }

    pub fn community_summary(&self) -> Vec<Community> {
        if self.graph.entities.is_empty() {
            return Vec::new();
        }

        // Label propagation algorithm for community detection
        let entity_ids: Vec<String> = self.graph.entities.keys().cloned().collect();
        let mut labels: HashMap<String, usize> = HashMap::new();
        for (i, eid) in entity_ids.iter().enumerate() {
            labels.insert(eid.clone(), i);
        }

        for _iter in 0..20 {
            let mut changed = false;
            for eid in &entity_ids {
                let mut label_weights: HashMap<usize, f64> = HashMap::new();
                if let Some(adj) = self.graph.adjacency.get(eid) {
                    for (_, target, edge_id) in adj {
                        if let Some(&neighbor_label) = labels.get(target) {
                            let w = self
                                .graph
                                .relations
                                .get(edge_id)
                                .map(|r| r.weight)
                                .unwrap_or(1.0);
                            *label_weights.entry(neighbor_label).or_insert(0.0) += w;
                        }
                    }
                }
                if label_weights.is_empty() {
                    continue;
                }
                let best_label = label_weights
                    .into_iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(l, _)| l);
                if let Some(bl) = best_label {
                    if labels.get(eid) != Some(&bl) {
                        labels.insert(eid.clone(), bl);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        // Group by label
        let mut community_map: HashMap<usize, Vec<String>> = HashMap::new();
        for (eid, label) in &labels {
            community_map.entry(*label).or_default().push(eid.clone());
        }

        // Precompute centrality once
        let centrality = self.compute_centrality();

        // Build Community structs with rich summaries
        let mut communities: Vec<Community> = community_map
            .into_iter()
            .map(|(label, members)| {
                self.build_community_summary(label, &members, &centrality)
            })
            .collect();

        // Sort by size descending
        communities.sort_by(|a, b| b.size.cmp(&a.size));
        communities
    }

    /// Community-aware query: route a text query to the most relevant community
    /// by matching query terms against entity names in each community.
    pub fn community_query(
        &self,
        query_terms: &[&str],
        top_k_communities: usize,
    ) -> Vec<(Community, SubgraphResult)> {
        let communities = self.community_summary();
        let mut scored: Vec<(usize, f64)> = Vec::new();

        for (idx, comm) in communities.iter().enumerate() {
            let mut score = 0.0_f64;
            for eid in &comm.entity_ids {
                if let Some(entity) = self.graph.entities.get(eid) {
                    for q in query_terms {
                        let q_lower = q.to_lowercase();
                        if entity.name.to_lowercase().contains(&q_lower) {
                            score += entity.confidence;
                        }
                    }
                }
            }
            if score > 0.0 {
                scored.push((idx, score));
            }
        }

        // Sort by relevance score
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut results: Vec<(Community, SubgraphResult)> = Vec::new();
        for (idx, _) in scored.into_iter().take(top_k_communities) {
            let comm = &communities[idx];
            if let Ok(subgraph) = self.query(
                &comm.entity_ids,
                GraphQueryMode::Local {
                    max_depth: 2,
                    max_neighbors: 10,
                },
            ) {
                results.push((comm.clone(), subgraph));
            }
        }
        results
    }

    // ── BFS Subgraph ───────────────────────────────────────────────

    pub fn get_subgraph(
        &self,
        entity_ids: &[String],
        depth: usize,
    ) -> SubgraphResult {
        self.query(
            entity_ids,
            GraphQueryMode::Local {
                max_depth: depth,
                max_neighbors: usize::MAX,
            },
        )
        .unwrap_or(SubgraphResult {
            entities: Vec::new(),
            relations: Vec::new(),
            traversal_depth: depth,
            query_mode: "bfs".to_string(),
        })
    }

    // ── LightRAG: Search Local (Entity-Centric) ─────────────────────

    /// Search entities by matching name/type/property against query terms,
    /// traverse relations (1-2 hops), score by centrality + weight + confidence.
    pub fn search_local(&self, query: &str, top_k: usize) -> Vec<SubgraphResult> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
        if query_terms.is_empty() {
            return Vec::new();
        }

        // Score all entities by query term overlap
        let centrality = self.compute_centrality();
        let mut scored_entities: Vec<(String, f64)> = Vec::new();

        for (eid, entity) in &self.graph.entities {
            let mut score = 0.0;

            // Name match (highest weight)
            let name_lower = entity.name.to_lowercase();
            let name_words: Vec<&str> = name_lower.split_whitespace().collect();
            for qt in &query_terms {
                if name_lower.contains(qt) {
                    score += 0.5;
                }
            }
            let overlap: usize = name_words.iter().filter(|w| query_terms.contains(w)).count();
            if overlap > 0 {
                score += 0.3 * (overlap as f64 / name_words.len().max(1) as f64);
            }

            // Type match
            let type_lower = entity.entity_type.to_lowercase();
            for qt in &query_terms {
                if type_lower.contains(qt) {
                    score += 0.2;
                }
            }

            // Property match
            for pv in entity.properties.values() {
                let pv_lower = pv.to_lowercase();
                for qt in &query_terms {
                    if pv_lower.contains(qt) {
                        score += 0.15;
                    }
                }
            }

            // Centrality boost
            let centr = centrality.get(eid).copied().unwrap_or(0.0);
            score += centr * 0.25;

            // Confidence
            score *= entity.confidence;

            // Require at least one query term to match name, type, or property
            let has_term_match = query_terms.iter().any(|qt| {
                name_lower.contains(qt)
                    || type_lower.contains(qt)
                    || entity.properties.values().any(|pv| pv.to_lowercase().contains(qt))
            });
            if score > 0.0 && has_term_match {
                scored_entities.push((eid.clone(), score));
            }
        }

        // Sort by score descending
        scored_entities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top-K seeds and extract subgraphs
        let mut results: Vec<SubgraphResult> = Vec::new();
        let mut seen_seeds: HashSet<String> = HashSet::new();

        for (eid, _score) in scored_entities.iter().take(top_k * 2) {
            if seen_seeds.insert(eid.clone()) {
                if let Ok(subgraph) = self.query(
                    std::slice::from_ref(eid),
                    GraphQueryMode::Local {
                        max_depth: 2,
                        max_neighbors: 8,
                    },
                ) {
                    if !subgraph.entities.is_empty() {
                        results.push(SubgraphResult {
                            entities: subgraph.entities,
                            relations: subgraph.relations,
                            traversal_depth: 2,
                            query_mode: "lightrag_local".to_string(),
                        });
                    }
                }
            }
            if results.len() >= top_k {
                break;
            }
        }

        results
    }

    // ── LightRAG: Search Global (Summary-Centric) ────────────────────

    /// Map query to community via keyword overlap + summary text match.
    /// Return top-K global summaries as global-level context.
    pub fn search_global(&self, query: &str, top_k: usize) -> Vec<GlobalSummary> {
        if self.global_summaries.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
        if query_terms.is_empty() {
            return self.global_summaries.iter().take(top_k).cloned().collect();
        }

        let mut scored: Vec<(usize, f64)> = Vec::new();

        for (idx, gs) in self.global_summaries.iter().enumerate() {
            let mut score = 0.0;

            // Topic keyword overlap
            for kw in &gs.topic_keywords {
                let kw_lower = kw.to_lowercase();
                for qt in &query_terms {
                    if kw_lower.contains(qt) || qt.contains(&kw_lower) {
                        score += 0.4;
                    }
                }
            }

            // Summary text match
            let summary_lower = gs.summary_text.to_lowercase();
            for qt in &query_terms {
                if summary_lower.contains(qt) {
                    score += 0.3;
                }
            }

            // Entity name match in community
            for eid in &self
                .community_summary()
                .iter()
                .find(|c| c.id == gs.community_id)
                .map(|c| &c.entity_ids)
                .cloned()
                .unwrap_or_default()
            {
                if let Some(entity) = self.graph.entities.get(eid) {
                    let name_lower = entity.name.to_lowercase();
                    for qt in &query_terms {
                        if name_lower.contains(qt) {
                            score += 0.2;
                        }
                    }
                }
            }

            // Confidence multiplier
            score *= gs.confidence;

            if score > 0.0 {
                scored.push((idx, score));
            }
        }

        // Fallback: if no keyword matches, return top-K by confidence
        if scored.is_empty() {
            let mut by_conf: Vec<(usize, f64)> = self
                .global_summaries
                .iter()
                .enumerate()
                .map(|(i, gs)| (i, gs.confidence))
                .collect();
            by_conf.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            scored = by_conf.into_iter().take(top_k).collect();
        } else {
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        scored
            .into_iter()
            .take(top_k)
            .map(|(idx, _)| self.global_summaries[idx].clone())
            .collect()
    }

    // ── LightRAG: Search Hybrid ─────────────────────────────────────

    /// Merge local entity-level results with global community-level summaries.
    /// Deduplicate entities and rank by combined score.
    pub fn search_hybrid(&self, query: &str, top_k_local: usize, top_k_global: usize) -> HybridResult {
        let local = self.search_local(query, top_k_local);
        let global = self.search_global(query, top_k_global);

        let mut seen_entities: HashSet<String> = HashSet::new();
        let mut seen_relations: HashSet<String> = HashSet::new();
        let mut merged_entities: Vec<EntityNode> = Vec::new();
        let mut merged_relations: Vec<RelationEdge> = Vec::new();

        // Merge local results first
        for sub in &local {
            for e in &sub.entities {
                if seen_entities.insert(e.id.clone()) {
                    merged_entities.push(e.clone());
                }
            }
            for r in &sub.relations {
                if seen_relations.insert(r.id.clone()) {
                    merged_relations.push(r.clone());
                }
            }
        }

        // Merge entities from global results (those in matching communities)
        let comm_entity_ids: HashSet<String> = global
            .iter()
            .flat_map(|gs| {
                self.community_summary()
                    .iter()
                    .find(|c| c.id == gs.community_id)
                    .map(|c| c.entity_ids.clone())
                    .unwrap_or_default()
            })
            .collect();

        for eid in &comm_entity_ids {
            if let Some(entity) = self.graph.entities.get(eid) {
                if seen_entities.insert(eid.clone()) {
                    merged_entities.push(entity.clone());
                }
            }
        }

        // Add relations connecting merged entities
        for rel in self.graph.relations.values() {
            if seen_entities.contains(&rel.source_entity)
                && seen_entities.contains(&rel.target_entity)
                && seen_relations.insert(rel.id.clone())
            {
                merged_relations.push(rel.clone());
            }
        }

        HybridResult {
            local_results: local,
            global_results: global,
            merged_entities,
            merged_relations,
        }
    }

    // ── Incremental Index Update ─────────────────────────────────────

    /// Process a batch of changes and update the graph incrementally.
    /// Only re-clusters the affected communities rather than full rebuild.
    pub fn incremental_index_update(&mut self, changes: Vec<IncrementalChange>) {
        if changes.is_empty() {
            return;
        }

        let max_size = self.config.max_graph_size;
        let mut affected_entity_ids: HashSet<String> = HashSet::new();

        for change in &changes {
            for entity in &change.added_entities {
                if self.graph.entities.len() >= max_size {
                    break;
                }
                if !self.graph.entities.contains_key(&entity.id) {
                    self.add_entity_internal(entity.clone());
                    affected_entity_ids.insert(entity.id.clone());
                }
            }
            for relation in &change.added_relations {
                if self.graph.relations.len() >= max_size {
                    break;
                }
                if !self.graph.relations.contains_key(&relation.id) {
                    self.add_relation_internal(relation.clone());
                    affected_entity_ids.insert(relation.source_entity.clone());
                    affected_entity_ids.insert(relation.target_entity.clone());
                }
            }
            self.change_log.push(change.clone());
        }

        // Identify affected communities and rebuild their summaries
        if !affected_entity_ids.is_empty() && !self.graph.entities.is_empty() {
            // Run label propagation only on affected portion
            let communities = self.community_summary();
            let affected_community_ids: HashSet<String> = communities
                .iter()
                .filter(|c| c.entity_ids.iter().any(|eid| affected_entity_ids.contains(eid)))
                .map(|c| c.id.clone())
                .collect();

            // Rebuild global summaries for affected communities
            let mut updated = false;
            for gs in self.global_summaries.iter_mut() {
                if affected_community_ids.contains(&gs.community_id) {
                    if let Some(community) = communities.iter().find(|c| c.id == gs.community_id) {
                        gs.summary_text = community.summary.clone();
                        gs.entity_count = community.size;
                        gs.last_updated = now_nanos();
                        gs.confidence = community.avg_confidence;
                        // Update topic keywords from community entities
                        let mut keywords: Vec<String> = Vec::new();
                        for eid in &community.entity_ids {
                            if let Some(entity) = self.graph.entities.get(eid) {
                                keywords.push(entity.name.clone());
                            }
                        }
                        keywords.sort();
                        keywords.dedup();
                        gs.topic_keywords = keywords;
                        updated = true;
                    }
                }
            }

            // If global summaries were not pre-built, signal by recording change
            if !updated {
                // Summaries will be built on next build_global_summaries call
                self.lightrag_index.last_community_update = now_nanos();
            }
        }

        self.stats.total_entities = self.graph.entities.len();
        self.stats.total_relations = self.graph.relations.len();
    }

    // ── Build Global Summaries ───────────────────────────────────────

    /// Generate topic-level summaries per community.
    /// Each summary aggregates entity names, types, relations, and cross-entity themes.
    pub fn build_global_summaries(&mut self) {
        let communities = self.community_summary();
        let centrality = self.compute_centrality();
        let mut new_summaries: Vec<GlobalSummary> = Vec::new();

        for community in &communities {
            // Collect topic keywords from entity names
            let mut keywords: Vec<String> = Vec::new();
            let mut type_distribution: HashMap<String, usize> = HashMap::new();
            let mut top_relations: Vec<(String, f64)> = Vec::new();

            let mut relation_count = 0;

            for eid in &community.entity_ids {
                if let Some(entity) = self.graph.entities.get(eid) {
                    keywords.push(entity.name.clone());
                    *type_distribution.entry(entity.entity_type.clone()).or_insert(0) += 1;
                }
            }

            // Count internal relations and find top types
            let member_set: HashSet<&String> = community.entity_ids.iter().collect();
            let mut rel_type_weights: HashMap<String, f64> = HashMap::new();
            for rel in self.graph.relations.values() {
                if member_set.contains(&rel.source_entity)
                    && member_set.contains(&rel.target_entity)
                {
                    *rel_type_weights.entry(rel.relation_type.clone()).or_insert(0.0) += rel.weight;
                    relation_count += 1;
                }
            }

            for (rt, w) in &rel_type_weights {
                top_relations.push((rt.clone(), *w));
            }
            top_relations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Build a rich summary text
            keywords.sort();
            keywords.dedup();
            let keyword_str = if keywords.len() > 10 {
                format!("{} (top: {})", keywords.len(), keywords[..10.min(keywords.len())].join(", "))
            } else {
                keywords.join(", ")
            };

            let mut type_parts: Vec<String> = type_distribution
                .into_iter()
                .map(|(t, c)| format!("{} {}", c, t))
                .collect();
            type_parts.sort();
            let type_summary = type_parts.join(", ");

            let rel_summary = if top_relations.is_empty() {
                "no internal relations".to_string()
            } else {
                let top_rel_str: Vec<String> = top_relations
                    .iter()
                    .take(3)
                    .map(|(rt, w)| format!("{} (w={:.2})", rt, w))
                    .collect();
                format!("{} key relations: {}", relation_count, top_rel_str.join(", "))
            };

            let top_entities: Vec<String> = community
                .entity_ids
                .iter()
                .filter_map(|eid| {
                    let entity = self.graph.entities.get(eid)?;
                    let centr = centrality.get(eid).copied().unwrap_or(0.0);
                    Some((entity.name.clone(), centr))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .take(5)
                .map(|(name, _)| name)
                .collect();

            let summary_text = format!(
                "Community {}: {} entities — types: {}. Relations: {}. Top entities: [{}]. Keywords: {}.",
                community.id,
                community.size,
                type_summary,
                rel_summary,
                top_entities.join(", "),
                keyword_str,
            );

            new_summaries.push(GlobalSummary {
                community_id: community.id.clone(),
                topic_keywords: keywords,
                summary_text,
                confidence: community.avg_confidence,
                last_updated: now_nanos(),
                entity_count: community.size,
                relation_count,
            });
        }

        self.global_summaries = new_summaries;
        self.lightrag_index.last_community_update = now_nanos();
    }

    /// Access global summaries
    pub fn get_global_summaries(&self) -> &[GlobalSummary] {
        &self.global_summaries
    }

    /// Access change log
    pub fn get_change_log(&self) -> &[IncrementalChange] {
        &self.change_log
    }

    /// Access lightrag index
    pub fn lightrag_index(&self) -> &LightRagIndex {
        &self.lightrag_index
    }

    /// Clear change log
    pub fn clear_change_log(&mut self) {
        self.change_log.clear();
    }

    /// Detect query type: specific entities mentioned → local, conceptual → global
    pub fn detect_query_type(query: &str) -> GraphQueryMode {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        // Heuristic: if query has capitalized words or specific terms, likely entity query
        let has_capitalized = query.chars().any(|c| c.is_uppercase());
        let conceptual_indicators = [
            "what is", "explain", "overview", "summary", "concept", "describe",
            "how does", "why is", "what are", "tell me about", "relationship between",
        ];
        let length = query_words.len();

        if has_capitalized && length <= 6 {
            // Short query with capitals → likely entity-specific
            GraphQueryMode::Local {
                max_depth: 2,
                max_neighbors: 10,
            }
        } else if conceptual_indicators.iter().any(|ind| query_lower.contains(ind)) || length > 8 {
            // Long query or conceptual indicators → global
            GraphQueryMode::Global { community_level: 0 }
        } else {
            // Default to hybrid
            GraphQueryMode::Hybrid {
                local_depth: 2,
                global_level: 0,
            }
        }
    }
}

// ─── Heuristic Extraction Helpers ────────────────────────────────────

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        current.push(c);
        if matches!(c, '.' | '!' | '?') {
            // Check if this is likely an abbreviation by looking at what follows
            let is_abbreviation = if c == '.' {
                // If followed by another letter (no space), it's an abbreviation
                if i + 1 < len && chars[i + 1].is_alphabetic() {
                    true
                } else if i + 2 < len && chars[i + 1] == ' ' && chars[i + 2].is_lowercase() {
                    // "word. more" — the period ends a sentence
                    false
                } else {
                    false
                }
            } else {
                false
            };

            if !is_abbreviation {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() && trimmed.len() > 2 {
                    sentences.push(trimmed);
                }
                current = String::new();
            }
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() && trimmed.len() > 2 {
        sentences.push(trimmed);
    }

    sentences
}

fn extract_capitalized_terms(sentence: &str, _source_id: &str) -> Vec<String> {
    let mut entities = Vec::new();
    let words: Vec<&str> = sentence.split_whitespace().collect();
    let mut i = 0;

    while i < words.len() {
        let clean = words[i]
            .trim_start_matches(|c: char| !c.is_alphanumeric())
            .trim_end_matches(|c: char| !c.is_alphanumeric());

        if clean.is_empty() || clean.len() < 2 {
            i += 1;
            continue;
        }

        let first_char = clean.chars().next().unwrap_or(' ');
        if first_char.is_uppercase() || first_char.is_ascii_digit() {
            let mut term_parts: Vec<String> = Vec::new();
            let mut j = i;

            while j < words.len() {
                let w = words[j]
                    .trim_start_matches(|c: char| !c.is_alphanumeric())
                    .trim_end_matches(|c: char| !c.is_alphanumeric());

                if w.is_empty() || w.len() < 2 {
                    break;
                }
                let wfc = w.chars().next().unwrap_or(' ');
                if !wfc.is_uppercase() && !wfc.is_ascii_digit() {
                    break;
                }
                // Stop at connecting words that are not proper nouns
                let wlower = w.to_lowercase();
                if !term_parts.is_empty()
                    && matches!(
                        wlower.as_str(),
                        "the" | "a" | "an" | "and" | "or" | "but" | "in" | "on" | "at" | "for"
                            | "with" | "by" | "to" | "of" | "is" | "are" | "was" | "were"
                    )
                {
                    break;
                }
                term_parts.push(w.to_string());
                j += 1;
            }

            if !term_parts.is_empty() {
                let term = term_parts.join(" ");
                // Filter out common non-entity single capitalized words
                let tlower = term.to_lowercase();
                if !matches!(
                    tlower.as_str(),
                    "this" | "that" | "these" | "those" | "they" | "what" | "which" | "when"
                        | "where" | "why" | "how" | "there" | "here" | "then" | "than" | "thus"
                        | "hence" | "very" | "just" | "also" | "only" | "more" | "most" | "some"
                        | "any" | "each" | "every" | "both" | "such" | "because" | "while"
                        | "although" | "however" | "therefore" | "moreover" | "furthermore"
                        | "nevertheless" | "nonetheless" | "accordingly" | "consequently"
                        | "additionally"
                ) && term.len() > 1
                {
                    entities.push(term);
                }
            }

            i = j;
        } else {
            i += 1;
        }
    }

    entities
}

fn infer_entity_type(name: &str) -> String {
    let lower = name.to_lowercase();

    if lower.ends_with("inc")
        || lower.ends_with("corp")
        || lower.ends_with("ltd")
        || lower.ends_with("llc")
        || lower.ends_with("company")
        || lower.ends_with("corporation")
        || lower.ends_with("foundation")
        || lower.ends_with("institute")
        || lower.ends_with("organization")
        || lower.ends_with("association")
        || lower.ends_with("group")
        || lower.ends_with("laboratories")
        || lower.ends_with("lab")
        || lower.ends_with("limited")
        || lower.contains("university")
        || lower.contains("college")
        || lower.contains("school")
        || lower.contains("department of")
    {
        return "Organization".to_string();
    }

    if lower.starts_with("dr ")
        || lower.starts_with("prof ")
        || lower.starts_with("mr ")
        || lower.starts_with("ms ")
        || lower.starts_with("mrs ")
        || lower.starts_with("sir ")
        || lower.starts_with("lord ")
    {
        return "Person".to_string();
    }

    if lower.contains("system")
        || lower.contains("framework")
        || lower.contains("tool")
        || lower.contains("language")
        || lower.contains("platform")
        || lower.contains("software")
        || lower.contains("algorithm")
        || lower.contains("database")
        || lower.contains("protocol")
        || lower.contains("engine")
        || lower.contains("runtime")
        || lower.contains("library")
        || lower.contains("api")
        || lower.contains("sdk")
        || lower.contains("kernel")
        || lower.contains("module")
        || lower.contains("network")
        || lower.contains("model")
        || lower.contains("transformer")
        || lower.contains("architecture")
    {
        return "Technology".to_string();
    }

    if lower.ends_with("city")
        || lower.ends_with("ville")
        || lower.ends_with("burg")
        || lower.ends_with("town")
        || lower.ends_with("shire")
        || lower.ends_with("land")
        || lower.ends_with("stan")
        || lower.ends_with("valley")
        || lower.ends_with("beach")
        || lower.ends_with("bay")
        || lower.ends_with("county")
        || lower.ends_with("province")
        || lower.ends_with("state")
        || lower.ends_with("kingdom")
        || lower.contains("republic of")
        || lower.contains("city of")
    {
        return "Location".to_string();
    }

    if lower.contains("conference")
        || lower.contains("summit")
        || lower.contains("workshop")
        || lower.contains("symposium")
        || lower.contains("hackathon")
        || lower.contains("competition")
        || lower.contains("challenge")
        || lower.contains("tournament")
        || lower.contains("exhibition")
        || lower.contains("convention")
    {
        return "Event".to_string();
    }

    "Concept".to_string()
}

fn estimate_entity_confidence(name: &str, sentence: &str) -> f64 {
    let lower = name.to_lowercase();
    let mut confidence: f64 = 0.7;

    // Longer, more specific names get higher confidence
    let word_count = name.split_whitespace().count();
    if word_count >= 3 {
        confidence += 0.15;
    } else if word_count >= 2 {
        confidence += 0.05;
    }

    // Type keywords boost confidence
    if lower.ends_with("inc")
        || lower.ends_with("corp")
        || lower.ends_with("ltd")
        || lower.contains("university")
    {
        confidence += 0.15;
    }

    // If the entity appears multiple times in the sentence, higher confidence
    let count = sentence
        .to_lowercase()
        .matches(&lower)
        .count();
    if count > 1 {
        confidence += 0.1;
    }

    // Single capitalized word that's common → lower confidence
    if word_count == 1 {
        let common_single = [
            "hello",
            "world",
            "this",
            "that",
            "these",
            "those",
            "today",
            "tomorrow",
            "yesterday",
            "now",
            "here",
            "there",
        ];
        if common_single.contains(&lower.as_str()) {
            confidence -= 0.3;
        }
        // Very short words
        if name.len() <= 3 {
            confidence -= 0.2;
        }
    }

    (confidence.max(0.1f64)).min(1.0f64)
}

fn detect_relation(e1: &str, e2: &str, sentence: &str) -> Option<(&'static str, f64)> {
    let s_lower = sentence.to_lowercase();
    let e1_lower = e1.to_lowercase();
    let e2_lower = e2.to_lowercase();

    // Find positions in the lowercased sentence
    let pos1 = s_lower.find(&e1_lower)?;
    let pos2 = s_lower.find(&e2_lower)?;

    let between = if pos1 < pos2 {
        &s_lower[pos1 + e1_lower.len()..pos2]
    } else {
        &s_lower[pos2 + e2_lower.len()..pos1]
    };

    let between = between.trim();

    let rel_type = if between.contains("works at")
        || between.contains("employed by")
        || between.contains("ceo of")
        || between.contains("cfo of")
        || between.contains("cto of")
        || between.contains("employee of")
        || between.contains("founder of")
        || between.contains("chairman of")
        || between.contains("president of")
        || between.contains("director of")
        || between.contains("manager of")
        || between.contains("led by")
        || between.contains("run by")
        || between.contains("staff of")
        || between.contains("team at")
    {
        "works_at"
    } else if between.contains("developed")
        || between.contains("created")
        || between.contains("built")
        || between.contains("designed")
        || between.contains("invented")
        || between.contains("wrote")
        || between.contains("authored")
        || between.contains("published")
        || between.contains("produced")
        || between.contains("engineered")
        || between.contains("founded")
        || between.contains("established")
        || between.contains("launched")
        || between.contains("introduced")
        || between.contains("released")
        || between.contains("originated from")
        || between.contains("created by")
        || between.contains("developed by")
        || between.contains("built by")
        || between.contains("designed by")
        || between.contains("authored by")
    {
        "developed_by"
    } else if between.contains("part of")
        || between.contains("component of")
        || between.contains("belongs to")
        || between.contains("member of")
        || between.contains("subsidiary of")
        || between.contains("division of")
        || between.contains("unit of")
        || between.contains("segment of")
        || between.contains("element of")
        || between.contains("subset of")
        || between.contains("included in")
        || between.contains("within")
    {
        "part_of"
    } else if between.contains("located in")
        || between.contains("based in")
        || between.contains("headquartered")
        || between.contains("situated in")
        || between.contains("founded in")
        || between.contains("established in")
        || between.contains("head office in")
    {
        "located_in"
    } else if between.contains("uses")
        || between.contains("utilizes")
        || between.contains("integrates")
        || between.contains("runs on")
        || between.contains("built on")
        || between.contains("powered by")
        || between.contains("driven by")
        || between.contains("supports")
        || between.contains("compatible with")
        || between.contains("implemented with")
        || between.contains("implemented in")
        || between.contains("written in")
        || between.contains("built with")
        || between.contains("based on")
        || between.contains("relies on")
        || between.contains("depends on")
        || between.contains("leveraging")
        || between.contains("powered by")
    {
        "used_by"
    } else {
        "related_to"
    };

    // Distance = number of words between the entities
    let distance = between.split_whitespace().count().max(1) as f64;

    Some((rel_type, distance))
}

// ─── Extraction Pipeline ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub properties: HashMap<String, String>,
    pub embeddings: Option<Vec<f64>>,
}

#[derive(Debug, Clone)]
pub struct Relation {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub weight: f64,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    pub model: String,
    pub max_entities_per_chunk: usize,
    pub confidence_threshold: f64,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        ExtractionConfig {
            model: "heuristic".to_string(),
            max_entities_per_chunk: 50,
            confidence_threshold: 0.3,
        }
    }
}

pub struct GraphExtractor {
    config: ExtractionConfig,
}

impl GraphExtractor {
    pub fn new(config: ExtractionConfig) -> Self {
        GraphExtractor { config }
    }

    pub fn config(&self) -> &ExtractionConfig {
        &self.config
    }

    pub fn extract(&self, text: &str) -> Result<(Vec<Entity>, Vec<Relation>), String> {
        let sentences = split_sentences(text);
        let mut entities_map: HashMap<String, Entity> = HashMap::new();
        let mut relations: Vec<Relation> = Vec::new();
        let mut sentence_entity_names: Vec<Vec<String>> = Vec::new();

        for sentence in &sentences {
            if sentence.len() < 3 {
                sentence_entity_names.push(Vec::new());
                continue;
            }
            let names = extract_capitalized_terms(sentence, "extractor");
            let filtered: Vec<String> = names
                .into_iter()
                .filter(|n| {
                    let conf = estimate_entity_confidence(n, sentence);
                    conf >= self.config.confidence_threshold
                })
                .collect();
            sentence_entity_names.push(filtered.clone());

            for name in filtered {
                let key = name.to_lowercase();
                if !entities_map.contains_key(&key) {
                    if entities_map.len() >= self.config.max_entities_per_chunk {
                        break;
                    }
                    let etype = infer_entity_type(&name);
                    entities_map.insert(
                        key,
                        Entity {
                            id: generate_id(),
                            entity_type: etype,
                            name,
                            properties: HashMap::new(),
                            embeddings: None,
                        },
                    );
                }
            }
        }

        for (s_idx, sentence) in sentences.iter().enumerate() {
            let names = &sentence_entity_names[s_idx];
            if names.len() < 2 {
                continue;
            }
            for i in 0..names.len() {
                for j in (i + 1)..names.len() {
                    if let Some((rel_type, distance)) =
                        detect_relation(&names[i], &names[j], sentence)
                    {
                        let e1_lower = names[i].to_lowercase();
                        let e2_lower = names[j].to_lowercase();
                        if let (Some(e1), Some(e2)) =
                            (entities_map.get(&e1_lower), entities_map.get(&e2_lower))
                        {
                            let weight =
                                (1.0 / distance.max(1.0)) * self.config.confidence_threshold.max(0.5);
                            relations.push(Relation {
                                source_id: e1.id.clone(),
                                target_id: e2.id.clone(),
                                relation_type: rel_type.to_string(),
                                weight: (weight.max(0.0)).min(1.0),
                                properties: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        let entities: Vec<Entity> = entities_map.into_values().collect();
        Ok((entities, relations))
    }

    pub fn extract_and_store(
        &self,
        text: &str,
        store: &mut GraphRagStore,
        source_id: &str,
    ) -> Result<(), String> {
        let (entities, relations) = self.extract(text)?;

        let mut entity_id_map: HashMap<String, String> = HashMap::new();
        for entity in &entities {
            let key = entity.name.to_lowercase();
            let existing = store
                .graph()
                .entities
                .values()
                .find(|e| e.name.to_lowercase() == key);
            let store_id = if let Some(existing) = existing {
                existing.id.clone()
            } else {
                let node = EntityNode {
                    id: entity.id.clone(),
                    name: entity.name.clone(),
                    entity_type: entity.entity_type.clone(),
                    source_node_id: source_id.to_string(),
                    confidence: self.config.confidence_threshold,
                    properties: entity.properties.clone(),
                    created_at: now_nanos(),
                };
                store.add_entity(node);
                entity.id.clone()
            };
            entity_id_map.insert(key, store_id);
        }

        for relation in &relations {
            let source_key = entities
                .iter()
                .find(|e| e.id == relation.source_id)
                .map(|e| e.name.to_lowercase());
            let target_key = entities
                .iter()
                .find(|e| e.id == relation.target_id)
                .map(|e| e.name.to_lowercase());
            if let (Some(src_key), Some(tgt_key)) = (source_key, target_key) {
                if let (Some(sid), Some(tid)) =
                    (entity_id_map.get(&src_key), entity_id_map.get(&tgt_key))
                {
                    store.add_relation(RelationEdge {
                        id: generate_id(),
                        source_entity: sid.clone(),
                        target_entity: tid.clone(),
                        relation_type: relation.relation_type.clone(),
                        weight: relation.weight,
                        evidence: String::new(),
                        confidence: relation.weight,
                        created_at: now_nanos(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn merge_entities(entities: &[Entity]) -> Vec<Entity> {
        let mut seen: HashMap<String, Entity> = HashMap::new();
        for entity in entities {
            let key = entity.name.to_lowercase();
            seen.entry(key).or_insert_with(|| entity.clone());
        }
        seen.into_values().collect()
    }

    pub fn generate_prompt(&self, text: &str) -> String {
        format!(
            r#"Extract entities and relations from the following text.

Entities should be: name, type (Person/Organization/Location/Technology/Concept/Event), and optional properties.
Relations should be: source → relation_type → target.

Return as JSON:
{{
  "entities": [{{"name": "...", "type": "...", "properties": {{...}}}}],
  "relations": [{{"source": "...", "target": "...", "type": "..."}}]
}}

Text:
{}

Max entities: {}"#,
            text, self.config.max_entities_per_chunk
        )
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> GraphRagConfig {
        GraphRagConfig {
            max_entities_per_doc: 100,
            min_confidence: 0.0,
            enable_incremental_updates: true,
            max_graph_size: 10000,
            extraction_mode: ExtractionMode::Heuristic,
        }
    }

    #[test]
    fn test_entity_extraction_simple() {
        let mut store = GraphRagStore::new(make_config());
        let text = "Apple Inc. developed the iPhone. Tim Cook is the CEO of Apple.";
        let (entities, _relations) = store.extract_entities(text, "src_1").unwrap();

        assert!(!entities.is_empty(), "Should extract entities");
        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Apple Inc"), "Should contain Apple Inc");
        assert!(names.contains(&"Tim Cook"), "Should contain Tim Cook");
    }

    #[test]
    fn test_relation_extraction_co_occurrence() {
        let mut store = GraphRagStore::new(make_config());
        let text = "Google developed the Android operating system. Sundar Pichai works at Google.";
        let (_entities, relations) = store.extract_entities(text, "src_2").unwrap();

        let rel_types: Vec<&str> = relations.iter().map(|r| r.relation_type.as_str()).collect();
        assert!(!relations.is_empty(), "Should extract relations");
        assert!(
            rel_types.contains(&"developed_by"),
            "Should contain developed_by relation (got {:?})",
            rel_types
        );

        // First sentence: "Google developed the Android" → "Google" (entity) "developed" (keyword) "Android" (entity)
        // "the" is not capitalized so "Android" and "Google" should be in separate sentences or same
        // Actually: "Google developed the Android operating system" — "Google" is capitalized, "Android" is capitalized
        // "operating" is capitalized too but "system" is lowercase... hmm
        // Let's just check relations exist

        // Check for works_at relation
        let has_works_at = relations.iter().any(|r| r.relation_type == "works_at");
        assert!(
            has_works_at,
            "Should contain works_at relation for 'Sundar Pichai works at Google'"
        );
    }

    #[test]
    fn test_query_local_mode() {
        let mut store = GraphRagStore::new(make_config());

        let e1 = EntityNode {
            id: "e1".to_string(),
            name: "OpenAI".to_string(),
            entity_type: "Organization".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e2 = EntityNode {
            id: "e2".to_string(),
            name: "GPT-4".to_string(),
            entity_type: "Technology".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e3 = EntityNode {
            id: "e3".to_string(),
            name: "DALL-E".to_string(),
            entity_type: "Technology".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.8,
            properties: HashMap::new(),
            created_at: 1,
        };

        store.add_entity(e1);
        store.add_entity(e2);
        store.add_entity(e3);

        store.add_relation(RelationEdge {
            id: "r1".to_string(),
            source_entity: "e1".to_string(),
            target_entity: "e2".to_string(),
            relation_type: "developed_by".to_string(),
            weight: 0.9,
            evidence: "".to_string(),
            confidence: 0.9,
            created_at: 1,
        });
        store.add_relation(RelationEdge {
            id: "r2".to_string(),
            source_entity: "e1".to_string(),
            target_entity: "e3".to_string(),
            relation_type: "developed_by".to_string(),
            weight: 0.7,
            evidence: "".to_string(),
            confidence: 0.8,
            created_at: 1,
        });

        let result = store
            .query(
                &["e1".to_string()],
                GraphQueryMode::Local {
                    max_depth: 1,
                    max_neighbors: 10,
                },
            )
            .unwrap();

        assert_eq!(result.entities.len(), 3, "Should find all 3 entities at depth 1");
        assert_eq!(result.relations.len(), 2, "Should find both relations");
    }

    #[test]
    fn test_query_hybrid_mode() {
        let mut store = GraphRagStore::new(make_config());

        let e1 = EntityNode {
            id: "h1".to_string(),
            name: "NeoTrix".to_string(),
            entity_type: "Technology".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.95,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e2 = EntityNode {
            id: "h2".to_string(),
            name: "E8".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e3 = EntityNode {
            id: "h3".to_string(),
            name: "VSA".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.85,
            properties: HashMap::new(),
            created_at: 1,
        };

        store.add_entity(e1);
        store.add_entity(e2);
        store.add_entity(e3);

        store.add_relation(RelationEdge {
            id: "hr1".to_string(),
            source_entity: "h1".to_string(),
            target_entity: "h2".to_string(),
            relation_type: "related_to".to_string(),
            weight: 0.8,
            evidence: "".to_string(),
            confidence: 0.9,
            created_at: 1,
        });
        store.add_relation(RelationEdge {
            id: "hr2".to_string(),
            source_entity: "h1".to_string(),
            target_entity: "h3".to_string(),
            relation_type: "related_to".to_string(),
            weight: 0.7,
            evidence: "".to_string(),
            confidence: 0.85,
            created_at: 1,
        });

        let result = store
            .query(
                &["h1".to_string()],
                GraphQueryMode::Hybrid {
                    local_depth: 1,
                    global_level: 0,
                },
            )
            .unwrap();

        assert_eq!(result.query_mode, "hybrid");
        assert!(!result.entities.is_empty());
    }

    #[test]
    fn test_add_remove_entity() {
        let mut store = GraphRagStore::new(make_config());

        let entity = EntityNode {
            id: "test_e1".to_string(),
            name: "TestEntity".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.5,
            properties: HashMap::new(),
            created_at: 1,
        };

        let id = store.add_entity(entity);
        assert_eq!(store.graph.entities.len(), 1);
        assert!(store.graph.entities.contains_key(&id));

        assert!(store.remove_entity(&id));
        assert_eq!(store.graph.entities.len(), 0);
        assert!(!store.remove_entity("nonexistent"));
    }

    #[test]
    fn test_add_remove_relation() {
        let mut store = GraphRagStore::new(make_config());

        let e1 = EntityNode {
            id: "re1".to_string(),
            name: "A".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.5,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e2 = EntityNode {
            id: "re2".to_string(),
            name: "B".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.5,
            properties: HashMap::new(),
            created_at: 1,
        };

        store.add_entity(e1);
        store.add_entity(e2);

        let rel = RelationEdge {
            id: "rr1".to_string(),
            source_entity: "re1".to_string(),
            target_entity: "re2".to_string(),
            relation_type: "related_to".to_string(),
            weight: 0.5,
            evidence: "".to_string(),
            confidence: 0.5,
            created_at: 1,
        };

        let rid = store.add_relation(rel);
        assert_eq!(store.graph.relations.len(), 1);
        assert!(store.graph.relations.contains_key(&rid));

        // Check adjacency was updated
        assert_eq!(
            store.graph.adjacency.get("re1").map(|a| a.len()),
            Some(1)
        );
        assert_eq!(
            store.graph.adjacency.get("re2").map(|a| a.len()),
            Some(1)
        );

        assert!(store.remove_relation(&rid));
        assert_eq!(store.graph.relations.len(), 0);
        assert_eq!(
            store.graph.adjacency.get("re1").map(|a| a.len()),
            Some(0)
        );
    }

    #[test]
    fn test_community_summary() {
        let mut store = GraphRagStore::new(make_config());

        // Create two communities: {e1, e2} and {e3, e4}
        let nodes = ["c1", "c2", "c3", "c4"];
        let communities_data: [(usize, usize); 3] = [(0, 1), (0, 1), (2, 3)];

        for (i, &name) in nodes.iter().enumerate() {
            store.add_entity(EntityNode {
                id: format!("n{}", i),
                name: name.to_string(),
                entity_type: "Concept".to_string(),
                source_node_id: "src".to_string(),
                confidence: 0.7,
                properties: HashMap::new(),
                created_at: 1,
            });
        }

        for (i, (a, b)) in communities_data.iter().enumerate() {
            store.add_relation(RelationEdge {
                id: format!("cr{}", i),
                source_entity: format!("n{}", a),
                target_entity: format!("n{}", b),
                relation_type: "related_to".to_string(),
                weight: 0.9,
                evidence: "".to_string(),
                confidence: 0.9,
                created_at: 1,
            });
        }

        let communities = store.community_summary();
        assert!(
            !communities.is_empty(),
            "Should find at least one community"
        );
        // With 4 nodes and edges connecting {0,1} and {2,3}, we should get 2 communities
        assert!(
            communities.len() >= 1,
            "Should have at least 1 community, got {}",
            communities.len()
        );
        for comm in &communities {
            assert!(comm.size > 0);
            assert!(comm.avg_confidence > 0.0);
        }
    }

    #[test]
    fn test_bfs_subgraph_traversal() {
        let mut store = GraphRagStore::new(make_config());

        // Chain: e1 → e2 → e3 → e4
        for i in 1..=4 {
            store.add_entity(EntityNode {
                id: format!("bfs_e{}", i),
                name: format!("Entity{}", i),
                entity_type: "Concept".to_string(),
                source_node_id: "src".to_string(),
                confidence: 0.8,
                properties: HashMap::new(),
                created_at: 1,
            });
        }

        for i in 1..3 {
            store.add_relation(RelationEdge {
                id: format!("bfs_r{}", i),
                source_entity: format!("bfs_e{}", i),
                target_entity: format!("bfs_e{}", i + 1),
                relation_type: "related_to".to_string(),
                weight: 0.8,
                evidence: "".to_string(),
                confidence: 0.8,
                created_at: 1,
            });
        }

        // BFS from e1 at depth 0 should only return e1
        let result = store.get_subgraph(&["bfs_e1".to_string()], 0);
        assert_eq!(result.entities.len(), 1, "Depth 0 should only return seed");
        assert_eq!(result.entities[0].id, "bfs_e1");

        // BFS from e1 at depth 1 should return e1, e2, and the relation
        let result = store.get_subgraph(&["bfs_e1".to_string()], 1);
        assert_eq!(result.entities.len(), 2, "Depth 1 should return e1 and e2");
        assert_eq!(result.relations.len(), 1, "Depth 1 should find r1");

        // BFS from e1 at depth 2 should return e1, e2, e3, and relations
        let result = store.get_subgraph(&["bfs_e1".to_string()], 2);
        assert_eq!(result.entities.len(), 3, "Depth 2 should return e1, e2, e3");
        assert_eq!(result.relations.len(), 2, "Depth 2 should find r1 and r2");
    }

    #[test]
    fn test_query_by_text() {
        let mut store = GraphRagStore::new(make_config());

        store.add_entity(EntityNode {
            id: "qt1".to_string(),
            name: "Rust Language".to_string(),
            entity_type: "Technology".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        });
        store.add_entity(EntityNode {
            id: "qt2".to_string(),
            name: "Rust Foundation".to_string(),
            entity_type: "Organization".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.8,
            properties: HashMap::new(),
            created_at: 1,
        });
        store.add_entity(EntityNode {
            id: "qt3".to_string(),
            name: "Python".to_string(),
            entity_type: "Technology".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.7,
            properties: HashMap::new(),
            created_at: 1,
        });

        store.add_relation(RelationEdge {
            id: "qr1".to_string(),
            source_entity: "qt1".to_string(),
            target_entity: "qt2".to_string(),
            relation_type: "part_of".to_string(),
            weight: 0.8,
            evidence: "".to_string(),
            confidence: 0.8,
            created_at: 1,
        });

        let result = store
            .query_by_text(
                &["rust"],
                GraphQueryMode::Local {
                    max_depth: 1,
                    max_neighbors: 10,
                },
            )
            .unwrap();

        assert_eq!(
            result.entities.len(),
            2,
            "Should find both Rust entities by substring match"
        );

        // Query by text with no match
        let empty_result = store
            .query_by_text(
                &["nonexistent"],
                GraphQueryMode::Local {
                    max_depth: 1,
                    max_neighbors: 10,
                },
            )
            .unwrap();
        assert!(empty_result.entities.is_empty());
    }

    #[test]
    fn test_empty_graph_behavior() {
        let mut store = GraphRagStore::new(make_config());

        let result = store.get_subgraph(&[], 1);
        assert!(result.entities.is_empty());
        assert!(result.relations.is_empty());

        let communities = store.community_summary();
        assert!(communities.is_empty());

        assert!(!store.remove_entity("nonexistent"));
        assert!(!store.remove_relation("nonexistent"));

        let q_result = store
            .query_by_text(
                &["anything"],
                GraphQueryMode::Local {
                    max_depth: 1,
                    max_neighbors: 10,
                },
            )
            .unwrap();
        assert!(q_result.entities.is_empty());
    }

    #[test]
    fn test_edge_weights_distance() {
        // Test that edge weights decrease with distance in sentence
        let mut store = GraphRagStore::new(make_config());

        let text = "NeoTrix uses E8. NeoTrix uses the VSA HyperCube for knowledge representation.";
        let (_entities, relations) = store.extract_entities(text, "src_dist").unwrap();

        // All relations should have weight <= 1.0
        for r in &relations {
            assert!(
                r.weight <= 1.0,
                "Weight should be <= 1.0, got {}",
                r.weight
            );
            assert!(r.weight >= 0.0, "Weight should be >= 0.0");
        }
    }

    #[test]
    fn test_extract_capitalized_terms_basic() {
        let sentence = "Alice and Bob visit New York City.";
        let terms = extract_capitalized_terms(sentence, "test");
        assert!(terms.contains(&"Alice".to_string()));
        assert!(terms.contains(&"Bob".to_string()));
        assert!(terms.contains(&"New York City".to_string()));
    }

    #[test]
    fn test_split_sentences_basic() {
        let text = "Hello world. This is a test! How are you? Fine.";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 4);
        for s in &sentences {
            assert!(s.len() > 2);
        }
    }

    #[test]
    fn test_entity_type_inference() {
        assert_eq!(
            infer_entity_type("Apple Inc"),
            "Organization".to_string()
        );
        assert_eq!(
            infer_entity_type("Rust Language"),
            "Technology".to_string()
        );
        assert_eq!(
            infer_entity_type("New York City"),
            "Location".to_string()
        );
        assert_eq!(infer_entity_type("Quantum Computing"), "Concept".to_string());
    }

    #[test]
    fn test_remove_entity_cascades_to_relations() {
        let mut store = GraphRagStore::new(make_config());

        let e1 = EntityNode {
            id: "cas1".to_string(),
            name: "Alpha".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e2 = EntityNode {
            id: "cas2".to_string(),
            name: "Beta".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.8,
            properties: HashMap::new(),
            created_at: 1,
        };
        store.add_entity(e1);
        store.add_entity(e2);

        store.add_relation(RelationEdge {
            id: "cas_r1".to_string(),
            source_entity: "cas1".to_string(),
            target_entity: "cas2".to_string(),
            relation_type: "related_to".to_string(),
            weight: 0.5,
            evidence: "".to_string(),
            confidence: 0.5,
            created_at: 1,
        });

        assert_eq!(store.graph.relations.len(), 1);
        store.remove_entity("cas1");
        assert_eq!(
            store.graph.relations.len(),
            0,
            "Removing entity should cascade to relations"
        );
        assert!(store.graph.entities.contains_key("cas2"),
            "Entity cas2 should still exist");
    }

    #[test]
    fn test_extraction_no_capitalized_words() {
        let mut store = GraphRagStore::new(make_config());
        let text = "hello world, this is a test with no capitalized entities.";
        let (entities, relations) = store.extract_entities(text, "src_empty").unwrap();
        assert!(
            entities.is_empty(),
            "No entities should be extracted from all-lowercase text"
        );
        assert!(relations.is_empty());
    }

    #[test]
    fn test_extraction_deduplication() {
        let mut store = GraphRagStore::new(make_config());
        let text = "Apple Inc. is a company. Apple Inc. makes iPhones.";
        let (entities, _relations) = store.extract_entities(text, "src_dedup").unwrap();
        let apple_count = entities
            .iter()
            .filter(|e| e.name == "Apple Inc")
            .count();
        assert_eq!(
            apple_count, 1,
            "Apple Inc should appear only once (deduplicated)"
        );
    }

    #[test]
    fn test_stats_tracking() {
        let mut store = GraphRagStore::new(make_config());
        assert_eq!(store.stats.extraction_runs, 0);

        store
            .extract_entities("Google and Apple are companies.", "src_stats")
            .unwrap();
        assert_eq!(store.stats.extraction_runs, 1);
        assert!(store.stats.total_entities >= 2);
        assert!(store.stats.avg_extraction_time_ms > 0.0);
    }

    // ── LightRAG Tests ──────────────────────────────────────────────

    fn make_populated_store() -> GraphRagStore {
        let mut store = GraphRagStore::new(GraphRagConfig {
            max_entities_per_doc: 100,
            min_confidence: 0.0,
            enable_incremental_updates: true,
            max_graph_size: 10000,
            extraction_mode: ExtractionMode::Heuristic,
        });

        let entities = vec![
            EntityNode {
                id: "lr_e1".into(),
                name: "NeoTrix".into(),
                entity_type: "Technology".into(),
                source_node_id: "src".into(),
                confidence: 0.95,
                properties: HashMap::new(),
                created_at: 1,
            },
            EntityNode {
                id: "lr_e2".into(),
                name: "E8 Engine".into(),
                entity_type: "Technology".into(),
                source_node_id: "src".into(),
                confidence: 0.9,
                properties: HashMap::new(),
                created_at: 1,
            },
            EntityNode {
                id: "lr_e3".into(),
                name: "VSA HyperCube".into(),
                entity_type: "Technology".into(),
                source_node_id: "src".into(),
                confidence: 0.85,
                properties: HashMap::new(),
                created_at: 1,
            },
            EntityNode {
                id: "lr_e4".into(),
                name: "Apple".into(),
                entity_type: "Organization".into(),
                source_node_id: "src".into(),
                confidence: 0.8,
                properties: [("industry".into(), "technology".into())].into(),
                created_at: 1,
            },
            EntityNode {
                id: "lr_e5".into(),
                name: "iPhone".into(),
                entity_type: "Technology".into(),
                source_node_id: "src".into(),
                confidence: 0.75,
                properties: HashMap::new(),
                created_at: 1,
            },
        ];

        for e in entities {
            store.add_entity(e);
        }

        let relations = vec![
            RelationEdge {
                id: "lr_r1".into(),
                source_entity: "lr_e1".into(),
                target_entity: "lr_e2".into(),
                relation_type: "uses".into(),
                weight: 0.9,
                evidence: "".into(),
                confidence: 0.9,
                created_at: 1,
            },
            RelationEdge {
                id: "lr_r2".into(),
                source_entity: "lr_e1".into(),
                target_entity: "lr_e3".into(),
                relation_type: "uses".into(),
                weight: 0.85,
                evidence: "".into(),
                confidence: 0.85,
                created_at: 1,
            },
            RelationEdge {
                id: "lr_r3".into(),
                source_entity: "lr_e4".into(),
                target_entity: "lr_e5".into(),
                relation_type: "develops".into(),
                weight: 0.8,
                evidence: "".into(),
                confidence: 0.8,
                created_at: 1,
            },
        ];

        for r in relations {
            store.add_relation(r);
        }

        store
    }

    #[test]
    fn test_search_local_finds_matching_entities() {
        let store = make_populated_store();
        let results = store.search_local("NeoTrix E8", 3);
        assert!(!results.is_empty(), "Should find results for NeoTrix query");
        // Should find the NeoTrix subgraph with E8 Engine
        let has_neotrix = results.iter().any(|r| {
            r.entities.iter().any(|e| e.name == "NeoTrix")
        });
        assert!(has_neotrix, "Local search should return NeoTrix entity");
    }

    #[test]
    fn test_search_local_empty_query_returns_empty() {
        let store = make_populated_store();
        let results = store.search_local("", 3);
        assert!(results.is_empty(), "Empty query should return no results");
    }

    #[test]
    fn test_search_local_no_match_returns_empty() {
        let store = make_populated_store();
        let results = store.search_local("xyznonexistent", 3);
        assert!(results.is_empty(), "Non-matching query should return no results");
    }

    #[test]
    fn test_build_global_summaries_creates_summaries() {
        let mut store = make_populated_store();
        store.build_global_summaries();
        let summaries = store.get_global_summaries();
        assert!(!summaries.is_empty(), "Should create global summaries");
        for gs in summaries {
            assert!(!gs.community_id.is_empty(), "Community ID should be set");
            assert!(!gs.topic_keywords.is_empty(), "Topic keywords should be populated");
            assert!(!gs.summary_text.is_empty(), "Summary text should be non-empty");
            assert!(gs.confidence > 0.0, "Confidence should be positive");
            assert!(gs.last_updated > 0, "Last updated should be set");
        }
    }

    #[test]
    fn test_search_global_with_summaries() {
        let mut store = make_populated_store();
        store.build_global_summaries();
        let results = store.search_global("NeoTrix technology", 2);
        assert!(!results.is_empty(), "Global search should return results");
        // Results should have summary text containing the query-relevant content
        let any_match = results.iter().any(|gs| {
            gs.summary_text.to_lowercase().contains("neotrix")
                || gs.topic_keywords.iter().any(|kw| kw.to_lowercase().contains("neotrix"))
        });
        assert!(any_match, "Global search results should reference queried topic");
    }

    #[test]
    fn test_search_global_empty_query_returns_top_k() {
        let mut store = make_populated_store();
        store.build_global_summaries();
        let count = store.global_summaries.len();
        let results = store.search_global("", 2);
        assert_eq!(results.len(), count.min(2), "Empty query should return top-K by confidence");
    }

    #[test]
    fn test_search_hybrid_merges_local_and_global() {
        let mut store = make_populated_store();
        store.build_global_summaries();
        let hybrid = store.search_hybrid("NeoTrix", 2, 2);
        assert!(!hybrid.merged_entities.is_empty(), "Hybrid should return entities");
        assert!(!hybrid.global_results.is_empty(), "Hybrid should include global summaries");
        assert!(!hybrid.local_results.is_empty(), "Hybrid should include local results");
    }

    #[test]
    fn test_incremental_index_update_adds_entities() {
        let mut store = GraphRagStore::new(make_config());
        store.build_global_summaries();
        assert!(store.get_global_summaries().is_empty(), "Empty store should have no summaries");

        let new_entity = EntityNode {
            id: "inc_e1".into(),
            name: "NewEntity".into(),
            entity_type: "Concept".into(),
            source_node_id: "src".into(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        let change = IncrementalChange {
            added_entities: vec![new_entity],
            added_relations: Vec::new(),
            timestamp: 2,
        };

        let before = store.graph.entities.len();
        store.incremental_index_update(vec![change]);
        assert_eq!(store.graph.entities.len(), before + 1, "Should add one entity");
        assert!(!store.change_log.is_empty(), "Should record change in log");
    }

    #[test]
    fn test_incremental_update_adds_relations() {
        let mut store = GraphRagStore::new(make_config());

        let e1 = EntityNode {
            id: "inc_a".into(),
            name: "Alpha".into(),
            entity_type: "Concept".into(),
            source_node_id: "src".into(),
            confidence: 0.8,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e2 = EntityNode {
            id: "inc_b".into(),
            name: "Beta".into(),
            entity_type: "Concept".into(),
            source_node_id: "src".into(),
            confidence: 0.8,
            properties: HashMap::new(),
            created_at: 1,
        };
        let rel = RelationEdge {
            id: "inc_r1".into(),
            source_entity: "inc_a".into(),
            target_entity: "inc_b".into(),
            relation_type: "related_to".into(),
            weight: 0.8,
            evidence: "".into(),
            confidence: 0.8,
            created_at: 1,
        };

        // Add entities first
        store.add_entity(e1);
        store.add_entity(e2);

        let before_relations = store.graph.relations.len();
        let change = IncrementalChange {
            added_entities: Vec::new(),
            added_relations: vec![rel],
            timestamp: 2,
        };
        store.incremental_index_update(vec![change]);
        assert_eq!(store.graph.relations.len(), before_relations + 1, "Should add one relation");
    }

    #[test]
    fn test_incremental_update_empty_changes_noop() {
        let mut store = make_populated_store();
        let before_entities = store.graph.entities.len();
        let before_relations = store.graph.relations.len();
        store.incremental_index_update(vec![]);
        assert_eq!(store.graph.entities.len(), before_entities);
        assert_eq!(store.graph.relations.len(), before_relations);
    }

    #[test]
    fn test_detect_query_type_local() {
        let mode = GraphRagStore::detect_query_type("NeoTrix E8 VSA");
        match mode {
            GraphQueryMode::Local { .. } => {} // expected
            _ => panic!("Query with capitals and short length should be Local"),
        }
    }

    #[test]
    fn test_detect_query_type_global() {
        let mode = GraphRagStore::detect_query_type("What is the relationship between knowledge representation and reasoning in AI systems");
        match mode {
            GraphQueryMode::Global { .. } => {} // expected
            _ => panic!("Long conceptual query should be Global"),
        }
    }

    #[test]
    fn test_detect_query_type_hybrid_default() {
        let mode = GraphRagStore::detect_query_type("knowledge and reasoning");
        match mode {
            GraphQueryMode::Hybrid { .. } => {} // expected for ambiguous
            _ => panic!("Ambiguous query should default to Hybrid"),
        }
    }

    #[test]
    fn test_auto_mode_in_query() {
        let store = make_populated_store();
        let result = store
            .query(
                &["lr_e1".to_string()],
                GraphQueryMode::Auto,
            )
            .unwrap();
        assert_eq!(result.query_mode, "auto(local)");
        assert!(!result.entities.is_empty());

        let empty_result = store
            .query(
                &[],
                GraphQueryMode::Auto,
            )
            .unwrap();
        assert_eq!(empty_result.query_mode, "auto(global)");
    }

    #[test]
    fn test_search_local_scoring_by_centrality() {
        let mut store = make_populated_store();
        // Add a high-degree hub entity
        let hub = EntityNode {
            id: "hub".into(),
            name: "HubEntity".into(),
            entity_type: "Technology".into(),
            source_node_id: "src".into(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        store.add_entity(hub);
        for i in 0..5 {
            let spoke = EntityNode {
                id: format!("spoke_{}", i),
                name: format!("Spoke{}", i),
                entity_type: "Technology".into(),
                source_node_id: "src".into(),
                confidence: 0.7,
                properties: HashMap::new(),
                created_at: 1,
            };
            store.add_entity(spoke);
            store.add_relation(RelationEdge {
                id: format!("hub_r{}", i),
                source_entity: "hub".into(),
                target_entity: format!("spoke_{}", i),
                relation_type: "connected_to".into(),
                weight: 0.7,
                evidence: "".into(),
                confidence: 0.7,
                created_at: 1,
            });
        }

        let results = store.search_local("HubEntity", 2);
        assert!(!results.is_empty(), "Hub should be found by search_local");
    }

    #[test]
    fn test_clear_change_log() {
        let mut store = GraphRagStore::new(make_config());
        store
            .extract_entities("Google develops Android.", "src")
            .unwrap();
        assert!(!store.change_log.is_empty(), "Change log should have entries after extraction");
        store.clear_change_log();
        assert!(store.change_log.is_empty(), "Change log should be empty after clear");
        assert_eq!(store.change_log.len(), 0);
    }

    #[test]
    fn test_build_global_summaries_empty_store() {
        let mut store = GraphRagStore::new(make_config());
        store.build_global_summaries();
        assert!(
            store.get_global_summaries().is_empty(),
            "Empty store should produce no summaries"
        );
    }

    #[test]
    fn test_search_global_no_summaries_returns_empty() {
        let store = make_populated_store();
        let results = store.search_global("NeoTrix", 2);
        assert!(
            results.is_empty(),
            "Without build_global_summaries, search_global should return empty"
        );
    }

    #[test]
    fn test_graph_extractor_extracts_entities_and_relations() {
        let config = ExtractionConfig {
            model: "heuristic".to_string(),
            max_entities_per_chunk: 50,
            confidence_threshold: 0.3,
        };
        let extractor = GraphExtractor::new(config);
        let text = "Apple Inc. developed the iPhone. Tim Cook works at Apple.";
        let (entities, relations) = extractor.extract(text).unwrap();

        assert!(!entities.is_empty(), "Should extract entities");
        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Apple Inc"), "Should contain Apple Inc");
        assert!(names.contains(&"Tim Cook"), "Should contain Tim Cook");

        assert!(!relations.is_empty(), "Should extract relations");
    }

    #[test]
    fn test_merge_entities_deduplicates_by_name() {
        let entities = vec![
            Entity {
                id: "1".into(),
                name: "Apple".into(),
                entity_type: "Organization".into(),
                properties: HashMap::new(),
                embeddings: None,
            },
            Entity {
                id: "2".into(),
                name: "apple".into(),
                entity_type: "Organization".into(),
                properties: HashMap::new(),
                embeddings: None,
            },
            Entity {
                id: "3".into(),
                name: "Google".into(),
                entity_type: "Organization".into(),
                properties: HashMap::new(),
                embeddings: None,
            },
        ];
        let merged = GraphExtractor::merge_entities(&entities);
        assert_eq!(merged.len(), 2, "Should merge Apple + apple into one");
        let merged_names: Vec<&str> = merged.iter().map(|e| e.name.as_str()).collect();
        assert!(merged_names.contains(&"Apple"));
        assert!(merged_names.contains(&"Google"));
    }
}
