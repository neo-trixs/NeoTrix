#!/usr/bin/env node
// **交付路径**门 —— 守 `neobot-ui/`（商用自研壳），不是 vendored 参考树。
//
// ## 为什么单独一门
//
// 2026-10-01 之前，所有前端门（布局/交互/1:1）守的是 `frontend/`（vendored 上游）。
// 商用确认后交付路径改成 `neobot-ui/`，于是那些门**整体守错了对象** ——
// 它们全绿，而交付物里有两个洞：
//   ① `tauri.conf.json` 的 frontendDist 指向 neobot-ui/dist，而那里**没有 pet.html**
//      ⇒ `pet.rs` 建的是 `WebviewUrl::App("pet.html")` ⇒ 桌宠窗指向不存在的页面；
//   ② 原生菜单只发 `macos-menu-action`，而自研壳**没有 listen** ⇒
//      macOS 上「菜单在、点了没反应」。
//
// 两个洞都不需要「跑起来」就能判定：dist 里有没有 pet.html、产物里有没有监听。
//
// 用法：`node scripts/ops/nt_check_ship_ui.mjs`

import { readFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { requireFreshDist } from './nt_dist_freshness.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const SHIP = join(ROOT, "apps/neobot-desktop/neobot-ui");
const DIST = join(SHIP, "dist");

// ⭐⭐ 2026-10-07 P0-1：产物新鲜度（公用件；本门的 DIST 由 SHIP 推导，
//    故用 requireFreshDist 显式传坐标，而不是 neobotDistFresh 的默认坐标）。
requireFreshDist({ dist: DIST, src: join(SHIP, "src"), label: "ship 门" });
const CONF = join(ROOT, "apps/neobot-desktop/tauri.conf.json");

const bad = [];
const ok = [];

// ── ① tauri.conf 指向交付路径 ──
const conf = JSON.parse(readFileSync(CONF, "utf8"));
const dist = conf.build?.frontendDist ?? "";
if (!/neobot-ui/.test(dist)) {
  bad.push(`tauri.conf.json 的 frontendDist=${JSON.stringify(dist)}，交付路径应是 neobot-ui/dist`);
} else {
  ok.push(`frontendDist=${dist}`);
}

// ── ② 桌宠页必须真的在产物里 ──
// ⛔ 只查源码里有 pet.tsx 不够：vite 默认只把 index.html 当入口，
//    根目录的 pet.html 不会进 dist —— 源码齐全而桌宠照样坏掉。
if (!existsSync(DIST)) {
  bad.push("neobot-ui/dist 不存在 —— 先跑 pnpm --dir apps/neobot-desktop/neobot-ui run build");
} else {
  if (!existsSync(join(DIST, "pet.html"))) {
    bad.push("neobot-ui/dist 缺 pet.html ⇒ 桌宠窗指向不存在的页面（vite 需要多入口配置）");
  } else {
    ok.push("dist/pet.html 存在");
  }
  const js = (await import("node:fs")).readdirSync(join(DIST, "assets"))
    .filter((f) => f.endsWith(".js"))
    .map((f) => readFileSync(join(DIST, "assets", f), "utf8"))
    .join("");
  if (!js.includes("macos-menu-action")) {
    bad.push("产物里没有 macos-menu-action 监听 ⇒ macOS 原生菜单点了没反应");
  } else {
    ok.push("产物含 macos-menu-action 监听");
  }
  for (const cmd of ["get_pet_status", "get_pet_asset", "move_pet_window"]) {
    if (!js.includes(cmd)) bad.push(`产物里没有 ${cmd} ⇒ 桌宠页缺调用`);
  }
}

// ── ③ 自研壳不许调未注册命令（同一判据，指向交付树）──
if (existsSync(join(SHIP, "src"))) {
  const { readdirSync: rd } = await import("node:fs");
  const walk = (dir, out = []) => {
    for (const e of rd(dir, { withFileTypes: true })) {
      const p = join(dir, e.name);
      if (e.isDirectory()) walk(p, out);
      else if (/\.(ts|tsx)$/.test(e.name)) out.push(p);
    }
    return out;
  };
  // ⭐⭐ 2026-10-07 修：注册表真源在 `src/lib.rs` 的 `neobot_commands!` 宏，
  //    **不在** `src/main.rs`（本仓第 6 次「搬家后下游没跟」，见
  //    `lib.rs` 段头的 2026-10-02 P0 复盘）。旧实现读 main.rs ⇒ 只捞到 6 处
  //    非注册引用 ⇒ `registered` 近乎为空 ⇒ 门把 70 多条**真注册**的命令
  //    全报成「未注册」（实测第一条就是 `write_clipboard_text`，它明明在
  //    `lib.rs:53` 注册着）。
  // ⛔ 找不到宏体就 FAIL：缺前提必须失败（`STATUS.md` §4 教训 38）。
  const libRs = readFileSync(join(ROOT, "apps/neobot-desktop/src/lib.rs"), "utf8");
  const macroAt = libRs.indexOf("macro_rules! neobot_commands");
  if (macroAt < 0) {
    bad.push(
      "lib.rs 里找不到 `macro_rules! neobot_commands` —— 注册表真源改名/搬走了，本门须同步（查不到 ≠ 通过）",
    );
    throw new Error("registry macro missing");
  }
  const firstEntry = libRs.indexOf("neobot_desktop::", macroAt);
  const macroBody = libRs
    .slice(firstEntry < 0 ? macroAt : firstEntry, libRs.indexOf("\n}", firstEntry))
    // ⚠️ 先块后行（反过来的话 `//` 落在 /* */ 里会吃掉后面一大段真注册）
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/(^|[^:])\/\/[^\n]*/g, "$1");
  const registered = new Set(
    [...macroBody.matchAll(/neobot_desktop::[a-z_:]+::([a-z_0-9]+)/g)].map((m) => m[1]),
  );
  const files = walk(join(SHIP, "src"));
  const calls = new Set();
  // ⛔⛔ **先剥注释**（2026-10-07 修，本仓第 3 次同款缺陷）。
  //
  // 本仓的注释里**大量**记录「改前是什么样」，于是写着
  //   `// ⛔ 旧形态：切会话就 invoke<ChatMessage[]>('neobot_convo_messages')`
  // 而 `neobot_convo_messages` 这个命令**早已被 `_page` 取代并从注册表删除**。
  // 不剥注释 ⇒ 门报「自研壳调用了未注册命令」，而**代码里根本没有这次调用**。
  //
  // ⓘ 同一族的另外两处已修：`nt_check_api.mjs`（④ 项）与
  //    `nt_check_ui_calls.mjs`。三道门各踩一次 = 修法必须**共用**一个实现，
  //    否则下次还是会漏掉其中一道。
  const stripJsComments = (src) =>
    src.replace(/\/\*[\s\S]*?\*\//g, "").replace(/(^|[^:])\/\/[^\n]*/g, "$1");

  for (const f of files) {
    const src = stripJsComments(readFileSync(f, "utf8"));
    for (const m of src.matchAll(/invoke(?:<[^>]*>)?\(\s*['"]([a-z_:]+)['"]/g)) {
      if (!m[1].startsWith("plugin:")) calls.add(m[1]);
    }
  }
  for (const c of [...calls].sort()) {
    if (!registered.has(c)) bad.push(`自研壳调用未注册命令 ${c}（${files.filter((f) => stripJsComments(readFileSync(f, "utf8")).includes(`'${c}'`)).map((f) => f.slice(SHIP.length + 1)).join(", ")}）`);
  }
  ok.push(`自研壳 ${files.length} 个源文件 / ${calls.size} 种调用，全部已注册`);
}

for (const o of ok) console.log(`  ✓ ${o}`);
if (bad.length) {
  console.error(`\n交付路径门 FAIL（${bad.length} 项）:`);
  for (const b of bad) console.error(`  · ${b}`);
  process.exit(1);
}
console.log("交付路径门 PASS（tauri 指向 neobot-ui + 桌宠页在产物里 + 原生菜单已接线 + 调用全部已注册）");
