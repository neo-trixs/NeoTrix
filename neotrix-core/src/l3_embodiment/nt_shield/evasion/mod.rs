//! NT-SHIELD 逃逸机制子模块
//!
//! 主动逃逸与对抗能力:
//! - FullbreakEngine: 全破多攻击面叠层
//! - CloudEvadeEngine: 云端混淆逃逸
//! - GrappleHookChain: 抓钩链逃逸

pub mod fullbreak;
pub mod cloud_evade;
pub mod grapple_hooks;

pub use fullbreak::{AttackResult, AttackSurface, FullbreakEngine};
pub use cloud_evade::{CloudEvadeEngine, EvasionResult, EvasionTechnique, ObfuscationType};
pub use grapple_hooks::GrappleHookChain;
