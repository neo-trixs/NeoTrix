//! IO Skills Facade — L5 对 L1 NT-IO 技能模块的 re-export 门面
//!
//! L5 认知层通过此模块访问 IO 技能模块，避免散布 `use crate::l1_action::nt_io::*`。

pub use crate::l1_action::nt_io::l3_vendor_skills::nt_io_excalidraw::*;
pub use crate::l1_action::nt_io::l3_vendor_skills::nt_io_hermes_community::*;
