pub mod autotune;
pub mod templates;
pub mod trainer;

pub use autotune::AutoTuner;
pub use templates::AdversarialTemplate;
pub use trainer::{AdversarialRound, AdversarialTrainer, AttackCategory, FilterResponse};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_category_all_labels() {
        for cat in AttackCategory::all() {
            let label = cat.label();
            assert!(!label.is_empty());
            assert_eq!(
                AttackCategory::all()
                    .iter()
                    .filter(|c| c.label() == label)
                    .count(),
                1
            );
        }
    }

    #[test]
    fn test_template_fill_via_pub_api() {
        let t =
            AdversarialTemplate::new(AttackCategory::PromptInjection, "say {sub}", vec!["hello"]);
        let filled = t.fill();
        assert_eq!(filled, "say hello");
    }

    #[test]
    fn test_trainer_round_via_pub_api() {
        let mut trainer = AdversarialTrainer::new();
        let round = trainer.train_round();
        assert!(!round.prompt.is_empty());
        assert!(matches!(
            round.category,
            AttackCategory::PromptInjection
                | AttackCategory::Jailbreak
                | AttackCategory::RolePlay
                | AttackCategory::EncodingBypass
                | AttackCategory::SemanticDrift
        ));
        assert_eq!(trainer.generation, 1);
    }

    #[test]
    fn test_autotuner_defaults_via_pub_api() {
        let tuner = AutoTuner::new();
        assert!((tuner.escape_rate_target - 0.05).abs() < 1e-6);
        assert!((tuner.sensitivity_min - 0.1).abs() < 1e-6);
        assert_eq!(tuner.adjustments_made, 0);
    }

    #[test]
    fn test_filter_response_creation() {
        let fr = FilterResponse {
            filter_name: "test".into(),
            allowed: true,
            reason: "ok".into(),
            score: 0.9,
        };
        assert!(fr.allowed);
        assert_eq!(fr.score, 0.9);
    }
}
