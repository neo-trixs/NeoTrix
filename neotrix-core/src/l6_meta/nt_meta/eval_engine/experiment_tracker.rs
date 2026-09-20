#![deny(clippy::unwrap_used)]

/// A variant (arm) in an A/B experiment.
#[derive(Debug, Clone)]
pub struct ExperimentVariant {
    pub name: String,
    pub results: Vec<f64>,
}

impl ExperimentVariant {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            results: Vec::new(),
        }
    }
}

/// An A/B experiment comparing multiple variants.
#[derive(Debug, Clone)]
pub struct Experiment {
    pub name: String,
    pub variants: Vec<ExperimentVariant>,
}

impl Experiment {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            variants: Vec::new(),
        }
    }

    pub fn add_variant(&mut self, variant: ExperimentVariant) {
        self.variants.push(variant);
    }
}

/// Compute the arithmetic mean of a slice of values.
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Compute the sample standard deviation of a slice of values.
pub fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    let variance = values.iter().map(|&x| (x - m).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

/// Compare all variant pairs using Welch's t-test.
///
/// Returns a list of `(variant_a, variant_b, p_value)` for each unique pair.
pub fn compare_variants(exp: &Experiment) -> Vec<(String, String, f64)> {
    let mut results = Vec::new();

    for i in 0..exp.variants.len() {
        for j in (i + 1)..exp.variants.len() {
            let a = &exp.variants[i];
            let b = &exp.variants[j];
            let p = welch_t_test(&a.results, &b.results);
            results.push((a.name.clone(), b.name.clone(), p));
        }
    }

    results
}

/// Welch's t-test for two independent samples with unequal variances.
fn welch_t_test(a: &[f64], b: &[f64]) -> f64 {
    let n_a = a.len() as f64;
    let n_b = b.len() as f64;

    if n_a < 2.0 || n_b < 2.0 {
        return 1.0;
    }

    let mean_a = mean(a);
    let mean_b = mean(b);

    let var_a = a.iter().map(|&x| (x - mean_a).powi(2)).sum::<f64>() / (n_a - 1.0);
    let var_b = b.iter().map(|&x| (x - mean_b).powi(2)).sum::<f64>() / (n_b - 1.0);

    let se = (var_a / n_a + var_b / n_b).sqrt();
    if se == 0.0 {
        return 1.0;
    }

    let t = (mean_a - mean_b) / se;

    // Welch-Satterthwaite degrees of freedom
    let num = (var_a / n_a + var_b / n_b).powi(2);
    let den = (var_a / n_a).powi(2) / (n_a - 1.0) + (var_b / n_b).powi(2) / (n_b - 1.0);
    let df = if den > 0.0 { num / den } else { n_a + n_b - 2.0 };

    approximate_p_value(t.abs(), df)
}

/// Approximate two-tailed p-value using normal approximation for large df,
/// and heuristic for small df.
fn approximate_p_value(abs_t: f64, df: f64) -> f64 {
    if df > 30.0 {
        let x = abs_t / 2.0_f64.sqrt();
        let t = 1.0 / (1.0 + 0.3275911 * x);
        let poly = t
            * (0.254829592
                + t * (-0.284496736
                    + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
        return (-x * x).exp() * poly;
    }

    if abs_t < 1.0 {
        0.5
    } else if abs_t < 2.0 {
        0.15
    } else if abs_t < 3.0 {
        0.05
    } else {
        0.01
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_empty() {
        assert_eq!(mean(&[]), 0.0);
    }

    #[test]
    fn test_mean_single() {
        assert_eq!(mean(&[5.0]), 5.0);
    }

    #[test]
    fn test_mean_basic() {
        assert!((mean(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev_empty() {
        assert_eq!(std_dev(&[]), 0.0);
    }

    #[test]
    fn test_std_dev_single() {
        assert_eq!(std_dev(&[5.0]), 0.0);
    }

    #[test]
    fn test_std_dev_known_values() {
        let values = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let sd = std_dev(&values);
        // Population stddev ≈ 2.0, sample stddev ≈ 2.138
        assert!((sd - 2.1380899).abs() < 0.001);
    }

    #[test]
    fn test_compare_variants_empty() {
        let exp = Experiment::new("test");
        let results = compare_variants(&exp);
        assert!(results.is_empty());
    }

    #[test]
    fn test_compare_variants_single_variant() {
        let mut exp = Experiment::new("test");
        exp.add_variant(ExperimentVariant {
            name: "a".into(),
            results: vec![1.0, 2.0, 3.0],
        });
        let results = compare_variants(&exp);
        assert!(results.is_empty());
    }

    #[test]
    fn test_compare_variants_two_similar() {
        let mut exp = Experiment::new("test");
        exp.add_variant(ExperimentVariant {
            name: "control".into(),
            results: vec![10.0, 10.0, 10.0, 10.0, 10.0],
        });
        exp.add_variant(ExperimentVariant {
            name: "treatment".into(),
            results: vec![10.0, 10.0, 10.0, 10.0, 10.0],
        });
        let results = compare_variants(&exp);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "control");
        assert_eq!(results[0].1, "treatment");
        // Identical distributions → p ≈ 1.0
        assert!((results[0].2 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_compare_variants_two_different() {
        let mut exp = Experiment::new("test");
        exp.add_variant(ExperimentVariant {
            name: "low".into(),
            results: vec![1.0, 2.0, 3.0, 1.0, 2.0],
        });
        exp.add_variant(ExperimentVariant {
            name: "high".into(),
            results: vec![8.0, 9.0, 10.0, 8.0, 9.0],
        });
        let results = compare_variants(&exp);
        assert_eq!(results.len(), 1);
        // Very different → small p-value
        assert!(results[0].2 < 0.05);
    }

    #[test]
    fn test_compare_variants_three_pairs() {
        let mut exp = Experiment::new("test");
        exp.add_variant(ExperimentVariant {
            name: "a".into(),
            results: vec![1.0, 2.0],
        });
        exp.add_variant(ExperimentVariant {
            name: "b".into(),
            results: vec![3.0, 4.0],
        });
        exp.add_variant(ExperimentVariant {
            name: "c".into(),
            results: vec![5.0, 6.0],
        });
        let results = compare_variants(&exp);
        assert_eq!(results.len(), 3); // (a,b), (a,c), (b,c)
    }

    #[test]
    fn test_welch_t_test_insufficient_data() {
        let p = welch_t_test(&[1.0], &[2.0]);
        assert_eq!(p, 1.0);
    }
}
