use std::fmt::Debug;

use crate::core::nt_core_experience::skill_acc;
use crate::core::nt_core_hcube::skill_compiler;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_self::attention_head::AttentionDomain;

/// Unified identifier for any skill
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct SkillId(pub u64);

impl SkillId {
    pub fn from_str(s: &str) -> Self {
        let mut h: u64 = 0xdead_beef;
        for b in s.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as u64);
        }
        SkillId(h)
    }
}

/// Representation of a skill for matching
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SkillRepr {
    Vsa(Vec<u8>),
    TextKeywords(Vec<String>),
    TextWithMeta {
        content: String,
        system_prompt: String,
        triggers: Vec<String>,
    },
}

/// Input context for skill matching
#[derive(Debug, Clone, Default)]
pub struct SkillContext {
    pub input_text: Option<String>,
    pub vsa_vector: Option<Vec<u8>>,
    pub task_type: Option<String>,
}

/// Outcome of executing a skill
#[derive(Debug, Clone)]
pub struct SkillOutcome {
    pub success: bool,
    pub output: String,
    pub execution_time_ms: u64,
    pub confidence_delta: f64,
}

/// Skill domain classification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SkillDomain {
    Reasoning,
    Coding,
    Search,
    Memory,
    Communication,
    Planning,
    Perception,
    Meta,
    Unknown,
}

/// The unified Skill trait
pub trait UnifiedSkill: Debug + Send + Sync {
    fn id(&self) -> SkillId;
    fn name(&self) -> &str;
    fn representation(&self) -> SkillRepr;
    fn domain(&self) -> SkillDomain;
    fn confidence(&self) -> f64;
    fn matches(&self, context: &SkillContext) -> f64;
    fn execute(&mut self, context: &SkillContext) -> Result<SkillOutcome, String>;
    fn update(&mut self, outcome: &SkillOutcome);
}

/// Unified registry holding any skill
#[derive(Debug)]
pub struct UnifiedSkillRegistry {
    pub skills: Vec<Box<dyn UnifiedSkill>>,
}

impl UnifiedSkillRegistry {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    pub fn register(&mut self, skill: Box<dyn UnifiedSkill>) -> SkillId {
        let id = skill.id();
        self.skills.push(skill);
        id
    }

    pub fn find(&self, context: &SkillContext, top_k: usize) -> Vec<(&dyn UnifiedSkill, f64)> {
        let mut scored: Vec<_> = self
            .skills
            .iter()
            .map(|s| (s.as_ref(), s.matches(context)))
            .filter(|(_, score)| *score > 0.0)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    pub fn execute_best(&mut self, context: &SkillContext) -> Result<SkillOutcome, String> {
        let scored = self.find(context, 1);
        if scored.is_empty() {
            return Err("No matching skill found".to_string());
        }
        let idx = self
            .skills
            .iter()
            .position(|s| s.id() == scored[0].0.id())
            .ok_or("Skill not found in registry")?;
        let result = self.skills[idx].execute(context)?;
        self.skills[idx].update(&result);
        Ok(result)
    }

    pub fn all_skills(&self) -> &[Box<dyn UnifiedSkill>] {
        &self.skills
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

impl Default for UnifiedSkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Adapter: VSASkill (Mechanism A — skill_acc) ───────────────────────────

/// Adapter wrapping [`skill_acc::VSASkill`] into [`UnifiedSkill`].
#[derive(Debug)]
pub struct VSASkillAdapter {
    pub inner: skill_acc::VSASkill,
}

impl VSASkillAdapter {
    pub fn new(skill: skill_acc::VSASkill) -> Self {
        Self { inner: skill }
    }
}

impl UnifiedSkill for VSASkillAdapter {
    fn id(&self) -> SkillId {
        SkillId(self.inner.id)
    }

    fn name(&self) -> &str {
        &self.inner.name
    }

    fn representation(&self) -> SkillRepr {
        SkillRepr::Vsa(self.inner.trigger.clone())
    }

    fn domain(&self) -> SkillDomain {
        match self.inner.domain {
            AttentionDomain::Reasoning | AttentionDomain::PatternMatch => SkillDomain::Reasoning,
            AttentionDomain::Code | AttentionDomain::ToolUse => SkillDomain::Coding,
            AttentionDomain::Semantic | AttentionDomain::Memory => SkillDomain::Memory,
            AttentionDomain::Planning | AttentionDomain::GoalAlignment => SkillDomain::Planning,
            AttentionDomain::Social | AttentionDomain::Emotional => SkillDomain::Communication,
            AttentionDomain::SelfReflection | AttentionDomain::RiskAssessment => SkillDomain::Meta,
            _ => SkillDomain::Unknown,
        }
    }

    fn confidence(&self) -> f64 {
        self.inner.utility
    }

    fn matches(&self, context: &SkillContext) -> f64 {
        if let Some(vsa) = &context.vsa_vector {
            if vsa.len() == self.inner.trigger.len() {
                return QuantizedVSA::similarity(&self.inner.trigger, vsa);
            }
        }
        if let Some(text) = &context.input_text {
            let text_bytes = text.as_bytes();
            let short = if text_bytes.len() > 4096 {
                &text_bytes[..4096]
            } else {
                text_bytes
            };
            let mut buf = vec![0u8; 4096];
            for (i, b) in short.iter().enumerate() {
                buf[i] = b & 1;
            }
            return QuantizedVSA::similarity(&self.inner.trigger, &buf);
        }
        0.0
    }

    fn execute(&mut self, _context: &SkillContext) -> Result<SkillOutcome, String> {
        Ok(SkillOutcome {
            success: true,
            output: format!("Executed VSA skill: {}", self.inner.name),
            execution_time_ms: 0,
            confidence_delta: 0.01,
        })
    }

    fn update(&mut self, outcome: &SkillOutcome) {
        self.inner.utility = (self.inner.utility + outcome.confidence_delta)
            .min(1.0)
            .max(0.0);
    }
}

// ─── Adapter: SkillChunk (Mechanism B — skill_compiler) ───────────────────

/// Adapter wrapping [`skill_compiler::SkillChunk`] into [`UnifiedSkill`].
#[derive(Debug)]
pub struct SkillChunkAdapter {
    pub inner: skill_compiler::SkillChunk,
}

impl SkillChunkAdapter {
    pub fn new(chunk: skill_compiler::SkillChunk) -> Self {
        Self { inner: chunk }
    }
}

impl UnifiedSkill for SkillChunkAdapter {
    fn id(&self) -> SkillId {
        SkillId(self.inner.id)
    }

    fn name(&self) -> &str {
        "skill_chunk"
    }

    fn representation(&self) -> SkillRepr {
        SkillRepr::Vsa(self.inner.trigger_pattern.clone())
    }

    fn domain(&self) -> SkillDomain {
        SkillDomain::Reasoning
    }

    fn confidence(&self) -> f64 {
        self.inner.strength
    }

    fn matches(&self, context: &SkillContext) -> f64 {
        if let Some(vsa) = &context.vsa_vector {
            return QuantizedVSA::similarity(&self.inner.trigger_pattern, vsa);
        }
        0.0
    }

    fn execute(&mut self, _context: &SkillContext) -> Result<SkillOutcome, String> {
        Ok(SkillOutcome {
            success: true,
            output: format!(
                "Executed SkillChunk {} with {} actions",
                self.inner.id,
                self.inner.action_sequence.len()
            ),
            execution_time_ms: 0,
            confidence_delta: 0.01,
        })
    }

    fn update(&mut self, outcome: &SkillOutcome) {
        self.inner.strength = (self.inner.strength + outcome.confidence_delta)
            .min(1.0)
            .max(0.0);
    }
}

// ─── Adapter: Skill (ECC Mechanism D — agent::skills) ───────────────────────
// NOTE: This adapter bridges the `agent::skills::Skill` type into the unified trait.
// Currently inactive — requires `agent` feature in Cargo.toml and the agent/skills module.

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa_skill(name: &str) -> skill_acc::VSASkill {
        let trigger = QuantizedVSA::seeded_random(42, 4096);
        let action = QuantizedVSA::seeded_random(7, 4096);
        let outcome = QuantizedVSA::seeded_random(99, 4096);
        skill_acc::VSASkill {
            id: 1,
            name: name.to_string(),
            trigger,
            action,
            outcome,
            domain: AttentionDomain::Code,
            success_rate: 0.8,
            use_count: 5,
            utility: 0.7,
            created_at: 0,
            last_used_at: 0,
            source_heuristic_ids: vec![],
            version: 1,
            refinement_count: 0,
        }
    }

    fn make_chunk() -> skill_compiler::SkillChunk {
        skill_compiler::SkillChunk {
            id: 10,
            trigger_pattern: QuantizedVSA::seeded_random(1, 4096),
            action_sequence: vec![QuantizedVSA::seeded_random(2, 4096)],
            expected_outcome: QuantizedVSA::seeded_random(3, 4096),
            frequency: 3,
            last_used_ns: 0,
            strength: 0.9,
        }
    }

    #[test]
    fn test_skill_id_from_str() {
        let id1 = SkillId::from_str("hello");
        let id2 = SkillId::from_str("hello");
        let id3 = SkillId::from_str("world");
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_vsa_adapter_wraps_correctly() {
        let vsa = make_vsa_skill("test_vsa");
        let adapter = VSASkillAdapter::new(vsa.clone());
        assert_eq!(adapter.id(), SkillId(vsa.id));
        assert_eq!(adapter.name(), "test_vsa");
        assert!(adapter.confidence() > 0.0);
        assert_eq!(adapter.domain(), SkillDomain::Coding);
    }

    #[test]
    fn test_vsa_adapter_matches_with_vsa_context() {
        let vsa = make_vsa_skill("test");
        let adapter = VSASkillAdapter::new(vsa.clone());
        let ctx = SkillContext {
            input_text: None,
            vsa_vector: Some(vsa.trigger.clone()),
            task_type: None,
        };
        let score = adapter.matches(&ctx);
        assert!(
            (score - 1.0).abs() < 1e-6,
            "identical VSA should give 1.0 score"
        );
    }

    #[test]
    fn test_vsa_adapter_execute_returns_ok() {
        let mut adapter = VSASkillAdapter::new(make_vsa_skill("test_exec"));
        let ctx = SkillContext::default();
        let result = adapter.execute(&ctx);
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(outcome.success);
        assert!(outcome.output.contains("test_exec"));
    }

    #[test]
    fn test_vsa_adapter_update_changes_confidence() {
        let mut adapter = VSASkillAdapter::new(make_vsa_skill("test_upd"));
        let before = adapter.confidence();
        adapter.update(&SkillOutcome {
            success: true,
            output: "ok".into(),
            execution_time_ms: 0,
            confidence_delta: 0.1,
        });
        assert!(
            (adapter.confidence() - before - 0.1).abs() < 1e-6,
            "confidence should increase by delta"
        );
    }

    #[test]
    fn test_chunk_adapter_wraps_correctly() {
        let chunk = make_chunk();
        let adapter = SkillChunkAdapter::new(chunk.clone());
        assert_eq!(adapter.id(), SkillId(chunk.id));
        assert!(adapter.confidence() > 0.0);
    }

    #[test]
    fn test_chunk_adapter_matches_with_vsa_context() {
        let chunk = make_chunk();
        let adapter = SkillChunkAdapter::new(chunk.clone());
        let ctx = SkillContext {
            input_text: None,
            vsa_vector: Some(chunk.trigger_pattern.clone()),
            task_type: None,
        };
        let score = adapter.matches(&ctx);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_chunk_adapter_execute_returns_ok() {
        let mut adapter = SkillChunkAdapter::new(make_chunk());
        let ctx = SkillContext::default();
        let result = adapter.execute(&ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_register_and_find() {
        let mut registry = UnifiedSkillRegistry::new();
        assert!(registry.is_empty());
        let id = registry.register(Box::new(VSASkillAdapter::new(make_vsa_skill("a"))));
        assert_eq!(registry.len(), 1);
        assert_eq!(id, SkillId(1));
    }

    #[test]
    fn test_registry_find_returns_top_k() {
        let mut registry = UnifiedSkillRegistry::new();
        let mut vsa = make_vsa_skill("best");
        vsa.utility = 0.9;
        registry.register(Box::new(VSASkillAdapter::new(vsa)));

        let mut vsa2 = make_vsa_skill("worst");
        vsa2.utility = 0.1;
        registry.register(Box::new(VSASkillAdapter::new(vsa2)));

        let ctx = SkillContext::default();
        let results = registry.find(&ctx, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0.name(), "best");
    }

    #[test]
    fn test_registry_find_filters_zero_score() {
        let mut registry = UnifiedSkillRegistry::new();
        let chunk = make_chunk();
        registry.register(Box::new(SkillChunkAdapter::new(chunk)));

        let ctx = SkillContext {
            input_text: Some("no match here".into()),
            vsa_vector: None,
            task_type: None,
        };
        let results = registry.find(&ctx, 5);
        assert!(
            results.is_empty(),
            "chunk needs VSA vector, text-only should yield 0 matches"
        );
    }

    #[test]
    fn test_registry_execute_best_returns_outcome() {
        let mut registry = UnifiedSkillRegistry::new();
        let chunk = make_chunk();
        let trigger = chunk.trigger_pattern.clone();
        registry.register(Box::new(SkillChunkAdapter::new(chunk)));

        let ctx = SkillContext {
            input_text: None,
            vsa_vector: Some(trigger),
            task_type: None,
        };
        let result = registry.execute_best(&ctx);
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert!(outcome.success);
    }

    #[test]
    fn test_registry_execute_best_no_match() {
        let mut registry = UnifiedSkillRegistry::new();
        let ctx = SkillContext::default();
        let result = registry.execute_best(&ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No matching skill"));
    }

    #[test]
    fn test_registry_can_hold_mixed_skill_types() {
        let mut registry = UnifiedSkillRegistry::new();
        registry.register(Box::new(VSASkillAdapter::new(make_vsa_skill("vsa"))));
        registry.register(Box::new(SkillChunkAdapter::new(make_chunk())));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_skill_domain_vsa_mapping() {
        let tests = vec![
            (AttentionDomain::Code, SkillDomain::Coding),
            (AttentionDomain::Reasoning, SkillDomain::Reasoning),
            (AttentionDomain::Memory, SkillDomain::Memory),
            (AttentionDomain::Planning, SkillDomain::Planning),
            (AttentionDomain::Social, SkillDomain::Communication),
            (AttentionDomain::SelfReflection, SkillDomain::Meta),
            (AttentionDomain::Creativity, SkillDomain::Unknown),
        ];
        for (ad, expected) in tests {
            let mut vsa = make_vsa_skill("x");
            vsa.domain = ad;
            let adapter = VSASkillAdapter::new(vsa);
            assert_eq!(
                adapter.domain(),
                expected,
                "AttentionDomain::{ad:?} → SkillDomain::{expected:?}"
            );
        }
    }

    #[test]
    fn test_registry_find_orders_by_score() {
        let mut registry = UnifiedSkillRegistry::new();
        let trigger = QuantizedVSA::seeded_random(100, 4096);

        let mut close = make_vsa_skill("close");
        close.trigger = trigger.clone();
        close.utility = 0.5;
        registry.register(Box::new(VSASkillAdapter::new(close)));

        let mut exact = make_vsa_skill("exact");
        exact.trigger = trigger.clone();
        exact.utility = 0.9;
        registry.register(Box::new(VSASkillAdapter::new(exact)));

        let ctx = SkillContext {
            input_text: None,
            vsa_vector: Some(trigger),
            task_type: None,
        };
        let results = registry.find(&ctx, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0.name(), "exact");
    }
}
