#!/usr/bin/env node
// 设置/界面「调了但没注册」门 —— 专治**静默失效**。
//
// ## 为什么需要它
//
// 上游 5 个配置面板里有 19 个 `invoke('X')` 在本仓**从未注册**（profiles 6 个、
// plugins 9 个、core 2 个、debug 1 个…）。它们的失效方式极其安静：
// react-query 拿到 reject 后回落到默认值 `[]`，面板画出一个**空列表**，
// 于是「没有插件/没有档案」看起来像事实，而不是「这个功能在本仓不存在」。
// 按钮点了才会抛 `command X not found`，而没人会为了一个空面板去点按钮。
//
// tsc 不会报、布局门不会报、交互门不会报（它没点那些按钮）——
// 没有任何既有门会因为这件事而红。
//
// ## 判定方式：走**真实 import 图**，不扫目录
//
// ⛔ 扫目录会把「已下线但文件还在」的 vendored 文件也算进来，于是要么误报、
//    要么逼着人删上游文件（那是更大的 1:1 声明）。只查从 `main.tsx` 可达的模块：
//    不可达 = 那些 invoke 永远不可能发生，不该拖红门。
//
// 用法：`node scripts/ops/nt_check_ui_calls.mjs`

import { readFileSync, existsSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

/**
 * ⭐⭐ 2026-10-07 修：真源从 `src/main.rs` 搬到 `src/lib.rs` 的
 *    `neobot_commands!` 宏，**本门没跟**（本仓第 6 次同款，见
 *    `STATUS.md` 与 `api.rs:123` 记的复发链）。
 *
 *    `lib.rs` 的段头写明了搬家理由（2026-10-02 P0 复盘）：
 *      「注册表若只留在 `main.rs`（bin 侧），`tests/` 根本够不着它
 *        ⇒ 漏注册不会有任何信号。⇒ 注册表进 lib、两侧展开同一个宏。」
 *
 *    ⛔ 旧实现读 `main.rs` 找 `neobot_desktop::` ⇒ `main.rs` 早就不展开
 *      注册表了（实测只剩 6 处非注册引用）⇒ **注册集合近乎为空**
 *      ⇒ 门把 70 多条**真注册**的命令全报成「未注册」。
 *    ⇒ 三道门（api / ui_calls / ship_ui）同时对着空注册表狂报 ⇒ 人只会
 *      认定「门又坏了」而忽略输出 ⇒ 门真正该抓的东西就此淹没。
 *
 * ⛔ 找不到宏体就 **FAIL**：缺前提必须失败，不能「查不了就放过」
 *   （`STATUS.md` §4 教训 38：「查不了」被写成「通过」，门从「证明没漂移」
 *   退化成「什么都没做」）。
 */
function registeredCommands() {
  const libRs = readFileSync(join(ROOT, "apps/neobot-desktop/src/lib.rs"), "utf8");
  const macroAt = libRs.indexOf("macro_rules! neobot_commands");
  if (macroAt < 0) {
    console.error("  ✗ lib.rs 里找不到 `macro_rules! neobot_commands` —— 注册表真源改名/搬走了，本门须同步");
    process.exit(1);
  }
  const first = libRs.indexOf("neobot_desktop::", macroAt);
  if (first < 0) {
    console.error("  ✗ `neobot_commands!` 宏体里没有 `neobot_desktop::` —— 空宏或改了写法，本门须同步");
    process.exit(1);
  }
  // 剥注释（不剥会把「见 desktop.rs」读成命令 `desktop`/`rs`）。
  const body = stripJsComments(libRs.slice(first, libRs.indexOf("\n}", first)));
  const out = new Set();
  for (const m of body.matchAll(/neobot_desktop::[a-z_:]+::([a-z_0-9]+)/g)) out.add(m[1]);
  return out;
}

/** 剥 JS 风格注释（本仓注释里写满命令名与文件名，不剥必假警）。 */
function stripJsComments(src) {
  return src.replace(/\/\*[\s\S]*?\*\//g, "").replace(/(^|[^:])\/\/[^\n]*/g, "$1");
}

/** 本仓前端内部调用但不经 Rust 的名字（插件 API 的 `plugin:*` 命名空间等）。 */
const NON_RUST_PREFIX = ["plugin:"];

const EXTS = [".ts", ".tsx", ".js", ".jsx", ""];

/** 解析一个 import 目标为磁盘路径；解析不出来返回 null（外部包/别名）。 */
function isFile(p) {
  try {
    return existsSync(p) && statSync(p).isFile();
  } catch {
    return false;
  }
}

function resolveImport(fromFile, spec, aliasRoot) {
  let base;
  if (spec.startsWith(".")) {
    base = resolve(dirname(fromFile), spec);
  } else if (spec.startsWith("@/")) {
    // ⛔ 别名必须一起走：全仓 UI 都用 `@/…`（vite alias → src/）。
    //    只认相对路径时，`config.tsx` 里的 4 个面板文件整条链都走不到 ——
    //    门会报「只有 15 种调用」的干净结论，而真凶一个都没看见。
    // ⚠️ 别名根**由调用方显式传入**（两棵树各有自己的 `src/` 与 vite alias）。
    //    ⛔⛔ 不要从 `fromFile` 反推：我第一版写成 `dirname(dirname(fromFile))`，
    //    对 `src/config.tsx` 恰好对、对 `src/ui/dialog/config.tsx` 就多退了一层
    //    ⇒ vendored 树的可达模块从 **81 掉到 38**（实测），
    //    而门只会显示一个更小、更「干净」的数字。
    //    ⛔ 这正是 `STATUS.md` §4 教训 12 的同款：**数数方式本身要被验证** ——
    //    这次靠「每树分别报数」才看出来（合计数会把它藏起来）。
    base = resolve(aliasRoot, spec.slice(2));
  } else {
    return null; // 外部包
  }
  // ⛔ 必须 stat 成文件：`./ui` 这类目录也会 existsSync 为真，
  //    直接返回就会 readFileSync 一个目录（EISDIR）。
  for (const ext of EXTS) {
    const p = base + ext;
    if (isFile(p)) return p;
  }
  for (const ext of [".ts", ".tsx", ".js", ".jsx"]) {
    const p = join(base, "index" + ext);
    if (isFile(p)) return p;
  }
  return null;
}

const IMPORT_RE = /(?:^|\n)\s*import\s+(?:[\s\S]*?)\s*from\s*['"]([^'"]+)['"]/g;
const SIDE_EFFECT_RE = /(?:^|\n)\s*import\s*['"]([^'"]+)['"]/g;
const INVOKE_RE = /invoke(?:<[^>]*>)?\(\s*['"]([a-z_:]+)['"]/g;

/** 从入口出发走真实 import 图，返回可达文件集合。 */
function reachable(entry, aliasRoot) {
  const seen = new Set();
  const stack = [entry];
  while (stack.length) {
    const f = stack.pop();
    if (!f || seen.has(f) || !isFile(f)) continue;
    seen.add(f);
    const src = readFileSync(f, "utf8");
    const specs = [];
    for (const re of [IMPORT_RE, SIDE_EFFECT_RE]) {
      re.lastIndex = 0;
      let m;
      while ((m = re.exec(src))) specs.push(m[1]);
    }
    for (const s of specs) {
      const r = resolveImport(f, s, aliasRoot);
      if (r) stack.push(r);
    }
  }
  return seen;
}

const problems = [];
const notes = [];
const reg = registeredCommands();

/**
 * ⭐⭐ 2026-10-07：门此前**只扫 vendored 参考树**，而那是**冻结**的
 *    （`AGENTS.md` §0：唯一交付 UI 是 `neobot-ui/`）。⇒ 交付树里新写的
 *    `invoke` **完全不在门的视野内** —— 那正是本门最该抓的东西。
 *
 *    ⭐ 两种口径（照抄 `nt_api_contract.py` ③ 项 2026-10-04 的裁决）：
 *      · 交付树 ⇒ 未注册即 **FAIL**（那是发布出去的东西）。
 *      · vendored ⇒ 只 **ℹ️**（冻结树的调用不该阻塞我方；
 *        `nt_api_contract.py` 已为同一个命中做过同样判断并写下理由）。
 */
const TREES = [
  {
    label: "交付树",
    entry: join(ROOT, "apps/neobot-desktop/neobot-ui/src/main.tsx"),
    aliasRoot: join(ROOT, "apps/neobot-desktop/neobot-ui/src"),
    fatal: true,
  },
  {
    label: "vendored 参考树",
    entry: join(ROOT, "apps/neobot-desktop/frontend/src/main.tsx"),
    aliasRoot: join(ROOT, "apps/neobot-desktop/frontend/src"),
    fatal: false,
  },
];

let totalFiles = 0;
const perTree = [];
const allCalls = new Map(); // 命令名 -> [{tree, file}]

for (const tree of TREES) {
  if (!isFile(tree.entry)) {
    notes.push(`ℹ️ ${tree.label}入口不存在：${tree.entry.slice(ROOT.length + 1)} ⇒ 该树本轮未扫（不阻塞，但要知情）`);
    continue;
  }
  const files = reachable(tree.entry, tree.aliasRoot);
  totalFiles += files.size;
  // ⭐ 每树**分别**报数：合计会把「一棵树的链断了」藏起来。
  //    实测踩过：合计 57（旧实现单树 81）—— 少的那一半正是没走到的模块，
  //    而门只会显示一个更小、更「干净」的数字（`STATUS.md` §4 教训 12 的同款）。
  perTree.push({ label: tree.label, modules: files.size, entry: tree.entry.slice(ROOT.length + 1) });
  for (const f of files) {
    // ⛔ 剥注释：本仓注释里大量出现 `invoke('neobot_xxx')` 的**历史形态**
    //    （记录「改前是什么样」），不剥就会把注释里的旧命令名当真实调用。
    const src = stripJsComments(readFileSync(f, "utf8"));
    INVOKE_RE.lastIndex = 0;
    let m;
    while ((m = INVOKE_RE.exec(src))) {
      const name = m[1];
      if (NON_RUST_PREFIX.some((p) => name.startsWith(p))) continue;
      if (!allCalls.has(name)) allCalls.set(name, []);
      allCalls.get(name).push({ tree: tree.label, file: f.slice(ROOT.length + 1) });
    }
  }
}

for (const [name, where] of [...allCalls].sort()) {
  if (reg.has(name)) continue;
  const fatalHit = where.filter((w) => w.tree === "交付树");
  const msg = `界面调用未注册命令 \`${name}\` —— ${where
    .slice(0, 3)
    .map((w) => `${w.tree}:${w.file}`)
    .join(", ")}`;
  if (fatalHit.length) problems.push(msg);
  else notes.push(`ℹ️ ${msg}（仅 vendored 冻结树）`);
}

const calls = allCalls;
console.log(
  `UI 调用门：import 图 ${totalFiles} 个模块（${perTree
    .map((t) => `${t.label} ${t.modules}`)
    .join(" + ")}）· 界面调用 ${allCalls.size} 种 Rust 命令 · 已注册 ${reg.size}`,
);
if (notes.length) {
  console.log(`ℹ️ 观察 ${notes.length} 条（不当失败）:`);
  for (const n of notes) console.log("  " + n);
}
if (problems.length) {
  console.error(`\nUI 调用门 FAIL（${problems.length} 项）:`);
  for (const p of problems) console.error(`  · ${p}`);
  console.error(
    "\n  每一条在界面上都表现为「空列表」或点了报 command not found —— 都不像故障。\n" +
      "  修法二选一：① 注册薄壳命令（真能力）；② 让面板别调它（该功能在本仓不存在就别渲染入口）。",
  );
  process.exit(1);
}
console.log("UI 调用门 PASS（界面可达模块里的每个 invoke 都有已注册命令）");
