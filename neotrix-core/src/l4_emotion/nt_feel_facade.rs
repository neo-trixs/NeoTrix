//! L4 NT-FEEL Facade — re-exports all public types from nt_feel submodules
//!
//! Single fact source lives in nt_feel submodules; this facade centralises
//! cross-layer imports so consumers never scatter `use crate::l4_emotion::nt_feel::*`.

pub use super::nt_feel::nt_feel::emotion_engine::{
    AttentionSignal, EmotionSnapshot, FeelConfig, FeelEngine, SocialState,
};
pub use super::nt_feel::nt_feel::nt_feel_vtuber::{
    CharacterPersona, EmotionReading, EmotionRegulation, EmotionResponse, EmotionSource,
    EmotionType, PersonalityTrait, RegulationStrategy, ResponseStyle, SpeakingPattern,
    VTuberEmotionEngine, VoiceConfig, VoiceOutput,
};
