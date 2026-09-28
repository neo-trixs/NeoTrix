/**
 * 统一 diff 引擎 — 行级 LCS/Myers + 行内字符级高亮。
 *
 * 吸收 `dsh-better-sidebar` 的「统一 diff 渲染」：删红 / 增绿 / 改蓝**配对**
 * + 行内字符级高亮 + 上下文折叠。零依赖手写，理由有二：
 * 1. CSP 是 `script-src 'self'`（无 CDN），任何高亮/编辑库都得 bundle 进来，
 *    而 bundle 一个 Monaco 级编辑器的体积换不来本项目只需要的那 200 行；
 * 2. 更要紧的是**可测**： Myers + 字符级配对是纯函数，能在 node 里单测；
 *    换成一个库就只能靠肉眼验收 diff 画对没有。
 *
 * 算法选型（与 git 同一族，但只取最要紧的一半）：
 * - 先剥公共前后缀（真实改动通常局部，能把 Myers 的输入砍到很小）；
 * - 主体走 **Myers O(ND)**，D 上限 `MAX_EDITS`；超限就**诚实降级**成
 *   「整块替换」并在 `truncated` 上标注 —— 绝不返回一个半对的 diff；
 * - 行内字符级：对配对的 del/add 行再跑一次同算法的字符版。
 *
 * 界律：O(ND) 的 D 有上限，N 也有上限（`MAX_LINES`）。两个上限都到顶时
 * 返回 `truncated: true`，UI 得以显示「已截断」而不是假装列全了。
 */

/** 一行的操作。 */
export type DiffOp = "eq" | "del" | "add";

/** diff 的一行。`oldNo`/`newNo` 为 `null` 表示该侧不存在此行。 */
export interface DiffLine {
  op: DiffOp;
  text: string;
  oldNo: number | null;
  newNo: number | null;
  /** 行内片段（有配对时非空；`eq` 行为 `undefined`）。 */
  spans?: CharSpan[];
}

/** 行内片段。 */
export interface CharSpan {
  kind: "eq" | "del" | "add";
  text: string;
}

/** 一个 hunk（默认 3 行上下文）。 */
export interface DiffHunk {
  /** `@@ -oldStart,oldCount +newStart,newCount @@`。 */
  header: string;
  lines: DiffLine[];
}

/** diff 全体。 */
export interface DiffResult {
  hunks: DiffHunk[];
  /** 新增行数。 */
  added: number;
  /** 删除行数。 */
  removed: number;
  /** 触碰上限被降级/截断（UI 必须如实显示）。 */
  truncated: boolean;
  /** 降级原因（`truncated` 为真时非空，给人看的白话）。 */
  note: string;
}

/**
 * Myers 的 D 上限：超过就整块替换。
 *
 * 数值不是随手取的 —— 它**同时是内存上限**。回溯要存每一步的 `v` 快照，
 * 朴素实现是 `D` 份 `2D+3` 个 int32，即 `8·D²` 字节：`D=4000` 是 **128 MB**，
 * 在浏览器标签页里会直接把 UI 卡死。故：
 * - `MAX_EDITS = 1200` → 朴素实现也要 11.5 MB；
 * - 再叠加下面「每步只存 `2d+1` 格」的省法 → 约 5.8 MB，稳。
 * 超限就整块替换并**标注**（`truncated` + 白话 note），绝不返回半对的 diff。
 */
export const MAX_EDITS = 1200;
/** 单侧行数上限。 */
export const MAX_LINES = 20000;
/** hunk 上下文行数。 */
export const CONTEXT_LINES = 3;

/** 空结果（两侧相同 / 全空）。 */
function emptyDiff(): DiffResult {
  return { hunks: [], added: 0, removed: 0, truncated: false, note: "" };
}

/** 按 `\n` 切行，**保留行尾空行语义**（`"a\n"` 是 1 行，`"a\n\n"` 是 2 行）。 */
export function splitLines(text: string): string[] {
  if (text === "") return [];
  const lines = text.split("\n");
  // 末尾换行不算多出一行空行。
  if (lines.length > 0 && lines[lines.length - 1] === "") lines.pop();
  return lines;
}

/** 通用 Myers：求 `a → b` 的编辑脚本。`a`/`b` 是任意可比较序列。 */
function myers<T>(a: readonly T[], b: readonly T[], maxEdits: number): EditScript<T> | null {
  const n = a.length;
  const m = b.length;
  if (n === 0 && m === 0) return [];
  const max = Math.min(n + m, maxEdits);
  const offset = max;
  // `v[k]` = 当前 d 下、走 k 条对角线能到达的最远 x。
  let v = new Int32Array(2 * max + 3);
  // 关键：第 d 步只会碰 `k ∈ [-d, d]`，即 `2d+1` 格。故快照只存这么宽，
  // 整条 trace 的内存从 `8·D²` 降到约 `4·D²`（MAX_EDITS=1200 → 约 5.8 MB）。
  // 用 `traceIndex(d, k) = k + d` 定位，天然落在 `[0, 2d]`。
  const trace: Int32Array[] = [];
  const traceIndex = (d: number, k: number): number => k + d;
  const at = (k: number): number => v[k + offset];

  for (let d = 0; d <= max; d += 1) {
    trace.push(new Int32Array(2 * d + 1));
    for (let k = -d; k <= d; k += 2) {
      let x: number;
      if (k === -d || (k !== d && at(k - 1) < at(k + 1))) {
        x = at(k + 1);
      } else {
        x = at(k - 1) + 1;
      }
      let y = x - k;
      while (x < n && y < m && same(a[x], b[y])) {
        x += 1;
        y += 1;
      }
      v[k + offset] = x;
      trace[d][traceIndex(d, k)] = x;
      if (x >= n && y >= m) {
        return backtrack(trace, a, b, d);
      }
    }
  }
  return null; // 超出 D 上限
}

/** 序列相等（默认按 `===`；行用字符串、字符用字符串，够用）。 */
function same<T>(x: T, y: T): boolean {
  return x === y;
}

type EditScript<T> = Array<{ op: "eq" | "del" | "add"; a?: T; b?: T }>;

/** 从 trace 回溯出编辑脚本。 */
function backtrack<T>(
  trace: Int32Array[],
  a: readonly T[],
  b: readonly T[],
  dEnd: number,
): EditScript<T> {
  const out: EditScript<T> = [];
  let x = a.length;
  let y = b.length;
  for (let d = dEnd; d > 0; d -= 1) {
    const k = x - y;
    // 走位方向：上一步在 `k+1`（= 插入）还是在 `k-1`（= 删除）。
    //
    // 这里**只能读第 d-1 步**的值（`k±1` 与 d-1 同奇偶，那是 d-1 步写下的格）。
    // 早先的写法是读 `trace[d]` 的快照 —— 那是「朴素实现」，每步存整个 `v`；
    // 改成每步只存 `2d+1` 格之后，`trace[d]` 里只有 d 步自己写的那一格奇偶，
    // `k±1` 全是 0，回溯就会数错增删行数（实测 added/removed 串位）。
    const prev = d - 1;
    const atPrev = (kk: number): number => trace[prev][kk + prev];
    // 这个判断**必须与 forward 里的一字不差**（连比较方向都得一致）：
    //   forward: if (at(k-1) < at(k+1)) x = at(k+1)  // 从 k+1 来 = 插入
    //   here:    if (at(k-1) < at(k+1)) prevK = k+1
    // 曾把两个操作数写反（`at(k+1) < at(k-1)`），回溯就会选**相反**的前驱：
    // 增删行数照样对得上（消耗量守恒），但「相等」的行会被配成不相等的两个字符
    // —— 渲染出来的 diff 里会出现 `一 ↔ 内` 这种根本不匹配的配对。
    // `selftest.ts` 的「charSpans 两侧可还原」不变式就是为抓这类 bug 存在的。
    const prevK = k === -d || (k !== d && atPrev(k - 1) < atPrev(k + 1)) ? k + 1 : k - 1;
    const prevX = atPrev(prevK);
    const prevY = prevX - prevK;
    // 先补走对角线（相等行）。
    while (x > prevX && y > prevY) {
      out.push({ op: "eq", a: a[x - 1], b: b[y - 1] });
      x -= 1;
      y -= 1;
    }
    if (x === prevX) {
      out.push({ op: "add", b: b[y - 1] });
      y -= 1;
    } else {
      out.push({ op: "del", a: a[x - 1] });
      x -= 1;
    }
  }
  while (x > 0 && y > 0) {
    out.push({ op: "eq", a: a[x - 1], b: b[y - 1] });
    x -= 1;
    y -= 1;
  }
  out.reverse();
  return out;
}

/** 剥公共前后缀（把 Myers 的输入砍小；真实改动通常局部）。 */
function trimCommon<T>(a: readonly T[], b: readonly T[]): [T[], T[], number] {
  let start = 0;
  while (start < a.length && start < b.length && same(a[start], b[start])) start += 1;
  let endA = a.length;
  let endB = b.length;
  while (endA > start && endB > start && same(a[endA - 1], b[endB - 1])) {
    endA -= 1;
    endB -= 1;
  }
  return [a.slice(start, endA), b.slice(start, endB), start];
}

/** 行级 diff（`before` → `after`）。 */
export function diffLines(before: string, after: string, context = CONTEXT_LINES): DiffResult {
  const a = splitLines(before);
  const b = splitLines(after);

  // 快捷路：完全相同。
  if (a.length === b.length && a.every((line, i) => line === b[i])) return emptyDiff();

  const [midA, midB, prefix] = trimCommon(a, b);
  const capped = a.length > MAX_LINES || b.length > MAX_LINES;
  const midA2 = capped ? midA.slice(0, MAX_LINES) : midA;
  const midB2 = capped ? midB.slice(0, MAX_LINES) : midB;

  const script = capped ? null : myers(midA2, midB2, MAX_EDITS);
  let full: EditScript<string>;
  let truncated = false;
  let note = "";

  if (script) {
    full = [
      ...a.slice(0, prefix).map((line) => ({ op: "eq" as const, a: line, b: line })),
      ...script,
      ...a.slice(prefix + midA.length).map((line) => ({ op: "eq" as const, a: line, b: line })),
    ];
  } else {
    // 降级：整块替换。**标注出来**，不假装这是精确 diff。
    truncated = true;
    note = capped
      ? `文件超过 ${MAX_LINES} 行，已按整块替换显示（不是精确 diff）`
      : `改动过大（超过 ${MAX_EDITS} 处编辑），已按整块替换显示（不是精确 diff）`;
    full = [
      ...a.slice(0, prefix).map((line) => ({ op: "eq" as const, a: line, b: line })),
      ...midA2.map((line) => ({ op: "del" as const, a: line })),
      ...midB2.map((line) => ({ op: "add" as const, b: line })),
      ...a.slice(prefix + midA.length).map((line) => ({ op: "eq" as const, a: line, b: line })),
    ];
  }

  return assemble(full, context, truncated, note);
}

/** 给编辑脚本编号、分 hunk、算增删。 */
function assemble(
  script: EditScript<string>,
  context: number,
  truncated: boolean,
  note: string,
): DiffResult {
  // 编号。
  let oldNo = 0;
  let newNo = 0;
  const numbered: DiffLine[] = script.map((step) => {
    if (step.op === "eq") {
      oldNo += 1;
      newNo += 1;
      return { op: "eq", text: String(step.a), oldNo, newNo };
    }
    if (step.op === "del") {
      oldNo += 1;
      return { op: "del", text: String(step.a), oldNo, newNo: null };
    }
    newNo += 1;
    return { op: "add", text: String(step.b), oldNo: null, newNo };
  });

  // 配对 del/add（同一处「改」）：连续的 del 段与 add 段按序两两配对，
  // 配上的两行做行内字符级 diff —— 这是 better-sidebar「改蓝配对」的来源。
  pairSpans(numbered);

  let added = 0;
  let removed = 0;
  for (const line of numbered) {
    if (line.op === "add") added += 1;
    if (line.op === "del") removed += 1;
  }

  return { hunks: groupHunks(numbered, context), added, removed, truncated, note };
}

/** 就地为配对行填 `spans`（行内字符级高亮）。 */
function pairSpans(lines: DiffLine[]): void {
  let i = 0;
  while (i < lines.length) {
    if (lines[i].op === "eq") {
      i += 1;
      continue;
    }
    // 「改动块」= 一段**连续的非 eq 行**，del/add 顺序不限。
    //
    // 这里不能写成「先找 del 段再找其后的 add 段」：Myers 回溯出来的顺序
    // 可能是 add 在前、del 在后（`a1→a2` 常得到 add 先出）。那样就永远配不上，
    // 行内高亮整块丢失（实测 `beta→BETA` 完全不高亮）。
    let end = i;
    while (end < lines.length && lines[end].op !== "eq") end += 1;
    const block = lines.slice(i, end);
    // `slice` 取的是**引用**，故下面改 `spans` 就是改 `lines` 里的原对象。
    const dels = block.filter((line) => line.op === "del");
    const adds = block.filter((line) => line.op === "add");
    // 段长不等时按序配对，多出来的一侧整体标（不硬凑）。
    const pairs = Math.min(dels.length, adds.length);
    for (let p = 0; p < pairs; p += 1) {
      const spans = charSpans(dels[p].text, adds[p].text);
      // 两侧挂同一份 spans：del 行看 del 段、add 行看 add 段。
      dels[p].spans = spans;
      adds[p].spans = spans;
    }
    for (let p = pairs; p < dels.length; p += 1) {
      dels[p].spans = [{ kind: "del", text: dels[p].text }];
    }
    for (let p = pairs; p < adds.length; p += 1) {
      adds[p].spans = [{ kind: "add", text: adds[p].text }];
    }
    i = end;
  }
}

/** 行内字符级 diff：`before` → `after`，产出一份两侧共用的 span 序列。 */
export function charSpans(before: string, after: string): CharSpan[] {
  if (before === after) return [{ kind: "eq", text: before }];
  const script = myers(Array.from(before), Array.from(after), 600);
  if (!script) {
    // 字符级也超限：整行对调，标清是粗粒度。
    return [
      { kind: "del", text: before },
      { kind: "add", text: after },
    ];
  }
  const spans: CharSpan[] = [];
  for (const step of script) {
    const kind: CharSpan["kind"] = step.op === "eq" ? "eq" : step.op;
    const text = step.op === "add" ? String(step.b) : String(step.a);
    const last = spans.length > 0 ? spans[spans.length - 1] : null;
    // 合并相邻同类 span（否则一行能出几百个 `<span>`）。
    if (last && last.kind === kind) last.text += text;
    else spans.push({ kind, text });
  }
  return spans;
}

/** 把编号后的行分组成带上下文的 hunk。 */
function groupHunks(lines: DiffLine[], context: number): DiffHunk[] {
  // 标记需要展示的行（改动行 + 前后 context 行）。
  const keep = new Uint8Array(lines.length);
  for (let i = 0; i < lines.length; i += 1) {
    if (lines[i].op === "eq") continue;
    const from = Math.max(0, i - context);
    const to = Math.min(lines.length - 1, i + context);
    for (let j = from; j <= to; j += 1) keep[j] = 1;
  }

  const hunks: DiffHunk[] = [];
  let i = 0;
  while (i < lines.length) {
    if (keep[i] === 0) {
      i += 1;
      continue;
    }
    let end = i;
    while (end < lines.length && keep[end] === 1) end += 1;
    const slice = lines.slice(i, end);
    hunks.push({ header: hunkHeader(slice), lines: slice });
    i = end;
  }
  return hunks;
}

/** `@@ -a,b +c,d @@` 表头。 */
function hunkHeader(lines: DiffLine[]): string {
  const first = lines[0];
  const last = lines[lines.length - 1];
  const oldStart = first.oldNo ?? first.newNo ?? 1;
  const newStart = first.newNo ?? first.oldNo ?? 1;
  const oldCount = lines.filter((line) => line.op !== "add").length;
  const newCount = lines.filter((line) => line.op !== "del").length;
  const range = (start: number, count: number): string =>
    count === 0 ? `${start - 1},0` : count === 1 ? `${start}` : `${start},${count}`;
  void last;
  return `@@ -${range(oldStart, oldCount)} +${range(newStart, newCount)} @@`;
}

/** 改动行数摘要（`+12 −3`；无改动返回空串）。 */
export function diffStat(result: DiffResult): string {
  if (result.added === 0 && result.removed === 0) return "";
  return `+${result.added} −${result.removed}`;
}
