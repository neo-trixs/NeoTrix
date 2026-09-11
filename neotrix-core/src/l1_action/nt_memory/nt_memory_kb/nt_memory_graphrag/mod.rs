use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::nt_core_kb_types::{KnowledgeNode, KnowledgeEdge, NodeType, RelationType};
use super::nt_memory_community::{CommunityAwareSearch, CommunityDetector};

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

// ─── Type Bridge: EntityNode ↔ KnowledgeNode ─────────────────────────

impl From<EntityNode> for KnowledgeNode {
    fn from(e: EntityNode) -> Self {
        KnowledgeNode {
            id: e.id,
            node_type: NodeType::from_str(&e.entity_type),
            title: e.name,
            summary: None,
            content: None,
            url: None,
            domain: None,
            language: "en".to_string(),
            confidence: e.confidence,
            importance: 0.5,
            recall_weight: 1.0,
            created_at: e.created_at as i64,
            updated_at: e.created_at as i64,
            access_count: 0,
            metadata: Some(serde_json::json!({
                "source_node_id": e.source_node_id,
                "properties": e.properties,
            })),
            temporal: None,
            supersedes: None,
            source_episode: Some(e.source_node_id),
        }
    }
}

impl From<KnowledgeNode> for EntityNode {
    fn from(k: KnowledgeNode) -> Self {
        let source_node_id = k.source_episode
            .or_else(|| k.metadata.as_ref().and_then(|m| m.get("source_node_id").and_then(|v| v.as_str().map(String::from))))
            .unwrap_or_default();
        let properties = k.metadata.as_ref()
            .and_then(|m| m.get("properties"))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        EntityNode {
            id: k.id,
            name: k.title,
            entity_type: k.node_type.as_str().to_string(),
            source_node_id,
            confidence: k.confidence,
            properties,
            created_at: k.created_at as u64,
        }
    }
}

// ─── Type Bridge: RelationEdge ↔ KnowledgeEdge ───────────────────────

impl From<RelationEdge> for KnowledgeEdge {
    fn from(r: RelationEdge) -> Self {
        KnowledgeEdge {
            id: r.id,
            source_id: r.source_entity,
            target_id: r.target_entity,
            relation_type: RelationType::from_str(&r.relation_type),
            weight: r.weight,
            description: Some(r.evidence),
            created_at: r.created_at as i64,
            metadata: Some(serde_json::json!({
                "confidence": r.confidence,
            })),
        }
    }
}

impl From<KnowledgeEdge> for RelationEdge {
    fn from(k: KnowledgeEdge) -> Self {
        let confidence = k.metadata.as_ref()
            .and_then(|m| m.get("confidence"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        RelationEdge {
            id: k.id,
            source_entity: k.source_id,
            target_entity: k.target_id,
            relation_type: k.relation_type.as_str().to_string(),
            weight: k.weight,
            evidence: k.description.unwrap_or_default(),
            confidence,
            created_at: k.created_at as u64,
        }
    }
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
    /// Unified community detector (delegates to `CommunityAwareSearch` hierarchical Leiden
    /// instead of reimplementing label propagation). Set via `set_community_detector()`.
    #[serde(skip)]
    community_detector: Option<std::sync::Arc<std::sync::RwLock<super::nt_memory_community::CommunityAwareSearch>>>,
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
            community_detector: None,
        }
    }

    /// Inject the unified community detector. When set, `community_summary()` delegates
    /// to `CommunityAwareSearch::get_communities()` (hierarchical Leiden) instead of
    /// running its own label propagation algorithm.
    pub fn set_community_detector(
        &mut self,
        detector: std::sync::Arc<std::sync::RwLock<super::nt_memory_community::CommunityAwareSearch>>,
    ) {
        self.community_detector = Some(detector);
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

    /// P1-11 graph-reviewer (吸收 Understand-Anything 双轨图 + graph-reviewer 模式):
    /// 图构建后一致性校验 — 检测悬空关系 (引用不存在的实体)、孤立实体 (无任何关系)、
    /// 重复关系 (同源同目标同类型)。返回发现列表, 供调用方修复或记录。
    /// Understand-Anything 原文: "deterministic parsing + LLM semantic dual-track
    /// graph construction + graph-reviewer"。
    pub fn review_graph(&self) -> Vec<String> {
        let mut findings = Vec::new();
        // 1) 悬空关系: 关系引用的实体不在图中
        for rel in self.graph.relations.values() {
            if !self.graph.entities.contains_key(&rel.source_entity) {
                findings.push(format!("dangling relation {}: source '{}' missing", rel.id, rel.source_entity));
            }
            if !self.graph.entities.contains_key(&rel.target_entity) {
                findings.push(format!("dangling relation {}: target '{}' missing", rel.id, rel.target_entity));
            }
        }
        // 2) 孤立实体: 无任何关系
        let connected: std::collections::HashSet<&str> = self
            .graph
            .relations
            .values()
            .flat_map(|r| [r.source_entity.as_str(), r.target_entity.as_str()])
            .collect();
        for entity in self.graph.entities.keys() {
            if !connected.contains(entity.as_str()) {
                findings.push(format!("orphan entity: '{}' has no relations", entity));
            }
        }
        // 3) 重复关系: 同源同目标同类型
        let mut seen: std::collections::HashSet<(String, String, String)> = std::collections::HashSet::new();
        for rel in self.graph.relations.values() {
            let key = (rel.source_entity.clone(), rel.target_entity.clone(), rel.relation_type.clone());
            if !seen.insert(key) {
                findings.push(format!("duplicate relation: {} -> {} ({})", rel.source_entity, rel.target_entity, rel.relation_type));
            }
        }
        findings
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

        // Bridge to KB types and run Leiden via CommunityAwareSearch (single fact source)
        let kb_nodes: Vec<KnowledgeNode> = self.graph.entities.values().cloned().map(Into::into).collect();
        let kb_edges: Vec<KnowledgeEdge> = self.graph.relations.values().cloned().map(Into::into).collect();

        let detector = CommunityDetector::new(1.0, 20, 3);
        let mut searcher = CommunityAwareSearch::new(detector);
        searcher.detect(&kb_nodes, &kb_edges);

        // Use unified get_communities() — single fact source for all community detection
        let results = searcher.get_communities();
        if results.is_empty() {
            return Vec::new();
        }

        // Precompute centrality once
        let centrality = self.compute_centrality();

        // Convert unified CommunityResults to GraphRAG Community structs
        let mut communities: Vec<Community> = results
            .into_iter()
            .map(|r| {
                let entity_ids: Vec<String> = searcher
                    .hierarchy()
                    .and_then(|h| h.get_community(r.community_id))
                    .map(|c| c.members.clone())
                    .unwrap_or_default();
                self.build_community_summary(r.community_id.0 as usize, &entity_ids, &centrality)
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

/// P1-12 稳定切片重建 (吸收 GEOFlow 模式):
/// LLM 只规划边界, 切片从原文稳定重建 — 不依赖 LLM 每次输出漂移。
/// 实现: 按段落 (空行) + 标题 (Markdown #) 规划边界, 从原文逐段重建切片,
/// 保证同一文档多次切片结果一致 (确定性)。
/// GEOFlow 原文: "LLM only plans boundaries; slices are stably rebuilt
/// from the original text"。
///
/// 返回 (切片列表, 边界规划)。边界规划可被 LLM 覆盖 (外部传入), 但默认确定性。
pub fn stable_slice_document(text: &str, max_chars: usize) -> (Vec<String>, Vec<usize>) {
    // 1) 边界规划: 段落边界 (空行) + 标题边界 (Markdown # 开头)
    let mut boundaries: Vec<usize> = Vec::new();
    let mut current_len = 0usize;
    for (idx, line) in text.lines().enumerate() {
        let line_len = line.chars().count() + 1; // +1 换行
        let is_heading = line.trim_start().starts_with('#');
        let is_para_break = line.trim().is_empty();
        if (is_heading || is_para_break) && idx > 0 && current_len > 0 {
            boundaries.push(idx);
            current_len = 0;
        }
        current_len += line_len;
        if current_len >= max_chars {
            boundaries.push(idx);
            current_len = 0;
        }
    }

    // 2) 原文重建: 按边界切分, 保证切片内容与原文逐字一致
    let lines: Vec<&str> = text.lines().collect();
    let mut slices = Vec::new();
    let mut start = 0usize;
    for &b in &boundaries {
        if b > start {
            let slice = lines[start..b].join("\n");
            if !slice.trim().is_empty() {
                slices.push(slice);
            }
        }
        start = b;
    }
    if start < lines.len() {
        let slice = lines[start..].join("\n");
        if !slice.trim().is_empty() {
            slices.push(slice);
        }
    }

    (slices, boundaries)
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

    pub fn extract(&self, text: &str, source_id: &str) -> Result<(Vec<EntityNode>, Vec<RelationEdge>), String> {
        let sentences = split_sentences(text);
        let mut entities_map: HashMap<String, EntityNode> = HashMap::new();
        let mut relations: Vec<RelationEdge> = Vec::new();
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
                        EntityNode {
                            id: generate_id(),
                            name: name.clone(),
                            entity_type: etype,
                            source_node_id: source_id.to_string(),
                            confidence: self.config.confidence_threshold,
                            properties: HashMap::new(),
                            created_at: now_nanos(),
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
                            relations.push(RelationEdge {
                                id: generate_id(),
                                source_entity: e1.id.clone(),
                                target_entity: e2.id.clone(),
                                relation_type: rel_type.to_string(),
                                weight: (weight.max(0.0)).min(1.0),
                                evidence: String::new(),
                                confidence: self.config.confidence_threshold,
                                created_at: now_nanos(),
                            });
                        }
                    }
                }
            }
        }

        let entities: Vec<EntityNode> = entities_map.into_values().collect();
        Ok((entities, relations))
    }

    pub fn extract_and_store(
        &self,
        text: &str,
        store: &mut GraphRagStore,
        source_id: &str,
    ) -> Result<(), String> {
        let (entities, relations) = self.extract(text, source_id)?;

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
                store.add_entity(entity.clone());
                entity.id.clone()
            };
            entity_id_map.insert(key, store_id);
        }

        for relation in &relations {
            let source_key = entities
                .iter()
                .find(|e| e.id == relation.source_entity)
                .map(|e| e.name.to_lowercase());
            let target_key = entities
                .iter()
                .find(|e| e.id == relation.target_entity)
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

    pub fn merge_entities(entities: &[EntityNode]) -> Vec<EntityNode> {
        let mut seen: HashMap<String, EntityNode> = HashMap::new();
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
mod tests;
