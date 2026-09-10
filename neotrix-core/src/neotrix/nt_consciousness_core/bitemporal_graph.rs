#![forbid(unsafe_code)]

//! Bitemporal Knowledge Graph
//!
//! Implements a bitemporal fact model with two independent time dimensions:
//! - **Transaction Time**: When the fact was recorded in the system
//! - **Valid Time**: When the fact was/will be true in the real world
//!
//! This enables:
//! - Querying "what did we believe at time T?" (as-of queries)
//! - Querying "what is true now according to records made by time T?"
//! - Fact supersession: old facts are replaced but preserved in history
//! - Full audit trail of knowledge evolution

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

/// Bitemporal graph errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum GraphError {
    #[error("fact not found: {0}")]
    FactNotFound(String),

    #[error("subject not found: {0}")]
    SubjectNotFound(String),

    #[error("invalid time range: start {start} > end {end}")]
    InvalidTimeRange { start: String, end: String },

    #[error("supersede conflict: fact {0} not found or already superseded")]
    SupersedeConflict(String),
}

pub type GraphResult<T> = Result<T, GraphError>;

// ============================================================================
// BitemporalFact — The Core Entity
// ============================================================================

/// A single bitemporal fact in the knowledge graph.
///
/// Each fact represents a statement: "subject predicate object"
/// with two time dimensions:
/// - `valid_time`: when this fact is/was true in reality
/// - `transaction_time`: when this fact was recorded in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitemporalFact {
    /// Unique fact identifier
    pub id: String,
    /// Subject entity (e.g., "Rust", "GWT", "NeoTrix")
    pub subject: String,
    /// Predicate/relation (e.g., "is_a", "depends_on", "implements")
    pub predicate: String,
    /// Object entity or literal value
    pub object: String,
    /// When this fact is true in the real world
    pub valid_time: DateTime<Utc>,
    /// When this fact was recorded
    pub transaction_time: DateTime<Utc>,
    /// Confidence in this fact (0.0 – 1.0)
    pub confidence: f64,
    /// Source of this fact
    pub source: String,
    /// Whether this fact has been superseded by a newer fact
    pub superseded: bool,
    /// ID of the fact that superseded this one (if any)
    pub superseded_by: Option<String>,
    /// ID of the fact this one superseded (if any)
    pub supersedes: Option<String>,
    /// Arbitrary metadata
    pub metadata: HashMap<String, String>,
}

impl BitemporalFact {
    /// Create a new fact with current transaction time.
    pub fn new(
        subject: &str,
        predicate: &str,
        object: &str,
        valid_time: DateTime<Utc>,
        confidence: f64,
        source: &str,
    ) -> Self {
        Self {
            id: format!("fact_{}", Uuid::new_v4()),
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: object.to_string(),
            valid_time,
            transaction_time: Utc::now(),
            confidence,
            source: source.to_string(),
            superseded: false,
            superseded_by: None,
            supersedes: None,
            metadata: HashMap::new(),
        }
    }

    /// Create with explicit transaction time (for replay/audit).
    pub fn new_with_transaction(
        subject: &str,
        predicate: &str,
        object: &str,
        valid_time: DateTime<Utc>,
        transaction_time: DateTime<Utc>,
        confidence: f64,
        source: &str,
    ) -> Self {
        Self {
            id: format!("fact_{}", Uuid::new_v4()),
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: object.to_string(),
            valid_time,
            transaction_time,
            confidence,
            source: source.to_string(),
            superseded: false,
            superseded_by: None,
            supersedes: None,
            metadata: HashMap::new(),
        }
    }

    /// Check if this fact was valid at a given point in valid time.
    pub fn valid_at(&self, time: DateTime<Utc>) -> bool {
        self.valid_time <= time
    }

    /// Check if this fact was recorded by a given point in transaction time.
    pub fn recorded_by(&self, time: DateTime<Utc>) -> bool {
        self.transaction_time <= time
    }
}

// ============================================================================
// FactChain — Supersession History
// ============================================================================

/// A chain of facts where each supersedes the previous one.
/// Represents the evolution of knowledge about a single statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactChain {
    /// Head (latest) fact ID
    pub head_id: String,
    /// All fact IDs in chronological order (oldest → newest)
    pub chain: Vec<String>,
    /// Total number of supersessions
    pub supersession_count: u32,
}

// ============================================================================
// BitemporalGraph — The Knowledge Graph
// ============================================================================

/// A bitemporal knowledge graph with supersession support.
///
/// # Data Model
/// - Facts are stored in a flat `HashMap<String, BitemporalFact>`
/// - Indexed by: subject, predicate, (subject, predicate), and valid_time bucket
/// - Supersession chains track knowledge evolution
///
/// # Query Patterns
/// - `query_as_of(subject, transaction_time)`: What did we believe at time T?
/// - `query_valid_at(subject, valid_time)`: What is true at time T?
/// - `query_current(subject)`: What do we currently believe?
/// - `query_history(subject, predicate)`: Full supersession chain
pub struct BitemporalGraph {
    /// All facts indexed by ID
    facts: HashMap<String, BitemporalFact>,
    /// Subject → fact IDs (forward index)
    subject_index: HashMap<String, Vec<String>>,
    /// Predicate → fact IDs (predicate index)
    predicate_index: HashMap<String, Vec<String>>,
    /// (subject, predicate) → fact ID (latest active fact for this pair)
    sp_index: HashMap<(String, String), String>,
    /// Supersession chains: subject → predicate → FactChain
    chains: HashMap<(String, String), FactChain>,
    /// Valid-time bucket index: year-month → fact IDs (for time-range queries)
    valid_time_index: HashMap<String, Vec<String>>,
    /// Transaction-time bucket index: year-month → fact IDs
    transaction_time_index: HashMap<String, Vec<String>>,
    /// Total facts ever inserted (including superseded)
    pub total_facts: u64,
    /// Total supersessions
    pub total_supersessions: u64,
}

impl Default for BitemporalGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl BitemporalGraph {
    /// Create a new empty bitemporal graph.
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            subject_index: HashMap::new(),
            predicate_index: HashMap::new(),
            sp_index: HashMap::new(),
            chains: HashMap::new(),
            valid_time_index: HashMap::new(),
            transaction_time_index: HashMap::new(),
            total_facts: 0,
            total_supersessions: 0,
        }
    }

    /// Insert a new fact into the graph.
    ///
    /// If a fact with the same (subject, predicate) already exists and is active,
    /// the old fact is superseded and a chain is created/extended.
    pub fn insert(&mut self, fact: BitemporalFact) -> String {
        let key = (fact.subject.clone(), fact.predicate.clone());
        let fact_id = fact.id.clone();
        let vt_bucket = fact.valid_time.format("%Y-%m").to_string();
        let tt_bucket = fact.transaction_time.format("%Y-%m").to_string();

        // Check for existing active fact with same (subject, predicate)
        if let Some(old_id) = self.sp_index.get(&key).cloned() {
            if let Some(old_fact) = self.facts.get_mut(&old_id) {
                // Supersede the old fact
                old_fact.superseded = true;
                old_fact.superseded_by = Some(fact_id.clone());
                self.total_supersessions += 1;

                // Update chain
                let chain = self.chains.entry(key.clone()).or_insert_with(|| FactChain {
                    head_id: old_id.clone(),
                    chain: vec![old_id.clone()],
                    supersession_count: 0,
                });
                chain.head_id = fact_id.clone();
                chain.chain.push(fact_id.clone());
                chain.supersession_count += 1;
            }
        }

        // Store the new fact
        let mut new_fact = fact;
        new_fact.supersedes = self.sp_index.get(&key).cloned();
        self.sp_index.insert(key.clone(), fact_id.clone());
        self.facts.insert(fact_id.clone(), new_fact);

        // Update indexes
        self.subject_index
            .entry(key.0.clone())
            .or_default()
            .push(fact_id.clone());
        self.predicate_index
            .entry(key.1.clone())
            .or_default()
            .push(fact_id.clone());
        self.valid_time_index
            .entry(vt_bucket)
            .or_default()
            .push(fact_id.clone());
        self.transaction_time_index
            .entry(tt_bucket)
            .or_default()
            .push(fact_id.clone());
        self.total_facts += 1;

        fact_id
    }

    /// Insert with supersession: explicitly supersede a specific fact.
    pub fn supersede(&mut self, old_fact_id: &str, new_fact: BitemporalFact) -> GraphResult<String> {
        let old_fact = self
            .facts
            .get(old_fact_id)
            .ok_or_else(|| GraphError::SupersedeConflict(old_fact_id.to_string()))?;

        if old_fact.superseded {
            return Err(GraphError::SupersedeConflict(format!(
                "{old_fact_id} already superseded by {}",
                old_fact.superseded_by.as_deref().unwrap_or("unknown")
            )));
        }

        let key = (old_fact.subject.clone(), old_fact.predicate.clone());
        let new_id = new_fact.id.clone();

        // Mark old as superseded
        if let Some(old) = self.facts.get_mut(old_fact_id) {
            old.superseded = true;
            old.superseded_by = Some(new_id.clone());
        }

        // Update chain
        let chain = self.chains.entry(key.clone()).or_insert_with(|| FactChain {
            head_id: old_fact_id.to_string(),
            chain: vec![old_fact_id.to_string()],
            supersession_count: 0,
        });
        chain.head_id = new_id.clone();
        chain.chain.push(new_id.clone());
        chain.supersession_count += 1;

        // Store new fact
        let mut new = new_fact;
        new.supersedes = Some(old_fact_id.to_string());
        let vt_bucket = new.valid_time.format("%Y-%m").to_string();
        let tt_bucket = new.transaction_time.format("%Y-%m").to_string();

        self.facts.insert(new_id.clone(), new);
        self.sp_index.insert(key.clone(), new_id.clone());
        self.subject_index
            .entry(key.0)
            .or_default()
            .push(new_id.clone());
        self.predicate_index
            .entry(key.1)
            .or_default()
            .push(new_id.clone());
        self.valid_time_index
            .entry(vt_bucket)
            .or_default()
            .push(new_id.clone());
        self.transaction_time_index
            .entry(tt_bucket)
            .or_default()
            .push(new_id.clone());
        self.total_facts += 1;
        self.total_supersessions += 1;

        Ok(new_id)
    }

    // --- Query Methods ---

    /// Query: What do we currently believe about (subject, predicate)?
    ///
    /// Returns the latest non-superseded fact for the pair.
    pub fn query_current(&self, subject: &str, predicate: &str) -> Option<&BitemporalFact> {
        let key = (subject.to_string(), predicate.to_string());
        self.sp_index
            .get(&key)
            .and_then(|id| self.facts.get(id))
            .filter(|f| !f.superseded)
    }

    /// Query: What did we believe about (subject, predicate) as of transaction time T?
    ///
    /// Returns the fact that was the latest active at time T.
    pub fn query_as_of(
        &self,
        subject: &str,
        predicate: &str,
        transaction_time: DateTime<Utc>,
    ) -> Option<&BitemporalFact> {
        let fact_ids = self.subject_index.get(subject)?;

        // Find facts for this (subject, predicate) recorded by transaction_time
        let mut candidates: Vec<&BitemporalFact> = fact_ids
            .iter()
            .filter_map(|id| self.facts.get(id))
            .filter(|f| f.predicate == predicate && f.recorded_by(transaction_time))
            .collect();

        // Sort by transaction time descending (most recent first)
        candidates.sort_by(|a, b| b.transaction_time.cmp(&a.transaction_time));

        candidates.first().copied()
    }

    /// Query: What is valid at valid time T for (subject, predicate)?
    ///
    /// Returns the fact whose valid_time is <= T and is the latest such fact.
    pub fn query_valid_at(
        &self,
        subject: &str,
        predicate: &str,
        valid_time: DateTime<Utc>,
    ) -> Option<&BitemporalFact> {
        let fact_ids = self.subject_index.get(subject)?;

        let mut candidates: Vec<&BitemporalFact> = fact_ids
            .iter()
            .filter_map(|id| self.facts.get(id))
            .filter(|f| f.predicate == predicate && f.valid_at(valid_time))
            .collect();

        candidates.sort_by(|a, b| b.valid_time.cmp(&a.valid_time));

        candidates.first().copied()
    }

    /// Query: All facts about a subject (current, non-superseded).
    pub fn query_subject(&self, subject: &str) -> Vec<&BitemporalFact> {
        self.subject_index
            .get(subject)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.facts.get(id))
                    .filter(|f| !f.superseded)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Query: Full supersession chain for (subject, predicate).
    pub fn query_history(
        &self,
        subject: &str,
        predicate: &str,
    ) -> Vec<&BitemporalFact> {
        let key = (subject.to_string(), predicate.to_string());
        self.chains
            .get(&key)
            .map(|chain| {
                chain
                    .chain
                    .iter()
                    .filter_map(|id| self.facts.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a fact by ID.
    pub fn get_fact(&self, fact_id: &str) -> Option<&BitemporalFact> {
        self.facts.get(fact_id)
    }

    /// Total active (non-superseded) facts.
    pub fn active_count(&self) -> usize {
        self.facts.values().filter(|f| !f.superseded).count()
    }

    /// Total superseded facts.
    pub fn superseded_count(&self) -> usize {
        self.facts.values().filter(|f| f.superseded).count()
    }

    /// All unique subjects.
    pub fn subjects(&self) -> Vec<&str> {
        self.subject_index.keys().map(|s| s.as_str()).collect()
    }

    /// All unique predicates.
    pub fn predicates(&self) -> Vec<&str> {
        self.predicate_index.keys().map(|s| s.as_str()).collect()
    }

    /// Graph statistics.
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            total_facts: self.total_facts,
            active_facts: self.active_count() as u64,
            superseded_facts: self.superseded_count() as u64,
            total_supersessions: self.total_supersessions,
            unique_subjects: self.subject_index.len(),
            unique_predicates: self.predicate_index.len(),
            chain_count: self.chains.len(),
        }
    }
}

// ============================================================================
// GraphStats
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_facts: u64,
    pub active_facts: u64,
    pub superseded_facts: u64,
    pub total_supersessions: u64,
    pub unique_subjects: usize,
    pub unique_predicates: usize,
    pub chain_count: usize,
}

impl std::fmt::Display for GraphStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        BitemporalGraph Statistics")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "Facts:        {} (active: {}, superseded: {})",
            self.total_facts, self.active_facts, self.superseded_facts)?;
        writeln!(f, "Supersessions: {}", self.total_supersessions)?;
        writeln!(f, "Subjects:     {}", self.unique_subjects)?;
        writeln!(f, "Predicates:   {}", self.unique_predicates)?;
        writeln!(f, "Chains:       {}", self.chain_count)?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, 0, 0, 0)
            .unwrap()
    }

    #[test]
    fn test_insert_and_query_current() {
        let mut graph = BitemporalGraph::new();
        let fact = BitemporalFact::new("Rust", "is_a", "language", ts(2024, 1, 1), 0.99, "docs");
        graph.insert(fact);

        let result = graph.query_current("Rust", "is_a").unwrap();
        assert_eq!(result.object, "language");
        assert!(!result.superseded);
    }

    #[test]
    fn test_supersession_chain() {
        let mut graph = BitemporalGraph::new();

        let f1 = BitemporalFact::new("GWT", "version", "1.0", ts(2024, 1, 1), 0.9, "release");
        graph.insert(f1);

        let f2 = BitemporalFact::new("GWT", "version", "2.0", ts(2024, 6, 1), 0.95, "release");
        graph.insert(f2);

        // Current should be v2.0
        let current = graph.query_current("GWT", "version").unwrap();
        assert_eq!(current.object, "2.0");

        // v1.0 should be superseded
        let history = graph.query_history("GWT", "version");
        assert_eq!(history.len(), 2);
        assert!(history[0].superseded);
        assert!(!history[1].superseded);

        // Chain should have 2 entries
        assert_eq!(graph.total_supersessions, 1);
    }

    #[test]
    fn test_query_as_of() {
        let mut graph = BitemporalGraph::new();

        let f1 = BitemporalFact::new("KB", "status", "experimental", ts(2024, 1, 1), 0.8, "dev");
        graph.insert(f1);

        let f2 = BitemporalFact::new("KB", "status", "production", ts(2024, 6, 1), 0.95, "dev");
        graph.insert(f2);

        // As of March 2024 → should see "experimental"
        let as_of = graph.query_as_of("KB", "status", ts(2024, 3, 1));
        assert!(as_of.is_some());
        // The most recently recorded fact by March 2024
        let fact = as_of.unwrap();
        assert_eq!(fact.object, "experimental");
    }

    #[test]
    fn test_query_valid_at() {
        let mut graph = BitemporalGraph::new();

        let f1 = BitemporalFact::new("E8", "dimension", "8", ts(2024, 1, 1), 0.99, "math");
        graph.insert(f1);

        let result = graph.query_valid_at("E8", "dimension", ts(2024, 6, 1));
        assert!(result.is_some());
        assert_eq!(result.unwrap().object, "8");
    }

    #[test]
    fn test_query_subject() {
        let mut graph = BitemporalGraph::new();
        graph.insert(BitemporalFact::new("Rust", "is_a", "language", ts(2024, 1, 1), 0.99, "docs"));
        graph.insert(BitemporalFact::new("Rust", "has_feature", "ownership", ts(2024, 1, 1), 0.95, "docs"));

        let facts = graph.query_subject("Rust");
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_supersede_explicit() {
        let mut graph = BitemporalGraph::new();
        let f1 = BitemporalFact::new("X", "val", "old", ts(2024, 1, 1), 0.5, "src");
        let id1 = graph.insert(f1);

        let f2 = BitemporalFact::new("X", "val", "new", ts(2024, 6, 1), 0.9, "src");
        let id2 = graph.supersede(&id1, f2).unwrap();

        let current = graph.query_current("X", "val").unwrap();
        assert_eq!(current.id, id2);
        assert_eq!(current.object, "new");
    }

    #[test]
    fn test_supersede_already_superseded() {
        let mut graph = BitemporalGraph::new();
        let f1 = BitemporalFact::new("Y", "val", "v1", ts(2024, 1, 1), 0.5, "src");
        let id1 = graph.insert(f1);

        let f2 = BitemporalFact::new("Y", "val", "v2", ts(2024, 6, 1), 0.9, "src");
        graph.supersede(&id1, f2).unwrap();

        let f3 = BitemporalFact::new("Y", "val", "v3", ts(2024, 12, 1), 0.95, "src");
        let result = graph.supersede(&id1, f3);
        assert!(result.is_err());
    }

    #[test]
    fn test_stats() {
        let mut graph = BitemporalGraph::new();
        graph.insert(BitemporalFact::new("A", "p", "v1", ts(2024, 1, 1), 0.9, "s"));
        graph.insert(BitemporalFact::new("A", "p", "v2", ts(2024, 6, 1), 0.95, "s"));
        graph.insert(BitemporalFact::new("B", "q", "w", ts(2024, 1, 1), 0.8, "s"));

        let stats = graph.stats();
        assert_eq!(stats.total_facts, 3);
        assert_eq!(stats.active_facts, 2);
        assert_eq!(stats.superseded_facts, 1);
        assert_eq!(stats.unique_subjects, 2);
    }

    #[test]
    fn test_empty_queries() {
        let graph = BitemporalGraph::new();
        assert!(graph.query_current("nope", "nope").is_none());
        assert!(graph.query_subject("nope").is_empty());
        assert!(graph.query_history("nope", "nope").is_empty());
    }
}
