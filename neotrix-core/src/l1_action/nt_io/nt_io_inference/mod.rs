//! # NT-IO 推理运行时
//!
//! 本地模型推理优化能力：量化、KV 缓存、推理运行时、模型选择、Apple Silicon 优化、推测解码。
//!
//! 从 nt_shield_local_inference 迁移而来（L3 → L1），
//! 推理运行时属于 IO 层（模型输出/推理服务），不应归属安全域。

/// 量化引擎（GGUF/MLX/FP8/GPTQ/AWQ 量化策略）
pub mod quantization_engine;

/// KV 缓存优化器（PagedAttention / TurboQuant 压缩）
pub mod kv_cache_optimizer;

/// 推理运行时后端（llama.cpp / MLX / Ollama / vLLM）
pub mod inference_runtime;

/// 模型选择器（硬件匹配 + E8 推理评分）
pub mod model_selector;

/// Apple Silicon 专用优化（ANE/Metal/MPS 调度）
pub mod apple_silicon;

/// 推测解码（Speculative Decoding / Draft Model）
pub mod speculative_decoding;
