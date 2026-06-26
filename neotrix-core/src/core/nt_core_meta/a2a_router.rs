use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct A2ARouter {
    pub agents: HashMap<String, String>,
    pub routes: HashMap<String, Vec<String>>,
}

impl A2ARouter {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            routes: HashMap::new(),
        }
    }
    pub fn register_agent(&mut self, name: &str, endpoint: &str) {
        self.agents.insert(name.into(), endpoint.into());
    }
    pub fn route(&mut self, from: &str, to: &str) {
        self.routes.entry(from.into()).or_default().push(to.into());
    }
    pub fn resolve(&self, name: &str) -> Option<&String> {
        self.agents.get(name)
    }
}
