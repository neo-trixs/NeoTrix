#![forbid(unsafe_code)]

pub mod digital_human;
pub mod emotion_engine;
pub mod fep_iit_bridge;
pub mod nt_feel_vtuber;
pub mod salesperson_profiling;

#[allow(unused_imports)]
pub(super) use nt_feel_vtuber::{
    _VTuberEmotionEngine, _EmotionReading, _CharacterPersona,
};
