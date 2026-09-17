use std::collections::VecDeque;

/// Context window capacity tiers for different model capability levels.
/// Sonnet 5 sets 1M context / 128K output as the new baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextTier {
    /// Default small window (legacy fallback)
    Tiny = 512,
    /// Claude 3/4 era default
    Standard = 128_000,
    /// Large model context (Claude Opus, Gemini 1.5 Pro)
    Large = 200_000,
    /// Modern frontier models (Claude Sonnet 5, GPT-5)
    Frontier = 1_000_000,
}

impl ContextTier {
    pub fn from_capacity(cap: usize) -> Self {
        match cap {
            0..=512 => ContextTier::Tiny,
            513..=128_000 => ContextTier::Standard,
            128_001..=200_000 => ContextTier::Large,
            _ => ContextTier::Frontier,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveUnitKind {
    Observation,
    ReasoningStep,
    Action,
    ActionResult,
    SelfReflection,
    ToolCall,
    GoalUpdate,
    KnowledgeRetrieval,
}

#[derive(Debug, Clone)]
pub struct CognitiveUnit {
    pub id: usize,
    pub kind: CognitiveUnitKind,
    pub content: String,
    pub domain: String,
    pub salience: f64,
    pub timestamp: f64,
    pub meta: Vec<(String, String)>,
    /// Token count estimate for this unit (0 = unknown, computed lazily)
    pub token_estimate: usize,
}

impl CognitiveUnit {
    pub fn new(id: usize, kind: CognitiveUnitKind, content: &str) -> Self {
        Self {
            id,
            kind,
            content: content.to_string(),
            domain: String::new(),
            salience: 0.5,
            timestamp: 0.0,
            meta: Vec::new(),
            token_estimate: 0,
        }
    }

    pub fn with_domain(mut self, domain: &str) -> Self {
        self.domain = domain.to_string();
        self
    }

    pub fn with_salience(mut self, salience: f64) -> Self {
        self.salience = salience;
        self
    }

    pub fn with_token_estimate(mut self, tokens: usize) -> Self {
        self.token_estimate = tokens;
        self
    }

    /// Rough content-based token estimate (chars / 4 for English-like text)
    pub fn estimate_tokens(&self) -> usize {
        if self.token_estimate > 0 {
            return self.token_estimate;
        }
        self.content.len().div_ceil(4)
    }
}

pub struct ContextWindow {
    pub capacity: usize,
    pub tier: ContextTier,
    pub(crate) units: VecDeque<CognitiveUnit>,
    pub(crate) next_id: usize,
    pub(crate) attention_mask: Vec<usize>,
    /// Running token budget tracker: total estimated tokens across all units
    pub total_token_budget: usize,
}

impl ContextWindow {
    pub fn new(capacity: usize) -> Self {
        Self {
            tier: ContextTier::from_capacity(capacity),
            capacity,
            units: VecDeque::with_capacity(capacity.min(16384)),
            next_id: 0,
            attention_mask: Vec::new(),
            total_token_budget: 0,
        }
    }

    /// Create a Frontier-tier context window (1M capacity)
    pub fn frontier() -> Self {
        Self::new(ContextTier::Frontier as usize)
    }

    /// Create a Standard-tier context window (128K capacity)
    pub fn standard() -> Self {
        Self::new(ContextTier::Standard as usize)
    }

    pub fn observe(&mut self, kind: CognitiveUnitKind, content: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let est_tokens = content.len().div_ceil(4);
        let unit = CognitiveUnit::new(id, kind, content).with_token_estimate(est_tokens);
        self.total_token_budget += est_tokens;

        // Evict from front when over capacity (unit-wise or token-wise)
        while self.units.len() >= self.capacity || self.total_token_budget > self.capacity * 4 {
            if let Some(evicted) = self.units.pop_front() {
                self.total_token_budget = self
                    .total_token_budget
                    .saturating_sub(evicted.estimate_tokens());
            } else {
                break;
            }
        }
        self.units.push_back(unit);
        id
    }

    pub fn attend(&mut self, unit_id: usize) {
        if !self.attention_mask.contains(&unit_id) {
            self.attention_mask.push(unit_id);
        }
    }

    pub fn recent(&self, n: usize) -> Vec<&CognitiveUnit> {
        self.units.iter().rev().take(n).collect()
    }

    pub fn attended_context(&self) -> Vec<&CognitiveUnit> {
        let mut result = Vec::new();
        for id in &self.attention_mask {
            if let Some(unit) = self.units.iter().find(|u| u.id == *id) {
                result.push(unit);
            }
        }
        result
    }

    pub fn by_domain(&self, domain: &str) -> Vec<&CognitiveUnit> {
        self.units.iter().filter(|u| u.domain == domain).collect()
    }

    pub fn by_kind(&self, kind: CognitiveUnitKind) -> Vec<&CognitiveUnit> {
        self.units.iter().filter(|u| u.kind == kind).collect()
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    pub fn clear_attention(&mut self) {
        self.attention_mask.clear();
    }

    /// Token budget utilization ratio (0.0 - 1.0+)
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            return 0.0;
        }
        self.total_token_budget as f64 / (self.capacity as f64 * 4.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_window_empty() {
        let w = ContextWindow::new(100);
        assert_eq!(w.len(), 0);
        assert_eq!(w.capacity, 100);
    }

    #[test]
    fn test_observe_adds_unit() {
        let mut w = ContextWindow::new(10);
        let id = w.observe(CognitiveUnitKind::Observation, "test observation");
        assert_eq!(id, 0);
        assert_eq!(w.len(), 1);
        assert_eq!(w.units[0].content, "test observation");
    }

    #[test]
    fn test_capacity_eviction() {
        let mut w = ContextWindow::new(3);
        w.observe(CognitiveUnitKind::Observation, "a");
        w.observe(CognitiveUnitKind::Observation, "b");
        w.observe(CognitiveUnitKind::Observation, "c");
        w.observe(CognitiveUnitKind::Observation, "d");
        assert_eq!(w.len(), 3);
        assert_eq!(w.units[0].content, "b");
        assert_eq!(w.units[2].content, "d");
    }

    #[test]
    fn test_attention_mask() {
        let mut w = ContextWindow::new(10);
        let id1 = w.observe(CognitiveUnitKind::Observation, "first");
        let _id2 = w.observe(CognitiveUnitKind::Observation, "second");
        w.attend(id1);
        assert_eq!(w.attention_mask.len(), 1);
        let attended = w.attended_context();
        assert_eq!(attended.len(), 1);
        assert_eq!(attended[0].content, "first");
    }

    #[test]
    fn test_clear_attention() {
        let mut w = ContextWindow::new(10);
        w.attend(0);
        w.attend(1);
        assert_eq!(w.attention_mask.len(), 2);
        w.clear_attention();
        assert_eq!(w.attention_mask.len(), 0);
    }

    #[test]
    fn test_recent_returns_newest_first() {
        let mut w = ContextWindow::new(10);
        w.observe(CognitiveUnitKind::Observation, "a");
        w.observe(CognitiveUnitKind::Observation, "b");
        w.observe(CognitiveUnitKind::Observation, "c");
        let recent = w.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].content, "c");
        assert_eq!(recent[1].content, "b");
    }

    #[test]
    fn test_by_domain_filter() {
        let mut w = ContextWindow::new(10);
        w.units.push_back(
            CognitiveUnit::new(0, CognitiveUnitKind::Observation, "rust code").with_domain("code"),
        );
        w.units.push_back(
            CognitiveUnit::new(1, CognitiveUnitKind::Observation, "design notes")
                .with_domain("design"),
        );
        let code_units = w.by_domain("code");
        assert_eq!(code_units.len(), 1);
        assert_eq!(code_units[0].content, "rust code");
    }

    #[test]
    fn test_next_id_monotonic() {
        let mut w = ContextWindow::new(10);
        assert_eq!(w.observe(CognitiveUnitKind::Observation, "a"), 0);
        assert_eq!(w.observe(CognitiveUnitKind::Observation, "b"), 1);
        assert_eq!(w.observe(CognitiveUnitKind::Observation, "c"), 2);
    }
}
