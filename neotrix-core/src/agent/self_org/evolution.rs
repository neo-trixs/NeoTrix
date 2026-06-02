//! # Template Evolution + KEEP Protocol
//!
//! Template evolution system for agent patterns with KEEP/MODIFY/DEPRECATE actions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use neotrix_types::core::nt_core_self_org::SelfOrgProtocol;

/// An agent template encapsulating roles, protocol, and evolutionary metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub id: String,
    pub name: String,
    pub roles: Vec<String>,
    pub protocol: SelfOrgProtocol,
    pub success_rate: f64,
    pub iteration_count: u64,
}

/// KEEP actions for template evolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeepAction {
    KEEP,
    MODIFY(String),
    DEPRECATE,
}

/// Template evolution registry managing agent patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEvolution {
    pub templates: HashMap<String, AgentTemplate>,
}

impl TemplateEvolution {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn register(&mut self, template: AgentTemplate) {
        let id = template.id.clone();
        self.templates.insert(id, template);
    }

    pub fn get(&self, id: &str) -> Option<&AgentTemplate> {
        self.templates.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut AgentTemplate> {
        self.templates.get_mut(id)
    }

    pub fn record_iteration(&mut self, id: &str, success: bool) {
        if let Some(template) = self.templates.get_mut(id) {
            template.iteration_count += 1;
            let n = template.iteration_count as f64;
            template.success_rate =
                ((template.success_rate * (n - 1.0)) + if success { 1.0 } else { 0.0 }) / n;
        }
    }

    pub fn evaluate(&self, id: &str) -> KeepAction {
        match self.templates.get(id) {
            Some(t) if t.success_rate >= 0.7 => KeepAction::KEEP,
            Some(t) if t.success_rate >= 0.3 => {
                KeepAction::MODIFY(format!("low success rate: {:.2}", t.success_rate))
            }
            Some(_) => KeepAction::DEPRECATE,
            None => KeepAction::DEPRECATE,
        }
    }
}

impl Default for TemplateEvolution {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_template(id: &str, success_rate: f64) -> AgentTemplate {
        AgentTemplate {
            id: id.into(), name: format!("Template {}", id),
            roles: vec!["coder".into()],
            protocol: SelfOrgProtocol::default(),
            success_rate,
            iteration_count: 0,
        }
    }

    #[test]
    fn test_template_evolution_new() {
        let te = TemplateEvolution::new();
        assert!(te.templates.is_empty());
    }

    #[test]
    fn test_template_evolution_register_and_get() {
        let mut te = TemplateEvolution::new();
        let t = make_template("t1", 0.8);
        te.register(t);
        assert!(te.get("t1").is_some());
    }

    #[test]
    fn test_template_evolution_get_mut() {
        let mut te = TemplateEvolution::new();
        te.register(make_template("t1", 0.5));
        let t = te.get_mut("t1").unwrap();
        t.success_rate = 0.9;
        assert!((te.get("t1").unwrap().success_rate - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_template_evolution_record_iteration() {
        let mut te = TemplateEvolution::new();
        te.register(make_template("t1", 0.0));
        te.record_iteration("t1", true);
        let t = te.get("t1").unwrap();
        assert_eq!(t.iteration_count, 1);
        assert!((t.success_rate - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_template_evolution_record_iteration_multiple() {
        let mut te = TemplateEvolution::new();
        te.register(make_template("t1", 0.0));
        te.record_iteration("t1", true);
        te.record_iteration("t1", false);
        let t = te.get("t1").unwrap();
        assert_eq!(t.iteration_count, 2);
        assert!((t.success_rate - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_evaluate_keep_when_high_success() {
        let mut te = TemplateEvolution::new();
        te.register(make_template("t1", 0.85));
        assert_eq!(te.evaluate("t1"), KeepAction::KEEP);
    }

    #[test]
    fn test_evaluate_modify_when_medium_success() {
        let mut te = TemplateEvolution::new();
        te.register(make_template("t1", 0.5));
        match te.evaluate("t1") {
            KeepAction::MODIFY(_) => {},
            _ => panic!("expected MODIFY"),
        }
    }

    #[test]
    fn test_evaluate_deprecate_when_low_success() {
        let mut te = TemplateEvolution::new();
        te.register(make_template("t1", 0.2));
        assert_eq!(te.evaluate("t1"), KeepAction::DEPRECATE);
    }

    #[test]
    fn test_evaluate_deprecate_when_missing() {
        let te = TemplateEvolution::new();
        assert_eq!(te.evaluate("nonexistent"), KeepAction::DEPRECATE);
    }

    #[test]
    fn test_evaluate_modify_message() {
        let mut te = TemplateEvolution::new();
        te.register(make_template("t1", 0.4));
        match te.evaluate("t1") {
            KeepAction::MODIFY(msg) => assert!(msg.contains("0.40")),
            _ => panic!("expected MODIFY with message"),
        }
    }
}
