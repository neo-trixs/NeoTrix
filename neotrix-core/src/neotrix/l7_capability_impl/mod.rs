//! # L7 — Capability (能力层)
//!
//! 能力注册、调度、成熟度进化、星脉通信协议。
//! 桥接模块: 将由 core::l7_capability 的导出类型通过 neotrix:: 命名空间公开。
//!
//! NT-CORE 推理核通过 L7 路由发现和调用所有能力。
//! L7 不执行能力，只调度 —— 4 道大过滤器：权限→预算→熔断→谦逊。

pub use crate::core::l7_capability::*;

// NT-IO 吸收模块 (C1, 来自 notes/absorption-20260827-full-iterative-evolution.md 条目 16,17,19,26,27,29,30,32)
pub mod nt_io_ai_image_prompts;
pub mod nt_io_promo_bgm;
pub mod nt_io_excalidraw;
pub mod nt_io_hermes_quota;
pub mod nt_io_hermes_community;
pub mod nt_io_eli5;
pub mod nt_io_show_me;
pub mod nt_io_unslop;
// NT-IO 吸收模块 (C1, 来自 notes/absorption-20260828-batch3.md 条目 3,5,6,7)
pub mod nt_io_pi_agent_desktop;
pub mod nt_io_video_shotcraft;
pub mod nt_io_cozyclay;
pub mod nt_io_generative_media_skills;
