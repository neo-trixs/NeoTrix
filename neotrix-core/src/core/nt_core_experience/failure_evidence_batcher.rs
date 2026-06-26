use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_u64() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Single piece of failure evidence collected from runtime.
/// MOSS §4: evidence accumulates from periodic background scan + user-flagged turns.
#[derive(Debug, Clone)]
pub struct FailureEvidence {
    pub source: EvidenceSource,
    pub pattern: String,
    pub severity: f64,
    pub timestamp: u64,
    pub context: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceSource {
    TraceBuffer,
    InterventionLog,
    WeaknessMiner,
    UserFlagged,
    MetaAccuracy,
}

/// A sealed batch of failure evidence ready for evolution pipeline.
/// MOSS: "every downstream stage is anchored to fixing this batch"
#[derive(Debug, Clone)]
pub struct EvidenceBatch {
    pub id: u64,
    pub created_at: u64,
    pub evidence: Vec<FailureEvidence>,
    pub sealed: bool,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct EvidenceBatcherStats {
    pub total_collected: u64,
    pub total_batches_sealed: u64,
    pub recent_items: VecDeque<String>,
}

impl EvidenceBatcherStats {
    pub fn summary(&self) -> String {
        format!(
            "moss_batcher: collected={} sealed={} recent={}",
            self.total_collected,
            self.total_batches_sealed,
            self.recent_items.len(),
        )
    }
}

/// MOSS §4: Failure Evidence Batcher
/// Automatically collects failure evidence from SEAL runtime sources,
/// seals batches when thresholds are reached, and triggers evolution.
pub struct FailureEvidenceBatcher {
    /// Evidence buffer (not yet batched)
    pub buffer: VecDeque<FailureEvidence>,
    /// Sealed batches ready for evolution
    pub sealed_batches: VecDeque<EvidenceBatch>,
    /// Max evidence before auto-seal
    pub batch_threshold: usize,
    /// Min severity to trigger evolution
    pub severity_threshold: f64,
    /// Max sealed batches to retain
    pub max_batches: usize,
    /// Running ID counter
    next_batch_id: u64,
    /// Stats
    pub stats: EvidenceBatcherStats,
}

impl FailureEvidenceBatcher {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(200),
            sealed_batches: VecDeque::with_capacity(20),
            batch_threshold: 5,
            severity_threshold: 0.4,
            max_batches: 20,
            next_batch_id: 1,
            stats: EvidenceBatcherStats {
                total_collected: 0,
                total_batches_sealed: 0,
                recent_items: VecDeque::with_capacity(50),
            },
        }
    }

    pub fn with_threshold(mut self, batch_threshold: usize, severity_threshold: f64) -> Self {
        self.batch_threshold = batch_threshold;
        self.severity_threshold = severity_threshold;
        self
    }

    /// MOSS: record failure evidence from any runtime source
    pub fn record(&mut self, source: EvidenceSource, pattern: &str, severity: f64, context: &str) {
        if severity < self.severity_threshold {
            return;
        }
        self.buffer.push_back(FailureEvidence {
            source,
            pattern: pattern.to_string(),
            severity,
            timestamp: now_u64(),
            context: context.to_string(),
        });
        self.stats.total_collected += 1;
        self.stats.recent_items.push_back(pattern.to_string());
        if self.stats.recent_items.len() > 50 {
            self.stats.recent_items.pop_front();
        }
        // Auto-seal when threshold reached
        if self.buffer.len() >= self.batch_threshold {
            self.seal_batch();
        }
    }

    /// MOSS: seal current buffer into a batch anchored to fixing these failures
    pub fn seal_batch(&mut self) -> Vec<FailureEvidence> {
        if self.buffer.is_empty() {
            return Vec::new();
        }
        let evidence: Vec<FailureEvidence> = self.buffer.drain(..).collect();
        let id = self.next_batch_id;
        self.next_batch_id += 1;
        let patterns: Vec<&str> = evidence.iter().map(|e| e.pattern.as_str()).collect();
        let summary = format!(
            "moss_batch_{}: {} items (patterns: {})",
            id,
            evidence.len(),
            patterns.join(", "),
        );
        self.sealed_batches.push_back(EvidenceBatch {
            id,
            created_at: now_u64(),
            evidence: evidence.clone(),
            sealed: true,
            summary: summary.clone(),
        });
        if self.sealed_batches.len() > self.max_batches {
            self.sealed_batches.pop_front();
        }
        self.stats.total_batches_sealed += 1;
        evidence
    }

    /// MOSS: get the most recent sealed batch for evolution
    pub fn latest_batch(&self) -> Option<&EvidenceBatch> {
        self.sealed_batches.back()
    }

    /// MOSS: consume the latest batch (evolution has processed it)
    pub fn consume_latest_batch(&mut self) -> Option<EvidenceBatch> {
        self.sealed_batches.pop_back()
    }

    /// Number of evidence items in buffer
    pub fn pending_count(&self) -> usize {
        self.buffer.len()
    }

    /// Number of sealed batches ready for evolution
    pub fn batch_count(&self) -> usize {
        self.sealed_batches.len()
    }

    /// Highest severity among pending evidence
    pub fn max_severity(&self) -> f64 {
        self.buffer
            .iter()
            .map(|e| e.severity)
            .fold(0.0_f64, f64::max)
    }

    pub fn summary(&self) -> String {
        format!(
            "moss_batcher: pending={} batches={} {}",
            self.buffer.len(),
            self.sealed_batches.len(),
            self.stats.summary(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_auto_seal() {
        let mut b = FailureEvidenceBatcher::new().with_threshold(3, 0.3);
        assert_eq!(b.pending_count(), 0);
        b.record(EvidenceSource::TraceBuffer, "high_ece", 0.7, "ece=0.35");
        assert_eq!(b.pending_count(), 1);
        b.record(
            EvidenceSource::WeaknessMiner,
            "overconfidence",
            0.6,
            "pattern=Overconfidence",
        );
        assert_eq!(b.pending_count(), 2);
        b.record(
            EvidenceSource::MetaAccuracy,
            "low_meta_acc",
            0.5,
            "meta_acc=0.4",
        );
        // Threshold 3 → auto-seal
        assert_eq!(b.pending_count(), 0);
        assert_eq!(b.batch_count(), 1);
        assert!(b.stats.total_collected >= 3);
        assert!(b.stats.total_batches_sealed >= 1);
    }

    #[test]
    fn test_severity_filter() {
        let mut b = FailureEvidenceBatcher::new().with_threshold(10, 0.5);
        b.record(EvidenceSource::MetaAccuracy, "low", 0.3, "below threshold");
        assert_eq!(b.pending_count(), 0); // filtered
        b.record(
            EvidenceSource::MetaAccuracy,
            "significant",
            0.7,
            "above threshold",
        );
        assert_eq!(b.pending_count(), 1);
    }

    #[test]
    fn test_seal_and_consume() {
        let mut b = FailureEvidenceBatcher::new().with_threshold(2, 0.0);
        b.record(EvidenceSource::InterventionLog, "failure_a", 0.8, "ctx");
        b.record(EvidenceSource::InterventionLog, "failure_b", 0.9, "ctx");
        assert_eq!(b.batch_count(), 1);
        let batch = b.consume_latest_batch().unwrap();
        assert_eq!(batch.evidence.len(), 2);
        assert_eq!(batch.sealed, true);
        assert!(batch.summary.contains("moss_batch_"));
        assert_eq!(b.batch_count(), 0);
    }

    #[test]
    fn test_max_batches() {
        let mut b = FailureEvidenceBatcher::new().with_threshold(1, 0.0);
        for i in 0..25 {
            b.record(
                EvidenceSource::UserFlagged,
                &format!("failure_{}", i),
                0.6 + (i as f64 * 0.01),
                "ctx",
            );
        }
        assert!(b.batch_count() <= 20);
        assert!(b.stats.total_batches_sealed >= 20);
    }

    #[test]
    fn test_summary() {
        let mut b = FailureEvidenceBatcher::new().with_threshold(10, 0.0);
        b.record(EvidenceSource::TraceBuffer, "test", 0.5, "context");
        let s = b.summary();
        assert!(s.contains("moss_batcher:"));
        assert!(s.contains("pending="));
    }

    #[test]
    fn test_latest_batch() {
        let mut b = FailureEvidenceBatcher::new().with_threshold(1, 0.0);
        b.record(EvidenceSource::UserFlagged, "first", 0.8, "ctx");
        assert!(b.latest_batch().is_some());
        assert_eq!(b.latest_batch().unwrap().evidence.len(), 1);
    }

    #[test]
    fn test_max_severity() {
        let mut b = FailureEvidenceBatcher::new().with_threshold(10, 0.0);
        assert_eq!(b.max_severity(), 0.0);
        b.record(EvidenceSource::MetaAccuracy, "a", 0.3, "");
        b.record(EvidenceSource::MetaAccuracy, "b", 0.9, "");
        assert!((b.max_severity() - 0.9).abs() < 0.001);
    }
}
