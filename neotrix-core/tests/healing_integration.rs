#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::Duration;

// ── Self-healing imports ──────────────────────────────────────────────
use neotrix::l6_meta::healing::self_healing::circuit_breaker::CircuitState;
use neotrix::l6_meta::healing::self_healing::{
    AutoRepair, CircuitBreaker, ComponentMonitor, Failure, FailureSeverity,
    ResilienceManager,
};

// ── Predictive maintenance imports ────────────────────────────────────
use neotrix::l6_meta::healing::predictive_maintenance::{
    AnomalyDetector, MaintenanceScheduler, PredictedIssue, Predictor,
    TrendAnalyzer, TrendDirection, Urgency,
};

// ── Diagnostic chain imports ──────────────────────────────────────────
use neotrix::l6_meta::healing::diagnostic_chain::{
    Diagnostician, HealingConfig, HealingLoop, HealthSignal,
};

// ── Governance enforcement imports ────────────────────────────────────
use neotrix::l6_meta::coordination::governance::enforcement::{
    AuditLog, ComplianceChecker, EnforcementEngine, EnforcementLevel, EnforcementResult, Policy,
    PolicyRule, RuleSeverity, Violation,
};

// ══════════════════════════════════════════════════════════════════════
// 1. Self-healing: full pipeline integration
// ══════════════════════════════════════════════════════════════════════

#[test]
fn integration_health_monitor_to_auto_repair_to_circuit_breaker() {
    let monitor = ComponentMonitor::new();
    let repair = AutoRepair::new(monitor);
    let cb = CircuitBreaker::new(3, Duration::from_secs(60));

    let failures = repair.detect_failures();
    for f in &failures {
        let result = cb.call(|| {
            let r = repair.repair(f);
            if r.success {
                Ok(r)
            } else {
                Err("repair failed")
            }
        });
        assert!(result.is_ok());
    }
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[test]
fn integration_circuit_breaker_trips_on_repeated_failures() {
    let cb = CircuitBreaker::new(3, Duration::from_secs(60));
    let monitor = ComponentMonitor::new();
    let repair = AutoRepair::new(monitor);

    for i in 0..3 {
        let f = Failure::new(format!("comp_{}", i), FailureSeverity::High, "fail");
        let _ = cb.call(|| {
            repair.repair(&f);
            Err::<String, _>("simulated failure")
        });
    }

    assert_eq!(cb.state(), CircuitState::Open);
    let result = cb.call(|| Ok::<_, String>("should be rejected"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("OPEN"));
}

#[test]
fn integration_circuit_breaker_recovery_cycle() {
    let cb = CircuitBreaker::new(2, Duration::from_millis(50));

    // Trip
    let _ = cb.call(|| Err::<String, _>("1"));
    let _ = cb.call(|| Err::<String, _>("2"));
    assert_eq!(cb.state(), CircuitState::Open);

    // Wait → HalfOpen
    std::thread::sleep(Duration::from_millis(60));
    assert_eq!(cb.state(), CircuitState::HalfOpen);

    // Recover → Closed
    let _ = cb.call(|| Ok::<_, String>("recovered"));
    assert_eq!(cb.state(), CircuitState::Closed);
    assert_eq!(cb.consecutive_failures(), 0);
}

#[test]
fn integration_resilience_manager_with_circuit_breaker() {
    let rm = ResilienceManager::new()
        .with_retry(3, Duration::from_millis(10))
        .with_circuit_breaker(2);

    // First two rounds of retries exhaust the circuit breaker
    let _ = rm.execute(|| Err::<String, _>("fail 1"));
    let _ = rm.execute(|| Err::<String, _>("fail 2"));
    assert_eq!(rm.circuit_state(), CircuitState::Open);

    // Blocked
    let result = rm.execute(|| Ok::<_, String>("should not run"));
    assert!(!result.success);

    // Reset and recover
    rm.reset_circuit();
    assert_eq!(rm.circuit_state(), CircuitState::Closed);
    let result = rm.execute(|| Ok::<_, String>("works now"));
    assert!(result.success);
}

#[test]
fn integration_resilience_retry_succeeds_on_third_attempt() {
    let rm = ResilienceManager::new().with_retry(5, Duration::from_millis(1));
    let calls = std::sync::atomic::AtomicU32::new(0);

    let result = rm.execute(|| {
        let n = calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        if n < 3 {
            Err("transient")
        } else {
            Ok("recovered")
        }
    });

    assert!(result.success);
    assert_eq!(result.attempts, 3);
    assert_eq!(result.value.as_deref(), Some("recovered"));
}

// ══════════════════════════════════════════════════════════════════════
// 2. Predictive maintenance: full pipeline
// ══════════════════════════════════════════════════════════════════════

#[test]
fn integration_trend_analysis_to_anomaly_detection() {
    let history: Vec<f64> = (0..30).map(|i| 100.0 + i as f64 * 0.5).collect();

    let analyzer = TrendAnalyzer::new();
    let trend = analyzer.analyze(&history);
    assert_eq!(trend.direction, TrendDirection::Degrading);

    let detector = AnomalyDetector::new(20);
    let result = detector.detect(150.0, &history);
    assert!(!result.is_anomaly);
}

#[test]
fn integration_anomaly_detection_to_maintenance_scheduling() {
    let history: Vec<f64> = vec![10.0; 10];
    let detector = AnomalyDetector::new(5);
    let result = detector.detect(100.0, &history);
    assert!(result.is_anomaly);

    let scheduler = MaintenanceScheduler::new();
    let issues = vec![PredictedIssue {
        component: "nt_core_cache".into(),
        description: "anomaly detected".into(),
        severity: 0.95,
        turns_until_failure: Some(3),
    }];

    let plan = scheduler.schedule(issues);
    assert_eq!(plan.tasks.len(), 1);
    assert_eq!(plan.tasks[0].urgency, Urgency::Critical);
    assert!(plan.tasks[0].action.contains("Immediate"));
}

#[test]
fn integration_predictor_to_failure_forecast() {
    let history: Vec<f64> = (0..20).map(|i| 100.0 + i as f64 * 5.0).collect();

    let predictor = Predictor::new();
    let turns = predictor.forecast_failure(&history, 200.0);
    assert!(turns.is_some());

    let scheduler = MaintenanceScheduler::new();
    let issues = vec![PredictedIssue {
        component: "db_pool".into(),
        description: "connection pool exhaustion".into(),
        severity: 0.95,
        turns_until_failure: turns,
    }];

    let plan = scheduler.schedule(issues);
    assert_eq!(plan.tasks[0].urgency, Urgency::Critical);
    assert!(plan.total_duration() > 0);
}

#[test]
fn integration_full_monitoring_pipeline() {
    // 1. Collect history
    let history: Vec<f64> = (0..30).map(|i| 50.0 + i as f64 * 2.0).collect();

    // 2. Trend analysis
    let analyzer = TrendAnalyzer::new();
    let trend = analyzer.analyze(&history);
    assert_eq!(trend.direction, TrendDirection::Degrading);

    // 3. Anomaly detection
    let detector = AnomalyDetector::new(15);
    let last = *history.last().unwrap();
    let anomaly = detector.detect(last + 100.0, &history);
    assert!(anomaly.is_anomaly);

    // 4. Prediction
    let predictor = Predictor::new();
    let future = predictor.predict(&history, 10);
    assert_eq!(future.len(), 10);

    // 5. Failure forecast
    let turns = predictor.forecast_failure(&history, 200.0);
    assert!(turns.is_some());

    // 6. Schedule maintenance
    let scheduler = MaintenanceScheduler::new();
    let issues = vec![PredictedIssue {
        component: "memory".into(),
        description: "memory usage trending up".into(),
        severity: 0.8,
        turns_until_failure: turns,
    }];
    let plan = scheduler.schedule(issues);
    assert_eq!(plan.tasks.len(), 1);
    assert_eq!(plan.tasks[0].urgency, Urgency::High);
}

// ══════════════════════════════════════════════════════════════════════
// 3. Diagnostic chain: end-to-end
// ══════════════════════════════════════════════════════════════════════

#[test]
fn integration_diagnostician_to_healing_loop() {
    let config = HealingConfig::default();
    let mut loop_inst = HealingLoop::new(config);

    let signals = vec![
        HealthSignal::critical("db", "error_rate", 0.95, 1000),
        HealthSignal::critical("db", "latency_ms", 5000.0, 1000),
    ];

    let action = loop_inst.run_cycle(&signals);
    assert!(action.is_some());
    assert_eq!(action.unwrap().action_type(), "restart");
    assert_eq!(loop_inst.attempt, 1);
}

#[test]
fn integration_healing_loop_multiple_cycles() {
    let config = HealingConfig {
        max_repair_attempts: 3,
        auto_heal_enabled: true,
        check_interval_ms: 1000,
    };
    let mut loop_inst = HealingLoop::new(config);
    let signals = vec![HealthSignal::critical("svc", "error", 0.9, 1000)];

    for _ in 0..3 {
        assert!(loop_inst.run_cycle(&signals).is_some());
    }
    assert!(loop_inst.is_exhausted());
    assert!(loop_inst.run_cycle(&signals).is_none());
}

#[test]
fn integration_diagnostic_only_mode() {
    let config = HealingConfig::diagnostic_only();
    let mut loop_inst = HealingLoop::new(config);
    let signals = vec![HealthSignal::critical("svc", "error", 0.9, 1000)];

    let action = loop_inst.run_cycle(&signals);
    assert!(action.is_some());
    // Auto-heal disabled: attempt counter does not increment
    assert_eq!(loop_inst.attempt, 0);
}

#[test]
fn integration_diagnostician_severity_mapping() {
    // Critical → Restart
    let signals = vec![HealthSignal::critical("db", "err", 0.9, 1000)];
    let result = Diagnostician::analyze(&signals).unwrap();
    assert!(result
        .recommended_actions
        .iter()
        .any(|a| a.contains("Restart")));

    // Warning → Monitor/Degrade
    let signals = vec![HealthSignal::warning("cache", "hit_rate", 0.2, 1000)];
    let result = Diagnostician::analyze(&signals).unwrap();
    assert!(result
        .recommended_actions
        .iter()
        .any(|a| a.contains("Monitor") || a.contains("Degrade")));

    // Info → Log
    let signals = vec![HealthSignal::info("svc", "uptime", 3600.0, 1000)];
    let result = Diagnostician::analyze(&signals).unwrap();
    assert!(result.recommended_actions.iter().any(|a| a.contains("Log")));
}

// ══════════════════════════════════════════════════════════════════════
// 4. Governance enforcement: end-to-end
// ══════════════════════════════════════════════════════════════════════

#[test]
fn integration_enforcement_engine_full_workflow() {
    let policy = Policy::new(
        "safety",
        "Code Safety",
        "No unsafe code allowed",
        EnforcementLevel::Blocking,
    )
    .with_rule(PolicyRule::new(
        "no_unsafe",
        "unsafe",
        "Reject unsafe blocks",
        RuleSeverity::Critical,
    ));

    let mut engine = EnforcementEngine::new(vec![policy]);

    // Allowed action
    let result = engine.check("deploy to staging", &HashMap::new());
    assert!(result.allowed);

    // Blocked action
    let result = engine.check("write unsafe code", &HashMap::new());
    assert!(!result.allowed);
    assert_eq!(result.violations.len(), 1);

    // Stats
    let stats = engine.stats();
    assert_eq!(stats.total_checks, 2);
    assert_eq!(stats.total_violations, 1);
}

#[test]
fn integration_enforcement_with_audit_log() {
    let policy = Policy::new(
        "security",
        "Security",
        "No hardcoded secrets",
        EnforcementLevel::Blocking,
    )
    .with_rule(PolicyRule::new(
        "no_secrets",
        "secret",
        "Do not hardcode secrets",
        RuleSeverity::Critical,
    ));

    let mut engine = EnforcementEngine::new(vec![policy]);
    let mut audit_log = AuditLog::new(100);

    let r1 = engine.check("deploy safely", &HashMap::new());
    audit_log.log(&r1, "deploy safely", "agent_1");
    assert!(r1.allowed);

    let r2 = engine.check("add secret key", &HashMap::new());
    audit_log.log(&r2, "add secret key", "agent_1");
    assert!(!r2.allowed);

    assert_eq!(audit_log.len(), 2);
    let blocked = audit_log.get_blocked();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].action, "add secret key");
}

#[test]
fn integration_compliance_checker_full_report() {
    let good_policy = Policy::new(
        "good",
        "Good Policy",
        "Properly configured",
        EnforcementLevel::Blocking,
    )
    .with_rule(PolicyRule::new(
        "r1",
        "valid condition",
        "valid action",
        RuleSeverity::High,
    ));

    let bad_policy = Policy::new(
        "bad",
        "Bad Policy",
        "Has empty rules",
        EnforcementLevel::Blocking,
    )
    .with_rule(PolicyRule::new("r1", "", "", RuleSeverity::Critical));

    let report = ComplianceChecker::check_system_compliance(&[good_policy, bad_policy]);
    assert!(!report.is_compliant());
    assert_eq!(report.passed, 1);
    assert_eq!(report.failed, 1);
    assert!(!report.recommendations.is_empty());
}

#[test]
fn integration_enforcement_three_levels() {
    // Advisory → warnings only
    let advisory = Policy::new(
        "adv",
        "Advisory",
        "advisory policy",
        EnforcementLevel::Advisory,
    )
    .with_rule(PolicyRule::new(
        "r1",
        "todo",
        "remove todos",
        RuleSeverity::Low,
    ));

    // Warning → warnings, not blocking
    let warning = Policy::new(
        "warn",
        "Warning",
        "warning policy",
        EnforcementLevel::Warning,
    )
    .with_rule(PolicyRule::new(
        "r1",
        "fixme",
        "remove fixmes",
        RuleSeverity::Medium,
    ));

    // Blocking → blocks action
    let blocking = Policy::new(
        "block",
        "Blocking",
        "blocking policy",
        EnforcementLevel::Blocking,
    )
    .with_rule(PolicyRule::new(
        "r1",
        "unsafe",
        "block unsafe",
        RuleSeverity::Critical,
    ));

    let mut engine = EnforcementEngine::new(vec![advisory, warning, blocking]);

    // Safe action → allowed
    let result = engine.check("deploy safely", &HashMap::new());
    assert!(result.allowed);

    // Action with advisory + warning → allowed (warnings only)
    let result = engine.check("add todo and fixme", &HashMap::new());
    assert!(result.allowed);
    assert!(result.warnings.len() >= 2);

    // Action with blocking → blocked
    let result = engine.check("write unsafe code", &HashMap::new());
    assert!(!result.allowed);
}

#[test]
fn integration_audit_log_queries() {
    let mut log = AuditLog::new(100);

    let allowed = EnforcementResult::allowed();
    let blocked = EnforcementResult::blocked(
        vec![Violation {
            policy_id: "p1".into(),
            rule_id: "r1".into(),
            message: "violation".into(),
            severity: RuleSeverity::High,
        }],
        vec![],
    );

    log.log(&allowed, "deploy", "agent_1");
    log.log(&blocked, "push secrets", "agent_1");
    log.log(&allowed, "merge", "agent_2");
    log.log(&blocked, "write unsafe", "agent_2");

    // By agent
    assert_eq!(log.get_by_agent("agent_1").len(), 2);
    assert_eq!(log.get_by_agent("agent_2").len(), 2);

    // Blocked entries
    assert_eq!(log.get_blocked().len(), 2);

    // Recent
    assert_eq!(log.get_recent(2).len(), 2);

    // By policy
    let p1_entries = log.get_by_policy("p1");
    assert!(p1_entries.len() >= 2);
}

#[test]
fn integration_policy_serialization_and_enforcement() {
    let policy = Policy::new(
        "test",
        "Test Policy",
        "Serialization test",
        EnforcementLevel::Blocking,
    )
    .with_rule(PolicyRule::new(
        "r1",
        "forbidden",
        "block it",
        RuleSeverity::High,
    ));

    let json = serde_json::to_string(&policy).unwrap();
    let deserialized: Policy = serde_json::from_str(&json).unwrap();

    let mut engine = EnforcementEngine::new(vec![deserialized]);
    let result = engine.check("this is forbidden", &HashMap::new());
    assert!(!result.allowed);
}
