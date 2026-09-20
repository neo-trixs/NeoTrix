//! Red team testing framework -- inspired by DeepTeam.
//! 50+ vulnerability types + 20+ attack methods for adversarial testing.

pub mod vulnerability;
pub mod attack;

pub use vulnerability::*;
pub use attack::*;
