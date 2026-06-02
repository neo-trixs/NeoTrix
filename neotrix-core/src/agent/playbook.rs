use chrono::Utc;

#[derive(Debug, Clone)]
pub struct Playbook {
    pub id: String,
    pub trigger: String,
    pub instruction: String,
    pub pitfall: Option<String>,
    pub source: PlaybookSource,
    pub confidence: f64,
    pub created_at: u64,
    pub use_count: u64,
}

#[derive(Debug, Clone)]
pub enum PlaybookSource {
    UserCorrection { original: String, corrected: String },
    ExpertExample { ideal: String, agent: String },
    SuccessPath { task: String, steps: Vec<String> },
    Pattern { cluster_size: usize, commonality: f64 },
}

pub struct PlaybookEngine {
    playbooks: Vec<Playbook>,
    max_playbooks: usize,
    min_confidence_threshold: f64,
}

impl PlaybookEngine {
    pub fn new(max_playbooks: usize, min_confidence: f64) -> Self {
        Self { playbooks: Vec::new(), max_playbooks, min_confidence_threshold: min_confidence }
    }

    pub fn extract_from_correction(&mut self, original: &str, corrected: &str, trigger: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let playbook = Playbook {
            id: id.clone(),
            trigger: trigger.to_string(),
            instruction: Self::diff_to_instruction(original, corrected),
            pitfall: Some(format!("Avoid: {}", original)),
            source: PlaybookSource::UserCorrection {
                original: original.to_string(),
                corrected: corrected.to_string(),
            },
            confidence: 0.5,
            created_at: Utc::now().timestamp() as u64,
            use_count: 0,
        };
        self.add_playbook(playbook);
        id
    }

    pub fn extract_from_expert(&mut self, ideal: &str, agent_output: &str, trigger: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let playbook = Playbook {
            id: id.clone(),
            trigger: trigger.to_string(),
            instruction: Self::diff_to_instruction(agent_output, ideal),
            pitfall: None,
            source: PlaybookSource::ExpertExample {
                ideal: ideal.to_string(),
                agent: agent_output.to_string(),
            },
            confidence: 0.7,
            created_at: Utc::now().timestamp() as u64,
            use_count: 0,
        };
        self.add_playbook(playbook);
        id
    }

    pub fn record_success_path(&mut self, task: &str, steps: Vec<String>) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let playbook = Playbook {
            id: id.clone(),
            trigger: task.to_string(),
            instruction: format!("Follow these steps: {}", steps.join(" -> ")),
            pitfall: None,
            source: PlaybookSource::SuccessPath {
                task: task.to_string(),
                steps,
            },
            confidence: 0.6,
            created_at: Utc::now().timestamp() as u64,
            use_count: 0,
        };
        self.add_playbook(playbook);
        id
    }

    pub fn find_matching(&self, task: &str) -> Vec<&Playbook> {
        let task_lower = task.to_lowercase();
        let mut matched: Vec<&Playbook> = self.playbooks.iter()
            .filter(|p| p.confidence >= self.min_confidence_threshold)
            .filter(|p| task_lower.contains(&p.trigger.to_lowercase()))
            .collect();
        matched.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        matched.truncate(5);
        matched
    }

    pub fn report_use(&mut self, id: &str) -> bool {
        if let Some(p) = self.playbooks.iter_mut().find(|p| p.id == id) {
            p.use_count += 1;
            p.confidence = (p.confidence + 0.05).min(1.0);
            true
        } else { false }
    }

    pub fn report_failure(&mut self, id: &str) -> bool {
        if let Some(p) = self.playbooks.iter_mut().find(|p| p.id == id) {
            p.confidence = (p.confidence - 0.1).max(0.0);
            true
        } else { false }
    }

    pub fn prune_low_confidence(&mut self, threshold: f64) -> usize {
        let before = self.playbooks.len();
        self.playbooks.retain(|p| p.confidence >= threshold);
        before - self.playbooks.len()
    }

    pub fn all_playbooks(&self) -> &[Playbook] { &self.playbooks }
    pub fn count(&self) -> usize { self.playbooks.len() }

    fn add_playbook(&mut self, pb: Playbook) {
        if self.playbooks.len() >= self.max_playbooks {
            self.playbooks.sort_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal));
            self.playbooks.remove(0);
        }
        self.playbooks.push(pb);
    }

    fn diff_to_instruction(before: &str, after: &str) -> String {
        if let Some(stripped) = after.strip_prefix(before) {
            let addition = stripped.trim();
            format!("Add: {}", addition)
        } else if before.len() > after.len() && before.contains(after.trim()) {
            format!("Remove unnecessary parts, keep: {}", after)
        } else if before.len() > 20 && after.len() > 20 {
            format!("Replace '{}...' with '{}...'", &before[..20], &after[..20])
        } else {
            format!("Change from '{}' to '{}'", before, after)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_correction_and_match() {
        let mut engine = PlaybookEngine::new(100, 0.3);
        let id = engine.extract_from_correction(
            "use unwrap()",
            "use expect() for better error messages",
            "error handling",
        );
        assert!(!id.is_empty());
        let matches = engine.find_matching("improve error handling");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, id);
    }

    #[test]
    fn test_extract_from_expert_example() {
        let mut engine = PlaybookEngine::new(100, 0.3);
        let id = engine.extract_from_expert(
            "use Arc<Mutex<T>> for thread safety",
            "use RefCell for shared state",
            "concurrency",
        );
        assert!(engine.all_playbooks().iter().any(|p| p.id == id));
        assert_eq!(engine.all_playbooks().iter().find(|p| p.id == id).expect("playbook should exist after extraction").confidence, 0.7);
    }

    #[test]
    fn test_record_success_path() {
        let mut engine = PlaybookEngine::new(100, 0.3);
        let steps = vec!["init".into(), "validate".into(), "execute".into()];
        let _id = engine.record_success_path("data pipeline", steps);
        let matches = engine.find_matching("data pipeline process");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].instruction.contains("init -> validate -> execute"));
    }

    #[test]
    fn test_report_use_and_failure() {
        let mut engine = PlaybookEngine::new(100, 0.3);
        let id = engine.extract_from_correction("bad", "good", "test");
        let initial_conf = engine.all_playbooks().iter().find(|p| p.id == id).expect("playbook should exist before use").confidence;
        engine.report_use(&id);
        let after_use = engine.all_playbooks().iter().find(|p| p.id == id).expect("playbook should exist after use").confidence;
        assert!(after_use > initial_conf);
        engine.report_failure(&id);
        let after_fail = engine.all_playbooks().iter().find(|p| p.id == id).expect("playbook should exist after failure").confidence;
        assert!(after_fail < after_use);
        assert!(!engine.report_use("nonexistent"));
        assert!(!engine.report_failure("nonexistent"));
    }

    #[test]
    fn test_prune_low_confidence() {
        let mut engine = PlaybookEngine::new(100, 0.1);
        engine.extract_from_correction("a", "b", "t1");
        engine.extract_from_correction("c", "d", "t2");
        assert_eq!(engine.count(), 2);
        let last_id = engine.extract_from_correction("e", "f", "t3");
        engine.report_failure(&last_id);
        engine.report_failure(&last_id);
        engine.report_failure(&last_id);
        let pruned = engine.prune_low_confidence(0.3);
        assert!(pruned > 0);
    }
}
