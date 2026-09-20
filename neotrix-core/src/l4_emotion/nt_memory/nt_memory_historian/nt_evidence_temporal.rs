use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Temporal scope of an evidence claim
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemporalScope {
    /// Specific point in time
    Point(i64),
    /// Time range (start, end)
    Range(i64, i64),
    /// Periodic/recurring event (start, end, period_seconds)
    Periodic(i64, i64, i64),
    /// Eternal/always-true claim
    Eternal,
}

/// Temporal relation between two time intervals
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TemporalRelation {
    Before,
    After,
    During,
    Contains,
    Overlaps,
    Meets,
    MetBy,
    Starts,
    StartedBy,
    Finishes,
    FinishedBy,
    Equal,
}

impl TemporalRelation {
    /// Map to Allen's interval algebra label
    pub fn allen_label(&self) -> &str {
        match self {
            TemporalRelation::Before => "x < y",
            TemporalRelation::After => "x > y",
            TemporalRelation::During => "x d y",
            TemporalRelation::Contains => "x di y",
            TemporalRelation::Overlaps => "x o y",
            TemporalRelation::Meets => "x m y",
            TemporalRelation::MetBy => "x mi y",
            TemporalRelation::Starts => "x s y",
            TemporalRelation::StartedBy => "x si y",
            TemporalRelation::Finishes => "x f y",
            TemporalRelation::FinishedBy => "x fi y",
            TemporalRelation::Equal => "x = y",
        }
    }
}

/// Allen's interval algebra: compute relation between two time intervals [a_start, a_end] and [b_start, b_end]
pub fn allen_relation(a_start: i64, a_end: i64, b_start: i64, b_end: i64) -> TemporalRelation {
    if a_end < b_start {
        TemporalRelation::Before
    } else if a_start > b_end {
        TemporalRelation::After
    } else if a_start >= b_start && a_end <= b_end {
        if a_start == b_start && a_end == b_end {
            TemporalRelation::Equal
        } else if a_start == b_start {
            TemporalRelation::Starts
        } else if a_end == b_end {
            TemporalRelation::Finishes
        } else {
            TemporalRelation::During
        }
    } else if a_start <= b_start && a_end >= b_end {
        if a_start == b_start {
            TemporalRelation::StartedBy
        } else if a_end == b_end {
            TemporalRelation::FinishedBy
        } else {
            TemporalRelation::Contains
        }
    } else if a_end == b_start {
        TemporalRelation::Meets
    } else if a_start == b_end {
        TemporalRelation::MetBy
    } else {
        TemporalRelation::Overlaps
    }
}

/// Anachronism detection: check if a claim references entities/timestamps that don't align with its stated period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnachronismDetector {
    pub known_timelines: HashMap<String, (i64, i64)>,
}

impl Default for AnachronismDetector {
    fn default() -> Self { Self::new() }
}

impl AnachronismDetector {
    pub fn new() -> Self {
        Self { known_timelines: HashMap::new() }
    }

    pub fn register_entity(&mut self, entity: &str, start: i64, end: i64) {
        self.known_timelines.insert(entity.to_string(), (start, end));
    }

    pub fn check_anachronism(&self, claim_timestamp: i64, entity: &str) -> Option<String> {
        self.known_timelines.get(entity).and_then(|(start, end)| {
            if claim_timestamp < *start {
                Some(format!(
                    "实体'{}'存在于{}至{}, 但被引用在{} (早于诞生)",
                    entity, start, end, claim_timestamp
                ))
            } else if claim_timestamp > *end {
                Some(format!(
                    "实体'{}'存在于{}至{}, 但被引用在{} (晚于消亡)",
                    entity, start, end, claim_timestamp
                ))
            } else {
                None
            }
        })
    }

    pub fn check_event_pair_consistency(&self, event_a: &str, a_time: i64, event_b: &str, b_time: i64) -> Option<String> {
        if let Some(&(a_start, a_end)) = self.known_timelines.get(event_a) {
            if a_time < a_start || a_time > a_end {
                return Some(format!("'{}'的引用时间{}不在其存在区间[{},{}]内", event_a, a_time, a_start, a_end));
            }
        }
        if let Some(&(b_start, b_end)) = self.known_timelines.get(event_b) {
            if b_time < b_start || b_time > b_end {
                return Some(format!("'{}'的引用时间{}不在其存在区间[{},{}]内", event_b, b_time, b_start, b_end));
            }
        }
        // Event A must precede event B
        if a_time > b_time {
            return Some(format!(
                "时间矛盾：'{}'({}) 晚于 '{}'({})",
                event_a, a_time, event_b, b_time
            ));
        }
        None
    }
}

/// Temporal reasoning: track changes in evidence claims over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEvidenceTracker {
    /// evidence_id -> Vec of (timestamp, value) pairs
    pub timelines: HashMap<String, Vec<(i64, f64)>>,
    pub revision_count: HashMap<String, usize>,
}

impl Default for TemporalEvidenceTracker {
    fn default() -> Self { Self::new() }
}

impl TemporalEvidenceTracker {
    pub fn new() -> Self {
        Self { timelines: HashMap::new(), revision_count: HashMap::new() }
    }

    pub fn record_value(&mut self, evidence_id: &str, timestamp: i64, value: f64) {
        let entry = self.timelines.entry(evidence_id.to_string()).or_default();
        entry.push((timestamp, value));
        *self.revision_count.entry(evidence_id.to_string()).or_insert(0) += 1;
    }

    pub fn trend(&self, evidence_id: &str) -> Option<TemporalTrend> {
        self.timelines.get(evidence_id).map(|points| {
            if points.len() < 2 {
                TemporalTrend {
                    direction: TrendDirection::Stable,
                    slope: 0.0,
                    volatility: 0.0,
                    recent_trend: TrendDirection::Stable,
                    n_points: points.len(),
                }
            } else {
                let sorted = {
                    let mut s = points.clone();
                    s.sort_by_key(|(t, _)| *t);
                    s
                };
                let values: Vec<f64> = sorted.iter().map(|(_, v)| *v).collect();
                let times: Vec<i64> = sorted.iter().map(|(t, _)| *t).collect();
                let n = values.len();
                let mean_t = times.iter().sum::<i64>() as f64 / n as f64;
                let mean_v = values.iter().sum::<f64>() / n as f64;
                let mut num = 0.0;
                let mut den = 0.0;
                for i in 0..n {
                    let dt = times[i] as f64 - mean_t;
                    let dv = values[i] - mean_v;
                    num += dt * dv;
                    den += dt * dt;
                }
                let slope = if den > 0.0 { num / den } else { 0.0 };
                let mut variance = 0.0;
                for v in &values {
                    variance += (v - mean_v).powi(2);
                }
                let volatility = (variance / n as f64).sqrt().max(0.0);
                let direction = if slope.abs() < 0.001 {
                    TrendDirection::Stable
                } else if slope > 0.0 {
                    TrendDirection::Increasing
                } else {
                    TrendDirection::Decreasing
                };
                let recent = if values.len() >= 3 {
                    let last3 = &values[values.len() - 3..];
                    let recent_mean = last3.iter().sum::<f64>() / 3.0;
                    let earlier_mean = values[..values.len() - 3].iter().sum::<f64>()
                        / (values.len() - 3) as f64;
                    if (recent_mean - earlier_mean).abs() < 0.02 {
                        TrendDirection::Stable
                    } else if recent_mean > earlier_mean {
                        TrendDirection::Increasing
                    } else {
                        TrendDirection::Decreasing
                    }
                } else {
                    TrendDirection::Stable
                };
                TemporalTrend { direction, slope, volatility, recent_trend: recent, n_points: n }
            }
        })
    }

    pub fn evidence_consistency(&self, evidence_id: &str) -> Option<f64> {
        self.timelines.get(evidence_id).map(|points| {
            if points.len() < 3 {
                return 1.0;
            }
            let values: Vec<f64> = points.iter().map(|(_, v)| *v).collect();
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let mut max_dev = 0.0;
            for v in &values {
                let dev = (v - mean).abs();
                if dev > max_dev {
                    max_dev = dev;
                }
            }
            let std_est = (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                / values.len() as f64)
                .sqrt()
                .max(0.001);
            let consistency = 1.0 - (std_est / mean.max(0.01)).min(1.0);
            consistency.max(0.0).min(1.0)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalTrend {
    pub direction: TrendDirection,
    pub slope: f64,
    pub volatility: f64,
    pub recent_trend: TrendDirection,
    pub n_points: usize,
}

/// Timeline reconstruction: order events chronologically and detect gaps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineReconstructor {
    pub events: Vec<TimelineEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub description: String,
    pub timestamp: i64,
    pub evidence_ids: Vec<String>,
    pub confidence: f64,
}

impl Default for TimelineReconstructor {
    fn default() -> Self { Self::new() }
}

impl TimelineReconstructor {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn add_event(&mut self, id: &str, description: &str, timestamp: i64, evidence_ids: Vec<String>, confidence: f64) {
        self.events.push(TimelineEvent {
            id: id.to_string(),
            description: description.to_string(),
            timestamp,
            evidence_ids,
            confidence,
        });
    }

    pub fn sorted_events(&self) -> Vec<&TimelineEvent> {
        let mut sorted: Vec<&TimelineEvent> = self.events.iter().collect();
        sorted.sort_by_key(|e| e.timestamp);
        sorted
    }

    pub fn detect_gaps(&self) -> Vec<(i64, i64, i64)> {
        let sorted = self.sorted_events();
        if sorted.len() < 2 {
            return vec![];
        }
        let mut gaps = Vec::new();
        for pair in sorted.windows(2) {
            let gap = pair[1].timestamp - pair[0].timestamp;
            if gap > 86400 {
                gaps.push((pair[0].timestamp, pair[1].timestamp, gap));
            }
        }
        gaps
    }

    pub fn max_time_span(&self) -> Option<(i64, i64)> {
        if self.events.is_empty() {
            return None;
        }
        let min = self.events.iter().map(|e| e.timestamp).min().unwrap_or(0);
        let max = self.events.iter().map(|e| e.timestamp).max().unwrap_or(0);
        Some((min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allen_before() {
        let r = allen_relation(100, 200, 300, 400);
        assert_eq!(r, TemporalRelation::Before);
    }

    #[test]
    fn test_allen_after() {
        let r = allen_relation(300, 400, 100, 200);
        assert_eq!(r, TemporalRelation::After);
    }

    #[test]
    fn test_allen_during() {
        let r = allen_relation(150, 250, 100, 300);
        assert_eq!(r, TemporalRelation::During);
    }

    #[test]
    fn test_allen_contains() {
        let r = allen_relation(100, 300, 150, 250);
        assert_eq!(r, TemporalRelation::Contains);
    }

    #[test]
    fn test_allen_overlaps() {
        let r = allen_relation(100, 200, 150, 250);
        assert_eq!(r, TemporalRelation::Overlaps);
    }

    #[test]
    fn test_allen_meets() {
        let r = allen_relation(100, 200, 200, 300);
        assert_eq!(r, TemporalRelation::Meets);
    }

    #[test]
    fn test_allen_met_by() {
        let r = allen_relation(200, 300, 100, 200);
        assert_eq!(r, TemporalRelation::MetBy);
    }

    #[test]
    fn test_allen_starts() {
        let r = allen_relation(100, 200, 100, 300);
        assert_eq!(r, TemporalRelation::Starts);
    }

    #[test]
    fn test_allen_started_by() {
        let r = allen_relation(100, 300, 100, 200);
        assert_eq!(r, TemporalRelation::StartedBy);
    }

    #[test]
    fn test_allen_finishes() {
        let r = allen_relation(200, 300, 100, 300);
        assert_eq!(r, TemporalRelation::Finishes);
    }

    #[test]
    fn test_allen_finished_by() {
        let r = allen_relation(100, 300, 200, 300);
        assert_eq!(r, TemporalRelation::FinishedBy);
    }

    #[test]
    fn test_allen_equal() {
        let r = allen_relation(100, 200, 100, 200);
        assert_eq!(r, TemporalRelation::Equal);
    }

    #[test]
    fn test_anachronism_detection() {
        let mut detector = AnachronismDetector::new();
        detector.register_entity("iPhone", 20_070_101_i64, 20_260_101_i64);
        let result = detector.check_anachronism(19_900_101_i64, "iPhone");
        assert!(result.is_some());
        assert!(result.unwrap().contains("早于诞生"));
    }

    #[test]
    fn test_anachronism_no_error() {
        let mut detector = AnachronismDetector::new();
        detector.register_entity("iPhone", 20_070_101_i64, 20_260_101_i64);
        let result = detector.check_anachronism(20_230_101_i64, "iPhone");
        assert!(result.is_none());
    }

    #[test]
    fn test_temporal_tracker_trend() {
        let mut tracker = TemporalEvidenceTracker::new();
        for i in 0..10 {
            tracker.record_value("ev1", i, 0.5 + i as f64 * 0.05);
        }
        let trend = tracker.trend("ev1");
        assert!(trend.is_some());
        let t = trend.unwrap();
        assert!(t.slope > 0.0);
        assert_eq!(t.direction, TrendDirection::Increasing, "Expected increasing trend but got {:?}", t.direction);
    }

    #[test]
    fn test_temporal_tracker_consistency() {
        let mut tracker = TemporalEvidenceTracker::new();
        for i in 0..5 {
            tracker.record_value("stable", i, 0.8);
        }
        let consistent = tracker.evidence_consistency("stable");
        assert!(consistent.is_some());
        assert!(consistent.unwrap() > 0.9);
    }

    #[test]
    fn test_timeline_sorted() {
        let mut tl = TimelineReconstructor::new();
        tl.add_event("e1", "Event 1", 300, vec!["ev1".into()], 0.9);
        tl.add_event("e2", "Event 2", 100, vec!["ev2".into()], 0.8);
        tl.add_event("e3", "Event 3", 200, vec!["ev3".into()], 0.7);
        let sorted = tl.sorted_events();
        assert_eq!(sorted[0].id, "e2");
        assert_eq!(sorted[1].id, "e3");
        assert_eq!(sorted[2].id, "e1");
    }

    #[test]
    fn test_timeline_gap_detection() {
        let mut tl = TimelineReconstructor::new();
        tl.add_event("e1", "E1", 100, vec![], 0.5);
        tl.add_event("e2", "E2", 100_000, vec![], 0.5);
        let gaps = tl.detect_gaps();
        assert!(!gaps.is_empty());
    }

    #[test]
    fn test_event_pair_consistency() {
        let mut detector = AnachronismDetector::new();
        detector.register_entity("FrenchRevolution", 1789, 1799);
        detector.register_entity("WWII", 1939, 1945);
        // FrenchRevolution should be before WWII
        let result = detector.check_event_pair_consistency(
            "WorldWarII", 1940,
            "FrenchRevolution", 1790,
        );
        assert!(result.is_some());
        assert!(result.unwrap().contains("时间矛盾"));
    }

    #[test]
    fn test_tracker_no_data_returns_none() {
        let tracker = TemporalEvidenceTracker::new();
        assert!(tracker.trend("nonexistent").is_none());
    }
}
