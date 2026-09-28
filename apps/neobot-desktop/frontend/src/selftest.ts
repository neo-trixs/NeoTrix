/**
 * 自测台 — 纯函数模块的 node 直跑校验（无 vitest 依赖）。
 *
 * 本项目的前端没有测试框架（`package.json` 只有 vite + tsc）。而 diff
 * 这类算法**不能靠肉眼验收**：画错了 diff 没人会发现，只会让人以为代码
 * 真的改了那一行。故给纯函数模块配一个零依赖自测台，用 tsc 编译后
 * `node` 直跑。
 *
 * 跑法（仓库 apps/neobot-desktop/frontend 下）：
 *   npm run selftest
 *
 * 只测**纯函数**（diff / highlight）。凡是要 DOM 或 IPC 的都不在这儿测 ——
 * 那是类型系统的事，不是这个台子的职责。
 */

import { MAX_EDITS, charSpans, diffLines, diffStat, splitLines } from "./diff";
import { detectLanguage, highlight } from "./highlight";
import {
  TREE_MAX_SHOWN,
  crumbsHtml,
  diffHtml,
  gitRowHtml,
  lineHtml,
  tallyHtml,
  treeRowHtml,
  truncNoteHtml,
  unifiedHtml,
  type GitFile,
  type PathTally,
} from "./render";
import { ancestorsOf, flattenTree, parentOf, sameEntries, type TreeEntry, type TreeRow } from "./tree";
import { LOCAL_COMMAND_STATUS, RUN_TURN_STATUSES, turnRanEngine, turnTaskToAttach } from "./turn_task";

/** 失败项收集器。 */
class Failures {
  readonly items: string[] = [];

  /** 断言相等（消息里带上下文，便于定位）。 */
  eq<T>(actual: T, expected: T, what: string): void {
    const a = JSON.stringify(actual);
    const b = JSON.stringify(expected);
    if (a !== b) this.items.push(`${what}\n    实际 ${a}\n    期望 ${b}`);
  }

  /** 断言为真。 */
  ok(cond: boolean, what: string): void {
    if (!cond) this.items.push(what);
  }
}

function testSplitLines(f: Failures): void {
  f.eq(splitLines(""), [], "splitLines(空串) 应为 0 行");
  f.eq(splitLines("a"), ["a"], "splitLines(单行无换行)");
  f.eq(splitLines("a\n"), ["a"], "末尾换行不算多一行");
  f.eq(splitLines("a\n\n"), ["a", ""], "空行是真实的行");
  f.eq(splitLines("a\r\nb"), ["a\r", "b"], "CRLF 保留在行尾（不擅自归一）");
}

function testIdentity(f: Failures): void {
  const same = diffLines("a\nb\nc", "a\nb\nc");
  f.eq(same.hunks.length, 0, "完全相同 → 无 hunk");
  f.eq(same.added, 0, "完全相同 → added=0");
  f.eq(diffStat(same), "", "完全相同 → stat 为空串");
}

function testPureAddRemove(f: Failures): void {
  const add = diffLines("a\nb", "a\nb\nc");
  f.eq(add.added, 1, "纯新增 added=1");
  f.eq(add.removed, 0, "纯新增 removed=0");
  f.eq(add.hunks.length, 1, "纯新增 → 1 个 hunk");

  const del = diffLines("a\nb\nc", "a\nb");
  f.eq(del.added, 0, "纯删除 added=0");
  f.eq(del.removed, 1, "纯删除 removed=1");

  // 从空到有 = 全新增。
  const fromEmpty = diffLines("", "x\ny");
  f.eq(fromEmpty.added, 2, "空 → 两行 = added 2");
  const toEmpty = diffLines("x\ny", "");
  f.eq(toEmpty.removed, 2, "两行 → 空 = removed 2");
}

function testReplacementIsPaired(f: Failures): void {
  const got = diffLines("alpha\nbeta\ngamma", "alpha\nBETA\ngamma");
  f.eq(got.added, 1, "改一行 = added 1");
  f.eq(got.removed, 1, "改一行 = removed 1");
  const del = got.hunks[0].lines.filter((l) => l.op === "del");
  const add = got.hunks[0].lines.filter((l) => l.op === "add");
  f.eq(del.length, 1, "恰好 1 个 del 行");
  f.eq(add.length, 1, "恰好 1 个 add 行");
  // 配对 → 两行都该有 spans（「改蓝配对」的前提）。
  f.ok(del[0].spans !== undefined, "配对的 del 行应带行内 spans");
  f.ok(add[0].spans !== undefined, "配对的 add 行应带行内 spans");
  // 行内：只有 BETA→BETA 是改的，"eta" 与 "ETA" 是不同的。
  const spans = del[0].spans ?? [];
  const delText = spans.filter((s) => s.kind === "del").map((s) => s.text).join("");
  const addText = spans.filter((s) => s.kind === "add").map((s) => s.text).join("");
  f.eq(delText, "beta", "行内 del 片段应拼回原行");
  f.eq(addText, "BETA", "行内 add 片段应拼回新行");
}

function testCharSpansRoundTrip(f: Failures): void {
  // 不变：单段 eq。
  f.eq(charSpans("abc", "abc"), [{ kind: "eq", text: "abc" }], "相同串 → 单 eq 段");
  // 纯插入。
  const ins = charSpans("ac", "abc");
  f.eq(
    ins.filter((s) => s.kind === "add").map((s) => s.text).join(""),
    "b",
    "纯插入：add 片段 = 'b'",
  );
  // 纯删除。
  const rem = charSpans("abc", "ac");
  f.eq(
    rem.filter((s) => s.kind === "del").map((s) => s.text).join(""),
    "b",
    "纯删除：del 片段 = 'b'",
  );
  // 换字：两侧各只含变化的那个字符。
  const sub = charSpans("cat", "cut");
  f.eq(sub.filter((s) => s.kind === "del").map((s) => s.text).join(""), "a", "换字 del='a'");
  f.eq(sub.filter((s) => s.kind === "add").map((s) => s.text).join(""), "u", "换字 add='u'");
  // 关键不变式：eq+del 拼起来 === 原串；eq+add 拼起来 === 新串。
  const samples: Array<[string, string]> = [
    ["hello world", "hello brave world"],
    ["fn main() {}", "fn main() -> i32 {}"],
    ["a\nb\nc", "a\nx\nc"],
    ["完全一样的行", "换了一行的内容"],
    ["", "x"],
    ["x", ""],
  ];
  for (const [before, after] of samples) {
    const spans = charSpans(before, after);
    const oldText = spans.filter((s) => s.kind !== "add").map((s) => s.text).join("");
    const newText = spans.filter((s) => s.kind !== "del").map((s) => s.text).join("");
    f.eq(oldText, before, `charSpans 旧侧可还原：${before} → ${after}`);
    f.eq(newText, after, `charSpans 新侧可还原：${before} → ${after}`);
  }
}

function testAdjacentSpansAreMerged(f: Failures): void {
  // 相邻同类必须合并，否则一行能出几百个 span。
  const spans = charSpans("aaaa", "aabb");
  for (let i = 1; i < spans.length; i += 1) {
    f.ok(
      spans[i - 1].kind !== spans[i].kind,
      `相邻同类 span 未合并：${spans[i - 1].kind} 后又是 ${spans[i].kind}`,
    );
  }
}

function testContextFolding(f: Failures): void {
  // 25 行里只改第 12 行 → 应该只有一个 hunk，且不含首尾行。
  const lines: string[] = [];
  for (let i = 1; i <= 25; i += 1) lines.push(`line ${i}`);
  const after = lines.slice();
  after[11] = "line 12 CHANGED";
  const got = diffLines(lines.join("\n"), after.join("\n"));
  f.eq(got.hunks.length, 1, "单点改动 → 1 个 hunk");
  const texts = got.hunks[0].lines.map((l) => l.text);
  f.ok(!texts.includes("line 1"), "上下文折叠应丢掉第 1 行");
  f.ok(!texts.includes("line 25"), "上下文折叠应丢掉第 25 行");
  f.ok(texts.includes("line 12 CHANGED"), "改动行必须在");
  f.ok(texts.includes("line 9"), "改动行前 3 行上下文在");
  f.ok(texts.includes("line 15"), "改动行后 3 行上下文在");
  f.ok(got.hunks[0].header.startsWith("@@"), "hunk 必须有 @@ 表头");
}

function testDistantChangesMakeTwoHunks(f: Failures): void {
  const lines: string[] = [];
  for (let i = 1; i <= 40; i += 1) lines.push(`l${i}`);
  const after = lines.slice();
  after[1] = "l2 X";
  after[37] = "l38 Y";
  const got = diffLines(lines.join("\n"), after.join("\n"));
  f.eq(got.hunks.length, 2, "相距很远的两处改动 → 2 个 hunk");
  f.eq(got.added, 2, "两处改动 added=2");
  f.eq(got.removed, 2, "两处改动 removed=2");
}

function testLineNumbers(f: Failures): void {
  const got = diffLines("a\nb\nc", "a\nB\nc");
  const del = got.hunks[0].lines.find((l) => l.op === "del");
  const add = got.hunks[0].lines.find((l) => l.op === "add");
  // b 在旧侧是第 2 行。
  f.eq(del?.oldNo, 2, "del 行的旧侧行号");
  f.eq(del?.newNo, null, "del 行没有新侧行号");
  f.eq(add?.newNo, 2, "add 行的新侧行号");
  f.eq(add?.oldNo, null, "add 行没有旧侧行号");
}

function testUnbalancedPairIsNotForced(f: Failures): void {
  // 删 2 行、加 1 行：只能配 1 对，多出的那个 del 整体标。
  const got = diffLines("k\na\nb\nz", "k\nz");
  const delLines = got.hunks[0].lines.filter((l) => l.op === "del");
  const addLines = got.hunks[0].lines.filter((l) => l.op === "add");
  f.eq(delLines.length, 2, "删 2 行");
  f.eq(addLines.length, 0, "纯删除不该有 add 行");
  // 纯删除时每行都该整体标 del。
  for (const line of delLines) {
    f.ok(line.spans !== undefined, "未配对的 del 行也应有 spans（整体标）");
  }
}

function testDegradationIsLabelled(f: Failures): void {
  // 构造改动量超过 MAX_EDITS 的输入 → 必须降级并**标注**。
  // 纯替换的 D 约等于 2×改动行数，故行数取到 MAX_EDITS 以上才够触发。
  const before: string[] = [];
  const after: string[] = [];
  for (let i = 0; i < MAX_EDITS; i += 1) {
    before.push(`old ${i}`);
    after.push(`new ${i}`);
  }
  const got = diffLines(before.join("\n"), after.join("\n"));
  f.ok(got.truncated, "超限必须标 truncated");
  f.ok(got.note.length > 0, "超限必须给出降级原因（不能静默）");
  f.ok(got.note.includes("不是精确 diff"), "降级说明要讲清不是精确 diff");
  // 降级后仍然是**诚实的**：增删行数对得上。
  f.eq(got.added, MAX_EDITS, "降级后 added 仍要准确");
  f.eq(got.removed, MAX_EDITS, "降级后 removed 仍要准确");
}

function testModerateRewriteStaysExact(f: Failures): void {
  // 上限不能「一碰就降级」：中等规模的重写必须仍是精确 diff。
  const before: string[] = [];
  const after: string[] = [];
  for (let i = 0; i < 200; i += 1) {
    before.push(`keep ${i}`);
    after.push(`keep ${i}`);
  }
  for (let i = 0; i < 100; i += 1) {
    before.push(`old ${i}`);
    after.push(`new ${i}`);
  }
  const got = diffLines(before.join("\n"), after.join("\n"));
  f.ok(!got.truncated, "200 行级重写不该降级");
  f.eq(got.note, "", "未降级时 note 应为空");
  f.eq(got.added, 100, "精确 diff 的 added 要准");
  f.eq(got.removed, 100, "精确 diff 的 removed 要准");
}

function testAddBeforeDelStillPairs(f: Failures): void {
  // 回归锁：Myers 回溯可能输出「add 在前、del 在后」。
  // 曾因配对循环只找「del 段 → 其后 add 段」而整块丢失行内高亮。
  // 逐个枚举单字符替换，验证**每个**被改的行都拿到了 spans。
  const alphabet = ["a", "b", "c", "d"];
  for (const before of alphabet) {
    for (const after of alphabet) {
      if (before === after) continue;
      const got = diffLines(`x\n${before}\ny`, `x\n${after}\ny`);
      const del = got.hunks[0].lines.find((l) => l.op === "del");
      const add = got.hunks[0].lines.find((l) => l.op === "add");
      f.ok(del !== undefined, `${before}→${after}: 应有 del 行`);
      f.ok(add !== undefined, `${before}→${after}: 应有 add 行`);
      if (!del || !add) continue;
      f.ok(del.spans !== undefined, `${before}→${after}: del 行应有行内 spans`);
      f.ok(add.spans !== undefined, `${before}→${after}: add 行应有行内 spans`);
      if (!del.spans || !add.spans) continue;
      // 两侧各自能还原（行内片段拼回去 = 原行/新行）。
      const delText = del.spans.filter((s) => s.kind !== "add").map((s) => s.text).join("");
      const addText = add.spans.filter((s) => s.kind !== "del").map((s) => s.text).join("");
      f.eq(delText, before, `${before}→${after}: 旧侧可还原`);
      f.eq(addText, after, `${before}→${after}: 新侧可还原`);
    }
  }
}

function testHighlightEscapesFirst(f: Failures): void {
  // 高亮的头号不变式：先转义，再切 token。忘了转义就是 XSS。
  const evil = `<script>alert(1)</script>`;
  const out = highlight(evil, "plain");
  f.ok(!out.includes("<script>"), "高亮必须转义 <script>");
  f.ok(out.includes("&lt;"), "尖括号应被转义成实体");
  // 注意别断言「不含 &」—— `&lt;` 本身就含 &。真正的不变式是
  // **没有裸的尖括号漏网**。
  f.ok(!out.includes("<script"), "不应有裸 <");
  f.ok(!out.includes(">"), "不应有裸 >");
  f.ok(out.includes("&amp;") || out.includes("&lt;"), "特殊字符应成实体");
  // 标签只有我们自己的 span，不含用户可控标签名。
  f.ok(!/<\/?(?!span\b)[a-z]/i.test(out), `输出里出现了非 span 标签：${out}`);
}

function testHighlightRoundTripsText(f: Failures): void {
  const samples: Array<[string, string]> = [
    ["const a = 1;", "ts"],
    ["fn main() { println!(\"hi\"); }", "rust"],
    ["{\n  \"a\": [1, 2]\n}", "json"],
    ["# 标题\n\n正文 **粗** `码`", "markdown"],
    ["plain text, nothing to do", "plain"],
  ];
  for (const [src, lang] of samples) {
    const html = highlight(src, lang);
    // 去掉所有标签后，文本必须与源文一致（丢了字符就是切错了）。
    const text = html
      .replace(/<[^>]*>/g, "")
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .replace(/&quot;/g, '"')
      .replace(/&#39;/g, "'")
      .replace(/&amp;/g, "&");
    f.eq(text, src, `高亮丢字符（${lang}）：${JSON.stringify(src)} → ${JSON.stringify(text)}`);
  }
}

function testLanguageDetection(f: Failures): void {
  f.eq(detectLanguage("a.rs"), "rust", "按 .rs 判 rust");
  f.eq(detectLanguage("a.TS"), "ts", "扩展名大写不敏感");
  f.eq(detectLanguage("x.py"), "python", ".py → python");
  f.eq(detectLanguage("x.json"), "json", ".json → json");
  f.eq(detectLanguage("README.md"), "markdown", ".md → markdown");
  // 认得出 Makefile / Dockerfile 这类**知名**文件名不是瞎猜。
  f.eq(detectLanguage("Makefile"), "shell", "Makefile → shell");
  f.eq(detectLanguage("package.json"), "json", "package.json → json");
  // 真认不出的才回落 plain。
  f.eq(detectLanguage("x.unknownext"), "plain", "未知扩展名 → plain");
  f.eq(detectLanguage("some-random-file"), "plain", "无扩展名未知名 → plain");
  f.eq(detectLanguage(""), "plain", "空名 → plain");
}

// ---- 文件树（回归锁：曾经「重绘只看一层，展开毫无反应」） ----

/** 造一个目录项。 */
function ent(rel: string, isDir: boolean, size = 0): TreeEntry {
  return {
    name: rel.slice(rel.lastIndexOf("/") + 1),
    rel,
    is_dir: isDir,
    is_symlink: false,
    broken_link: false,
    size,
    mtime: 0,
  };
}

function rowRels(rows: TreeRow[]): string[] {
  return rows.map((r) => r.rel);
}

function testTreeOnlyShowsExpandedDirs(f: Failures): void {
  const cache = new Map<string, TreeEntry[]>([
    ["", [ent("src", true), ent("README.md", false)]],
    ["src", [ent("src/main.rs", false)]],
  ]);
  // 一个都没展开 → 只看得到根。
  const collapsed = rowRels(flattenTree(cache, new Set<string>()));
  f.eq(collapsed, ["src", "README.md"], "未展开时不该出现子项");
  // 展开 src → 子项出现，且带 depth 1。
  const opened = flattenTree(cache, new Set(["src"]));
  f.eq(rowRels(opened), ["src", "src/main.rs", "README.md"], "展开后子项要出现");
  f.eq(opened[1]?.depth, 1, "子项 depth 必须是 1");
  f.eq(opened[0]?.depth, 0, "根层 depth 必须是 0");
  // 顺序是先序遍历（目录紧跟它的子项），不是「目录全在前」。
  f.eq(opened.map((r) => r.rel), ["src", "src/main.rs", "README.md"], "先序遍历");
}

function testTreeSkipsUnloadedDirs(f: Failures): void {
  // 目录在 expanded 里但**没加载**（刚展开、拉取失败）→ 不能崩，也不能编出子项。
  const cache = new Map<string, TreeEntry[]>([["", [ent("a", true)]]]);
  const rows = rowRels(flattenTree(cache, new Set(["a"])));
  f.eq(rows, ["a"], "未加载的展开目录不该产出子项");
  // 空缓存 → 空树，不抛。
  f.eq(rowRels(flattenTree(new Map(), new Set())), [], "空缓存 → 空树");
}

function testTreeNestsDeeply(f: Failures): void {
  // 深层嵌套：a/b/c/d/e.txt 全展开。
  const cache = new Map<string, TreeEntry[]>([
    ["", [ent("a", true)]],
    ["a", [ent("a/b", true)]],
    ["a/b", [ent("a/b/c", true)]],
    ["a/b/c", [ent("a/b/c/d", true)]],
    ["a/b/c/d", [ent("a/b/c/d/e.txt", false)]],
  ]);
  const all = new Set(["a", "a/b", "a/b/c", "a/b/c/d"]);
  const rows = flattenTree(cache, all);
  f.eq(rowRels(rows), ["a", "a/b", "a/b/c", "a/b/c/d", "a/b/c/d/e.txt"], "深层应全展开");
  f.eq(rows.map((r) => r.depth), [0, 1, 2, 3, 4], "depth 逐层递增");
  // 只展开到 b：c 及以下不可见。
  const partial = rowRels(flattenTree(cache, new Set(["a", "a/b"])));
  f.eq(partial, ["a", "a/b", "a/b/c"], "未展开的深层不可见");
}

function testTreeSkipsChildrenOfUnloadedParent(f: Failures): void {
  // 声称展开了 a/b，但 a 没加载 → 遍历根本走不到 b。
  const cache = new Map<string, TreeEntry[]>([["a/b", [ent("a/b/x", false)]]]);
  const rows = rowRels(flattenTree(cache, new Set(["a", "a/b"])));
  f.eq(rows, [], "父目录没加载就不该看到子目录的内容");
}

function testTreeLeavesAreNeverExpanded(f: Failures): void {
  // 把一个**文件**路径放进 expanded（不该发生，但要防住）：
  // 它的子目录条目在缓存里也不该被渲染出来。
  const cache = new Map<string, TreeEntry[]>([
    ["", [ent("f.txt", false)]],
    ["f.txt", [ent("f.txt/ghost", false)]],
  ]);
  const rows = rowRels(flattenTree(cache, new Set(["f.txt"])));
  f.eq(rows, ["f.txt"], "文件不该被当作可展开的目录");
}

function testSameEntriesDetectsRealChanges(f: Failures): void {
  const base = [ent("a", false, 10), ent("b", false, 20)];
  f.ok(sameEntries(base, [ent("a", false, 10), ent("b", false, 20)]), "完全相同 → 没变");
  // 长度变了。
  f.ok(!sameEntries(base, [ent("a", false, 10)]), "少了项 → 变了");
  // 顺序变了（Rust 侧排序保证不该发生，但变了就该重绘）。
  f.ok(!sameEntries(base, [ent("b", false, 20), ent("a", false, 10)]), "换序 → 变了");
  // 大小变了（内容被外部工具改过）。
  f.ok(!sameEntries(base, [ent("a", false, 11), ent("b", false, 20)]), "大小变了 → 变了");
  // 新增项。
  f.ok(!sameEntries(base, [...base, ent("c", false, 1)]), "新增项 → 变了");
  // 只改 mtime 不重绘（省得每 4 秒重刷一次）。
  const touched = [ent("a", false, 10), ent("b", false, 20)];
  touched[0]!.mtime = 999;
  f.ok(sameEntries(base, touched), "只改 mtime 不算变（避免无谓重绘）");
}

function testUnreadableDirShowsAPlaceholderRow(f: Failures): void {
  // 展开过但拉取失败的目录：必须**产出一行「打不开」**，
  // 而不是静默什么都不出 —— 否则用户以为文件没了。
  const cache = new Map<string, TreeEntry[]>([["", [ent("secret", true), ent("ok.txt", false)]]]);
  const rows = flattenTree(cache, new Set(["secret"]), "", new Set(["secret"]));
  f.eq(rowRels(rows), ["secret", "ok.txt"], "打不开的目录**只占一行**（不能父子各一行）");
  const bad = rows[0];
  f.ok(bad?.entry.unreadable === true, "该行应带 unreadable 标记");
  f.eq(bad?.entry.is_dir, true, "占位仍是目录形态");
  f.eq(bad?.depth, 0, "深度与同级一致");
  // 它的「子项」不该被造出来。
  f.ok(!rowRels(rows).some((r) => r.startsWith("secret/")), "不该编造子项");
  // 失败集合为空时行为不变。
  const normal = flattenTree(cache, new Set(["secret"]));
  f.eq(rowRels(normal), ["secret", "ok.txt"], "未标失败时行为不变");
  f.ok(normal[0]?.entry.unreadable !== true, "没标记失败就不该显示占位");
  // 折叠之后**仍然**保留「打不开」标记。
  //
  // 「这个目录读不了」是一条**稳定事实**，不是一次性 UI 状态：折叠了就当没这回事，
  // 会让用户以为问题自己好了（再展开又冒出来）。这与 `dirTrunc`（条目被截断）
  // 同一个道理 —— 两者都是「如实报告」的持久标记。
  const collapsed = flattenTree(cache, new Set<string>(), "", new Set(["secret"]));
  f.eq(rowRels(collapsed), ["secret", "ok.txt"], "全折叠时只有根层");
  f.ok(
    collapsed[0]?.entry.unreadable === true,
    "折叠后仍应保留「打不开」（读不了是事实，不是瞬时状态）",
  );
  // 但**没试过**的目录不该被剧透：`failed` 里没有它就一切照常。
  const untouched = flattenTree(cache, new Set(["secret"]), "", new Set());
  f.ok(
    untouched[0]?.entry.unreadable !== true,
    "没标记失败就不该显示占位（哪怕它还没被加载过）",
  );
}

function testPathHelpers(f: Failures): void {
  f.eq(parentOf("a/b/c.txt"), "a/b", "parentOf 取到目录");
  f.eq(parentOf("top.txt"), "", "顶层文件的父是空串（= 工作区根）");
  f.eq(parentOf(""), "", "空串的父还是空串");

  f.eq(ancestorsOf("a/b/c.txt"), ["a", "a/b"], "祖先含每一级目录，不含文件本身");
  f.eq(ancestorsOf("top.txt"), [], "顶层文件没有祖先目录");
  f.eq(ancestorsOf("a/b"), ["a"], "目录的祖先不含自己");
  f.eq(ancestorsOf(""), [], "空路径无祖先");
  // 祖先顺序必须是根→深（面包屑要按这个顺序逐个展开）。
  f.eq(ancestorsOf("x/y/z/w.txt"), ["x", "x/y", "x/y/z"], "祖先由浅到深");
}

// ---- 侧边栏渲染（`render.ts`）----
//
// 前面那几节验的是「算得对不对」，这一节验的是**画出来对不对**。
// 两类都出过事，且都不可能被 `tsc` 抓到：
// 1. **XSS**：文件名、git 提交信息、diff 片段全来自磁盘 / 模型 / 远端，
//    拼进 `innerHTML` 前不转义就是一个存储型 XSS（本项目 CSP 是
//    `script-src 'self'`，拦不住 `<img onerror>`）；
// 2. **画反**：行号取错侧、a11y 属性漏了、少说一句「已截断」——
//    编译全过，肉眼扫一眼也看不出来。
// 故这里**不测「字符串等于某串」**，而测**结构 + 不变式**：
// 标签只能是我们的、属性齐全、行号在正确的一侧。

/**
 * 抽出输出里出现的**全部标签名**。
 *
 * 「转义了没有」不能断言 `!out.includes("<img")` 就算完 —— 那只查了一个
 * payload。真正的不变式是**白名单**：输出里除我们自己那几个标签外，
 * 不该出现任何别的标签名（攻击者的标签名由磁盘决定，不受我们控制）。
 */
function tagNames(html: string): string[] {
  const out: string[] = [];
  const re = /<\/?([a-zA-Z][a-zA-Z0-9-]*)/g;
  let m: RegExpExecArray | null = re.exec(html);
  while (m !== null) {
    const name = m[1];
    if (name !== undefined) out.push(name.toLowerCase());
    m = re.exec(html);
  }
  return out;
}

/**
 * 输出里出现的标签必须全在白名单里。
 *
 * `svg`/`path`/`circle` 来自 `icons.ts` 的**闭合表**（键是编译期字面量，
 * 攻击者碰不到），其余是本模块自己写的行/按钮/包屑。
 */
function fOnlyOurTags(f: Failures, html: string, what: string): void {
  const allowed = ["div", "span", "button", "svg", "path", "circle"];
  for (const name of tagNames(html)) {
    if (allowed.indexOf(name) === -1) {
      f.ok(false, `${what}：出现了非白名单标签 <${name}> —— ${html}`);
      return;
    }
  }
  f.ok(true, what);
}

/** 恶意文件名 / 路径：够打穿「忘了转义」和「只转义了尖括号没转义引号」两种写法。 */
const EVIL_NAME = `<img src=x onerror=alert(1)>`;
/** 恶意 git 提交信息 / xy 字段：带 `</script>` 断尾的经典 payload。 */
const EVIL_SUBJECT = `fix <script>alert(1)</script>`;
/** 恶意 diff 行：`&` 与两种引号齐活，专打「只转义 `<>`」的半吊子转义。 */
const EVIL_LINE = `x = "&" + 'q' <b>bold</b>`;

/** 造一行树（`unreadable` / `broken_link` 等按需打开）。 */
function treeRow(rel: string, isDir: boolean, depth: number, extra: Partial<TreeEntry> = {}): TreeRow {
  return {
    rel,
    depth,
    entry: {
      name: rel.slice(rel.lastIndexOf("/") + 1),
      rel,
      is_dir: isDir,
      is_symlink: false,
      broken_link: false,
      size: 0,
      mtime: 0,
      ...extra,
    },
  };
}

function testRenderTreeNameIsInert(f: Failures): void {
  // 头号不变式：磁盘上的文件名原样进 innerHTML 前必须转义。
  const out = treeRowHtml(treeRow(EVIL_NAME, false, 0), false);
  f.ok(!out.includes("<img"), `文件名里的 <img 必须被转义：${out}`);
  f.ok(out.includes("&lt;img"), "尖括号应落成实体（而不是被删掉 —— 删掉就骗人了）");
  f.ok(out.includes("onerror=alert(1)"), "转义不等于抹掉：原文仍要能看见");
  fOnlyOurTags(f, out, "恶意文件名不该带出任何非白名单标签");
}

/** 引号逃逸：属性值里出现 `"` 就等于**跳出属性**加一个新事件处理器。 */
function testRenderEscapesQuotesInAttributes(f: Failures): void {
  const nasty = `a" onmouseover="alert(1)`;
  const out = treeRowHtml(treeRow(nasty, false, 0), false);
  f.ok(!out.includes(`onmouseover="alert`), `属性里的引号必须转义：${out}`);
  f.ok(out.includes("&quot;"), "双引号应转义成 &quot;（data-rel / data-path 都靠它）");
  fOnlyOurTags(f, out, "引号逃逸不该带出非白名单标签");
  // 单引号同样要转 —— 属性有时用单引号拼。
  f.ok(treeRowHtml(treeRow(`b' onfocus='alert(1)`, false, 0), false).includes("&#39;"), "单引号也要转义");
}

function testRenderGitRowIsInert(f: Failures): void {
  // git 提交信息（`subject`）与路径同源：都可能是别人写的一段 HTML。
  const file: GitFile = {
    path: EVIL_SUBJECT,
    xy: `<img src=x onerror=alert(1)>`,
    staged: true,
    unstaged: true,
    untracked: false,
  };
  const out = gitRowHtml(file);
  f.ok(!out.includes("<script"), `git 行里的 <script 必须被转义：${out}`);
  f.ok(!out.includes("</script"), "闭合标签也要转义");
  f.ok(out.includes("&lt;script&gt;"), "提交信息应落成实体");
  f.ok(!out.includes("<img"), "xy 字段同样要转义（它也进 title= 属性）");
  f.ok(out.includes("&lt;img src=x onerror=alert(1)&gt;"), "xy 的 payload 应原样落成实体（仍可读，但不执行）");
  fOnlyOurTags(f, out, "git 行不该带出任何非白名单标签");
  // 未跟踪文件没有「还原」（还原一个从没进过索引的文件没有意义）。
  f.ok(
    !gitRowHtml({ path: "x", xy: "??", staged: false, unstaged: false, untracked: true }).includes("git-revert"),
    "未跟踪文件不该给「还原」按钮",
  );
  // 已暂存 → 按钮变成「取消暂存」（否则会给出反向操作）。
  f.ok(out.includes("git-unstage"), "已暂存的文件应给「取消暂存」");
  f.ok(gitRowHtml({ path: "x", xy: " M", staged: false, unstaged: true, untracked: false }).includes("git-stage"), "未暂存的文件应给「暂存」");
}

function testRenderTallyIsInert(f: Failures): void {
  // 路径计数里的 `path` 来自账本（工具写文件时记的），`last_change` 是 id —— 都当不可信。
  const t: PathTally = {
    path: EVIL_NAME,
    reads: 1,
    writes: 0,
    edits: 0,
    last_change: `" onmouseover="alert(1)`,
  };
  const out = tallyHtml(t);
  f.ok(!out.includes("<img"), `变动行里的文件名必须被转义：${out}`);
  f.ok(!out.includes(`onmouseover="alert`), "data-id 属性里的引号必须转义");
  f.ok(out.includes("读1"), "读计数要画出来");
  f.ok(!out.includes("写0") && !out.includes("改0"), "计数为 0 的操作不该占位");
  fOnlyOurTags(f, out, "变动行不该带出任何非白名单标签");
}

function testRenderDiffTextIsInert(f: Failures): void {
  // diff 内容全是文件正文；`&` 与引号必须成实体，否则 `&lt;` 会被浏览器
  // 解回 `<`（二次解释），引号则会打断属性。
  const out = lineHtml({ op: "add", text: EVIL_LINE, oldNo: null, newNo: 3 });
  f.ok(out.includes("&amp;"), "& 必须最先转义（否则 &lt; 会被解回尖括号）");
  f.ok(out.includes("&quot;"), "双引号要转义");
  f.ok(out.includes("&#39;"), "单引号要转义");
  f.ok(!out.includes("<b>"), "diff 文本里的 <b> 必须被转义");
  fOnlyOurTags(f, out, "diff 行不该带出任何非白名单标签");

  // 行内 span 走的是另一条拼串路径（`span.text` 逐段转义），也要单独钉。
  const inl = lineHtml({
    op: "add",
    text: "x",
    oldNo: null,
    newNo: 1,
    spans: [
      { kind: "eq", text: "a" },
      { kind: "add", text: `<b>${EVIL_LINE}</b>` },
    ],
  });
  f.ok(!inl.includes("<b>"), "行内 span 里的标签必须被转义");
  fOnlyOurTags(f, inl, "行内 span 不该带出非白名单标签");

  // 标题（文件路径）与降级说明同样来自磁盘。
  const head = diffHtml(diffLines("a", `b${EVIL_NAME}`), EVIL_NAME);
  f.ok(!head.includes("<img"), "diff 标题（文件路径）必须被转义");
  const degraded = diffHtml(
    { hunks: [], added: 1, removed: 1, truncated: true, note: EVIL_NAME },
    "x",
  );
  f.ok(!degraded.includes("<img"), "降级说明（note）必须被转义");
  f.ok(degraded.includes("sb-note warn"), "降级必须明说出来（界律 2：不能静默给一段假 diff）");
}

/** 行结构：行号取哪一侧、标记是什么 —— 画反了 diff 就读不懂。 */
function testDiffRowStructure(f: Failures): void {
  const add = lineHtml({ op: "add", text: "new", oldNo: null, newNo: 7 });
  f.ok(add.includes('<div class="sb-dl add">'), "add 行的类名要带 add");
  f.ok(add.includes('<span class="sb-no">7</span>'), "add 行显示**新侧**行号");
  f.ok(!add.includes('<span class="sb-no">null</span>'), "缺失的行号不能画出字面量 null");
  f.ok(add.includes('<span class="sb-mk">+</span>'), "add 行标记是 +");

  const del = lineHtml({ op: "del", text: "old", oldNo: 7, newNo: null });
  f.ok(del.includes('<div class="sb-dl del">'), "del 行的类名要带 del");
  f.ok(del.includes('<span class="sb-no">7</span>'), "del 行显示**旧侧**行号");
  f.ok(del.includes('<span class="sb-mk">−</span>'), "del 行标记是减号 U+2212（与 stat 一致）");

  // eq 行：行号取旧侧（`op` 既非 add 也非 del 的兜底），标记是空格。
  // 上下文行两侧同号才是常态；真的重排过时哪一侧都不完全对，锁定现状口径。
  const eq = lineHtml({ op: "eq", text: "same", oldNo: 4, newNo: 4 });
  f.ok(eq.includes('<span class="sb-mk"> </span>'), "eq 行标记是空格");
  f.ok(eq.includes('<span class="sb-no">4</span>'), "eq 行仍显示行号（对齐用）");

  // 空行不能画成「什么都没有」——CSS 里 .sb-dl 靠内容撑高度。
  const blank = lineHtml({ op: "add", text: "", oldNo: null, newNo: 1 });
  f.ok(blank.includes("&nbsp;"), "空行要退化成 &nbsp;，否则行高塌陷");
}

/** 行内高亮：`sb-in-add` / `sb-in-del` 缺一个就等于没做行内 diff。 */
function testDiffInlineSpanColours(f: Failures): void {
  const out = lineHtml({
    op: "del",
    text: "abc",
    oldNo: 1,
    newNo: null,
    spans: [
      { kind: "eq", text: "a" },
      { kind: "del", text: "<b>b</b>" },
      { kind: "eq", text: "c" },
    ],
  });
  f.ok(out.includes('<span class="sb-in-del">&lt;b&gt;b&lt;/b&gt;</span>'), "del 片段要包 sb-in-del 且文本已转义");
  f.ok(!out.includes("sb-in-add"), "del 行不该出现 sb-in-add");
  f.ok(out.startsWith(`<div class="sb-dl del">`), "行级类名仍由 op 决定");
  // 配对的新侧行：同理走 sb-in-add。
  const add = lineHtml({
    op: "add",
    text: "aBc",
    oldNo: null,
    newNo: 1,
    spans: [
      { kind: "eq", text: "a" },
      { kind: "add", text: "B" },
      { kind: "eq", text: "c" },
    ],
  });
  f.ok(add.includes('<span class="sb-in-add">B</span>'), "add 片段要包 sb-in-add");
  f.ok(!add.includes("sb-in-del"), "add 行不该出现 sb-in-del");
}

/** 树的无障碍属性：少一个，键盘用户就走不动这棵树。 */
function testTreeRowAriaAttributes(f: Failures): void {
  const dirOpen = treeRowHtml(treeRow("src", true, 0), true);
  f.ok(dirOpen.includes('role="treeitem"'), "目录行要有 role=treeitem");
  f.ok(dirOpen.includes('tabindex="0"'), "目录行要可聚焦（tabindex=0）");
  f.ok(dirOpen.includes('aria-expanded="true"'), "展开的目录 aria-expanded=true");
  f.ok(dirOpen.includes('data-dir="1"'), "目录要标 data-dir=1（事件装配靠它判开合）");

  const dirShut = treeRowHtml(treeRow("src", true, 0), false);
  f.ok(dirShut.includes('aria-expanded="false"'), "收起的目录 aria-expanded=false（不能省掉）");
  f.ok(dirShut.includes('class="sb-row sb-tree-row dir'), "目录行要有 dir 类");

  // 文件：不可展开，故 aria-expanded 留空。⚠️ 规范上叶子节点应**整条省略**该属性
  // （空串不是合法取值，AT 可能读成 undefined/false）—— 现状锁定，见交付说明。
  const file = treeRowHtml(treeRow("a.txt", false, 0), false);
  f.ok(file.includes('role="treeitem"'), "文件行也要 role=treeitem");
  f.ok(file.includes('tabindex="0"'), "文件行要可聚焦");
  f.ok(file.includes('aria-expanded=""'), "文件行 aria-expanded 留空（现状）");
  f.ok(file.includes('data-dir="0"'), "文件要标 data-dir=0");
  f.ok(file.includes('class="sb-caret leaf"'), "文件没有展开箭头");
}

/** 缩进必须随 depth 单调增 —— 否则树看起来是平的/乱的。 */
function testTreeIndentGrowsWithDepth(f: Failures): void {
  const pads: number[] = [];
  for (let depth = 0; depth <= 4; depth += 1) {
    const out = treeRowHtml(treeRow("x", false, depth), false);
    const m = /padding-left:(\d+)px/.exec(out);
    f.ok(m !== null, `depth=${depth} 的行要有 padding-left`);
    pads.push(m ? Number(m[1]) : -1);
  }
  for (let i = 1; i < pads.length; i += 1) {
    f.ok(
      (pads[i] ?? 0) > (pads[i - 1] ?? 0),
      `缩进必须随 depth 递增：depth ${i - 1}=${pads[i - 1]}px → depth ${i}=${pads[i]}px`,
    );
  }
  f.eq(pads[0], 6, "根层缩进是 6px");
  f.ok(treeRowHtml(treeRow("x", false, 0), false).includes('data-depth="0"'), "行上要带 data-depth");
}

/** 打不开的目录：标出来，但不给「引用」——引用一个打不开的路径没有意义。 */
function testUnreadableRowIsMarkedNotQuoted(f: Failures): void {
  const out = treeRowHtml(treeRow("secret", true, 0, { unreadable: true }), true);
  f.ok(out.includes("unreadable"), "要带 unreadable 类");
  f.ok(out.includes("（打不开）"), "标签要说明为什么没有子项");
  f.ok(!out.includes('data-act="ref"'), "打不开的行**不该**有 @ 引用按钮");
  f.ok(!out.includes(">@</button>"), "打不开的行不该画出 @ 按钮");
  f.ok(out.includes('class="sb-caret leaf"'), "打不开的目录不该有展开箭头（点了也拉不到）");
  // 对照：能打开的目录必须**有** @ 按钮（少了它就是功能缺失）。
  f.ok(treeRowHtml(treeRow("ok", false, 0), false).includes('data-act="ref"'), "普通行要有 @ 引用按钮");
  // 断链：名字要带失效标记，且不因此变成目录。
  const broken = treeRowHtml(treeRow("dangling", false, 0, { is_symlink: true, broken_link: true }), false);
  f.ok(broken.includes("（链接失效）"), "断链要标「链接失效」");
  f.ok(broken.includes("broken"), "断链要带 broken 类");
  f.ok(broken.includes("sb-link"), "软链接要有 ↗ 标记");
  f.ok(broken.includes('aria-expanded=""'), "断链文件仍不可展开");
}

/** 截断提示：**逐个**点名，且集合为空时一句话都不说。 */
function testTruncNoticeNamesEveryDir(f: Failures): void {
  f.eq(truncNoteHtml([]), "", "没有目录被截断 → 不出提示（空 div 也是噪音）");
  f.eq(truncNoteHtml([], 10), "", "空集合（显式传上限）同样不出提示");

  // 回归锁：早先只判根目录，展开的子目录超限时**一声不吭**。
  const two = truncNoteHtml(["src/deep", "docs"]);
  f.ok(two.includes("src/deep"), "子目录被截断必须被点名（不只是根）");
  f.ok(two.includes("docs"), "每个被截断的目录都要被点名");
  f.ok(two.includes("2 个目录"), "多个时要报数量");
  f.ok(two.includes(String(TREE_MAX_SHOWN)), "提示里要带上限数字（否则用户不知道被切了多少）");
  f.ok(two.includes("不是全部"), "提示必须说清「这不是全部」");
  // 顺序稳定（同名提示每次重绘都一样，便于 diff/朗读）。
  f.ok(two.indexOf("docs") < two.indexOf("src/deep"), "多个目录应排序后输出（稳定）");

  // 单个目录：说清是谁，且根目录不能显示成空串。
  const one = truncNoteHtml([""]);
  f.ok(one.includes("工作区根目录"), "根目录要有可读的名字（不能是个空路径）");
  f.ok(one.includes(String(TREE_MAX_SHOWN)), "单目录也要报上限");
  f.ok(truncNoteHtml(["src"], 42).includes("42"), "上限可由调用方指定（口径与 Rust 侧对齐，不是写死的数）");

  // 目录名来自磁盘 → 同样要转义（这条曾经是漏的：rel 直接拼进了 innerHTML）。
  const evil = truncNoteHtml([EVIL_NAME]);
  f.ok(!evil.includes("<img"), `截断提示里的目录名必须被转义：${evil}`);
  f.ok(evil.includes("&lt;img"), "尖括号应落成实体");
  fOnlyOurTags(f, evil, "截断提示不该带出非白名单标签");
}

/** unified diff 着色：这是「不重算 diff、直接给 git 文本」的路径。 */
function testUnifiedDiffColouring(f: Failures): void {
  const out = unifiedHtml(
    [
      "diff --git a/x b/x",
      "index 111..222 100644",
      "--- a/x",
      "+++ b/x",
      "@@ -1,3 +1,3 @@",
      " keep",
      "-old <img src=x onerror=alert(1)>",
      "+new & \"quoted\"",
      "",
    ].join("\n"),
  );
  f.ok(out.includes('<div class="sb-dl hunk">@@ -1,3 +1,3 @@</div>'), "@@ 开头 → hunk 类");
  f.ok(out.includes('<div class="sb-dl meta">--- a/x</div>'), "--- 开头 → meta 类");
  f.ok(out.includes('<div class="sb-dl meta">+++ b/x</div>'), "+++ 开头 → meta 类");
  f.ok(out.includes('<div class="sb-dl del">-old'), "- 开头 → del 类");
  f.ok(out.includes('<div class="sb-dl add">+new'), "+ 开头 → add 类");
  f.ok(out.includes('<div class="sb-dl eq"> keep'), "无前缀 → eq 类");
  f.ok(out.includes('<div class="sb-dl eq">index 111'), "index 行按普通上下文着色");
  f.ok(out.includes('<div class="sb-dl eq">diff --git'), "diff --git 行按普通上下文着色");
  // 文本必须转义：git 文本里带 `<img>` 的删除行是最容易漏的一条路径。
  f.ok(!out.includes("<img"), `unified 文本必须被转义：${out}`);
  f.ok(out.includes("&lt;img src=x onerror=alert(1)&gt;"), "尖括号应落成实体");
  f.ok(out.includes("&amp; &quot;quoted&quot;"), "& 与引号都要转义（注意顺序：& 先转，否则 &lt; 会被解回尖括号）");
  fOnlyOurTags(f, out, "unified 行不该带出任何非白名单标签");
  // 空行不能画成空 div（行高塌陷）。
  f.ok(out.includes('<div class="sb-dl eq"> </div>'), "空行退化成空格");
  // 删掉一行内容为 `-- x` 的文件，git 文本里是 `--- x`，与文件头 `---` 撞形
  // → 会被判成 meta（不着色）。这是 unified 语法自带的歧义（git 自己也只靠
  // 「`---` 只出现在文件头」这个约定区分）。锁住**当前口径**（meta 优先），
  // 免得以后无声地改掉着色。
  f.ok(unifiedHtml("--- x").includes('class="sb-dl meta"'), "meta 判定优先于 del（口径锁定）");
}

function testCrumbsAreInert(f: Failures): void {
  // 面包屑的每一级 = 一个可点按钮，`data-crumb` 会被拿去展开祖先。
  const out = crumbsHtml("a/b/c.txt");
  f.ok(out.includes('data-crumb=""'), "第一级永远是工作区根（data-crumb 为空串）");
  f.ok(out.includes('data-crumb="a"'), "祖先目录要逐级出现");
  f.ok(out.includes('data-crumb="a/b"'), "祖先目录要逐级出现（到第二级）");
  f.ok(!out.includes('data-crumb="a/b/c.txt"'), "文件本身不是可点的目录");
  f.ok(out.includes(">c.txt</span>"), "末级显示当前文件名（不可点）");
  f.eq((out.match(/sb-sep/g) ?? []).length, 3, "三级分隔符 = 祖先数 + 1");

  // 顶层文件：没有祖先，只有「工作区 / 文件」。
  const top = crumbsHtml("README.md");
  f.ok(top.includes(">README.md</span>"), "顶层文件也要显示自己");
  f.eq((top.match(/sb-crumb\b/g) ?? []).length, 2, "顶层只有工作区 + 当前文件两个 crumb");

  // 目录名来自磁盘 → 转义。
  const evil = crumbsHtml(`${EVIL_NAME}/x.txt`);
  f.ok(!evil.includes("<img"), `面包屑里的目录名必须被转义：${evil}`);
  f.ok(evil.includes("&lt;img"), "尖括号应落成实体");
  f.ok(!evil.includes(`data-crumb="${EVIL_NAME}`), "data-crumb 属性值必须被转义（否则能拼出新属性）");
  fOnlyOurTags(f, evil, "面包屑不该带出任何非白名单标签");
}

/** 走一遍真实链路：`flattenTree` 的输出直接喂渲染器。 */
function testRenderConsumesFlattenedTree(f: Failures): void {
  const cache = new Map<string, TreeEntry[]>([
    ["", [{ name: EVIL_NAME, rel: EVIL_NAME, is_dir: true }, { name: "ok", rel: "ok", is_dir: false }]],
    [EVIL_NAME, [{ name: "x", rel: `${EVIL_NAME}/x`, is_dir: false }]],
  ]);
  const rows = flattenTree(cache, new Set([EVIL_NAME]), "", new Set<string>());
  // `open` 由调用方从展开集合取 —— 这里照 `sidebar.paintTree` 的取法。
  const open = new Set([EVIL_NAME]);
  const html = rows.map((row) => treeRowHtml(row, open.has(row.entry.rel))).join("");
  f.ok(!html.includes("<img"), "整棵树渲染出来也不能带出攻击者标签");
  f.ok(html.includes('aria-expanded="true"'), "已展开的目录标 aria-expanded=true");
  f.ok(html.includes('data-depth="1"'), "子项带 depth=1");
  f.ok(html.includes("sb-tree-row dir"), "目录行带 dir 类");
  fOnlyOurTags(f, html, "整棵树只应有我们自己的标签");
}

/** 跑全部自测；返回失败项（空数组 = 全过）。 */
/**
 * 任务卡归属（`turn_task`）。
 *
 * 这组用例是**回归锁**：曾经收尾处无条件取 `tasks[0]`，于是本地指令
 * （`/help` 之类，**不进跑轮**）也会把**上一轮真实对话**的任务卡挂到自己气泡下。
 * 修法是改用**语义判据**（这一轮到底跑没跑轮），所以这里特意包含一条
 * 「列表非空也必须不挂」——防止有人后来把判据退化成「列表空不空」而测试照样绿。
 */
function testTurnTaskAttribution(f: Failures): void {
  const before = new Set(["old-1", "old-2"]);
  const stale = [
    { id: "old-2", conversation_id: "g1" },
    { id: "old-1", conversation_id: "g1" },
  ];
  const fresh = [
    { id: "t-new", conversation_id: "g1" },
    { id: "old-2", conversation_id: "g1" },
  ];

  f.ok(!turnRanEngine(LOCAL_COMMAND_STATUS), "本地指令不算跑轮");
  f.ok(turnRanEngine("done"), "done 算跑轮");
  f.ok(turnRanEngine("continue"), "continue 算跑轮");
  f.ok(turnRanEngine("needs_clarification"), "needs_clarification 算跑轮");
  f.ok(turnRanEngine("blocked"), "blocked 算跑轮");
  f.ok(turnRanEngine("waiting"), "waiting 算跑轮");
  f.ok(!turnRanEngine(""), "空 status 必须 fail-closed");
  f.ok(!turnRanEngine("随便什么"), "未知 status 必须 fail-closed");
  f.ok(
    !RUN_TURN_STATUSES.includes(LOCAL_COMMAND_STATUS),
    "本地指令不得混进跑轮终态白名单（否则语义闸自己就漏了）",
  );

  // 缺陷现场：本地指令 + 库里全是上一轮任务 → 绝不挂
  f.eq(
    turnTaskToAttach(LOCAL_COMMAND_STATUS, "g1", before, stale)?.id,
    undefined,
    "本地指令绝不挂上一轮的任务卡（缺陷现场）",
  );
  // 判据必须是语义而非存在性：列表**非空**也照样不挂
  f.eq(
    turnTaskToAttach(LOCAL_COMMAND_STATUS, "g1", new Set<string>(), [])?.id,
    undefined,
    "本地指令 + 空库 → 也不挂（不许用『列表空不空』当判据）",
  );
  // 真实跑轮：只认窗口内新建的那条
  f.eq(
    turnTaskToAttach("done", "g1", before, fresh)?.id,
    "t-new",
    "真实跑轮挂本轮新建的任务",
  );
  f.eq(
    turnTaskToAttach("done", "g1", before, stale)?.id,
    undefined,
    "真实跑轮但无新建 → 不挂，且**不退回 tasks[0]**",
  );
  // 别人的任务不认
  f.eq(
    turnTaskToAttach("done", "g1", before, [{ id: "t-x", conversation_id: "g9" }])?.id,
    undefined,
    "别的会话的新任务不挂",
  );
  f.eq(
    turnTaskToAttach("done", "g1", before, [{ id: "t-x", conversation_id: null }])?.id,
    undefined,
    "无归属的新任务不挂",
  );
  // 读不到就不猜（宁可少挂一张卡，也不挂错）
  f.eq(turnTaskToAttach("done", "g1", null, fresh)?.id, undefined, "起点快照缺失 → 不挂");
  f.eq(turnTaskToAttach("done", "g1", before, null)?.id, undefined, "终点列表缺失 → 不挂");
  // 未知本地标记变体也 fail-closed
  f.eq(
    turnTaskToAttach("本地指令v2", "g1", before, fresh)?.id,
    undefined,
    "未知的本地标记变体必须 fail-closed",
  );
}

export function runSelfTest(): string[] {
  const f = new Failures();
  testTurnTaskAttribution(f);
  testSplitLines(f);
  testIdentity(f);
  testPureAddRemove(f);
  testReplacementIsPaired(f);
  testCharSpansRoundTrip(f);
  testAdjacentSpansAreMerged(f);
  testContextFolding(f);
  testDistantChangesMakeTwoHunks(f);
  testLineNumbers(f);
  testUnbalancedPairIsNotForced(f);
  testAddBeforeDelStillPairs(f);
  testDegradationIsLabelled(f);
  testModerateRewriteStaysExact(f);
  testHighlightEscapesFirst(f);
  testHighlightRoundTripsText(f);
  testLanguageDetection(f);
  testTreeOnlyShowsExpandedDirs(f);
  testTreeSkipsUnloadedDirs(f);
  testTreeNestsDeeply(f);
  testTreeSkipsChildrenOfUnloadedParent(f);
  testTreeLeavesAreNeverExpanded(f);
  testSameEntriesDetectsRealChanges(f);
  testPathHelpers(f);
  testUnreadableDirShowsAPlaceholderRow(f);
  testRenderTreeNameIsInert(f);
  testRenderEscapesQuotesInAttributes(f);
  testRenderGitRowIsInert(f);
  testRenderTallyIsInert(f);
  testRenderDiffTextIsInert(f);
  testDiffRowStructure(f);
  testDiffInlineSpanColours(f);
  testTreeRowAriaAttributes(f);
  testTreeIndentGrowsWithDepth(f);
  testUnreadableRowIsMarkedNotQuoted(f);
  testTruncNoticeNamesEveryDir(f);
  testUnifiedDiffColouring(f);
  testCrumbsAreInert(f);
  testRenderConsumesFlattenedTree(f);
  return f.items;
}
