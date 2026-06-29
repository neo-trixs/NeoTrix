use super::*;
use ops;

#[test]
fn test_hdlib_similarity_self() {
    let a = ops::random(HD_DIM);
    let sim = ops::similarity(&a, &a);
    assert!(
        (sim - 1.0).abs() < 1e-10,
        "self-similarity should be 1.0, got {}",
        sim
    );
}

#[test]
fn test_hdlib_similarity_orthogonal() {
    // Two axis-aligned unit vectors in first 2 dims: [1,0,...] and [0,1,...]
    let mut a = vec![0.0; HD_DIM];
    let mut b = vec![0.0; HD_DIM];
    a[0] = 1.0;
    b[1] = 1.0;
    let sim = ops::similarity(&a, &b);
    assert!(
        (sim - 0.0).abs() < 1e-10,
        "orthogonal vectors should have 0 similarity, got {}",
        sim
    );
}

#[test]
fn test_hdlib_bundle_basics() {
    let a = ops::random(HD_DIM);
    let b = ops::random(HD_DIM);
    let bundled = ops::bundle(&[&a, &b]);
    // Bundled vector should be similar to both components
    let sim_a = ops::similarity(&bundled, &a);
    let sim_b = ops::similarity(&bundled, &b);
    assert!(sim_a > 0.3, "bundle should be similar to a, got {}", sim_a);
    assert!(sim_b > 0.3, "bundle should be similar to b, got {}", sim_b);
}

#[test]
fn test_hdlib_bind_unbind_roundtrip() {
    let a = ops::random(HD_DIM);
    let b = ops::random(HD_DIM);
    let bound = ops::bind(&a, &b);
    let rebound = ops::unbind(&bound, &b);
    let sim = ops::similarity(&rebound, &a);
    // For MAP-VSA (element-wise multiply), unbind is self-inverse for bipolar
    // For real-valued vectors this is approximate
    assert!(
        sim > 0.5,
        "bind/unbind roundtrip should recover a, sim={}",
        sim
    );
}

#[test]
fn test_hdlib_bind_bundle_combination() {
    // Verify that bind distributes approximately over bundle
    let a = ops::random(HD_DIM);
    let b = ops::random(HD_DIM);
    let c = ops::random(HD_DIM);
    // bind(a, bundle(b, c)) vs bundle(bind(a,b), bind(a,c))
    let bound_bc = ops::bind(&a, &ops::bundle(&[&b, &c]));
    let bundle_ab_ac = ops::bundle(&[&ops::bind(&a, &b), &ops::bind(&a, &c)]);
    let sim = ops::similarity(&bound_bc, &bundle_ab_ac);
    assert!(
        sim > 0.8,
        "bind should approximately distribute over bundle, sim={}",
        sim
    );
}

#[test]
fn test_hdlib_permute_reversible() {
    let v = ops::random(HD_DIM);
    let p = ops::permute(&v, 100);
    let r = ops::permute(&p, -100);
    let sim = ops::similarity(&r, &v);
    assert!(
        (sim - 1.0).abs() < 1e-10,
        "permute/permute roundtrip should be identity, sim={}",
        sim
    );
}

#[test]
fn test_hdlib_dot_product() {
    let a = ops::random(HD_DIM);
    let b = ops::random(HD_DIM);
    let d = ops::dot(&a, &b);
    assert!(d.is_finite(), "dot product should be finite");
}

#[test]
fn test_hdlib_seeded_stability() {
    let a = ops::seeded_random(42, HD_DIM);
    let b = ops::seeded_random(42, HD_DIM);
    let sim = ops::similarity(&a, &b);
    assert!(
        (sim - 1.0).abs() < 1e-10,
        "same seed should produce same vector, sim={}",
        sim
    );
}

#[test]
fn test_hdlib_l2_normalize() {
    let v = ops::random(HD_DIM);
    let n = ops::l2_normalize(&v);
    let norm: f64 = n.iter().map(|x| x * x).sum::<f64>().sqrt();
    assert!(
        (norm - 1.0).abs() < 1e-10,
        "normalized vector should have unit length, got {}",
        norm
    );
}

#[test]
fn test_hdlib_default_engine() {
    let engine = ops::default_engine();
    assert_eq!(engine.dimensions(), HD_DIM);
    assert_eq!(engine.name(), "map-vsa");
}

#[test]
fn test_hdlib_bundle_empty() {
    let bundled = ops::bundle(&[]);
    assert_eq!(bundled.len(), HD_DIM);
    assert!(
        bundled.iter().all(|&x| x == 0.0),
        "empty bundle should be all zeros"
    );
}

#[test]
fn test_hdlib_bind_distinct_from_inputs() {
    let a = ops::random(HD_DIM);
    let b = ops::random(HD_DIM);
    let bound = ops::bind(&a, &b);
    let sim_a = ops::similarity(&bound, &a);
    let sim_b = ops::similarity(&bound, &b);
    assert!(
        sim_a.abs() < 0.15,
        "bound vector should be dissimilar to a, sim={}",
        sim_a
    );
    assert!(
        sim_b.abs() < 0.15,
        "bound vector should be dissimilar to b, sim={}",
        sim_b
    );
}

#[test]
fn test_hdlib_permute_different_from_original() {
    let v = ops::random(HD_DIM);
    let p = ops::permute(&v, 1);
    let sim = ops::similarity(&v, &p);
    assert!(
        sim.abs() < 0.1,
        "permuted vector should be dissimilar to original, sim={}",
        sim
    );
}

#[test]
fn test_hdlib_cosine_similarity_alias() {
    let a = ops::random(HD_DIM);
    let b = ops::random(HD_DIM);
    let s1 = ops::similarity(&a, &b);
    let s2 = ops::cosine_similarity(&a, &b);
    assert!(
        (s1 - s2).abs() < 1e-15,
        "cosine_similarity should match similarity"
    );
}
