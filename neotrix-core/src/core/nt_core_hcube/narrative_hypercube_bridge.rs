use crate::core::nt_core_consciousness::narrative_self::NarrativeEvent;
use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::nt_core_hcube::cube::{CubeEntry, KnowledgeHyperCube};
use crate::core::nt_core_hcube::vsa_vector::bytes_to_vsa_vector;
use crate::core::nt_core_hcube::QuantizedVSA;
use std::collections::VecDeque;

const BRIDGE_MAX_LOOKBACK: usize = 200;

#[derive(Debug, Clone)]
pub struct NarrativeHyperCubeResult {
    pub key: String,
    pub summary: String,
    pub session_id: String,
    pub coord_similarity: f64,
    pub vsa_similarity: f64,
}

pub struct NarrativeHyperCubeBridge {
    pub hypercube: KnowledgeHyperCube,
    pub recent_keys: VecDeque<String>,
    pub max_lookback: usize,
}

impl NarrativeHyperCubeBridge {
    pub fn new(hypercube: KnowledgeHyperCube) -> Self {
        Self {
            hypercube,
            recent_keys: VecDeque::with_capacity(BRIDGE_MAX_LOOKBACK),
            max_lookback: BRIDGE_MAX_LOOKBACK,
        }
    }

    pub fn store_event(&mut self, event: &NarrativeEvent) -> String {
        let key = format!("narrative-{}-{}", event.session_id, event.timestamp);

        let time_val = (event.timestamp as f64 % 1_000_000.0) / 1_000_000.0;
        let mut coord = HyperCoord::new();
        coord.set(DimensionAxis::Time, time_val);
        coord.set(DimensionAxis::Agency, event.reward.clamp(0.0, 1.0));
        coord.set(DimensionAxis::Certainty, 0.5);

        let vsa = event
            .vsa_fingerprint
            .as_ref()
            .and_then(|fp| {
                let seed: u64 = event
                    .summary
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
                if fp.len() == 4096 {
                    bytes_to_vsa_vector(fp.clone()).ok()
                } else {
                    let bytes = QuantizedVSA::seeded_random(seed, 4096);
                    bytes_to_vsa_vector(bytes).ok()
                }
            })
            .unwrap_or_else(|| {
                let seed: u64 = event
                    .summary
                    .bytes()
                    .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
                let bytes = QuantizedVSA::seeded_random(seed, 4096);
                bytes_to_vsa_vector(bytes).unwrap_or_else(|_| crate::core::nt_core_hcube::VsaVector::default())
            });

        let entry = CubeEntry {
            key: key.clone(),
            coord: coord.clone(),
            value: event.reward,
            label: event.summary.clone(),
            source: "narrative_self".to_string(),
            access_count: 0,
            task_type: None,
            vsa: None,
        };

        self.hypercube.insert_with_vsa(entry, vsa);

        self.recent_keys.push_back(key.clone());
        if self.recent_keys.len() > self.max_lookback {
            self.recent_keys.pop_front();
        }

        key
    }

    pub fn search_similar(
        &mut self,
        event: &NarrativeEvent,
        top_k: usize,
    ) -> Vec<NarrativeHyperCubeResult> {
        let time_val = (event.timestamp as f64 % 1_000_000.0) / 1_000_000.0;
        let mut coord = HyperCoord::new();
        coord.set(DimensionAxis::Time, time_val);
        coord.set(DimensionAxis::Agency, event.reward.clamp(0.0, 1.0));

        let seed: u64 = event
            .summary
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let bytes = QuantizedVSA::seeded_random(seed, 4096);
        let query_vsa = bytes_to_vsa_vector(bytes).ok();

        if let Some(qvsa) = query_vsa {
            let fused = self.hypercube.search_multi_modal(&coord, &qvsa, 0.6, top_k);
            fused
                .into_iter()
                .map(|(score, entry)| NarrativeHyperCubeResult {
                    key: entry.key.clone(),
                    summary: entry.label.clone(),
                    session_id: entry.source.clone(),
                    coord_similarity: entry.coord.cosine_similarity(&coord),
                    vsa_similarity: score,
                })
                .collect()
        } else {
            let results = self.hypercube.query(&coord, top_k);
            let coord_clone = coord;
            results
                .into_iter()
                .map(|entry| NarrativeHyperCubeResult {
                    key: entry.key.clone(),
                    summary: entry.label.clone(),
                    session_id: entry.source.clone(),
                    coord_similarity: entry.coord.cosine_similarity(&coord_clone),
                    vsa_similarity: 0.0,
                })
                .collect()
        }
    }

    pub fn get_similarity(&self, a: &NarrativeEvent, b: &NarrativeEvent) -> f64 {
        let fp_a = a.vsa_fingerprint.as_ref();
        let fp_b = b.vsa_fingerprint.as_ref();
        match (fp_a, fp_b) {
            (Some(fa), Some(fb)) => {
                let _max_len = fa.len().max(fb.len());
                let sim = QuantizedVSA::similarity(fa, fb);
                let time_diff = if a.timestamp > b.timestamp {
                    a.timestamp - b.timestamp
                } else {
                    b.timestamp - a.timestamp
                };
                let time_factor = (1.0 - (time_diff as f64 / 86_400_000.0).min(1.0)) * 0.3;
                sim * 0.7 + time_factor
            }
            _ => 0.0,
        }
    }

    pub fn recent_narratives(&self, n: usize) -> Vec<String> {
        self.recent_keys.iter().rev().take(n).cloned().collect()
    }

    pub fn cross_session_themes(&mut self, top_k: usize) -> Vec<(String, f64)> {
        let mut theme_scores: std::collections::HashMap<String, (f64, u32)> =
            std::collections::HashMap::new();

        for key in &self.recent_keys {
            if let Some(entry) = self.hypercube.get_entry(key) {
                let theme = entry
                    .label
                    .split_whitespace()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" ");
                let acc = theme_scores.entry(theme).or_insert((0.0, 0));
                acc.0 += entry.value;
                acc.1 += 1;
            }
        }

        let mut scored: Vec<(String, f64)> = theme_scores
            .into_iter()
            .map(|(theme, (score, count))| (theme, score / count as f64))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    pub fn into_hypercube(self) -> KnowledgeHyperCube {
        self.hypercube
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::narrative_self::NarrativeEvent;

    fn sample_event(session: &str, ts: u64, summary: &str) -> NarrativeEvent {
        NarrativeEvent {
            session_id: session.to_string(),
            timestamp: ts,
            summary: summary.to_string(),
            reward: 0.5,
            duration_ms: 1000,
            key_insights: vec!["test".to_string()],
            vsa_fingerprint: None,
        }
    }

    #[test]
    fn test_store_event_creates_entry() {
        let hypercube = KnowledgeHyperCube::new();
        let mut bridge = NarrativeHyperCubeBridge::new(hypercube);
        let event = sample_event("s1", 1000, "test event");
        let key = bridge.store_event(&event);
        assert!(bridge.hypercube.get_entry(&key).is_some());
    }

    #[test]
    fn test_store_event_increments_count() {
        let hypercube = KnowledgeHyperCube::new();
        let mut bridge = NarrativeHyperCubeBridge::new(hypercube);
        let e1 = sample_event("s1", 1000, "first");
        let e2 = sample_event("s1", 2000, "second");
        bridge.store_event(&e1);
        bridge.store_event(&e2);
        assert_eq!(bridge.hypercube.len(), 2);
    }

    #[test]
    fn test_search_returns_results() {
        let hypercube = KnowledgeHyperCube::new();
        let mut bridge = NarrativeHyperCubeBridge::new(hypercube);
        let e1 = sample_event("s1", 1000, "alpha");
        let e2 = sample_event("s1", 2000, "beta");
        bridge.store_event(&e1);
        bridge.store_event(&e2);

        let query = sample_event("s2", 1500, "alpha-like");
        let results = bridge.search_similar(&query, 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_get_similarity_returns_value() {
        let hypercube = KnowledgeHyperCube::new();
        let bridge = NarrativeHyperCubeBridge::new(hypercube);
        let a = sample_event("s1", 1000, "same");
        let b = sample_event("s1", 1001, "same");
        let sim = bridge.get_similarity(&a, &b);
        assert!(sim >= 0.0);
        assert!(sim <= 1.0);
    }

    #[test]
    fn test_recent_narratives_returns_keys() {
        let hypercube = KnowledgeHyperCube::new();
        let mut bridge = NarrativeHyperCubeBridge::new(hypercube);
        let e1 = sample_event("s1", 1000, "a");
        let e2 = sample_event("s1", 2000, "b");
        let k1 = bridge.store_event(&e1);
        let k2 = bridge.store_event(&e2);

        let recent = bridge.recent_narratives(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0], k2);
    }

    #[test]
    fn test_cross_session_themes() {
        let hypercube = KnowledgeHyperCube::new();
        let mut bridge = NarrativeHyperCubeBridge::new(hypercube);
        let e1 = sample_event("s1", 1000, "important thing happened");
        let e2 = sample_event("s2", 2000, "important thing again");
        bridge.store_event(&e1);
        bridge.store_event(&e2);

        let themes = bridge.cross_session_themes(5);
        assert!(!themes.is_empty());
    }

    #[test]
    fn test_into_hypercube_recovers() {
        let hypercube = KnowledgeHyperCube::new();
        let mut bridge = NarrativeHyperCubeBridge::new(hypercube);
        let e1 = sample_event("s1", 1000, "test");
        bridge.store_event(&e1);
        let recovered = bridge.into_hypercube();
        assert_eq!(recovered.len(), 1);
    }
}
