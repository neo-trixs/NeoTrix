//! # Perception Bridge
//!
// //! 连接 SensoryIntegrationHub (L2 感知层) 和 SelectiveState (L5 意识层)。
//! 使用 awareness_score() 作为注意力门控:
//! - 高觉醒度 → 处理更多感知事件
//! - 低觉醒度 → 过滤低重要性事件
//!
//! ## 性能优化
//! - 使用静态权重表避免 HashMap 分配
//! - 预计算有效阈值
//! - 内联关键路径函数

use crate::core::nt_core_sense::{SensoryEvent, SensoryEventKind};
// // use crate::l4_emotion::nt_feel::// nt_core_signal::core::SelectiveState;

/// 事件类型索引 (用于静态权重表)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
enum EventKindIndex {
    Visual = 0,
    Auditory = 1,
    Data = 2,
    Conversation = 3,
}

impl EventKindIndex {
    #[inline]
    fn from_kind(kind: &SensoryEventKind) -> Self {
        match kind {
            SensoryEventKind::Visual(_) => Self::Visual,
            SensoryEventKind::Auditory(_) => Self::Auditory,
            SensoryEventKind::Data(_) => Self::Data,
            SensoryEventKind::Conversation(_) => Self::Conversation,
        }
    }
}

/// 静态默认权重表 (避免 HashMap 分配)
const DEFAULT_WEIGHTS: [f64; 4] = [
    0.7,  // Visual
    0.6,  // Auditory
    0.5,  // Data
    0.8,  // Conversation
];

/// 感知桥接: 将 SensoryHub 的原始事件流经注意力门控后送往意识层
pub struct PerceptionBridge {
    /// 注意力阈值: awareness_score 低于此值时过滤低重要性事件
    attention_threshold: f64,
    /// 自定义权重表 (仅存储非默认值)
    custom_weights: std::collections::HashMap<EventKindIndex, f64>,
}

impl Default for PerceptionBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl PerceptionBridge {
    pub fn new() -> Self {
        Self {
            attention_threshold: 0.3,
            custom_weights: std::collections::HashMap::new(),
        }
    }

    /// 设置注意力阈值
    #[inline]
    pub fn set_attention_threshold(&mut self, threshold: f64) {
        self.attention_threshold = threshold.clamp(0.0, 1.0);
    }

    /// 为特定事件类型设置自定义权重
    pub fn set_event_importance(&mut self, kind: SensoryEventKind, weight: f64) {
        let idx = EventKindIndex::from_kind(&kind);
        self.custom_weights.insert(idx, weight.clamp(0.0, 1.0));
    }

//     /// 门控过滤: 根据 SelectiveState 的觉醒度决定是否处理事件
    #[inline]
//     pub fn gate_event(&self, event: &SensoryEvent, state: &SelectiveState) -> bool {
//         let awareness = state.awareness_score();
//         let event_importance = self.get_event_importance(event);
// 
//         // 高觉醒度 → 降低门控阈值 → 更多事件通过
//         // 低觉醒度 → 提高门控阈值 → 只有高重要性事件通过
//         let effective_threshold = self.attention_threshold * (1.0 - awareness * 0.5);
// 
//         event_importance >= effective_threshold
//     }

    /// 批量门控过滤 (预分配结果 Vec)
//     pub fn gate_events(&self, events: &[SensoryEvent], state: &SelectiveState) -> Vec<SensoryEvent> {
//         let mut result = Vec::with_capacity(events.len());
//         for event in events {
//             if self.gate_event(event, state) {
//                 result.push(event.clone());
//             }
//         }
//         result
//     }

    /// 获取事件重要性权重 (内联优化)
    #[inline]
    fn get_event_importance(&self, event: &SensoryEvent) -> f64 {
        let idx = EventKindIndex::from_kind(&event.kind);
        
        // 优先使用自定义权重
        if let Some(weight) = self.custom_weights.get(&idx) {
            return *weight;
        }
        
        // 使用静态默认权重
        DEFAULT_WEIGHTS[idx as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_sense::{AnalysisReport, SensoryEventKind};
    use std::collections::HashMap;

    fn make_visual_event(priority: u8) -> SensoryEvent {
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
            confidence: 0.8,
            description: "test event".into(),
            raw_data_size: 100,
        }
    }

    #[test]
    fn test_gate_high_awareness() {
        let bridge = PerceptionBridge::new();
        let event = make_visual_event(8);
//         let mut state = SelectiveState::new(8, 16);
        state.data = vec![0.8; 8];

        assert!(bridge.gate_event(&event, &state));
    }

    #[test]
    fn test_gate_low_awareness_low_priority() {
        let bridge = PerceptionBridge::new();
        let event = make_visual_event(2);
//         let mut state = SelectiveState::new(8, 16);
        state.data = vec![0.1; 8];

        // visual weight = 0.7, threshold = 0.3 * (1 - 0.1 * 0.5) = 0.3 * 0.95 = 0.285
        // 0.7 >= 0.285 → true
        assert!(bridge.gate_event(&event, &state));
    }

    #[test]
    fn test_gate_batch() {
        let bridge = PerceptionBridge::new();
        let events = vec![
            make_visual_event(8),
            make_visual_event(2),
        ];
//         let mut state = SelectiveState::new(8, 16);
        state.data = vec![0.5; 8];

        let passed = bridge.gate_events(&events, &state);
        assert!(!passed.is_empty());
    }

    #[test]
    fn test_custom_importance() {
        let mut bridge = PerceptionBridge::new();
        let kind = SensoryEventKind::Visual(AnalysisReport {
            description: "test".into(),
            detected_elements: vec![],
            dominant_colors: vec![],
            layout_summary: "".into(),
        });
        bridge.set_event_importance(kind, 0.95);

        let event = make_visual_event(5);
//         let mut state = SelectiveState::new(8, 16);
        state.data = vec![0.8; 8];

        // 自定义权重 0.95 > threshold 0.3 * (1 - 0.8*0.5) = 0.3 * 0.6 = 0.18
        assert!(bridge.gate_event(&event, &state));
    }

    #[test]
    fn test_performance_comparison() {
        use std::time::Instant;
        
        let bridge = PerceptionBridge::new();
//         let mut state = SelectiveState::new(8, 16);
        state.data = vec![0.5; 8];
        
        let events: Vec<_> = (0..1000)
            .map(|i| make_visual_event((i % 10) as u8))
            .collect();
        
        let start = Instant::now();
        let _result = bridge.gate_events(&events, &state);
        let duration = start.elapsed();
        
        // 1000 事件应在 1ms 内完成
        assert!(duration.as_millis() < 10, "Performance test failed: {:?}", duration);
    }
}
