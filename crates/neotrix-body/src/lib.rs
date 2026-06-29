//! # NeoTrix — Physical Body (body/ layer)
//!
//! What I use to interact with the world.
//! Hot-pluggable IO drivers, security filter chain, agent bus.
//! Depends on neotrix-mind (and transitively neotrix-self).

pub mod io;
pub mod security;
pub mod agent;

pub use io::*;
pub use security::*;
pub use agent::*;
