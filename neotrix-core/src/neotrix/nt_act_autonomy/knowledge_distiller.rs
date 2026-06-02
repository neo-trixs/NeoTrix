use crate::core::nt_core_cap::CapabilityVector;
use crate::neotrix::nt_act_goal::rl_feedback::RLFeedbackLoop;
use crate::core::ReasoningHexagram;

#[derive(Debug, Clone)]
pub struct Principle {
    pub id: String,
    pub description: String,
    pub confidence: f64,
    pub dimension: String,
    pub delta: f64,
    pub source_session: String,
    pub category: PrincipleCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrincipleCategory {
    Pattern,
    AntiPattern,
    Optimization,
    Insight,
}

#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: String,
    pub user_messages: Vec<String>,
    pub actions_taken: Vec<String>,
    pub outcomes: Vec<String>,
    pub reward_signal: f64,
    pub timestamp: u64,
    pub task_type: Option<String>,
    pub e8_mode: Option<ReasoningHexagram>,
    pub edit_types: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct KnowledgeDistiller {
    principles: Vec<Principle>,
    min_confidence: f64,
    _rl_loop: RLFeedbackLoop,
    distill_count: u64,
}

impl KnowledgeDistiller {
    pub fn new() -> Self {
        Self {
            principles: Vec::new(),
            min_confidence: 0.5,
            _rl_loop: RLFeedbackLoop::default(),
            distill_count: 0,
        }
    }

    pub fn distill(&mut self, session: &SessionRecord) -> Vec<Principle> {
        let new_principles = self.generate_principles(session);
        self.principles.extend(new_principles.clone());
        self.distill_count += 1;
        new_principles
    }

    pub fn absorb(&mut self, cv: &mut CapabilityVector) -> u32 {
        let mut absorbed = 0u32;
        let retained: Vec<Principle> = self.principles.drain(..).collect();
        for principle in retained {
            if principle.confidence < self.min_confidence {
                continue;
            }
            if let Some(idx) = CapabilityVector::index_from_name(&principle.dimension) {
                if idx < cv.arr.len() {
                    let delta = principle.delta * principle.confidence;
                    cv.arr[idx] = (cv.arr[idx] + delta).clamp(0.0, 1.0);
                    absorbed += 1;
                }
            }
        }
        absorbed
    }

    fn generate_principles(&self, session: &SessionRecord) -> Vec<Principle> {
        let mut principles = Vec::new();

        // Rule 1: Reward-based reinforcement (existing, evolved)
        if session.reward_signal > 0.3 && !session.actions_taken.is_empty() {
            for (i, action) in session.actions_taken.iter().enumerate() {
                let desc = format!("Reinforce: {}", action);
                let id = format!("{}-p-{}", session.id, i);
                principles.push(Principle {
                    id,
                    description: desc,
                    confidence: session.reward_signal.min(1.0),
                    dimension: "analysis".to_string(),
                    delta: 0.15,
                    source_session: session.id.clone(),
                    category: PrincipleCategory::Pattern,
                });
            }
        }

        // Rule 2: Negative reward anti-patterns
        if session.reward_signal < -0.3 {
            for (i, action) in session.actions_taken.iter().enumerate() {
                let desc = format!("Avoid: {}", action);
                let id = format!("{}-ap-{}", session.id, i);
                principles.push(Principle {
                    id,
                    description: desc,
                    confidence: (-session.reward_signal).min(1.0),
                    dimension: "quality_gates".to_string(),
                    delta: -0.15,
                    source_session: session.id.clone(),
                    category: PrincipleCategory::AntiPattern,
                });
            }
        }

        // Rule 3: Keyword "should always" (existing)
        for msg in &session.user_messages {
            if msg.contains("should always") {
                let id = format!("{}-kw-p", session.id);
                principles.push(Principle {
                    id,
                    description: format!("Rule: {}", msg),
                    confidence: 0.7,
                    dimension: "inference_depth".to_string(),
                    delta: 0.2,
                    source_session: session.id.clone(),
                    category: PrincipleCategory::Pattern,
                });
            }
        }

        // Rule 4: Refactor/optimize keywords
        for action in &session.actions_taken {
            let lower = action.to_lowercase();
            if lower.contains("refactor") || lower.contains("optimize") {
                let id = format!("{}-opt", session.id);
                principles.push(Principle {
                    id,
                    description: format!("Optimization: {}", action),
                    confidence: 0.6,
                    dimension: "verification".to_string(),
                    delta: 0.1,
                    source_session: session.id.clone(),
                    category: PrincipleCategory::Optimization,
                });
            }
        }

        // Rule 5: Task-type specific patterns (NEW)
        if let Some(ref task_type) = session.task_type {
            let lower_tt = task_type.to_lowercase();
            if (lower_tt.contains("code") || lower_tt.contains("debug")) && session.reward_signal > 0.5 {
                principles.push(Principle {
                    id: format!("{}-tt-code", session.id),
                    description: format!("Code task success with E8 mode {:?}", session.e8_mode),
                    confidence: session.reward_signal.min(1.0),
                    dimension: "code_generation".to_string(),
                    delta: 0.2,
                    source_session: session.id.clone(),
                    category: PrincipleCategory::Pattern,
                });
            }
            if lower_tt.contains("design") && session.reward_signal > 0.5 {
                principles.push(Principle {
                    id: format!("{}-tt-design", session.id),
                    description: format!("Design task success"),
                    confidence: session.reward_signal.min(1.0),
                    dimension: "system_design".to_string(),
                    delta: 0.15,
                    source_session: session.id.clone(),
                    category: PrincipleCategory::Pattern,
                });
            }
            if lower_tt.contains("plan") && session.reward_signal > 0.3 {
                principles.push(Principle {
                    id: format!("{}-tt-plan", session.id),
                    description: format!("Planning pattern"),
                    confidence: session.reward_signal.min(1.0),
                    dimension: "planning".to_string(),
                    delta: 0.1,
                    source_session: session.id.clone(),
                    category: PrincipleCategory::Pattern,
                });
            }
        }

        // Rule 6: Edit-type effectiveness patterns (NEW)
        for et in &session.edit_types {
            let lower_et = et.to_lowercase();
            if lower_et.contains("adjust") && session.reward_signal > 0.5 {
                principles.push(Principle {
                    id: format!("{}-edit-{}", session.id, et),
                    description: format!("Effective edit: {}", et),
                    confidence: session.reward_signal.min(1.0),
                    dimension: "precision".to_string(),
                    delta: 0.05,
                    source_session: session.id.clone(),
                    category: PrincipleCategory::Pattern,
                });
            }
            if lower_et.contains("normalize") && session.reward_signal > 0.4 {
                principles.push(Principle {
                    id: format!("{}-nrm", session.id),
                    description: format!("Normalization beneficial (reward={:.2})", session.reward_signal),
                    confidence: session.reward_signal.min(1.0),
                    dimension: "stability".to_string(),
                    delta: 0.08,
                    source_session: session.id.clone(),
                    category: PrincipleCategory::Optimization,
                });
            }
        }

        // Rule 7: E8 mode insight (NEW)
        if let Some(mode) = session.e8_mode {
            if session.reward_signal.abs() > 0.5 {
                let cat = if session.reward_signal > 0.0 { PrincipleCategory::Insight } else { PrincipleCategory::AntiPattern };
                let dir = if session.reward_signal > 0.0 { "effective" } else { "ineffective" };
                principles.push(Principle {
                    id: format!("{}-e8-{}", session.id, mode.0),
                    description: format!("E8 mode {} {} for task", mode.0, dir),
                    confidence: session.reward_signal.abs().min(1.0),
                    dimension: "reasoning_depth".to_string(),
                    delta: if session.reward_signal > 0.0 { 0.1 } else { -0.05 },
                    source_session: session.id.clone(),
                    category: cat,
                });
            }
        }

        // Rule 8: High confidence from consistent success (NEW)
        if session.reward_signal > 0.7 && !session.edit_types.is_empty() {
            principles.push(Principle {
                id: format!("{}-hq", session.id),
                description: "High-quality session: multiple successful edits".to_string(),
                confidence: 0.8,
                dimension: "quality_gates".to_string(),
                delta: 0.05,
                source_session: session.id.clone(),
                category: PrincipleCategory::Insight,
            });
        }

        principles
    }

    pub fn principles(&self) -> &[Principle] {
        &self.principles
    }

    pub fn summary(&self) -> String {
        let total = self.principles.len();
        let patterns = self.principles.iter().filter(|p| matches!(p.category, PrincipleCategory::Pattern)).count();
        let anti = self.principles.iter().filter(|p| matches!(p.category, PrincipleCategory::AntiPattern)).count();
        let opts = self.principles.iter().filter(|p| matches!(p.category, PrincipleCategory::Optimization)).count();
        let insights = self.principles.iter().filter(|p| matches!(p.category, PrincipleCategory::Insight)).count();
        format!(
            "KnowledgeDistiller: {} distill runs | {} active principles (Patterns: {}, AntiPatterns: {}, Optimizations: {}, Insights: {}) | min_conf={}",
            self.distill_count, total, patterns, anti, opts, insights, self.min_confidence,
        )
    }
}

impl Default for KnowledgeDistiller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn positive_session() -> SessionRecord {
        SessionRecord {
            id: "sess-1".into(),
            user_messages: vec!["add search feature".into()],
            actions_taken: vec!["implemented search".into(), "added tests".into()],
            outcomes: vec!["search works".into()],
            reward_signal: 0.8,
            timestamp: 1000,
            task_type: None,
            e8_mode: None,
            edit_types: vec![],
        }
    }

    fn negative_session() -> SessionRecord {
        SessionRecord {
            id: "sess-2".into(),
            user_messages: vec!["fix bug".into()],
            actions_taken: vec!["patched workaround".into()],
            outcomes: vec!["still broken".into()],
            reward_signal: -0.6,
            timestamp: 2000,
            task_type: None,
            e8_mode: None,
            edit_types: vec![],
        }
    }

    fn keyword_session() -> SessionRecord {
        SessionRecord {
            id: "sess-3".into(),
            user_messages: vec!["you should always validate input".into()],
            actions_taken: vec![],
            outcomes: vec![],
            reward_signal: 0.0,
            timestamp: 3000,
            task_type: None,
            e8_mode: None,
            edit_types: vec![],
        }
    }

    #[test]
    fn test_distill_positive_session() {
        let session = positive_session();
        let mut d = KnowledgeDistiller::new();
        let principles = d.distill(&session);
        assert!(!principles.is_empty(), "positive session should produce principles");
        assert!(principles.iter().all(|p| p.category == PrincipleCategory::Pattern));
        assert_eq!(principles.len(), 2);
    }

    #[test]
    fn test_distill_negative_session() {
        let session = negative_session();
        let mut d = KnowledgeDistiller::new();
        let principles = d.distill(&session);
        assert!(!principles.is_empty(), "negative session should produce principles");
        assert!(principles.iter().all(|p| p.category == PrincipleCategory::AntiPattern));
    }

    #[test]
    fn test_keyword_should_always() {
        let session = keyword_session();
        let d = KnowledgeDistiller::new();
        let principles = d.generate_principles(&session);
        assert_eq!(principles.len(), 1, "should always keyword should produce one pattern");
        assert_eq!(principles[0].category, PrincipleCategory::Pattern);
        assert!((principles[0].confidence - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_low_confidence_discarded() {
        let mut d = KnowledgeDistiller::new();
        d.principles.push(Principle {
            id: "low".into(),
            description: "low confidence".into(),
            confidence: 0.1,
            dimension: "analysis".into(),
            delta: 0.5,
            source_session: "s".into(),
            category: PrincipleCategory::Pattern,
        });
        let mut cv = CapabilityVector::default();
        let absorbed = d.absorb(&mut cv);
        assert_eq!(absorbed, 0, "low confidence principle should be discarded");
    }

    #[test]
    fn test_absorb_updates_capability() {
        let mut d = KnowledgeDistiller::new();
        d.principles.push(Principle {
            id: "p1".into(),
            description: "test principle".into(),
            confidence: 1.0,
            dimension: "analysis".into(),
            delta: 0.2,
            source_session: "s".into(),
            category: PrincipleCategory::Pattern,
        });
        let mut cv = CapabilityVector::default();
        let absorbed = d.absorb(&mut cv);
        assert_eq!(absorbed, 1);
        assert!((cv.analysis() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_summary_format() {
        let d = KnowledgeDistiller::new();
        let s = d.summary();
        assert!(s.contains("KnowledgeDistiller"));
        assert!(s.contains("min_conf"));
        assert!(s.contains("active principles"));
    }

    #[test]
    fn test_empty_session_no_principles() {
        let session = SessionRecord {
            id: "empty".into(),
            user_messages: vec![],
            actions_taken: vec![],
            outcomes: vec![],
            reward_signal: 0.0,
            timestamp: 0,
            task_type: None,
            e8_mode: None,
            edit_types: vec![],
        };
        let d = KnowledgeDistiller::new();
        let principles = d.generate_principles(&session);
        assert!(principles.is_empty(), "empty session should produce no principles");
    }

    #[test]
    fn test_optimization_keyword() {
        let session = SessionRecord {
            id: "opt-sess".into(),
            user_messages: vec![],
            actions_taken: vec!["refactored module".into(), "optimized query".into()],
            outcomes: vec![],
            reward_signal: 0.0,
            timestamp: 5000,
            task_type: None,
            e8_mode: None,
            edit_types: vec![],
        };
        let d = KnowledgeDistiller::new();
        let principles = d.generate_principles(&session);
        assert_eq!(principles.len(), 2);
        assert!(principles.iter().all(|p| p.category == PrincipleCategory::Optimization));
    }

    #[test]
    fn test_absorb_clears_principles() {
        let mut d = KnowledgeDistiller::new();
        d.principles.push(Principle {
            id: "p1".into(),
            description: "test".into(),
            confidence: 0.9,
            dimension: "analysis".into(),
            delta: 0.1,
            source_session: "s".into(),
            category: PrincipleCategory::Pattern,
        });
        let mut cv = CapabilityVector::default();
        let _ = d.absorb(&mut cv);
        assert!(d.principles().is_empty(), "principles should be cleared after absorb");
    }
}
