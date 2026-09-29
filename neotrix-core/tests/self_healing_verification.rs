#![forbid(unsafe_code)]

//! Comprehensive verification tests for the self-healing subsystem.
//!
//! Tests cover: health monitoring, auto-repair, circuit breaker lifecycle,
//! resilience (retry/timeout), predictive maintenance (anomaly/failure prediction),
//! and diagnostic chain execution.

use std::sync::Arc;
use std::time::Duration;

use neotrix::l6_meta::healing::self_healing::auto_repair::{AutoRepair, FailureSeverity};
use neotrix::l6_meta::healing::self_healing::circuit_breaker::{CircuitBreaker, CircuitState};
use neotrix::l6_meta::healing::diagnostic_chain::config::HealingConfig;
use neotrix::l6_meta::healing::diagnostic_chain::healing_loop::HealingLoop;
use neotrix::l6_meta::healing::diagnostic_chain::health_signal::HealthSignal;
use neotrix::l6_meta::healing::self_healing::health_monitor::{ComponentMonitor, HealthStatusKind};
use neotrix::l6_meta::healing::predictive_maintenance::anomaly::{AnomalyDetector, AnomalySeverity};
use neotrix::l6_meta::healing::predictive_maintenance::predictor::Predictor;
use neotrix::l6_meta::healing::self_healing::resilience::ResilienceManager;

// ---------------------------------------------------------------------------
// Test 1: health_monitor_detects_all_components
// ---------------------------------------------------------------------------

#[test]
fn health_monitor_detects_all_components() {
    let monitor = ComponentMonitor::new();

    let all = monitor.get_all_status();
    assert_eq!(all.len(), 5, "ComponentMonitor must register exactly 5 components");

    let expected_names = ["memory", "cpu", "disk", "network", "agent_pool"];
    for name in &expected_names {
        let status = monitor.check_component(name);
        assert_eq!(status.component, *name);
        assert!(
            matches!(
                status.status,
                HealthStatusKind::Healthy | HealthStatusKind::Degraded | HealthStatusKind::Critical
            ),
            "component '{}' returned invalid status kind",
            name
        );
        assert!(
            !status.metrics.is_empty(),
            "component '{}' returned empty metrics",
            name
        );
        assert!(
            status.last_check > 0,
            "component '{}' has zero last_check timestamp",
            name
        );
    }

    // Verify get_all_status returns all five names
    let names: Vec<&str> = all.iter().map(|s| s.component.as_str()).collect();
    for name in &expected_names {
        assert!(
            names.contains(name),
            "get_all_status missing component '{}'",
            name
        );
    }
}

// ---------------------------------------------------------------------------
// Test 2: auto_repair_fixes_known_failures
// ---------------------------------------------------------------------------

#[test]
fn auto_repair_fixes_known_failures() {
    // Register an unknown component — it will be reported as Critical
    let mut monitor = ComponentMonitor::new();
    monitor.register_component("mock_degraded_component");

    let repair = AutoRepair::new(monitor);

    // detect_failures should find the bogus component
    let failures = repair.detect_failures();
    assert!(
        !failures.is_empty(),
        "detect_failures should find at least one failure for a bogus component"
    );

    let target = failures
        .iter()
        .find(|f| f.component == "mock_degraded_component")
        .expect("mock_degraded_component failure not found");

    // Severity should be High (unknown components are Critical → mapped to High)
    assert_eq!(
        target.severity,
        FailureSeverity::High,
        "unknown component should be High severity"
    );

    // repair() should succeed
    let result = repair.repair(target);
    assert!(result.success, "repair should succeed for a degraded component");
    assert!(
        !result.action_taken.is_empty(),
        "repair action_taken should be non-empty"
    );
}

// ---------------------------------------------------------------------------
// Test 3: circuit_breaker_full_lifecycle
// ---------------------------------------------------------------------------

#[test]
fn circuit_breaker_full_lifecycle() {
    let cb = CircuitBreaker::new(3, Duration::from_millis(80));

    // Starts Closed
    assert_eq!(cb.state(), CircuitState::Closed, "should start Closed");

    // 3 consecutive failures → Open
    let _ = cb.call(|| Err::<String, _>("fail_1"));
    let _ = cb.call(|| Err::<String, _>("fail_2"));
    let _ = cb.call(|| Err::<String, _>("fail_3"));
    assert_eq!(cb.state(), CircuitState::Open, "should be Open after 3 failures");
    assert_eq!(cb.consecutive_failures(), 3);

    // Calls while Open are rejected
    let rejected = cb.call(|| Ok::<_, String>("blocked"));
    assert!(rejected.is_err(), "calls should be rejected while Open");
    assert!(
        rejected.unwrap_err().contains("OPEN"),
        "error message should mention OPEN"
    );

    // Wait for reset timeout → HalfOpen
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(cb.state(), CircuitState::HalfOpen, "should be HalfOpen after timeout");

    // Successful call in HalfOpen → Closed
    let recovered = cb.call(|| Ok::<_, String>("recovered"));
    assert!(recovered.is_ok(), "successful call in HalfOpen should succeed");
    assert_eq!(cb.state(), CircuitState::Closed, "should return to Closed after recovery");
    assert_eq!(cb.consecutive_failures(), 0, "failures should reset to 0");
    assert_eq!(cb.total_failures(), 3, "total_failures should remain 3");

    // Concurrent access should not panic
    let cb = Arc::new(CircuitBreaker::new(5, Duration::from_secs(60)));
    let mut handles = vec![];
    for i in 0..8 {
        let cb_clone = Arc::clone(&cb);
        handles.push(std::thread::spawn(move || {
            let result = cb_clone.call(|| Ok::<_, String>(format!("ok_{}", i)));
            assert!(result.is_ok(), "concurrent call should not panic or fail");
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(cb.total_calls(), 8, "all concurrent calls should be counted");
}

// ---------------------------------------------------------------------------
// Test 4: resilience_manager_retry_exhaustion
// ---------------------------------------------------------------------------

#[test]
fn resilience_manager_retry_exhaustion() {
    let rm = ResilienceManager::new()
        .with_retry(3, Duration::from_millis(10))
        .with_circuit_breaker(100); // high threshold to avoid tripping

    let attempts = std::sync::atomic::AtomicU32::new(0);
    let result = rm.execute(|| {
        let n = attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        Err::<String, _>(format!("attempt_{}", n))
    });

    assert!(!result.success, "should return failure after exhaustion");
    assert_eq!(result.attempts, 3, "should have made exactly 3 attempts");
    assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 3, "closure should have been called exactly 3 times");
    assert!(result.value.is_none(), "value should be None on failure");
}

// ---------------------------------------------------------------------------
// Test 5: resilience_manager_timeout
// ---------------------------------------------------------------------------

#[test]
fn resilience_manager_timeout() {
    let rm = ResilienceManager::new()
        .with_retry(100, Duration::from_millis(5))
        .with_timeout(Duration::from_millis(100));

    let start = std::time::Instant::now();
    let result = rm.execute(|| {
        std::thread::sleep(Duration::from_millis(200));
        Ok::<_, String>("should_not_reach".to_string())
    });
    let elapsed = start.elapsed();

    assert!(!result.success, "should return failure on timeout");
    // Timeout should trigger well before 100 retries * 5ms = 500ms
    assert!(
        elapsed < Duration::from_secs(2),
        "timeout should stop execution early, but took {:?}",
        elapsed
    );
}

// ---------------------------------------------------------------------------
// Test 6: predictive_maintenance_detects_anomaly
// ---------------------------------------------------------------------------

#[test]
fn predictive_maintenance_detects_anomaly() {
    let detector = AnomalyDetector::new(5);
    let baseline = vec![1.0, 1.1, 0.9, 1.0, 1.1];

    let result = detector.detect(5.0, &baseline);

    assert!(result.is_anomaly, "5.0 should be detected as anomaly against baseline near 1.0");
    assert!(
        matches!(
            result.severity,
            AnomalySeverity::Warning | AnomalySeverity::Critical
        ),
        "anomaly severity should be Warning or Critical, got {:?}",
        result.severity
    );
    assert!(
        result.deviation > 2.0,
        "z-score deviation should be significant (>2.0), got {}",
        result.deviation
    );
}

// ---------------------------------------------------------------------------
// Test 7: predictive_maintenance_predicts_failure
// ---------------------------------------------------------------------------

#[test]
fn predictive_maintenance_predicts_failure() {
    let predictor = Predictor::new();
    let degrading = vec![100.0, 90.0, 80.0, 70.0, 60.0];

    let turns = predictor.forecast_failure(&degrading, 50.0);

    assert!(turns.is_some(), "should predict failure for degrading trend toward 50");
    let t = turns.unwrap();
    assert!(
        t >= 1 && t <= 5,
        "failure should be predicted within 1-5 turns, got {}",
        t
    );
}

// ---------------------------------------------------------------------------
// Test 8: diagnostic_chain_completes
// ---------------------------------------------------------------------------

#[test]
fn diagnostic_chain_completes() {
    let config = HealingConfig::new(1000, 5, true);
    let mut loop_inst = HealingLoop::new(config);

    let signals = vec![
        HealthSignal::critical("db_pool", "error_rate", 0.95, 1000),
        HealthSignal::critical("db_pool", "latency_ms", 5000.0, 1000),
    ];

    // First cycle
    let action = loop_inst.run_cycle(&signals);
    assert!(action.is_some(), "first cycle should return an action");
    let action = action.unwrap();
    assert_eq!(
        action.action_type(),
        "restart",
        "critical signals should trigger restart, got {:?}",
        action.action_type()
    );
    assert_eq!(loop_inst.attempt, 1, "attempt should be 1 after first cycle");

    // Second cycle
    let action2 = loop_inst.run_cycle(&signals);
    assert!(action2.is_some(), "second cycle should return an action");
    assert_eq!(loop_inst.attempt, 2, "attempt should be 2 after second cycle");

    // Exhaust the config (max_repair_attempts = 5)
    loop_inst.run_cycle(&signals);
    loop_inst.run_cycle(&signals);
    loop_inst.run_cycle(&signals);
    assert_eq!(loop_inst.attempt, 5, "attempt should be 5");
    assert!(loop_inst.is_exhausted(), "should be exhausted at max_repair_attempts=5");

    // 6th cycle should return None (cap reached)
    let action_exhausted = loop_inst.run_cycle(&signals);
    assert!(
        action_exhausted.is_none(),
        "should return None when exhausted"
    );
}
