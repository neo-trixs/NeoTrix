use crate::core::nt_core_translate::language::Language;

pub const TRANSLATION_ENGINE_STORAGE_FILE: &str = "translation_engine.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TranslationStrategy {
    DirectLookup,
    Compositional,
    Analogical,
    Refinement,
}

#[derive(Debug, Clone)]
pub struct TranslationResult {
    pub source_text: String,
    pub target_text: String,
    pub source_lang: Language,
    pub target_lang: Language,
    pub strategy: TranslationStrategy,
    pub confidence: f64,
    pub vsa_similarity: f64,
    pub entry_id: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineSaveData {
    pub entries: Vec<crate::core::nt_core_translate::bilingual::BilingualEntry>,
    pub next_id: u64,
    pub max_entries: usize,
    pub strategy_stats: std::collections::HashMap<String, (u64, u64)>,
    pub total_translations: u64,
}
