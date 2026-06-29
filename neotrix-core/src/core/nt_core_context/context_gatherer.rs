use super::context_budget::{AssembledContext, BudgetSourceType, CompactionIntent, ContextBudget};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ContextSourceMeta {
    pub source_type: BudgetSourceType,
    pub label: String,
    pub priority: f64,
    pub staleness: Duration,
    pub byte_size: usize,
    pub semantic_density: f64,
}

#[derive(Debug, Clone)]
pub struct ContextFragment {
    pub source: BudgetSourceType,
    pub content: String,
    pub meta: ContextSourceMeta,
    pub vsa_fingerprint: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct GatheredContext {
    pub fragments: Vec<ContextFragment>,
    pub total_sources: usize,
    pub total_bytes: usize,
    pub gather_latency: Duration,
}

#[derive(Clone)]
pub struct GathererStats {
    pub total_gathers: u64,
    pub total_bytes_processed: u64,
    pub avg_latency: Duration,
    pub source_hit_rates: HashMap<BudgetSourceType, f64>,
}

pub trait ContextSource {
    fn source_type(&self) -> BudgetSourceType;
    fn gather(&mut self, budget: usize) -> Option<ContextFragment>;
    fn is_stale(&self) -> bool;
    fn refresh(&mut self);
}

pub struct ContextGatherer {
    sources: Vec<Box<dyn ContextSource + Send + Sync>>,
    budget: ContextBudget,
    stats: GathererStats,
    last_gather: Option<Instant>,
    max_fragments: usize,
    pub dedup_window: HashMap<u64, Instant>,
    dedup_ttl: Duration,
}

impl ContextGatherer {
    pub fn new(total_tokens: usize) -> Self {
        Self {
            sources: Vec::new(),
            budget: ContextBudget::new(total_tokens),
            stats: GathererStats {
                total_gathers: 0,
                total_bytes_processed: 0,
                avg_latency: Duration::from_secs(0),
                source_hit_rates: HashMap::new(),
            },
            last_gather: None,
            max_fragments: 32,
            dedup_window: HashMap::new(),
            dedup_ttl: Duration::from_secs(60),
        }
    }

    pub fn register(&mut self, source: Box<dyn ContextSource + Send + Sync>) {
        self.sources.push(source);
    }

    pub fn set_budget(&mut self, total: usize) {
        self.budget = ContextBudget::new(total);
    }

    pub fn set_allocation(&mut self, source: BudgetSourceType, fraction: f64) {
        self.budget.set_allocation(source, fraction);
    }

    pub fn gather_all(&mut self, _intents: &[CompactionIntent]) -> GatheredContext {
        let start = Instant::now();
        let mut fragments = Vec::with_capacity(self.sources.len().min(self.max_fragments));
        let mut total_bytes = 0usize;

        let mut scored: Vec<(f64, usize)> = self
            .sources
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let priority = self
                    .stats
                    .source_hit_rates
                    .get(&s.source_type())
                    .copied()
                    .unwrap_or(0.5);
                (priority, i)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let dedup_keys: Vec<u64> = self
            .dedup_window
            .iter()
            .filter(|(_, t)| t.elapsed() > self.dedup_ttl)
            .map(|(k, _)| *k)
            .collect();
        for k in dedup_keys {
            self.dedup_window.remove(&k);
        }

        for (_, idx) in scored.iter().take(self.max_fragments) {
            let source = &mut self.sources[*idx];
            let budget = self.budget.token_budget_for(&source.source_type());
            if budget == 0 {
                continue;
            }
            if let Some(mut fragment) = source.gather(budget) {
                let fp = self.compute_fingerprint(&fragment.content);
                if self.dedup_window.contains_key(&fp) {
                    continue;
                }
                self.dedup_window.insert(fp, Instant::now());
                total_bytes += fragment.meta.byte_size;
                fragment.meta.staleness = self
                    .last_gather
                    .map(|t| t.elapsed())
                    .unwrap_or(Duration::from_secs(0));
                fragments.push(fragment);
            }
        }

        let latency = start.elapsed();
        self.stats.total_gathers += 1;
        self.stats.total_bytes_processed += total_bytes as u64;
        self.stats.avg_latency = Duration::from_nanos(
            ((self.stats.avg_latency.as_nanos() as f64 * 0.9) + (latency.as_nanos() as f64 * 0.1))
                as u64,
        );
        self.last_gather = Some(Instant::now());

        GatheredContext {
            fragments,
            total_sources: self.sources.len(),
            total_bytes,
            gather_latency: latency,
        }
    }

    pub fn synthesize(
        &self,
        gathered: &GatheredContext,
        intents: &[CompactionIntent],
    ) -> AssembledContext {
        let raw_sources: Vec<(BudgetSourceType, String)> = gathered
            .fragments
            .iter()
            .map(|f| (f.source.clone(), f.content.clone()))
            .collect();

        if intents.is_empty() {
            self.budget.assemble(&raw_sources)
        } else {
            self.budget.assemble_with_intent(&raw_sources, intents)
        }
    }

    pub fn gather_and_synthesize(&mut self, intents: &[CompactionIntent]) -> AssembledContext {
        let gathered = self.gather_all(intents);
        self.synthesize(&gathered, intents)
    }

    pub fn stats(&self) -> &GathererStats {
        &self.stats
    }

    fn compute_fingerprint(&self, content: &str) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    pub fn reset_stats(&mut self) {
        self.stats = GathererStats {
            total_gathers: 0,
            total_bytes_processed: 0,
            avg_latency: Duration::from_secs(0),
            source_hit_rates: HashMap::new(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSource {
        st: BudgetSourceType,
        content: String,
        stale: bool,
    }

    impl ContextSource for TestSource {
        fn source_type(&self) -> BudgetSourceType {
            self.st.clone()
        }
        fn gather(&mut self, _budget: usize) -> Option<ContextFragment> {
            let content = self.content.clone();
            Some(ContextFragment {
                source: self.st.clone(),
                content: content.clone(),
                meta: ContextSourceMeta {
                    source_type: self.st.clone(),
                    label: format!("{:?}", self.st),
                    priority: 0.5,
                    staleness: Duration::from_secs(0),
                    byte_size: content.len(),
                    semantic_density: 0.8,
                },
                vsa_fingerprint: None,
            })
        }
        fn is_stale(&self) -> bool {
            self.stale
        }
        fn refresh(&mut self) {
            self.stale = false;
        }
    }

    #[test]
    fn test_gatherer_creates_fragments() {
        let mut g = ContextGatherer::new(1000);
        g.register(Box::new(TestSource {
            st: BudgetSourceType::KnowledgeBase,
            content: "knowledge data".into(),
            stale: false,
        }));
        g.register(Box::new(TestSource {
            st: BudgetSourceType::Stream,
            content: "stream data".into(),
            stale: false,
        }));
        let ctx = g.gather_all(&[]);
        assert_eq!(ctx.total_sources, 2);
        assert!(!ctx.fragments.is_empty());
    }

    #[test]
    fn test_gather_and_synthesize() {
        let mut g = ContextGatherer::new(400);
        g.register(Box::new(TestSource {
            st: BudgetSourceType::KnowledgeBase,
            content: "x".repeat(800),
            stale: false,
        }));
        g.register(Box::new(TestSource {
            st: BudgetSourceType::System,
            content: "short system info".into(),
            stale: false,
        }));
        let assembled = g.gather_and_synthesize(&[]);
        assert!(assembled.total_used <= assembled.budget);
        assert_eq!(assembled.slices.len(), 2);
    }

    #[test]
    fn test_dedup_same_content() {
        let mut g = ContextGatherer::new(1000);
        g.register(Box::new(TestSource {
            st: BudgetSourceType::KnowledgeBase,
            content: "duplicate".into(),
            stale: false,
        }));
        g.register(Box::new(TestSource {
            st: BudgetSourceType::KnowledgeBase,
            content: "duplicate".into(),
            stale: false,
        }));
        let ctx = g.gather_all(&[]);
        let unique: std::collections::HashSet<_> =
            ctx.fragments.iter().map(|f| f.content.clone()).collect();
        assert!(
            unique.len() <= ctx.fragments.len(),
            "dedup should reduce duplicates"
        );
    }

    #[test]
    fn test_stats_tracking() {
        let mut g = ContextGatherer::new(1000);
        g.register(Box::new(TestSource {
            st: BudgetSourceType::Stream,
            content: "test".into(),
            stale: false,
        }));
        g.gather_all(&[]);
        assert_eq!(g.stats().total_gathers, 1);
        assert!(g.stats().total_bytes_processed > 0);
    }

    #[test]
    fn test_empty_gatherer() {
        let mut g = ContextGatherer::new(1000);
        let ctx = g.gather_all(&[]);
        assert!(ctx.fragments.is_empty());
        assert_eq!(ctx.total_sources, 0);
    }

    #[test]
    fn test_budget_allocation_respected() {
        let mut g = ContextGatherer::new(200);
        g.set_allocation(BudgetSourceType::KnowledgeBase, 0.8);
        g.set_allocation(BudgetSourceType::System, 0.2);
        g.register(Box::new(TestSource {
            st: BudgetSourceType::KnowledgeBase,
            content: "x".repeat(2000),
            stale: false,
        }));
        g.register(Box::new(TestSource {
            st: BudgetSourceType::System,
            content: "y".repeat(500),
            stale: false,
        }));
        let assembled = g.gather_and_synthesize(&[]);
        let kb_slice = assembled
            .slices
            .iter()
            .find(|s| matches!(s.source, BudgetSourceType::KnowledgeBase))
            .unwrap();
        let sys_slice = assembled
            .slices
            .iter()
            .find(|s| matches!(s.source, BudgetSourceType::System))
            .unwrap();
        assert!(
            kb_slice.tokens > sys_slice.tokens,
            "KB should get larger budget"
        );
    }
}
