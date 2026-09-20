// ══════════════════════════════════════════════════════════════════════════
//  FlowerOfLife — 生命之花（Canvas 程序化绘制）
//  神圣几何 + 液态玻璃 + 无色水质感
// ══════════════════════════════════════════════════════════════════════════
import { onMount, createSignal } from 'solid-js'

interface FlowerOfLifeProps {
  size?: number
  onRender?: (canvas: HTMLCanvasElement) => void
}

export function FlowerOfLife(props: FlowerOfLifeProps) {
  const size = () => props.size ?? 512
  let canvasRef: HTMLCanvasElement | undefined
  const [ready, setReady] = createSignal(false)

  onMount(() => {
    if (!canvasRef) return
    const ctx = canvasRef.getContext('2d')!
    const s = size()
    const cx = s * 0.5, cy = s * 0.5

    // ── 无色水背景 ──
    const bgGrad = ctx.createRadialGradient(cx, cy * 0.9, 0, cx, cy, s * 0.72)
    bgGrad.addColorStop(0, '#f8fffe')
    bgGrad.addColorStop(0.3, '#f0f8f5')
    bgGrad.addColorStop(0.6, '#e8f4f0')
    bgGrad.addColorStop(1, '#e0f0ea')

    ctx.beginPath()
    ctx.roundRect(0, 0, s, s, s * 0.225)
    ctx.fillStyle = bgGrad
    ctx.fill()

    // ── 水波纹 ──
    for (let i = 0; i < 4; i++) {
      const r = s * (0.18 + i * 0.08)
      ctx.beginPath()
      ctx.arc(cx, cy, r, 0, Math.PI * 2)
      ctx.strokeStyle = `rgba(200,235,228,${0.08 - i * 0.015})`
      ctx.lineWidth = 0.8
      ctx.stroke()
    }

    // ── E8 微底层 ──
    ctx.save()
    ctx.globalAlpha = 0.025
    ctx.strokeStyle = '#3a7a58'
    ctx.lineWidth = 0.4
    for (let i = 0; i < 8; i++) {
      const a1 = (Math.PI * 2 * i) / 8
      const a2 = (Math.PI * 2 * ((i + 1) % 8)) / 8
      ctx.beginPath()
      ctx.moveTo(cx + Math.cos(a1) * s * 0.42, cy + Math.sin(a1) * s * 0.42)
      ctx.lineTo(cx + Math.cos(a2) * s * 0.42, cy + Math.sin(a2) * s * 0.42)
      ctx.stroke()
    }
    ctx.restore()

    // ═══ 生命之花 ═══
    const R = s * 0.117  // 圆半径
    const layers = [
      { circles: [{ x: 0, y: 0 }], opacity: 0.65, width: 2 },           // 中心
      { circles: Array.from({ length: 6 }, (_, i) => ({
        x: Math.cos(Math.PI / 3 * i - Math.PI / 2) * R,
        y: Math.sin(Math.PI / 3 * i - Math.PI / 2) * R,
      })), opacity: 0.55, width: 1.8 },
      { circles: Array.from({ length: 12 }, (_, i) => {
        const a = (Math.PI / 6) * i
        const ring = i < 6 ? 2 : 2.6
        return { x: Math.cos(a) * R * ring * 0.58, y: Math.sin(a) * R * ring * 0.58 }
      }), opacity: 0.4, width: 1.4 },
      { circles: Array.from({ length: 6 }, (_, i) => ({
        x: Math.cos((Math.PI / 3) * i) * R * 3,
        y: Math.sin((Math.PI / 3) * i) * R * 3,
      })), opacity: 0.22, width: 1 },
    ]

    for (const layer of layers) {
      for (const c of layer.circles) {
        const grad = ctx.createLinearGradient(
          cx + c.x - R, cy + c.y - R,
          cx + c.x + R, cy + c.y + R
        )
        grad.addColorStop(0, `rgba(136,200,168,${layer.opacity})`)
        grad.addColorStop(0.5, `rgba(90,154,120,${layer.opacity * 0.8})`)
        grad.addColorStop(1, `rgba(58,122,88,${layer.opacity * 0.7})`)

        ctx.beginPath()
        ctx.arc(cx + c.x, cy + c.y, R, 0, Math.PI * 2)
        ctx.strokeStyle = grad
        ctx.lineWidth = layer.width
        ctx.stroke()
      }
    }

    // ── 中心光辉 ──
    const coreGrad = ctx.createRadialGradient(cx, cy, 0, cx, cy, R * 0.8)
    coreGrad.addColorStop(0, 'rgba(212,245,224,0.4)')
    coreGrad.addColorStop(0.6, 'rgba(168,224,192,0.15)')
    coreGrad.addColorStop(1, 'rgba(128,200,160,0)')
    ctx.beginPath()
    ctx.arc(cx, cy, R * 0.8, 0, Math.PI * 2)
    ctx.fillStyle = coreGrad
    ctx.fill()

    // 中心光点
    ctx.beginPath()
    ctx.arc(cx, cy, s * 0.012, 0, Math.PI * 2)
    ctx.fillStyle = 'rgba(255,255,255,0.85)'
    ctx.fill()

    // ── 六个交汇点高光 ──
    for (let i = 0; i < 6; i++) {
      const a = (Math.PI / 3) * i - Math.PI / 2
      const px = cx + Math.cos(a) * R
      const py = cy + Math.sin(a) * R
      ctx.beginPath()
      ctx.arc(px, py, s * 0.006, 0, Math.PI * 2)
      ctx.fillStyle = 'rgba(200,240,225,0.5)'
      ctx.fill()
    }

    // ── 液态玻璃折射 ──
    ctx.beginPath()
    ctx.arc(cx, cy, R * 1.8, 0, Math.PI * 2)
    ctx.strokeStyle = 'rgba(255,255,255,0.12)'
    ctx.lineWidth = 0.6
    ctx.stroke()

    // ── 微光散点 ──
    const dots = [
      { x: s * 0.12, y: s * 0.12, r: 2 },
      { x: s * 0.88, y: s * 0.15, r: 1.8 },
      { x: s * 0.10, y: s * 0.88, r: 1.5 },
      { x: s * 0.90, y: s * 0.85, r: 1.6 },
      { x: s * 0.5, y: s * 0.06, r: 1.2 },
      { x: s * 0.06, y: s * 0.5, r: 1 },
      { x: s * 0.94, y: s * 0.5, r: 1.1 },
      { x: s * 0.5, y: s * 0.94, r: 1.3 },
    ]
    for (const d of dots) {
      ctx.beginPath()
      ctx.arc(d.x, d.y, d.r, 0, Math.PI * 2)
      ctx.fillStyle = 'rgba(180,220,200,0.2)'
      ctx.fill()
    }

    setReady(true)
    props.onRender?.(canvasRef)
  })

  return (
    <canvas
      ref={canvasRef}
      width={size()}
      height={size()}
      style={{ width: `${size()}px`, height: `${size()}px` }}
    />
  )
}
