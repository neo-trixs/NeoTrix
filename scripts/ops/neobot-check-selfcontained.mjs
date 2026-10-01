/**
 * 自持性门 —— 断言 `neobot-ui` 的构建**只依赖版本控制里的文件**。
 *
 * # ⭐ 它守的是一类**已经发生过的阻断级缺陷**
 *
 * `neobot-root.tsx` 直接 import：
 *   `./vendor/openghost/tex.js` → `markdown.js` → `highlight.js` + `shim.ts`
 * 而根 `.gitignore` 有一条 blanket `vendor/`，把整个目录**连文件一起**排除。
 * `build` 脚本就是 `tsc --noEmit && vite build`，**没有任何 prebuild 拷贝步骤**
 * ⇒ **干净克隆根本构建不出 `neobot-ui`**。
 *
 * 实测（发现时）：`git ls-files apps/neobot-desktop/neobot-ui/src/vendor/` = **0**，
 * 而 `dist/` 里 `Markdown` 全局确实存在 ⇒ **本地能构建、能跑、能过 18 道门**，
 * 唯一症状是「换台机器/换个克隆就炸」。**所有既有门都测不出来** ——
 * 它们都在同一台机器上读同一份未跟踪的文件。
 *
 * ⓘ 这就是为什么它需要一道**独立**的门：门必须质疑「前提」，
 *    而不只是测「产物」。
 *
 * # 判据
 *
 * |  | 判据 | 拦住什么 |
 * |---|------|---------|
 * | A | 每个**相对 import** 解析到的文件**都被 git 跟踪** | 未跟踪的 vendored 代码（本次缺陷） |
 * | B | 没有 import **逃出** `neobot-ui/`（如伸进 `../frontend/`） | 自持树偷偷依赖参考树 ⇒ 删参考树即崩 |
 * | C | 相对 import 全部**可解析**（不悬空） | 改名/移动留下的死 import |
 *
 * ⛔ **A 是重点**：本机 `dist` 正常 ≠ 仓库完整。判据必须查**版本控制**，
 *    而不是查文件系统 —— 文件在磁盘上但不在 git 里，等于不存在。
 * ⛔ **B 不可省**：`neobot-ui` 的全部意义就是自持；一旦它依赖
 *    `frontend/`（ slated for 删除），删参考树就会同时打掉自持 UI。
 * ⛔ **C 用扩展名解析表**：TS 源码写 `./x` 可能对应 `x.ts`/`x.tsx`/`x.js`/
 *    `x.css` 或 `x/index.ts`。只试 `.ts` 会把好代码误判成悬空。
 *
 * 退出码：0 = PASS，1 = FAIL。
 */
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { execFileSync } from 'node:child_process'
import { dirname, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO = resolve(fileURLToPath(new URL('../../', import.meta.url)))
const UI = join(REPO, 'apps/neobot-desktop/neobot-ui')
const SRC = join(UI, 'src')
const EXTS = ['', '.ts', '.tsx', '.js', '.jsx', '.mjs', '.css', '.json']
const DIRS = ['', '/index.ts', '/index.tsx', '/index.js']

/** 递归列出 src 下的 .ts/.tsx（vendor 也要 —— 它同样是被 import 的一环）。 */
function walk(dir, out = []) {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name)
    if (e.isDirectory()) walk(p, out)
    else if (/\.tsx?$/.test(e.name)) out.push(p)
  }
  return out
}

// ⛔ 一次性取 git 跟踪集合：**只看文件系统会漏掉「文件在但没进版本控制」**
//    这正是本门存在的理由。
const tracked = new Set(
  execFileSync('git', ['ls-files'], { cwd: REPO, encoding: 'utf8', maxBuffer: 1 << 26 })
    .split('\n').filter(Boolean).map((p) => resolve(REPO, p)))

const files = walk(SRC)
// ⛔⛔ 必须**按文件**收集，不能拍平成一个 Set 再回头找「哪个文件导入它」。
//   我第一版用 `content.includes("'./x'")` 反查导入方，只认**单引号**；
//   而 `api-panel.ts` 写的是**双引号** `from "./dom.ts"` ⇒ 反查落空 ⇒
//   `from` 为空 ⇒ `dirname(SRC)` 把基准算到了 `neobot-ui/`（而不是 `neobot-ui/src/`）
//   ⇒ **`dom.ts` 被误报成悬空**。判据错 ≠ 代码有缺陷。
const imports = []
for (const f of files) {
  const src = readFileSync(f, 'utf8')
  // `import ... from './x'` / `import './x'`（⛔ 不含 bare specifier，那是包依赖）
  for (const m of src.matchAll(/(?:from|import)\s*\(?\s*(['"])(\.[^'"]+)\1/g)) imports.push({ file: f, spec: m[2] })
}
const specs = [...new Set(imports.map((i) => i.spec))]

let fail = 0
const bad = (m) => { console.log(`     ⛔ ${m}`); fail++ }

console.log(`自持性门（${files.length} 个源文件，${specs.length} 条相对 import）\n`)

let untracked = 0, escaped = 0, dangling = 0
for (const spec of specs.sort()) {
  const base = resolve(dirname(imports.find((i) => i.spec === spec).file), spec)

  // C：可解析
  let hit = null
  for (const d of DIRS) for (const e of EXTS) {
    const cand = base + d + e
    try { if (statSync(cand).isFile()) { hit = cand; break } } catch { /* 继续试 */ }
    if (hit) break
  }
  if (!hit) { dangling++; bad(`悬空 import \`${spec}\`（在 ${relative(REPO, imports.find((i) => i.spec === spec).file)}）⇒ 改名/移动留下的死引用`); continue }

  // B：不得逃出自持树
  if (!hit.startsWith(UI + '/')) {
    escaped++
    bad(`import \`${spec}\` 逃出自持树 → ${relative(REPO, hit)}`
      + ` ⇒ neobot-ui 依赖了 neobot-ui 之外的东西（删参考树即崩）`)
    continue
  }

  // A：必须被 git 跟踪 ⭐ 本门重点
  if (!tracked.has(hit)) {
    untracked++
    bad(`未跟踪：\`${spec}\` → ${relative(REPO, hit)}`
      + `\n        ⛔ 文件在磁盘上但**不在版本控制里** ⇒ 干净克隆构建会失败`
      + `\n        ⛔ 本机能跑能过门，是因为门和产物读的是同一份未跟踪文件`)
  }
}

console.log(`  A 相对 import 均已被 git 跟踪：${specs.length - untracked - escaped - dangling}/${specs.length} ${untracked ? '⛔' : '✅'}`)
console.log(`  B 未逃出自持树：${escaped ? `⛔ ${escaped} 条` : '✅'}`)
console.log(`  C 无悬空 import：${dangling ? `⛔ ${dangling} 条` : '✅'}`)

console.log()
console.log(fail
  ? `FAIL: ${fail} 项 —— 自持 UI 不可从干净克隆构建。`
  : `PASS: ${specs.length} 条相对 import 全部已跟踪、不逃出自持树、可解析。`)
process.exit(fail ? 1 : 0)
