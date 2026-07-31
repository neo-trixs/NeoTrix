// Reasoning: attention routing, types, cognitive map, sleep consolidation
//
// Fused real module directory (Cycle 161b): flat nt_mind modules moved in,
// replacing the earlier empty re-export shell.
//

pub mod attention_router;
pub mod cognitive_map;
pub mod reasoning_engine;
pub mod reasoning_types;
pub mod sleep;
pub mod stagnation;
pub mod thinking_bridge;

// Cross-domain surface: re-export nt_mind scope so internal super:: refs resolve at domain level
pub use super::*;
