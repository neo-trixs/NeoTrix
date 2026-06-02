use super::self_edit::MicroEdit;
use super::self_iterating::ReasoningBrain;
use super::memory::{ReasoningBank, ReasoningMemory};
use crate::core::nt_core_self::skill_crystal::{SkillCrystal, CrystalRegistry};
use crate::core::nt_core_self::reasoning_strategy::StrategyKind;
use crate::core::nt_core_self::attention_head::AttentionDomain;
use crate::neotrix::nt_world_model::TaskType;

pub struct AutoCrystallizer {
    pub registry: CrystalRegistry,
    pub auto_crystallize: bool,
    pub min_reward_threshold: f64,
    pub total_crystallized: u64,
}

impl AutoCrystallizer {
    pub fn new() -> Self {
        Self {
            registry: CrystalRegistry::new(),
            auto_crystallize: true,
            min_reward_threshold: 0.3,
            total_crystallized: 0,
        }
    }

    pub fn crystallize_from_absorption(
        &mut self,
        _brain: &mut ReasoningBrain,
        bank: &mut ReasoningBank,
        source_url: &str,
        source_name: &str,
        domain: &str,
        edits: &[MicroEdit],
        reward: f64,
    ) -> Option<SkillCrystal> {
        if !self.auto_crystallize || reward < self.min_reward_threshold {
            return None;
        }

        let description = format!("Auto-crystallized from {} ({})", source_name, domain);
        let crystal_id = self.registry.next_id;
        let pattern = edits.iter().map(|e| format!("{:?}", e)).collect::<Vec<_>>().join(", ");
        let crystal = SkillCrystal {
            id: crystal_id,
            name: description,
            pattern,
            effectiveness: reward,
            use_count: 0,
            source_trace_ids: Vec::new(),
            tags: vec![source_name.to_string(), domain.to_string()],
            strategy: StrategyKind::Reflection,
            domain: AttentionDomain::PatternMatch,
            created_at: 0,
            last_used: 0,
        };

        self.registry.crystals.push(crystal.clone());
        self.registry.next_id += 1;

        let mem = ReasoningMemory::new(
            &format!("Crystal: {} from {}", crystal.id, source_url),
            TaskType::General,
            edits,
            reward,
        );
        bank.store(mem);

        self.total_crystallized += 1;

        Some(crystal)
    }

    pub fn crystallize_from_trace(
        &mut self,
        bank: &mut ReasoningBank,
        description: &str,
        insights: &[String],
        confidence: f64,
    ) -> Option<SkillCrystal> {
        if !self.auto_crystallize || confidence < self.min_reward_threshold {
            return None;
        }

        let crystal_id = self.registry.next_id;
        let crystal = SkillCrystal {
            id: crystal_id,
            name: description.to_string(),
            pattern: insights.join("; "),
            effectiveness: confidence,
            use_count: 0,
            source_trace_ids: Vec::new(),
            tags: insights.to_vec(),
            strategy: StrategyKind::Reflection,
            domain: AttentionDomain::PatternMatch,
            created_at: 0,
            last_used: 0,
        };

        self.registry.crystals.push(crystal.clone());
        self.registry.next_id += 1;

        let mem = ReasoningMemory::new(
            &format!("Crystal: {} (trace)", crystal.id),
            TaskType::General,
            &[],
            confidence,
        );
        bank.store(mem);

        self.total_crystallized += 1;

        Some(crystal)
    }

    pub fn summary(&self) -> String {
        format!(
            "AutoCrystallizer: {} crystals | auto={} | threshold={:.2}",
            self.total_crystallized,
            self.auto_crystallize,
            self.min_reward_threshold,
        )
    }
}

impl Default for AutoCrystallizer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_crystallizer() {
        let c = AutoCrystallizer::new();
        assert!(c.auto_crystallize);
        assert_eq!(c.total_crystallized, 0);
    }

    #[test]
    fn test_crystallize_from_absorption_low_reward() {
        let mut c = AutoCrystallizer::new();
        let mut brain = ReasoningBrain::new();
        let mut bank = ReasoningBank::new(100);
        let result = c.crystallize_from_absorption(
            &mut brain, &mut bank,
            "https://example.com", "test", "general",
            &[], 0.1,
        );
        assert!(result.is_none());
        assert_eq!(c.total_crystallized, 0);
    }

    #[test]
    fn test_crystallize_from_absorption_high_reward() {
        let mut c = AutoCrystallizer::new();
        let mut brain = ReasoningBrain::new();
        let mut bank = ReasoningBank::new(100);
        let edits = vec![
            MicroEdit::AdjustDimension("compound_composition".to_string(), 0.1),
            MicroEdit::NormalizeVector,
        ];
        let result = c.crystallize_from_absorption(
            &mut brain, &mut bank,
            "https://example.com", "test", "general",
            &edits, 0.8,
        );
        assert!(result.is_some());
        assert_eq!(c.total_crystallized, 1);
        let crystal = result.unwrap();
        assert!(crystal.tags.len() <= 2);
    }

    #[test]
    fn test_crystallize_from_trace() {
        let mut c = AutoCrystallizer::new();
        let mut bank = ReasoningBank::new(100);
        let insights = vec!["pattern: use agent isolation".to_string(), "pattern: module boundaries".to_string()];
        let result = c.crystallize_from_trace(&mut bank, "agent design", &insights, 0.7);
        assert!(result.is_some());
        assert_eq!(c.total_crystallized, 1);
    }

    #[test]
    fn test_crystallize_from_trace_low_confidence() {
        let mut c = AutoCrystallizer::new();
        let mut bank = ReasoningBank::new(100);
        let result = c.crystallize_from_trace(&mut bank, "test", &[], 0.1);
        assert!(result.is_none());
    }

    #[test]
    fn test_crystallizer_disabled() {
        let mut c = AutoCrystallizer::new();
        c.auto_crystallize = false;
        let mut brain = ReasoningBrain::new();
        let mut bank = ReasoningBank::new(100);
        let result = c.crystallize_from_absorption(
            &mut brain, &mut bank,
            "url", "name", "domain", &[], 0.9,
        );
        assert!(result.is_none());
    }
}
