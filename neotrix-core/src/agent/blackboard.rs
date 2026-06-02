use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    pub content: String,
    pub source_agent: String,
    pub confidence: f64,
    pub timestamp: u64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub id: String,
    pub description: String,
    pub status: IntentStatus,
    pub source_agent: String,
    pub created_at: u64,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentStatus {
    Proposed,
    Claimed,
    Completed,
    Failed,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hint {
    pub id: String,
    pub content: String,
    pub source: String,
    pub priority: u8,
    pub timestamp: u64,
}

pub struct Blackboard {
    facts: Vec<Fact>,
    intents: Vec<Intent>,
    hints: Vec<Hint>,
    max_history: usize,
}

impl Blackboard {
    pub fn new(max_history: usize) -> Self {
        Blackboard {
            facts: Vec::new(),
            intents: Vec::new(),
            hints: Vec::new(),
            max_history,
        }
    }

    pub fn add_fact(&mut self, content: &str, source: &str, confidence: f64, tags: Vec<String>) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let fact = Fact {
            id: id.clone(),
            content: content.to_string(),
            source_agent: source.to_string(),
            confidence,
            timestamp: Utc::now().timestamp() as u64,
            tags,
        };
        self.facts.push(fact);
        if self.facts.len() > self.max_history {
            self.facts.remove(0);
        }
        id
    }

    pub fn get_facts(&self) -> &[Fact] {
        &self.facts
    }

    pub fn get_facts_by_tag(&self, tag: &str) -> Vec<&Fact> {
        self.facts.iter().filter(|f| f.tags.iter().any(|t| t == tag)).collect()
    }

    pub fn get_facts_by_source(&self, source: &str) -> Vec<&Fact> {
        self.facts.iter().filter(|f| f.source_agent == source).collect()
    }

    pub fn get_fact(&self, id: &str) -> Option<&Fact> {
        self.facts.iter().find(|f| f.id == id)
    }

    pub fn search_facts(&self, query: &str) -> Vec<&Fact> {
        let lower = query.to_lowercase();
        self.facts.iter().filter(|f| f.content.to_lowercase().contains(&lower)).collect()
    }

    pub fn add_intent(&mut self, description: &str, source: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let intent = Intent {
            id: id.clone(),
            description: description.to_string(),
            status: IntentStatus::Proposed,
            source_agent: source.to_string(),
            created_at: Utc::now().timestamp() as u64,
            claimed_by: None,
            claimed_at: None,
        };
        self.intents.push(intent);
        id
    }

    pub fn claim_intent(&mut self, id: &str, agent: &str) -> bool {
        if let Some(intent) = self.intents.iter_mut().find(|i| i.id == id) {
            if intent.status == IntentStatus::Proposed {
                intent.status = IntentStatus::Claimed;
                intent.claimed_by = Some(agent.to_string());
                intent.claimed_at = Some(Utc::now().timestamp() as u64);
                return true;
            }
        }
        false
    }

    pub fn complete_intent(&mut self, id: &str) -> bool {
        if let Some(intent) = self.intents.iter_mut().find(|i| i.id == id) {
            if intent.status == IntentStatus::Claimed {
                intent.status = IntentStatus::Completed;
                return true;
            }
        }
        false
    }

    pub fn fail_intent(&mut self, id: &str) -> bool {
        if let Some(intent) = self.intents.iter_mut().find(|i| i.id == id) {
            if intent.status == IntentStatus::Claimed || intent.status == IntentStatus::Proposed {
                intent.status = IntentStatus::Failed;
                return true;
            }
        }
        false
    }

    pub fn supersede_intent(&mut self, id: &str, replacement_desc: &str, source: &str) -> Option<String> {
        let found = self.intents.iter_mut().find(|i| i.id == id)?;
        if found.status != IntentStatus::Claimed && found.status != IntentStatus::Proposed {
            return None;
        }
        found.status = IntentStatus::Superseded;
        Some(self.add_intent(replacement_desc, source))
    }

    pub fn get_pending_intents(&self) -> Vec<&Intent> {
        self.intents.iter().filter(|i| {
            i.status == IntentStatus::Proposed || i.status == IntentStatus::Claimed
        }).collect()
    }

    pub fn get_intents_by_status(&self, status: IntentStatus) -> Vec<&Intent> {
        self.intents.iter().filter(|i| i.status == status).collect()
    }

    pub fn get_intent(&self, id: &str) -> Option<&Intent> {
        self.intents.iter().find(|i| i.id == id)
    }

    pub fn add_hint(&mut self, content: &str, source: &str, priority: u8) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let hint = Hint {
            id: id.clone(),
            content: content.to_string(),
            source: source.to_string(),
            priority,
            timestamp: Utc::now().timestamp() as u64,
        };
        self.hints.push(hint);
        if self.hints.len() > self.max_history {
            self.hints.remove(0);
        }
        id
    }

    pub fn get_hints(&self) -> &[Hint] {
        &self.hints
    }

    pub fn get_high_priority_hints(&self, min_priority: u8) -> Vec<&Hint> {
        self.hints.iter().filter(|h| h.priority >= min_priority).collect()
    }

    pub fn graph_summary(&self) -> BlackboardSummary {
        let total_facts = self.facts.len();
        let total_intents = self.intents.len();
        let total_hints = self.hints.len();
        let pending_intents = self.get_pending_intents().len();
        let completed_intents = self.get_intents_by_status(IntentStatus::Completed).len();

        let mut sources: Vec<&str> = Vec::new();
        for f in &self.facts {
            if !sources.contains(&f.source_agent.as_str()) {
                sources.push(&f.source_agent);
            }
        }
        for i in &self.intents {
            if !sources.contains(&i.source_agent.as_str()) {
                sources.push(&i.source_agent);
            }
        }
        for h in &self.hints {
            if !sources.contains(&h.source.as_str()) {
                sources.push(&h.source);
            }
        }

        let average_confidence = if self.facts.is_empty() {
            0.0
        } else {
            self.facts.iter().map(|f| f.confidence).sum::<f64>() / self.facts.len() as f64
        };

        BlackboardSummary {
            total_facts,
            total_intents,
            total_hints,
            pending_intents,
            completed_intents,
            unique_sources: sources.len(),
            average_confidence,
        }
    }

    pub fn fact_to_intent_ratio(&self) -> f64 {
        let total = self.facts.len() + self.intents.len();
        if total == 0 {
            0.0
        } else {
            self.facts.len() as f64 / total as f64
        }
    }

    pub fn reachable_goal_estimate(&self, goal_tags: &[String]) -> f64 {
        if goal_tags.is_empty() {
            return 0.0;
        }
        let mut matched = 0;
        for tag in goal_tags {
            let in_facts = self.facts.iter().any(|f| f.tags.iter().any(|t| t == tag));
            let in_intents = self.intents.iter().any(|i| i.description.to_lowercase().contains(&tag.to_lowercase()));
            if in_facts || in_intents {
                matched += 1;
            }
        }
        matched as f64 / goal_tags.len() as f64
    }

    pub fn observe(&self) -> BlackboardSnapshot {
        BlackboardSnapshot {
            facts: self.facts.clone(),
            intents: self.intents.clone(),
            hints: self.hints.clone(),
            timestamp: Utc::now().timestamp() as u64,
        }
    }

    pub fn orient(&self, _snapshot: &BlackboardSnapshot) -> Vec<&Intent> {
        self.get_pending_intents()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardSummary {
    pub total_facts: usize,
    pub total_intents: usize,
    pub total_hints: usize,
    pub pending_intents: usize,
    pub completed_intents: usize,
    pub unique_sources: usize,
    pub average_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct BlackboardSnapshot {
    pub facts: Vec<Fact>,
    pub intents: Vec<Intent>,
    pub hints: Vec<Hint>,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bb() -> Blackboard {
        Blackboard::new(100)
    }

    #[test]
    fn test_add_and_retrieve_facts() {
        let mut bb = make_bb();
        let id = bb.add_fact("port 443 is open on target", "scanner", 0.95, vec!["network".into(), "port".into()]);
        let fact = bb.get_fact(&id).expect("fact should exist");
        assert_eq!(fact.content, "port 443 is open on target");
        assert_eq!(fact.source_agent, "scanner");
        assert_eq!(fact.confidence, 0.95);
        assert_eq!(bb.get_facts().len(), 1);
    }

    #[test]
    fn test_intent_lifecycle() {
        let mut bb = make_bb();
        let id = bb.add_intent("scan subnet 10.0.0.0/24", "planner");

        let pending = bb.get_pending_intents();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].status, IntentStatus::Proposed);

        assert!(bb.claim_intent(&id, "nmap_agent"));
        let intent = bb.get_intent(&id).expect("value should be ok in test");
        assert_eq!(intent.status, IntentStatus::Claimed);
        assert_eq!(intent.claimed_by.as_deref(), Some("nmap_agent"));

        assert!(bb.complete_intent(&id));
        let completed = bb.get_intents_by_status(IntentStatus::Completed);
        assert_eq!(completed.len(), 1);
        assert_eq!(bb.get_pending_intents().len(), 0);
    }

    #[test]
    fn test_hint_priority_filtering() {
        let mut bb = make_bb();
        bb.add_hint("check firewall rules", "operator", 8);
        bb.add_hint("log levels are noisy", "operator", 3);
        bb.add_hint("use verbose output", "operator", 6);

        let high = bb.get_high_priority_hints(6);
        assert_eq!(high.len(), 2);
        assert!(high.iter().all(|h| h.priority >= 6));
    }

    #[test]
    fn test_search_facts_by_tag() {
        let mut bb = make_bb();
        bb.add_fact("host is up", "ping", 0.9, vec!["network".into(), "discovery".into()]);
        bb.add_fact("ssl cert expired", "tls", 1.0, vec!["ssl".into(), "security".into()]);
        bb.add_fact("dns resolves", "dns", 0.8, vec!["network".into(), "dns".into()]);

        let network_facts = bb.get_facts_by_tag("network");
        assert_eq!(network_facts.len(), 2);

        let ssl_facts = bb.get_facts_by_tag("ssl");
        assert_eq!(ssl_facts.len(), 1);
        assert_eq!(ssl_facts[0].source_agent, "tls");
    }

    #[test]
    fn test_fact_to_intent_ratio() {
        let mut bb = make_bb();
        assert_eq!(bb.fact_to_intent_ratio(), 0.0);

        bb.add_fact("a", "src", 1.0, vec![]);
        bb.add_intent("b", "src");
        let ratio = bb.fact_to_intent_ratio();
        assert!((ratio - 0.5).abs() < 0.001);

        bb.add_fact("c", "src", 1.0, vec![]);
        let ratio2 = bb.fact_to_intent_ratio();
        assert!((ratio2 - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_supersede_intent_with_replacement() {
        let mut bb = make_bb();
        let original = bb.add_intent("scan with nmap", "planner");
        bb.claim_intent(&original, "agent");

        let replacement = bb.supersede_intent(&original, "scan with masscan instead", "planner");
        assert!(replacement.is_some());

        let orig = bb.get_intent(&original).expect("value should be ok in test");
        assert_eq!(orig.status, IntentStatus::Superseded);

        let repl = bb.get_intent(&replacement.expect("replacement should be ok in test")).expect("replacement should be ok in test");
        assert_eq!(repl.status, IntentStatus::Proposed);
        assert_eq!(repl.description, "scan with masscan instead");
    }

    #[test]
    fn test_graph_summary() {
        let mut bb = make_bb();
        bb.add_fact("fact a", "alpha", 0.8, vec![]);
        bb.add_fact("fact b", "beta", 0.9, vec![]);
        bb.add_intent("intent c", "alpha");
        bb.add_hint("hint d", "gamma", 5);

        let s = bb.graph_summary();
        assert_eq!(s.total_facts, 2);
        assert_eq!(s.total_intents, 1);
        assert_eq!(s.total_hints, 1);
        assert_eq!(s.pending_intents, 1);
        assert_eq!(s.completed_intents, 0);
        assert_eq!(s.unique_sources, 3);
        assert!((s.average_confidence - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_reachable_goal_estimate() {
        let mut bb = make_bb();
        bb.add_fact("network ok", "ping", 1.0, vec!["network".into()]);
        bb.add_intent("secure the ssl endpoint", "planner");

        let goals = vec!["network".to_string(), "ssl".to_string(), "database".to_string()];
        let estimate = bb.reachable_goal_estimate(&goals);
        assert!((estimate - 2.0 / 3.0).abs() < 0.001);
    }
}
