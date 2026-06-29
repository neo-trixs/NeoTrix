use std::collections::HashMap;
use super::coord::HyperCoord;

#[derive(Debug, Clone)]
pub struct CubeEntry {
    pub key: String,
    pub coord: HyperCoord,
    pub value: f64,
    pub label: String,
    pub source: String,
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
        Self { entries: HashMap::new() }
    }

    pub fn insert(&mut self, coord: &HyperCoord, source: &str, title: &str) {
        let key = format!("{}-{}", source, title);
        self.entries.insert(key.clone(), CubeEntry {
            key, coord: coord.clone(), value: 0.0,
            label: title.to_string(), source: source.to_string(),
        });
    }

    pub fn query(&self, _coord: &HyperCoord, _top_k: usize) -> Vec<&CubeEntry> {
        self.entries.values().collect()
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

    pub fn coord_density(&self, _dim: usize) -> f64 {
        if self.entries.is_empty() { return 0.0; }
        let total: f64 = self.entries.values().map(|e| e.value).sum();
        (total / self.entries.len() as f64).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::coord::HyperCoord;

    fn make_coord(_values: &[f64]) -> HyperCoord { HyperCoord::new() }

    #[test]
    fn test_new_creates_empty_cube() {
        let cube = KnowledgeHyperCube::new();
        assert!(cube.is_empty());
        assert_eq!(cube.len(), 0);
    }

    #[test]
    fn test_insert_and_query() {
        let mut cube = KnowledgeHyperCube::new();
        let coord = make_coord(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        cube.insert(&coord, "test-source", "test-entry");
        assert_eq!(cube.len(), 1);
        let results = cube.query(&coord, 10);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_coord_density_empty() {
        let cube = KnowledgeHyperCube::new();
        assert_eq!(cube.coord_density(0), 0.0);
    }
}
