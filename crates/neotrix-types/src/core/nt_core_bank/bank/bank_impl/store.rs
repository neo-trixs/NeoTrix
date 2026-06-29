use std::sync::atomic::Ordering;

use crate::core::layered_memory::{LayeredMemory, MemoryEntry, MemoryLayer};
use crate::core::nt_core_bank::{L1Memory, MemoryTier, OffloadManager, PipelineConfig, PipelineState, ReasoningMemory};

impl super::ReasoningBank {
    pub fn store_with_pipeline(
        &mut self,
        memory: ReasoningMemory,
        offload: &mut OffloadManager,
        state: &mut PipelineState,
        config: &PipelineConfig,
    ) {
        let _ = offload.offload_memory(&memory);
        self.store(memory);
        state.record_memory();
        if state.should_trigger_l1(config) {
            let memories: Vec<ReasoningMemory> = self.memories().iter().cloned().collect();
            let l1_results: Vec<L1Memory> = memories.iter().filter_map(|m| {
                let content = m.t3_views.semantic_view.as_ref()
                    .or(m.t3_views.struct_view.as_ref())?;
                let mem_type = if m.task_description.contains("security") || m.task_description.contains("safety") {
                    "instruction"
                } else if m.reward > 0.7 {
                    "episodic"
                } else {
                    "persona"
                };
                Some(L1Memory {
                    id: uuid::Uuid::new_v4().to_string(),
                    content: content.clone(),
                    mem_type: mem_type.to_string(),
                    priority: m.reward,
                    source_memory_id: m.id.clone(),
                    created_at: chrono::Utc::now().timestamp(),
                })
            }).collect();
            if !l1_results.is_empty() {
                let _ = offload.save_l1_extraction(&l1_results);
                state.l1_count += 1;
                state.pending_memories = 0;
                state.last_l1_time = chrono::Utc::now().timestamp();
            }
        }
    }

    pub fn store(&mut self, memory: ReasoningMemory) {
        if self.memories.len() >= self.max_memories {
            if let Some(oldest) = self.memories.pop_front() {
                if let Some(indices) = self.task_type_index.get_mut(&oldest.task_type) {
                    indices.retain(|&i| i != 0);
                    for indices in self.task_type_index.values_mut() {
                        for idx in indices.iter_mut() {
                            if *idx > 0 { *idx -= 1; }
                        }
                    }
                }
                #[cfg(feature = "e8-theory")]
                if let Some(ref mut wh) = self.wh_index {
                    wh.remove(&oldest.id);
                }
            }
        }
        let new_idx = self.memories.len();
        let mem_id = memory.id.clone();
        let mem_desc = memory.task_description.clone();
        let mem_reward = memory.reward;
        let mem_success = memory.success;
        self.task_type_index.entry(memory.task_type).or_default().push(new_idx);
        #[cfg(feature = "e8-theory")]
        if let Some(ref mut wh) = self.wh_index {
            let mut text = format!("{} {:?}", memory.task_description, memory.task_type);
            if let Some(ref v) = memory.t3_views.struct_view { text.push_str(&format!(" struct:{}", v)); }
            if let Some(ref v) = memory.t3_views.semantic_view { text.push_str(&format!(" semantic:{}", v)); }
            if let Some(ref v) = memory.t3_views.reflect_view { text.push_str(&format!(" reflect:{}", v)); }
            wh.store(&memory.id, &text);
        }
        if self.wraps_layered_memory {
            if let Some(ref mut layered) = self.layered {
                let tag = if mem_success { "OK" } else { "FAIL" };
                let value = format!("[{}] {} (reward={:.4})", tag, mem_desc, mem_reward);
                let entry = MemoryEntry::new(&value, MemoryLayer::L4Archive, mem_reward)
                    .with_tags(vec![format!("__key:{}", mem_id)]);
                layered.store(entry);
            }
        }
        self.memories.push_back(memory);
        self.bm25_dirty.store(true, Ordering::SeqCst);
    }

    pub fn enable_layered(&mut self) {
        self.wraps_layered_memory = true;
        let mut layered = LayeredMemory::new();
        for mem in &self.memories {
            let tag = if mem.success { "OK" } else { "FAIL" };
            let value = format!("[{}] {} (reward={:.4})", tag, mem.task_description, mem.reward);
            let entry = MemoryEntry::new(&value, MemoryLayer::L4Archive, mem.reward)
                .with_tags(vec![format!("__key:{}", mem.id)]);
            layered.store(entry);
        }
        self.layered = Some(layered);
    }

    pub fn store_with_layer(&mut self, memory: ReasoningMemory, layer: MemoryLayer) {
        if self.wraps_layered_memory {
            if let Some(ref mut layered) = self.layered {
                let tag = if memory.success { "OK" } else { "FAIL" };
                let value = format!("[{}] {} (reward={:.4})", tag, memory.task_description, memory.reward);
                let entry = MemoryEntry::new(&value, layer, memory.reward)
                    .with_tags(vec![format!("__key:{}", memory.id)]);
                layered.store(entry);
            }
        }
        self.store(memory);
    }

    pub fn recall_from_layer(&mut self, layer: MemoryLayer) -> Vec<&ReasoningMemory> {
        let mem_ids: Vec<String> = match self.layered {
            Some(ref mut layered) => {
                layered.query_layer(&layer).iter()
                    .filter_map(|e| e.tags.iter().find(|t| t.starts_with("__key:")))
                    .map(|t| t["__key:".len()..].to_string())
                    .collect()
            }
            None => return Vec::new(),
        };
        self.memories.iter().filter(|m| mem_ids.contains(&m.id)).collect()
    }

    pub fn store_with_embedding(&mut self, mut memory: ReasoningMemory, embedding: Vec<f64>) {
        memory.embedding = Some(embedding);
        self.store(memory);
    }

    pub fn store_deferred(&mut self, memory: ReasoningMemory) {
        self.memories.push_back(memory);
        self.bm25_dirty.store(true, Ordering::Relaxed);
    }

    pub fn save_pipeline_checkpoint(state: &PipelineState, path: &std::path::Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(state).map_err(|e| format!("Checkpoint serialize: {}", e))?;
        std::fs::write(path, &json).map_err(|e| format!("Checkpoint write: {}", e))
    }

    pub fn load_pipeline_checkpoint(path: &std::path::Path) -> PipelineState {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("[checkpoint] read failed: {}", e);
                return PipelineState::new();
            }
        };
        match serde_json::from_str::<PipelineState>(&content) {
            Ok(state) => state,
            Err(e) => {
                log::warn!("[checkpoint] parse failed: {}", e);
                PipelineState::new()
            }
        }
    }

    pub fn split_context(memories: &[ReasoningMemory]) -> (Vec<String>, Vec<String>) {
        let mut stable = Vec::new();
        let mut dynamic = Vec::new();
        for m in memories {
            let entry = format!("[{}] {} (reward={:.2})", if m.success { "OK" } else { "FAIL" }, m.task_description, m.reward);
            if m.reward > 0.7 || m.tier == MemoryTier::Procedural {
                stable.push(entry);
            } else {
                dynamic.push(entry);
            }
        }
        (stable, dynamic)
    }
}
