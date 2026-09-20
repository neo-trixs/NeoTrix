//! LayaEngine - Direct model inference engine
//!
//! This engine uses the Laya model directly for structured decisions.

use std::collections::HashMap;
use std::path::Path;
use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use crate::types::*;
use crate::error::Result;
use crate::tokenizer::LayaTokenizer;
use crate::sequence::{SequenceBuilder, BuiltSequence};
use crate::model::{LayaModel, ModelConfig};
use crate::temperature::TemperatureScaler;

/// LayaEngine for direct model inference
pub struct LayaEngine {
    model: LayaModel,
    sequence_builder: SequenceBuilder,
    temperature: TemperatureScaler,
    device: Device,
    /// Per-question errors collected during evaluate()
    pub errors: std::sync::Mutex<Vec<(String, String)>>,
}

impl LayaEngine {
    /// Create from files
    pub fn from_files(
        weights_path: &Path,
        tokenizer_path: &Path,
        config: ModelConfig,
        temperature: TemperatureScaler,
        device: Device,
    ) -> Result<Self> {
        Self::from_files_with_max_len(
            weights_path, tokenizer_path, config, temperature, device, 512,
        )
    }

    /// Create from files with configurable max sequence length
    pub fn from_files_with_max_len(
        weights_path: &Path,
        tokenizer_path: &Path,
        config: ModelConfig,
        temperature: TemperatureScaler,
        device: Device,
        max_len: usize,
    ) -> Result<Self> {
        // Load tokenizer
        let tokenizer = LayaTokenizer::from_file(tokenizer_path)?;
        
        // Load SafeTensors data
        let weights_data = std::fs::read(weights_path)?;
        
        // Resolve dtype from config
        let dtype = match config.weight_dtype.as_str() {
            "f16" | "half" => {
                tracing::info!("Loading weights in f16 (half precision)");
                DType::F16
            }
            _ => DType::F32,
        };
        
        // Create VarBuilder directly from SafeTensors bytes
        let vb = VarBuilder::from_slice_safetensors(&weights_data, dtype, &device)?;
        
        // Load model
        let model = LayaModel::from_weights(&vb, &config, &device)?;
        
        let sequence_builder = SequenceBuilder::new(tokenizer, max_len, 192);
        
        Ok(Self {
            model,
            sequence_builder,
            temperature,
            device,
            errors: std::sync::Mutex::new(Vec::new()),
        })
    }

    /// Create from a model directory (auto-discovers config.json, weights, tokenizer)
    pub fn from_model_dir(
        model_dir: &Path,
        temperature: TemperatureScaler,
        device: Device,
    ) -> Result<Self> {
        let config = ModelConfig::from_config_json(model_dir)?;
        let weights_path = model_dir.join("model.safetensors");
        let tokenizer_path = model_dir.join("tokenizer.json");

        if !weights_path.exists() {
            return Err(crate::error::Error::ModelNotFound(format!(
                "model.safetensors not found in {}", model_dir.display()
            )));
        }
        if !tokenizer_path.exists() {
            return Err(crate::error::Error::ModelNotFound(format!(
                "tokenizer.json not found in {}", model_dir.display()
            )));
        }

        let max_len = config.encoder_max_position_embeddings.min(512);
        Self::from_files_with_max_len(
            &weights_path, &tokenizer_path, config, temperature, device, max_len,
        )
    }
    
    /// Evaluate state against questions
    ///
    /// Each question is evaluated independently (matching Python Laya behavior),
    /// since different question types have different numbers of options/markers.
    /// Per-question errors are collected and returned alongside successful answers.
    pub fn evaluate(&self, state: &State, questions: &[Question]) -> Result<EvaluationResult> {
        let mut answers = HashMap::new();
        let mut total_input_tokens = 0u32;
        let mut errors = self.errors.lock().unwrap_or_else(|e| e.into_inner());
        errors.clear();

        for question in questions {
            match self.evaluate_one(state, question) {
                Ok((answer, tokens)) => {
                    answers.insert(question.id.clone(), answer);
                    total_input_tokens += tokens;
                }
                Err(e) => {
                    tracing::warn!("Failed to evaluate question '{}': {}", question.id, e);
                    errors.push((question.id.clone(), e.to_string()));
                }
            }
        }

        let usage = Usage {
            input_tokens: total_input_tokens,
            output_tokens: 0,
        };

        Ok(EvaluationResult {
            answers,
            model: Some("laya-rust".to_string()),
            usage: Some(usage),
        })
    }

    /// Evaluate multiple (state, questions) pairs sequentially
    ///
    /// Returns a Vec of EvaluationResult, one per input pair.
    /// Errors in individual questions are collected within each result
    /// (same as `evaluate`); errors in entire pairs propagate via `?`.
    pub fn evaluate_batch(
        &self,
        requests: &[(State, Vec<Question>)],
    ) -> Result<Vec<EvaluationResult>> {
        requests.iter()
            .map(|(state, questions)| self.evaluate(state, questions))
            .collect()
    }

    /// Evaluate a single question, returning the answer and token count
    fn evaluate_one(&self, state: &State, question: &Question) -> Result<(Answer, u32)> {
        let sequence = self.sequence_builder.build(state, question)?;
        let (input_ids, attention_mask, marker_pos, qtype, marker_mask) =
            self.prepare_inputs(&[sequence])?;

        let (logits, _act_logits) = self.model.forward(
            &input_ids,
            &attention_mask,
            &marker_pos,
            &marker_mask,
            &qtype,
        )?;

        let answer = self.process_question(&logits, 0, question)?;
        let tokens = attention_mask.sum_all()?.to_scalar::<u32>()?;

        Ok((answer, tokens))
    }

    /// Drain accumulated per-question errors from the last evaluate() call
    pub fn drain_errors(&self) -> Vec<(String, String)> {
        let mut errors = self.errors.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut *errors)
    }
    
    /// Prepare batch inputs
    fn prepare_inputs(
        &self,
        sequences: &[BuiltSequence],
    ) -> Result<(Tensor, Tensor, Tensor, Tensor, Tensor)> {
        let max_len = sequences.iter()
            .map(|s| s.input_ids.len())
            .max()
            .unwrap_or(0);
        let max_markers = sequences.iter()
            .map(|s| s.marker_positions.len())
            .max()
            .unwrap_or(0);
        
        let mut all_input_ids = Vec::new();
        let mut all_attention_mask = Vec::new();
        let mut all_marker_pos = Vec::new();
        let mut all_qtype = Vec::new();
        let mut all_marker_mask = Vec::new();

        for seq in sequences {
            let mut ids = seq.input_ids.clone();
            let mut mask = seq.attention_mask.clone();

            while ids.len() < max_len {
                ids.push(self.sequence_builder.pad_token_id());
                mask.push(0);
            }

            let mut markers = seq.marker_positions.clone();
            // Pad marker positions with sentinel value (seq_len) instead of 0,
            // so out-of-range markers produce zero vectors via bounds check
            while markers.len() < max_markers {
                markers.push(max_len);
            }

            // marker_mask: 1 for real markers, 0 for padding
            let marker_mask: Vec<f32> = (0..max_markers)
                .map(|i| if i < seq.marker_positions.len() { 1.0 } else { 0.0 })
                .collect();
            
            all_input_ids.push(ids);
            all_attention_mask.push(mask);
            all_marker_pos.push(markers);
            all_marker_mask.push(marker_mask);
            
            let type_id = match &seq.question_type {
                QuestionType::Noul { .. } => 0u32,
                QuestionType::Choice { .. } => 1u32,
                QuestionType::Score { .. } => 2u32,
            };
            all_qtype.push(type_id);
        }
        
        let input_ids = Tensor::new(all_input_ids, &self.device)?;
        let attention_mask = Tensor::new(all_attention_mask, &self.device)?;
        
        // Convert marker positions to u32 for Tensor creation
        let marker_pos_u32: Vec<Vec<u32>> = all_marker_pos
            .iter()
            .map(|v| v.iter().map(|&x| x as u32).collect())
            .collect();
        let marker_pos = Tensor::new(marker_pos_u32, &self.device)?;
        
        let qtype = Tensor::new(all_qtype, &self.device)?;
        let marker_mask = Tensor::new(all_marker_mask, &self.device)?;
        
        Ok((input_ids, attention_mask, marker_pos, qtype, marker_mask))
    }
    
    /// Process a single question
    fn process_question(
        &self,
        logits: &Tensor,
        idx: usize,
        question: &Question,
    ) -> Result<Answer> {
        let logits_row = logits.get(idx)?;
        let logits_vec_f32: Vec<f32> = logits_row.to_vec1()?;
        let mut logits_vec: Vec<f64> = logits_vec_f32.iter().map(|&x| x as f64).collect();
        
        let num_options = logits_vec.len();
        self.temperature.scale(&mut logits_vec, &question.question_type, num_options);
        
        let probs = softmax(&logits_vec);
        
        match &question.question_type {
            QuestionType::Noul { .. } => {
                // JEV format: value is boolean, probability is P(true)
                let is_true = probs[1] >= probs[0];
                let probability = probs[1]; // P(true)
                Ok(Answer::Noul(NoulAnswer {
                    value: is_true,
                    probability,
                    status: DecisionStatus::Selected,
                }))
            }
            QuestionType::Choice { criteria, .. } => {
                let keys: Vec<String> = criteria.keys().cloned().collect();
                let max_idx = probs.iter().enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                
                // Compute margin: gap between top-1 and top-2
                let mut sorted_probs = probs.clone();
                sorted_probs.sort_by(|a, b| b.partial_cmp(a).unwrap());
                let margin = if sorted_probs.len() >= 2 {
                    sorted_probs[0] - sorted_probs[1]
                } else {
                    sorted_probs[0]
                };
                
                Ok(Answer::Choice(ChoiceAnswer {
                    status: DecisionStatus::Selected,
                    value: keys[max_idx].clone(),
                    probability: probs[max_idx],
                    margin,
                }))
            }
            QuestionType::Score { .. } => {
                let exp_score: f64 = probs.iter().enumerate()
                    .map(|(i, p)| i as f64 * p)
                    .sum();
                
                let probabilities: HashMap<String, f64> = probs.iter().enumerate()
                    .map(|(i, p)| (i.to_string(), *p))
                    .collect();
                
                Ok(Answer::Score(ScoreAnswer {
                    status: DecisionStatus::Scored,
                    value: exp_score,
                    probabilities,
                }))
            }
        }
    }
}

/// Softmax function
fn softmax(logits: &[f64]) -> Vec<f64> {
    let max = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = logits.iter().map(|l| (l - max).exp()).collect();
    let sum: f64 = exps.iter().sum();
    exps.iter().map(|e| e / sum).collect()
}

/// Implement DecisionBackend for LayaEngine so it can be used directly with the Router
impl crate::router::DecisionBackend for LayaEngine {
    fn predict(&self, state: &State, questions: &[Question]) -> Result<EvaluationResult> {
        self.evaluate(state, questions)
    }
}

/// Builder for constructing LayaEngine with sensible defaults
///
/// # Example
/// ```no_run
/// use neotrix_decision_engine::LayaEngineBuilder;
/// use candle_core::Device;
///
/// let engine = LayaEngineBuilder::new()
///     .model_dir(std::path::Path::new("models/modernbert-large"))
///     .device(Device::Cpu)
///     .max_len(512)
///     .build()
///     .unwrap();
/// ```
pub struct LayaEngineBuilder {
    model_dir: Option<std::path::PathBuf>,
    weights_path: Option<std::path::PathBuf>,
    tokenizer_path: Option<std::path::PathBuf>,
    config: Option<ModelConfig>,
    temperature: Option<TemperatureScaler>,
    device: Option<Device>,
    max_len: Option<usize>,
    weight_dtype: Option<String>,
}

impl LayaEngineBuilder {
    /// Create a new builder with no settings
    pub fn new() -> Self {
        Self {
            model_dir: None,
            weights_path: None,
            tokenizer_path: None,
            config: None,
            temperature: None,
            device: None,
            max_len: None,
            weight_dtype: None,
        }
    }

    /// Set model directory (auto-discovers config.json, model.safetensors, tokenizer.json)
    pub fn model_dir(mut self, path: &std::path::Path) -> Self {
        self.model_dir = Some(path.to_path_buf());
        self
    }

    /// Set explicit weights path (overrides model_dir)
    pub fn weights(mut self, path: &std::path::Path) -> Self {
        self.weights_path = Some(path.to_path_buf());
        self
    }

    /// Set explicit tokenizer path (overrides model_dir)
    pub fn tokenizer(mut self, path: &std::path::Path) -> Self {
        self.tokenizer_path = Some(path.to_path_buf());
        self
    }

    /// Set model config (overrides auto-discovery from config.json)
    pub fn config(mut self, config: ModelConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set temperature scaler (default: all 1.0)
    pub fn temperature(mut self, temperature: TemperatureScaler) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set device (default: auto-detect Metal on macOS, CPU otherwise)
    pub fn device(mut self, device: Device) -> Self {
        self.device = Some(device);
        self
    }

    /// Set max sequence length (default: min(encoder_max_position_embeddings, 512))
    pub fn max_len(mut self, max_len: usize) -> Self {
        self.max_len = Some(max_len);
        self
    }

    /// Set weight dtype: "f32" (default) or "f16" (faster on Metal, half memory)
    pub fn weight_dtype(mut self, dtype: impl Into<String>) -> Self {
        self.weight_dtype = Some(dtype.into());
        self
    }

    /// Build the LayaEngine
    ///
    /// Resolution order:
    /// 1. If `model_dir` is set, auto-discover files from it
    /// 2. If explicit `weights`/`tokenizer` paths are set, use those
    /// 3. `config` overrides any auto-discovered config
    /// 4. `device` defaults to Metal (macOS) or CPU
    /// 5. `max_len` defaults to min(encoder_max_position_embeddings, 512)
    pub fn build(self) -> Result<LayaEngine> {
        // Resolve model directory
        let model_dir = self.model_dir.as_deref();

        // Resolve config
        let mut config = if let Some(c) = self.config {
            c
        } else if let Some(dir) = model_dir {
            ModelConfig::from_config_json(dir)?
        } else {
            ModelConfig::modernbert_large()
        };
        
        // Override weight_dtype if set
        if let Some(dtype) = self.weight_dtype {
            config.weight_dtype = dtype;
        }

        // Resolve weights path
        let weights_path = self.weights_path
            .or_else(|| model_dir.map(|d| d.join("model.safetensors")))
            .ok_or_else(|| crate::error::Error::ModelNotFound(
                "No model path specified. Use .model_dir() or .weights()".into()
            ))?;

        // Resolve tokenizer path
        let tokenizer_path = self.tokenizer_path
            .or_else(|| model_dir.map(|d| d.join("tokenizer.json")))
            .ok_or_else(|| crate::error::Error::ModelNotFound(
                "No tokenizer path specified. Use .model_dir() or .tokenizer()".into()
            ))?;

        // Resolve device
        let device = self.device
            .unwrap_or_else(|| ModelConfig::resolve_device("auto"));

        // Resolve max_len
        let max_len = self.max_len
            .unwrap_or_else(|| config.encoder_max_position_embeddings.min(512));

        // Resolve temperature
        let temperature = self.temperature.unwrap_or_default();

        LayaEngine::from_files_with_max_len(
            &weights_path, &tokenizer_path, config, temperature, device, max_len,
        )
    }
}

impl Default for LayaEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
