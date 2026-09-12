//! NT-GAME Consciousness Feedback — emotion, attention, phi, VSA, and health
//! feedback from game outcomes to the consciousness core.

pub mod attention;
pub mod feedback;
pub mod health;
pub mod phi;
pub mod vsa;

pub use attention::{AttentionReport, AttentionMapping, generate_attention_report};
pub use feedback::{FeedbackReport, AppraisalSignal, PressureSignal, EmotionLabel, generate_feedback};
pub use health::{HealthReport, Recommendation, generate_health_report};
pub use phi::{PhiReport, generate_phi_report};
pub use vsa::{VsaReport, GameVsaEncoder, StrategySignature, generate_vsa_report};
