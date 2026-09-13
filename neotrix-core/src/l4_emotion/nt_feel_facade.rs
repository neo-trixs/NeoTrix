#![allow(unused_imports)]
//! L4 NT-FEEL Facade — re-exports all public types from nt_feel submodules
//!
//! Single fact source lives in nt_feel submodules; this facade centralises
//! cross-layer imports so consumers never scatter `use crate::l4_emotion::nt_feel::*`.

pub(crate) use crate::l4_emotion::nt_feel::emotion_engine::{
    Emotion, EmotionalState, EmotionEngine, RegulationStrategy, EmotionalIntelligence,
};
pub(crate) use crate::l4_emotion::nt_feel::nt_feel_vtuber::{
    _VTuberEmotionEngine, _CharacterPersona, _PersonalityTrait, _ResponseStyle, _SpeakingPattern,
};
