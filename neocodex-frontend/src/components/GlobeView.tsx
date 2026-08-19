import { onMount, onCleanup, createSignal, createEffect, Show } from 'solid-js'
import Globe from 'globe.gl'
import {
  geoPoints,
  geoPointsPack,
  geoLayers,
  geoElevations,
  isMirage,
  trajectoryAdd,
  trajectoryQuery,
  geoOfflinePack,
  type GeoPoint,
  type GeoLayerSummary,
  type TrajectoryRecord,
} from '../api/geo'
import { errText } from '../api/client'

/* ════════════════════════════════════════════
   GlobeView.tsx — 地球知识世界仿真 (3D 地图)
   前后端分离：数据经 api/geo.ts（Tauri IPC）拉取，本组件只负责复杂加载与渲染。

   分层渲染：
   - 真实层：world-atlas 国家边界 + geonames 城市点（青绿）+ geo-tag 地理标签节点（黄）
   - 自然层：natural-earth 河流（蓝）/ 湖泊（青）/ 海岸线（白）
   - 幻境层：shanhai-peaks（橙红）+ shanhai-mappings（琥珀），扩散光环动画
   - 海拔层：geo_elevation 数据 → 按高度渐变着色（绿→黄→橙→红→白）

   复杂加载策略：
   1. 先拉分层摘要 (kb_geo_layers) → 决定各层预算
   2. 幻境层全量拉取（数量少，必须完整）
   3. 真实层按预算采样（城市点 117k，只取 top-N）
   4. 渐进渲染：先真实层后幻境层，加载状态 + 图层开关
   ════════════════════════════════════════════ */

interface GlobeViewProps {
  limit?: number
  height?: number
  /** B2 v0: true 时数据源走 NT-Pack 高密度文件 (kb_geo_points_pack, 绕 SQLite) */
  usePack?: boolean
}

/* HTML 转义：pointLabel 经 globe.gl innerHTML 渲染，防 KB 数据注入 */
function escHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

/** 海拔渐变着色：0-5000m 绿→黄→橙→红→白 */
function elevColor(m: number): string {
  const stops: [number, string][] = [
    [0, '#228b22'],
    [300, '#66bb6a'],
    [800, '#c8e6c9'],
    [1500, '#ffd166'],
    [2500, '#ff8c42'],
    [3500, '#c62828'],
    [5000, '#ffffff'],
  ]
  if (m <= stops[0][0]) return stops[0][1]
  for (let i = 1; i < stops.length; i++) {
    if (m <= stops[i][0]) {
      const [t0, c0] = stops[i - 1]
      const [t1, c1] = stops[i]
      const f = (m - t0) / (t1 - t0)
      const hex = (a: number, b: number) =>
        Math.round(a + (b - a) * f)
          .toString(16)
          .padStart(2, '0')
      const r = hex(parseInt(c0.slice(1, 3), 16), parseInt(c1.slice(1, 3), 16))
      const g = hex(parseInt(c0.slice(3, 5), 16), parseInt(c1.slice(3, 5), 16))
      const b = hex(parseInt(c0.slice(5, 7), 16), parseInt(c1.slice(5, 7), 16))
      return `#${r}${g}${b}`
    }
  }
  return stops[stops.length - 1][1]
}

/** natural-earth 数据源前缀 → 渲染色 */
const NATURAL_COLOR: Record<string, string> = {
  'natural-earth-river': '#3d9bff',
  'natural-earth-lake': '#26c6da',
  'natural-earth-coastline': '#e0e0e0',
}

export function GlobeView(props: GlobeViewProps) {
  let containerRef: HTMLDivElement | undefined
  // 卸载守卫：异步数据到达时组件可能已销毁，禁止再 setSignal / 触碰 globe
  let disposed = false
  // B2 v0: usePack=true 时 7 路地理点走 NT-Pack 高密度文件 (kb_geo_points_pack),
  // 海拔仍走 SQLite (kb_geo_elevations, 无 pack 版本)。
  const fetchPts = props.usePack ? geoPointsPack : geoPoints
  const [points, setPoints] = createSignal<GeoPoint[]>([])
  const [layers, setLayers] = createSignal<GeoLayerSummary[]>([])
  const [loading, setLoading] = createSignal(true)
  const [error, setError] = createSignal('')
  const [hovered, setHovered] = createSignal<GeoPoint | null>(null)
  const [showReal, setShowReal] = createSignal(true)
  const [showMirage, setShowMirage] = createSignal(true)
  const [showNatural, setShowNatural] = createSignal(true)
  const [showElev, setShowElev] = createSignal(true)
  // globe 实例 + 分层数据（供顶层图层切换 effect 使用）
  const [globeInst, setGlobeInst] = createSignal<InstanceType<typeof Globe> | null>(null)
  const [cityPts, setCityPts] = createSignal<GeoPoint[]>([])
  const [geoTagPts, setGeoTagPts] = createSignal<GeoPoint[]>([])
  const [naturalPts, setNaturalPts] = createSignal<GeoPoint[]>([])
  const [shanhaiPts, setShanhaiPts] = createSignal<GeoPoint[]>([])
  // 海拔表：node_id → 海拔米（着色用）
  const [elevMap, setElevMap] = createSignal<Map<string, number>>(new Map())

  // C1/C2 状态：轨迹记录 + 离线包
  const [trajName, setTrajName] = createSignal('我的路线')
  const [trajKind, setTrajKind] = createSignal('route')
  const [trajRecording, setTrajRecording] = createSignal(false)
  const [trajPoints, setTrajPoints] = createSignal<GeoPoint[]>([])
  const [offlineName, setOfflineName] = createSignal('offline-pack')
  const [offlinePacks, setOfflinePacks] = createSignal<TrajectoryRecord[]>([])

  // 图层开关：真实层 / 幻境层 / 自然层 / 海拔层（响应式，顶层 effect）
  createEffect(() => {
    const globe = globeInst()
    if (!globe) return
    const real = showReal() ? [...cityPts(), ...geoTagPts()] : []
    const mirage = showMirage() ? shanhaiPts() : []
    const natural = showNatural() ? naturalPts() : []
    // 海拔开关同样注册为响应式依赖（Solid effect 依赖为读取跟踪式，非依赖数组）：
    // 切换后 effect 重跑 → pointsData 重设（kapsule setter 无值相等性检查，必然触发
    // three-globe 重渲）→ pointColor 回调在渲染期以最新 showElev() 值重新着色
    void showElev()
    globe.pointsData([...real, ...mirage, ...natural])
    globe.ringsData(showMirage() ? shanhaiPts() : [])
  })

  onMount(async () => {
    if (!containerRef) return
    const limit = props.limit ?? 3000
    const height = props.height ?? 600

    // 卸载守卫必须在同步前缀注册：await 之后调用 onCleanup 会因 Solid owner 上下文
    // 丢失而静默失效（solid-js 1.9.14 的 onCleanup 在 Owner === null 时 no-op），
    // 否则 disposed 永不置位，globe._destructor 与 resize 监听会泄漏。
    let globe: InstanceType<typeof Globe> | undefined
    let onResize: (() => void) | undefined
    onCleanup(() => {
      disposed = true
      if (onResize) window.removeEventListener('resize', onResize)
      globe?._destructor?.()
    })

    // 1) 分层摘要 → 决定加载预算（前后端分离：后端给摘要，前端定策略）
    let cityBudget = limit
    try {
      const ls = await geoLayers()
      if (disposed) return
      setLayers(ls)
      const otherTotal = ls
        .filter((l) => !['geonames-cities', 'geo-tag:keyword'].includes(l.source) && !isMirage(l))
        .reduce((acc, l) => acc + l.count, 0)
      cityBudget = Math.max(200, limit - otherTotal)
    } catch (e) {
      console.warn('[GlobeView] 分层摘要加载失败，使用默认预算:', e)
    }

    // 2) 分层拉取：真实城市点（预算采样）+ geo-tag 地理标签 + natural-earth + 全部 shanhai
    let cityPts: GeoPoint[] = []
    let shanhaiPts: GeoPoint[] = []
    let geoTagPts: GeoPoint[] = []
    let naturalPts: GeoPoint[] = []
    try {
      const [cities, shanhai, geotag, rivers, lakes, coast, glaciers, elevs] = await Promise.all([
        fetchPts(cityBudget, undefined),
        fetchPts(5000, 'shanhai'),
        fetchPts(2000, 'geo-tag:keyword'),
        fetchPts(200, 'natural-earth-river'),
        fetchPts(200, 'natural-earth-lake'),
        fetchPts(200, 'natural-earth-coastline'),
        fetchPts(200, 'natural-earth-glacier'),
        geoElevations(4000),
      ])
      if (disposed) return
      cityPts = cities.filter((p) => !isMirage(p) && !p.source.startsWith('geo-tag') && !p.source.startsWith('natural-earth'))
      shanhaiPts = shanhai
      geoTagPts = geotag.filter((p) => !isMirage(p))
      naturalPts = [...rivers, ...lakes, ...coast, ...glaciers]
      const em = new Map<string, number>()
      for (const e of elevs) em.set(e.node_id, e.elevation_m)
      setElevMap(em)
      setCityPts(cityPts)
      setGeoTagPts(geoTagPts)
      setNaturalPts(naturalPts)
      setShanhaiPts(shanhaiPts)
      setPoints([...cityPts, ...geoTagPts, ...naturalPts, ...shanhaiPts])
    } catch (e) {
      setError(errText(e))
    } finally {
      if (!disposed) setLoading(false)
    }

    // 3) 初始化 3D 地球
    globe = new Globe(containerRef, { animateIn: true })
      .width(containerRef.clientWidth)
      .height(height)
      .backgroundColor('rgba(5,10,25,0.95)')
      .showAtmosphere(true)
      .atmosphereColor('#3a6ea5')
      .atmosphereAltitude(0.18)
      .showGraticules(true)
    setGlobeInst(globe)

    // 4) 真实层：国家边界 (world-atlas TopoJSON)
    try {
      const res = await fetch('https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json')
      const topo = await res.json()
      const tc = await import('topojson-client')
      if (disposed) return
      const countries = (tc as unknown as { featureCollection(t: unknown, o: unknown): { features: GeoJSON.Feature[] } })
        .featureCollection(topo, topo.objects.countries)
      globe
        .polygonsData(countries.features)
        .polygonCapColor(() => 'rgba(40,70,120,0.35)')
        .polygonSideColor(() => 'rgba(40,70,120,0.15)')
        .polygonStrokeColor(() => 'rgba(90,140,200,0.5)')
        .polygonAltitude(0.01)
    } catch (e) {
      console.warn('[GlobeView] 国家边界加载失败(离线降级):', e)
    }

    // 5) 统一渲染：真实/自然/幻境 全部点（海拔模式开启时优先用海拔渐变着色）
    globe
      .pointLat((d: object) => (d as GeoPoint).lat)
      .pointLng((d: object) => (d as GeoPoint).lng)
      .pointAltitude(0.02)
      .pointsTransitionDuration(1200)
      .pointRadius((d: object) => {
        const p = d as GeoPoint
        if (isMirage(p)) return 0.55
        if (p.source.startsWith('natural-earth')) return 0.45
        if (p.source.startsWith('geo-tag')) return 0.36
        return 0.28
      })
      .pointColor((d: object) => {
        const p = d as GeoPoint
        // 海拔模式：有海拔记录的点按高度渐变（优先于固定色）
        if (showElev()) {
          const m = elevMap().get(p.node_id)
          if (m !== undefined) return elevColor(m)
        }
        if (p.source === 'shanhai-peaks') return '#ff6b35'
        if (p.source === 'shanhai-mappings') return '#ffd166'
        if (p.source.startsWith('natural-earth')) return NATURAL_COLOR[p.source] ?? '#4ecdc4'
        if (p.source.startsWith('geo-tag')) return '#fdd835'
        return '#4ecdc4'
      })
      .pointLabel((d: object) => {
        const p = d as GeoPoint
        const mirage = isMirage(p)
        const mirageBadge = mirage ? `<span style="color:#ffd166">【幻境·山海经】</span><br/>` : ''
        const natBadge = p.source.startsWith('natural-earth')
          ? `<span style="color:#3d9bff">【自然地理】</span><br/>`
          : p.source.startsWith('geo-tag')
            ? `<span style="color:#fdd835">【地理标签】</span><br/>`
            : ''
        const elev = elevMap().get(p.node_id)
        const elevLine = elev !== undefined
          ? `<br/><span style="color:#ffd166">⛰️ ${Math.round(elev)} m</span>`
          : ''
        return `<div style="font-family:system-ui;font-size:12px;color:#e6f1ff;background:rgba(10,20,40,0.92);padding:6px 10px;border-radius:6px;border:1px solid ${mirage ? 'rgba(255,209,102,0.5)' : 'rgba(90,140,200,0.4)'};max-width:260px">
          ${mirageBadge}${natBadge}<b>${escHtml(p.city || p.tags || '未名')}</b><br/>
          <span style="color:#8ab4f8">${escHtml(p.country || '未知地区')}</span>
          ${p.tags ? `<br/><span style="color:#9aa7bd">${escHtml(p.tags)}</span>` : ''}
          <br/><span style="color:#6b7a90">${p.lat.toFixed(2)}, ${p.lng.toFixed(2)}</span>${elevLine}
        </div>`
      })
      .onPointHover((d: object | null) => setHovered(d ? (d as GeoPoint) : null))
      // 🟡 修复：轨迹记录接线。此前 trajPoints 无任何采集路径——「开始记录」后点击
      // 地球不产生任何轨迹点，保存按钮因 length<2 永久禁用（功能坏死）。
      // 现在录制状态下点按任意城市/地理点即将其加入轨迹序列（相邻同坐标去重）。
      .onPointClick((d: object | null) => {
        if (!d || !trajRecording()) return
        const p = d as GeoPoint
        setTrajPoints(prev => {
          const last = prev[prev.length - 1]
          if (last && last.lat === p.lat && last.lng === p.lng) return prev
          return [...prev, p]
        })
      })

    // 6) 幻境层：shanhai 扩散光环 (rings 动画) + 高亮大点叠加
    globe
      .ringsData(shanhaiPts)
      .ringLat((d: object) => (d as GeoPoint).lat)
      .ringLng((d: object) => (d as GeoPoint).lng)
      .ringColor((d: object) => (d as GeoPoint).source === 'shanhai-peaks'
        ? (t: number) => `rgba(255,107,53,${1 - t})`     // 山峰：橙红扩散
        : (t: number) => `rgba(255,209,102,${1 - t})`)    // 映射：琥珀扩散
      .ringMaxRadius(4)
      .ringPropagationSpeed(1.8)
      .ringRepeatPeriod(900)

    // 7) 自适应尺寸
    onResize = () => globe!.width(containerRef?.clientWidth ?? 800)
    window.addEventListener('resize', onResize)
  })

  // 图层计数跟随开关：显示的是当前可见层的数据量
  const mirageCount = () => (showMirage() ? points().filter((p) => isMirage(p)).length : 0)
  const realCount = () => (showReal() ? points().filter((p) => !isMirage(p) && !p.source.startsWith('natural-earth')).length : 0)
  const naturalCount = () => (showNatural() ? points().filter((p) => p.source.startsWith('natural-earth')).length : 0)
  const elevCount = () => elevMap().size

  // C1 轨迹记录：开始/停止/保存
  function startTrajRecording() {
    setTrajRecording(true)
    setTrajPoints([])
  }
  function saveTraj() {
    if (trajPoints().length < 2) return
    const id = `traj-${Date.now()}`
    // 提取 lat/lng 序列
    const pts = trajPoints().flatMap(p => [p.lat, p.lng])
    // 计算 bbox
    const lats = trajPoints().map(p => p.lat)
    const lngs = trajPoints().map(p => p.lng)
    const west = Math.min(...lngs)
    const east = Math.max(...lngs)
    const south = Math.min(...lats)
    const north = Math.max(...lats)
    // 简单距离估算
    let dist = 0
    for (let i = 1; i < trajPoints().length; i++) {
      const p1 = trajPoints()[i - 1]
      const p2 = trajPoints()[i]
      dist += haversine(p1.lat, p1.lng, p2.lat, p2.lng)
    }
    trajectoryAdd(id, trajName(), trajKind(), pts, [west, south, east, north], dist)
      .then(() => {
        setTrajRecording(false)
        setTrajPoints([])
      })
      .catch((e: Error) => console.error('保存轨迹失败:', e))
  }

  // C2 离线包导出
  function exportOfflinePack() {
    if (!offlineName()) return
    // 使用当前视口 bbox (简化：用全球 bbox)
    const bbox: [number, number, number, number] = [-180, -90, 180, 90]
    geoOfflinePack(bbox, offlineName())
      .then((res: { path: string; count: number; bytes: number }) => {
        alert(`离线包已导出: ${res.path} (${res.count} 点, ${res.bytes} bytes)`)
      })
      .catch((e: Error) => console.error('导出离线包失败:', e))
  }

  // 列出离线包（简化：查询 trajectory 表中 kind='offline_pack' 的记录）
  function listOfflinePacks() {
    trajectoryQuery()
      .then((list: TrajectoryRecord[]) => setOfflinePacks(list))
      .catch((e: Error) => console.error('列出离线包失败:', e))
  }

  // Haversine 距离 (km)
  function haversine(lat1: number, lng1: number, lat2: number, lng2: number): number {
    const R = 6371
    const dLat = (lat2 - lat1) * Math.PI / 180
    const dLng = (lng2 - lng1) * Math.PI / 180
    const a = Math.sin(dLat/2)**2 + Math.cos(lat1*Math.PI/180)*Math.cos(lat2*Math.PI/180)*Math.sin(dLng/2)**2
    return 2 * R * Math.asin(Math.sqrt(a))
  }

  return (
    <div class="relative w-full overflow-hidden rounded-xl border border-slate-700/50 bg-[#050a19]">
      <div ref={containerRef} class="w-full" style={{ height: `${props.height ?? 600}px` }} />
      <Show when={loading()}>
        <div class="absolute inset-0 flex items-center justify-center text-slate-400 text-sm">
          🌍 加载地球知识世界…
        </div>
      </Show>
      <Show when={error()}>
        <div class="absolute top-3 left-3 rounded bg-red-900/60 px-3 py-1.5 text-xs text-red-200">
          {error()}
        </div>
      </Show>
      <Show when={!loading() && !error()}>
        {/* 图层控制条 */}
        <div class="absolute top-3 left-3 flex items-center gap-2 rounded bg-slate-900/70 px-3 py-1.5 text-xs text-slate-300 backdrop-blur">
          <span>🌍 {realCount()} 真实</span>
          <button
            class={clsx('layer-pill', showReal() && 'on-real')}
            onClick={() => setShowReal(!showReal())}
          >
            {showReal() ? '● 城市' : '○ 城市'}
          </button>
          <span class="text-cyan-300">🏞️ {naturalCount()} 自然</span>
          <button
            class={clsx('layer-pill', showNatural() && 'on-natural')}
            onClick={() => setShowNatural(!showNatural())}
          >
            {showNatural() ? '● 河湖' : '○ 河湖'}
          </button>
          <span class="text-amber-300">✨ {mirageCount()} 幻境</span>
          <button
            class={clsx('layer-pill', showMirage() && 'on-mirage')}
            onClick={() => setShowMirage(!showMirage())}
          >
            {showMirage() ? '● 山海' : '○ 山海'}
          </button>
          <span class="text-emerald-300">⛰️ {elevCount()}</span>
          <button
            class={clsx('layer-pill', showElev() && 'on-elev')}
            onClick={() => setShowElev(!showElev())}
          >
            {showElev() ? '● 海拔' : '○ 海拔'}
          </button>
          <span class="hidden lg:inline text-slate-500" title="数据源分布（分层摘要）">
            {layers()
              .map((l) => `${l.source.replace(/^shanhai-/, '').replace(/^natural-earth-/, 'NE-')}:${l.count}`)
              .join(' · ')}
          </span>
        </div>
      </Show>
      {/* C1/C2 usePack 扩展面板：轨迹记录 + 离线地图包 */}
      {props.usePack && !loading() && !error() && (
        <div class="absolute top-3 right-3 flex flex-col items-end gap-2">
          {/* 轨迹记录 */}
          <div class="flex items-center gap-2 rounded bg-slate-900/70 px-3 py-1.5 text-xs text-slate-300 backdrop-blur">
            <span>🗺️</span>
            <input
              type="text"
              placeholder="轨迹名称"
              onInput={(e) => setTrajName(e.currentTarget.value)}
              class="rounded bg-slate-800 px-2 py-1 text-white w-40 focus:outline-none focus:ring-1 focus:ring-cyan-500"
            />
            <input
              type="text"
              placeholder="类型 (route/flight/ship)"
              onInput={(e) => setTrajKind(e.currentTarget.value)}
              class="rounded bg-slate-800 px-2 py-1 text-white w-32 focus:outline-none focus:ring-1 focus:ring-cyan-500"
            />
            <button
              onClick={() => startTrajRecording()}
              class="px-2 py-1 rounded bg-cyan-600 hover:bg-cyan-500 text-white"
              disabled={trajRecording()}
            >
              {trajRecording() ? '⏺ 记录中' : '▶ 开始记录'}
            </button>
            <button
              onClick={() => saveTraj()}
              class="px-2 py-1 rounded bg-amber-600 hover:bg-amber-500 text-white"
              disabled={!trajRecording() || trajPoints().length < 2}
            >
              💾 保存轨迹
            </button>
          </div>
          {/* 离线包 */}
          <div class="flex items-center gap-2 rounded bg-slate-900/70 px-3 py-1.5 text-xs text-slate-300 backdrop-blur">
            <span>📦</span>
            <input
              type="text"
              placeholder="离线包名称"
              onInput={(e) => setOfflineName(e.currentTarget.value)}
              class="rounded bg-slate-800 px-2 py-1 text-white w-40 focus:outline-none focus:ring-1 focus:ring-cyan-500"
            />
            <button
              onClick={() => exportOfflinePack()}
              class="px-2 py-1 rounded bg-emerald-600 hover:bg-emerald-500 text-white"
              disabled={!offlineName()}
            >
              📦 导出离线包
            </button>
            <button
              onClick={() => listOfflinePacks()}
              class="px-2 py-1 rounded bg-slate-600 hover:bg-slate-500 text-white"
            >
              📋 列表
            </button>
          </div>
        </div>
      )}
      <Show when={hovered()}>
        <div class="absolute bottom-3 left-3 rounded bg-slate-900/80 px-3 py-1.5 text-xs text-slate-200 backdrop-blur">
          {isMirage(hovered()!) ? '✨ ' : ''}{hovered()?.city || hovered()?.tags} · {hovered()?.country}
        </div>
      </Show>
    </div>
  )
}

function clsx(...parts: (string | false | undefined)[]): string {
  return parts.filter(Boolean).join(' ')
}