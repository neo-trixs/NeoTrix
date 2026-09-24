#![allow(unused_imports)]
//! L4 NT-FEEL Facade — re-exports all public types from nt_feel submodules
//!
//! Single fact source lives in nt_feel submodules; this facade centralises
//! cross-layer imports so consumers never scatter `use crate::l4_emotion::nt_feel::*`.

pub use crate::l4_emotion::nt_feel::digital_human::{
    Emotion as DigitalHumanEmotion, EmotionEngine as DigitalHumanEmotionEngine,
    emotion_from_expression,
};
pub use crate::l4_emotion::nt_feel::emotion_engine::{
    Emotion, EmotionalState, EmotionEngine, RegulationStrategy, EmotionalIntelligence,
};
