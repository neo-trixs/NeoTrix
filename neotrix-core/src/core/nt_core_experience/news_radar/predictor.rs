#![forbid(unsafe_code)]

use std::collections::HashMap;

use super::correlator::CrossSourceResult;
use super::types::{TopicTrend, STOP_WORDS};

// ── PredictedTrend ──

#[derive(Debug, Clone)]
pub struct PredictedTrend {
    pub topic: String,
    pub confidence: f64,
    pub signal_strength: f64,
    pub based_on: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SignalPredictor {
    pub predictions: Vec<PredictedTrend>,
    history: Vec<TopicTrend>,
}

impl SignalPredictor {
    pub fn new() -> Self {
        Self {
            predictions: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Predict future trends based on current topic trajectories
    /// using ALF-inspired signal mutation: combine history + velocity + cross-source convergence
    pub fn predict(
        &mut self,
        trends: &HashMap<String, TopicTrend>,
        cross_source: &[CrossSourceResult],
    ) {
        self.predictions.clear();
        self.history = trends.values().cloned().collect();

        // Signal 1: Rising topics with acceleration
        for trend in trends.values() {
            if trend.is_rising && trend.acceleration > 0.0 {
                let signal_strength = trend.velocity * 0.4
                    + trend.acceleration * 0.3
                    + (trend.cross_source_count as f64).min(5.0) / 5.0 * 0.3;
                let confidence = signal_strength.clamp(0.0, 1.0);
                if confidence > 0.3 {
                    self.predictions.push(PredictedTrend {
                        topic: trend.topic.clone(),
                        confidence,
                        signal_strength,
                        based_on: vec![
                            "rising-velocity".to_string(),
                            "positive-acceleration".to_string(),
                        ],
                    });
                }
            }
        }

        // Signal 2: Multi-source convergence
        for story in cross_source {
            if story.coverage_count >= 2 {
                let signal_strength = (story.coverage_count as f64 / 5.0).min(1.0) * 0.6
                    + (story.avg_score / 10.0) * 0.4;
                let words: Vec<&str> = story
                    .canonical_title
                    .split_whitespace()
                    .filter(|w| w.len() > 3 && !STOP_WORDS.contains(w))
                    .collect();
                for word in words.iter().take(3) {
                    if !self.predictions.iter().any(|p| p.topic == *word) {
                        self.predictions.push(PredictedTrend {
                            topic: word.to_string(),
                            confidence: signal_strength.clamp(0.0, 1.0),
                            signal_strength,
                            based_on: vec![format!(
                                "cross-source-{}-sources",
                                story.coverage_count
                            )],
                        });
                    }
                }
            }
        }

        // Signal 3: ALF-style mutation
        let rising: Vec<&TopicTrend> = trends.values().filter(|t| t.is_rising).collect();
        for i in 0..rising.len().min(5) {
            for j in (i + 1)..rising.len().min(5) {
                let combined = format!("{}+{}", rising[i].topic, rising[j].topic);
                let signal_strength = (rising[i].velocity + rising[j].velocity) / 10.0;
                if signal_strength > 0.3 {
                    self.predictions.push(PredictedTrend {
                        topic: combined,
                        confidence: signal_strength.clamp(0.0, 1.0),
                        signal_strength,
                        based_on: vec![
                            format!("alf-mutation:{}", rising[i].topic),
                            format!("alf-mutation:{}", rising[j].topic),
                        ],
                    });
                }
            }
        }

        self.predictions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.predictions.truncate(10);
    }

    pub fn signal_confidence(&self) -> f64 {
        if self.predictions.is_empty() {
            return 0.0;
        }
        self.predictions.iter().map(|p| p.confidence).sum::<f64>() / self.predictions.len() as f64
    }
}
