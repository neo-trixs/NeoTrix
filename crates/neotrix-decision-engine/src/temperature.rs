//! Temperature scaling for logits
//!
//! Supports two config formats:
//! 1. Laya format: `"1,3": 0.7` (qtype_idx, num_options → temp)
//! 2. JEV format: `"choice:3-5": 1.251` (qtype_name:option_range → temp)

use crate::types::QuestionType;
use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;

/// Temperature scaler for logits
pub struct TemperatureScaler {
    default: [f64; 3],  // [noul, choice, score]
    by_options: HashMap<(usize, usize), f64>,
}

/// Parsed rl_agent_config.json structure
#[derive(serde::Deserialize)]
struct LayaAgentConfig {
    /// Default temperatures [noul, choice, score]
    temperature: Option<Vec<f64>>,
    /// Per-option-count temperature overrides (Laya format: "qtype_idx,num_options")
    temperature_by_options: Option<HashMap<String, f64>>,
}

/// JEV-style config with bucketed temperatures
#[derive(serde::Deserialize)]
pub struct JevConfig {
    /// Default temperatures [noul, choice, score]
    temperature: Option<Vec<f64>>,
    /// Per-option-count temperature overrides (JEV format: "qtype:range")
    temperature_by_options: Option<HashMap<String, f64>>,
}

/// Parse JEV format bucket spec like "choice:3-5" or "noul:2"
/// Returns (qtype_idx, min_options, max_options)
fn parse_jev_bucket(spec: &str) -> Option<(usize, usize, usize)> {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let qtype_idx = match parts[0] {
        "noul" => 0,
        "choice" => 1,
        "score" => 2,
        _ => return None,
    };
    
    let range_str = parts[1];
    if let Some(dash_pos) = range_str.find('-') {
        // Range format: "3-5"
        let min: usize = range_str[..dash_pos].parse().ok()?;
        let max: usize = range_str[dash_pos + 1..].parse().ok()?;
        Some((qtype_idx, min, max))
    } else {
        // Exact format: "2"
        let n: usize = range_str.parse().ok()?;
        Some((qtype_idx, n, n))
    }
}

impl TemperatureScaler {
    /// Create with default temperatures
    pub fn new(default: [f64; 3], by_options: HashMap<(usize, usize), f64>) -> Self {
        Self {
            default,
            by_options,
        }
    }
    
    /// Load from rl_agent_config.json (supports both Laya and JEV formats)
    pub fn from_config_path(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: LayaAgentConfig = serde_json::from_str(&content)?;
        Self::from_config(config)
    }

    /// Load from rl_agent_config.json bytes
    pub fn from_config_bytes(data: &[u8]) -> Result<Self> {
        let config: LayaAgentConfig = serde_json::from_slice(data)?;
        Self::from_config(config)
    }

    /// Load from JEV-style config JSON with bucketed temperatures
    pub fn from_jev_config(config: JevConfig) -> Result<Self> {
        let default = config.temperature
            .and_then(|v| {
                if v.len() >= 3 {
                    Some([v[0], v[1], v[2]])
                } else {
                    None
                }
            })
            .unwrap_or([1.0, 1.0, 1.0]);

        for (i, &t) in default.iter().enumerate() {
            if t <= 0.0 {
                return Err(crate::error::Error::WeightLoadError(format!(
                    "Temperature must be positive, got {} for index {}", t, i
                )));
            }
        }

        let mut by_options = HashMap::new();
        if let Some(overrides) = config.temperature_by_options {
            for (key, temp) in overrides {
                if temp <= 0.0 {
                    return Err(crate::error::Error::WeightLoadError(format!(
                        "Temperature must be positive, got {} for key '{}'", temp, key
                    )));
                }
                
                // Try JEV format first: "choice:3-5"
                if let Some((qtype_idx, min, max)) = parse_jev_bucket(&key) {
                    for n in min..=max {
                        by_options.insert((qtype_idx, n), temp);
                    }
                }
                // Fall back to Laya format: "1,3"
                else if let Some((qtype_idx, num_opts)) = parse_laya_bucket(&key) {
                    by_options.insert((qtype_idx, num_opts), temp);
                }
            }
        }

        Ok(Self { default, by_options })
    }

    fn from_config(config: LayaAgentConfig) -> Result<Self> {
        let default = config.temperature
            .and_then(|v| {
                if v.len() >= 3 {
                    Some([v[0], v[1], v[2]])
                } else {
                    None
                }
            })
            .unwrap_or([1.0, 1.0, 1.0]);

        for (i, &t) in default.iter().enumerate() {
            if t <= 0.0 {
                return Err(crate::error::Error::WeightLoadError(format!(
                    "Temperature must be positive, got {} for index {}", t, i
                )));
            }
        }

        let mut by_options = HashMap::new();
        if let Some(overrides) = config.temperature_by_options {
            for (key, temp) in overrides {
                if temp <= 0.0 {
                    return Err(crate::error::Error::WeightLoadError(format!(
                        "Temperature must be positive, got {} for key '{}'", temp, key
                    )));
                }
                
                // Try JEV format first: "choice:3-5"
                if let Some((qtype_idx, min, max)) = parse_jev_bucket(&key) {
                    for n in min..=max {
                        by_options.insert((qtype_idx, n), temp);
                    }
                }
                // Fall back to Laya format: "1,3"
                else if let Some((qtype_idx, num_opts)) = parse_laya_bucket(&key) {
                    by_options.insert((qtype_idx, num_opts), temp);
                }
            }
        }

        Ok(Self { default, by_options })
    }

    /// Scale logits in place
    pub fn scale(&self, logits: &mut [f64], qtype: &QuestionType, num_options: usize) {
        let temp = self.get_temperature(qtype, num_options);
        for logit in logits.iter_mut() {
            *logit /= temp;
        }
    }
    
    /// Get temperature for question type
    pub fn get_temperature(&self, qtype: &QuestionType, num_options: usize) -> f64 {
        let qtype_idx = match qtype {
            QuestionType::Noul { .. } => 0,
            QuestionType::Choice { .. } => 1,
            QuestionType::Score { .. } => 2,
        };
        
        let bucket = (qtype_idx, num_options);
        self.by_options.get(&bucket)
            .copied()
            .unwrap_or(self.default[qtype_idx])
    }
}

/// Parse Laya format bucket spec like "1,3"
fn parse_laya_bucket(key: &str) -> Option<(usize, usize)> {
    let parts: Vec<&str> = key.split(',').collect();
    if parts.len() == 2 {
        let qtype_idx = parts[0].parse::<usize>().ok()?;
        let num_opts = parts[1].parse::<usize>().ok()?;
        Some((qtype_idx, num_opts))
    } else {
        None
    }
}

impl Default for TemperatureScaler {
    fn default() -> Self {
        Self {
            default: [1.0, 1.0, 1.0],
            by_options: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_config() {
        let config_json = r#"{
            "temperature": [0.8, 1.0, 0.9],
            "temperature_by_options": {
                "1,3": 0.7,
                "2,4": 0.6
            }
        }"#;
        let config: LayaAgentConfig = serde_json::from_str(config_json).unwrap();
        let scaler = TemperatureScaler::from_config(config).unwrap();

        // Default temps
        let mut logits = vec![1.0, 2.0, 3.0];
        let noul = QuestionType::Noul { instructions: "test".into(), criteria: None };
        scaler.scale(&mut logits, &noul, 2);
        assert!((logits[0] - 1.25).abs() < 0.001); // 1.0 / 0.8

        // Override: (Choice, 3 options) -> 0.7
        let mut logits = vec![1.0, 2.0, 3.0];
        let choice = QuestionType::Choice { instructions: "test".into(), criteria: HashMap::new() };
        scaler.scale(&mut logits, &choice, 3);
        assert!((logits[0] - 1.0 / 0.7).abs() < 0.001);
    }

    #[test]
    fn test_jev_bucket_parsing() {
        // "choice:3-5" should create buckets for (Choice, 3), (Choice, 4), (Choice, 5)
        let config = JevConfig {
            temperature: Some(vec![1.0, 1.0, 1.0]),
            temperature_by_options: Some(HashMap::from([
                ("choice:3-5".to_string(), 1.251),
                ("noul:2".to_string(), 1.637),
            ])),
        };
        let scaler = TemperatureScaler::from_jev_config(config).unwrap();

        // Choice with 4 options should use 1.251
        let choice = QuestionType::Choice { instructions: "test".into(), criteria: HashMap::new() };
        let temp = scaler.get_temperature(&choice, 4);
        assert!((temp - 1.251).abs() < 0.001);

        // Choice with 2 options should use default
        let temp = scaler.get_temperature(&choice, 2);
        assert!((temp - 1.0).abs() < 0.001);

        // Noul with 2 options should use 1.637
        let noul = QuestionType::Noul { instructions: "test".into(), criteria: None };
        let temp = scaler.get_temperature(&noul, 2);
        assert!((temp - 1.637).abs() < 0.001);
    }

    #[test]
    fn test_default() {
        let scaler = TemperatureScaler::default();
        let mut logits = vec![1.0, 2.0];
        let noul = QuestionType::Noul { instructions: "test".into(), criteria: None };
        scaler.scale(&mut logits, &noul, 2);
        // Default temp is 1.0, so logits unchanged
        assert!((logits[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_invalid_temperature_rejected() {
        let config_json = r#"{"temperature": [-0.5, 1.0, 0.9]}"#;
        let config: LayaAgentConfig = serde_json::from_str(config_json).unwrap();
        let result = TemperatureScaler::from_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_temperature_rejected() {
        let config_json = r#"{"temperature": [0.0, 1.0, 0.9]}"#;
        let config: LayaAgentConfig = serde_json::from_str(config_json).unwrap();
        let result = TemperatureScaler::from_config(config);
        assert!(result.is_err());
    }
}
