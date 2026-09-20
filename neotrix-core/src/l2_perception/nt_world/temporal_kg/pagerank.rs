use std::collections::HashMap;

use super::knowledge_graph::KnowledgeGraph;

pub fn personalized_page_rank(
    graph: &KnowledgeGraph,
    seed_ids: &[String],
    damping: f64,
    iterations: usize,
) -> Vec<(String, f64)> {
    let n = graph.entities.len();
    if n == 0 || seed_ids.is_empty() {
        return Vec::new();
    }

    let id_to_idx: HashMap<&str, usize> = graph
        .entities
        .keys()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    let mut out_edges: Vec<Vec<usize>> = vec![Vec::new(); n];
    for r in &graph.relations {
        if let (Some(&src), Some(&dst)) = (
            id_to_idx.get(r.source_id.as_str()),
            id_to_idx.get(r.target_id.as_str()),
        ) {
            out_edges[src].push(dst);
        }
    }

    let mut scores = vec![1.0 / n as f64; n];

    let mut personalization = vec![0.0; n];
    let seed_weight = 1.0 / seed_ids.len() as f64;
    for qid in seed_ids {
        if let Some(&idx) = id_to_idx.get(qid.as_str()) {
            personalization[idx] = seed_weight;
        }
    }

    for _ in 0..iterations {
        let mut new_scores = vec![0.0; n];
        let mut dangling_sum = 0.0;

        for i in 0..n {
            if out_edges[i].is_empty() {
                dangling_sum += scores[i];
            } else {
                let share = scores[i] / out_edges[i].len() as f64;
                for &j in &out_edges[i] {
                    new_scores[j] += share;
                }
            }
        }

        let dangling_share = dangling_sum / n as f64;
        for i in 0..n {
            new_scores[i] =
                (1.0 - damping) * personalization[i] + damping * (new_scores[i] + dangling_share);
        }

        scores = new_scores;
    }

    let mut ranks: Vec<(String, f64)> = graph
        .entities
        .keys()
        .enumerate()
        .map(|(i, id)| (id.clone(), scores[i]))
        .collect();

    ranks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranks
}

#[cfg(test)]
mod tests {
    use super::super::entity::{Entity, EntityType};
    use super::super::relation::TemporalRelation;
    use super::*;
    use chrono::NaiveDateTime;

    fn dt(year: i32, month: u32, day: u32, hour: u32, min: u32) -> NaiveDateTime {
        NaiveDateTime::from_ymd_opt(year, month, day, hour, min, 0).unwrap()
    }

    fn make_chain_graph() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        let t0 = dt(2024, 1, 1, 0, 0);

        for i in 0..5 {
            kg.add_entity(Entity {
                id: format!("e{}", i),
                name: format!("E{}", i),
                entity_type: EntityType::Concept,
                properties: HashMap::new(),
            });
        }

        for i in 0..4 {
            kg.add_relation(TemporalRelation::new(
                format!("e{}", i),
                format!("e{}", i + 1),
                "connects",
                t0,
            ));
        }

        kg
    }

    #[test]
    fn test_ppr_single_seed() {
        let kg = make_chain_graph();
        let ranks = personalized_page_rank(&kg, &["e0".to_string()], 0.85, 20);
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0].0, "e0");
        assert!(ranks[0].1 > ranks[1].1);
    }

    #[test]
    fn test_ppr_empty_graph() {
        let kg = KnowledgeGraph::new();
        let ranks = personalized_page_rank(&kg, &["e0".to_string()], 0.85, 10);
        assert!(ranks.is_empty());
    }

    #[test]
    fn test_ppr_star_topology() {
        let mut kg = KnowledgeGraph::new();
        let t0 = dt(2024, 1, 1, 0, 0);

        kg.add_entity(Entity {
            id: "e0".into(),
            name: "Center".into(),
            entity_type: EntityType::Concept,
            properties: HashMap::new(),
        });
        for i in 1..=4 {
            kg.add_entity(Entity {
                id: format!("e{}", i),
                name: format!("Leaf{}", i),
                entity_type: EntityType::Concept,
                properties: HashMap::new(),
            });
            kg.add_relation(TemporalRelation::new(
                format!("e{}", i),
                "e0",
                "points_to",
                t0,
            ));
        }

        let ranks = personalized_page_rank(&kg, &["e0".to_string()], 0.85, 20);
        assert_eq!(ranks[0].0, "e0");
    }

    #[test]
    fn test_ppr_multiple_seeds() {
        let mut kg = KnowledgeGraph::new();
        let t0 = dt(2024, 1, 1, 0, 0);

        kg.add_entity(Entity {
            id: "a".into(),
            name: "A".into(),
            entity_type: EntityType::Concept,
            properties: HashMap::new(),
        });
        kg.add_entity(Entity {
            id: "b".into(),
            name: "B".into(),
            entity_type: EntityType::Concept,
            properties: HashMap::new(),
        });
        kg.add_entity(Entity {
            id: "c".into(),
            name: "C".into(),
            entity_type: EntityType::Concept,
            properties: HashMap::new(),
        });
        kg.add_relation(TemporalRelation::new("a", "b", "rel", t0));
        kg.add_relation(TemporalRelation::new("b", "c", "rel", t0));

        let ranks = personalized_page_rank(&kg, &["a".into(), "b".into()], 0.85, 20);
        assert_eq!(ranks.len(), 3);
        let seed_score: f64 = ranks
            .iter()
            .filter(|(id, _)| id == "a" || id == "b")
            .map(|(_, s)| s)
            .sum();
        let non_seed: f64 = ranks
            .iter()
            .filter(|(id, _)| id == "c")
            .map(|(_, s)| s)
            .sum();
        assert!(seed_score > non_seed);
    }
}
