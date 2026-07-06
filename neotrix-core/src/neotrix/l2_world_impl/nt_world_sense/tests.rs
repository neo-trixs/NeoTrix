use std::path::PathBuf;
use crate::core::nt_core_sense::*;
use super::*;

#[test]
fn test_visual_cortex_scan_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("sight_sample.txt");
    std::fs::write(&file_path, b"simulated visual data").unwrap();
    let mut cortex = VisualCortex::new();
    let event = cortex.scan_from_file(&file_path);
    assert!(event.is_some());
    let event = event.unwrap();
    assert!(matches!(event.kind, SensoryEventKind::Visual(_)));
    assert!(event.description.contains("sight_sample"));
}

#[test]
fn test_visual_cortex_activate_deactivate() {
    let mut cortex = VisualCortex::new();
    assert!(!cortex.is_active());
    cortex.activate();
    assert!(cortex.is_active());
    cortex.deactivate();
    assert!(!cortex.is_active());
}

#[test]
fn test_auditory_cortex_listen_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("hearing_sample.txt");
    std::fs::write(&file_path, b"simulated audio data").unwrap();
    let mut cortex = AuditoryCortex::new();
    let event = cortex.listen_from_file(&file_path);
    assert!(event.is_some());
    let event = event.unwrap();
    assert!(matches!(event.kind, SensoryEventKind::Auditory(_)));
    assert!(event.description.contains("hearing_sample"));
}

#[test]
fn test_auditory_cortex_activate_deactivate() {
    let mut cortex = AuditoryCortex::new();
    assert!(!cortex.is_active());
    cortex.activate();
    assert!(cortex.is_active());
    cortex.deactivate();
    assert!(!cortex.is_active());
}

#[test]
fn test_hub_poll_all_returns_events() {
    let dir = tempfile::tempdir().unwrap();
    let sight_path = dir.path().join("sight.txt");
    let hearing_path = dir.path().join("hearing.txt");
    std::fs::write(&sight_path, b"visual data").unwrap();
    std::fs::write(&hearing_path, b"audio data").unwrap();
    let mut hub = SensoryIntegrationHub::new();
    hub.active = true;
    hub.visual.activate();
    hub.auditory.activate();
    hub.set_sight_path(sight_path);
    hub.set_hearing_path(hearing_path);
    let events = hub.poll_all();
    assert_eq!(events.len(), 2);
    assert_eq!(hub.memory.len(), 2);
}

#[test]
fn test_hub_inactive_returns_nothing() {
    let mut hub = SensoryIntegrationHub::new();
    hub.visual.activate();
    let events = hub.poll_all();
    assert!(events.is_empty());
}

#[test]
fn test_world_consciousness_record_turn() {
    let mut wc = WorldConsciousness::new();
    let count = wc.record_conversation_turn("Hello", "Hi there!", vec![], 100);
    assert_eq!(count, 1);
    assert_eq!(wc.conversation_observer.turns.len(), 1);
    assert_eq!(wc.conversation_observer.turns[0].user_message, "Hello");
}

#[test]
fn test_world_consciousness_god_view() {
    let mut wc = WorldConsciousness::new();
    wc.record_conversation_turn("Hello", "Hi!", vec!["search".into()], 50);
    wc.record_conversation_turn("What is Rust?", "A systems language.", vec![], 200);
    wc.record_conversation_turn("Write code", "Done.", vec!["compile".into(), "test".into()], 500);
    let report = wc.god_view();
    assert_eq!(report.total_turns, 3);
    assert!(report.efficiency_ratio > 0.0);
}

#[test]
fn test_consciousness_status_contains_three_dimensions() {
    let mut wc = WorldConsciousness::new();
    wc.record_conversation_turn("Hello", "Hi!", vec![], 50);
    let status = wc.consciousness_status();
    assert!(status.contains("Self"));
    assert!(status.contains("World"));
    assert!(status.contains("Observer"));
    assert!(status.contains("God"));
    assert!(status.contains("Dialogue"));
}

#[test]
fn test_omniscient_view_from_observer() {
    let mut wc = WorldConsciousness::new();
    wc.record_conversation_turn("Hi", "Hello!", vec![], 50);
    wc.record_conversation_turn("What can you do?", "I can code, design, and reason.", vec![], 150);
    let view = wc.omniscient_status();
    assert_eq!(view.conversation_arc.len(), 2);
    assert!(view.self_narrative.contains("2 turns"));
    assert!(view.user_model.contains("2 messages"));
}

#[test]
fn test_refresh_self_awareness() {
    let mut wc = WorldConsciousness::new();
    wc.record_conversation_turn("First", "Resp1", vec![], 50);
    assert_eq!(wc.omniscient_view.conversation_arc.len(), 1);
    wc.record_conversation_turn("Second", "Resp2", vec![], 100);
    assert_eq!(wc.omniscient_view.conversation_arc.len(), 2);
}

#[test]
fn test_dialogue_arc_detects_phases() {
    let mut wc = WorldConsciousness::new();
    wc.record_conversation_turn("Hello", "Hi!", vec![], 30);
    wc.record_conversation_turn("I found a bug", "Show me.", vec!["search".into()], 200);
    wc.record_conversation_turn("Write a test for it", "Done.", vec!["compile".into()], 300);
    let view = wc.omniscient_status();
    assert_eq!(view.dialogue_phases.len(), 3);
    assert_eq!(view.dialogue_phases[0], "greeting");
    assert_eq!(view.dialogue_phases[1], "debug");
    assert_eq!(view.dialogue_phases[2], "build");
    assert!(view.god_eye_narrative.contains("omniscient"));
    assert!(view.god_eye_narrative.contains("3 phases"));
}

#[test]
fn test_god_eye_narrative_generated() {
    let mut wc = WorldConsciousness::new();
    wc.record_conversation_turn("Hi", "Hello", vec![], 10);
    let view = wc.omniscient_status();
    assert!(!view.god_eye_narrative.is_empty());
    assert!(!view.capability_summary.is_empty());
}

#[test]
fn test_dialogue_arc_empty_no_panic() {
    let analysis = {
        let obs = ConversationObserver::new(100);
        obs.analyze_dialogue_arc()
    };
    assert!(analysis.phases.is_empty());
    assert!((analysis.sentiment_trend - 0.0).abs() < 1e-9);
}

#[test]
fn test_visual_cortex_non_existent_file() {
    let mut cortex = VisualCortex::new();
    let path = PathBuf::from("/tmp/nonexistent_sight_file_xyz_test");
    let event = cortex.scan_from_file(&path);
    assert!(event.is_none());
}

#[test]
fn test_visual_cortex_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("empty_sight.txt");
    std::fs::write(&file_path, b"").unwrap();
    let mut cortex = VisualCortex::new();
    let event = cortex.scan_from_file(&file_path);
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.raw_data_size, 0);
    assert_eq!(event.priority, 0);
}

#[test]
fn test_auditory_cortex_non_existent_file() {
    let mut cortex = AuditoryCortex::new();
    let path = PathBuf::from("/tmp/nonexistent_hearing_file_xyz_test");
    let event = cortex.listen_from_file(&path);
    assert!(event.is_none());
}

#[test]
fn test_auditory_cortex_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("empty_hearing.txt");
    std::fs::write(&file_path, b"").unwrap();
    let mut cortex = AuditoryCortex::new();
    let event = cortex.listen_from_file(&file_path);
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.raw_data_size, 0);
}

#[test]
fn test_hub_set_sight_hearing_paths() {
    let mut hub = SensoryIntegrationHub::new();
    let sight = PathBuf::from("/tmp/sight.txt");
    let hearing = PathBuf::from("/tmp/hearing.txt");
    hub.set_sight_path(sight.clone());
    hub.set_hearing_path(hearing.clone());
    assert_eq!(hub.sight_path, Some(sight));
    assert_eq!(hub.hearing_path, Some(hearing));
}

#[test]
fn test_hub_perception_narrative_when_inactive() {
    let hub = SensoryIntegrationHub::new();
    let narrative = hub.current_perception_narrative();
    assert!(!narrative.is_empty());
    assert!(narrative.contains("inactive"));
    assert!(narrative.contains("Perception"));
    assert!(narrative.contains("no data"));
}

#[test]
fn test_world_consciousness_god_view_zero_turns() {
    let wc = WorldConsciousness::new();
    let report = wc.god_view();
    assert_eq!(report.total_turns, 0);
    assert!(report.avg_user_satisfaction.is_none());
    assert!(report.dominant_intent.is_none());
    assert!((report.efficiency_ratio - 0.0).abs() < 1e-6);
    assert!(!report.repetition_detected);
    assert!(report.dialogue_arc.is_empty());
}

#[test]
fn test_consciousness_status_format_zero_turns() {
    let wc = WorldConsciousness::new();
    let status = wc.consciousness_status();
    assert!(status.contains("Self-awareness"));
    assert!(status.contains("World-awareness"));
    assert!(status.contains("Observer-awareness"));
    assert!(status.contains("awaiting first interaction"));
    assert!(status.contains("Consciousness Status"));
}

#[test]
fn test_omniscient_view_from_observer_empty() {
    let obs = ConversationObserver::new(100);
    let view = OmniscientView::new();
    let view = view.from_observer(&obs);
    assert!(view.conversation_arc.is_empty());
    assert!(view.self_narrative.contains("0 turns"));
    assert!(view.user_model.contains("No user data yet"));
    assert_eq!(view.consciousness_scores, [0.0; 3]);
}

#[test]
fn test_omniscient_view_user_model_count() {
    let mut obs = ConversationObserver::new(100);
    obs.record_turn(ConversationTurn {
        turn_number: 1, user_message: "first".into(), system_response: "resp1".into(),
        intent_label: None, user_satisfaction: None, duration_ms: 10, tools_used: vec![],
    });
    obs.record_turn(ConversationTurn {
        turn_number: 2, user_message: "second".into(), system_response: "resp2".into(),
        intent_label: None, user_satisfaction: None, duration_ms: 20, tools_used: vec!["tool".into()],
    });
    let view = OmniscientView::new();
    let view = view.from_observer(&obs);
    assert!(view.user_model.contains("2 messages"));
    assert_eq!(view.conversation_arc.len(), 2);
}

#[test]
fn test_record_conversation_turn_with_intent_label() {
    let mut wc = WorldConsciousness::new();
    wc.record_conversation_turn("Write a parser", "Done", vec!["compile".into()], 500);
    assert_eq!(wc.conversation_observer.turns.len(), 1);
    let report = wc.god_view();
    assert_eq!(report.total_turns, 1);
    assert!((report.efficiency_ratio - 1.0).abs() < 1e-6);
}

#[test]
fn test_god_view_with_repetition_detected() {
    let mut wc = WorldConsciousness::new();
    wc.record_conversation_turn("Fix the bug", "Trying", vec![], 100);
    wc.record_conversation_turn("Fix the bug", "Still working", vec![], 100);
    wc.record_conversation_turn("Fix the bug", "Not working", vec![], 100);
    let report = wc.god_view();
    assert_eq!(report.total_turns, 3);
    assert!(report.repetition_detected);
    assert!(!report.repeated_topics.is_empty());
}

#[test]
fn test_omniscient_view_multi_phase_dialogue() {
    let mut wc = WorldConsciousness::new();
    wc.record_conversation_turn("Hi", "Hello!", vec![], 10);
    wc.record_conversation_turn("What is Rust?", "A systems language.", vec![], 50);
    wc.record_conversation_turn("Write a parser", "Here's the code.", vec!["compile".into()], 200);
    wc.record_conversation_turn("Fix the error", "Fixed.", vec!["lint".into()], 100);
    wc.record_conversation_turn("Check the tests", "All pass.", vec![], 50);
    let view = wc.omniscient_status();
    assert_eq!(view.dialogue_phases.len(), 5);
    assert_eq!(view.dialogue_phases[0], "greeting");
    assert_eq!(view.dialogue_phases[1], "query");
    assert_eq!(view.dialogue_phases[2], "build");
    assert_eq!(view.dialogue_phases[3], "debug");
    assert_eq!(view.dialogue_phases[4], "verify");
    assert!(view.god_eye_narrative.contains("5 phases"));
}

#[test]
fn test_omniscient_view_repetition_fields() {
    let mut obs = ConversationObserver::new(100);
    obs.record_turn(ConversationTurn {
        turn_number: 1, user_message: "Fix the bug".into(), system_response: "Ok".into(),
        intent_label: None, user_satisfaction: Some(0.3), duration_ms: 100, tools_used: vec![],
    });
    obs.record_turn(ConversationTurn {
        turn_number: 2, user_message: "Fix the bug".into(), system_response: "Trying".into(),
        intent_label: None, user_satisfaction: Some(0.2), duration_ms: 100, tools_used: vec![],
    });
    let view = OmniscientView::new().from_observer(&obs);
    assert!(view.repetition_detected);
    assert!(!view.repeated_topics.is_empty());
    assert_eq!(view.sentiment_trend, obs.analyze_dialogue_arc().sentiment_trend);
}
