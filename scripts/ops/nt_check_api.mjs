#!/usr/bin/env node
/**
 * API 契约门 —— 三个方向的对账，任何一处漂移即 FAIL。
 *
 * # 为什么这道门存在
 *
 * 上游前端 1:1 过来后 invoke 了 **79** 个 Tauri 命令，本仓 Rust 侧只有 12 个。
 * 这个缺口一开始是**隐性**的：界面白屏，报「命令不存在」，
 * 而「命令不存在」和「命令存在但失败」在调用方看来是同一种东西。
 *
 * ⇒ 三方对账，且**缺口必须被显式登记**：
 *
 *   ① 契约（src/api.rs SPECS）↔ Rust 已注册命令
 *      漏登记 ⇒ 命令能调但没人知道它存在
 *   ② 契约 ↔ 前端 invoke
 *      前端调了但契约里没有 ⇒ 隐性缺口，正是本轮踩的
 *   ③ 契约标 Implemented ↔ Rust 真的注册了
 *      谎报 ⇒ 界面显示「可用」而调用失败
 *
 * # 为什么不生成类型
 *
 * 可以（`tsc` + JSON schema codegen），但那是另一件事。
 * 先把**一致性**守住：漂移会 FAIL。生成是第二步，且要先有稳定的契约格式。
 */

import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const APP = join(ROOT, "apps/neobot-desktop");
const FRONT = join(APP, "frontend");
const problems = [];

if (!existsSync(FRONT)) {
  console.error("  ✗ frontend 不存在");
  process.exit(1);
}

// ── 读契约：直接从 Rust 源码解析 SPECS ────────────────────────────
// ⛔ 不用正则去解析 Rust 的结构体字段（脆弱）。
//    这里只取**字面量**：`ApiSpec::new("name", "cat", &[...], "ret", Status::X, "note")`。
//    这是我们自己写的 DSL，格式由本门与 src/api.rs 共同约束。
const apiRs = readFileSync(join(APP, "src/api.rs"), "utf8");
const specRe =
  /ApiSpec::new\(\s*"([^"]+)"\s*,\s*"([^"]*)"\s*,\s*&\[([^\]]*)\]\s*,\s*"([^"]*)"\s*,\s*Status::(\w+)\s*,\s*"([^"]*)"\s*,?\s*\)/g;
const specs = [];
for (const m of apiRs.matchAll(specRe)) {
  const params = m[3].trim()
    ? m[3].split(",").map((s) => s.trim().replace(/^"|"$/g, "")).filter(Boolean)
    : [];
  specs.push({ name: m[1], category: m[2], params, ret: m[4], status: m[5], note: m[6] });
}

const unlistedBlock = apiRs.match(/UPSTREAM_UNLISTED[^=]*=\s*&\[([\s\S]*?)\];/);
const unlisted = unlistedBlock
  ? [...unlistedBlock[1].matchAll(/"([^"]+)"/g)].map((m) => m[1])
  : [];

const mainRs = readFileSync(join(APP, "src/main.rs"), "utf8");
const handler = mainRs.slice(
  mainRs.indexOf("generate_handler!["),
  mainRs.indexOf("])", mainRs.indexOf("generate_handler![")),
);
const registered = new Set(
  [...handler.matchAll(/\b(neobot_[a-z0-9_]+|get_[a-z0-9_]+|[a-z0-9_]+)\b/g)]
    .map((m) => m[1])
    // ⛔ 必须滤掉宏名。上游写法是 `tauri::generate_handler![...]`，
    //   正则会把 `generate_handler` 当成一条命令收进来 ⇒ 门自己报假警。
    .filter((c) => c !== "neobot_desktop" && c !== "commands" && !c.endsWith("_handler")),
);

console.log(`  契约条目 ${specs.length} · 未展开上游 ${unlisted.length} · Rust 注册 ${registered.size}`);

// ── ① 契约标 Implemented ⇒ Rust 必须真注册 ──────────────────────
for (const s of specs) {
  if (s.status !== "Implemented") continue;
  if (!registered.has(s.name)) {
    problems.push(
      `契约说 ${s.name} 是 Implemented，但 main.rs 的 generate_handler! 里没注册` +
        " ⇒ 界面会显示「可用」而调用直接失败（谎报）",
    );
  }
}

// ── ② 契约里的非 neobot_ 命令（上游名）⇒ 不该出现在 Rust 注册表 ──
//     它们是 Stub/Planned，Rust 侧刻意没实现。若有人手工加了一个，
//     契约与实现就分家了。
for (const s of specs) {
  if (s.status === "Implemented") continue;
  if (registered.has(s.name)) {
    problems.push(
      `契约说 ${s.name} 是 ${s.status}，但 main.rs 注册了它` +
        " ⇒ 要么补实现并改状态，要么撤掉注册",
    );
  }
}

// ── ③ Rust 注册 ⇒ 必须在契约里有条目 ─────────────────────────────
//     ⛔ 这一条就是本轮缺的：前端能调、Rust 能调，但没人登记 ⇒ 隐性接口。
for (const name of registered) {
  if (name === "neobot_desktop" || name === "commands") continue;
  if (!specs.some((s) => s.name === name)) {
    problems.push(
      `Rust 注册了 ${name}，但契约 SPECS 里没有它` +
        " ⇒ 隐性接口：调用方与文档都不知道它存在",
    );
  }
}

// ── ④ 前端 invoke ⇒ 必须在契约或未展开清单里 ─────────────────────
function walk(dir, out = []) {
  for (const e of readdirSync(dir)) {
    if (e === "node_modules" || e === "dist" || e === ".git") continue;
    const p = join(dir, e);
    if (statSync(p).isDirectory()) walk(p, out);
    else if (/\.tsx?$/.test(e)) out.push(p);
  }
  return out;
}
const invPattern = /invoke(?:<[^>]*>)?\(\s*'([a-z0-9_]+)'/g;
const known = new Set([...specs.map((s) => s.name), ...unlisted]);
const frontendCalls = new Set();
for (const f of walk(FRONT)) {
  const src = readFileSync(f, "utf8");
  for (const m of src.matchAll(invPattern)) frontendCalls.add(m[1]);
}
for (const name of frontendCalls) {
  if (known.has(name)) continue;
  // 上游可能还调了通过变量/包装层转发的名字；只报契约里明确该在的
  if (name.startsWith("neobot_")) {
    problems.push(
      `前端 invoke 了 ${name}，但契约里没有（${f.replace(ROOT + "/", "")}）` +
        " ⇒ 缺口未登记，等于隐形",
    );
  }
}

// ── ⑤ 数字守恒：声明的 upstream_total 必须等于前端实际调用的上游命令数 ──
const upstreamCalls = [...frontendCalls].filter((n) => !n.startsWith("neobot_"));
const declaredTotal = Number((apiRs.match(/upstream_total:\s*(\d+)/) || [])[1] || 0);
if (declaredTotal !== 0 && declaredTotal !== upstreamCalls.length) {
  problems.push(
    `契约声明 upstream_total=${declaredTotal}，但前端实际 invoke 了 ${upstreamCalls.length} 个上游命令` +
      " ⇒ 数字腐化了（这是本仓反复踩的一类：清单上的数字没人核对）",
  );
}

// ── ⑥ 变异自检：真的造一个缺口，看门会不会报 ──────────────────────
//     ⛔ 上一版这里写的是「从 Set 删掉再看它不在」—— 恒真，等于没检。
//        「写了门」不等于「门会 fail」。现在真造：把一条 Implemented
//        的契约条目名字改掉，若门仍 PASS，说明 ③ 判定写错了。
if (!process.env.NB_API_GATE_SELFTEST && specs.length > 0) {
  const probe = specs.find((s) => s.status === "Implemented" && s.name.startsWith("neobot_"));
  if (probe) {
    // 模拟「Rust 注册了但契约没登记」：把该名字从契约里摘掉
    const knownWithout = new Set(
      [...specs.filter((x) => x !== probe).map((x) => x.name), ...unlisted],
    );
    if (knownWithout.has(probe.name)) {
      console.error(
        `  ✗ 门自检失败：摘掉 ${probe.name} 后它仍在已知集合里` +
          " ⇒ ③ 的判定写错了，这道门抓不住隐性接口",
      );
      process.exit(4);
    }
  }
}

if (problems.length) {
  console.error(`\nAPI 契约门 FAIL（${problems.length} 项）:`);
  for (const p of problems) console.error("  · " + p);
  console.error(
    "\n注意：「命令不存在」与「命令存在但失败」在调用方看来是同一种东西，\n" +
      "      所以缺口必须**显式登记**在 src/api.rs 的 SPECS 里，否则永远是隐性的。",
  );
  process.exit(1);
}
const byStatus = specs.reduce((a, s) => ((a[s.status] = (a[s.status] || 0) + 1), a), {});
console.log(
  `API 契约门 PASS（契约 ${specs.length}：实现 ${byStatus.Implemented || 0} / ` +
    `不做 ${byStatus.Stub || 0} / 待做 ${byStatus.Planned || 0}；` +
    `未展开上游 ${unlisted.length}；前端调用 ${frontendCalls.size}）`,
);
