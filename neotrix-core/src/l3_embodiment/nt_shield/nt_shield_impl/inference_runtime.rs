//! # Inference Runtime
//!
//! Absorbs inference engine technologies:
//! - **llama.cpp**: C/C++ inference, GGUF native, cross-platform, zero dependencies
//! - **MLX**: Apple Silicon native, fused kernels, lazy evaluation, 30-50% faster than llama.cpp
//! - **Ollama**: One-command local inference, MLX backend (0.19+), REST API
//! - **vLLM**: Production inference server, PagedAttention, continuous batching, 23x throughput
//! - **SGLang**: RadixAttention, 29% faster than vLLM on H100, prefix sharing
//! - **TensorRT-LLM**: NVIDIA's deep learning compiler, maximum GPU utilization
//! - **LMDeploy**: TurboMind + PyTorch dual backends
//! - **TGI**: HuggingFace production inference
//!
//! Benchmarks (Apple Silicon M4 Max 128GB, Qwen3-0.6B Q4):
//! - MLX: 525.5 tok/s
//! - llama.cpp: 281.5 tok/s (+87% MLX advantage)
//! - M4 Pro 48GB Qwen3.5-35B-A3B: 70-80 tok/s (MLX)
//!
//! Benchmarks (H100, production):
//! - vLLM: 180-220 tok/s for gpt-oss-120B (batch=32)
//! - SGLang: 29% faster than vLLM
//! - TensorRT-LLM: Maximum throughput for fixed models

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Inference runtime configuration
pub struct InferenceRuntime {
    /// Backend engine type
    pub backend: BackendEngine,
    /// Runtime configuration parameters
    pub config: RuntimeConfig,
    /// Active session handles
    pub sessions: HashMap<String, SessionHandle>,
}

/// Backend engine type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendEngine {
    /// llama.cpp (C/C++, cross-platform)
    LlamaCpp,
    /// MLX (Apple Silicon native)
    MLX,
    /// Ollama (one-command, wraps llama.cpp/MLX)
    Ollama,
    /// vLLM (production GPU serving)
    VLLM,
    /// SGLang (production GPU serving with RadixAttention)
    SGLang,
    /// TensorRT-LLM (NVIDIA optimized)
    TensorRTLLM,
    /// LMDeploy (TurboMind + PyTorch)
    LMDeploy,
    /// HuggingFace TGI
    TGI,
}

/// Runtime configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub model_path: String,
    pub quantization: String,
    pub context_length: usize,
    pub num_threads: usize,
    pub gpu_layers: usize,
    pub tensor_parallel_size: usize,
    pub flash_attention: bool,
    pub continuous_batching: bool,
    pub speculative_decoding: bool,
    pub max_num_seqs: usize,
    pub batch_size: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub repetition_penalty: f32,
    pub seed: u64,
}

/// Session handle for active inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHandle {
    pub session_id: String,
    pub model: String,
    pub backend: BackendEngine,
    pub status: SessionStatus,
    pub tokens_generated: u64,
    pub tokens_per_second: f64,
    pub memory_used_gb: f64,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Idle,
    Running,
    Paused,
    Error(String),
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub tokens_per_second: f64,
    pub time_to_first_token_ms: u64,
    pub tokens_per_output_token_ms: f64,
    pub memory_used_gb: f64,
    pub gpu_utilization_pct: f64,
    pub batch_size: usize,
    pub queue_depth: usize,
}

/// Throughput benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputBenchmark {
    pub model: String,
    pub backend: BackendEngine,
    pub quantization: String,
    pub hardware: String,
    pub tokens_per_second: f64,
    pub ttft_ms: u64,
    pub tpot_ms: f64,
    pub memory_gb: f64,
}

impl InferenceRuntime {
    /// Create default runtime with llama.cpp backend
    pub fn new() -> Self {
        Self {
            backend: BackendEngine::LlamaCpp,
            config: RuntimeConfig {
                model_path: String::new(),
                quantization: "Q4_K_M".to_string(),
                context_length: 4096,
                num_threads: 8,
                gpu_layers: 99,
                tensor_parallel_size: 1,
                flash_attention: true,
                continuous_batching: false,
                speculative_decoding: false,
                max_num_seqs: 256,
                batch_size: 1,
                temperature: 0.8,
                top_p: 0.95,
                top_k: 40,
                repetition_penalty: 1.1,
                seed: 42,
            },
            sessions: HashMap::new(),
        }
    }
    
    /// Initialize with MLX backend (Apple Silicon)
    pub fn with_mlx() -> Self {
        let mut runtime = Self::new();
        runtime.backend = BackendEngine::MLX;
        runtime.config.flash_attention = true;
        runtime
    }
    
    /// Initialize with Ollama backend
    pub fn with_ollama() -> Self {
        let mut runtime = Self::new();
        runtime.backend = BackendEngine::Ollama;
        runtime.config.flash_attention = true;
        runtime.config.continuous_batching = true;
        runtime
    }
    
    /// Initialize with vLLM backend (production GPU)
    pub fn with_vllm() -> Self {
        let mut runtime = Self::new();
        runtime.backend = BackendEngine::VLLM;
        runtime.config.flash_attention = true;
        runtime.config.continuous_batching = true;
        runtime.config.tensor_parallel_size = 4;
        runtime
    }
    
    /// Initialize with SGLang backend
    pub fn with_sglang() -> Self {
        let mut runtime = Self::new();
        runtime.backend = BackendEngine::SGLang;
        runtime.config.flash_attention = true;
        runtime.config.continuous_batching = true;
        runtime.config.max_num_seqs = 256;
        runtime
    }
    
    /// Start inference session
    pub async fn start_session(&mut self, session_id: &str) -> Result<SessionHandle, String> {
        let handle = SessionHandle {
            session_id: session_id.to_string(),
            model: self.config.model_path.clone(),
            backend: self.backend.clone(),
            status: SessionStatus::Idle,
            tokens_generated: 0,
            tokens_per_second: 0.0,
            memory_used_gb: 0.0,
        };
        
        self.sessions.insert(session_id.to_string(), handle.clone());
        Ok(handle)
    }
    
    /// Get benchmark data for hardware/model combos
    pub fn get_benchmarks(&self) -> Vec<ThroughputBenchmark> {
        vec![
            // Apple Silicon benchmarks
            ThroughputBenchmark {
                model: "Qwen3-0.6B".to_string(),
                backend: BackendEngine::MLX,
                quantization: "Q4".to_string(),
                hardware: "M4 Max 128GB".to_string(),
                tokens_per_second: 525.5,
                ttft_ms: 5,
                tpot_ms: 2.0,
                memory_gb: 2.0,
            },
            ThroughputBenchmark {
                model: "Qwen3-0.6B".to_string(),
                backend: BackendEngine::LlamaCpp,
                quantization: "Q4".to_string(),
                hardware: "M4 Max 128GB".to_string(),
                tokens_per_second: 281.5,
                ttft_ms: 8,
                tpot_ms: 3.5,
                memory_gb: 2.0,
            },
            ThroughputBenchmark {
                model: "Qwen3-8B".to_string(),
                backend: BackendEngine::MLX,
                quantization: "Q4".to_string(),
                hardware: "M4 Max 128GB".to_string(),
                tokens_per_second: 93.3,
                ttft_ms: 10,
                tpot_ms: 10.7,
                memory_gb: 5.0,
            },
            ThroughputBenchmark {
                model: "Qwen3-8B".to_string(),
                backend: BackendEngine::LlamaCpp,
                quantization: "Q4".to_string(),
                hardware: "M4 Max 128GB".to_string(),
                tokens_per_second: 76.9,
                ttft_ms: 12,
                tpot_ms: 13.0,
                memory_gb: 5.0,
            },
            // Production GPU benchmarks
            ThroughputBenchmark {
                model: "gpt-oss-120B".to_string(),
                backend: BackendEngine::VLLM,
                quantization: "FP8".to_string(),
                hardware: "H100 80GB".to_string(),
                tokens_per_second: 200.0,
                ttft_ms: 50,
                tpot_ms: 5.0,
                memory_gb: 80.0,
            },
        ]
    }
}
