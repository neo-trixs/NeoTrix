#!/usr/bin/env node
// ui-design C4 生产接线 — 设计审计门禁 (des/ui DESIGN.md lint + 可计算性检查).
// 审计目标: docs 主题生产路径是否消费设计语言 token 系统 (单一事实源).
// 检查项:
//   1. TOKEN-DRIFT: theme CSS 无硬编码 hex (全部走 var(--*))
//   2. DESIGN 资产: tokens.css/tokens.json/design-system.md 存在
//   3. CONTRAST: 关键语义色对 WCAG AA 对比度 (ui-design Phase 5 可计算性检查)
//   4. THEME-CONSUME: style.css @import 了 /design/tokens.css
// 用法: node design-audit.mjs [--json]
// 退出码: 0 = 通过 (C4 门禁) / 1 = 违约
import { readFileSync, existsSync, statSync } from 'node:fs'
import { join, resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const THEME_CSS = join(ROOT, '.vitepress/theme/style.css')
const DESIGN_DIR = join(ROOT, 'public/design')
const HEX_RE = /\b(?:#[0-9a-fA-F]{3,8}|rgba?\([^)]*\))\b/g

function rel(p) { return p.replace(ROOT + '/', '') }

function parseTokens(cssPath) {
  const css = readFileSync(cssPath, 'utf8')
  const tokens = {}
  for (const m of css.matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{3,8}|rgba?\([^)]*\))\s*;/g)) {
    tokens[m[1]] = m[2]
  }
  return tokens
}

// 只解析 :root 块 (light 默认档) — 键值锚定用, 避免 dark 覆盖干扰
function parseRootTokens(cssPath) {
  const css = readFileSync(cssPath, 'utf8')
  const m = css.match(/:root\s*{([^}]*)}/)
  if (!m) return {}
  const tokens = {}
  for (const kv of m[1].matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{3,8}|rgba?\([^)]*\))\s*;/g)) {
    tokens[kv[1]] = kv[2]
  }
  return tokens
}

function rgbOf(color) {
  let c = color.trim()
  if (c.startsWith('#')) {
    c = c.slice(1)
    if (c.length === 3) c = c.split('').map((x) => x + x).join('')
    if (c.length === 6) {
      const n = parseInt(c, 16)
      return [(n >> 16) & 255, (n >> 8) & 255, n & 255]
    }
  }
  if (c.startsWith('rgba')) {
    const m = c.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/)
    if (m) return [Number(m[1]), Number(m[2]), Number(m[3])]
  }
  return null
}

function luminance([r, g, b]) {
  const f = (v) => {
    v /= 255
    return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4
  }
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)
}

function contrast(a, b) {
  const la = luminance(rgbOf(a)), lb = luminance(rgbOf(b))
  const [hi, lo] = la > lb ? [la, lb] : [lb, la]
  return (hi + 0.05) / (lo + 0.05)
}

const checks = []
const fail = (id, msg) => checks.push({ id, ok: false, msg })
const pass = (id, msg) => checks.push({ id, ok: true, msg })

// 1. TOKEN-DRIFT
if (existsSync(THEME_CSS)) {
  const css = readFileSync(THEME_CSS, 'utf8')
  // 只检查 :root 外的 var 消费区域 — 从 design tokens 导入后的使用处不应有裸 hex
  const body = css.replace(/:root\s*{[^}]*}/g, '')
  const hexes = body.match(HEX_RE) || []
  // 允许 var(--...) 引用, 不允许裸 hex
  const bare = hexes.filter((h) => !h.startsWith('var(') && !h.includes('('))
  if (bare.length === 0) {
    pass('TOKEN-DRIFT', `${rel(THEME_CSS)} 无裸 hex (${body.match(/var\(--[\w-]+\)/g)?.length ?? 0} 处 var() 消费)`)
  } else {
    fail('TOKEN-DRIFT', `发现 ${bare.length} 处裸 hex: ${bare.slice(0, 5).join(' ')}`)
  }
  const imported = css.includes("'/design/tokens.css'") || css.includes('"/design/tokens.css"')
  imported
    ? pass('THEME-CONSUME', `${rel(THEME_CSS)} @import 消费 /design/tokens.css`)
    : fail('THEME-CONSUME', `${rel(THEME_CSS)} 未 import /design/tokens.css`)
} else {
  fail('TOKEN-DRIFT', `theme css 缺失: ${rel(THEME_CSS)}`)
}

// 2. DESIGN 资产
for (const f of ['tokens.css', 'tokens.json', 'design-system.md']) {
  const p = join(DESIGN_DIR, f)
  existsSync(p)
    ? pass(`ASSET-${f}`, `${rel(p)} 存在`)
    : fail(`ASSET-${f}`, `${rel(p)} 缺失`)
}

// 3. CONTRAST — 关键语义色对 AA
if (existsSync(join(DESIGN_DIR, 'tokens.css'))) {
  const tokens = parseTokens(join(DESIGN_DIR, 'tokens.css'))
  const pairs = [
    ['ink-1', 'background', 4.5],      // 正文/背景
    ['ink-2', 'background', 4.5],
    ['gold-500', 'background', 3.0],   // 品牌色/背景 (大文本 AA)
    ['destructive', 'background', 4.5],
    ['ink-1', 'card', 4.5],
  ]
  for (const [fg, bg, min] of pairs) {
    const f = tokens[fg], b = tokens[bg]
    if (!f || !b) { fail('CONTRAST', `缺 token: ${fg}/${bg}`); continue }
    const r = contrast(f, b)
    r >= min
      ? pass('CONTRAST', `${fg} on ${bg} = ${r.toFixed(2)}:1 (≥${min})`)
      : fail('CONTRAST', `${fg} on ${bg} = ${r.toFixed(2)}:1 (<${min} AA 违约)`)
  }
}

// 4. TOKEN-DRIFT-CROSS (C5 自愈): tokens.json ↔ tokens.css 双向一致
//    tokens.json 是单一事实源; tokens.css 必须覆盖其全部色值 (含 dark + nucleus),
//    且 tokens.css 不得出现 tokens.json 之外的色值。
const TOKENS_JSON = join(DESIGN_DIR, 'tokens.json')
const TOKENS_CSS = join(DESIGN_DIR, 'tokens.css')
if (existsSync(TOKENS_JSON) && existsSync(TOKENS_CSS)) {
  let tok
  try { tok = JSON.parse(readFileSync(TOKENS_JSON, 'utf8')) } catch { tok = null }
  if (tok) {
    // 收集 tokens.json color + dark 段 (CSS 可消费的色), 排除 assetDerived (SVG 锚点白名单)
    const jsonVals = new Set()
    const walk = (o) => {
      if (o && typeof o === 'object') {
        if (typeof o.value === 'string' && /^#[0-9a-fA-F]{3,8}$/.test(o.value)) jsonVals.add(o.value.toUpperCase())
        for (const v of Object.values(o)) walk(v)
      }
    }
    walk(tok.color)
    walk(tok.dark)
    // 收集 tokens.css 出现的全部色值 (light + dark 两档, 不做 key 覆盖)
    const cssRaw = readFileSync(TOKENS_CSS, 'utf8')
    const cssVals = new Set()
    for (const m of cssRaw.matchAll(/#[0-9a-fA-F]{3,8}\b/g)) cssVals.add(m[0].toUpperCase())
    const cssMissing = [...jsonVals].filter((v) => !cssVals.has(v))
    const cssExtra = [...cssVals].filter((v) => !jsonVals.has(v))

    // 键→值映射校验: tokens.css 的已知 primitive/semantic 键必须等于单一事实源值
    // (防止把 token 键改成另一个合法色值 — 色集合检查抓不到的漂移)
    const cssTokenMap = parseRootTokens(TOKENS_CSS)
    const keyAnchor = {}
    const anchorMap = (pathPrefix, obj, cssKey) => {
      if (obj && typeof obj === 'object') {
        if (typeof obj.value === 'string' && /^#[0-9a-fA-F]{3,8}$/.test(obj.value)) {
          keyAnchor[cssKey] = obj.value.toUpperCase()
        }
        for (const [k, v] of Object.entries(obj)) anchorMap(pathPrefix, v, cssKey)
      }
    }
    // 显式映射: tokens.json 层级 → tokens.css 键
    const explicit = {
      'color.gold.50': 'gold-50', 'color.gold.100': 'gold-100', 'color.gold.200': 'gold-200',
      'color.gold.300': 'gold-300', 'color.gold.400': 'gold-400', 'color.gold.500': 'gold-500',
      'color.gold.600': 'gold-600', 'color.gold.700': 'gold-700', 'color.gold.800': 'gold-800',
      'color.gold.900': 'gold-900',
      'color.ink.1': 'ink-1', 'color.ink.2': 'ink-2', 'color.ink.3': 'ink-3', 'color.ink.line': 'ink-line',
      'color.nucleus.from': 'nucleus-from', 'color.nucleus.mid': 'nucleus-mid', 'color.nucleus.to': 'nucleus-to',
      'color.semantic.background': 'background', 'color.semantic.foreground': 'foreground',
      'color.semantic.card': 'card', 'color.semantic.primary': 'primary',
      'color.semantic.secondary': 'secondary', 'color.semantic.muted': 'muted',
      'color.semantic.mutedForeground': 'muted-foreground', 'color.semantic.accent': 'accent',
      'color.semantic.destructive': 'destructive', 'color.semantic.border': 'border',
      'color.semantic.ring': 'ring',
    }
    const getPath = (o, path) => path.split('.').reduce((a, k) => (a && a[k] !== undefined ? a[k] : undefined), o)
    const keyDrift = []
    for (const [tokPath, cssKey] of Object.entries(explicit)) {
      const node = getPath(tok, tokPath)
      const cssVal = cssTokenMap[cssKey]
      if (!node || !node.value || !cssVal) continue
      if (node.value.toUpperCase() !== cssVal.toUpperCase()) {
        keyDrift.push(`--${cssKey} (css=${cssVal} vs token=${node.value})`)
      }
    }
    if (cssMissing.length === 0 && cssExtra.length === 0 && keyDrift.length === 0) {
      pass('TOKEN-DRIFT-CROSS', 'tokens.json 与 tokens.css 双向一致 + 键值锚定')
    } else {
      if (cssMissing.length) fail('TOKEN-DRIFT-CROSS', `tokens.css 缺单一事实源色值: ${cssMissing.join(' ')}`)
      if (cssExtra.length) fail('TOKEN-DRIFT-CROSS', `tokens.css 含 tokens.json 之外色值: ${cssExtra.join(' ')}`)
      if (keyDrift.length) fail('TOKEN-DRIFT-CROSS', `tokens.css 键值漂移: ${keyDrift.join(' ')}`)
    }

    // dark 块键值锚定 (对 tok.dark)
    if (tok.dark && typeof tok.dark === 'object') {
      const darkCss = readFileSync(TOKENS_CSS, 'utf8')
      const dm = darkCss.match(/\.dark\s*{([^}]*)}/)
      const darkKeys = {}
      if (dm) {
        for (const kv of dm[1].matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{3,8})\s*;/g)) darkKeys[kv[1]] = kv[2]
      }
      const darkAnchor = {
        'background': 'dark.background', 'foreground': 'dark.foreground', 'card': 'dark.card',
        'primary': 'dark.primary', 'secondary': 'dark.secondary', 'muted': 'dark.muted',
        'muted-foreground': 'dark.mutedForeground', 'accent': 'dark.accent',
        'destructive': 'dark.destructive', 'border': 'dark.border', 'ring': 'dark.ring',
        'ink-1': 'dark.ink.1', 'ink-2': 'dark.ink.2', 'ink-3': 'dark.ink.3', 'ink-line': 'dark.ink.line',
      }
      const darkDrift = []
      for (const [cssKey, tokPath] of Object.entries(darkAnchor)) {
        const node = getPath(tok, tokPath)
        const cssVal = darkKeys[cssKey]
        if (node && node.value && cssVal && node.value.toUpperCase() !== cssVal.toUpperCase()) {
          darkDrift.push(`--${cssKey}(dark)` )
        }
      }
      if (darkDrift.length) fail('TOKEN-DRIFT-CROSS', `tokens.css .dark 键值漂移: ${darkDrift.join(' ')}`)
    }
  }
}

// 5. SVG-BRAND-DRIFT (C5 自愈): 品牌 SVG 色值必须锚定在 tokens.json 单一事实源
//    (含 assetDerived 白名单)。新色 = 注册 assetDerived 或等于 token 值, 否则违约。
if (existsSync(TOKENS_JSON)) {
  let tok
  try { tok = JSON.parse(readFileSync(TOKENS_JSON, 'utf8')) } catch { tok = null }
  if (tok) {
    const allowed = new Set()
    const walk = (o) => {
      if (o && typeof o === 'object') {
        if (typeof o.value === 'string' && /^#[0-9a-fA-F]{3,8}$/.test(o.value)) allowed.add(o.value.toUpperCase())
        for (const v of Object.values(o)) walk(v)
      }
    }
    walk(tok)
    const brandSvgs = ['logo.svg', 'logo-animated.svg', 'hero.svg', 'background.svg', 'favicon.svg']
    let svgDrift = []
    for (const name of brandSvgs) {
      const p = join(ROOT, 'public', name)
      if (!existsSync(p)) { fail('SVG-BRAND-DRIFT', `品牌资产缺失: ${name}`); continue }
      const raw = readFileSync(p, 'utf8')
      const colors = new Set()
      for (const m of raw.matchAll(/#[0-9a-fA-F]{6}\b/g)) colors.add(m[0].toUpperCase())
      const extra = [...colors].filter((c) => !allowed.has(c))
      if (extra.length) svgDrift.push(`${name}: ${extra.join(' ')}`)
    }
    if (svgDrift.length === 0) {
      pass('SVG-BRAND-DRIFT', '品牌 SVG 色值全部锚定单一事实源 (token/assetDerived)')
    } else {
      svgDrift.forEach((d) => fail('SVG-BRAND-DRIFT', `品牌 SVG 含未注册色: ${d} → 注册到 tokens.json assetDerived`))
    }
  }
}

const failed = checks.filter((c) => !c.ok)
const json = process.argv.includes('--json')
if (json) {
  console.log(JSON.stringify({ passed: checks.length - failed.length, total: checks.length, checks }, null, 2))
} else {
  for (const c of checks) console.log(`[${c.ok ? 'PASS' : 'FAIL'}] ${c.id}: ${c.msg}`)
  console.log(`--- design-audit: ${checks.length - failed.length}/${checks.length} pass`)
}
process.exit(failed.length ? 1 : 0)