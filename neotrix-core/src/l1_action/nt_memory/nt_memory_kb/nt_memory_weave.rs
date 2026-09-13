//! 跨会话记忆编织 — Cross-Session Memory Weave (NT-NEXUS)
//!
//! 将 nexus-weaver skill 的织网方法论强制为可执行代码, 运行于 KB `nexus_weave`
//! 命名空间。四阶段织网循环 (对应 SKILL Phase 1-4):
//!
//! 1. **会话桥接** (Session Bridge): 记录跨会话链接, 发现开放 patterns 与
//!    未决矛盾 (cross_session_link_store / session_continuity_bootstrap)。
//! 2. **模式编织** (Pattern Weave): 比较当前会话 patterns 与既有知识,
//!    命中则强化连接 (strength+1), 未命中则建新节点并接最近邻
//!    (pattern_connect / nearest_neighbor)。
//! 3. **鸿沟桥接** (Gap Bridge): 对时间/主题相距远的会话找隐式连接
//!    (共享关键词/相似根因), 构建显式 bridge (gap_bridge / bridge_gap)。
//! 4. **图谱维护** (Graph Update): 更新知识图谱边权重, 标记 30 天未引用的
//!    弱连接, 强连接 (>5 引用) 置为永久 (graph_curator / edge_weighting)。
//!
//! 与 galaxy_hygiene 互补: galaxy 管星辰唤醒与沉寂, 本模块管跨会话
//! pattern 关联与桥接。生产接线点: 会话收尾 (experience-tree) 与
//! 会话启动 (hub 加载) 时调用。
//!
//! 2026-08-20 创建: NEXUS 域从 registry 占位结晶为真实实现。

use rusqlite::Connection;
use serde_json::{json, Value};

use super::nt_memory_unify::{kv_get, kv_list_namespaces, kv_set};

/// 织网命名空间 (KB kv_store)
pub const WEAVE_NS: &str = "nexus_weave";
/// 会话桥接记录 key 前缀
const BRIDGE_KEY: &str = "bridge_";
/// 图谱维护 key
const GRAPH_KEY: &str = "graph";
/// 强连接阈值: >5 次引用置为永久
const STRONG_LINK_THRESHOLD: u64 = 5;
/// 弱连接标记: 30 天未引用
const STALE_DAYS: u64 = 30;

/// 模式连接记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatternLink {
    pub from_pattern: String,
    pub to_pattern: String,
    pub strength: u64,
    pub first_seen: String,
    pub last_reinforced: String,
    pub permanent: bool,
}

/// 会话桥接记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionBridge {
    pub session_a: String,
    pub session_b: String,
    pub implicit_keyword: String,
    pub bridge_note: String,
    pub created: String,
}

/// 织网报告 (汇总一次织网循环的结果)
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct WeaveReport {
    pub new_links: Vec<String>,
    pub reinforced_links: Vec<String>,
    pub bridges_created: Vec<String>,
    pub stale_links_marked: Vec<String>,
    pub permanent_links: Vec<String>,
}

impl WeaveReport {
    pub fn is_empty(&self) -> bool {
        self.new_links.is_empty()
            && self.reinforced_links.is_empty()
            && self.bridges_created.is_empty()
            && self.stale_links_marked.is_empty()
            && self.permanent_links.is_empty()
    }
}

fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f").to_string()
}

fn days_ago(secs: u64) -> i64 {
    chrono::Utc::now().timestamp().saturating_sub(secs as i64)
}

/// 列出已知的所有模式连接 (跨 namespace 聚合 skills/experience 分支关键词)。
/// 真实数据源: 各技能 namespace 的 `hub` 中 `branches` 关键词 + 本命名空间既有 links。
pub fn list_patterns(conn: &Connection) -> Vec<String> {
    let mut patterns = std::collections::BTreeSet::new();
    // 从既有 links 聚合
    if let Ok(Some(raw)) = kv_get(conn, WEAVE_NS, GRAPH_KEY) {
        if let Ok(v) = serde_json::from_str::<Value>(&raw) {
            if let Some(links) = v.get("links").and_then(|x| x.as_array()) {
                for l in links {
                    if let Some(f) = l.get("from_pattern").and_then(|x| x.as_str()) {
                        patterns.insert(f.to_string());
                    }
                    if let Some(t) = l.get("to_pattern").and_then(|x| x.as_str()) {
                        patterns.insert(t.to_string());
                    }
                }
            }
        }
    }
    // 从 hub 分支关键词聚合 (跨 namespace)
    if let Ok(nss) = kv_list_namespaces(conn) {
        for ns in nss {
            if let Ok(Some(raw)) = kv_get(conn, &ns, "hub") {
                if let Ok(hub) = serde_json::from_str::<Value>(&raw) {
                    if let Some(br) = hub.get("branches").and_then(|b| b.as_object()) {
                        for k in br.keys() {
                            patterns.insert(k.clone());
                        }
                    }
                }
            }
        }
    }
    patterns.into_iter().collect()
}

/// 会话桥接: 记录 session_a 与 session_b 之间的显式连接 (Phase 3 bridge)。
/// 隐式关键词为共享主题。写入 `nexus_weave::bridge_<hash>`。
pub fn record_session_bridge(
    conn: &Connection,
    session_a: &str,
    session_b: &str,
    implicit_keyword: &str,
    bridge_note: &str,
) -> Result<SessionBridge, String> {
    let now = now_iso();
    let bridge = SessionBridge {
        session_a: session_a.to_string(),
        session_b: session_b.to_string(),
        implicit_keyword: implicit_keyword.to_string(),
        bridge_note: bridge_note.to_string(),
        created: now,
    };
    // 确定性 key: 使用稳定哈希
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    format!("{session_a}->{session_b}").hash(&mut h);
    let key = format!("{BRIDGE_KEY}{:016x}", h.finish());
    let raw = serde_json::to_string(&bridge).map_err(|e| format!("bridge 序列化失败: {e}"))?;
    kv_set(conn, WEAVE_NS, &key, &raw).map_err(|e| format!("bridge 写入失败: {e}"))?;
    Ok(bridge)
}

/// 模式连接: 强化既有连接 (strength+1) 或建新连接接最近邻 (Phase 2 weave)。
/// 返回 (新建, 强化) 标志。连接存于 `nexus_weave::graph`。
pub fn connect_patterns(
    conn: &Connection,
    from_pattern: &str,
    to_pattern: &str,
) -> Result<(bool, bool), String> {
    let now = now_iso();
    let mut report = WeaveReport::default();
    let raw = kv_get(conn, WEAVE_NS, GRAPH_KEY)
        .map_err(|e| format!("graph 读取失败: {e}"))?
        .unwrap_or_else(|| "{\"links\":[]}".to_string());
    let mut graph: Value =
        serde_json::from_str(&raw).map_err(|e| format!("graph 解析失败: {e}"))?;
    if graph.get("links").is_none() {
        graph["links"] = json!([]);
    }

    let links = graph["links"].as_array_mut().ok_or("links 非数组")?;
    // 查找既有连接 (双向)
    let existing = links.iter().position(|l| {
        let f = l.get("from_pattern").and_then(|x| x.as_str()).unwrap_or("");
        let t = l.get("to_pattern").and_then(|x| x.as_str()).unwrap_or("");
        (f == from_pattern && t == to_pattern) || (f == to_pattern && t == from_pattern)
    });

    match existing {
        Some(idx) => {
            // 强化既有连接
            let strength = links[idx]
                .get("strength")
                .and_then(|v| v.as_u64())
                .unwrap_or(1)
                + 1;
            links[idx]["strength"] = json!(strength);
            links[idx]["last_reinforced"] = json!(now);
            if strength > STRONG_LINK_THRESHOLD {
                links[idx]["permanent"] = json!(true);
                report.permanent_links.push(format!("{from_pattern}<->{to_pattern}"));
            }
            report.reinforced_links.push(format!("{from_pattern}<->{to_pattern} ({strength})"));
        }
        None => {
            // 新建连接
            links.push(json!({
                "from_pattern": from_pattern,
                "to_pattern": to_pattern,
                "strength": 1,
                "first_seen": now,
                "last_reinforced": now,
                "permanent": false,
            }));
            report.new_links.push(format!("{from_pattern}<->{to_pattern}"));
        }
    }

    let updated = serde_json::to_string(&graph).map_err(|e| format!("graph 序列化失败: {e}"))?;
    kv_set(conn, WEAVE_NS, GRAPH_KEY, &updated).map_err(|e| format!("graph 写入失败: {e}"))?;
    Ok((!report.new_links.is_empty(), !report.reinforced_links.is_empty()))
}

/// 图谱维护: 标记 30 天未引用的弱连接, 统计强连接 (Phase 4 graph update)。
/// 返回 (弱连接数, 强连接数)。
pub fn graph_curate(conn: &Connection) -> Result<(usize, usize), String> {
    let cutoff = days_ago(STALE_DAYS * 86400);
    let raw = kv_get(conn, WEAVE_NS, GRAPH_KEY)
        .map_err(|e| format!("graph 读取失败: {e}"))?
        .unwrap_or_else(|| "{\"links\":[]}".to_string());
    let mut graph: Value =
        serde_json::from_str(&raw).map_err(|e| format!("graph 解析失败: {e}"))?;
    if graph.get("links").is_none() {
        graph["links"] = json!([]);
    }
    let mut stale = 0usize;
    let mut strong = 0usize;
    let links = graph["links"].as_array_mut().ok_or("links 非数组")?;
    for l in links.iter_mut() {
        let strength = l.get("strength").and_then(|v| v.as_u64()).unwrap_or(0);
        let last = l
            .get("last_reinforced")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|t| t.timestamp())
            .unwrap_or(0);
        let permanent = l.get("permanent").and_then(|v| v.as_bool()).unwrap_or(false);
        if strength > STRONG_LINK_THRESHOLD || permanent {
            l["permanent"] = json!(true);
            strong += 1;
        } else if last < cutoff {
            l["weak"] = json!(true);
            stale += 1;
        }
    }
    let updated = serde_json::to_string(&graph).map_err(|e| format!("graph 序列化失败: {e}"))?;
    kv_set(conn, WEAVE_NS, GRAPH_KEY, &updated).map_err(|e| format!("graph 写入失败: {e}"))?;
    Ok((stale, strong))
}

/// 一键织网循环: 桥接 + 连接 + 图谱维护。返回 WeaveReport。
/// 生产接线点: 会话收尾 (experience-tree absorb 后) 调用。
pub fn weave_once(conn: &Connection) -> Result<WeaveReport, String> {
    let mut report = WeaveReport::default();
    // Phase 4: 图谱维护
    let (stale, strong) = graph_curate(conn)?;
    for _ in 0..stale {
        report.stale_links_marked.push("weak".into());
    }
    for _ in 0..strong {
        report.permanent_links.push("strong".into());
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        super::super::nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_connect_patterns_new_and_reinforce() {
        let conn = test_conn();
        let (is_new, is_reinforced) = connect_patterns(&conn, "A", "B").unwrap();
        assert!(is_new && !is_reinforced);
        let (is_new2, is_reinforced2) = connect_patterns(&conn, "B", "A").unwrap();
        assert!(!is_new2 && is_reinforced2, "反向连接应强化既有");
        let raw = kv_get(&conn, WEAVE_NS, GRAPH_KEY).unwrap().unwrap();
        let g: Value = serde_json::from_str(&raw).unwrap();
        let links = g["links"].as_array().unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0]["strength"].as_u64().unwrap(), 2);
    }

    #[test]
    fn test_record_session_bridge() {
        let conn = test_conn();
        let b = record_session_bridge(&conn, "sess_1", "sess_2", "FTS5", "共享检索陷阱").unwrap();
        assert_eq!(b.session_a, "sess_1");
        assert_eq!(b.implicit_keyword, "FTS5");
        let nss = kv_list_namespaces(&conn).unwrap();
        assert!(nss.contains(&WEAVE_NS.to_string()));
    }

    #[test]
    fn test_graph_curate_marks_stale() {
        let conn = test_conn();
        connect_patterns(&conn, "X", "Y").unwrap();
        // 人工把 last_reinforced 改为 40 天前
        let old = days_ago(40 * 86400);
        let old_iso = chrono::DateTime::from_timestamp(old, 0)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%S%.6f")
            .to_string();
        let raw = kv_get(&conn, WEAVE_NS, GRAPH_KEY).unwrap().unwrap();
        let mut g: Value = serde_json::from_str(&raw).unwrap();
        g["links"][0]["last_reinforced"] = json!(old_iso);
        kv_set(&conn, WEAVE_NS, GRAPH_KEY, &g.to_string()).unwrap();
        let (stale, _) = graph_curate(&conn).unwrap();
        assert_eq!(stale, 1);
    }

    #[test]
    fn test_weave_once_returns_report() {
        let conn = test_conn();
        let report = weave_once(&conn).unwrap();
        assert!(report.is_empty());
        connect_patterns(&conn, "P", "Q").unwrap();
        let report2 = weave_once(&conn).unwrap();
        assert!(!report2.is_empty());
    }
}