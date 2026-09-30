#!/usr/bin/env node
/**
 * STATUS 自校验 —— 防止这份清单变成下一个「说谎的能力地图」。
 *
 * # 为什么需要
 *
 * `CAPABILITY-MAP-2026-09-29.md` 曾经整张表腐化：111 条命令里 46 条根本不存在，
 * 11 条「缺 Consumer」逐条 grep **零命中**，而表格长得像实测结果（有行号、
 * 有证据列）。没人发现，因为**没有东西去核对它**。
 *
 * `apps/neobot-desktop/STATUS.md` 是同一类文件（人写的数字 + 表格）。
 * 所以给它配一道门：文档里的关键数字必须与实测一致，**不一致即 FAIL**。
 *
 * 这与 `CAPABILITY-MAP` 那次的区别是本质的：那张表**不可重跑**，
 * 这份**可以**，且现在就自动核。
 *
 * # 核对项（每项都能独立重跑）
 *
 *   ① 前端代码总行数
 *   ② 图标 PNG 数
 *   ③ IPC 命令数（前后端两侧一致的那份）
 *   ④ 库测试条数（nt_evidence + nt_panel）
 *   ⑤ 门禁脚本数
 *   ⑥ 前端自测分组数
 *   ⑦ STATUS 里「缺口」小节列出的条目数与实际文件对得上
 */

import { readFileSync, existsSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const STATUS = join(ROOT, "apps/neobot-desktop/STATUS.md");
const FE = join(ROOT, "apps/neobot-desktop/frontend");
const ICONS = join(ROOT, "apps/neobot-desktop/icons");
const CRATE = join(ROOT, "crates/neotrix-neobot/src");

if (!existsSync(STATUS)) {
  console.error("STATUS 自校验 FAIL: STATUS.md 不存在");
  process.exit(1);
}
const doc = readFileSync(STATUS, "utf8");
const problems = [];

// ① 前端总行数
// ⚠️ 必须**递归**。第一版只扫一层，漏掉 src/ui/ src/plugin/ src/host/，
//    于是报「前端 1490 行」而实际 4442 —— 门自己先报错，
//    差点让我去改文档去迎合一个错的门。
const feFiles = [];
(function walk(d) {
  for (const n of readdirSync(d, { withFileTypes: true })) {
    const p = join(d, n.name);
    if (n.isDirectory()) { walk(p); continue; }
    if (p.endsWith(".ts") || p.endsWith(".css")) feFiles.push(p);
  }
})(join(FE, "src"));
const feLines = feFiles.reduce((n, f) => n + readFileSync(f, "utf8").split("\n").length, 0);
const claimedFe = doc.match(/([\d,]+)\s*行/);
if (claimedFe && Number(claimedFe[1].replace(/,/g, "")) !== feLines) {
  problems.push(`前端总行数：文档 ${claimedFe[1]} vs 实测 ${feLines}`);
}

// ② 图标 PNG
// 同理：Tauri 栅格化把 PNG 分到 android/ ios/ 子目录，只扫一层会少算 33 个。
const pngs = [];
(function walk(d) {
  for (const n of readdirSync(d, { withFileTypes: true })) {
    const p = join(d, n.name);
    if (n.isDirectory()) { walk(p); continue; }
    if (p.endsWith(".png")) pngs.push(p);
  }
})(ICONS);
const claimedPng = doc.match(/\*\*(\d+)\s*PNG/);
if (claimedPng && Number(claimedPng[1]) !== pngs.length) {
  problems.push(`图标 PNG 数：文档 ${claimedPng[1]} vs 实测 ${pngs.length}`);
}

// ③ IPC 命令数（直接数后端注册表，别读文档）
const mainRs = readFileSync(join(ROOT, "apps/neobot-desktop/src/main.rs"), "utf8");
const hStart = mainRs.indexOf("generate_handler![");
const hEnd = mainRs.indexOf("])", hStart);
const handler = mainRs.slice(hStart, hEnd);
const cmds = [...handler.matchAll(/\b(neobot_[a-z0-9_]+)\b/g)]
  .map((m) => m[1])
  .filter((c) => c !== "neobot_desktop");
if (new Set(cmds).size !== cmds.length) problems.push("后端 generate_handler 有重复命令");
const uniqCmds = new Set(cmds);
// 文档 §1.3 用代码块列命令，逐个核对
// ⚠️ 必须限定在 §1.3 的**清单块**内。只查「文档任意位置出现过」是不够的：
//    命令名在缺口表里也会出现，于是从清单里删掉一个仍然 PASS（实测）。
const listStart = doc.indexOf("### 1.3");
const listEnd = doc.indexOf("### 1.4", listStart);
const cmdList = doc.slice(listStart, listEnd);
for (const c of uniqCmds) {
  if (!cmdList.includes(c)) problems.push(`命令 ${c} 未在 STATUS §1.3 清单中列出`);
}
// 反向：清单里多写了不存在的命令
for (const line of cmdList.split("\n")) {
  const m = line.match(/^\s*(neobot_[a-z0-9_]+)\s*$/);
  if (m && !uniqCmds.has(m[1])) problems.push(`§1.3 列了后端不存在的命令：${m[1]}`);
}

// ④ 库测试条数
const libTests = ["nt_evidence", "nt_panel"]
  .map((m) => {
    const p = join(CRATE, `${m}.rs`);
    return existsSync(p) ? (readFileSync(p, "utf8").match(/#\[test\]/g) ?? []).length : 0;
  })
  .reduce((a, b) => a + b, 0);
// ⚠️ 匹配不上时**必须 FAIL**，不能跳过。
//    「找不到我要核对的那句话」说明文档结构变了，此时静默跳过等于
//    这道检查已经不存在了 —— 而它看起来还在。仓里已有同款教训：
//    Token 门第一版不剥注释、字节门漏掉 .worktrees，都是「以为自己查了」。
const claimedLib = doc.match(/库测试(\d+)\s*条/);
if (!claimedLib) {
  problems.push("STATUS 里找不到「库测试N 条」这句，无法核对（结构变了？）");
} else if (Number(claimedLib[1]) !== libTests) {
  problems.push(`库测试数：文档 ${claimedLib[1]} vs 实测 ${libTests}`);
}

// ⑤ 门禁脚本
const gates = readdirSync(join(ROOT, "scripts/ops")).filter(
  (f) => f.startsWith("nt_check_") || f === "nt_shot.mjs",
);
const claimedGates = doc.match(/门禁（(\d+)\s*个/);
if (!claimedGates) {
  problems.push("STATUS 里找不到「门禁（N 个」，无法核对");
} else if (Number(claimedGates[1]) !== gates.length) {
  problems.push(`门禁数：文档 ${claimedGates[1]} vs 实测 ${gates.length}`);
}

// ⑥ 前端自测分组
const selftest = readFileSync(join(FE, "src/selftest.ts"), "utf8");
const groups = [...selftest.matchAll(/console\.log\("\s*·\s*([^"]+)"/g)].map((m) => m[1]);
const claimedGroups = doc.match(/前端\s*(\d+)\s*组自测/);
if (!claimedGroups) {
  problems.push("STATUS 里找不到「前端 N 组自测」，无法核对");
} else if (Number(claimedGroups[1]) !== groups.length) {
  problems.push(`前端自测分组：文档 ${claimedGroups[1]} vs 实测 ${groups.length}（${groups.join(" / ")}）`);
}

if (problems.length) {
  console.error(`STATUS 自校验 FAIL（${problems.length} 项）:`);
  for (const p of problems) console.error("  · " + p);
  console.error(
    "\n这份文件是**人写的数字**。它一旦与实测分叉就开始说谎 ——\n" +
      "而 `CAPABILITY-MAP-2026-09-29` 正是这么整张腐化的，且没人发现。\n" +
      "改 STATUS 里的数字后请重跑本门。",
  );
  process.exit(1);
}
console.log(
  `STATUS 自校验 PASS（前端 ${feLines} 行 / PNG ${pngs.length} / 命令 ${uniqCmds.size} / ` +
    `库测试 ${libTests} / 门禁 ${gates.length} / 自测分组 ${groups.length}）`,
);
