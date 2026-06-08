pub mod vsa_tag;
pub mod first_person_ref;
pub mod specious_present;
pub mod stream_buffer;
pub mod awakening;
pub mod volition;
pub mod inner_critic;
pub mod cognitive_load;
pub mod resource_pool;

pub use vsa_tag::{
    VsaOrigin, VsaSelfCategory, VsaWorldCategory, VsaTagged,
};
pub use first_person_ref::FirstPersonRef;
pub use specious_present::SpeciousPresent;
pub use stream_buffer::ConsciousnessStream;
pub use awakening::{ConsciousnessAwakening, AwakeningReport};
pub use volition::{VolitionEngine, ActionCandidate};
pub use inner_critic::{InnerCritic, CritiqueResult};
pub use cognitive_load::{CognitiveLoadMonitor, ThinkingMode};
pub use resource_pool::{ResourcePool, PoolTier, Resource};
