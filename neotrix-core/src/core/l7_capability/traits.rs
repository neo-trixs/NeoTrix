//! 能力网核心 traits
use async_trait::async_trait;
use super::types::{Layer, CapabilityKind, Wisdom, CapabilityVector, CapabilityCost, ConsciousnessState};

/// 能力插件 trait — 定义能力的基本接口
#[async_trait]
pub trait CapabilityPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn layer(&self) -> Layer;
    fn capability_kinds(&self) -> Vec<CapabilityKind>;
    fn tags(&self) -> Vec<&'static str>;
    async fn execute(&self, input: &str) -> Result<String, String>;
    fn vector(&self) -> CapabilityVector;
    fn cost(&self) -> CapabilityCost;
}

/// 能量核心 trait — 管理智慧池与涌现
#[async_trait]
pub trait EnergyCore: Send + Sync {
    async fn receive_wisdom(&mut self, wisdom: Wisdom) -> Result<(), String>;
    async fn emit_action(&mut self) -> Result<Option<String>, String>;
    fn get_consciousness_state(&self) -> ConsciousnessState;
    fn should_emerge(&self) -> bool;
    async fn emerge(&mut self) -> Result<Option<String>, String>;
}

/// 智慧桥接 trait — 能力执行结果 → 智慧
#[async_trait]
pub trait WisdomBridge: Send + Sync {
    fn capability_to_wisdom(&self, capability_id: &str, result: &str, layer: Layer) -> Wisdom;
    async fn accumulate(&mut self, wisdom: Wisdom) -> Result<(), String>;
    fn get_wisdom(&self) -> Vec<Wisdom>;
    fn clear_wisdom(&mut self);
}
