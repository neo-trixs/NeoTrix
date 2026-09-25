//! nt_geo_elevation — 高程数据 (Open-Meteo)，行为零变更纯搬移。

use rusqlite::{params, Connection, Result};

// ─────────────────────────────────────────────────────────────────────────
// 海拔数据 (Elevation) — 对照 windy.com 的 3D 地形层。
// 数据源: Open-Meteo 免费海拔 API (https://api.open-meteo.com/v1/elevation,
// 无注册, 单点/批量查询, 速率限制约 0.5 req/s)。
// ─────────────────────────────────────────────────────────────────────────

/// 确保 geo_elevation 表存在。
pub fn ensure_elevation_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS geo_elevation (
            node_id TEXT PRIMARY KEY,
            lat REAL NOT NULL,
            lng REAL NOT NULL,
            elevation_m REAL NOT NULL,
            source TEXT NOT NULL DEFAULT 'open-meteo',
            fetched_at INTEGER NOT NULL
        );",
    )?;
    Ok(())
}

/// 为 geo_index 中无海拔记录的高置信度节点批量补海拔。
///
/// 策略: 优先 shanhai 山峰 + geo-tag 节点 + natural-earth 要素 (数量少, 高价值),
/// 每批查询后写入 geo_elevation 表。返回本次写入条数。
/// `limit` 控制本次最多处理节点数 (按 confidence 排序)。
pub fn fetch_elevations(conn: &Connection, limit: usize) -> Result<usize, String> {
    ensure_elevation_table(conn).map_err(|e| format!("elevation table: {}", e))?;
    // 只处理尚未有海拔记录的节点 (排除 geonames-cities 大量低价值点)
    let mut stmt = conn
        .prepare(
            "SELECT g.node_id, g.lat, g.lng
             FROM geo_index g
             WHERE g.source != 'geonames-cities'
               AND g.node_id NOT IN (SELECT node_id FROM geo_elevation)
             ORDER BY g.confidence DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("elevation prep: {}", e))?;
    let rows = stmt
        .query_map(params![limit as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, f64>(1)?,
                r.get::<_, f64>(2)?,
            ))
        })
        .map_err(|e| format!("elevation query: {}", e))?;

    let mut targets: Vec<(String, f64, f64)> = Vec::new();
    for row in rows {
        targets.push(row.map_err(|e| format!("row: {}", e))?);
    }

    let mut written = 0usize;
    let mut batch_coords: Vec<f64> = Vec::new(); // 扁平 lat,lng 交替
    let mut batch_ids: Vec<(String, f64, f64)> = Vec::new();
    for (i, (id, lat, lng)) in targets.iter().enumerate() {
        batch_ids.push((id.clone(), *lat, *lng));
        batch_coords.push(*lat);
        batch_coords.push(*lng);
        // 每 20 个坐标一批 + 末批 flush
        if batch_ids.len() >= 20 || i + 1 == targets.len() {
            let lats = batch_coords
                .iter()
                .step_by(2)
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let lngs = batch_coords
                .iter()
                .skip(1)
                .step_by(2)
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let url = format!(
                "https://api.open-meteo.com/v1/elevation?latitude={}&longitude={}",
                lats, lngs
            );
            match crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::run_blocking(|| {
                crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::shared_blocking_client().get(&url).send()
            }) {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(body) = resp.text() {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                            if let Some(arr) = v.get("elevation").and_then(|a| a.as_array()) {
                                for (j, (id, lat, lng)) in batch_ids.iter().enumerate() {
                                    if let Some(elev) = arr.get(j).and_then(|n| n.as_f64()) {
                                        let ts = chrono::Utc::now().timestamp();
                                        let _ = conn.execute(
                                            "INSERT INTO geo_elevation (node_id, lat, lng, elevation_m, source, fetched_at)
                                             VALUES (?1, ?2, ?3, ?4, 'open-meteo', ?5)
                                             ON CONFLICT(node_id) DO UPDATE SET
                                                elevation_m = excluded.elevation_m,
                                                fetched_at = excluded.fetched_at",
                                            params![id, lat, lng, elev, ts],
                                        );
                                        written += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(resp) => {
                    // 速率限制/临时失败: 跳过本批, 下一轮重试
                    let _ = resp.status();
                }
                Err(_) => {}
            }
            batch_ids.clear();
            batch_coords.clear();
            std::thread::sleep(std::time::Duration::from_millis(2100));
        }
    }

    Ok(written)
}

/// 查询已缓存的海拔记录 (node_id → 海拔米)。
pub fn query_elevations(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<(String, f64, f64, f64)>, String> {
    ensure_elevation_table(conn).map_err(|e| format!("elevation table: {}", e))?;
    let mut stmt = conn
        .prepare(
            "SELECT node_id, lat, lng, elevation_m FROM geo_elevation ORDER BY elevation_m DESC LIMIT ?1",
        )
        .map_err(|e| format!("elevation query: {}", e))?;
    let rows = stmt
        .query_map(params![limit as i64], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .map_err(|e| format!("elevation query map: {}", e))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("row: {}", e))?);
    }
    Ok(out)
}
