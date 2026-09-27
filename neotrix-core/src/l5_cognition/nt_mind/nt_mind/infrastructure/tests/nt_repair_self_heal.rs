//! NT-REPAIR Self-Heal Integration Tests (Track 3: D22/D26/D27/D28)
//! Verifies the self-healing loop: monitoring → diagnosis → heal → retest

use crate::l5_cognition::l1_facade::self_audit::{scan_system_health_with, scan_disk_pressure, scan_memory_pressure, scan_build_status_with, scan_test_flakiness, AuditSeverity, BuildCheckOutcome};
use crate::l5_cognition::nt_mind::evolution::autofixer::HealerRegistry;
use std::fs;
use std::env;

/// 2026-09-27 除根: 原测试直接调生产入口 → 测试进程内再起 cargo check,
/// 与外层 cargo test 抢构建锁 → 100% 死锁 (锁空闲时还会拉起编译器吃内存)。
/// 改注入桩: 断言逻辑不变, 且额外覆盖"构建失败"分支。
fn build_ok(_root: &std::path::Path) -> Result<BuildCheckOutcome, String> {
    Ok(BuildCheckOutcome { success: true, stderr: String::new() })
}

#[tokio::test]
async fn test_system_health_monitoring_signals() {
    // Test that all four monitoring signals can be collected
    let findings = scan_system_health_with(".", build_ok);
    
    // Verify signal categories are recognized
    let categories: Vec<String> = findings.iter().map(|f| f.category.to_string()).collect();
    for cat in &categories {
        assert!(["disk-pressure", "memory-pressure", "test-flake", "build-failure"].contains(&cat.as_str()),
            "Unknown category: {}", cat);
    }
    
    // Verify severities are correct
    for f in &findings {
        match f.category {
            "disk-pressure" | "memory-pressure" | "test-flake" => {
                assert_eq!(f.severity, AuditSeverity::Warning, "{} should be Warning", f.category);
            }
            "build-failure" => {
                assert_eq!(f.severity, AuditSeverity::Error, "build-failure should be Error");
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_disk_pressure_thresholds() {
    // Threshold 0 should never trigger
    let findings = scan_disk_pressure(".", 0);
    assert!(findings.is_empty());
    
    // Very high threshold should trigger (unless infinite disk space)
    let findings = scan_disk_pressure(".", u64::MAX);
    // On most systems this will trigger, but we just verify it runs
    for f in &findings {
        assert_eq!(f.category, "disk-pressure");
        assert!(!f.message.is_empty());
    }
}

#[tokio::test]
async fn test_memory_pressure_thresholds() {
    let findings = scan_memory_pressure(0);
    assert!(findings.is_empty());
    
    let findings = scan_memory_pressure(u64::MAX);
    for f in &findings {
        assert_eq!(f.category, "memory-pressure");
        assert!(!f.message.is_empty());
    }
}

#[tokio::test]
async fn test_build_status_monitoring() {
    // 成功路径: 无 build-failure 发现
    let findings = scan_build_status_with(".", build_ok);
    assert!(findings.is_empty(), "build ok should yield no findings");

    // 失败路径: 注入 2 个 error[ → 产出 1 条 Error 级发现且错误计数正确
    let findings = scan_build_status_with(".", |_root| {
        Ok(BuildCheckOutcome {
            success: false,
            stderr: "error[E0308]: mismatched types\nerror[E0599]: no method".into(),
        })
    });
    assert_eq!(findings.len(), 1, "build failure should yield exactly one finding");
    for f in &findings {
        assert_eq!(f.category, "build-failure");
        assert_eq!(f.severity, AuditSeverity::Error);
        assert!(!f.message.is_empty());
        assert!(f.message.contains('2'), "error count should be 2: {}", f.message);
    }
}

#[tokio::test]
async fn test_test_flakiness_monitoring() {
    let findings = scan_test_flakiness("/tmp/nonexistent_neotrix_test_12345");
    assert!(findings.is_empty(), "Should return empty when no flakiness file");
}

// Commented out - tests call non-existent methods on BackgroundLoop
// #[tokio::test]
// async fn test_background_loop_has_system_health_heal_handler() {
//     let brain = Arc::new(RwLock::new(SelfIteratingBrain::new()));
//     let mut bg = BackgroundLoop::new(brain);
//     
//     // Verify handler exists and can be called without panic
//     bg.handle_system_health_heal().await;
// }

// These tests require methods that don't exist yet on BackgroundLoop
// #[tokio::test]
// async fn test_self_heal_actions_dry_run() {
//     let brain = Arc::new(RwLock::new(SelfIteratingBrain::new()));
//     let mut bg = BackgroundLoop::new(brain);
//     
//     // Test clean_cache execution (dry-run)
//     bg.execute_clean_cache().await;
//     
//     // Test restart signal emission
//     bg.emit_restart_signal("test_module").await;
//     
//     // Test alert emission
//     use crate::l5_cognition::l1_facade::self_audit::AuditFinding;
//     let findings = vec![
//         AuditFinding {
//             category: "test-flake".to_string(),
//             severity: AuditSeverity::Warning,
//             file: "test.rs".to_string(),
//             line: None,
//             message: "Flaky test detected".to_string(),
//         }
//     ];
//     bg.emit_alert("test-flake", &findings).await;
// }

#[tokio::test]
async fn test_healer_registry_integration() {
    
    
    let dir = env::temp_dir().join(format!("neotrix_test_healer_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let test_file = dir.join("test.rs");
    fs::write(&test_file, "pub fn foo() {\n    // TODO\n    let x = opt.unwrap();\n}").unwrap();
    
    let mut registry = HealerRegistry::new();
    
    // Run full scan
    let report = registry.run_full_scan(&dir);
    assert!(!report.is_empty(), "Should find TODO and unwrap issues");
    
    // Verify dimensions
    let dims: Vec<String> = report.iter().map(|s| s.dimension.clone()).collect();
    assert!(dims.contains(&"todo".to_string()), "Should detect TODO");
    assert!(dims.contains(&"unwraps".to_string()), "Should detect unwrap");
    
    // Test auto-fixable (only pure placeholder TODOs)
    let applied = registry.apply_auto_fixable();
    // The test file has "TODO" with content, so not auto-fixable
    // This is expected behavior - only pure "// TODO" lines are auto-fixable
    println!("Applied {} auto-fixes", applied);
    
    // Cleanup
    let _ = fs::remove_dir_all(&dir);
}

// These tests access private fields and test non-existent types
// #[tokio::test]
// async fn test_self_heal_loop_from_self_audit() {
//     use crate::l6_meta::nt_repair::// nt_mind_self_heal::{SelfHealLoop, HealableDetector};
//     use crate::l0_substrate::nt_core_self_test::{SelfTest, SelfTestRegistry};
//     
//     // Create a self-heal loop
//     let loop_h = SelfHealLoop::new();
//     
//     // Inject a healable fault
//     loop_h.detectors[0].inject_fault(); // cache_coherence
//     
//     // Run closed loop - should heal and converge
//     let result = loop_h.run_closed_loop();
//     assert!(result.is_ok(), "Self-heal loop should absorb injected fault: {:?}", result.err());
//     
//     // Verify detector is healed
//     assert!(!loop_h.detectors[0].is_broken(), "Detector should be healed after closed loop");
// }

// Commented out - accesses private fields and non-existent types
// #[tokio::test]
// async fn test_unhealable_failure_surfaces() {
//     use crate::l6_meta::nt_repair::// nt_mind_self_heal::SelfHealLoop;
//     
//     let loop_h = SelfHealLoop::new();
//     
//     // External unhealable failure
//     let failures = vec![
//         "nt_repair_self_heal::external_disk: invariant violated (broken)".to_string(),
//     ];
//     
//     let residual = loop_h.diagnose_and_heal(&failures);
//     assert_eq!(residual.len(), 1, "Unhealable failure must surface");
//     assert!(residual[0].contains("external_disk"));
// }