use std::collections::HashMap;

pub struct DeferredLoader {
    modules: HashMap<String, ModuleState>,
    load_order: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleState {
    pub name: String,
    pub loaded: bool,
    pub load_time_ms: Option<u64>,
    pub size_bytes: usize,
}

impl DeferredLoader {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            load_order: Vec::new(),
        }
    }

    pub fn register(&mut self, name: &str, size_bytes: usize) {
        if !self.modules.contains_key(name) {
            self.modules.insert(
                name.to_string(),
                ModuleState {
                    name: name.to_string(),
                    loaded: false,
                    load_time_ms: None,
                    size_bytes,
                },
            );
            self.load_order.push(name.to_string());
        }
    }

    pub fn load(&mut self, name: &str) -> Result<u64, String> {
        let module = self
            .modules
            .get_mut(name)
            .ok_or_else(|| format!("Module not found: {}", name))?;
        if module.loaded {
            return Ok(module.load_time_ms.unwrap_or(0));
        }
        let start = std::time::Instant::now();
        // Simulate loading - in real impl would load from disk/network
        std::thread::sleep(std::time::Duration::from_millis(1));
        let elapsed = start.elapsed().as_millis() as u64;
        module.loaded = true;
        module.load_time_ms = Some(elapsed);
        Ok(elapsed)
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.modules.get(name).map_or(false, |m| m.loaded)
    }
    pub fn loaded_count(&self) -> usize {
        self.modules.values().filter(|m| m.loaded).count()
    }
    pub fn total_count(&self) -> usize {
        self.modules.len()
    }
    pub fn total_size(&self) -> usize {
        self.modules.values().map(|m| m.size_bytes).sum()
    }
    pub fn loaded_size(&self) -> usize {
        self.modules
            .values()
            .filter(|m| m.loaded)
            .map(|m| m.size_bytes)
            .sum()
    }
    pub fn load_order(&self) -> &[String] {
        &self.load_order
    }
    pub fn modules(&self) -> &HashMap<String, ModuleState> {
        &self.modules
    }
}
impl Default for DeferredLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_register_and_load() {
        let mut l = DeferredLoader::new();
        l.register("core", 1024);
        l.register("utils", 512);
        assert!(!l.is_loaded("core"));
        l.load("core").unwrap();
        assert!(l.is_loaded("core"));
        assert_eq!(l.loaded_count(), 1);
    }
    #[test]
    fn test_load_nonexistent() {
        let mut l = DeferredLoader::new();
        assert!(l.load("missing").is_err());
    }
    #[test]
    fn test_size_tracking() {
        let mut l = DeferredLoader::new();
        l.register("a", 100);
        l.register("b", 200);
        assert_eq!(l.total_size(), 300);
        l.load("a").unwrap();
        assert_eq!(l.loaded_size(), 100);
    }
}
