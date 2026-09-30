#!/usr/bin/env node
/**
 * 前后端命令对齐门 —— 静态、零构建、秒级。
 *
 * # 为什么必须有这道门
 *
 * 本项目的一条主要 bug 来源是「**前端调了后端没有的命令**」。
 * 真实发生过三次：
 *   ① `neobot_evidence_summary` / `neobot_send` 在另一仓侧，本仓的库没有
 *      ⇒ 界面点了必然失败，而「看起来在工作」比明确报错更贵
 *   ② `nt_cmd_files.rs` / `nt_cmd_sidebar.rs` 被文档引用，但**文件不存在** ⇒
 *      一份能力地图列了 11 条「缺 Consumer」，逐条 grep **零命中**
 *   ③ `apps/neobot-desktop` 整个 app 不在 workspace members ⇒
 *      `cargo check -p neobot-desktop` 报「did not match any packages」
 *
 * 三次都不是「写错了」，是**两侧各自自洽、没人对着看**。
 * tsc 只看类型表，cargo 只看注册表，两边都绿，而产品是坏的。
 *
 * # 判据（三条都要）
 *
 *   ① 前端 `ipc.ts` 的每个命令键，必须出现在 Rust `generate_handler!` 里
 *   ② Rust `generate_handler!` 的每个命令，必须在前端命令表里有对应条目
 *      （反查：注册了但前端调不到 = 又一个「用户点不到的能力」）
 *   ③ 两侧都不得为空 —— 任一侧解析失败即 FAIL，不能静默跳过
 */

import { readFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const IPC = join(ROOT, "apps/neobot-desktop/frontend/src/ipc.ts");
const MAIN = join(ROOT, "apps/neobot-desktop/src/main.rs");

const problems = [];

for (const [p, what] of [[IPC, "ipc.ts"], [MAIN, "main.rs"]]) {
  if (!existsSync(p)) {
    console.error(`前后端对齐门 FAIL: ${what} 不存在（${p}）`);
    process.exit(1);
  }
}

const ipc = readFileSync(IPC, "utf8");
const main = readFileSync(MAIN, "utf8");

// ── 前端命令表 ──
const tableStart = ipc.indexOf("export interface Commands {");
const tableEnd = ipc.indexOf("\n}", tableStart);
if (tableStart < 0 || tableEnd < 0) {
  console.error("前后端对齐门 FAIL: 在 ipc.ts 里找不到 Commands 表");
  process.exit(1);
}
const table = ipc.slice(tableStart, tableEnd);
const frontend = [...table.matchAll(/^\s{2}(neobot_[a-z0-9_]+):/gm)].map((m) => m[1]);

// ── Rust generate_handler! ──
const hStart = main.indexOf("generate_handler![");
const hEnd = main.indexOf("])", hStart);
if (hStart < 0 || hEnd < 0) {
  console.error("前后端对齐门 FAIL: 在 main.rs 里找不到 generate_handler!");
  process.exit(1);
}
const handler = main.slice(hStart, hEnd);
// 取 `::` 之后的最后一段：注册表里写的是 `neobot_desktop::commands::neobot_send`，
// 命令名是尾段。
const CRATE = "neobot_desktop";
const backend = [...handler.matchAll(/\b(neobot_[a-z0-9_]+)\b/g)]
  .map((m) => m[1])
  // 去掉模块/crate 前缀留下的误匹配
  .filter((c) => c !== CRATE);

if (frontend.length === 0) problems.push("前端命令表解析出 0 条 —— 门会虚假通过");
if (backend.length === 0) problems.push("后端 generate_handler 解析出 0 条 —— 门会虚假通过");

const fb = new Set(backend);
const ff = new Set(frontend);

for (const c of frontend) {
  if (!fb.has(c)) problems.push(`前端调用了后端未注册的命令：${c}`);
}
for (const c of backend) {
  if (!ff.has(c)) problems.push(`后端注册了前端命令表里没有的命令：${c}（用户点不到）`);
}

// 重复注册
const dup = backend.filter((c, i) => backend.indexOf(c) !== i);
if (dup.length) problems.push(`generate_handler 里有重复项：${[...new Set(dup)].join(", ")}`);

if (problems.length) {
  console.error(`前后端对齐门 FAIL（${problems.length} 项）:`);
  for (const p of problems) console.error("  · " + p);
  console.error(
    "\n这类问题两侧各自都绿：tsc 只看类型表，cargo 只看注册表。\n" +
      "只有把两张表对着看才抓得到 —— 而人不会每次都对着看。",
  );
  process.exit(1);
}
console.log(`前后端对齐门 PASS（${frontend.length} 个命令两侧完全一致）`);
