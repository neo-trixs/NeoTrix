use super::context_budget::{
    AssembledContext, BudgetSourceType, CompactionIntent, CompactionPriority,
};
use super::context_gatherer::{ContextFragment, ContextGatherer, ContextSourceMeta};
use super::context_predictor::ContextPredictor;
use super::working_memory::{WorkingMemory, WorkingMemoryItem};

pub struct ContextOS {
    pub gatherer: ContextGatherer,
    pub working_memory: WorkingMemory,
    pub predictor: ContextPredictor,
    last_assembly: Option<AssembledContext>,
    total_assemblies: u64,
    total_tokens_processed: u64,
    cross_synth_enabled: bool,
    temporal_fusion_enabled: bool,
}

impl ContextOS {
    pub fn new(total_tokens: usize) -> Self {
        Self {
            gatherer: ContextGatherer::new(total_tokens),
            working_memory: WorkingMemory::default(),
            predictor: ContextPredictor::new(),
            last_assembly: None,
            total_assemblies: 0,
            total_tokens_processed: 0,
            cross_synth_enabled: true,
            temporal_fusion_enabled: true,
        }
    }

    pub fn gather(&mut self, intents: &[CompactionIntent]) -> AssembledContext {
        let assembled = self.gatherer.gather_and_synthesize(intents);
        self.total_assemblies += 1;
        self.total_tokens_processed += assembled.total_used as u64;
        self.last_assembly = Some(assembled.clone());
        assembled
    }

    pub fn gather_with_working_memory(&mut self, intents: &[CompactionIntent]) -> AssembledContext {
        let wm_content = self.working_memory.current_content();
        let wm_text: String = wm_content
            .iter()
            .map(|(item, salience)| {
                format!("[WM:{} salience={:.2}] {}", item.id, salience, item.content)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let _wm_len = wm_text.len();
        let wm_len = wm_text.len();
        let wm_fragment = ContextFragment {
            source: super::context_budget::BudgetSourceType::Stream,
            content: wm_text,
            meta: ContextSourceMeta {
                source_type: super::context_budget::BudgetSourceType::Stream,
                label: "working_memory".into(),
                priority: 0.9,
                staleness: std::time::Duration::from_secs(0),
                byte_size: wm_len,
                semantic_density: 0.95,
            },
            vsa_fingerprint: None,
        };

        let mut all_fragments = self.gatherer.gather_all(intents).fragments;
        all_fragments.push(wm_fragment);

        let raw_sources: Vec<(BudgetSourceType, String)> = all_fragments
            .iter()
            .map(|f| (f.source.clone(), f.content.clone()))
            .collect();

        let assembled = if intents.is_empty() {
            let budget = &self.gatherer.gather_all(intents);
            self.gatherer.synthesize(budget, intents)
        } else {
            let budget = ContextBudgetWrapper::from_gatherer(&self.gatherer);
            budget.assemble_with_intent_impl(&raw_sources, intents)
        };

        self.total_assemblies += 1;
        self.total_tokens_processed += assembled.total_used as u64;
        self.last_assembly = Some(assembled.clone());
        assembled
    }

    pub fn update_working_memory(
        &mut self,
        content: String,
        importance: f64,
        vsa: Option<Vec<u8>>,
    ) {
        self.working_memory.push(content, importance, vsa);
    }

    /// Like update_working_memory but also records the query context for prediction.
    /// `domain` and `intent` describe the user's current focus area.
    pub fn update_working_memory_with_domain(
        &mut self,
        content: String,
        importance: f64,
        vsa: Option<Vec<u8>>,
        domain: String,
        intent: String,
    ) {
        self.predictor.record_query(content.clone(), domain, intent);
        self.working_memory.push(content, importance, vsa);
    }

    /// Record a user query for ACI pattern learning without adding to working memory.
    pub fn record_user_query(&mut self, query: String, domain: String, intent: String) {
        self.predictor.record_query(query, domain, intent);
    }

    /// Predict next domains and pre-fetch context for them.
    /// Returns the list of pre-fetched context entries.
    pub fn predict_and_fetch(&mut self) -> Vec<(String, f64)> {
        let predicted = self.predictor.predict_next_domains();
        if !predicted.is_empty() {
            self.predictor.pre_fetch(predicted.clone());
        }
        predicted
    }

    pub fn chunk_working_memory(
        &mut self,
        ids: &[u64],
        label: String,
    ) -> Option<WorkingMemoryItem> {
        self.working_memory.chunk(ids, label)
    }

    pub fn last_assembly(&self) -> Option<&AssembledContext> {
        self.last_assembly.as_ref()
    }

    pub fn stats(&self) -> ContextOSStats {
        ContextOSStats {
            total_assemblies: self.total_assemblies,
            total_tokens_processed: self.total_tokens_processed,
            wm_load: self.working_memory.load(),
            wm_coherence: self.working_memory.coherence(),
            wm_count: self.working_memory.item_count(),
            total_gathers: self.gatherer.stats().total_gathers,
            predictor_active: self.predictor.active(),
            prediction_confidence: self.predictor.confidence(),
            predictor_query_count: self.predictor.query_count(),
            predictor_prefetch_count: self.predictor.prefetch_count(),
            aci_bonus: self.predictor.aci_bonus(),
        }
    }

    pub fn enable_cross_synthesis(&mut self, enabled: bool) {
        self.cross_synth_enabled = enabled;
    }

    pub fn enable_temporal_fusion(&mut self, enabled: bool) {
        self.temporal_fusion_enabled = enabled;
    }
}

pub struct ContextOSStats {
    pub total_assemblies: u64,
    pub total_tokens_processed: u64,
    pub wm_load: f64,
    pub wm_coherence: f64,
    pub wm_count: usize,
    pub total_gathers: u64,
    pub predictor_active: bool,
    pub prediction_confidence: f64,
    pub predictor_query_count: usize,
    pub predictor_prefetch_count: usize,
    pub aci_bonus: f64,
}

struct ContextBudgetWrapper {
    total_tokens: usize,
    reserve_tokens: usize,
}

impl ContextBudgetWrapper {
    fn from_gatherer(_gatherer: &ContextGatherer) -> Self {
        Self {
            total_tokens: 4096,
            reserve_tokens: 0,
        }
    }

    fn assemble_with_intent_impl(
        &self,
        sources: &[(BudgetSourceType, String)],
        intents: &[CompactionIntent],
    ) -> AssembledContext {
        let intent_reserve: usize = intents
            .iter()
            .filter(|i| {
                matches!(
                    i.priority,
                    CompactionPriority::Critical | CompactionPriority::High
                )
            })
            .map(|i| i.reserve_tokens)
            .sum();
        let effective = self
            .total_tokens
            .saturating_sub(intent_reserve)
            .saturating_sub(self.reserve_tokens);
        let tokens_per_source = if sources.is_empty() {
            0
        } else {
            effective / sources.len()
        };

        let mut slices = Vec::new();
        let mut total_used = 0usize;

        for (st, content) in sources {
            let budget = if st == &BudgetSourceType::Stream
                && intents
                    .iter()
                    .any(|i| i.priority == CompactionPriority::Critical)
            {
                tokens_per_source + intent_reserve
            } else {
                tokens_per_source
            };
            let max_chars = budget * 4;
            let trimmed: String = content.chars().take(max_chars).collect();
            let used = trimmed.len() / 4;
            total_used += used;
            slices.push(super::context_budget::AllocatedSlice {
                source: st.clone(),
                tokens: used,
                content: trimmed,
            });
        }

        AssembledContext {
            slices,
            total_used,
            budget: self.total_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_os_new() {
        let os = ContextOS::new(4000);
        assert_eq!(os.total_assemblies, 0);
        assert!(os.cross_synth_enabled);
        assert!(os.temporal_fusion_enabled);
    }

    #[test]
    fn test_gather_basic() {
        let mut os = ContextOS::new(1000);
        let ctx = os.gather(&[]);
        assert_eq!(ctx.budget, 1000);
        assert!(os.last_assembly().is_some());
    }

    #[test]
    fn test_update_working_memory() {
        let mut os = ContextOS::new(1000);
        os.update_working_memory("critical fact".into(), 0.95, None);
        assert_eq!(os.working_memory.item_count(), 1);
    }

    #[test]
    fn test_stats() {
        let mut os = ContextOS::new(4000);
        os.gather(&[]);
        let stats = os.stats();
        assert_eq!(stats.total_assemblies, 1);
        assert!(stats.wm_load >= 0.0);
    }

    #[test]
    fn test_gather_with_wm() {
        let mut os = ContextOS::new(2000);
        os.update_working_memory("working data".into(), 0.9, None);
        let ctx = os.gather_with_working_memory(&[]);
        assert!(ctx.total_used <= ctx.budget);
    }

    #[test]
    fn test_chunk_wm() {
        let mut os = ContextOS::new(1000);
        os.update_working_memory("a".into(), 0.7, None);
        os.update_working_memory("b".into(), 0.6, None);
        let chunk = os.chunk_working_memory(&[1, 2], "pair".into());
        assert!(chunk.is_some());
    }

    #[test]
    fn test_cross_synthesis_flag() {
        let mut os = ContextOS::new(4000);
        os.enable_cross_synthesis(false);
        assert!(!os.cross_synth_enabled);
        os.enable_cross_synthesis(true);
        assert!(os.cross_synth_enabled);
    }
}
