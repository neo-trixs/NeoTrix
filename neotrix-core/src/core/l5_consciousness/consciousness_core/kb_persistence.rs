//! KB 快照持久化 — 读写 + 趋势追加 + 金标刷新

use super::core::CoreSnapshot;

/// KB namespace 与 key
pub(crate) const NAMESPACE: &str = "consciousness";
pub(crate) const KEY: &str = "core";

/// D1 口径标记常量 — NT-CORE 树快照口径
pub const PHI_SOURCE_TAG_TREE: &str = "tree_snapshot_iit";
/// D3 金标键
pub(crate) const GOLD_STANDARD_KEY: &str = "gold_standard";
/// D3 来源标注
pub(crate) const GOLD_STANDARD_SOURCE_CORE: &str = "core_snapshot";
/// D2 趋势序列上限
pub(crate) const TREND_CAP: usize = 128;
/// 金标双阈值本地镜像
pub(crate) const GOLD_STANDARD_PHI_THRESHOLD: f64 = 0.33;
pub(crate) const GOLD_STANDARD_COHERENCE_THRESHOLD: f64 = 0.7;

/// 打开 KB 连接
fn open_kb() -> Result<rusqlite::Connection, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let path = std::path::PathBuf::from(home)
        .join(".neotrix")
        .join("knowledge.db");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("KB dir: {}", e))?;
    }
    let conn = rusqlite::Connection::open(&path).map_err(|e| format!("KB open: {}", e))?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| format!("KB busy_timeout: {}", e))?;
    let _ = conn.execute_batch("PRAGMA journal_mode=WAL;");
    crate::core::nt_core_kb_primitives::schema_initialize(&conn)
        .map_err(|e| format!("KB init: {}", e))?;
    Ok(conn)
}

pub(crate) fn load_snapshot() -> Option<CoreSnapshot> {
    let conn = open_kb().ok()?;
    load_snapshot_from_conn(&conn)
}

/// 连接注入版快照读取
fn load_snapshot_from_conn(conn: &rusqlite::Connection) -> Option<CoreSnapshot> {
    let raw =
        crate::core::nt_core_kb_primitives::kv_get(conn, NAMESPACE, KEY).ok()??;
    serde_json::from_str(&raw).ok()
}

/// 持久化核心快照, 返回实际落盘的合并视图
pub(crate) fn persist_snapshot(snap: &CoreSnapshot) -> Result<CoreSnapshot, String> {
    let conn = open_kb()?;
    persist_snapshot_to_conn(&conn, snap)
}

/// 连接注入版持久化
pub(crate) fn persist_snapshot_to_conn(
    conn: &rusqlite::Connection,
    snap: &CoreSnapshot,
) -> Result<CoreSnapshot, String> {
    let mut out = snap.clone();
    let (prev_phi_hist, prev_coh_hist, prev_cycle) = match load_snapshot_from_conn(conn)
    {
        Some(prev) => (prev.phi_trend, prev.coherence_trend, Some(prev.cycle)),
        None => (Vec::new(), Vec::new(), None),
    };
    out.phi_trend = merge_trend_sample(prev_phi_hist, prev_cycle, snap.cycle, snap.phi);
    out.coherence_trend =
        merge_trend_sample(prev_coh_hist, prev_cycle, snap.cycle, snap.coherence);
    let json = serde_json::to_string(&out).map_err(|e| format!("snapshot serialize: {}", e))?;
    crate::core::nt_core_kb_primitives::kv_set(conn, NAMESPACE, KEY, &json)?;
    refresh_gold_standard(conn, &out);
    Ok(out)
}

/// D2 趋势采样合并
fn merge_trend_sample(
    mut hist: Vec<f64>,
    prev_cycle: Option<u64>,
    cur_cycle: u64,
    value: f64,
) -> Vec<f64> {
    match (hist.last_mut(), prev_cycle) {
        (Some(last), Some(c)) if c == cur_cycle => *last = value,
        _ => hist.push(value),
    }
    if hist.len() > TREND_CAP {
        let excess = hist.len() - TREND_CAP;
        hist.drain(..excess);
    }
    hist
}

/// D3 接线: core 快照落盘时同步刷新 `consciousness/gold_standard`
fn refresh_gold_standard(conn: &rusqlite::Connection, snap: &CoreSnapshot) {
    let is_phi_conscious = snap.phi > GOLD_STANDARD_PHI_THRESHOLD;
    let is_coherent = snap.coherence > GOLD_STANDARD_COHERENCE_THRESHOLD;
    let gs = serde_json::json!({
        "phi": snap.phi,
        "coherence": snap.coherence,
        "is_phi_conscious": is_phi_conscious,
        "is_coherent": is_coherent,
        "is_conscious_like": is_phi_conscious && is_coherent,
        "phi_confidence": snap.phi.clamp(0.0, 1.0),
        "coherence_confidence": snap.coherence.clamp(0.0, 1.0),
        "phi_threshold": GOLD_STANDARD_PHI_THRESHOLD,
        "coherence_threshold": GOLD_STANDARD_COHERENCE_THRESHOLD,
        "calibrated": true,
        "source": GOLD_STANDARD_SOURCE_CORE,
        "cycle": snap.cycle,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    let _ = crate::core::nt_core_kb_primitives::kv_set(
        conn,
        NAMESPACE,
        GOLD_STANDARD_KEY,
        &gs.to_string(),
    );
}

/// 从 KB 读取快照并重建树
pub(crate) fn load_or_new() -> crate::core::nt_core_consciousness_tree::ConsciousnessTree {
    use super::core::{core_snapshot_from_tree, tree_from_snapshot};
    let tree = crate::core::nt_core_consciousness_tree::ConsciousnessTree::new();
    match load_snapshot() {
        Some(snap) => tree_from_snapshot(&snap),
        None => tree,
    }
}
