use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AttentionDomain {
    PatternMatch,
    Code,
    Semantic,
    Temporal,
    Planning,
    SelfReflection,
    ToolUse,
    GoalAlignment,
    RiskAssessment,
    Creativity,
    Reasoning,
    Memory,
    Social,
    Emotional,
}

impl AttentionDomain {
    pub fn all() -> Vec<AttentionDomain> {
        vec![
            AttentionDomain::PatternMatch,
            AttentionDomain::Code,
            AttentionDomain::Semantic,
            AttentionDomain::Temporal,
            AttentionDomain::Planning,
            AttentionDomain::SelfReflection,
            AttentionDomain::ToolUse,
            AttentionDomain::GoalAlignment,
            AttentionDomain::RiskAssessment,
            AttentionDomain::Creativity,
            AttentionDomain::Reasoning,
            AttentionDomain::Memory,
            AttentionDomain::Social,
            AttentionDomain::Emotional,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            AttentionDomain::PatternMatch => "pattern_match",
            AttentionDomain::Code => "code",
            AttentionDomain::Semantic => "semantic",
            AttentionDomain::Temporal => "temporal",
            AttentionDomain::Planning => "planning",
            AttentionDomain::SelfReflection => "self_reflection",
            AttentionDomain::ToolUse => "tool_use",
            AttentionDomain::GoalAlignment => "goal_alignment",
            AttentionDomain::RiskAssessment => "risk_assessment",
            AttentionDomain::Creativity => "creativity",
            AttentionDomain::Reasoning => "reasoning",
            AttentionDomain::Memory => "memory",
            AttentionDomain::Social => "social",
            AttentionDomain::Emotional => "emotional",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttentionHead {
    pub id: usize,
    pub domain: AttentionDomain,
    pub receptive_field: usize,
    pub activation: f64,
    pub specialization: Vec<f64>,
    pub focus: Vec<String>,
    pub decay_rate: f64,
    pub priority: u8,
}

impl AttentionHead {
    pub fn new(id: usize, domain: AttentionDomain) -> Self {
        Self {
            id,
            domain,
            receptive_field: 10,
            activation: 0.0,
            specialization: Vec::new(),
            focus: Vec::new(),
            decay_rate: 0.1,
            priority: 5,
        }
    }

    pub fn salience(&self, novelty: f64, coherence: f64) -> f64 {
        self.activation * novelty * coherence
    }

    pub fn stimulate(&mut self, amount: f64) {
        self.activation = (self.activation + amount).min(1.0);
    }

    pub fn decay(&mut self) {
        self.activation = (self.activation - self.decay_rate).max(0.0);
    }

    pub fn focus_on(&mut self, concept: &str) {
        if !self.focus.contains(&concept.to_string()) {
            self.focus.push(concept.to_string());
        }
        self.stimulate(0.1);
    }

    pub fn is_activated(&self, threshold: f64) -> bool {
        self.activation >= threshold
    }
}

#[derive(Debug, Clone)]
pub struct AttentionProfile {
    pub dominant: AttentionDomain,
    pub distribution: HashMap<AttentionDomain, f64>,
    pub num_activated_heads: usize,
}

impl AttentionProfile {
    pub fn new(
        dominant: AttentionDomain,
        distribution: HashMap<AttentionDomain, f64>,
        num_activated_heads: usize,
    ) -> Self {
        Self {
            dominant,
            distribution,
            num_activated_heads,
        }
    }
}

pub struct AttentionManager {
    pub heads: Vec<AttentionHead>,
    pub global_threshold: f64,
}

impl AttentionManager {
    pub fn new(threshold: f64) -> Self {
        let heads: Vec<AttentionHead> = AttentionDomain::all()
            .into_iter()
            .enumerate()
            .map(|(i, domain)| AttentionHead::new(i, domain))
            .collect();
        Self {
            heads,
            global_threshold: threshold,
        }
    }

    pub fn stimulate_domain(&mut self, domain: AttentionDomain, amount: f64) {
        if let Some(head) = self.heads.iter_mut().find(|h| h.domain == domain) {
            head.stimulate(amount);
        }
    }

    pub fn decay_all(&mut self) {
        for head in &mut self.heads {
            head.decay();
        }
    }

    pub fn active_heads(&self) -> Vec<&AttentionHead> {
        self.heads
            .iter()
            .filter(|h| h.activation >= self.global_threshold)
            .collect()
    }

    pub fn dominant_domain(&self) -> Option<AttentionDomain> {
        self.heads
            .iter()
            .max_by(|a, b| {
                a.activation
                    .partial_cmp(&b.activation)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .filter(|h| h.activation > 0.0)
            .map(|h| h.domain)
    }

    pub fn profile(&self) -> AttentionProfile {
        let distribution: HashMap<AttentionDomain, f64> = self
            .heads
            .iter()
            .map(|h| (h.domain, h.activation))
            .collect();
        let dominant = self
            .dominant_domain()
            .unwrap_or(AttentionDomain::PatternMatch);
        let num_activated = self.active_heads().len();
        AttentionProfile::new(dominant, distribution, num_activated)
    }

    pub fn reset(&mut self) {
        for head in &mut self.heads {
            head.activation = 0.0;
            head.focus.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attention_head_new() {
        let h = AttentionHead::new(0, AttentionDomain::Code);
        assert_eq!(h.domain, AttentionDomain::Code);
        assert_eq!(h.activation, 0.0);
        assert_eq!(h.id, 0);
    }

    #[test]
    fn test_stimulate_and_decay() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.stimulate(0.5);
        assert!((h.activation - 0.5).abs() < 1e-6);
        h.decay();
        assert!((h.activation - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_activation_capped() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.stimulate(1.5);
        assert!((h.activation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_salience_formula() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.stimulate(0.8);
        let s = h.salience(0.5, 0.5);
        assert!((s - 0.8 * 0.5 * 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_attention_manager_decay_all() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::Code, 0.9);
        mgr.stimulate_domain(AttentionDomain::Planning, 0.7);
        let code_idx = AttentionDomain::all()
            .iter()
            .position(|d| *d == AttentionDomain::Code)
            .expect("value should be ok in test");
        let plan_idx = AttentionDomain::all()
            .iter()
            .position(|d| *d == AttentionDomain::Planning)
            .expect("value should be ok in test");
        assert_eq!(mgr.active_heads().len(), 2);
        mgr.decay_all();
        assert!((mgr.heads[code_idx].activation - 0.8).abs() < 1e-6);
        assert!((mgr.heads[plan_idx].activation - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_dominant_domain() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::Code, 0.9);
        mgr.stimulate_domain(AttentionDomain::Planning, 0.3);
        assert_eq!(mgr.dominant_domain(), Some(AttentionDomain::Code));
    }

    #[test]
    fn test_attention_profile() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::SelfReflection, 0.8);
        let profile = mgr.profile();
        assert_eq!(profile.dominant, AttentionDomain::SelfReflection);
        assert!(profile.num_activated_heads >= 1);
        assert!(
            profile
                .distribution
                .get(&AttentionDomain::SelfReflection)
                .copied()
                .unwrap_or(0.0)
                > 0.0
        );
    }

    #[test]
    fn test_focus_on_concept() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.focus_on("rust");
        assert!(h.focus.contains(&"rust".to_string()));
        assert!(h.activation > 0.0);
        let act_before = h.activation;
        h.focus_on("rust");
        assert_eq!(h.focus.len(), 1);
        assert!(h.activation >= act_before);
    }

    #[test]
    fn test_reset_manager() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::Code, 0.9);
        mgr.stimulate_domain(AttentionDomain::Planning, 0.7);
        assert!(mgr.active_heads().len() > 0);
        mgr.reset();
        assert_eq!(mgr.active_heads().len(), 0);
    }

    #[test]
    fn test_all_domains_count() {
        let domains = AttentionDomain::all();
        assert_eq!(domains.len(), 10);
    }

    #[test]
    fn test_attention_head_is_activated() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        assert!(!h.is_activated(0.5));
        h.stimulate(0.6);
        assert!(h.is_activated(0.5));
    }
}
