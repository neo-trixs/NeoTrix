//! L6 Meta-Cognition Layer
//!
//! 4 目录架构:
//!   coordination/ — 协调类 (meta + governance)
//!   memory/       — 记忆类 (memory + nexus)
//!   healing/      — 修复类 (repair)
//!   evolution/    — 进化类 (mind + act)

pub mod traits;
pub mod coordination;
pub mod memory;
pub mod healing;
pub mod evolution;

pub use coordination as nt_meta;
pub use coordination as nt_governance;
pub use healing as nt_repair;
pub use memory as nt_nexus;
