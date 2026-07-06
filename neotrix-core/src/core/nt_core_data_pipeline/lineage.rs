//! Data lineage — track every resource from acquisition through processing
//! to storage and consumption. Enables full provenance queries:
//! "Where did this proxy come from?" "When was it last validated?"
//! "Which LLM providers were discovered from OpenRouter?"

use std::collections::VecDeque;

/// One step in a resource's lifecycle.
#[derive(Debug, Clone)]
pub struct LineageEntry {
    /// Source identifier (e.g. "geonode-api", "openrouter", "github:owner/repo")
    pub source: String,
    /// Stage name (e.g. "acquire", "normalize", "store", "distill")
    pub stage: String,
    /// Items entering this stage
    pub items_processed: usize,
    /// Items that passed through successfully
    pub items_succeeded: usize,
    /// Items that failed
    pub items_failed: usize,
    /// How long this stage took in ms
    pub duration_ms: f64,
}

impl LineageEntry {
    pub fn new(source: &str, stage: &str) -> Self {
        Self {
            source: source.to_string(),
            stage: stage.to_string(),
            items_processed: 0,
            items_succeeded: 0,
            items_failed: 0,
            duration_ms: 0.0,
        }
    }
}

/// Tracks the lifecycle of data as it flows through pipelines.
pub struct DataLineage {
    entries: VecDeque<LineageEntry>,
    max_entries: usize,
}

impl DataLineage {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(1000),
            max_entries: 10_000,
        }
    }

    pub fn with_max(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Record a lineage entry.
    pub fn record(&mut self, entry: LineageEntry) {
        self.entries.push_back(entry);
        if self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
    }

    /// Get recent N entries.
    pub async fn recent(&self, n: usize) -> Vec<LineageEntry> {
        self.entries.iter().rev().take(n).cloned().collect()
    }

    /// Get entries for a specific source.
    pub fn for_source(&self, source: &str) -> Vec<LineageEntry> {
        self.entries
            .iter()
            .filter(|e| e.source == source)
            .cloned()
            .collect()
    }

    /// Get statistics for a stage.
    pub fn stage_stats(&self, stage: &str) -> (usize, usize, usize) {
        let mut processed = 0;
        let mut succeeded = 0;
        let mut failed = 0;
        for entry in &self.entries {
            if entry.stage == stage {
                processed += entry.items_processed;
                succeeded += entry.items_succeeded;
                failed += entry.items_failed;
            }
        }
        (processed, succeeded, failed)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for DataLineage {
    fn default() -> Self {
        Self::new()
    }
}
