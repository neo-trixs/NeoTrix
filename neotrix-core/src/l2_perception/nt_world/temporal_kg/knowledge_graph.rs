use std::collections::HashMap;

use chrono::NaiveDateTime;

use super::entity::Entity;
use super::relation::TemporalRelation;

#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    pub entities: HashMap<String, Entity>,
    pub relations: Vec<TemporalRelation>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            relations: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id.clone(), entity);
    }

    pub fn add_relation(&mut self, relation: TemporalRelation) {
        self.relations.push(relation);
    }

    pub fn query_entity(&self, id: &str) -> Option<&Entity> {
        self.entities.get(id)
    }

    pub fn query_at(&self, t: NaiveDateTime) -> Vec<&TemporalRelation> {
        self.relations.iter().filter(|r| r.is_valid_at(t)).collect()
    }

    pub fn invalidate_relation(&mut self, idx: usize, at: NaiveDateTime) {
        if let Some(r) = self.relations.get_mut(idx) {
            r.valid_to = Some(at);
        }
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::entity::EntityType;
    use super::*;

    fn dt(year: i32, month: u32, day: u32, hour: u32, min: u32) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(year,month,day)
            .and_then(|d| d.and_hms_opt(hour,min,0))
            .expect("测试时间戳构造：日期/时间应合法")
    }

    fn make_graph() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        let t0 = dt(2024, 1, 1, 0, 0);

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

        kg.add_relation(TemporalRelation::new("e1", "e2", "works_at", t0));
        kg.add_relation(TemporalRelation::new("e1", "e3", "knows", t0));

        kg
    }

    #[test]
    fn test_add_entity_and_query() {
        let kg = make_graph();
        assert_eq!(kg.entities.len(), 3);
        assert!(kg.query_entity("e1").is_some());
        assert!(kg.query_entity("nonexistent").is_none());
    }

    #[test]
    fn test_add_relation_and_query_at() {
        let kg = make_graph();
        let t = dt(2024, 6, 15, 12, 0);
        let rels = kg.query_at(t);
        assert_eq!(rels.len(), 2);
    }

    #[test]
    fn test_invalidate_relation() {
        let mut kg = make_graph();
        let t0 = dt(2024, 1, 1, 0, 0);
        let t_after = dt(2024, 6, 15, 12, 0);

        kg.invalidate_relation(0, t_after);
        let rels = kg.query_at(dt(2024, 7, 1, 0, 0));
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].relation_type, "knows");
    }

    #[test]
    fn test_invalidate_relation_out_of_bounds() {
        let mut kg = make_graph();
        kg.invalidate_relation(99, dt(2024, 1, 1, 0, 0));
        assert_eq!(kg.relations.len(), 2);
    }

    #[test]
    fn test_query_at_expired() {
        let mut kg = KnowledgeGraph::new();
        let t0 = dt(2024, 1, 1, 0, 0);
        let t1 = dt(2024, 3, 1, 0, 0);

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
        kg.add_relation(TemporalRelation::new("a", "b", "rel", t0).with_valid_to(t1));

        assert_eq!(kg.query_at(t0).len(), 1);
        assert_eq!(kg.query_at(t1).len(), 1);
        assert_eq!(kg.query_at(dt(2024, 4, 1, 0, 0)).len(), 0);
    }

    #[test]
    fn test_default() {
        let kg = KnowledgeGraph::default();
        assert!(kg.entities.is_empty());
        assert!(kg.relations.is_empty());
    }
}
