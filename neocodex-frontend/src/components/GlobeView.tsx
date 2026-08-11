import { onMount, onCleanup, createSignal, createEffect, Show } from 'solid-js'
import Globe from 'globe.gl'
import { geoPoints, geoLayers, isMirage, type GeoPoint, type GeoLayerSummary } from '../api/geo'

/* ════════════════════════════════════════════
   GlobeView.tsx — 地球知识世界仿真 (3D 地图)
   前后端分离：数据经 api/geo.ts（Tauri IPC）拉取，本组件只负责复杂加载与渲染。

   分层渲染：
   - 真实层：world-atlas 国家边界 + geonames 城市点（青绿）
   - 幻境层：shanhai-peaks（山海经山峰，橙红）+ shanhai-mappings（山海经全球映射，琥珀）
     以扩散光环 (rings 动画) 叠加在真实地理之上，突出"幻境"质感

   复杂加载策略：
   1. 先拉分层摘要 (kb_geo_layers) → 决定各层预算
   2. 幻境层全量拉取（数量少，必须完整）
   3. 真实层按预算采样（城市点 117k，只取 top-N）
   4. 渐进渲染：先真实层后幻境层，加载状态 + 图层开关
   ════════════════════════════════════════════ */

interface GlobeViewProps {
  limit?: number
  height?: number
}

export function GlobeView(props: GlobeViewProps) {
  let containerRef: HTMLDivElement | undefined
  const [points, setPoints] = createSignal<GeoPoint[]>([])
  const [layers, setLayers] = createSignal<GeoLayerSummary[]>([])
  const [loading, setLoading] = createSignal(true)
  const [error, setError] = createSignal('')
  const [hovered, setHovered] = createSignal<GeoPoint | null>(null)
  const [showReal, setShowReal] = createSignal(true)
  const [showMirage, setShowMirage] = createSignal(true)
  // globe 实例 + 分层数据（供顶层图层切换 effect 使用）
  const [globeInst, setGlobeInst] = createSignal<InstanceType<typeof Globe> | null>(null)
  const [cityPts, setCityPts] = createSignal<GeoPoint[]>([])
  const [shanhaiPts, setShanhaiPts] = createSignal<GeoPoint[]>([])

  // 图层开关：真实层 / 幻境层 显隐（响应式，顶层 effect）
  createEffect(() => {
    const globe = globeInst()
    if (!globe) return
    const real = showReal() ? cityPts() : []
    const mirage = showMirage() ? shanhaiPts() : []
    globe.pointsData([...real, ...mirage])
    globe.ringsData(showMirage() ? shanhaiPts() : [])
  })

  onMount(async () => {
    if (!containerRef) return
    const limit = props.limit ?? 3000
    const height = props.height ?? 600

    // 1) 分层摘要 → 决定加载预算（前后端分离：后端给摘要，前端定策略）
    let cityBudget = limit
    try {
      const ls = await geoLayers()
      setLayers(ls)
      const mirageTotal = ls
        .filter((l) => isMirage(l))
        .reduce((acc, l) => acc + l.count, 0)
      cityBudget = Math.max(200, limit - mirageTotal)
    } catch (e) {
      console.warn('[GlobeView] 分层摘要加载失败，使用默认预算:', e)
    }

    // 2) 分层拉取：真实城市点（预算采样）+ 全部 shanhai 幻境点
    let cityPts: GeoPoint[] = []
    let shanhaiPts: GeoPoint[] = []
    try {
      const [cities, shanhai] = await Promise.all([
        geoPoints(cityBudget, undefined),
        geoPoints(5000, 'shanhai'),
      ])
      cityPts = cities.filter((p) => !isMirage(p))
      shanhaiPts = shanhai
      setCityPts(cityPts)
      setShanhaiPts(shanhaiPts)
      setPoints([...cityPts, ...shanhaiPts])
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }

    // 3) 初始化 3D 地球
    const globe = new Globe(containerRef, { animateIn: true })
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

    // 5) 真实层：城市知识节点光点
    globe
      .pointsData(cityPts)
      .pointLat((d: object) => (d as GeoPoint).lat)
      .pointLng((d: object) => (d as GeoPoint).lng)
      .pointColor(() => '#4ecdc4')
      .pointAltitude(0.02)
      .pointRadius(0.28)
      .pointsTransitionDuration(1200)

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

    // 幻境层大点（区别于真实层的小点）
    globe
      .pointsData([...cityPts, ...shanhaiPts])
      .pointRadius((d: object) => isMirage(d as GeoPoint) ? 0.55 : 0.28)
      .pointColor((d: object) => {
        const p = d as GeoPoint
        if (p.source === 'shanhai-peaks') return '#ff6b35'
        if (p.source === 'shanhai-mappings') return '#ffd166'
        return '#4ecdc4'
      })
      .pointLabel((d: object) => {
        const p = d as GeoPoint
        const mirage = isMirage(p)
        const mirageBadge = mirage ? `<span style="color:#ffd166">【幻境·山海经】</span><br/>` : ''
        return `<div style="font-family:system-ui;font-size:12px;color:#e6f1ff;background:rgba(10,20,40,0.92);padding:6px 10px;border-radius:6px;border:1px solid ${mirage ? 'rgba(255,209,102,0.5)' : 'rgba(90,140,200,0.4)'};max-width:260px">
          ${mirageBadge}<b>${p.city || '未名'}</b><br/>
          <span style="color:#8ab4f8">${p.country || '未知地区'}</span>
          ${p.tags ? `<br/><span style="color:#9aa7bd">${p.tags}</span>` : ''}
          <br/><span style="color:#6b7a90">${p.lat.toFixed(2)}, ${p.lng.toFixed(2)}</span>
        </div>`
      })
      .onPointHover((d: object | null) => setHovered(d ? (d as GeoPoint) : null))

    // 7) 自适应尺寸
    const onResize = () => globe.width(containerRef?.clientWidth ?? 800)
    window.addEventListener('resize', onResize)

    onCleanup(() => {
      window.removeEventListener('resize', onResize)
      globe._destructor?.()
    })
  })

  const mirageCount = () => points().filter((p) => isMirage(p)).length
  const realCount = () => points().length - mirageCount()

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
          <span class="text-amber-300">✨ {mirageCount()} 幻境</span>
          <button
            class={clsx('layer-pill', showMirage() && 'on-mirage')}
            onClick={() => setShowMirage(!showMirage())}
          >
            {showMirage() ? '● 山海' : '○ 山海'}
          </button>
        </div>
      </Show>
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