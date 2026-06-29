use std::collections::VecDeque;

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
}

pub struct ContextWindow {
    pub capacity: usize,
    pub units: VecDeque<CognitiveUnit>,
    pub next_id: usize,
    pub attention_mask: Vec<usize>,
}

impl ContextWindow {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            units: VecDeque::with_capacity(capacity),
            next_id: 0,
            attention_mask: Vec::new(),
        }
    }

    pub fn observe(&mut self, kind: CognitiveUnitKind, content: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let unit = CognitiveUnit::new(id, kind, content);
        if self.units.len() >= self.capacity {
            self.units.pop_front();
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
