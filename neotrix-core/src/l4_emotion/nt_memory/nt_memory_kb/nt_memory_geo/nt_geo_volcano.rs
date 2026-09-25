//! nt_geo_volcano — 火山摄取 (Smithsonian GVP WFS)，行为零变更纯搬移。

use rusqlite::Connection;

use super::nt_geo_base::upsert_geo;
use super::nt_geo_types::GeoRecord;

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
        std::fs::read_to_string(path).map_err(|e| format!("读取本地文件失败: {} ({})", e, path))?
    } else {
        let url = format!(
            "{}&PropertyName=VolcanoNumber,VolcanoName,Country,GeoLocation&maxFeatures={}",
            url_or_path, limit
        );
        let resp = crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::run_blocking(|| {
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::shared_blocking_client().get(&url).send()
        })
        .map_err(|e| format!("volcanoes fetch error: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {} for {}", resp.status(), url_or_path));
        }
        resp.text().map_err(|e| format!("read: {}", e))?
    };

    let fc: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("GeoJSON 解析失败 (volcanoes): {}", e))?;
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
        let props = feat
            .get("properties")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
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
        let geom = feat
            .get("geometry")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let coords = geom
            .get("coordinates")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let lng = coords
            .as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_f64());
        let lat = coords
            .as_array()
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_f64());
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
