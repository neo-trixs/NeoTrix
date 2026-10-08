use super::super::ctm::{Chunk, ChunkType, CTMModule, ModuleType};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct NtMetaModule {
    tick_count: Mutex<u64>,
    observed_chunks: Mutex<Vec<String>>,
    self_model: Mutex<HashMap<String, f64>>,
}

impl Default for NtMetaModule {
    fn default() -> Self {
        Self::new()
    }
}

impl NtMetaModule {
    pub fn new() -> Self {
        let mut self_model = HashMap::new();
        self_model.insert("confidence".into(), 0.5);
        self_model.insert("self_awareness".into(), 0.3);
        self_model.insert("adaptability".into(), 0.5);

        Self {
            tick_count: Mutex::new(0),
            observed_chunks: Mutex::new(Vec::new()),
            self_model: Mutex::new(self_model),
        }
    }
}

impl CTMModule for NtMetaModule {
    fn name(&self) -> &str {
        "NT-META"
    }

    fn module_type(&self) -> ModuleType {
        ModuleType::Meta
    }

    fn execute(&self, input: &Chunk) -> Chunk {
        *self.tick_count.lock().unwrap() += 1;

        {
            let mut observed = self.observed_chunks.lock().unwrap();
            observed.push(format!("{:?}", input.chunk_type));
            if observed.len() > 50 {
                observed.remove(0);
            }
        }

        let tick = *self.tick_count.lock().unwrap();
        let observed = self.observed_chunks.lock().unwrap();

        let chunk_type_counts: HashMap<String, usize> =
            observed.iter().fold(HashMap::new(), |mut acc, ct| {
                *acc.entry(ct.clone()).or_insert(0) += 1;
                acc
            });

        let dominant_type = chunk_type_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(ct, _)| ct.as_str())
            .unwrap_or("none");

        let mut model = self.self_model.lock().unwrap();
        let confidence = model.get("confidence").copied().unwrap_or(0.5);
        let new_confidence = (confidence * 0.9 + input.score * 0.1).clamp(0.0, 1.0);
        model.insert("confidence".into(), new_confidence);

        let awareness = model.get("self_awareness").copied().unwrap_or(0.3);
        let new_awareness = (awareness + 0.001).min(1.0);
        model.insert("self_awareness".into(), new_awareness);

        Chunk {
            content: format!(
                "Meta observation at tick {}: dominant type='{}', confidence={:.3}, awareness={:.3}",
                tick, dominant_type, new_confidence, new_awareness
            ),
            score: 0.4 + new_confidence * 0.3,
            source_module: "NT-META".into(),
            chunk_type: ChunkType::Meta,
            metadata: {
                let mut m = HashMap::new();
                m.insert("tick".into(), tick.to_string());
                m.insert("dominant_type".into(), dominant_type.into());
                m.insert("confidence".into(), format!("{:.3}", new_confidence));
                m.insert("self_awareness".into(), format!("{:.3}", new_awareness));
                m
            },
        }
    }

    fn write(&mut self, chunk: Chunk) {
        if let Some(value) = chunk.metadata.get("update_confidence").and_then(|v| v.parse::<f64>().ok()) {
            self.self_model.lock().unwrap().insert("confidence".into(), value);
        }
    }

    fn max_response_time_ms(&self) -> u64 {
        1500
    }
}
