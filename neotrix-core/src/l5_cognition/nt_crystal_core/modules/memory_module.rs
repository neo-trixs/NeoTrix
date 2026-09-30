use super::super::ctm::{Chunk, ChunkType, CTMModule, ModuleType};
use super::super::consciousness::{CrystalConsciousness, MemoryType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct NtMemoryModule {
    consciousness: Arc<Mutex<CrystalConsciousness>>,
}

impl NtMemoryModule {
    pub fn new(consciousness: Arc<Mutex<CrystalConsciousness>>) -> Self {
        Self { consciousness }
    }
}

impl CTMModule for NtMemoryModule {
    fn name(&self) -> &str {
        "NT-MEMORY"
    }

    fn module_type(&self) -> ModuleType {
        ModuleType::Memory
    }

    fn execute(&self, input: &Chunk) -> Chunk {
        let domain = input
            .metadata
            .get("domain")
            .cloned()
            .unwrap_or_else(|| "default".into());

        let recalled = {
            let mut cc = self.consciousness.lock().unwrap();
            cc.recall(&domain, None, 5)
        };

        let content = if recalled.is_empty() {
            format!("No memories found in domain '{}'", domain)
        } else {
            let summaries: Vec<String> = recalled
                .iter()
                .map(|m| format!("[{}] {}", m.id, m.content))
                .collect();
            format!("Recalled {} memories: {}", recalled.len(), summaries.join("; "))
        };

        let score = if recalled.is_empty() { 0.3 } else { 0.6 };

        Chunk {
            content,
            score,
            source_module: "NT-MEMORY".into(),
            chunk_type: ChunkType::Memory,
            metadata: {
                let mut m = HashMap::new();
                m.insert("domain".into(), domain);
                m.insert(
                    "memory_count".into(),
                    recalled.len().to_string(),
                );
                m
            },
        }
    }

    fn write(&mut self, chunk: Chunk) {
        let domain = chunk
            .metadata
            .get("domain")
            .cloned()
            .unwrap_or_else(|| "broadcast".into());

        let memory_type = match chunk.chunk_type {
            ChunkType::Perception => MemoryType::Fact,
            ChunkType::Action => MemoryType::Experience,
            ChunkType::Emotion => MemoryType::Pattern,
            ChunkType::Meta => MemoryType::Causal,
            ChunkType::Safety => MemoryType::Lesson,
            ChunkType::Override => MemoryType::Contradiction,
            _ => MemoryType::Fact,
        };

        let mut cc = self.consciousness.lock().unwrap();
        cc.remember(&chunk.content, memory_type, &domain, chunk.score);
    }

    fn max_response_time_ms(&self) -> u64 {
        2000
    }
}
