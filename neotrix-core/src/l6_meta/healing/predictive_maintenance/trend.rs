#![forbid(unsafe_code)]

use std::fmt;

/// Direction of a trend detected in the data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

impl fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrendDirection::Improving => write!(f, "IMPROVING"),
            TrendDirection::Stable => write!(f, "STABLE"),
            TrendDirection::Degrading => write!(f, "DEGRADING"),
        }
    }
}

/// Result of a trend analysis.
#[derive(Debug, Clone)]
pub struct Trend {
    /// Overall direction of the trend.
    pub direction: TrendDirection,
    /// Linear slope of the trend (units per step).
    pub slope: f64,
    /// Confidence in the trend classification (0.0 ..= 1.0).
    pub confidence: f64,
    /// Human-readable description of the trend.
    pub description: String,
}

/// Analyzes trends in time-series data.
pub struct TrendAnalyzer {
    /// Threshold for considering a slope as "stable".
    stability_threshold: f64,
}

impl TrendAnalyzer {
    pub fn new() -> Self {
        Self {
            stability_threshold: 1e-6,
        }
    }

    pub fn with_stability_threshold(threshold: f64) -> Self {
        Self {
            stability_threshold: threshold.abs(),
        }
    }

    /// Analyze a sequence of data points and return a Trend.
    pub fn analyze(&self, history: &[f64]) -> Trend {
        if history.len() < 2 {
            return Trend {
                direction: TrendDirection::Stable,
                slope: 0.0,
                confidence: 0.0,
                description: "Insufficient data points for trend analysis.".into(),
            };
        }

        let (slope, r_squared) = self.compute_regression(history);

        let direction = if slope > self.stability_threshold {
            TrendDirection::Degrading
        } else if slope < -self.stability_threshold {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        };

        let confidence = r_squared.clamp(0.0, 1.0);

        let description = self.format_description(direction, slope, confidence, history.len());

        Trend {
            direction,
            slope,
            confidence,
            description,
        }
    }

    /// Simple linear regression returning (slope, r_squared).
    fn compute_regression(&self, data: &[f64]) -> (f64, f64) {
        let n = data.len() as f64;
        let sum_x: f64 = (0..data.len()).map(|i| i as f64).sum();
        let sum_y: f64 = data.iter().sum();
        let sum_xy: f64 = data.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..data.len()).map(|i| (i as f64).powi(2)).sum();

        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-12 {
            return (0.0, 0.0);
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denom;

        // R-squared calculation.
        let mean_y = sum_y / n;
        let ss_total: f64 = data.iter().map(|y| (y - mean_y).powi(2)).sum();
        let intercept = (sum_y - slope * sum_x) / n;
        let ss_res: f64 = data
            .iter()
            .enumerate()
            .map(|(i, y)| {
                let predicted = intercept + slope * i as f64;
                (y - predicted).powi(2)
            })
            .sum();

        let r_squared = if ss_total > 1e-12 {
            1.0 - ss_res / ss_total
        } else {
            0.0
        };

        (slope, r_squared)
    }

    fn format_description(
        &self,
        direction: TrendDirection,
        slope: f64,
        confidence: f64,
        data_points: usize,
    ) -> String {
        let trend_word = match direction {
            TrendDirection::Improving => "improving",
            TrendDirection::Stable => "stable",
            TrendDirection::Degrading => "degrading",
        };

        let confidence_pct = (confidence * 100.0).round() as u64;

        format!(
            "Trend is {} (slope: {:.4}, confidence: {}%, data points: {})",
            trend_word, slope, confidence_pct, data_points
        )
    }
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degrading_trend() {
        let analyzer = TrendAnalyzer::new();
        let history: Vec<f64> = (0..20).map(|i| 100.0 + i as f64 * 5.0).collect();
        let trend = analyzer.analyze(&history);
        assert_eq!(trend.direction, TrendDirection::Degrading);
        assert!(trend.slope > 0.0);
        assert!(trend.confidence > 0.9);
    }

    #[test]
    fn test_improving_trend() {
        let analyzer = TrendAnalyzer::new();
        let history: Vec<f64> = (0..20).map(|i| 100.0 - i as f64 * 2.0).collect();
        let trend = analyzer.analyze(&history);
        assert_eq!(trend.direction, TrendDirection::Improving);
        assert!(trend.slope < 0.0);
    }

    #[test]
    fn test_stable_trend() {
        let analyzer = TrendAnalyzer::new();
        let history = vec![50.0; 20];
        let trend = analyzer.analyze(&history);
        assert_eq!(trend.direction, TrendDirection::Stable);
        assert!(trend.slope.abs() < 1e-6);
    }

    #[test]
    fn test_single_point() {
        let analyzer = TrendAnalyzer::new();
        let trend = analyzer.analyze(&[42.0]);
        assert_eq!(trend.direction, TrendDirection::Stable);
        assert_eq!(trend.confidence, 0.0);
    }

    #[test]
    fn test_empty_history() {
        let analyzer = TrendAnalyzer::new();
        let trend = analyzer.analyze(&[]);
        assert_eq!(trend.direction, TrendDirection::Stable);
        assert!(trend.description.contains("Insufficient"));
    }

    #[test]
    fn test_description_contains_slope() {
        let analyzer = TrendAnalyzer::new();
        let history: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let trend = analyzer.analyze(&history);
        assert!(trend.description.contains("slope:"));
        assert!(trend.description.contains("confidence:"));
    }

    #[test]
    fn test_display() {
        assert_eq!(TrendDirection::Improving.to_string(), "IMPROVING");
        assert_eq!(TrendDirection::Stable.to_string(), "STABLE");
        assert_eq!(TrendDirection::Degrading.to_string(), "DEGRADING");
    }

    #[test]
    fn test_noisy_data_low_confidence() {
        let analyzer = TrendAnalyzer::new();
        let history: Vec<f64> = (0..10)
            .map(|i| {
                let base = i as f64;
                // Add large noise.
                base + if i % 2 == 0 { 100.0 } else { -100.0 }
            })
            .collect();
        let trend = analyzer.analyze(&history);
        assert!(trend.confidence < 0.5);
    }

    #[test]
    fn test_custom_stability_threshold() {
        let analyzer = TrendAnalyzer::with_stability_threshold(0.01);
        let history = vec![10.0, 10.005, 10.01, 10.015, 10.02];
        let trend = analyzer.analyze(&history);
        // Very small slope, should be stable with high threshold
        assert_eq!(trend.direction, TrendDirection::Stable);
    }

    #[test]
    fn test_description_length() {
        let analyzer = TrendAnalyzer::new();
        let history: Vec<f64> = (0..5).map(|i| i as f64).collect();
        let trend = analyzer.analyze(&history);
        assert!(trend.description.len() > 20);
    }

    #[test]
    fn test_improving_direction_slope_negative() {
        let analyzer = TrendAnalyzer::new();
        let history: Vec<f64> = (0..10).map(|i| 100.0 - i as f64 * 3.0).collect();
        let trend = analyzer.analyze(&history);
        assert_eq!(trend.direction, TrendDirection::Improving);
        assert!(trend.slope < 0.0);
    }

    #[test]
    fn test_stable_trend_near_zero_slope() {
        let analyzer = TrendAnalyzer::new();
        let history: Vec<f64> = (0..20).map(|_| 42.0 + 0.0000001).collect();
        let trend = analyzer.analyze(&history);
        assert_eq!(trend.direction, TrendDirection::Stable);
    }

    #[test]
    fn test_default_impl() {
        let a1 = TrendAnalyzer::new();
        let a2 = TrendAnalyzer::default();
        let history = vec![1.0, 2.0, 3.0];
        assert_eq!(
            a1.analyze(&history).direction,
            a2.analyze(&history).direction
        );
    }

    #[test]
    fn test_two_points_degrading() {
        let analyzer = TrendAnalyzer::new();
        let trend = analyzer.analyze(&[0.0, 100.0]);
        assert_eq!(trend.direction, TrendDirection::Degrading);
        assert!(trend.slope > 0.0);
    }

    #[test]
    fn test_two_points_improving() {
        let analyzer = TrendAnalyzer::new();
        let trend = analyzer.analyze(&[100.0, 0.0]);
        assert_eq!(trend.direction, TrendDirection::Improving);
        assert!(trend.slope < 0.0);
    }

    #[test]
    fn test_confidence_bounded() {
        let analyzer = TrendAnalyzer::new();
        let history: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let trend = analyzer.analyze(&history);
        assert!(trend.confidence >= 0.0);
        assert!(trend.confidence <= 1.0);
    }
}
