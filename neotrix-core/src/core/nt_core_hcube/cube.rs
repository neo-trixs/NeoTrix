use super::coord::HyperCoord;
use super::vsa_vector::VsaBackend;
use super::MapVsaBackend;
use super::VsaVector;
use crate::core::nt_core_shared_types::TaskType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CubeEntry {
    pub key: String,
    pub coord: HyperCoord,
    pub value: f64,
    pub label: String,
    pub source: String,
    pub access_count: u64,
    pub task_type: Option<TaskType>,
    pub vsa: Option<VsaVector<4096>>,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub entry: CubeEntry,
    pub distance: f64,
    pub similarity: f64,
}

pub struct KnowledgeHyperCube {
    entries: HashMap<String, CubeEntry>,
}

impl Default for KnowledgeHyperCube {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeHyperCube {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, coord: &HyperCoord, source: &str, title: &str) {
        let key = format!("{}-{}", source, title);
        self.entries.insert(
            key.clone(),
            CubeEntry {
                key,
                coord: coord.clone(),
                value: 0.0,
                label: title.to_string(),
                source: source.to_string(),
                access_count: 0,
                task_type: None,
                vsa: None,
            },
        );
    }

    pub fn insert_with_task_type(
        &mut self,
        coord: &HyperCoord,
        source: &str,
        title: &str,
        task_type: TaskType,
    ) {
        let key = format!("{}-{}", source, title);
        self.entries.insert(
            key.clone(),
            CubeEntry {
                key,
                coord: coord.clone(),
                value: 0.0,
                label: title.to_string(),
                source: source.to_string(),
                access_count: 0,
                task_type: Some(task_type),
                vsa: None,
            },
        );
    }

    pub fn query(&self, coord: &HyperCoord, top_k: usize) -> Vec<&CubeEntry> {
        if self.entries.is_empty() || top_k == 0 {
            return Vec::new();
        }
        let coord_dense = coord.to_dense();
        let mut scored: Vec<(&str, f64)> = self
            .entries
            .keys()
            .map(|k| {
                let entry = &self.entries[k];
                let entry_dense = entry.coord.to_dense();
                let dist = euclidean_16d(&coord_dense, &entry_dense);
                (k.as_str(), dist)
            })
            .collect();
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        let results: Vec<&CubeEntry> = scored.into_iter().map(|(k, _)| &self.entries[k]).collect();
        results
    }

    pub fn query_with_scores(&mut self, coord: &HyperCoord, top_k: usize) -> Vec<QueryResult> {
        if self.entries.is_empty() || top_k == 0 {
            return Vec::new();
        }
        let coord_dense = coord.to_dense();
        let mut scored: Vec<QueryResult> = self
            .entries
            .values()
            .map(|entry| {
                let entry_dense = entry.coord.to_dense();
                let distance = euclidean_16d(&coord_dense, &entry_dense);
                let similarity = cosine_16d(&coord_dense, &entry_dense);
                QueryResult {
                    entry: entry.clone(),
                    distance,
                    similarity,
                }
            })
            .collect();
        scored.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top_k);
        for qr in &mut scored {
            qr.entry.access_count += 1;
        }
        scored
    }

    pub fn query_by_task_type(
        &self,
        coord: &HyperCoord,
        task_type: TaskType,
        top_k: usize,
    ) -> Vec<&CubeEntry> {
        if self.entries.is_empty() || top_k == 0 {
            return Vec::new();
        }
        let coord_dense = coord.to_dense();
        let mut scored: Vec<(&str, f64)> = self
            .entries
            .keys()
            .filter_map(|k| {
                let entry = &self.entries[k];
                if entry.task_type != Some(task_type) {
                    return None;
                }
                let entry_dense = entry.coord.to_dense();
                let dist = euclidean_16d(&coord_dense, &entry_dense);
                Some((k.as_str(), dist))
            })
            .collect();
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored.into_iter().map(|(k, _)| &self.entries[k]).collect()
    }

    pub fn insert_with_value(&mut self, coord: &HyperCoord, source: &str, title: &str, value: f64) {
        let key = format!("{}-{}", source, title);
        self.entries.insert(
            key.clone(),
            CubeEntry {
                key,
                coord: coord.clone(),
                value,
                label: title.to_string(),
                source: source.to_string(),
                access_count: 0,
                task_type: None,
                vsa: None,
            },
        );
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<CubeEntry> {
        self.entries.remove(key)
    }

    pub fn update_value(&mut self, key: &str, value: f64) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.value = value;
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn cell_count(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> impl Iterator<Item = &CubeEntry> {
        self.entries.values()
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = &mut CubeEntry> {
        self.entries.values_mut()
    }

    pub fn get_entry(&self, key: &str) -> Option<&CubeEntry> {
        self.entries.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut CubeEntry> {
        self.entries.get_mut(key)
    }

    pub fn coord_density(&self, _dim: usize) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        let total: f64 = self.entries.values().map(|e| e.value).sum();
        (total / self.entries.len() as f64).max(0.0).min(1.0)
    }

    pub fn prune_low_access(&mut self, min_access: u64) -> usize {
        let before = self.entries.len();
        self.entries.retain(|_, e| e.access_count >= min_access);
        before - self.entries.len()
    }

    pub fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    pub fn insert_with_vsa(&mut self, entry: CubeEntry, vsa: VsaVector<4096>) {
        let mut entry = entry;
        entry.vsa = Some(vsa);
        self.entries.insert(entry.key.clone(), entry);
    }

    pub fn search_by_vsa(&self, query: &VsaVector<4096>, top_k: usize) -> Vec<(f64, &CubeEntry)> {
        if self.entries.is_empty() || top_k == 0 {
            return Vec::new();
        }
        let backend = MapVsaBackend;
        let mut scored: Vec<(f64, &CubeEntry)> = self
            .entries
            .values()
            .filter_map(|entry| {
                entry
                    .vsa
                    .as_ref()
                    .map(|vsa| (backend.similarity(query, vsa), entry))
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    pub fn search_multi_modal(
        &self,
        coord: &HyperCoord,
        vsa: &VsaVector<4096>,
        w: f64,
        top_k: usize,
    ) -> Vec<(f64, &CubeEntry)> {
        if self.entries.is_empty() || top_k == 0 {
            return Vec::new();
        }
        let backend = MapVsaBackend;
        let coord_dense = coord.to_dense();
        let mut scored: Vec<(f64, &CubeEntry)> = self
            .entries
            .values()
            .filter_map(|entry| {
                entry.vsa.as_ref().map(|entry_vsa| {
                    let vsa_sim = backend.similarity(vsa, entry_vsa);
                    let entry_dense = entry.coord.to_dense();
                    let coord_dist = euclidean_16d(&coord_dense, &entry_dense);
                    let coord_sim = 1.0 / (1.0 + coord_dist);
                    let fused = w * vsa_sim + (1.0 - w) * coord_sim;
                    (fused, entry)
                })
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }
}

fn euclidean_16d(a: &[f64; 16], b: &[f64; 16]) -> f64 {
    let mut sum = 0.0_f64;
    for i in 0..16 {
        let d = a[i] - b[i];
        sum += d * d;
    }
    sum.sqrt()
}

fn cosine_16d(a: &[f64; 16], b: &[f64; 16]) -> f64 {
    let mut dot = 0.0_f64;
    let mut mag_a = 0.0_f64;
    let mut mag_b = 0.0_f64;
    for i in 0..16 {
        dot += a[i] * b[i];
        mag_a += a[i] * a[i];
        mag_b += b[i] * b[i];
    }
    let denom = mag_a.sqrt() * mag_b.sqrt();
    if denom < 1e-12 {
        0.0
    } else {
        dot / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::axis::DimensionAxis;
    use crate::core::nt_core_hcube::coord::HyperCoord;

    fn test_coord(x: f64) -> HyperCoord {
        let mut c = HyperCoord::new();
        c.set(DimensionAxis::Abstraction, x);
        c.set(DimensionAxis::Creativity, 1.0 - x);
        c
    }

    #[test]
    fn test_new_creates_empty_cube() {
        let cube = KnowledgeHyperCube::new();
        assert!(cube.is_empty());
        assert_eq!(cube.len(), 0);
    }

    #[test]
    fn test_insert_and_query() {
        let mut cube = KnowledgeHyperCube::new();
        let coord = test_coord(0.5);
        cube.insert(&coord, "test-source", "test-entry");
        assert_eq!(cube.len(), 1);
        let results = cube.query(&coord, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "test-entry");
    }

    #[test]
    fn test_query_returns_closest_first() {
        let mut cube = KnowledgeHyperCube::new();
        cube.insert(&test_coord(0.1), "s1", "far");
        cube.insert(&test_coord(0.9), "s2", "near");
        let query_coord = test_coord(0.95);
        let results = cube.query(&query_coord, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].label, "near", "closest entry should be first");
    }

    #[test]
    fn test_query_top_k_respected() {
        let mut cube = KnowledgeHyperCube::new();
        cube.insert(&test_coord(0.1), "s1", "a");
        cube.insert(&test_coord(0.5), "s2", "b");
        cube.insert(&test_coord(0.9), "s3", "c");
        let results = cube.query(&test_coord(0.5), 2);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_empty_returns_empty() {
        let cube = KnowledgeHyperCube::new();
        let results = cube.query(&HyperCoord::new(), 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_insert_with_value_stores_value() {
        let mut cube = KnowledgeHyperCube::new();
        cube.insert_with_value(&test_coord(0.5), "src", "t", 0.75);
        let entry = cube.get_entry("src-t").unwrap();
        assert!((entry.value - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_prune_low_access() {
        let mut cube = KnowledgeHyperCube::new();
        cube.insert(&test_coord(0.1), "s1", "a");
        cube.insert(&test_coord(0.5), "s2", "b");
        {
            let entry = cube.get_entry_mut("s2-b").unwrap();
            entry.access_count = 5;
        }
        let removed = cube.prune_low_access(3);
        assert_eq!(removed, 1);
        assert!(cube.get_entry("s1-a").is_none());
        assert!(cube.get_entry("s2-b").is_some());
    }

    #[test]
    fn test_coord_density_empty() {
        let cube = KnowledgeHyperCube::new();
        assert_eq!(cube.coord_density(0), 0.0);
    }

    #[test]
    fn test_query_with_scores_includes_distance() {
        let mut cube = KnowledgeHyperCube::new();
        cube.insert(&test_coord(0.5), "src", "entry");
        let qr = cube.query_with_scores(&test_coord(0.5), 1);
        assert_eq!(qr.len(), 1);
        assert!(
            (qr[0].distance).abs() < 1e-9,
            "identical coord should have distance 0"
        );
        assert!(
            (qr[0].similarity - 1.0).abs() < 1e-9,
            "identical coord should have similarity 1"
        );
    }

    #[test]
    fn test_insert_with_task_type_sets_field() {
        let mut cube = KnowledgeHyperCube::new();
        cube.insert_with_task_type(&test_coord(0.5), "src", "t1", TaskType::CodeAnalysis);
        let entry = cube.get_entry("src-t1").unwrap();
        assert_eq!(entry.task_type, Some(TaskType::CodeAnalysis));
    }

    #[test]
    fn test_insert_defaults_to_none() {
        let mut cube = KnowledgeHyperCube::new();
        cube.insert(&test_coord(0.5), "src", "t2");
        let entry = cube.get_entry("src-t2").unwrap();
        assert_eq!(entry.task_type, None);
    }

    #[test]
    fn test_query_by_task_type_filters_correctly() {
        let mut cube = KnowledgeHyperCube::new();
        cube.insert_with_task_type(&test_coord(0.9), "s1", "code-entry", TaskType::CodeAnalysis);
        cube.insert_with_task_type(&test_coord(0.1), "s2", "design-entry", TaskType::Design);
        cube.insert_with_task_type(&test_coord(0.5), "s3", "security-entry", TaskType::Security);
        // Query for CodeAnalysis — should only return the code entry
        let results = cube.query_by_task_type(&test_coord(0.8), TaskType::CodeAnalysis, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "code-entry");
    }

    #[test]
    fn test_query_by_task_type_returns_closest_first() {
        let mut cube = KnowledgeHyperCube::new();
        cube.insert_with_task_type(&test_coord(0.1), "s1", "far", TaskType::Design);
        cube.insert_with_task_type(&test_coord(0.9), "s2", "near", TaskType::Design);
        cube.insert_with_task_type(&test_coord(0.5), "s3", "other", TaskType::Security);
        let results = cube.query_by_task_type(&test_coord(0.95), TaskType::Design, 5);
        assert_eq!(results.len(), 2);
        assert_eq!(
            results[0].label, "near",
            "closest Design entry should be first"
        );
    }

    #[test]
    fn test_query_by_task_type_empty_when_none_match() {
        let mut cube = KnowledgeHyperCube::new();
        cube.insert_with_task_type(&test_coord(0.5), "s1", "only-design", TaskType::Design);
        let results = cube.query_by_task_type(&test_coord(0.5), TaskType::Research, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_insert_and_find_vsa() {
        let mut cube = KnowledgeHyperCube::new();
        let coord = test_coord(0.5);
        let vsa = VsaVector::<4096>::random(42);
        let entry = CubeEntry {
            key: "vsa-test".to_string(),
            coord: coord.clone(),
            value: 1.0,
            label: "vsa-entry".to_string(),
            source: "vsa-src".to_string(),
            access_count: 0,
            task_type: None,
            vsa: None,
        };
        cube.insert_with_vsa(entry, vsa.clone());
        assert_eq!(cube.len(), 1);
        let results = cube.search_by_vsa(&vsa, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.label, "vsa-entry");
        assert!(
            (results[0].0 - 1.0).abs() < 1e-10,
            "identical VSA should have similarity 1"
        );
    }

    #[test]
    fn test_search_vsa_returns_nearest() {
        let mut cube = KnowledgeHyperCube::new();
        let query = VsaVector::<4096>::random(100);
        let exact = VsaVector::<4096>::random(100);
        let other = VsaVector::<4096>::random(999);

        cube.insert_with_vsa(
            CubeEntry {
                key: "exact".to_string(),
                coord: test_coord(0.5),
                value: 1.0,
                label: "exact-match".to_string(),
                source: "src".to_string(),
                access_count: 0,
                task_type: None,
                vsa: None,
            },
            exact.clone(),
        );
        cube.insert_with_vsa(
            CubeEntry {
                key: "other".to_string(),
                coord: test_coord(0.1),
                value: 1.0,
                label: "other-entry".to_string(),
                source: "src".to_string(),
                access_count: 0,
                task_type: None,
                vsa: None,
            },
            other.clone(),
        );

        let results = cube.search_by_vsa(&query, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1.label, "exact-match");
        assert!((results[0].0 - 1.0).abs() < 1e-10);
        assert!(results[0].0 > results[1].0);
    }
}
