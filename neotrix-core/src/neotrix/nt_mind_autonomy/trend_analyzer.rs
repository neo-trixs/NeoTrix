use crate::core::nt_core_cap::{CapabilityVector, FIELD_NAMES};

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub timestamp: u64,
    pub label: String,
    pub value: f64,
    pub dimension: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    InsufficientData,
}

#[derive(Debug, Clone)]
pub struct Trend {
    pub label: String,
    pub direction: TrendDirection,
    pub slope: f64,
    pub intercept: f64,
    pub confidence: f64,
    pub data_points: usize,
    pub prediction_next: f64,
}

#[derive(Debug, Clone)]
pub struct TrendReport {
    pub trends: Vec<Trend>,
    pub overall_direction: TrendDirection,
    pub declining_count: u32,
    pub improving_count: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct EvolutionTrendAnalyzer {
    history: Vec<DataPoint>,
    min_points_for_trend: usize,
    stable_threshold: f64,
    max_history: usize,
}

impl EvolutionTrendAnalyzer {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            min_points_for_trend: 3,
            stable_threshold: 0.02,
            max_history: 1000,
        }
    }

    pub fn record(&mut self, label: &str, value: f64, dimension: Option<&str>) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.history.push(DataPoint {
            timestamp,
            label: label.to_string(),
            value,
            dimension: dimension.map(|s| s.to_string()),
        });
        self.prune();
    }

    pub fn record_capability(&mut self, cv: &CapabilityVector, prefix: &str) {
        for (i, name) in FIELD_NAMES.iter().enumerate() {
            let label = if prefix.is_empty() {
                (*name).to_string()
            } else {
                format!("{}.{}", prefix, name)
            };
            let val = cv.arr.get(i).copied().unwrap_or(0.0);
            self.record(&label, val, Some(name));
        }
        for (ext_name, ext_val) in &cv.extension {
            let label = if prefix.is_empty() {
                ext_name.clone()
            } else {
                format!("{}.{}", prefix, ext_name)
            };
            self.record(&label, *ext_val, Some(ext_name));
        }
    }

    pub fn analyze(&self) -> TrendReport {
        let mut label_map: std::collections::HashMap<&str, Vec<&DataPoint>> =
            std::collections::HashMap::new();
        for dp in &self.history {
            label_map.entry(&dp.label).or_default().push(dp);
        }

        let mut trends: Vec<Trend> = Vec::new();
        let mut improving = 0u32;
        let mut declining = 0u32;

        for (label, points) in &label_map {
            if points.len() < self.min_points_for_trend {
                trends.push(Trend {
                    label: label.to_string(),
                    direction: TrendDirection::InsufficientData,
                    slope: 0.0,
                    intercept: 0.0,
                    confidence: 0.0,
                    data_points: points.len(),
                    prediction_next: points.last().map(|p| p.value).unwrap_or(0.0),
                });
                continue;
            }

            let mut sorted = points.clone();
            sorted.sort_by_key(|dp| dp.timestamp);

            let (slope, intercept, r_squared) = Self::linear_regression(&sorted);
            let direction = self.classify_direction(slope);

            let next_idx = sorted.len() as f64;
            let prediction_next = slope * next_idx + intercept;

            match direction {
                TrendDirection::Improving => improving += 1,
                TrendDirection::Declining => declining += 1,
                _ => {}
            }

            trends.push(Trend {
                label: label.to_string(),
                direction,
                slope,
                intercept,
                confidence: r_squared,
                data_points: sorted.len(),
                prediction_next,
            });
        }

        let overall = if improving > declining {
            TrendDirection::Improving
        } else if declining > improving {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        TrendReport {
            trends,
            overall_direction: overall,
            declining_count: declining,
            improving_count: improving,
            timestamp,
        }
    }

    pub fn trend_for(&self, label: &str) -> Option<Trend> {
        let points: Vec<&DataPoint> = self.history.iter().filter(|dp| dp.label == label).collect();
        if points.is_empty() {
            return None;
        }
        if points.len() < self.min_points_for_trend {
            let val = points.last().map(|p| p.value).unwrap_or(0.0);
            return Some(Trend {
                label: label.to_string(),
                direction: TrendDirection::InsufficientData,
                slope: 0.0,
                intercept: 0.0,
                confidence: 0.0,
                data_points: points.len(),
                prediction_next: val,
            });
        }

        let mut sorted = points.clone();
        sorted.sort_by_key(|dp| dp.timestamp);
        let (slope, intercept, r_squared) = Self::linear_regression(&sorted);
        let direction = self.classify_direction(slope);
        let next_idx = sorted.len() as f64;
        let prediction_next = slope * next_idx + intercept;

        Some(Trend {
            label: label.to_string(),
            direction,
            slope,
            intercept,
            confidence: r_squared,
            data_points: sorted.len(),
            prediction_next,
        })
    }

    fn linear_regression(points: &[&DataPoint]) -> (f64, f64, f64) {
        let n = points.len() as f64;
        let mean_x = (0..points.len()).map(|i| i as f64).sum::<f64>() / n;
        let mean_y = points.iter().map(|p| p.value).sum::<f64>() / n;

        let mut sxx = 0.0;
        let mut syy = 0.0;
        let mut sxy = 0.0;

        for (i, p) in points.iter().enumerate() {
            let xi = i as f64;
            let dx = xi - mean_x;
            let dy = p.value - mean_y;
            sxx += dx * dx;
            syy += dy * dy;
            sxy += dx * dy;
        }

        if sxx == 0.0 {
            return (0.0, mean_y, 0.0);
        }

        let slope = sxy / sxx;
        let intercept = mean_y - slope * mean_x;
        let r_squared = if syy == 0.0 {
            1.0
        } else {
            (sxy * sxy) / (sxx * syy)
        };

        (slope, intercept, r_squared)
    }

    fn classify_direction(&self, slope: f64) -> TrendDirection {
        if slope > self.stable_threshold {
            TrendDirection::Improving
        } else if slope < -self.stable_threshold {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }

    fn prune(&mut self) {
        if self.history.len() > self.max_history {
            let excess = self.history.len() - self.max_history;
            self.history.drain(0..excess);
        }
    }

    pub fn summary(&self) -> String {
        let report = self.analyze();
        format!(
            "EvolutionTrendAnalyzer: {} trends ({} improving, {} declining, overall {:?})",
            report.trends.len(),
            report.improving_count,
            report.declining_count,
            report.overall_direction
        )
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn history(&self) -> &[DataPoint] {
        &self.history
    }
}

impl Default for EvolutionTrendAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _make_points(values: &[f64]) -> Vec<DataPoint> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        values
            .iter()
            .enumerate()
            .map(|(i, &v)| DataPoint {
                timestamp: now + i as u64,
                label: "test_metric".to_string(),
                value: v,
                dimension: None,
            })
            .collect()
    }

    #[test]
    fn test_improving_trend() {
        let mut analyzer = EvolutionTrendAnalyzer::new();
        analyzer.min_points_for_trend = 2;
        for v in &[1.0, 2.0, 3.0, 4.0, 5.0] {
            analyzer.record("score", *v, None);
        }
        let trend = analyzer.trend_for("score").unwrap();
        assert_eq!(trend.direction, TrendDirection::Improving);
        assert!(trend.slope > 0.0);
        assert!(trend.confidence > 0.9);
    }

    #[test]
    fn test_declining_trend() {
        let mut analyzer = EvolutionTrendAnalyzer::new();
        analyzer.min_points_for_trend = 2;
        for v in &[5.0, 4.0, 3.0, 2.0, 1.0] {
            analyzer.record("score", *v, None);
        }
        let trend = analyzer.trend_for("score").unwrap();
        assert_eq!(trend.direction, TrendDirection::Declining);
        assert!(trend.slope < 0.0);
    }

    #[test]
    fn test_insufficient_data() {
        let mut analyzer = EvolutionTrendAnalyzer::new();
        analyzer.record("sparse", 1.0, None);
        analyzer.record("sparse", 2.0, None);
        let trend = analyzer.trend_for("sparse").unwrap();
        assert_eq!(trend.direction, TrendDirection::InsufficientData);
    }

    #[test]
    fn test_stable_trend() {
        let mut analyzer = EvolutionTrendAnalyzer::new();
        analyzer.min_points_for_trend = 2;
        analyzer.stable_threshold = 0.5;
        for v in &[1.0, 1.1, 0.9, 1.0, 1.05] {
            analyzer.record("stable_metric", *v, None);
        }
        let trend = analyzer.trend_for("stable_metric").unwrap();
        assert_eq!(trend.direction, TrendDirection::Stable);
    }

    #[test]
    fn test_record_capability() {
        let mut analyzer = EvolutionTrendAnalyzer::new();
        let cv = CapabilityVector::from_values(
            0.5, 0.6, 0.7, 0.8, 0.9, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.1, 0.2, 0.3,
            0.4, 0.5, 0.6, 0.7, 0.8, 0.9,
        );
        analyzer.record_capability(&cv, "cap");
        let count = analyzer.history().len();
        assert_eq!(count, 23);
        let trend = analyzer.trend_for("cap.typography").unwrap();
        assert_ne!(trend.data_points, 0);
    }

    #[test]
    fn test_summary_format() {
        let mut analyzer = EvolutionTrendAnalyzer::new();
        analyzer.record("a", 1.0, None);
        let s = analyzer.summary();
        assert!(s.contains("EvolutionTrendAnalyzer"));
        assert!(s.contains("trends"));
    }

    #[test]
    fn test_pruning() {
        let mut analyzer = EvolutionTrendAnalyzer::new();
        analyzer.max_history = 5;
        for i in 0..10 {
            analyzer.record("metric", i as f64, None);
        }
        assert_eq!(analyzer.len(), 5);
    }

    #[test]
    fn test_trend_report_aggregates() {
        let mut analyzer = EvolutionTrendAnalyzer::new();
        analyzer.min_points_for_trend = 2;
        analyzer.record("up", 1.0, None);
        analyzer.record("up", 2.0, None);
        analyzer.record("up", 3.0, None);
        analyzer.record("down", 3.0, None);
        analyzer.record("down", 2.0, None);
        analyzer.record("down", 1.0, None);
        let report = analyzer.analyze();
        assert_eq!(report.improving_count, 1);
        assert_eq!(report.declining_count, 1);
    }

    #[test]
    fn test_single_trend_no_trend() {
        let analyzer = EvolutionTrendAnalyzer::new();
        let trend = analyzer.trend_for("nonexistent");
        assert!(trend.is_none());
    }

    #[test]
    fn test_linear_regression_perfect() {
        let points: Vec<DataPoint> = (0..5)
            .map(|i| DataPoint {
                timestamp: i as u64,
                label: "perf".to_string(),
                value: (i * 2) as f64,
                dimension: None,
            })
            .collect();
        let refs: Vec<&DataPoint> = points.iter().collect();
        let (slope, intercept, r2) = EvolutionTrendAnalyzer::linear_regression(&refs);
        assert!((slope - 2.0).abs() < 1e-10);
        assert!((intercept - 0.0).abs() < 1e-10);
        assert!((r2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_linear_regression_flat() {
        let points: Vec<DataPoint> = (0..5)
            .map(|i| DataPoint {
                timestamp: i as u64,
                label: "flat".to_string(),
                value: 3.0,
                dimension: None,
            })
            .collect();
        let refs: Vec<&DataPoint> = points.iter().collect();
        let (slope, intercept, r2) = EvolutionTrendAnalyzer::linear_regression(&refs);
        assert!((slope - 0.0).abs() < 1e-10);
        assert!((intercept - 3.0).abs() < 1e-10);
        assert!((r2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_default() {
        let analyzer = EvolutionTrendAnalyzer::default();
        assert_eq!(analyzer.min_points_for_trend, 3);
        assert_eq!(analyzer.max_history, 1000);
    }
}
