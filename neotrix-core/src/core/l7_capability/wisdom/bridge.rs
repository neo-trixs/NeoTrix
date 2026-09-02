//! 智慧桥接实现
//! 
//! 将能力执行结果转化为智慧

use crate::core::l7_capability::types::{Layer, Wisdom};
use crate::core::l7_capability::traits::WisdomBridge;
use async_trait::async_trait;

/// 智慧桥接实现
pub struct WisdomBridgeImpl {
    /// 智慧累积器
    accumulator: super::accumulator::WisdomAccumulator,
}

impl WisdomBridgeImpl {
    /// 创建新的智慧桥接
    pub fn new() -> Self {
        Self {
            accumulator: super::accumulator::WisdomAccumulator::new(),
        }
    }
}

impl Default for WisdomBridgeImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WisdomBridge for WisdomBridgeImpl {
    fn capability_to_wisdom(&self, capability_id: &str, result: &str, layer: Layer) -> Wisdom {
        match layer {
            Layer::L1Action => Wisdom::Action {
                capability_id: capability_id.to_string(),
                insight: result.to_string(),
                strength: 0.5,
            },
            Layer::L2Perception => Wisdom::Perception {
                capability_id: capability_id.to_string(),
                pattern: result.to_string(),
                confidence: 0.5,
            },
            Layer::L3Embodiment => Wisdom::Action {
                capability_id: capability_id.to_string(),
                insight: result.to_string(),
                strength: 0.5,
            },
            Layer::L4Emotion => Wisdom::Emotion {
                capability_id: capability_id.to_string(),
                emotion: result.to_string(),
                intensity: 0.5,
            },
            Layer::L5Cognition => Wisdom::Cognition {
                capability_id: capability_id.to_string(),
                reasoning: result.to_string(),
                depth: 1,
            },
            Layer::L6MetaCognition => Wisdom::MetaCognition {
                capability_id: capability_id.to_string(),
                reflection: result.to_string(),
                insight: result.to_string(),
            },
        }
    }
    
    async fn accumulate(&mut self, wisdom: Wisdom) -> Result<(), String> {
        self.accumulator.accumulate(wisdom).await
    }
    
    fn get_wisdom(&self) -> Vec<Wisdom> {
        self.accumulator.get_wisdom()
    }
    
    fn clear_wisdom(&mut self) {
        self.accumulator.clear();
    }
}
