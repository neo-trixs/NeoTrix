//! NT-GAME Consciousness Feedback — emotion, attention, phi, VSA, and health
//! feedback from game outcomes to the consciousness core.

pub mod attention;
pub mod feedback;
pub mod health;
pub mod phi;
pub mod vsa;

pub use attention::{generate_attention_report, AttentionMapping, AttentionReport};
pub use feedback::{
    generate_feedback, AppraisalSignal, EmotionLabel, FeedbackReport, PressureSignal,
};
pub use health::{generate_health_report, HealthReport, Recommendation};
pub use phi::{generate_phi_report, PhiReport};
pub use vsa::{generate_vsa_report, GameVsaEncoder, StrategySignature, VsaReport};
