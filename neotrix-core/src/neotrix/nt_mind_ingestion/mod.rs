use std::collections::HashMap;

pub mod scratchpad;
pub use scratchpad::*;

mod reflection_loop;
pub use reflection_loop::*;

pub mod skill_docs;
pub use skill_docs::*;

mod book_pipeline;
pub use book_pipeline::*;

mod paper_pipeline;
pub use paper_pipeline::*;

pub mod integration_stage;
pub use integration_stage::*;

pub mod pipeline_stages;

pub mod agent_orchestrator;
pub mod fingerprint;
pub mod meta_improvement;
pub use fingerprint::*;

pub mod canonical;
pub use canonical::*;

pub mod e8_routing;
pub use e8_routing::*;

pub mod stream_hygiene;
pub use stream_hygiene::*;

pub mod graceful_degradation;
pub mod intrinsic_value;
pub mod self_preservation;

pub mod storm_breaker;
pub use storm_breaker::*;

pub mod document_parser;

pub mod dmn;
pub use dmn::*;

pub mod forgetting;
pub use forgetting::*;

pub mod compaction;
pub use compaction::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IngestionSourceType {
    Book,
    Paper,
    Code,
    Web,
    Conversation,
    Finance,
    Media,
    Social,
}

impl IngestionSourceType {
    pub fn name(&self) -> &str {
        match self {
            Self::Book => "book",
            Self::Paper => "paper",
            Self::Code => "code",
            Self::Web => "web",
            Self::Conversation => "conversation",
            Self::Finance => "finance",
            Self::Media => "media",
            Self::Social => "social",
        }
    }

    pub fn all() -> Vec<IngestionSourceType> {
        vec![
            Self::Book,
            Self::Paper,
            Self::Code,
            Self::Web,
            Self::Conversation,
            Self::Finance,
            Self::Media,
            Self::Social,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct IngestionConfig {
    pub source_type: IngestionSourceType,
    pub max_rounds: usize,
    pub convergence_threshold: f64,
    pub quality_threshold: f64,
    pub auto_store: bool,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            source_type: IngestionSourceType::Web,
            max_rounds: 5,
            convergence_threshold: 0.05,
            quality_threshold: 0.7,
            auto_store: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReflectionRound {
    pub round: usize,
    pub insights: Vec<String>,
    pub clarity_delta: f64,
    pub converged: bool,
}

#[derive(Debug, Clone)]
pub struct IngestionResult {
    pub source_type: IngestionSourceType,
    pub title: String,
    pub summary: String,
    pub total_rounds: usize,
    pub final_quality: f64,
    pub converged: bool,
    pub entities: Vec<String>,
    pub relations: Vec<(String, String, String)>,
    pub reasoning_notes: Vec<String>,
    pub reflection_history: Vec<ReflectionRound>,
}

pub trait IngestionPipeline: Send + Sync {
    fn name(&self) -> &str;
    fn source_type(&self) -> IngestionSourceType;
    fn process(&self, input: &str, config: &IngestionConfig) -> IngestionResult;
}

pub struct IngestionCore {
    pipelines: HashMap<IngestionSourceType, Box<dyn IngestionPipeline>>,
}

impl IngestionCore {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    pub fn register(&mut self, pipeline: Box<dyn IngestionPipeline>) {
        let st = pipeline.source_type();
        self.pipelines.insert(st, pipeline);
    }

    pub fn process(&self, input: &str, source_type: IngestionSourceType) -> IngestionResult {
        let config = IngestionConfig {
            source_type,
            ..Default::default()
        };
        self.process_with_config(input, &config)
    }

    pub fn process_with_config(&self, input: &str, config: &IngestionConfig) -> IngestionResult {
        match self.pipelines.get(&config.source_type) {
            Some(pipeline) => pipeline.process(input, config),
            None => IngestionResult {
                source_type: config.source_type,
                title: "unprocessed".to_string(),
                summary: format!("No pipeline registered for {:?}", config.source_type),
                total_rounds: 0,
                final_quality: 0.0,
                converged: false,
                entities: vec![],
                relations: vec![],
                reasoning_notes: vec![],
                reflection_history: vec![],
            },
        }
    }

    pub fn auto_detect_type(input: &str) -> IngestionSourceType {
        let lower = input.to_lowercase();
        if lower.starts_with("http") || lower.starts_with("www.") {
            IngestionSourceType::Web
        } else if lower.contains("arxiv") || lower.contains("doi") || lower.contains(".pdf") {
            IngestionSourceType::Paper
        } else if lower.contains("github.com") || lower.contains("gitlab") {
            IngestionSourceType::Code
        } else if lower.contains("book") || lower.contains("chapter") || lower.contains("isbn") {
            IngestionSourceType::Book
        } else {
            IngestionSourceType::Web
        }
    }
}

impl Default for IngestionCore {
    fn default() -> Self {
        let mut core = Self::new();
        core.register(Box::new(BookPipeline::new()));
        core.register(Box::new(PaperPipeline::new()));
        core
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_name() {
        assert_eq!(IngestionSourceType::Book.name(), "book");
        assert_eq!(IngestionSourceType::Paper.name(), "paper");
        assert_eq!(IngestionSourceType::Code.name(), "code");
        assert_eq!(IngestionSourceType::Media.name(), "media");
    }

    #[test]
    fn test_source_type_all() {
        let all = IngestionSourceType::all();
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn test_auto_detect_type() {
        assert_eq!(
            IngestionCore::auto_detect_type("https://example.com"),
            IngestionSourceType::Web
        );
        assert_eq!(
            IngestionCore::auto_detect_type("arxiv:2301.12345"),
            IngestionSourceType::Paper
        );
        assert_eq!(
            IngestionCore::auto_detect_type("github.com/neotrix/neotrix"),
            IngestionSourceType::Code
        );
        assert_eq!(
            IngestionCore::auto_detect_type("a book about AI"),
            IngestionSourceType::Book
        );
        assert_eq!(
            IngestionCore::auto_detect_type("hi there"),
            IngestionSourceType::Web
        );
    }

    #[test]
    fn test_ingestion_core_default() {
        let core = IngestionCore::default();
        let result = core.process(
            "A sample book text for ingestion",
            IngestionSourceType::Book,
        );
        assert_eq!(result.source_type, IngestionSourceType::Book);
        assert!(result.total_rounds > 0);
    }

    #[test]
    fn test_ingestion_core_no_pipeline_fallback() {
        let core = IngestionCore::new();
        let result = core.process("anything", IngestionSourceType::Finance);
        assert_eq!(result.total_rounds, 0);
        assert!(result.summary.contains("No pipeline"));
    }

    #[test]
    fn test_ingestion_core_with_config() {
        let core = IngestionCore::default();
        let config = IngestionConfig {
            source_type: IngestionSourceType::Paper,
            max_rounds: 3,
            convergence_threshold: 0.01,
            quality_threshold: 0.5,
            auto_store: false,
        };
        let result = core.process_with_config("arxiv:2401.12345 a paper about ML", &config);
        assert_eq!(result.total_rounds, 3);
        assert!(result.final_quality > 0.0);
    }

    #[test]
    fn test_book_pipeline_rounds() {
        let pipeline = BookPipeline::new();
        let config = IngestionConfig {
            source_type: IngestionSourceType::Book,
            max_rounds: 5,
            convergence_threshold: 0.1,
            quality_threshold: 0.6,
            auto_store: true,
        };
        let result = pipeline.process("The Art of Computer Programming\nChapter 1: Algorithms\nThis is a test book content for processing.", &config);
        assert_eq!(result.source_type, IngestionSourceType::Book);
        assert!(result.total_rounds <= 5);
        assert!(result.total_rounds >= 1);
    }

    #[test]
    fn test_reflection_loop_convergence() {
        let mut rl = ReflectionLoop::new(10, 0.05);
        assert!(rl.should_continue());
        rl.record_round(vec!["insight1".to_string()], 0.8);
        assert!(rl.should_continue());
        rl.record_round(vec!["insight2".to_string()], 0.03);
        assert!(!rl.should_continue());
        assert!(rl.converged);
        assert_eq!(rl.total_rounds(), 2);
    }

    #[test]
    fn test_reflection_loop_max_rounds() {
        let mut rl = ReflectionLoop::new(3, 0.001);
        for i in 0..3 {
            assert!(rl.should_continue());
            rl.record_round(vec![format!("round_{}", i + 1)], 0.5);
        }
        assert!(!rl.should_continue());
        assert!(rl.converged);
        assert_eq!(rl.total_rounds(), 3);
    }

    #[test]
    fn test_quality_monitor() {
        let mut qm = QualityMonitor::new(0.7);
        let score = qm.evaluate(0.2, 5, 10);
        assert!(score > 0.0);
        assert!(score <= 1.0);
        assert!(qm.is_acceptable());
        let score2 = qm.evaluate(0.9, 0, 0);
        assert!(score2 >= 0.0);
    }

    #[test]
    fn test_paper_pipeline() {
        let pipeline = PaperPipeline::new();
        let config = IngestionConfig::default();
        let result = pipeline.process("Attention Is All You Need\narxiv:1706.03762", &config);
        assert_eq!(result.source_type, IngestionSourceType::Paper);
        assert!(result.total_rounds > 0);
    }
}
