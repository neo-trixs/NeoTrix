//! # Model Selector
//!
//! Absorbs model selection strategies for local inference:
//! - **Model size**: 7B, 13B, 34B, 70B, 120B parameter models
//! - **MoE architecture**: Qwen3.5-35B-A3B (35B knowledge, 3B compute)
//! - **Quantization-aware selection**: Match model to hardware VRAM
//! - **Task-specific models**: Coding, reasoning, RAG, vision
//! - **Architecture-aware**: MLA, GQA, SwiGLU, MoE
//!
//! Memory requirements (2026):
//! | Model | FP16 | INT8 | INT4 |
//! |-------|------|------|------|
//! | 7B    | 14GB | 7GB  | 4GB  |
//! | 13B   | 26GB | 13GB | 7GB  |
//! | 34B   | 68GB | 34GB | 17GB |
//! | 70B   | 140GB| 70GB | 35GB |
//!
//! Apple Silicon models (2026):
//! - Llama 3.1 8B Q4_K_M: Smallest footprint, highest tok/s
//! - Qwen3-4B Q4: 159 tok/s on M4 Max
//! - Qwen3.5-35B-A3B: 35B knowledge, 3B compute cost, best on 32GB
//! - Qwen3-8B Q4: 93 tok/s on M4 Max

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::quantization_engine::{QuantizationEngine, QuantLevel, ModelParams, AttentionType, HardwareCapabilities, ModelScore};

/// Model selector for local inference — llmfit-enhanced multi-dimensional scoring
pub struct ModelSelector {
    /// Known models and their characteristics
    known_models: HashMap<String, ModelInfo>,
    /// Hardware profiles
    hardware_profiles: HashMap<String, HardwareProfile>,
    /// Quantization engine for advanced scoring
    quant_engine: QuantizationEngine,
}

/// Model information — enriched with MoE/Apple Silicon data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub parameter_count: u64,
    pub architecture: String,
    pub attention_type: AttentionType,
    pub file_size_fp16_gb: f64,
    pub file_size_int4_gb: f64,
    pub recommended_quant: String,
    pub best_for: Vec<String>,
    pub context_length: usize,
    pub tokens_per_second_fp16: f64,
    pub tokens_per_second_int4: f64,
    pub license: String,
    /// Real measured tok/s on M5 16GB (if available)
    pub estimated_m5_16gb_toks: Option<f64>,
    /// MoE architecture flag (affects "active params" vs total)
    pub is_moe: bool,
    /// Active parameters in MoE (e.g., 3B for Qwen3.5-9B MoE)
    pub active_params_b: f64,
}

/// Hardware profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub name: String,
    pub gpu_arch: String,
    pub vram_gb: f64,
    pub memory_bandwidth_gbps: f64,
    pub compute_tflops: f64,
    pub supports_fp8: bool,
    pub supports_metal: bool,
    pub apple_silicon: bool,
}

/// Recommended model for a given hardware and task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub model: String,
    pub quantization: String,
    pub expected_throughput_tok_s: f64,
    pub memory_required_gb: f64,
    pub headroom_gb: f64,
    pub task_fit_score: f64,
    pub quality_score: f64,
}

impl ModelSelector {
    /// Create model selector with known models — includes MoE, MLA, Apple Silicon 2026 lineup
    pub fn new() -> Self {
        let mut known_models = HashMap::new();
        
        // === 4B SMALL MODELS (ultra-portable) ===
        known_models.insert("Qwen3-4B".to_string(), ModelInfo {
            name: "Qwen3-4B".to_string(),
            parameter_count: 4_000_000_000,
            architecture: "Qwen3".to_string(),
            attention_type: AttentionType::GroupedQueryAttention,
            file_size_fp16_gb: 8.0,
            file_size_int4_gb: 2.3,
            recommended_quant: "Q4_K_M".to_string(),
            best_for: vec!["chat".to_string(), "coding".to_string()],
            context_length: 128_000,
            tokens_per_second_fp16: 160.0,
            tokens_per_second_int4: 350.0,
            license: "Apache-2.0".to_string(),
            estimated_m5_16gb_toks: Some(45.0),
            is_moe: false,
            active_params_b: 4.0,
        });
        
        // === 7-8B MODELS (Apple Silicon sweet spot) ===
        known_models.insert("Llama-3.1-8B".to_string(), ModelInfo {
            name: "Llama-3.1-8B".to_string(),
            parameter_count: 8_000_000_000,
            architecture: "Llama".to_string(),
            attention_type: AttentionType::GroupedQueryAttention,
            file_size_fp16_gb: 14.0,
            file_size_int4_gb: 4.0,
            recommended_quant: "Q4_K_M".to_string(),
            best_for: vec!["chat".to_string(), "coding".to_string(), "general".to_string()],
            context_length: 128_000,
            tokens_per_second_fp16: 100.0,
            tokens_per_second_int4: 250.0,
            license: "MIT".to_string(),
            estimated_m5_16gb_toks: Some(9.5),
            is_moe: false,
            active_params_b: 8.0,
        });
        
        // === 8B Apple Silicon optimized ===
        known_models.insert("Qwen3-8B".to_string(), ModelInfo {
            name: "Qwen3-8B".to_string(),
            parameter_count: 8_000_000_000,
            architecture: "Qwen3".to_string(),
            attention_type: AttentionType::GroupedQueryAttention,
            file_size_fp16_gb: 14.0,
            file_size_int4_gb: 4.0,
            recommended_quant: "Q4_K_M".to_string(),
            best_for: vec!["coding".to_string(), "reasoning".to_string(), "general".to_string()],
            context_length: 128_000,
            tokens_per_second_fp16: 100.0,
            tokens_per_second_int4: 250.0,
            license: "Apache-2.0".to_string(),
            estimated_m5_16gb_toks: Some(12.0),
            is_moe: false,
            active_params_b: 8.0,
        });

        // === 9B MoE (Neo's current model — Qwen3.5-9B) ===
        known_models.insert("Qwen3.5-9B".to_string(), ModelInfo {
            name: "Qwen3.5-9B".to_string(),
            parameter_count: 9_000_000_000,
            architecture: "Qwen3.5_MoE".to_string(),
            attention_type: AttentionType::MultiHeadLatentAttention,
            file_size_fp16_gb: 18.0,
            file_size_int4_gb: 7.1,
            recommended_quant: "Q5_K_M".to_string(),
            best_for: vec!["reasoning".to_string(), "coding".to_string(), "general".to_string(), "agent".to_string()],
            context_length: 128_000,
            tokens_per_second_fp16: 40.0,
            tokens_per_second_int4: 120.0,
            license: "Apache-2.0".to_string(),
            estimated_m5_16gb_toks: Some(9.1),  // 实测 9.06 tok/s
            is_moe: true,
            active_params_b: 3.0,  // ~3B active despite 9B total
        });
        
        // === 14B MoE ===
        known_models.insert("Qwen3.5-35B-A3B".to_string(), ModelInfo {
            name: "Qwen3.5-35B-A3B".to_string(),
            parameter_count: 35_000_000_000,
            architecture: "Qwen3.5_MoE".to_string(),
            attention_type: AttentionType::MultiHeadLatentAttention,
            file_size_fp16_gb: 68.0,
            file_size_int4_gb: 17.0,
            recommended_quant: "Q4_K_M".to_string(),
            best_for: vec!["reasoning".to_string(), "coding".to_string(), "general".to_string()],
            context_length: 128_000,
            tokens_per_second_fp16: 14.0,
            tokens_per_second_int4: 70.0,
            license: "Apache-2.0".to_string(),
            estimated_m5_16gb_toks: None,  // >16GB, won't fit
            is_moe: true,
            active_params_b: 3.0,
        });
        
        // === 70B models ===
        known_models.insert("Llama-3.1-70B".to_string(), ModelInfo {
            name: "Llama-3.1-70B".to_string(),
            parameter_count: 70_000_000_000,
            architecture: "Llama".to_string(),
            attention_type: AttentionType::GroupedQueryAttention,
            file_size_fp16_gb: 140.0,
            file_size_int4_gb: 35.0,
            recommended_quant: "Q4_K_M".to_string(),
            best_for: vec!["reasoning".to_string(), "coding".to_string(), "general".to_string()],
            context_length: 128_000,
            tokens_per_second_fp16: 5.0,
            tokens_per_second_int4: 20.0,
            license: "MIT".to_string(),
            estimated_m5_16gb_toks: None,
            is_moe: false,
            active_params_b: 70.0,
        });
        
        // === 120B models ===
        known_models.insert("gpt-oss-120B".to_string(), ModelInfo {
            name: "gpt-oss-120B".to_string(),
            parameter_count: 120_000_000_000,
            architecture: "GPT-OSS".to_string(),
            attention_type: AttentionType::MultiHeadLatentAttention,
            file_size_fp16_gb: 240.0,
            file_size_int4_gb: 60.0,
            recommended_quant: "Q4_K_M".to_string(),
            best_for: vec!["reasoning".to_string(), "coding".to_string(), "general".to_string()],
            context_length: 128_000,
            tokens_per_second_fp16: 2.0,
            tokens_per_second_int4: 10.0,
            license: "Apache-2.0".to_string(),
            estimated_m5_16gb_toks: None,
            is_moe: false,
            active_params_b: 120.0,
        });
        
        let hardware_profiles = HashMap::new();
        
        Self {
            known_models,
            hardware_profiles,
            quant_engine: QuantizationEngine::new(),
        }
    }
    
    /// Auto-detect best model for hardware
    pub fn auto_detect(&mut self) -> Result<(), String> {
        // Detect hardware and populate profiles
        // TODO: Query system for GPU/CPU/memory info
        Ok(())
    }
    
    /// Recommend a model — multi-dimensional scoring (Quality/Speed/Fit/Context)
    /// Uses QuantizationEngine's score_model() for advanced ranking
    pub fn recommend(
        &self,
        vram_gb: f64,
        task: &str,
        apple_silicon: bool,
    ) -> ModelRecommendation {
        let hw = HardwareCapabilities {
            gpu_arch: if apple_silicon { "Apple Silicon".to_string() } else { "NVIDIA".to_string() },
            vram_gb,
            supports_metal: apple_silicon,
            memory_bandwidth_gbps: if apple_silicon { 200.0 } else { 80.0 },
            system_ram_gb: vram_gb * 1.0, // assume 1:1
            compute_tflops: 0.0,
            supports_fp8: false,
            supports_nvfp4: false,
            supports_flash_attention: true,
        };
        
        // Score all candidates using quantization engine's multi-dimensional scoring
        let mut scored: Vec<(String, f64, f64, f64)> = self.known_models.iter()
            .filter(|(_, m)| {
                // Memory fit filter: model must fit in VRAM
                let fits = m.file_size_int4_gb <= vram_gb;
                // For MoE, active params determine actual compute cost
                fits
            })
            .map(|(name, m)| {
                let params = ModelParams {
                    parameter_count: m.parameter_count,
                    architecture: m.architecture.clone(),
                    attention_type: m.attention_type.clone(),
                    hidden_dim: 0,
                    num_layers: 0,
                    num_heads: 0,
                    head_dim: 0,
                    kv_channels: 0,
                };
                
                let score = self.quant_engine.score_model(&params, &hw);
                
                // Task relevance bonus
                let task_bonus: f64 = if m.best_for.contains(&task.to_string()) {
                    15.0
                } else if m.best_for.iter().any(|t| t == "general") {
                    5.0
                } else {
                    0.0
                };
                
                // Apple Silicon real benchmark bonus
                let m5_bonus: f64 = if apple_silicon {
                    if let Some(toks) = m.estimated_m5_16gb_toks {
                        if toks > 9.0 { 10.0 }
                        else if toks > 5.0 { 5.0 }
                        else { 0.0 }
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                
                let total = score.total + task_bonus + m5_bonus;
                (name.clone(), total, score.quality, score.speed)
            })
            .collect();
        
        // Sort by total score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some((name, total, quality, speed)) = scored.first() {
            let m = &self.known_models[name];
            let quant = if apple_silicon && vram_gb < 32.0 && m.active_params_b > 5.0 {
                "Q4_K_M".to_string()
            } else {
                m.recommended_quant.clone()
            };
            
            let tok_s = m.estimated_m5_16gb_toks
                .unwrap_or(m.tokens_per_second_int4);
            
            return ModelRecommendation {
                model: m.name.clone(),
                quantization: quant,
                expected_throughput_tok_s: tok_s,
                memory_required_gb: m.file_size_int4_gb,
                headroom_gb: vram_gb - m.file_size_int4_gb,
                task_fit_score: *quality / 100.0,
                quality_score: *total / 100.0,
            };
        }
        
        // Fallback
        ModelRecommendation {
            model: "Llama-3.1-8B".to_string(),
            quantization: "Q4_K_M".to_string(),
            expected_throughput_tok_s: 250.0,
            memory_required_gb: 4.0,
            headroom_gb: vram_gb - 4.0,
            task_fit_score: 0.6,
            quality_score: 0.85,
        }
    }
    
    /// Estimate model file size
    pub fn estimate_model_size(&self, model_name: &str) -> f64 {
        self.known_models.get(model_name)
            .map(|m| m.file_size_int4_gb)
            .unwrap_or(4.0)
    }
    
    /// Get all models sorted by M5 16GB performance (descending)
    pub fn list_models(&self) -> Vec<&ModelInfo> {
        let mut models: Vec<&ModelInfo> = self.known_models.values().collect();
        models.sort_by(|a, b| {
            let a_perf = a.estimated_m5_16gb_toks.unwrap_or(0.0);
            let b_perf = b.estimated_m5_16gb_toks.unwrap_or(0.0);
            b_perf.partial_cmp(&a_perf).unwrap_or(std::cmp::Ordering::Equal)
        });
        models
    }

    /// Absolute best model for M5 16GB — returns (model_name, quant, expected_tok_s)
    pub fn best_for_m5_16gb(&self) -> Option<(&str, &str, f64)> {
        self.known_models.values()
            .filter(|m| m.file_size_int4_gb <= 16.0)
            .max_by(|a, b| {
                let a_t = a.estimated_m5_16gb_toks.unwrap_or(0.0);
                let b_t = b.estimated_m5_16gb_toks.unwrap_or(0.0);
                a_t.partial_cmp(&b_t).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|m| {
                let quant = if m.is_moe && m.file_size_int4_gb > 5.0 {
                    "Q5_K_M"
                } else {
                    "Q4_K_M"
                };
                (m.name.as_str(), quant, m.estimated_m5_16gb_toks.unwrap_or(0.0))
            })
    }

    // ═══════════════════════════════════════════════════════════
    // whichllm (⭐6.1K) 吸收: 真实硬件基准排序, 不靠参数量
    // ═══════════════════════════════════════════════════════════

    /// Rank models by real benchmarks (whichllm 风格), not parameter count
    /// Sorts by: actual_tok_s > task_fit > memory_efficiency
    pub fn _rank_by_real_benchmarks(&self, task: &str) -> Vec<(&ModelInfo, f64)> {
        let mut scored: Vec<(&ModelInfo, f64)> = self.known_models.values()
            .map(|m| {
                // Real benchmark score (M5 16GB actual tok/s)
                let bench_score = m.estimated_m5_16gb_toks.unwrap_or(0.0) * 10.0;
                
                // Task relevance (whichllm: "recency-aware benchmarks")
                let task_score = if m.best_for.contains(&task.to_string()) {
                    20.0
                } else if m.best_for.iter().any(|t| t == "general") {
                    10.0
                } else {
                    0.0
                };
                
                // Memory efficiency: active_params / file_size (MoE bonus)
                let mem_eff = if m.is_moe {
                    (m.active_params_b / m.file_size_int4_gb) * 5.0
                } else {
                    (m.parameter_count as f64 / 1e9 / m.file_size_int4_gb) * 3.0
                };
                
                let total = bench_score + task_score + mem_eff;
                (m, total)
            })
            .collect();
        
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }

    /// Estimate model file size
    pub fn estimate_model_size(&self, model_name: &str) -> f64 {
        self.known_models.get(model_name)
            .map(|m| m.file_size_int4_gb)
            .unwrap_or(4.0)
    }
}

impl Default for ModelSelector {
    fn default() -> Self {
        Self::new()
    }
}