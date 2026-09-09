use std::collections::HashMap;
use super::plugin_interface::MediaSourcePlugin;

pub struct PluginLoader {
    plugins: HashMap<String, Box<dyn MediaSourcePlugin>>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self { plugins: HashMap::new() }
    }
    pub fn register(&mut self, plugin: Box<dyn MediaSourcePlugin>) {
        self.plugins.insert(plugin.id().to_string(), plugin);
    }
    pub fn get(&self, id: &str) -> Option<&dyn MediaSourcePlugin> {
        self.plugins.get(id).map(|p| p.as_ref())
    }
    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }
}
