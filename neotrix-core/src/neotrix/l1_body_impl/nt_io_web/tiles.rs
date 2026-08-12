//! B3 HTTP 瓦片服务 — 以 bbox+zoom 提供 NT-Pack 冷层地理数据。
//!
//! 复用 [`crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::query_bbox_with_cold`]
//! (热表 + 冷层兜底, B1 透明读路径) 作为数据源, 避免二次实现解码。
//! 端点: `GET /api/geo/tiles?bbox=w,s,e,n&limit=N&source=S`
//!   - bbox: 逗号分隔 4 值 (west, south, east, north), 经纬度
//!   - limit: 单请求上限 (默认 2000, 硬上限 20k)
//!   - source: 可选源过滤 (如 `shanhai` → peaks+mappings)
//! 返回 GeoJSON FeatureCollection, 供外部 GIS/前端瓦片叠加消费。

use axum::{extract::Query, Json};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TileParams {
    pub bbox: String,
    pub limit: Option<usize>,
    pub source: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct TileResponse {
    pub features: Vec<TileFeature>,
    pub count: usize,
    pub cold_hits: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct TileFeature {
    pub node_id: String,
    pub lon: f64,
    pub lat: f64,
    pub source: String,
    pub confidence: f64,
    pub props: serde_json::Value,
}

pub async fn geo_tiles_handler(
    Query(params): Query<TileParams>,
) -> Result<Json<TileResponse>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let parsed = parse_bbox(&params.bbox);
    let (w, s, e, n) = match parsed {
        Ok(v) => v,
        Err(msg) => {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid_bbox", "message": msg })),
            ))
        }
    };
    if !(w < e && s < n) {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_bbox",
                "message": "bbox 需满足 west<east && south<north"
            })),
        ));
    }
    if !(-180.0..=180.0).contains(&w) || !(-180.0..=180.0).contains(&e)
        || !(-90.0..=90.0).contains(&s) || !(-90.0..=90.0).contains(&n)
    {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_bbox",
                "message": "经纬度越界: lon∈[-180,180], lat∈[-90,90]"
            })),
        ));
    }
    let limit = (params.limit.unwrap_or(2000)).clamp(1, 20_000);

    // query_bbox_with_cold 是同步 SQLite + 冷层解码, 移到阻塞线程池避免卡 async 运行时
    let (w, s, e, n, limit) = (w, s, e, n, limit);
    let source = params.source.clone();
    let result = tokio::task::spawn_blocking(move || {
        let kb = kb_path();
        let conn = rusqlite::Connection::open(&kb)
            .map_err(|e| format!("open kb: {}", e))?;
        let (records, cold_hits) = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::query_bbox_with_cold(
            &conn,
            s,
            w,
            n,
            e,
            limit,
            &cold_dir(),
        )?;
        // source 过滤 (与 kb_geo_points_pack 语义对齐)
        let mut out = Vec::new();
        for r in records {
            let matches = match source.as_deref() {
                Some("shanhai") => r.source == "shanhai-peaks" || r.source == "shanhai-mappings",
                Some(sf) => r.source == sf,
                None => true,
            };
            if !matches {
                continue;
            }
            out.push(TileFeature {
                node_id: r.node_id.clone(),
                lon: r.lng,
                lat: r.lat,
                source: r.source.clone(),
                confidence: r.confidence,
                props: serde_json::json!({
                    "country": r.country,
                    "region": r.region,
                    "city": r.city,
                    "tags": r.tags,
                }),
            });
        }
        Ok::<(Vec<TileFeature>, usize), String>((out, cold_hits))
    })
    .await
    .map_err(|e| internal_err(&e.to_string()))?;

    let (features, cold_hits) = match result {
        Ok(v) => v,
        Err(e) => return Err(internal_err(&e)),
    };
    let count = features.len();
    Ok(Json(TileResponse { features, count, cold_hits }))
}

fn cold_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home).join(".neotrix").join("geo").to_string_lossy().to_string()
}

fn kb_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home).join(".neotrix").join("knowledge.db")
}

fn parse_bbox(raw: &str) -> Result<(f64, f64, f64, f64), String> {
    let parts: Vec<&str> = raw.split(',').map(|s| s.trim()).collect();
    if parts.len() != 4 {
        return Err("bbox 需为 4 个逗号分隔数字 (west,south,east,north)".into());
    }
    let vals: Result<Vec<f64>, _> = parts.iter().map(|p| p.parse::<f64>()).collect();
    match vals {
        Ok(v) => Ok((v[0], v[1], v[2], v[3])),
        Err(_) => Err("bbox 含非数字".into()),
    }
}

fn internal_err(msg: &str) -> (axum::http::StatusCode, Json<serde_json::Value>) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "internal", "message": msg })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bbox_ok() {
        assert_eq!(parse_bbox("73.6,18.5,135.1,53.6").unwrap(), (73.6, 18.5, 135.1, 53.6));
        assert_eq!(parse_bbox(" -180 , -90 , 180 , 90 ").unwrap(), (-180.0, -90.0, 180.0, 90.0));
    }

    #[test]
    fn parse_bbox_rejects_bad_input() {
        assert!(parse_bbox("").is_err());
        assert!(parse_bbox("1,2,3").is_err());
        assert!(parse_bbox("a,b,c,d").is_err());
        assert!(parse_bbox("1,2,3,4,5").is_err());
    }

    #[test]
    fn bbox_validation_orders_and_ranges() {
        // 无效顺序 (w>=e 或 s>=n) 会被 handler 拒绝
        let bad_order = parse_bbox("135.1,18.5,73.6,53.6").unwrap();
        assert!(!(bad_order.0 < bad_order.2 && bad_order.1 < bad_order.3));
        // 越界值会被 handler 拒绝
        let out_of_range = parse_bbox("181,-90,135,90").unwrap();
        assert!(!(-180.0..=180.0).contains(&out_of_range.0));
    }
}
