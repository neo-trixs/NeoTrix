#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

/// A resolved entity record linking names, VSA embeddings, and source provenance.
#[derive(Debug, Clone)]
pub struct EntityRecord {
    /// Unique numeric identifier for this entity.
    pub id: u64,
    /// Canonical name string.
    pub name: String,
    /// Alternative spellings and nicknames.
    pub aliases: HashSet<String>,
    /// 4096-bit VSA vector encoding the entity semantics.
    pub vsa_vector: Vec<u8>,
    /// Identifiers of source(s) that provided evidence for this entity.
    pub source_ids: HashSet<u64>,
    /// Confidence in the resolution [0.0, 1.0].
    pub confidence: f64,
}

/// Entity resolution engine that fuses VSA cosine similarity with string-level
/// character similarity to decide whether two mentions refer to the same entity.
///
/// # Architecture
///
/// ```text
///         name_a ──┐                    ┌── vsa_cosine(encode(name_a), encode(name_b))
///                  ├── fused_score ───► threshold → merge / keep separate
///         name_b ──┘                    └── string_similarity(name_a, name_b)
/// ```
///
/// A `merge_threshold` (default 0.85) gates automatic merging. Sources are tracked
/// so downstream systems can attribute resolution decisions.
pub struct EntityResolver {
    /// id → record
    entities: HashMap<u64, EntityRecord>,
    /// name / alias → id
    name_index: HashMap<String, u64>,
    /// Monotonically increasing id counter.
    next_id: u64,
    /// Cosine–string fusion threshold for automatic merging (default 0.85).
    pub merge_threshold: f64,
}

impl Default for EntityResolver {
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
            name_index: HashMap::new(),
            next_id: 1,
            merge_threshold: 0.85,
        }
    }
}

impl EntityResolver {
    /// Create a new empty resolver with the default threshold (0.85).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a resolver with a custom merge threshold.
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            merge_threshold: threshold.clamp(0.0, 1.0),
            ..Self::default()
        }
    }

    /// Resolve two names into a fused similarity score in [0, 1].
    ///
    /// The score is a weighted combination:
    ///   `0.6 * vsa_cosine(encode(name_a), encode(name_b)) + 0.4 * string_similarity(name_a, name_b)`
    pub fn resolve(&self, name_a: &str, name_b: &str) -> f64 {
        let va = Self::encode(name_a);
        let vb = Self::encode(name_b);
        let vsim = QuantizedVSA::cosine(&va, &vb);
        let ssim = self.string_similarity(name_a, name_b);
        0.6 * vsim + 0.4 * ssim
    }

    /// Register a name from a given source and return the entity id.
    ///
    /// If an entity with a sufficiently similar name already exists (fused score
    /// ≥ `merge_threshold`), the new name is added as an alias of the existing
    /// entity and its source id is attached.
    ///
    /// Otherwise a brand-new entity record is created.
    pub fn register(&mut self, name: &str, source_id: u64) -> u64 {
        if let Some(&eid) = self.name_index.get(name) {
            // Exact match: just attach the source if not already present.
            if let Some(record) = self.entities.get_mut(&eid) {
                record.source_ids.insert(source_id);
            }
            return eid;
        }

        // Check for a close match via fusion.
        for record in self.entities.values() {
            let fused = self.resolve(name, &record.name);
            if fused >= self.merge_threshold {
                let eid = record.id;
                self.name_index.insert(name.to_string(), eid);
                if let Some(r) = self.entities.get_mut(&eid) {
                    r.aliases.insert(name.to_string());
                    r.source_ids.insert(source_id);
                    // Re-bundle VSA to incorporate the new mention.
                    let new_v = Self::encode(name);
                    r.vsa_vector = QuantizedVSA::bundle(&[&r.vsa_vector, &new_v]);
                }
                return eid;
            }
        }

        let id = self.next_id;
        self.next_id += 1;
        let vsa = Self::encode(name);
        let record = EntityRecord {
            id,
            name: name.to_string(),
            aliases: HashSet::new(),
            vsa_vector: vsa,
            source_ids: {
                let mut s = HashSet::new();
                s.insert(source_id);
                s
            },
            confidence: 1.0,
        };
        self.name_index.insert(name.to_string(), id);
        self.entities.insert(id, record);
        id
    }

    /// Merge entity `id_b` into `id_a`, combining aliases, source_ids, and VSA
    /// vectors. Returns `false` if either id is unknown.
    ///
    /// After merge the `id_b` record is removed from the index.
    pub fn merge(&mut self, id_a: u64, id_b: u64) -> bool {
        if id_a == id_b {
            return false;
        }
        let record_b = match self.entities.remove(&id_b) {
            Some(r) => r,
            None => return false,
        };
        let record_a = match self.entities.get_mut(&id_a) {
            Some(r) => r,
            None => {
                // Put back what we removed.
                self.entities.insert(id_b, record_b);
                return false;
            }
        };

        // Merge aliases.
        for alias in &record_b.aliases {
            record_a.aliases.insert(alias.clone());
            self.name_index.insert(alias.clone(), id_a);
        }
        record_a.aliases.insert(record_b.name.clone());
        self.name_index.insert(record_b.name.clone(), id_a);

        // Merge source ids.
        record_a.source_ids.extend(&record_b.source_ids);

        // Bundle VSA vectors (weighted: existing vector bundled with incoming).
        let merged_vsa = QuantizedVSA::bundle(&[&record_a.vsa_vector, &record_b.vsa_vector]);
        record_a.vsa_vector = merged_vsa;

        // Update confidence: weighted average favoring higher-confidence side.
        let total_conf = record_a.confidence + record_b.confidence;
        if total_conf > 0.0 {
            record_a.confidence = (record_a.confidence * record_a.confidence
                + record_b.confidence * record_b.confidence)
                / total_conf;
        }

        // Remove id_b from name_index (already redirected above for the old name).
        self.name_index.remove(&record_b.name);

        true
    }

    /// Find an entity id by exact name, then by alias lookup.
    pub fn find_by_name(&self, name: &str) -> Option<u64> {
        self.name_index.get(name).copied()
    }

    /// Return a reference to an entity record by id.
    pub fn get(&self, id: u64) -> Option<&EntityRecord> {
        self.entities.get(&id)
    }

    /// Return a mutable reference to an entity record.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut EntityRecord> {
        self.entities.get_mut(&id)
    }

    /// Total number of distinct entities tracked.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns `true` when no entities are registered.
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Iterate over all entity records.
    pub fn iter(&self) -> impl Iterator<Item = &EntityRecord> {
        self.entities.values()
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Deterministic VSA encoding from a string (seeded hash).
    fn encode(text: &str) -> Vec<u8> {
        let seed: u64 = text
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    /// Simple character-level string similarity based on common prefix length
    /// and length ratio, modelling a coarse approximation of sequence matching.
    ///
    /// Return value in [0, 1].
    ///
    /// Formula:
    ///   ```
    ///   prefix_score = common_prefix_len / max(a_len, b_len)
    ///   length_ratio = min(a_len, b_len) / max(a_len, b_len)
    ///   score = 0.6 * prefix_score + 0.4 * length_ratio
    ///   ```
    pub fn string_similarity(&self, a: &str, b: &str) -> f64 {
        let (a_bytes, b_bytes) = (a.as_bytes(), b.as_bytes());
        let (len_a, len_b) = (a_bytes.len(), b_bytes.len());
        if len_a == 0 && len_b == 0 {
            return 1.0;
        }
        if len_a == 0 || len_b == 0 {
            return 0.0;
        }
        let max_len = len_a.max(len_b) as f64;
        let min_len = len_a.min(len_b) as f64;
        let common_prefix = a_bytes
            .iter()
            .zip(b_bytes.iter())
            .take_while(|(x, y)| x == y)
            .count() as f64;
        let prefix_score = common_prefix / max_len;
        let length_ratio = min_len / max_len;
        0.6 * prefix_score + 0.4 * length_ratio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_returns_id() {
        let mut er = EntityResolver::new();
        let id = er.register("Alice Johnson", 1);
        assert!(id > 0);
        assert_eq!(er.len(), 1);
    }

    #[test]
    fn test_register_duplicate_exact() {
        let mut er = EntityResolver::new();
        let id_a = er.register("Alice Johnson", 1);
        let id_b = er.register("Alice Johnson", 2);
        assert_eq!(id_a, id_b, "exact duplicate should return same id");
        assert_eq!(er.len(), 1);
        let rec = er.get(id_a).unwrap();
        assert!(rec.source_ids.contains(&1));
        assert!(rec.source_ids.contains(&2));
    }

    #[test]
    fn test_register_similar_fuses() {
        let mut er = EntityResolver::new();
        let id_a = er.register("Alice Johnson", 1);
        let id_b = er.register("Alice Jhonson", 2);
        // With 0.85 threshold these should merge via fuzzy match.
        assert_eq!(id_a, id_b, "similar names should merge");
        let rec = er.get(id_a).unwrap();
        assert!(rec.aliases.contains("Alice Jhonson"));
    }

    #[test]
    fn test_register_different_creates_separate() {
        let mut er = EntityResolver::new();
        let id_a = er.register("Alice Johnson", 1);
        let id_b = er.register("Bob Smith", 2);
        assert_ne!(id_a, id_b);
        assert_eq!(er.len(), 2);
    }

    #[test]
    fn test_find_by_name_exact() {
        let mut er = EntityResolver::new();
        let id = er.register("Alice Johnson", 1);
        assert_eq!(er.find_by_name("Alice Johnson"), Some(id));
    }

    #[test]
    fn test_find_by_name_alias() {
        let mut er = EntityResolver::new();
        let id = er.register("Alice Johnson", 1);
        // Register similar to trigger alias insertion.
        let _ = er.register("Alice Jhonson", 2);
        assert_eq!(er.find_by_name("Alice Jhonson"), Some(id));
    }

    #[test]
    fn test_find_by_name_missing() {
        let er = EntityResolver::new();
        assert_eq!(er.find_by_name("Nobody"), None);
    }

    #[test]
    fn test_resolve_high_similarity() {
        let er = EntityResolver::new();
        let score = er.resolve("Alice Johnson", "Alice Jhonson");
        assert!(
            score > 0.7,
            "similar names should score high, got {}",
            score
        );
    }

    #[test]
    fn test_resolve_low_similarity() {
        let er = EntityResolver::new();
        let score = er.resolve("Alice Johnson", "Quantum Physics");
        assert!(
            score < 0.6,
            "different names should score low, got {}",
            score
        );
    }

    #[test]
    fn test_merge_combines_aliases_and_sources() {
        let mut er = EntityResolver::new();
        let id_a = er.register("Alice Johnson", 1);
        let id_b = er.register("A. Johnson", 2);
        // If they don't auto-merge (different enougj), force merge.
        if id_a != id_b {
            assert!(er.merge(id_a, id_b));
        }
        let rec = er.get(id_a).unwrap();
        // "A. Johnson" should appear as either name or alias.
        let has_a = rec.name == "A. Johnson" || rec.aliases.contains("A. Johnson");
        assert!(has_a, "merged entity should carry 'A. Johnson'");
        assert!(rec.source_ids.contains(&1));
        assert!(rec.source_ids.contains(&2));
    }

    #[test]
    fn test_merge_nonexistent_ids() {
        let mut er = EntityResolver::new();
        let id_a = er.register("Alice", 1);
        assert!(!er.merge(id_a, 999));
        assert!(!er.merge(999, id_a));
        assert!(!er.merge(999, 888));
    }

    #[test]
    fn test_merge_same_id() {
        let mut er = EntityResolver::new();
        let id = er.register("Alice", 1);
        assert!(!er.merge(id, id));
    }

    #[test]
    fn test_string_similarity_exact() {
        let er = EntityResolver::new();
        assert!((er.string_similarity("hello", "hello") - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_string_similarity_empty() {
        let er = EntityResolver::new();
        assert!((er.string_similarity("", "") - 1.0).abs() < 1e-10);
        assert!((er.string_similarity("hello", "") - 0.0).abs() < 1e-10);
        assert!((er.string_similarity("", "world") - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_string_similarity_no_match() {
        let er = EntityResolver::new();
        let sim = er.string_similarity("abc", "xyz");
        assert!(
            sim < 0.3,
            "completely different strings should score low, got {}",
            sim
        );
    }

    #[test]
    fn test_register_under_custom_threshold() {
        let mut er = EntityResolver::with_threshold(0.95);
        let id_a = er.register("Alice Johnson", 1);
        let id_b = er.register("Alice Jhonson", 2);
        // With a stricter threshold these should NOT merge.
        assert_ne!(id_a, id_b, "strict threshold should keep entities separate");
        assert_eq!(er.len(), 2);
    }

    #[test]
    fn test_merge_bundles_vsa() {
        let mut er = EntityResolver::new();
        let id_a = er.register("Alpha Corp", 1);
        let id_b = er.register("Alpha Corporation", 2);
        // Force merge if not auto-merged.
        if id_a != id_b {
            assert!(er.merge(id_a, id_b));
        }
        let rec = er.get(id_a).unwrap();
        // The VSA vector should differ from either original encoding alone.
        let va = EntityResolver::encode("Alpha Corp");
        let vb = EntityResolver::encode("Alpha Corporation");
        let sim_to_a = QuantizedVSA::similarity(&rec.vsa_vector, &va);
        let sim_to_b = QuantizedVSA::similarity(&rec.vsa_vector, &vb);
        // The bundled vector should be similar (but not identical) to both.
        assert!(
            sim_to_a > 0.45,
            "bundled should be similar to A, got {}",
            sim_to_a
        );
        assert!(
            sim_to_b > 0.45,
            "bundled should be similar to B, got {}",
            sim_to_b
        );
    }

    #[test]
    fn test_is_empty_and_len() {
        let er = EntityResolver::new();
        assert!(er.is_empty());
        assert_eq!(er.len(), 0);
    }
}
