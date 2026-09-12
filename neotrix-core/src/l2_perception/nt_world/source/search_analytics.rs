use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct _QueryRecord {
    pub query: String,
    pub results_count: usize,
    pub latency_ms: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct SearchAnalytics {
    queries: Vec<_QueryRecord>,
    max_records: usize,
}

impl SearchAnalytics {
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
            max_records: 10000,
        }
    }

    pub fn with_capacity(max_records: usize) -> Self {
        Self {
            queries: Vec::with_capacity(max_records),
            max_records,
        }
    }

    pub fn record(&mut self, query: String, results_count: usize, latency_ms: u64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.queries.push(_QueryRecord {
            query,
            results_count,
            latency_ms,
            timestamp,
        });

        if self.queries.len() > self.max_records {
            self.queries.drain(0..self.queries.len() - self.max_records);
        }
    }

    pub fn _slow_queries(&self, threshold_ms: u64) -> Vec<&_QueryRecord> {
        self.queries
            .iter()
            .filter(|r| r.latency_ms >= threshold_ms)
            .collect()
    }

    pub fn _top_queries(&self, limit: usize) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for record in &self.queries {
            *counts.entry(record.query.clone()).or_insert(0) += 1;
        }
        let mut pairs: Vec<(String, usize)> = counts.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.into_iter().take(limit).collect()
    }

    pub fn avg_latency(&self) -> f64 {
        if self.queries.is_empty() {
            return 0.0;
        }
        let total: u64 = self.queries.iter().map(|r| r.latency_ms).sum();
        total as f64 / self.queries.len() as f64
    }

    pub fn _zero_result_queries(&self) -> Vec<&_QueryRecord> {
        self.queries
            .iter()
            .filter(|r| r.results_count == 0)
            .collect()
    }

    pub fn total_queries(&self) -> usize {
        self.queries.len()
    }

    pub fn clear(&mut self) {
        self.queries.clear();
    }
}

impl Default for SearchAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_count() {
        let mut analytics = SearchAnalytics::new();
        analytics.record("test".into(), 5, 120);
        analytics.record("test".into(), 3, 200);
        analytics.record("other".into(), 0, 50);
        assert_eq!(analytics.total_queries(), 3);
    }

    #[test]
    fn test_slow_queries() {
        let mut analytics = SearchAnalytics::new();
        analytics.record("fast".into(), 5, 50);
        analytics.record("slow".into(), 5, 500);
        let slow = analytics._slow_queries(200);
        assert_eq!(slow.len(), 1);
        assert_eq!(slow[0].query, "slow");
    }

    #[test]
    fn test_top_queries() {
        let mut analytics = SearchAnalytics::new();
        for _ in 0..5 {
            analytics.record("popular".into(), 1, 10);
        }
        analytics.record("rare".into(), 1, 10);
        let top = analytics._top_queries(1);
        assert_eq!(top[0], ("popular".into(), 5));
    }

    #[test]
    fn test_avg_latency() {
        let mut analytics = SearchAnalytics::new();
        analytics.record("a".into(), 1, 100);
        analytics.record("b".into(), 1, 300);
        assert!((analytics.avg_latency() - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_result_queries() {
        let mut analytics = SearchAnalytics::new();
        analytics.record("a".into(), 0, 10);
        analytics.record("b".into(), 5, 10);
        assert_eq!(analytics._zero_result_queries().len(), 1);
    }

    #[test]
    fn test_max_records_eviction() {
        let mut analytics = SearchAnalytics::with_capacity(3);
        analytics.record("a".into(), 1, 10);
        analytics.record("b".into(), 1, 10);
        analytics.record("c".into(), 1, 10);
        analytics.record("d".into(), 1, 10);
        assert_eq!(analytics.total_queries(), 3);
    }
}
