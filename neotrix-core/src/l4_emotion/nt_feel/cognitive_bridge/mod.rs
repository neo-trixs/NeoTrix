pub mod coupling;
pub mod emotion_state;
pub mod feedback;
pub mod influence;

pub use coupling::{AdjustmentParam, CognitiveCoupling, ReasoningAdjustment};
pub use emotion_state::{EmotionState, EmotionVariant};
pub use feedback::{FeedbackLoop, Outcome};
pub use influence::{InfluenceEngine, WeightedOption};
