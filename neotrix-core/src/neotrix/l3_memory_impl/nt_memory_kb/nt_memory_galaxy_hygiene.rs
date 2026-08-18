//! 星系卫生代码强制 — Consciousness Galaxy Hygiene Enforcement (NT-MEMORY)
//!
//! 将 star-memory skill 的星系卫生法则强制为可执行代码，运行于 KB `consciousness`
//! 命名空间。三条法则：
//!
//! 1. **幽灵分支预防** (Ghost Branch Prevention): 每个 hub 的 `route_table` 中
//!    每条路由指向的 `branch_*` key 必须真实存在于 kv_store。缺失即幽灵分支。
//! 2. **星辰沉寂检测** (Star Staleness Detection): 检测长时间未更新的星辰
//!    (hub 的 `updated_at` 超过阈值)，标记为沉寂。
//! 3. **星系完整性验证** (Galaxy Integrity): 验证所有 hub 存在、route_table 非空、
//!    共鸣通道 (cross_route) 指向的 hub 存在。
//!
//! 与 experience.rs 的 `cmd_route_verify` (运维侧手工巡检) 互补：本模块是
//! 生产路径的强制检查，可被 BackgroundLoop 周期调用 (T3 生产接线)。
//!
//! 2026-08-18 修复 (深度分析): hub 真实存放于各技能 namespace 的 `key='hub'`
//! (含 `star_name` 字段)，而非 `consciousness` 命名空间。原实现对真实星系
//! 完全不可见 (hub_count≈0 / 沉寂检测永不触发)。现已改为跨 namespace 发现
//! 真实 hub；`last_loaded` (ISO 字符串) 为活跃信号字段，替代不存在的
//! `updated_at`。同时新增原生 `galaxy_wake_star` 唤醒路径 (对应
//! `nt_csgn_evolve.py record`)，供生产接线。

use rusqlite::Connection;
use serde_json::Value;

use super::nt_memory_unify::{kv_exists, kv_get, kv_list, kv_list_namespaces, kv_set};

/// 星系卫生检查报告
#[derive(Debug, Default, Clone)]
pub struct GalaxyHygieneReport {
    /// 幽灵分支数 (route_table 指向不存在的 branch_*)
    pub ghost_branches: usize,
    /// 沉寂星辰数 (超过 staleness_days 未更新)
    pub stale_stars: usize,
    /// 缺失 hub 数 (cross_route 指向不存在的 hub)
    pub missing_hubs: usize,
    /// 空 route_table 的 hub 数
    pub empty_route_tables: usize,
    /// 检查的 hub 总数
    pub hub_count: usize,
    /// 具体发现 (人类可读)
    pub findings: Vec<String>,
}

impl GalaxyHygieneReport {
    pub fn is_clean(&self) -> bool {
        self.ghost_branches == 0
            && self.stale_stars == 0
            && self.missing_hubs == 0
            && self.empty_route_tables == 0
    }
}

/// 星系卫生检查配置
#[derive(Debug, Clone)]
pub struct GalaxyHygieneConfig {
    /// 星辰沉寂阈值 (天)
    pub staleness_days: u64,
    /// 是否清理幽灵分支 (否则仅报告)
    pub clean_ghosts: bool,
}

impl Default for GalaxyHygieneConfig {
    fn default() -> Self {
        Self {
            staleness_days: 90,
            clean_ghosts: false,
        }
    }
}

/// 解析 hub 活跃时间字段 → epoch 秒。
/// 真实 hub 用 ISO 字符串 `last_loaded` ("2026-08-11T13:18:23.829840" 或 "None")，
/// 旧格式用 `updated_at` epoch 秒。返回 None 表示从未活跃。
pub fn hub_active_epoch(hub: &Value) -> Option<u64> {
    if let Some(ts) = hub.get("updated_at").and_then(|t| t.as_u64()) {
        return Some(ts);
    }
    let raw = hub.get("last_loaded")?.as_str()?;
    if raw.is_empty() || raw == "None" {
        return None;
    }
    // ISO 8601: YYYY-MM-DDTHH:MM:SS[.ffffff][Z|+off] — 容错截断
    let s = raw.trim();
    let s = s.strip_suffix('Z').unwrap_or(s);
    let s = s.split('+').next().unwrap_or(s);
    let s = s.split('.').next().unwrap_or(s);
    let Ok(ts) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") else {
        return None;
    };
    Some(ts.and_utc().timestamp().max(0) as u64)
}

/// 发现真实星系 hub: 扫描所有 namespace 下 `key='hub'` 且含 `star_name` 的星辰。
/// 兼容压缩值 (NTZ1: Rust 侧不解码, 跳过) 与非星辰 hub (experience / domain_nt_*)。
pub fn galaxy_list_hubs(conn: &Connection) -> Vec<(String, Value)> {
    let mut hubs = Vec::new();
    let Ok(nss) = kv_list_namespaces(conn) else { return hubs };
    for ns in nss {
        let Ok(Some(raw)) = kv_get(conn, &ns, "hub") else { continue };
        let Ok(v) = serde_json::from_str::<Value>(&raw) else { continue };
        if v.get("star_name").is_some() {
            hubs.push((ns, v));
        }
    }
    hubs
}

/// 原生星辰唤醒 (对应 `nt_csgn_evolve.py record`): `invocations++`、
/// `last_loaded` 置为当前 ISO 时间、`metrics.total_runs++`。R-P97 写回单一事实源。
pub fn galaxy_wake_star(conn: &Connection, ns: &str) -> Result<String, String> {
    let raw = kv_get(conn, ns, "hub")
        .map_err(|e| format!("hub 读取失败 ({ns}): {e}"))?
        .ok_or_else(|| format!("星辰 {ns} 不存在 hub"))?;
    let mut hub: Value =
        serde_json::from_str(&raw).map_err(|e| format!("hub 解析失败 ({ns}): {e}"))?;
    if hub.get("star_name").is_none() {
        return Err(format!("{ns} 非星辰 hub, 拒绝唤醒"));
    }
    let inv = hub.get("invocations").and_then(|v| v.as_u64()).unwrap_or(0) + 1;
    hub["invocations"] = serde_json::json!(inv);
    hub["last_loaded"] = serde_json::json!(
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f").to_string()
    );
    let total = hub
        .get("metrics")
        .and_then(|m| m.get("total_runs"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        + 1;
    hub["metrics"] = serde_json::json!({ "total_runs": total });
    let out = serde_json::to_string(&hub).map_err(|e| format!("hub 序列化失败: {e}"))?;
    kv_set(conn, ns, "hub", &out)?;
    Ok(format!(
        "[csgn] ✓ {ns} 唤醒 #{inv} (last: {})",
        hub["last_loaded"].as_str().unwrap_or("")
    ))
}

/// 沉寂星辰扫描: 返回 `(ns, 上次活跃 epoch, invocations)`，`None` 表示从未加载。
/// 对应 `nt_csgn_evolve.py stale` 的生产路径 (供 BackgroundLoop 巡检)。
pub fn galaxy_wake_scan(conn: &Connection, staleness_days: u64) -> Vec<(String, Option<u64>, u64)> {
    let cutoff = chrono::Utc::now().timestamp().max(0) as u64 - staleness_days * 86400;
    let mut dormant = Vec::new();
    for (ns, hub) in galaxy_list_hubs(conn) {
        let runs = hub.get("invocations").and_then(|v| v.as_u64()).unwrap_or(0);
        match hub_active_epoch(&hub) {
            None => dormant.push((ns, None, runs)),
            Some(ts) if ts < cutoff => dormant.push((ns, Some(ts), runs)),
            _ => {}
        }
    }
    dormant
}

/// 执行星系卫生检查。
///
/// 跨 namespace 发现真实 hub (各技能 namespace 的 `key='hub'`)，验证：
/// - 每个 hub 的 `route_table` 路由指向的 key 在该 namespace 真实存在 (幽灵分支预防)
/// - 每个 hub 的 `last_loaded`/`updated_at` 是否超过沉寂阈值 (星辰沉寂检测)
/// - 引力中枢 `router` 的 `cross_route` 指向的 hub 存在 (星系完整性)
///
/// `clean_ghosts` 为 true 时移除幽灵路由 (写回 hub)。
pub fn galaxy_hygiene_check(conn: &Connection, config: &GalaxyHygieneConfig) -> GalaxyHygieneReport {
    let mut report = GalaxyHygieneReport::default();

    let staleness_secs = config.staleness_days * 86400;
    let now_secs = chrono::Utc::now().timestamp().max(0) as u64;

    // ── 1. 跨 namespace 发现真实 hub (is_real=true: 存于 ns 的 key='hub') ──
    let mut hubs: Vec<(String, Value, bool)> = galaxy_list_hubs(conn)
        .into_iter()
        .map(|(ns, v)| (ns, v, true))
        .collect();

    // 兼容旧格式: consciousness 命名空间下 `{"hub": {...}}` 包裹结构 (is_real=false)
    if let Ok(list) = kv_list(conn, "consciousness") {
        for (key, value) in list {
            if let Ok(v) = serde_json::from_str::<Value>(&value) {
                if v.get("hub").is_some() || v.get("identity").is_some() {
                    hubs.push((key, v, false));
                }
            }
        }
    }

    for (key, v, is_real) in &hubs {
        // 真实 hub 顶层即 hub 结构; 旧格式在 "hub"/"identity" 包裹
        let hub = if v.get("star_name").is_some() {
            v
        } else {
            match v.get("hub").or_else(|| v.get("identity")) {
                Some(w) => w,
                None => continue,
            }
        };
        report.hub_count += 1;

        // ── 幽灵分支预防: route_table 校验 (目标 key 在同 namespace 存在) ──
        if let Some(rt) = hub.get("route_table").and_then(|r| r.as_object()) {
            if rt.is_empty() {
                report.empty_route_tables += 1;
                report.findings.push(format!("[empty-route] hub '{}' 的 route_table 为空", key));
            }
            let mut new_rt: serde_json::Map<String, Value> = serde_json::Map::new();
            for (kw, arr) in rt {
                let mut keep: Vec<Value> = Vec::new();
                if let Some(list) = arr.as_array() {
                    for b in list {
                        let branch_key = b.as_str().unwrap_or("");
                        if branch_key.is_empty() {
                            continue;
                        }
                        // 目标 key 存在性: 真实 hub 查自身 namespace, 兼容旧格式
                        // (consciousness 下 branch_*)
                        let exists = if *is_real {
                            kv_get(conn, key, branch_key).is_ok_and(|x| x.is_some())
                        } else {
                            kv_get(conn, "consciousness", branch_key).is_ok_and(|x| x.is_some())
                        };
                        if exists {
                            keep.push(b.clone());
                        } else {
                            report.ghost_branches += 1;
                            report.findings.push(format!(
                                "[ghost-branch] hub '{}' 路由 '{}' → '{}' 指向不存在的 key",
                                key, kw, branch_key
                            ));
                        }
                    }
                }
                if !keep.is_empty() {
                    new_rt.insert(kw.clone(), serde_json::json!(keep));
                }
            }
            // 清理幽灵路由 (写回)
            if config.clean_ghosts && report.ghost_branches > 0 {
                let mut cleaned = v.clone();
                if *is_real {
                    cleaned["route_table"] = serde_json::json!(new_rt);
                    if let Ok(out) = serde_json::to_string(&cleaned) {
                        let _ = kv_set(conn, key, "hub", &out);
                    }
                } else if let Some(obj) = cleaned.as_object_mut() {
                    if let Some(h) = obj.get_mut("hub") {
                        h["route_table"] = serde_json::json!(new_rt);
                    }
                    if let Ok(out) = serde_json::to_string(&cleaned) {
                        let _ = kv_set(conn, "consciousness", key, &out);
                    }
                }
            }
        }

        // ── 星辰沉寂检测: last_loaded (ISO) / updated_at (epoch) 校验 ──
        match hub_active_epoch(hub) {
            None => {
                report.stale_stars += 1;
                report.findings.push(format!(
                    "[stale-star] hub '{}' 从未被唤醒 (invocations={})",
                    key,
                    hub.get("invocations").and_then(|i| i.as_u64()).unwrap_or(0)
                ));
            }
            Some(ts) if now_secs.saturating_sub(ts) > staleness_secs => {
                report.stale_stars += 1;
                let days = (now_secs.saturating_sub(ts)) / 86400;
                report.findings.push(format!(
                    "[stale-star] hub '{}' 已沉寂 {} 天 (阈值 {} 天)",
                    key, days, config.staleness_days
                ));
            }
            _ => {}
        }
    }

    // ── 星系完整性: 引力中枢 router 的 cross_route 校验 ──
    let known_hubs: std::collections::HashSet<String> = hubs.iter().map(|(k, _, _)| k.clone()).collect();
    if let Ok(Some(router_json)) = kv_get(conn, "consciousness", "router") {
        if let Ok(router) = serde_json::from_str::<Value>(&router_json) {
            if let Some(cross) = router.get("cross_route").and_then(|c| c.as_object()) {
                for (kw, targets) in cross {
                    if let Some(arr) = targets.as_array() {
                        for t in arr {
                            let hub_key = t.as_str().unwrap_or("");
                            if hub_key.is_empty() {
                                continue;
                            }
                            // cross_route 目标可能是 namespace 或 consciousness 键; 用 kv_exists
                            // (压缩值也算存在) 避免把 NTZ1 hub 误判为缺失。
                            let exists = known_hubs.contains(hub_key)
                                || kv_exists(conn, "consciousness", hub_key).is_ok_and(|x| x)
                                || kv_exists(conn, hub_key, "hub").is_ok_and(|x| x);
                            if !exists {
                                report.missing_hubs += 1;
                                report.findings.push(format!(
                                    "[missing-hub] cross_route '{}' → '{}' 指向不存在的 hub",
                                    kw, hub_key
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    report
}

/// 便捷函数: 执行检查并返回人类可读摘要 (供 CLI / BackgroundLoop 接线)
pub fn galaxy_hygiene_summary(report: &GalaxyHygieneReport) -> String {
    if report.is_clean() {
        format!(
            "[galaxy-hygiene] ✅ 星系卫生: {} hubs 全部健康 (0 ghost / 0 stale / 0 missing)",
            report.hub_count
        )
    } else {
        format!(
            "[galaxy-hygiene] ⚠️ 星系卫生: {} hubs, {} ghost-branch, {} stale-star, {} missing-hub, {} empty-route",
            report.hub_count,
            report.ghost_branches,
            report.stale_stars,
            report.missing_hubs,
            report.empty_route_tables
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        super::super::nt_memory_schema::initialize(&conn).expect("schema init");
        conn
    }

    #[test]
    fn test_clean_galaxy_no_findings() {
        let conn = test_conn();
        // 建一个健康 hub: route_table 指向存在的 branch
        let hub = serde_json::json!({
            "hub": {
                "identity": {"name": "test-star"},
                "route_table": {"review": ["branch_a"]},
                "updated_at": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            }
        });
        super::super::nt_memory_unify::kv_set(&conn, "consciousness", "test_hub", &hub.to_string()).unwrap();
        super::super::nt_memory_unify::kv_set(&conn, "consciousness", "branch_a", "{}").unwrap();

        let report = galaxy_hygiene_check(&conn, &GalaxyHygieneConfig::default());
        assert!(report.is_clean(), "健康星系应无发现: {:?}", report.findings);
        assert_eq!(report.hub_count, 1);
    }

    #[test]
    fn test_ghost_branch_detected() {
        let conn = test_conn();
        let hub = serde_json::json!({
            "hub": {
                "route_table": {"review": ["branch_ghost"]},
                "updated_at": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            }
        });
        super::super::nt_memory_unify::kv_set(&conn, "consciousness", "test_hub", &hub.to_string()).unwrap();
        // branch_ghost 不存在 → 幽灵分支

        let report = galaxy_hygiene_check(&conn, &GalaxyHygieneConfig::default());
        assert_eq!(report.ghost_branches, 1, "应检测到 1 个幽灵分支");
        assert!(!report.is_clean());
    }

    #[test]
    fn test_stale_star_detected() {
        let conn = test_conn();
        let old_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() - 200 * 86400;
        let hub = serde_json::json!({
            "hub": {
                "route_table": {},
                "updated_at": old_ts,
            }
        });
        super::super::nt_memory_unify::kv_set(&conn, "consciousness", "old_hub", &hub.to_string()).unwrap();

        let report = galaxy_hygiene_check(&conn, &GalaxyHygieneConfig::default());
        assert_eq!(report.stale_stars, 1, "应检测到 1 个沉寂星辰");
    }

    #[test]
    fn test_missing_hub_in_cross_route() {
        let conn = test_conn();
        let router = serde_json::json!({
            "cross_route": {"review": ["nonexistent_hub"]}
        });
        super::super::nt_memory_unify::kv_set(&conn, "consciousness", "router", &router.to_string()).unwrap();

        let report = galaxy_hygiene_check(&conn, &GalaxyHygieneConfig::default());
        assert_eq!(report.missing_hubs, 1, "应检测到 1 个缺失 hub");
    }

    #[test]
    fn test_clean_ghosts_removes_route() {
        let conn = test_conn();
        let hub = serde_json::json!({
            "hub": {
                "route_table": {"review": ["branch_ghost", "branch_real"]},
                "updated_at": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            }
        });
        super::super::nt_memory_unify::kv_set(&conn, "consciousness", "test_hub", &hub.to_string()).unwrap();
        super::super::nt_memory_unify::kv_set(&conn, "consciousness", "branch_real", "{}").unwrap();

        let mut config = GalaxyHygieneConfig::default();
        config.clean_ghosts = true;
        let report = galaxy_hygiene_check(&conn, &GalaxyHygieneConfig { clean_ghosts: true, ..config });
        assert_eq!(report.ghost_branches, 1);

        // 验证写回: route_table 只剩 branch_real
        let saved = super::super::nt_memory_unify::kv_get(&conn, "consciousness", "test_hub").unwrap().unwrap();
        let v: Value = serde_json::from_str(&saved).unwrap();
        let rt = v["hub"]["route_table"]["review"].as_array().unwrap();
        assert_eq!(rt.len(), 1, "清理后应只剩 1 条路由");
        assert_eq!(rt[0], "branch_real");
    }

    fn insert_real_hub(conn: &Connection, ns: &str, invocations: u64, last_loaded: Option<&str>) {
        let mut hub = serde_json::json!({
            "version": 3, "id": ns, "level": "L1", "domain": "NT-ACT",
            "star_name": format!("Test-{}", ns), "role": "测试星辰",
            "route_table": {ns: ["hub"]},
            "invocations": invocations,
            "metrics": {"total_runs": invocations},
        });
        if let Some(ll) = last_loaded {
            hub["last_loaded"] = serde_json::json!(ll);
        }
        super::super::nt_memory_unify::kv_set(conn, ns, "hub", &hub.to_string()).unwrap();
    }

    #[test]
    fn test_real_hub_discovery_finds_star_hubs() {
        let conn = test_conn();
        insert_real_hub(&conn, "star_a", 0, None);
        insert_real_hub(&conn, "star_b", 2, Some("2026-08-01T10:00:00"));
        // 非星辰 hub (experience 式) 不应被发现
        super::super::nt_memory_unify::kv_set(&conn, "plain_ns", "hub", r#"{"no_star": true}"#).unwrap();

        let hubs = galaxy_list_hubs(&conn);
        assert_eq!(hubs.len(), 2, "应发现 2 颗星辰");
        assert_eq!(hubs[0].0, "star_a");
    }

    #[test]
    fn test_galaxy_wake_star_increments_and_stamps() {
        let conn = test_conn();
        insert_real_hub(&conn, "star_a", 3, Some("2026-08-01T10:00:00"));

        let msg = galaxy_wake_star(&conn, "star_a").unwrap();
        assert!(msg.contains("唤醒 #4"), "第 4 次唤醒: {}", msg);

        let raw = super::super::nt_memory_unify::kv_get(&conn, "star_a", "hub").unwrap().unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["invocations"].as_u64(), Some(4));
        assert_eq!(v["metrics"]["total_runs"].as_u64(), Some(4));
        let last = v["last_loaded"].as_str().unwrap();
        assert!(last.starts_with("2026-"), "last_loaded 应为当前 ISO: {}", last);
    }

    #[test]
    fn test_galaxy_wake_star_rejects_non_star() {
        let conn = test_conn();
        super::super::nt_memory_unify::kv_set(&conn, "plain_ns", "hub", r#"{"no_star": true}"#).unwrap();
        assert!(galaxy_wake_star(&conn, "plain_ns").is_err(), "非星辰应拒绝唤醒");
    }

    #[test]
    fn test_hub_active_epoch_iso_and_none() {
        let v = serde_json::json!({"last_loaded": "2026-08-11T13:18:23.829840"});
        let ts = hub_active_epoch(&v).unwrap();
        // 2026-08-11T13:18:23Z 与 now 相差远小于 90 天 → 不沉寂 (用回归断言)
        let now = chrono::Utc::now().timestamp().max(0) as u64;
        assert!(now.saturating_sub(ts) < 30 * 86400);

        assert!(hub_active_epoch(&serde_json::json!({"last_loaded": "None"})).is_none());
        assert!(hub_active_epoch(&serde_json::json!({})).is_none());
        assert!(hub_active_epoch(&serde_json::json!({"last_loaded": "2026-08-11T13:18:23Z"})).is_some());
        assert!(hub_active_epoch(&serde_json::json!({"updated_at": 1700000000u64})).is_some());
    }

    #[test]
    fn test_real_hub_hygiene_detects_never_loaded() {
        let conn = test_conn();
        // 真实 hub: 无 last_loaded → 从未唤醒 → 沉寂
        insert_real_hub(&conn, "star_dormant", 0, None);
        // 真实 hub: last_loaded 200 天前 → 沉寂
        insert_real_hub(&conn, "star_old", 0, Some("2025-01-01T10:00:00"));
        // 真实 hub: 刚唤醒 → 健康
        insert_real_hub(&conn, "star_fresh", 1, Some("2026-08-18T10:00:00"));

        let report = galaxy_hygiene_check(&conn, &GalaxyHygieneConfig::default());
        assert_eq!(report.hub_count, 3, "应发现全部 3 颗真实星辰");
        assert_eq!(report.stale_stars, 2, "2 颗沉寂 (从未加载 + 200 天前)");
        assert!(!report.is_clean());
    }

    #[test]
    fn test_wake_scan_lists_dormant() {
        let conn = test_conn();
        insert_real_hub(&conn, "star_dormant", 0, None);
        insert_real_hub(&conn, "star_fresh", 1, Some("2026-08-18T10:00:00"));

        let dormant = galaxy_wake_scan(&conn, 90);
        assert_eq!(dormant.len(), 1, "仅 1 颗沉寂");
        assert_eq!(dormant[0].0, "star_dormant");
        assert_eq!(dormant[0].1, None, "从未加载");
    }

    /// 只读集成验证: 对真实 KB (~/.neotrix/knowledge.db) 执行 hygiene 检查。
    /// 不写入, 仅报告当前星系状态。`cargo test -- --ignored galaxy` 手动触发。
    #[test]
    #[ignore]
    fn test_real_kb_hygiene_readonly() {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_path = std::path::PathBuf::from(&home).join(".neotrix").join("knowledge.db");
        let Ok(conn) = Connection::open(&db_path) else {
            eprintln!("无法打开真实 KB: {}", db_path.display());
            return;
        };
        let hubs = galaxy_list_hubs(&conn);
        println!("[integration] 真实星系 hub 数: {}", hubs.len());
        let nss = kv_list_namespaces(&conn).unwrap_or_default();
        println!("[integration] kv_list_namespaces 返回 {} 个 namespace", nss.len());
        let mut hub_ns = 0usize;
        for ns in &nss {
            match kv_get(&conn, ns, "hub") {
                Ok(Some(raw)) => {
                    hub_ns += 1;
                    if !raw.contains("\"star_name\"") && hub_ns < 6 {
                        println!("[integration]   ns={} hub 无 star_name (len={})", ns, raw.len());
                    }
                }
                Ok(None) => {}
                Err(e) => println!("[integration]   ns={} kv_get err: {}", ns, e),
            }
        }
        println!("[integration] 命中 hub key 的 namespace 数: {}", hub_ns);
        for (ns, hub) in hubs.iter().take(5) {
            let inv = hub.get("invocations").and_then(|i| i.as_u64()).unwrap_or(0);
            println!("[integration]   - {} (invocations={}, last={})", ns, inv,
                hub.get("last_loaded").and_then(|l| l.as_str()).unwrap_or("None"));
        }
        let report = galaxy_hygiene_check(&conn, &GalaxyHygieneConfig::default());
        println!("[integration] hygiene: {} hubs, {} ghost, {} stale, {} missing, {} empty",
            report.hub_count, report.ghost_branches, report.stale_stars,
            report.missing_hubs, report.empty_route_tables);
        for f in report.findings.iter().take(8) {
            println!("[integration]   finding: {}", f);
        }
        let dormant = galaxy_wake_scan(&conn, 90);
        println!("[integration] 沉寂星辰 (90 天): {}", dormant.len());
        for f in report.findings.iter().filter(|f| f.contains("missing-hub")) {
            println!("[integration]   missing: {}", f);
        }
        assert!(report.hub_count >= 50, "真实星系应发现 50+ 星辰, 实得 {}", report.hub_count);
    }
}