#![forbid(unsafe_code)]

pub mod emotion_engine;

pub use emotion_engine::{FeelEngine, FeelConfig, EmotionSnapshot, AttentionSignal, SocialState};
