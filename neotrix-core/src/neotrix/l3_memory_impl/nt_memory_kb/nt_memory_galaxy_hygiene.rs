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

use rusqlite::Connection;
use serde_json::Value;

use super::nt_memory_unify::{kv_get, kv_list};

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

/// 执行星系卫生检查。
///
/// 扫描 `consciousness` 命名空间的所有 hub，验证：
/// - 每个 hub 的 `route_table` 路由指向的 `branch_*` 真实存在 (幽灵分支预防)
/// - 每个 hub 的 `updated_at` 是否超过沉寂阈值 (星辰沉寂检测)
/// - 引力中枢 `router` 的 `cross_route` 指向的 hub 存在 (星系完整性)
///
/// `clean_ghosts` 为 true 时移除幽灵路由 (写回 hub)。
pub fn galaxy_hygiene_check(conn: &Connection, config: &GalaxyHygieneConfig) -> GalaxyHygieneReport {
    let mut report = GalaxyHygieneReport::default();

    // ── 1. 扫描所有 hub ──
    let hubs = match kv_list(conn, "consciousness") {
        Ok(list) => list,
        Err(e) => {
            report.findings.push(format!("[error] 无法扫描 consciousness 命名空间: {}", e));
            return report;
        }
    };

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let staleness_secs = config.staleness_days * 86400;

    for (key, value) in &hubs {
        // 只处理 hub 结构 (含 route_table 的 JSON)
        let Ok(v) = serde_json::from_str::<Value>(value) else { continue };
        let Some(hub) = v.get("hub").or_else(|| v.get("identity")) else { continue };
        report.hub_count += 1;

        // ── 幽灵分支预防: route_table 校验 ──
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
                        if branch_key.starts_with("branch_") && kv_get(conn, "consciousness", branch_key).is_ok_and(|x| x.is_some()) {
                            keep.push(b.clone());
                        } else {
                            report.ghost_branches += 1;
                            report.findings.push(format!(
                                "[ghost-branch] hub '{}' 路由 '{}' → '{}' 指向不存在的分支",
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
                if let Some(obj) = v.as_object() {
                    let mut hub_obj = hub.clone();
                    hub_obj["route_table"] = serde_json::json!(new_rt);
                    let mut new_v = obj.clone();
                    if let Some(h) = new_v.get_mut("hub") {
                        *h = hub_obj;
                    }
                    if let Ok(cleaned) = serde_json::to_string(&new_v) {
                        let _ = super::nt_memory_unify::kv_set(conn, "consciousness", key, &cleaned);
                    }
                }
            }
        }

        // ── 星辰沉寂检测: updated_at 校验 ──
        if let Some(ts) = hub.get("updated_at").and_then(|t| t.as_u64()) {
            if now_secs.saturating_sub(ts) > staleness_secs {
                report.stale_stars += 1;
                let days = (now_secs.saturating_sub(ts)) / 86400;
                report.findings.push(format!(
                    "[stale-star] hub '{}' 已沉寂 {} 天 (阈值 {} 天)",
                    key, days, config.staleness_days
                ));
            }
        }
    }

    // ── 星系完整性: 引力中枢 router 的 cross_route 校验 ──
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
                            // cross_route 目标可能是 hub key 或 skill id
                            if kv_get(conn, "consciousness", hub_key).is_err()
                                || kv_get(conn, "consciousness", hub_key).is_ok_and(|x| x.is_none())
                            {
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
}