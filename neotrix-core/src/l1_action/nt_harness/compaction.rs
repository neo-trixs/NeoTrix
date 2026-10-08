//! Lossless context compaction -- inspired by fast-jev-compaction.
//! Every tool call and result is scored; stale ones are dropped or truncated;
//! everything kept stays verbatim. No summarization, no loss.

/// Decision for a tool call during compaction
#[derive(Debug, Clone)]
pub enum CompactionDecision {
    /// Keep call and result verbatim
    Keep,
    /// Keep call but truncate result to head + note
    TruncateResult { head_chars: usize },
    /// Remove call and result entirely
    Remove,
}

/// Score for a tool call
#[derive(Debug, Clone)]
pub struct ToolCallScore {
    pub tool_use_id: String,
    pub tool_name: String,
    pub keep_call_probability: f64,
    pub keep_result_probability: f64,
    pub decision: CompactionDecision,
}

/// Result of a compaction pass
#[derive(Debug, Clone)]
pub struct CompactionResult {
    pub decisions: Vec<ToolCallScore>,
    pub original_count: usize,
    pub kept_count: usize,
    pub removed_count: usize,
    pub truncated_count: usize,
    pub reduction_ratio: f64,
}

/// Configuration for lossless compaction
#[derive(Debug, Clone)]
pub struct CompactionConfig {
    /// Minimum keep probability threshold
    pub keep_threshold: f64,
    /// Number of recent messages to never touch
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub preserve_recent: usize,
    /// Max characters to keep from truncated results
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub truncate_head_chars: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            keep_threshold: 0.5,
            preserve_recent: 6,
            truncate_head_chars: 300,
        }
    }
}

/// Lossless compactor -- scores tool calls and decides keep/truncate/remove
pub struct LosslessCompactor {
    config: CompactionConfig,
}

impl LosslessCompactor {
    pub fn new(config: CompactionConfig) -> Self {
        Self { config }
    }

    /// Score a batch of tool calls and return compaction decisions
    pub fn score_calls(&self, tool_call_count: usize) -> CompactionResult {
        // Placeholder -- will be wired to TypeSafe Jev API
        let decisions: Vec<ToolCallScore> = (0..tool_call_count)
            .map(|i| ToolCallScore {
                tool_use_id: format!("tool_{}", i),
                tool_name: "unknown".into(),
                keep_call_probability: 0.5,
                keep_result_probability: 0.5,
                decision: CompactionDecision::Keep,
            })
            .collect();

        let kept = decisions.iter().filter(|d| matches!(d.decision, CompactionDecision::Keep)).count();
        let removed = decisions.iter().filter(|d| matches!(d.decision, CompactionDecision::Remove)).count();
        let truncated = decisions.iter().filter(|d| matches!(d.decision, CompactionDecision::TruncateResult { .. })).count();

        CompactionResult {
            decisions,
            original_count: tool_call_count,
            kept_count: kept,
            removed_count: removed,
            truncated_count: truncated,
            reduction_ratio: if tool_call_count > 0 {
                (tool_call_count - kept) as f64 / tool_call_count as f64
            } else {
                0.0
            },
        }
    }

    /// Check if compaction is worth doing (reduction > threshold)
    pub fn is_worth_compacting(&self, result: &CompactionResult) -> bool {
        result.reduction_ratio > 0.25
    }
}
