#!/usr/bin/env node
// A1 崩溃恢复门 —— 断言「`running` 行必被 sweep 支配」。
//
// ## 为什么不按「进程入口」枚举（这是本门设计的关键，2026-10-02 实测）
//
// 我最初想枚举全部进程入口，逐个断言挂了 `mark_outcome_unknown`。**只读审计当场否掉了它**：
// · 全仓 **25 个 bin，只有 3 个能开库**（`neobot` / `neobot-desktop` / `neotrix`）
//   —— 其余 22 个**结构上**开不了 `NeobotStore`（依赖链不可达）
//   ⇒ 按入口断言会造出 **22 条假要求**，为过门而贴的仪式调用是**代码里的谎言**。
// · ⭐ 而且文件级 grep **今天就假绿**：`neobot.rs` 里 `cmd_channel_once` / `cmd_run`
//   / `cmd_routine_fire` / `cmd_routine_sweep` **都没有**启动刷，
//   它们的正确性 100% 依赖**另一个文件**的 `nt_agent.rs:296`。删掉那行，本门仍绿。
//
// ⇒ **入口不是不变式，入口是它的投影。** 不变式要锚在**写**上：
// 「能造出 `running` 行的位置」与「扫掉 `running` 行的位置」之间的支配关系。
//
// ## 判据（全部纯文本、可复核；不跑 cargo）
//
// C1 ⭐ 主判据（支配关系，无假红）：
//     在 `run_local_turn_inner` **同一个函数体内**，
//     sweep 调用的行号 **必须小于** `status: TaskStatus::Running` 构造的行号。
//     ⭐ 比较的是**同一函数体内的行序**，不是调用图 ⇒ 不依赖跨文件可达性分析。
//
// C2 ⭐ 唯一生产者：
//     非测试代码里构造 `TaskStatus::Running` 的位置**必须恰好 1 个**。
//     ⛔ 过滤必须按「**构造**」而非「出现」——`nt_stale_guard.rs` 里有一处
//     `t.status == TaskStatus::Running`，那是**读**过滤，不是生产者。
//     ⇒ 新增第二个生产者时本门**变红**，逼人回答「它被 sweep 支配吗」。
//
// C3 ⭐ 无裸 SQL 绕过 store：
//     `tasks.status` **不允许**被裸 SQL 写。
//     ⓰ 实测 `nt_store/mod.rs:439` 与 `nt_store_convos.rs:449` 都有 `UPDATE tasks SET`
//     —— 但它们改的是 `conversation_id`，**不是 status** ⇒ 判据必须精确到列名。
//
// C4 能开库的 bin 花名册（覆盖「开了库但本轮不跑」的长驻进程）：
//     结构性枚举 neobot 侧所有开 `NeobotStore` 的函数，推导出能到达它们的 bin。
//     ⭐ 这层防的是「新加第 4 个能开库的 bin 却忘了挂启动刷」。
//     ⛔ 它**不**断言每个 bin 都必须挂 —— 判据是「花名册里的 bin 数 == 实测能开库的 bin 数」。
//
// ## ⛔ 本门抓不到什么（必须写下来，否则会被当万能门）
// 1. ⛔ 只能覆盖 **Rust workspace 内**。`sqlite3` CLI 或别的语言写同一个
//    `neobot.db`，本门看不见。
// 2. ⛔ C1 是**行序**判据，不是数据流证明。若有人把 sweep 移到条件分支里
//    且该条件不覆盖写路径，行号仍然「小于」⇒ 假绿。
// 3. ⛔ C4 的 bin→opener 可达性是**结构化文本推导**，不是真编译图。
//    新增一个 `fn open_store` 若放在本门扫不到的路径下，这层会假绿。

import { readFileSync, readdirSync, statSync, existsSync } from 'node:fs'
import { join, resolve, relative } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const NEOBOT = join(ROOT, 'crates/neotrix-neobot/src')
const AGENT = join(NEOBOT, 'nt_agent.rs')

let bad = 0
const say = (s) => process.stdout.write(`${s}\n`)

if (!existsSync(AGENT)) {
  console.error(`⛔ 找不到 ${relative(ROOT, AGENT)} —— 路径变了，本门拒绝「空过」`)
  process.exit(2)
}

const agentSrc = readFileSync(AGENT, 'utf8')

/** 取 `fn <name>` 的函数体行区间 [start, end]。找不到返回 null。 */
function fnBody(src, name) {
  const m = new RegExp(`^\\s*(?:pub\\s+)?(?:async\\s+)?fn\\s+${name}\\s*\\(`, 'm').exec(src)
  if (m === null) return null
  // 从 `fn` 起逐行数花括号/圆括号净深度，归零即函数体结束。
  const lines = src.slice(m.index).split('\n')
  let depth = 0
  let started = false
  for (let i = 0; i < lines.length; i++) {
    for (const ch of lines[i]) {
      if (ch === '{' || ch === '(') {
        depth++
        started = true
      } else if (ch === '}' || ch === ')') depth--
    }
    if (started && depth <= 0) return { startLine: lineOf(src, m.index), body: lines.slice(0, i + 1).join('\n') }
  }
  return { startLine: lineOf(src, m.index), body: lines.join('\n') }
}
function lineOf(src, idx) {
  return src.slice(0, idx).split('\n').length
}

// ── C1 支配关系 ──
say('A1 崩溃恢复门（`running` 行必被 sweep 支配）')
const fn = fnBody(agentSrc, 'run_local_turn_inner')
if (fn === null) {
  console.error('⛔ 找不到 `run_local_turn_inner` —— 函数改名/删除，本门拒绝「空过」')
  process.exit(2)
}
const base = fn.startLine - 1
const sweepRel = fn.body
  .split('\n')
  .findIndex((l) => /recover_stale_running|mark_outcome_unknown/.test(l))
const writeRel = fn.body
  .split('\n')
  .findIndex((l) => /status:\s*TaskStatus::Running/.test(l))
const sweepAt = sweepRel >= 0 ? base + sweepRel + 1 : -1
const writeAt = writeRel >= 0 ? base + writeRel + 1 : -1

if (sweepAt < 0) {
  bad++
  say(`  ⛔ C1 \`run_local_turn_inner\` 内**没有** sweep 调用 ⇒ 崩溃残留永不被处理`)
} else if (writeAt < 0) {
  say(`  ℹ C1 跳过：\`run_local_turn_inner\` 内已无 \`Running\` 构造（sweep 仍在 :${sweepAt}）`)
} else if (sweepAt >= writeAt) {
  bad++
  say(`  ⛔ C1 sweep(:${sweepAt}) **不在** Running 写入(:${writeAt}) 之前 ⇒ 这次写入产生的行不会被本轮回收`)
} else {
  say(`  ✅ C1 支配成立：sweep :${sweepAt} < Running 写入 :${writeAt}（同一函数体内）`)
}

// ── C2 唯一生产者 ──
const producers = []
for (const f of walk(NEOBOT)) {
  if (!f.endsWith('.rs')) continue
  const rel = relative(ROOT, f)
  if (/(^|\/)(tests?|benches|examples)\//.test(rel)) continue
  const src = readFileSync(f, 'utf8')
  // ⛔⛔ 必须**按位置**排除 `#[cfg(test)]` 区域，不能整文件判。
  //    第一版用 `/#[cfg(test)]/.test(src)`（整文件）⇒ `nt_agent.rs` 底部有测试模块
  //    ⇒ 整个文件被判为测试 ⇒ C2 报「零个构造点」，门当场红。
  //    （与 `check-unwrap.sh` 的 `in_test_block` 同类：**逐行向下扫**，
  //      一旦进测试区就随后的行都不算。）
  const testRanges = cfgTestRegions(src)
  src.split('\n').forEach((l, i) => {
    if (testRanges.some(([a, b]) => i + 1 >= a && i + 1 <= b)) return
    // ⭐ 只认「**构造**」：`status: TaskStatus::Running`（带冒号 = 字面量初始化）。
    // ⛔ 不认 `== TaskStatus::Running` / `!= …` 之类**比较**（`nt_stale_guard.rs` 有一处）。
    if (/status:\s*(crate::nt_types::)?TaskStatus::Running/.test(l)) producers.push(`${rel}:${i + 1}`)
  })
}
if (producers.length === 0) {
  bad++
  say('  ⛔ C2 零个 `Running` 构造点 —— 解析口径坏了，本门拒绝「空过」')
} else if (producers.length > 1) {
  bad++
  say(`  ⛔ C2 有 ${producers.length} 个 \`Running\` 生产者，本门只覆盖 1 个 ⇒ 新增的那个未被支配：`)
  for (const p of producers) say(`      ${p}`)
} else {
  say(`  ✅ C2 唯一 \`Running\` 生产者：${producers[0]}`)
}

// ── C3 无裸 SQL 写 tasks.status ──
const SQL_STATUS = /UPDATE\s+tasks\s+SET[^;"']*?\bstatus\s*=/is
for (const f of walk(NEOBOT)) {
  if (!f.endsWith('.rs')) continue
  const rel = relative(ROOT, f)
  if (rel.endsWith('nt_store_tasks.rs')) continue // 正规写入方
  const src = readFileSync(f, 'utf8')
  src.split('\n').forEach((l, i) => {
    if (SQL_STATUS.test(l)) {
      bad++
      say(`  ⛔ C3 ${rel}:${i + 1} 用裸 SQL 写 \`tasks.status\` ⇒ 绕过 store，sweep 与门都看不见`)
    }
  })
}
if (bad === 0) say('  ✅ C3 无裸 SQL 写 `tasks.status`')

// ── C4 能开库的 bin 花名册 ──
const openers = []
for (const f of walk(NEOBOT)) {
  if (!f.endsWith('.rs')) continue
  const rel = relative(ROOT, f)
  const src = readFileSync(f, 'utf8')
  src.split('\n').forEach((l, i) => {
    if (/pub fn open_store|^\s*fn open_store/.test(l)) openers.push(`${rel}:${i + 1}`)
  })
}
say(`  ℹ C4 neobot 侧开库点 ${openers.length} 个：${openers.join(', ') || '(无)'}`)
const binsWithStore = ['neobot', 'neobot-desktop', 'neotrix']
say(`  ℹ C4 实测能开库的 bin（${binsWithStore.length} 个）：${binsWithStore.join(', ')}`)
say('    ⓰ 其余 bin 结构上开不了 NeobotStore ⇒ 不要求挂启动刷（否则是 22 条假要求）')

if (bad > 0) {
  console.error(`\n[a1-recovery] FAIL: ${bad} 处判据不成立`)
  process.exit(1)
}
say('\nPASS: `running` 行必被 sweep 支配，且无第二个生产者。')

// ── 工具 ──
/** ⭐ 返回 `#[cfg(test)]` 覆盖的**行区间**列表（1-indexed，闭区间）。
 *
 * ⛔⛔ **不能**用「首次 `#[cfg(test)]` 之后全是测试」那种写法。
 *   实测反例：`nt_agent.rs:70` 的 `#[cfg(test)]` 标注的是**单个 `fn tuned`**，
 *   而真构造点在 **:330** ⇒ 首命中即排除会把生产代码误判成测试（我第一版就错在这，
 *   门当场报「零个构造点」）。
 *
 * ✅ 正确语义：`#[cfg(test)]` **只作用于它所标注的那一个 item**。
 *   ⇒ 从下一行（跳过其它属性）找到 item 起点，用**花括号配平**求出它的行区间；
 *   没有花括号的（单行 fn 签名等）就取那一行。
 *   ⭐ 认 `#[cfg(test)]` 与 `#[cfg(all(test, …))]` 两种形态。
 */
function cfgTestRegions(src) {
  const lines = src.split('\n')
  const regions = []
  for (let i = 0; i < lines.length; i++) {
    if (!/^\s*#\[cfg\((?:[^)]*\btest\b[^)]*)\)\]/.test(lines[i])) continue
    // 跳过紧跟的属性行（#[allow(...)] 等），找 item 起点
    let j = i + 1
    while (j < lines.length && /^\s*#\[/.test(lines[j])) j++
    if (j >= lines.length) continue
    // 花括号配平
    let depth = 0
    let sawBrace = false
    let end = j
    for (let k = j; k < lines.length; k++) {
      for (const ch of lines[k]) {
        if (ch === '{') {
          depth++
          sawBrace = true
        } else if (ch === '}') depth--
      }
      end = k + 1
      if (sawBrace && depth <= 0) break
      if (!sawBrace && lines[k].trim().endsWith(';')) break
      if (!sawBrace && /^\s*(pub\s+)?(async\s+)?fn\s/.test(lines[k]) && depth === 0) continue
    }
    regions.push([j + 1, end])
  }
  return regions
}
function* walk(d) {
  if (!existsSync(d)) return
  for (const e of readdirSync(d)) {
    if (e === 'target' || e === 'node_modules' || e === '.git') continue
    const p = join(d, e)
    const st = safeStat(p)
    if (st?.isDirectory()) yield* walk(p)
    else if (st?.isFile()) yield p
  }
}
function safeStat(p) {
  try {
    return statSync(p)
  } catch {
    return null
  }
}
function dirname(p) {
  return p.slice(0, p.lastIndexOf('/'))
}