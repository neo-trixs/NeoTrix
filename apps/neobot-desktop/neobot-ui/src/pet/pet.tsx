/**
 * 桌宠页 —— **自研**（非上游移植）。
 *
 * # 为什么自己写而不复用上游 `src/pet/`
 *
 * 商用已确认 ⇒ 上游 `deepseek-harness-desktop` 的「No Commercial Secondary
 * Development」附加条款生效，其桌宠页源码同样不可直接用于交付。
 * 这里只依据**本仓已注册的命令**（`get_pet_status` / `get_pet_asset` /
 * `move_pet_window` / `set_pet_ignore_cursor_events`）自行实现。
 *
 * # 与上游的差别（刻意）
 *
 * - 上游有行走/待机/拖动等多套动作池与手势方向判定；这里只做**逐帧循环 + 拖动**，
 *   因为自研壳的精灵图来源是用户导入的包，动作池数量没有事实依据。
 * - ⛔ 读不到资产时**显示一行提示**而不是留一块透明窗口：透明窗 + 无内容在
 *   macOS 上看起来像「桌宠没启动」，而实际是「这个宠物包读不出来」。
 */

import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useRef, useState } from 'react'
import './pet.css'

interface PetStatus {
  enabled: boolean
  visible: boolean
  active_pet: string
  pet_size: number | null
}

/** `get_pet_asset` 返回值（`PetAsset`，见 `nt_pet.rs`）。 */
interface PetAsset {
  id: string
  /** data URL，可直接当背景图。 */
  spritesheet: string
  columns: number
  rows: number
}

/** 每帧停留时长（ms）。⛔ 不做成配置项：没有依据的旋钮就是负担。 */
const FRAME_MS = 140

export function Pet() {
  const [status, setStatus] = useState<PetStatus | null>(null)
  const [asset, setAsset] = useState<PetAsset | null>(null)
  const [err, setErr] = useState('')
  const [frame, setFrame] = useState(0)
  const [dragging, setDragging] = useState(false)
  const dragRef = useRef<{ x: number, y: number } | null>(null)
  const hostRef = useRef<HTMLDivElement | null>(null)

  // 状态：挂载读一次 + 监听窗口显隐（用户可能在设置里开关桌宠）。
  useEffect(() => {
    let alive = true
    const read = () => {
      void invoke<PetStatus>('get_pet_status')
        .then((s) => alive && setStatus(s))
        .catch((e) => alive && setErr(String(e).slice(0, 120)))
    }
    read()
    const timer = window.setInterval(read, 1500)
    return () => {
      alive = false
      window.clearInterval(timer)
    }
  }, [])

  // 资产：跟着 active_pet 换。
  useEffect(() => {
    const id = status?.active_pet?.trim()
    if (!status?.enabled || !id) {
      setAsset(null)
      return
    }
    let alive = true
    void invoke<PetAsset>('get_pet_asset', { id })
      .then((a) => {
        if (!alive) return
        setAsset(a)
        setErr('')
      })
      .catch((e) => {
        if (!alive) return
        setAsset(null)
        setErr(`读不到宠物包「${id}」：${String(e).slice(0, 100)}`)
      })
    return () => {
      alive = false
    }
  }, [status?.enabled, status?.active_pet])

  // 逐帧循环。⛔ 拖动时停播：拖动中换帧会让「按住哪一帧」变成随机事件。
  useEffect(() => {
    if (!asset || dragging || !status?.visible) return
    const rows = Math.max(1, asset.rows)
    const t = window.setInterval(() => setFrame(f => (f + 1) % rows), FRAME_MS)
    return () => window.clearInterval(t)
  }, [asset, dragging, status?.visible])

  // 拖动：相对增量交给后端（窗口是 decorations(false)，网页侧拖不动它）。
  const onPointerDown = useCallback((e: React.PointerEvent) => {
    dragRef.current = { x: e.clientX, y: e.clientY }
    setDragging(true)
    ;(e.target as HTMLElement).setPointerCapture?.(e.pointerId)
  }, [])

  const onPointerMove = useCallback((e: React.PointerEvent) => {
    const from = dragRef.current
    if (!from) return
    const dx = Math.round(e.clientX - from.x)
    const dy = Math.round(e.clientY - from.y)
    if (dx === 0 && dy === 0) return
    dragRef.current = { x: e.clientX, y: e.clientY }
    void invoke('move_pet_window', { deltaX: dx, deltaY: dy }).catch(() => {})
  }, [])

  const onPointerUp = useCallback((e: React.PointerEvent) => {
    if (!dragRef.current) return
    dragRef.current = null
    setDragging(false)
    ;(e.target as HTMLElement).releasePointerCapture?.(e.pointerId)
  }, [])

  // 穿透：不在拖动时让指针穿透到桌面，否则用户永远抓不住它；
  // 拖动中必须**不**穿透，否则 move_pet_window 收不到事件。
  const setIgnore = useCallback((ignore: boolean) => {
    void invoke('set_pet_ignore_cursor_events', { ignore }).catch(() => {})
  }, [])
  useEffect(() => {
    if (!status?.visible) return
    setIgnore(false)
    return () => setIgnore(true)
  }, [status?.visible, setIgnore])

  const hidden = !status?.enabled || !status.visible
  const col = asset ? frame % Math.max(1, asset.columns) : 0
  const row = asset ? Math.floor(frame / Math.max(1, asset.columns)) % Math.max(1, asset.rows) : 0

  return (
    <div
      ref={hostRef}
      className="nb-pet-root"
      style={{ cursor: dragging ? 'grabbing' : 'grab' }}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
    >
      {hidden ? null : asset ? (
        <div
          className="nb-pet-sprite"
          title={`${asset.id}（拖动可移动）`}
          style={{
            backgroundImage: `url(${asset.spritesheet})`,
            // 精灵图是 columns×rows 网格：把图放大到容器宽高的整数倍，
            // 再用 background-position 露出其中一格。
            backgroundSize: `${asset.columns * 100}% ${asset.rows * 100}%`,
            backgroundPosition: `${(col / (asset.columns - 1 || 1)) * 100}% ${(row / (asset.rows - 1 || 1)) * 100}%`,
          }}
        />
      ) : (
        <div className="nb-pet-hint">
          {err || '正在读宠物包…'}
        </div>
      )}
    </div>
  )
}
