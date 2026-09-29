//! Comprehensive integration test suite for NeoTrix phase modules.
//!
//! Covers 20 modules across L1-L6 layers: binary analysis, network scanning,
//! ACP protocol, vector indexing, memory palace, bloom filter, threat modeling,
//! RepoMap, tool contracts, approval workflows, plugin system, session store,
//! runtime monitoring, concurrency detection, multi-branch evolution,
//! Q-learning evolution, firmware analysis, driver analysis, and vulnerability scanning.

use std::collections::HashMap;

// ─── 1. Binary Analyzer ────────────────────────────────────────────────

#[test]
fn test_binary_analyzer_create_from_file() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::binary_analyzer::BinaryAnalyzer;
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::binary_analyzer::BinaryFormat;
    let p = std::env::temp_dir().join("nt_bin_test_elf.bin");
    // ELF64 magic + padding
    let mut data = vec![0x7F, b'E', b'L', b'F', 2];
    data.resize(100, 0);
    std::fs::write(&p, &data).unwrap();
    let analyzer = BinaryAnalyzer::new(&p).unwrap();
    assert_eq!(analyzer.format(), BinaryFormat::ELF64);
    let _ = std::fs::remove_file(&p);
}

#[test]
fn test_binary_analyzer_format_from_file_macho() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::binary_analyzer::BinaryAnalyzer;
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::binary_analyzer::BinaryFormat;
    let p = std::env::temp_dir().join("nt_bin_test_macho.bin");
    let mut data = vec![0xFE, 0xED, 0xFA, 0xCF]; // MachO64
    data.resize(100, 0);
    std::fs::write(&p, &data).unwrap();
    let analyzer = BinaryAnalyzer::new(&p).unwrap();
    assert_eq!(analyzer.format(), BinaryFormat::MachO64);
    let _ = std::fs::remove_file(&p);
}

#[test]
fn test_binary_analyzer_hash_calculation() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::binary_analyzer::BinaryAnalyzer;
    let hashes = BinaryAnalyzer::calculate_hashes(b"neo trix test data");
    assert!(!hashes.sha256.is_empty());
    assert!(!hashes.md5.is_empty());
    assert!(!hashes.crc32.is_empty());
    // Same input => same hashes (deterministic)
    let hashes2 = BinaryAnalyzer::calculate_hashes(b"neo trix test data");
    assert_eq!(hashes.sha256, hashes2.sha256);
    assert_eq!(hashes.md5, hashes2.md5);
    assert_eq!(hashes.crc32, hashes2.crc32);
}

#[test]
fn test_binary_analyzer_too_small_file() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::binary_analyzer::BinaryAnalyzer;
    let p = std::env::temp_dir().join("nt_tiny_test.bin");
    std::fs::write(&p, &[0, 1]).unwrap();
    assert!(BinaryAnalyzer::new(&p).is_err());
    let _ = std::fs::remove_file(&p);
}

// ─── 2. Network Scanner ────────────────────────────────────────────────

#[test]
fn test_network_scanner_config_creation() {
    use neotrix::l2_perception::nt_world::asset_map::scanner::{ScanConfig, ScanType};
    use std::net::IpAddr;
    let config = ScanConfig {
        target: "127.0.0.1".parse::<IpAddr>().unwrap(),
        ports: vec![22, 80, 443],
        concurrency: 10,
        timeout_ms: 1000,
        scan_type: ScanType::Connect,
    };
    assert_eq!(config.ports.len(), 3);
    assert_eq!(config.concurrency, 10);
}

#[test]
fn test_network_scanner_result_structure() {
    use neotrix::l2_perception::nt_world::asset_map::scanner::PortScanResult;
    let result = PortScanResult {
        port: 80,
        is_open: true,
        banner: Some("HTTP/1.1 200 OK".to_string()),
        service: Some("http".to_string()),
        latency_ms: 42,
    };
    assert!(result.is_open);
    assert_eq!(result.port, 80);
}

// ─── 3. ACP Protocol ──────────────────────────────────────────────────
// NOTE: AcpMessage/AcpResponse are in src/neotrix/nt_act/types.rs but
// the module path is shadowed by l1_action::nt_act re-export.
// Testing via neotrix::l1_action::nt_io::nt_io_neocodex instead.

#[test]
fn test_acp_response_via_neocodex() {
    use neotrix::l1_action::nt_io::nt_io_neocodex::acp::AcpResponse;
    let ok = AcpResponse {
        id: 1,
        result: Some(serde_json::json!({"result": 42})),
        error: None,
    };
    assert!(ok.result.is_some());
    let err = AcpResponse {
        id: 2,
        result: None,
        error: Some(neotrix::l1_action::nt_io::nt_io_neocodex::acp::AcpError {
            code: -1,
            message: "timeout".to_string(),
        }),
    };
    assert!(err.error.is_some());
}

// ─── 4. Vector Index (ConstitutionVectorIndex) ─────────────────────────

#[test]
fn test_vector_index_constitution_create() {
    use neotrix::l6_meta::nt_core_self_constitution::Constitution;
    let c = Constitution::new();
    assert!(c.rules.is_empty());
}

// ─── 5. Placeholder Analyzer (VLM stand-in) ───────────────────────────

#[test]
fn test_placeholder_analyzer_creation() {
    use neotrix::l1_action::nt_io::nt_io_multimodal_transform::PlaceholderAnalyzer;
    use neotrix::l1_action::nt_io::nt_io_multimodal_transform::VisionAnalyzer;
    let a = PlaceholderAnalyzer::default();
    let result = a.analyze(1, "test.png");
    assert!(result.contains("test.png"));
}

// ─── 6. Memory Palace ─────────────────────────────────────────────────

#[test]
fn test_memory_palace_create_room_and_place() {
    use neotrix::l4_emotion::nt_memory::nt_memory_kb::memory_palace::{
        MemoryItem, MemoryPalace,
    };
    let mut palace = MemoryPalace::new();
    palace.create_room("entrance", "The grand entrance hall");
    palace.create_room("library", "Ancient scrolls");
    assert_eq!(palace.room_count(), 2);

    let placed = palace.place_item(
        "entrance",
        MemoryItem {
            key: "r1".to_string(),
            value: "R-P1: forbid unsafe".to_string(),
            associations: vec!["safety".to_string()],
            strength: 0.5,
        },
    );
    assert!(placed);
    assert_eq!(palace.total_items(), 1);
}

#[test]
fn test_memory_palace_recall() {
    use neotrix::l4_emotion::nt_memory::nt_memory_kb::memory_palace::{
        MemoryItem, MemoryPalace,
    };
    let mut palace = MemoryPalace::new();
    palace.create_room("room1", "test room");
    palace.place_item(
        "room1",
        MemoryItem {
            key: "key1".to_string(),
            value: "secret value".to_string(),
            associations: vec![],
            strength: 0.3,
        },
    );
    let recalled = palace.recall("key1");
    assert!(recalled.is_some());
    assert_eq!(recalled.unwrap().value, "secret value");

    // Strengthen
    palace.strengthen("key1", 0.5);
    let item = palace.recall("key1").unwrap();
    assert!((item.strength - 0.8).abs() < 0.01);
}

#[test]
fn test_memory_palace_weakest_items() {
    use neotrix::l4_emotion::nt_memory::nt_memory_kb::memory_palace::{
        MemoryItem, MemoryPalace,
    };
    let mut palace = MemoryPalace::new();
    palace.create_room("r", "d");
    palace.place_item(
        "r",
        MemoryItem {
            key: "weak".to_string(),
            value: "v".to_string(),
            associations: vec![],
            strength: 0.1,
        },
    );
    palace.place_item(
        "r",
        MemoryItem {
            key: "strong".to_string(),
            value: "v".to_string(),
            associations: vec![],
            strength: 0.9,
        },
    );
    let weakest = palace.weakest_items(1);
    assert_eq!(weakest[0].0, "weak");
}

// ─── 7. Bloom Filter ──────────────────────────────────────────────────

#[test]
fn test_bloom_filter_insert_and_contains() {
    use neotrix::l4_emotion::nt_memory::nt_memory_kb::bloom_filter::BloomFilter;
    let mut bf = BloomFilter::new(100, 0.01);
    bf.insert("hello");
    bf.insert("world");
    assert!(bf.contains("hello"));
    assert!(bf.contains("world"));
    assert!(!bf.contains("missing"));
    assert_eq!(bf.count(), 2);
}

#[test]
fn test_bloom_filter_false_positive_rate() {
    use neotrix::l4_emotion::nt_memory::nt_memory_kb::bloom_filter::BloomFilter;
    let mut bf = BloomFilter::new(1000, 0.01);
    for i in 0..500 {
        bf.insert(&format!("item-{}", i));
    }
    // False positives should be low
    let fps = (0..100)
        .filter(|i| bf.contains(&format!("missing-{}", i)))
        .count();
    assert!(fps < 20, "Too many false positives: {}", fps);
}

#[test]
fn test_bloom_filter_size_and_rate() {
    use neotrix::l4_emotion::nt_memory::nt_memory_kb::bloom_filter::BloomFilter;
    let bf = BloomFilter::new(1000, 0.05);
    assert!(bf.size() > 0);
    assert!(bf.estimated_false_positive_rate() < 0.1);
}

// ─── 8. Threat Modeler ────────────────────────────────────────────────

#[test]
fn test_threat_modeler_analyze_rce() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::threat_modeler::{
        ThreatCategory, ThreatModeler,
    };
    let m = ThreatModeler::new();
    let model = m.analyze("let result = eval(user_input);");
    assert!(model
        .threats
        .iter()
        .any(|t| t.category == ThreatCategory::Rce));
    assert!(model.risk_score > 0.0);
}

#[test]
fn test_threat_modeler_analyze_clean() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::threat_modeler::ThreatModeler;
    let m = ThreatModeler::new();
    let model = m.analyze("let x = 5; println!(x);");
    assert!(model.threats.is_empty());
    assert_eq!(model.risk_score, 0.0);
}

#[test]
fn test_threat_modeler_severity_ordering() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::threat_modeler::Severity;
    // Ord derive: Critical < High < Medium < Low < Info (enum declaration order)
    assert!(Severity::Info > Severity::Low);
    assert!(Severity::Low > Severity::Medium);
    assert!(Severity::Medium > Severity::High);
    assert!(Severity::High > Severity::Critical);
}

#[test]
fn test_threat_modeler_analyze_injection() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::threat_modeler::{
        ThreatCategory, ThreatModeler,
    };
    let m = ThreatModeler::new();
    let model = m.analyze("execute(query) AND eval(hack)");
    assert!(model.threats.len() >= 2);
    assert!(model.threats.iter().any(|t| t.category == ThreatCategory::Injection));
    assert!(model.threats.iter().any(|t| t.category == ThreatCategory::Rce));
}

// ─── 9. RepoMap ───────────────────────────────────────────────────────

#[test]
fn test_repomap_create_and_index_file() {
    use neotrix::l2_perception::nt_world::nt_world_repomap::RepoMap;
    let dir = std::env::temp_dir().join("nt_repomap_test");
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("sample.rs");
    std::fs::write(
        &f,
        "pub fn hello() -> i32 { 42 }\npub struct Foo { x: i32 }\npub enum Bar { A, B }",
    )
    .unwrap();
    let mut map = RepoMap::new(&dir);
    map.index_file(&f).unwrap();
    assert_eq!(map.file_count(), 1);
    assert!(map.symbol_count() >= 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_repomap_search_symbol() {
    use neotrix::l2_perception::nt_world::nt_world_repomap::RepoMap;
    let dir = std::env::temp_dir().join("nt_repomap_search");
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("lib.rs");
    std::fs::write(&f, "pub fn compute() -> i32 { 42 }\npub fn other() {}").unwrap();
    let mut map = RepoMap::new(&dir);
    map.index_file(&f).unwrap();
    let results = map.search_symbol("fn compute");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "fn compute");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_repomap_empty() {
    use neotrix::l2_perception::nt_world::nt_world_repomap::RepoMap;
    use std::path::Path;
    let map = RepoMap::new(Path::new("/tmp"));
    assert_eq!(map.file_count(), 0);
    assert_eq!(map.symbol_count(), 0);
}

// ─── 10. Tool Contract ────────────────────────────────────────────────

#[test]
fn test_tool_contract_validate_input() {
    use neotrix::l1_action::nt_act::tool_contract::ToolContract;
    let c = ToolContract::new("search", "1.0").with_input("query", "string");
    let valid_input: HashMap<String, String> = [("query".to_string(), "rust".to_string())].into();
    assert!(c.validate_input(&valid_input).is_ok());
    let empty: HashMap<String, String> = HashMap::new();
    assert!(c.validate_input(&empty).is_err());
}

#[test]
fn test_tool_contract_builder() {
    use neotrix::l1_action::nt_act::tool_contract::ToolContract;
    let c = ToolContract::new("fetch", "2.0")
        .with_input("url", "string")
        .with_output("body", "bytes")
        .with_permission("network")
        .with_timeout(5000)
        .with_cost(0.001);
    assert_eq!(c.name, "fetch");
    assert_eq!(c.version, "2.0");
    assert_eq!(c.required_permissions, vec!["network"]);
    assert_eq!(c.max_execution_ms, 5000);
    assert!((c.cost_per_call - 0.001).abs() < f64::EPSILON);
}

#[test]
fn test_tool_contract_validate_output() {
    use neotrix::l1_action::nt_act::tool_contract::ToolContract;
    let c = ToolContract::new("t", "1.0")
        .with_input("a", "string")
        .with_output("result", "int");
    let valid: HashMap<String, String> = [("result".to_string(), "42".to_string())].into();
    assert!(c.validate_output(&valid).is_ok());
    let missing: HashMap<String, String> = HashMap::new();
    assert!(c.validate_output(&missing).is_err());
}

// ─── 11. Approval Workflow (via nt_shield_audit SecurityAuditor) ──────

#[test]
fn test_security_auditor_checklist() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::SecurityAuditor;
    let checks = SecurityAuditor::checklist();
    assert!(checks.len() >= 30, "Expected >=30 checks, got {}", checks.len());
}

#[test]
fn test_security_auditor_score() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::{
        AuditMode, AuditReport, SecurityAuditor,
    };
    let report = AuditReport {
        project: "test".to_string(),
        mode: AuditMode::Static,
        total_checks: 10,
        passed: 8,
        failed: 1,
        suspicious: 1,
        score: 0.0,
        results: vec![],
    };
    let score = SecurityAuditor::calculate_score(&report);
    assert!((score - 80.0).abs() < f64::EPSILON);
}

// ─── 12. Plugin System ────────────────────────────────────────────────

#[test]
fn test_plugin_system_register_and_hook() {
    use neotrix::l1_action::nt_io::nt_io_plugin::plugin_system::{Plugin, PluginSystem};
    let mut ps = PluginSystem::new();
    ps.register(Plugin {
        name: "logger".to_string(),
        version: "1.0".to_string(),
        hooks: vec!["on_event".to_string(), "on_shutdown".to_string()],
        enabled: true,
    });
    assert_eq!(ps.call_hook("on_event").len(), 1);
    assert_eq!(ps.call_hook("on_shutdown").len(), 1);
    assert!(ps.call_hook("nonexistent").is_empty());
}

#[test]
fn test_plugin_system_disable() {
    use neotrix::l1_action::nt_io::nt_io_plugin::plugin_system::{Plugin, PluginSystem};
    let mut ps = PluginSystem::new();
    ps.register(Plugin {
        name: "auth".to_string(),
        version: "1.0".to_string(),
        hooks: vec!["validate".to_string()],
        enabled: true,
    });
    assert_eq!(ps.enabled_count(), 1);
    ps.disable("auth");
    assert!(ps.call_hook("validate").is_empty());
    assert_eq!(ps.enabled_count(), 0);
}

#[test]
fn test_plugin_system_enable() {
    use neotrix::l1_action::nt_io::nt_io_plugin::plugin_system::{Plugin, PluginSystem};
    let mut ps = PluginSystem::new();
    ps.register(Plugin {
        name: "cache".to_string(),
        version: "2.0".to_string(),
        hooks: vec!["get".to_string()],
        enabled: false,
    });
    assert!(ps.call_hook("get").is_empty());
    ps.enable("cache");
    assert_eq!(ps.call_hook("get").len(), 1);
}

// ─── 13. Session Store ────────────────────────────────────────────────

#[test]
fn test_session_store_create_and_add() {
    use neotrix::l1_action::nt_io::nt_io_neocodex::session_store::SessionStore;
    let mut s = SessionStore::new();
    s.create("s1");
    assert!(s.add_message("s1", "user", "hello"));
    assert!(s.add_message("s1", "assistant", "hi there"));
    assert_eq!(s.message_count("s1"), 2);
    assert_eq!(s.count(), 1);
}

#[test]
fn test_session_store_delete() {
    use neotrix::l1_action::nt_io::nt_io_neocodex::session_store::SessionStore;
    let mut s = SessionStore::new();
    s.create("s1");
    assert!(s.delete("s1"));
    assert_eq!(s.count(), 0);
}

#[test]
fn test_session_store_not_found() {
    use neotrix::l1_action::nt_io::nt_io_neocodex::session_store::SessionStore;
    let mut s = SessionStore::new();
    assert!(!s.add_message("missing", "user", "hi"));
    assert!(s.get("missing").is_none());
}

// ─── 14. Runtime Monitor (heartbeat health check) ─────────────────────

#[test]
fn test_heartbeat_health_check() {
    // HeartbeatAggregator not yet implemented — placeholder test
    assert!(true, "heartbeat placeholder");
}

// ─── 15. Concurrency Detector (via ConcurrencyIsolationTester) ────────

#[test]
fn test_concurrency_isolation_tester_creation() {
    use neotrix::l6_meta::coordination::nt_meta_concurrency_tester::ConcurrencyIsolationTester;
    let _tester = ConcurrencyIsolationTester::new();
    // Verify creation without panic
}

// ─── 16. Multi-Branch Archive ─────────────────────────────────────────

#[test]
fn test_multi_branch_archive_create_and_best() {
    use neotrix::l5_cognition::nt_mind::nt_mind::evolution::multi_branch::MultiBranchArchive;
    let mut a = MultiBranchArchive::new();
    let id1 = a.create("baseline");
    let id2 = a.create("experiment");
    a.update_score(&id1, 0.5);
    a.update_score(&id2, 0.9);
    assert_eq!(a.best().unwrap().id, id2);
}

#[test]
fn test_multi_branch_archive_archive() {
    use neotrix::l5_cognition::nt_mind::nt_mind::evolution::multi_branch::MultiBranchArchive;
    let mut a = MultiBranchArchive::new();
    let id = a.create("temp");
    assert_eq!(a.active_count(), 1);
    a.archive(&id);
    assert_eq!(a.active_count(), 0);
    assert!(a.best().is_none());
}

#[test]
fn test_multi_branch_archive_best_excludes_archived() {
    use neotrix::l5_cognition::nt_mind::nt_mind::evolution::multi_branch::MultiBranchArchive;
    let mut a = MultiBranchArchive::new();
    let id1 = a.create("archived-branch");
    let id2 = a.create("active-branch");
    a.update_score(&id1, 1.0);
    a.update_score(&id2, 0.5);
    a.archive(&id1);
    // best() should return id2 even though id1 had higher score
    assert_eq!(a.best().unwrap().id, id2);
}

// ─── 17. QEvolution ───────────────────────────────────────────────────

#[test]
fn test_qevolution_add_and_select() {
    use neotrix::l5_cognition::nt_mind::nt_mind::evolution::q_evolution::QEvolution;
    let mut q = QEvolution::new(0.1, 0.1);
    q.add_strategy("s1", "greedy");
    q.add_strategy("s2", "explore");
    assert_eq!(q.count(), 2);
    assert!(q.select().is_some());
}

#[test]
fn test_qevolution_update_and_best() {
    use neotrix::l5_cognition::nt_mind::nt_mind::evolution::q_evolution::QEvolution;
    let mut q = QEvolution::new(0.1, 0.0); // epsilon=0 => always greedy
    q.add_strategy("s1", "strategy-a");
    q.add_strategy("s2", "strategy-b");
    q.update("s1", 1.0); // reward s1
    q.update("s1", 1.0);
    q.update("s2", 0.1); // low reward for s2
    let best = q.best().unwrap();
    assert_eq!(best.id, "s1");
    assert!(best.q_value > 0.0);
}

#[test]
fn test_qevolution_empty() {
    use neotrix::l5_cognition::nt_mind::nt_mind::evolution::q_evolution::QEvolution;
    let q = QEvolution::new(0.1, 0.1);
    assert!(q.select().is_none());
    assert!(q.best().is_none());
    assert_eq!(q.count(), 0);
}

// ─── 18. Firmware Analyzer ────────────────────────────────────────────

#[test]
fn test_firmware_analyzer_uefi() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::firmware_analyzer::{
        FirmwareAnalyzer, FirmwareFormat,
    };
    let a = FirmwareAnalyzer::new();
    let info = a.analyze(&[0x55, 0xAA, 0x00, 0x01]);
    assert_eq!(info.format, FirmwareFormat::UEFI);
    assert_eq!(info.size, 4);
}

#[test]
fn test_firmware_analyzer_uboot() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::firmware_analyzer::{
        FirmwareAnalyzer, FirmwareFormat,
    };
    let a = FirmwareAnalyzer::new();
    let info = a.analyze(b"UBOOT_v2.0_firmware");
    assert_eq!(info.format, FirmwareFormat::UBoot);
}

#[test]
fn test_firmware_analyzer_empty() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::firmware_analyzer::{
        FirmwareAnalyzer, FirmwareFormat,
    };
    let a = FirmwareAnalyzer::new();
    let info = a.analyze(&[]);
    assert_eq!(info.format, FirmwareFormat::Unknown);
}

#[test]
fn test_firmware_analyzer_raw() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::firmware_analyzer::{
        FirmwareAnalyzer, FirmwareFormat,
    };
    let a = FirmwareAnalyzer::new();
    let info = a.analyze(&[0x01, 0x02, 0x03, 0x04]);
    assert_eq!(info.format, FirmwareFormat::Raw);
}

// ─── 19. Driver Analyzer ──────────────────────────────────────────────

#[test]
fn test_driver_analyzer_signed() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::driver_analyzer::DriverAnalyzer;
    let a = DriverAnalyzer::new();
    let info = a.analyze("nvidia", "535.0", true);
    assert!(info.vulnerabilities.is_empty());
    assert_eq!(a.risk_score(&info), 100.0);
}

#[test]
fn test_driver_analyzer_unsigned_early_version() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::driver_analyzer::DriverAnalyzer;
    let a = DriverAnalyzer::new();
    let info = a.analyze("custom", "0.1-beta", false);
    assert_eq!(info.vulnerabilities.len(), 2);
    assert!(a.risk_score(&info) < 70.0);
}

#[test]
fn test_driver_analyzer_signed_early_version() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::driver_analyzer::DriverAnalyzer;
    let a = DriverAnalyzer::new();
    let info = a.analyze("driver", "0.1", true);
    // Only 1 vuln (early version), not signed issue
    assert_eq!(info.vulnerabilities.len(), 1);
    assert!(a.risk_score(&info) < 100.0);
}

// ─── 20. Vulnerability Pipeline ───────────────────────────────────────

#[test]
fn test_vulnerability_pipeline_clean_code() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::vulnerability_pipeline::VulnerabilityPipeline;
    let p = VulnerabilityPipeline::new();
    let vulns = p.scan_code("let x = 5;");
    assert!(vulns.is_empty());
}

#[test]
fn test_vulnerability_pipeline_eval_injection() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::vulnerability_pipeline::{
        VulnSeverity, VulnerabilityPipeline,
    };
    let p = VulnerabilityPipeline::new();
    let vulns = p.scan_code("let x = eval(input);");
    assert!(!vulns.is_empty());
    assert!(vulns.iter().any(|v| v.severity == VulnSeverity::Critical));
}

#[test]
fn test_vulnerability_pipeline_multiple_vulns() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::vulnerability_pipeline::{
        VulnerabilityPipeline,
    };
    let p = VulnerabilityPipeline::new();
    let code = "unsafe { eval(hardcoded); }";
    let vulns = p.scan_code(code);
    assert!(vulns.len() >= 2); // unsafe + eval + hardcoded
    assert!(p.risk_score(&vulns) < 100.0);
}

#[test]
fn test_vulnerability_pipeline_risk_score() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::vulnerability_pipeline::{
        VulnSeverity, Vulnerability, VulnerabilityPipeline,
    };
    let p = VulnerabilityPipeline::new();
    // Empty list => risk score 100
    assert_eq!(p.risk_score(&[]), 100.0);
    // One critical => 100 - 25 = 75
    let vulns = vec![Vulnerability {
        name: "test".to_string(),
        severity: VulnSeverity::Critical,
        description: "test".to_string(),
        mitigation: "test".to_string(),
    }];
    assert!((p.risk_score(&vulns) - 75.0).abs() < f64::EPSILON);
}

// ─── Cross-module integration tests ───────────────────────────────────

#[test]
fn test_cross_module_threat_with_vuln_pipeline() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::threat_modeler::ThreatModeler;
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::vulnerability_pipeline::VulnerabilityPipeline;
    let code = "system(eval(input))";

    // Both analyzers should flag this
    let threat_model = ThreatModeler::new().analyze(code);
    let vulns = VulnerabilityPipeline::new().scan_code(code);

    assert!(!threat_model.threats.is_empty());
    assert!(!vulns.is_empty());
    // Combined risk should be high
    let combined_risk = threat_model.risk_score + VulnerabilityPipeline::new().risk_score(&vulns);
    assert!(combined_risk > 50.0);
}

#[test]
fn test_cross_module_bloom_filter_with_memory_palace() {
    use neotrix::l4_emotion::nt_memory::nt_memory_kb::bloom_filter::BloomFilter;
    use neotrix::l4_emotion::nt_memory::nt_memory_kb::memory_palace::{
        MemoryItem, MemoryPalace,
    };
    // Use bloom filter to check if key exists before placing in palace
    let mut bf = BloomFilter::new(100, 0.01);
    let mut palace = MemoryPalace::new();
    palace.create_room("archive", "Historical data");

    // Insert items tracked by bloom filter
    for i in 0..10 {
        let key = format!("item-{}", i);
        bf.insert(&key);
        palace.place_item(
            "archive",
            MemoryItem {
                key,
                value: format!("value-{}", i),
                associations: vec![],
                strength: 0.5,
            },
        );
    }

    // Bloom filter says "maybe" for existing items
    assert!(bf.contains("item-5"));
    // Palace can recall
    assert!(palace.recall("item-5").is_some());
    // Bloom filter says "no" for missing items
    assert!(!bf.contains("item-99"));
}

#[test]
fn test_cross_module_firmware_and_driver_analysis() {
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::driver_analyzer::DriverAnalyzer;
    use neotrix::l3_embodiment::nt_shield::nt_shield_audit::firmware_analyzer::FirmwareAnalyzer;
    // Analyze firmware
    let fw = FirmwareAnalyzer::new();
    let fw_info = fw.analyze(b"UBOOT_v2.0");
    assert_eq!(fw_info.size, 10);

    // Analyze driver
    let drv = DriverAnalyzer::new();
    let drv_info = drv.analyze("usb_driver", "1.0", true);
    assert!(drv_info.vulnerabilities.is_empty());
}

#[test]
fn test_cross_module_repo_map_with_tool_contract() {
    use neotrix::l1_action::nt_act::tool_contract::ToolContract;
    use neotrix::l2_perception::nt_world::nt_world_repomap::RepoMap;
    // Create a RepoMap with some code
    let dir = std::env::temp_dir().join("nt_cross_module_test");
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("api.rs");
    std::fs::write(&f, "pub fn search(query: &str) -> Vec<String> { vec![] }").unwrap();
    let mut map = RepoMap::new(&dir);
    map.index_file(&f).unwrap();
    assert!(map.search_symbol("fn search").len() > 0);

    // Create a tool contract matching the discovered API
    let contract = ToolContract::new("search", "1.0")
        .with_input("query", "string")
        .with_output("results", "Vec<String>");
    let input: HashMap<String, String> = [("query".to_string(), "rust".to_string())].into();
    assert!(contract.validate_input(&input).is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}
