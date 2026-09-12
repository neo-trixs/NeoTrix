#![forbid(unsafe_code)]

pub mod emotion_engine;
pub mod fep_iit_bridge;
pub mod nt_feel_vtuber;

pub(super) use nt_feel_vtuber::{
    _VTuberEmotionEngine, _EmotionReading, _CharacterPersona,
};
