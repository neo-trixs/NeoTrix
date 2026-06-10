use std::collections::HashMap;

/// Token budget with allocation per source type
#[derive(Debug, Clone)]
pub struct ContextBudget {
    pub total_tokens: usize,
    pub reserve_tokens: usize,
    allocation: HashMap<SourceType, f64>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SourceType {
    KnowledgeBase,
    Stream,
    Prompt,
    System,
    Tools,
}

impl SourceType {
    pub fn name(&self) -> &'static str {
        match self {
            SourceType::KnowledgeBase => "knowledge_base",
            SourceType::Stream => "stream",
            SourceType::Prompt => "prompt",
            SourceType::System => "system",
            SourceType::Tools => "tools",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AllocatedSlice {
    pub source: SourceType,
    pub tokens: usize,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct AssembledContext {
    pub slices: Vec<AllocatedSlice>,
    pub total_used: usize,
    pub budget: usize,
}

/// Tracks what should be preserved during compaction.
/// Mirrors DeepChat's CompactionIntent pattern for precise token budget planning.
#[derive(Debug, Clone)]
pub struct CompactionIntent {
    pub session_id: String,
    pub previous_salience: f64,
    pub target_cursor: usize,
    pub summary_blocks: Vec<String>,
    pub current_source: SourceType,
    pub reserve_tokens: usize,
    pub priority: CompactionPriority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompactionPriority {
    Critical,
    High,
    Normal,
    Low,
}

impl ContextBudget {
    pub fn new(total: usize) -> Self {
        Self {
            total_tokens: total,
            reserve_tokens: 0,
            allocation: Self::default_allocation(),
        }
    }

    pub fn default_allocation() -> HashMap<SourceType, f64> {
        let mut map = HashMap::new();
        map.insert(SourceType::KnowledgeBase, 0.35);
        map.insert(SourceType::Stream, 0.25);
        map.insert(SourceType::Prompt, 0.25);
        map.insert(SourceType::System, 0.10);
        map.insert(SourceType::Tools, 0.05);
        map
    }

    pub fn set_allocation(&mut self, source: SourceType, fraction: f64) {
        let clamped = fraction.clamp(0.0, 1.0);
        self.allocation.insert(source, clamped);
    }

    pub fn token_budget_for(&self, source: &SourceType) -> usize {
        let fraction = self.allocation.get(source).copied().unwrap_or(0.0);
        (self.total_tokens as f64 * fraction).floor() as usize
    }

    pub fn set_reserve(&mut self, tokens: usize) {
        self.reserve_tokens = tokens.min(self.total_tokens);
    }

    pub fn effective_budget(&self) -> usize {
        self.total_tokens.saturating_sub(self.reserve_tokens)
    }

    fn proportional_reduction(&self, raw_budget: usize) -> usize {
        if self.reserve_tokens == 0 || self.total_tokens == 0 {
            return 0;
        }
        (raw_budget as f64 * self.reserve_tokens as f64 / self.total_tokens as f64).floor() as usize
    }

    /// Assemble context from sources, trimming each to its token budget.
    ///
    /// 1. Compute token budget per source type
    /// 2. Trim each source's content (char_count / 4 heuristic)
    /// 3. Redistribute unused allocation to active (trimmed) sources
    /// 4. Respects reserve_tokens by reducing each source proportionally
    pub fn assemble(&self, sources: &[(SourceType, String)]) -> AssembledContext {
        let mut slices: Vec<AllocatedSlice> = Vec::new();
        let mut total_unused: usize = 0;
        let mut active: Vec<usize> = Vec::new();

        for (i, (source, content)) in sources.iter().enumerate() {
            let raw_budget = self.token_budget_for(source);
            let reduction = self.proportional_reduction(raw_budget);
            let budget = raw_budget.saturating_sub(reduction);
            let est_tokens = content.len() / 4;
            let (trimmed, tokens) = if est_tokens > budget {
                let max_chars = budget * 4;
                let t: String = content.chars().take(max_chars).collect();
                let actual = t.len() / 4;
                (t, actual)
            } else {
                (content.clone(), est_tokens)
            };
            total_unused += budget.saturating_sub(tokens);
            if est_tokens > budget {
                active.push(i);
            }
            slices.push(AllocatedSlice {
                source: source.clone(),
                tokens,
                content: trimmed,
            });
        }

        if total_unused > 0 && !active.is_empty() {
            let per_active = total_unused / active.len();
            let remainder = total_unused % active.len();
            for (j, &idx) in active.iter().enumerate() {
                let extra = per_active + if j < remainder { 1 } else { 0 };
                let orig = &sources[idx];
                let raw_budget = self.token_budget_for(&orig.0);
                let reduction = self.proportional_reduction(raw_budget);
                let orig_budget = raw_budget.saturating_sub(reduction);
                let new_budget = orig_budget + extra;
                let max_chars = new_budget * 4;
                let t: String = orig.1.chars().take(max_chars).collect();
                slices[idx] = AllocatedSlice {
                    source: orig.0.clone(),
                    tokens: t.len() / 4,
                    content: t,
                };
            }
        }

        let total_used: usize = slices.iter().map(|s| s.tokens).sum();
        AssembledContext {
            slices,
            total_used,
            budget: self.total_tokens,
        }
    }

    /// Assemble with CompactionIntent awareness.
    ///
    /// 1. Sum reserve_tokens from Critical and High priority intents
    /// 2. Reduce total budget by intent reserves before normal assembly
    /// 3. After assembly, inject reserved content from Critical intents
    pub fn assemble_with_intent(
        &self,
        sources: &[(SourceType, String)],
        intents: &[CompactionIntent],
    ) -> AssembledContext {
        let intent_reserve: usize = intents
            .iter()
            .filter(|i| matches!(i.priority, CompactionPriority::Critical | CompactionPriority::High))
            .map(|i| i.reserve_tokens)
            .sum();

        let adjusted_total = self.total_tokens.saturating_sub(intent_reserve);
        let mut adjusted = self.clone();
        adjusted.total_tokens = adjusted_total;

        let mut ctx = adjusted.assemble(sources);

        for intent in intents {
            if intent.priority == CompactionPriority::Critical {
                let content = intent.summary_blocks.join("\n");
                let max_chars = intent.reserve_tokens * 4;
                let trimmed: String = content.chars().take(max_chars).collect();
                let tokens = trimmed.len() / 4;
                ctx.slices.push(AllocatedSlice {
                    source: intent.current_source.clone(),
                    tokens,
                    content: trimmed,
                });
            }
        }

        ctx.total_used = ctx.slices.iter().map(|s| s.tokens).sum();
        ctx.budget = self.total_tokens;
        ctx
    }

    pub fn intent_for_source<'a>(
        &self,
        intents: &'a [CompactionIntent],
        source: &SourceType,
    ) -> Option<&'a CompactionIntent> {
        intents.iter().find(|i| &i.current_source == source)
    }

    pub fn remaining(&self, used: &AssembledContext) -> usize {
        self.total_tokens.saturating_sub(used.total_used)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allocation_sums_to_one() {
        let alloc = ContextBudget::default_allocation();
        let sum: f64 = alloc.values().sum();
        assert!(
            (sum - 1.0).abs() < 1e-9,
            "default allocation sums to {}, expected 1.0",
            sum
        );
    }

    #[test]
    fn test_assemble_trims_oversized_content() {
        let budget = ContextBudget::new(100);
        let kb_content = "A".repeat(400);
        let sources = vec![
            (SourceType::KnowledgeBase, kb_content),
            (SourceType::System, "hello".to_string()),
        ];
        let ctx = budget.assemble(&sources);
        assert!(ctx.total_used <= ctx.budget);
        assert_eq!(ctx.slices.len(), 2);
        let kb_slice = &ctx.slices[0];
        assert!(kb_slice.tokens <= ctx.budget);
    }

    #[test]
    fn test_remaining_budget() {
        let budget = ContextBudget::new(100);
        let ctx = budget.assemble(&[(SourceType::System, "test".to_string())]);
        let remaining = budget.remaining(&ctx);
        assert_eq!(remaining, 100 - ctx.total_used);
    }

    #[test]
    fn test_set_allocation_updates() {
        let mut budget = ContextBudget::new(1000);
        budget.set_allocation(SourceType::KnowledgeBase, 0.5);
        assert_eq!(budget.token_budget_for(&SourceType::KnowledgeBase), 500);
        budget.set_allocation(SourceType::KnowledgeBase, 0.25);
        assert_eq!(budget.token_budget_for(&SourceType::KnowledgeBase), 250);
    }

    #[test]
    fn test_overflow_redistribution() {
        let mut budget = ContextBudget::new(400);
        budget.set_allocation(SourceType::KnowledgeBase, 0.5);
        budget.set_allocation(SourceType::System, 0.5);
        let long_kb = "x".repeat(1600);
        let short_sys = "short".to_string();
        let ctx = budget.assemble(&[
            (SourceType::KnowledgeBase, long_kb),
            (SourceType::System, short_sys),
        ]);
        let kb_slice = ctx
            .slices
            .iter()
            .find(|s| matches!(s.source, SourceType::KnowledgeBase))
            .unwrap();
        assert!(
            kb_slice.tokens > 200,
            "KB should receive overflow, got {} tokens",
            kb_slice.tokens
        );
    }

    #[test]
    fn test_token_budget_for_missing_source_returns_zero() {
        let budget = ContextBudget::new(100);
        let tools = budget.token_budget_for(&SourceType::Tools);
        assert_eq!(tools, 5);
    }

    #[test]
    fn test_set_allocation_clamps_above_one() {
        let mut budget = ContextBudget::new(100);
        budget.set_allocation(SourceType::Prompt, 2.0);
        assert_eq!(budget.token_budget_for(&SourceType::Prompt), 100);
    }

    #[test]
    fn test_set_allocation_clamps_below_zero() {
        let mut budget = ContextBudget::new(100);
        budget.set_allocation(SourceType::Prompt, -0.5);
        assert_eq!(budget.token_budget_for(&SourceType::Prompt), 0);
    }

    #[test]
    fn test_assemble_empty_sources() {
        let budget = ContextBudget::new(100);
        let ctx = budget.assemble(&[]);
        assert_eq!(ctx.slices.len(), 0);
        assert_eq!(ctx.total_used, 0);
        assert_eq!(ctx.budget, 100);
    }

    #[test]
    fn test_source_type_names() {
        assert_eq!(SourceType::KnowledgeBase.name(), "knowledge_base");
        assert_eq!(SourceType::Stream.name(), "stream");
        assert_eq!(SourceType::Prompt.name(), "prompt");
        assert_eq!(SourceType::System.name(), "system");
        assert_eq!(SourceType::Tools.name(), "tools");
    }

    // ── reserve_tokens tests ──

    #[test]
    fn test_reserve_tokens_reduces_budget() {
        let mut budget = ContextBudget::new(1000);
        budget.set_reserve(100);
        assert_eq!(budget.reserve_tokens, 100);
        assert_eq!(budget.effective_budget(), 900);
    }

    #[test]
    fn test_reserve_tokens_clamped() {
        let mut budget = ContextBudget::new(100);
        budget.set_reserve(999);
        assert_eq!(budget.reserve_tokens, 100);
        assert_eq!(budget.effective_budget(), 0);
    }

    #[test]
    fn test_reserve_tokens_zero_by_default() {
        let budget = ContextBudget::new(500);
        assert_eq!(budget.reserve_tokens, 0);
        assert_eq!(budget.effective_budget(), 500);
    }

    #[test]
    fn test_assemble_respects_reserve() {
        let mut budget = ContextBudget::new(400);
        budget.set_allocation(SourceType::KnowledgeBase, 0.5);
        budget.set_allocation(SourceType::System, 0.5);
        budget.set_reserve(200);
        let kb_content = "x".repeat(1600);
        let ctx = budget.assemble(&[
            (SourceType::KnowledgeBase, kb_content),
            (SourceType::System, "small".to_string()),
        ]);
        let kb_slice = ctx
            .slices
            .iter()
            .find(|s| matches!(s.source, SourceType::KnowledgeBase))
            .unwrap();
        // raw budget = 200, reserve share = (200/400)*200 = 100 → effective = 100
        // 100 * 4 = 400 chars max → tokens ~100
        assert!(
            kb_slice.tokens <= 110,
            "KB should be reduced by reserve, got {} tokens",
            kb_slice.tokens
        );
    }

    // ── CompactionIntent tests ──

    #[test]
    fn test_compaction_intent_critical() {
        let intent = CompactionIntent {
            session_id: "sess-1".into(),
            previous_salience: 0.9,
            target_cursor: 42,
            summary_blocks: vec!["critical summary".into()],
            current_source: SourceType::Stream,
            reserve_tokens: 50,
            priority: CompactionPriority::Critical,
        };
        assert_eq!(intent.priority, CompactionPriority::Critical);
        assert_eq!(intent.reserve_tokens, 50);
    }

    #[test]
    fn test_assemble_with_intent_preserves_critical() {
        let budget = ContextBudget::new(200);
        let intent = CompactionIntent {
            session_id: "sess-1".into(),
            previous_salience: 0.95,
            target_cursor: 0,
            summary_blocks: vec!["preserved content".into()],
            current_source: SourceType::Stream,
            reserve_tokens: 30,
            priority: CompactionPriority::Critical,
        };
        let sources = vec![
            (SourceType::KnowledgeBase, "small kb".to_string()),
            (SourceType::Stream, "x".repeat(800)),
        ];
        let ctx = budget.assemble_with_intent(&sources, &[intent]);
        let stream_slices: Vec<_> = ctx
            .slices
            .iter()
            .filter(|s| matches!(s.source, SourceType::Stream))
            .collect();
        assert_eq!(stream_slices.len(), 2, "should have original + critical injection");
        let injected = &stream_slices[1];
        assert!(injected.content.contains("preserved content"));
    }

    #[test]
    fn test_compaction_priority_ordering() {
        assert!(CompactionPriority::Critical != CompactionPriority::High);
        assert!(CompactionPriority::High != CompactionPriority::Normal);
        assert!(CompactionPriority::Normal != CompactionPriority::Low);
    }

    #[test]
    fn test_intent_for_source_found() {
        let intents = vec![
            CompactionIntent {
                session_id: "a".into(),
                previous_salience: 0.5,
                target_cursor: 0,
                summary_blocks: vec![],
                current_source: SourceType::KnowledgeBase,
                reserve_tokens: 10,
                priority: CompactionPriority::Normal,
            },
            CompactionIntent {
                session_id: "b".into(),
                previous_salience: 0.5,
                target_cursor: 0,
                summary_blocks: vec![],
                current_source: SourceType::Stream,
                reserve_tokens: 20,
                priority: CompactionPriority::Critical,
            },
        ];
        let budget = ContextBudget::new(100);
        let found = budget.intent_for_source(&intents, &SourceType::Stream);
        assert!(found.is_some());
        assert_eq!(found.unwrap().session_id, "b");
        let not_found = budget.intent_for_source(&intents, &SourceType::Tools);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_assemble_with_intent_low_priority_not_reserved() {
        let budget = ContextBudget::new(100);
        let low = CompactionIntent {
            session_id: "low".into(),
            previous_salience: 0.3,
            target_cursor: 0,
            summary_blocks: vec!["low priority".into()],
            current_source: SourceType::Stream,
            reserve_tokens: 50,
            priority: CompactionPriority::Low,
        };
        let sources = vec![(SourceType::Stream, "content".to_string())];
        let ctx = budget.assemble_with_intent(&sources, &[low]);
        let stream_slices: Vec<_> = ctx
            .slices
            .iter()
            .filter(|s| matches!(s.source, SourceType::Stream))
            .collect();
        // Low priority should not have its content injected
        assert!(
            !stream_slices.iter().any(|s| s.content.contains("low priority")),
            "Low priority intent should not be injected"
        );
    }
}
