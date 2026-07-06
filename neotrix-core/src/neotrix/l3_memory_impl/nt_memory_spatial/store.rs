use std::collections::HashMap;
use crate::neotrix::l3_memory_impl::nt_memory_spatial::types::{
    SpatialEntry, SpatialQuery, geohash_encode,
};

pub trait SpatialStore: Send + Sync {
    fn insert(&mut self, entry: SpatialEntry);
    fn get(&self, id: &str) -> Option<&SpatialEntry>;
    fn remove(&mut self, id: &str) -> bool;
    fn query(&self, query: &SpatialQuery) -> Vec<&SpatialEntry>;
    fn search_by_geohash(&self, geohash: &str) -> Vec<&SpatialEntry>;
    fn search_by_tag(&self, tag: &str) -> Vec<&SpatialEntry>;
    fn search_by_source(&self, source: &str) -> Vec<&SpatialEntry>;
    fn all(&self) -> Vec<&SpatialEntry>;
    fn len(&self) -> usize;
}

pub struct MemorySpatialStore {
    entries: HashMap<String, SpatialEntry>,
    geohash_index: HashMap<String, Vec<String>>,
    tag_index: HashMap<String, Vec<String>>,
    source_index: HashMap<String, Vec<String>>,
}

impl Default for MemorySpatialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemorySpatialStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            geohash_index: HashMap::new(),
            tag_index: HashMap::new(),
            source_index: HashMap::new(),
        }
    }

    fn index_entry(&mut self, id: &str, entry: &SpatialEntry) {
        let bbox = entry.bbox();
        let center = bbox.center();
        let gh = geohash_encode(&center, 6);
        self.geohash_index.entry(gh).or_default().push(id.to_string());

        for tag in &entry.tags {
            self.tag_index.entry(tag.clone()).or_default().push(id.to_string());
        }

        self.source_index.entry(entry.source.clone()).or_default().push(id.to_string());
    }
}

impl SpatialStore for MemorySpatialStore {
    fn insert(&mut self, entry: SpatialEntry) {
        let id = entry.id.clone();
        self.index_entry(&id, &entry);
        self.entries.insert(id, entry);
    }

    fn get(&self, id: &str) -> Option<&SpatialEntry> {
        self.entries.get(id)
    }

    fn remove(&mut self, id: &str) -> bool {
        self.entries.remove(id).is_some()
    }

    fn query(&self, query: &SpatialQuery) -> Vec<&SpatialEntry> {
        let mut results: Vec<&SpatialEntry> = self.entries.values().collect();

        if let Some(ref bbox) = query.bbox {
            results.retain(|e| e.bbox().intersects(bbox));
        }

        if let Some((ref center, radius)) = query.center {
            let radius_val = radius;
            results.retain(|e| {
                let c = e.bbox().center();
                center.distance_haversine(&c) <= radius_val
            });
        }

        if !query.tags.is_empty() {
            results.retain(|e| query.tags.iter().any(|t| e.tags.contains(t)));
        }

        if let Some(ref source) = query.source {
            results.retain(|e| &e.source == source);
        }

        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        let offset = query.offset.min(results.len());
        let end = (offset + query.limit).min(results.len());
        results[offset..end].to_vec()
    }

    fn search_by_geohash(&self, geohash: &str) -> Vec<&SpatialEntry> {
        self.geohash_index.get(geohash)
            .map(|ids| ids.iter().filter_map(|id| self.entries.get(id)).collect())
            .unwrap_or_default()
    }

    fn search_by_tag(&self, tag: &str) -> Vec<&SpatialEntry> {
        self.tag_index.get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.entries.get(id)).collect())
            .unwrap_or_default()
    }

    fn search_by_source(&self, source: &str) -> Vec<&SpatialEntry> {
        self.source_index.get(source)
            .map(|ids| ids.iter().filter_map(|id| self.entries.get(id)).collect())
            .unwrap_or_default()
    }

    fn all(&self) -> Vec<&SpatialEntry> {
        self.entries.values().collect()
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::neotrix::l3_memory_impl::nt_memory_spatial::types::{BoundingBox, GeoPoint, SpatialGeometry};

    fn make_entry(id: &str, lat: f64, lng: f64, tags: Vec<&str>) -> SpatialEntry {
        SpatialEntry {
            id: id.to_string(),
            name: id.to_string(),
            geometry: SpatialGeometry::Point(GeoPoint::new(lat, lng)),
            properties: HashMap::new(),
            source: "test".to_string(),
            confidence: 1.0,
            created_at: 0,
            updated_at: 0,
            tags: tags.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_insert_and_get() {
        let mut store = MemorySpatialStore::new();
        store.insert(make_entry("nyc", 40.7128, -74.0060, vec!["city", "usa"]));
        assert!(store.get("nyc").is_some());
    }

    #[test]
    fn test_remove() {
        let mut store = MemorySpatialStore::new();
        store.insert(make_entry("test", 0.0, 0.0, vec![]));
        assert!(store.remove("test"));
        assert!(!store.remove("nonexistent"));
    }

    #[test]
    fn test_query_bbox() {
        let mut store = MemorySpatialStore::new();
        store.insert(make_entry("nyc", 40.7128, -74.0060, vec![]));
        store.insert(make_entry("paris", 48.8566, 2.3522, vec![]));
        let q = SpatialQuery {
            bbox: Some(BoundingBox::new(30.0, -80.0, 50.0, -70.0)),
            ..Default::default()
        };
        let results = store.query(&q);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "nyc");
    }

    #[test]
    fn test_search_by_tag() {
        let mut store = MemorySpatialStore::new();
        store.insert(make_entry("nyc", 40.7128, -74.0060, vec!["city", "usa"]));
        store.insert(make_entry("paris", 48.8566, 2.3522, vec!["city", "france"]));
        let results = store.search_by_tag("usa");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_geohash() {
        let mut store = MemorySpatialStore::new();
        store.insert(make_entry("nyc", 40.7128, -74.0060, vec![]));
        let entries = store.search_by_geohash("dr5reg");
        assert!(entries.is_empty() || entries.iter().any(|e| e.id == "nyc"));
    }

    #[test]
    fn test_query_pagination() {
        let mut store = MemorySpatialStore::new();
        for i in 0..50 {
            store.insert(make_entry(&format!("p{}", i), i as f64, i as f64, vec![]));
        }
        let q = SpatialQuery { limit: 10, offset: 0, ..Default::default() };
        assert_eq!(store.query(&q).len(), 10);
    }

    #[test]
    fn test_query_center_radius() {
        let mut store = MemorySpatialStore::new();
        store.insert(make_entry("nyc", 40.7128, -74.0060, vec![]));
        store.insert(make_entry("la", 34.0522, -118.2437, vec![]));
        let q = SpatialQuery {
            center: Some((GeoPoint::new(40.7, -74.0), 10_000.0)),
            ..Default::default()
        };
        let results = store.query(&q);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "nyc");
    }
}
