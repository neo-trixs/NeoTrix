use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub handler: String,
    pub hot_reloadable: bool,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct SkillRegistry {
    pub skills: HashMap<String, SkillDefinition>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, description: &str, handler: &str, hot_reloadable: bool) {
        self.skills.insert(
            name.to_string(),
            SkillDefinition {
                name: name.to_string(),
                description: description.to_string(),
                handler: handler.to_string(),
                hot_reloadable,
                version: 1,
            },
        );
    }

    pub fn get(&self, name: &str) -> Option<&SkillDefinition> {
        self.skills.get(name)
    }

    pub fn dispatch(&self, name: &str) -> Option<&str> {
        self.skills.get(name).map(|s| s.handler.as_str())
    }

    pub fn unregister(&mut self, name: &str) {
        self.skills.remove(name);
    }

    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.skills.keys().map(|k| k.as_str()).collect();
        names.sort();
        names
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
