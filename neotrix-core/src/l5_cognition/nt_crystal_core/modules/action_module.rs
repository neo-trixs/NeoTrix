use super::super::ctm::{Chunk, ChunkType, CTMModule, ModuleType};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct NtActionModule {
    action_history: Mutex<Vec<String>>,
    total_actions: Mutex<u64>,
}

impl Default for NtActionModule {
    fn default() -> Self {
        Self::new()
    }
}

impl NtActionModule {
    pub fn new() -> Self {
        Self {
            action_history: Mutex::new(Vec::new()),
            total_actions: Mutex::new(0),
        }
    }
}

impl CTMModule for NtActionModule {
    fn name(&self) -> &str {
        "NT-ACTION"
    }

    fn module_type(&self) -> ModuleType {
        ModuleType::Action
    }

    fn execute(&self, input: &Chunk) -> Chunk {
        let action_suggestion = match input.chunk_type {
            ChunkType::Memory => "consolidate_and_index",
            ChunkType::Perception => "process_and_respond",
            ChunkType::Emotion => "regulate_and_balance",
            ChunkType::Meta => "reflect_and_adjust",
            ChunkType::Safety => "enforce_and_protect",
            ChunkType::Override => "abort_and_secure",
            ChunkType::Action => "continue_current",
        };

        {
            let mut history = self.action_history.lock().unwrap();
            history.push(action_suggestion.to_string());
            if history.len() > 100 {
                history.remove(0);
            }
        }
        *self.total_actions.lock().unwrap() += 1;

        let total = *self.total_actions.lock().unwrap();
        let score = 0.5 + (input.score * 0.3);

        Chunk {
            content: format!(
                "Action suggestion: {} based on {} input (total actions: {})",
                action_suggestion,
                format!("{:?}", input.chunk_type),
                total
            ),
            score: score.clamp(0.0, 1.0),
            source_module: "NT-ACTION".into(),
            chunk_type: ChunkType::Action,
            metadata: {
                let mut m = HashMap::new();
                m.insert("action_type".into(), action_suggestion.into());
                m.insert("total_actions".into(), total.to_string());
                m
            },
        }
    }

    fn write(&mut self, chunk: Chunk) {
        let mut history = self.action_history.lock().unwrap();
        history.push(format!("broadcast: {}", chunk.content));
        if history.len() > 100 {
            history.remove(0);
        }
    }

    fn max_response_time_ms(&self) -> u64 {
        800
    }
}
