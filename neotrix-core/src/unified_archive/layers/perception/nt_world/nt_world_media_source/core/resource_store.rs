use std::collections::HashMap;

pub struct ResourceStore {
    resources: HashMap<String, Vec<u8>>,
}

impl ResourceStore {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.resources.get(key).map(|v| v.as_slice())
    }

    pub fn insert(&mut self, key: String, data: Vec<u8>) {
        self.resources.insert(key, data);
    }

    pub fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        self.resources.remove(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.resources.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.resources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    pub fn clear(&mut self) {
        self.resources.clear();
    }
}

impl Default for ResourceStore {
    fn default() -> Self {
        Self::new()
    }
}
