//! L0 Substrate Layer - Foundation modules

pub mod nt_core_error;
pub mod nt_core_hot_data;
/// Shared time utilities (now_secs, now_millis)
pub mod nt_core_time;

/// Speculative Decoding Engine (GPT-5.6 pattern)
pub mod nt_core_speculative_decoding;

/// Prompt Caching System (GPT-5.6 pattern)
pub mod nt_core_prompt_cache;

/// SelfTest trait — 跨模块共享的自测试接口定义 (下沉自 L6)
pub mod nt_core_self_test;

/// 统一能力接口类型定义 (下沉自 L6 nt_core_capability)
pub mod nt_core_capability_types;

/// Math utilities (cosine similarity, URL normalization, Hamming distance)
pub mod nt_core_math;

/// E₈ × 64 state-space reasoning model (hexagram→reasoning mode mapping)
pub mod nt_core_hex;

/// Shared types (Modality, E8VsaEmbedding) — broken from circular deps
pub mod nt_core_shared_types;
