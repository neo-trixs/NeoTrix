// ══════════════════════════════════════════════════════════════════════════
//  SproutIcon v3 — 3D 螺旋嫩芽 · 无色水 · 液态玻璃
//  技术：Archimedean 螺旋 + Phyllotaxis 叶序 + 3D 光照 + 水滴折射
// ══════════════════════════════════════════════════════════════════════════
import { onMount, createSignal } from 'solid-js'

interface SproutIconProps {
  size?: number
  onRender?: (canvas: HTMLCanvasElement) => void
}

export function SproutIcon(props: SproutIconProps) {
  const size = () => props.size ?? 512
  let canvasRef: HTMLCanvasElement | undefined
  const [ready, setReady] = createSignal(false)

  onMount(() => {
    if (!canvasRef) return
    const ctx = canvasRef.getContext('2d')!
    const s = size()
    const cx = s * 0.5, cy = s * 0.5

    // ── 背景：无色水的质感（极淡青白 + 折射波纹）──
    const bgGrad = ctx.createRadialGradient(cx, cy * 0.9, 0, cx, cy, s * 0.72)
    bgGrad.addColorStop(0, '#f8fffe')       // 中心：近乎纯白
    bgGrad.addColorStop(0.3, '#f0f8f5')     // 极淡青
    bgGrad.addColorStop(0.6, '#e8f4f0')     // 淡水色
    bgGrad.addColorStop(1, '#e0f0ea')       // 边缘微青

    ctx.beginPath()
    ctx.roundRect(0, 0, s, s, s * 0.225)
    ctx.fillStyle = bgGrad
    ctx.fill()

    // ── 水波纹：同心圆，无色透明 ──
    for (let i = 0; i < 6; i++) {
      const r = s * (0.12 + i * 0.07)
      ctx.beginPath()
      ctx.arc(cx, cy * 0.92, r, 0, Math.PI * 2)
      ctx.strokeStyle = `rgba(200,230,220,${0.15 - i * 0.02})`
      ctx.lineWidth = 1.2 - i * 0.15
      ctx.stroke()
    }

    // ── E8 几何微底层 ──
    ctx.save()
    ctx.globalAlpha = 0.03
    ctx.strokeStyle = '#3a9a6a'
    ctx.lineWidth = 0.5
    for (let i = 0; i < 8; i++) {
      const a1 = (Math.PI * 2 * i) / 8
      const a2 = (Math.PI * 2 * ((i + 1) % 8)) / 8
      ctx.beginPath()
      ctx.moveTo(cx + Math.cos(a1) * s * 0.36, cy + Math.sin(a1) * s * 0.36)
      ctx.lineTo(cx + Math.cos(a2) * s * 0.36, cy + Math.sin(a2) * s * 0.36)
      ctx.stroke()
    }
    ctx.restore()

    // ═══ 3D 螺旋茎：Archimedean 螺旋 + 3D 光照 ═══
    const stemBot = s * 0.80
    const stemTop = s * 0.18
    const stemH = stemBot - stemTop
    const spiralTurns = 2.5        // 螺旋圈数
    const spiralRadius = s * 0.06  // 螺旋半径
    const lightDir = { x: -0.4, y: -0.6 } // 光照方向（左上）

    // 绘制螺旋茎：逐段绘制，模拟3D
    const stemSegments = 200
    for (let i = 0; i < stemSegments; i++) {
      const t = i / stemSegments
      const angle = t * spiralTurns * Math.PI * 2
      const y = stemBot - t * stemH
      // 螺旋偏移
      const sx = cx + Math.sin(angle) * spiralRadius * (1 - t * 0.3) // 顶部收窄
      // 3D 深度：cos 决定前后
      const depth = Math.cos(angle)
      // 光照：基于法线和光照方向
      const nx = Math.sin(angle)
      const ny = 0.3
      const light = Math.max(0, nx * lightDir.x + ny * lightDir.y) * 0.3 + 0.7
      // 颜色：底部深翠 → 顶部嫩绿
      const r = Math.round(60 + t * 100)
      const g = Math.round(150 + t * 80)
      const b = Math.round(80 + t * 60)
      const a = 0.7 + depth * 0.15

      ctx.beginPath()
      ctx.arc(sx, y, s * 0.012 + depth * s * 0.003, 0, Math.PI * 2)
      ctx.fillStyle = `rgba(${r},${g},${b},${a})`
      ctx.fill()
    }

    // 茎高光线
    ctx.beginPath()
    for (let i = 0; i < stemSegments; i++) {
      const t = i / stemSegments
      const angle = t * spiralTurns * Math.PI * 2
      const y = stemBot - t * stemH
      const sx = cx + Math.sin(angle) * spiralRadius * (1 - t * 0.3)
      const depth = Math.cos(angle)
      if (depth > 0.3) {
        const hx = sx + 1
        if (i === 0 || Math.cos(((i - 1) / stemSegments) * spiralTurns * Math.PI * 2) <= 0.3) {
          ctx.moveTo(hx, y)
        } else {
          ctx.lineTo(hx, y)
        }
      }
    }
    ctx.strokeStyle = 'rgba(255,255,255,0.35)'
    ctx.lineWidth = 1.5
    ctx.lineCap = 'round'
    ctx.stroke()

    // ═══ Phyllotaxis 叶序：黄金角 137.5° 排布叶片 ═══
    const goldenAngle = 137.508 * Math.PI / 180
    const leafCount = 5

    for (let i = 0; i < leafCount; i++) {
      const t = 0.3 + (i / leafCount) * 0.45  // 叶片纵向分布
      const angle = i * goldenAngle
      const y = stemBot - t * stemH
      const stemX = cx + Math.sin(t * spiralTurns * Math.PI * 2) * spiralRadius * (1 - t * 0.3)

      // 叶片大小：中间最大
      const leafSize = s * (0.08 + 0.06 * Math.sin(Math.PI * t))
      // 叶片方向：左右交替 + 螺旋角度
      const side = i % 2 === 0 ? -1 : 1
      const leafAngle = side * (0.4 + t * 0.3)

      ctx.save()
      ctx.translate(stemX, y)
      ctx.rotate(leafAngle)

      // 叶片：3D 弯曲感（贝塞尔曲线）
      ctx.beginPath()
      ctx.moveTo(0, 0)
      ctx.bezierCurveTo(
        side * leafSize * 0.8, -leafSize * 0.3,
        side * leafSize * 1.1, leafSize * 0.1,
        side * leafSize * 0.2, leafSize * 0.15
      )
      ctx.closePath()

      // 叶片渐变：3D 光照
      const leafGrad = ctx.createLinearGradient(0, -leafSize * 0.3, side * leafSize, leafSize * 0.15)
      leafGrad.addColorStop(0, `rgba(180,240,160,${0.85 - t * 0.1})`)
      leafGrad.addColorStop(0.5, `rgba(120,210,120,${0.75 - t * 0.1})`)
      leafGrad.addColorStop(1, `rgba(80,170,90,${0.65 - t * 0.1})`)
      ctx.fillStyle = leafGrad
      ctx.fill()

      // 叶片高光
      ctx.beginPath()
      ctx.moveTo(side * leafSize * 0.1, -leafSize * 0.05)
      ctx.quadraticCurveTo(
        side * leafSize * 0.5, -leafSize * 0.15,
        side * leafSize * 0.8, -leafSize * 0.05
      )
      ctx.strokeStyle = 'rgba(255,255,255,0.45)'
      ctx.lineWidth = 1.5
      ctx.lineCap = 'round'
      ctx.stroke()

      ctx.restore()
    }

    // ═══ 顶端嫩芽：3D 螺旋收束 ═══
    const budR = s * 0.022
    const budGrad = ctx.createRadialGradient(cx - 2, stemTop - 2, 0, cx, stemTop, budR)
    budGrad.addColorStop(0, 'rgba(220,255,230,0.95)')
    budGrad.addColorStop(0.5, 'rgba(160,230,180,0.85)')
    budGrad.addColorStop(1, 'rgba(100,200,140,0.7)')
    ctx.beginPath()
    ctx.arc(cx, stemTop, budR, 0, Math.PI * 2)
    ctx.fillStyle = budGrad
    ctx.fill()
    // 芽尖高光
    ctx.beginPath()
    ctx.arc(cx - budR * 0.3, stemTop - budR * 0.3, budR * 0.35, 0, Math.PI * 2)
    ctx.fillStyle = 'rgba(255,255,255,0.85)'
    ctx.fill()

    // ═══ 无色水滴：折射 + 焦散 ═══
    const drawWaterDrop = (dx: number, dy: number, dr: number, refraction = 1.0) => {
      // 水滴本体：近乎无色，只有微弱青调
      const dropGrad = ctx.createRadialGradient(
        dx - dr * 0.3, dy - dr * 0.3, 0,
        dx, dy, dr
      )
      dropGrad.addColorStop(0, 'rgba(255,255,255,0.95)')
      dropGrad.addColorStop(0.3, 'rgba(240,252,248,0.7)')
      dropGrad.addColorStop(0.7, 'rgba(220,245,240,0.4)')
      dropGrad.addColorStop(1, 'rgba(200,235,230,0.15)')
      ctx.beginPath()
      ctx.arc(dx, dy, dr, 0, Math.PI * 2)
      ctx.fillStyle = dropGrad
      ctx.fill()

      // 折射高光
      ctx.beginPath()
      ctx.arc(dx - dr * 0.25, dy - dr * 0.25, dr * 0.2, 0, Math.PI * 2)
      ctx.fillStyle = 'rgba(255,255,255,0.9)'
      ctx.fill()

      // 焦散光环
      ctx.beginPath()
      ctx.arc(dx, dy, dr * 1.3, 0, Math.PI * 2)
      ctx.strokeStyle = `rgba(200,240,235,${0.15 * refraction})`
      ctx.lineWidth = 0.8
      ctx.stroke()
    }

    // 茎上水滴
    drawWaterDrop(cx + s * 0.05, s * 0.50, s * 0.014, 1.2)
    drawWaterDrop(cx - s * 0.08, s * 0.62, s * 0.010, 1.0)
    drawWaterDrop(cx + s * 0.03, s * 0.38, s * 0.008, 0.9)

    // 叶面水珠
    drawWaterDrop(cx - s * 0.14, s * 0.48, s * 0.007, 0.8)
    drawWaterDrop(cx + s * 0.16, s * 0.42, s * 0.006, 0.7)

    // ── 水面反射弧 ──
    ctx.beginPath()
    ctx.arc(cx, stemBot + s * 0.04, s * 0.15, Math.PI * 1.1, Math.PI * 1.9)
    ctx.strokeStyle = 'rgba(200,235,228,0.3)'
    ctx.lineWidth = s * 0.005
    ctx.lineCap = 'round'
    ctx.stroke()

    // ── 微光气泡（无色）──
    const bubbles = [
      { x: cx - s * 0.20, y: s * 0.32, r: 2.5 },
      { x: cx + s * 0.22, y: s * 0.68, r: 2 },
      { x: cx - s * 0.16, y: s * 0.74, r: 1.8 },
      { x: cx + s * 0.18, y: s * 0.26, r: 2.2 },
      { x: cx - s * 0.24, y: s * 0.52, r: 1.5 },
    ]
    for (const b of bubbles) {
      ctx.beginPath()
      ctx.arc(b.x, b.y, b.r, 0, Math.PI * 2)
      ctx.strokeStyle = 'rgba(200,235,228,0.25)'
      ctx.lineWidth = 0.6
      ctx.stroke()
      // 气泡高光
      ctx.beginPath()
      ctx.arc(b.x - b.r * 0.2, b.y - b.r * 0.2, b.r * 0.25, 0, Math.PI * 2)
      ctx.fillStyle = 'rgba(255,255,255,0.6)'
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
