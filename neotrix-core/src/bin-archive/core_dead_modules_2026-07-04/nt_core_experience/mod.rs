//! # Phase 5 — Self-Evolving Experience Loop
//!
//! 闭合 Loop Engineering 的最后闭环:
//! Context OS → Decision Compression → **Experience Reflection → Skill Accumulation
//! → Curriculum Generation → Policy Repair → Epistemic Self-Knowledge**
//!
//! ⚠️ 此模块尚在构建中。以下文件已实现，剩余子模块待创建。

pub mod hypothesis_tree;
pub use hypothesis_tree::{HypothesisNode, HypothesisStatus, HypothesisTree, HypothesisTreeConfig, HypothesisTreeStats};

pub mod safety_gate;
pub use safety_gate::{SafetyGate, SafetyReport, CheckResult};

pub mod self_introspection;
pub use self_introspection::{IntrospectionEngine, DiagnosticSnapshot, CorrectiveAction, DefectPattern};
