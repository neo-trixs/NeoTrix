//! Healing subsystem integration tests for NeoTrix.
//!
//! Tests health monitoring, auto-repair, diagnostic chains, circuit breakers,
//! and resilience management.

use std::sync::Arc;
use std::time::Duration;

use neotrix::l6_meta::healing::self_healing::auto_repair::{AutoRepair, FailureSeverity};
use neotrix::l6_meta::healing::self_healing::circuit_breaker::{CircuitBreaker, CircuitState};
use neotrix::l6_meta::healing::self_healing::health_monitor::{
    ComponentMonitor, HealthStatusKind,
};
use neotrix::l6_meta::healing::self_healing::resilience::ResilienceManager;

#[test]
fn test_health_monitor_detect_failure_auto_repair_verify() {
    let mut monitor = ComponentMonitor::new();
    monitor.register_component("fake_db");
    let status = monitor.check_component("fake_db");
    assert_eq!(status.status, HealthStatusKind::Critical);

    let repair = AutoRepair::new(monitor);
    let failures = repair.detect_failures();
    assert!(!failures.is_empty());

    let db_failure = failures.iter().find(|f| f.component == "fake_db").unwrap();
    assert_eq!(db_failure.severity, FailureSeverity::High);

    let heal_results = repair.auto_heal();
    assert!(!heal_results.is_empty());
    assert!(heal_results.iter().all(|r| r.success));
}

#[test]
fn test_circuit_breaker_trip_reset_resume() {
    let cb = CircuitBreaker::new(3, Duration::from_millis(50));
    assert_eq!(cb.state(), CircuitState::Closed);

    let _ = cb.call(|| Err::<String, _>("error 1"));
    let _ = cb.call(|| Err::<String, _>("error 2"));
    let _ = cb.call(|| Err::<String, _>("error 3"));
    assert_eq!(cb.state(), CircuitState::Open);

    let result = cb.call(|| Ok::<String, String>("should not run".to_string()));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("OPEN"));

    std::thread::sleep(Duration::from_millis(60));
    assert_eq!(cb.state(), CircuitState::HalfOpen);

    let _ = cb.call(|| Ok::<String, String>("recovered".to_string()));
    assert_eq!(cb.state(), CircuitState::Closed);
    assert_eq!(cb.consecutive_failures(), 0);
}

#[test]
fn test_circuit_breaker_manual_reset() {
    let cb = CircuitBreaker::new(2, Duration::from_secs(60));
    let _ = cb.call(|| Err::<String, _>("fail"));
    let _ = cb.call(|| Err::<String, _>("fail"));
    assert_eq!(cb.state(), CircuitState::Open);

    cb.reset();
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[test]
fn test_resilience_manager_with_circuit_breaker() {
    let manager = ResilienceManager::new()
        .with_retry(3, Duration::from_millis(10))
        .with_circuit_breaker(3);

    let result = manager.execute(|| Ok::<_, String>("success".to_string()));
    assert!(result.success);
    assert_eq!(result.value, Some("success".to_string()));
    assert_eq!(result.attempts, 1);
    assert_eq!(result.circuit_state, CircuitState::Closed);
}

#[test]
fn test_resilience_manager_persistent_failure() {
    let manager = ResilienceManager::new()
        .with_retry(3, Duration::from_millis(10))
        .with_circuit_breaker(3);

    let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let cc = call_count.clone();
    let result = manager.execute(move || {
        cc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Err::<String, _>("persistent failure".to_string())
    });
    assert!(!result.success);
    assert!(result.attempts <= 3);
}

#[test]
fn test_health_status_kind_ordering() {
    assert!(HealthStatusKind::Healthy < HealthStatusKind::Degraded);
    assert!(HealthStatusKind::Degraded < HealthStatusKind::Critical);
}

#[test]
fn test_health_status_kind_display() {
    assert_eq!(format!("{}", HealthStatusKind::Healthy), "Healthy");
    assert_eq!(format!("{}", HealthStatusKind::Degraded), "Degraded");
    assert_eq!(format!("{}", HealthStatusKind::Critical), "Critical");
}

#[test]
fn test_circuit_breaker_success_resets_count() {
    let cb = CircuitBreaker::new(3, Duration::from_millis(100));

    let _ = cb.call(|| Err::<String, _>("fail1"));
    let _ = cb.call(|| Err::<String, _>("fail2"));

    let _ = cb.call(|| Ok::<String, String>("success".to_string()));
    assert_eq!(cb.consecutive_failures(), 0);
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[test]
fn test_auto_repair_multiple_components() {
    let mut monitor = ComponentMonitor::new();
    monitor.register_component("db");
    monitor.register_component("cache");
    monitor.register_component("queue");

    let repair = AutoRepair::new(monitor);
    let failures = repair.detect_failures();
    assert!(failures.len() >= 2);

    let heal_results = repair.auto_heal();
    assert!(heal_results.len() >= 2);
}
