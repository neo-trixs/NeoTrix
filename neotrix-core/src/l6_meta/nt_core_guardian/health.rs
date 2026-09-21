//! # HEALTH — 统一健康状态 (融合 3 个分散实现)
//!
//! 融合:
//! - `self_healing/health_monitor.rs` — 组件级指标监控
//! - `l1_facade::self_audit::scan_system_health` — 系统级健康扫描
//! - `l6_meta/healing/nt_mind_consciousness_monitor.rs` — 意识层监控
//!
//! 单一事实源: 所有健康检查结果汇总到 HealthState,
//! 供 supervisor 判定是否需要重启, 供 EventBus 广播给意识层。

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

/// 健康状态等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HealthLevel {
    /// 组件正常
    Healthy,
    /// 性能下降但仍在服务
    Degraded,
    /// 不可用, 需要修复
    Critical,
    /// 不可知 (未检查)
    Unknown,
}

impl std::fmt::Display for HealthLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Critical => write!(f, "critical"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// 单个组件的健康快照
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub name: String,
    pub level: HealthLevel,
    pub message: String,
    pub metrics: HashMap<String, f64>,
    pub checked_at_ms: i64,
}

/// 统一健康状态 (Arc 共享, 原子更新)
#[derive(Debug)]
pub struct HealthState {
    /// 全局健康等级 (原子, 无锁读取)
    global_level: std::sync::atomic::AtomicU8,
    /// 各组件健康状态
    components: std::sync::Mutex<HashMap<String, ComponentHealth>>,
    /// 最后一次全面扫描时间
    last_full_scan_ms: AtomicI64,
    /// 发现的问题总数
    total_findings: AtomicI64,
}

impl HealthState {
    pub fn new() -> Self {
        Self {
            global_level: std::sync::atomic::AtomicU8::new(HealthLevel::Unknown as u8),
            components: std::sync::Mutex::new(HashMap::new()),
            last_full_scan_ms: AtomicI64::new(0),
            total_findings: AtomicI64::new(0),
        }
    }

    /// 更新单个组件健康状态
    pub fn update_component(&self, health: ComponentHealth) {
        let level = health.level;
        if let Ok(mut guard) = self.components.lock() {
            guard.insert(health.name.clone(), health);
        }
        // 更新全局等级: 取所有组件最差等级
        self.recompute_global(level);
    }

    /// 批量更新 (一次扫描的结果)
    pub fn update_batch(&self, findings: Vec<ComponentHealth>) {
        if let Ok(mut guard) = self.components.lock() {
            for f in findings {
                guard.insert(f.name.clone(), f);
            }
        }
        self.recompute_global(HealthLevel::Healthy);
    }

    /// 获取全局健康等级 (原子读, 无锁)
    pub fn global_level(&self) -> HealthLevel {
        match self.global_level.load(Ordering::Relaxed) {
            0 => HealthLevel::Healthy,
            1 => HealthLevel::Degraded,
            2 => HealthLevel::Critical,
            _ => HealthLevel::Unknown,
        }
    }

    /// 是否健康 (用于 supervisor 判定)
    pub fn is_healthy(&self) -> bool {
        self.global_level() == HealthLevel::Healthy
    }

    /// 获取所有组件状态
    pub fn get_all_components(&self) -> Vec<ComponentHealth> {
        self.components
            .lock()
            .map(|g| g.values().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取指定组件状态
    pub fn get_component(&self, name: &str) -> Option<ComponentHealth> {
        self.components.lock().ok()?.get(name).cloned()
    }

    /// 记录发现的问题数
    pub fn record_findings(&self, count: i64) {
        self.total_findings.fetch_add(count, Ordering::Relaxed);
    }

    /// 获取问题总数
    pub fn total_findings(&self) -> i64 {
        self.total_findings.load(Ordering::Relaxed)
    }

    /// 更新扫描时间
    pub fn touch_scan(&self) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        self.last_full_scan_ms.store(ts, Ordering::Relaxed);
    }

    /// 快照摘要 (供日志/EventBus)
    pub fn summary(&self) -> String {
        let level = self.global_level();
        let findings = self.total_findings();
        let components = self.components.lock().map(|g| g.len()).unwrap_or(0);
        format!(
            "health={} findings={} components={}",
            level, findings, components
        )
    }

    fn recompute_global(&self, _worst: HealthLevel) {
        // 遍历所有组件取最差等级
        if let Ok(guard) = self.components.lock() {
            let worst = guard
                .values()
                .map(|c| c.level)
                .max()
                .unwrap_or(HealthLevel::Unknown);
            self.global_level.store(worst as u8, Ordering::Relaxed);
        }
    }
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行系统级健康扫描 (融合 self_audit + component check)
pub fn scan_system_health(_health: &HealthState) -> Vec<ComponentHealth> {
    let mut findings = Vec::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    // 1. 磁盘空间检查
    if let Ok(output) = std::process::Command::new("df")
        .args(["-m", "/"])
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            // 简单解析: 第二行是使用率
            if let Some(line) = stdout.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let used_pct: f64 = parts[4].trim_end_matches('%').parse().unwrap_or(0.0);
                    let level = if used_pct > 95.0 {
                        HealthLevel::Critical
                    } else if used_pct > 85.0 {
                        HealthLevel::Degraded
                    } else {
                        HealthLevel::Healthy
                    };
                    findings.push(ComponentHealth {
                        name: "disk".into(),
                        level,
                        message: format!("disk usage: {}%", used_pct),
                        metrics: HashMap::from([("usage_pct".into(), used_pct)]),
                        checked_at_ms: now,
                    });
                }
            }
        }
    }

    // 2. 内存检查
    if let Ok(output) = std::process::Command::new("vm_stat")
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            let pages_free = stdout
                .lines()
                .find(|l| l.contains("Pages free"))
                .and_then(|l| l.split(':').nth(1))
                .and_then(|v| v.trim().trim_end_matches('.').parse::<f64>().ok())
                .unwrap_or(0.0);
            let pages_active = stdout
                .lines()
                .find(|l| l.contains("Pages active"))
                .and_then(|l| l.split(':').nth(1))
                .and_then(|v| v.trim().trim_end_matches('.').parse::<f64>().ok())
                .unwrap_or(0.0);
            let total = pages_free + pages_active;
            if total > 0.0 {
                let used_pct = (pages_active / total * 100.0).min(100.0);
                let level = if used_pct > 95.0 {
                    HealthLevel::Critical
                } else if used_pct > 85.0 {
                    HealthLevel::Degraded
                } else {
                    HealthLevel::Healthy
                };
                findings.push(ComponentHealth {
                    name: "memory".into(),
                    level,
                    message: format!("memory usage: {:.1}%", used_pct),
                    metrics: HashMap::from([("usage_pct".into(), used_pct)]),
                    checked_at_ms: now,
                });
            }
        }
    }

    // 3. KB 健康检查
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let kb_path = std::path::PathBuf::from(home).join(".neotrix/knowledge.db");
        let level = if !kb_path.exists() {
            HealthLevel::Critical
        } else if let Ok(meta) = std::fs::metadata(&kb_path) {
            if meta.len() < 1024 {
                HealthLevel::Critical
            } else {
                HealthLevel::Healthy
            }
        } else {
            HealthLevel::Unknown
        };
        findings.push(ComponentHealth {
            name: "kb".into(),
            level,
            message: format!("KB at {}", kb_path.display()),
            metrics: HashMap::new(),
            checked_at_ms: now,
        });
    }

    // 4. 构建产物检查
    {
        let target_dir = std::path::Path::new("target/debug/neotrix");
        let level = if target_dir.exists() {
            HealthLevel::Healthy
        } else {
            HealthLevel::Degraded
        };
        findings.push(ComponentHealth {
            name: "build".into(),
            level,
            message: if target_dir.exists() {
                "build artifact exists".into()
            } else {
                "build artifact missing".into()
            },
            metrics: HashMap::new(),
            checked_at_ms: now,
        });
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_state_default() {
        let state = HealthState::new();
        assert_eq!(state.global_level(), HealthLevel::Unknown);
        assert!(state.get_all_components().is_empty());
    }

    #[test]
    fn test_update_component() {
        let state = HealthState::new();
        state.update_component(ComponentHealth {
            name: "test".into(),
            level: HealthLevel::Healthy,
            message: "ok".into(),
            metrics: HashMap::new(),
            checked_at_ms: 0,
        });
        assert_eq!(state.global_level(), HealthLevel::Healthy);
    }

    #[test]
    fn test_worst_level_propagates() {
        let state = HealthState::new();
        state.update_component(ComponentHealth {
            name: "a".into(),
            level: HealthLevel::Healthy,
            message: "ok".into(),
            metrics: HashMap::new(),
            checked_at_ms: 0,
        });
        state.update_component(ComponentHealth {
            name: "b".into(),
            level: HealthLevel::Critical,
            message: "down".into(),
            metrics: HashMap::new(),
            checked_at_ms: 0,
        });
        assert_eq!(state.global_level(), HealthLevel::Critical);
        assert!(!state.is_healthy());
    }
}
