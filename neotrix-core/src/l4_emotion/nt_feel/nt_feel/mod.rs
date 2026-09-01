#![forbid(unsafe_code)]

pub mod emotion_engine;
pub mod nt_feel_vtuber;

pub use emotion_engine::{FeelEngine, FeelConfig, EmotionSnapshot, AttentionSignal, SocialState};
pub use nt_feel_vtuber::{VTuberEmotionEngine, EmotionReading, CharacterPersona};
