#!/usr/bin/env node
// 字节安全门 —— 纯静态、零构建。
//
// # 为什么需要这道门
//
// 2026-09-30 实测：全仓 6 个文件、9 处注释里带着 **U+FFFD 替换字符**（即 `\uFFFD`），
// 且**全部已在 git HEAD 里**（不是当天引入的）。
//
// 危害不在于那 9 个字符本身（都在注释里，不影响运行），而在于：
//
//   1. **它是写文件环节有 bug 的信号。** 替换字符只可能来自「按字节写文本时
//      把多字节序列截断/覆盖」，也就是 CJK 内容在某条写入路径上被切坏。
//      注释能被切坏，字符串字面量同样能被切坏 —— 而后者会静默改变行为。
//   2. **它会骗过所有常规门。** tsc 不看注释编码，cargo 不看，CSS 也不看。
//      仓库里已有一整套门（token / cap-gate / layout / audit-dead），
//      唯独没有一道看字节的。
//
// 仓库纪律里早有同类教训（父仓 AGENTS.md：全角标点吃字节），但**没有门**，
// 于是同一个坑在不同时期被踩了至少两次。这道门就是那个缺失的门。
//
// # 判据
//
//   任何源文件/文档里不得出现 U+FFFD（`\uFFFD`）。
//   U+FFFD 几乎不可能是作者有意写进去的，所以零容忍是对的。
//
// ⛔ 本文件自身**不得**含字面 U+FFFD，否则门永远红自己（第一版就这么翻车了：
//    注释里为了说明「替换字符长什么样」直接贴了那个字符）。故一律用 `\uFFFD` 转义。
//
// # 刻意不查的
//
//   · 是否 UTF-8 可解码 —— Python 的 `read_text(errors="replace")` 会在解码
//     失败时**自动**产生 U+FFFD，所以「能解码」这件事已经被这道门覆盖了。
//   · BOM / 行尾 / 缩进 —— 那是 formatter 的活，混进来会让这道门变得可疑。

import { readdirSync, statSync, readFileSync } from "node:fs";
import { join, dirname, extname, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
// ⛔ `.worktrees` **必须**在列：我迁移这个门时把它删掉换成新条目，
//    结果它走进 worktree 撞上已删除的路径直接 ENOENT 崩掉。
//    这就是「一个从不读文件的门比没有门更危险」的同款 —— 而且更隐蔽：
//    它不是给假绿灯，是**自己先炸**，让人以为门坏了而不是迁移写错了。
const SKIP_DIRS = new Set([
  ".git", "node_modules", "target", "dist", ".worktrees",
  "models", "thirdparty", "evals", "__pycache__", ".opencode",
]);
const EXTS = new Set([".ts", ".tsx", ".js", ".mjs", ".css", ".html", ".rs", ".md", ".json", ".py", ".sh", ".toml", ".yml", ".yaml"]);

const hits = [];
let scanned = 0;

function walk(dir) {
  for (const name of readdirSync(dir)) {
    if (SKIP_DIRS.has(name)) continue;
    const full = join(dir, name);
    let st;
    try {
      st = statSync(full);
    } catch {
      continue;   // 悬空软链或与别的窗口并发删除 —— 跳过，不崩
    }
    if (st.isDirectory()) { walk(full); continue; }
    if (!EXTS.has(extname(name))) continue;
    scanned += 1;
    // errors:"replace"：解码失败会在这里变成 U+FFFD，从而被本门一并抓到。
    const text = readFileSync(full, "utf8");
    const lines = text.split("\n");
    lines.forEach((line, i) => {
      if (line.includes("\uFFFD")) {
        hits.push({ file: relative(ROOT, full), line: i + 1, text: line.trim().slice(0, 70) });
      }
    });
  }
}

walk(ROOT);

if (hits.length) {
  console.error(`字节安全门 FAIL（${hits.length} 处 U+FFFD）:`);
  for (const h of hits) console.error(`  · ${h.file}:${h.line}  ${h.text}`);
  console.error(
    "\nU+FFFD 说明多字节字符在某条写入路径上被切坏。它出现在注释里只是**症状**；\n" +
      "同样的写文件 bug 切到字符串字面量就会静默改变行为。查最近一次写该文件的路径。",
  );
  process.exit(1);
}
console.log(`字节安全门 PASS（扫描 ${scanned} 个文件，0 处 U+FFFD）`);
