//! 山海世界查询与 GeoJSON 导出 — 从 bin/shanhai_query.rs 吸收归档 (R-P42)。
//!
//! 一次性脚本 `neotrix-shanhai-query` 的通用查询能力融合为本库模块的公共函数,
//! bin 脚本退化为薄壳。数据形态: KB 中 id 前缀为 `shanhai-*` 的节点。

use rusqlite::Connection;
use serde_json::Value;

/// 统计山海世界 KB 节点/边 (按类型与关系分布)。
/// 返回 (节点总数, 边总数, 按类型计数, 按关系计数)。
pub fn shanhai_stats(
    conn: &Connection,
) -> rusqlite::Result<(i64, i64, Vec<(String, i64)>, Vec<(String, i64)>)> {
    let nodes: i64 = conn.query_row(
        "SELECT COUNT(*) FROM nodes WHERE id LIKE 'shanhai-%'",
        [],
        |r| r.get(0),
    )?;
    let edges: i64 = conn.query_row(
        "SELECT COUNT(*) FROM edges WHERE id LIKE 'shanhai-edge:%'",
        [],
        |r| r.get(0),
    )?;

    let mut node_types = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT node_type, COUNT(*) FROM nodes WHERE id LIKE 'shanhai-%' \
             GROUP BY node_type ORDER BY COUNT(*) DESC",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            node_types.push(row?);
        }
    }

    let mut edge_types = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT relation_type, COUNT(*) FROM edges WHERE id LIKE 'shanhai-edge:%' \
             GROUP BY relation_type ORDER BY COUNT(*) DESC",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            edge_types.push(row?);
        }
    }

    Ok((nodes, edges, node_types, edge_types))
}

/// 查询山海世界山峰 (id 前缀 `shanhai-peak:`)。
/// 返回 (id, title, importance, location)。
pub fn shanhai_peaks(
    conn: &Connection,
) -> rusqlite::Result<Vec<(String, String, f64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, importance, metadata FROM nodes \
         WHERE id LIKE 'shanhai-peak:%' ORDER BY id",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, f64>(2)?,
            meta_key(r.get::<_, Option<String>>(3)?, "location"),
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// 查询山海世界全球对应映射 (id 前缀 `shanhai-map:`)。
/// 返回结构: (id, title, summary, modern_name, location, confidence, scholars)。
pub fn shanhai_mappings(
    conn: &Connection,
) -> rusqlite::Result<Vec<MappingRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, summary, metadata FROM nodes \
         WHERE id LIKE 'shanhai-map:%' ORDER BY id",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, title, summary, meta_str) = row?;
        let meta: Option<Value> = meta_str.as_deref().and_then(|s| serde_json::from_str(s).ok());
        let modern_name = meta.as_ref()
            .and_then(|v| v.get("modern_name").and_then(|l| l.as_str()))
            .unwrap_or("")
            .to_string();
        let location = meta.as_ref()
            .and_then(|v| v.get("modern_location").and_then(|l| l.as_str()))
            .unwrap_or("")
            .to_string();
        let confidence = meta.as_ref()
            .and_then(|v| v.get("confidence").and_then(|c| c.as_f64()))
            .unwrap_or(0.0);
        let scholars = meta.as_ref()
            .and_then(|v| v.get("attributed_by").and_then(|a| a.as_array()))
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        out.push(MappingRecord {
            id,
            title,
            summary: summary.unwrap_or_default(),
            modern_name,
            location,
            confidence,
            scholars,
        });
    }
    Ok(out)
}

/// 单个映射记录的导出结构。
#[derive(Debug, Clone)]
pub struct MappingRecord {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub modern_name: String,
    pub location: String,
    pub confidence: f64,
    pub scholars: Vec<String>,
}

/// 查询证据节点 (id 前缀 `shanhai-evidence:`)。
/// 返回 (id, title, importance, evidence_type, scholar, key_insight)。
pub fn shanhai_evidence(
    conn: &Connection,
) -> rusqlite::Result<Vec<(String, String, f64, String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, importance, metadata FROM nodes \
         WHERE id LIKE 'shanhai-evidence:%' ORDER BY importance DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, f64>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, title, importance, meta_str) = row?;
        let meta: Option<Value> = meta_str.as_deref().and_then(|s| serde_json::from_str(s).ok());
        let ev_type = meta.as_ref()
            .and_then(|v| v.get("type").and_then(|t| t.as_str()))
            .unwrap_or("unknown")
            .to_string();
        let scholar = meta.as_ref()
            .and_then(|v| v.get("scholar").or_else(|| v.get("archaeologist")).and_then(|s| s.as_str()))
            .unwrap_or("")
            .to_string();
        let key = meta.as_ref()
            .and_then(|v| v.get("key_insight").or_else(|| v.get("conclusion")).and_then(|k| k.as_str()))
            .unwrap_or("")
            .to_string();
        out.push((id, title, importance, ev_type, scholar, key));
    }
    Ok(out)
}

/// 查询学术流派 (id 前缀 `shanhai-school:`)。
/// 返回 (id, title, summary, importance, tags)。
pub fn shanhai_schools(
    conn: &Connection,
) -> rusqlite::Result<Vec<(String, String, String, f64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, summary, importance, metadata FROM nodes \
         WHERE id LIKE 'shanhai-school:%' ORDER BY importance DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, f64>(3)?,
            r.get::<_, Option<String>>(4)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, title, summary, importance, meta_str) = row?;
        let meta: Option<Value> = meta_str.as_deref().and_then(|s| serde_json::from_str(s).ok());
        let tags = meta.as_ref()
            .and_then(|v| v.get("tags").and_then(|t| t.as_array()))
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_default();
        out.push((id, title, summary.unwrap_or_default(), importance, tags));
    }
    Ok(out)
}

/// 导出山海映射为 GeoJSON FeatureCollection 字符串。
/// 仅包含有合法经纬度 (modern_location = "lat,lng") 的映射。
pub fn export_geojson(conn: &Connection) -> rusqlite::Result<String> {
    let mappings = shanhai_mappings(conn)?;
    let mut features = Vec::new();

    for m in &mappings {
        if m.location.is_empty() {
            continue;
        }
        let parts: Vec<&str> = m.location.split(',').collect();
        if parts.len() != 2 {
            continue;
        }
        let lat: f64 = parts[0].trim().parse().unwrap_or(0.0);
        let lng: f64 = parts[1].trim().parse().unwrap_or(0.0);
        if lat == 0.0 && lng == 0.0 {
            continue;
        }

        let props = serde_json::json!({
            "title": m.title,
            "modern_name": m.modern_name,
            "confidence": m.confidence,
            "scholar": m.scholars.join("; "),
            "summary": m.summary,
            "id": m.id,
        });
        features.push(serde_json::json!({
            "type": "Feature",
            "geometry": { "type": "Point", "coordinates": [lng, lat] },
            "properties": props,
        }));
    }

    let geojson = serde_json::json!({
        "type": "FeatureCollection",
        "features": features,
        "metadata": {
            "name": "Shanhai Jing Global Mappings",
            "description": "《山海经》全球地理对应映射 — nt_shanhai_geo::query",
            "total_mappings": features.len(),
        },
    });
    serde_json::to_string_pretty(&geojson).map_err(|e| {
        rusqlite::Error::InvalidColumnName(format!("GeoJSON serialization: {}", e))
    })
}

/// 从节点 metadata JSON 中提取指定字符串 key (用于查询展示)。
fn meta_key(meta: Option<String>, key: &str) -> String {
    match meta.as_deref().and_then(|s| serde_json::from_str::<Value>(s).ok()) {
        Some(v) => v.get(key).and_then(|k| k.as_str()).unwrap_or("").to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_memory_kb::nt_memory_schema;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_shanhai_stats_empty() {
        let conn = test_db();
        let (nodes, edges, nt, et) = shanhai_stats(&conn).unwrap();
        assert_eq!(nodes, 0);
        assert_eq!(edges, 0);
        assert!(nt.is_empty());
        assert!(et.is_empty());
    }

    #[test]
    fn test_export_geojson_empty() {
        let conn = test_db();
        let json = export_geojson(&conn).unwrap();
        assert!(json.contains("FeatureCollection"));
        assert!(json.contains("\"features\": []"));
    }
}
