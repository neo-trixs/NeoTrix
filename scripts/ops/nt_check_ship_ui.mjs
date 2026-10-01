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

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const SHIP = join(ROOT, "apps/neobot-desktop/neobot-ui");
const DIST = join(SHIP, "dist");
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
  const mainRs = readFileSync(join(ROOT, "apps/neobot-desktop/src/main.rs"), "utf8");
  const registered = new Set(
    [...mainRs.replace(/\/\/.*$/gm, "").matchAll(/neobot_desktop::[a-z_:]+::([a-z_0-9]+)/g)].map((m) => m[1]),
  );
  const files = walk(join(SHIP, "src"));
  const calls = new Set();
  for (const f of files) {
    const src = readFileSync(f, "utf8");
    for (const m of src.matchAll(/invoke(?:<[^>]*>)?\(\s*['"]([a-z_:]+)['"]/g)) {
      if (!m[1].startsWith("plugin:")) calls.add(m[1]);
    }
  }
  for (const c of [...calls].sort()) {
    if (!registered.has(c)) bad.push(`自研壳调用未注册命令 ${c}（${files.filter((f) => readFileSync(f, "utf8").includes(`'${c}'`)).map((f) => f.slice(SHIP.length + 1)).join(", ")}）`);
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
