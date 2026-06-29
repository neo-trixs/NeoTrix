use std::collections::HashMap;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_translate::bilingual::BilingualLexicon;
use crate::core::nt_core_translate::language::Language;
use crate::core::nt_core_translate::translate_engine::translate_types::TranslationStrategy;
use crate::core::nt_core_translate::translate_engine::VsaTranslationEngine;

impl VsaTranslationEngine {
    pub fn select_strategy(&self, hexagram: ReasoningHexagram) -> TranslationStrategy {
        let bits = hexagram.0;
        for &(mask, ref strat) in &self.strategy_map {
            if bits & 0b101100 == mask & 0b101100 {
                let diff = bits ^ mask;
                let relevant = diff & 0b101100;
                if relevant.count_ones() <= 1 {
                    return *strat;
                }
            }
        }
        if hexagram.depth() == 1 {
            return TranslationStrategy::Refinement;
        }
        if hexagram.method() == 1 {
            return TranslationStrategy::Compositional;
        }
        if hexagram.abstraction() == 1 {
            return TranslationStrategy::Analogical;
        }
        TranslationStrategy::DirectLookup
    }

    pub fn select_hexagram(&self, text: &str, source_lang: Language) -> ReasoningHexagram {
        let word_count = text.split_whitespace().filter(|w| !w.is_empty()).count();

        if source_lang == Language::Chinese {
            return ReasoningHexagram::new(0b001000);
        }

        if source_lang == Language::English && word_count >= 5 && word_count < 20 {
            return ReasoningHexagram::new(0b100000);
        }

        if word_count < 5 {
            ReasoningHexagram::new(0b000000)
        } else if word_count < 20 {
            ReasoningHexagram::new(0b001000)
        } else {
            ReasoningHexagram::new(0b000100)
        }
    }

    pub fn adapt_strategy(
        &self,
        base_strategy: TranslationStrategy,
        source_lang: Language,
        target_lang: Language,
    ) -> TranslationStrategy {
        let pair_key = (source_lang, target_lang);
        let Some(log) = self.lang_pair_stats.get(&pair_key) else {
            return base_strategy;
        };
        if log.len() < 5 {
            return base_strategy;
        }

        let mut accuracy: HashMap<TranslationStrategy, f64> = HashMap::new();
        let mut count: HashMap<TranslationStrategy, usize> = HashMap::new();
        for &(strat, success) in log.iter().rev().take(20) {
            let c = count.entry(strat).or_insert(0);
            *c += 1;
            if success {
                *accuracy.entry(strat).or_insert(0.0) += 1.0;
            }
        }
        for (strat, total) in &count {
            if *total > 0 {
                let acc = accuracy.get(strat).copied().unwrap_or(0.0) / *total as f64;
                accuracy.insert(*strat, acc);
            }
        }

        let base_acc = accuracy.get(&base_strategy).copied().unwrap_or(0.5);
        if base_acc < 0.3 {
            if let Some((best, _)) = accuracy
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                if *best != base_strategy {
                    return *best;
                }
            }
        }

        base_strategy
    }

    pub fn translate_direct(
        &self,
        query_vsa: &[u8],
        source_lang: Language,
        target_lang: Language,
    ) -> Option<(
        crate::core::nt_core_translate::bilingual::BilingualEntry,
        f64,
    )> {
        if self.use_resonator && self.lexicon.len() > 5 {
            let results = self.lexicon.lookup_resonator(
                query_vsa,
                source_lang,
                target_lang,
                1,
                5,
                self.cleanup_rule,
            );
            if let Some(entry) = results.first() {
                let sim = QuantizedVSA::similarity(query_vsa, &entry.source_vsa);
                if sim >= self.min_similarity {
                    return Some((entry.clone(), sim));
                }
            }
            return None;
        }

        let pair_entries = self.lexicon.entries_for_pair(source_lang, target_lang);
        if pair_entries.is_empty() {
            return None;
        }

        let bindings: Vec<Vec<u8>> = pair_entries
            .iter()
            .map(|e| QuantizedVSA::bind(&e.source_vsa, &e.target_vsa))
            .collect();
        let binding_refs: Vec<&[u8]> = bindings.iter().map(|v| v.as_slice()).collect();
        let tm = QuantizedVSA::bundle(&binding_refs);
        let noisy = QuantizedVSA::unbind(&tm, query_vsa);

        let mut best_sim = 0.0_f64;
        let mut best_entry: Option<usize> = None;
        for (i, entry) in pair_entries.iter().enumerate() {
            let sim = QuantizedVSA::similarity(&noisy, &entry.target_vsa);
            if sim > best_sim {
                best_sim = sim;
                best_entry = Some(i);
            }
        }

        if let Some(idx) = best_entry {
            if best_sim >= self.min_similarity {
                return Some((pair_entries[idx].clone(), best_sim));
            }
        }

        let mut fallback_sim = 0.0_f64;
        let mut fallback_entry: Option<usize> = None;
        for (i, entry) in pair_entries.iter().enumerate() {
            let sim = QuantizedVSA::similarity(query_vsa, &entry.source_vsa);
            if sim > fallback_sim {
                fallback_sim = sim;
                fallback_entry = Some(i);
            }
        }

        if let Some(idx) = fallback_entry {
            if fallback_sim >= self.min_similarity {
                return Some((pair_entries[idx].clone(), fallback_sim));
            }
        }

        None
    }

    pub fn translate_compositional(
        &mut self,
        text: &str,
        source_lang: Language,
        target_lang: Language,
    ) -> String {
        let words: Vec<&str> = text.split_whitespace().filter(|w| !w.is_empty()).collect();
        let mut translated: Vec<String> = Vec::with_capacity(words.len());

        for word in &words {
            let word_vsa = BilingualLexicon::text_to_vsa_deterministic(word);
            let pair_entries = self.lexicon.entries_for_pair(source_lang, target_lang);
            let mut best_sim = 0.0_f64;
            let mut best_target: Option<String> = None;

            for entry in &pair_entries {
                let sim = QuantizedVSA::similarity(&word_vsa, &entry.source_vsa);
                if sim > best_sim && sim >= self.min_similarity {
                    best_sim = sim;
                    best_target = Some(entry.target_text.clone());
                }
            }

            let punct: String = word.chars().filter(|c| c.is_ascii_punctuation()).collect();
            let clean: String = word.chars().filter(|c| !c.is_ascii_punctuation()).collect();
            if best_target.is_none() && !clean.is_empty() {
                let clean_vsa = BilingualLexicon::text_to_vsa_deterministic(&clean);
                for entry in &pair_entries {
                    let sim = QuantizedVSA::similarity(&clean_vsa, &entry.source_vsa);
                    if sim > best_sim && sim >= self.min_similarity {
                        best_sim = sim;
                        let mut t = entry.target_text.clone();
                        if !punct.is_empty() {
                            t.push_str(&punct);
                        }
                        best_target = Some(t);
                    }
                }
            }

            translated.push(best_target.unwrap_or_else(|| word.to_string()));
        }

        translated.join(" ")
    }

    pub fn translate_analogical(
        &self,
        query_vsa: &[u8],
        source_lang: Language,
        target_lang: Language,
    ) -> Option<(
        crate::core::nt_core_translate::bilingual::BilingualEntry,
        f64,
    )> {
        let pair_entries = self.lexicon.entries_for_pair(source_lang, target_lang);
        if pair_entries.len() < 2 {
            return None;
        }

        let mut best_sim = 0.0_f64;
        let mut best_entry: Option<crate::core::nt_core_translate::bilingual::BilingualEntry> =
            None;

        for i in 0..pair_entries.len() {
            for j in 0..pair_entries.len() {
                if i == j {
                    continue;
                }
                let a_src = &pair_entries[i].source_vsa;
                let a_tgt = &pair_entries[i].target_vsa;
                let b_src = &pair_entries[j].source_vsa;
                let b_tgt = &pair_entries[j].target_vsa;

                let ab = QuantizedVSA::bind(a_src, a_tgt);
                let mapping = QuantizedVSA::unbind(&ab, b_src);
                let analog_target = QuantizedVSA::bind(&mapping, query_vsa);

                let target_sim = QuantizedVSA::similarity(&analog_target, b_tgt);
                if target_sim > best_sim {
                    let candidate_sim = QuantizedVSA::similarity(query_vsa, a_src);
                    if candidate_sim > 0.4 {
                        best_sim = target_sim;
                        if target_sim >= self.min_similarity {
                            best_entry = Some(pair_entries[j].clone());
                        }
                    }
                }
            }
        }

        best_entry.map(|e| (e, best_sim))
    }

    pub fn translate_refinement(
        &mut self,
        text: &str,
        source_lang: Language,
        target_lang: Language,
    ) -> String {
        let mut current = self.translate_compositional(text, source_lang, target_lang);
        let mut best_confidence = 0.0_f64;
        let mut best_result = current.clone();

        for pass in 0..self.max_refinement_passes {
            let words: Vec<&str> = current.split_whitespace().collect();
            let source_words: Vec<&str> = text.split_whitespace().collect();
            let pair_entries = self.lexicon.entries_for_pair(source_lang, target_lang);

            if pair_entries.is_empty() {
                break;
            }

            let mut word_scores: Vec<(String, f64)> = Vec::new();
            let mut all_confident = true;

            for (word_idx, word) in words.iter().enumerate() {
                let src_word = source_words
                    .get(word_idx)
                    .or_else(|| source_words.last())
                    .unwrap_or(word);

                let src_vsa = BilingualLexicon::text_to_vsa_deterministic(src_word);
                let word_vsa = BilingualLexicon::text_to_vsa_deterministic(word);

                let mut best_sim = 0.0_f64;
                let mut best_target: Option<String> = None;

                for entry in &pair_entries {
                    let sim = QuantizedVSA::similarity(&src_vsa, &entry.source_vsa);
                    if sim > best_sim {
                        best_sim = sim;
                        best_target = Some(entry.target_text.clone());
                    }
                }

                let mut target_sim = 0.0_f64;
                for entry in &pair_entries {
                    let sim = QuantizedVSA::similarity(&word_vsa, &entry.target_vsa);
                    if sim > target_sim {
                        target_sim = sim;
                    }
                }

                let estimate = (best_sim + target_sim) / 2.0;

                let (refined_word, conf) = if estimate > self.min_similarity + 0.1 {
                    let word = best_target.unwrap_or_else(|| word.to_string());
                    (word, true)
                } else if pass < self.max_refinement_passes - 1 {
                    let mut alt_sim = 0.0_f64;
                    let mut alt_target = word.to_string();
                    for entry in &pair_entries {
                        let sim = QuantizedVSA::similarity(&src_vsa, &entry.source_vsa);
                        if sim > alt_sim && sim > best_sim - 0.1 {
                            alt_sim = sim;
                            alt_target = entry.target_text.clone();
                        }
                    }
                    (alt_target, false)
                } else {
                    (word.to_string(), false)
                };

                word_scores.push((refined_word, estimate));
                if !conf {
                    all_confident = false;
                }
            }

            let refined: Vec<String> = word_scores.iter().map(|(w, _)| w.clone()).collect();
            current = refined.join(" ");

            if all_confident {
                best_result = current.clone();
                break;
            }

            let avg_score: f64 =
                word_scores.iter().map(|(_, s)| s).sum::<f64>() / word_scores.len() as f64;
            let conf = (avg_score + 0.5).min(0.9);

            if conf > best_confidence {
                best_confidence = conf;
                best_result = current.clone();
            }
        }

        best_result
    }

    pub fn record_outcome(&mut self, strategy: TranslationStrategy, success: bool) {
        let entry = self.strategy_stats.entry(strategy).or_insert((0, 0));
        entry.0 += 1;
        if success {
            entry.1 += 1;
        }
    }

    pub fn strategy_accuracy(&self) -> Vec<(TranslationStrategy, f64)> {
        let mut results: Vec<_> = self
            .strategy_stats
            .iter()
            .map(|(strat, &(attempts, successes))| {
                let acc = if attempts > 0 {
                    successes as f64 / attempts as f64
                } else {
                    0.0
                };
                (*strat, acc)
            })
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}
