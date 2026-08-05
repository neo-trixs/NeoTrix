use super::{SourceType, IngestionConfig, ReflectionRound};
use super::reflection_loop::QualityMonitor;

#[derive(Debug, Clone)]
pub struct IngestionScratchpad {
    pub input: String,
    pub source_type: SourceType,
    pub config: IngestionConfig,
    pub round: usize,
    pub converged: bool,
    pub collated: String,
    pub sections: Vec<String>,
    pub entities: Vec<String>,
    pub events: Vec<String>,
    pub relations: Vec<(String, String, String)>,
    pub aligned_entities: Vec<String>,
    pub reasoning: Vec<String>,
    pub skus: Vec<String>,
    pub final_summary: Option<String>,
    pub clarity_delta: f64,
    pub reflection_history: Vec<ReflectionRound>,
    pub quality_monitor: QualityMonitor,
}

impl IngestionScratchpad {
    pub fn new(input: String, source_type: SourceType, config: IngestionConfig) -> Self {
        Self {
            round: 1,
            converged: false,
            input,
            source_type,
            config: config.clone(),
            collated: String::new(),
            sections: Vec::new(),
            entities: Vec::new(),
            events: Vec::new(),
            relations: Vec::new(),
            aligned_entities: Vec::new(),
            reasoning: Vec::new(),
            skus: Vec::new(),
            final_summary: None,
            clarity_delta: 1.0,
            reflection_history: Vec::new(),
            quality_monitor: QualityMonitor::new(config.quality_threshold),
        }
    }
}

pub fn should_continue_reflection(pad: &IngestionScratchpad) -> bool {
    !pad.converged && pad.round <= pad.config.max_rounds
}
