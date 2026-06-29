use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Geographic coordinate with optional altitude.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoCoord {
    /// Latitude in degrees, -90 to 90
    pub lat: f64,
    /// Longitude in degrees, -180 to 180
    pub lng: f64,
    /// Optional altitude in meters above sea level
    pub alt: Option<f64>,
}

impl GeoCoord {
    pub fn new(lat: f64, lng: f64) -> Self {
        GeoCoord {
            lat,
            lng,
            alt: None,
        }
    }

    pub fn with_alt(lat: f64, lng: f64, alt: f64) -> Self {
        GeoCoord {
            lat,
            lng,
            alt: Some(alt),
        }
    }

    /// Haversine distance in meters between two coordinates.
    pub fn haversine_distance(&self, other: &GeoCoord) -> f64 {
        let r = 6_371_000.0;
        let d_lat = (other.lat - self.lat).to_radians();
        let d_lng = (other.lng - self.lng).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + self.lat.to_radians().cos()
                * other.lat.to_radians().cos()
                * (d_lng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        r * c
    }

    /// Approximate squared euclidean distance using degrees as units.
    /// Suitable for KD-tree ordering near the same latitude band.
    pub fn squared_euclidean(&self, other: &GeoCoord) -> f64 {
        let d_lat = self.lat - other.lat;
        let d_lng = self.lng - other.lng;
        d_lat * d_lat + d_lng * d_lng
    }

    pub fn is_valid(&self) -> bool {
        self.lat >= -90.0 && self.lat <= 90.0 && self.lng >= -180.0 && self.lng <= 180.0
    }
}

/// OSM-inspired spatial feature type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpatialFeatureType {
    City,
    Town,
    Village,
    Road,
    Building,
    Park,
    WaterBody,
    Forest,
    LandUse,
    POI,
    AdminBoundary,
    Railway,
    Transit,
    Custom(String),
}

impl SpatialFeatureType {
    pub fn as_str(&self) -> &str {
        match self {
            SpatialFeatureType::City => "city",
            SpatialFeatureType::Town => "town",
            SpatialFeatureType::Village => "village",
            SpatialFeatureType::Road => "road",
            SpatialFeatureType::Building => "building",
            SpatialFeatureType::Park => "park",
            SpatialFeatureType::WaterBody => "water_body",
            SpatialFeatureType::Forest => "forest",
            SpatialFeatureType::LandUse => "land_use",
            SpatialFeatureType::POI => "poi",
            SpatialFeatureType::AdminBoundary => "admin_boundary",
            SpatialFeatureType::Railway => "railway",
            SpatialFeatureType::Transit => "transit",
            SpatialFeatureType::Custom(s) => s.as_str(),
        }
    }
}

/// A spatial node in the graph, representing a geographic entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialNode {
    pub id: String,
    pub coord: GeoCoord,
    pub feature_type: SpatialFeatureType,
    pub name: String,
    pub tags: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

impl SpatialNode {
    pub fn new(id: &str, lat: f64, lng: f64, name: &str, feature_type: SpatialFeatureType) -> Self {
        SpatialNode {
            id: id.to_string(),
            coord: GeoCoord::new(lat, lng),
            feature_type,
            name: name.to_string(),
            tags: HashMap::new(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_alt(mut self, alt: f64) -> Self {
        self.coord.alt = Some(alt);
        self
    }
}

/// Relation between two spatial nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpatialRelation {
    ConnectedBy { road_name: String, distance_m: f64 },
    Contains,
    Adjacent,
    Near { distance_m: f64 },
    PartOf { parent_id: String },
}

// ---------------------------------------------------------------------------
// KD-tree spatial index
// ---------------------------------------------------------------------------

/// A node in the KD-tree.
#[derive(Debug, Clone)]
pub struct KdNode {
    pub point: GeoCoord,
    pub node_id: String,
    pub axis: usize,
    pub left: Option<Box<KdNode>>,
    pub right: Option<Box<KdNode>>,
}

/// KD-tree index for efficient spatial queries on geographic coordinates.
#[derive(Debug, Clone)]
pub struct KdTree {
    root: Option<Box<KdNode>>,
    size: usize,
}

impl KdTree {
    pub fn new() -> Self {
        KdTree {
            root: None,
            size: 0,
        }
    }

    /// Build a KD-tree from a slice of (coordinate, node_id) pairs.
    /// Uses recursive median-based construction: O(n log n).
    pub fn build(nodes: &[(GeoCoord, String)]) -> Self {
        if nodes.is_empty() {
            return KdTree::new();
        }
        let mut items: Vec<(GeoCoord, String)> = nodes.to_vec();
        let root = Self::build_rec(&mut items, 0);
        KdTree {
            root: Some(Box::new(root)),
            size: items.len(),
        }
    }

    fn build_rec(items: &mut [(GeoCoord, String)], depth: usize) -> KdNode {
        if items.is_empty() {
            log::error!("[spatial_graph] KD-Tree build_rec received empty slice");
            return KdNode {
                point: GeoCoord {
                    lat: 0.0,
                    lng: 0.0,
                    alt: None,
                },
                node_id: String::new(),
                axis: 0,
                left: None,
                right: None,
            };
        }
        let axis = depth % 2;
        items.sort_by(|a, b| {
            let va = if axis == 0 { a.0.lat } else { a.0.lng };
            let vb = if axis == 0 { b.0.lat } else { b.0.lng };
            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
        });
        let mid = items.len() / 2;
        let (point, node_id) = items[mid].clone();
        let left = if mid > 0 {
            Some(Box::new(Self::build_rec(&mut items[..mid], depth + 1)))
        } else {
            None
        };
        let right = if mid + 1 < items.len() {
            Some(Box::new(Self::build_rec(&mut items[mid + 1..], depth + 1)))
        } else {
            None
        };
        KdNode {
            point,
            node_id,
            axis,
            left,
            right,
        }
    }

    /// Insert a single point into the tree. O(log n) average.
    pub fn insert(&mut self, coord: GeoCoord, node_id: &str) {
        self.size += 1;
        let new_node = Box::new(KdNode {
            point: coord,
            node_id: node_id.to_string(),
            axis: 0,
            left: None,
            right: None,
        });
        match self.root {
            Some(ref mut root) => Self::insert_rec(root, new_node, 0),
            None => self.root = Some(new_node),
        }
    }

    fn insert_rec(current: &mut Box<KdNode>, new_node: Box<KdNode>, depth: usize) {
        let axis = depth % 2;
        let coord_val = if axis == 0 {
            new_node.point.lat
        } else {
            new_node.point.lng
        };
        let node_val = if axis == 0 {
            current.point.lat
        } else {
            current.point.lng
        };
        if coord_val < node_val {
            match current.left {
                Some(ref mut child) => Self::insert_rec(child, new_node, depth + 1),
                None => current.left = Some(new_node),
            }
        } else {
            match current.right {
                Some(ref mut child) => Self::insert_rec(child, new_node, depth + 1),
                None => current.right = Some(new_node),
            }
        }
    }

    /// Find the nearest neighbor to a query coordinate.
    /// Returns `(node_id, euclidean_distance)` where distance is in degree-units.
    pub fn nearest_neighbor(&self, coord: &GeoCoord) -> Option<(String, f64)> {
        let mut best: Option<(String, f64)> = None;
        let mut best_dist = f64::MAX;

        fn search(
            node: Option<&Box<KdNode>>,
            coord: &GeoCoord,
            best: &mut Option<(String, f64)>,
            best_dist: &mut f64,
            depth: usize,
        ) {
            let node = match node {
                Some(n) => n,
                None => return,
            };

            let dist = coord.squared_euclidean(&node.point);
            if dist < *best_dist {
                *best_dist = dist;
                *best = Some((node.node_id.clone(), dist));
            }

            let axis = depth % 2;
            let coord_val = if axis == 0 { coord.lat } else { coord.lng };
            let node_val = if axis == 0 {
                node.point.lat
            } else {
                node.point.lng
            };

            let (first, second) = if coord_val < node_val {
                (node.left.as_ref(), node.right.as_ref())
            } else {
                (node.right.as_ref(), node.left.as_ref())
            };

            search(first, coord, best, best_dist, depth + 1);

            let diff = coord_val - node_val;
            if diff * diff < *best_dist {
                search(second, coord, best, best_dist, depth + 1);
            }
        }

        search(self.root.as_ref(), coord, &mut best, &mut best_dist, 0);
        best.map(|(id, dist_sq)| (id, dist_sq.sqrt()))
    }

    /// Find the k nearest neighbors to a query coordinate.
    pub fn k_nearest(&self, coord: &GeoCoord, k: usize) -> Vec<(String, f64)> {
        let mut results: Vec<(String, f64)> = Vec::new();
        let mut best_dist = f64::MAX;

        fn search(
            node: Option<&Box<KdNode>>,
            coord: &GeoCoord,
            results: &mut Vec<(String, f64)>,
            best_dist: &mut f64,
            k: usize,
            depth: usize,
        ) {
            let node = match node {
                Some(n) => n,
                None => return,
            };

            let dist = coord.squared_euclidean(&node.point);
            if dist < *best_dist || results.len() < k {
                results.push((node.node_id.clone(), dist));
                results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                if results.len() > k {
                    results.truncate(k);
                }
                if results.len() == k {
                    *best_dist = results.last().unwrap().1;
                }
            }

            let axis = depth % 2;
            let coord_val = if axis == 0 { coord.lat } else { coord.lng };
            let node_val = if axis == 0 {
                node.point.lat
            } else {
                node.point.lng
            };

            let (first, second) = if coord_val < node_val {
                (node.left.as_ref(), node.right.as_ref())
            } else {
                (node.right.as_ref(), node.left.as_ref())
            };

            search(first, coord, results, best_dist, k, depth + 1);

            let diff = coord_val - node_val;
            if diff * diff < *best_dist {
                search(second, coord, results, best_dist, k, depth + 1);
            }
        }

        search(
            self.root.as_ref(),
            coord,
            &mut results,
            &mut best_dist,
            k,
            0,
        );
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results.into_iter().map(|(id, d)| (id, d.sqrt())).collect()
    }

    /// Find all node IDs within a given radius (in meters) from the center.
    pub fn range_query(&self, center: &GeoCoord, radius_m: f64) -> Vec<String> {
        let mut found = Vec::new();
        fn search(
            node: Option<&Box<KdNode>>,
            center: &GeoCoord,
            radius_m: f64,
            found: &mut Vec<String>,
            depth: usize,
        ) {
            let node = match node {
                Some(n) => n,
                None => return,
            };

            let dist = center.haversine_distance(&node.point);
            if dist <= radius_m {
                found.push(node.node_id.clone());
            }

            let axis = depth % 2;
            let coord_val = if axis == 0 { center.lat } else { center.lng };
            let node_val = if axis == 0 {
                node.point.lat
            } else {
                node.point.lng
            };

            let (first, second) = if coord_val < node_val {
                (node.left.as_ref(), node.right.as_ref())
            } else {
                (node.right.as_ref(), node.left.as_ref())
            };

            search(first, center, radius_m, found, depth + 1);

            let d_deg = coord_val - node_val;
            let d_m = d_deg.abs() * 111_320.0;
            if d_m <= radius_m {
                search(second, center, radius_m, found, depth + 1);
            }
        }

        search(self.root.as_ref(), center, radius_m, &mut found, 0);
        found
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl Default for KdTree {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SpatialGraph — main container
// ---------------------------------------------------------------------------

/// A spatial knowledge graph that binds geographic coordinates to named entities
/// and supports spatial queries via an embedded KD-tree index.
#[derive(Debug, Clone)]
pub struct SpatialGraph {
    nodes: HashMap<String, SpatialNode>,
    index: KdTree,
    relations: Vec<(String, String, SpatialRelation)>,
}

impl SpatialGraph {
    pub fn new() -> Self {
        SpatialGraph {
            nodes: HashMap::new(),
            index: KdTree::new(),
            relations: Vec::new(),
        }
    }

    /// Add a spatial node and rebuild the KD-tree index.
    pub fn add_node(&mut self, node: SpatialNode) {
        let id = node.id.clone();
        let coord = node.coord;
        self.nodes.insert(id.clone(), node);
        self.index.insert(coord, &id);
    }

    /// Remove a node by ID and rebuild the index.
    pub fn remove_node(&mut self, id: &str) -> Option<SpatialNode> {
        let node = self.nodes.remove(id)?;
        self.rebuild_index();
        self.relations.retain(|(a, b, _)| a != id && b != id);
        Some(node)
    }

    pub fn get_node(&self, id: &str) -> Option<&SpatialNode> {
        self.nodes.get(id)
    }

    /// Find all nodes within a given radius (meters) of a coordinate.
    pub fn find_nearby(&self, coord: &GeoCoord, radius_m: f64) -> Vec<&SpatialNode> {
        let ids = self.index.range_query(coord, radius_m);
        ids.iter().filter_map(|id| self.nodes.get(id)).collect()
    }

    /// Find the k nearest nodes to a coordinate, ordered by distance.
    pub fn find_nearest(&self, coord: &GeoCoord, k: usize) -> Vec<&SpatialNode> {
        let nearest = self.index.k_nearest(coord, k);
        nearest
            .iter()
            .filter_map(|(id, _)| self.nodes.get(id))
            .collect()
    }

    /// Add a spatial relation between two existing nodes.
    pub fn add_relation(&mut self, from_id: &str, to_id: &str, relation: SpatialRelation) {
        if self.nodes.contains_key(from_id) && self.nodes.contains_key(to_id) {
            self.relations
                .push((from_id.to_string(), to_id.to_string(), relation));
        }
    }

    /// Get all relations for a given node ID.
    pub fn get_relations(&self, id: &str) -> Vec<(String, SpatialRelation)> {
        self.relations
            .iter()
            .filter_map(|(a, b, r)| {
                if a == id {
                    Some((b.clone(), r.clone()))
                } else if b == id {
                    Some((a.clone(), r.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find nodes matching a specific tag key and value.
    pub fn find_by_tag(&self, key: &str, value: &str) -> Vec<&SpatialNode> {
        self.nodes
            .values()
            .filter(|n| n.tags.get(key).map(|v| v == value).unwrap_or(false))
            .collect()
    }

    /// Find nodes by their spatial feature type.
    pub fn find_by_type(&self, feature_type: &SpatialFeatureType) -> Vec<&SpatialNode> {
        self.nodes
            .values()
            .filter(|n| n.feature_type == *feature_type)
            .collect()
    }

    /// Rebuild the KD-tree index from all current nodes.
    pub fn rebuild_index(&mut self) {
        let items: Vec<(GeoCoord, String)> = self
            .nodes
            .values()
            .map(|n| (n.coord, n.id.clone()))
            .collect();
        self.index = KdTree::build(&items);
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Export all spatial nodes as a GeoJSON FeatureCollection string.
    pub fn export_geojson(&self) -> String {
        let mut features = Vec::new();
        for node in self.nodes.values() {
            let alt = node
                .coord
                .alt
                .map(|a| format!(",{}", a))
                .unwrap_or_default();
            let props = serde_json::json!({
                "id": node.id,
                "name": node.name,
                "feature_type": node.feature_type.as_str(),
                "tags": node.tags,
                "metadata": node.metadata,
            });
            let feature = serde_json::json!({
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [node.coord.lng, node.coord.lat, alt],
                },
                "properties": props,
            });
            features.push(feature);
        }
        let collection = serde_json::json!({
            "type": "FeatureCollection",
            "features": features,
        });
        serde_json::to_string_pretty(&collection).unwrap_or_default()
    }

    /// Import nodes from a GeoJSON FeatureCollection string.
    /// Returns the number of nodes successfully imported.
    pub fn import_geojson(json: &str) -> Result<usize, String> {
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("Invalid GeoJSON: {}", e))?;
        let features = value
            .get("features")
            .and_then(|f| f.as_array())
            .ok_or_else(|| "Missing 'features' array".to_string())?;

        let mut count = 0usize;
        // We need a mutable graph to insert into, but this is a static method.
        // Return count and let the caller insert nodes individually.
        // Count valid features only.
        for feature in features {
            let geom = feature
                .get("geometry")
                .and_then(|g| g.as_object())
                .ok_or_else(|| "Feature missing geometry".to_string())?;
            let geom_type = geom
                .get("type")
                .and_then(|t| t.as_str())
                .ok_or_else(|| "Geometry missing type".to_string())?;
            if geom_type != "Point" {
                continue;
            }
            let coords = geom
                .get("coordinates")
                .and_then(|c| c.as_array())
                .ok_or_else(|| "Point missing coordinates".to_string())?;
            if coords.len() < 2 {
                continue;
            }
            let lng = coords[0]
                .as_f64()
                .ok_or_else(|| "Invalid longitude".to_string())?;
            let lat = coords[1]
                .as_f64()
                .ok_or_else(|| "Invalid latitude".to_string())?;
            let coord = GeoCoord::new(lat, lng);
            if !coord.is_valid() {
                continue;
            }
            count += 1;
        }
        Ok(count)
    }
}

impl Default for SpatialGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_coord_haversine() {
        // NYC (40.7128, -74.0060) to London (51.5074, -0.1278) ~5570 km
        let nyc = GeoCoord::new(40.7128, -74.0060);
        let london = GeoCoord::new(51.5074, -0.1278);
        let dist = nyc.haversine_distance(&london);
        // Allow ±50 km tolerance
        assert!(
            (dist - 5_570_000.0).abs() < 50_000.0,
            "NYC-London distance {} m outside expected range",
            dist
        );
    }

    #[test]
    fn test_geo_coord_validity() {
        assert!(GeoCoord::new(0.0, 0.0).is_valid());
        assert!(GeoCoord::new(90.0, 180.0).is_valid());
        assert!(GeoCoord::new(-90.0, -180.0).is_valid());
        assert!(!GeoCoord::new(91.0, 0.0).is_valid());
        assert!(!GeoCoord::new(0.0, 181.0).is_valid());
        assert!(!GeoCoord::new(-91.0, 0.0).is_valid());
    }

    #[test]
    fn test_kdtree_single_node() {
        let mut tree = KdTree::new();
        tree.insert(GeoCoord::new(40.0, -74.0), "nyc");
        let nn = tree.nearest_neighbor(&GeoCoord::new(40.1, -74.1));
        assert!(nn.is_some());
        let (id, dist) = nn.unwrap();
        assert_eq!(id, "nyc");
        assert!(dist < 0.2);
    }

    #[test]
    fn test_kdtree_multiple_nodes() {
        let mut tree = KdTree::new();
        tree.insert(GeoCoord::new(40.7, -74.0), "nyc");
        tree.insert(GeoCoord::new(34.05, -118.25), "la");
        tree.insert(GeoCoord::new(41.88, -87.63), "chi");
        let nn = tree.nearest_neighbor(&GeoCoord::new(40.71, -74.01));
        assert!(nn.is_some());
        assert_eq!(nn.unwrap().0, "nyc");
    }

    #[test]
    fn test_kdtree_build() {
        let nodes = vec![
            (GeoCoord::new(40.7, -74.0), "nyc".to_string()),
            (GeoCoord::new(34.05, -118.25), "la".to_string()),
            (GeoCoord::new(41.88, -87.63), "chi".to_string()),
        ];
        let tree = KdTree::build(&nodes);
        assert_eq!(tree.len(), 3);
        let nn = tree.nearest_neighbor(&GeoCoord::new(34.06, -118.26));
        assert!(nn.is_some());
        assert_eq!(nn.unwrap().0, "la");
    }

    #[test]
    fn test_kdtree_k_nearest() {
        let mut tree = KdTree::new();
        tree.insert(GeoCoord::new(40.7, -74.0), "nyc");
        tree.insert(GeoCoord::new(34.05, -118.25), "la");
        tree.insert(GeoCoord::new(41.88, -87.63), "chi");
        let knn = tree.k_nearest(&GeoCoord::new(40.71, -74.01), 2);
        assert_eq!(knn.len(), 2);
        assert_eq!(knn[0].0, "nyc");
    }

    #[test]
    fn test_kdtree_range_query() {
        let mut tree = KdTree::new();
        // Nodes all near NYC
        tree.insert(GeoCoord::new(40.7128, -74.0060), "nyc");
        tree.insert(GeoCoord::new(40.7580, -73.9855), "times_sq");
        tree.insert(GeoCoord::new(40.7484, -73.9857), "empire_state");
        // Far away
        tree.insert(GeoCoord::new(34.05, -118.25), "la");
        let inside = tree.range_query(&GeoCoord::new(40.71, -74.01), 10_000.0);
        assert_eq!(inside.len(), 3);
        assert!(inside.contains(&"nyc".to_string()));
        assert!(inside.contains(&"times_sq".to_string()));
        assert!(!inside.contains(&"la".to_string()));
    }

    #[test]
    fn test_kdtree_empty() {
        let tree = KdTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert!(tree.nearest_neighbor(&GeoCoord::new(0.0, 0.0)).is_none());
        assert!(tree.k_nearest(&GeoCoord::new(0.0, 0.0), 5).is_empty());
        assert!(tree
            .range_query(&GeoCoord::new(0.0, 0.0), 1000.0)
            .is_empty());
    }

    #[test]
    fn test_spatial_graph_add_and_get() {
        let mut graph = SpatialGraph::new();
        let node = SpatialNode::new(
            "nyc",
            40.7128,
            -74.0060,
            "New York City",
            SpatialFeatureType::City,
        );
        graph.add_node(node);
        let retrieved = graph.get_node("nyc");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "New York City");
        assert_eq!(graph.len(), 1);
    }

    #[test]
    fn test_spatial_graph_find_nearby() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new(
            "nyc",
            40.7128,
            -74.0060,
            "NYC",
            SpatialFeatureType::City,
        ));
        graph.add_node(SpatialNode::new(
            "times_sq",
            40.7580,
            -73.9855,
            "Times Square",
            SpatialFeatureType::POI,
        ));
        graph.add_node(SpatialNode::new(
            "la",
            34.05,
            -118.25,
            "Los Angeles",
            SpatialFeatureType::City,
        ));
        let nearby = graph.find_nearby(&GeoCoord::new(40.71, -74.01), 10_000.0);
        assert_eq!(nearby.len(), 2);
        let names: Vec<&str> = nearby.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"NYC"));
        assert!(names.contains(&"Times Square"));
        assert!(!names.contains(&"Los Angeles"));
    }

    #[test]
    fn test_spatial_graph_find_nearest() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new(
            "nyc",
            40.7128,
            -74.0060,
            "NYC",
            SpatialFeatureType::City,
        ));
        graph.add_node(SpatialNode::new(
            "la",
            34.05,
            -118.25,
            "LA",
            SpatialFeatureType::City,
        ));
        graph.add_node(SpatialNode::new(
            "chi",
            41.88,
            -87.63,
            "Chicago",
            SpatialFeatureType::City,
        ));
        let nearest = graph.find_nearest(&GeoCoord::new(40.71, -74.01), 2);
        assert_eq!(nearest.len(), 2);
        assert_eq!(nearest[0].name, "NYC");
    }

    #[test]
    fn test_spatial_graph_find_by_type() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new(
            "nyc",
            40.7128,
            -74.0060,
            "NYC",
            SpatialFeatureType::City,
        ));
        graph.add_node(SpatialNode::new(
            "cp",
            40.78,
            -73.97,
            "Central Park",
            SpatialFeatureType::Park,
        ));
        graph.add_node(SpatialNode::new(
            "la",
            34.05,
            -118.25,
            "LA",
            SpatialFeatureType::City,
        ));
        let cities = graph.find_by_type(&SpatialFeatureType::City);
        assert_eq!(cities.len(), 2);
        let parks = graph.find_by_type(&SpatialFeatureType::Park);
        assert_eq!(parks.len(), 1);
    }

    #[test]
    fn test_spatial_graph_find_by_tag() {
        let mut graph = SpatialGraph::new();
        let mut tags = HashMap::new();
        tags.insert("capital".to_string(), "true".to_string());
        let node = SpatialNode::new(
            "dc",
            38.9072,
            -77.0369,
            "Washington DC",
            SpatialFeatureType::City,
        )
        .with_tags(tags);
        graph.add_node(node);
        graph.add_node(SpatialNode::new(
            "nyc",
            40.7128,
            -74.0060,
            "NYC",
            SpatialFeatureType::City,
        ));
        let capitals = graph.find_by_tag("capital", "true");
        assert_eq!(capitals.len(), 1);
        assert_eq!(capitals[0].name, "Washington DC");
    }

    #[test]
    fn test_spatial_graph_relations() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new(
            "nyc",
            40.7128,
            -74.0060,
            "NYC",
            SpatialFeatureType::City,
        ));
        graph.add_node(SpatialNode::new(
            "cp",
            40.78,
            -73.97,
            "Central Park",
            SpatialFeatureType::Park,
        ));
        graph.add_relation("nyc", "cp", SpatialRelation::Contains);
        let rels = graph.get_relations("cp");
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].0, "nyc");
    }

    #[test]
    fn test_spatial_graph_remove_node() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new(
            "nyc",
            40.7128,
            -74.0060,
            "NYC",
            SpatialFeatureType::City,
        ));
        graph.add_node(SpatialNode::new(
            "la",
            34.05,
            -118.25,
            "LA",
            SpatialFeatureType::City,
        ));
        assert_eq!(graph.len(), 2);
        let removed = graph.remove_node("nyc");
        assert!(removed.is_some());
        assert_eq!(graph.len(), 1);
        assert!(graph.get_node("nyc").is_none());
    }

    #[test]
    fn test_spatial_graph_export_geojson() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new(
            "nyc",
            40.7128,
            -74.0060,
            "NYC",
            SpatialFeatureType::City,
        ));
        graph.add_node(SpatialNode::new(
            "la",
            34.05,
            -118.25,
            "LA",
            SpatialFeatureType::City,
        ));
        let json_str = graph.export_geojson();
        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value["type"], "FeatureCollection");
        let features = value["features"].as_array().unwrap();
        assert_eq!(features.len(), 2);
        let coords = features[0]["geometry"]["coordinates"].as_array().unwrap();
        assert_eq!(coords.len(), 3); // lng, lat, alt (empty string for none)
    }

    #[test]
    fn test_spatial_graph_import_geojson() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": { "type": "Point", "coordinates": [-74.006, 40.7128] },
                    "properties": { "id": "nyc", "name": "NYC" }
                },
                {
                    "type": "Feature",
                    "geometry": { "type": "Point", "coordinates": [-118.25, 34.05] },
                    "properties": { "id": "la", "name": "LA" }
                }
            ]
        }"#;
        let count = SpatialGraph::import_geojson(geojson).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_spatial_graph_import_geojson_invalid() {
        // Missing features array
        let err = SpatialGraph::import_geojson(r#"{"type": "NotAFeatureCollection"}"#);
        assert!(err.is_err());

        // Empty features
        let count =
            SpatialGraph::import_geojson(r#"{"type":"FeatureCollection","features":[]}"#).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_spatial_feature_type_as_str() {
        assert_eq!(SpatialFeatureType::City.as_str(), "city");
        assert_eq!(SpatialFeatureType::POI.as_str(), "poi");
        assert_eq!(
            SpatialFeatureType::Custom("museum".to_string()).as_str(),
            "museum"
        );
    }
}
