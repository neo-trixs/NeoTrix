//! nt_geo_ntpack_cold — NT-Pack 打包/冷存透明读 (B1/A4)，行为零变更纯搬移。

use rusqlite::{params, Connection, Result};

use super::nt_geo_base::{query_bbox, query_by_place, upsert_geo};
use super::nt_geo_types::GeoRecord;

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
    use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_pack::{GeoPoint, PackEncoder};

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
pub fn import_geo_ntpack(
    path: &str,
) -> Result<
    (
        usize,
        Vec<crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_pack::GeoPoint>,
    ),
    String,
> {
    use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_pack::PackDecoder;
    let bytes = std::fs::read(path).map_err(|e| format!("读文件 {}: {}", path, e))?;
    let (_dec, points) = PackDecoder::decode(&bytes)?;
    Ok((points.len(), points))
}

/// **A4 追加模式** — 向 NT-Pack 归档文件增量合并写入 (幂等)。
///
/// NT-Pack 是"LCP 字典 + 坐标 delta 链 + zstd 整块 + 尾部 CRC" 的紧凑格式,
/// **不支持原地追加**: 字典重叠、delta 尾态、checksum 都会随新数据改变。
/// 因此追加采用 merge-append: 读旧文件 → 按 node_id 合并 (新覆盖旧) → 全量重编。
/// 当前规模 (50k 条 encode <1s) 下开销可接受; 若需避免全量重编, 需切分块格式 (A5)。
///
/// `path` 不存在时等价于新建; 返回 (写入后总条数, 文件字节)。
pub fn append_geo_ntpack(
    path: &str,
    new_points: &[crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_pack::GeoPoint],
) -> Result<(usize, usize), String> {
    use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_pack::{GeoPoint, PackEncoder};
    use std::collections::HashMap;

    let mut merged: HashMap<String, GeoPoint> = HashMap::new();
    match import_geo_ntpack(path) {
        Ok((_, existing)) => {
            for p in existing {
                merged.insert(p.node_id.clone(), p);
            }
        }
        Err(_e) if !std::path::Path::new(path).exists() => {
            // 新文件: 无既有数据
        }
        Err(e) => return Err(format!("读既有归档 {}: {}", path, e)),
    }
    for p in new_points {
        merged.insert(p.node_id.clone(), p.clone());
    }

    let mut pts: Vec<GeoPoint> = merged.into_values().collect();
    pts.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    let enc = PackEncoder::new(5, true);
    let bytes = enc.encode(&pts);
    std::fs::write(path, &bytes).map_err(|e| format!("写 {}: {}", path, e))?;
    Ok((pts.len(), bytes.len()))
}

/// 从 NT-Pack 文件导入回 KB geo_index (备份恢复/跨机器传输, 幂等 upsert)
///
/// 返回导入条数。confidence 默认 0.0 (NT-Pack 不携带该字段)。
/// 采用 BATCH 分批提交, 避免单事务持写锁过久 (B1 冷恢复大文件场景)。
pub fn import_geo_ntpack_to_kb(conn: &Connection, path: &str) -> Result<usize, String> {
    const BATCH: usize = 500;
    let (n, points) = import_geo_ntpack(path)?;
    if n == 0 {
        return Err("NT-Pack 文件无数据".into());
    }
    let mut tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("tx begin: {}", e))?;
    for (i, p) in points.iter().enumerate() {
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
        if (i + 1).is_multiple_of(BATCH) {
            tx.commit().map_err(|e| format!("tx commit: {}", e))?;
            tx = conn
                .unchecked_transaction()
                .map_err(|e| format!("tx begin: {}", e))?;
        }
    }
    tx.commit().map_err(|e| format!("tx commit: {}", e))?;
    Ok(n)
}

/// B1 冷存储: 将指定 source 的 geo_index 数据归档为 NT-Pack 冷层文件并从热表删除。
///
/// 流程: 导出该 source → 文件落盘 → 事务内 DELETE 热表行 → 返回 (归档条数, 字节)。
/// 命名约定 `geo_<source>.ntpack` 供 [`geo_cold_layers`] 枚举。幂等: 若该 source
/// 已全部归档 (热表无数据) 则返回 Err, 不会重复导出空文件。
///
/// 崩溃窗口: 文件已写而 DELETE 未提交时数据双在 (文件 + 热表), 可安全重跑或导入恢复;
/// DELETE 先于写文件不会发生 (顺序保证), 不丢数据。
pub fn archive_geo_cold(
    conn: &Connection,
    source: &str,
    path: &str,
) -> Result<(usize, usize), String> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM geo_index WHERE source = ?",
            params![source],
            |r| r.get(0),
        )
        .map_err(|e| format!("count geo source: {}", e))?;
    if n == 0 {
        return Err(format!(
            "geo source '{}' 热表无数据 (可能已归档, 幂等拒绝重复归档)",
            source
        ));
    }

    let (exported, bytes) = export_geo_ntpack(conn, Some(source), 0, path)?;
    if exported == 0 {
        return Err("导出 0 条, 取消归档".into());
    }

    // 文件已落盘, 现在从热表删除该 source (事务保证原子)
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("archive tx begin: {}", e))?;
    tx.execute("DELETE FROM geo_index WHERE source = ?", params![source])
        .map_err(|e| format!("archive delete: {}", e))?;
    tx.commit()
        .map_err(|e| format!("archive tx commit: {}", e))?;

    Ok((exported, bytes))
}

/// B1 冷存储: 枚举冷层目录中 `geo_*.ntpack` 归档文件。
///
/// 返回 Vec<(source, 路径, 文件字节数)>, 供恢复/前端层感知使用。
pub fn geo_cold_layers(dir: &str) -> Result<Vec<(String, String, u64)>, String> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(dir).map_err(|e| format!("读冷层目录 {}: {}", dir, e))?;
    for entry in rd.flatten() {
        let fname = entry.file_name().to_string_lossy().to_string();
        // 全量导出文件 (geo_index.ntpack) 是 export/import 的整库镜像, 非单个 source 冷层
        if fname == "geo_index.ntpack" {
            continue;
        }
        let Some(rest) = fname.strip_prefix("geo_").map(|s| s.to_string()) else {
            continue;
        };
        if !rest.ends_with(".ntpack") {
            continue;
        }
        let source = rest.trim_end_matches(".ntpack").to_string();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        out.push((source, entry.path().to_string_lossy().to_string(), size));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// 冷层数据源枚举 + 解码: 该目录下所有 `geo_<source>.ntpack` 记录合并。
///
/// 供透明读路径 (B1) 使用 — 归档后热表已删, 查询须兜底解码冷层。
/// 返回 (source, 解码出的 GeoRecord 列表)。confidence 默认 0.0 (NT-Pack 不携带)。
fn cold_layers_records(dir: &str) -> Vec<(String, Vec<GeoRecord>)> {
    use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_pack::PackDecoder;
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname == "geo_index.ntpack" {
            continue; // 全量镜像非冷层
        }
        let Some(rest) = fname.strip_prefix("geo_") else {
            continue;
        };
        if !rest.ends_with(".ntpack") {
            continue;
        }
        let source = rest.trim_end_matches(".ntpack").to_string();
        let Ok(bytes) = std::fs::read(entry.path()) else {
            continue;
        };
        let Ok((_, pts)) = PackDecoder::decode(&bytes) else {
            continue;
        };
        let recs: Vec<GeoRecord> = pts
            .into_iter()
            .map(|p| GeoRecord {
                node_id: p.node_id,
                lat: p.lat,
                lng: p.lng,
                country: p.country,
                region: p.region,
                city: p.city,
                tags: p.tags,
                source: p.source,
                confidence: 0.0,
            })
            .collect();
        out.push((source, recs));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// B1 透明读路径: 包围盒查询 + 冷层兜底。
///
/// 语义 = [`query_bbox`] 但 hot 结果不足 `limit` 时, 惰性解码冷层目录
/// (`~/.neotrix/geo`) 补充命中记录 (按 confidence 降序, 冷层置信 0.0 → 排末)。返回
/// `(records, cold_hits)` 其中 cold_hits 为冷层补充条数。
pub fn query_bbox_with_cold(
    conn: &Connection,
    min_lat: f64,
    min_lng: f64,
    max_lat: f64,
    max_lng: f64,
    limit: usize,
    cold_dir: &str,
) -> Result<(Vec<GeoRecord>, usize), String> {
    let hot = query_bbox(conn, min_lat, min_lng, max_lat, max_lng, limit)
        .map_err(|e| format!("hot bbox: {}", e))?;
    if hot.len() >= limit {
        return Ok((hot, 0));
    }

    let mut seen: std::collections::HashSet<String> =
        hot.iter().map(|r| r.node_id.clone()).collect();
    let mut cold_hits = Vec::new();
    let quota = limit - hot.len();
    for (_, recs) in cold_layers_records(cold_dir) {
        for r in recs {
            if cold_hits.len() >= quota {
                break;
            }
            if r.lat >= min_lat
                && r.lat <= max_lat
                && r.lng >= min_lng
                && r.lng <= max_lng
                && seen.insert(r.node_id.clone())
            {
                cold_hits.push(r);
            }
        }
    }
    let cold_count = cold_hits.len();
    let mut all = hot;
    all.append(&mut cold_hits);
    // hot 已按 confidence DESC; 冷层 0.0 一律排后, 保持总体降序
    Ok((all, cold_count))
}

/// B1 透明读路径: 国家/区域/城市查询 + 冷层兜底 (同 [`query_bbox_with_cold`])。
pub fn query_by_place_with_cold(
    conn: &Connection,
    country: &str,
    region: &str,
    city: &str,
    limit: usize,
    cold_dir: &str,
) -> Result<(Vec<GeoRecord>, usize), String> {
    let hot = query_by_place(conn, country, region, city, limit)
        .map_err(|e| format!("hot by_place: {}", e))?;
    if hot.len() >= limit {
        return Ok((hot, 0));
    }

    let mut seen: std::collections::HashSet<String> =
        hot.iter().map(|r| r.node_id.clone()).collect();
    let mut cold_added = 0usize;
    let quota = limit - hot.len();
    let mut out = hot;
    for (_, recs) in cold_layers_records(cold_dir) {
        for r in recs {
            if cold_added >= quota {
                break;
            }
            if (country.is_empty() || r.country == country)
                && (region.is_empty() || r.region == region)
                && (city.is_empty() || r.city == city)
                && seen.insert(r.node_id.clone())
            {
                out.push(r);
                cold_added += 1;
            }
        }
    }
    Ok((out, cold_added))
}
///
/// 修调研发现的"前端层计数突变"风险 — 归档后 `kb_geo_layers` 只统计热表,
/// 层计数会骤减; 本函数冷热合并, 让层感知不被归档扭曲。返回
/// Vec<(source, warm_count, cold_bytes, 冷层路径?)>。
pub fn geo_layer_inventory(
    conn: &Connection,
    cold_dir: &str,
) -> Result<Vec<(String, i64, Option<(String, u64)>)>, String> {
    let mut stmt = conn
        .prepare("SELECT source, COUNT(*) FROM geo_index GROUP BY source ORDER BY source")
        .map_err(|e| format!("inventory query: {}", e))?;
    let warm: Vec<(String, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| format!("inventory map: {}", e))?
        .map(|r| r.map_err(|e| format!("inventory row: {}", e)))
        .collect::<Result<_, _>>()?;

    let cold: Vec<(String, String, u64)> = geo_cold_layers(cold_dir)?;

    // 合并: warm 优先, cold 仅附加冷层信息 (若该 source 既热又冷, 仍保留冷层标记)
    let mut out: Vec<(String, i64, Option<(String, u64)>)> =
        warm.into_iter().map(|(s, c)| (s, c, None)).collect();
    let cold_map: std::collections::HashMap<String, (String, u64)> =
        cold.into_iter().map(|(s, p, b)| (s, (p, b))).collect();
    for (s, (p, b)) in &cold_map {
        if let Some(entry) = out.iter_mut().find(|(src, _, _)| src == s) {
            entry.2 = Some((p.clone(), *b));
        } else {
            out.push((s.clone(), 0, Some((p.clone(), *b))));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}
