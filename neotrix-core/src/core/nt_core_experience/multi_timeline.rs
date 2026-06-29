/// MTRE — Multi-Timeline Research Engine
/// Spawns parallel research timelines, each with isolated context.
/// Supports fork/merge/compare across timelines.
use std::collections::HashMap;

/// A single research trajectory with its own hypothesis buffer
#[derive(Debug, Clone)]
pub struct ResearchTimeline {
    pub id: String,
    pub name: String,
    pub created_cycle: u64,
    pub last_active_cycle: u64,
    /// Research focus/question
    pub focus: String,
    /// Discovered hypotheses along this timeline
    pub hypotheses: Vec<TimelineHypothesis>,
    /// Number of research iterations
    pub iteration_count: u64,
    /// Parent timeline (None for root)
    pub parent_id: Option<String>,
    /// Current confidence in this timeline's direction
    pub confidence: f64,
    /// Freshness — decays without activity
    pub freshness: f64,
}

/// A hypothesis discovered on a timeline
#[derive(Debug, Clone)]
pub struct TimelineHypothesis {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub created_cycle: u64,
    pub validated: bool,
    pub validation_score: f64,
}

impl ResearchTimeline {
    pub fn new(id: &str, name: &str, focus: &str, cycle: u64) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            created_cycle: cycle,
            last_active_cycle: cycle,
            focus: focus.to_string(),
            hypotheses: Vec::new(),
            iteration_count: 0,
            parent_id: None,
            confidence: 0.5,
            freshness: 1.0,
        }
    }

    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent_id = Some(parent.to_string());
        self
    }

    pub fn add_hypothesis(&mut self, hypothesis: TimelineHypothesis) {
        self.hypotheses.push(hypothesis);
    }

    pub fn iterate(&mut self, cycle: u64) {
        self.iteration_count += 1;
        self.last_active_cycle = cycle;
        self.freshness = (self.freshness + 0.1).min(1.0);
    }

    pub fn decay(&mut self) {
        self.freshness = (self.freshness - 0.05).max(0.0);
        self.confidence = (self.confidence * 0.99).max(0.1);
    }
}

/// Orchestrator managing multiple parallel research timelines
#[derive(Debug, Clone)]
pub struct TimelineOrchestrator {
    timelines: HashMap<String, ResearchTimeline>,
    next_id: u64,
    max_timelines: usize,
    cycle: u64,
}

impl TimelineOrchestrator {
    pub fn new(max_timelines: usize) -> Self {
        Self {
            timelines: HashMap::new(),
            next_id: 1,
            max_timelines,
            cycle: 0,
        }
    }

    pub fn advance_cycle(&mut self) {
        self.cycle += 1;
    }

    /// Fork a new timeline from an existing one
    pub fn fork_timeline(&mut self, parent_id: &str, new_focus: &str) -> Option<String> {
        let parent = self.timelines.get(parent_id)?;
        if self.timelines.len() >= self.max_timelines {
            return None;
        }
        let id = format!("tl_{}", self.next_id);
        self.next_id += 1;
        let timeline = ResearchTimeline::new(
            &id,
            &format!("Fork of {}", parent.name),
            new_focus,
            self.cycle,
        )
        .with_parent(parent_id);
        let id_clone = id.clone();
        self.timelines.insert(id, timeline);
        Some(id_clone)
    }

    /// Spawn a root timeline
    pub fn spawn_timeline(&mut self, name: &str, focus: &str) -> String {
        if self.timelines.len() >= self.max_timelines {
            return String::new();
        }
        let id = format!("tl_{}", self.next_id);
        self.next_id += 1;
        let timeline = ResearchTimeline::new(&id, name, focus, self.cycle);
        self.timelines.insert(id.clone(), timeline);
        id
    }

    /// Get a timeline by ID
    pub fn get(&self, id: &str) -> Option<&ResearchTimeline> {
        self.timelines.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut ResearchTimeline> {
        self.timelines.get_mut(id)
    }

    /// Get all active timelines (freshness > 0.3)
    pub fn active_timelines(&self) -> Vec<&ResearchTimeline> {
        self.timelines
            .values()
            .filter(|t| t.freshness > 0.3)
            .collect()
    }

    /// Get all timelines
    pub fn all_timelines(&self) -> Vec<&ResearchTimeline> {
        self.timelines.values().collect()
    }

    /// Add a hypothesis to a timeline
    pub fn add_hypothesis(&mut self, timeline_id: &str, hypothesis: TimelineHypothesis) -> bool {
        if let Some(tl) = self.timelines.get_mut(timeline_id) {
            tl.add_hypothesis(hypothesis);
            true
        } else {
            false
        }
    }

    /// Compare two timelines and return divergence score
    pub fn compare(&self, id_a: &str, id_b: &str) -> Option<f64> {
        let a = self.timelines.get(id_a)?;
        let b = self.timelines.get(id_b)?;
        if a.hypotheses.is_empty() || b.hypotheses.is_empty() {
            return Some(1.0);
        }
        let mut total_sim = 0.0;
        let mut pairs = 0;
        for ha in &a.hypotheses {
            for hb in &b.hypotheses {
                let desc_sim = text_similarity(&ha.description, &hb.description);
                total_sim += desc_sim;
                pairs += 1;
            }
        }
        let avg_sim = if pairs > 0 {
            total_sim / pairs as f64
        } else {
            0.0
        };
        Some(1.0 - avg_sim)
    }

    /// Merge two timelines into a new one
    pub fn merge(&mut self, id_a: &str, id_b: &str, merged_focus: &str) -> Option<String> {
        let (a, b) = (self.timelines.get(id_a)?, self.timelines.get(id_b)?);
        if self.timelines.len() >= self.max_timelines {
            return None;
        }
        let id = format!("tl_{}", self.next_id);
        self.next_id += 1;
        let mut merged = ResearchTimeline::new(
            &id,
            &format!("Merge: {} & {}", a.name, b.name),
            merged_focus,
            self.cycle,
        );
        for h in &a.hypotheses {
            merged.add_hypothesis(h.clone());
        }
        for h in &b.hypotheses {
            if !merged
                .hypotheses
                .iter()
                .any(|mh| text_similarity(&mh.description, &h.description) > 0.7)
            {
                merged.add_hypothesis(h.clone());
            }
        }
        merged.confidence = (a.confidence + b.confidence) / 2.0;
        self.timelines.insert(id.clone(), merged);
        Some(id)
    }

    /// Age all timelines (decay freshness)
    pub fn age_all(&mut self) {
        for tl in self.timelines.values_mut() {
            tl.decay();
        }
    }

    /// Prune stale timelines
    pub fn prune(&mut self) {
        self.timelines
            .retain(|_, tl| tl.freshness > 0.1 || tl.iteration_count > 0);
    }

    pub fn len(&self) -> usize {
        self.timelines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.timelines.is_empty()
    }

    /// Summary for dashboard
    pub fn summary(&self) -> String {
        let active = self.active_timelines().len();
        let total = self.timelines.len();
        let total_hypotheses: usize = self.timelines.values().map(|t| t.hypotheses.len()).sum();
        format!(
            "Timelines: {}/{} active, {} hypotheses, {} forks/merges",
            active, total, total_hypotheses, self.next_id
        )
    }
}

fn text_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let words_a: Vec<&str> = a.split_whitespace().collect();
    let words_b: Vec<&str> = b.split_whitespace().collect();
    let intersection: usize = words_a.iter().filter(|w| words_b.contains(w)).count();
    let union = words_a.len() + words_b.len() - intersection;
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_timeline() {
        let mut orch = TimelineOrchestrator::new(10);
        let id = orch.spawn_timeline("Research Alpha", "What is the best architecture?");
        assert!(!id.is_empty());
        assert_eq!(orch.len(), 1);
    }

    #[test]
    fn test_fork_timeline() {
        let mut orch = TimelineOrchestrator::new(10);
        let parent = orch.spawn_timeline("Parent", "initial question");
        let child = orch.fork_timeline(&parent, "forked question");
        assert!(child.is_some());
        assert_eq!(orch.len(), 2);
        let child_id = child.unwrap();
        let child_tl = orch.get(&child_id);
        assert!(child_tl.is_some());
        assert_eq!(child_tl.unwrap().parent_id, Some(parent));
    }

    #[test]
    fn test_add_hypothesis() {
        let mut orch = TimelineOrchestrator::new(10);
        let id = orch.spawn_timeline("Test", "test focus");
        let hyp = TimelineHypothesis {
            id: "hyp_1".into(),
            title: "Discovery".into(),
            description: "Found a pattern in the data".into(),
            source: "research".into(),
            confidence: 0.8,
            evidence: vec!["source1".into()],
            created_cycle: 1,
            validated: false,
            validation_score: 0.0,
        };
        assert!(orch.add_hypothesis(&id, hyp));
        let tl = orch.get(&id).unwrap();
        assert_eq!(tl.hypotheses.len(), 1);
    }

    #[test]
    fn test_active_timelines() {
        let mut orch = TimelineOrchestrator::new(10);
        orch.spawn_timeline("Active", "focus");
        orch.spawn_timeline("Stale", "focus");
        if let Some(tl) = orch.get_mut("tl_2") {
            tl.freshness = 0.1;
        }
        let active = orch.active_timelines();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_compare_timelines() {
        let mut orch = TimelineOrchestrator::new(10);
        let a = orch.spawn_timeline("A", "focus");
        let b = orch.spawn_timeline("B", "focus");
        orch.add_hypothesis(
            &a,
            TimelineHypothesis {
                id: "h1".into(),
                title: "H1".into(),
                description: "pattern found".into(),
                source: "src".into(),
                confidence: 0.8,
                evidence: vec![],
                created_cycle: 1,
                validated: false,
                validation_score: 0.0,
            },
        );
        orch.add_hypothesis(
            &b,
            TimelineHypothesis {
                id: "h2".into(),
                title: "H2".into(),
                description: "different result".into(),
                source: "src".into(),
                confidence: 0.6,
                evidence: vec![],
                created_cycle: 1,
                validated: false,
                validation_score: 0.0,
            },
        );
        let divergence = orch.compare(&a, &b);
        assert!(divergence.is_some());
        assert!(divergence.unwrap() > 0.0);
    }

    #[test]
    fn test_merge_timelines() {
        let mut orch = TimelineOrchestrator::new(10);
        let a = orch.spawn_timeline("A", "focus A");
        let b = orch.spawn_timeline("B", "focus B");
        orch.add_hypothesis(
            &a,
            TimelineHypothesis {
                id: "h1".into(),
                title: "H1".into(),
                description: "insight alpha".into(),
                source: "src".into(),
                confidence: 0.8,
                evidence: vec![],
                created_cycle: 1,
                validated: false,
                validation_score: 0.0,
            },
        );
        orch.add_hypothesis(
            &b,
            TimelineHypothesis {
                id: "h2".into(),
                title: "H2".into(),
                description: "insight beta".into(),
                source: "src".into(),
                confidence: 0.7,
                evidence: vec![],
                created_cycle: 1,
                validated: false,
                validation_score: 0.0,
            },
        );
        let merged = orch.merge(&a, &b, "merged focus");
        assert!(merged.is_some());
        let merged_id = merged.unwrap();
        let mtl = orch.get(&merged_id).unwrap();
        assert_eq!(mtl.hypotheses.len(), 2);
        assert!(mtl.confidence > 0.7);
    }

    #[test]
    fn test_age_and_prune() {
        let mut orch = TimelineOrchestrator::new(10);
        orch.spawn_timeline("Fresh", "focus");
        orch.age_all();
        let before = orch.len();
        if let Some(tl) = orch.get_mut("tl_1") {
            tl.freshness = 0.0;
        }
        orch.prune();
        assert!(orch.len() <= before);
    }

    #[test]
    fn test_max_timelines() {
        let mut orch = TimelineOrchestrator::new(2);
        orch.spawn_timeline("A", "focus");
        orch.spawn_timeline("B", "focus");
        let id = orch.spawn_timeline("C", "focus");
        assert!(id.is_empty(), "should not exceed max");
    }

    #[test]
    fn test_summary() {
        let mut orch = TimelineOrchestrator::new(10);
        orch.spawn_timeline("Test", "focus");
        let s = orch.summary();
        assert!(s.contains("active"));
        assert!(s.contains("hypotheses"));
    }
}
