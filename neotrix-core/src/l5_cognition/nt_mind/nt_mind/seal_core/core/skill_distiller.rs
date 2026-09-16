//! Skill Distiller — 技能蒸馏器
//! 从成功的执行轨迹中提取可复用技能

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistilledSkill {
    pub skill_id: String,
    pub name: String,
    pub pattern: String,
    pub domain: String,
    pub success_count: u32,
    pub total_attempts: u32,
    pub avg_latency_ms: f64,
    pub confidence: f64,
}

pub struct SkillDistiller {
    skills: HashMap<String, DistilledSkill>,
    min_success_rate: f64,
    min_occurrences: u32,
}

impl SkillDistiller {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            min_success_rate: 0.7,
            min_occurrences: 3,
        }
    }

    pub fn observe_trajectory(&mut self, pattern: &str, domain: &str, success: bool, latency_ms: u64) {
        let key = format!("{}:{}", domain, pattern);
        let skill = self.skills.entry(key.clone()).or_insert_with(|| DistilledSkill {
            skill_id: key,
            name: pattern.to_string(),
            pattern: pattern.to_string(),
            domain: domain.to_string(),
            success_count: 0,
            total_attempts: 0,
            avg_latency_ms: 0.0,
            confidence: 0.0,
        });
        
        skill.total_attempts += 1;
        if success { skill.success_count += 1; }
        skill.avg_latency_ms = (skill.avg_latency_ms * (skill.total_attempts - 1) as f64 + latency_ms as f64) / skill.total_attempts as f64;
        skill.confidence = skill.success_count as f64 / skill.total_attempts as f64;
    }

    pub fn get_skills(&self) -> Vec<&DistilledSkill> {
        self.skills.values()
            .filter(|s| s.total_attempts >= self.min_occurrences && s.confidence >= self.min_success_rate)
            .collect()
    }

    pub fn get_skill(&self, domain: &str, pattern: &str) -> Option<&DistilledSkill> {
        let key = format!("{}:{}", domain, pattern);
        self.skills.get(&key)
    }

    pub fn skills_for_domain(&self, domain: &str) -> Vec<&DistilledSkill> {
        self.skills.values()
            .filter(|s| s.domain == domain && s.total_attempts >= self.min_occurrences)
            .collect()
    }
}

impl Default for SkillDistiller {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distiller_basic() {
        let mut d = SkillDistiller::new();
        // Observe same pattern 4 times, 3 successes
        d.observe_trajectory("error_handling", "coding", true, 100);
        d.observe_trajectory("error_handling", "coding", true, 120);
        d.observe_trajectory("error_handling", "coding", true, 110);
        d.observe_trajectory("error_handling", "coding", false, 200);
        
        let skills = d.get_skills();
        assert_eq!(skills.len(), 1);
        assert!(skills[0].confidence >= 0.7);
    }
}
