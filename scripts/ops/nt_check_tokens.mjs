#!/usr/bin/env node
// Token 解析门 —— 纯静态、零浏览器。
//
// # 为什么需要这道门
//
// 2026-09-30 实测事故（两例，都是我自己犯的）：
//
//   ① 我在 `neobot/src/ui/sheet.css` 里凭感觉写了 20 个 token 名
//      （`--nb-surface` / `--nb-radius-md` / `--nb-fs-md` / `--nb-dur-fast` …），
//      而 `tokens.css` 里真实的名字是 `--nb-bg-surface` / `--nb-radius-control` …
//   ② `tokens.css` 自己写着 `--nb-c-accent-wash: var(--skype-a12)`，
//      而 `styles.css` 里只有 `--skype-a08`，**`--skype-a12` 从来不存在**。
//
// 两例的共同点，也是它们能活下来的原因：
//
//   **CSS 变量解析失败不报错。** `var(--不存在的)` 静默变成「无效」，
//   该属性回落到 inherit 或初始值 —— 框还在、按钮还在，只是**没有颜色**。
//   浏览器控制台不报错，build 不报错，typecheck 更是管不到 CSS。
//   于是「20 个 token 全错」这种量级的错误，肉眼看截图只会觉得「颜色有点淡」。
//
// 这道门把「引用的 token 必须真的存在」变成可判定的断言。
//
// # 判据
//
//   ① `--nb-*` 必须在 tokens.css 里有定义（含深色模式下的重定义）
//   ② tokens.css 里指向**旧 token**（非 `--nb-`）的引用，目标必须在 styles.css 里有定义
//   ③ 允许链式：primitive 允许再指向别的旧 token，但每一跳都必须落地
//
// 不查「值是否好看」「对比度是否够」—— 那是另一件事，别混进这道门。

import { readFileSync, existsSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const TOKENS = join(ROOT, "apps/neobot-desktop/frontend/src/ui/tokens.css");
// 本仓 app 的 token 层是**自足**的（primitive 就是真色），没有 legacy styles.css。
// 所以「底层缺失」这条判据改为：semantic 指向的 --nb-c-* 必须在 tokens.css 里定义。
const BASE = TOKENS;
const UI_DIR = join(ROOT, "apps/neobot-desktop/frontend/src/ui");

const read = (p) => (existsSync(p) ? readFileSync(p, "utf8") : "");

/**
 * ⛔ 扫描前**必须剥掉 CSS 注释**。
 *
 * 2026-09-30 实测：本文件从另一个仓迁过来时，第一次跑就报
 *   「--nb-c-accent → var(--skype)，但 styles.css 里没有 --skype」
 * 而 tokens.css 里根本没这句 —— 它在**注释里**，是解释「另一个仓的 token 是
 * 派生的」那句话的一部分。那边能过是因为那边 styles.css 恰好真定义了
 * --skype，**把 bug 掩盖了**。
 *
 * 教训与仓库既有纪律同款：陈旧/被掩盖的门比没有门更危险。
 * 注释里的 token 不是定义，扫之前就该剥掉。
 */
const stripCssComments = (css) => css.replace(/\/\*[\s\S]*?\*\//g, "");
const tokensCss = stripCssComments(read(TOKENS));
const baseCss = stripCssComments(read(BASE));

if (!tokensCss) {
  console.error("Token 门 FAIL: neobot/src/tokens/tokens.css 不存在");
  process.exit(1);
}

const definedNb = new Set([...tokensCss.matchAll(/^\s*(--nb-[\w-]+)\s*:/gm)].map((m) => m[1]));
const definedBase = new Set([...baseCss.matchAll(/^\s*(--[\w-]+)\s*:/gm)].map((m) => m[1]));
const problems = [];

// ── ①②③ tokens.css 自身：每个 --nb-* 的值最终必须落到 styles.css ──
for (const m of tokensCss.matchAll(/(--nb-[\w-]+)\s*:\s*([^;]+);/g)) {
  const [, name, rawValue] = m;
  for (const ref of rawValue.matchAll(/var\((--[\w-]+)/g)) {
    const target = ref[1];
    if (target.startsWith("--nb-")) {
      if (!definedNb.has(target)) problems.push(`tokens.css: ${name} 引用了未定义的 ${target}`);
    } else if (!definedBase.has(target)) {
      problems.push(`tokens.css: ${name} → var(${target})，但 styles.css 里没有 ${target}`);
    }
  }
}

// ── ① UI 层：只能引用 semantic token，且必须已定义 ──
// ⛔ 第一版这里写的是 `require("node:fs").readdirSync` —— ESM 里没有 require，
//    于是 catch 吞掉异常、uiFiles 恒为空，门报「UI 层 0 个文件全部走语义层」并 PASS。
//    **一个从不读文件的门，比没有门更危险**：它给出的是虚假的绿灯。
//    这正是仓库纪律里「陈旧门记录会让下一个 agent 去『修』正确代码」的同款。
const uiFiles = existsSync(UI_DIR)
  ? readdirSync(UI_DIR).filter((f) => f.endsWith(".css"))
  : [];
if (existsSync(UI_DIR) && uiFiles.length === 0) {
  console.error("Token 门 FAIL: ui/ 存在但没读到任何 .css —— 读目录这一步坏了，门会虚假通过。");
  process.exit(1);
}
for (const f of uiFiles) {
  const css = stripCssComments(read(join(UI_DIR, f)));
  for (const m of css.matchAll(/var\((--[\w-]+)/g)) {
    const t = m[1];
    if (t.startsWith("--nb-")) {
      if (!definedNb.has(t)) problems.push(`neobot/src/ui/${f} 引用了未定义的 ${t}`);
    } else {
      // UI 层直连旧 token = 绕过两层结构，语义重新绑死在具体色上
      problems.push(`apps/neobot-desktop/frontend/src/ui/${f} 直接引用了旧 token ${t}（应经 --nb-* 语义层）`);
    }
  }
  // tokens.css 是 **primitive 层本身**，hex 在那里合法（那就是它存在的意义）。
  // 只对「组件层」查裸 hex —— 组件层出现 hex 才叫绕过语义层。
  if (f === "tokens.css") continue;
  // ② 裸 hex（#RRGGBB / #RGB）漂移
  //    两层 token 的全部意义就是「颜色只在 tokens.css 出现一次」。组件里写
  //    `#6B7A82` 就是把语义重新钉死成具体色，换主题时它不会跟着变。
  //    ⛔ 剔除 6 位 hex 在 URL 片段等语境（这里没有）与注释（已剥）。
  for (const m of css.matchAll(/#[0-9A-Fa-f]{3,8}\b/g)) {
    const hex = m[0];
    // 允许透明度写法 rgb(...)/rgba(...) 不含 #，此处只管 #
    problems.push(`apps/neobot-desktop/frontend/src/ui/${f} 含裸 hex ${hex}（应经 --nb-* 语义层）`);
  }
}

if (problems.length) {
  console.error(`Token 门 FAIL（${problems.length} 项）:`);
  for (const p of problems) console.error("  · " + p);
  console.error("\n注意：CSS 变量解析失败**不报错**，只会让该属性失效（颜色变透明/继承）。");
  process.exit(1);
}
console.log(
  `Token 门 PASS（--nb-* ${definedNb.size} 个；UI 层 ${uiFiles.length} 个文件全部走语义层）`,
);
