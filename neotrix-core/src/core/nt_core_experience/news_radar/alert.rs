#![forbid(unsafe_code)]

use std::collections::HashMap;

use super::correlator::CrossSourceResult;
use super::predictor::PredictedTrend;
use super::types::{
    AlertLevel, EarlyWarningSignal, HotnessScore, NewsBriefing, OpinionAlert, SentimentDivergence,
    SentimentLabel, TopicTrend,
};

// ── OpinionFlowReport ──

#[derive(Debug, Clone)]
pub struct OpinionFlowReport {
    pub trends: Vec<TopicTrend>,
    pub cross_source: Vec<CrossSourceResult>,
    pub predictions: Vec<PredictedTrend>,
    pub signal_confidence: f64,
    pub briefing: NewsBriefing,
    pub sentiment_summary: HashMap<String, usize>,
    pub alerts: Vec<OpinionAlert>,
}

impl OpinionFlowReport {
    pub fn is_empty(&self) -> bool {
        self.trends.is_empty() && self.predictions.is_empty() && self.briefing.is_empty()
    }
}

// ── Alert generation functions ──

/// Generate BettaFish-style multi-factor alerts from trend data.
pub fn alerts_from_trends(
    trends: &[TopicTrend],
    cross_source: &[CrossSourceResult],
    predictions: &[PredictedTrend],
) -> Vec<OpinionAlert> {
    let mut alerts: Vec<OpinionAlert> = Vec::new();

    // Build cross-source coverage map
    let mut coverage_map: HashMap<String, Vec<String>> = HashMap::new();
    for cs in cross_source {
        if cs.coverage_count >= 2 {
            for (src_name, _) in &cs.sources {
                for word in cs.canonical_title.split_whitespace() {
                    if word.len() > 3 {
                        coverage_map
                            .entry(word.to_lowercase())
                            .or_default()
                            .push(src_name.clone());
                    }
                }
            }
        }
    }

    for trend in trends {
        let cross_source_count = coverage_map.get(&trend.topic).map(|v| v.len()).unwrap_or(0);
        let cross_src_names = coverage_map.get(&trend.topic).cloned().unwrap_or_default();
        let sentiment_divergence = SentimentDivergence::compute(&trend.sentiment_trend);

        let velocities: Vec<f64> = trend.window_counts.iter().map(|&c| c as f64).collect();
        let early_warning = EarlyWarningSignal::compute(&velocities);

        let sentiment_momentum = if trend.sentiment_trend.is_empty() {
            0.0
        } else {
            let recent: Vec<&SentimentLabel> = trend.sentiment_trend.iter().rev().take(5).collect();
            let pos_count = recent
                .iter()
                .filter(|s| matches!(s, SentimentLabel::VeryPositive | SentimentLabel::Positive))
                .count();
            let neg_count = recent
                .iter()
                .filter(|s| matches!(s, SentimentLabel::VeryNegative | SentimentLabel::Negative))
                .count();
            let total = (pos_count + neg_count).max(1) as f64;
            (pos_count as f64 - neg_count as f64) / total
        };

        let hotness = HotnessScore::new(
            trend.velocity,
            trend.acceleration,
            sentiment_divergence.divergence_index,
            trend.cross_source_count.max(cross_source_count),
            sentiment_momentum,
        );

        let pred_conf = predictions
            .iter()
            .find(|p| p.topic == trend.topic)
            .map(|p| p.confidence)
            .unwrap_or(0.0);

        let level = AlertLevel::from_score(hotness.composite);
        if level >= AlertLevel::Yellow {
            let mut triggers: Vec<String> = Vec::new();
            if trend.velocity > 0.5 {
                triggers.push(format!("velocity:{:.1}", trend.velocity));
            }
            if sentiment_divergence.is_polarizing {
                triggers.push("polarization".to_string());
            }
            if early_warning.is_pre_tipping {
                triggers.push("pre-tipping".to_string());
            }
            if hotness.composite >= 5.0 {
                triggers.push(format!("hotness:{:.1}", hotness.composite));
            }
            if cross_source_count >= 3 {
                triggers.push(format!("cross-source:{}", cross_source_count));
            }

            alerts.push(OpinionAlert {
                topic: trend.topic.clone(),
                level,
                hotness,
                divergence: sentiment_divergence,
                early_warning,
                prediction_confidence: pred_conf,
                sources_covering: cross_src_names,
                trigger_reason: if triggers.is_empty() {
                    "monitoring".to_string()
                } else {
                    triggers.join(", ")
                },
            });
        }
    }

    alerts.sort_by(|a, b| {
        b.hotness
            .composite
            .partial_cmp(&a.hotness.composite)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    alerts.truncate(10);
    alerts
}
