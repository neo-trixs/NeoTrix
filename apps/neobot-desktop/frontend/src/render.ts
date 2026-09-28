/**
 * 侧边栏工作台的**纯渲染** —— 零 IO、零 DOM 状态、零 Tauri 依赖。
 *
 * 为什么要单独成文件：`sidebar.ts` import 了 `@tauri-apps/api`（`invoke`），
 * 于是它**没法**在 plain node 里被单测 import —— 于是「渲染出来的 HTML 对不对」
 * 就从来没被验过：diff 画反了、a11y 属性漏了、文件名里带尖括号就把 innerHTML
 * 炸了，这些**都不会**让 `tsc` 报错，肉眼又看不出来（编译过了不等于画对了）。
 * 树已经吃过一次同样的亏（见 `tree.ts` 头注），故渲染侧照抄那一刀：
 * **只出串的搬走，DOM 查询 / 事件装配 / `invoke` 全部留在 `sidebar.ts`。**
 *
 * 三条被自测盯住的不变式（`selftest.ts` 里有对应小节）：
 * 1. **渲染前先转义**：文件名、git 提交信息、diff 片段、目录路径全都来自磁盘
 *    或模型，都可能是恶意的。凡是拼进串里的动态文本一律过 `esc()`。
 * 2. **如实说**：截断、降级、打不开的目录，界面上都要说出来（见 `truncNoteHtml`）。
 * 3. **a11y 属性不能少**：`role="treeitem"` / `tabindex="0"` / `aria-expanded`
 *    是键盘能走这棵树的前提，少一个键盘用户就卡死在第一行。
 *
 * 本模块**不 import `core.ts`**：那个文件在模块顶层就摸 `document`（`$()`），
 * 一 import 就会在 node 里 ReferenceError。故自带一份 `esc`，口径与 `core.ts`
 * 一致（同 `highlight.ts` 的做法），并由自测钉住字符表。
 */

import { type DiffLine, type DiffResult, diffStat } from "./diff";
import { icon } from "./icons";
import { parentOf, type TreeRow } from "./tree";

/** HTML 转义（与 `core.ts` 的 `esc` 同口径；此处自带一份以免与壳耦合）。 */
function esc(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/** 一次列举最多显示多少（与 Rust 侧 `MAX_DIR_ENTRIES` 对齐的展示口径）。 */
export const TREE_MAX_SHOWN = 2000;

// ---- 文件树 ----

/**
 * 截断提示：**逐个**点名被 Rust 侧截断的目录。
 *
 * 早先只判根目录，于是一个展开了的子目录条目超限时**一声不吭** ——
 * 那正好违反本项目自己定的「如实显示截断」律：用户以为看到的就是全部。
 *
 * `truncated` 传进来而不是读模块状态：这样「哪些目录被截断了」这个决定留在
 * `sidebar.ts`（它才知道 IPC 回了什么），本函数只负责把名单**如实排版**。
 */
export function truncNoteHtml(truncated: readonly string[], maxShown: number = TREE_MAX_SHOWN): string {
  const hit = truncated.slice().sort();
  if (hit.length === 0) return "";
  const label = (rel: string): string => (rel === "" ? "工作区根目录" : esc(rel));
  return `<div class="sb-note warn">${
    hit.length === 1
      ? `${label(hit[0] as string)} 条目过多，只显示前 ${maxShown} 项（不是全部）`
      : `${hit.length} 个目录条目过多（各只显示前 ${maxShown} 项，都不是全部）：${hit.map(label).join("、")}`
  }</div>`;
}

/** 一行树。`open` = 该目录当前是否展开（由调用方从 `expanded` 集合取）。 */
export function treeRowHtml(row: TreeRow, open: boolean): string {
  const { entry, depth } = row;
  const cls = ["sb-row", "sb-tree-row"];
  if (entry.is_dir) cls.push("dir");
  if (entry.broken_link) cls.push("broken");
  if (entry.unreadable) cls.push("unreadable");
  // 打不开的占位不该有展开箭头（点了也拉不到）和 @ 引用（引用一个打不开的路径没用）。
  const caret =
    entry.unreadable || !entry.is_dir
      ? '<span class="sb-caret leaf"></span>'
      : `<span class="sb-caret${open ? " open" : ""}">${icon("chevron", 12)}</span>`;
  const link = entry.is_symlink ? '<span class="sb-link" title="软链接">↗</span>' : "";
  const label = entry.broken_link
    ? `${entry.name}（链接失效）`
    : entry.unreadable
      ? `${entry.name}（打不开）`
      : entry.name;
  // `depth` 来自 `flattenTree`（是整数），不是磁盘字符串，故不转义。
  return `<div class="${cls.join(" ")}" role="treeitem" tabindex="0"
    aria-expanded="${entry.is_dir ? String(open) : ""}"
    data-rel="${esc(entry.rel)}" data-dir="${entry.is_dir ? 1 : 0}" data-depth="${depth}"
    style="padding-left:${6 + depth * 14}px">
    ${caret}
    <span class="sb-ico">${icon(entry.is_dir ? "folder" : "file", 14)}</span>
    <span class="sb-name">${esc(label)}</span>${link}
    ${
      entry.unreadable
        ? ""
        : `<button class="sb-mini" data-act="ref" data-path="${esc(entry.rel)}" title="引用进输入框">@</button>`
    }
  </div>`;
}

/**
 * 面包屑 = **当前打开文件所在目录**的路径链。
 *
 * 点击某一级 = 「展开到那一层」：把它的祖先逐个加进 `expanded`（并按需加载），
 * 于是那一行在树里露出来。这是树视图里面包屑唯一有意义的动作 ——
 * 早先的写法把面包屑当成「导航到那个目录」，结果树根本不是树。
 */
export function crumbsHtml(rel: string): string {
  const parts = parentOf(rel).split("/").filter(Boolean);
  const out = ['<button class="sb-crumb" data-crumb="">工作区</button>'];
  let acc = "";
  for (const part of parts) {
    acc = acc ? `${acc}/${part}` : part;
    out.push(
      `<span class="sb-sep">/</span><button class="sb-crumb" data-crumb="${esc(acc)}">${esc(part)}</button>`,
    );
  }
  out.push('<span class="sb-sep">/</span>');
  out.push(`<span class="sb-crumb cur mono">${esc(rel.slice(rel.lastIndexOf("/") + 1))}</span>`);
  return out.join("");
}

// ---- 变动页（只用到渲染的字段；Rust 侧 DTO 见 `nt_cmd_files.rs`） ----

/** 路径计数（`neobot_changes_recent_paths` 回的每一项）。 */
export interface PathTally {
  path: string;
  reads: number;
  writes: number;
  edits: number;
  last_change: string;
}

/** Git 状态的一行（`neobot_git_status`）。 */
export interface GitFile {
  path: string;
  xy: string;
  staged: boolean;
  unstaged: boolean;
  untracked: boolean;
}

export function tallyHtml(t: PathTally): string {
  const ops: string[] = [];
  if (t.reads) ops.push(`读${t.reads}`);
  if (t.writes) ops.push(`写${t.writes}`);
  if (t.edits) ops.push(`改${t.edits}`);
  return `<div class="sb-row">
    <span class="sb-ico">${icon("file", 14)}</span>
    <span class="sb-name">${esc(t.path)}</span>
    <span class="sb-badge" data-act="change" data-id="${esc(t.last_change)}" title="看这一处的 diff">${esc(ops.join(" · "))}</span>
  </div>`;
}

export function gitRowHtml(file: GitFile): string {
  const stage = file.staged
    ? '<button class="sb-mini" data-act="git-unstage" data-path="' + esc(file.path) + '">取消暂存</button>'
    : '<button class="sb-mini" data-act="git-stage" data-path="' + esc(file.path) + '">暂存</button>';
  const revert = file.untracked
    ? ""
    : '<button class="sb-mini danger" data-act="git-revert" data-path="' + esc(file.path) + '" title="丢弃未提交改动">还原</button>';
  return `<div class="sb-row">
    <span class="sb-xy mono" title="${esc(file.xy)}">${esc(file.xy)}</span>
    <span class="sb-name">${esc(file.path)}</span>
    ${stage}${revert}
    <button class="sb-mini" data-act="git-diff" data-path="${esc(file.path)}" data-staged="${file.staged ? 1 : 0}">diff</button>
  </div>`;
}

// ---- Diff ----

/** 统一 diff 结果 → HTML。 */
export function diffHtml(result: DiffResult, label: string): string {
  const head = `<div class="sb-stat">${esc(label)} <span class="sb-add">+${result.added}</span> <span class="sb-del">−${result.removed}</span></div>`;
  if (result.truncated) {
    // 界律 2：降级必须说出来。
    return head + `<div class="sb-note warn">${esc(result.note)}</div>`;
  }
  if (result.hunks.length === 0) return head + '<div class="sb-empty">没有差异</div>';
  let out = "";
  for (const hunk of result.hunks) {
    out += `<div class="sb-dl hunk">${esc(hunk.header)}</div>`;
    for (const line of hunk.lines) out += lineHtml(line);
  }
  return head + `<div class="sb-stat">${esc(diffStat(result))}</div>` + out;
}

/** diff 一行（含行内 span 与行号）。 */
export function lineHtml(line: DiffLine): string {
  const no = line.op === "add" ? line.newNo : line.oldNo;
  const marker = line.op === "add" ? "+" : line.op === "del" ? "−" : " ";
  let text: string;
  if (line.spans) {
    text = line.spans
      .map((span) => {
        const body = esc(span.text) || "&nbsp;";
        return span.kind === "eq" ? body : `<span class="sb-in-${span.kind}">${body}</span>`;
      })
      .join("");
  } else {
    text = esc(line.text) || "&nbsp;";
  }
  return `<div class="sb-dl ${line.op}"><span class="sb-no">${no ?? ""}</span><span class="sb-mk">${marker}</span><span class="sb-tx">${text}</span></div>`;
}

/** unified diff 文本 → HTML（按 hunk 头 / +− 着色，不重算 diff）。 */
export function unifiedHtml(text: string): string {
  const out: string[] = [];
  for (const line of text.split("\n")) {
    const cls = line.startsWith("@@")
      ? "sb-dl hunk"
      : line.startsWith("+++") || line.startsWith("---")
        ? "sb-dl meta"
        : line.startsWith("+")
          ? "sb-dl add"
          : line.startsWith("-")
            ? "sb-dl del"
            : "sb-dl eq";
    out.push(`<div class="${cls}">${esc(line) || " "}</div>`);
  }
  return out.join("");
}
