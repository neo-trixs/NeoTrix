use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SkillTier {
    Builtin,
    Verified,
    Community,
    Custom,
    Ephemeral,
    Crystallized,
}

#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub tier: SkillTier,
    pub version: String,
    pub entry_point: String,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SkillRegistry {
    skills: HashMap<String, SkillDefinition>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        SkillRegistry {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: SkillDefinition) -> Result<(), String> {
        let id = skill.id.clone();
        if self.skills.contains_key(&id) {
            return Err(format!("Skill '{}' already registered", id));
        }
        self.skills.insert(id, skill);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&SkillDefinition> {
        self.skills.get(id)
    }

    pub fn list_by_tier(&self, tier: SkillTier) -> Vec<&SkillDefinition> {
        self.skills.values().filter(|s| s.tier == tier).collect()
    }

    pub fn unregister(&mut self, id: &str) -> bool {
        self.skills.remove(id).is_some()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get() {
        let mut reg = SkillRegistry::new();
        let def = SkillDefinition {
            id: "test".into(),
            name: "Test Skill".into(),
            tier: SkillTier::Builtin,
            version: "1.0.0".into(),
            entry_point: "test.rs".into(),
            dependencies: vec![],
            metadata: HashMap::new(),
        };
        reg.register(def.clone()).expect("register should succeed");
        assert!(reg.get("test").is_some());
    }

    #[test]
    fn test_register_duplicate_fails() {
        let mut reg = SkillRegistry::new();
        let def = SkillDefinition {
            id: "dup".into(),
            name: "Dup".into(),
            tier: SkillTier::Verified,
            version: "1.0.0".into(),
            entry_point: "dup.rs".into(),
            dependencies: vec![],
            metadata: HashMap::new(),
        };
        reg.register(def.clone()).expect("register should succeed");
        assert!(reg.register(def).is_err());
    }

    #[test]
    fn test_list_by_tier() {
        let mut reg = SkillRegistry::new();
        reg.register(SkillDefinition {
            id: "a".into(), name: "A".into(), tier: SkillTier::Builtin,
            version: "1".into(), entry_point: "a.rs".into(),
            dependencies: vec![], metadata: HashMap::new(),
        }).expect("register a should succeed");
        reg.register(SkillDefinition {
            id: "b".into(), name: "B".into(), tier: SkillTier::Custom,
            version: "1".into(), entry_point: "b.rs".into(),
            dependencies: vec![], metadata: HashMap::new(),
        }).expect("register b should succeed");
        reg.register(SkillDefinition {
            id: "c".into(), name: "C".into(), tier: SkillTier::Builtin,
            version: "1".into(), entry_point: "c.rs".into(),
            dependencies: vec![], metadata: HashMap::new(),
        }).expect("register c should succeed");
        assert_eq!(reg.list_by_tier(SkillTier::Builtin).len(), 2);
        assert_eq!(reg.list_by_tier(SkillTier::Custom).len(), 1);
    }

    #[test]
    fn test_unregister() {
        let mut reg = SkillRegistry::new();
        reg.register(SkillDefinition {
            id: "x".into(), name: "X".into(), tier: SkillTier::Ephemeral,
            version: "1".into(), entry_point: "x.rs".into(),
            dependencies: vec![], metadata: HashMap::new(),
        }).expect("register x should succeed");
        assert!(reg.unregister("x"));
        assert!(!reg.unregister("x"));
    }
}
