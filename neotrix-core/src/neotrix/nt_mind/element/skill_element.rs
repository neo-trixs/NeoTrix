use super::bus::ElementBus;
use super::{CapabilityAccess, CapabilityOp, Element, ElementError, ElementType};
use crate::core::nt_core_self::{AttentionDomain, CrystalRegistry, StrategyKind, ThinkingTrace};
use std::any::Any;

#[derive(Debug)]
pub struct SkillElement {
    pub registry: CrystalRegistry,
    init_called: bool,
    started: bool,
}

impl Default for SkillElement {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillElement {
    pub fn new() -> Self {
        Self {
            registry: CrystalRegistry::new(),
            init_called: false,
            started: false,
        }
    }

    pub fn extract(&mut self, trace: &ThinkingTrace, iteration: usize) -> Option<usize> {
        self.registry.extract_from_trace(trace, iteration)
    }

    pub fn recommend(&self, strategy: StrategyKind, domain: AttentionDomain) -> Option<String> {
        self.registry
            .find_similar(strategy, domain)
            .map(|c| format!("💎 #{}: {} (eff={:.3})", c.id, c.pattern, c.effectiveness))
    }

    pub fn crystal_count(&self) -> usize {
        self.registry.crystals.len()
    }
}

impl Element for SkillElement {
    fn id(&self) -> &str {
        "element.skill"
    }
    fn name(&self) -> &str {
        "Skill Crystalization"
    }
    fn version(&self) -> &str {
        "0.2.0"
    }
    fn element_type(&self) -> ElementType {
        ElementType::Feature
    }

    fn init(&mut self, _bus: &ElementBus) -> Result<(), ElementError> {
        self.init_called = true;
        Ok(())
    }

    fn start(&mut self) -> Result<(), ElementError> {
        self.started = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ElementError> {
        self.started = false;
        Ok(())
    }

    fn destroy(&mut self) -> Result<(), ElementError> {
        self.init_called = false;
        self.started = false;
        Ok(())
    }

    fn depends_on(&self) -> Vec<&str> {
        vec!["element.capability"]
    }

    fn provides(&self) -> Vec<CapabilityAccess> {
        vec![
            CapabilityAccess {
                name: "skill.extract",
                description: "Extract a skill crystal from a thinking trace",
                operations: vec![CapabilityOp::Command],
            },
            CapabilityAccess {
                name: "skill.recommend",
                description: "Recommend a skill by strategy and domain",
                operations: vec![CapabilityOp::Query],
            },
        ]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self::{ReflectionGrade, ThinkingStep, ThinkingTrace};

    fn make_good_trace() -> ThinkingTrace {
        let mut trace = ThinkingTrace::new(0, "refactor module");
        trace.grade = ReflectionGrade::Good;
        trace.steps.push(
            ThinkingStep::new(1, "analyze", StrategyKind::Reflection)
                .with_domain(AttentionDomain::Code),
        );
        trace
    }

    #[test]
    fn test_new_skill_element() {
        let el = SkillElement::new();
        assert_eq!(el.id(), "element.skill");
        assert_eq!(el.element_type(), ElementType::Feature);
        assert_eq!(el.crystal_count(), 0);
    }

    #[test]
    fn test_extract_from_trace() {
        let mut el = SkillElement::new();
        let trace = make_good_trace();
        let id = el.extract(&trace, 1);
        assert!(id.is_some());
        assert_eq!(el.crystal_count(), 1);
    }

    #[test]
    fn test_recommend_skill() {
        let mut el = SkillElement::new();
        let trace = make_good_trace();
        el.extract(&trace, 1);

        let rec = el.recommend(StrategyKind::Reflection, AttentionDomain::Code);
        assert!(rec.is_some());
        assert!(rec.expect("rec should be ok in test").contains("💎"));
    }

    #[test]
    fn test_depends_on_capability() {
        let el = SkillElement::new();
        let deps = el.depends_on();
        assert!(deps.contains(&"element.capability"));
    }

    #[test]
    fn test_recommend_not_found() {
        let el = SkillElement::new();
        let rec = el.recommend(StrategyKind::Direct, AttentionDomain::Code);
        assert!(rec.is_none());
    }
}
