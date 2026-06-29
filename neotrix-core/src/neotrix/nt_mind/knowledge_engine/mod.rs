pub mod graph;
pub mod search;
#[cfg(test)]
mod tests;
pub mod types;

pub use graph::KnowledgeEngine;
pub use search::LiteratureSearcher;
pub use types::{
    KnowledgeEngineStats, KnowledgeEntry, KnowledgeEvidenceResult, KnowledgeRelation,
    KnowledgeSourceType, RelationType,
};

#[cfg(test)]
pub(crate) use types::strip_html;
