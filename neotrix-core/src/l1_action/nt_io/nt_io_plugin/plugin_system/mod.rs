#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub hooks: Vec<String>,
    pub enabled: bool,
}

pub struct PluginSystem {
    plugins: Vec<Plugin>,
}

impl PluginSystem {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
    pub fn register(&mut self, p: Plugin) {
        self.plugins.push(p);
    }
    pub fn call_hook(&self, hook: &str) -> Vec<String> {
        self.plugins
            .iter()
            .filter(|p| p.enabled && p.hooks.iter().any(|h| h == hook))
            .map(|p| p.name.clone())
            .collect()
    }
    pub fn enable(&mut self, name: &str) -> bool {
        if let Some(p) = self.plugins.iter_mut().find(|p| p.name == name) {
            p.enabled = true;
            true
        } else {
            false
        }
    }
    pub fn disable(&mut self, name: &str) -> bool {
        if let Some(p) = self.plugins.iter_mut().find(|p| p.name == name) {
            p.enabled = false;
            true
        } else {
            false
        }
    }
    pub fn enabled_count(&self) -> usize {
        self.plugins.iter().filter(|p| p.enabled).count()
    }
}

impl Default for PluginSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_hook() {
        let mut ps = PluginSystem::new();
        ps.register(Plugin {
            name: "logger".into(),
            version: "1.0".into(),
            hooks: vec!["on_event".into()],
            enabled: true,
        });
        assert_eq!(ps.call_hook("on_event").len(), 1);
    }

    #[test]
    fn test_disable() {
        let mut ps = PluginSystem::new();
        ps.register(Plugin {
            name: "p".into(),
            version: "1.0".into(),
            hooks: vec!["h".into()],
            enabled: true,
        });
        ps.disable("p");
        assert!(ps.call_hook("h").is_empty());
    }
}
