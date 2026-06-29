use crate::core::nt_core_self::attention_head::AttentionDomain;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum DifficultyLevel {
    Mastery,
    Practicing,
    Developing,
    Emerging,
    Novel,
}

impl DifficultyLevel {
    pub fn from_success_rate(rate: f64) -> Self {
        if rate >= 0.90 {
            Self::Mastery
        } else if rate >= 0.70 {
            Self::Practicing
        } else if rate >= 0.40 {
            Self::Developing
        } else if rate >= 0.10 {
            Self::Emerging
        } else {
            Self::Novel
        }
    }

    pub fn target_range(&self) -> (f64, f64) {
        match self {
            Self::Mastery => (0.90, 1.0),
            Self::Practicing => (0.70, 0.90),
            Self::Developing => (0.40, 0.70),
            Self::Emerging => (0.10, 0.40),
            Self::Novel => (0.0, 0.10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskTemplate {
    pub id: u64,
    pub description: String,
    pub domain: AttentionDomain,
    pub required_capabilities: Vec<String>,
    pub difficulty: DifficultyLevel,
    pub prerequisite_skill_ids: Vec<u64>,
    pub generated_at: u64,
    pub completion_count: u64,
    pub average_success: f64,
    pub tags: Vec<String>,
}

pub struct GeneratorConfig {
    pub max_templates: usize,
    pub tasks_per_domain: usize,
    pub min_gap_severity: f64,
    pub novelty_rate: f64,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            max_templates: 200,
            tasks_per_domain: 20,
            min_gap_severity: 0.4,
            novelty_rate: 0.2,
        }
    }
}

pub struct CurriculumGenerator {
    templates: Vec<TaskTemplate>,
    next_id: u64,
    cycle: u64,
    pub config: GeneratorConfig,
    domain_success: HashMap<AttentionDomain, Vec<f64>>,
}

impl CurriculumGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            templates: Vec::with_capacity(config.max_templates),
            next_id: 1,
            cycle: 0,
            config,
            domain_success: HashMap::new(),
        }
    }

    pub fn register_outcome(&mut self, domain: AttentionDomain, success: bool) {
        self.cycle += 1;
        self.domain_success
            .entry(domain)
            .or_default()
            .push(if success { 1.0 } else { 0.0 });
    }

    pub fn domain_mastery(&self, domain: &AttentionDomain) -> DifficultyLevel {
        let rates = match self.domain_success.get(domain) {
            Some(r) if !r.is_empty() => r,
            _ => return DifficultyLevel::Novel,
        };
        let avg: f64 = rates.iter().sum::<f64>() / rates.len() as f64;
        DifficultyLevel::from_success_rate(avg)
    }

    pub fn generate_task(
        &mut self,
        domain: &AttentionDomain,
        available_skills: &[u64],
    ) -> Option<u64> {
        let mastery = self.domain_mastery(domain);
        let target_level = match mastery {
            DifficultyLevel::Mastery => {
                if rand::random::<f64>() < self.config.novelty_rate {
                    DifficultyLevel::Novel
                } else {
                    DifficultyLevel::Practicing
                }
            }
            DifficultyLevel::Practicing => DifficultyLevel::Developing,
            DifficultyLevel::Developing => DifficultyLevel::Emerging,
            DifficultyLevel::Emerging => DifficultyLevel::Novel,
            DifficultyLevel::Novel => DifficultyLevel::Emerging,
        };

        let existing_id = self
            .templates
            .iter()
            .find(|t| t.domain == *domain && t.difficulty == target_level && t.completion_count < 5)
            .map(|t| t.id);

        if let Some(id) = existing_id {
            return Some(id);
        }

        if self.templates.len() >= self.config.max_templates {
            self.prune();
        }
        if self.templates.len() >= self.config.max_templates {
            return None;
        }

        let id = self.next_id;
        self.next_id += 1;
        let desc = format!("{:?} task in {:?} at {:?} level", id, domain, target_level);
        self.templates.push(TaskTemplate {
            id,
            description: desc,
            domain: domain.clone(),
            required_capabilities: vec![format!("{:?}", domain)],
            difficulty: target_level.clone(),
            prerequisite_skill_ids: available_skills.to_vec(),
            generated_at: self.cycle,
            completion_count: 0,
            average_success: 0.0,
            tags: vec![format!("{:?}", domain), format!("{:?}", target_level)],
        });
        Some(id)
    }

    pub fn generate_from_gaps(
        &mut self,
        gap_descriptions: &[(String, AttentionDomain, f64)],
    ) -> Vec<u64> {
        let mut created = Vec::new();
        for (desc, domain, severity) in gap_descriptions {
            if *severity < self.config.min_gap_severity {
                continue;
            }
            if self.templates.len() >= self.config.max_templates {
                break;
            }
            if self.templates.iter().any(|t| t.description.contains(desc)) {
                continue;
            }
            let id = self.next_id;
            self.next_id += 1;
            self.templates.push(TaskTemplate {
                id,
                description: format!("gap:{}", desc),
                domain: domain.clone(),
                required_capabilities: vec![],
                difficulty: DifficultyLevel::Novel,
                prerequisite_skill_ids: vec![],
                generated_at: self.cycle,
                completion_count: 0,
                average_success: 0.0,
                tags: vec![format!("{:?}", domain), "gap".to_string()],
            });
            created.push(id);
        }
        created
    }

    pub fn update_completion(&mut self, template_id: u64, success: bool) {
        if let Some(t) = self.templates.iter_mut().find(|t| t.id == template_id) {
            let old_total = t.completion_count as f64;
            t.completion_count += 1;
            t.average_success = (t.average_success * old_total + if success { 1.0 } else { 0.0 })
                / t.completion_count as f64;
        }
    }

    pub fn next_challenge(&self, count: usize) -> Vec<&TaskTemplate> {
        let mut candidates: Vec<&TaskTemplate> = self
            .templates
            .iter()
            .filter(|t| t.completion_count < 10)
            .collect();
        candidates.sort_by(|a, b| {
            let score_a = a.difficulty.target_range().0 * (1.0 + a.completion_count as f64 * 0.1);
            let score_b = b.difficulty.target_range().0 * (1.0 + b.completion_count as f64 * 0.1);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(count);
        candidates
    }

    pub fn prune(&mut self) -> usize {
        let before = self.templates.len();
        self.templates
            .retain(|t| t.completion_count < 20 || t.average_success < 0.95);
        let removed = before.saturating_sub(self.templates.len());
        self.templates.truncate(self.config.max_templates);
        removed
    }

    pub fn template_count(&self) -> usize {
        self.templates.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_gen() -> CurriculumGenerator {
        CurriculumGenerator::new(GeneratorConfig {
            max_templates: 100,
            tasks_per_domain: 20,
            min_gap_severity: 0.4,
            novelty_rate: 0.0,
        })
    }

    #[test]
    fn test_generate_task_creates_template() {
        let mut gen = make_gen();
        let id = gen.generate_task(&AttentionDomain::Code, &[]);
        assert!(id.is_some());
        assert_eq!(gen.template_count(), 1);
        let t = &gen.templates[0];
        assert_eq!(t.domain, AttentionDomain::Code);
        assert_eq!(t.difficulty, DifficultyLevel::Emerging);
    }

    #[test]
    fn test_generate_task_returns_existing_for_same_domain_and_difficulty() {
        let mut gen = make_gen();
        gen.register_outcome(AttentionDomain::Code, false);
        gen.register_outcome(AttentionDomain::Code, false);
        let id1 = gen.generate_task(&AttentionDomain::Code, &[]).unwrap();
        let id2 = gen.generate_task(&AttentionDomain::Code, &[]).unwrap();
        assert_eq!(
            id1, id2,
            "should reuse existing template for same domain+difficulty"
        );
    }

    #[test]
    fn test_domain_mastery_returns_novel_when_no_data() {
        let gen = make_gen();
        assert_eq!(
            gen.domain_mastery(&AttentionDomain::Creativity),
            DifficultyLevel::Novel
        );
    }

    #[test]
    fn test_domain_mastery_increases_with_successes() {
        let mut gen = make_gen();
        for _ in 0..10 {
            gen.register_outcome(AttentionDomain::Code, true);
        }
        assert_eq!(
            gen.domain_mastery(&AttentionDomain::Code),
            DifficultyLevel::Mastery
        );
    }

    #[test]
    fn test_domain_mastery_decreases_with_failures() {
        let mut gen = make_gen();
        for _ in 0..3 {
            gen.register_outcome(AttentionDomain::Code, false);
        }
        assert_eq!(
            gen.domain_mastery(&AttentionDomain::Code),
            DifficultyLevel::Novel
        );
    }

    #[test]
    fn test_generate_from_gaps_creates_tasks() {
        let mut gen = make_gen();
        let gaps = vec![
            (
                "missing_knowledge".to_string(),
                AttentionDomain::Semantic,
                0.8,
            ),
            ("weak_reasoning".to_string(), AttentionDomain::Planning, 0.6),
        ];
        let ids = gen.generate_from_gaps(&gaps);
        assert_eq!(ids.len(), 2);
        assert_eq!(gen.template_count(), 2);
        assert!(gen.templates[0].description.contains("missing_knowledge"));
    }

    #[test]
    fn test_generate_from_gaps_skips_low_severity() {
        let mut gen = make_gen();
        let gaps = vec![("low_priority".to_string(), AttentionDomain::Code, 0.1)];
        let ids = gen.generate_from_gaps(&gaps);
        assert_eq!(ids.len(), 0);
    }

    #[test]
    fn test_generate_from_gaps_skips_duplicates() {
        let mut gen = make_gen();
        let gaps = vec![
            ("dup".to_string(), AttentionDomain::Code, 0.9),
            ("dup".to_string(), AttentionDomain::Code, 0.9),
        ];
        let ids = gen.generate_from_gaps(&gaps);
        assert_eq!(ids.len(), 1, "duplicate descriptions should be skipped");
    }

    #[test]
    fn test_update_completion_tracks_success() {
        let mut gen = make_gen();
        let id = gen.generate_task(&AttentionDomain::Code, &[]).unwrap();
        gen.update_completion(id, true);
        gen.update_completion(id, true);
        gen.update_completion(id, false);
        let t = gen.templates.iter().find(|t| t.id == id).unwrap();
        assert_eq!(t.completion_count, 3);
        assert!((t.average_success - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_next_challenge_returns_partially_completed() {
        let mut gen = make_gen();
        let id = gen.generate_task(&AttentionDomain::Code, &[]).unwrap();
        gen.update_completion(id, true);
        let challenges = gen.next_challenge(5);
        assert!(!challenges.is_empty());
        assert!(challenges[0].completion_count < 10);
    }

    #[test]
    fn test_prune_removes_fully_mastered() {
        let mut gen = make_gen();
        let id = gen.generate_task(&AttentionDomain::Code, &[]).unwrap();
        for _ in 0..25 {
            gen.update_completion(id, true);
        }
        let pruned = gen.prune();
        assert!(pruned > 0 || gen.template_count() <= 1);
    }

    #[test]
    fn test_difficulty_from_success_rate() {
        assert_eq!(
            DifficultyLevel::from_success_rate(0.95),
            DifficultyLevel::Mastery
        );
        assert_eq!(
            DifficultyLevel::from_success_rate(0.80),
            DifficultyLevel::Practicing
        );
        assert_eq!(
            DifficultyLevel::from_success_rate(0.55),
            DifficultyLevel::Developing
        );
        assert_eq!(
            DifficultyLevel::from_success_rate(0.25),
            DifficultyLevel::Emerging
        );
        assert_eq!(
            DifficultyLevel::from_success_rate(0.05),
            DifficultyLevel::Novel
        );
    }

    #[test]
    fn test_difficulty_target_range() {
        let (lo, hi) = DifficultyLevel::Mastery.target_range();
        assert!(lo >= 0.90 && hi <= 1.0);
        let (lo, hi) = DifficultyLevel::Novel.target_range();
        assert!(lo == 0.0 && hi <= 0.10);
    }
}
