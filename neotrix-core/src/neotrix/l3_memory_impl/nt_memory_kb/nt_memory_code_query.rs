use std::collections::{HashMap, HashSet, VecDeque};
use super::nt_memory_types::*;

#[derive(Debug, Clone)]
pub struct CodeEntity {
    pub name: String,
    pub kind: CodeEntityKind,
    pub file_path: Option<String>,
    pub line_range: Option<(usize, usize)>,
    pub doc_comment: Option<String>,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeEntityKind {
    Module, Function, Method, Struct, Enum, Trait, Impl,
    TypeAlias, Const, Static, Macro, Attribute, File,
}

impl CodeEntityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CodeEntityKind::Module => "module",
            CodeEntityKind::Function => "function",
            CodeEntityKind::Method => "method",
            CodeEntityKind::Struct => "struct",
            CodeEntityKind::Enum => "enum",
            CodeEntityKind::Trait => "trait",
            CodeEntityKind::Impl => "impl",
            CodeEntityKind::TypeAlias => "type_alias",
            CodeEntityKind::Const => "const",
            CodeEntityKind::Static => "static",
            CodeEntityKind::Macro => "macro",
            CodeEntityKind::Attribute => "attribute",
            CodeEntityKind::File => "file",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DependencyChain {
    pub from: CodeEntity,
    pub to: CodeEntity,
    pub hops: Vec<Hop>,
    pub total_weight: f64,
}

#[derive(Debug, Clone)]
pub struct Hop {
    pub source_name: String,
    pub target_name: String,
    pub relation: String,
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub struct CodeGraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub entity_counts: HashMap<String, usize>,
    pub relation_counts: HashMap<String, usize>,
    pub hub_nodes: Vec<(String, usize)>,
    pub isolated_nodes: Vec<String>,
}

// ── Explosion Radius Analysis (absorbed from code-review-graph) ──

#[derive(Debug, Clone)]
pub struct ExplosionRadius {
    /// The name of the entity that changed.
    pub changed_entity: String,
    /// All entities affected within the radius, with depth and path info.
    pub affected_entities: Vec<AffectedEntity>,
    /// Total number of affected entities.
    pub total_affected: usize,
    /// Maximum depth reached.
    pub max_depth: usize,
    /// Weighted impact score (0.0–1.0).
    pub impact_score: f64,
    /// Categorized risk level.
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone)]
pub struct AffectedEntity {
    pub name: String,
    pub kind: CodeEntityKind,
    pub depth: usize,
    pub path_from_change: Vec<String>,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

/// Compute the explosion radius for a changed entity: all entities that would be
/// transitively affected within `max_depth` hops. Uses BFS from the change point.
pub fn explosion_radius(
    nodes: &[KnowledgeNode],
    edges: &[KnowledgeEdge],
    changed_name: &str,
    max_depth: usize,
) -> ExplosionRadius {
    let start_id = match find_node_by_name(nodes, changed_name) {
        Some(id) => id,
        None => {
            return ExplosionRadius {
                changed_entity: changed_name.to_string(),
                affected_entities: vec![],
                total_affected: 0,
                max_depth: 0,
                impact_score: 0.0,
                risk_level: RiskLevel::Low,
            };
        }
    };

    let id_to_node: HashMap<&str, &KnowledgeNode> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let edge_index = build_edge_index(edges);
    let id_to_name: HashMap<&str, &str> = nodes.iter().map(|n| (n.id.as_str(), n.title.as_str())).collect();

    let mut visited: HashSet<&str> = HashSet::new();
    let mut affected = Vec::new();
    let mut queue: VecDeque<(&str, usize, Vec<String>)> = VecDeque::new();
    visited.insert(start_id);
    queue.push_back((start_id, 0, vec![id_to_name.get(start_id).copied().unwrap_or("").to_string()]));

    while let Some((current, depth, path)) = queue.pop_front() {
        if depth > 0 {
            if let Some(node) = id_to_node.get(current) {
                let relation = if path.len() >= 2 {
                    let parent_name = &path[path.len() - 2];
                    edges.iter()
                        .find(|e| {
                            let source_is_parent = id_to_name.get(e.source_id.as_str()) == Some(&parent_name.as_str());
                            let target_is_current = id_to_name.get(e.target_id.as_str()) == Some(&node.title.as_str());
                            let source_is_current = id_to_name.get(e.source_id.as_str()) == Some(&node.title.as_str());
                            let target_is_parent = id_to_name.get(e.target_id.as_str()) == Some(&parent_name.as_str());
                            (source_is_parent && target_is_current) || (source_is_current && target_is_parent)
                        })
                        .map(|e| e.relation_type.as_str().to_string())
                        .unwrap_or_else(|| "depends_on".to_string())
                } else {
                    "depends_on".to_string()
                };

                affected.push(AffectedEntity {
                    name: node.title.clone(),
                    kind: kind_from_node_type(&node.node_type),
                    depth,
                    path_from_change: path.clone(),
                    relation,
                });
            }
        }

        if depth < max_depth {
            let neighbors: Vec<&str> = edge_index.get(current).into_iter()
                .flat_map(|v| v.iter())
                .map(|e| {
                    if e.source_id == current { e.target_id.as_str() } else { e.source_id.as_str() }
                })
                .collect();

            for next in neighbors {
                if visited.insert(next) {
                    let next_name = id_to_name.get(next).copied().unwrap_or("").to_string();
                    let mut next_path = path.clone();
                    next_path.push(next_name);
                    queue.push_back((next, depth + 1, next_path));
                }
            }
        }
    }

    let total_affected = affected.len();
    let max_actual_depth = affected.iter().map(|a| a.depth).max().unwrap_or(0);
    let critical_types = affected.iter().filter(|a| {
        matches!(a.kind, CodeEntityKind::Struct | CodeEntityKind::Trait | CodeEntityKind::Enum)
    }).count();

    let count_factor = (total_affected as f64 / 50.0).min(1.0);
    let depth_factor = (max_actual_depth as f64 / 5.0).min(1.0);
    let critical_factor = (critical_types as f64 / 10.0).min(1.0);
    let score = (count_factor * 0.4 + depth_factor * 0.3 + critical_factor * 0.3).min(1.0);

    let risk_level = if score >= 0.8 {
        RiskLevel::Critical
    } else if score >= 0.5 {
        RiskLevel::High
    } else if score >= 0.2 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };

    ExplosionRadius {
        changed_entity: changed_name.to_string(),
        affected_entities: affected,
        total_affected,
        max_depth: max_actual_depth,
        impact_score: score,
        risk_level,
    }
}

/// Find entities within an explosion radius that match a specific kind.
pub fn find_affected_entities_by_kind<'a>(
    radius: &'a ExplosionRadius,
    kind: CodeEntityKind,
) -> Vec<&'a AffectedEntity> {
    radius.affected_entities.iter().filter(|a| a.kind == kind).collect()
}

/// Find affected entities whose name contains a pattern (e.g., "test").
pub fn find_affected_by_name_pattern<'a>(
    radius: &'a ExplosionRadius,
    pattern: &str,
) -> Vec<&'a AffectedEntity> {
    let lower = pattern.to_lowercase();
    radius.affected_entities.iter()
        .filter(|a| a.name.to_lowercase().contains(&lower))
        .collect()
}

/// Produce a human-readable blast radius summary.
pub fn blast_radius_summary(radius: &ExplosionRadius) -> String {
    let test_count = radius.affected_entities.iter()
        .filter(|a| a.name.to_lowercase().contains("test")).count();
    let critical_count = radius.affected_entities.iter()
        .filter(|a| matches!(a.kind, CodeEntityKind::Struct | CodeEntityKind::Trait | CodeEntityKind::Enum))
        .count();

    format!(
        "Blast Radius for '{}': {} affected entities across {} levels [{} risk, score={:.2}]. \
         {} critical types (structs/traits/enums), {} test-related.",
        radius.changed_entity, radius.total_affected, radius.max_depth,
        radius.risk_level.as_str(), radius.impact_score,
        critical_count, test_count,
    )
}

fn kind_from_node_type(nt: &NodeType) -> CodeEntityKind {
    match nt {
        NodeType::CodeSnippet => CodeEntityKind::Function,
        NodeType::Concept => CodeEntityKind::Module,
        NodeType::Tool | NodeType::Framework => CodeEntityKind::Struct,
        NodeType::Event | NodeType::EvolutionPattern => CodeEntityKind::Attribute,
        NodeType::Book | NodeType::Article | NodeType::Textbook | NodeType::Guide => CodeEntityKind::File,
        _ => CodeEntityKind::File,
    }
}

/// Find the shortest dependency path between two named entities.
pub fn find_code_path(
    nodes: &[KnowledgeNode],
    edges: &[KnowledgeEdge],
    from_name: &str,
    to_name: &str,
    max_depth: usize,
) -> Option<DependencyChain> {
    if from_name == to_name {
        return None;
    }
    let start = find_node_by_name(nodes, from_name)?;
    let target = find_node_by_name(nodes, to_name)?;

    let id_to_name: HashMap<&str, &str> = nodes.iter().map(|n| (n.id.as_str(), n.title.as_str())).collect();
    let edge_index = build_edge_index(edges);

    let mut dist: HashMap<&str, f64> = HashMap::new();
    let mut prev: HashMap<&str, (&str, &str)> = HashMap::new();
    let mut queue: VecDeque<&str> = VecDeque::new();

    dist.insert(start, 0.0);
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        let d = *dist.get(current)?;
        if d as usize >= max_depth {
            continue;
        }
        let neighbors: Vec<(&str, &str, f64)> = edge_index.get(current).into_iter()
            .flat_map(|v| v.iter())
            .map(|e| {
                let neighbor = if e.source_id == current { e.target_id.as_str() } else { e.source_id.as_str() };
                (neighbor, e.id.as_str(), e.weight)
            })
            .collect();

        for (next_id, edge_id, w) in neighbors {
            let new_d = d + w;
            if !dist.contains_key(next_id) || new_d < dist[next_id] {
                dist.insert(next_id, new_d);
                prev.insert(next_id, (current, edge_id));
                queue.push_back(next_id);
            }
        }
    }

    if !prev.contains_key(target) {
        return None;
    }

    let mut hops = vec![];
    let mut cur = target;
    while let Some(&(p, eid)) = prev.get(cur) {
        let rel = edges.iter().find(|e| e.id == eid)
            .map(|e| e.relation_type.as_str().to_string())
            .unwrap_or_default();
        let w = edges.iter().find(|e| e.id == eid)
            .map(|e| e.weight)
            .unwrap_or(1.0);
        hops.push(Hop {
            source_name: id_to_name.get(p).unwrap_or(&"").to_string(),
            target_name: id_to_name.get(cur).unwrap_or(&"").to_string(),
            relation: rel,
            weight: w,
        });
        cur = p;
        if cur == start {
            break;
        }
    }
    hops.reverse();

    let total_weight = dist.get(target).copied().unwrap_or(0.0);

    Some(DependencyChain {
        from: entity_from_node(nodes.iter().find(|n| n.id == start)?),
        to: entity_from_node(nodes.iter().find(|n| n.id == target)?),
        hops,
        total_weight,
    })
}

/// Find all nodes reachable from a starting node within depth limit.
pub fn reachable_subgraph(
    nodes: &[KnowledgeNode],
    edges: &[KnowledgeEdge],
    start_name: &str,
    max_depth: usize,
) -> Vec<CodeEntity> {
    let start = match find_node_by_name(nodes, start_name) {
        Some(id) => id,
        None => return vec![],
    };
    let edge_index = build_edge_index(edges);
    let id_to_node: HashMap<&str, &KnowledgeNode> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    let mut visited: HashSet<&str> = HashSet::new();
    let mut result = vec![];
    let mut queue: VecDeque<(&str, usize)> = VecDeque::new();
    visited.insert(start);
    queue.push_back((start, 0));

    while let Some((current, depth)) = queue.pop_front() {
        if let Some(n) = id_to_node.get(current) {
            result.push(entity_from_node(n));
        }
        if depth < max_depth {
            let neighbors: Vec<&str> = edge_index.get(current).into_iter()
                .flat_map(|v| v.iter())
                .map(|e| if e.source_id == current { e.target_id.as_str() } else { e.source_id.as_str() })
                .collect();
            for next in neighbors {
                if visited.insert(next) {
                    queue.push_back((next, depth + 1));
                }
            }
        }
    }
    result
}

/// Compute graph-level statistics over the KB code graph.
pub fn code_graph_stats(nodes: &[KnowledgeNode], edges: &[KnowledgeEdge]) -> CodeGraphStats {
    let mut entity_counts: HashMap<String, usize> = HashMap::new();
    let mut relation_counts: HashMap<String, usize> = HashMap::new();
    let mut adjacency: HashMap<&str, usize> = HashMap::new();

    for node in nodes {
        let kind = node.node_type.as_str().to_string();
        *entity_counts.entry(kind).or_insert(0) += 1;
    }
    for edge in edges {
        let rel = edge.relation_type.as_str().to_string();
        *relation_counts.entry(rel).or_insert(0) += 1;
        *adjacency.entry(edge.source_id.as_str()).or_insert(0) += 1;
        *adjacency.entry(edge.target_id.as_str()).or_insert(0) += 1;
    }

    let mut hub: Vec<(String, usize)> = adjacency.iter()
        .map(|(k, v)| ((*k).to_string(), *v))
        .collect();
    hub.sort_by(|a, b| b.1.cmp(&a.1));

    let node_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    let connected: HashSet<&str> = adjacency.keys().copied().collect();
    let isolated: Vec<String> = node_ids.difference(&connected)
        .map(|s| s.to_string())
        .collect();

    CodeGraphStats {
        total_nodes: nodes.len(),
        total_edges: edges.len(),
        entity_counts,
        relation_counts,
        hub_nodes: hub.into_iter().take(10).collect(),
        isolated_nodes: isolated,
    }
}

fn build_edge_index<'a>(edges: &'a [KnowledgeEdge]) -> HashMap<&'a str, Vec<&'a KnowledgeEdge>> {
    let mut idx: HashMap<&str, Vec<&KnowledgeEdge>> = HashMap::new();
    for edge in edges {
        idx.entry(edge.source_id.as_str()).or_default().push(edge);
        idx.entry(edge.target_id.as_str()).or_default().push(edge);
    }
    idx
}

fn find_node_by_name<'a>(nodes: &'a [KnowledgeNode], name: &str) -> Option<&'a str> {
    let lower = name.to_lowercase();
    nodes.iter()
        .find(|n| n.title.to_lowercase() == lower || n.id.to_lowercase() == lower)
        .map(|n| n.id.as_str())
}

fn entity_from_node(node: &KnowledgeNode) -> CodeEntity {
    let kind = match node.node_type {
        NodeType::CodeSnippet => CodeEntityKind::Function,
        NodeType::Concept => CodeEntityKind::Module,
        NodeType::Tool | NodeType::Framework => CodeEntityKind::Struct,
        _ => CodeEntityKind::File,
    };
    CodeEntity {
        name: node.title.clone(),
        kind,
        file_path: node.url.clone(),
        line_range: None,
        doc_comment: node.summary.clone(),
        node_id: Some(node.id.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: &str, title: &str, nt: NodeType) -> KnowledgeNode {
        KnowledgeNode {
            id: id.to_string(), node_type: nt, title: title.to_string(),
            summary: None, content: None, url: None, domain: None,
            language: "rust".into(), confidence: 1.0, importance: 0.5,
            created_at: 0, updated_at: 0, access_count: 0,
            metadata: None, temporal: None, supersedes: None,
            source_episode: None,
        }
    }

    fn make_edge(id: &str, src: &str, tgt: &str, rt: RelationType, w: f64) -> KnowledgeEdge {
        KnowledgeEdge {
            id: id.to_string(), source_id: src.to_string(),
            target_id: tgt.to_string(), relation_type: rt,
            weight: w, description: None, created_at: 0, metadata: None,
        }
    }

    // ── Existing tests ──

    #[test]
    fn test_find_code_path_direct() {
        let nodes = vec![
            make_node("a1", "AuthService", NodeType::CodeSnippet),
            make_node("b1", "DatabasePool", NodeType::CodeSnippet),
        ];
        let edges = vec![
            make_edge("e1", "a1", "b1", RelationType::DependsOn, 1.0),
        ];
        let result = find_code_path(&nodes, &edges, "AuthService", "DatabasePool", 5);
        assert!(result.is_some());
        let chain = result.unwrap();
        assert_eq!(chain.hops.len(), 1);
        assert_eq!(chain.from.name, "AuthService");
        assert_eq!(chain.to.name, "DatabasePool");
    }

    #[test]
    fn test_find_code_path_multi_hop() {
        let nodes = vec![
            make_node("a", "Frontend", NodeType::CodeSnippet),
            make_node("b", "ApiGateway", NodeType::CodeSnippet),
            make_node("c", "UserService", NodeType::CodeSnippet),
            make_node("d", "PostgresDB", NodeType::CodeSnippet),
        ];
        let edges = vec![
            make_edge("e1", "a", "b", RelationType::DependsOn, 1.0),
            make_edge("e2", "b", "c", RelationType::DependsOn, 0.8),
            make_edge("e3", "c", "d", RelationType::Uses, 1.2),
        ];
        let result = find_code_path(&nodes, &edges, "Frontend", "PostgresDB", 5);
        assert!(result.is_some());
        let chain = result.unwrap();
        assert_eq!(chain.hops.len(), 3);
        assert_eq!(chain.total_weight, 3.0);
    }

    #[test]
    fn test_reachable_subgraph() {
        let nodes = vec![
            make_node("n1", "Core", NodeType::Concept),
            make_node("n2", "PluginA", NodeType::Tool),
            make_node("n3", "PluginB", NodeType::Tool),
            make_node("n4", "PluginC", NodeType::Tool),
        ];
        let edges = vec![
            make_edge("e1", "n1", "n2", RelationType::Contains, 1.0),
            make_edge("e2", "n1", "n3", RelationType::Contains, 1.0),
            make_edge("e3", "n3", "n4", RelationType::DependsOn, 0.5),
        ];
        let sub = reachable_subgraph(&nodes, &edges, "Core", 2);
        assert_eq!(sub.len(), 4);
    }

    #[test]
    fn test_no_path() {
        let nodes = vec![
            make_node("x", "X", NodeType::CodeSnippet),
            make_node("y", "Y", NodeType::CodeSnippet),
        ];
        let edges = vec![];
        let result = find_code_path(&nodes, &edges, "X", "Y", 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_stats() {
        let nodes = vec![
            make_node("a", "A", NodeType::CodeSnippet),
            make_node("b", "B", NodeType::CodeSnippet),
            make_node("c", "C", NodeType::Concept),
        ];
        let edges = vec![
            make_edge("e1", "a", "b", RelationType::DependsOn, 1.0),
            make_edge("e2", "b", "c", RelationType::AboutTopic, 1.0),
        ];
        let stats = code_graph_stats(&nodes, &edges);
        assert_eq!(stats.total_nodes, 3);
        assert_eq!(stats.total_edges, 2);
        assert!(!stats.hub_nodes.is_empty());
    }

    #[test]
    fn test_isolated_nodes() {
        let nodes = vec![
            make_node("a", "Connected", NodeType::CodeSnippet),
            make_node("b", "Alone", NodeType::CodeSnippet),
        ];
        let edges = vec![];
        let stats = code_graph_stats(&nodes, &edges);
        assert_eq!(stats.isolated_nodes.len(), 2);
    }

    // ── Explosion Radius tests ──

    #[test]
    fn test_explosion_radius_unknown_entity() {
        let nodes = vec![make_node("a", "Known", NodeType::CodeSnippet)];
        let edges = vec![];
        let radius = explosion_radius(&nodes, &edges, "Unknown", 3);
        assert_eq!(radius.total_affected, 0);
        assert_eq!(radius.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_explosion_radius_direct_dependents() {
        let nodes = vec![
            make_node("core", "CoreLib", NodeType::Concept),
            make_node("p1", "PluginA", NodeType::Tool),
            make_node("p2", "PluginB", NodeType::Tool),
        ];
        let edges = vec![
            make_edge("e1", "p1", "core", RelationType::DependsOn, 1.0),
            make_edge("e2", "p2", "core", RelationType::DependsOn, 1.0),
        ];
        let radius = explosion_radius(&nodes, &edges, "CoreLib", 3);
        assert_eq!(radius.total_affected, 2);
        assert!(radius.impact_score > 0.0);
    }

    #[test]
    fn test_explosion_radius_chain() {
        let nodes = vec![
            make_node("a", "A", NodeType::CodeSnippet),
            make_node("b", "B", NodeType::CodeSnippet),
            make_node("c", "C", NodeType::CodeSnippet),
            make_node("d", "D", NodeType::CodeSnippet),
        ];
        let edges = vec![
            make_edge("e1", "a", "b", RelationType::DependsOn, 1.0),
            make_edge("e2", "b", "c", RelationType::DependsOn, 1.0),
            make_edge("e3", "c", "d", RelationType::DependsOn, 1.0),
        ];
        let radius = explosion_radius(&nodes, &edges, "A", 2);
        assert_eq!(radius.total_affected, 2); // B, C at depth 2
        assert_eq!(radius.max_depth, 2);
    }

    #[test]
    fn test_explosion_radius_depth_limit() {
        let nodes = vec![
            make_node("a", "A", NodeType::CodeSnippet),
            make_node("b", "B", NodeType::CodeSnippet),
            make_node("c", "C", NodeType::CodeSnippet),
        ];
        let edges = vec![
            make_edge("e1", "a", "b", RelationType::DependsOn, 1.0),
            make_edge("e2", "b", "c", RelationType::DependsOn, 1.0),
        ];
        let radius = explosion_radius(&nodes, &edges, "A", 1);
        assert_eq!(radius.total_affected, 1); // only B
        assert_eq!(radius.max_depth, 1);
    }

    #[test]
    fn test_risk_level_escalation() {
        let mut nodes = vec![make_node("core", "CoreStruct", NodeType::CodeSnippet)];
        let mut edges = vec![];
        // Add many dependents to push score up
        for i in 0..30 {
            let nid = format!("p{}", i);
            let ntitle = format!("Plugin{}", i);
            nodes.push(make_node(&nid, &ntitle, NodeType::Tool));
            edges.push(make_edge(&format!("e{}", i), &nid, "core", RelationType::DependsOn, 1.0));
        }
        let radius = explosion_radius(&nodes, &edges, "CoreStruct", 3);
        assert!(radius.total_affected >= 30);
        assert!(radius.impact_score > 0.2);
    }

    #[test]
    fn test_find_affected_by_kind() {
        let nodes = vec![
            make_node("core", "CoreLib", NodeType::Concept),
            make_node("t1", "TestHelper", NodeType::Event),
            make_node("p1", "PluginA", NodeType::Tool),
        ];
        let edges = vec![
            make_edge("e1", "t1", "core", RelationType::DependsOn, 1.0),
            make_edge("e2", "p1", "core", RelationType::DependsOn, 1.0),
        ];
        let radius = explosion_radius(&nodes, &edges, "CoreLib", 3);
        let tools = find_affected_entities_by_kind(&radius, CodeEntityKind::Struct);
        assert!(!tools.is_empty());
        let tests = find_affected_entities_by_kind(&radius, CodeEntityKind::Attribute);
        assert!(!tests.is_empty());
    }

    #[test]
    fn test_find_affected_by_name_pattern() {
        let nodes = vec![
            make_node("core", "CoreLib", NodeType::Concept),
            make_node("t1", "test_core", NodeType::Event),
            make_node("t2", "core_test", NodeType::Event),
            make_node("p1", "PluginA", NodeType::Tool),
        ];
        let edges = vec![
            make_edge("e1", "t1", "core", RelationType::DependsOn, 1.0),
            make_edge("e2", "t2", "core", RelationType::DependsOn, 1.0),
            make_edge("e3", "p1", "core", RelationType::DependsOn, 1.0),
        ];
        let radius = explosion_radius(&nodes, &edges, "CoreLib", 3);
        let test_affected = find_affected_by_name_pattern(&radius, "test");
        assert_eq!(test_affected.len(), 2);
    }

    #[test]
    fn test_blast_radius_summary_format() {
        let nodes = vec![
            make_node("core", "CoreLib", NodeType::Concept),
            make_node("p1", "PluginA", NodeType::Tool),
        ];
        let edges = vec![
            make_edge("e1", "p1", "core", RelationType::DependsOn, 1.0),
        ];
        let radius = explosion_radius(&nodes, &edges, "CoreLib", 3);
        let summary = blast_radius_summary(&radius);
        assert!(summary.contains("CoreLib"));
        assert!(summary.contains("affected"));
        assert!(summary.contains("score"));
    }
}
