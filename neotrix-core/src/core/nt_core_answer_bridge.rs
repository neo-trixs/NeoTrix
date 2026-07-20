use crate::core::nt_core_answer_engine::{AnswerMode, PreparedQuery};
use crate::core::nt_core_gwt::selection_strategy::{SelectionContext, SelectionResult};
use crate::core::nt_core_traits::SpecialistType;

#[derive(Debug, Clone)]
pub enum RoutingMode {
    Direct(SpecialistType),
    Broadcast(Vec<SpecialistType>),
    Cascade(SpecialistType, SpecialistType),
}

pub struct AnswerGwtBridge;

impl AnswerGwtBridge {
    pub fn route_for_mode(mode: AnswerMode) -> RoutingMode {
        match mode {
            AnswerMode::Speed => RoutingMode::Direct(SpecialistType::PatternMatcher),
            AnswerMode::Balanced => RoutingMode::Broadcast(vec![
                SpecialistType::KnowledgeRetriever,
                SpecialistType::PatternMatcher,
            ]),
            AnswerMode::Quality => RoutingMode::Cascade(
                SpecialistType::KnowledgeRetriever,
                SpecialistType::KnowledgeIntegrator,
            ),
        }
    }

    pub fn build_selection_context(query: &PreparedQuery) -> SelectionContext {
        let bias = match query.mode {
            AnswerMode::Speed => vec![0.9, 0.1, 0.0, 0.0, 0.0],
            AnswerMode::Balanced => vec![0.3, 0.4, 0.3, 0.0, 0.0],
            AnswerMode::Quality => vec![0.1, 0.3, 0.6, 0.0, 0.0],
        };
        let embedding = Some(vec![
            query.temperature,
            query.max_sources as f64 / 25.0,
            query.context_sources as f64 / 10.0,
        ]);
        SelectionContext {
            e8_attention_bias: bias,
            task_embedding: embedding,
            threshold: match query.mode {
                AnswerMode::Speed => 0.3,
                AnswerMode::Balanced => 0.5,
                AnswerMode::Quality => 0.7,
            },
        }
    }

    pub fn map_specialist_to_answer(specialist: &SpecialistType) -> AnswerMode {
        match specialist {
            SpecialistType::PatternMatcher => AnswerMode::Speed,
            SpecialistType::KnowledgeRetriever => AnswerMode::Balanced,
            SpecialistType::KnowledgeIntegrator => AnswerMode::Quality,
            _ => AnswerMode::Balanced,
        }
    }

    pub fn evaluate_selection(result: &SelectionResult) -> f64 {
        let base = result.ignition_strength.max(0.0).min(1.0);
        match result.runner_up_id {
            Some(_) => base * 0.85,
            None => base,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_routes_to_pattern_matcher() {
        match AnswerGwtBridge::route_for_mode(AnswerMode::Speed) {
            RoutingMode::Direct(s) => assert_eq!(s, SpecialistType::PatternMatcher),
            _ => panic!("expected Direct"),
        }
    }

    #[test]
    fn test_balanced_routes_to_broadcast() {
        match AnswerGwtBridge::route_for_mode(AnswerMode::Balanced) {
            RoutingMode::Broadcast(ref v) => assert_eq!(v.len(), 2),
            _ => panic!("expected Broadcast"),
        }
    }

    #[test]
    fn test_quality_routes_to_cascade() {
        match AnswerGwtBridge::route_for_mode(AnswerMode::Quality) {
            RoutingMode::Cascade(a, b) => {
                assert_eq!(a, SpecialistType::KnowledgeRetriever);
                assert_eq!(b, SpecialistType::KnowledgeIntegrator);
            }
            _ => panic!("expected Cascade"),
        }
    }

    #[test]
    fn test_selection_context_has_threshold() {
        let pq = PreparedQuery {
            query: "test".into(),
            mode: AnswerMode::Quality,
            context_sources: 5,
            widget: crate::core::nt_core_answer_engine::WidgetKind::None,
            max_sources: 25,
            temperature: 0.5,
        };
        let ctx = AnswerGwtBridge::build_selection_context(&pq);
        assert!((ctx.threshold - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_specialist_mapping_is_symmetric() {
        assert_eq!(
            AnswerGwtBridge::map_specialist_to_answer(&SpecialistType::PatternMatcher),
            AnswerMode::Speed
        );
        assert_eq!(
            AnswerGwtBridge::map_specialist_to_answer(&SpecialistType::KnowledgeRetriever),
            AnswerMode::Balanced
        );
    }

    #[test]
    fn test_evaluate_selection_with_runner_up() {
        let r = SelectionResult {
            winner_id: 0,
            ignition_strength: 0.8,
            runner_up_id: Some(1),
            strategy_used: "test",
        };
        let score = AnswerGwtBridge::evaluate_selection(&r);
        assert!((score - 0.68).abs() < 0.01);
    }

    #[test]
    fn test_evaluate_selection_no_runner_up() {
        let r = SelectionResult {
            winner_id: 0,
            ignition_strength: 0.5,
            runner_up_id: None,
            strategy_used: "test",
        };
        let score = AnswerGwtBridge::evaluate_selection(&r);
        assert!((score - 0.5).abs() < 0.01);
    }
}
