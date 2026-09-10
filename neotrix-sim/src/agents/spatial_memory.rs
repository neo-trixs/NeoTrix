use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: u64,
    pub name: String,
    pub position: [f32; 2],
    pub biome: String,
    pub resources: Vec<String>,
    pub danger_level: f32,
    pub visited: bool,
    pub visit_count: u32,
    pub last_visit_tick: u64,
    pub first_discovered_tick: u64,
    pub connections: Vec<u64>,
}

pub struct SpatialMemory {
    locations: Vec<Location>,
    next_id: u64,
    pub current_position: [f32; 2],
    max_locations: usize,
    position_index: HashMap<(i32, i32), Vec<u64>>,
    cell_size: f32,
}

impl SpatialMemory {
    pub fn new(start_position: [f32; 2]) -> Self {
        Self {
            locations: Vec::new(),
            next_id: 0,
            current_position: start_position,
            max_locations: 200,
            position_index: HashMap::new(),
            cell_size: 50.0,
        }
    }

    fn cell_key(&self, pos: [f32; 2]) -> (i32, i32) {
        ((pos[0] / self.cell_size).floor() as i32, (pos[1] / self.cell_size).floor() as i32)
    }

    fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
    }

    pub fn visit(
        &mut self,
        position: [f32; 2],
        biome: &str,
        resources: Vec<String>,
        danger_level: f32,
        tick: u64,
    ) -> u64 {
        self.current_position = position;
        // Check if already known — merge radius much smaller than cell_size
        let merge_threshold = self.cell_size * 0.1;
        if let Some(existing) = self.locations.iter_mut().find(|l| {
            Self::dist(l.position, position) < merge_threshold
        }) {
            existing.visit_count += 1;
            existing.last_visit_tick = tick;
            existing.visited = true;
            for r in &resources {
                if !existing.resources.contains(r) {
                    existing.resources.push(r.clone());
                }
            }
            existing.danger_level = (existing.danger_level + danger_level) / 2.0;
            return existing.id;
        }

        let id = self.next_id;
        self.next_id += 1;
        let key = self.cell_key(position);

        // Connect to nearby locations
        let nearby: Vec<u64> = self.locations.iter()
            .filter(|l| Self::dist(l.position, position) < self.cell_size)
            .map(|l| l.id)
            .collect();

        let loc = Location {
            id,
            name: format!("loc_{}", id),
            position,
            biome: biome.to_string(),
            resources,
            danger_level,
            visited: true,
            visit_count: 1,
            last_visit_tick: tick,
            first_discovered_tick: tick,
            connections: nearby.clone(),
        };

        // Add reverse connections
        for nid in &nearby {
            if let Some(n) = self.locations.iter_mut().find(|l| l.id == *nid) {
                if !n.connections.contains(&id) {
                    n.connections.push(id);
                }
            }
        }

        self.locations.push(loc);
        self.position_index.entry(key).or_default().push(id);
        self.prune();
        id
    }

    pub fn nearest(&self, position: [f32; 2]) -> Option<&Location> {
        self.locations.iter()
            .min_by(|a, b| {
                Self::dist(a.position, position).partial_cmp(&Self::dist(b.position, position)).unwrap()
            })
    }

    pub fn by_biome(&self, biome: &str) -> Vec<&Location> {
        self.locations.iter().filter(|l| l.biome == biome).collect()
    }

    pub fn with_resources(&self, resources: &[String]) -> Vec<&Location> {
        self.locations.iter()
            .filter(|l| resources.iter().any(|r| l.resources.contains(r)))
            .collect()
    }

    pub fn safe_locations(&self, max_danger: f32) -> Vec<&Location> {
        self.locations.iter().filter(|l| l.danger_level <= max_danger).collect()
    }

    pub fn unvisited(&self) -> Vec<&Location> {
        self.locations.iter().filter(|l| !l.visited).collect()
    }

    pub fn stale(&self, current_tick: u64, stale_threshold: u64) -> Vec<&Location> {
        self.locations.iter()
            .filter(|l| l.visited && current_tick.saturating_sub(l.last_visit_tick) > stale_threshold)
            .collect()
    }

    pub fn connect(&mut self, id_a: u64, id_b: u64) {
        if let Some(a) = self.locations.iter_mut().find(|l| l.id == id_a) {
            if !a.connections.contains(&id_b) { a.connections.push(id_b); }
        }
        if let Some(b) = self.locations.iter_mut().find(|l| l.id == id_b) {
            if !b.connections.contains(&id_a) { b.connections.push(id_a); }
        }
    }

    pub fn connections(&self, location_id: u64) -> Vec<&Location> {
        if let Some(loc) = self.locations.iter().find(|l| l.id == location_id) {
            loc.connections.iter()
                .filter_map(|cid| self.locations.iter().find(|l| l.id == *cid))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn prune(&mut self) {
        if self.locations.len() <= self.max_locations { return; }
        self.locations.sort_by(|a, b| {
            b.visit_count.cmp(&a.visit_count)
                .then(b.last_visit_tick.cmp(&a.last_visit_tick))
        });
        self.locations.truncate(self.max_locations);
    }

    pub fn location_count(&self) -> usize { self.locations.len() }
    pub fn visited_count(&self) -> usize { self.locations.iter().filter(|l| l.visited).count() }
    pub fn unvisited_count(&self) -> usize { self.locations.iter().filter(|l| !l.visited).count() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visit_creates_location() {
        let mut sm = SpatialMemory::new([0.0, 0.0]);
        let id = sm.visit([10.0, 10.0], "forest", vec!["wood".to_string()], 0.1, 0);
        assert_eq!(id, 0);
        assert_eq!(sm.location_count(), 1);
    }

    #[test]
    fn visit_same_location_updates() {
        let mut sm = SpatialMemory::new([0.0, 0.0]);
        sm.visit([10.0, 10.0], "forest", vec!["wood".to_string()], 0.1, 0);
        sm.visit([10.0, 10.0], "forest", vec!["berries".to_string()], 0.2, 1);
        assert_eq!(sm.location_count(), 1);
        assert_eq!(sm.locations[0].visit_count, 2);
        assert!(sm.locations[0].resources.contains(&"berries".to_string()));
    }

    #[test]
    fn nearest_returns_closest() {
        let mut sm = SpatialMemory::new([0.0, 0.0]);
        sm.visit([10.0, 0.0], "forest", vec![], 0.0, 0);
        sm.visit([100.0, 0.0], "desert", vec![], 0.0, 0);
        let nearest = sm.nearest([12.0, 0.0]).unwrap();
        assert_eq!(nearest.biome, "forest");
    }

    #[test]
    fn by_biome_filters() {
        let mut sm = SpatialMemory::new([0.0, 0.0]);
        sm.visit([10.0, 0.0], "forest", vec![], 0.0, 0);
        sm.visit([20.0, 0.0], "desert", vec![], 0.0, 0);
        sm.visit([30.0, 0.0], "forest", vec![], 0.0, 0);
        assert_eq!(sm.by_biome("forest").len(), 2);
        assert_eq!(sm.by_biome("desert").len(), 1);
    }

    #[test]
    fn connections_link_nearby() {
        let mut sm = SpatialMemory::new([0.0, 0.0]);
        let id1 = sm.visit([10.0, 0.0], "forest", vec![], 0.0, 0);
        let id2 = sm.visit([20.0, 0.0], "forest", vec![], 0.0, 0);
        assert!(sm.connections(id1).iter().any(|l| l.id == id2));
        assert!(sm.connections(id2).iter().any(|l| l.id == id1));
    }

    #[test]
    fn safe_locations_filters() {
        let mut sm = SpatialMemory::new([0.0, 0.0]);
        sm.visit([10.0, 0.0], "forest", vec![], 0.1, 0);
        sm.visit([20.0, 0.0], "volcanic", vec![], 0.9, 0);
        assert_eq!(sm.safe_locations(0.5).len(), 1);
    }

    #[test]
    fn unvisited_returns_only_unvisited() {
        let mut sm = SpatialMemory::new([0.0, 0.0]);
        sm.visit([10.0, 0.0], "forest", vec![], 0.0, 0);
        sm.locations.push(Location {
            id: 99, name: "far".to_string(), position: [999.0, 999.0],
            biome: "desert".to_string(), resources: vec![], danger_level: 0.0,
            visited: false, visit_count: 0, last_visit_tick: 0,
            first_discovered_tick: 0, connections: vec![],
        });
        assert_eq!(sm.unvisited().len(), 1);
        assert_eq!(sm.unvisited()[0].id, 99);
    }
}
