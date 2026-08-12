import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@solidjs/testing-library'
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

describe('GlobeView B2 usePack 数据源切换', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // kb_geo_layers 分层摘要返回空 → 预算 = max(200, limit)
    invokeMock.mockResolvedValue([])
  })

  it('usePack 默认 (false) 时 7 路地理点走 SQLite kb_geo_points，海拔走 kb_geo_elevations', async () => {
    // kb_geo_points 返回单点
    invokeMock
      .mockResolvedValueOnce([]) // kb_geo_layers
      .mockResolvedValueOnce([mkPoint({ node_id: 'c0' })]) // kb_geo_points cities
    render(() => <GlobeView limit={2000} />)
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('kb_geo_layers', {})
    })
    // 8 路：7 点通道 + 1 海拔通道
    await waitFor(() => {
      const geoPointsCalls = invokeMock.mock.calls.filter(([c]) => c === 'kb_geo_points')
      const elevCalls = invokeMock.mock.calls.filter(([c]) => c === 'kb_geo_elevations')
      const packCalls = invokeMock.mock.calls.filter(([c]) => c === 'kb_geo_points_pack')
      expect(geoPointsCalls.length).toBe(7)
      expect(elevCalls.length).toBe(1)
      expect(packCalls.length).toBe(0)
      // 城市预算 max(200, 2000-0)=2000
      expect(geoPointsCalls[0][1]).toEqual({ limit: 2000, source: null })
      // 海拔恒 4000
      expect(elevCalls[0][1]).toEqual({ limit: 4000 })
    })
  })

  it('usePack=true 时 7 路地理点切到 NT-Pack kb_geo_points_pack，海拔仍 SQLite', async () => {
    invokeMock.mockResolvedValue([mkPoint({ node_id: 'p0' })])
    render(() => <GlobeView limit={2000} usePack />)
    await waitFor(() => {
      const packCalls = invokeMock.mock.calls.filter(([c]) => c === 'kb_geo_points_pack')
      const geoPointsCalls = invokeMock.mock.calls.filter(([c]) => c === 'kb_geo_points')
      const elevCalls = invokeMock.mock.calls.filter(([c]) => c === 'kb_geo_elevations')
      expect(packCalls.length).toBe(7)
      expect(elevCalls.length).toBe(1)
      expect(geoPointsCalls.length).toBe(0)
      // 契约同构：source 精确透传
      const shanhai = packCalls.find(([, a]) => (a as Record<string, unknown>).source === 'shanhai')
      expect(shanhai?.[1]).toEqual({ limit: 5000, source: 'shanhai' })
    })
  })
})
