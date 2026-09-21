import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, waitFor } from '@solidjs/testing-library'
import { invoke } from '@tauri-apps/api/core'
import { GlobeView } from './GlobeView'

// 数据源契约：调用通道名称
const invokeMock = vi.mocked(invoke)

// Tauri IPC 全 mock：invoke 可断言的 vi.fn
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([]),
}))

// globe.gl 三.js 无法在 jsdom 挂载，整体 mock 为可链式 kapsule
vi.mock('globe.gl', () => {
  const chainable = new Proxy({}, {
    get: (_t, prop: string) => {
      if (prop === 'default') return undefined
      return () => chainable
    },
  })
  const Globe = vi.fn().mockImplementation(() => chainable)
  return { default: Globe }
})

// 文件点数据格式：后端 GeoPointPayload → GeoPoint
function mkPoint(overrides: Record<string, unknown> = {}) {
  return {
    node_id: 'n1', name: 'Test', source: 'geonames-cities', lat: 31.2, lon: 121.5,
    confidence: 0.9, elevation_m: 10, country: 'CN', ...overrides,
  }
}

/** geo.ts 经 domain_call 路由（kb 域）；按 action 过滤调用 */
function domainCalls(action: string) {
  return (invokeMock.mock.calls as [string, Record<string, unknown>][]).filter(
    ([c, req]) => c === 'domain_call' && (req as { action?: string }).action === action,
  )
}

function domainArgs(action: string) {
  return domainCalls(action).map(([, req]) => (req as { args?: unknown }).args)
}

describe('GlobeView B2 usePack 数据源切换', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // kb_geo_layers 分层摘要返回空 → 预算 = max(200, limit)
    invokeMock.mockResolvedValue({ ok: true, data: [] })
  })

  it('usePack 默认 (false) 时 7 路地理点走 SQLite kb_geo_points，海拔走 kb_geo_elevations', async () => {
    // kb_geo_points 返回单点
    invokeMock
      .mockResolvedValueOnce({ ok: true, data: [] }) // kb_geo_layers
      .mockResolvedValueOnce({ ok: true, data: [mkPoint({ node_id: 'c0' })] }) // kb_geo_points cities
    render(() => <GlobeView limit={2000} />)
    await waitFor(() => {
      expect(domainCalls('geo_layers').length).toBeGreaterThan(0)
    })
    // 8 路：7 点通道 + 1 海拔通道
    await waitFor(() => {
      const geoPointsCalls = domainCalls('geo_points')
      const elevCalls = domainCalls('geo_elevations')
      const packCalls = domainCalls('geo_points_pack')
      expect(geoPointsCalls.length).toBe(7)
      expect(elevCalls.length).toBe(1)
      expect(packCalls.length).toBe(0)
      // 城市预算 max(200, 2000-0)=2000
      expect(domainArgs('geo_points')[0]).toEqual({ limit: 2000, source: null })
      // 海拔恒 4000
      expect(domainArgs('geo_elevations')[0]).toEqual({ limit: 4000 })
    })
  })

  it('usePack=true 时 7 路地理点切到 NT-Pack kb_geo_points_pack，海拔仍 SQLite', async () => {
    invokeMock.mockResolvedValue({ ok: true, data: [mkPoint({ node_id: 'p0' })] })
    render(() => <GlobeView limit={2000} usePack />)
    await waitFor(() => {
      const packCalls = domainCalls('geo_points_pack')
      const geoPointsCalls = domainCalls('geo_points')
      const elevCalls = domainCalls('geo_elevations')
      expect(packCalls.length).toBe(7)
      expect(elevCalls.length).toBe(1)
      expect(geoPointsCalls.length).toBe(0)
      // 契约同构：source 精确透传
      const shanhai = domainArgs('geo_points_pack').find(
        (a) => (a as Record<string, unknown>).source === 'shanhai',
      ) as Record<string, unknown> | undefined
      expect(shanhai).toEqual({ limit: 5000, source: 'shanhai' })
    })
  })
})
