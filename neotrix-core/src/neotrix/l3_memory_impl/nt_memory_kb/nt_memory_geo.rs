//! 地理索引层 — geo_index 表的读写访问 (NT-MEMORY 域)。
//!
//! 地球知识世界仿真的索引层：把 KB 知识节点按地理标签 (lat/lng/国家/区域/城市)
//! 整合，供地图渲染与地理检索使用。数据来源包括 shanhai 坐标、行政区划摄取、
//! 以及任意带坐标的节点元数据。

use rusqlite::{params, Connection, Result};

/// 单条地理索引记录。
#[derive(Debug, Clone)]
pub struct GeoRecord {
    pub node_id: String,
    pub lat: f64,
    pub lng: f64,
    pub country: String,
    pub region: String,
    pub city: String,
    pub tags: String,
    pub source: String,
    pub confidence: f64,
}

/// 写入或更新一条地理索引记录 (幂等 upsert)。
pub fn upsert_geo(
    conn: &Connection,
    rec: &GeoRecord,
) -> Result<()> {
    conn.execute(
        "INSERT INTO geo_index (node_id, lat, lng, country, region, city, tags, source, confidence, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(node_id) DO UPDATE SET
            lat = excluded.lat,
            lng = excluded.lng,
            country = excluded.country,
            region = excluded.region,
            city = excluded.city,
            tags = excluded.tags,
            source = excluded.source,
            confidence = excluded.confidence,
            updated_at = excluded.updated_at",
        params![
            rec.node_id,
            rec.lat,
            rec.lng,
            rec.country,
            rec.region,
            rec.city,
            rec.tags,
            rec.source,
            rec.confidence,
            chrono::Utc::now().timestamp(),
        ],
    )?;
    Ok(())
}

/// 按地理范围查询 (包围盒)。返回按置信度降序的记录。
pub fn query_bbox(
    conn: &Connection,
    min_lat: f64,
    min_lng: f64,
    max_lat: f64,
    max_lng: f64,
    limit: usize,
) -> Result<Vec<GeoRecord>> {
    let mut stmt = conn.prepare(
        "SELECT node_id, lat, lng, country, region, city, tags, source, confidence
         FROM geo_index
         WHERE lat BETWEEN ?1 AND ?3 AND lng BETWEEN ?2 AND ?4
         ORDER BY confidence DESC
         LIMIT ?5",
    )?;
    let rows = stmt.query_map(
        params![min_lat, min_lng, max_lat, max_lng, limit as i64],
        |r| {
            Ok(GeoRecord {
                node_id: r.get(0)?,
                lat: r.get(1)?,
                lng: r.get(2)?,
                country: r.get(3)?,
                region: r.get(4)?,
                city: r.get(5)?,
                tags: r.get(6)?,
                source: r.get(7)?,
                confidence: r.get(8)?,
            })
        },
    )?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// 按国家/区域/城市过滤查询。
pub fn query_by_place(
    conn: &Connection,
    country: &str,
    region: &str,
    city: &str,
    limit: usize,
) -> Result<Vec<GeoRecord>> {
    let mut stmt = conn.prepare(
        "SELECT node_id, lat, lng, country, region, city, tags, source, confidence
         FROM geo_index
         WHERE (?1 = '' OR country = ?1)
           AND (?2 = '' OR region = ?2)
           AND (?3 = '' OR city = ?3)
         ORDER BY confidence DESC
         LIMIT ?4",
    )?;
    let rows = stmt.query_map(
        params![country, region, city, limit as i64],
        |r| {
            Ok(GeoRecord {
                node_id: r.get(0)?,
                lat: r.get(1)?,
                lng: r.get(2)?,
                country: r.get(3)?,
                region: r.get(4)?,
                city: r.get(5)?,
                tags: r.get(6)?,
                source: r.get(7)?,
                confidence: r.get(8)?,
            })
        },
    )?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// 统计地理索引规模。
pub fn geo_stats(conn: &Connection) -> Result<(i64, i64)> {
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM geo_index", [], |r| r.get(0))?;
    let with_country: i64 = conn.query_row(
        "SELECT COUNT(*) FROM geo_index WHERE country != ''",
        [],
        |r| r.get(0),
    )?;
    Ok((total, with_country))
}

/// 导出全部地理索引为 GeoJSON FeatureCollection (供前端地图渲染)。
pub fn export_geojson(conn: &Connection) -> Result<String> {
    let mut stmt = conn.prepare(
        "SELECT node_id, lat, lng, country, region, city, tags, source, confidence
         FROM geo_index ORDER BY confidence DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(GeoRecord {
            node_id: r.get(0)?,
            lat: r.get(1)?,
            lng: r.get(2)?,
            country: r.get(3)?,
            region: r.get(4)?,
            city: r.get(5)?,
            tags: r.get(6)?,
            source: r.get(7)?,
            confidence: r.get(8)?,
        })
    })?;

    let mut features = Vec::new();
    for row in rows {
        let rec = row?;
        features.push(serde_json::json!({
            "type": "Feature",
            "geometry": { "type": "Point", "coordinates": [rec.lng, rec.lat] },
            "properties": {
                "id": rec.node_id,
                "country": rec.country,
                "region": rec.region,
                "city": rec.city,
                "tags": rec.tags,
                "source": rec.source,
                "confidence": rec.confidence,
            },
        }));
    }

    let geojson = serde_json::json!({
        "type": "FeatureCollection",
        "features": features,
        "metadata": {
            "name": "NeoTrix Geo Index",
            "description": "KB 知识节点地理索引 — nt_memory_geo",
            "total": features.len(),
        },
    });
    serde_json::to_string_pretty(&geojson).map_err(|e| {
        rusqlite::Error::InvalidColumnName(format!("GeoJSON serialization: {}", e))
    })
}

// ─────────────────────────────────────────────────────────────────────────
// 地理标签挂载 (Geo Tagging) — 把 KB 中无坐标的知识节点按国家关键词挂上
// 地理标签, 便于统一管理与区域覆盖度分析。
// ─────────────────────────────────────────────────────────────────────────

/// 国家 → (中文名, 首都坐标)。内置主要国家/地区词典, 供标题/摘要关键词匹配。
/// 坐标取首都经纬度 (作为该国家知识节点的代表位置)。
pub const COUNTRY_CAPITALS: &[(&str, &str, f64, f64)] = &[
    ("中国", "中国", 39.9042, 116.4074),
    ("China", "中国", 39.9042, 116.4074),
    ("美国", "美国", 38.9072, -77.0369),
    ("United States", "美国", 38.9072, -77.0369),
    ("日本", "日本", 35.6762, 139.6503),
    ("Japan", "日本", 35.6762, 139.6503),
    ("英国", "英国", 51.5074, -0.1278),
    ("United Kingdom", "英国", 51.5074, -0.1278),
    ("法国", "法国", 48.8566, 2.3522),
    ("France", "法国", 48.8566, 2.3522),
    ("德国", "德国", 52.52, 13.405),
    ("Germany", "德国", 52.52, 13.405),
    ("俄罗斯", "俄罗斯", 55.7558, 37.6173),
    ("Russia", "俄罗斯", 55.7558, 37.6173),
    ("印度", "印度", 28.6139, 77.209),
    ("India", "印度", 28.6139, 77.209),
    ("巴西", "巴西", -15.8267, -47.9218),
    ("Brazil", "巴西", -15.8267, -47.9218),
    ("加拿大", "加拿大", 45.4215, -75.6972),
    ("Canada", "加拿大", 45.4215, -75.6972),
    ("澳大利亚", "澳大利亚", -35.2809, 149.13),
    ("Australia", "澳大利亚", -35.2809, 149.13),
    ("意大利", "意大利", 41.9028, 12.4964),
    ("Italy", "意大利", 41.9028, 12.4964),
    ("西班牙", "西班牙", 40.4168, -3.7038),
    ("Spain", "西班牙", 40.4168, -3.7038),
    ("韩国", "韩国", 37.5665, 126.978),
    ("South Korea", "韩国", 37.5665, 126.978),
    ("朝鲜", "朝鲜", 39.0392, 125.7625),
    ("North Korea", "朝鲜", 39.0392, 125.7625),
    ("墨西哥", "墨西哥", 19.4326, -99.1332),
    ("Mexico", "墨西哥", 19.4326, -99.1332),
    ("埃及", "埃及", 30.0444, 31.2357),
    ("Egypt", "埃及", 30.0444, 31.2357),
    ("土耳其", "土耳其", 39.9334, 32.8597),
    ("Turkey", "土耳其", 39.9334, 32.8597),
    ("希腊", "希腊", 37.9838, 23.7275),
    ("Greece", "希腊", 37.9838, 23.7275),
    ("荷兰", "荷兰", 52.3676, 4.9041),
    ("Netherlands", "荷兰", 52.3676, 4.9041),
    ("瑞士", "瑞士", 46.948, 7.4474),
    ("Switzerland", "瑞士", 46.948, 7.4474),
    ("瑞典", "瑞典", 59.3293, 18.0686),
    ("Sweden", "瑞典", 59.3293, 18.0686),
    ("挪威", "挪威", 59.9139, 10.7522),
    ("Norway", "挪威", 59.9139, 10.7522),
    ("芬兰", "芬兰", 60.1699, 24.9384),
    ("Finland", "芬兰", 60.1699, 24.9384),
    ("波兰", "波兰", 52.2297, 21.0122),
    ("Poland", "波兰", 52.2297, 21.0122),
    ("越南", "越南", 21.0278, 105.8342),
    ("Vietnam", "越南", 21.0278, 105.8342),
    ("泰国", "泰国", 13.7563, 100.5018),
    ("Thailand", "泰国", 13.7563, 100.5018),
    ("新加坡", "新加坡", 1.3521, 103.8198),
    ("Singapore", "新加坡", 1.3521, 103.8198),
    ("马来西亚", "马来西亚", 3.139, 101.6869),
    ("Malaysia", "马来西亚", 3.139, 101.6869),
    ("印度尼西亚", "印度尼西亚", -6.2088, 106.8456),
    ("Indonesia", "印度尼西亚", -6.2088, 106.8456),
    ("菲律宾", "菲律宾", 14.5995, 120.9842),
    ("Philippines", "菲律宾", 14.5995, 120.9842),
    ("伊朗", "伊朗", 35.6892, 51.389),
    ("Iran", "伊朗", 35.6892, 51.389),
    ("以色列", "以色列", 31.7683, 35.2137),
    ("Israel", "以色列", 31.7683, 35.2137),
    ("沙特阿拉伯", "沙特阿拉伯", 24.7136, 46.6753),
    ("Saudi Arabia", "沙特阿拉伯", 24.7136, 46.6753),
    ("南非", "南非", -25.7479, 28.2293),
    ("South Africa", "南非", -25.7479, 28.2293),
    ("阿根廷", "阿根廷", -34.6037, -58.3816),
    ("Argentina", "阿根廷", -34.6037, -58.3816),
    ("乌克兰", "乌克兰", 50.4501, 30.5234),
    ("Ukraine", "乌克兰", 50.4501, 30.5234),
    ("比利时", "比利时", 50.8503, 4.3517),
    ("Belgium", "比利时", 50.8503, 4.3517),
    ("奥地利", "奥地利", 48.2082, 16.3738),
    ("Austria", "奥地利", 48.2082, 16.3738),
    ("葡萄牙", "葡萄牙", 38.7223, -9.1393),
    ("Portugal", "葡萄牙", 38.7223, -9.1393),
    ("爱尔兰", "爱尔兰", 53.3498, -6.2603),
    ("Ireland", "爱尔兰", 53.3498, -6.2603),
    ("新西兰", "新西兰", -41.2865, 174.7762),
    ("New Zealand", "新西兰", -41.2865, 174.7762),
    ("巴基斯坦", "巴基斯坦", 33.6844, 73.0479),
    ("Pakistan", "巴基斯坦", 33.6844, 73.0479),
    ("孟加拉", "孟加拉", 23.8103, 90.4125),
    ("Bangladesh", "孟加拉", 23.8103, 90.4125),
    ("尼日利亚", "尼日利亚", 9.0579, 7.4951),
    ("Nigeria", "尼日利亚", 9.0579, 7.4951),
    ("肯尼亚", "肯尼亚", -1.2921, 36.8219),
    ("Kenya", "肯尼亚", -1.2921, 36.8219),
];

/// 世界主要城市词典 (中英双语) — 用于知识节点的城市级地理标签。
///
/// 元组: (关键词, 城市名, 国家名, lat, lng)。覆盖知识库标题/摘要中
/// 高频出现的城市 (论文机构、仓库名、新闻地域等)。城市级匹配比国家
/// 首都坐标更精确, 大幅提升 geo-tag 覆盖率。
pub const CITY_LATLNG: &[(&str, &str, &str, f64, f64)] = &[
    // 中国
    ("北京", "北京", "中国", 39.9042, 116.4074),
    ("Beijing", "北京", "中国", 39.9042, 116.4074),
    ("上海", "上海", "中国", 31.2304, 121.4737),
    ("Shanghai", "上海", "中国", 31.2304, 121.4737),
    ("广州", "广州", "中国", 23.1291, 113.2644),
    ("Guangzhou", "广州", "中国", 23.1291, 113.2644),
    ("深圳", "深圳", "中国", 22.5431, 114.0579),
    ("Shenzhen", "深圳", "中国", 22.5431, 114.0579),
    ("成都", "成都", "中国", 30.5728, 104.0668),
    ("Chengdu", "成都", "中国", 30.5728, 104.0668),
    ("杭州", "杭州", "中国", 30.2741, 120.1551),
    ("Hangzhou", "杭州", "中国", 30.2741, 120.1551),
    ("武汉", "武汉", "中国", 30.5928, 114.3055),
    ("Wuhan", "武汉", "中国", 30.5928, 114.3055),
    ("西安", "西安", "中国", 34.3416, 108.9398),
    ("Xi'an", "西安", "中国", 34.3416, 108.9398),
    ("南京", "南京", "中国", 32.0603, 118.7969),
    ("Nanjing", "南京", "中国", 32.0603, 118.7969),
    ("天津", "天津", "中国", 39.3434, 117.3616),
    ("Tianjin", "天津", "中国", 39.3434, 117.3616),
    ("重庆", "重庆", "中国", 29.563, 106.5516),
    ("Chongqing", "重庆", "中国", 29.563, 106.5516),
    ("苏州", "苏州", "中国", 31.2989, 120.5853),
    ("Suzhou", "苏州", "中国", 31.2989, 120.5853),
    ("香港", "香港", "中国", 22.3193, 114.1694),
    ("Hong Kong", "香港", "中国", 22.3193, 114.1694),
    ("澳门", "澳门", "中国", 22.1987, 113.5439),
    ("Macau", "澳门", "中国", 22.1987, 113.5439),
    ("台北", "台湾", "中国", 25.033, 121.5654),
    ("Taipei", "台湾", "中国", 25.033, 121.5654),
    // 美国
    ("纽约", "纽约", "美国", 40.7128, -74.006),
    ("New York", "纽约", "美国", 40.7128, -74.006),
    ("洛杉矶", "洛杉矶", "美国", 34.0522, -118.2437),
    ("Los Angeles", "洛杉矶", "美国", 34.0522, -118.2437),
    ("芝加哥", "芝加哥", "美国", 41.8781, -87.6298),
    ("Chicago", "芝加哥", "美国", 41.8781, -87.6298),
    ("休斯顿", "休斯顿", "美国", 29.7604, -95.3698),
    ("Houston", "休斯顿", "美国", 29.7604, -95.3698),
    ("旧金山", "旧金山", "美国", 37.7749, -122.4194),
    ("San Francisco", "旧金山", "美国", 37.7749, -122.4194),
    // 湾区子地点统一归入旧金山市中心坐标, 保持 city 字段与坐标一致
    ("Stanford", "旧金山", "美国", 37.7749, -122.4194),
    ("Palo Alto", "旧金山", "美国", 37.7749, -122.4194),
    ("硅谷", "旧金山", "美国", 37.7749, -122.4194),
    ("Silicon Valley", "旧金山", "美国", 37.7749, -122.4194),
    ("西雅图", "西雅图", "美国", 47.6062, -122.3321),
    ("Seattle", "西雅图", "美国", 47.6062, -122.3321),
    ("波士顿", "波士顿", "美国", 42.3601, -71.0589),
    ("Boston", "波士顿", "美国", 42.3601, -71.0589),
    ("华盛顿", "华盛顿", "美国", 38.9072, -77.0369),
    ("Washington", "华盛顿", "美国", 38.9072, -77.0369),
    ("达拉斯", "达拉斯", "美国", 32.7767, -96.797),
    ("Dallas", "达拉斯", "美国", 32.7767, -96.797),
    ("迈阿密", "迈阿密", "美国", 25.7617, -80.1918),
    ("Miami", "迈阿密", "美国", 25.7617, -80.1918),
    ("亚特兰大", "亚特兰大", "美国", 33.749, -84.388),
    ("Atlanta", "亚特兰大", "美国", 33.749, -84.388),
    // 加拿大
    ("多伦多", "多伦多", "加拿大", 43.6532, -79.3832),
    ("Toronto", "多伦多", "加拿大", 43.6532, -79.3832),
    ("温哥华", "温哥华", "加拿大", 49.2827, -123.1207),
    ("Vancouver", "温哥华", "加拿大", 49.2827, -123.1207),
    ("蒙特利尔", "蒙特利尔", "加拿大", 45.5017, -73.5673),
    ("Montreal", "蒙特利尔", "加拿大", 45.5017, -73.5673),
    // 欧洲
    ("伦敦", "伦敦", "英国", 51.5074, -0.1278),
    ("London", "伦敦", "英国", 51.5074, -0.1278),
    ("剑桥", "剑桥", "英国", 52.2053, 0.1218),
    ("Cambridge", "剑桥", "英国", 52.2053, 0.1218),
    ("牛津", "牛津", "英国", 51.752, -1.2577),
    ("Oxford", "牛津", "英国", 51.752, -1.2577),
    ("巴黎", "巴黎", "法国", 48.8566, 2.3522),
    ("Paris", "巴黎", "法国", 48.8566, 2.3522),
    ("柏林", "柏林", "德国", 52.52, 13.405),
    ("Berlin", "柏林", "德国", 52.52, 13.405),
    ("慕尼黑", "慕尼黑", "德国", 48.1351, 11.582),
    ("Munich", "慕尼黑", "德国", 48.1351, 11.582),
    ("法兰克福", "法兰克福", "德国", 50.1109, 8.6821),
    ("Frankfurt", "法兰克福", "德国", 50.1109, 8.6821),
    ("阿姆斯特丹", "阿姆斯特丹", "荷兰", 52.3676, 4.9041),
    ("Amsterdam", "阿姆斯特丹", "荷兰", 52.3676, 4.9041),
    ("布鲁塞尔", "布鲁塞尔", "比利时", 50.8503, 4.3517),
    ("Brussels", "布鲁塞尔", "比利时", 50.8503, 4.3517),
    ("苏黎世", "苏黎世", "瑞士", 47.3769, 8.5417),
    ("Zurich", "苏黎世", "瑞士", 47.3769, 8.5417),
    ("日内瓦", "日内瓦", "瑞士", 46.2044, 6.1432),
    ("Geneva", "日内瓦", "瑞士", 46.2044, 6.1432),
    ("维也纳", "维也纳", "奥地利", 48.2082, 16.3738),
    ("Vienna", "维也纳", "奥地利", 48.2082, 16.3738),
    ("罗马", "罗马", "意大利", 41.9028, 12.4964),
    ("Rome", "罗马", "意大利", 41.9028, 12.4964),
    ("米兰", "米兰", "意大利", 45.4642, 9.19),
    ("Milan", "米兰", "意大利", 45.4642, 9.19),
    ("马德里", "马德里", "西班牙", 40.4168, -3.7038),
    ("Madrid", "马德里", "西班牙", 40.4168, -3.7038),
    ("巴塞罗那", "巴塞罗那", "西班牙", 41.3874, 2.1686),
    ("Barcelona", "巴塞罗那", "西班牙", 41.3874, 2.1686),
    ("里斯本", "里斯本", "葡萄牙", 38.7223, -9.1393),
    ("Lisbon", "里斯本", "葡萄牙", 38.7223, -9.1393),
    ("斯德哥尔摩", "斯德哥尔摩", "瑞典", 59.3293, 18.0686),
    ("Stockholm", "斯德哥尔摩", "瑞典", 59.3293, 18.0686),
    ("哥本哈根", "哥本哈根", "丹麦", 55.6761, 12.5683),
    ("Copenhagen", "哥本哈根", "丹麦", 55.6761, 12.5683),
    ("赫尔辛基", "赫尔辛基", "芬兰", 60.1699, 24.9384),
    ("Helsinki", "赫尔辛基", "芬兰", 60.1699, 24.9384),
    ("奥斯陆", "奥斯陆", "挪威", 59.9139, 10.7522),
    ("Oslo", "奥斯陆", "挪威", 59.9139, 10.7522),
    ("都柏林", "都柏林", "爱尔兰", 53.3498, -6.2603),
    ("Dublin", "都柏林", "爱尔兰", 53.3498, -6.2603),
    ("莫斯科", "莫斯科", "俄罗斯", 55.7558, 37.6173),
    ("Moscow", "莫斯科", "俄罗斯", 55.7558, 37.6173),
    ("圣彼得堡", "圣彼得堡", "俄罗斯", 59.9311, 30.3609),
    ("St Petersburg", "圣彼得堡", "俄罗斯", 59.9311, 30.3609),
    ("基辅", "基辅", "乌克兰", 50.4501, 30.5234),
    ("Kyiv", "基辅", "乌克兰", 50.4501, 30.5234),
    ("华沙", "华沙", "波兰", 52.2297, 21.0122),
    ("Warsaw", "华沙", "波兰", 52.2297, 21.0122),
    ("布拉格", "布拉格", "捷克", 50.0755, 14.4378),
    ("Prague", "布拉格", "捷克", 50.0755, 14.4378),
    ("布达佩斯", "布达佩斯", "匈牙利", 47.4979, 19.0402),
    ("Budapest", "布达佩斯", "匈牙利", 47.4979, 19.0402),
    ("雅典", "雅典", "希腊", 37.9838, 23.7275),
    ("Athens", "雅典", "希腊", 37.9838, 23.7275),
    ("伊斯坦布尔", "伊斯坦布尔", "土耳其", 41.0082, 28.9784),
    ("Istanbul", "伊斯坦布尔", "土耳其", 41.0082, 28.9784),
    // 亚洲
    ("东京", "东京", "日本", 35.6762, 139.6503),
    ("Tokyo", "东京", "日本", 35.6762, 139.6503),
    ("大阪", "大阪", "日本", 34.6937, 135.5023),
    ("Osaka", "大阪", "日本", 34.6937, 135.5023),
    ("京都", "京都", "日本", 35.0116, 135.7681),
    ("Kyoto", "京都", "日本", 35.0116, 135.7681),
    ("首尔", "首尔", "韩国", 37.5665, 126.978),
    ("Seoul", "首尔", "韩国", 37.5665, 126.978),
    ("曼谷", "曼谷", "泰国", 13.7563, 100.5018),
    ("Bangkok", "曼谷", "泰国", 13.7563, 100.5018),
    ("雅加达", "雅加达", "印度尼西亚", -6.2088, 106.8456),
    ("Jakarta", "雅加达", "印度尼西亚", -6.2088, 106.8456),
    ("马尼拉", "马尼拉", "菲律宾", 14.5995, 120.9842),
    ("Manila", "马尼拉", "菲律宾", 14.5995, 120.9842),
    ("新德里", "新德里", "印度", 28.6139, 77.209),
    ("New Delhi", "新德里", "印度", 28.6139, 77.209),
    ("孟买", "孟买", "印度", 19.076, 72.8777),
    ("Mumbai", "孟买", "印度", 19.076, 72.8777),
    ("班加罗尔", "班加罗尔", "印度", 12.9716, 77.5946),
    ("Bangalore", "班加罗尔", "印度", 12.9716, 77.5946),
    ("迪拜", "迪拜", "阿联酋", 25.2048, 55.2708),
    ("Dubai", "迪拜", "阿联酋", 25.2048, 55.2708),
    ("特拉维夫", "特拉维夫", "以色列", 32.0853, 34.7818),
    ("Tel Aviv", "特拉维夫", "以色列", 32.0853, 34.7818),
    ("吉隆坡", "吉隆坡", "马来西亚", 3.139, 101.6869),
    ("Kuala Lumpur", "吉隆坡", "马来西亚", 3.139, 101.6869),
    ("河内", "河内", "越南", 21.0278, 105.8342),
    ("Hanoi", "河内", "越南", 21.0278, 105.8342),
    ("新加坡", "新加坡", "新加坡", 1.3521, 103.8198),
    ("Singapore", "新加坡", "新加坡", 1.3521, 103.8198),
    // 大洋洲
    ("悉尼", "悉尼", "澳大利亚", -33.8688, 151.2093),
    ("Sydney", "悉尼", "澳大利亚", -33.8688, 151.2093),
    ("墨尔本", "墨尔本", "澳大利亚", -37.8136, 144.9631),
    ("Melbourne", "墨尔本", "澳大利亚", -37.8136, 144.9631),
    ("奥克兰", "奥克兰", "新西兰", -36.8509, 174.7645),
    ("Auckland", "奥克兰", "新西兰", -36.8509, 174.7645),
    // 其他
    ("墨西哥城", "墨西哥城", "墨西哥", 19.4326, -99.1332),
    ("Mexico City", "墨西哥城", "墨西哥", 19.4326, -99.1332),
    ("圣保罗", "圣保罗", "巴西", -23.5505, -46.6333),
    ("Sao Paulo", "圣保罗", "巴西", -23.5505, -46.6333),
    ("里约热内卢", "里约热内卢", "巴西", -22.9068, -43.1729),
    ("Rio de Janeiro", "里约热内卢", "巴西", -22.9068, -43.1729),
    ("布宜诺斯艾利斯", "布宜诺斯艾利斯", "阿根廷", -34.6037, -58.3816),
    ("Buenos Aires", "布宜诺斯艾利斯", "阿根廷", -34.6037, -58.3816),
    ("开罗", "开罗", "埃及", 30.0444, 31.2357),
    ("Cairo", "开罗", "埃及", 30.0444, 31.2357),
    ("拉各斯", "拉各斯", "尼日利亚", 6.5244, 3.3792),
    ("Lagos", "拉各斯", "尼日利亚", 6.5244, 3.3792),
    ("内罗毕", "内罗毕", "肯尼亚", -1.2921, 36.8219),
    ("Nairobi", "内罗毕", "肯尼亚", -1.2921, 36.8219),
    ("约翰内斯堡", "约翰内斯堡", "南非", -26.2041, 28.0473),
    ("Johannesburg", "约翰内斯堡", "南非", -26.2041, 28.0473),
    ("德黑兰", "德黑兰", "伊朗", 35.6892, 51.389),
    ("Tehran", "德黑兰", "伊朗", 35.6892, 51.389),
    ("利雅得", "利雅得", "沙特阿拉伯", 24.7136, 46.6753),
    ("Riyadh", "利雅得", "沙特阿拉伯", 24.7136, 46.6753),
];

/// 在文本中匹配国家关键词, 返回 (国家中文名, 首都 lat, lng, 是否命中)。
/// 命中优先级: 中文全名 > 英文全名, 避免 "China" 与 "Chinese" 等误判。
fn match_country_in_text(text: &str) -> Option<(&'static str, f64, f64)> {
    for (kw, name, lat, lng) in COUNTRY_CAPITALS {
        if text.contains(kw) {
            return Some((name, *lat, *lng));
        }
    }
    None
}

/// 给 KB 中无地理标签的非 resource 节点挂载地理标签。
///
/// 策略: 对每个无 geo 标签的节点, 在 title/summary 中匹配国家关键词,
/// 命中则写一条 geo_index 记录 (node_id = 节点 id, source = "geo-tag:keyword")。
/// 返回本次挂载的数量。
pub fn geo_tag_nodes(conn: &Connection, limit: usize) -> rusqlite::Result<usize> {
    // 找出所有无地理标签的节点 (不在 geo_index 中), 且非 resource 类型 (城市点已有 geo)。
    let mut stmt = conn.prepare(
        "SELECT n.id, n.title, COALESCE(n.summary, '')
         FROM nodes n
         WHERE n.node_type != 'resource'
           AND n.id NOT IN (SELECT node_id FROM geo_index)
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
    })?;

    let mut tagged = 0usize;
    let mut tagged_nodes: Vec<(String, &'static str, f64, f64)> = Vec::new();
    for row in rows {
        let (id, title, summary) = row?;
        let haystack = format!("{} {}", title, summary);
        if let Some((name, lat, lng)) = match_country_in_text(&haystack) {
            tagged_nodes.push((id, name, lat, lng));
        }
    }

    if !tagged_nodes.is_empty() {
        let tx = conn.unchecked_transaction()?;
        for (id, name, lat, lng) in &tagged_nodes {
            tx.execute(
                "INSERT INTO geo_index (node_id, lat, lng, country, region, city, tags, source, confidence, updated_at)
                 VALUES (?1, ?2, ?3, ?4, '', '', ?5, 'geo-tag:keyword', 0.7, ?6)
                 ON CONFLICT(node_id) DO UPDATE SET
                    lat = excluded.lat, lng = excluded.lng,
                    country = excluded.country, tags = excluded.tags,
                    source = excluded.source, confidence = excluded.confidence,
                    updated_at = excluded.updated_at",
                params![id, lat, lng, name, format!("国家,{}", name), chrono::Utc::now().timestamp()],
            )?;
        }
        tx.commit()?;
        tagged = tagged_nodes.len();
    }

    Ok(tagged)
}

/// 在文本中匹配世界城市关键词 (CITY_LATLNG 词典), 返回 (城市名, 国家名, lat, lng)。
/// 中文关键词用 contains; 英文关键词用词边界正则 (防 "San" 误命中 "Santa",
/// "Tokyo" 误命中 "Tokyopop"), 并允许词形派生后缀 (London → Londoner/Londoners/
/// London-based/Londonian), 使 "A study of Londoner culture" 正确命中伦敦。
fn match_city_in_text(text: &str) -> Option<(&'static str, &'static str, f64, f64)> {
    for (kw, city, country, lat, lng) in CITY_LATLNG {
        let hits = if kw.chars().any(|c| c.is_ascii_alphabetic()) {
            // 英文/拼音: 词边界 + 常见词形派生后缀 (含空格变体, 如 "London-based")
            let escaped = regex::escape(kw);
            let pattern = format!(
                r"(?i)\b{}(?:s|es|er|ers|ese|ian|ians|ite|ites|based|wider|less)?\b",
                escaped
            );
            let re = match regex::Regex::new(&pattern) {
                Ok(re) => re,
                Err(_) => continue,
            };
            re.is_match(text)
        } else {
            text.contains(kw)
        };
        if hits {
            return Some((city, country, *lat, *lng));
        }
    }
    None
}

/// 给 KB 中无地理标签的节点挂载城市级地理标签 (CITY_LATLNG 词典)。
///
/// 与 geo_tag_nodes 的国家级匹配互补: 城市级更精确 (精确到城市坐标而非首都),
/// 能覆盖标题含 "Stanford"/"London" 等城市名的论文/仓库节点。
/// 返回本次挂载数量。
pub fn geo_tag_cities(conn: &Connection, limit: usize) -> rusqlite::Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT n.id, n.title, COALESCE(n.summary, '')
         FROM nodes n
         WHERE n.node_type != 'resource'
           AND n.id NOT IN (SELECT node_id FROM geo_index)
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
    })?;

    let mut tagged: Vec<(String, &'static str, &'static str, f64, f64)> = Vec::new();
    for row in rows {
        let (id, title, summary) = row?;
        let haystack = format!("{} {}", title, summary);
        if let Some((city, country, lat, lng)) = match_city_in_text(&haystack) {
            tagged.push((id, city, country, lat, lng));
        }
    }

    if !tagged.is_empty() {
        let tx = conn.unchecked_transaction()?;
        for (id, city, country, lat, lng) in &tagged {
            tx.execute(
                "INSERT INTO geo_index (node_id, lat, lng, country, region, city, tags, source, confidence, updated_at)
                 VALUES (?1, ?2, ?3, ?4, '', ?5, ?6, 'geo-tag:city', 0.6, ?7)
                 ON CONFLICT(node_id) DO UPDATE SET
                    lat = excluded.lat, lng = excluded.lng,
                    country = excluded.country, city = excluded.city,
                    tags = excluded.tags, source = excluded.source,
                    confidence = excluded.confidence, updated_at = excluded.updated_at",
                params![id, lat, lng, country, city, format!("城市,{},{}", city, country), chrono::Utc::now().timestamp()],
            )?;
        }
        tx.commit()?;
    }

    Ok(tagged.len())
}

/// 反向关联查询: 给定国家或城市名, 列出关联的知识节点 (geo_index 的
/// geo-tag 记录 node_id 即 nodes.id, 打通地理坐标 ↔ 知识库节点)。
///
/// `place` 匹配 country 或 city 字段; 返回节点 id/标题/类型 + 坐标。
pub fn geo_linked_nodes(
    conn: &Connection,
    place: &str,
    limit: usize,
) -> rusqlite::Result<Vec<(String, String, String, f64, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT g.node_id, n.title, n.node_type, g.lat, g.lng
         FROM geo_index g
         JOIN nodes n ON n.id = g.node_id
         WHERE (g.country = ?1 OR g.city = ?1)
           AND g.source LIKE 'geo-tag%'
         ORDER BY n.importance DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![place, limit as i64], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, f64>(3)?,
            r.get::<_, f64>(4)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// 区域覆盖度报告 — 按国家统计知识节点密度, 识别数据缺失区域。
///
/// 返回 Vec<(国家, 节点数)>。覆盖度 = 该国家已挂载的知识节点数 (geo_index 中
/// country 字段非空且非 geonames-cities 城市点)。缺失区域 = 节点数低于阈值
/// 或完全无节点的国家。
pub fn geo_coverage_report(conn: &Connection, min_threshold: i64) -> rusqlite::Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT country, COUNT(*) AS cnt
         FROM geo_index
         WHERE source != 'geonames-cities'
           AND country != ''
         GROUP BY country
         ORDER BY cnt DESC",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    // 标记缺失: 覆盖度低于阈值的国家排在最后
    out.sort_by(|a, b| {
        let a_missing = a.1 < min_threshold;
        let b_missing = b.1 < min_threshold;
        b_missing.cmp(&a_missing).then(b.1.cmp(&a.1))
    });
    Ok(out)
}

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

/// 查询单个坐标的海拔 (Open-Meteo API)。失败返回 Err, 不含缓存。
fn fetch_elevation_single(lat: f64, lng: f64) -> Result<Option<f64>, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/elevation?latitude={}&longitude={}",
        lat, lng
    );
    let resp = super::nt_http::run_blocking(|| {
        super::nt_http::shared_blocking_client().get(&url).send()
    })
    .map_err(|e| format!("elevation fetch error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} for elevation", resp.status()));
    }
    let body = resp.text().map_err(|e| format!("elevation read: {}", e))?;
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("elevation parse: {}", e))?;
    let elev = v
        .get("elevation")
        .and_then(|a| a.as_array())
        .and_then(|a| a.first())
        .and_then(|n| n.as_f64());
    Ok(elev)
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
            match super::nt_http::run_blocking(|| {
                super::nt_http::shared_blocking_client().get(&url).send()
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
pub fn query_elevations(conn: &Connection, limit: usize) -> Result<Vec<(String, f64, f64, f64)>, String> {
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

/// 气象快照记录。
#[derive(Debug, Clone)]
pub struct WeatherRecord {
    pub node_id: String,
    pub lat: f64,
    pub lng: f64,
    pub temp_c: Option<f64>,
    pub pressure_msl: Option<f64>,
    pub wind_kmh: Option<f64>,
    pub precip_mm: Option<f64>,
    pub elevation_m: Option<f64>,
    pub fetched_at: i64,
}

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

/// 单个坐标的实时气象快照 (Open-Meteo forecast API)。
/// 返回值: (temp_c, pressure_msl_hpa, wind_kmh, precip_mm, elevation_m)。
fn fetch_weather_single(lat: f64, lng: f64) -> Result<Option<(f64, f64, f64, f64, f64)>, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,pressure_msl,wind_speed_10m,precipitation&forecast_days=1",
        lat, lng
    );
    let resp = super::nt_http::run_blocking(|| {
        super::nt_http::shared_blocking_client().get(&url).send()
    })
    .map_err(|e| format!("weather fetch error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} for weather", resp.status()));
    }
    let body = resp.text().map_err(|e| format!("weather read: {}", e))?;
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("weather parse: {}", e))?;
    let cur = match v.get("current") {
        Some(c) => c,
        None => return Ok(None),
    };
    let temp = cur.get("temperature_2m").and_then(|n| n.as_f64());
    let pressure = cur.get("pressure_msl").and_then(|n| n.as_f64());
    let wind = cur.get("wind_speed_10m").and_then(|n| n.as_f64());
    let precip = cur.get("precipitation").and_then(|n| n.as_f64());
    let elev = v.get("elevation").and_then(|n| n.as_f64());
    match (temp, pressure, wind, precip) {
        (Some(t), Some(p), Some(w), Some(pr)) => Ok(Some((t, p, w, pr, elev.unwrap_or(0.0)))),
        _ => Ok(None),
    }
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
            match super::nt_http::run_blocking(|| {
                super::nt_http::shared_blocking_client().get(&url).send()
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
                                    let wind =
                                        item.get("wind_speed_10m").and_then(|n| n.as_f64());
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
pub fn query_weather(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<WeatherRecord>, String> {
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


/// 摄取全球 Holocene 火山 (Smithsonian GVP WFS, ~1,214 个)。
/// 数据源: GeoServer WFS GVP-VOTW:E3WebApp_HoloceneVolcanoes (JSON, 2026 更新 v5.4.0)。
/// 注意: PropertyName 必须限定为 VolcanoNumber,VolcanoName,Country,GeoLocation,
///      否则服务器对超长 Remarks 字段截断 JSON 响应。
/// JSON 结构: features[].geometry = Point [lng, lat]; properties = {VolcanoNumber, VolcanoName, Country}。
/// node_id = geo:volcano:{volcano_number} (GVP 火山编号保证唯一)。
pub fn ingest_geo_volcanoes(
    conn: &mut Connection,
    url_or_path: &str,
    limit: usize,
) -> Result<usize, String> {
    let body = if url_or_path.starts_with("file://") || std::path::Path::new(url_or_path).exists() {
        let path = url_or_path.strip_prefix("file://").unwrap_or(url_or_path);
        std::fs::read_to_string(path)
            .map_err(|e| format!("读取本地文件失败: {} ({})", e, path))?
    } else {
        let url = format!(
            "{}&PropertyName=VolcanoNumber,VolcanoName,Country,GeoLocation&maxFeatures={}",
            url_or_path, limit
        );
        let resp = super::nt_http::run_blocking(|| {
            super::nt_http::shared_blocking_client().get(&url).send()
        })
        .map_err(|e| format!("volcanoes fetch error: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {} for {}", resp.status(), url_or_path));
        }
        resp.text().map_err(|e| format!("read: {}", e))?
    };

    let fc: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("GeoJSON 解析失败 (volcanoes): {}", e))?;
    let features = fc
        .get("features")
        .and_then(|f| f.as_array())
        .ok_or_else(|| "GeoJSON 缺少 features 数组".to_string())?
        .clone();

    let mut existing: std::collections::HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT node_id FROM geo_index WHERE source='gvp-volcanoes'")
            .map_err(|e| format!("existing volcanoes prepare: {}", e))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| format!("existing volcanoes query: {}", e))?;
        let mut set = std::collections::HashSet::new();
        for r in rows {
            set.insert(r.map_err(|e| format!("existing volcano row: {}", e))?);
        }
        set
    };

    let mut tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("tx begin: {}", e))?;
    let mut count = 0usize;
    const BATCH: usize = 500;

    for feat in features.iter().take(limit) {
        let props = feat.get("properties").cloned().unwrap_or(serde_json::Value::Null);
        let num = props
            .get("VolcanoNumber")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let name = props
            .get("VolcanoName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let country = props
            .get("Country")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if num == 0 || name.is_empty() {
            continue;
        }
        let geom = feat.get("geometry").cloned().unwrap_or(serde_json::Value::Null);
        let coords = geom.get("coordinates").cloned().unwrap_or(serde_json::Value::Null);
        let lng = coords.as_array().and_then(|a| a.first()).and_then(|v| v.as_f64());
        let lat = coords.as_array().and_then(|a| a.get(1)).and_then(|v| v.as_f64());
        let (Some(lat), Some(lng)) = (lat, lng) else {
            continue;
        };
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
            continue;
        }

        let node_id = format!("geo:volcano:{}", num);
        if existing.contains(&node_id) {
            continue;
        }
        existing.insert(node_id.clone());

        let name_clean = name.replace('\'', "''");
        let country_clean = country.replace('\'', "''");
        let tags = format!("火山,holocene,{}", country_clean);
        upsert_geo(
            &tx,
            &GeoRecord {
                node_id,
                lat,
                lng,
                country: country_clean.clone(),
                region: String::new(),
                city: name_clean.clone(),
                tags,
                source: "gvp-volcanoes".into(),
                confidence: 1.0,
            },
        )
        .map_err(|e| format!("volcano upsert: {}", e))?;
        count += 1;

        if count % BATCH == 0 {
            tx.commit().map_err(|e| format!("tx commit: {}", e))?;
            tx = conn
                .unchecked_transaction()
                .map_err(|e| format!("tx begin: {}", e))?;
        }
    }
    tx.commit().map_err(|e| format!("tx commit: {}", e))?;

    Ok(count)
}

/// 导出 geo_index 为 NT-Pack 高密度格式文件 (R-P79 接线: NT-Pack 生产消费者)
///
/// `source` 过滤来源 (如 "ourairports"), None = 全部; `limit` 上限, 0 = 不限。
/// 返回 (导出条数, 文件字节数)。
pub fn export_geo_ntpack(
    conn: &Connection,
    source: Option<&str>,
    limit: usize,
    path: &str,
) -> Result<(usize, usize), String> {
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pack::{GeoPoint, PackEncoder};

    let mut sql = String::from(
        "SELECT node_id, lat, lng, country, region, city, tags, source FROM geo_index",
    );
    let mut params: Vec<String> = Vec::new();
    if let Some(s) = source {
        sql.push_str(" WHERE source = ?");
        params.push(s.to_string());
    }
    sql.push_str(" ORDER BY node_id");
    if limit > 0 {
        sql.push_str(" LIMIT ?");
        params.push(limit.to_string());
    }

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |r| {
            Ok(GeoPoint {
                node_id: r.get(0)?,
                lat: r.get(1)?,
                lng: r.get(2)?,
                country: r.get(3)?,
                region: r.get(4)?,
                city: r.get(5)?,
                tags: r.get(6)?,
                source: r.get(7)?,
            })
        })
        .map_err(|e| format!("query: {}", e))?;

    let points: Vec<GeoPoint> = rows
        .map(|r| r.map_err(|e| format!("row: {}", e)))
        .collect::<Result<_, _>>()?;
    let n = points.len();
    if n == 0 {
        return Err("geo_index 无匹配数据".into());
    }

    // E5 定点 + zstd 熵压缩 (默认配置, 见 nt_memory_pack)
    let enc = PackEncoder::new(5, true);
    let bytes = enc.encode(&points);

    std::fs::write(path, &bytes).map_err(|e| format!("写文件 {}: {}", path, e))?;
    Ok((n, bytes.len()))
}

/// 从 NT-Pack 文件解码回读 geo_index 数据 (验证/恢复用)
pub fn import_geo_ntpack(path: &str) -> Result<(usize, Vec<crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pack::GeoPoint>), String> {
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pack::PackDecoder;
    let bytes = std::fs::read(path).map_err(|e| format!("读文件 {}: {}", path, e))?;
    let (dec, points) = PackDecoder::decode(&bytes)?;
    Ok((points.len(), points))
}

/// 从 NT-Pack 文件导入回 KB geo_index (备份恢复/跨机器传输, 幂等 upsert)
///
/// 返回导入条数。confidence 默认 0.0 (NT-Pack 不携带该字段)。
pub fn import_geo_ntpack_to_kb(conn: &Connection, path: &str) -> Result<usize, String> {
    let (n, points) = import_geo_ntpack(path)?;
    if n == 0 {
        return Err("NT-Pack 文件无数据".into());
    }
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("tx begin: {}", e))?;
    for p in &points {
        upsert_geo(
            &tx,
            &GeoRecord {
                node_id: p.node_id.clone(),
                lat: p.lat,
                lng: p.lng,
                country: p.country.clone(),
                region: p.region.clone(),
                city: p.city.clone(),
                tags: p.tags.clone(),
                source: p.source.clone(),
                confidence: 0.0,
            },
        )
        .map_err(|e| format!("upsert {}: {}", p.node_id, e))?;
    }
    tx.commit().map_err(|e| format!("tx commit: {}", e))?;
    Ok(n)
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
        assert_eq!(match_city_in_text("Stanford University research").map(|c| c.0), Some("旧金山"));
        assert_eq!(match_city_in_text("牛津大学 的研究").map(|c| c.0), Some("牛津"));
        assert_eq!(match_city_in_text("A study of Londoner culture").map(|c| c.0), Some("伦敦"));
        // 词边界: "San" 不应命中 "Santa", "Tokyo" 不应命中 "Tokyopop"
        assert_eq!(match_city_in_text("Santa Monica beach").map(|c| c.0), None);
        assert_eq!(match_city_in_text("量子力学导论").map(|c| c.0), None);
        // 词形派生: "Londoner"/"London-based" 命中伦敦, "Oxfordian" 命中牛津
        assert_eq!(match_city_in_text("a London-based firm").map(|c| c.0), Some("伦敦"));
        assert_eq!(match_city_in_text("Oxfordian scholarship").map(|c| c.0), Some("牛津"));
        // 子地点归入主城市, 坐标与 city 字段一致 (Stanford/硅谷 → 旧金山市中心)
        assert_eq!(match_city_in_text("Stanford University research").map(|c| c.0), Some("旧金山"));
        assert_eq!(match_city_in_text("Stanford University research").map(|c| c.2), Some(37.7749));
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
}