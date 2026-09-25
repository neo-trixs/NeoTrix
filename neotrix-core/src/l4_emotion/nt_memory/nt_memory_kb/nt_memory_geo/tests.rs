//! nt_memory_geo tests — 原内联 tests 整体搬移，行为零变更。

use super::nt_geo_base::{match_city_in_text, match_country_in_text};
use super::*;
use rusqlite::{params, Connection};

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
fn test_upsert_and_query_bbox() {
    let conn = test_conn();
    upsert_geo(
        &conn,
        &GeoRecord {
            node_id: "shanhai-map:kunlun".into(),
            lat: 1.0,
            lng: 37.0,
            country: "肯尼亚".into(),
            region: "东非".into(),
            city: "".into(),
            tags: "昆仑山,山海经".into(),
            source: "shanhai".into(),
            confidence: 0.75,
        },
    )
    .unwrap();
    // 幂等 upsert
    upsert_geo(
        &conn,
        &GeoRecord {
            node_id: "shanhai-map:kunlun".into(),
            lat: 1.0,
            lng: 37.0,
            country: "肯尼亚".into(),
            region: "东非".into(),
            city: "".into(),
            tags: "昆仑山,山海经".into(),
            source: "shanhai".into(),
            confidence: 0.8,
        },
    )
    .unwrap();

    let hits = query_bbox(&conn, -10.0, 30.0, 10.0, 45.0, 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].node_id, "shanhai-map:kunlun");
    assert_eq!(hits[0].confidence, 0.8);
    assert_eq!(geo_stats(&conn).unwrap(), (1, 1));
}

#[test]
fn test_query_by_place_and_geojson() {
    let conn = test_conn();
    upsert_geo(
        &conn,
        &GeoRecord {
            node_id: "shanhai-map:buzhou".into(),
            lat: 1.0,
            lng: 36.0,
            country: "肯尼亚".into(),
            region: "东非".into(),
            city: "".into(),
            tags: "不周山".into(),
            source: "shanhai".into(),
            confidence: 0.7,
        },
    )
    .unwrap();

    let hits = query_by_place(&conn, "肯尼亚", "", "", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].node_id, "shanhai-map:buzhou");

    let geojson = export_geojson(&conn).unwrap();
    assert!(geojson.contains("FeatureCollection"));
    assert!(geojson.contains("不周山"));
}

#[test]
fn test_country_dict_matches() {
    assert!(match_country_in_text("关于中国哲学的研究").is_some());
    assert!(match_country_in_text("A study of France literature").is_some());
    assert!(match_country_in_text("量子力学导论").is_none());
}

#[test]
fn test_weather_table() {
    let conn = test_conn();
    conn.execute_batch("CREATE TABLE geo_elevation (node_id TEXT PRIMARY KEY, lat REAL, lng REAL, elevation_m REAL, source TEXT, fetched_at INTEGER);")
        .unwrap();
    ensure_weather_table(&conn).unwrap();
    let ts = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO geo_weather (node_id, lat, lng, temp_c, pressure_msl, wind_kmh, precip_mm, elevation_m, fetched_at)
         VALUES ('geo:city:BJ', 39.9, 116.4, 32.8, 1006.2, 9.0, 0.0, 47.0, ?1)",
        params![ts],
    )
    .unwrap();
    let rows = query_weather(&conn, 10).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].node_id, "geo:city:BJ");
    assert_eq!(rows[0].temp_c, Some(32.8));
    assert_eq!(rows[0].pressure_msl, Some(1006.2));
}

#[test]
fn test_city_dict_matches() {
    assert_eq!(
        match_city_in_text("Stanford University research").map(|c| c.0),
        Some("旧金山")
    );
    assert_eq!(
        match_city_in_text("牛津大学 的研究").map(|c| c.0),
        Some("牛津")
    );
    assert_eq!(
        match_city_in_text("A study of Londoner culture").map(|c| c.0),
        Some("伦敦")
    );
    // 词边界: "San" 不应命中 "Santa", "Tokyo" 不应命中 "Tokyopop"
    assert_eq!(match_city_in_text("Santa Monica beach").map(|c| c.0), None);
    assert_eq!(match_city_in_text("量子力学导论").map(|c| c.0), None);
    // 词形派生: "Londoner"/"London-based" 命中伦敦, "Oxfordian" 命中牛津
    assert_eq!(
        match_city_in_text("a London-based firm").map(|c| c.0),
        Some("伦敦")
    );
    assert_eq!(
        match_city_in_text("Oxfordian scholarship").map(|c| c.0),
        Some("牛津")
    );
    // 子地点归入主城市, 坐标与 city 字段一致 (Stanford/硅谷 → 旧金山市中心)
    assert_eq!(
        match_city_in_text("Stanford University research").map(|c| c.0),
        Some("旧金山")
    );
    assert_eq!(
        match_city_in_text("Stanford University research").map(|c| c.2),
        Some(37.7749)
    );
}

#[test]
fn test_geo_tag_cities_and_links() {
    let conn = test_conn();
    conn.execute_batch(
        "CREATE TABLE nodes (
            id TEXT PRIMARY KEY,
            node_type TEXT NOT NULL,
            title TEXT NOT NULL,
            summary TEXT,
            content TEXT,
            url TEXT,
            domain TEXT,
            language TEXT DEFAULT 'en',
            confidence REAL DEFAULT 1.0,
            importance REAL DEFAULT 0.5,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            access_count INTEGER DEFAULT 0,
            metadata TEXT,
            data_tier TEXT NOT NULL DEFAULT 'core',
            temporal TEXT,
            supersedes TEXT,
            source_episode TEXT,
            tier TEXT NOT NULL DEFAULT 'warm'
        );",
    )
    .unwrap();
    let ts = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO nodes (id, node_type, title, summary, created_at, updated_at, importance)
         VALUES ('n1', 'article', 'Stanford NLP 研究综述', 'transformer 语言模型', ?, ?, 0.9)",
        params![ts, ts],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO nodes (id, node_type, title, summary, created_at, updated_at)
         VALUES ('n2', 'concept', '机器学习', '泛化理论', ?, ?)",
        params![ts, ts],
    )
    .unwrap();

    let tagged = geo_tag_cities(&conn, 100).unwrap();
    assert_eq!(tagged, 1); // 只有 n1 命中 "Stanford" → 旧金山

    // 验证挂载坐标 = 旧金山 (非国家首都)
    let (lat, city): (f64, String) = conn
        .query_row(
            "SELECT lat, city FROM geo_index WHERE node_id = 'n1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(city, "旧金山");
    assert!((lat - 37.7749).abs() < 0.001);

    // 反向关联查询
    let links = geo_linked_nodes(&conn, "旧金山", 10).unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].1, "Stanford NLP 研究综述");
}

#[test]
fn test_geo_tag_nodes_and_coverage() {
    let conn = test_conn();
    // 建一个带国家关键词的节点 (需有 nodes 表)
    conn.execute_batch(
        "CREATE TABLE nodes (
            id TEXT PRIMARY KEY,
            node_type TEXT NOT NULL,
            title TEXT NOT NULL,
            summary TEXT,
            content TEXT,
            url TEXT,
            domain TEXT,
            language TEXT DEFAULT 'en',
            confidence REAL DEFAULT 1.0,
            importance REAL DEFAULT 0.5,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            access_count INTEGER DEFAULT 0,
            metadata TEXT,
            data_tier TEXT NOT NULL DEFAULT 'core',
            temporal TEXT,
            supersedes TEXT,
            source_episode TEXT,
            tier TEXT NOT NULL DEFAULT 'warm'
        );",
    )
    .unwrap();
    let ts = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO nodes (id, node_type, title, summary, created_at, updated_at)
         VALUES ('n1', 'article', '中国近代史研究', '关于日本维新', ?, ?)",
        params![ts, ts],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO nodes (id, node_type, title, summary, created_at, updated_at)
         VALUES ('n2', 'concept', '机器学习', '泛化理论', ?, ?)",
        params![ts, ts],
    )
    .unwrap();

    let tagged = geo_tag_nodes(&conn, 100).unwrap();
    assert_eq!(tagged, 1); // 只有 n1 命中 "中国"

    let coverage = geo_coverage_report(&conn, 0).unwrap();
    assert!(coverage.iter().any(|(c, _)| c == "中国"));
}

#[test]
fn test_archive_geo_cold_roundtrip() {
    let conn = test_conn();
    let tmp = std::env::temp_dir().join(format!("ntpack_b1_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    // 两个 source 各 2 条
    for (src, i) in [
        ("ourairports", 0),
        ("ourairports", 1),
        ("gvp-volcanoes", 0),
        ("gvp-volcanoes", 1),
    ] {
        upsert_geo(
            &conn,
            &GeoRecord {
                node_id: format!("{}:{}", src, i),
                lat: 30.0 + i as f64,
                lng: 110.0 + i as f64,
                country: "CN".into(),
                region: "华东".into(),
                city: "".into(),
                tags: "test".into(),
                source: src.into(),
                confidence: 0.9,
            },
        )
        .unwrap();
    }

    // 归档 ourairports → 热表删除, 文件生成
    let path = tmp.join("geo_ourairports.ntpack");
    let (n, bytes) = archive_geo_cold(&conn, "ourairports", path.to_str().unwrap()).unwrap();
    assert_eq!(n, 2);
    assert!(bytes > 0);
    assert_eq!(geo_stats(&conn).unwrap(), (2, 2)); // 只剩火山 2 条

    // 幂等: 重复归档已空 source → Err
    assert!(archive_geo_cold(&conn, "ourairports", path.to_str().unwrap()).is_err());

    // 冷层枚举可见
    let layers = geo_cold_layers(tmp.to_str().unwrap()).unwrap();
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].0, "ourairports");

    // 恢复: 导入回 KB → 幂等 upsert, 总数回到 4
    let restored = import_geo_ntpack_to_kb(&conn, path.to_str().unwrap()).unwrap();
    assert_eq!(restored, 2);
    assert_eq!(geo_stats(&conn).unwrap(), (4, 4));

    // 冷热层 inventory: 恢复后 warm=4, 冷层文件仍可见 (源双在)
    let inv = geo_layer_inventory(&conn, tmp.to_str().unwrap()).unwrap();
    let oa = inv.iter().find(|(s, _, _)| s == "ourairports").unwrap();
    assert_eq!(oa.1, 2); // warm 2 条
    assert!(oa.2.is_some()); // 冷层仍有归档文件
    let gv = inv.iter().find(|(s, _, _)| s == "gvp-volcanoes").unwrap();
    assert_eq!(gv.1, 2);
    assert!(gv.2.is_none()); // 从未归档

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_import_batch_commit_large() {
    // B1 修复回归: 单事务长写锁 → BATCH 分批提交, 大量记录不卡死
    let conn = test_conn();
    let tmp = std::env::temp_dir().join(format!("ntpack_b1b_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("geo_big.ntpack");

    // 用 export 先构造 1200 条 (> 500 BATCH) 的 NT-Pack 文件
    let mut pts = Vec::new();
    for i in 0..1200 {
        pts.push(
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_pack::GeoPoint {
                node_id: format!("bulk:{}", i),
                lat: (i as f64 % 90.0),
                lng: (i as f64 % 180.0),
                country: "CN".into(),
                region: String::new(),
                city: String::new(),
                tags: String::new(),
                source: "bulk".into(),
            },
        );
    }
    let enc =
        crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_pack::PackEncoder::new(5, true);
    let bytes = enc.encode(&pts);
    std::fs::write(&path, &bytes).unwrap();

    let n = import_geo_ntpack_to_kb(&conn, path.to_str().unwrap()).unwrap();
    assert_eq!(n, 1200);
    assert_eq!(geo_stats(&conn).unwrap(), (1200, 1200));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_append_merge_semantics() {
    // A4: 追加模式 — 新覆盖旧, 文件可重读
    use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_pack::GeoPoint;
    let tmp = std::env::temp_dir().join(format!("ntpack_a4_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("geo_append.ntpack").to_str().unwrap().to_string();

    let mk = |id: &str, lat: f64, src: &str| GeoPoint {
        node_id: id.into(),
        lat,
        lng: 100.0,
        country: "CN".into(),
        region: String::new(),
        city: String::new(),
        tags: String::new(),
        source: src.into(),
    };

    // 第一批 3 条
    let (n, b) = append_geo_ntpack(
        &path,
        &[
            mk("a", 1.0, "src1"),
            mk("b", 2.0, "src1"),
            mk("c", 3.0, "src2"),
        ],
    )
    .unwrap();
    assert_eq!(n, 3);
    assert!(b > 0);

    // 追加: a 覆盖 (新 lat), d 新增 → 总数 4, a 的 lat 更新
    let (n2, _) =
        append_geo_ntpack(&path, &[mk("a", 99.0, "src1"), mk("d", 4.0, "src2")]).unwrap();
    assert_eq!(n2, 4);

    let (dec_n, pts) = import_geo_ntpack(&path).unwrap();
    assert_eq!(dec_n, 4);
    let a = pts.iter().find(|p| p.node_id == "a").unwrap();
    assert_eq!(a.lat, 99.0); // 新覆盖旧

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_cold_layers_excludes_full_export() {
    // 全量导出 geo_index.ntpack 不应被当作单个 source 冷层计入
    let tmp = std::env::temp_dir().join(format!("ntpack_cold_excl_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("geo_index.ntpack"), b"full mirror").unwrap();
    std::fs::write(tmp.join("geo_ourairports.ntpack"), b"cold").unwrap();

    let layers = geo_cold_layers(tmp.to_str().unwrap()).unwrap();
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].0, "ourairports");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_transparent_query_cold_fallback() {
    // B1 透明读路径: 归档 source 后, 热查询仍能经冷层兜底见数据
    let conn = test_conn();
    let tmp = std::env::temp_dir().join(format!("ntpack_tr_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    // 两个 source: cold (归档) + hot (留热表)
    upsert_geo(
        &conn,
        &GeoRecord {
            node_id: "cold:1".into(),
            lat: 30.0,
            lng: 110.0,
            country: "CN".into(),
            region: "华东".into(),
            city: "上海".into(),
            tags: String::new(),
            source: "cold-src".into(),
            confidence: 0.9,
        },
    )
    .unwrap();
    upsert_geo(
        &conn,
        &GeoRecord {
            node_id: "hot:1".into(),
            lat: 30.1,
            lng: 110.1,
            country: "CN".into(),
            region: "华东".into(),
            city: "上海".into(),
            tags: String::new(),
            source: "hot-src".into(),
            confidence: 0.8,
        },
    )
    .unwrap();

    // 归档 cold-src → 热表只剩 hot
    let cold_path = tmp.join("geo_cold-src.ntpack");
    let (n, _) = archive_geo_cold(&conn, "cold-src", cold_path.to_str().unwrap()).unwrap();
    assert_eq!(n, 1);

    // 透明 bbox: 热表命中 1 (hot), 冷层兜底 1 (cold) → 2 条
    let (recs, cold_hits) =
        query_bbox_with_cold(&conn, 29.0, 109.0, 31.0, 111.0, 10, tmp.to_str().unwrap())
            .unwrap();
    assert_eq!(recs.len(), 2);
    assert_eq!(cold_hits, 1);
    let cold_rec = recs.iter().find(|r| r.node_id == "cold:1").unwrap();
    assert_eq!(cold_rec.confidence, 0.0); // NT-Pack 无 confidence

    // 透明 by_place: 中国全部 → 2 条 (hot + cold)
    let (recs2, cold2) =
        query_by_place_with_cold(&conn, "CN", "", "", 10, tmp.to_str().unwrap()).unwrap();
    assert_eq!(recs2.len(), 2);
    assert_eq!(cold2, 1);

    // 非透明语义回归: 纯热表只剩 1
    let hot_only = query_bbox(&conn, 29.0, 109.0, 31.0, 111.0, 10).unwrap();
    assert_eq!(hot_only.len(), 1);

    let _ = std::fs::remove_dir_all(&tmp);
}
