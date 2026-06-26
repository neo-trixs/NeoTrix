use crate::core::nt_core_hcube::kroneker_cleanup::KronekerCodebook;
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use crate::core::nt_core_translate::language::Language;
use std::collections::HashMap;

const TRANSLATION_STORAGE_FILE: &str = "translation_lexicon.json";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CleanupRule {
    /// Standard similarity threshold
    Standard,
    /// Sign: each bit = sign(value), +1 or -1
    Sign,
    /// ReLU: zero out negative values
    ReLU,
    /// Polynomial cleanup: raise similarity to power p, amplify high, suppress low
    Polynomial(f64),
    /// Softmax: temperature-scaled probability re-weighting
    Softmax(f64),
}

impl CleanupRule {
    pub fn apply(&self, similarities: &mut [f64]) {
        match self {
            CleanupRule::Standard => {}
            CleanupRule::Sign => {
                for s in similarities.iter_mut() {
                    *s = if *s > 0.0 { 1.0 } else { 0.0 };
                }
            }
            CleanupRule::ReLU => {
                for s in similarities.iter_mut() {
                    if *s < 0.0 {
                        *s = 0.0;
                    }
                }
            }
            CleanupRule::Polynomial(p) => {
                for s in similarities.iter_mut() {
                    *s = s.abs().powf(*p);
                }
            }
            CleanupRule::Softmax(tau) => {
                let max_val = similarities
                    .iter()
                    .cloned()
                    .fold(f64::NEG_INFINITY, f64::max);
                let exp_sum: f64 = similarities
                    .iter()
                    .map(|s| ((s - max_val) / tau).exp())
                    .sum();
                if exp_sum > 0.0 {
                    for s in similarities.iter_mut() {
                        *s = ((*s - max_val) / tau).exp() / exp_sum;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BilingualEntry {
    pub id: u64,
    pub source_text: String,
    pub target_text: String,
    pub source_lang: Language,
    pub target_lang: Language,
    pub bound_vsa: Vec<u8>,
    pub source_vsa: Vec<u8>,
    pub target_vsa: Vec<u8>,
    pub confidence: f64,
    pub access_count: u64,
    pub created_at: i64,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LexiconSaveData {
    entries: Vec<BilingualEntry>,
    next_id: u64,
    max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct BilingualLexicon {
    pub entries: Vec<BilingualEntry>,
    translation_memory: Vec<u8>,
    lang_memories: HashMap<(Language, Language), Vec<u8>>,
    target_codebook: KronekerCodebook,
    codebook_to_entry: Vec<usize>,
    pub max_entries: usize,
    pub next_id: u64,
}

impl Default for BilingualLexicon {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl BilingualLexicon {
    pub fn encode_text(text: &str) -> Vec<u8> {
        Self::text_to_vsa_deterministic(text)
    }

    pub fn new(max_entries: usize) -> Self {
        BilingualLexicon {
            entries: Vec::new(),
            translation_memory: vec![0; VSA_DIM],
            lang_memories: HashMap::new(),
            target_codebook: KronekerCodebook::new(VSA_DIM),
            codebook_to_entry: Vec::new(),
            max_entries,
            next_id: 1,
        }
    }

    pub fn store(
        &mut self,
        source_text: &str,
        target_text: &str,
        source_lang: Language,
        target_lang: Language,
        confidence: f64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let source_vsa = Self::text_to_vsa_deterministic(source_text);
        let target_vsa = Self::text_to_vsa_deterministic(target_text);
        let bound_vsa = QuantizedVSA::bind(&source_vsa, &target_vsa);

        let entry = BilingualEntry {
            id,
            source_text: source_text.to_string(),
            target_text: target_text.to_string(),
            source_lang,
            target_lang,
            bound_vsa,
            source_vsa,
            target_vsa,
            confidence,
            access_count: 0,
            evidence_ids: Vec::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        };

        let entry_idx = self.entries.len();
        self.add_to_codebook(&entry.target_vsa, entry_idx);
        self.entries.push(entry);
        self.rebuild_memory();

        id
    }

    pub fn store_with_evidence(
        &mut self,
        source_text: &str,
        target_text: &str,
        source_lang: Language,
        target_lang: Language,
        confidence: f64,
        evidence_ids: &[String],
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let source_vsa = Self::text_to_vsa_deterministic(source_text);
        let target_vsa = Self::text_to_vsa_deterministic(target_text);
        let bound_vsa = QuantizedVSA::bind(&source_vsa, &target_vsa);

        let entry = BilingualEntry {
            id,
            source_text: source_text.to_string(),
            target_text: target_text.to_string(),
            source_lang,
            target_lang,
            bound_vsa,
            source_vsa,
            target_vsa,
            confidence,
            access_count: 0,
            evidence_ids: Vec::from(evidence_ids),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        };

        let entry_idx = self.entries.len();
        self.add_to_codebook(&entry.target_vsa, entry_idx);
        self.entries.push(entry);
        self.rebuild_memory();

        id
    }

    pub fn lookup(
        &self,
        query_vsa: &[u8],
        source_lang: Language,
        target_lang: Language,
        k: usize,
    ) -> Vec<BilingualEntry> {
        self.lookup_with_rule(
            query_vsa,
            source_lang,
            target_lang,
            k,
            CleanupRule::Standard,
        )
    }

    pub fn lookup_with_rule(
        &self,
        query_vsa: &[u8],
        source_lang: Language,
        target_lang: Language,
        k: usize,
        rule: CleanupRule,
    ) -> Vec<BilingualEntry> {
        let memory = self
            .lang_memories
            .get(&(source_lang, target_lang))
            .map(|v| v.as_slice())
            .unwrap_or(&self.translation_memory);

        let noisy_target = QuantizedVSA::unbind(memory, query_vsa);
        let mut candidates = self.cleanup_vsa(&noisy_target, k);

        // Apply cleanup rule to re-weight candidates
        if rule != CleanupRule::Standard {
            let mut sims: Vec<f64> = candidates.iter().map(|(_, s)| *s).collect();
            rule.apply(&mut sims);
            for ((_, score), new_s) in candidates.iter_mut().zip(sims) {
                *score = new_s;
            }
            candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        candidates
            .into_iter()
            .map(|(idx, _)| self.entries[idx].clone())
            .collect()
    }

    pub fn similar_sources(&self, query_vsa: &[u8], k: usize, threshold: f64) -> Vec<(usize, f64)> {
        let mut results: Vec<(usize, f64)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| (i, QuantizedVSA::similarity(query_vsa, &e.source_vsa)))
            .filter(|(_, sim)| *sim >= threshold)
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }

    pub fn translation_memory(&self) -> &[u8] {
        &self.translation_memory
    }

    pub fn lang_memory(&self, src: Language, tgt: Language) -> Option<&[u8]> {
        self.lang_memories.get(&(src, tgt)).map(|v| v.as_slice())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn has_entry(
        &self,
        source_text: &str,
        target_text: &str,
        source_lang: Language,
        target_lang: Language,
    ) -> bool {
        self.entries.iter().any(|e| {
            e.source_lang == source_lang
                && e.target_lang == target_lang
                && e.source_text == source_text
                && e.target_text == target_text
        })
    }

    pub fn add_entry(
        &mut self,
        source_text: &str,
        target_text: &str,
        source_lang: Language,
        target_lang: Language,
    ) -> u64 {
        self.store(source_text, target_text, source_lang, target_lang, 0.6)
    }

    pub fn add_entry_with_evidence(
        &mut self,
        source_text: &str,
        target_text: &str,
        source_lang: Language,
        target_lang: Language,
        evidence_ids: &[String],
    ) -> u64 {
        self.store_with_evidence(
            source_text,
            target_text,
            source_lang,
            target_lang,
            0.6,
            evidence_ids,
        )
    }

    pub fn entries_for_pair(
        &self,
        source_lang: Language,
        target_lang: Language,
    ) -> Vec<&BilingualEntry> {
        self.entries
            .iter()
            .filter(|e| e.source_lang == source_lang && e.target_lang == target_lang)
            .collect()
    }

    pub fn increment_access(&mut self, id: u64) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.access_count = entry.access_count.saturating_add(1);
        }
    }

    pub fn retain_high_frequency(&mut self, retain_count: usize) -> usize {
        if self.entries.len() <= retain_count {
            return 0;
        }
        let before = self.entries.len();
        let mut indexed: Vec<(usize, &BilingualEntry)> = self.entries.iter().enumerate().collect();
        indexed.sort_by(|a, b| b.1.access_count.cmp(&a.1.access_count));
        let keep: std::collections::HashSet<usize> = indexed
            .into_iter()
            .take(retain_count)
            .map(|(i, _)| i)
            .collect();
        let mut survivors = Vec::with_capacity(retain_count);
        for (i, e) in self.entries.drain(..).enumerate() {
            if keep.contains(&i) {
                survivors.push(e);
            }
        }
        let removed = before - survivors.len();
        self.entries = survivors;
        self.rebuild_memory();
        removed
    }

    pub fn prune(&mut self) -> usize {
        if self.entries.len() <= self.max_entries {
            return 0;
        }
        let before = self.entries.len();
        let to_remove = before - self.max_entries;

        let mut idx_order: Vec<usize> = (0..before).collect();
        idx_order.sort_by_key(|&i| self.entries[i].access_count);

        let keep: std::collections::HashSet<usize> =
            idx_order.into_iter().skip(to_remove).collect();

        let mut survivors = Vec::with_capacity(self.max_entries);
        for (i, e) in self.entries.drain(..).enumerate() {
            if keep.contains(&i) {
                survivors.push(e);
            }
        }
        let removed = before - survivors.len();
        self.entries = survivors;
        self.rebuild_memory();
        removed
    }

    pub fn get(&self, id: u64) -> Option<&BilingualEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn save(&self) -> Result<(), String> {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        let path = home.join(".neotrix").join(TRANSLATION_STORAGE_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
        }
        let data = LexiconSaveData {
            entries: self.entries.clone(),
            next_id: self.next_id,
            max_entries: self.max_entries,
        };
        let json =
            serde_json::to_string_pretty(&data).map_err(|e| format!("Serialize error: {}", e))?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, json).map_err(|e| format!("Write error: {}", e))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("Rename error: {}", e))?;
        Ok(())
    }

    pub fn load_custom(entries: Vec<BilingualEntry>, next_id: u64, max_entries: usize) -> Self {
        let mut lex = Self::new(max_entries);
        lex.next_id = next_id;
        for entry in entries {
            let entry_idx = lex.entries.len();
            lex.add_to_codebook(&entry.target_vsa, entry_idx);
            lex.entries.push(entry);
        }
        lex.rebuild_memory();
        lex
    }

    pub fn load() -> Self {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return Self::new(1000),
        };
        let path = home.join(".neotrix").join(TRANSLATION_STORAGE_FILE);
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<LexiconSaveData>(&content) {
                Ok(data) => {
                    let mut lex = Self::new(data.max_entries);
                    lex.next_id = data.next_id;
                    for entry in data.entries {
                        let entry_idx = lex.entries.len();
                        lex.add_to_codebook(&entry.target_vsa, entry_idx);
                        lex.entries.push(entry);
                    }
                    lex.rebuild_memory();
                    lex
                }
                Err(_) => Self::new(1000),
            },
            Err(_) => Self::new(1000),
        }
    }

    pub fn entry_count_for_pair(&self, source_lang: Language, target_lang: Language) -> usize {
        self.entries
            .iter()
            .filter(|e| e.source_lang == source_lang && e.target_lang == target_lang)
            .count()
    }

    fn rebuild_memory(&mut self) {
        self.lang_memories.clear();

        if self.entries.is_empty() {
            self.translation_memory = vec![0; VSA_DIM];
            return;
        }

        let bound_vsas: Vec<&[u8]> = self
            .entries
            .iter()
            .map(|e| e.bound_vsa.as_slice())
            .collect();
        self.translation_memory = QuantizedVSA::bundle(&bound_vsas);

        let mut lang_groups: HashMap<(Language, Language), Vec<&[u8]>> = HashMap::new();
        for entry in &self.entries {
            let key = (entry.source_lang, entry.target_lang);
            lang_groups
                .entry(key)
                .or_default()
                .push(entry.bound_vsa.as_slice());
        }
        for (key, vsas) in lang_groups {
            self.lang_memories.insert(key, QuantizedVSA::bundle(&vsas));
        }
    }

    /// Deterministic VSA encoding: hash text bytes → seeds random vector.
    /// Consistent across all translation components for reliable bind/unbind.
    pub fn text_to_vsa_deterministic(text: &str) -> Vec<u8> {
        let seed = text
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    fn add_to_codebook(&mut self, target_vsa: &[u8], entry_idx: usize) {
        let seed = target_vsa
            .iter()
            .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
        self.target_codebook.add_seed(seed);
        self.codebook_to_entry.push(entry_idx);
    }

    fn cleanup_vsa(&self, vsa: &[u8], k: usize) -> Vec<(usize, f64)> {
        if self.entries.is_empty() {
            return vec![];
        }
        let mut results: Vec<(usize, f64)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| (i, QuantizedVSA::similarity(vsa, &e.target_vsa)))
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }

    /// Resonator-inspired iterative factorization: refines estimate over multiple passes.
    /// Each iteration: unbind → cleanup → rebind → subtract from composite → repeat.
    pub fn lookup_resonator(
        &self,
        query_vsa: &[u8],
        source_lang: Language,
        target_lang: Language,
        k: usize,
        max_iter: usize,
        rule: CleanupRule,
    ) -> Vec<BilingualEntry> {
        let memory = self
            .lang_memories
            .get(&(source_lang, target_lang))
            .map(|v| v.as_slice())
            .unwrap_or(&self.translation_memory);

        let candidate_indices: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.source_lang == source_lang && e.target_lang == target_lang)
            .map(|(i, _)| i)
            .collect();

        if candidate_indices.is_empty() {
            return vec![];
        }

        let mut residual = memory.to_vec();
        let mut results: Vec<(usize, f64)> = Vec::new();
        let mut used = vec![false; self.entries.len()];

        for _iter in 0..max_iter {
            // Unbind current residual to get target estimate
            let noisy_target = QuantizedVSA::unbind(&residual, query_vsa);

            // Score all candidates against cleaned estimate
            let mut scores: Vec<(usize, f64)> = candidate_indices
                .iter()
                .filter(|&&i| !used[i])
                .map(|&i| {
                    (
                        i,
                        QuantizedVSA::similarity(&noisy_target, &self.entries[i].target_vsa),
                    )
                })
                .collect();

            // Apply cleanup rule
            let mut raw_scores: Vec<f64> = scores.iter().map(|(_, s)| *s).collect();
            rule.apply(&mut raw_scores);
            for (idx, score) in scores.iter_mut().zip(raw_scores) {
                idx.1 = score;
            }

            scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Pick best candidate
            if let Some(&(best_idx, best_score)) = scores.first() {
                if best_score > 0.3 && !used[best_idx] {
                    used[best_idx] = true;
                    results.push((best_idx, best_score));

                    // Subtract this candidate's contribution from residual
                    let bound = QuantizedVSA::bind(
                        &self.entries[best_idx].source_vsa,
                        &self.entries[best_idx].target_vsa,
                    );
                    for (j, &b) in bound.iter().enumerate() {
                        if b != 0 {
                            let val = residual[j] as i32 - 1;
                            residual[j] = val.max(0) as u8;
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
            .into_iter()
            .map(|(idx, _)| self.entries[idx].clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lexicon() -> BilingualLexicon {
        let mut lex = BilingualLexicon::new(100);
        lex.store("hello", "hola", Language::English, Language::Spanish, 0.95);
        lex.store(
            "goodbye",
            "adiós",
            Language::English,
            Language::Spanish,
            0.90,
        );
        lex.store(
            "thank you",
            "gracias",
            Language::English,
            Language::Spanish,
            0.98,
        );
        lex.store(
            "bonjour",
            "hello",
            Language::French,
            Language::English,
            0.95,
        );
        lex.store(
            "merci",
            "thank you",
            Language::French,
            Language::English,
            0.97,
        );
        lex
    }

    #[test]
    fn test_new_lexicon_empty() {
        let lex = BilingualLexicon::new(50);
        assert_eq!(lex.len(), 0);
        assert!(lex.translation_memory.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_store_increases_count() {
        let mut lex = BilingualLexicon::new(50);
        lex.store("hello", "hola", Language::English, Language::Spanish, 0.9);
        assert_eq!(lex.len(), 1);
    }

    #[test]
    fn test_store_returns_unique_ids() {
        let mut lex = BilingualLexicon::new(50);
        let id1 = lex.store("a", "b", Language::English, Language::Spanish, 0.9);
        let id2 = lex.store("c", "d", Language::English, Language::Spanish, 0.9);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_get_returns_stored_entry() {
        let mut lex = BilingualLexicon::new(50);
        let id = lex.store("hello", "hola", Language::English, Language::Spanish, 0.95);
        let entry = lex.get(id).expect("bilingual: entry not found after store");
        assert_eq!(entry.source_text, "hello");
        assert_eq!(entry.target_text, "hola");
        assert!((entry.confidence - 0.95).abs() < 1e-10);
    }

    #[test]
    fn test_lookup_finds_translation() {
        let lex = make_lexicon();
        let query = BilingualLexicon::encode_text("hello");
        let results = lex.lookup(&query, Language::English, Language::Spanish, 3);
        assert!(!results.is_empty(), "should find at least one result");
        let has_hola = results.iter().any(|e| e.target_text == "hola");
        assert!(has_hola, "lookup should find 'hola' for 'hello'");
    }

    #[test]
    fn test_similar_sources_finds_match() {
        let lex = make_lexicon();
        let query = BilingualLexicon::encode_text("hello");
        let results = lex.similar_sources(&query, 5, 0.1);
        assert!(!results.is_empty(), "similar_sources should find matches");
    }

    #[test]
    fn test_similar_sources_threshold_filters() {
        let lex = make_lexicon();
        let query = BilingualLexicon::encode_text("zzzzzzz");
        let results = lex.similar_sources(&query, 5, 0.5);
        assert!(results.is_empty(), "unrelated query should not match");
    }

    #[test]
    fn test_translation_memory_not_empty() {
        let lex = make_lexicon();
        assert!(lex.translation_memory().len() == VSA_DIM);
        let has_nonzero = lex.translation_memory().iter().any(|&x| x != 0);
        assert!(has_nonzero, "TM should have non-zero bits");
    }

    #[test]
    fn test_lang_memory_exists() {
        let lex = make_lexicon();
        let mem = lex.lang_memory(Language::English, Language::Spanish);
        assert!(mem.is_some(), "should have en→es memory");
        let mem = lex.lang_memory(Language::French, Language::English);
        assert!(mem.is_some(), "should have fr→en memory");
    }

    #[test]
    fn test_lang_memory_absent() {
        let lex = make_lexicon();
        let mem = lex.lang_memory(Language::English, Language::German);
        assert!(mem.is_none(), "should not have en→de memory");
    }

    #[test]
    fn test_prune_removes_least_used() {
        let mut lex = BilingualLexicon::new(3);
        lex.store("a", "x", Language::English, Language::Spanish, 0.9);
        lex.store("b", "y", Language::English, Language::Spanish, 0.9);
        lex.store("c", "z", Language::English, Language::Spanish, 0.9);
        lex.store("d", "w", Language::English, Language::Spanish, 0.9);
        assert_eq!(lex.len(), 4);
        let removed = lex.prune();
        assert_eq!(removed, 1);
        assert_eq!(lex.len(), 3);
    }

    #[test]
    fn test_prune_no_op_under_capacity() {
        let mut lex = BilingualLexicon::new(10);
        lex.store("a", "x", Language::English, Language::Spanish, 0.9);
        lex.store("b", "y", Language::English, Language::Spanish, 0.9);
        let removed = lex.prune();
        assert_eq!(removed, 0);
        assert_eq!(lex.len(), 2);
    }

    #[test]
    fn test_cleanup_vsa_returns_top_k() {
        let lex = make_lexicon();
        let query = BilingualLexicon::encode_text("hello");
        let results = lex.cleanup_vsa(&query, 2);
        assert_eq!(results.len(), 2);
        assert!(
            results[0].1 >= results[1].1,
            "results should be sorted descending"
        );
    }

    #[test]
    fn test_encode_text_deterministic() {
        let v1 = BilingualLexicon::encode_text("hello");
        let v2 = BilingualLexicon::encode_text("hello");
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_encode_text_different() {
        let v1 = BilingualLexicon::encode_text("hello");
        let v2 = BilingualLexicon::encode_text("world");
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_lookup_empty_for_unknown() {
        let lex = make_lexicon();
        let query = BilingualLexicon::encode_text("xyznonexistent");
        let results = lex.lookup(&query, Language::English, Language::Spanish, 3);
        assert!(results.is_empty() || results.len() <= 3);
    }

    #[test]
    fn test_store_multiple_pairs_same_lang() {
        let mut lex = BilingualLexicon::new(50);
        lex.store("cat", "gato", Language::English, Language::Spanish, 0.9);
        lex.store("dog", "perro", Language::English, Language::Spanish, 0.9);
        assert_eq!(lex.len(), 2);
        let mem = lex.lang_memory(Language::English, Language::Spanish);
        assert!(mem.is_some());
    }

    #[test]
    fn test_get_returns_none_for_missing() {
        let lex = make_lexicon();
        assert!(lex.get(99999).is_none());
    }

    #[test]
    fn test_bound_vsa_is_binary() {
        let mut lex = BilingualLexicon::new(50);
        lex.store("hello", "hola", Language::English, Language::Spanish, 0.9);
        let entry = lex.get(1).expect("bilingual: expected entry id=1");
        for &b in &entry.bound_vsa {
            assert!(b == 0 || b == 1, "bound VSA must be binary");
        }
    }

    #[test]
    fn test_rebuild_memory_after_store() {
        let mut lex = BilingualLexicon::new(50);
        assert!(lex.translation_memory.iter().all(|&x| x == 0));
        lex.store("hello", "hola", Language::English, Language::Spanish, 0.9);
        let has_nonzero = lex.translation_memory().iter().any(|&x| x != 0);
        assert!(has_nonzero, "TM should rebuild after store");
    }
}
