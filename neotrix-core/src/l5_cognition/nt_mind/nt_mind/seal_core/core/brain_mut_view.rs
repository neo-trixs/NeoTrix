//! Trait to break quasi-circular dependency between reasoning_engine and self_iterating.
//! reasoning_engine::ReasoningEngine holds Box<dyn BrainMutView> instead of
//! self_iterating::ReasoningBrain directly.
//! self_iterating::ReasoningBrain implements this trait.

use super::CapabilityVector;

/// Mutable view into a brain's capability vector.
/// Type-erased bridge so that ReasoningEngine does not import ReasoningBrain.
pub trait BrainMutView: Send + Sync {
    fn capability(&self) -> &CapabilityVector;
    fn capability_mut(&mut self) -> &mut CapabilityVector;
}
