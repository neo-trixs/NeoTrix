//! 能力网核心 traits
use async_trait::async_trait;
use super::types::{Layer, Wisdom};

/// 智慧桥接 trait — 能力执行结果 → 智慧
#[async_trait]
pub trait WisdomBridge: Send + Sync {
    fn capability_to_wisdom(&self, capability_id: &str, result: &str, layer: Layer) -> Wisdom;
}
