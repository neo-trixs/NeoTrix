use std::path::Path;
use std::sync::atomic::Ordering;

use crate::core::nt_core_bank::{
    L1Memory, MemoryTier, OffloadManager, PipelineConfig, PipelineState, ReasoningBank,
    ReasoningMemory,
};

impl ReasoningBank {
    pub fn store_with_pipeline(
        &mut self,
        memory: ReasoningMemory,
        offload: &mut OffloadManager,
        state: &mut PipelineState,
        config: &PipelineConfig,
    ) {
        if let Err(e) = offload.offload_memory(&memory) {
            log::warn!("[bank] offload_memory failed: {e}");
        }
        self.store(memory);
        state.record_memory();
        if state.should_trigger_l1(config) {
            let memories: Vec<ReasoningMemory> = self.memories().iter().cloned().collect();
            let l1_results: Vec<L1Memory> = memories
                .iter()
                .filter_map(|m| {
                    let content = m
                        .t3_views
                        .semantic_view
                        .as_ref()
                        .or(m.t3_views.struct_view.as_ref())?;
                    let mem_type = if m.task_description.contains("security")
                        || m.task_description.contains("safety")
                    {
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
                })
                .collect();
            if !l1_results.is_empty() {
                if let Err(e) = offload.save_l1_extraction(&l1_results) {
                    log::warn!("[bank] save_l1_extraction failed: {e}");
                }
                state.l1_count += 1;
                state.pending_memories = 0;
                state.last_l1_time = chrono::Utc::now().timestamp();
            }
        }
    }

    pub fn save_pipeline_checkpoint(state: &PipelineState, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| format!("Checkpoint serialize: {}", e))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json).map_err(|e| format!("Checkpoint write: {}", e))?;
        std::fs::rename(&tmp, path).map_err(|e| format!("Checkpoint rename: {}", e))
    }

    pub fn load_pipeline_checkpoint(path: &Path) -> PipelineState {
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
            let entry = format!(
                "[{}] {} (reward={:.2})",
                if m.success { "OK" } else { "FAIL" },
                m.task_description,
                m.reward
            );
            if m.reward > 0.7 || m.tier == MemoryTier::Procedural {
                stable.push(entry);
            } else {
                dynamic.push(entry);
            }
        }
        (stable, dynamic)
    }

    pub fn store_deferred(&mut self, memory: ReasoningMemory) {
        self.memories.push_back(memory);
        self.bm25_dirty.store(true, Ordering::Relaxed);
    }

    #[cfg(feature = "rkyv-storage")]
    pub fn store_to_rkyv(&self, subdir: &str) -> Result<(), String> {
        use std::path::PathBuf;
        let dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("neotrix")
            .join("rkyv")
            .join(subdir);
        std::fs::create_dir_all(&dir).map_err(|e| format!("create rkyv dir: {}", e))?;
        let path = dir.join("bank.json");
        let memories: Vec<&ReasoningMemory> = self.memories.iter().collect();
        let json = serde_json::to_string_pretty(&memories)
            .map_err(|e| format!("serialize bank: {}", e))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &json).map_err(|e| format!("write bank: {}", e))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("rename bank: {}", e))?;
        log::info!("[rkyv] stored {} memories to {:?}", memories.len(), path);
        Ok(())
    }

    #[cfg(feature = "rkyv-storage")]
    pub fn load_from_rkyv(&mut self, subdir: &str) -> Result<usize, String> {
        use std::path::PathBuf;
        let path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("neotrix")
            .join("rkyv")
            .join(subdir)
            .join("bank.json");
        if !path.exists() {
            return Ok(0);
        }
        let json = std::fs::read_to_string(&path).map_err(|e| format!("read bank: {}", e))?;
        let memories: Vec<ReasoningMemory> =
            serde_json::from_str(&json).map_err(|e| format!("deserialize bank: {}", e))?;
        let count = memories.len();
        for mem in memories {
            self.store(mem);
        }
        log::info!("[rkyv] loaded {} memories from {:?}", count, path);
        Ok(count)
    }
}
