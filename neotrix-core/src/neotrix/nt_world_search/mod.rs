pub mod agentic_searcher;
pub mod ast_search;
pub mod engine;
pub mod evolvable_harness;
pub mod file_watcher;
pub mod frecency;
pub mod grep;
pub mod intervention_engine;
pub mod pathway_awareness;
pub mod web_search;

pub use ast_search::{search_ast, search_ast_in_path, AstPattern};
pub use engine::{FileSearchEngine, SearchQuery, SearchReport};
pub use evolvable_harness::{
    global_harness, record_tool_call, suggest_tool_order, EvolvableHarness, HarnessSummary,
    ToolProfile, TraceRecord,
};
pub use file_watcher::{FileChangeEntry, FileWatcher};
pub use frecency::FrecencyIndex;
pub use grep::{
    format_results, search_file_content, search_file_content_with_mode, Match, SearchMode,
};
pub use intervention_engine::{
    global_intervention_engine, record_failure_pattern, suggest_tool_interventions, FailurePattern,
    InterventionEngine, InterventionTemplate,
};
pub use pathway_awareness::{
    global_pathway_awareness, AsiPathway, Bottleneck, BottleneckCategory, PathwayAwareness,
    PathwaySignals,
};
pub use web_search::{WebSearchEngine, WebSearchResult, WebSearchTool};

pub use agentic_searcher::{
    AgenticSearcher, HybridRetriever, RRFFuser, SearchBudget, SearchEvaluator, SearchPlan,
    SearchPlanner, SearchResultItem, SearchStrategy, SearchVerdict, SearchedDocument,
    SearchedDocumentCollection,
};

use std::sync::Mutex;

static FRECENCY: std::sync::OnceLock<Mutex<FrecencyIndex>> = std::sync::OnceLock::new();

pub fn global_frecency() -> &'static Mutex<FrecencyIndex> {
    FRECENCY.get_or_init(|| Mutex::new(FrecencyIndex::new()))
}

pub fn record_file_access(path: &str) {
    if let Ok(mut idx) = global_frecency().lock() {
        idx.record_access(std::path::PathBuf::from(path));
    }
}

pub fn rank_paths(paths: &mut [String]) {
    if let Ok(idx) = global_frecency().lock() {
        idx.rank(paths);
    }
}

// TODO: add #[serial] to any new tests that use global singletons
