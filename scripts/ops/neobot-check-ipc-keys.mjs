#!/usr/bin/env node
// ⛔⛔ IPC 载荷键名门 —— 抓「snake_case 载荷键 vs Tauri 默认 camelCase」这一整类缺陷。
//
// ## 为什么必须有这道门（2026-10-02 实测事故）
//
// 本仓**所有 UI 门都把 Tauri IPC 打桩掉了**（`neobot-ui-smoke.mjs`、`neobot-check-*.mjs`
// 各自 `invoke = async (cmd, args) => …`，直接读 `args.xxx` 返回 fixture）。
// ⇒ **它们在结构上就抓不到「键名对不上 Rust 形参」**：桩是 JS，键名怎么写都能读到。
// ⭐ 这就是事故能带着「22 门全绿」活下来的原因 —— 门全绿，但真 app 里：
//   · `neobot_convo_messages(convo_id: String)`  → 硬失败 `missing required key convoId`
//     ⇒ 前端 `.catch(() => setMsgs([]))` **静默** ⇒ 切会话历史**永远空**；
//   · `neobot_send(convo_id: Option<String>)`     → ⭐ `deserialize_option` 缺键走
//     `visitor.visit_none()`（`tauri-2.12.0/src/ipc/command.rs:145-153`）
//     ⇒ **静默变 `None`，不报错** ⇒ 每条消息**都不落库**。
//
// ## 判据来源（不是「我记得 Tauri 是 camelCase」）
//  ① `tauri-macros-2.7.0/src/command/wrapper.rs:51`  `argument_case: ArgumentCase::Camel`（默认）
//  ② 同文件 `:499-502`  `ArgumentCase::Camel => key = key.to_lower_camel_case()`
//  ③ `tauri-2.12.0/src/ipc/command.rs:109`  `InvokeBody::Json(v) => match v.get(self.key)`
//     ⇒ **精确匹配，无 snake_case 回退**。
// ⛔ 本门**不使用**「`rename_all` 有没有被覆盖」这条捷径：那需要完整解析 Rust 属性，
//    容易假绿。这里直接按 ①②③ 的规则算期望键名。
//
// ## ⛔ 本门**不**做的事
//  · 不校验 `main.rs` 的 `generate_handler!` 注册（那是 `nt_check_api.mjs` 的活）；
//  · 不校验返回值的 serde 形状（那是 `nt_api_contract.py` 的活）；
//  · ⛔ **不做运行时往返**：真判据仍需真 app 起进程验一次（见门输出末尾的提示）。
//    本门是**静态**门，抓的是「键名与形参名不一致」这一类**可静态判定**的缺陷。

import { readFileSync, existsSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const APP = join(ROOT, 'apps/neobot-desktop')
/** ⛔ 只扫**交付树**。`frontend/` 是 vendored 参考树且正在退役
 *  （`apps/neobot-desktop/neobot-ui/PROVENANCE.md` §7 列为待删），
 *  且改它会撞 `nt_check_status.mjs` ①（`STATUS.md:35` 的 16405 行）。 */
const UI = join(APP, 'neobot-ui/src')
const RUST_DIRS = [join(APP, 'src')]

let bad = 0
const say = (s) => process.stdout.write(`${s}\n`)

// ── heck 的 `to_lower_camel_case`：逐字照搬 tauri-macros 用的那条规则 ──
// heck 的规则是「下划线分段 + 首段全小写 + 后续段首字母大写」。
// ⛔ 刻意**不引入** `heck` 依赖（它只是 tauri-macros 的依赖，不是本 workspace 的）。
function toLowerCamelCase(snake) {
  const parts = String(snake).split('_').filter(Boolean)
  if (parts.length === 0) return snake
  return parts[0] + parts.slice(1).map((p) => p[0].toUpperCase() + p.slice(1)).join('')
}

// ── ① 从 Rust 侧抽出每个 `#[tauri::command]` 的形参名 ──
// ⛔ 必须按**括号配平**切签名，不能用 `fn name(...)` 的非贪婪正则：
//    形参里有 `Vec<String>` / `Option<i64>` 这类嵌套尖括号时 `[^)]*` 会截断。
const COMMANDS = new Map() // name -> { file, params: string[] }
for (const dir of RUST_DIRS) {
  for (const f of walk(dir)) {
    if (!f.endsWith('.rs')) continue
    const src = readFileSync(f, 'utf8')
    const attrRe = /#\[tauri::command(?:\([^\]]*\))?\]/g
    let m
    while ((m = attrRe.exec(src)) !== null) {
      const after = src.slice(m.index + m[0].length)
      const fnM = /fn\s+([a-z0-9_]+)\s*\(/.exec(after)
      if (fnM === null) continue
      const name = fnM[1]
      const open = after.indexOf('(', fnM.index)
      const args = readBalanced(after, open)
      if (args === null) continue
      const params = splitTopLevel(args)
        .map((a) => /^\s*([a-z_][a-z0-9_]*)\s*:/.exec(a))
        .filter(Boolean)
        .map((mm) => mm[1])
      // 跳过 app/window/state 这类注入形参（它们不来自载荷）。
        .filter((p) => !/^(app|state|window|webview|webviewWindow)$/i.test(p))
      COMMANDS.set(name, { file: f, params })
    }
  }
}
if (COMMANDS.size === 0) {
  console.error('⛔ 抽出 0 个 tauri::command —— 解析器坏了，本门拒绝「空过」')
  process.exit(2)
}

// ── ② 扫交付树所有 `invoke('cmd', { … })` 的载荷键 ──
const calls = []
for (const f of walk(UI)) {
  if (!/\.(ts|tsx)$/.test(f)) continue
  const src = readFileSync(f, 'utf8')
  // 匹配 invoke[泛型]('name', { … })，载荷块用括号配平切。
  const re = /invoke(?:<[^>]*>)?\(\s*'([a-z0-9_]+)'\s*,\s*\{/g
  let m
  while ((m = re.exec(src)) !== null) {
    const open = m.index + m[0].length - 1
    const body = readBalanced(src, open)
    if (body === null) continue
    // ⛔⛔ 必须按对象字面量**切条目**，不能对全文扫 `ident:` ——
    //    `{ convoId: sel }` 里 `sel` 是**值**，扫 `ident` 会把它误当键
    //    （我第一版就是这么错的：门把 `sel`/`member`/`href` 报成缺键）。
    //    正确口径：条目形如 `key: value` ⇒ 键是冒号前的 ident；
    //    形如裸 ident（ES6 简写 `{ sel }`）⇒ 键就是它本身。
    const keys = []
    for (const entry of splitTopLevel(body)) {
      const e = entry.trim()
      if (!e) continue
      const kv = /^([a-z_$][a-z0-9_$]*)\s*:/.exec(e)
      if (kv !== null) keys.push(kv[1])
      else if (/^[a-z_$][a-z0-9_$]*$/.test(e)) keys.push(e)
    }
    calls.push({ file: f, cmd: m[1], keys: [...new Set(keys)], line: lineOf(src, m.index) })
  }
}
if (calls.length === 0) {
  console.error('⛔ 抽出 0 个 invoke 调用 —— 正则坏了，本门拒绝「空过」')
  process.exit(2)
}

say('IPC 载荷键名门（Tauri 默认 camelCase）')
say(`  Rust #[tauri::command] ${COMMANDS.size} 个；交付树 invoke ${calls.length} 处`)

let checked = 0
for (const c of calls) {
  const cmd = COMMANDS.get(c.cmd)
  if (cmd === undefined) continue // 未在本仓定义的命令（如平台命令）不在本门职责内
  for (const key of c.keys) {
    // 期望键 = camelCase(形参)。若前端键恰好等于某个形参的 camelCase 形式 ⇒ 通过。
    // ⛔⛔ **只**认 camelCase 形式。曾经写成 `cmd.params.includes(key) || …`，
    //    那个 `includes` 会让**原始 snake_case 形参名**直接短路通过
    //    ⇒ 门变成假通过（负向测试实测抓到：注入 convo_id 仍报绿）。
    const ok = cmd.params.some((p) => toLowerCamelCase(p) === key)
    if (!ok) {
      // 区分「拼错」与「用了 snake_case 形参名」—— 后者的修法是明确的。
      const snakeHit = cmd.params.find((p) => p === key)
      bad++
      say(
        `  ⛔ ${rel(c.file)}:${c.line}  ${c.cmd}  载荷键「${key}」` +
          (snakeHit
            ? ` ⭐ Rust 形参是 \`${snakeHit}\` ⇒ Tauri 期望键是 \`${toLowerCamelCase(snakeHit)}\``
            : ` ⇒ Rust 形参无一匹配（期望 ${cmd.params.map((p) => toLowerCamelCase(p)).join(', ') || '（无）'}）`),
      )
      say(`      Rust 签名：${rel(cmd.file)}  形参 [${cmd.params.join(', ')}]`)
      say(`      ⚠️ Tauri 取键是 \`v.get("${key}")\`，**无 snake_case 回退**`)
      continue
    }
    checked++
  }
}

if (bad > 0) {
  console.error(`\n[sipc-keys] FAIL: ${bad} 个载荷键与 Rust 形参名不一致`)
  console.error('⏰ 本门是静态的。真判据仍需起一次真 app 验「切会话有历史 / 消息落库」。')
  process.exit(1)
}
say(`  ✅ ${checked} 个载荷键全部与 Rust 形参的 camelCase 形式一致`)

// ── 小工具 ──
function readBalanced(s, open) {
  let depth = 0
  for (let i = open; i < s.length; i++) {
    const c = s[i]
    if (c === '(' || c === '{' || c === '[') depth++
    else if (c === ')' || c === '}' || c === ']') {
      depth--
      if (depth === 0) return s.slice(open + 1, i)
    }
  }
  return null
}
function splitTopLevel(s) {
  const out = []
  let depth = 0
  let cur = ''
  for (const c of s) {
    if ('([{<'.includes(c)) depth++
    else if (')]}>'.includes(c)) depth--
    if (c === ',' && depth === 0) {
      out.push(cur)
      cur = ''
      continue
    }
    cur += c
  }
  if (cur.trim()) out.push(cur)
  return out
}
function lineOf(s, idx) {
  return s.slice(0, idx).split('\n').length
}
function rel(p) {
  return p.startsWith(ROOT) ? p.slice(ROOT.length + 1) : p
}
function* walk(d) {
  if (!existsSync(d)) return
  for (const e of readdirSyncSafe(d)) {
    const p = join(d, e)
    if (e === 'node_modules' || e === 'dist' || e === 'target') continue
    const st = statSafe(p)
    if (st?.isDirectory()) yield* walk(p)
    else if (st?.isFile()) yield p
  }
}
import { readdirSync as _r, statSync as _s } from 'node:fs'
function readdirSyncSafe(d) {
  try {
    return _r(d)
  } catch {
    return []
  }
}
function statSafe(p) {
  try {
    return _s(p)
  } catch {
    return null
  }
}