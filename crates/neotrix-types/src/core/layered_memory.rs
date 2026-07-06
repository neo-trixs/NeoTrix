/// LayeredMemory — stub for hierarchical memory
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryLayer {
    L0Working,
    L1Recent,
    L2Consolidated,
    L3LongTerm,
    L4Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub value: String,
    pub layer: MemoryLayer,
    pub priority: f64,
    pub tags: Vec<String>,
}

impl MemoryEntry {
    pub fn new(value: &str, layer: MemoryLayer, priority: f64) -> Self {
        Self {
            value: value.to_string(),
            layer,
            priority,
            tags: Vec::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayeredMemory {
    entries: Vec<MemoryEntry>,
}

impl LayeredMemory {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn store(&mut self, entry: MemoryEntry) {
        self.entries.push(entry);
    }

    pub fn query_layer(&self, layer: &MemoryLayer) -> Vec<&MemoryEntry> {
        self.entries.iter().filter(|e| std::mem::discriminant(&e.layer) == std::mem::discriminant(layer)).collect()
    }
}
