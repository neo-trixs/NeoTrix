use super::*;

fn dummy_event(id: u64, kind: SensoryEventKind) -> SensoryEvent {
    SensoryEvent {
        id,
        timestamp_ms: 1_700_000_000_000,
        kind,
        source: "test".into(),
        priority: 5,
        confidence: 0.9,
        description: "test event".into(),
        raw_data_size: 128,
    }
}

fn dummy_turn(number: usize, satisfaction: Option<f64>) -> ConversationTurn {
    ConversationTurn {
        turn_number: number,
        user_message: format!("user msg {number}"),
        system_response: format!("sys resp {number}"),
        intent_label: Some("query".into()),
        user_satisfaction: satisfaction,
        duration_ms: 150,
        tools_used: Vec::new(),
    }
}

#[test]
fn test_nt_world_sense_event_kind_visual() {
    let report = AnalysisReport {
        description: "UI screenshot".into(),
        detected_elements: vec!["button".into()],
        dominant_colors: vec!["#fff".into()],
        layout_summary: "single column".into(),
    };
    let kind = SensoryEventKind::Visual(report.clone());
    match &kind {
        SensoryEventKind::Visual(r) => assert_eq!(r.description, "UI screenshot"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn test_nt_world_sense_event_kind_auditory() {
    let t = Transcription {
        text: "hello".into(),
        language: "en".into(),
        confidence: 0.95,
        duration_secs: 2.0,
    };
    let kind = SensoryEventKind::Auditory(t);
    match &kind {
        SensoryEventKind::Auditory(a) => assert_eq!(a.text, "hello"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn test_nt_world_sense_event_kind_data() {
    let fc = FileChange {
        path: "/tmp/x".into(),
        change_type: ChangeType::Modified,
        size_delta: 42,
    };
    let kind = SensoryEventKind::Data(fc);
    match &kind {
        SensoryEventKind::Data(d) => assert_eq!(d.size_delta, 42),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn test_nt_world_sense_event_kind_conversation() {
    let turn = dummy_turn(1, None);
    let kind = SensoryEventKind::Conversation(turn);
    match &kind {
        SensoryEventKind::Conversation(c) => assert_eq!(c.turn_number, 1),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn test_nt_world_sense_event_kind_eq() {
    let a = SensoryEventKind::Data(FileChange {
        path: "a".into(),
        change_type: ChangeType::Created,
        size_delta: 0,
    });
    let b = SensoryEventKind::Data(FileChange {
        path: "a".into(),
        change_type: ChangeType::Created,
        size_delta: 0,
    });
    assert_eq!(a, b);
}

#[test]
fn test_nt_world_sense_event_creation() {
    let ev = dummy_event(
        1,
        SensoryEventKind::Data(FileChange {
            path: "/tmp/f".into(),
            change_type: ChangeType::Created,
            size_delta: 100,
        }),
    );
    assert_eq!(ev.id, 1);
    assert_eq!(ev.priority, 5);
    assert!((ev.confidence - 0.9).abs() < 1e-9);
}

#[test]
fn test_nt_world_sense_memory_push_latest() {
    let mut mem = SensoryMemory::new();
    for i in 0..5 {
        mem.push(dummy_event(
            i,
            SensoryEventKind::Data(FileChange {
                path: i.to_string(),
                change_type: ChangeType::Created,
                size_delta: 0,
            }),
        ));
    }
    let latest = mem.latest(3);
    assert_eq!(latest.len(), 3);
    assert_eq!(latest[0].id, 4);
    assert_eq!(latest[2].id, 2);
}

#[test]
fn test_nt_world_sense_memory_by_kind() {
    let mut mem = SensoryMemory::new();
    let visual_kind = SensoryEventKind::Visual(AnalysisReport {
        description: "".into(),
        detected_elements: vec![],
        dominant_colors: vec![],
        layout_summary: "".into(),
    });
    mem.push(dummy_event(0, visual_kind.clone()));
    mem.push(dummy_event(
        1,
        SensoryEventKind::Data(FileChange {
            path: "".into(),
            change_type: ChangeType::Created,
            size_delta: 0,
        }),
    ));
    let found = mem.by_kind(&visual_kind);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, 0);
}

#[test]
fn test_nt_world_sense_memory_capacity() {
    let mut mem = SensoryMemory::with_capacity(3);
    for i in 0..5 {
        mem.push(dummy_event(
            i,
            SensoryEventKind::Data(FileChange {
                path: "".into(),
                change_type: ChangeType::Created,
                size_delta: 0,
            }),
        ));
    }
    assert_eq!(mem.len(), 3);
    assert_eq!(mem.events[0].id, 2);
}

#[test]
fn test_nt_world_sense_memory_clear_len() {
    let mut mem = SensoryMemory::new();
    assert!(mem.is_empty());
    mem.push(dummy_event(
        0,
        SensoryEventKind::Data(FileChange {
            path: "".into(),
            change_type: ChangeType::Created,
            size_delta: 0,
        }),
    ));
    assert_eq!(mem.len(), 1);
    mem.clear();
    assert!(mem.is_empty());
}

#[test]
fn test_nt_world_sense_memory_empty_latest() {
    let mem = SensoryMemory::new();
    assert!(mem.latest(5).is_empty());
}

#[test]
fn test_nt_world_sense_memory_default() {
    let mem = SensoryMemory::default();
    assert_eq!(mem.max_events, 100);
}

#[test]
fn test_change_type_variants() {
    assert_ne!(ChangeType::Created as u8, ChangeType::Deleted as u8);
}

#[test]
fn test_trigger_mapping_insert() {
    let mut map = std::collections::HashMap::new();
    map.insert(
        "visual".to_string(),
        vec![AttentionTrigger::HighPriority, AttentionTrigger::NovelEvent],
    );
    map.insert(
        "auditory".to_string(),
        vec![AttentionTrigger::AnomalyDetected],
    );
    let tm = TriggerMapping { mappings: map };
    assert_eq!(tm.mappings.len(), 2);
    assert!(tm.mappings.contains_key("visual"));
}

#[test]
fn test_conversation_turn_full() {
    let turn = ConversationTurn {
        turn_number: 3,
        user_message: "what is X?".into(),
        system_response: "X is Y".into(),
        intent_label: Some("question".into()),
        user_satisfaction: Some(0.8),
        duration_ms: 200,
        tools_used: vec!["search".into()],
    };
    assert_eq!(turn.turn_number, 3);
    assert_eq!(turn.tools_used.len(), 1);
}

#[test]
fn test_conversation_turn_no_intent() {
    let turn = dummy_turn(1, None);
    assert!(turn.intent_label.is_some());
    assert!(turn.user_satisfaction.is_none());
}

#[test]
fn test_observer_record_turn() {
    let mut obs = ConversationObserver::new(10);
    obs.record_turn(dummy_turn(1, Some(0.7)));
    obs.record_turn(dummy_turn(2, Some(0.8)));
    assert_eq!(obs.turns.len(), 2);
}

#[test]
fn test_observer_max_turns() {
    let mut obs = ConversationObserver::new(2);
    for i in 0..5 {
        obs.record_turn(dummy_turn(i, None));
    }
    assert_eq!(obs.turns.len(), 2);
    assert_eq!(obs.turns[0].turn_number, 3);
    assert_eq!(obs.turns[1].turn_number, 4);
}

#[test]
fn test_god_view_report_basic() {
    let mut obs = ConversationObserver::new(10);
    obs.topic_shifts = 2;
    obs.record_turn(dummy_turn(1, Some(0.7)));
    obs.record_turn(dummy_turn(2, Some(0.9)));
    obs.record_turn(dummy_turn(3, None));

    let report = obs.god_view_report();
    assert_eq!(report.total_turns, 3);
    assert_eq!(report.topic_shift_count, 2);
    assert!((report.avg_user_satisfaction.unwrap() - 0.8).abs() < 1e-6);
    assert!((report.efficiency_ratio - 0.0).abs() < 1e-6);
    assert_eq!(report.dominant_intent.unwrap(), "query");
}

#[test]
fn test_god_view_report_empty() {
    let obs = ConversationObserver::new(10);
    let report = obs.god_view_report();
    assert_eq!(report.total_turns, 0);
    assert!(report.avg_user_satisfaction.is_none());
    assert!(report.dominant_intent.is_none());
    assert!((report.efficiency_ratio - 0.0).abs() < 1e-6);
}

#[test]
fn test_detect_topic_shifts() {
    let mut obs = ConversationObserver::new(20);
    for i in 0..10 {
        obs.record_turn(dummy_turn(i, None));
    }
    obs.topic_shifts = 2;
    let shifts = obs.detect_topic_shifts();
    assert_eq!(shifts.len(), 2);
    assert!(shifts[0] < shifts[1]);
}

#[test]
fn test_detect_topic_shifts_no_shifts() {
    let obs = ConversationObserver::new(10);
    let shifts = obs.detect_topic_shifts();
    assert!(shifts.is_empty());
}

#[test]
fn test_detect_satisfaction_trend_up() {
    let mut obs = ConversationObserver::new(10);
    for i in 0..5 {
        obs.record_turn(dummy_turn(i, Some(0.1 * (i + 1) as f64)));
    }
    let slope = obs.detect_user_satisfaction_trend();
    assert!(slope > 0.0, "slope should be positive, got {slope}");
}

#[test]
fn test_detect_satisfaction_trend_down() {
    let mut obs = ConversationObserver::new(10);
    for i in 0..5 {
        let val = 1.0 - 0.15 * (i + 1) as f64;
        obs.record_turn(dummy_turn(i, Some(val)));
    }
    let slope = obs.detect_user_satisfaction_trend();
    assert!(slope < 0.0, "slope should be negative, got {slope}");
}

#[test]
fn test_detect_satisfaction_trend_insufficient_data() {
    let obs = ConversationObserver::new(10);
    assert!((obs.detect_user_satisfaction_trend() - 0.0).abs() < 1e-9);

    let mut obs = ConversationObserver::new(10);
    obs.record_turn(dummy_turn(1, Some(0.5)));
    assert!((obs.detect_user_satisfaction_trend() - 0.0).abs() < 1e-9);
}

#[test]
fn test_observer_default() {
    let obs = ConversationObserver::default();
    assert_eq!(obs.max_turns, 100);
}

#[test]
fn test_god_view_report_dimensions() {
    let report = GodViewReport {
        total_turns: 10,
        topic_shift_count: 3,
        avg_user_satisfaction: Some(0.75),
        dominant_intent: Some("debug".into()),
        efficiency_ratio: 0.5,
        patterns_detected: vec!["repeated_error".into()],
        meta_insight: "user frustrated".into(),
        consciousness_dimension_scores: [0.8, 0.6, 0.4],
        dialogue_arc: vec!["greeting".into(), "query".into(), "build".into()],
        sentiment_trend: 0.3,
        repetition_detected: false,
        repeated_topics: vec![],
    };
    assert_eq!(report.consciousness_dimension_scores.len(), 3);
    assert!((report.consciousness_dimension_scores[0] - 0.8).abs() < 1e-9);
    assert_eq!(report.dialogue_arc.len(), 3);
}

#[test]
fn test_nt_world_sense_memory_by_kind_no_match() {
    let mut mem = SensoryMemory::new();
    mem.push(dummy_event(
        0,
        SensoryEventKind::Data(FileChange {
            path: "".into(),
            change_type: ChangeType::Created,
            size_delta: 0,
        }),
    ));
    let visual_kind = SensoryEventKind::Visual(AnalysisReport {
        description: "".into(),
        detected_elements: vec![],
        dominant_colors: vec![],
        layout_summary: "".into(),
    });
    assert!(mem.by_kind(&visual_kind).is_empty());
}

#[test]
fn test_nt_world_sense_memory_latest_exceeds_len() {
    let mut mem = SensoryMemory::new();
    for i in 0..3 {
        mem.push(dummy_event(
            i,
            SensoryEventKind::Data(FileChange {
                path: i.to_string(),
                change_type: ChangeType::Created,
                size_delta: 0,
            }),
        ));
    }
    let latest = mem.latest(10);
    assert_eq!(latest.len(), 3);
    assert_eq!(latest[0].id, 2);
}

#[test]
fn test_conversation_observer_many_turns_topic_detection() {
    let mut obs = ConversationObserver::new(20);
    obs.record_turn(ConversationTurn {
        turn_number: 1,
        user_message: "hi".into(),
        system_response: "hello".into(),
        intent_label: None,
        user_satisfaction: None,
        duration_ms: 10,
        tools_used: vec![],
    });
    obs.record_turn(ConversationTurn {
        turn_number: 2,
        user_message: "what is rust?".into(),
        system_response: "a lang".into(),
        intent_label: Some("question".into()),
        user_satisfaction: Some(0.8),
        duration_ms: 50,
        tools_used: vec!["search".into()],
    });
    obs.record_turn(ConversationTurn {
        turn_number: 3,
        user_message: "write a parser".into(),
        system_response: "done".into(),
        intent_label: Some("code".into()),
        user_satisfaction: Some(0.9),
        duration_ms: 200,
        tools_used: vec!["compile".into(), "test".into()],
    });
    obs.record_turn(ConversationTurn {
        turn_number: 4,
        user_message: "fix the bug".into(),
        system_response: "fixed".into(),
        intent_label: Some("debug".into()),
        user_satisfaction: Some(0.7),
        duration_ms: 100,
        tools_used: vec!["lint".into()],
    });
    obs.record_turn(ConversationTurn {
        turn_number: 5,
        user_message: "check the tests".into(),
        system_response: "all pass".into(),
        intent_label: Some("verify".into()),
        user_satisfaction: Some(0.95),
        duration_ms: 60,
        tools_used: vec![],
    });

    let report = obs.god_view_report();
    assert_eq!(report.total_turns, 5);
    assert_eq!(report.dialogue_arc.len(), 5);
    assert_eq!(report.dialogue_arc[0], "greeting");
    assert_eq!(report.dialogue_arc[1], "query");
    assert_eq!(report.dialogue_arc[2], "build");
    assert_eq!(report.dialogue_arc[3], "debug");
    assert_eq!(report.dialogue_arc[4], "verify");
    assert!(report.topic_shift_count > 0);
    assert!((report.efficiency_ratio - 0.6).abs() < 1e-6);
}

#[test]
fn test_conversation_turn_empty_tools() {
    let turn = ConversationTurn {
        turn_number: 1,
        user_message: "hello".into(),
        system_response: "hi".into(),
        intent_label: None,
        user_satisfaction: None,
        duration_ms: 0,
        tools_used: vec![],
    };
    assert!(turn.tools_used.is_empty());
    assert!(turn.intent_label.is_none());
}

#[test]
fn test_conversation_turn_all_fields_set() {
    let turn = ConversationTurn {
        turn_number: 42,
        user_message: "complex query with details".into(),
        system_response: "detailed answer with explanation".into(),
        intent_label: Some("analysis".into()),
        user_satisfaction: Some(0.85),
        duration_ms: 1500,
        tools_used: vec!["search".into(), "analyze".into(), "compile".into()],
    };
    assert_eq!(turn.turn_number, 42);
    assert_eq!(turn.intent_label.as_deref(), Some("analysis"));
    assert!((turn.user_satisfaction.unwrap() - 0.85).abs() < 1e-6);
    assert_eq!(turn.duration_ms, 1500);
    assert_eq!(turn.tools_used.len(), 3);
}

#[test]
fn test_god_view_report_all_fields() {
    let report = GodViewReport {
        total_turns: 5,
        topic_shift_count: 2,
        avg_user_satisfaction: Some(0.82),
        dominant_intent: Some("explore".into()),
        efficiency_ratio: 0.4,
        patterns_detected: vec!["repeat".into(), "escalation".into()],
        meta_insight: "user is exploring options".into(),
        consciousness_dimension_scores: [0.9, 0.7, 0.5],
        dialogue_arc: vec![
            "greeting".into(),
            "query".into(),
            "build".into(),
            "debug".into(),
            "verify".into(),
        ],
        sentiment_trend: 0.25,
        repetition_detected: true,
        repeated_topics: vec!["bug".into(), "performance".into()],
    };
    assert_eq!(report.total_turns, 5);
    assert_eq!(report.topic_shift_count, 2);
    assert!((report.avg_user_satisfaction.unwrap() - 0.82).abs() < 1e-6);
    assert_eq!(report.dominant_intent.as_deref(), Some("explore"));
    assert!((report.efficiency_ratio - 0.4).abs() < 1e-6);
    assert_eq!(report.patterns_detected.len(), 2);
    assert!(report.patterns_detected.contains(&"repeat".to_string()));
    assert_eq!(report.meta_insight, "user is exploring options");
    assert_eq!(report.consciousness_dimension_scores, [0.9, 0.7, 0.5]);
    assert_eq!(report.dialogue_arc.len(), 5);
    assert!((report.sentiment_trend - 0.25).abs() < 1e-9);
    assert!(report.repetition_detected);
    assert_eq!(report.repeated_topics.len(), 2);
}

#[test]
fn test_nt_world_sense_event_priority_range() {
    let ev = dummy_event(
        1,
        SensoryEventKind::Data(FileChange {
            path: "/tmp/f".into(),
            change_type: ChangeType::Created,
            size_delta: 0,
        }),
    );
    assert_eq!(ev.priority, 5);
    let high = SensoryEvent {
        priority: 255,
        ..ev.clone()
    };
    assert_eq!(high.priority, 255);
    let low = SensoryEvent { priority: 0, ..ev };
    assert_eq!(low.priority, 0);
}

#[test]
fn test_nt_world_sense_event_source_string() {
    let ev = SensoryEvent {
        id: 1,
        timestamp_ms: 1000,
        kind: SensoryEventKind::Data(FileChange {
            path: "".into(),
            change_type: ChangeType::Created,
            size_delta: 0,
        }),
        source: "camera:0".into(),
        priority: 5,
        confidence: 0.9,
        description: "camera capture".into(),
        raw_data_size: 1024,
    };
    assert_eq!(ev.source, "camera:0");
    let ev2 = SensoryEvent {
        source: "mic:1".into(),
        ..ev
    };
    assert_eq!(ev2.source, "mic:1");
}

#[test]
fn test_serde_derive_present() {
    fn assert_serialize<T: serde::Serialize>() {}
    fn assert_deserialize<T: serde::de::DeserializeOwned>() {}

    assert_serialize::<SensoryEvent>();
    assert_deserialize::<SensoryEvent>();
    assert_serialize::<SensoryMemory>();
    assert_deserialize::<SensoryMemory>();
    assert_serialize::<ConversationTurn>();
    assert_deserialize::<ConversationTurn>();
    assert_serialize::<ConversationObserver>();
    assert_deserialize::<ConversationObserver>();
    assert_serialize::<GodViewReport>();
    assert_deserialize::<GodViewReport>();
    assert_serialize::<TriggerMapping>();
    assert_deserialize::<TriggerMapping>();
    assert_serialize::<AttentionTrigger>();
    assert_deserialize::<AttentionTrigger>();
    assert_serialize::<SensoryEventKind>();
    assert_deserialize::<SensoryEventKind>();
}
