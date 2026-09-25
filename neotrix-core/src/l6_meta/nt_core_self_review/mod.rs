//! Self-review gate — 27+ mechanical checks for code quality.
//! Distilled from 2026-07-05 3-deep-roam cycle: 49 weak links, 4 P0/P1 fixes,
//! Internet research (cargo-panic-audit PA001-PA009, Revet blast-radius, evole-loop audit phase).
//! Cycle 27 additions: PA010-PA015 — negative signal zeroing, FTS desync, zscore length,
//! substring tag matching, HashMap iteration order, clamp semantics.
//! Cycle 28 additions: PA016-PA019 — Python bare except, SQL injection, legacy tables, main guard.
//! Cycle 29 additions: PA020-PA023 — Karpathy-inspired code principles (Simplicity First, Surgical Changes,
//! Complexity Budget, Goal-Driven Execution). Inspired by multica-ai/andrej-karpathy-skills.
//! Cycle 30 additions: PA024-PA027 — SEAL stage health, CI workflow coverage, test assertion hygiene,
//! dependency version consistency. Distilled from Architecture Rebirth full-stack audit cycle.
//! Run via: `cargo test --lib -p neotrix -- self_review`

pub mod nt_review_report;
pub mod nt_review_runner;
pub mod nt_review_types;
pub use nt_review_runner::*;
pub use nt_review_types::*;
mod scanners;
pub use neotrix_types::shared::Severity;

#[cfg(test)]
mod tests;
