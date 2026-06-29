use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum TemporalLayer {
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
}

impl TemporalLayer {
    pub fn retention_ms(&self) -> u64 {
        match self {
            TemporalLayer::Millisecond => 1_000,
            TemporalLayer::Second => 60_000,
            TemporalLayer::Minute => 3_600_000,
            TemporalLayer::Hour => 86_400_000,
            TemporalLayer::Day => u64::MAX,
        }
    }

    pub fn capacity(&self) -> usize {
        match self {
            TemporalLayer::Millisecond => 100,
            TemporalLayer::Second => 200,
            TemporalLayer::Minute => 150,
            TemporalLayer::Hour => 100,
            TemporalLayer::Day => 500,
        }
    }

    pub fn next_layer(&self) -> Option<TemporalLayer> {
        match self {
            TemporalLayer::Millisecond => Some(TemporalLayer::Second),
            TemporalLayer::Second => Some(TemporalLayer::Minute),
            TemporalLayer::Minute => Some(TemporalLayer::Hour),
            TemporalLayer::Hour => Some(TemporalLayer::Day),
            TemporalLayer::Day => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeStampedEntry {
    pub timestamp: u64,
    pub data: Vec<u8>,
    pub context: Option<Vec<u8>>,
    pub access_count: u32,
    pub layer: TemporalLayer,
}

#[derive(Debug, Clone)]
pub struct LayerStore {
    pub entries: Vec<TimeStampedEntry>,
    pub capacity: usize,
    pub retention_ms: u64,
}

impl LayerStore {
    pub fn new(capacity: usize, retention_ms: u64) -> Self {
        LayerStore {
            entries: Vec::with_capacity(capacity),
            capacity,
            retention_ms,
        }
    }

    pub fn push(&mut self, entry: TimeStampedEntry) {
        if self.entries.len() >= self.capacity {
            let oldest_idx = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.timestamp)
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.entries.remove(oldest_idx);
        }
        self.entries.push(entry);
    }

    pub fn query(&self, query_vec: &[u8], k: usize) -> Vec<&TimeStampedEntry> {
        let mut scored: Vec<(&TimeStampedEntry, f64)> = self
            .entries
            .iter()
            .map(|e| (e, QuantizedVSA::similarity(query_vec, &e.data)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored.into_iter().map(|(e, _)| e).collect()
    }

    pub fn query_temporal(
        &self,
        query: &[u8],
        start_time: u64,
        end_time: u64,
        k: usize,
    ) -> Vec<&TimeStampedEntry> {
        let mut scored: Vec<(&TimeStampedEntry, f64)> = self
            .entries
            .iter()
            .filter(|e| e.timestamp >= start_time && e.timestamp <= end_time)
            .map(|e| (e, QuantizedVSA::similarity(query, &e.data)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored.into_iter().map(|(e, _)| e).collect()
    }

    pub fn consolidate(&self, _target_layer: TemporalLayer) -> Option<Vec<u8>> {
        let window = self.retention_ms / 2;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let recent: Vec<&[u8]> = self
            .entries
            .iter()
            .filter(|e| e.timestamp + window >= now)
            .map(|e| e.data.as_slice())
            .collect();
        if recent.is_empty() {
            return None;
        }
        Some(QuantizedVSA::bundle(&recent))
    }

    pub fn prune_expired(&mut self, current_time: u64) {
        if self.retention_ms == u64::MAX {
            return;
        }
        self.entries
            .retain(|e| e.timestamp + self.retention_ms >= current_time);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub struct TemporalMemory {
    pub layers: HashMap<TemporalLayer, LayerStore>,
}

impl TemporalMemory {
    pub fn new() -> Self {
        let layers: HashMap<TemporalLayer, LayerStore> = vec![
            (
                TemporalLayer::Millisecond,
                LayerStore::new(
                    TemporalLayer::Millisecond.capacity(),
                    TemporalLayer::Millisecond.retention_ms(),
                ),
            ),
            (
                TemporalLayer::Second,
                LayerStore::new(
                    TemporalLayer::Second.capacity(),
                    TemporalLayer::Second.retention_ms(),
                ),
            ),
            (
                TemporalLayer::Minute,
                LayerStore::new(
                    TemporalLayer::Minute.capacity(),
                    TemporalLayer::Minute.retention_ms(),
                ),
            ),
            (
                TemporalLayer::Hour,
                LayerStore::new(
                    TemporalLayer::Hour.capacity(),
                    TemporalLayer::Hour.retention_ms(),
                ),
            ),
            (
                TemporalLayer::Day,
                LayerStore::new(
                    TemporalLayer::Day.capacity(),
                    TemporalLayer::Day.retention_ms(),
                ),
            ),
        ]
        .into_iter()
        .collect();
        TemporalMemory { layers }
    }

    pub fn store(&mut self, data: Vec<u8>, layer: TemporalLayer, context: Option<Vec<u8>>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let entry = TimeStampedEntry {
            timestamp: now,
            data,
            context,
            access_count: 0,
            layer: layer.clone(),
        };
        if let Some(store) = self.layers.get_mut(&layer) {
            store.push(entry);
        }
    }

    pub fn recall(&self, query: &[u8], layer: TemporalLayer, k: usize) -> Vec<&TimeStampedEntry> {
        self.layers
            .get(&layer)
            .map(|store| store.query(query, k))
            .unwrap_or_default()
    }

    pub fn recall_cross_layer(&self, query: &[u8], k: usize) -> Vec<&TimeStampedEntry> {
        let mut all: Vec<(&TimeStampedEntry, f64)> = Vec::new();
        for store in self.layers.values() {
            for entry in &store.entries {
                let sim = QuantizedVSA::similarity(query, &entry.data);
                all.push((entry, sim));
            }
        }
        all.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        all.truncate(k);
        all.into_iter().map(|(e, _)| e).collect()
    }

    pub fn cascade(&mut self, current_time: u64) {
        let layers_order = [
            TemporalLayer::Millisecond,
            TemporalLayer::Second,
            TemporalLayer::Minute,
            TemporalLayer::Hour,
            TemporalLayer::Day,
        ];
        for layer in &layers_order {
            if let Some(next) = layer.next_layer() {
                if let Some(store) = self.layers.get(layer) {
                    if let Some(summary) = store.consolidate(next.clone()) {
                        let entry = TimeStampedEntry {
                            timestamp: current_time,
                            data: summary,
                            context: None,
                            access_count: 0,
                            layer: next.clone(),
                        };
                        if let Some(next_store) = self.layers.get_mut(&next) {
                            next_store.push(entry);
                        }
                    }
                }
            }
        }
    }

    pub fn prune_all(&mut self, current_time: u64) {
        for store in self.layers.values_mut() {
            store.prune_expired(current_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn random_vsa() -> Vec<u8> {
        QuantizedVSA::random_binary()
    }

    fn entry_at_time(data: Vec<u8>, time: u64, layer: TemporalLayer) -> TimeStampedEntry {
        TimeStampedEntry {
            timestamp: time,
            data,
            context: None,
            access_count: 0,
            layer,
        }
    }

    fn entry_with_context(
        data: Vec<u8>,
        time: u64,
        context: Vec<u8>,
        layer: TemporalLayer,
    ) -> TimeStampedEntry {
        TimeStampedEntry {
            timestamp: time,
            data,
            context: Some(context),
            access_count: 0,
            layer,
        }
    }

    #[test]
    fn test_store_and_recall_within_layer() {
        let mut tm = TemporalMemory::new();
        let v1 = random_vsa();
        let v2 = random_vsa();
        let v3 = random_vsa();

        tm.store(v1.clone(), TemporalLayer::Minute, None);
        tm.store(v2.clone(), TemporalLayer::Minute, None);
        tm.store(v3.clone(), TemporalLayer::Minute, None);

        // recall with v1 as query — v1 should be nearest to itself
        let results = tm.recall(&v1, TemporalLayer::Minute, 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].data, v1);
    }

    #[test]
    fn test_cross_layer_recall() {
        let mut tm = TemporalMemory::new();
        let v1 = random_vsa();
        let v2 = random_vsa();
        let v3 = random_vsa();

        tm.store(v1.clone(), TemporalLayer::Millisecond, None);
        tm.store(v2.clone(), TemporalLayer::Second, None);
        tm.store(v3.clone(), TemporalLayer::Hour, None);

        let results = tm.recall_cross_layer(&v1, 3);
        assert_eq!(results.len(), 3);
        // v1 stored in Millisecond should be top result
        assert_eq!(results[0].data, v1);
    }

    #[test]
    fn test_temporal_query() {
        let layer = TemporalLayer::Second;
        let store = LayerStore::new(100, 60_000);

        let v1 = random_vsa();
        let v2 = random_vsa();
        let v3 = random_vsa();

        let mut s = store;
        s.push(entry_at_time(v1.clone(), 1000, layer.clone()));
        s.push(entry_at_time(v2.clone(), 2000, layer.clone()));
        s.push(entry_at_time(v3.clone(), 3000, layer.clone()));

        // query within time range [1500, 2500] should only return v2
        let results = s.query_temporal(&v2, 1500, 2500, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].timestamp, 2000);
    }

    #[test]
    fn test_cascade_consolidation() {
        let mut tm = TemporalMemory::new();
        let now = 1_000_000u64;

        // fill millisecond layer
        let v1 = random_vsa();
        tm.store(v1, TemporalLayer::Millisecond, None);

        // cascade: millisecond → second
        tm.cascade(now);

        // second layer should now have one consolidated entry
        let second_entries = tm.layers.get(&TemporalLayer::Second).unwrap().len();
        assert_eq!(second_entries, 1);
    }

    #[test]
    fn test_prune_expired() {
        let mut store = LayerStore::new(10, 1000);
        let v = random_vsa();
        store.push(entry_at_time(v, 0, TemporalLayer::Millisecond));

        // current time far in future → entry expired
        store.prune_expired(5000);
        assert!(store.is_empty());
    }

    #[test]
    fn test_prune_keeps_recent() {
        let mut store = LayerStore::new(10, 1000);
        let v = random_vsa();
        store.push(entry_at_time(v.clone(), 4900, TemporalLayer::Millisecond));

        store.prune_expired(5000);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_capacity_eviction_oldest_first() {
        let mut store = LayerStore::new(3, 100_000);
        let v1 = random_vsa();
        let v2 = random_vsa();
        let v3 = random_vsa();
        let v4 = random_vsa();

        store.push(entry_at_time(v1.clone(), 100, TemporalLayer::Millisecond));
        store.push(entry_at_time(v2.clone(), 200, TemporalLayer::Millisecond));
        store.push(entry_at_time(v3.clone(), 300, TemporalLayer::Millisecond));
        // push 4th → oldest (timestamp 100) should be evicted
        store.push(entry_at_time(v4.clone(), 400, TemporalLayer::Millisecond));

        assert_eq!(store.len(), 3);
        // oldest remaining should be timestamp 200
        let min_ts = store.entries.iter().map(|e| e.timestamp).min().unwrap();
        assert_eq!(min_ts, 200);
    }

    #[test]
    fn test_empty_store_edge_case() {
        let store: LayerStore = LayerStore::new(10, 1000);
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        let v = random_vsa();
        let results = store.query(&v, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_single_entry_edge_case() {
        let mut tm = TemporalMemory::new();
        let v = random_vsa();
        tm.store(v.clone(), TemporalLayer::Hour, None);

        let results = tm.recall(&v, TemporalLayer::Hour, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].data, v);

        let cross = tm.recall_cross_layer(&v, 5);
        assert_eq!(cross.len(), 1);
    }

    #[test]
    fn test_context_filtering() {
        let layer = TemporalLayer::Hour;
        let store = LayerStore::new(10, 86_400_000);
        let ctx_a = random_vsa();
        let ctx_b = random_vsa();

        let v1 = random_vsa();
        let v2 = random_vsa();
        let v3 = random_vsa();

        let mut s = store;
        s.push(entry_with_context(
            v1.clone(),
            1000,
            ctx_a.clone(),
            layer.clone(),
        ));
        s.push(entry_with_context(
            v2.clone(),
            2000,
            ctx_b.clone(),
            layer.clone(),
        ));
        s.push(entry_with_context(
            v3.clone(),
            3000,
            ctx_a.clone(),
            layer.clone(),
        ));

        // query should return top-k across all, context is metadata only
        let results = s.query(&v1, 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].data, v1);
    }

    #[test]
    fn test_recall_ordering_by_similarity() {
        let layer = TemporalLayer::Day;
        let store = LayerStore::new(10, u64::MAX);
        let anchor = random_vsa();
        let close = QuantizedVSA::bind(&anchor, &QuantizedVSA::random_binary());
        let far = QuantizedVSA::random_binary();

        let mut s = store;
        s.push(entry_at_time(close.clone(), 100, layer.clone()));
        s.push(entry_at_time(far.clone(), 200, layer.clone()));

        let results = s.query(&anchor, 2);
        assert_eq!(results.len(), 2);
        // close (bound from anchor) should be more similar than random far
        let sim_close = QuantizedVSA::similarity(&anchor, &results[0].data);
        let sim_far = QuantizedVSA::similarity(&anchor, &results[1].data);
        assert!(sim_close >= sim_far);
    }

    #[test]
    fn test_prune_all_removes_expired() {
        let mut tm = TemporalMemory::new();
        let now = 5_000u64;

        // store in second layer (retention 60s)
        tm.store(random_vsa(), TemporalLayer::Second, None);

        // force timestamp to be old by directly pushing
        let v = random_vsa();
        let old_entry = TimeStampedEntry {
            timestamp: 0,
            data: v,
            context: None,
            access_count: 0,
            layer: TemporalLayer::Second,
        };
        tm.layers
            .get_mut(&TemporalLayer::Second)
            .unwrap()
            .push(old_entry);

        tm.prune_all(now);

        // second layer should still have the recent entry (but not the old one)
        let second = tm.layers.get(&TemporalLayer::Second).unwrap();
        assert_eq!(second.len(), 1);
        assert!(
            second.entries[0].timestamp > 0
                || second.entries[0].timestamp + second.retention_ms >= now
        );
    }

    #[test]
    fn test_day_layer_never_expires() {
        let mut store = LayerStore::new(10, u64::MAX);
        let v = random_vsa();
        store.push(entry_at_time(v, 0, TemporalLayer::Day));

        store.prune_expired(u64::MAX);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_temporal_memory_new_initializes_all_layers() {
        let tm = TemporalMemory::new();
        assert_eq!(tm.layers.len(), 5);
        assert!(tm.layers.contains_key(&TemporalLayer::Millisecond));
        assert!(tm.layers.contains_key(&TemporalLayer::Second));
        assert!(tm.layers.contains_key(&TemporalLayer::Minute));
        assert!(tm.layers.contains_key(&TemporalLayer::Hour));
        assert!(tm.layers.contains_key(&TemporalLayer::Day));
    }
}
