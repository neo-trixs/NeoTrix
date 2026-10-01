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
const FE = join(ROOT, "apps/neobot-desktop/frontend/src");
const ENTRY = join(FE, "main.tsx");

/** Rust 侧 `generate_handler!` 注册的命令名（按函数名去重）。 */
function registeredCommands() {
  const mainRs = readFileSync(join(ROOT, "apps/neobot-desktop/src/main.rs"), "utf8");
  const out = new Set();
  for (const m of mainRs.matchAll(/neobot_desktop::[a-z_:]+::([a-z_0-9]+)/g)) out.add(m[1]);
  return out;
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

function resolveImport(fromFile, spec) {
  let base;
  if (spec.startsWith(".")) {
    base = resolve(dirname(fromFile), spec);
  } else if (spec.startsWith("@/")) {
    // ⛔ 别名必须一起走：全仓 UI 都用 `@/…`（vite alias → src/）。
    //    只认相对路径时，`config.tsx` 里的 4 个面板文件整条链都走不到 ——
    //    门会报「只有 15 种调用」的干净结论，而真凶一个都没看见。
    base = resolve(FE, spec.slice(2));
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
function reachable(entry) {
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
      const r = resolveImport(f, s);
      if (r) stack.push(r);
    }
  }
  return seen;
}

const problems = [];
const reg = registeredCommands();
const files = reachable(ENTRY);
const calls = new Map(); // 命令名 -> 调用它的文件

for (const f of files) {
  const src = readFileSync(f, "utf8");
  INVOKE_RE.lastIndex = 0;
  let m;
  while ((m = INVOKE_RE.exec(src))) {
    const name = m[1];
    if (NON_RUST_PREFIX.some((p) => name.startsWith(p))) continue;
    if (!calls.has(name)) calls.set(name, []);
    calls.get(name).push(f.slice(ROOT.length + 1));
  }
}

for (const [name, where] of [...calls].sort()) {
  if (!reg.has(name)) {
    problems.push(`界面调用未注册命令 \`${name}\` —— ${where.slice(0, 3).join(", ")}`);
  }
}

console.log(`UI 调用门：import 图 ${files.size} 个模块 · 界面调用 ${calls.size} 种 Rust 命令 · 已注册 ${reg.size}`);
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
