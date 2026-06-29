use std::collections::VecDeque;

/// 5-tier context compaction pipeline.
/// Each tier sacrifices different dimensions (cost, info loss, cache impact).
/// Progressive: starts cheap, gets more expensive but preserves more.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionTier {
    /// Snip: discard old tool outputs. Free, high information loss.
    Snip,
    /// Microcompact: clear individual agent results while respecting cache ranges.
    Microcompact,
    /// Context collapse: compress context, originals preserved in VSA.
    Collapse,
    /// Auto-compact: VSA vector summary (cheap alternative to LLM summary).
    AutoCompact,
    /// Blocking: only manual /compact works.
    Blocking,
}

impl CompactionTier {
    pub fn name(&self) -> &'static str {
        match self {
            CompactionTier::Snip => "snip",
            CompactionTier::Microcompact => "microcompact",
            CompactionTier::Collapse => "collapse",
            CompactionTier::AutoCompact => "auto-compact",
            CompactionTier::Blocking => "blocking",
        }
    }

    pub fn cost_rank(&self) -> u8 {
        match self {
            CompactionTier::Snip => 0,
            CompactionTier::Microcompact => 1,
            CompactionTier::Collapse => 2,
            CompactionTier::AutoCompact => 3,
            CompactionTier::Blocking => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompactionReport {
    pub tier: CompactionTier,
    pub bytes_freed: u64,
    pub info_loss: InfoLossLevel,
    pub items_removed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoLossLevel {
    None,
    Low,
    Medium,
    High,
    Total,
}

impl InfoLossLevel {
    pub fn name(&self) -> &'static str {
        match self {
            InfoLossLevel::None => "none",
            InfoLossLevel::Low => "low",
            InfoLossLevel::Medium => "medium",
            InfoLossLevel::High => "high",
            InfoLossLevel::Total => "total",
        }
    }
}

pub trait Compactable {
    /// Estimate current byte size of this context
    fn estimated_bytes(&self) -> u64;
    /// Apply a compaction tier, returns report
    fn compact(&mut self, tier: CompactionTier) -> CompactionReport;
}

/// Tracks compaction history for monitoring
#[derive(Debug, Clone)]
pub struct CompactionHistory {
    entries: VecDeque<CompactionReport>,
    max_entries: usize,
}

impl CompactionHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }

    pub fn push(&mut self, report: CompactionReport) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(report);
    }

    pub fn total_bytes_freed(&self) -> u64 {
        self.entries.iter().map(|e| e.bytes_freed).sum()
    }

    pub fn last_report(&self) -> Option<&CompactionReport> {
        self.entries.back()
    }
}

impl Default for CompactionHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

pub struct CompactionPipeline {
    history: CompactionHistory,
    auto_compact_threshold_bytes: u64,
    current_tier: CompactionTier,
}

impl CompactionPipeline {
    pub fn new(threshold_bytes: u64) -> Self {
        Self {
            history: CompactionHistory::default(),
            auto_compact_threshold_bytes: threshold_bytes,
            current_tier: CompactionTier::Snip,
        }
    }

    /// Run compaction on a Compactable, auto-escalating tiers if needed.
    /// Starts at Snip, escalates until bytes_freed < threshold.
    pub fn compress<T: Compactable>(&mut self, target: &mut T) -> Vec<CompactionReport> {
        let mut reports = Vec::new();
        let tiers = [
            CompactionTier::Snip,
            CompactionTier::Microcompact,
            CompactionTier::Collapse,
            CompactionTier::AutoCompact,
        ];

        for &tier in &tiers {
            let before = target.estimated_bytes();
            if before < self.auto_compact_threshold_bytes {
                break;
            }
            let report = target.compact(tier);
            self.history.push(report.clone());
            self.current_tier = tier;
            reports.push(report);
        }

        reports
    }

    pub fn history(&self) -> &CompactionHistory {
        &self.history
    }

    pub fn current_tier(&self) -> CompactionTier {
        self.current_tier
    }

    pub fn needs_blocking(&self) -> bool {
        self.current_tier == CompactionTier::Blocking
    }

    pub fn set_threshold(&mut self, bytes: u64) {
        self.auto_compact_threshold_bytes = bytes;
    }
}

impl Default for CompactionPipeline {
    fn default() -> Self {
        Self::new(40_000) // 40KB default threshold
    }
}
