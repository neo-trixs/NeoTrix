#!/usr/bin/env node
// 上游 UI 1:1 对账门 —— 防止「壳层 UI 与上游漂移」变成无人发现的事。
//
// ## 为什么需要它（这是本仓吃过亏的地方）
//
// `apps/neobot-desktop/frontend` 是 vendored 上游代码，**本仓只该有 3 处**差异
// （自持分支、品牌、API 页签）。但「只该有」靠人记 = 迟早会多一处或多一处删了，
// 而 vendored 前端全绿、tsc 过、所有门过 —— **没有任何门会因为多改了一个文件而红**。
//
// 关键教训：macOS 上原生菜单缺失这件事，**前端逐字一致、tsc 过、10 个门全绿**，
// 但用户在 macOS 上点不到「设置/关于/更新」。因为差异不在前端，在 Rust 侧
// （`install_macos_menu` 从未被移植），而没有任何门看 Rust 侧与上游的差集。
//
// 本门把这个差集变成可执行的断言：
//   ① vendored 前端逐文件 diff —— 差异清单必须**恰好**等于白名单；
//   ② 白名单里的每一处都必须有登记理由（改了什么、为什么不算漂移）；
//   ③ i18n 只许换品牌串；④ macOS 原生菜单必须已安装；⑤ 窗口 chrome 与上游一致（Rust 侧有 `menu::install` 且 main.rs 调用）。
//
// 用法：`node scripts/ops/nt_check_upstream_1to1.mjs`

import { readFileSync, existsSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const FE = join(ROOT, "apps/neobot-desktop/frontend");

/**
 * 上游参考树的查找顺序：环境变量 → 两个已知落点。
 *
 * ⛔ 曾经写死 `/Users/neo/Downloads/deepseek-harness-desktop-0.19.1`，
 *    2026-10-01 那份被移到 `Downloads/Neo/GitHub/` 下 ⇒ 门找不到参考树，
 *    而它当时的行为是 **打印 SKIP 并 exit 0** —— 也就是「查不了漂移」长得
 *    和「没有漂移」一模一样。这正是本仓反复吃亏的那一类。
 *    现在：找不到 ⇒ **FAIL**，并把查找路径打印出来。
 */
const UP_CANDIDATES = [
  process.env["NB_UPSTREAM"],
  "/Users/neo/Downloads/deepseek-harness-desktop-0.19.1",
  "/Users/neo/Downloads/Neo/GitHub/deepseek-harness-desktop-0.19.1",
].filter(Boolean);
const UP = UP_CANDIDATES.find((p) => existsSync(join(p, "src/layout/index.tsx"))) ?? "";

/**
 * 允许与上游不同的文件，及其理由。
 *
 * ⛔ 往这里加一行 = 声明「这处不算漂移」。理由必须写清**改了什么、为什么不影响 1:1**。
 *    写不出理由的，说明是漂移，要改上游而不是改白名单。
 */
const ALLOWED = {
  "src/layout/components/webview.tsx":
    "自持分支：中间区域渲染本仓 NeoBotRoot 而非 DSH iframe（上游运行时 source/deepseek-harness 是空 submodule）。工具栏/布局代码未动。",
  "src/store/modules/harness/store.ts":
    "新增 selfHosted 字段 + 无服务时跳过依赖安装整段（上游那段围绕 DSH 运行时，本仓无此运行时）。其余 boot 流程未动。",
  "src/ui/dialog/config.tsx":
    "新增第 5 个页签「API」（后端契约可视化）。四元组：Modal 容器、侧栏宽度、nav 按钮类名、Panel.Body 布局全部逐字未改。",
  "src/ui/dialog/about.tsx":
    "powered_by 兜底文案 'DeepSeek Harness Desktop' → 'NeoBot'（品牌，非结构）。其余逐字。",
  "src/i18n/locales/zh-CN.json":
    "4 处 'DeepSeek Harness' → 'NeoBot'（app.open_editor / status.updating / errors.service_start_timeout / ui.iframe_error）。键名与条数完全未动，只换品牌串。",
  "src/i18n/locales/en-US.json":
    "同 zh-CN 的 4 处品牌替换。键名与条数完全未动。",
};

/** i18n 的品牌替换必须是「只换串」，条数与键名不变 —— 单独断言这一点。 */
const I18N_BRAND_ONLY = ["src/i18n/locales/zh-CN.json", "src/i18n/locales/en-US.json"];

const bad = [];
const notes = [];

if (!UP) {
  console.error("1:1 对账门 FAIL：找不到上游参考树 —— 本门无法判断漂移。");
  console.error(`  找过：\n${UP_CANDIDATES.map((p) => `    ${p}`).join("\n")}`);
  console.error("  ⛔ 不再 SKIP/exit 0：「查不了」必须长得像「失败」，不能像「通过」。");
  console.error("  解法：把参考树放回上述任一路径，或设 NB_UPSTREAM 指向它。");
  process.exit(1);
}

// ── ① vendored 前端逐文件 diff ──
const files = execFileSync("find", ["src", "-type", "f"], { cwd: FE, encoding: "utf8" })
  .split("\n")
  .filter(Boolean)
  .filter((f) => /\.(tsx?|css|json|html)$/.test(f));

const seen = new Set();
let same = 0;
for (const f of files) {
  const mine = join(FE, f);
  const theirs = join(UP, f);
  if (!existsSync(theirs)) {
    notes.push(`新增 ${f}`);
    continue;
  }
  seen.add(f);
  const a = readFileSync(theirs);
  const b = readFileSync(mine);
  if (a.equals(b)) {
    same++;
  } else if (!ALLOWED[f]) {
    bad.push(`未登记的差异：${f} —— 门不知道这是有意改动还是漂移`);
  }
}

for (const f of Object.keys(ALLOWED)) {
  if (!seen.has(f)) {
    bad.push(`白名单项 ${f} 已不存在（改动被回退了？删白名单条目）`);
  }
}

// ── ② i18n 只许换品牌串，不许增删条目 ──
// ⛔ 「改 i18n」最常见的漂移是顺手加一条译文（键多了 = 上游没有的能力，
//    却没人审过它指向的 UI 是否真的存在）。条数必须与上游相同。
for (const f of I18N_BRAND_ONLY) {
  const mine = JSON.parse(readFileSync(join(FE, f), "utf8"));
  const theirs = JSON.parse(readFileSync(join(UP, f), "utf8"));
  const mk = Object.keys(mine).sort();
  const tk = Object.keys(theirs).sort();
  if (mk.length !== tk.length) {
    bad.push(`${f} 条目数 ${mk.length} ≠ 上游 ${tk.length}（新增/删除键 = 未审的能力或丢掉的文案）`);
    continue;
  }
  for (const k of tk) {
    if (!(k in mine)) {
      bad.push(`${f} 缺键 ${k}`);
    } else if (mine[k] !== theirs[k]) {
      // 值不同处必须仍含同一个品牌占位：既不许漏替换，也不许顺手改文案。
      const brandOnly = /DeepSeek Harness/.test(theirs[k]) && /NeoBot/.test(mine[k]);
      if (!brandOnly) {
        bad.push(`${f} 的 ${k} 既非纯品牌替换（上游="${theirs[k]}" 本仓="${mine[k]}"）`);
      }
    }
  }
}

// ── ③ macOS 原生菜单（Rust 侧，不可由 ① 覆盖）──
// ⛔ 这条断言是本门存在的核心理由：前端逐字一致 ≠ 壳层行为一致。
const menuPath = join(ROOT, "apps/neobot-desktop/src/menu.rs");
if (!existsSync(menuPath)) {
  bad.push("apps/neobot-desktop/src/menu.rs 不存在 ⇒ macOS 原生菜单缺失，" +
    "navbar.tsx 在 macOS 上会把「文件/运行/帮助」整组隐藏 ⇒ 设置/关于/更新全无入口");
} else {
  const mainRs = readFileSync(join(ROOT, "apps/neobot-desktop/src/main.rs"), "utf8");
  if (!mainRs.includes("menu::install")) {
    bad.push("menu.rs 存在但 main.rs 没有调用 menu::install ⇒ 菜单写了不装，等于没写");
  }
  // 编辑菜单（issue #85）：macOS 挂主菜单后不挂编辑项，⌘C/⌘V/⌘X/⌘A 会被吞。
  const menuRs = readFileSync(menuPath, "utf8");
  for (const item of ["PredefinedMenuItem::copy", "PredefinedMenuItem::paste", "PredefinedMenuItem::select_all"]) {
    if (!menuRs.includes(item)) {
      bad.push(`原生菜单缺 ${item} ⇒ 输入框无法复制/粘贴（上游 issue #85）`);
    }
  }
}

// ── ④ 窗口 chrome：44px 导航栏与交通灯必须融合（Overlay + hiddenTitle）──
const conf = JSON.parse(readFileSync(join(ROOT, "apps/neobot-desktop/tauri.conf.json"), "utf8"));
const main = (conf.app?.windows || []).find((w) => w.label === "main") || {};
if (conf.app?.macOSPrivateApi !== true) {
  bad.push("macOSPrivateApi 未开 ⇒ 桌宠窗不能透明");
}
if (main.titleBarStyle !== "Overlay" || main.hiddenTitle !== true) {
  bad.push(
    `主窗缺 titleBarStyle=Overlay/hiddenTitle=true（实际 ${main.titleBarStyle}/${main.hiddenTitle}）` +
    " ⇒ macOS 上多出一条独立标题栏，且 navbar 的 pl-20 让位给不存在的交通灯",
  );
}
// 交通灯纵向锚点须与 44px 导航栏同线：上游 y = SHELL_NAV_HEIGHT/2 - 2.5 = 19.5。
const tlp = main.trafficLightPosition;
if (!tlp || Math.abs(tlp.y - 19.5) > 0.01) {
  bad.push(`trafficLightPosition.y 应为 19.5（= 44/2 - 2.5，与导航栏 flex 居中同线），实际 ${JSON.stringify(tlp)}`);
}

console.log(`1:1 对账（参考树 ${UP}）：vendored 前端 ${same}/${files.length} 逐字一致；登记差异 ${Object.keys(ALLOWED).length} 处`);
if (notes.length) console.log(`  本仓新增文件 ${notes.length} 个（上游无对应，不计漂移）`);
for (const [f, why] of Object.entries(ALLOWED)) console.log(`  登记 ${f}\n    ${why}`);

if (bad.length) {
  console.error(`\n1:1 对账门 FAIL（${bad.length} 项）:`);
  for (const b of bad) console.error(`  · ${b}`);
  process.exit(1);
}
console.log("1:1 对账门 PASS（前端差集已登记 + macOS 菜单已装 + 窗口 chrome 与上游一致）");
