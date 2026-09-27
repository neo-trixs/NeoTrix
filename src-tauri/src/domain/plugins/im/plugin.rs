//! IM domain plugin - facade (pure move, zero behavior change).
//!
//! Extracted from single-file `plugin.rs`; types live in super::types.
//! Struct `ImPlugin` lives in `nt_connection`; theme impls live in
//! `nt_connection`/`nt_message`/`nt_group`/`nt_sync`. This file only
//! declares submodules and re-exports, preserving `plugin::ImPlugin` path.

pub mod nt_connection;
pub mod nt_group;
pub mod nt_message;
pub mod nt_sync;

pub use nt_connection::ImPlugin;
