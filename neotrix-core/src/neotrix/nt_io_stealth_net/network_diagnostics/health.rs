use super::types::{HealthGrade, HealthScore};

pub fn compute_health(
    p95_latency_ms: f64,
    latency_slo_ms: f64,
    error_rate: f64,
    error_budget_rate: f64,
    dns_failure_rate: f64,
) -> HealthScore {
    let latency_score = 1.0 - (p95_latency_ms / latency_slo_ms).clamp(0.0, 1.0);
    let error_score = 1.0 - (error_rate / error_budget_rate.max(0.01)).clamp(0.0, 1.0);
    let dns_score = 1.0 - dns_failure_rate.clamp(0.0, 1.0);

    const W_LATENCY: f64 = 0.45;
    const W_ERROR: f64 = 0.35;
    const W_DNS: f64 = 0.20;

    let overall = W_LATENCY * latency_score + W_ERROR * error_score + W_DNS * dns_score;

    let grade = if overall > 0.8 {
        HealthGrade::Green
    } else if overall > 0.5 {
        HealthGrade::Yellow
    } else {
        HealthGrade::Red
    };

    HealthScore {
        overall,
        latency_score,
        error_score,
        dns_score,
        grade,
    }
}
