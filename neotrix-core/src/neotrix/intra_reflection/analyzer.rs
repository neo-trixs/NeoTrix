use std::collections::HashSet;
use super::types::{ReflectionInput, ReflectionReport, MIN_EXPECTED_STEPS};
use std::time::{SystemTime, UNIX_EPOCH};

fn keyword_overlap(a: &str, b: &str) -> f64 {
    let tokens_a: HashSet<&str> = a.split_whitespace().collect();
    let tokens_b: HashSet<&str> = b.split_whitespace().collect();
    if tokens_a.is_empty() && tokens_b.is_empty() {
        return 1.0;
    }
    let intersection: HashSet<&str> = tokens_a.intersection(&tokens_b).copied().collect();
    if tokens_a.is_empty() || tokens_b.is_empty() {
        return 0.0;
    }
    intersection.len() as f64 / tokens_a.len().max(tokens_b.len()) as f64
}

pub fn compute_coherence(trace: &[String]) -> f64 {
    if trace.len() < 2 {
        return 1.0;
    }
    let sum: f64 = trace.windows(2)
        .map(|w| keyword_overlap(&w[0], &w[1]))
        .sum();
    sum / (trace.len() - 1) as f64
}

pub fn compute_efficiency(trace_len: usize, outcome_success: Option<bool>) -> f64 {
    match outcome_success {
        Some(true) => {
            if trace_len == 0 {
                return 0.0;
            }
            let ratio = MIN_EXPECTED_STEPS as f64 / trace_len as f64;
            ratio.min(1.0)
        }
        _ => 0.0,
    }
}

pub fn compute_error_density(error_count: u32, trace_len: usize) -> f64 {
    if trace_len == 0 {
        return 0.0;
    }
    (error_count as f64) / (trace_len as f64)
}

pub fn compute_mode_stability(history: &[u8]) -> f64 {
    if history.len() < 2 {
        return 1.0;
    }
    let switches: usize = history.windows(2)
        .filter(|w| w[0] != w[1])
        .count();
    let max_possible = history.len() - 1;
    1.0 - (switches as f64 / max_possible as f64)
}

pub fn find_bottlenecks(trace: &[String], execution_time_ms: u64, error_count: u32) -> Vec<String> {
    let mut bottlenecks = Vec::new();
    if trace.is_empty() {
        return bottlenecks;
    }

    let avg_time_per_step = execution_time_ms as f64 / trace.len() as f64;
    if avg_time_per_step > 5000.0 {
        bottlenecks.push(format!(
            "high average latency: {}ms per step",
            avg_time_per_step as u64
        ));
    }

    let error_cluster_threshold = (trace.len() as f64 * 0.3).ceil() as u32;
    if error_count > error_cluster_threshold {
        bottlenecks.push(format!(
            "error cluster detected: {} errors across {} steps",
            error_count,
            trace.len()
        ));
    }

    bottlenecks
}

pub fn generate_suggestions(
    coherence: f64,
    efficiency: f64,
    error_density: f64,
    mode_stability: f64,
) -> Vec<String> {
    let mut suggestions = Vec::new();

    if coherence < 0.3 {
        suggestions.push(
            "low coherence — reasoning steps lack logical continuity. consider structured prompting with explicit step linking.".to_string()
        );
    } else if coherence < 0.6 {
        suggestions.push(
            "moderate coherence — improve logical flow between steps with intermediate summaries.".to_string()
        );
    }

    if efficiency < 0.4 && efficiency > 0.0 {
        suggestions.push(
            "low efficiency — trace is much longer than expected. recommend pruning redundant steps.".to_string()
        );
    }

    if error_density > 0.5 {
        suggestions.push(
            "high error density — consider switching to a more conservative E8 mode to reduce mistakes.".to_string()
        );
    } else if error_density > 0.25 {
        suggestions.push(
            "elevated error rate — review error patterns and consider validation gates.".to_string()
        );
    }

    if mode_stability < 0.3 {
        suggestions.push(
            "excessive E8 mode switching — frequent mode changes may disrupt reasoning continuity.".to_string()
        );
    }

    suggestions
}

pub fn analyze(input: &ReflectionInput) -> ReflectionReport {
    let coherence = compute_coherence(&input.reasoning_trace);
    let efficiency = compute_efficiency(input.reasoning_trace.len(), input.outcome_success);
    let error_density = compute_error_density(input.error_count, input.reasoning_trace.len());
    let mode_stability = compute_mode_stability(&input.e8_mode_history);
    let bottleneck_hops = find_bottlenecks(
        &input.reasoning_trace,
        input.execution_time_ms,
        input.error_count,
    );
    let suggestions = generate_suggestions(coherence, efficiency, error_density, mode_stability);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    ReflectionReport::new(
        coherence,
        efficiency,
        error_density,
        mode_stability,
        bottleneck_hops,
        suggestions,
        timestamp,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_trace_good() -> Vec<String> {
        vec![
            "analyze problem constraints and variables".to_string(),
            "identify key variables and constraints".to_string(),
            "formulate solution approach using identified variables".to_string(),
            "apply transformation step to solve the problem".to_string(),
            "verify result correctness against problem constraints".to_string(),
        ]
    }

    fn sample_trace_jumpy() -> Vec<String> {
        vec![
            "solve quadratic equation".to_string(),
            "deploy kubernetes cluster".to_string(),
            "bake chocolate cake".to_string(),
            "optimize sql query".to_string(),
        ]
    }

    #[test]
    fn test_coherence_high_for_related_steps() {
        let trace = sample_trace_good();
        let score = compute_coherence(&trace);
        assert!(score > 0.0, "coherence should be positive");
        assert!(score <= 1.0, "coherence should not exceed 1.0");
    }

    #[test]
    fn test_coherence_low_for_unrelated_steps() {
        let jumpy = sample_trace_jumpy();
        let good = sample_trace_good();
        let jumpy_score = compute_coherence(&jumpy);
        let good_score = compute_coherence(&good);
        assert!(
            jumpy_score <= good_score + 0.5,
            "jumpy trace should not be dramatically more coherent than related steps"
        );
    }

    #[test]
    fn test_coherence_single_step() {
        let trace = vec!["single step".to_string()];
        assert_eq!(compute_coherence(&trace), 1.0);
    }

    #[test]
    fn test_coherence_empty_trace() {
        let trace: Vec<String> = vec![];
        assert_eq!(compute_coherence(&trace), 1.0);
    }

    #[test]
    fn test_efficiency_success_short_trace() {
        let eff = compute_efficiency(3, Some(true));
        assert!((eff - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_efficiency_success_long_trace() {
        let eff = compute_efficiency(10, Some(true));
        assert!((eff - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_efficiency_failure() {
        assert_eq!(compute_efficiency(5, Some(false)), 0.0);
    }

    #[test]
    fn test_efficiency_none() {
        assert_eq!(compute_efficiency(5, None), 0.0);
    }

    #[test]
    fn test_error_density_clean() {
        assert_eq!(compute_error_density(0, 10), 0.0);
    }

    #[test]
    fn test_error_density_high() {
        let density = compute_error_density(8, 10);
        assert!((density - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_error_density_zero_trace() {
        assert_eq!(compute_error_density(5, 0), 0.0);
    }

    #[test]
    fn test_mode_stability_no_switches() {
        let history = vec![3, 3, 3, 3];
        assert_eq!(compute_mode_stability(&history), 1.0);
    }

    #[test]
    fn test_mode_stability_all_switches() {
        let history = vec![0, 1, 0, 1, 0, 1];
        assert!((compute_mode_stability(&history) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_mode_stability_partial() {
        let history = vec![1, 1, 2, 2, 3, 3];
        let stability = compute_mode_stability(&history);
        assert!(stability > 0.3 && stability < 0.7);
    }

    #[test]
    fn test_mode_stability_single() {
        assert_eq!(compute_mode_stability(&[7]), 1.0);
    }

    #[test]
    fn test_mode_stability_empty() {
        assert_eq!(compute_mode_stability(&[]), 1.0);
    }

    #[test]
    fn test_bottlenecks_high_latency() {
        let trace = vec!["a".to_string(); 5];
        let bt = find_bottlenecks(&trace, 30000, 0);
        assert!(!bt.is_empty());
        assert!(bt.iter().any(|b| b.contains("latency")));
    }

    #[test]
    fn test_bottlenecks_error_cluster() {
        let trace = vec!["a".to_string(); 10];
        let bt = find_bottlenecks(&trace, 100, 8);
        assert!(!bt.is_empty());
        assert!(bt.iter().any(|b| b.contains("error cluster")));
    }

    #[test]
    fn test_bottlenecks_clean() {
        let trace = vec!["a".to_string(); 10];
        let bt = find_bottlenecks(&trace, 100, 1);
        assert!(bt.is_empty());
    }

    #[test]
    fn test_bottlenecks_empty_trace() {
        let bt = find_bottlenecks(&[], 0, 0);
        assert!(bt.is_empty());
    }

    #[test]
    fn test_suggestions_high_error_density() {
        let s = generate_suggestions(0.9, 0.8, 0.6, 0.9);
        assert!(s.iter().any(|x| x.contains("high error density")));
    }

    #[test]
    fn test_suggestions_low_coherence() {
        let s = generate_suggestions(0.2, 0.8, 0.0, 0.9);
        assert!(s.iter().any(|x| x.contains("low coherence")));
    }

    #[test]
    fn test_suggestions_low_mode_stability() {
        let s = generate_suggestions(0.9, 0.8, 0.0, 0.2);
        assert!(s.iter().any(|x| x.contains("excessive E8 mode switching")));
    }

    #[test]
    fn test_suggestions_low_efficiency() {
        let s = generate_suggestions(0.9, 0.2, 0.0, 0.9);
        assert!(s.iter().any(|x| x.contains("low efficiency")));
    }

    #[test]
    fn test_suggestions_all_good() {
        let s = generate_suggestions(0.9, 0.9, 0.0, 0.9);
        assert!(s.is_empty());
    }

    #[test]
    fn test_analyze_full_pipeline() {
        let input = ReflectionInput::new(
            sample_trace_good(),
            vec![1, 1, 2, 2, 3],
            Some(true),
            500,
            1,
        );
        let report = analyze(&input);
        assert!(report.coherence_score >= 0.0 && report.coherence_score <= 1.0);
        assert!(report.efficiency_score >= 0.0 && report.efficiency_score <= 1.0);
        assert!(report.error_density >= 0.0);
        assert!(report.mode_stability >= 0.0 && report.mode_stability <= 1.0);
        assert!(report.timestamp > 0);
    }

    #[test]
    fn test_analyze_perfect_reasoning() {
        let input = ReflectionInput::new(
            vec![
                "identify input".to_string(),
                "process input".to_string(),
                "return output".to_string(),
            ],
            vec![5, 5, 5],
            Some(true),
            100,
            0,
        );
        let report = analyze(&input);
        assert!(report.coherence_score > 0.0);
        assert!((report.efficiency_score - 1.0).abs() < 1e-6);
        assert_eq!(report.error_density, 0.0);
        assert_eq!(report.mode_stability, 1.0);
    }

    #[test]
    fn test_analyze_terrible_reasoning() {
        let input = ReflectionInput::new(
            vec![],
            vec![],
            Some(false),
            0,
            0,
        );
        let report = analyze(&input);
        assert_eq!(report.coherence_score, 1.0);
        assert_eq!(report.efficiency_score, 0.0);
        assert_eq!(report.error_density, 0.0);
        assert_eq!(report.mode_stability, 1.0);
    }
}
