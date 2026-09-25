//! nt_memory_crawl · 地理摄取: 城市/山峰/国界/矢量/机场/边界 (仅本文件内容, 不动 nt_memory_geo).
use log::warn;
use rusqlite::Connection;

use super::super::nt_memory_store as store;
use super::super::nt_memory_types::*;
use super::nt_crawl_core::http_client;
/// 摄取全球城市坐标 (cities.json 格式: [{country, name, lat, lng}, ...])。
/// 数据源: jsdelivr CDN 镜像的 npm `cities.json` 包 (127k 城市 / 246 国家)。
/// 每个城市写入 KB 节点 (NodeType::Resource) + geo_index 地理索引。
/// 返回 (城市节点数, geo_index 条数)。
///
/// 网络策略: 优先读本地缓存 `~/.neotrix/geo/cities.json` (避免 fake-ip 抖动截断);
/// 无缓存时走 http_client 下载, 失败自动重试 (最多 3 次)。
pub fn ingest_geo_cities(
    conn: &mut Connection,
    url: &str,
    country_filter: Option<&str>,
    limit: usize,
) -> Result<(usize, usize), String> {
    // 1) 本地缓存优先
    let cache_path = std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
        .join(".neotrix/geo/cities.json");
    let mut data: Option<serde_json::Value> = None;
    if cache_path.exists() {
        match std::fs::read_to_string(&cache_path) {
            Ok(body) => match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(v) if v.is_array() => data = Some(v),
                _ => warn!("[geo] cache parse failed, falling back to network"),
            },
            Err(e) => warn!("[geo] cache read failed: {}, falling back to network", e),
        }
    }

    // 2) 网络下载 + 重试 (仅当缓存不可用)
    if data.is_none() {
        let mut last_err = String::new();
        for attempt in 0..3 {
            match super::super::nt_http::run_blocking(|| http_client().get(url).send()) {
                Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>() {
                    Ok(v) if v.is_array() => {
                        data = Some(v);
                        break;
                    }
                    Ok(_) => {
                        last_err = "cities payload is not an array".into();
                        break;
                    }
                    Err(e) => last_err = format!("cities JSON parse error: {}", e),
                },
                Ok(resp) => last_err = format!("HTTP {} for {}", resp.status(), url),
                Err(e) => last_err = format!("geo cities fetch error: {}", e),
            }
            std::thread::sleep(std::time::Duration::from_millis(800 * (attempt as u64 + 1)));
        }
        if data.is_none() {
            return Err(format!(
                "geo cities download failed after 3 attempts: {}",
                last_err
            ));
        }
    }

    let arr = data
        .as_ref()
        .ok_or_else(|| "geo cities payload missing after download attempts".to_string())?
        .as_array()
        .ok_or_else(|| "geo cities payload is not an array".to_string())?;

    // 批量写入: 分批事务 (每 BATCH 条提交一次), 避免单事务过大拖慢 WAL,
    // 也避免每条独立提交 (5 万城市 30 倍加速)。
    // 预加载已存在城市 title 集合 → 内存判重, 避免 12 万次 SQL prepare。
    const BATCH: usize = 5000;
    let mut existing: std::collections::HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT title FROM nodes WHERE node_type='resource' AND url IS NULL")
            .map_err(|e| format!("existing titles prepare: {}", e))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| format!("existing titles query: {}", e))?;
        let mut set = std::collections::HashSet::new();
        for r in rows {
            set.insert(r.map_err(|e| format!("existing title row: {}", e))?);
        }
        set
    };

    let mut node_count = 0usize;
    let mut geo_count = 0usize;
    let mut tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("tx begin: {}", e))?;
    for city in arr.iter() {
        let country = city["country"].as_str().unwrap_or("").to_string();
        if let Some(f) = country_filter {
            if !f.is_empty() && !country.eq_ignore_ascii_case(f) {
                continue;
            }
        }
        if geo_count >= limit {
            break;
        }
        let name = city["name"].as_str().unwrap_or("").to_string();
        let lat = city["lat"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let lng = city["lng"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        if name.is_empty() || (lat == 0.0 && lng == 0.0) {
            continue;
        }

        let title = format!("{} ({})", name, country);
        // 内存判重: 已存在则跳过 (幂等重跑安全)
        if existing.contains(&title) {
            continue;
        }
        let node_id = store::insert_or_get_node(
            &tx,
            &title,
            NodeType::Resource,
            Some(&format!(
                "城市坐标: {},{} (GeoNames cities1000 via cities.json)",
                lat, lng
            )),
            None,
            Some("geonames.org"),
        )
        .map_err(|e| format!("DB error: {}", e))?;
        existing.insert(title);
        node_count += 1;

        super::super::nt_memory_geo::upsert_geo(
            &tx,
            &super::super::nt_memory_geo::GeoRecord {
                node_id,
                lat,
                lng,
                country,
                region: "".into(),
                city: name,
                tags: "城市,地理".into(),
                source: "geonames-cities".into(),
                confidence: 1.0,
            },
        )
        .map_err(|e| format!("geo upsert error: {}", e))?;
        geo_count += 1;

        // 每 BATCH 条提交并重开事务
        if geo_count.is_multiple_of(BATCH) {
            tx.commit().map_err(|e| format!("tx commit: {}", e))?;
            tx = conn
                .unchecked_transaction()
                .map_err(|e| format!("tx begin: {}", e))?;
        }
    }
    tx.commit().map_err(|e| format!("tx commit: {}", e))?;

    Ok((node_count, geo_count))
}

/// 摄取 GeoNames 峰点 (feature class T: mountain/hill/peak/range 等)。
/// 数据源: GeoNames 全量 dump `allCountries.txt` (TSV, 19 列制表符分隔) 或按国家 zip。
/// 支持 `file://` 本地路径与 HTTP URL (zip 需先解压为 txt)。
/// 列索引: 0=geonameid, 1=name, 4=lat, 5=lng, 6=featureClass, 7=featureCode, 8=country, 15=elevation
///
/// 只摄取 T 类峰点; 复用 cities 管线的 BATCH 事务模式 (insert_or_get_node 无内部事务)。
/// node_id = geo:peak:{geonameid} (用 geonameid 保证唯一, 避免同名山峰覆盖)。
pub fn ingest_geo_peaks(
    conn: &mut Connection,
    url_or_path: &str,
    limit: usize,
) -> Result<usize, String> {
    // 支持本地文件路径与 file:// 前缀
    let body = if url_or_path.starts_with("file://") || std::path::Path::new(url_or_path).exists() {
        let path = url_or_path.strip_prefix("file://").unwrap_or(url_or_path);
        std::fs::read_to_string(path).map_err(|e| format!("读取本地文件失败: {} ({})", e, path))?
    } else {
        let resp = super::super::nt_http::run_blocking(|| http_client().get(url_or_path).send())
            .map_err(|e| format!("peaks fetch error: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {} for {}", resp.status(), url_or_path));
        }
        resp.text().map_err(|e| format!("read: {}", e))?
    };

    // 预加载已存在 geonameid → 内存判重 (幂等重跑安全)
    let mut existing: std::collections::HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT node_id FROM geo_index WHERE source='geonames-peaks'")
            .map_err(|e| format!("existing peaks prepare: {}", e))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| format!("existing peaks query: {}", e))?;
        let mut set = std::collections::HashSet::new();
        for r in rows {
            set.insert(r.map_err(|e| format!("existing peak row: {}", e))?);
        }
        set
    };

    const BATCH: usize = 5000;
    let mut count = 0usize;
    let mut tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("tx begin: {}", e))?;
    for line in body.lines() {
        if count >= limit {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 16 {
            continue;
        }
        // feature class T 过滤 (mountain/hill/peak/rock/island 等)
        if cols[6] != "T" {
            continue;
        }
        let name = cols[1].trim();
        let geonameid = cols[0].trim();
        if name.is_empty() || geonameid.is_empty() {
            continue;
        }
        let lat: f64 = cols[4].trim().parse().unwrap_or(0.0);
        let lng: f64 = cols[5].trim().parse().unwrap_or(0.0);
        if lat == 0.0 && lng == 0.0 {
            continue;
        }
        let elevation = cols[15].trim().parse::<i64>().unwrap_or(0);
        let feature_code = cols[7].trim();
        let country = cols[8].trim();

        let node_id = format!("geo:peak:{}", geonameid);
        if existing.contains(&node_id) {
            continue;
        }
        let title = format!("{} ({})", name, country);
        store::insert_or_get_node(
            &tx,
            &title,
            NodeType::Resource,
            Some(&format!(
                "GeoNames 峰点: {}m ({} {})",
                elevation, feature_code, name
            )),
            None,
            Some("geonames.org"),
        )
        .map_err(|e| format!("DB error: {}", e))?;
        existing.insert(node_id.clone());

        let tags = format!("山峰,{},{}", feature_code, name);
        super::super::nt_memory_geo::upsert_geo(
            &tx,
            &super::super::nt_memory_geo::GeoRecord {
                node_id,
                lat,
                lng,
                country: country.to_string(),
                region: String::new(),
                city: name.to_string(),
                tags,
                source: "geonames-peaks".into(),
                confidence: 1.0,
            },
        )
        .map_err(|e| format!("geo upsert error: {}", e))?;
        count += 1;

        if count.is_multiple_of(BATCH) {
            tx.commit().map_err(|e| format!("tx commit: {}", e))?;
            tx = conn
                .unchecked_transaction()
                .map_err(|e| format!("tx begin: {}", e))?;
        }
    }
    tx.commit().map_err(|e| format!("tx commit: {}", e))?;

    Ok(count)
}

/// 摄取国家边界 TopoJSON (world-atlas countries-110m.json)。
/// 边界数据体积大, 不逐点入库; 仅登记国家节点 + 记录数据源 URL 供前端加载。
/// 返回登记的国家数。
pub fn ingest_country_boundaries(conn: &Connection, url: &str) -> Result<usize, String> {
    let resp = super::super::nt_http::run_blocking(|| http_client().get(url).send())
        .map_err(|e| format!("country boundaries fetch error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {}", resp.status(), url));
    }
    let body = resp.text().map_err(|e| format!("read: {}", e))?;
    let topo: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("TopoJSON parse error: {}", e))?;

    // world-atlas countries 对象: geometries[].properties.name
    let countries = topo
        .get("objects")
        .and_then(|o| o.get("countries"))
        .and_then(|c| c.get("geometries"))
        .and_then(|g| g.as_array())
        .ok_or("TopoJSON missing objects.countries.geometries")?;

    let mut count = 0usize;
    for geom in countries {
        let name = geom
            .get("properties")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        store::insert_or_get_node(
            conn,
            &name,
            NodeType::Resource,
            Some("国家边界 (world-atlas countries-110m)"),
            None,
            Some("world-atlas"),
        )
        .map_err(|e| format!("DB error: {}", e))?;
        count += 1;
    }

    Ok(count)
}

/// 摄取 Natural Earth 矢量地理要素 (河流/湖泊/海岸线) 到 geo_index。
///
/// 数据源: nvkelso/natural-earth-vector 的 GeoJSON (public domain, CC0)。
/// - 河流: ne_110m_rivers_lake_centerlines.geojson (LineString)
/// - 湖泊: ne_110m_lakes.geojson (Polygon)
/// - 海岸线: ne_110m_coastline.geojson (LineString)
///
/// 每个要素取几何中心点写入 geo_index (node_id = `geo:<kind>:<name>`),
/// 同时登记 KB 节点 (NodeType::Resource)。返回写入的 geo_index 条数。
pub fn ingest_geo_vectors(conn: &Connection, url: &str, kind: &str) -> Result<usize, String> {
    // 支持本地文件路径与 file:// 前缀, 避免大文件走网络下载
    let body = if url.starts_with("file://") || std::path::Path::new(url).exists() {
        let path = url.strip_prefix("file://").unwrap_or(url);
        std::fs::read_to_string(path).map_err(|e| format!("读取本地文件失败: {} ({})", e, path))?
    } else {
        let resp = super::super::nt_http::run_blocking(|| http_client().get(url).send())
            .map_err(|e| format!("geo vectors fetch error: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {} for {}", resp.status(), url));
        }
        resp.text().map_err(|e| format!("read: {}", e))?
    };
    let fc: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("GeoJSON parse error: {}", e))?;

    let features = fc
        .get("features")
        .and_then(|f| f.as_array())
        .ok_or("GeoJSON missing features array")?;

    let mut count = 0usize;
    for (idx, feat) in features.iter().enumerate() {
        let geom = feat
            .get("geometry")
            .and_then(|g| g.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("");
        let name = feat
            .get("properties")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        // 无 name 的要素 (如海岸线) 用 featurecla 或序号兜底
        let name = if name.is_empty() {
            feat.get("properties")
                .and_then(|p| p.get("featurecla"))
                .and_then(|n| n.as_str())
                .map(|s| format!("{}-{}", s, idx))
                .unwrap_or_else(|| format!("{}-{}", kind, idx))
        } else {
            name
        };
        // 计算几何中心点 (取第一个坐标点, 简化处理)
        let coords = feat.get("geometry").and_then(|g| g.get("coordinates"));
        let (lat, lng) = match geom {
            "LineString" => coords
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|p| p.as_array())
                .map(|p| {
                    (
                        p.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
                        p.first().and_then(|v| v.as_f64()).unwrap_or(0.0),
                    )
                })
                .unwrap_or((0.0, 0.0)),
            "Polygon" => coords
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|ring| ring.as_array())
                .and_then(|ring| ring.first())
                .and_then(|p| p.as_array())
                .map(|p| {
                    (
                        p.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
                        p.first().and_then(|v| v.as_f64()).unwrap_or(0.0),
                    )
                })
                .unwrap_or((0.0, 0.0)),
            _ => (0.0, 0.0),
        };
        if lat == 0.0 && lng == 0.0 {
            continue;
        }

        let node_id = format!("geo:{}:{}", kind, name);
        let tags = match kind {
            "river" => format!("河流,{}", name),
            "lake" => format!("湖泊,{}", name),
            "coastline" => format!("海岸线,{}", name),
            _ => name.clone(),
        };
        let rec = super::super::nt_memory_geo::GeoRecord {
            node_id: node_id.clone(),
            lat,
            lng,
            country: String::new(),
            region: String::new(),
            city: String::new(),
            tags,
            source: format!("natural-earth-{}", kind),
            confidence: 0.8,
        };
        super::super::nt_memory_geo::upsert_geo(conn, &rec)
            .map_err(|e| format!("geo upsert error: {}", e))?;
        store::insert_or_get_node(
            conn,
            &name,
            NodeType::Resource,
            Some(&format!("{} (Natural Earth {})", name, kind)),
            None,
            Some("natural-earth"),
        )
        .map_err(|e| format!("DB error: {}", e))?;
        count += 1;
    }

    Ok(count)
}

/// 摄取全球机场 (OurAirports CSV, Public Domain)。
/// 数据源: airports.csv (davidmegginson/ourairports-data, 每日更新, 40,000+ 机场)。
/// CSV 列索引: 0=id, 1=ident, 2=type, 3=name, 4=latitude_deg, 5=longitude_deg,
///             6=elevation_ft, 7=continent, 8=iso_country, 9=iso_region,
///             10=municipality, 12=icao_code, 13=iata_code
/// 过滤 type: large_airport / medium_airport / small_airport (排除 heliport/seaplane/closed)。
/// node_id = geo:airport:{ident} (ident 如 KLAX, 保证唯一)。
pub fn ingest_geo_airports(
    conn: &mut Connection,
    url_or_path: &str,
    limit: usize,
) -> Result<usize, String> {
    let body = if url_or_path.starts_with("file://") || std::path::Path::new(url_or_path).exists() {
        let path = url_or_path.strip_prefix("file://").unwrap_or(url_or_path);
        std::fs::read_to_string(path).map_err(|e| format!("读取本地文件失败: {} ({})", e, path))?
    } else {
        let resp = super::super::nt_http::run_blocking(|| http_client().get(url_or_path).send())
            .map_err(|e| format!("airports fetch error: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {} for {}", resp.status(), url_or_path));
        }
        resp.text().map_err(|e| format!("read: {}", e))?
    };

    // 预加载已存在 ident → 幂等重跑安全
    let mut existing: std::collections::HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT node_id FROM geo_index WHERE source='ourairports'")
            .map_err(|e| format!("existing airports prepare: {}", e))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| format!("existing airports query: {}", e))?;
        let mut set = std::collections::HashSet::new();
        for r in rows {
            set.insert(r.map_err(|e| format!("existing airport row: {}", e))?);
        }
        set
    };

    // 简单 CSV 解析 (带引号, 逗号字段) — 按逗号分割并剥引号
    fn split_csv(line: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut cur = String::new();
        let mut in_quotes = false;
        for ch in line.chars() {
            match ch {
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    out.push(cur.trim().to_string());
                    cur = String::new();
                }
                _ => cur.push(ch),
            }
        }
        out.push(cur.trim().to_string());
        out
    }

    const BATCH: usize = 5000;
    let mut count = 0usize;
    let mut tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("tx begin: {}", e))?;
    for (idx, line) in body.lines().enumerate() {
        if count >= limit {
            break;
        }
        if idx == 0 {
            continue; // header
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols = split_csv(line);
        if cols.len() < 14 {
            continue;
        }
        let atype = cols[2].as_str();
        if atype != "large_airport" && atype != "medium_airport" && atype != "small_airport" {
            continue;
        }
        let ident = cols[1].trim();
        let name = cols[3].trim();
        if ident.is_empty() || name.is_empty() {
            continue;
        }
        let lat: f64 = cols[4].parse().unwrap_or(0.0);
        let lng: f64 = cols[5].parse().unwrap_or(0.0);
        if lat == 0.0 && lng == 0.0 {
            continue;
        }
        let elev_ft = cols[6].parse::<f64>().unwrap_or(0.0);
        let country = cols[8].trim();
        let region = cols[9].trim();
        let city = cols[10].trim();
        let icao = cols[12].trim();
        let iata = cols[13].trim();

        let node_id = format!("geo:airport:{}", ident);
        if existing.contains(&node_id) {
            continue;
        }
        let title = format!("{} ({})", name, ident);
        let elev_m = if elev_ft != 0.0 {
            (elev_ft * 0.3048) as i64
        } else {
            0
        };
        store::insert_or_get_node(
            &tx,
            &title,
            NodeType::Resource,
            Some(&format!(
                "OurAirports 机场: {}m (ICAO={} IATA={} {})",
                elev_m, icao, iata, atype
            )),
            None,
            Some("ourairports.org"),
        )
        .map_err(|e| format!("DB error: {}", e))?;
        existing.insert(node_id.clone());

        let mut tags = format!("机场,{}", atype);
        if !icao.is_empty() {
            tags.push_str(&format!(",{}", icao));
        }
        if !iata.is_empty() {
            tags.push_str(&format!(",{}", iata));
        }
        super::super::nt_memory_geo::upsert_geo(
            &tx,
            &super::super::nt_memory_geo::GeoRecord {
                node_id,
                lat,
                lng,
                country: country.to_string(),
                region: region.to_string(),
                city: if city.is_empty() {
                    name.to_string()
                } else {
                    city.to_string()
                },
                tags,
                source: "ourairports".into(),
                confidence: 1.0,
            },
        )
        .map_err(|e| format!("geo upsert error: {}", e))?;
        count += 1;

        if count.is_multiple_of(BATCH) {
            tx.commit().map_err(|e| format!("tx commit: {}", e))?;
            tx = conn
                .unchecked_transaction()
                .map_err(|e| format!("tx begin: {}", e))?;
        }
    }
    tx.commit().map_err(|e| format!("tx commit: {}", e))?;

    Ok(count)
}

/// 摄取国家/省级行政边界 (Natural Earth Admin 0/1 GeoJSON, Public Domain)。
/// 数据源: ne_10m_admin_0_countries.geojson (258 国) / ne_10m_admin_1_states_provinces.geojson (4,500+ 省级)。
/// 用每个 feature 的质心 (Polygon/MultiPolygon 首环平均) 写入 geo_index。
/// node_id = geo:boundary:{adm0_a3}/{name} — 保证国家/省唯一。
pub fn ingest_geo_boundaries(
    conn: &mut Connection,
    url_or_path: &str,
    level: &str, // "admin0" | "admin1"
    limit: usize,
) -> Result<usize, String> {
    let body = if url_or_path.starts_with("file://") || std::path::Path::new(url_or_path).exists() {
        let path = url_or_path.strip_prefix("file://").unwrap_or(url_or_path);
        std::fs::read_to_string(path).map_err(|e| format!("读取本地文件失败: {} ({})", e, path))?
    } else {
        let resp = super::super::nt_http::run_blocking(|| http_client().get(url_or_path).send())
            .map_err(|e| format!("boundaries fetch error: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {} for {}", resp.status(), url_or_path));
        }
        resp.text().map_err(|e| format!("read: {}", e))?
    };

    let source = if level == "admin1" {
        "natural-earth-admin1"
    } else {
        "natural-earth-admin0"
    };
    let mut existing: std::collections::HashSet<String> = {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT node_id FROM geo_index WHERE source='{}'",
                source
            ))
            .map_err(|e| format!("existing {} prepare: {}", e, source))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| format!("existing {} query: {}", e, source))?;
        let mut set = std::collections::HashSet::new();
        for r in rows {
            set.insert(r.map_err(|e| format!("existing row: {}", e))?);
        }
        set
    };

    let fc: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("GeoJSON parse: {}", e))?;
    let features = fc
        .get("features")
        .and_then(|f| f.as_array())
        .ok_or_else(|| "GeoJSON 缺少 features".to_string())?;

    const BATCH: usize = 1000;
    let mut count = 0usize;
    let mut tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("tx begin: {}", e))?;

    for feat in features {
        if count >= limit {
            break;
        }
        let props = feat
            .get("properties")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let name = props
            .get(if level == "admin1" { "name" } else { "NAME" })
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let adm0 = props
            .get(if level == "admin1" {
                "adm0_a3"
            } else {
                "ADM0_A3"
            })
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let iso = props
            .get(if level == "admin1" {
                "iso_3166_2"
            } else {
                "ISO_A2"
            })
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let pop: i64 = props
            .get(if level == "admin1" {
                "pop_est"
            } else {
                "POP_EST"
            })
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let _continent = props
            .get("CONTINENT")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 质心: 取 geometry 所有坐标的平均 (粗略但足够 geo_index 点模型)
        let geom = feat
            .get("geometry")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let coords = geom
            .get("coordinates")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let mut lat_sum = 0.0f64;
        let mut lng_sum = 0.0f64;
        let mut n_pts = 0usize;
        fn collect_coords(v: &serde_json::Value, lat: &mut f64, lng: &mut f64, n: &mut usize) {
            if let Some(arr) = v.as_array() {
                if arr.len() >= 2 && arr[0].is_number() && arr[1].is_number() {
                    let x = arr[0].as_f64().unwrap_or(0.0);
                    let y = arr[1].as_f64().unwrap_or(0.0);
                    *lng += x;
                    *lat += y;
                    *n += 1;
                } else {
                    for item in arr {
                        collect_coords(item, lat, lng, n);
                    }
                }
            }
        }
        collect_coords(&coords, &mut lat_sum, &mut lng_sum, &mut n_pts);
        if n_pts == 0 {
            continue;
        }
        let lat = lat_sum / n_pts as f64;
        let lng = lng_sum / n_pts as f64;

        let node_id = format!(
            "geo:boundary:{}/{}",
            if adm0.is_empty() { "?" } else { &adm0 },
            name
        );
        if existing.contains(&node_id) {
            continue;
        }
        let title = format!(
            "{} {}",
            if level == "admin1" {
                "省/州"
            } else {
                "国家"
            },
            name
        );
        store::insert_or_get_node(
            &tx,
            &title,
            NodeType::Resource,
            Some(&format!(
                "Natural Earth {} 行政边界: iso={} pop={}",
                level, iso, pop
            )),
            None,
            Some("natural-earth"),
        )
        .map_err(|e| format!("DB error: {}", e))?;
        existing.insert(node_id.clone());

        let mut tags = format!("边界,{}", level);
        if !iso.is_empty() {
            tags.push_str(&format!(",{}", iso));
        }
        if pop > 0 {
            tags.push_str(&format!(",pop:{}", pop));
        }
        super::super::nt_memory_geo::upsert_geo(
            &tx,
            &super::super::nt_memory_geo::GeoRecord {
                node_id,
                lat,
                lng,
                country: if level == "admin1" {
                    adm0.clone()
                } else {
                    name.clone()
                },
                region: if level == "admin1" {
                    name.clone()
                } else {
                    String::new()
                },
                city: String::new(),
                tags,
                source: source.into(),
                confidence: 1.0,
            },
        )
        .map_err(|e| format!("geo upsert error: {}", e))?;
        count += 1;

        if count.is_multiple_of(BATCH) {
            tx.commit().map_err(|e| format!("tx commit: {}", e))?;
            tx = conn
                .unchecked_transaction()
                .map_err(|e| format!("tx begin: {}", e))?;
        }
    }
    tx.commit().map_err(|e| format!("tx commit: {}", e))?;

    Ok(count)
}
