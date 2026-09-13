pub mod episodic;
pub mod semantic;
pub mod emotional;
pub mod consolidation;

pub use episodic::{EpisodicMemory, EpisodicMemoryStore};
pub use semantic::{SemanticMemory, SemanticMemoryStore, AbstractionLevel};
pub use emotional::{EmotionalMemory, EmotionalMemoryStore, EmotionLabel};
pub use consolidation::{MemoryConsolidation, ConsolidationConfig, ConsolidationStatistics};
