//! # L7 — Capability (能力层)
//!
//! 能力注册、调度、成熟度进化、星脉通信协议。
//! 桥接模块: 将由 core::l7_capability 的导出类型通过 neotrix:: 命名空间公开。
//!
//! NT-CORE 推理核通过 L7 路由发现和调用所有能力。
//! L7 不执行能力，只调度 —— 4 道大过滤器：权限→预算→熔断→谦逊。

pub use crate::core::l7_capability::*;

// L5 Cognition 核心增强模块
pub mod nt_core_kg_traversal;
pub mod nt_core_rag;
pub mod nt_core_model_router;
pub mod nt_core_prompt_template;
pub mod nt_core_knowledge_mgmt;
pub mod nt_core_xai;
pub mod nt_core_planning;
pub mod nt_core_autonomous_ai;
pub mod nt_core_safety_alignment;
pub mod nt_core_knowledge_repr;
pub mod nt_core_fsm_topology;
pub mod nt_core_cuda_rl;
pub mod nt_core_parallel;

// 意识核心模块 (Consciousness Core)
pub mod nt_core_resonance_complexity;
pub mod nt_core_field_resonance;
pub mod nt_core_golden_ratio;
pub mod nt_core_integrated_information;
pub mod nt_core_vibrational_resonance;

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

// 节奏重算模块 (动态漫爽点-节奏设计吸收)
pub mod seal;

// Parallel task execution
// pub mod nt_core_parallel; // Removed duplicate
