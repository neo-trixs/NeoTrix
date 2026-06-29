use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

const SSP_DIM: usize = 1024;

#[derive(Debug, Clone)]
pub struct SpatialSceneEngine {
    _dim: usize,
}

impl Default for SpatialSceneEngine {
    fn default() -> Self {
        Self::new(SSP_DIM)
    }
}

impl SpatialSceneEngine {
    pub fn new(dim: usize) -> Self {
        Self { _dim: dim }
    }

    fn basis_vector(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, SSP_DIM)
    }

    pub fn encode_position(x: f32, y: f32, z: f32) -> Vec<u8> {
        let x_basis = Self::basis_vector(1001);
        let y_basis = Self::basis_vector(2002);
        let z_basis = Self::basis_vector(3003);

        let x_shift = (x.abs() * 100.0) as isize % SSP_DIM as isize;
        let y_shift = (y.abs() * 100.0) as isize % SSP_DIM as isize;
        let z_shift = (z.abs() * 100.0) as isize % SSP_DIM as isize;

        let x_vec = QuantizedVSA::permute(&x_basis, x_shift);
        let y_vec = QuantizedVSA::permute(&y_basis, y_shift);
        let z_vec = QuantizedVSA::permute(&z_basis, z_shift);

        let xy = QuantizedVSA::bind(&x_vec, &y_vec);
        QuantizedVSA::bind(&xy, &z_vec)
    }

    pub fn encode_object(name_seed: u64, position: (f32, f32, f32), category_seed: u64) -> Vec<u8> {
        let pos_vec = Self::encode_position(position.0, position.1, position.2);
        let id_vec = QuantizedVSA::seeded_random(name_seed, SSP_DIM);
        let cat_vec = QuantizedVSA::seeded_random(category_seed, SSP_DIM);
        QuantizedVSA::bundle(&[
            &QuantizedVSA::bind(&pos_vec, &id_vec),
            &QuantizedVSA::bind(&cat_vec, &id_vec),
        ])
    }

    pub fn spatial_similarity(a: &[u8], b: &[u8]) -> f64 {
        QuantizedVSA::similarity(a, b)
    }

    pub fn nearest_neighbor<'a>(
        query: &[u8],
        candidates: &[(&'a str, Vec<u8>)],
        top_k: usize,
    ) -> Vec<(&'a str, f64)> {
        let mut scored: Vec<_> = candidates
            .iter()
            .map(|(name, vec)| (*name, Self::spatial_similarity(query, vec)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).collect()
    }

    pub fn scene_contains(scene_vec: &[u8], object_vec: &[u8], threshold: f64) -> bool {
        Self::spatial_similarity(scene_vec, object_vec) > threshold
    }

    pub fn region_query<'a>(
        scene_objects: &'a [(&'a str, Vec<u8>, (f32, f32, f32))],
        center: (f32, f32, f32),
        radius: f32,
    ) -> Vec<&'a str> {
        scene_objects
            .iter()
            .filter(|(_, _, pos)| {
                let dx = pos.0 - center.0;
                let dy = pos.1 - center.1;
                let dz = pos.2 - center.2;
                (dx * dx + dy * dy + dz * dz).sqrt() <= radius
            })
            .map(|(name, _, _)| *name)
            .collect()
    }

    pub fn bundle_scene(objects: &[&[u8]]) -> Vec<u8> {
        let all: Vec<&[u8]> = objects.iter().copied().collect();
        QuantizedVSA::bundle(&all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_sphere() -> (&'static str, Vec<u8>, (f32, f32, f32)) {
        (
            "sphere",
            SpatialSceneEngine::encode_position(1.0, 2.0, 3.0),
            (1.0, 2.0, 3.0),
        )
    }

    fn test_cube() -> (&'static str, Vec<u8>, (f32, f32, f32)) {
        (
            "cube",
            SpatialSceneEngine::encode_position(4.0, 5.0, 6.0),
            (4.0, 5.0, 6.0),
        )
    }

    #[test]
    fn test_encode_position_unique() {
        let p1 = SpatialSceneEngine::encode_position(0.0, 0.0, 0.0);
        let p2 = SpatialSceneEngine::encode_position(1.0, 0.0, 0.0);
        let sim = SpatialSceneEngine::spatial_similarity(&p1, &p2);
        assert!(sim < 0.95, "distinct positions should be dissimilar");
    }

    #[test]
    fn test_encode_position_self_similar() {
        let p = SpatialSceneEngine::encode_position(1.5, 2.5, 3.5);
        let sim = SpatialSceneEngine::spatial_similarity(&p, &p);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_object_encoding_has_dim() {
        let obj = SpatialSceneEngine::encode_object(42, (1.0, 2.0, 3.0), 7);
        assert_eq!(obj.len(), SSP_DIM);
    }

    #[test]
    fn test_bundle_scene() {
        let objs = vec![
            SpatialSceneEngine::encode_object(1, (0.0, 0.0, 0.0), 10),
            SpatialSceneEngine::encode_object(2, (1.0, 1.0, 1.0), 20),
        ];
        let scene: Vec<&[u8]> = objs.iter().map(|o| o.as_slice()).collect();
        let scene_vec = SpatialSceneEngine::bundle_scene(&scene);
        assert_eq!(scene_vec.len(), SSP_DIM);
    }

    #[test]
    fn test_nearest_neighbor() {
        let p1 = SpatialSceneEngine::encode_position(0.0, 0.0, 0.0);
        let p2 = SpatialSceneEngine::encode_position(1.0, 0.0, 0.0);
        let p3 = SpatialSceneEngine::encode_position(10.0, 10.0, 10.0);
        let candidates = vec![("near", p1.clone()), ("mid", p2), ("far", p3)];
        let results = SpatialSceneEngine::nearest_neighbor(&p1, &candidates, 2);
        assert_eq!(results[0].0, "near");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_region_query() {
        let objects = vec![test_sphere(), test_cube()];
        let results = SpatialSceneEngine::region_query(&objects, (0.0, 0.0, 0.0), 5.0);
        assert!(
            !results.contains(&"cube"),
            "cube at (4,5,6) outside radius 5 from origin"
        );
    }

    #[test]
    fn test_scene_contains_threshold() {
        let obj = SpatialSceneEngine::encode_object(1, (0.0, 0.0, 0.0), 10);
        assert!(SpatialSceneEngine::scene_contains(&obj, &obj, 0.5));
    }

    #[test]
    fn test_position_nearby() {
        let p1 = SpatialSceneEngine::encode_position(1.0, 1.0, 1.0);
        let p2 = SpatialSceneEngine::encode_position(1.01, 1.01, 1.01);
        let sim = SpatialSceneEngine::spatial_similarity(&p1, &p2);
        assert!(sim > 0.5, "nearby positions should have high similarity");
    }
}
