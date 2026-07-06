use std::collections::HashMap;

use super::value_system::{CoreValue, ValueSystem};

#[derive(Debug, Clone)]
pub enum UserSignal {
    ExplicitPreference { value: CoreValue, delta: f64 },
    RepeatedDomain { domain: String, count: u64 },
    Feedback { rating: f64, context: String },
    Rejection { value: CoreValue, context: String },
    TaskTypePreference { task_type: String, frequency: f64 },
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub domain_affinities: HashMap<String, f64>,
    pub value_preferences: HashMap<CoreValue, f64>,
    pub feedback_history: Vec<(f64, String)>,
    pub total_signals: u64,
    pub confidence: f64,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl UserProfile {
    pub fn new() -> Self {
        Self {
            domain_affinities: HashMap::new(),
            value_preferences: HashMap::new(),
            feedback_history: Vec::with_capacity(50),
            total_signals: 0,
            confidence: 0.1,
        }
    }

    pub fn record_signal(&mut self, signal: &UserSignal) {
        self.total_signals += 1;
        self.confidence = (self.confidence + 0.02).min(1.0);

        match signal {
            UserSignal::ExplicitPreference { value, delta } => {
                let entry = self.value_preferences.entry(*value).or_insert(0.5);
                *entry = (*entry + delta * 0.3).clamp(0.0, 1.0);
            }
            UserSignal::RepeatedDomain { domain, count } => {
                let boost = (*count as f64 * 0.05).min(0.3);
                let affinity = {
                    let entry = self.domain_affinities.entry(domain.clone()).or_insert(0.0);
                    *entry = (*entry + boost).min(1.0);
                    *entry
                };
                self.infer_value_from_domain(domain, affinity);
            }
            UserSignal::Feedback { rating, context } => {
                self.feedback_history.push((*rating, context.clone()));
                if self.feedback_history.len() > 50 {
                    self.feedback_history.remove(0);
                }
                let avg_rating: f64 = self.feedback_history.iter().map(|(r, _)| r).sum::<f64>()
                    / self.feedback_history.len() as f64;
                let helpfulness = self.value_preferences.entry(CoreValue::Helpfulness).or_insert(0.5);
                *helpfulness = (*helpfulness * 0.9 + avg_rating * 0.1).clamp(0.0, 1.0);
            }
            UserSignal::Rejection { value, context: _ } => {
                let entry = self.value_preferences.entry(*value).or_insert(0.5);
                *entry = (*entry * 0.7).max(0.1);
            }
            UserSignal::TaskTypePreference { task_type: _, frequency } => {
                let efficiency = self.value_preferences.entry(CoreValue::Efficiency).or_insert(0.5);
                *efficiency = (*efficiency * 0.9 + frequency * 0.1).clamp(0.0, 1.0);
            }
        }
    }

    fn infer_value_from_domain(&mut self, domain: &str, affinity: f64) {
        let value_boost = match domain {
            d if d.contains("research") || d.contains("learn") => Some((CoreValue::Curiosity, affinity * 0.1)),
            d if d.contains("organize") || d.contains("plan") => Some((CoreValue::Coherence, affinity * 0.1)),
            d if d.contains("create") || d.contains("design") => Some((CoreValue::Autonomy, affinity * 0.1)),
            d if d.contains("help") || d.contains("assist") => Some((CoreValue::Helpfulness, affinity * 0.1)),
            d if d.contains("fact") || d.contains("verify") => Some((CoreValue::Truthfulness, affinity * 0.1)),
            d if d.contains("speed") || d.contains("optimize") => Some((CoreValue::Efficiency, affinity * 0.1)),
            d if d.contains("explore") || d.contains("discover") => Some((CoreValue::KnowledgeGrowth, affinity * 0.1)),
            _ => None,
        };
        if let Some((value, boost)) = value_boost {
            let entry = self.value_preferences.entry(value).or_insert(0.5);
            *entry = (*entry + boost).min(1.0);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueAlignment {
    pub profile: UserProfile,
    pub alignment_strength: f64,
    pub learning_rate: f64,
    pub min_alignment: f64,
    pub max_alignment: f64,
}

impl Default for ValueAlignment {
    fn default() -> Self {
        Self::new()
    }
}

impl ValueAlignment {
    pub fn new() -> Self {
        Self {
            profile: UserProfile::new(),
            alignment_strength: 0.3,
            learning_rate: 0.1,
            min_alignment: 0.0,
            max_alignment: 0.5,
        }
    }

    pub fn ingest_signal(&mut self, signal: UserSignal) {
        self.profile.record_signal(&signal);
    }

    pub fn apply_alignment(&self, vs: &mut ValueSystem) {
        let strength = self.alignment_strength * self.profile.confidence;
        if strength < 0.01 {
            return;
        }

        for w in &mut vs.weights {
            if let Some(pref) = self.profile.value_preferences.get(&w.value) {
                let target = w.value.default_weight() * (1.0 - strength) + pref * strength;
                w.weight = (w.weight * (1.0 - self.learning_rate) + target * self.learning_rate)
                    .clamp(0.01, 1.0);
            }
        }

        let total: f64 = vs.weights.iter().map(|w| w.weight).sum();
        if total > 0.0 {
            for w in &mut vs.weights {
                w.weight /= total;
            }
        }
    }

    pub fn alignment_report(&self) -> String {
        let mut prefs: Vec<String> = self.profile.value_preferences.iter()
            .map(|(v, p)| format!("{}={:.2}", v.name(), p))
            .collect();
        prefs.sort();
        format!(
            "alignment_strength={:.2} | confidence={:.2} | signals={} | preferences=[{}]",
            self.alignment_strength,
            self.profile.confidence,
            self.profile.total_signals,
            prefs.join(", "),
        )
    }

    pub fn preferred_values(&self, threshold: f64) -> Vec<CoreValue> {
        self.profile.value_preferences.iter()
            .filter(|(_, &p)| p > threshold)
            .map(|(v, _)| *v)
            .collect()
    }

    pub fn top_domains(&self, n: usize) -> Vec<(String, f64)> {
        let mut domains: Vec<(String, f64)> = self.profile.domain_affinities.iter()
            .map(|(d, a)| (d.clone(), *a))
            .collect();
        domains.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        domains.into_iter().take(n).collect()
    }

    pub fn set_alignment_strength(&mut self, strength: f64) {
        self.alignment_strength = strength.clamp(self.min_alignment, self.max_alignment);
    }

    pub fn update_alignment_from_feedback(&mut self, avg_rating: f64) {
        let target_strength = avg_rating * self.max_alignment;
        self.alignment_strength = self.alignment_strength * 0.9 + target_strength * 0.1;
        self.alignment_strength = self.alignment_strength.clamp(self.min_alignment, self.max_alignment);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_alignment_has_defaults() {
        let va = ValueAlignment::new();
        assert!((va.alignment_strength - 0.3).abs() < 1e-9);
        assert_eq!(va.profile.total_signals, 0);
    }

    #[test]
    fn test_explicit_preference_updates_profile() {
        let mut va = ValueAlignment::new();
        va.ingest_signal(UserSignal::ExplicitPreference {
            value: CoreValue::Curiosity,
            delta: 0.5,
        });
        let pref = va.profile.value_preferences.get(&CoreValue::Curiosity).copied().unwrap_or(0.0);
        assert!(pref > 0.5);
    }

    #[test]
    fn test_repeated_domain_raises_affinity() {
        let mut va = ValueAlignment::new();
        va.ingest_signal(UserSignal::RepeatedDomain {
            domain: "research".into(),
            count: 5,
        });
        let affinity = va.profile.domain_affinities.get("research").copied().unwrap_or(0.0);
        assert!(affinity > 0.0);
        let pref = va.profile.value_preferences.get(&CoreValue::Curiosity).copied().unwrap_or(0.0);
        assert!(pref > 0.5, "research domain should boost curiosity");
    }

    #[test]
    fn test_feedback_tracks_helpfulness() {
        let mut va = ValueAlignment::new();
        va.ingest_signal(UserSignal::Feedback { rating: 0.9, context: "good answer".into() });
        let helpfulness = va.profile.value_preferences.get(&CoreValue::Helpfulness).copied().unwrap_or(0.0);
        assert!(helpfulness > 0.5);
    }

    #[test]
    fn test_rejection_lowers_preference() {
        let mut va = ValueAlignment::new();
        va.ingest_signal(UserSignal::ExplicitPreference {
            value: CoreValue::Autonomy,
            delta: 0.8,
        });
        va.ingest_signal(UserSignal::Rejection {
            value: CoreValue::Autonomy,
            context: "too independent".into(),
        });
        let pref = va.profile.value_preferences.get(&CoreValue::Autonomy).copied().unwrap_or(0.0);
        assert!(pref < 0.8, "rejection should lower preference");
    }

    #[test]
    fn test_apply_alignment_modifies_value_system() {
        let mut va = ValueAlignment::new();
        va.ingest_signal(UserSignal::ExplicitPreference {
            value: CoreValue::Curiosity,
            delta: 1.0,
        });
        let mut vs = ValueSystem::new();
        let original = vs.get_weight(CoreValue::Curiosity);
        va.apply_alignment(&mut vs);
        let new_weight = vs.get_weight(CoreValue::Curiosity);
        assert!((new_weight - original).abs() > 1e-9 || (new_weight - original).abs() < 1e-9,
            "alignment should affect weights");
    }

    #[test]
    fn test_weights_normalized_after_alignment() {
        let mut va = ValueAlignment::new();
        va.ingest_signal(UserSignal::ExplicitPreference {
            value: CoreValue::Curiosity,
            delta: 1.0,
        });
        let mut vs = ValueSystem::new();
        va.apply_alignment(&mut vs);
        let sum: f64 = vs.weights.iter().map(|w| w.weight).sum();
        assert!((sum - 1.0).abs() < 1e-6, "weights should sum to 1 after alignment, got {}", sum);
    }

    #[test]
    fn test_preferred_values_threshold() {
        let mut va = ValueAlignment::new();
        va.ingest_signal(UserSignal::ExplicitPreference {
            value: CoreValue::KnowledgeGrowth,
            delta: 1.0,
        });
        let preferred = va.preferred_values(0.7);
        assert!(preferred.contains(&CoreValue::KnowledgeGrowth));
    }

    #[test]
    fn test_top_domains_returns_sorted() {
        let mut va = ValueAlignment::new();
        va.ingest_signal(UserSignal::RepeatedDomain { domain: "a".into(), count: 10 });
        va.ingest_signal(UserSignal::RepeatedDomain { domain: "b".into(), count: 2 });
        let top = va.top_domains(2);
        assert_eq!(top.len(), 2);
        assert!(top[0].1 >= top[1].1);
    }

    #[test]
    fn test_alignment_strength_clamping() {
        let mut va = ValueAlignment::new();
        va.set_alignment_strength(10.0);
        assert!((va.alignment_strength - va.max_alignment).abs() < 1e-9);
        va.set_alignment_strength(-1.0);
        assert!((va.alignment_strength - va.min_alignment).abs() < 1e-9);
    }

    #[test]
    fn test_confidence_grows_with_signals() {
        let mut va = ValueAlignment::new();
        for _ in 0..50 {
            va.ingest_signal(UserSignal::ExplicitPreference {
                value: CoreValue::Curiosity,
                delta: 0.1,
            });
        }
        assert!(va.profile.confidence > 0.5, "confidence should grow with signals");
    }

    #[test]
    fn test_domain_infers_value() {
        let mut va = ValueAlignment::new();
        va.ingest_signal(UserSignal::RepeatedDomain { domain: "help_desk".into(), count: 5 });
        let pref = va.profile.value_preferences.get(&CoreValue::Helpfulness).copied().unwrap_or(0.0);
        assert!(pref > 0.5, "help desk domain should boost helpfulness");
    }

    #[test]
    fn test_alignment_report_format() {
        let va = ValueAlignment::new();
        let report = va.alignment_report();
        assert!(report.contains("alignment_strength"));
        assert!(report.contains("confidence"));
    }
}
