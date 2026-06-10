pub mod vsa_tag;
pub mod source_hierarchy;
pub mod first_person_ref;
pub mod specious_present;
pub mod stream_buffer;
pub mod awakening;
pub mod volition;
pub mod inner_critic;
pub mod cognitive_load;
pub mod resource_pool;
pub mod narrative_self;
pub mod valence_axis;
pub mod conformal_uq;
pub mod confidence_calibrator;
pub mod sleep_gate;
pub mod authority;
pub mod default_mode_network;
pub mod value_system;
pub mod value_alignment;

pub use vsa_tag::{
    VsaOrigin, VsaSelfCategory, VsaWorldCategory, VsaTagged,
};
pub use authority::{
    AuthorityLevel, AuthorityTag, AuthorityResolver, Constitution,
    ConflictResolution,
};
pub use first_person_ref::FirstPersonRef;
pub use specious_present::SpeciousPresent;
pub use stream_buffer::ConsciousnessStream;
pub use awakening::{ConsciousnessAwakening, AwakeningReport};
pub use volition::{VolitionEngine, ActionCandidate};
pub use inner_critic::{InnerCritic, CritiqueResult};
pub use cognitive_load::{CognitiveLoadMonitor, ThinkingMode};
pub use resource_pool::{ResourcePool, PoolTier, Resource};
pub use confidence_calibrator::ConfidenceCalibrator;
pub use narrative_self::{NarrativeSelf, NarrativeEvent};
pub use valence_axis::{ValenceAxis, NamedEmotion};
pub use conformal_uq::{ConformalUQ, ConformalSet};
pub use sleep_gate::{SleepGate, SleepReport};
pub use default_mode_network::{DefaultModeNetwork, DMNActivity};
pub use value_system::{ValueSystem, CoreValue};
pub use value_alignment::{ValueAlignment, UserSignal, UserProfile};
