//! nt_geo_trajectory — 轨迹存储 (B1)，行为零变更纯搬移。

use rusqlite::Connection;

use super::nt_geo_types::TrajectoryRow;

/// 轨迹表: 记录一次行程的空间-时间片段 (B1 轨迹存储)。
pub fn ensure_trajectory_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS trajectory (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT,
            points_json TEXT NOT NULL,
            bbox_west REAL NOT NULL,
            bbox_south REAL NOT NULL,
            bbox_east REAL NOT NULL,
            bbox_north REAL NOT NULL,
            distance_km REAL,
            created_at INTEGER NOT NULL
        );",
    )
}

/// 写入轨迹记录 (幂等 upsert)。
pub fn insert_trajectory(
    conn: &Connection,
    id: &str,
    name: &str,
    kind: Option<&str>,
    points: &[f64],             // GeoJSON LineString 坐标序列 (交替 lat, lng)
    bbox: (f64, f64, f64, f64), // (west, south, east, north)
    distance_km: f64,
) -> Result<(), rusqlite::Error> {
    ensure_trajectory_table(conn)?;
    let points_json = serde_json::json!(points
        .chunks_exact(2)
        .map(|c| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {"type": "Point", "coordinates": [c[0], c[1]]}
            })
        })
        .collect::<serde_json::Value>());
    conn.execute(
        "INSERT OR REPLACE INTO trajectory (id, name, kind, points_json, bbox_west, bbox_south, bbox_east, bbox_north, distance_km, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            id,
            name,
            kind,
            points_json.to_string(),
            bbox.0,
            bbox.1,
            bbox.2,
            bbox.3,
            distance_km,
            chrono::Utc::now().timestamp(),
        ],
    )?;
    Ok(())
}

/// 查询所有轨迹 (按 created_at 降序)。
pub fn query_trajectories(conn: &Connection) -> Result<Vec<TrajectoryRow>, rusqlite::Error> {
    ensure_trajectory_table(conn)?;
    let sql = "SELECT id, name, kind, points_json, bbox_west, bbox_south, bbox_east, bbox_north, distance_km, created_at FROM trajectory ORDER BY created_at DESC";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(TrajectoryRow {
            id: row.get::<_, String>(0)?,
            name: row.get::<_, String>(1)?,
            kind: row.get::<_, Option<String>>(2)?,
            points_json: row.get::<_, String>(3)?,
            bbox_west: row.get::<_, f64>(4)?,
            bbox_south: row.get::<_, f64>(5)?,
            bbox_east: row.get::<_, f64>(6)?,
            bbox_north: row.get::<_, f64>(7)?,
            distance_km: row.get::<_, f64>(8)?,
            created_at: row.get::<_, i64>(9)?,
        })
    })?;
    rows.collect()
}
