#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineConfig {
    pub l1_trigger_count: usize,
    pub l2_trigger_count: usize,
    pub l3_trigger_count: usize,
    pub offload_threshold: usize,
    pub max_ref_files: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self { l1_trigger_count: 5, l2_trigger_count: 3, l3_trigger_count: 5, offload_threshold: 20, max_ref_files: 100 }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineState {
    pub total_memories: usize,
    pub l1_count: usize,
    pub l2_count: usize,
    pub l3_count: usize,
    pub last_l1_time: i64,
    pub last_l2_time: i64,
    pub last_l3_time: i64,
    pub pending_memories: usize,
}

impl Default for PipelineState {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineState {
    pub fn new() -> Self {
        Self { total_memories: 0, l1_count: 0, l2_count: 0, l3_count: 0, last_l1_time: 0, last_l2_time: 0, last_l3_time: 0, pending_memories: 0 }
    }

    pub fn record_memory(&mut self) {
        self.total_memories += 1;
        self.pending_memories += 1;
    }

    pub fn should_trigger_l1(&self, config: &PipelineConfig) -> bool {
        self.pending_memories >= config.l1_trigger_count
    }

    pub fn should_trigger_l2(&self, config: &PipelineConfig) -> bool {
        self.l1_count > 0 && self.l1_count.is_multiple_of(config.l2_trigger_count)
            && self.l2_count < self.l1_count / config.l2_trigger_count
    }

    pub fn should_trigger_l3(&self, config: &PipelineConfig) -> bool {
        self.l2_count > 0 && self.l2_count.is_multiple_of(config.l3_trigger_count)
            && self.l3_count < self.l2_count / config.l3_trigger_count
    }
}
