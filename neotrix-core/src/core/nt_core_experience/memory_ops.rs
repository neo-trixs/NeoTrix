use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

// ── Core Types ──

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryOperationType {
    Add,
    Update { old_id: u64 },
    Delete { reason: String },
    Noop { reason: String },
}

#[derive(Debug, Clone)]
pub struct MemoryOperation {
    pub id: u64,
    pub operation_type: MemoryOperationType,
    pub content: String,
    pub tags: Vec<String>,
    pub timestamp: u64,
    pub source: String,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionedFact {
    pub id: u64,
    pub fact_id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub version: u64,
    pub timestamp: u64,
    pub source: String,
    pub embedding: Vec<f64>,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResult {
    NoConflict,
    Contradiction {
        existing: VersionedFact,
        incoming: VersionedFact,
        similarity: f64,
    },
    PartialOverlap {
        existing: VersionedFact,
        overlap_score: f64,
    },
    Duplicate {
        existing_id: u64,
    },
}

// ── MemoryStore Trait ──

pub trait MemoryStore {
    fn add(&mut self, fact: VersionedFact) -> Result<u64, String>;
    fn update(&mut self, id: u64, new_fact: VersionedFact) -> Result<ConflictResult, String>;
    fn delete(&mut self, id: u64, reason: &str) -> Result<(), String>;
    fn get(&self, id: u64) -> Option<&VersionedFact>;
    fn search(&self, query: &str, max_results: usize) -> Vec<&VersionedFact>;
    fn get_active(&self) -> Vec<&VersionedFact>;
    fn get_version_history(&self, fact_id: &str) -> Vec<&VersionedFact>;
}

// ── VSA Embedding ──

fn hash_to_unit_vector(seed: u64, dim: usize) -> Vec<f64> {
    let mut rng_state = seed;
    let mut vec = Vec::with_capacity(dim);
    for _ in 0..dim {
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let val = (rng_state >> 33) as f64 / u64::MAX as f64;
        vec.push(val * 2.0 - 1.0);
    }
    let mag: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
    if mag > 0.0 {
        for v in &mut vec {
            *v /= mag;
        }
    }
    vec
}

pub fn compute_embedding(content: &str, tags: &[String], dim: usize) -> Vec<f64> {
    let mut sum = vec![0.0_f64; dim];
    let mut count = 0usize;

    for word in content.split_whitespace() {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        word.hash(&mut hasher);
        let seed = hasher.finish();
        let unit = hash_to_unit_vector(seed, dim);
        for (s, u) in sum.iter_mut().zip(unit.iter()) {
            *s += u;
        }
        count += 1;
    }

    for tag in tags {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        tag.hash(&mut hasher);
        let seed = hasher.finish();
        let unit = hash_to_unit_vector(seed, dim);
        for (s, u) in sum.iter_mut().zip(unit.iter()) {
            *s += u * 2.0;
        }
        count += 2;
    }

    if count > 0 {
        let mag: f64 = sum.iter().map(|v| v * v).sum::<f64>().sqrt();
        if mag > 0.0 {
            for v in &mut sum {
                *v /= mag;
            }
        }
    }

    sum
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let ma: f64 = a.iter().map(|v| v * v).sum::<f64>().sqrt();
    let mb: f64 = b.iter().map(|v| v * v).sum::<f64>().sqrt();
    if ma == 0.0 || mb == 0.0 {
        return 0.0;
    }
    (dot / (ma * mb)).clamp(-1.0, 1.0)
}

// ── FactBuilder ──

pub struct FactBuilder {
    fact_id: String,
    content: String,
    tags: Vec<String>,
    source: String,
    dim: usize,
    now: u64,
}

impl FactBuilder {
    pub fn new(fact_id: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            fact_id: fact_id.to_string(),
            content: String::new(),
            tags: Vec::new(),
            source: String::new(),
            dim: 64,
            now,
        }
    }

    pub fn content(mut self, c: &str) -> Self {
        self.content = c.to_string();
        self
    }

    pub fn tag(mut self, t: &str) -> Self {
        self.tags.push(t.to_string());
        self
    }

    pub fn source(mut self, s: &str) -> Self {
        self.source = s.to_string();
        self
    }

    pub fn build(&self) -> VersionedFact {
        let embedding = compute_embedding(&self.content, &self.tags, self.dim);
        VersionedFact {
            id: 0,
            fact_id: self.fact_id.clone(),
            content: self.content.clone(),
            tags: self.tags.clone(),
            version: 1,
            timestamp: self.now,
            source: self.source.clone(),
            embedding,
            is_active: true,
        }
    }
}

// ── VsaMemoryStore ──

pub struct VsaMemoryStore {
    facts: Vec<VersionedFact>,
    next_id: u64,
    pub conflict_threshold: f64,
    pub duplicate_threshold: f64,
}

impl VsaMemoryStore {
    pub fn new(conflict_threshold: f64, duplicate_threshold: f64) -> Self {
        Self {
            facts: Vec::new(),
            next_id: 1,
            conflict_threshold,
            duplicate_threshold,
        }
    }

    pub fn detect_conflict(&self, new_fact: &VersionedFact) -> Vec<ConflictResult> {
        let mut results = Vec::new();
        for existing in &self.facts {
            if !existing.is_active {
                continue;
            }
            let sim = cosine_similarity(&existing.embedding, &new_fact.embedding);
            if sim > self.duplicate_threshold {
                results.push(ConflictResult::Duplicate {
                    existing_id: existing.id,
                });
            } else if sim > 0.0 && sim < self.conflict_threshold {
                results.push(ConflictResult::Contradiction {
                    existing: existing.clone(),
                    incoming: new_fact.clone(),
                    similarity: sim,
                });
            } else if sim > 0.0 && sim < self.duplicate_threshold {
                results.push(ConflictResult::PartialOverlap {
                    existing: existing.clone(),
                    overlap_score: sim,
                });
            }
        }
        results
    }

    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    pub fn active_count(&self) -> usize {
        self.facts.iter().filter(|f| f.is_active).count()
    }
}

impl MemoryStore for VsaMemoryStore {
    fn add(&mut self, mut fact: VersionedFact) -> Result<u64, String> {
        let conflicts = self.detect_conflict(&fact);
        for conflict in &conflicts {
            if let ConflictResult::Duplicate { existing_id } = conflict {
                return Err(format!("Duplicate detected with fact id={}", existing_id));
            }
        }
        fact.id = self.next_id;
        self.next_id += 1;
        let id = fact.id;
        self.facts.push(fact);
        Ok(id)
    }

    fn update(&mut self, id: u64, mut new_fact: VersionedFact) -> Result<ConflictResult, String> {
        let existing = self
            .facts
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| format!("Fact with id={} not found", id))?;

        if !existing.is_active {
            return Err(format!("Fact with id={} is already deleted", id));
        }

        let sim = cosine_similarity(&existing.embedding, &new_fact.embedding);
        if sim > self.duplicate_threshold {
            return Ok(ConflictResult::NoConflict);
        }

        new_fact.id = self.next_id;
        self.next_id += 1;
        new_fact.version = existing.version + 1;

        existing.is_active = false;

        if sim > 0.0 && sim < self.conflict_threshold {
            let result = ConflictResult::Contradiction {
                existing: existing.clone(),
                incoming: new_fact.clone(),
                similarity: sim,
            };
            self.facts.push(new_fact);
            Ok(result)
        } else {
            let result = ConflictResult::NoConflict;
            self.facts.push(new_fact);
            Ok(result)
        }
    }

    fn delete(&mut self, id: u64, _reason: &str) -> Result<(), String> {
        let fact = self
            .facts
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| format!("Fact with id={} not found", id))?;
        fact.is_active = false;
        Ok(())
    }

    fn get(&self, id: u64) -> Option<&VersionedFact> {
        self.facts.iter().find(|f| f.id == id)
    }

    fn search(&self, query: &str, max_results: usize) -> Vec<&VersionedFact> {
        let q_lower = query.to_lowercase();
        self.facts
            .iter()
            .filter(|f| f.is_active)
            .filter(|f| {
                f.content.to_lowercase().contains(&q_lower)
                    || f.tags.iter().any(|t| t.to_lowercase().contains(&q_lower))
                    || f.fact_id.to_lowercase().contains(&q_lower)
            })
            .take(max_results)
            .collect()
    }

    fn get_active(&self) -> Vec<&VersionedFact> {
        self.facts.iter().filter(|f| f.is_active).collect()
    }

    fn get_version_history(&self, fact_id: &str) -> Vec<&VersionedFact> {
        let mut history: Vec<&VersionedFact> =
            self.facts.iter().filter(|f| f.fact_id == fact_id).collect();
        history.sort_by_key(|f| f.version);
        history
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> VsaMemoryStore {
        VsaMemoryStore::new(0.3, 0.85)
    }

    fn fact(content: &str, tags: &[&str], fact_id: &str) -> VersionedFact {
        let mut b = FactBuilder::new(fact_id).content(content).source("test");
        for t in tags {
            b = b.tag(t);
        }
        b.build()
    }

    // 1. Add a fact, get it back
    #[test]
    fn test_add_and_get() {
        let mut store = make_store();
        let f = fact("User likes Python", &["preference"], "user-lang");
        let id = store.add(f.clone()).unwrap();
        assert!(id > 0);
        let retrieved = store.get(id).unwrap();
        assert_eq!(retrieved.content, "User likes Python");
        assert!(retrieved.is_active);
        assert_eq!(retrieved.version, 1);
    }

    // 2. Add two facts, search finds both
    #[test]
    fn test_search_finds_multiple() {
        let mut store = make_store();
        store.add(fact("Python is great", &["lang"], "py")).unwrap();
        store.add(fact("Rust is fast", &["lang"], "rs")).unwrap();
        let results = store.search("lang", 10);
        assert_eq!(results.len(), 2);
    }

    // 3. Update a fact -> version increments, old is inactive
    #[test]
    fn test_update_version_and_deactivate() {
        let mut store = make_store();
        let f = fact("User likes Python", &["preference"], "user-lang");
        let id = store.add(f).unwrap();

        let new_fact = fact("User switched to Rust", &["preference"], "user-lang");
        let result = store.update(id, new_fact).unwrap();

        let old = store.get(id).unwrap();
        assert!(!old.is_active);

        let versions = store.get_version_history("user-lang");
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, 1);
        assert_eq!(versions[0].content, "User likes Python");
        assert_eq!(versions[1].version, 2);
        assert_eq!(versions[1].content, "User switched to Rust");
        assert!(versions[1].is_active);
    }

    // 4. Delete a fact -> is_active = false, not in active results
    #[test]
    fn test_delete_soft() {
        let mut store = make_store();
        let id = store.add(fact("secret data", &["secret"], "sec")).unwrap();
        store.delete(id, "no longer needed").unwrap();

        let retrieved = store.get(id).unwrap();
        assert!(!retrieved.is_active);

        let active = store.get_active();
        assert!(active.iter().all(|f| f.is_active));
        assert!(!active.iter().any(|f| f.id == id));

        let search = store.search("secret", 10);
        assert!(search.is_empty());
    }

    // 5. Version history: same fact_id -> multiple versions retrievable
    #[test]
    fn test_version_history_multiple() {
        let mut store = make_store();
        let id1 = store.add(fact("v1 content", &["a"], "multi")).unwrap();
        let id2 = store.add(fact("v2 content", &["a"], "multi")).unwrap();
        let id3 = store.add(fact("v3 content", &["a"], "multi")).unwrap();

        let history = store.get_version_history("multi");
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].version, 1);
        assert_eq!(history[0].content, "v1 content");
        assert_eq!(history[1].version, 1);
        assert_eq!(history[1].content, "v2 content");
        assert_eq!(history[2].version, 1);
        assert_eq!(history[2].content, "v3 content");
    }

    // 6. Conflict detection: contradictory facts with low similarity
    #[test]
    fn test_conflict_detection() {
        let mut store = make_store();
        store
            .add(fact(
                "User likes Python",
                &["preference", "user"],
                "user-lang",
            ))
            .unwrap();
        let f2 = fact(
            "User switched to Rust",
            &["preference", "user"],
            "user-lang",
        );
        let conflicts = store.detect_conflict(&f2);
        let has_contradiction = conflicts
            .iter()
            .any(|c| matches!(c, ConflictResult::Contradiction { .. }));
        assert!(
            has_contradiction,
            "Should detect contradiction between likes Python and switched to Rust"
        );
    }

    // 7. Duplicate detection: same content added twice
    #[test]
    fn test_duplicate_detection() {
        let mut store = make_store();
        let f = fact("User likes Python", &["preference"], "user-lang");
        store.add(f.clone()).unwrap();
        let result = store.add(f.clone());
        assert!(result.is_err(), "Duplicate addition should be rejected");
        assert!(result.unwrap_err().contains("Duplicate"));
    }

    // 8. Noop on no-change update (duplicate threshold)
    #[test]
    fn test_noop_on_same_content_update() {
        let mut store = make_store();
        let f = fact("User likes Python", &["preference"], "user-lang");
        let id = store.add(f.clone()).unwrap();
        let result = store.update(id, f.clone()).unwrap();
        assert_eq!(result, ConflictResult::NoConflict);
    }

    // 9. Conflict detection via update
    #[test]
    fn test_update_detects_contradiction() {
        let mut store = make_store();
        store.conflict_threshold = 0.6;
        let f = fact(
            "User likes Python heavily",
            &["preference", "user"],
            "user-lang",
        );
        let id = store.add(f).unwrap();
        let f2 = fact(
            "User switched to Rust entirely",
            &["preference", "user"],
            "user-lang",
        );
        let result = store.update(id, f2).unwrap();
        assert!(
            matches!(&result, ConflictResult::Contradiction { .. }),
            "Update should detect contradiction, got {:?}",
            result
        );
    }

    // 10. Search by tag substring
    #[test]
    fn test_search_by_tag() {
        let mut store = make_store();
        store
            .add(fact("alpha", &["important", "urgent"], "a"))
            .unwrap();
        store.add(fact("beta", &["normal"], "b")).unwrap();
        let results = store.search("urgent", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "alpha");
    }

    // 11. Delete non-existent returns error
    #[test]
    fn test_delete_nonexistent() {
        let mut store = make_store();
        let result = store.delete(999, "not found");
        assert!(result.is_err());
    }

    // 12. Update non-existent returns error
    #[test]
    fn test_update_nonexistent() {
        let mut store = make_store();
        let f = fact("anything", &[], "x");
        let result = store.update(999, f);
        assert!(result.is_err());
    }

    // 13. Embedding is deterministic for same input
    #[test]
    fn test_embedding_deterministic() {
        let tags = vec!["a".to_string(), "b".to_string()];
        let e1 = compute_embedding("hello world", &tags, 64);
        let e2 = compute_embedding("hello world", &tags, 64);
        assert_eq!(e1, e2);
    }

    // 14. Empty search returns empty
    #[test]
    fn test_empty_search() {
        let store = make_store();
        let results = store.search("anything", 10);
        assert!(results.is_empty());
    }

    // 15. Cosine similarity is symmetric
    #[test]
    fn test_cosine_symmetry() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-10);
        assert!((cosine_similarity(&b, &a) - 0.0).abs() < 1e-10);
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-10);
    }
}
