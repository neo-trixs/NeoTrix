//! C3: Connectivity & Traversal — 连通性与穿透层
//!
//! ICE Agent, STUN/TURN Client, 路径自适应评分, Relay 降级。

pub mod hole_punch;
pub mod ice_agent;
pub mod path_score;
pub mod relay_fallback;
pub mod stun_client;
pub mod turn_client;

// SANS-IO实现
pub mod ice;
pub mod stun;
pub mod turn;
