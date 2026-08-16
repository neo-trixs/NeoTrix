pub mod awakening;
pub mod cognitive_load;
pub mod consciousness_runtime;
pub mod first_person_ref;
pub mod inner_critic;
pub mod source_hierarchy;
pub mod specious_present;
pub mod stream_buffer;
pub mod volition;
pub mod vsa_tag;

pub use awakening::{AwakeningReport, ConsciousnessAwakening};
pub use cognitive_load::{CognitiveLoadMonitor, ThinkingMode};
pub use first_person_ref::FirstPersonRef;
pub use inner_critic::{CritiqueResult, InnerCritic};
pub use specious_present::SpeciousPresent;
pub use stream_buffer::ConsciousnessStream;
pub use volition::{ActionCandidate, VolitionEngine};
pub use vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged, VsaWorldCategory};
