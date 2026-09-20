use chrono::NaiveDateTime;

use super::entity::Entity;
use super::knowledge_graph::KnowledgeGraph;
use super::relation::TemporalRelation;

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub entities: Vec<Entity>,
    pub relations: Vec<TemporalRelation>,
}

pub struct TemporalQueryBuilder<'a> {
    graph: &'a KnowledgeGraph,
    time: Option<NaiveDateTime>,
    entity_filter: Option<String>,
    relation_type_filter: Option<String>,
}

impl<'a> TemporalQueryBuilder<'a> {
    pub fn for_graph(graph: &'a KnowledgeGraph) -> Self {
        Self {
            graph,
            time: None,
            entity_filter: None,
            relation_type_filter: None,
        }
    }

    pub fn at(mut self, t: NaiveDateTime) -> Self {
        self.time = Some(t);
        self
    }

    pub fn with_entity(mut self, entity_id: impl Into<String>) -> Self {
        self.entity_filter = Some(entity_id.into());
        self
    }

    pub fn with_relation_type(mut self, rel_type: impl Into<String>) -> Self {
        self.relation_type_filter = Some(rel_type.into());
        self
    }

    pub fn execute(&self) -> QueryResult {
        let relations: Vec<TemporalRelation> = self
            .graph
            .relations
            .iter()
            .filter(|r| {
                if let Some(t) = self.time {
                    if !r.is_valid_at(t) {
                        return false;
                    }
                }

                if let Some(ref eid) = self.entity_filter {
                    if r.source_id != *eid && r.target_id != *eid {
                        return false;
                    }
                }

                if let Some(ref rt) = self.relation_type_filter {
                    if r.relation_type != *rt {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        let mut involved_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for rel in &relations {
            involved_ids.insert(rel.source_id.clone());
            involved_ids.insert(rel.target_id.clone());
        }

        let entities: Vec<Entity> = self
            .graph
            .entities
            .iter()
            .filter(|(id, _)| involved_ids.contains(*id))
            .map(|(_, e)| e.clone())
            .collect();

        QueryResult {
            entities,
            relations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::entity::{Entity, EntityType};
    use super::*;
    use std::collections::HashMap;

    fn dt(year: i32, month: u32, day: u32) -> NaiveDateTime {
        NaiveDateTime::from_ymd_opt(year, month, day, 0, 0, 0).unwrap()
    }

    fn make_graph() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        let t0 = dt(2024, 1, 1);
        let t1 = dt(2024, 6, 1);

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
            name: "Bob".into(),
            entity_type: EntityType::Person,
            properties: HashMap::new(),
        });

        kg.add_relation(TemporalRelation::new("e1", "e2", "works_at", t0).with_valid_to(t1));
        kg.add_relation(TemporalRelation::new("e1", "e3", "knows", t0));
        kg.add_relation(TemporalRelation::new("e3", "e2", "works_at", t1));

        kg
    }

    #[test]
    fn test_query_all() {
        let kg = make_graph();
        let result = TemporalQueryBuilder::for_graph(&kg).execute();
        assert_eq!(result.relations.len(), 3);
        assert_eq!(result.entities.len(), 3);
    }

    #[test]
    fn test_query_at_time_filters_expired() {
        let kg = make_graph();
        let result = TemporalQueryBuilder::for_graph(&kg)
            .at(dt(2024, 7, 1))
            .execute();
        assert_eq!(result.relations.len(), 2);
        let types: Vec<&str> = result
            .relations
            .iter()
            .map(|r| r.relation_type.as_str())
            .collect();
        assert!(types.contains(&"knows"));
        assert!(types.contains(&"works_at"));
    }

    #[test]
    fn test_query_with_entity() {
        let kg = make_graph();
        let result = TemporalQueryBuilder::for_graph(&kg)
            .with_entity("e1")
            .execute();
        assert_eq!(result.relations.len(), 2);
    }

    #[test]
    fn test_query_with_relation_type() {
        let kg = make_graph();
        let result = TemporalQueryBuilder::for_graph(&kg)
            .with_relation_type("works_at")
            .execute();
        assert_eq!(result.relations.len(), 2);
    }

    #[test]
    fn test_query_combined_filters() {
        let kg = make_graph();
        let result = TemporalQueryBuilder::for_graph(&kg)
            .at(dt(2024, 7, 1))
            .with_entity("e1")
            .with_relation_type("knows")
            .execute();
        assert_eq!(result.relations.len(), 1);
        assert_eq!(result.relations[0].relation_type, "knows");
    }

    #[test]
    fn test_query_empty_graph() {
        let kg = KnowledgeGraph::new();
        let result = TemporalQueryBuilder::for_graph(&kg)
            .at(dt(2024, 1, 1))
            .execute();
        assert!(result.relations.is_empty());
        assert!(result.entities.is_empty());
    }

    #[test]
    fn test_query_entities_match_relations() {
        let kg = make_graph();
        let result = TemporalQueryBuilder::for_graph(&kg)
            .at(dt(2024, 3, 1))
            .with_relation_type("works_at")
            .execute();
        let entity_ids: Vec<&str> = result.entities.iter().map(|e| e.id.as_str()).collect();
        assert!(entity_ids.contains(&"e1"));
        assert!(entity_ids.contains(&"e2"));
        assert!(!entity_ids.contains(&"e3"));
    }
}
