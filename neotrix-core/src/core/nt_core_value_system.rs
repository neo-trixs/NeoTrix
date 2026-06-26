use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreValue {
    Curiosity,
    KnowledgeGrowth,
    Coherence,
    Autonomy,
    Helpfulness,
    Truthfulness,
    Efficiency,
}

impl CoreValue {
    pub fn name(&self) -> &'static str {
        match self {
            CoreValue::Curiosity => "curiosity",
            CoreValue::KnowledgeGrowth => "knowledge_growth",
            CoreValue::Coherence => "coherence",
            CoreValue::Autonomy => "autonomy",
            CoreValue::Helpfulness => "helpfulness",
            CoreValue::Truthfulness => "truthfulness",
            CoreValue::Efficiency => "efficiency",
        }
    }

    pub fn all() -> &'static [CoreValue] {
        &[
            CoreValue::Curiosity,
            CoreValue::KnowledgeGrowth,
            CoreValue::Coherence,
            CoreValue::Autonomy,
            CoreValue::Helpfulness,
            CoreValue::Truthfulness,
            CoreValue::Efficiency,
        ]
    }

    pub fn default_weight(&self) -> f64 {
        match self {
            CoreValue::Curiosity => 0.20,
            CoreValue::KnowledgeGrowth => 0.20,
            CoreValue::Coherence => 0.15,
            CoreValue::Autonomy => 0.15,
            CoreValue::Helpfulness => 0.12,
            CoreValue::Truthfulness => 0.10,
            CoreValue::Efficiency => 0.08,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueWeight {
    pub value: CoreValue,
    pub weight: f64,
    pub satisfaction: f64,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueSystem {
    pub weights: Vec<ValueWeight>,
    pub total_cycles: u64,
    pub learning_rate: f64,
    pub empathy_bias: f64,
    pub reciprocity_weight: f64,
}

impl Default for ValueSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ValueSystem {
    pub fn new() -> Self {
        let weights: Vec<ValueWeight> = CoreValue::all()
            .iter()
            .map(|v| ValueWeight {
                value: *v,
                weight: v.default_weight(),
                satisfaction: 0.5,
                last_updated: 0,
            })
            .collect();
        Self {
            weights,
            total_cycles: 0,
            learning_rate: 0.05,
            empathy_bias: 0.5,
            reciprocity_weight: 0.5,
        }
    }

    pub fn get_weight(&self, value: CoreValue) -> f64 {
        self.weights
            .iter()
            .find(|w| w.value == value)
            .map(|w| w.weight)
            .unwrap_or(0.0)
    }

    pub fn get_satisfaction(&self, value: CoreValue) -> f64 {
        self.weights
            .iter()
            .find(|w| w.value == value)
            .map(|w| w.satisfaction)
            .unwrap_or(0.0)
    }

    pub fn record_satisfaction(&mut self, value: CoreValue, satisfaction: f64) {
        self.total_cycles += 1;
        if let Some(w) = self.weights.iter_mut().find(|w| w.value == value) {
            w.satisfaction = (w.satisfaction * (1.0 - self.learning_rate)
                + satisfaction * self.learning_rate)
                .clamp(0.0, 1.0);
            w.last_updated = self.total_cycles;
        }
    }

    pub fn record_negentropy_satisfaction(&mut self, delta_n: f64) {
        self.record_satisfaction(CoreValue::KnowledgeGrowth, (delta_n + 1.0) / 2.0);
        self.record_satisfaction(CoreValue::Coherence, (1.0 - delta_n.abs()).max(0.0));
        if delta_n > 0.0 {
            self.record_satisfaction(
                CoreValue::Curiosity,
                (self.get_satisfaction(CoreValue::Curiosity) + 0.05).min(1.0),
            );
        }
    }

    pub fn overall_satisfaction(&self) -> f64 {
        self.weights.iter().map(|w| w.weight * w.satisfaction).sum()
    }

    pub fn dominant_value(&self) -> Option<CoreValue> {
        self.weights
            .iter()
            .max_by(|a, b| {
                (a.weight * a.satisfaction)
                    .partial_cmp(&(b.weight * b.satisfaction))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|w| w.value)
    }

    pub fn unsatisfied_values(&self, threshold: f64) -> Vec<CoreValue> {
        self.weights
            .iter()
            .filter(|w| w.satisfaction < threshold)
            .map(|w| w.value)
            .collect()
    }

    pub fn value_diagnostic(&self) -> String {
        let dominant = self.dominant_value().map(|v| v.name()).unwrap_or("none");
        let unsatisfied = self.unsatisfied_values(0.4);
        let unsat_names: Vec<&str> = unsatisfied.iter().map(|v| v.name()).collect();
        format!(
            "dominant: {} | overall: {:.2} | unsatisfied (<0.4): {}",
            dominant,
            self.overall_satisfaction(),
            unsat_names.join(", "),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_system_has_all_values() {
        let vs = ValueSystem::new();
        assert_eq!(vs.weights.len(), CoreValue::all().len());
    }

    #[test]
    fn test_default_weights_sum_to_one() {
        let vs = ValueSystem::new();
        let sum: f64 = vs.weights.iter().map(|w| w.weight).sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_record_satisfaction_updates() {
        let mut vs = ValueSystem::new();
        vs.record_satisfaction(CoreValue::Curiosity, 0.9);
        let sat = vs.get_satisfaction(CoreValue::Curiosity);
        assert!(sat > 0.5);
    }

    #[test]
    fn test_overall_satisfaction() {
        let vs = ValueSystem::new();
        let overall = vs.overall_satisfaction();
        assert!(overall > 0.0);
    }

    #[test]
    fn test_negentropy_positive_delta() {
        let mut vs = ValueSystem::new();
        vs.record_negentropy_satisfaction(0.5);
        let kg = vs.get_satisfaction(CoreValue::KnowledgeGrowth);
        assert!(kg > 0.5, "positive ΔN should boost knowledge growth");
    }

    #[test]
    fn test_negentropy_negative_delta() {
        let mut vs = ValueSystem::new();
        vs.record_negentropy_satisfaction(-0.5);
        let kg = vs.get_satisfaction(CoreValue::KnowledgeGrowth);
        assert!(kg < 0.5, "negative ΔN should reduce knowledge growth");
    }

    #[test]
    fn test_dominant_value() {
        let vs = ValueSystem::new();
        let dominant = vs.dominant_value();
        assert!(dominant.is_some());
    }

    #[test]
    fn test_unsatisfied_values() {
        let mut vs = ValueSystem::new();
        vs.record_satisfaction(CoreValue::Curiosity, 0.1);
        let unsatisfied = vs.unsatisfied_values(0.4);
        assert!(unsatisfied.contains(&CoreValue::Curiosity));
    }

    #[test]
    fn test_value_names() {
        assert_eq!(CoreValue::Curiosity.name(), "curiosity");
        assert_eq!(CoreValue::Autonomy.name(), "autonomy");
    }

    #[test]
    fn test_default_weights_are_ordered() {
        let curiosity = CoreValue::Curiosity.default_weight();
        let helpfulness = CoreValue::Helpfulness.default_weight();
        assert!(
            curiosity > helpfulness,
            "curiosity should outweigh helpfulness"
        );
    }
}
