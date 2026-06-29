use std::collections::VecDeque;

use super::types::ExplorationTrajectory;

#[derive(Debug, Clone)]
pub struct ExplorationExperience {
    pub id: String,
    pub trajectory: ExplorationTrajectory,
    pub outcome_score: f64,
    pub successful_actions: Vec<String>,
    pub key_insights: Vec<String>,
    pub domain: String,
    pub timestamp: i64,
}

pub struct ExperiencePool {
    pub experiences: VecDeque<ExplorationExperience>,
    pub max_experiences: usize,
}

impl ExperiencePool {
    pub fn new(max: usize) -> Self {
        Self {
            experiences: VecDeque::new(),
            max_experiences: max,
        }
    }

    pub fn store(&mut self, experience: ExplorationExperience) {
        if self.experiences.len() >= self.max_experiences {
            self.experiences.pop_front();
        }
        self.experiences.push_back(experience);
    }

    pub fn retrieve_by_domain(&self, domain: &str, limit: usize) -> Vec<&ExplorationExperience> {
        self.experiences
            .iter()
            .filter(|e| e.domain == domain)
            .take(limit)
            .collect()
    }

    pub fn extract_patterns(&self, domain: &str) -> Vec<String> {
        let relevant: Vec<&ExplorationExperience> = self.retrieve_by_domain(domain, 10);
        let mut patterns = Vec::new();

        let action_sets = Self::strip_actions(relevant.iter().map(|e| (*e).clone()).collect());
        for actions in &action_sets {
            for action in actions {
                let pattern = format!("pattern:{}", action);
                if !patterns.contains(&pattern) {
                    patterns.push(pattern);
                }
            }
        }

        patterns.sort_by(|a, b| {
            let count_a = action_sets
                .iter()
                .filter(|s| s.iter().any(|act| format!("pattern:{}", act) == *a))
                .count();
            let count_b = action_sets
                .iter()
                .filter(|s| s.iter().any(|act| format!("pattern:{}", act) == *b))
                .count();
            count_b.cmp(&count_a)
        });

        patterns
    }

    pub fn strip_actions(experiences: Vec<ExplorationExperience>) -> Vec<Vec<String>> {
        experiences
            .into_iter()
            .map(|e| e.trajectory.actions)
            .collect()
    }
}
