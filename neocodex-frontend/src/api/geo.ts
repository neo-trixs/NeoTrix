import { call } from './client'

/* ════════════════════════════════════════════
   api/geo.ts — 地图数据 API 层（前后端分离）
   后端: kb_cmds.rs → kb_geo_points / kb_geo_stats / kb_geo_layers
   前端: 本模块封装 Tauri IPC，GlobeView 只负责渲染
   ════════════════════════════════════════════ */

export interface GeoPoint {
  node_id: string
  lat: number
  lng: number
  country: string
  region: string
  city: string
  tags: string
  source: string
  confidence: number
}

/** 地图分层摘要（各数据源计数） */
export interface GeoLayerSummary {
  source: string
  count: number
}

/** 幻境层数据源（山海经） */
export const MIRAGE_SOURCES = ['shanhai-peaks', 'shanhai-mappings'] as const

export function isMirage(p: { source: string }): boolean {
  return (MIRAGE_SOURCES as readonly string[]).includes(p.source)
}

/** 导出地理索引点 (geo_index) — 3D 地球知识节点数据源。
 *  `source` 可选过滤："shanhai" 返回全部幻境点（peaks+mappings），
 *  其他字符串按 source 精确匹配，undefined 返回混合（按 confidence 排序）。 */
export function geoPoints(limit?: number, source?: string): Promise<GeoPoint[]> {
  return call('kb_geo_points', { limit: limit ?? null, source: source ?? null })
}

/** B2 v0: 从 NT-Pack 高密度文件读地理点（绕 SQLite，冷层文件优先）。
 *  IPC 契约同 `geoPoints`，前端无感切换。默认仍走 SQLite，需显式开启 pack 源。 */
export function geoPointsPack(limit?: number, source?: string): Promise<GeoPoint[]> {
  return call('kb_geo_points_pack', { limit: limit ?? null, source: source ?? null })
}

/** 地理索引统计 (总数, 有国家数) */
export function geoStats(): Promise<[number, number]> {
  return call('kb_geo_stats', {})
}

/** 地图分层摘要 — 各数据源计数，前端据此决定加载策略 */
export function geoLayers(): Promise<GeoLayerSummary[]> {
  return call('kb_geo_layers', {})
}

/** 海拔点记录 — geo_elevation 表 + geo_index 来源，供海拔渐变着色 */
export interface GeoElevationPoint {
  node_id: string
  lat: number
  lng: number
  elevation_m: number
  source: string
}

/** 导出海拔记录 (geo_elevation) — 前端按高度渐变着色 */
export function geoElevations(limit?: number): Promise<GeoElevationPoint[]> {
  return call('kb_geo_elevations', { limit: limit ?? null })
}