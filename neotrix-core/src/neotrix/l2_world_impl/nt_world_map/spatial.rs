use crate::neotrix::l2_world_impl::nt_world_map::types::{GeoPoint, BoundingBox};

#[derive(Debug, Clone)]
pub struct SpatialQuery {
    pub bbox: Option<BoundingBox>,
    pub center: Option<(GeoPoint, f64)>,
    pub point: Option<GeoPoint>,
    pub limit: usize,
}

impl Default for SpatialQuery {
    fn default() -> Self {
        Self { bbox: None, center: None, point: None, limit: 100 }
    }
}

pub trait SpatialIndex: Send + Sync {
    fn insert(&mut self, id: u64, bbox: BoundingBox);
    fn remove(&mut self, id: u64);
    fn search_bbox(&self, bbox: &BoundingBox) -> Vec<u64>;
    fn search_point(&self, point: &GeoPoint, radius_m: f64) -> Vec<u64>;
    fn nearest(&self, point: &GeoPoint, k: usize) -> Vec<(u64, f64)>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
}

pub struct GridIndex {
    cell_size_deg: f64,
    cells: Vec<Vec<u64>>,
    cols: usize,
    rows: usize,
    item_bbox: Vec<(u64, BoundingBox)>,
}

impl GridIndex {
    pub fn new(cell_size_deg: f64) -> Self {
        let cols = (360.0 / cell_size_deg).ceil() as usize;
        let rows = (180.0 / cell_size_deg).ceil() as usize;
        Self { cell_size_deg, cells: vec![vec![]; cols * rows], cols, rows, item_bbox: Vec::new() }
    }

    fn cell(&self, lat: f64, lng: f64) -> (usize, usize) {
        let col = ((lng + 180.0) / self.cell_size_deg).floor() as usize;
        let row = ((lat + 90.0) / self.cell_size_deg).floor() as usize;
        (col.min(self.cols - 1), row.min(self.rows - 1))
    }
}

impl SpatialIndex for GridIndex {
    fn insert(&mut self, id: u64, bbox: BoundingBox) {
        self.item_bbox.push((id, bbox));
        let (c1x, c1y) = self.cell(bbox.min_lat, bbox.min_lng);
        let (c2x, c2y) = self.cell(bbox.max_lat, bbox.max_lng);
        for cy in c1y..=c2y {
            for cx in c1x..=c2x {
                self.cells[cy * self.cols + cx].push(id);
            }
        }
    }

    fn remove(&mut self, id: u64) {
        self.item_bbox.retain(|(i, _)| *i != id);
        for cell in &mut self.cells {
            cell.retain(|i| *i != id);
        }
    }

    fn search_bbox(&self, bbox: &BoundingBox) -> Vec<u64> {
        let (c1x, c1y) = self.cell(bbox.min_lat, bbox.min_lng);
        let (c2x, c2y) = self.cell(bbox.max_lat, bbox.max_lng);
        let mut ids = Vec::new();
        for cy in c1y..=c2y {
            for cx in c1x..=c2x {
                for id in &self.cells[cy * self.cols + cx] {
                    if !ids.contains(id) {
                        if let Some((_, bb)) = self.item_bbox.iter().find(|(i, _)| i == id) {
                            if bb.intersects(bbox) {
                                ids.push(*id);
                            }
                        }
                    }
                }
            }
        }
        ids.sort();
        ids.dedup();
        ids
    }

    fn search_point(&self, point: &GeoPoint, radius_m: f64) -> Vec<u64> {
        let lat_deg = radius_m / 111_320.0;
        let lng_deg = radius_m / (111_320.0 * point.lat.to_radians().cos().max(0.01));
        let bbox = BoundingBox::new(
            point.lat - lat_deg, point.lng - lng_deg,
            point.lat + lat_deg, point.lng + lng_deg,
        );
        self.search_bbox(&bbox).into_iter().filter(|id| {
            if let Some((_, bb)) = self.item_bbox.iter().find(|(i, _)| i == id) {
                bb.contains(point)
            } else { false }
        }).collect()
    }

    fn nearest(&self, point: &GeoPoint, k: usize) -> Vec<(u64, f64)> {
        let mut dists: Vec<(u64, f64)> = self.item_bbox.iter().map(|(id, bb)| {
            let c = bb.center();
            (*id, point.distance_haversine(&c))
        }).collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        dists.truncate(k);
        dists
    }

    fn len(&self) -> usize { self.item_bbox.len() }
}

pub fn cluster_points(points: &[GeoPoint], radius_m: f64) -> Vec<Vec<usize>> {
    let mut clusters: Vec<Vec<usize>> = Vec::new();
    let mut assigned = vec![false; points.len()];
    for i in 0..points.len() {
        if assigned[i] { continue; }
        let mut cluster = vec![i];
        assigned[i] = true;
        for j in (i + 1)..points.len() {
            if !assigned[j] && points[i].distance_haversine(&points[j]) < radius_m {
                cluster.push(j);
                assigned[j] = true;
            }
        }
        clusters.push(cluster);
    }
    clusters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_index_insert_search() {
        let mut idx = GridIndex::new(10.0);
        let bb = BoundingBox::new(30.0, -10.0, 50.0, 20.0);
        idx.insert(42, bb);
        assert_eq!(idx.len(), 1);
        let results = idx.search_bbox(&BoundingBox::new(35.0, -5.0, 45.0, 10.0));
        assert!(results.contains(&42));
    }

    #[test]
    fn test_grid_index_remove() {
        let mut idx = GridIndex::new(10.0);
        idx.insert(1, BoundingBox::new(0.0, 0.0, 10.0, 10.0));
        idx.remove(1);
        assert!(idx.is_empty());
    }

    #[test]
    fn test_grid_index_nearest() {
        let mut idx = GridIndex::new(45.0);
        idx.insert(1, BoundingBox::new(40.0, -74.0, 41.0, -73.0));
        idx.insert(2, BoundingBox::new(48.0, 2.0, 49.0, 3.0));
        let nearest = idx.nearest(&GeoPoint::new(40.7, -73.9), 1);
        assert_eq!(nearest[0].0, 1);
    }

    #[test]
    fn test_cluster_points() {
        let points = vec![
            GeoPoint::new(40.0, -74.0),
            GeoPoint::new(40.01, -74.01),
            GeoPoint::new(48.0, 2.0),
        ];
        let clusters = cluster_points(&points, 2000.0);
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].len(), 2);
        assert_eq!(clusters[1].len(), 1);
    }

    #[test]
    fn test_search_point_with_radius() {
        let mut idx = GridIndex::new(45.0);
        let bb1 = BoundingBox::new(20.0, -100.0, 50.0, -50.0);
        idx.insert(1, bb1);
        let bb2 = BoundingBox::new(40.0, 0.0, 60.0, 30.0);
        idx.insert(2, bb2);
        let ny = GeoPoint::new(40.7128, -74.0060);
        let results = idx.search_point(&ny, 2_000_000.0);
        assert!(results.contains(&1));
        assert!(!results.contains(&2));
    }
}
