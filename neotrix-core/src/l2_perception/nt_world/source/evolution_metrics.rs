use std::collections::HashMap;

pub struct EvolutionMetrics {
    pub search_success_rate: f64,
    pub avg_latency_ms: f64,
    pub cache_hit_rate: f64,
    pub source_health: HashMap<String, f64>,
}

impl EvolutionMetrics {
    pub fn new() -> Self {
        Self {
            search_success_rate: 1.0,
            avg_latency_ms: 0.0,
            cache_hit_rate: 0.0,
            source_health: HashMap::new(),
        }
    }
    pub fn _update_source_health(&mut self, source: &str, healthy: bool) {
        let entry = self.source_health.entry(source.to_string()).or_insert(1.0);
        *entry = if healthy { (*entry + 1.0) / 2.0 } else { *entry * 0.5 };
    }
}
