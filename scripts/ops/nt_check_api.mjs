#!/usr/bin/env node
/**
 * API 契约门 —— 三个方向的对账，任何一处漂移即 FAIL。
 *
 * # 为什么这道门存在
 *
 * 上游前端 1:1 过来后 invoke 上游命令，而本仓 Rust 侧起步只有 14 个 `neobot_*`。
 * 这个缺口一开始是**隐性**的：界面白屏，报「命令不存在」，
 * 而「命令不存在」和「命令存在但失败」在调用方看来是同一种东西。
 *
 * ⇒ 三方对账，且**缺口必须被显式登记**：
 *
 *   ① 契约（src/api.rs SPECS）↔ Rust 已注册命令
 *      漏登记 ⇒ 命令能调但没人知道它存在
 *   ② 契约标 Stub/Planned ⇒ Rust 侧刻意没实现；标 Refused ⇒ 必须注册（显式拒绝）
 *      若有人手工加了一个，契约与实现就分家了
 *      （Implemented 的上游同名命令**可以且应当**注册 —— ③ 会查它）
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
//    note 允许两种形态：字符串字面量，或全大写常量（如 DSH_ONLY）。
//    常量形态是故意的 —— 47 个 Stub 共用同一理由，逐条复制会抄错，
//    而理由措辞漂移会让「欠账清单」不可比（见 api.rs DSH_ONLY 注释）。
const apiRs = readFileSync(join(APP, "src/api.rs"), "utf8");
const specRe =
  /ApiSpec::new\(\s*"([^"]+)"\s*,\s*"([^"]*)"\s*,\s*&\[([^\]]*)\]\s*,\s*"([^"]*)"\s*,\s*Status::(\w+)\s*,\s*(?:"([^"]*)"|([A-Z_][A-Z0-9_]*))\s*,?\s*\)/g;
const specs = [];
for (const m of apiRs.matchAll(specRe)) {
  const params = m[3].trim()
    ? m[3].split(",").map((s) => s.trim().replace(/^"|"$/g, "")).filter(Boolean)
    : [];
  specs.push({ name: m[1], category: m[2], params, ret: m[4], status: m[5], note: m[6] ?? m[7] });
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
// ⛔ 先剥行注释：注册块里我写了「见 desktop.rs 段头注释」这类说明，
//    正则会把 `desktop` 与 `rs` 当成命令名 —— 门于是报「Rust 注册了 rs」。
//    门犯的错和它要抓的错是同一种：**把注释里的字当数据**。
const handlerCode = handler.replace(/\/\/.*$/gm, "");

// 注册项形如 `neobot_desktop::platform::open_external_url`。
//
// ⛔ **命令名是 `::` 之后的最后一段**，之前的段全是模块路径。
//   我先前用「硬编码模块名黑名单」来滤（desktop/api/host/core…），
//   那是**错的修法**：每加一个模块就要往黑名单里加一条，忘了就报假警。
//   ⇒ 现在按结构判断：取最后一段。
const registered = new Set(
  [...handlerCode.matchAll(/\b([a-z][a-z0-9_]*(?:::[a-z][a-z0-9_]*)*)\b/g)]
    .map((m) => m[1].split("::").pop())
    // 宏名与关键字不是命令
    .filter((c) => c !== "generate_handler" && !c.endsWith("_handler")),
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

// ── ② 契约标 Stub/Planned ⇒ Rust 侧刻意没实现 ──
//     若有人手工加了一个，契约与实现就分家了。
//     （Implemented 的上游同名命令是正常注册的 —— ③ 覆盖它们。）
// ⛔ `Refused` 是**必须注册**的：它表示「已注册、但实现是一句明确的拒绝」。
//    那是「决定不做」与「让界面静默炸掉」之间的区别 —— 上游 11 处 invoke
//    落在本仓不提供的体系上，不注册它们时界面上是空列表 + command not found。
for (const s of specs) {
  if (s.status === "Implemented" || s.status === "Refused") continue;
  if (registered.has(s.name)) {
    problems.push(
      `契约说 ${s.name} 是 ${s.status}，但 main.rs 注册了它` +
        " ⇒ 要么补实现并改状态，要么撤掉注册" +
        "（若实现是「一句拒绝」，状态写 Refused —— 见 api.rs 的 Status 定义）",
    );
  }
}
// 反向：标了 Refused 却没注册 = 承诺没兑现（界面又会静默炸回去）。
for (const s of specs) {
  if (s.status === "Refused" && !registered.has(s.name)) {
    problems.push(`契约说 ${s.name} 是 Refused（已注册的显式拒绝），但 main.rs 没有注册它`);
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

// ── ⑤ 数字守恒：upstream_total == 非 neobot_ 条目 + 未展开 ──
//     ⛔ 旧口径曾是「upstream_total == 前端实际 invoke 的上游命令数」，
//     那是错的：前端只调 78 个上游命令中的一部分是常态
//     （如 pet 窗没打开时 pet 系命令一次都不调），
//     按它对账会逼着把「没调」当成「缺口」。
//     新口径对的是**分母**：上游 Rust 侧 95 个命令，一个都不漏
//     （非 neobot_ 条目 + 未展开 == upstream_total，Rust 侧同名测试亦守）。
const nonNeobotSpecs = specs.filter((s) => !s.name.startsWith("neobot_")).length;
const declaredTotal = Number((apiRs.match(/upstream_total:\s*(\d+)/) || [])[1] || 0);
if (declaredTotal !== 0 && declaredTotal !== nonNeobotSpecs + unlisted.length) {
  problems.push(
    `契约声明 upstream_total=${declaredTotal}，但非 neobot_ 条目 ${nonNeobotSpecs} + 未展开 ${unlisted.length} = ${nonNeobotSpecs + unlisted.length}` +
      " ⇒ 有上游命令没登记（这是本仓反复踩的一类：清单上的数字没人核对）",
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

// ── ⑦ 参数对账：契约声明的每个参数名，必须出现在 Rust 的 fn 签名里 ──
//
// ⛔ **这条是被真实的错误逼出来的。** 契约里 `log_frontend` 写成 2 个参数
//    （level, message），而前端实际 invoke 传的是 3 个
//    （`{ level, target, message }`，见 src/utils/logger.ts:81）。
//    ①②③④⑤ 全绿 —— 因为它们只对**命令名**，不对**参数**。
//
// ⇒ 「按名传参」的 Tauri 接口，参数名写错的后果是**静默的**：
//   Rust 侧少一个字段就反序列化失败，但那个 invoke 是 `.catch(() => {})`
//   吞掉的 ⇒ 日志静默不落盘，界面无任何异常。
//   只靠「跑一次看看」发现不了，因为「不报错」正是症状。
const cmdsRs = readFileSync(join(APP, "src/commands.rs"), "utf8") +
  readFileSync(join(APP, "src/desktop.rs"), "utf8") +
  readFileSync(join(APP, "src/core.rs"), "utf8") +
  readFileSync(join(APP, "src/platform.rs"), "utf8") +
  readFileSync(join(APP, "src/pet.rs"), "utf8") + apiRs;
for (const s of specs) {
  if (s.status !== "Implemented") continue;
  // 找该命令的 fn 签名（允许跨行，允许 #[tauri::command] 在上一行）
  const sigRe = new RegExp(
    `fn\\s+${s.name}\\s*\\(([\\s\\S]{0,400}?)\\)\\s*(->[^\\{]*)?\\{`,
  );
  const m = cmdsRs.match(sigRe);
  if (!m) continue; // 签名形态特殊（如 main.rs 里直接引用），跳过而非误报
  const sig = m[1];
  for (const want of s.params) {
    if (!new RegExp(`\\b${want}\\s*:`).test(sig)) {
      problems.push(
        `契约说 ${s.name} 有参数 ${want}，但 Rust fn 签名里找不到` +
          " ⇒ 按名传参会反序列化失败，而前端常把 invoke 的错误 .catch 掉" +
          " ⇒ 症状是「静默不生效」，不是报错",
      );
    }
  }
  // ⛔ **反向**也要查：Rust 有、契约没写。
  //   只查正向的话，「契约少写一个参数」这个**最常见的错法**反而漏 ——
  //   而那正是 log_frontend 真实的错法（契约 2 个、实际 3 个）。
  //   变异验证：摘掉契约里的 target ⇒ 门必须 FAIL。
  for (const decl of sig.matchAll(/(\w+)\s*:\s*(?:String|Option<[^>]+>|bool|u64|i64|Value|\w+)/g)) {
    const name = decl[1];
    // 跳过 Tauri 注入的固定形参（AppHandle / State / tauri::…）
    if (/^(app|state|window|webview)$/i.test(name)) continue;
    if (name === "tauri" ) continue;
    if (!s.params.includes(name)) {
      problems.push(
        `${s.name} 的 Rust 签名有参数 ${name}，契约 SPECS 没写` +
          " ⇒ 契约漏项，读这份清单的人会以为该接口不收它",
      );
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
    `显式拒绝 ${byStatus.Refused || 0} / 不做 ${byStatus.Stub || 0} / 待做 ${byStatus.Planned || 0}；` +
    `未展开上游 ${unlisted.length}；前端调用 ${frontendCalls.size}）`,
);
