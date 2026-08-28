// ══════════════════════════════════════════════════════════════════════════
//  Smart Canvas — 能力网自进化路线 (Capability Evolution Route)
//  把画板能力网接入 NeoTrix 自进化：搜索/发现 → 使用遥测 → 成熟度阶梯
//  (C0–C5, 对齐 NeoTrix Constellations) → 标记停滞能力(Dark Forest 回收候选)
//  → 落盘 KB kv_store，使 SEAL / ConsciousnessTree 可读取此进化轨迹。
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, createRoot, createEffect } from 'solid-js'
import { kbKvGet, kbKvSet, canvasSyncCapabilities } from '../api/neocodex'
import { listCapabilities } from './nodeRegistry'

const NS = 'canvas_evo'
const KEY = 'route'

export interface CapabilityTelemetry {
  kind: string
  label: string
  spawnCount: number
  firstSeen: number
  lastUsed: number
  /** 是否由用户经 search 面板显式注册（自定义能力，可被回收） */
  userAdded: boolean
}

/** NeoTrix Constellations 对齐的成熟度阶梯 */
export type Stage = 'C0' | 'C1' | 'C2' | 'C3' | 'C4' | 'C5'

function stageNumber(s: Stage): number {
  return { C0: 0, C1: 1, C2: 2, C3: 3, C4: 4, C5: 5 }[s]
}

/** 已并入 NeoTrix 能力树的同步状态（进化路线覆盖层展示）。 */
export const [treeStatus, setTreeStatus] = createSignal<{
  canvasNodes: number
  deprecated: number
  cycle: string
  matured: number
  plans: { action: string; nodeId: string; rationale: string }[]
  canonical: Record<string, { constellation: string; deprecated: boolean }>
} | null>(null)

/**
 * 把画板能力网并入 NeoTrix 能力树 (KB kv_store `capability_tree`)。
 * 每个能力 → `canvas::<kind>` 能力节点 (Domain::Io)：
 *   - 新发现 → Budding (C0 起步)
 *   - 已存在 → Strengthen (按使用量晋升 C0–C5)
 *   - 用户自定义且 0 使用 → Dark Forest 回收 (deprecated)
 * 使 SEAL / ConsciousnessTree 能通过同一棵能力树蒸馏 / 优化画板能力网。
 */
async function syncToCapabilityTree(): Promise<void> {
  try {
    const route = evolutionRoute()
    const payload = route.map((c) => ({
      kind: c.kind,
      label: c.label,
      stage: stageNumber(c.stage),
      usage: c.count,
      user_added: c.userAdded,
    }))
    const res = await canvasSyncCapabilities(payload)
    const canonical: Record<string, { constellation: string; deprecated: boolean }> = {}
    for (const c of res.canonical) canonical[c.kind] = { constellation: c.constellation, deprecated: c.deprecated }
    setTreeStatus({
      canvasNodes: res.nodes_synced,
      deprecated: res.deprecated,
      cycle: res.tree_cycle,
      matured: res.matured,
      plans: res.plans.map((p) => ({ action: p.action, nodeId: p.node_id, rationale: p.rationale })),
      canonical,
    })
  } catch (e) {
    console.warn('[evo] sync to capability tree failed:', e)
  }
}

export function stageOf(count: number): { stage: Stage; label: string } {
  if (count <= 0) return { stage: 'C0', label: 'C0 注册' }
  if (count < 5) return { stage: 'C1', label: 'C1 单用' }
  if (count < 20) return { stage: 'C2', label: 'C2 复用' }
  if (count < 50) return { stage: 'C3', label: 'C3 常用' }
  if (count < 200) return { stage: 'C4', label: 'C4 集成' }
  return { stage: 'C5', label: 'C5 自适应' }
}

const [telemetry, setTelemetry] = createSignal<Record<string, CapabilityTelemetry>>({})

function ensure(kind: string, label: string, userAdded = false): CapabilityTelemetry {
  const t = telemetry()
  const cur = t[kind]
  if (cur) return cur
  const fresh: CapabilityTelemetry = {
    kind, label, spawnCount: 0,
    firstSeen: Date.now(), lastUsed: Date.now(), userAdded,
  }
  setTelemetry({ ...t, [kind]: fresh })
  return fresh
}

/** 每次 spawn 记录遥测（store.spawn 调用）。 */
export function recordSpawn(kind: string, label: string, userAdded = false): void {
  const t = { ...telemetry() }
  const cur = t[kind] ?? ensure(kind, label, userAdded)
  t[kind] = { ...cur, label: cur.label || label, spawnCount: cur.spawnCount + 1, lastUsed: Date.now() }
  setTelemetry(t)
}

/** 标记某能力为用户经 search 显式注册（可被 Dark Forest 回收）。 */
export function markUserAdded(kind: string, label: string): void {
  const cur = ensure(kind, label, true)
  if (!cur.userAdded) setTelemetry({ ...telemetry(), [kind]: { ...cur, userAdded: true } })
}

/**
 * Dark Forest 回收候选：已注册但从未被使用的"用户自定义"能力。
 * 内建能力常驻（即使 0 使用），仅用户添加且 0 使用的可被回收。
 */
export function pruneCandidates(): string[] {
  return Object.values(telemetry())
    .filter((c) => c.userAdded && c.spawnCount === 0)
    .map((c) => c.kind)
}

/** 当前能力网快照（含成熟度），供进化路线视图 / SEAL 读取。 */
export function evolutionRoute(): { kind: string; label: string; stage: Stage; count: number; userAdded: boolean }[] {
  return listCapabilities()
    .map((c) => {
      const t = telemetry()[c.kind] ?? { spawnCount: 0, userAdded: false, label: c.label }
      const { stage } = stageOf(t.spawnCount)
      return { kind: c.kind, label: c.label, stage, count: t.spawnCount, userAdded: t.userAdded }
    })
    .sort((a, b) => b.count - a.count)
}

let saveTimer: ReturnType<typeof setTimeout> | undefined

/** 启动进化路线（幂等）：加载 + 防抖落盘 KB。 */
export function initCanvasEvolution(): void {
  kbKvGet(NS, KEY)
    .then((raw) => {
      if (!raw) return
      try {
        const parsed = JSON.parse(raw) as Record<string, CapabilityTelemetry>
        if (parsed && typeof parsed === 'object') setTelemetry(parsed)
      } catch { /* 损坏忽略 */ }
    })
    .catch(() => {})

  createRoot(() => {
    createEffect(() => {
      const snap = JSON.stringify(telemetry())
      if (saveTimer) clearTimeout(saveTimer)
      saveTimer = setTimeout(() => {
        kbKvSet(NS, KEY, snap).catch(() => {})
        syncToCapabilityTree()
      }, 1000)
    })
  })
}
