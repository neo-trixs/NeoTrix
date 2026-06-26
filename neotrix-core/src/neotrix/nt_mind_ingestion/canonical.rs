use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Canonical ordering for E8 hexagrams (0..63) and GWT specialist names.
/// Ensures deterministic ordering for mode selection and fingerprinting.

/// Map each E8 hexagram (0..63) to its canonical position
pub fn canonical_hexagram_order() -> Vec<usize> {
    (0..64).collect()
}

/// Sort hexagram indices canonically (by ID)
pub fn sort_hexagrams_canonical(hexagrams: &mut [usize]) {
    hexagrams.sort_unstable();
}

/// Canonical GWT specialist names for deterministic ordering
pub const CANONICAL_SPECIALIST_NAMES: &[&str] = &[
    "attention",
    "decision",
    "emotion",
    "memory",
    "perception",
    "planning",
    "reasoning",
    "self_model",
    "social",
    "metacognition",
    "language",
];

/// Sort GWT specialist names canonically (by name)
pub fn sort_specialist_names(names: &mut Vec<String>) {
    names.sort();
}

/// Generate a deterministic catalog fingerprint from hexagram IDs and specialist names.
/// The fingerprint is order-independent: {10,20,30} yields the same result as {30,10,20}.
pub fn canonical_catalog_fingerprint(
    hexagram_ids: &[usize],
    specialist_names: &[String],
) -> String {
    let mut hex_sorted = hexagram_ids.to_vec();
    hex_sorted.sort_unstable();
    let mut spec_sorted = specialist_names.to_vec();
    spec_sorted.sort();
    let mut hasher = DefaultHasher::new();
    for h in &hex_sorted {
        h.hash(&mut hasher);
    }
    for s in &spec_sorted {
        s.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_hexagram_order_is_0_to_63() {
        let order = canonical_hexagram_order();
        assert_eq!(order.len(), 64);
        for i in 0..64 {
            assert_eq!(order[i], i);
        }
    }

    #[test]
    fn test_sort_hexagrams_canonical() {
        let mut hex = vec![5, 3, 1, 4, 2, 0];
        sort_hexagrams_canonical(&mut hex);
        assert_eq!(hex, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_sort_specialist_names() {
        let mut names = vec![
            "memory".to_string(),
            "attention".to_string(),
            "self_model".to_string(),
        ];
        sort_specialist_names(&mut names);
        assert_eq!(names, vec!["attention", "memory", "self_model"]);
    }

    #[test]
    fn test_canonical_fingerprint_deterministic() {
        let hex = vec![10, 20, 30];
        let specs = vec!["memory".to_string(), "attention".to_string()];
        let fp1 = canonical_catalog_fingerprint(&hex, &specs);
        let fp2 = canonical_catalog_fingerprint(&hex, &specs);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_canonical_fingerprint_order_independent() {
        let hex1 = vec![10, 20, 30];
        let hex2 = vec![30, 10, 20];
        let specs1 = vec!["memory".to_string(), "attention".to_string()];
        let specs2 = vec!["attention".to_string(), "memory".to_string()];
        let fp1 = canonical_catalog_fingerprint(&hex1, &specs1);
        let fp2 = canonical_catalog_fingerprint(&hex2, &specs2);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_canonical_fingerprint_different_on_change() {
        let hex = vec![10, 20, 30];
        let specs1 = vec!["memory".to_string()];
        let specs2 = vec!["memory".to_string(), "attention".to_string()];
        let fp1 = canonical_catalog_fingerprint(&hex, &specs1);
        let fp2 = canonical_catalog_fingerprint(&hex, &specs2);
        assert_ne!(fp1, fp2);
    }
}
