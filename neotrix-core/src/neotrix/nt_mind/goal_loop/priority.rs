use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RICEScore {
    pub reach: f64,
    pub impact: f64,
    pub confidence: f64,
    pub effort: f64,
}

impl RICEScore {
    pub fn new(reach: f64, impact: f64, confidence: f64, effort: f64) -> Self {
        Self {
            reach,
            impact,
            confidence,
            effort,
        }
    }
    pub fn compute(&self) -> f64 {
        (self.reach * self.impact * self.confidence) / self.effort.max(0.1)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICEScore {
    pub impact: f64,
    pub confidence: f64,
    pub ease: f64,
}

impl ICEScore {
    pub fn new(impact: f64, confidence: f64, ease: f64) -> Self {
        Self {
            impact,
            confidence,
            ease,
        }
    }
    pub fn compute(&self) -> f64 {
        (self.impact * self.confidence) / self.ease.max(0.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoscowClass {
    MustHave,
    ShouldHave,
    CouldHave,
    WontHave,
}

impl MoscowClass {
    pub fn from_score(score: f64) -> Self {
        if score >= 8.0 {
            MoscowClass::MustHave
        } else if score >= 5.0 {
            MoscowClass::ShouldHave
        } else if score >= 2.0 {
            MoscowClass::CouldHave
        } else {
            MoscowClass::WontHave
        }
    }
    pub fn rank(&self) -> u8 {
        match self {
            MoscowClass::MustHave => 4,
            MoscowClass::ShouldHave => 3,
            MoscowClass::CouldHave => 2,
            MoscowClass::WontHave => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriorityFramework {
    RICE,
    ICE,
    Moscow,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityDecision {
    pub goal_id: String,
    pub framework: PriorityFramework,
    pub score: f64,
    pub moscow_class: Option<MoscowClass>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityEngine {
    pub framework: PriorityFramework,
    pub weights: [f64; 4],
}

impl Default for PriorityEngine {
    fn default() -> Self {
        Self {
            framework: PriorityFramework::Hybrid,
            weights: [1.0, 1.0, 5.0, 0.5],
        }
    }
}

impl PriorityEngine {
    pub fn new(framework: PriorityFramework) -> Self {
        Self {
            framework,
            ..Default::default()
        }
    }

    pub fn evaluate(&self, description: &str, complexity: f64) -> f64 {
        let word_count = description.split_whitespace().count() as f64;
        let has_critical = description.to_lowercase().contains("critical")
            || description.to_lowercase().contains("nt_shield");
        let has_urgent = description.to_lowercase().contains("urgent")
            || description.to_lowercase().contains("blocker");
        let has_improve = description.to_lowercase().contains("improve")
            || description.to_lowercase().contains("optimize");

        let reach = if has_critical {
            9.0
        } else if has_urgent {
            7.0
        } else {
            4.0
        };
        let impact = if has_critical {
            9.0
        } else if has_improve {
            3.0
        } else {
            5.0
        };
        let confidence = (5.0 - (word_count / 20.0).min(4.0)).max(1.0);
        let effort = (complexity * 5.0).clamp(1.0, 10.0);
        let ease = (11.0 - effort).max(0.5);

        let rice = RICEScore::new(reach, impact, confidence, effort).compute();
        let ice = ICEScore::new(impact, confidence, ease).compute();

        match self.framework {
            PriorityFramework::RICE => rice.min(100.0),
            PriorityFramework::ICE => ice.min(100.0),
            PriorityFramework::Moscow => MoscowClass::from_score(rice).rank() as f64 * 2.5,
            PriorityFramework::Hybrid => {
                (rice * self.weights[0] + ice * self.weights[1])
                    / (self.weights[0] + self.weights[1])
            }
        }
    }

    pub fn rank(&self, goals: &[(String, f64)]) -> Vec<usize> {
        let scores: Vec<f64> = goals
            .iter()
            .map(|(desc, complexity)| self.evaluate(desc, *complexity))
            .collect();
        let mut indices: Vec<usize> = (0..goals.len()).collect();
        indices.sort_by(|&a, &b| {
            scores[b]
                .partial_cmp(&scores[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        indices
    }

    pub fn to_moscow(&self, score: f64) -> MoscowClass {
        MoscowClass::from_score(score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rice_computation() {
        let rice = RICEScore::new(8.0, 7.0, 9.0, 3.0);
        let score = rice.compute();
        assert!((score - 168.0).abs() < 0.01);
    }

    #[test]
    fn test_ice_computation() {
        let ice = ICEScore::new(8.0, 7.0, 3.0);
        let score = ice.compute();
        assert!((score - 18.666).abs() < 0.01);
    }

    #[test]
    fn test_moscow_classification() {
        assert_eq!(MoscowClass::from_score(9.0), MoscowClass::MustHave);
        assert_eq!(MoscowClass::from_score(6.0), MoscowClass::ShouldHave);
        assert_eq!(MoscowClass::from_score(3.0), MoscowClass::CouldHave);
        assert_eq!(MoscowClass::from_score(1.0), MoscowClass::WontHave);
    }

    #[test]
    fn test_priority_engine_rank() {
        let engine = PriorityEngine::new(PriorityFramework::Hybrid);
        let goals = vec![
            ("optimize css spacing".to_string(), 1.0),
            ("refactor button colors".to_string(), 1.0),
            ("urgent critical nt_shield bug".to_string(), 5.0),
        ];
        let ranked = engine.rank(&goals);
        assert_eq!(ranked[0], 2);
        assert_eq!(ranked[2], 0);
    }

    #[test]
    fn test_rice_zero_effort() {
        let rice = RICEScore::new(5.0, 5.0, 5.0, 0.0);
        let score = rice.compute();
        assert!(score <= 2500.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_priority_engine_consistency() {
        let engine = PriorityEngine::new(PriorityFramework::RICE);
        let score_a = engine.evaluate("fix critical nt_shield bug", 5.0);
        let score_b = engine.evaluate("polish button color", 1.0);
        assert!(score_a > score_b);
    }
}
