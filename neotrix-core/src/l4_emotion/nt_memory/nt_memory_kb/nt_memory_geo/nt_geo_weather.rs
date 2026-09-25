//! nt_geo_weather — 气象快照 (Open-Meteo forecast)，行为零变更纯搬移。

use rusqlite::{params, Connection, Result};

use super::nt_geo_elevation::ensure_elevation_table;
use super::nt_geo_types::WeatherRecord;

pub fn ensure_weather_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS geo_weather (
            node_id TEXT PRIMARY KEY,
            lat REAL NOT NULL,
            lng REAL NOT NULL,
            temp_c REAL,
            pressure_msl REAL,
            wind_kmh REAL,
            precip_mm REAL,
            elevation_m REAL,
            fetched_at INTEGER NOT NULL
        );",
    )?;
    Ok(())
}

/// 为 geo_index 节点批量摄取实时气象快照 (Open-Meteo forecast API)。
///
/// 策略与 fetch_elevations 一致: 排除 geonames-cities 低价值点，
/// 每批 10 个坐标 (响应体较大)，写入 geo_weather 表。
/// 同时复用响应的 elevation 字段回填 geo_elevation 表。
/// `limit` 控制本次最多处理节点数。
pub fn fetch_weather_snapshot(conn: &Connection, limit: usize) -> Result<usize, String> {
    ensure_weather_table(conn).map_err(|e| format!("weather table: {}", e))?;
    ensure_elevation_table(conn).map_err(|e| format!("elevation table: {}", e))?;
    let mut stmt = conn
        .prepare(
            "SELECT g.node_id, g.lat, g.lng
             FROM geo_index g
             WHERE g.source != 'geonames-cities'
               AND g.node_id NOT IN (SELECT node_id FROM geo_weather)
             ORDER BY g.confidence DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("weather prep: {}", e))?;
    let rows = stmt
        .query_map(params![limit as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, f64>(1)?,
                r.get::<_, f64>(2)?,
            ))
        })
        .map_err(|e| format!("weather query: {}", e))?;

    let mut targets: Vec<(String, f64, f64)> = Vec::new();
    for row in rows {
        targets.push(row.map_err(|e| format!("row: {}", e))?);
    }

    let mut written = 0usize;
    let mut batch_coords: Vec<f64> = Vec::new();
    let mut batch_ids: Vec<(String, f64, f64)> = Vec::new();
    for (i, (id, lat, lng)) in targets.iter().enumerate() {
        batch_ids.push((id.clone(), *lat, *lng));
        batch_coords.push(*lat);
        batch_coords.push(*lng);
        if batch_ids.len() >= 10 || i + 1 == targets.len() {
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
                "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,pressure_msl,wind_speed_10m,precipitation&forecast_days=1",
                lats, lngs
            );
            match crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::run_blocking(|| {
                crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::shared_blocking_client().get(&url).send()
            }) {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(body) = resp.text() {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                            if let Some(arr) = v.get("current").and_then(|a| a.as_array()) {
                                let elev_arr = v.get("elevation").and_then(|a| a.as_array());
                                for (j, (id, lat, lng)) in batch_ids.iter().enumerate() {
                                    let item = match arr.get(j) {
                                        Some(it) => it,
                                        None => continue,
                                    };
                                    let temp = item.get("temperature_2m").and_then(|n| n.as_f64());
                                    let pressure =
                                        item.get("pressure_msl").and_then(|n| n.as_f64());
                                    let wind = item.get("wind_speed_10m").and_then(|n| n.as_f64());
                                    let precip = item.get("precipitation").and_then(|n| n.as_f64());
                                    if temp.is_none() || pressure.is_none() {
                                        continue;
                                    }
                                    let elev =
                                        elev_arr.and_then(|e| e.get(j)).and_then(|n| n.as_f64());
                                    let ts = chrono::Utc::now().timestamp();
                                    let _ = conn.execute(
                                        "INSERT INTO geo_weather (node_id, lat, lng, temp_c, pressure_msl, wind_kmh, precip_mm, elevation_m, fetched_at)
                                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                                         ON CONFLICT(node_id) DO UPDATE SET
                                            temp_c = excluded.temp_c,
                                            pressure_msl = excluded.pressure_msl,
                                            wind_kmh = excluded.wind_kmh,
                                            precip_mm = excluded.precip_mm,
                                            elevation_m = excluded.elevation_m,
                                            fetched_at = excluded.fetched_at",
                                        params![
                                            id,
                                            lat,
                                            lng,
                                            temp.unwrap_or(f64::NAN),
                                            pressure.unwrap_or(f64::NAN),
                                            wind,
                                            precip,
                                            elev,
                                            ts
                                        ],
                                    );
                                    // 复用 elevation 回填海拔表
                                    if let Some(e) = elev {
                                        let _ = conn.execute(
                                            "INSERT INTO geo_elevation (node_id, lat, lng, elevation_m, source, fetched_at)
                                             VALUES (?1, ?2, ?3, ?4, 'open-meteo-weather', ?5)
                                             ON CONFLICT(node_id) DO UPDATE SET
                                                elevation_m = excluded.elevation_m,
                                                fetched_at = excluded.fetched_at",
                                            params![id, lat, lng, e, ts],
                                        );
                                    }
                                    written += 1;
                                }
                            }
                        }
                    }
                }
                Ok(_resp) => {
                    // 速率限制/临时失败: 跳过本批
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

/// 查询已缓存的气象快照 (node_id → 温度/气压/风/降水)。
pub fn query_weather(conn: &Connection, limit: usize) -> Result<Vec<WeatherRecord>, String> {
    ensure_weather_table(conn).map_err(|e| format!("weather table: {}", e))?;
    let mut stmt = conn
        .prepare(
            "SELECT node_id, lat, lng, temp_c, pressure_msl, wind_kmh, precip_mm, elevation_m, fetched_at
             FROM geo_weather ORDER BY pressure_msl DESC LIMIT ?1",
        )
        .map_err(|e| format!("weather query: {}", e))?;
    let rows = stmt
        .query_map(params![limit as i64], |r| {
            Ok(WeatherRecord {
                node_id: r.get(0)?,
                lat: r.get(1)?,
                lng: r.get(2)?,
                temp_c: r.get(3)?,
                pressure_msl: r.get(4)?,
                wind_kmh: r.get(5)?,
                precip_mm: r.get(6)?,
                elevation_m: r.get(7)?,
                fetched_at: r.get(8)?,
            })
        })
        .map_err(|e| format!("weather query map: {}", e))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| format!("row: {}", e))?);
    }
    Ok(out)
}
