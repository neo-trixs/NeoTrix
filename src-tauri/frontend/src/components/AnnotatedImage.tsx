import { createSignal, onMount, onCleanup, Show, For } from 'solid-js'
import { Square, Move, Eraser, Check, X, MousePointer2 } from 'lucide-solid'
import { clsx } from 'clsx'
import { ConfirmModal, type ModalReq } from './ConfirmModal'

export interface Annotation {
  id: number
  kind: 'box' | 'arrow'
  /** Normalized coordinates in [0,1] relative to the image. */
  x: number
  y: number
  w: number
  h: number
  /** Arrow end point (normalized). */
  ex?: number
  ey?: number
}

interface Props {
  imageUrl: string
  imageName: string
  onConfirm: (annotations: Annotation[]) => void
  onCancel: () => void
}

/**
 * Canvas-overlay annotation editor for images. Lets the user box or arrow
 * regions of a screenshot/image, then returns normalized coordinates that the
 * composer turns into a text hint the model can reason about.
 */
export function AnnotatedImage(props: Props) {
  const [canvas, setCanvas] = createSignal<HTMLCanvasElement | null>(null)
  const [imgEl, setImgEl] = createSignal<HTMLImageElement | null>(null)
  const [tool, setTool] = createSignal<'box' | 'arrow'>('box')
  // 统一确认模态（替换原生 confirm）
  const [modalReq, setModalReq] = createSignal<ModalReq | null>(null)
  const [annotations, setAnnotations] = createSignal<Annotation[]>([])
  // 当前拖拽形状；坐标为 normalized 值（相对图像 [0,1]）
  const [drawing, setDrawing] = createSignal<{ startX: number; startY: number; curX: number; curY: number } | null>(null)

  let imgLoaded = false

  const nextId = () =>
    annotations().reduce((m, a) => Math.max(m, a.id), 0) + 1

  const drawAll = () => {
    const c = canvas()
    const img = imgEl()
    if (!c || !img || !imgLoaded) return
    const ctx = c.getContext('2d')
    if (!ctx) return

    const dw = c.width
    const dh = c.height
    ctx.clearRect(0, 0, dw, dh)

    // Completed annotations (normalized -> pixel)
    for (const a of annotations()) {
      const px = a.x * dw
      const py = a.y * dh
      const pw = a.w * dw
      const ph = a.h * dh
      if (a.kind === 'box') {
        ctx.strokeStyle = '#6366f1'
        ctx.lineWidth = 2
        ctx.strokeRect(px, py, pw, ph)
        ctx.fillStyle = 'rgba(99, 102, 241, 0.12)'
        ctx.fillRect(px, py, pw, ph)
        // number badge — clamp to canvas top so top-edge boxes stay fully visible
        ctx.fillStyle = '#6366f1'
        const label = String(a.id)
        ctx.font = 'bold 12px monospace'
        const badgeTop = Math.max(0, py - 18)
        ctx.fillRect(px, badgeTop, ctx.measureText(label).width + 8, 16)
        ctx.fillStyle = '#fff'
        ctx.fillText(label, px + 4, badgeTop + 12)
      } else if (a.ex !== undefined && a.ey !== undefined) {
        ctx.strokeStyle = '#ef4444'
        ctx.lineWidth = 2
        ctx.beginPath()
        ctx.moveTo(px, py)
        ctx.lineTo(a.ex * dw, a.ey * dh)
        ctx.stroke()
        const ang = Math.atan2(a.ey * dh - py, a.ex * dw - px)
        ctx.beginPath()
        ctx.moveTo(a.ex * dw, a.ey * dh)
        ctx.lineTo(a.ex * dw - 12 * Math.cos(ang - 0.4), a.ey * dh - 12 * Math.sin(ang - 0.4))
        ctx.moveTo(a.ex * dw, a.ey * dh)
        ctx.lineTo(a.ex * dw - 12 * Math.cos(ang + 0.4), a.ey * dh - 12 * Math.sin(ang + 0.4))
        ctx.stroke()
      }
    }

    // In-progress drawing (normalized -> pixel), layered on top of completed ones
    const d = drawing()
    if (d) {
      const sx = d.startX * dw
      const sy = d.startY * dh
      const ex = d.curX * dw
      const ey = d.curY * dh
      if (tool() === 'box') {
        ctx.strokeStyle = '#6366f1'
        ctx.lineWidth = 2
        ctx.setLineDash([6, 4])
        ctx.strokeRect(sx, sy, ex - sx, ey - sy)
        ctx.setLineDash([])
      } else {
        ctx.strokeStyle = '#ef4444'
        ctx.lineWidth = 2
        ctx.setLineDash([6, 4])
        ctx.beginPath()
        ctx.moveTo(sx, sy)
        ctx.lineTo(ex, ey)
        ctx.stroke()
        ctx.setLineDash([])
        // arrow head
        const ang = Math.atan2(ey - sy, ex - sx)
        ctx.beginPath()
        ctx.moveTo(ex, ey)
        ctx.lineTo(ex - 12 * Math.cos(ang - 0.4), ey - 12 * Math.sin(ang - 0.4))
        ctx.moveTo(ex, ey)
        ctx.lineTo(ex - 12 * Math.cos(ang + 0.4), ey - 12 * Math.sin(ang + 0.4))
        ctx.stroke()
      }
    }
  }

  onMount(() => {
    const c = canvas()
    const img = imgEl()
    if (c && img) {
      img.onload = () => {
        imgLoaded = true
        c.width = img.naturalWidth
        c.height = img.naturalHeight
        drawAll()
      }
      if (img.complete && img.naturalWidth > 0) {
        imgLoaded = true
        c.width = img.naturalWidth
        c.height = img.naturalHeight
        drawAll()
      }
    }
  })

  onCleanup(() => {
    const img = imgEl()
    if (img) img.onload = null
  })

  const toNormalized = (clientX: number, clientY: number) => {
    const c = canvas()
    const img = imgEl()
    if (!c || !img) return { nx: 0, ny: 0 }
    const rect = c.getBoundingClientRect()
    // rect 是 CSS 显示尺寸（受 max-h-80 / max-w-full 缩放影响），
    // 归一化必须除以 CSS 尺寸而非 canvas 自然像素，否则缩放后落点偏差数倍。
    // 渲染侧（drawAll）再用 normalized × c.width/height 映射回自然像素，两条路径换算一致。
    const px = clientX - rect.left
    const py = clientY - rect.top
    return {
      nx: Math.min(1, Math.max(0, px / rect.width)),
      ny: Math.min(1, Math.max(0, py / rect.height)),
    }
  }

  const onMouseDown = (e: MouseEvent) => {
    const { nx, ny } = toNormalized(e.clientX, e.clientY)
    // 拖拽起始/结束坐标统一走 toNormalized（CSS 尺寸归一化），保证预览与最终标注一致
    setDrawing({ startX: nx, startY: ny, curX: nx, curY: ny })
  }

  const onMouseMove = (e: MouseEvent) => {
    const d = drawing()
    if (!d) return
    const { nx, ny } = toNormalized(e.clientX, e.clientY)
    setDrawing({ ...d, curX: nx, curY: ny })
    // 完整重绘：已完成标注 + 当前拖拽形状叠加，避免拖拽期间已完成标注闪烁消失
    drawAll()
  }

  const onMouseUp = (e: MouseEvent) => {
    const d = drawing()
    setDrawing(null)
    if (!d) return
    const { nx, ny } = toNormalized(e.clientX, e.clientY)
    const startNx = d.startX
    const startNy = d.startY

    const id = nextId()
    if (tool() === 'box') {
      const w = nx - startNx
      const h = ny - startNy
      if (Math.abs(w) > 0.005 || Math.abs(h) > 0.005) {
        setAnnotations(prev => [...prev, {
          id,
          kind: 'box',
          x: Math.min(startNx, nx),
          y: Math.min(startNy, ny),
          w: Math.abs(w),
          h: Math.abs(h),
        }])
      }
    } else {
      if (Math.hypot(nx - startNx, ny - startNy) > 0.01) {
        setAnnotations(prev => [...prev, {
          id,
          kind: 'arrow',
          x: startNx,
          y: startNy,
          w: 0,
          h: 0,
          ex: nx,
          ey: ny,
        }])
      }
    }
    drawAll()
  }

  const confirm = () => {
    if (annotations().length === 0) {
      // Allow confirm with no annotations (image itself still sent).
      props.onConfirm([])
      return
    }
    props.onConfirm(annotations())
  }

  return (
    <div class="space-y-2">
      <div class="relative inline-block">
        {/* Hidden img drives natural size; canvas overlays at same spot */}
        <div class="relative">
          <img
            ref={setImgEl}
            src={props.imageUrl}
            alt={props.imageName}
            class="max-h-80 max-w-full rounded-md select-none"
            draggable={false}
          />
          <canvas
            ref={setCanvas}
            class="absolute inset-0 w-full h-full cursor-crosshair touch-none"
            onMouseDown={onMouseDown}
            onMouseMove={onMouseMove}
            onMouseUp={onMouseUp}
            onMouseLeave={() => {
              setDrawing(null)
              drawAll()
            }}
          />
        </div>
      </div>

      {/* Toolbar */}
      <div class="flex items-center gap-2 flex-wrap">
        <button
          class={clsx(
            'flex items-center gap-2 px-3 py-2 rounded-lg text-xs transition-colors',
            tool() === 'box' ? 'bg-nt-core-500/20 text-nt-core-700' : 'text-text-muted hover:bg-bg-tertiary'
          )}
          onClick={() => setTool('box')}
          title="框选区域"
          aria-pressed={tool() === 'box'}
        >
          <Square class="w-3.5 h-3.5" /> 框选
        </button>
        <button
          class={clsx(
            'flex items-center gap-2 px-3 py-2 rounded-lg text-xs transition-colors',
            tool() === 'arrow' ? 'bg-nt-core-500/20 text-nt-core-700' : 'text-text-muted hover:bg-bg-tertiary'
          )}
          onClick={() => setTool('arrow')}
          title="箭头指向"
          aria-pressed={tool() === 'arrow'}
        >
          <Move class="w-3.5 h-3.5" /> 箭头
        </button>
        <button
          class="flex items-center gap-2 px-3 py-2 rounded-lg text-xs text-text-muted hover:bg-bg-tertiary transition-colors"
          onClick={() => {
            if (annotations().length > 0) {
              setModalReq({
                title: '清除标注',
                message: '确定清除所有标注？',
                danger: true,
                confirmLabel: '清除',
              })
              return
            }
            setAnnotations([])
            drawAll()
          }}
          title="清除所有标注"
        >
          <Eraser class="w-3.5 h-3.5" /> 清除
        </button>

        <div class="flex-1" />

        <Show when={annotations().length > 0}>
          <span class="text-xs text-text-muted">{annotations().length} 个标注</span>
        </Show>
        <button
          class="flex items-center gap-2 px-3 py-2 rounded-lg text-xs text-text-muted hover:bg-bg-tertiary transition-colors"
          onClick={props.onCancel}
          title="取消"
        >
          <X class="w-3.5 h-3.5" /> 取消
        </button>
        <button
          class="flex items-center gap-2 px-3 py-2 rounded-lg text-xs bg-nt-io-500 text-text-primary hover:bg-nt-io-600 transition-colors"
          onClick={confirm}
          title="确认标注并加入消息"
        >
          <Check class="w-3.5 h-3.5" /> 确认标注
        </button>
      </div>

      {/* Annotation list */}
      <Show when={annotations().length > 0}>
        <div class="space-y-1">
          <For each={annotations()}>
            {(a) => (
              <div class="flex items-center gap-2 text-xs text-text-muted font-mono">
                <MousePointer2 class="w-3.5 h-3.5 flex-shrink-0" />
                <span>
                  {a.kind === 'box'
                    ? `#${a.id} 矩形 @ (${(a.x * 100).toFixed(1)}%, ${(a.y * 100).toFixed(1)}%) ${(a.w * 100).toFixed(1)}×${(a.h * 100).toFixed(1)}%`
                    : `#${a.id} 箭头 @ (${(a.x * 100).toFixed(1)}%, ${(a.y * 100).toFixed(1)}%) → (${((a.ex ?? 0) * 100).toFixed(1)}%, ${((a.ey ?? 0) * 100).toFixed(1)}%)`}
                </span>
                <button
                  class="ml-auto p-1 rounded text-text-muted hover:text-red-500"
                  onClick={() => {
                    setAnnotations(prev => prev.filter(x => x.id !== a.id))
                    drawAll()
                  }}
                  aria-label="删除标注"
                >
                  <X class="w-3.5 h-3.5" />
                </button>
              </div>
            )}
          </For>
        </div>
      </Show>

      <ConfirmModal
        req={modalReq()}
        onConfirm={() => {
          setAnnotations([])
          setModalReq(null)
          drawAll()
        }}
        onClose={() => setModalReq(null)}
      />
    </div>
  )
}
