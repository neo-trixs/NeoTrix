use std::collections::{HashMap, VecDeque};

use super::knowledge_graph::KnowledgeGraph;

pub struct GraphTraversal;

impl GraphTraversal {
    pub fn bfs(graph: &KnowledgeGraph, start_id: &str, max_depth: usize) -> Vec<(String, f64)> {
        if !graph.entities.contains_key(start_id) {
            return Vec::new();
        }

        let mut visited: HashMap<String, f64> = HashMap::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();

        visited.insert(start_id.to_string(), 1.0);
        queue.push_back((start_id.to_string(), 0));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            for rel in &graph.relations {
                let neighbor = if rel.source_id == current {
                    Some(rel.target_id.clone())
                } else if rel.target_id == current {
                    Some(rel.source_id.clone())
                } else {
                    None
                };

                if let Some(nid) = neighbor {
                    if !visited.contains_key(&nid) {
                        let score = 1.0 / (1.0 + depth as f64);
                        visited.insert(nid.clone(), score);
                        queue.push_back((nid, depth + 1));
                    }
                }
            }
        }

        let mut result: Vec<(String, f64)> = visited.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn shortest_path(graph: &KnowledgeGraph, from: &str, to: &str) -> Option<Vec<String>> {
        if from == to {
            return Some(vec![from.to_string()]);
        }
        if !graph.entities.contains_key(from) || !graph.entities.contains_key(to) {
            return None;
        }

        let mut visited: HashMap<String, String> = HashMap::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();

        visited.insert(from.to_string(), String::new());
        queue.push_back((from.to_string(), 0));

        while let Some((current, depth)) = queue.pop_front() {
            for rel in &graph.relations {
                let neighbor = if rel.source_id == current {
                    Some(rel.target_id.clone())
                } else if rel.target_id == current {
                    Some(rel.source_id.clone())
                } else {
                    None
                };

                if let Some(nid) = neighbor {
                    if !visited.contains_key(&nid) {
                        visited.insert(nid.clone(), current.clone());

                        if nid == to {
                            return Some(reconstruct_path(&visited, from, to));
                        }

                        queue.push_back((nid, depth + 1));
                    }
                }
            }
        }

        None
    }

    pub fn neighborhood(graph: &KnowledgeGraph, center_id: &str, radius: usize) -> Vec<String> {
        Self::bfs(graph, center_id, radius)
            .into_iter()
            .map(|(id, _score)| id)
            .collect()
    }
}

fn reconstruct_path(visited: &HashMap<String, String>, from: &str, to: &str) -> Vec<String> {
    let mut path = vec![to.to_string()];
    let mut current = to;

    while let Some(parent) = visited.get(current) {
        if parent.is_empty() {
            break;
        }
        path.push(parent.clone());
        current = parent;
    }

    path.reverse();

    if path.first().map(|s| s.as_str()) == Some(from) {
        path
    } else {
        vec![from.to_string(), to.to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::super::entity::{Entity, EntityType};
    use super::super::relation::TemporalRelation;
    use super::*;
    use chrono::NaiveDateTime;
    use std::collections::HashMap;

    fn dt(year: i32, month: u32, day: u32) -> NaiveDateTime {
        NaiveDateTime::from_ymd_opt(year, month, day, 0, 0, 0).unwrap()
    }

    fn make_graph() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        let t0 = dt(2024, 1, 1);

        // e1 --works_at--> e2 --located_in--> e3 --known_by--> e1
        kg.add_entity(Entity {
            id: "e1".into(),
            name: "Alice".into(),
            entity_type: EntityType::Person,
            properties: HashMap::new(),
        });
        kg.add_entity(Entity {
            id: "e2".into(),
            name: "Google".into(),
            entity_type: EntityType::Organization,
            properties: HashMap::new(),
        });
        kg.add_entity(Entity {
            id: "e3".into(),
            name: "MountainView".into(),
            entity_type: EntityType::Place,
            properties: HashMap::new(),
        });
        kg.add_entity(Entity {
            id: "e4".into(),
            name: "Bob".into(),
            entity_type: EntityType::Person,
            properties: HashMap::new(),
        });

        kg.add_relation(TemporalRelation::new("e1", "e2", "works_at", t0));
        kg.add_relation(TemporalRelation::new("e2", "e3", "located_in", t0));
        kg.add_relation(TemporalRelation::new("e3", "e1", "known_by", t0));
        kg.add_relation(TemporalRelation::new("e4", "e2", "works_at", t0));

        kg
    }

    #[test]
    fn test_bfs_from_alice() {
        let kg = make_graph();
        let result = GraphTraversal::bfs(&kg, "e1", 3);
        let ids: Vec<&str> = result.iter().map(|(id, _)| id.as_str()).collect();
        assert!(ids.contains(&"e1"));
        assert!(ids.contains(&"e2"));
        assert!(ids.contains(&"e3"));
        assert!(ids.contains(&"e4"));
    }

    #[test]
    fn test_bfs_depth_limit() {
        let kg = make_graph();
        let result = GraphTraversal::bfs(&kg, "e1", 1);
        let ids: Vec<&str> = result.iter().map(|(id, _)| id.as_str()).collect();
        assert!(ids.contains(&"e1"));
        assert!(ids.contains(&"e2"));
    }

    #[test]
    fn test_bfs_nonexistent_start() {
        let kg = make_graph();
        let result = GraphTraversal::bfs(&kg, "nonexistent", 3);
        assert!(result.is_empty());
    }

    #[test]
    fn test_shortest_path_alice_to_mountainview() {
        let kg = make_graph();
        let path = GraphTraversal::shortest_path(&kg, "e1", "e3");
        assert!(path.is_some());
        let p = path.unwrap();
        assert_eq!(p.first().unwrap(), "e1");
        assert_eq!(p.last().unwrap(), "e3");
    }

    #[test]
    fn test_shortest_path_same_node() {
        let kg = make_graph();
        let path = GraphTraversal::shortest_path(&kg, "e1", "e1");
        assert_eq!(path, Some(vec!["e1".to_string()]));
    }

    #[test]
    fn test_shortest_path_nonexistent() {
        let kg = make_graph();
        assert!(GraphTraversal::shortest_path(&kg, "e1", "nonexistent").is_none());
    }

    #[test]
    fn test_neighborhood() {
        let kg = make_graph();
        let hood = GraphTraversal::neighborhood(&kg, "e2", 1);
        assert!(hood.contains(&"e2".to_string()));
        assert!(hood.contains(&"e1".to_string()));
        assert!(hood.contains(&"e3".to_string()));
        assert!(hood.contains(&"e4".to_string()));
    }

    #[test]
    fn test_neighborhood_radius_zero() {
        let kg = make_graph();
        let hood = GraphTraversal::neighborhood(&kg, "e1", 0);
        assert_eq!(hood.len(), 1);
        assert_eq!(hood[0], "e1");
    }
}
