use super::IngestionSourceType;
use crate::core::{ReasoningHexagram, MODE_NAMES};
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

pub fn source_type_to_e8_mode(source: IngestionSourceType) -> Option<String> {
    let idx = match source {
        IngestionSourceType::Book => 48, // Pattern Match — textual pattern discovery
        IngestionSourceType::Paper => 16, // Formal Proof — rigorous claim analysis
        IngestionSourceType::Code => 4,  // Code Review — systematic inspection
        IngestionSourceType::Web => 22,  // Exploration — open-ended browsing
        IngestionSourceType::Conversation => 5, // Pair Review — dialogue-mode reasoning
        IngestionSourceType::Finance => 24, // Data Analysis — quantitative reasoning
        IngestionSourceType::Media => 14, // Brainstorm — creative ideation
        IngestionSourceType::Social => 48, // Default to Pattern Match
    };
    Some(MODE_NAMES[idx].to_string())
}

pub fn apply_source_e8_routing(brain: &mut SelfIteratingBrain, source_type: IngestionSourceType) {
    let idx = match source_type {
        IngestionSourceType::Book => 48,
        IngestionSourceType::Paper => 16,
        IngestionSourceType::Code => 4,
        IngestionSourceType::Web => 22,
        IngestionSourceType::Conversation => 5,
        IngestionSourceType::Finance => 24,
        IngestionSourceType::Media => 14,
        IngestionSourceType::Social => 48,
    };
    let mode = ReasoningHexagram(idx);
    brain._e8_policy.set_previous(mode);
    log::info!(
        "[e8-routing] source={:?} → mode={} ({})",
        source_type,
        idx,
        MODE_NAMES[idx as usize],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_to_e8_mode_returns_all() {
        for st in IngestionSourceType::all() {
            let name = source_type_to_e8_mode(st);
            assert!(
                name.is_some(),
                "every source type should map to a mode name"
            );
            assert!(!name.unwrap().is_empty(), "mode name should not be empty");
        }
    }

    #[test]
    fn test_apply_source_e8_routing_sets_previous() {
        let mut brain = SelfIteratingBrain::new();
        assert!(brain._e8_policy.previous_mode().is_none());
        apply_source_e8_routing(&mut brain, IngestionSourceType::Paper);
        assert!(brain._e8_policy.previous_mode().is_some());
        assert_eq!(brain._e8_policy.previous_mode().unwrap().0, 16);
    }

    #[test]
    fn test_distinct_modes_per_source() {
        let mut modes = std::collections::HashSet::new();
        for st in IngestionSourceType::all() {
            let idx = match st {
                IngestionSourceType::Book => 48,
                IngestionSourceType::Paper => 16,
                IngestionSourceType::Code => 4,
                IngestionSourceType::Web => 22,
                IngestionSourceType::Conversation => 5,
                IngestionSourceType::Finance => 24,
                IngestionSourceType::Media => 14,
                IngestionSourceType::Social => 48,
            };
            assert!(modes.insert(idx), "duplicate mode {} for {:?}", idx, st);
        }
        assert_eq!(
            modes.len(),
            7,
            "all 7 source types must map to distinct modes"
        );
    }
}
