#![forbid(unsafe_code)]

pub mod digital_human;
pub mod emotion_engine;
pub mod fep_iit_bridge;
pub mod nt_feel_vtuber;
pub mod salesperson_profiling;

/// 人类情感交互界面 — 感知→建模→共情→表达
pub mod affective_interface;

/// Emotion-cognition coupling: maps emotional states to reasoning adjustments.
pub mod cognitive_bridge;

#[allow(unused_imports)]
pub(super) use nt_feel_vtuber::{
    _VTuberEmotionEngine, _EmotionReading, _CharacterPersona,
};
