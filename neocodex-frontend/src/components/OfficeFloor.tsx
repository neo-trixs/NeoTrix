/**
 * OfficeFloor — 2D top-down agent workspace visualization
 *
 * Each agent is a "desk" with avatar, status indicator, and activity ring.
 * Active communication shown as animated particles between desks.
 * Inspired by Munder Difflin office floor (Pixi.js) but using lightweight SVG.
 */
import { createSignal, For, Show, onMount, onCleanup, createEffect } from 'solid-js'
import { clsx } from 'clsx'
import { getFloorState } from '../api/hive'
import type { FloorState } from '../api/hive'

// ─── Types ──────────────────────────────────────────────────────────────────

export interface FloorAgent {
  id: string
  name: string
  avatar: string
  status: 'idle' | 'running' | 'busy' | 'error'
  specialty: string
  x: number  // 0-1 normalized position
  y: number  // 0-1 normalized position
}

export interface FloorMessage {
  from: string
  to: string
  type: 'task' | 'result' | 'query' | 'escalation'
  timestamp: number
}

export interface OfficeFloorProps {
  agents: () => FloorAgent[]
  messages: () => FloorMessage[]
  onSelectAgent?: (id: string) => void
  width?: number
  height?: number
  /** When true, fetch real agent data from HiveRouter backend */
  useRealData?: boolean
}

// ─── Status Colors ──────────────────────────────────────────────────────────

const STATUS_RING: Record<FloorAgent['status'], string> = {
  idle: '#6b7280',
  running: '#10b981',
  busy: '#f59e0b',
  error: '#ef4444',
}

const STATUS_GLOW: Record<FloorAgent['status'], string> = {
  idle: 'none',
  running: 'drop-shadow(0 0 6px rgba(16, 185, 129, 0.6))',
  busy: 'drop-shadow(0 0 6px rgba(245, 158, 11, 0.6))',
  error: 'drop-shadow(0 0 6px rgba(239, 68, 68, 0.6))',
}

const MSG_COLOR: Record<FloorMessage['type'], string> = {
  task: '#3b82f6',
  result: '#10b981',
  query: '#8b5cf6',
  escalation: '#ef4444',
}

// ─── Component ──────────────────────────────────────────────────────────────

export function OfficeFloor(props: OfficeFloorProps) {
  const w = () => props.width ?? 480
  const h = () => props.height ?? 360
  const [hoveredAgent, setHoveredAgent] = createSignal<string | null>(null)
  const [selectedAgent, setSelectedAgent] = createSignal<string | null>(null)
  const [animated, setAnimated] = createSignal(false)
  const [tick, setTick] = createSignal(0)
  const [realData, setRealData] = createSignal<FloorState | null>(null)

  // Animation tick for particles
  let animFrame: number | undefined
  let containerRef!: HTMLDivElement
  onMount(() => {
    const animate = () => {
      setTick((t) => t + 1)
      animFrame = requestAnimationFrame(animate)
    }
    animFrame = requestAnimationFrame(animate)

    // Trigger entrance animations
    setTimeout(() => setAnimated(true), 100)

    // Fetch real data if enabled
    if (props.useRealData) {
      getFloorState()
        .then(setRealData)
        .catch(() => {/* fallback to props */})
    }

    // Click outside to close detail panel
    const handleClickOutside = (e: MouseEvent) => {
      if (selectedAgent() && containerRef && !containerRef.contains(e.target as Node)) {
        setSelectedAgent(null)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    onCleanup(() => document.removeEventListener('mousedown', handleClickOutside))
  })
  onCleanup(() => { if (animFrame) cancelAnimationFrame(animFrame) })

  // Use real data when available, fall back to props
  const agents = () => {
    const rd = realData()
    if (rd) {
      return rd.agents.map(a => ({
        id: a.id,
        name: a.name,
        avatar: a.avatar,
        status: a.status as FloorAgent['status'],
        specialty: a.specialty,
        x: a.x,
        y: a.y,
      }))
    }
    return props.agents()
  }
  const messages = () => {
    const rd = realData()
    if (rd) {
      return rd.messages.map(m => ({
        from: m.from,
        to: m.to,
        type: m.msg_type as FloorMessage['type'],
        timestamp: m.timestamp,
      }))
    }
    return props.messages()
  }

  // Convert normalized coords to SVG coords
  const toSvg = (nx: number, ny: number) => ({
    x: 40 + nx * (w() - 80),
    y: 40 + ny * (h() - 80),
  })

  // Get agent position by id
  const agentPos = (id: string) => {
    const agent = agents().find((a) => a.id === id)
    return agent ? toSvg(agent.x, agent.y) : { x: 0, y: 0 }
  }

  return (
    <div class="office-floor" ref={containerRef}>
      <svg
        width={w()}
        height={h()}
        viewBox={`0 0 ${w()} ${h()}`}
        class="office-floor-svg"
      >
        {/* Background grid */}
        <defs>
          <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
            <path d="M 20 0 L 0 0 0 20" fill="none" stroke="currentColor" stroke-width="0.3" opacity="0.15" />
          </pattern>
          <radialGradient id="desk-gradient" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stop-color="rgba(255,255,255,0.08)" />
            <stop offset="100%" stop-color="rgba(255,255,255,0)" />
          </radialGradient>
        </defs>
        <rect width="100%" height="100%" fill="url(#grid)" />

        {/* Floor label */}
        <text x={w() / 2} y="20" text-anchor="middle" fill="currentColor" opacity="0.3" font-size="10" font-weight="500" letter-spacing="2">
          OFFICE FLOOR
        </text>

        {/* Communication lines (animated particles) */}
        <For each={messages()}>
          {(msg) => {
            const from = agentPos(msg.from)
            const to = agentPos(msg.to)
            const dx = to.x - from.x
            const dy = to.y - from.y
            const dist = Math.sqrt(dx * dx + dy * dy)
            // Particle travels from → to over ~60 frames, loops
            const progress = ((tick() * 2 + msg.timestamp) % 60) / 60
            const px = from.x + dx * progress
            const py = from.y + dy * progress
            return (
              <>
                {/* Trail line */}
                <line
                  x1={from.x} y1={from.y} x2={to.x} y2={to.y}
                  stroke={MSG_COLOR[msg.type]}
                  stroke-width="1"
                  opacity="0.15"
                  stroke-dasharray="4 4"
                />
                {/* Particle */}
                <circle
                  cx={px} cy={py} r="3"
                  fill={MSG_COLOR[msg.type]}
                  opacity={0.6 + 0.4 * Math.sin(progress * Math.PI)}
                />
              </>
            )
          }}
        </For>

        {/* Agent desks */}
        <For each={agents()}>
          {(agent, index) => {
            const pos = toSvg(agent.x, agent.y)
            const isHovered = () => hoveredAgent() === agent.id
            const isSelected = () => selectedAgent() === agent.id
            const deskR = 32
            const agentIndex = () => agents().findIndex(a => a.id === agent.id)
            const animDelay = () => agentIndex() * 50
            return (
              <g
                class="agent-desk"
                transform={`translate(${pos.x}, ${pos.y})`}
                style={{
                  cursor: 'pointer',
                  filter: isHovered()
                    ? `brightness(1.3) drop-shadow(0 0 8px rgba(255,255,255,0.25))`
                    : STATUS_GLOW[agent.status],
                  opacity: animated() ? 1 : 0,
                  transition: `opacity 0.3s ease-out ${animDelay()}ms, filter 0.2s ease-out`,
                }}
                onMouseEnter={() => setHoveredAgent(agent.id)}
                onMouseLeave={() => setHoveredAgent(null)}
                onClick={() => {
                  setSelectedAgent(isSelected() ? null : agent.id)
                  props.onSelectAgent?.(agent.id)
                }}
              >
                {/* Desk circle (background) */}
                <circle
                  r={deskR}
                  fill={isSelected() ? 'rgba(59, 130, 246, 0.15)' : 'rgba(30, 30, 50, 0.8)'}
                  stroke={isSelected() ? '#3b82f6' : STATUS_RING[agent.status]}
                  stroke-width={isHovered() || isSelected() ? 2.5 : 1.5}
                />

                {/* Activity ring (running = animated) */}
                {agent.status === 'running' && (
                  <circle
                    r={deskR + 4}
                    fill="none"
                    stroke={STATUS_RING[agent.status]}
                    stroke-width="1"
                    stroke-dasharray="8 12"
                    opacity="0.5"
                    style={{
                      transform: `rotate(${tick() * 3}deg)`,
                      'transform-origin': '0 0',
                    }}
                  />
                )}

                {/* Avatar emoji */}
                <text
                  text-anchor="middle"
                  dominant-baseline="central"
                  font-size="18"
                  y="-2"
                >
                  {agent.avatar}
                </text>

                {/* Name label */}
                <text
                  text-anchor="middle"
                  y={deskR + 14}
                  fill="currentColor"
                  font-size="10"
                  font-weight="500"
                  opacity={isHovered() ? 1 : 0.7}
                >
                  {agent.name}
                </text>

                {/* Specialty tag (shown on hover) */}
                <Show when={isHovered()}>
                  <rect
                    x="-40" y={deskR + 20}
                    width="80" height="16"
                    rx="4"
                    fill="rgba(30, 30, 50, 0.9)"
                    stroke="currentColor"
                    stroke-width="0.5"
                    opacity="0.8"
                  />
                  <text
                    text-anchor="middle"
                    y={deskR + 30}
                    fill="currentColor"
                    font-size="8"
                    opacity="0.6"
                  >
                    {agent.specialty}
                  </text>
                </Show>

                {/* Status dot */}
                <circle
                  cx={deskR * 0.6}
                  cy={-deskR * 0.6}
                  r="4"
                  fill={STATUS_RING[agent.status]}
                  stroke="rgba(0,0,0,0.3)"
                  stroke-width="1"
                />
              </g>
            )
          }}
        </For>

        {/* Legend */}
        <g transform={`translate(12, ${h() - 60})`}>
          {(['idle', 'running', 'busy', 'error'] as const).map((status, i) => (
            <g transform={`translate(0, ${i * 14})`}>
              <circle cx="5" cy="0" r="3" fill={STATUS_RING[status]} />
              <text x="12" y="3" fill="currentColor" font-size="8" opacity="0.5">
                {status === 'idle' ? '空闲' : status === 'running' ? '运行中' : status === 'busy' ? '忙碌' : '错误'}
              </text>
            </g>
          ))}
        </g>

        {/* Message type legend */}
        <g transform={`translate(${w() - 100}, ${h() - 60})`}>
          {(['task', 'result', 'query', 'escalation'] as const).map((type, i) => (
            <g transform={`translate(0, ${i * 14})`}>
              <circle cx="5" cy="0" r="3" fill={MSG_COLOR[type]} />
              <text x="12" y="3" fill="currentColor" font-size="8" opacity="0.5">
                {type === 'task' ? '任务' : type === 'result' ? '结果' : type === 'query' ? '查询' : '升级'}
              </text>
            </g>
          ))}
        </g>
      </svg>

      {/* Agent Detail Panel (selected) */}
      <Show when={selectedAgent()}>
        {(id) => {
          const agent = () => agents().find(a => a.id === id())
          if (!agent()) return null
          const a = agent()!
          return (
            <div
              class="absolute right-0 top-0 h-full w-64 bg-bg-primary/95 backdrop-blur-sm border-l border-border-primary shadow-xl z-50 p-4 overflow-y-auto"
              style={{ animation: 'slideInRight 0.2s ease-out' }}
            >
              <div class="flex items-center justify-between mb-4">
                <h3 class="text-sm font-semibold text-text-primary">{a.name}</h3>
                <button
                  class="text-text-secondary hover:text-text-primary text-xs"
                  onClick={() => setSelectedAgent(null)}
                >
                  ✕
                </button>
              </div>
              <div class="flex items-center gap-3 mb-4">
                <span class="text-2xl">{a.avatar}</span>
                <div>
                  <div class="text-xs text-text-secondary">{a.specialty}</div>
                  <div class="flex items-center gap-1.5 mt-1">
                    <span
                      class="w-2 h-2 rounded-full"
                      style={{ background: STATUS_RING[a.status] }}
                    />
                    <span class="text-xs text-text-secondary capitalize">{a.status}</span>
                  </div>
                </div>
              </div>
              <div class="space-y-2 text-xs text-text-secondary">
                <div class="flex justify-between">
                  <span>Position</span>
                  <span class="text-text-primary">{Math.round(a.x * 100)}%, {Math.round(a.y * 100)}%</span>
                </div>
                <div class="flex justify-between">
                  <span>Status</span>
                  <span class="text-text-primary capitalize">{a.status}</span>
                </div>
              </div>
            </div>
          )
        }}
      </Show>

      {/* Tooltip (hovered, not selected) */}
      <Show when={hoveredAgent() && hoveredAgent() !== selectedAgent()}>
        {(id) => {
          const agent = () => agents().find(a => a.id === id())
          if (!agent()) return null
          const a = agent()!
          const pos = toSvg(a.x, a.y)
          return (
            <div
              class="absolute bg-bg-primary/95 backdrop-blur-sm shadow-lg rounded-lg p-3 text-sm z-40 pointer-events-none"
              style={{
                left: `${pos.x + 40}px`,
                top: `${pos.y - 20}px`,
                animation: 'fadeIn 0.15s ease-out',
              }}
            >
              <div class="font-medium text-text-primary">{a.name}</div>
              <div class="text-text-secondary text-xs">{a.specialty}</div>
              <div class="flex items-center gap-1.5 mt-1">
                <span
                  class="w-1.5 h-1.5 rounded-full"
                  style={{ background: STATUS_RING[a.status] }}
                />
                <span class="text-xs text-text-secondary capitalize">{a.status}</span>
              </div>
            </div>
          )
        }}
      </Show>
    </div>
  )
}

/* ─── Animation Keyframes ──────────────────────────────────────────────────── */
const style = document.createElement('style')
style.textContent = `
@keyframes slideInRight {
  from { transform: translateX(100%); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}
@keyframes slideIn {
  from { opacity: 0; transform: translateY(-8px); }
  to { opacity: 1; transform: translateY(0); }
}
`
if (!document.head.querySelector('#office-floor-styles')) {
  style.id = 'office-floor-styles'
  document.head.appendChild(style)
}
