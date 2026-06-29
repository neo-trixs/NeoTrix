use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoveltyLevel {
    Known,
    NearKnown,
    Ambiguous,
    Novel,
}

impl NoveltyLevel {
    pub fn curiosity_bonus(&self) -> f64 {
        match self {
            NoveltyLevel::Known => 0.0,
            NoveltyLevel::NearKnown => 0.3,
            NoveltyLevel::Ambiguous => 0.6,
            NoveltyLevel::Novel => 1.0,
        }
    }
}

pub struct NoveltyDetector {
    pub window: VecDeque<f64>,
    pub max_window: usize,
    pub baseline_threshold: f64,
    pub novelty_history: VecDeque<NoveltyLevel>,
    pub current_level: NoveltyLevel,
    pub current_score: f64,
    pub total_novel_events: u64,
    pub total_predictions: u64,
}

impl NoveltyDetector {
    pub fn new(baseline_threshold: f64, window_size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            max_window: window_size,
            baseline_threshold,
            novelty_history: VecDeque::with_capacity(20),
            current_level: NoveltyLevel::Known,
            current_score: 0.0,
            total_novel_events: 0,
            total_predictions: 0,
        }
    }

    pub fn feed_prediction_error(&mut self, error: f64) -> f64 {
        let error = error.clamp(0.0, 1.0);
        self.total_predictions += 1;

        self.window.push_back(error);
        if self.window.len() > self.max_window {
            self.window.pop_front();
        }

        let smoothed = self.ewma();
        self.current_score = smoothed;

        let level = self.classify(smoothed);
        self.current_level = level;
        self.novelty_history.push_back(level);
        if self.novelty_history.len() > 20 {
            self.novelty_history.pop_front();
        }

        if level == NoveltyLevel::Novel {
            self.total_novel_events += 1;
        }

        self.current_score
    }

    pub fn current_novelty_level(&self) -> NoveltyLevel {
        self.current_level
    }

    pub fn mean_error(&self) -> f64 {
        let len = self.window.len();
        if len == 0 {
            return 0.0;
        }
        self.window.iter().sum::<f64>() / len as f64
    }

    pub fn variance(&self) -> f64 {
        let len = self.window.len();
        if len < 2 {
            return 0.0;
        }
        let mean = self.mean_error();
        let sum_sq: f64 = self.window.iter().map(|e| (e - mean).powi(2)).sum();
        sum_sq / (len - 1) as f64
    }

    pub fn trend(&self) -> f64 {
        let n = self.window.len();
        if n < 5 {
            return 0.0;
        }
        let recent: Vec<f64> = self.window.iter().rev().take(5).copied().collect();
        let recent: Vec<f64> = recent.into_iter().rev().collect();
        let m = recent.len() as f64;
        let sum_x: f64 = (0..recent.len()).map(|i| i as f64).sum();
        let sum_y: f64 = recent.iter().sum();
        let sum_xy: f64 = recent.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..recent.len()).map(|i| (i as f64).powi(2)).sum();
        let denom = m * sum_x2 - sum_x.powi(2);
        if denom.abs() < 1e-12 {
            return 0.0;
        }
        (m * sum_xy - sum_x * sum_y) / denom
    }

    pub fn reset_baseline(&mut self) {
        let mean = self.mean_error();
        self.baseline_threshold = 2.0 * mean;
    }

    pub fn stats(&self) -> NoveltyStats {
        NoveltyStats {
            current_score: self.current_score,
            current_level: self.current_level,
            mean_error: self.mean_error(),
            variance: self.variance(),
            trend: self.trend(),
            total_novel: self.total_novel_events,
            total_predictions: self.total_predictions,
        }
    }

    fn ewma(&self) -> f64 {
        let len = self.window.len();
        if len == 0 {
            return 0.0;
        }
        let alpha = 2.0 / (len as f64 + 1.0);
        let mut ema = self.window[0];
        for i in 1..len {
            ema = alpha * self.window[i] + (1.0 - alpha) * ema;
        }
        ema
    }

    fn classify(&self, score: f64) -> NoveltyLevel {
        let t = self.baseline_threshold;
        if score < 0.2 * t {
            NoveltyLevel::Known
        } else if score < 0.5 * t {
            NoveltyLevel::NearKnown
        } else if score < t {
            NoveltyLevel::Ambiguous
        } else {
            NoveltyLevel::Novel
        }
    }
}

impl Default for NoveltyDetector {
    fn default() -> Self {
        Self::new(0.35, 20)
    }
}

pub struct NoveltyStats {
    pub current_score: f64,
    pub current_level: NoveltyLevel,
    pub mean_error: f64,
    pub variance: f64,
    pub trend: f64,
    pub total_novel: u64,
    pub total_predictions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let nd = NoveltyDetector::default();
        assert_eq!(nd.current_level, NoveltyLevel::Known);
        assert_eq!(nd.current_score, 0.0);
        assert_eq!(nd.total_novel_events, 0);
        assert_eq!(nd.total_predictions, 0);
    }

    #[test]
    fn test_low_error_stays_known() {
        let mut nd = NoveltyDetector::default();
        nd.feed_prediction_error(0.1);
        nd.feed_prediction_error(0.05);
        nd.feed_prediction_error(0.08);
        assert_eq!(nd.current_level, NoveltyLevel::Known);
    }

    #[test]
    fn test_high_error_becomes_novel() {
        let mut nd = NoveltyDetector::default();
        nd.feed_prediction_error(0.9);
        assert_eq!(nd.current_level, NoveltyLevel::Novel);
    }

    #[test]
    fn test_mean_error() {
        let mut nd = NoveltyDetector::default();
        nd.feed_prediction_error(0.2);
        nd.feed_prediction_error(0.4);
        nd.feed_prediction_error(0.6);
        let mean = nd.mean_error();
        assert!((mean - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_window_bounded() {
        let mut nd = NoveltyDetector::new(0.35, 20);
        for i in 0..30 {
            nd.feed_prediction_error(0.1 * (i as f64 % 10.0));
        }
        assert_eq!(nd.window.len(), 20);
    }

    #[test]
    fn test_stats() {
        let mut nd = NoveltyDetector::default();
        nd.feed_prediction_error(0.1);
        nd.feed_prediction_error(0.2);
        nd.feed_prediction_error(0.9);
        let stats = nd.stats();
        assert_eq!(stats.total_predictions, 3);
        assert!(stats.variance >= 0.0);
    }

    #[test]
    fn test_reset_baseline() {
        let mut nd = NoveltyDetector::default();
        nd.feed_prediction_error(0.3);
        nd.feed_prediction_error(0.4);
        nd.feed_prediction_error(0.5);
        let old = nd.baseline_threshold;
        nd.reset_baseline();
        assert!((nd.baseline_threshold - 2.0 * 0.4).abs() < 1e-6);
        assert!(nd.baseline_threshold != old);
    }

    #[test]
    fn test_trend_positive() {
        let mut nd = NoveltyDetector::default();
        for i in 0..10 {
            nd.feed_prediction_error(0.05 * (i + 1) as f64);
        }
        assert!(nd.trend() > 0.0);
    }

    #[test]
    fn test_trend_negative() {
        let mut nd = NoveltyDetector::default();
        for i in 0..10 {
            nd.feed_prediction_error(1.0 - 0.05 * (i + 1) as f64);
        }
        assert!(nd.trend() < 0.0);
    }

    #[test]
    fn test_curiosity_bonus_levels() {
        assert_eq!(NoveltyLevel::Known.curiosity_bonus(), 0.0);
        assert_eq!(NoveltyLevel::NearKnown.curiosity_bonus(), 0.3);
        assert_eq!(NoveltyLevel::Ambiguous.curiosity_bonus(), 0.6);
        assert_eq!(NoveltyLevel::Novel.curiosity_bonus(), 1.0);
    }

    #[test]
    fn test_error_clamped() {
        let mut nd = NoveltyDetector::default();
        nd.feed_prediction_error(-0.5);
        assert!((nd.current_score - 0.0).abs() < 1e-6);
        nd.feed_prediction_error(1.5);
        assert!((nd.current_score - 1.0).abs() > 1e-6 || nd.current_level == NoveltyLevel::Novel);
    }
}
