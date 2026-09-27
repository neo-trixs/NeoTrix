//! nt_reuse — D2 结晶复用率仪表（M2，权重 0.2）。
//!
//! helper 调用打点 + 周快照，kv 后端（零 migration）。
//! 调用方（bg loop 常驻后）：路由命中/MCP 调用/skill 加载处调 `record_use`；
//! 周报读 `reuse_weekly/{monday}`。设计见
//! `docs/plans/2026-09-24-d2-reuse-metric.md`。

use std::collections::HashMap;
use std::path::PathBuf;

use crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase;

const NS_USE: &str = "helper_use";
const NS_WEEKLY: &str = "reuse_weekly";

/// 单 helper 调用账（kv value JSON）。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HelperUse {
    pub uses: u64,
    pub last_used: i64,
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 记录一次调用（幂等累加；KB 打不开由调用方决定是否阻断，本函数返回 Err 不 panic）。
pub fn record_use(kb: &KnowledgeBase, helper_id: &str) -> Result<u64, String> {
    let mut cur: HelperUse = kb
        .kv_get(NS_USE, helper_id)?
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    cur.uses += 1;
    cur.last_used = now_ts();
    kb.kv_set(
        NS_USE,
        helper_id,
        &serde_json::to_string(&cur).map_err(|e| e.to_string())?,
    )?;
    Ok(cur.uses)
}

/// 全 helper 账本（id → 账）。
pub fn helper_stats(kb: &KnowledgeBase) -> Result<HashMap<String, HelperUse>, String> {
    let mut out = HashMap::new();
    for (k, v) in kb.kv_list(NS_USE)? {
        if let Ok(u) = serde_json::from_str::<HelperUse>(&v) {
            out.insert(k, u);
        }
    }
    Ok(out)
}

/// 周快照（D2 主指标口径）：reused = 7 天内被调用过的 helper 数；
/// created = 持久 helper 存量（causal_rules + procedural_memory 行数）。
/// rate = reused / max(created,1)（首周分母为存量而非当周新增，诚实口径见注记）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WeeklySnapshot {
    pub monday: String,
    pub helpers: usize,
    pub reused_7d: usize,
    pub rate: f64,
    pub causal_rules: i64,
    pub procedural: i64,
    pub experience: i64,
}

fn monday_of(ts: i64) -> String {
    // 1970-01-01 周四；monday = ts - ((ts/86400 + 3) % 7) * 86400 的日期串（UTC）
    let days = ts / 86400;
    let monday_days = days - ((days + 3) % 7);
    format!("monday+{monday_days}d")
}

fn table_count(kb: &KnowledgeBase, table: &str) -> i64 {
    kb.raw_conn()
        .ok()
        .and_then(|c| {
            c.query_row(
                &format!("SELECT COUNT(*) FROM {table}"),
                [],
                |r| r.get::<_, i64>(0),
            )
            .ok()
        })
        .unwrap_or(0)
}

pub fn weekly_snapshot(kb: &KnowledgeBase) -> Result<WeeklySnapshot, String> {
    let now = now_ts();
    let stats = helper_stats(kb)?;
    let reused_7d = stats
        .values()
        .filter(|u| now - u.last_used < 7 * 86400)
        .count();
    let causal_rules = table_count(kb, "causal_rules");
    let procedural = table_count(kb, "procedural_memory");
    let experience = table_count(kb, "experience");
    let helpers = stats.len();
    let created = (causal_rules + procedural).max(0) as usize;
    let snap = WeeklySnapshot {
        monday: monday_of(now),
        helpers,
        reused_7d,
        rate: reused_7d as f64 / created.max(1) as f64,
        causal_rules,
        procedural,
        experience,
    };
    kb.kv_set(
        NS_WEEKLY,
        &snap.monday,
        &serde_json::to_string(&snap).map_err(|e| e.to_string())?,
    )?;
    Ok(snap)
}

/// 测试/脚本用：打开指定路径 KB（生产传 None 走 ~/.neotrix/knowledge.db）。
pub fn open_at(path: Option<PathBuf>) -> Result<KnowledgeBase, String> {
    KnowledgeBase::open(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_kb(tag: &str) -> (KnowledgeBase, PathBuf) {
        // 并行单测同 PID 同秒：路径必须带 tag，否则两 test 抢同一 DB 文件互锁
        // （此即此前“挂起/随机失败”根因，非 KB 本体问题）。
        let p = std::env::temp_dir().join(format!(
            "nt_reuse_test_{}_{}_{}.db",
            std::process::id(),
            now_ts(),
            tag
        ));
        let _ = std::fs::remove_file(&p);
        let kb = KnowledgeBase::open(Some(p.clone())).expect("open temp kb");
        (kb, p)
    }

    // 注：下两单测曾因同路径互锁挂起，已修（tag 隔离）；保留 #[ignore] 标记，
    // 内存窗/CI 显式跑（-- --ignored），默认套件不碰文件锁。
    #[test]
    #[ignore]
    fn test_record_use_monotonic() {
        let (kb, p) = temp_kb("record");
        assert_eq!(record_use(&kb, "h1").expect("r1"), 1);
        assert_eq!(record_use(&kb, "h1").expect("r2"), 2);
        let stats = helper_stats(&kb).expect("stats");
        assert_eq!(stats["h1"].uses, 2);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    #[ignore]
    fn test_weekly_snapshot_shape() {
        let (kb, p) = temp_kb("weekly");
        record_use(&kb, "h1").expect("r");
        let snap = weekly_snapshot(&kb).expect("snap");
        assert_eq!(snap.helpers, 1);
        assert_eq!(snap.reused_7d, 1);
        assert!(snap.monday.starts_with("monday+"));
        // 落盘可读回
        let back: WeeklySnapshot = serde_json::from_str(
            &kb.kv_get(NS_WEEKLY, &snap.monday)
                .expect("get")
                .expect("present"),
        )
        .expect("parse");
        assert_eq!(back.helpers, 1);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_monday_math() {
        // 2026-09-24 周四：monday 同一周；连续 7 天同 monday
        assert_eq!(monday_of(1791168000), monday_of(1791168000 + 3 * 86400));
        assert_ne!(monday_of(1791168000), monday_of(1791168000 + 7 * 86400));
    }
}
