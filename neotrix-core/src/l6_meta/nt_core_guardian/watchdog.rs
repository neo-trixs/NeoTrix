//! # WATCHDOG — 统一看门狗 (融合 schema_watchdog + build_watchdog)
//!
//! 融合:
//! - `l0_substrate/nt_core_schema_watchdog.rs` — Schema 漂移检测
//! - `l6_meta/coordination/nt_meta_build_watchdog.rs` — 构建状态监控
//!
//! 单一注册表: 所有 watchdog 检查项注册到 WatchdogRegistry,
//! 由 BackgroundLoop 的 spawn_handler 统一驱动。

#![forbid(unsafe_code)]

use std::collections::HashMap;

/// 看门狗检查结果
#[derive(Debug, Clone)]
pub struct WatchdogFinding {
    /// 检查项 ID
    pub check_id: String,
    /// 所属域 (schema/build/kb/network)
    pub domain: String,
    /// 严重程度
    pub severity: FindingSeverity,
    /// 描述
    pub message: String,
    /// 建议修复动作
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingSeverity {
    Info,
    Warning,
    Critical,
}

/// 看门狗检查项 trait
pub trait WatchdogCheck: Send + Sync {
    /// 检查项名称
    fn name(&self) -> &str;
    /// 所属域
    fn domain(&self) -> &str;
    /// 执行检查, 返回发现的问题
    fn check(&self) -> Vec<WatchdogFinding>;
}

/// 统一看门狗注册表
pub struct WatchdogRegistry {
    checks: Vec<Box<dyn WatchdogCheck>>,
    last_results: Vec<WatchdogFinding>,
    stats: HashMap<String, u64>,
}

impl WatchdogRegistry {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            last_results: Vec::new(),
            stats: HashMap::new(),
        }
    }

    /// 注册检查项
    pub fn register(&mut self, check: Box<dyn WatchdogCheck>) {
        self.checks.push(check);
    }

    /// 运行所有检查
    pub fn run_all(&mut self) -> Vec<WatchdogFinding> {
        let mut findings = Vec::new();
        for check in &self.checks {
            let mut result = check.check();
            findings.append(&mut result);
        }
        // 统计
        for f in &findings {
            *self.stats.entry(f.domain.clone()).or_insert(0) += 1;
        }
        self.last_results = findings.clone();
        findings
    }

    /// 运行指定域的检查
    pub fn run_domain(&mut self, domain: &str) -> Vec<WatchdogFinding> {
        let mut findings = Vec::new();
        for check in &self.checks {
            if check.domain() == domain {
                let mut result = check.check();
                findings.append(&mut result);
            }
        }
        findings
    }

    /// 获取上次结果
    pub fn last_results(&self) -> &[WatchdogFinding] {
        &self.last_results
    }

    /// 获取统计
    pub fn stats(&self) -> &HashMap<String, u64> {
        &self.stats
    }
}

impl Default for WatchdogRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// 内置检查项
// ═══════════════════════════════════════════════════════════════════

/// Schema 漂移检查 (从 l0_substrate/nt_core_schema_watchdog.rs 融合)
pub struct SchemaCheck;

impl WatchdogCheck for SchemaCheck {
    fn name(&self) -> &str { "schema_drift" }
    fn domain(&self) -> &str { "schema" }

    fn check(&self) -> Vec<WatchdogFinding> {
        let mut findings = Vec::new();
        let home = std::env::var("HOME").unwrap_or_default();
        let kb_path = std::path::PathBuf::from(home).join(".neotrix/knowledge.db");

        if !kb_path.exists() {
            findings.push(WatchdogFinding {
                check_id: "schema_kb_missing".into(),
                domain: "schema".into(),
                severity: FindingSeverity::Critical,
                message: "KB file missing".into(),
                suggested_action: Some("restore from backup".into()),
            });
            return findings;
        }

        // 检查关键表存在
        if let Ok(conn) = rusqlite::Connection::open(&kb_path) {
            let required_tables = ["nodes", "edges", "kv_store", "crawl_queue"];
            for table in &required_tables {
                let exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [table],
                        |r| r.get::<_, i64>(0),
                    )
                    .map(|n| n > 0)
                    .unwrap_or(false);
                if !exists {
                    findings.push(WatchdogFinding {
                        check_id: format!("schema_missing_table_{}", table),
                        domain: "schema".into(),
                        severity: FindingSeverity::Critical,
                        message: format!("Required table '{}' missing", table),
                        suggested_action: Some("run migration".into()),
                    });
                }
            }
        }

        findings
    }
}

/// KB 守卫检查 (从 foundation/guardian.rs 融合)
pub struct KbGuardCheck;

impl WatchdogCheck for KbGuardCheck {
    fn name(&self) -> &str { "kb_health" }
    fn domain(&self) -> &str { "kb" }

    fn check(&self) -> Vec<WatchdogFinding> {
        let mut findings = Vec::new();
        let home = std::env::var("HOME").unwrap_or_default();
        let kb_path = std::path::PathBuf::from(home).join(".neotrix/knowledge.db");

        if !kb_path.exists() {
            findings.push(WatchdogFinding {
                check_id: "kb_missing".into(),
                domain: "kb".into(),
                severity: FindingSeverity::Critical,
                message: "KB file missing".into(),
                suggested_action: Some("restore from backup".into()),
            });
            return findings;
        }

        // 检查文件大小
        if let Ok(meta) = std::fs::metadata(&kb_path) {
            if meta.len() < 1024 {
                findings.push(WatchdogFinding {
                    check_id: "kb_too_small".into(),
                    domain: "kb".into(),
                    severity: FindingSeverity::Critical,
                    message: format!("KB file too small ({} bytes)", meta.len()),
                    suggested_action: Some("restore from backup".into()),
                });
            }
        }

        // 检查 WAL 文件大小
        let wal_path = kb_path.with_extension("db-wal");
        if let Ok(meta) = std::fs::metadata(&wal_path) {
            if meta.len() > 100 * 1024 * 1024 { // > 100MB
                findings.push(WatchdogFinding {
                    check_id: "kb_wal_large".into(),
                    domain: "kb".into(),
                    severity: FindingSeverity::Warning,
                    message: format!("WAL file large ({} MB)", meta.len() / 1024 / 1024),
                    suggested_action: Some("consider checkpoint".into()),
                });
            }
        }

        findings
    }
}

/// 构建产物检查 (从 coordination/nt_meta_build_watchdog.rs 融合)
pub struct BuildCheck;

impl WatchdogCheck for BuildCheck {
    fn name(&self) -> &str { "build_artifact" }
    fn domain(&self) -> &str { "build" }

    fn check(&self) -> Vec<WatchdogFinding> {
        let mut findings = Vec::new();
        let target = std::path::Path::new("target/debug/neotrix");
        if !target.exists() {
            findings.push(WatchdogFinding {
                check_id: "build_missing".into(),
                domain: "build".into(),
                severity: FindingSeverity::Warning,
                message: "Build artifact missing".into(),
                suggested_action: Some("cargo build".into()),
            });
        }

        // 检查 target 目录大小
        let target_dir = std::path::Path::new("target");
        if let Ok(entries) = std::fs::read_dir(target_dir) {
            let total_size: u64 = entries
                .flatten()
                .filter_map(|e| {
                    if e.file_name().to_string_lossy().starts_with("nt-target-") {
                        e.metadata().ok().map(|m| m.len())
                    } else {
                        None
                    }
                })
                .sum();
            if total_size > 500 * 1024 * 1024 { // > 500MB orphan targets
                findings.push(WatchdogFinding {
                    check_id: "orphan_targets".into(),
                    domain: "build".into(),
                    severity: FindingSeverity::Warning,
                    message: format!("Orphan target dirs ({} MB)", total_size / 1024 / 1024),
                    suggested_action: Some("clean orphan targets".into()),
                });
            }
        }

        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_empty() {
        let mut reg = WatchdogRegistry::new();
        let findings = reg.run_all();
        assert!(findings.is_empty());
    }

    #[test]
    fn test_build_check() {
        let check = BuildCheck;
        let findings = check.check();
        // 不会 panic
        for f in &findings {
            assert!(!f.check_id.is_empty());
        }
    }
}
