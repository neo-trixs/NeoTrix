//! Layer facade aliases — consolidates stub facades
//!
//! All layer-specific facades (act, io, kb, l2, l3, io_skills) were just
//! re-exporting l1_facade. This module provides a single source.

pub use super::l1_facade::*;
