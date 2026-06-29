use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub steps: Vec<String>,
    pub capability_snapshot: Vec<f64>,
    pub reuse_count: u32,
    pub total_reward: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
}

impl Skill {
    pub fn new(
        name: String,
        description: String,
        tags: Vec<String>,
        steps: Vec<String>,
        capability_snapshot: Vec<f64>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: format!("skill-{}", uuid::Uuid::new_v4()),
            name,
            description,
            tags,
            steps,
            capability_snapshot,
            reuse_count: 0,
            total_reward: 0.0,
            created_at: now,
            last_used_at: now,
        }
    }

    pub fn record_reuse(&mut self, reward: f64) {
        self.reuse_count += 1;
        self.total_reward += reward;
        self.last_used_at = chrono::Utc::now();
    }

    pub fn average_reward(&self) -> f64 {
        if self.reuse_count == 0 {
            0.0
        } else {
            self.total_reward / self.reuse_count as f64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.id.clone(), skill);
    }

    pub fn get(&self, id: &str) -> Option<&Skill> {
        self.skills.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Skill> {
        self.skills.get_mut(id)
    }

    pub fn all(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    pub fn find_by_tags(&self, tags: &[String]) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| tags.iter().all(|t| s.tags.contains(t)))
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Skill> {
        let q = query.to_lowercase();
        self.skills
            .values()
            .filter(|s| {
                s.name.to_lowercase().contains(&q) || s.description.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_new_and_record_reuse() {
        let mut skill = Skill::new(
            "test".into(),
            "a test skill".into(),
            vec!["tag1".into()],
            vec!["step1".into()],
            vec![0.5, 0.8],
        );
        assert_eq!(skill.name, "test");
        assert_eq!(skill.reuse_count, 0);
        skill.record_reuse(1.0);
        skill.record_reuse(3.0);
        assert_eq!(skill.reuse_count, 2);
        assert!((skill.average_reward() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_skill_registry_search() {
        let mut reg = SkillRegistry::new();
        reg.register(Skill::new(
            "write".into(),
            "writing skills".into(),
            vec!["code".into(), "doc".into()],
            vec![],
            vec![],
        ));
        reg.register(Skill::new(
            "read".into(),
            "reading skills".into(),
            vec!["doc".into()],
            vec![],
            vec![],
        ));
        assert_eq!(reg.len(), 2);
        assert!(!reg.is_empty());
        let results = reg.search("write");
        assert_eq!(results.len(), 1);
        let found = reg.find_by_tags(&["doc".into()]);
        assert_eq!(found.len(), 2);
    }
}
