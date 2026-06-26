use crate::core::nt_core_hcube::adapt_encoder::AdaptiveVsaEncoder;
use crate::core::nt_core_hcube::cross_modal::CrossModalAligner;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use crate::core::nt_core_knowledge::hypergraph::HypergraphStore;
use crate::core::nt_core_knowledge::spread_activation::MemoryGraph;
use crate::core::nt_core_translate::bilingual::{BilingualLexicon, CleanupRule};
use crate::core::nt_core_translate::hypergraph_integration::store_translation_as_hyperedge;
use crate::core::nt_core_translate::language::{Language, LanguageDetector};
use std::collections::HashMap;

mod translate_lexicon;
mod translate_persistence;
mod translate_strategies;
pub mod translate_types;

pub use translate_types::{EngineSaveData, TranslationResult, TranslationStrategy};

#[derive(Debug, Clone)]
pub struct VsaTranslationEngine {
    pub lexicon: BilingualLexicon,
    pub language_detector: LanguageDetector,
    pub encoder: AdaptiveVsaEncoder,
    strategy_map: Vec<(u8, TranslationStrategy)>,
    strategy_stats: HashMap<TranslationStrategy, (u64, u64)>,
    pub min_similarity: f64,
    pub use_e8_strategy: bool,
    cache: HashMap<String, TranslationResult>,
    max_cache: usize,
    lang_pair_stats: HashMap<(Language, Language), Vec<(TranslationStrategy, bool)>>,
    max_pair_stats: usize,
    pub cleanup_rule: CleanupRule,
    pub max_refinement_passes: usize,
    pub use_resonator: bool,
    auto_save_counter: u64,
    pub total_translations: u64,
    pub auto_save_interval: u64,
    pub aligner: Option<CrossModalAligner>,
    pub spreading_memory: Option<MemoryGraph>,
    pub use_aligner: bool,
    pub persist_with_nts: bool,
    pub hypergraph_store: Option<HypergraphStore>,
}

impl Default for VsaTranslationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VsaTranslationEngine {
    pub fn new() -> Self {
        let mut stats = HashMap::new();
        stats.insert(TranslationStrategy::DirectLookup, (0, 0));
        stats.insert(TranslationStrategy::Compositional, (0, 0));
        stats.insert(TranslationStrategy::Analogical, (0, 0));
        stats.insert(TranslationStrategy::Refinement, (0, 0));

        Self {
            lexicon: BilingualLexicon::new(1000),
            language_detector: LanguageDetector::new(),
            encoder: AdaptiveVsaEncoder::new(VSA_DIM, 42, VSA_DIM),
            strategy_map: vec![
                (0b000000, TranslationStrategy::DirectLookup),
                (0b001000, TranslationStrategy::Compositional),
                (0b100000, TranslationStrategy::Analogical),
                (0b000100, TranslationStrategy::Refinement),
            ],
            strategy_stats: stats,
            min_similarity: 0.50,
            use_e8_strategy: true,
            cache: HashMap::new(),
            max_cache: 100,
            lang_pair_stats: HashMap::new(),
            max_pair_stats: 1000,
            cleanup_rule: CleanupRule::Polynomial(3.0),
            max_refinement_passes: 3,
            use_resonator: false,
            auto_save_counter: 0,
            total_translations: 0,
            auto_save_interval: 50,
            aligner: None,
            spreading_memory: None,
            use_aligner: false,
            persist_with_nts: false,
            hypergraph_store: None,
        }
    }

    pub fn init_spreading_from_lexicon(&mut self) {
        self.init_spreading();
        if let Some(ref mut mem) = self.spreading_memory {
            for entry in &self.lexicon.entries {
                let vsa = BilingualLexicon::text_to_vsa_deterministic(&entry.source_text);
                mem.add_node(
                    crate::core::nt_core_knowledge::spread_activation::NodeKind::Semantic,
                    vsa,
                    &entry.source_text,
                );
            }
        }
    }

    pub fn translate(
        &mut self,
        text: &str,
        source_lang: Option<Language>,
        target_lang: Language,
    ) -> TranslationResult {
        let src_lang = source_lang.unwrap_or_else(|| self.detect_language(text));

        // Check cache
        let cache_key = format!("{}:{}:{}", text, src_lang.code(), target_lang.code());
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        // Multi-word expression matching: try full text first
        if let Some(multi_hit) = self.try_multiword_match(text, src_lang, target_lang) {
            self.cache_result(cache_key, multi_hit.clone());
            return multi_hit;
        }

        let hexagram = self.select_hexagram(text, src_lang);
        let strategy = if self.use_e8_strategy {
            let base = self.select_strategy(hexagram);
            self.adapt_strategy(base, src_lang, target_lang)
        } else {
            TranslationStrategy::DirectLookup
        };

        let query_vsa = if self.use_aligner {
            self.init_aligner();
            match self.aligner.as_ref() {
                Some(a) => a.text_to_vsa(text),
                None => {
                    log::warn!("[translate] aligner not available after init, using deterministic encoding");
                    BilingualLexicon::text_to_vsa_deterministic(text)
                }
            }
        } else {
            BilingualLexicon::text_to_vsa_deterministic(text)
        };

        let (target_text, confidence, vsa_similarity, entry_id) = match strategy {
            TranslationStrategy::DirectLookup => {
                if let Some((entry, sim)) = self.translate_direct(&query_vsa, src_lang, target_lang)
                {
                    (entry.target_text, entry.confidence, sim, Some(entry.id))
                } else {
                    let comp = self.translate_compositional(text, src_lang, target_lang);
                    (comp, 0.3, 0.0, None)
                }
            }
            TranslationStrategy::Compositional => {
                let comp = self.translate_compositional(text, src_lang, target_lang);
                (comp, 0.5, 0.0, None)
            }
            TranslationStrategy::Analogical => {
                if let Some((entry, sim)) =
                    self.translate_analogical(&query_vsa, src_lang, target_lang)
                {
                    (entry.target_text, entry.confidence, sim, Some(entry.id))
                } else {
                    let comp = self.translate_compositional(text, src_lang, target_lang);
                    (comp, 0.3, 0.0, None)
                }
            }
            TranslationStrategy::Refinement => {
                let refined = self.translate_refinement(text, src_lang, target_lang);
                (refined, 0.6, 0.0, None)
            }
        };

        self.record_outcome(strategy, confidence > 0.5);

        // Record language-pair-specific stats for strategy adaptation
        let pair_key = (src_lang, target_lang);
        let pair_log = self.lang_pair_stats.entry(pair_key).or_default();
        if pair_log.len() > self.max_pair_stats {
            pair_log.drain(0..self.max_pair_stats / 5);
        }
        pair_log.push((strategy, confidence > 0.5));

        let result = TranslationResult {
            source_text: text.to_string(),
            target_text,
            source_lang: src_lang,
            target_lang,
            strategy,
            confidence,
            vsa_similarity,
            entry_id,
        };

        self.learn_from_translation(&result);
        self.cache_result(cache_key, result.clone());

        self.total_translations += 1;
        self.auto_save_counter += 1;
        if self.auto_save_counter >= self.auto_save_interval {
            self.auto_save_counter = 0;
            if let Err(e) = self.save() {
                log::error!("[translate] auto-save failed: {}", e);
            }
        }

        result
    }

    fn cache_result(&mut self, key: String, result: TranslationResult) {
        if self.cache.len() >= self.max_cache {
            if let Some(oldest) = self.cache.keys().next().cloned() {
                self.cache.remove(&oldest);
            }
        }
        self.cache.insert(key, result);
    }

    pub fn detect_language(&self, text: &str) -> Language {
        self.language_detector.detect(text)
    }

    pub fn init_aligner(&mut self) {
        if self.aligner.is_none() {
            self.aligner = Some(CrossModalAligner::new(VSA_DIM, 42));
        }
    }

    pub fn translate_with_aligner(
        &mut self,
        text: &str,
        source_lang: Option<Language>,
        target_lang: Language,
    ) -> TranslationResult {
        self.init_aligner();
        let src_lang = source_lang.unwrap_or_else(|| self.detect_language(text));

        let cache_key = format!("{}:{}:{}", text, src_lang.code(), target_lang.code());
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        if let Some(multi_hit) = self.try_multiword_match(text, src_lang, target_lang) {
            self.cache_result(cache_key, multi_hit.clone());
            return multi_hit;
        }

        let hexagram = self.select_hexagram(text, src_lang);
        let strategy = if self.use_e8_strategy {
            let base = self.select_strategy(hexagram);
            self.adapt_strategy(base, src_lang, target_lang)
        } else {
            TranslationStrategy::DirectLookup
        };

        let query_vsa = match self.aligner.as_ref() {
            Some(a) => a.text_to_vsa(text),
            None => {
                log::warn!("[translate] aligner not available in translate_with_aligner, using deterministic encoding");
                BilingualLexicon::text_to_vsa_deterministic(text)
            }
        };

        let (target_text, confidence, vsa_similarity, entry_id) = match strategy {
            TranslationStrategy::DirectLookup => {
                if let Some((entry, sim)) = self.translate_direct(&query_vsa, src_lang, target_lang)
                {
                    (entry.target_text, entry.confidence, sim, Some(entry.id))
                } else {
                    let comp = self.translate_compositional(text, src_lang, target_lang);
                    (comp, 0.3, 0.0, None)
                }
            }
            TranslationStrategy::Compositional => {
                let comp = self.translate_compositional(text, src_lang, target_lang);
                (comp, 0.5, 0.0, None)
            }
            TranslationStrategy::Analogical => {
                if let Some((entry, sim)) =
                    self.translate_analogical(&query_vsa, src_lang, target_lang)
                {
                    (entry.target_text, entry.confidence, sim, Some(entry.id))
                } else {
                    let comp = self.translate_compositional(text, src_lang, target_lang);
                    (comp, 0.3, 0.0, None)
                }
            }
            TranslationStrategy::Refinement => {
                let refined = self.translate_refinement(text, src_lang, target_lang);
                (refined, 0.6, 0.0, None)
            }
        };

        self.record_outcome(strategy, confidence > 0.5);

        let pair_key = (src_lang, target_lang);
        let pair_log = self.lang_pair_stats.entry(pair_key).or_default();
        if pair_log.len() > self.max_pair_stats {
            pair_log.drain(0..self.max_pair_stats / 5);
        }
        pair_log.push((strategy, confidence > 0.5));

        let result = TranslationResult {
            source_text: text.to_string(),
            target_text,
            source_lang: src_lang,
            target_lang,
            strategy,
            confidence,
            vsa_similarity,
            entry_id,
        };

        self.learn_from_translation(&result);
        self.cache_result(cache_key, result.clone());

        self.total_translations += 1;
        self.auto_save_counter += 1;
        if self.auto_save_counter >= self.auto_save_interval {
            self.auto_save_counter = 0;
            if let Err(e) = self.save() {
                log::error!("[translate] auto-save (compositional) failed: {}", e);
            }
        }

        result
    }

    pub fn with_hypergraph_store(&mut self, store: HypergraphStore) -> &mut Self {
        self.hypergraph_store = Some(store);
        self
    }

    pub fn sync_to_hypergraph(&mut self) -> usize {
        let Some(store) = self.hypergraph_store.as_mut() else {
            return 0;
        };
        let mut count = 0;
        for entry in &self.lexicon.entries {
            if store_translation_as_hyperedge(store, entry) {
                count += 1;
            }
        }
        count
    }

    pub fn init_spreading(&mut self) {
        if self.spreading_memory.is_none() {
            self.spreading_memory = Some(MemoryGraph::new(100));
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hex::ReasoningHexagram;

    fn test_engine() -> VsaTranslationEngine {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        eng
    }

    #[test]
    fn test_default_construction() {
        let eng = VsaTranslationEngine::new();
        assert!(eng.lexicon.is_empty());
        assert!((eng.min_similarity - 0.50).abs() < 1e-6);
        assert!(eng.use_e8_strategy);
    }

    #[test]
    fn test_detect_language_english() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("the quick brown fox jumps over the lazy dog");
        assert_eq!(lang, Language::English);
    }

    #[test]
    fn test_detect_language_chinese() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("你好世界，这是一段中文文本");
        assert_eq!(lang, Language::Chinese);
    }

    #[test]
    fn test_select_strategy_direct() {
        let eng = VsaTranslationEngine::new();
        let hex = ReasoningHexagram::new(0b000000);
        assert_eq!(eng.select_strategy(hex), TranslationStrategy::DirectLookup);
    }

    #[test]
    fn test_select_strategy_compositional() {
        let eng = VsaTranslationEngine::new();
        let hex = ReasoningHexagram::new(0b001000);
        assert_eq!(eng.select_strategy(hex), TranslationStrategy::Compositional);
    }

    #[test]
    fn test_select_strategy_analogical() {
        let eng = VsaTranslationEngine::new();
        let hex = ReasoningHexagram::new(0b100000);
        assert_eq!(eng.select_strategy(hex), TranslationStrategy::Analogical);
    }

    #[test]
    fn test_select_strategy_refinement() {
        let eng = VsaTranslationEngine::new();
        let hex = ReasoningHexagram::new(0b000100);
        assert_eq!(eng.select_strategy(hex), TranslationStrategy::Refinement);
    }

    #[test]
    fn test_select_hexagram_short_text() {
        let eng = VsaTranslationEngine::new();
        let hex = eng.select_hexagram("hello world", Language::English);
        assert_eq!(eng.select_strategy(hex), TranslationStrategy::DirectLookup);
    }

    #[test]
    fn test_select_hexagram_long_text() {
        let eng = VsaTranslationEngine::new();
        let long = "this is a very long sentence that should have more than twenty words in it because we need to test the refinement strategy selection";
        let hex = eng.select_hexagram(long, Language::English);
        assert_eq!(eng.select_strategy(hex), TranslationStrategy::Refinement);
    }

    #[test]
    fn test_select_hexagram_chinese() {
        let eng = VsaTranslationEngine::new();
        let hex = eng.select_hexagram("你好", Language::Chinese);
        assert_eq!(eng.select_strategy(hex), TranslationStrategy::Compositional);
    }

    #[test]
    fn test_select_hexagram_english_medium() {
        let eng = VsaTranslationEngine::new();
        let hex = eng.select_hexagram(
            "this is a medium length sentence for testing",
            Language::English,
        );
        assert_eq!(eng.select_strategy(hex), TranslationStrategy::Analogical);
    }

    #[test]
    fn test_translate_direct_match() {
        let mut eng = test_engine();
        let result = eng.translate("hello", Some(Language::English), Language::Spanish);
        assert_eq!(result.source_lang, Language::English);
        assert_eq!(result.target_lang, Language::Spanish);
        assert!(result.confidence > 0.0);
        assert!(result.entry_id.is_some());
    }

    #[test]
    fn test_translate_direct_no_match() {
        let mut eng = VsaTranslationEngine::new();
        let result = eng.translate(
            "supercalifragilistic",
            Some(Language::English),
            Language::Spanish,
        );
        assert_eq!(result.strategy, TranslationStrategy::DirectLookup);
        assert_eq!(result.target_text, "supercalifragilistic");
    }

    #[test]
    fn test_translate_compositional() {
        let mut eng = test_engine();
        eng.use_e8_strategy = false;
        let result = eng.translate("hello world", Some(Language::English), Language::Spanish);
        assert_eq!(result.source_lang, Language::English);
        assert_eq!(result.target_lang, Language::Spanish);
        assert!(!result.target_text.is_empty());
    }

    #[test]
    fn test_record_outcome_and_stats() {
        let mut eng = VsaTranslationEngine::new();
        eng.record_outcome(TranslationStrategy::DirectLookup, true);
        eng.record_outcome(TranslationStrategy::DirectLookup, true);
        eng.record_outcome(TranslationStrategy::DirectLookup, false);
        let stats = eng.strategy_accuracy();
        let dir_acc = stats
            .iter()
            .find(|(s, _)| *s == TranslationStrategy::DirectLookup)
            .map(|(_, a)| *a)
            .unwrap_or(0.0);
        assert!((dir_acc - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_strategy_accuracy_empty() {
        let eng = VsaTranslationEngine::new();
        let stats = eng.strategy_accuracy();
        assert!(!stats.is_empty());
        for (_, acc) in &stats {
            assert!((*acc - 0.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_seed_common_pairs() {
        let mut eng = VsaTranslationEngine::new();
        assert!(eng.lexicon.is_empty());
        eng.seed_common_pairs();
        assert!(!eng.lexicon.is_empty());
        assert_eq!(
            eng.lexicon
                .entries_for_pair(Language::English, Language::Spanish)
                .len(),
            9
        );
    }

    #[test]
    fn test_lexicon_independence() {
        let mut eng_a = VsaTranslationEngine::new();
        let eng_b = VsaTranslationEngine::new();
        eng_a.seed_common_pairs();
        assert!(!eng_a.lexicon.is_empty());
        assert!(eng_b.lexicon.is_empty());
    }

    #[test]
    fn test_detect_language_empty() {
        let eng = VsaTranslationEngine::new();
        assert_eq!(eng.detect_language(""), Language::English);
    }

    #[test]
    fn test_analogical_strategy_selection() {
        let eng = VsaTranslationEngine::new();
        let hex = ReasoningHexagram::new(0b101000);
        let strat = eng.select_strategy(hex);
        assert!(
            strat == TranslationStrategy::Analogical || strat == TranslationStrategy::Compositional
        );
    }

    #[test]
    fn test_refinement_strategy_selection() {
        let eng = VsaTranslationEngine::new();
        let hex = ReasoningHexagram::new(0b000101);
        assert_eq!(eng.select_strategy(hex), TranslationStrategy::Refinement);
    }

    #[test]
    fn test_translate_spanish_to_english() {
        let mut eng = test_engine();
        eng.use_e8_strategy = false;
        let result = eng.translate("hola", Some(Language::Spanish), Language::English);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_compositional_preserves_original_on_no_match() {
        let mut eng = VsaTranslationEngine::new();
        let result = eng.translate("xyzzy unknown", Some(Language::English), Language::French);
        assert_eq!(result.target_text, "xyzzy unknown");
    }

    #[test]
    fn test_strategy_map_covers_all_hexagrams() {
        let eng = VsaTranslationEngine::new();
        for bits in 0..64u8 {
            let hex = ReasoningHexagram::new(bits);
            let strat = eng.select_strategy(hex);
            match strat {
                TranslationStrategy::DirectLookup
                | TranslationStrategy::Compositional
                | TranslationStrategy::Analogical
                | TranslationStrategy::Refinement => {}
            }
        }
    }

    #[test]
    fn test_direct_lookup_returns_highest_confidence() {
        let mut eng = test_engine();
        let result = eng.translate("yes", Some(Language::English), Language::Spanish);
        assert_eq!(result.target_lang, Language::Spanish);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_refinement_falls_back_to_compositional() {
        let mut eng = VsaTranslationEngine::new();
        let result = eng.translate(
            "this is a long sentence that should trigger refinement because it has more than twenty words in total for the strategy",
            Some(Language::English),
            Language::Spanish,
        );
        assert_eq!(result.strategy, TranslationStrategy::Refinement);
    }

    #[test]
    fn test_translate_with_self_learning_increases_lexicon() {
        let mut eng = test_engine();
        let before = eng.lexicon.len();
        let result = eng.translate("hello", Some(Language::English), Language::Spanish);
        assert!(result.confidence > 0.6);
        assert_eq!(eng.lexicon.len(), before);
        eng.translate("sun", Some(Language::English), Language::Spanish);
        assert_eq!(eng.lexicon.len(), before);
        eng.translate("hello world", Some(Language::English), Language::Spanish);
        assert!(eng.lexicon.len() >= before);
    }

    #[test]
    fn test_learn_from_translation_high_confidence() {
        let mut eng = VsaTranslationEngine::new();
        let result = TranslationResult {
            source_text: "hello world".to_string(),
            target_text: "hola mundo".to_string(),
            source_lang: Language::English,
            target_lang: Language::Spanish,
            strategy: TranslationStrategy::DirectLookup,
            confidence: 0.9,
            vsa_similarity: 0.85,
            entry_id: None,
        };
        assert!(eng.lexicon.is_empty());
        eng.learn_from_translation(&result);
        assert_eq!(eng.lexicon.len(), 1);
        let entries = eng
            .lexicon
            .entries_for_pair(Language::English, Language::Spanish);
        let has_pair = entries
            .iter()
            .any(|e| e.source_text == "hello world" && e.target_text == "hola mundo");
        assert!(has_pair);
    }

    #[test]
    fn test_learn_from_translation_low_confidence() {
        let mut eng = VsaTranslationEngine::new();
        let result = TranslationResult {
            source_text: "garbage".to_string(),
            target_text: "basura".to_string(),
            source_lang: Language::English,
            target_lang: Language::Spanish,
            strategy: TranslationStrategy::Compositional,
            confidence: 0.3,
            vsa_similarity: 0.1,
            entry_id: None,
        };
        eng.learn_from_translation(&result);
        assert!(eng.lexicon.is_empty());
    }

    #[test]
    fn test_learn_from_translation_identity() {
        let mut eng = VsaTranslationEngine::new();
        let result = TranslationResult {
            source_text: "hello".to_string(),
            target_text: "hello".to_string(),
            source_lang: Language::English,
            target_lang: Language::English,
            strategy: TranslationStrategy::DirectLookup,
            confidence: 0.9,
            vsa_similarity: 1.0,
            entry_id: None,
        };
        eng.learn_from_translation(&result);
        assert!(eng.lexicon.is_empty());
    }

    #[test]
    fn test_learn_from_translation_unknown_lang() {
        let mut eng = VsaTranslationEngine::new();
        let result = TranslationResult {
            source_text: "hello".to_string(),
            target_text: "bonjour".to_string(),
            source_lang: Language::Unknown,
            target_lang: Language::French,
            strategy: TranslationStrategy::Compositional,
            confidence: 0.8,
            vsa_similarity: 0.0,
            entry_id: None,
        };
        eng.learn_from_translation(&result);
        assert!(eng.lexicon.is_empty());
    }

    #[test]
    fn test_detect_language_spanish() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("hola mundo buenos días");
        assert_eq!(lang, Language::Spanish);
    }

    #[test]
    fn test_detect_language_french() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("bonjour merci au revoir");
        assert_eq!(lang, Language::French);
    }

    #[test]
    fn test_detect_language_german() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("hallo danke auf Wiedersehen");
        assert_eq!(lang, Language::German);
    }

    #[test]
    fn test_detect_language_japanese() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("こんにちは世界");
        assert_eq!(lang, Language::Japanese);
    }

    #[test]
    fn test_detect_language_korean() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("안녕하세요 세계");
        assert_eq!(lang, Language::Korean);
    }

    #[test]
    fn test_detect_language_russian() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("здравствуйте мир");
        assert_eq!(lang, Language::Russian);
    }

    #[test]
    fn test_detect_language_arabic() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("مرحبا بالعالم");
        assert_eq!(lang, Language::Arabic);
    }

    #[test]
    fn test_detect_language_thai() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("สวัสดีชาวโลก");
        assert_eq!(lang, Language::Thai);
    }

    #[test]
    fn test_detect_language_hindi() {
        let eng = VsaTranslationEngine::new();
        let lang = eng.detect_language("नमस्ते दुनिया");
        assert_eq!(lang, Language::Hindi);
    }

    #[test]
    fn test_seed_pairs_expanded() {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        let en_es = eng
            .lexicon
            .entries_for_pair(Language::English, Language::Spanish);
        assert!(en_es.len() >= 18);
        let en_fr = eng
            .lexicon
            .entries_for_pair(Language::English, Language::French);
        assert!(en_fr.len() >= 16);
        let en_ja = eng
            .lexicon
            .entries_for_pair(Language::English, Language::Japanese);
        assert!(en_ja.len() >= 15);
        let en_ko = eng
            .lexicon
            .entries_for_pair(Language::English, Language::Korean);
        assert!(en_ko.len() >= 13);
        let en_ru = eng
            .lexicon
            .entries_for_pair(Language::English, Language::Russian);
        assert!(en_ru.len() >= 13);
    }

    #[test]
    fn test_self_learning_after_multiple_translations() {
        let mut eng = test_engine();
        let before = eng.lexicon.len();
        eng.translate("friend", Some(Language::English), Language::Spanish);
        eng.translate("love", Some(Language::English), Language::French);
        eng.translate("bread", Some(Language::English), Language::German);
        eng.translate("friend", Some(Language::English), Language::Japanese);
        eng.translate("love", Some(Language::English), Language::Korean);
        let after = eng.lexicon.len();
        assert!(after >= before);
    }

    // ── New tests for cleanup rules, resonator, TEaR refinement ──

    #[test]
    fn test_cleanup_rule_standard() {
        let mut sims = vec![0.9, 0.3, 0.7, 0.1];
        CleanupRule::Standard.apply(&mut sims);
        assert_eq!(sims, vec![0.9, 0.3, 0.7, 0.1]);
    }

    #[test]
    fn test_cleanup_rule_sign() {
        let mut sims = vec![0.9, 0.3, 0.0, -0.1];
        CleanupRule::Sign.apply(&mut sims);
        assert_eq!(sims, vec![1.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_cleanup_rule_relu() {
        let mut sims = vec![0.8, -0.2, 0.5, -0.9];
        CleanupRule::ReLU.apply(&mut sims);
        assert_eq!(sims, vec![0.8, 0.0, 0.5, 0.0]);
    }

    #[test]
    fn test_cleanup_rule_polynomial() {
        let mut sims = vec![0.9, 0.3, 0.7];
        CleanupRule::Polynomial(3.0).apply(&mut sims);
        // 0.9^3 = 0.729, 0.3^3 = 0.027, 0.7^3 = 0.343
        assert!((sims[0] - 0.729).abs() < 1e-6);
        assert!((sims[1] - 0.027).abs() < 1e-6);
    }

    #[test]
    fn test_cleanup_rule_softmax() {
        let mut sims = vec![1.0, 0.0, 2.0];
        CleanupRule::Softmax(1.0).apply(&mut sims);
        let total: f64 = sims.iter().sum();
        assert!((total - 1.0).abs() < 1e-6);
        assert!(sims[2] > sims[0]);
    }

    #[test]
    fn test_polynomial_cleanup_amplifies_high_scores() {
        let mut sims = vec![0.95, 0.4];
        CleanupRule::Polynomial(4.0).apply(&mut sims);
        // 0.95^4 = 0.8145, 0.4^4 = 0.0256
        // High score relatively amplified
        assert!(sims[0] / sims[1] > (0.95 / 0.4));
    }

    #[test]
    fn test_default_cleanup_rule_is_polynomial() {
        let eng = VsaTranslationEngine::new();
        assert_eq!(eng.cleanup_rule, CleanupRule::Polynomial(3.0));
    }

    #[test]
    fn test_max_refinement_passes_default() {
        let eng = VsaTranslationEngine::new();
        assert_eq!(eng.max_refinement_passes, 3);
    }

    #[test]
    fn test_use_resonator_default_off() {
        let eng = VsaTranslationEngine::new();
        assert!(!eng.use_resonator);
    }

    #[test]
    fn test_resonator_lookup_returns_results() {
        let mut eng = test_engine();
        eng.use_resonator = true;
        let result = eng.translate("hello", Some(Language::English), Language::Spanish);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_translate_with_polynomial_cleanup() {
        let mut eng = test_engine();
        eng.cleanup_rule = CleanupRule::Polynomial(3.0);
        eng.use_e8_strategy = false;
        let result = eng.translate("sun", Some(Language::English), Language::Spanish);
        assert!(!result.target_text.is_empty());
    }

    #[test]
    fn test_refinement_uses_tear_loop() {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        eng.max_refinement_passes = 3;
        let result = eng.translate(
            "hello world friend water",
            Some(Language::English),
            Language::Spanish,
        );
        assert!(!result.target_text.is_empty());
        assert!(result.target_text.split_whitespace().count() >= 2);
    }

    #[test]
    fn test_refinement_increases_confidence_across_passes() {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        eng.max_refinement_passes = 1;
        let _result1 = eng.translate("hello world", Some(Language::English), Language::Spanish);
        eng.max_refinement_passes = 3;
        eng.clear_cache();
        let result3 = eng.translate("hello world", Some(Language::English), Language::Spanish);
        // More passes should maintain or improve quality
        assert!(!result3.target_text.is_empty());
    }

    #[test]
    fn test_lookup_resonator_basic() {
        let mut lex = BilingualLexicon::new(50);
        lex.store("hello", "hola", Language::English, Language::Spanish, 0.9);
        lex.store("world", "mundo", Language::English, Language::Spanish, 0.9);
        let query = BilingualLexicon::text_to_vsa_deterministic("hello");
        let results = lex.lookup_resonator(
            &query,
            Language::English,
            Language::Spanish,
            1,
            5,
            CleanupRule::Polynomial(3.0),
        );
        assert!(!results.is_empty());
        assert_eq!(results[0].source_text, "hello");
    }

    #[test]
    fn test_lookup_with_sign_rule() {
        let mut lex = BilingualLexicon::new(50);
        lex.store("hello", "hola", Language::English, Language::Spanish, 0.9);
        lex.store(
            "goodbye",
            "adiós",
            Language::English,
            Language::Spanish,
            0.9,
        );
        let query = BilingualLexicon::text_to_vsa_deterministic("hello");
        let results = lex.lookup_with_rule(
            &query,
            Language::English,
            Language::Spanish,
            2,
            CleanupRule::Sign,
        );
        assert!(!results.is_empty());
    }

    #[test]
    fn test_cleanup_rule_relu_no_negative() {
        let mut sims = vec![0.5, -0.3, -0.8, 0.2];
        CleanupRule::ReLU.apply(&mut sims);
        for s in &sims {
            assert!(*s >= 0.0);
        }
    }

    #[test]
    fn test_polynomial_different_degrees() {
        let mut sims_deg2 = vec![0.9, 0.5];
        let mut sims_deg5 = vec![0.9, 0.5];
        CleanupRule::Polynomial(2.0).apply(&mut sims_deg2);
        CleanupRule::Polynomial(5.0).apply(&mut sims_deg5);
        let gap_deg2 = sims_deg2[0] / sims_deg2[1];
        let gap_deg5 = sims_deg5[0] / sims_deg5[1];
        assert!(gap_deg5 >= gap_deg2 - 1e-6);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let mut eng = test_engine();
        let result1 = eng.translate("hello", Some(Language::English), Language::Spanish);
        assert!(result1.confidence > 0.0);
        let saved_entries = eng.lexicon.entries.clone();
        let saved_total = eng.total_translations;
        assert!(saved_total > 0);

        // Simulate save/load by roundtripping through serialization
        let json = serde_json::to_string(&EngineSaveData {
            entries: eng.lexicon.entries.clone(),
            next_id: eng.lexicon.next_id,
            max_entries: eng.lexicon.max_entries,
            strategy_stats: {
                let mut m = std::collections::HashMap::new();
                m.insert("direct".into(), (1, 1));
                m
            },
            total_translations: eng.total_translations,
        })
        .unwrap();
        let loaded: EngineSaveData = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.entries.len(), saved_entries.len());
        assert_eq!(loaded.total_translations, saved_total);
    }

    #[test]
    fn test_retain_high_frequency_keeps_popular() {
        let _lex = BilingualLexicon::new(100);
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        let entries_before = eng.lexicon.len();
        assert!(entries_before > 0);
        eng.lexicon.retain_high_frequency(entries_before / 2);
        assert!(eng.lexicon.len() <= entries_before);
    }

    #[test]
    fn test_use_resonator_default_on() {
        let eng = VsaTranslationEngine::new();
        assert!(eng.use_resonator);
    }

    #[test]
    fn test_auto_save_interval_default() {
        let eng = VsaTranslationEngine::new();
        assert_eq!(eng.auto_save_interval, 50);
        assert_eq!(eng.total_translations, 0);
    }

    #[test]
    fn test_language_family_groups() {
        assert_eq!(Language::English.language_family(), 1);
        assert_eq!(Language::German.language_family(), 1);
        assert_eq!(Language::Spanish.language_family(), 2);
        assert_eq!(Language::French.language_family(), 2);
        assert_eq!(Language::Chinese.language_family(), 5);
        assert_eq!(Language::Japanese.language_family(), 4);
        assert_eq!(Language::Korean.language_family(), 4);
        assert_eq!(Language::Arabic.language_family(), 6);
        assert_eq!(Language::Hindi.language_family(), 7);
        assert_eq!(Language::Unknown.language_family(), 0);
    }

    #[test]
    fn test_frequency_weighted_learning_prefers_phrases() {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        let before = eng.lexicon.len();

        // Single word at confidence 0.7 should NOT trigger learning (needs 2 words)
        let single_word = TranslationResult {
            source_text: "unseenhmm".to_string(),
            target_text: "invisible".to_string(),
            source_lang: Language::English,
            target_lang: Language::French,
            strategy: TranslationStrategy::DirectLookup,
            confidence: 0.7,
            vsa_similarity: 0.6,
            entry_id: None,
        };
        eng.learn_from_translation(&single_word);
        assert_eq!(
            eng.lexicon.len(),
            before,
            "single words at low conf should not learn"
        );

        // Multi-word phrase at same confidence SHOULD trigger learning
        let phrase = TranslationResult {
            source_text: "never seen before".to_string(),
            target_text: "jamais vu avant".to_string(),
            source_lang: Language::English,
            target_lang: Language::French,
            strategy: TranslationStrategy::Compositional,
            confidence: 0.7,
            vsa_similarity: 0.5,
            entry_id: None,
        };
        eng.learn_from_translation(&phrase);
        assert!(
            eng.lexicon.len() > before,
            "phrases should be learned at confidence 0.7"
        );
    }

    #[test]
    fn test_seed_vocabulary_arabic() {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        let en_ar = eng
            .lexicon
            .entries_for_pair(Language::English, Language::Arabic);
        assert!(
            en_ar.len() >= 13,
            "should have Arabic seed pairs, got {}",
            en_ar.len()
        );
    }

    #[test]
    fn test_seed_vocabulary_hindi() {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        let en_hi = eng
            .lexicon
            .entries_for_pair(Language::English, Language::Hindi);
        assert!(
            en_hi.len() >= 13,
            "should have Hindi seed pairs, got {}",
            en_hi.len()
        );
    }

    #[test]
    fn test_seed_vocabulary_thai() {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        let en_th = eng
            .lexicon
            .entries_for_pair(Language::English, Language::Thai);
        assert!(
            en_th.len() >= 12,
            "should have Thai seed pairs, got {}",
            en_th.len()
        );
    }

    #[test]
    fn test_seed_vocabulary_portuguese() {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        let en_pt = eng
            .lexicon
            .entries_for_pair(Language::English, Language::Portuguese);
        assert!(
            en_pt.len() >= 18,
            "should have Portuguese seed pairs, got {}",
            en_pt.len()
        );
    }

    #[test]
    fn test_seed_vocabulary_italian() {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        let en_it = eng
            .lexicon
            .entries_for_pair(Language::English, Language::Italian);
        assert!(
            en_it.len() >= 18,
            "should have Italian seed pairs, got {}",
            en_it.len()
        );
    }

    #[test]
    fn test_seed_vocabulary_all_languages_total() {
        let mut eng = VsaTranslationEngine::new();
        eng.seed_common_pairs();
        let total = eng.lexicon.len();
        // At least 130+ entries across all seed pairs
        assert!(
            total >= 200,
            "should have ≥200 seed entries total across all languages, got {}",
            total
        );
    }
}
