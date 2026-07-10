#![forbid(unsafe_code)]

pub struct FuzzyOperator;

impl FuzzyOperator {
    pub fn and(a: f64, b: f64) -> f64 {
        a.min(b).max(0.0).min(1.0)
    }

    pub fn or(a: f64, b: f64) -> f64 {
        a.max(b).max(0.0).min(1.0)
    }

    pub fn not(a: f64) -> f64 {
        (1.0 - a).max(0.0).min(1.0)
    }

    pub fn implication(a: f64, b: f64) -> f64 {
        (1.0 - a + b).min(1.0).max(0.0).min(1.0)
    }

    pub fn bounded_sum(a: f64, b: f64) -> f64 {
        (a + b).min(1.0).max(0.0).min(1.0)
    }

    pub fn bounded_product(a: f64, b: f64) -> f64 {
        (a + b - 1.0).max(0.0).min(1.0)
    }

    pub fn drastic_sum(a: f64, b: f64) -> f64 {
        let result = if a == 0.0 {
            b
        } else if b == 0.0 {
            a
        } else {
            1.0
        };
        result.max(0.0).min(1.0)
    }

    pub fn drastic_product(a: f64, b: f64) -> f64 {
        let result = if a == 1.0 {
            b
        } else if b == 1.0 {
            a
        } else {
            0.0
        };
        result.max(0.0).min(1.0)
    }

    pub fn hamacher_product(a: f64, b: f64) -> f64 {
        let denom = a + b - a * b;
        let result = if denom == 0.0 { 0.0 } else { (a * b) / denom };
        result.max(0.0).min(1.0)
    }

    pub fn einstein_sum(a: f64, b: f64) -> f64 {
        let result = (a + b) / (1.0 + a * b);
        result.max(0.0).min(1.0)
    }

    pub fn defuzzify(values: &[(f64, f64)]) -> f64 {
        let total_weight: f64 = values.iter().map(|(_, w)| w).sum();
        if total_weight == 0.0 {
            return 0.0;
        }
        let weighted_sum: f64 = values.iter().map(|(v, w)| v * w).sum();
        (weighted_sum / total_weight).max(0.0).min(1.0)
    }

    pub fn fuzzy_compare(a: f64, b: f64, threshold: f64) -> bool {
        (a - b).abs() <= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and() {
        assert!((FuzzyOperator::and(0.5, 0.3) - 0.3).abs() < 1e-9);
        assert!((FuzzyOperator::and(0.0, 1.0) - 0.0).abs() < 1e-9);
        assert!((FuzzyOperator::and(1.0, 1.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_or() {
        assert!((FuzzyOperator::or(0.5, 0.3) - 0.5).abs() < 1e-9);
        assert!((FuzzyOperator::or(0.0, 0.0) - 0.0).abs() < 1e-9);
        assert!((FuzzyOperator::or(0.0, 1.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_not() {
        assert!((FuzzyOperator::not(0.0) - 1.0).abs() < 1e-9);
        assert!((FuzzyOperator::not(1.0) - 0.0).abs() < 1e-9);
        assert!((FuzzyOperator::not(0.3) - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_implication() {
        assert!((FuzzyOperator::implication(0.0, 0.0) - 1.0).abs() < 1e-9);
        assert!((FuzzyOperator::implication(1.0, 0.0) - 0.0).abs() < 1e-9);
        assert!((FuzzyOperator::implication(0.5, 0.7) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_bounded_sum() {
        assert!((FuzzyOperator::bounded_sum(0.7, 0.5) - 1.0).abs() < 1e-9);
        assert!((FuzzyOperator::bounded_sum(0.3, 0.2) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_bounded_product() {
        assert!((FuzzyOperator::bounded_product(0.7, 0.5) - 0.2).abs() < 1e-9);
        assert!((FuzzyOperator::bounded_product(0.3, 0.2) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_drastic_sum() {
        assert!((FuzzyOperator::drastic_sum(0.0, 0.7) - 0.7).abs() < 1e-9);
        assert!((FuzzyOperator::drastic_sum(0.5, 0.0) - 0.5).abs() < 1e-9);
        assert!((FuzzyOperator::drastic_sum(0.5, 0.3) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_drastic_product() {
        assert!((FuzzyOperator::drastic_product(1.0, 0.7) - 0.7).abs() < 1e-9);
        assert!((FuzzyOperator::drastic_product(0.5, 1.0) - 0.5).abs() < 1e-9);
        assert!((FuzzyOperator::drastic_product(0.5, 0.3) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_hamacher_product() {
        let v = FuzzyOperator::hamacher_product(0.8, 0.5);
        assert!(v > 0.0 && v <= 1.0);
        assert!((FuzzyOperator::hamacher_product(0.0, 0.5) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_einstein_sum() {
        let v = FuzzyOperator::einstein_sum(0.5, 0.5);
        assert!(v > 0.0 && v <= 1.0);
        assert!((FuzzyOperator::einstein_sum(0.0, 0.5) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_defuzzify() {
        let values = vec![(0.2, 1.0), (0.8, 1.0)];
        let centroid = FuzzyOperator::defuzzify(&values);
        assert!((centroid - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_defuzzify_empty() {
        assert!((FuzzyOperator::defuzzify(&[]) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_fuzzy_compare() {
        assert!(FuzzyOperator::fuzzy_compare(0.5, 0.51, 0.05));
        assert!(!FuzzyOperator::fuzzy_compare(0.5, 0.6, 0.05));
    }

    #[test]
    fn test_clamping() {
        assert!((FuzzyOperator::and(-0.5, 1.5) - 0.0).abs() < 1e-9);
        assert!((FuzzyOperator::or(2.0, -1.0) - 1.0).abs() < 1e-9);
    }
}
