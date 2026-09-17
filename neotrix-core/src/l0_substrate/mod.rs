//! L0 Substrate Layer - Foundation modules

pub mod nt_core_error;
pub mod nt_core_hot_data;
/// Shared time utilities (now_secs, now_millis)
pub mod nt_core_time;

/// Speculative Decoding Engine (GPT-5.6 pattern)
pub mod nt_core_speculative_decoding;

/// Prompt Caching System (GPT-5.6 pattern)
pub mod nt_core_prompt_cache;
