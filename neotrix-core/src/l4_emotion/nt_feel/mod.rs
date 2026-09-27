#![forbid(unsafe_code)]

pub mod digital_human;
pub mod emotion_engine;
pub mod fep_iit_bridge;
pub mod salesperson_profiling;

/// 人类情感交互界面 — 感知→建模→共情→表达
pub mod affective_interface;

/// Emotion-cognition coupling: maps emotional states to reasoning adjustments.
pub mod cognitive_bridge;

/// 写作风格分析器 —— 本文件 1,241 行, 此前从未被 mod 声明, 从未参与编译。
/// 由 tests/test_business.rs 的 88 个测试重新接上。
pub mod writing_style;
