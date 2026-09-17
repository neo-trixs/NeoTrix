#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct SkillMetrics {
    pub success_rate: f64,
    pub avg_tokens: f64,
    pub avg_turns: f64,
    pub sample_count: u32,
}

impl Default for SkillMetrics {
    fn default() -> Self {
        Self {
            success_rate: 0.0,
            avg_tokens: 0.0,
            avg_turns: 0.0,
            sample_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillDoc {
    pub id: String,
    pub content: String,
    pub version: u32,
    pub domain: String,
    pub created_at: u64,
    pub metrics: SkillMetrics,
}

impl SkillDoc {
    pub fn new(
        id: impl Into<String>,
        content: impl Into<String>,
        domain: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            version: 1,
            domain: domain.into(),
            created_at: 0,
            metrics: SkillMetrics::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TrajectoryOutcome {
    Success { score: f64 },
    Failure { error: String, partial_score: f64 },
}

#[derive(Debug, Clone)]
pub struct TrajectoryStep {
    pub action: String,
    pub observation: String,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct Trajectory {
    pub task: String,
    pub domain: String,
    pub steps: Vec<TrajectoryStep>,
    pub outcome: TrajectoryOutcome,
    pub total_tokens: u64,
    pub total_turns: u32,
}

#[derive(Debug, Clone)]
pub enum EditType {
    Add,
    Delete,
    Replace,
}

#[derive(Debug, Clone)]
pub struct BoundedEdit {
    pub edit_type: EditType,
    pub target: String,
    pub content: String,
    pub score_delta: f64,
}

#[derive(Debug, Clone)]
pub struct ValidationGate;

impl ValidationGate {
    pub fn validate(doc: &SkillDoc, held_out: &[Trajectory]) -> bool {
        if held_out.is_empty() {
            return true;
        }
        let successes = held_out
            .iter()
            .filter(|t| matches!(t.outcome, TrajectoryOutcome::Success { .. }))
            .count();
        let rate = successes as f64 / held_out.len() as f64;
        rate >= doc.metrics.success_rate
    }
}

#[derive(Debug, Clone)]
pub struct RejectedEditBuffer {
    edits: VecDeque<BoundedEdit>,
    max_size: usize,
}

impl RejectedEditBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            edits: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add_rejection(&mut self, edit: BoundedEdit) {
        if self.edits.len() >= self.max_size {
            self.edits.pop_front();
        }
        self.edits.push_back(edit);
    }

    pub fn get_feedback(&self) -> Vec<String> {
        self.edits
            .iter()
            .map(|e| {
                format!(
                    "{:?} on '{}' (delta: {:.3})",
                    e.edit_type, e.target, e.score_delta
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct SkillOptimizer {
    pub learning_rate: f64,
    pub validation_gate: ValidationGate,
    pub rejected_buffer: RejectedEditBuffer,
}

impl SkillOptimizer {
    pub fn new(learning_rate: f64) -> Self {
        Self {
            learning_rate: learning_rate.max(0.0).min(1.0),
            validation_gate: ValidationGate,
            rejected_buffer: RejectedEditBuffer::new(50),
        }
    }

    pub fn optimize(&mut self, doc: &SkillDoc, batch: &[Trajectory]) -> SkillDoc {
        let _successes: Vec<&Trajectory> = batch
            .iter()
            .filter(|t| matches!(t.outcome, TrajectoryOutcome::Success { .. }))
            .collect();
        let failures: Vec<&Trajectory> = batch
            .iter()
            .filter(|t| matches!(t.outcome, TrajectoryOutcome::Failure { .. }))
            .collect();

        let mut new_content = doc.content.clone();

        if !failures.is_empty() && self.learning_rate > 0.1 {
            let errors: Vec<String> = failures
                .iter()
                .filter_map(|t| match &t.outcome {
                    TrajectoryOutcome::Failure { error, .. } => Some(error.clone()),
                    _ => None,
                })
                .collect();
            if !errors.is_empty() {
                new_content.push_str("\n### Error Prevention\n\n");
                for err in &errors {
                    new_content.push_str(&format!("- Avoid: {}\n", err));
                }
            }
        }

        let held_out: Vec<Trajectory> = batch.iter().take(batch.len() / 2).cloned().collect();
        let candidate = SkillDoc {
            content: new_content,
            version: doc.version + 1,
            ..doc.clone()
        };

        if ValidationGate::validate(&candidate, &held_out) {
            candidate
        } else {
            let edit = BoundedEdit {
                edit_type: EditType::Replace,
                target: "content".to_string(),
                content: doc.content.clone(),
                score_delta: -0.1,
            };
            self.rejected_buffer.add_rejection(edit);
            doc.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct BankStats {
    pub total: u32,
    pub by_domain: HashMap<String, u32>,
    pub avg_success_rate: f64,
}

#[derive(Debug, Clone)]
pub struct SkillBank {
    skills: HashMap<String, SkillDoc>,
}

impl SkillBank {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn add(&mut self, doc: SkillDoc) {
        self.skills.insert(doc.id.clone(), doc);
    }

    pub fn get(&self, id: &str) -> Option<&SkillDoc> {
        self.skills.get(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<SkillDoc> {
        self.skills.remove(id)
    }

    pub fn list(&self, domain: &str) -> Vec<&SkillDoc> {
        self.skills
            .values()
            .filter(|s| s.domain == domain)
            .collect()
    }

    pub fn evolve(
        &mut self,
        id: &str,
        batch: &[Trajectory],
        optimizer: &mut SkillOptimizer,
    ) -> bool {
        if let Some(doc) = self.skills.get(id).cloned() {
            let updated = optimizer.optimize(&doc, batch);
            if updated.version != doc.version {
                self.skills.insert(id.to_string(), updated);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn stats(&self) -> BankStats {
        let mut by_domain: HashMap<String, u32> = HashMap::new();
        let mut total_success = 0.0;
        for skill in self.skills.values() {
            *by_domain.entry(skill.domain.clone()).or_insert(0) += 1;
            total_success += skill.metrics.success_rate;
        }
        let avg = if self.skills.is_empty() {
            0.0
        } else {
            total_success / self.skills.len() as f64
        };
        BankStats {
            total: self.skills.len() as u32,
            by_domain,
            avg_success_rate: avg,
        }
    }
}

impl Default for SkillBank {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_trajectory(domain: &str, success: bool) -> Trajectory {
        let steps = vec![
            TrajectoryStep {
                action: "analyze".to_string(),
                observation: "found target".to_string(),
                score: 1.0,
            },
            TrajectoryStep {
                action: "execute".to_string(),
                observation: "done".to_string(),
                score: 1.0,
            },
        ];
        let outcome = if success {
            TrajectoryOutcome::Success { score: 1.0 }
        } else {
            TrajectoryOutcome::Failure {
                error: "timeout".to_string(),
                partial_score: 0.3,
            }
        };
        Trajectory {
            task: "test".to_string(),
            domain: domain.to_string(),
            steps,
            outcome,
            total_tokens: 100,
            total_turns: 2,
        }
    }

    #[test]
    fn test_validation_gate_rejects_bad() {
        let doc = SkillDoc::new("test", "bad skill", "test");
        let held_out = vec![sample_trajectory("test", false)];
        assert!(!ValidationGate::validate(
            &SkillDoc {
                metrics: SkillMetrics {
                    success_rate: 0.8,
                    ..Default::default()
                },
                ..doc
            },
            &held_out
        ));
    }

    #[test]
    fn test_bank_crud() {
        let mut bank = SkillBank::new();
        let doc = SkillDoc::new("s1", "content", "domain");
        bank.add(doc);
        assert!(bank.get("s1").is_some());
        assert_eq!(bank.list("domain").len(), 1);
        bank.remove("s1");
        assert!(bank.get("s1").is_none());
    }

    #[test]
    fn test_rejected_buffer_stores() {
        let mut buf = RejectedEditBuffer::new(3);
        let edit = BoundedEdit {
            edit_type: EditType::Add,
            target: "section".to_string(),
            content: "bad".to_string(),
            score_delta: -0.3,
        };
        buf.add_rejection(edit);
        let feedback = buf.get_feedback();
        assert_eq!(feedback.len(), 1);
        assert!(feedback[0].contains("Add"));
    }

    #[test]
    fn test_optimizer_preserves_good_skill() {
        let mut opt = SkillOptimizer::new(0.5);
        let doc = SkillDoc::new("s1", "good content", "test");
        let batch = vec![sample_trajectory("test", true)];
        let result = opt.optimize(&doc, &batch);
        assert_eq!(result.id, "s1");
    }
}
