use super::super::ctm::{Chunk, ChunkType, CTMModule, ModuleType};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct NtPerceptionModule {
    perceived_count: Mutex<u64>,
    last_input_domain: Mutex<String>,
}

impl NtPerceptionModule {
    pub fn new() -> Self {
        Self {
            perceived_count: Mutex::new(0),
            last_input_domain: Mutex::new(String::new()),
        }
    }
}

impl CTMModule for NtPerceptionModule {
    fn name(&self) -> &str {
        "NT-PERCEPTION"
    }

    fn module_type(&self) -> ModuleType {
        ModuleType::Perception
    }

    fn execute(&self, input: &Chunk) -> Chunk {
        {
            let mut count = self.perceived_count.lock().unwrap();
            *count += 1;
        }

        let domain = input
            .metadata
            .get("domain")
            .cloned()
            .unwrap_or_else(|| "unknown".into());

        *self.last_input_domain.lock().unwrap() = domain.clone();

        let urgency = input.metadata.get("urgency").and_then(|v| v.parse::<f64>().ok());
        let arousal = input.metadata.get("arousal").and_then(|v| v.parse::<f64>().ok());

        let score = input.score * 0.8
            + urgency.unwrap_or(0.0) * 0.1
            + arousal.unwrap_or(0.5) * 0.1;

        let perceived_count = *self.perceived_count.lock().unwrap();

        Chunk {
            content: format!(
                "Perceived input from domain '{}': {} (total perceptions: {})",
                domain, input.content, perceived_count
            ),
            score: score.clamp(0.0, 1.0),
            source_module: "NT-PERCEPTION".into(),
            chunk_type: ChunkType::Perception,
            metadata: {
                let mut m = HashMap::new();
                m.insert("domain".into(), domain);
                m.insert("total_perceptions".into(), perceived_count.to_string());
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

        *self.last_input_domain.lock().unwrap() = domain;
    }

    fn max_response_time_ms(&self) -> u64 {
        500
    }
}
