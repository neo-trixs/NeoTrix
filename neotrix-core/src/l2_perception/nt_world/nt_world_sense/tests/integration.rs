//! PerceptionBridge 集成测试

#[cfg(test)]
mod perception_bridge_integration {
    use crate::neotrix::nt_world_sense::perception_bridge::PerceptionBridge;
    use crate::neotrix::nt_world_sense::nt_world_sense_hub::SensoryIntegrationHub;
    use crate::neotrix::nt_core_signal::core::SelectiveState;
    use crate::core::nt_core_sense::{SensoryEvent, SensoryEventKind, AnalysisReport};

    fn make_test_event(priority: u8, confidence: f64) -> SensoryEvent {
        SensoryEvent {
            id: 1,
            timestamp_ms: 1000,
            kind: SensoryEventKind::Visual(AnalysisReport {
                description: "test".into(),
                detected_elements: vec![],
                dominant_colors: vec![],
                layout_summary: "".into(),
            }),
            source: "test".into(),
            priority,
            confidence,
            description: "test event".into(),
            raw_data_size: 100,
        }
    }

    #[test]
    fn test_perception_bridge_with_sensory_hub() {
        let mut hub = SensoryIntegrationHub::new();
        
        // 设置高觉醒度状态
        let mut state = SelectiveState::new(8, 16);
        state.data = vec![0.8; 8];
        hub.set_consciousness_state(state);
        
        // 验证觉醒度
        assert!((hub.awareness_score() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_perception_bridge_attention_gating() {
        let bridge = PerceptionBridge::new();
        let mut state = SelectiveState::new(8, 16);
        
        // 高觉醒度 - 更多事件通过
        state.data = vec![0.9; 8];
        let high_priority_event = make_test_event(9, 0.95);
        assert!(bridge.gate_event(&high_priority_event, &state));
        
        // 低觉醒度 - 更少事件通过
        state.data = vec![0.1; 8];
        let low_priority_event = make_test_event(2, 0.3);
        // 低觉醒度 + 低重要性事件应被过滤
        // importance = (0.2 + 0.3) / 2 = 0.25
        // threshold = 0.3 * (1 - 0.1 * 0.5) = 0.3 * 0.95 = 0.285
        // 0.25 < 0.285 → false
        assert!(!bridge.gate_event(&low_priority_event, &state));
    }

    #[test]
    fn test_perception_bridge_batch_filtering() {
        let bridge = PerceptionBridge::new();
        let mut state = SelectiveState::new(8, 16);
        state.data = vec![0.5; 8];
        
        let events = vec![
            make_test_event(9, 0.95),  // 高重要性
            make_test_event(2, 0.3),   // 低重要性
            make_test_event(7, 0.8),   // 中等重要性
        ];
        
        let filtered = bridge.gate_events(&events, &state);
        // 至少高重要性事件应通过
        assert!(!filtered.is_empty());
    }

    #[test]
    fn test_perception_bridge_custom_threshold() {
        let mut bridge = PerceptionBridge::new();
        let mut state = SelectiveState::new(8, 16);
        state.data = vec![0.5; 8];
        
        // 设置高阈值
        bridge.set_attention_threshold(0.8);
        
        let event = make_test_event(5, 0.6);
        // importance = (0.5 + 0.6) / 2 = 0.55
        // threshold = 0.8 * (1 - 0.5 * 0.5) = 0.8 * 0.75 = 0.6
        // 0.55 < 0.6 → false
        assert!(!bridge.gate_event(&event, &state));
    }
}
