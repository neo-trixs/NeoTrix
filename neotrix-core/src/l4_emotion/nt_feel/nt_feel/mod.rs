#![forbid(unsafe_code)]

pub mod emotion_engine;
pub mod nt_feel_vtuber;

pub(crate) use emotion_engine::{
    _FeelEngine, _FeelConfig, _EmotionSnapshot, _AttentionSignal, _SocialState,
};
pub(crate) use nt_feel_vtuber::{
    _VTuberEmotionEngine, _EmotionReading, _CharacterPersona,
};
