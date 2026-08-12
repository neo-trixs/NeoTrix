//! shanhai 坐标 → geo_index 同步 (R-P42: 强化现有节点，不建平行模块)。
//!
//! 把 nt_shanhai_geo 硬编码的现代坐标 (mountains.rs / mappings.rs) 灌入
//! NT-MEMORY 的 geo_index 表，作为地球知识世界仿真的第一批地理锚点。

use rusqlite::Connection;

use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::{upsert_geo, GeoRecord};
use crate::neotrix::nt_shanhai_geo::{all_mappings, known_peaks};

/// 同步全部 shanhai 坐标到 geo_index。返回写入条数。
pub fn sync_shanhai_to_geo(conn: &Connection) -> rusqlite::Result<usize> {
    let mut count = 0usize;

    // 1) 山峰 (mountains.rs) — 华夏说定位，country=中国
    for peak in known_peaks() {
        if let Some(coord) = peak.modern_location {
            let tags = format!("山海经,山峰,{}", peak.name);
            upsert_geo(
                conn,
                &GeoRecord {
                    node_id: format!("shanhai-peak:{}", peak.id),
                    lat: coord.lat,
                    lng: coord.lng,
                    country: "中国".into(),
                    region: "".into(),
                    city: "".into(),
                    tags,
                    source: "shanhai-peaks".into(),
                    confidence: peak.identification_confidence,
                },
            )?;
            count += 1;
        }
    }

    // 2) 全球映射 (mappings.rs) — 世界圈说定位，country 从现代名推断
    for m in all_mappings() {
        if let Some(coord) = m.modern_location {
            let country = infer_country(&m.modern_name);
            let tags = format!("山海经,{}", m.shanhai_name);
            upsert_geo(
                conn,
                &GeoRecord {
                    node_id: format!("shanhai-map:{}", m.shanhai_name),
                    lat: coord.lat,
                    lng: coord.lng,
                    country,
                    region: "".into(),
                    city: "".into(),
                    tags,
                    source: "shanhai-mappings".into(),
                    confidence: m.confidence,
                },
            )?;
            count += 1;
        }
    }

    Ok(count)
}

/// 从现代地名粗粒度推断国家 (仅用于地图着色，非精确地理编码)。
fn infer_country(modern_name: &str) -> String {
    if modern_name.contains("肯尼亚") || modern_name.contains("埃塞俄比亚") || modern_name.contains("东非") {
        "肯尼亚/埃塞俄比亚".into()
    } else if modern_name.contains("落基") || modern_name.contains("北美") || modern_name.contains("科罗拉多") {
        "美国".into()
    } else if modern_name.contains("中国") || modern_name.contains("华夏") {
        "中国".into()
    } else if modern_name.contains("印度") {
        "印度".into()
    } else if modern_name.contains("埃及") || modern_name.contains("尼罗河") {
        "埃及".into()
    } else {
        "".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE geo_index (
                node_id TEXT PRIMARY KEY,
                lat REAL NOT NULL,
                lng REAL NOT NULL,
                country TEXT DEFAULT '',
                region TEXT DEFAULT '',
                city TEXT DEFAULT '',
                tags TEXT DEFAULT '',
                source TEXT DEFAULT '',
                confidence REAL DEFAULT 0.0,
                updated_at INTEGER NOT NULL
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_sync_shanhai_to_geo() {
        let conn = test_conn();
        let n = sync_shanhai_to_geo(&conn).unwrap();
        // mountains.rs 有坐标的山峰 + mappings.rs 有坐标的映射
        assert!(n >= 10, "expected >=10 geo records, got {}", n);

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM geo_index", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total as usize, n);

        // 昆仑山映射应存在 (肯尼亚/埃塞俄比亚高地)
        let kunlun: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM geo_index WHERE node_id = 'shanhai-map:昆仑山'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kunlun, 1);
    }

    #[test]
    fn test_infer_country() {
        assert_eq!(infer_country("肯尼亚/埃塞俄比亚高地"), "肯尼亚/埃塞俄比亚");
        assert_eq!(infer_country("北美落基山脉"), "美国");
        assert_eq!(infer_country("华夏范围内"), "中国");
        assert_eq!(infer_country("未知地名"), "");
    }
}