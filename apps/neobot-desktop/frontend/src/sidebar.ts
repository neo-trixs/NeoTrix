/**
 * 侧边栏工作台 — 吸收 `dsh-better-sidebar` 的右列工作台。
 *
 * 四页签（文件 / 变动 / 任务 / 侧聊）+ 顶部页签条 + 可拖拽把手。
 * 页签表**不在前端写死**：一律从 Rust 的 `neobot_sidebar_tabs` 拿
 * （`nt_sidebar::TabRegistry`）。理由与源仓库一致 —— 决策集中在一处，
 * 前端各处 `if (tab === "files")` 猜，猜散了就必有一处猜错且无人发现。
 *
 * 三条界律（与 Rust 侧同源，不是各写各的）：
 * 1. **前端不发一个 fetch**：文件读写、Git、侧聊全走 `invoke`，
 *    路径在 Rust 侧过 `nt_workspace` 的两道 jail。绕开它就等于在前端重写一遍越狱判定。
 * 2. **如实显示截断**：Rust 回 `truncated`/`content_omitted`/`untracked_new`
 *    时，UI 必须说「已截断 / 内容过大 / 新增文件」，不假装列全了。
 * 3. **渲染前先转义**：所有插进 innerHTML 的文本一律经 `esc()`；文件内容、
 *    diff 片段、Git 提交信息全都来自磁盘或模型，都可能是恶意的。
 *
 * 纯「出串」的渲染函数都在 `render.ts`（零依赖，可被自测台 import）：
 * 本文件只留 **DOM 查询 / 事件装配 / `invoke`**。同 `tree.ts` 的理由 ——
 * 混在一起时 `invoke` 让整个模块没法在 plain node 里测，于是「画出来对不对」
 * 从来没人验过。
 */

import { ntInvoke } from "./invoke";

import { esc, toast } from "./core";
import { diffLines } from "./diff";
import { detectLanguage, highlight } from "./highlight";
import { icon, type IconName } from "./icons";
import {
  crumbsHtml,
  diffHtml,
  gitRowHtml,
  tallyHtml,
  treeRowHtml,
  truncNoteHtml,
  unifiedHtml,
  type GitFile as GitFileView,
  type PathTally as PathTallyView,
} from "./render";
import { ancestorsOf, flattenTree, parentOf, sameEntries, type TreeEntry, type TreeRow } from "./tree";

// ---- Rust 侧 DTO（与 nt_cmd_files.rs / nt_cmd_sidebar.rs 同形）----

export interface SidebarTab {
  id: string;
  title: string;
  icon: string;
  order: number;
  needs: string[];
}

export interface ViewerSpec {
  id: string;
  label: string;
  extensions: string[];
  editable: boolean;
  language: string;
}

export type DirEntry = TreeEntry;

export interface DirListing {
  rel: string;
  parent: string | null;
  entries: DirEntry[];
  truncated: boolean;
}

export interface FileText {
  rel: string;
  content: string;
  size: number;
  mtime: number;
  truncated: boolean;
  binary: boolean;
}

export interface SearchHit {
  rel: string;
  name: string;
  is_dir: boolean;
  size: number;
  mtime: number;
}

/** 哪条上限砍掉了搜索结果（与 nt_workspace::SearchCap 同名）。 */
export type SearchCap = "hits" | "visits" | "depth";

/** 与 `DirListing` / `FileText` 同一口径：结果体 + `truncated`。 */
export interface SearchResults {
  hits: SearchHit[];
  truncated: boolean;
  truncated_by: SearchCap | null;
}

export interface ChangeItem {
  id: string;
  task_id: string;
  at: string;
  path: string;
  kind: string;
  bytes: number;
  content_omitted: boolean;
}

export interface ChangeView {
  change: ChangeItem;
  before: string | null;
  after: string | null;
}

export interface RecentPaths {
  paths: PathTally[];
  task_id: string | null;
  task_title: string | null;
}

/** 与 Rust 侧同形；渲染用到的字段由 `render.ts` 的同形子集定义（那边要能脱离本文件被单测）。 */
export interface PathTally extends PathTallyView {
  bytes: number;
  last_at: string;
}

export type GitFile = GitFileView;

export interface GitDiff {
  text: string;
  untracked_new: boolean;
}

export interface GitCommit {
  id: string;
  short: string;
  at: string;
  author: string;
  subject: string;
}

export interface SideThread {
  id: string;
  kind: string;
  title: string;
  created_at: string;
  last_active: string;
  task_count: number;
  unread: number;
  parent_id: string | null;
  origin: string;
}

// ---- 壳状态 ----

/** 当前激活页签。 */
let activeTab = "files";
/** 页签表（Rust 给的）。 */
let tabs: SidebarTab[] = [];
/** 文件树：已展开的目录集合。 */
const expanded = new Set<string>([""]);
/** 文件树：已加载的目录 → 条目（**缓存是重绘的唯一数据源**）。 */
const dirCache = new Map<string, DirEntry[]>();
/** 被 Rust 侧截断过的目录（`MAX_DIR_ENTRIES`）—— 单独一张表，别往缓存里塞哨兵。 */
const dirTrunc = new Set<string>();
/** 展开过但**拉取失败**的目录（权限 / 断链 / 已删）—— 树上要显示「打不开」。 */
const dirFailed = new Set<string>();
/** 树当前聚焦的文件（面包屑显示它的目录链；点面包屑=展开祖先把它露出来）。 */
let treeFocus = "";
/** 文件树：搜索词（非空时树切换成搜索结果）。 */
let searchQuery = "";
/** 当前打开的文件（编辑器状态）。 */
interface EditorState {
  rel: string;
  original: string;
  draft: string;
  viewer: ViewerSpec | null;
  file: FileText;
}
let editor: EditorState | null = null;
/** 当前选中的改动（diff 面板状态）。 */
let currentChange: ChangeView | null = null;
/** 当前母会话 id（侧聊页要用；由 main.ts 注入）。 */
let parentConvoId = "";
/** 当前打开的侧聊 id。 */
let currentThread = "";
/** Git 改动清单缓存。 */
let gitFiles: GitFile[] = [];
/** Git 可用性（不探活就不显示 Git 分组，不给用户一个点不动的按钮）。 */
let gitOk = false;

// ---- 入口 ----

/** 挂载侧边栏（由 main.ts 在 boot 末尾调一次）。 */
export function mountSidebar(parent: HTMLElement, injected: SidebarOpts): void {
  opts = injected;
  render(parent, injected);
}

/** 外部注入的能力（避免 sidebar 反向依赖 main.ts 的内部状态）。 */
export interface SidebarOpts {
  /** 把 `@路径` 引用插进主输入框。 */
  onInsertRef: (text: string) => void;
  /** 取当前会话 id。 */
  onGetParentConvoId: () => string;
  /** 跑一轮对话（侧聊用）。 */
  onRunTurn: (text: string, convoId: string) => Promise<void>;
}

let opts: SidebarOpts | null = null;

/**
 * 展开目录的轮询刷新（对齐 better-sidebar 的「展开的目录自动重列」）。
 *
 * 真 fs watch 需要平台相关的原生句柄，且 Tauri 的 fs 插件权限我们**刻意没放宽**，
 * 所以这里用「重拉已展开的目录 + 内容没变就不重绘」达到同样的效果：
 * 代价是一次几个 IPC，收益是不碰 ACL、不引入新依赖。
 *
 * 只有**可见且已展开**的目录会被重拉 —— 折叠着的目录没有看的必要，
 * 全量重拉在大仓里会把 IPC 打成瀑布。
 */
/**
 * 侧边栏的周期刷新（挂主壳那个 15s 心跳上）。
 *
 * 早先只有文件树有轮询，于是「变动」「任务」两页的数据会一直停在打开那一刻 ——
 * 模型明明刚改完文件，那一页还不动。编辑器开着的时候不重绘（会打断打字）。
 */
export async function refreshSidebarPanels(): Promise<void> {
  const root = document.getElementById("sb-panel");
  if (!root || root.classList.contains("hidden")) return;
  if (activeTab === "files") return; // 树有自己的 4s 轮询
  if (editor) return; // 正在编辑，别重绘
  if (currentChange) return; // 正在看某个 diff，别把人踢回列表
  const body = root.querySelector<HTMLElement>("#sb-body");
  if (!body) return;
  const scroll = body.scrollTop;
  if (activeTab === "changes") await paintChanges(root);
  else if (activeTab === "tasks") await paintTasks(root);
  else if (activeTab === "chat") await paintChat(root);
  // 重绘后把滚动位置放回去（否则每 15 秒一次地跳回顶部，比不刷新还烦）。
  const after = root.querySelector<HTMLElement>("#sb-body");
  if (after) after.scrollTop = scroll;
}

export async function refreshSidebarTree(): Promise<void> {
  if (activeTab !== "files" || searchQuery.trim() || editor) return;
  const root = document.getElementById("sb-panel");
  if (!root || !dirLoaded("")) return;
  // 面板收起时不轮询：`.hidden` 只是 `display:none`（元素还在 DOM 里），
  // 不判这一条就会在用户根本看不见工作台时每 4 秒发一轮 IPC。
  if (root.classList.contains("hidden")) return;
  const targets = Array.from(expanded).filter((rel) => dirLoaded(rel));
  let changed = false;
  for (const rel of targets) {
    try {
      const listing = await ntInvoke<DirListing>("neobot_fs_list", { rel });
      const before = dirCache.get(rel);
      if (!before || !sameEntries(before, listing.entries)) {
        dirCache.set(rel, listing.entries);
        if (listing.truncated) dirTrunc.add(rel);
        else dirTrunc.delete(rel);
        changed = true;
      }
    } catch {
      // 目录在刷新途中被删了：把它从缓存和展开集里摘掉（否则每次都白报一次错）。
      dirCache.delete(rel);
      dirTrunc.delete(rel);
      expanded.delete(rel);
      changed = true;
    }
  }
  if (changed) paintTree(root);
}

/** 目录内容是否变了（按 name+dir+size 比；mtime 变了但大小没变也当没变，省得重绘）。 */

/** 整块重绘。 */
function render(root: HTMLElement, injected: SidebarOpts): void {
  opts = injected;
  parentConvoId = injected.onGetParentConvoId();
  root.innerHTML = shellHtml();
  wire(root);
  void refreshActive(root);
}

// ---- 骨架 ----

function shellHtml(): string {
  return `
    <div class="sb-tabs" id="sb-tabs">${tabsHtml()}</div>
    <div class="sb-body" id="sb-body"></div>
    <div class="sb-status" id="sb-status"></div>
  `;
}

function tabsHtml(): string {
  return tabs
    .slice()
    .sort((a, b) => a.order - b.order || a.id.localeCompare(b.id))
    .map(
      (tab) =>
        `<button class="sb-tab${tab.id === activeTab ? " on" : ""}" data-tab="${esc(tab.id)}" title="${esc(tab.title)}">${icon(tab.icon as IconName, 16)}<span>${esc(tab.title)}</span></button>`,
    )
    .join("");
}

// ---- 事件装配（事件留在本模块；渲染只管出串） ----

function wire(root: HTMLElement): void {
  const tabsEl = root.querySelector<HTMLElement>("#sb-tabs");
  tabsEl?.addEventListener("click", (ev) => {
    const btn = (ev.target as HTMLElement).closest<HTMLElement>("[data-tab]");
    if (!btn) return;
    activeTab = btn.dataset["tab"] ?? "files";
    tabsEl.innerHTML = tabsHtml();
    void refreshActive(root);
  });

  const body = root.querySelector<HTMLElement>("#sb-body");
  body?.addEventListener("click", (ev) => {
    const t = ev.target as HTMLElement;
    const act = (name: string): HTMLElement | null => t.closest<HTMLElement>(`[data-act="${name}"]`);
    // 目录展开 / 文件打开
    const treeItem = t.closest<HTMLElement>("[data-rel]");
    if (treeItem && !act("none")) {
      const rel = treeItem.dataset["rel"] ?? "";
      if (treeItem.dataset["dir"] === "1") {
        void toggleDir(root, rel);
        return;
      }
      treeFocus = rel;
      void openFile(root, rel);
      return;
    }
    // 面包屑 = 展开到那一层（把目标行在树里露出来）
    const crumb = t.closest<HTMLElement>("[data-crumb]");
    if (crumb) {
      void revealPath(root, crumb.dataset["crumb"] ?? "");
      return;
    }
    // 搜索
    const clear = act("search-clear");
    if (clear) {
      searchQuery = "";
      dirCache.clear();
      void refreshActive(root);
      return;
    }
    // 编辑器
    if (act("save")) {
      void saveFile(root);
      return;
    }
    if (act("revert")) {
      if (editor) editor.draft = editor.original;
      paintEditor(root);
      return;
    }
    if (act("editor-close")) {
      editor = null;
      void refreshActive(root);
      return;
    }
    // 插入引用
    const ref = act("ref");
    if (ref) {
      opts?.onInsertRef(`@${ref.dataset["path"] ?? ""} `);
      return;
    }
    // 变动页
    if (act("change")) {
      void openChange(root, act("change")?.dataset["id"] ?? "");
      return;
    }
    // Git
    if (act("git-diff")) {
      void openGitDiff(root, act("git-diff")?.dataset["path"] ?? "", act("git-diff")?.dataset["staged"] === "1");
      return;
    }
    const gitAct = act("git-stage") ?? act("git-unstage");
    if (gitAct) {
      void gitMutate(root, gitAct.dataset["act"] === "git-stage" ? "stage" : "unstage", gitAct.dataset["path"] ?? "");
      return;
    }
    if (act("git-revert")) {
      const path = act("git-revert")?.dataset["path"] ?? "";
      if (!window.confirm(`丢弃 ${path} 的未提交改动？此操作不可撤销。`)) return;
      void gitMutate(root, "revert", path);
      return;
    }
    if (act("git-commit")) {
      void doCommit(root);
      return;
    }
    if (act("git-commit-open")) {
      void gitCommit(root);
      return;
    }
    // 侧聊
    if (act("thread-new")) {
      void newThread(root);
      return;
    }
    const th = t.closest<HTMLElement>("[data-thread]");
    if (th) {
      currentThread = th.dataset["thread"] ?? "";
      void refreshActive(root);
      return;
    }
    if (act("thread-promote")) {
      void promoteThread(root);
      return;
    }
    if (act("thread-send")) {
      void sendThread(root);
    }
  });

  // 键盘：树要能用键盘走一遍（role=tree 的基本约定）。
  // Enter/Space = 目录开合 / 文件打开；←→ = 收起 / 展开并移到子项。
  body?.addEventListener("keydown", (ev) => {
    const row = (ev.target as HTMLElement).closest<HTMLElement>("[data-rel]");
    if (!row) return;
    const rel = row.dataset["rel"] ?? "";
    const isDir = row.dataset["dir"] === "1";
    const move = (delta: number): void => {
      const rows = Array.from(body.querySelectorAll<HTMLElement>("[data-rel]"));
      const idx = rows.indexOf(row);
      const next = rows[idx + delta];
      next?.focus();
    };
    switch (ev.key) {
      case "Enter":
        ev.preventDefault();
        if (isDir) void toggleDir(root, rel);
        else {
          treeFocus = rel;
          void openFile(root, rel);
        }
        return;
      case " ":
        ev.preventDefault();
        if (isDir) void toggleDir(root, rel);
        return;
      case "ArrowRight":
        if (isDir && !expanded.has(rel)) {
          ev.preventDefault();
          void toggleDir(root, rel);
        }
        return;
      case "ArrowLeft":
        if (isDir && expanded.has(rel)) {
          ev.preventDefault();
          void toggleDir(root, rel);
        } else {
          // 已收起：跳到父目录那一行。
          ev.preventDefault();
          const parent = parentOf(rel);
          if (parent !== rel) {
            body.querySelector<HTMLElement>(`[data-rel="${cssEscape(parent)}"]`)?.focus();
          }
        }
        return;
      case "ArrowDown":
        ev.preventDefault();
        move(1);
        return;
      case "ArrowUp":
        ev.preventDefault();
        move(-1);
        return;
      case "Home":
        ev.preventDefault();
        body.querySelector<HTMLElement>("[data-rel]")?.focus();
        return;
      case "End": {
        ev.preventDefault();
        const rows = body.querySelectorAll<HTMLElement>("[data-rel]");
        rows[rows.length - 1]?.focus();
        return;
      }
      default:
    }
  });

  body?.addEventListener("input", (ev) => {
    const t = ev.target as HTMLInputElement | HTMLTextAreaElement;
    if (t.id === "sb-search") {
      searchQuery = t.value;
      void refreshActive(root);
      return;
    }
    if (t.id === "sb-editor" && editor) {
      editor.draft = t.value;
      paintDirty(root);
    }
    if (t.id === "sb-thread-input") {
      paintThreadSend(root);
    }
    if (t.id === "sb-commit-msg") {
      paintThreadSend(root);
    }
  });
}

/** 切换目录展开。 */
async function toggleDir(root: HTMLElement, rel: string): Promise<void> {
  if (expanded.has(rel)) {
    expanded.delete(rel);
  } else {
    expanded.add(rel);
    // 懒加载：第一次展开才去拉。断链/无权限的目录拉不到就地放弃，
    // 不阻断整棵树 —— 但**要在那一行说清**，否则用户看到的是一个
    // 「展开着但是空的」目录，会以为是文件真没了。
    if (!dirLoaded(rel)) await loadDirCache(rel);
  }
  paintTree(root);
}

// ---- 数据加载 ----

/** 拉一个目录进缓存（**只更缓存，不重绘**）。
 *
 * 重绘一律走 `paintTree`（同步、纯从缓存算），这样展开目录时不会出现
 * 「先空白再刷出内容」的抖动 —— 早先的写法是拉完直接重绘整页。
 */
async function loadDirCache(rel: string): Promise<boolean> {
  try {
    const listing = await ntInvoke<DirListing>("neobot_fs_list", { rel });
    dirCache.set(listing.rel, listing.entries);
    if (listing.truncated) dirTrunc.add(listing.rel);
    else dirTrunc.delete(listing.rel);
    dirFailed.delete(listing.rel);
    return true;
  } catch (err) {
    // 拉不到就在树上标出来（而不是只弹个 toast 就没了）。
    dirFailed.add(rel);
    toast(String(err), "err");
    return false;
  }
}

/** 目录是否已加载（决定要不要去拉）。 */
function dirLoaded(rel: string): boolean {
  return dirCache.has(rel);
}

async function refreshActive(root: HTMLElement): Promise<void> {
  switch (activeTab) {
    case "files":
      await paintFiles(root);
      break;
    case "changes":
      await paintChanges(root);
      break;
    case "tasks":
      await paintTasks(root);
      break;
    case "chat":
      await paintChat(root);
      break;
    default:
      break;
  }
  if (tabs.length === 0) {
    const list = await ntInvoke<SidebarTab[]>("neobot_sidebar_tabs");
    tabs = list;
    const tabsEl = root.querySelector<HTMLElement>("#sb-tabs");
    if (tabsEl) tabsEl.innerHTML = tabsHtml();
  }
  if (!gitOk) {
    gitOk = await ntInvoke<boolean>("neobot_git_available");
  }
}

// ---- 文件页 ----

async function paintFiles(root: HTMLElement): Promise<void> {
  // 根目录没缓存就拉一次；之后每次重绘都是纯同步的。
  if (!dirLoaded("")) {
    const ok = await loadDirCache("");
    if (!ok) {
      const body = root.querySelector<HTMLElement>("#sb-body");
      if (body) body.innerHTML = '<div class="sb-empty">读工作区失败（见提示）</div>';
      return;
    }
  }
  paintTree(root);
}

async function paintSearch(root: HTMLElement): Promise<void> {
  const body = root.querySelector<HTMLElement>("#sb-body");
  if (!body) return;
  if (body.querySelector("#sb-results") === null) body.innerHTML = searchShellHtml();
  try {
    const res = await ntInvoke<SearchResults>("neobot_fs_find", { query: searchQuery });
    const list = root.querySelector<HTMLElement>("#sb-results");
    if (!list) return;
    const hits = res.hits;
    // 截断**只信 Rust 的标志位**，不靠 `hits.length >= 200` 猜：
    // 猜的话「恰好 200 个匹配」会被误报成被砍过（假截断），
    // 而访问数 / 深度撞顶这两种真截断又会被漏报（假完整）。
    const note = res.truncated ? searchTruncNoteHtml(res) : "";
    if (hits.length === 0) {
      // 空结果 + 已截断 ≠ 「没有这个文件」：没扫的地方还没扫。
      list.innerHTML =
        note +
        (res.truncated
          ? `<div class="sb-empty">已扫过的部分里没有匹配「${esc(searchQuery)}」的文件（搜索已截断，不能据此断定它不存在）</div>`
          : `<div class="sb-empty">没有匹配「${esc(searchQuery)}」的文件</div>`);
      return;
    }
    list.innerHTML =
      note +
      hits
        .map(
          (hit) => `<div class="sb-row" data-rel="${esc(hit.rel)}" data-dir="${hit.is_dir ? 1 : 0}">
            <span class="sb-ico">${icon(hit.is_dir ? "folder" : "file", 14)}</span>
            <span class="sb-name">${esc(hit.name)}</span>
            <span class="sb-sub mono">${esc(dirOf(hit.rel))}</span>
          </div>`,
        )
        .join("");
  } catch (err) {
    body.innerHTML = `<div class="sb-empty">搜索失败：${esc(String(err))}</div>`;
  }
}

/**
 * 搜索截断提示。与 `truncNoteHtml`（目录项数截断）同一措辞口径：
 * `sb-note warn` + 说清「不是全部」。
 *
 * 三条上限给不同的「为什么」，免得用户把「只扫了一半目录」误读成
 * 「这个文件不存在」——半句不同的提示，结论就完全相反。
 */
function searchTruncNoteHtml(res: SearchResults): string {
  const n = res.hits.length;
  const why =
    res.truncated_by === "visits"
      ? `扫描的目录过多，部分目录只扫了一半（已列出 ${n} 条）`
      : res.truncated_by === "depth"
        ? `目录层级过深，更深处没有搜索（已列出 ${n} 条）`
        : `只显示前 ${n} 条匹配`;
  return `<div class="sb-note warn">已截断，结果可能不完整：${why}（可能还有更多）</div>`;
}

function dirOf(rel: string): string {
  const idx = rel.lastIndexOf("/");
  return idx === -1 ? "" : rel.slice(0, idx);
}

/** 摊平当前模块状态下的树。 */
function flattenVisible(): TreeRow[] {
  return flattenTree(dirCache, expanded, "", dirFailed);
}

/** 文件页（同步重绘：数据全来自缓存）。 */
function paintTree(root: HTMLElement): void {
  const body = root.querySelector<HTMLElement>("#sb-body");
  if (!body) return;
  if (editor) {
    paintEditor(root);
    return;
  }
  if (searchQuery.trim()) {
    body.innerHTML = searchShellHtml();
    void paintSearch(root);
    return;
  }
  const rows = flattenVisible();
  body.innerHTML = `
    <div class="sb-searchrow">
      <input id="sb-search" value="${esc(searchQuery)}" placeholder="搜索文件名…" />
      ${searchQuery ? '<button class="sb-mini" data-act="search-clear">清除</button>' : ""}
    </div>
    ${treeFocus ? `<div class="sb-crumbs">${crumbsHtml(treeFocus)}</div>` : ""}
    <div class="sb-tree" id="sb-list" role="tree" aria-label="工作区文件树">
      ${rows.length === 0 ? '<div class="sb-empty">空目录</div>' : rows.map((row) => treeRowHtml(row, expanded.has(row.entry.rel))).join("")}
    </div>
    ${truncNoteHtml(Array.from(dirTrunc))}
  `;
  const input = body.querySelector<HTMLInputElement>("#sb-search");
  // 只有「本来就在搜」时才抢焦点；展开/折叠时别把光标从树上抢走。
  if (searchQuery.trim() && document.activeElement !== input) input?.focus();
}

/** 展开到某路径（面包屑点击 / 定位用）。 */
async function revealPath(root: HTMLElement, rel: string): Promise<void> {
  for (const ancestor of ancestorsOf(rel)) {
    expanded.add(ancestor);
    if (!dirLoaded(ancestor)) await loadDirCache(ancestor);
  }
  treeFocus = rel;
  paintTree(root);
  // 露出来之后把焦点挪到那一行（键盘用户接着就能 Enter 打开）。
  const row = root.querySelector<HTMLElement>(`[data-rel="${cssEscape(rel)}"]`);
  row?.focus();
  row?.scrollIntoView({ block: "nearest" });
}

/** 极简 CSS 标识符转义（路径里可能有引号 / 方括号）。 */
function cssEscape(value: string): string {
  return value.replace(/["\\]/g, "\\$&");
}

// ---- 编辑器 ----

/**
 * 打开文件。**先问未保存的改动**。
 *
 * 早先的写法直接覆盖 `editor`，于是「改了没存 → 点另一个文件」会把改动
 * 静默丢掉 —— 没有任何提示、没有任何撤销。那是实打实的数据丢失，
 * 比功能缺失严重得多。
 */
async function openFile(root: HTMLElement, rel: string): Promise<void> {
  if (editor && editor.draft !== editor.original && editor.rel !== rel) {
    const path = editor.rel;
    const choice = window.prompt(
      `${path} 有未保存的改动。\n输入 s 保存后继续，或直接回车放弃改动。`,
      "s",
    );
    if (choice === null) return; // 取消 = 什么都不做
    const trimmed = choice.trim().toLowerCase();
    if (trimmed === "s") {
      await saveFile(root);
      // 保存失败（超预算 / 越狱）就别继续了 —— 继续等于把改动扔掉。
      if (editor && editor.draft !== editor.original) return;
    }
  }
  try {
    const file = await ntInvoke<FileText>("neobot_fs_read", { rel });
    // 二进制不塞进 textarea（塞进去是乱码，还可能被当作可编辑）。
    if (file.binary) {
      editor = null;
      toast(`${rel} 是二进制文件，不提供编辑`, "err");
      return;
    }
    const viewer = await ntInvoke<ViewerSpec | null>("neobot_sidebar_viewer_for", { rel });
    editor = {
      rel,
      original: file.content,
      draft: file.content,
      viewer,
      file,
    };
    treeFocus = rel;
    paintEditor(root);
  } catch (err) {
    toast(String(err), "err");
  }
}

function paintEditor(root: HTMLElement): void {
  const body = root.querySelector<HTMLElement>("#sb-body");
  if (!body || !editor) return;
  const st = editor;
  const lang = st.viewer ? (st.viewer.language === "auto" ? detectLanguage(st.rel) : st.viewer.language) : "plain";
  const editable = st.viewer?.editable !== false && !st.file.truncated;
  const meta: string[] = [`${fmtBytes(st.file.size)}`, new Date(st.file.mtime * 1000).toLocaleString()];
  if (st.file.truncated) meta.push("已截断（超出预览上限）");
  body.innerHTML = `
    <div class="sb-editorhead">
      <button class="sb-mini" data-act="editor-close" title="关闭">${icon("x", 14)}</button>
      <span class="sb-path mono" title="${esc(st.rel)}">${esc(st.rel)}</span>
      <div class="grow"></div>
      <span class="sb-dirty" id="sb-dirty"></span>
      ${editable ? '<button class="sb-mini primary" data-act="save">' + icon("save", 14) + "保存</button>" : ""}
    </div>
    <div class="sb-meta">${esc(meta.join(" · "))}</div>
    <div class="sb-editorwrap">
      <pre class="sb-code hl" id="sb-code">${highlight(st.draft, lang)}</pre>
      <textarea id="sb-editor" spellcheck="false" ${editable ? "" : "readonly"}>${esc(st.draft)}</textarea>
    </div>
  `;
  const ta = body.querySelector<HTMLTextAreaElement>("#sb-editor");
  const pre = body.querySelector<HTMLElement>("#sb-code");
  if (ta && pre) {
    // 编辑时不高亮（重排会跳光标）；失焦后再刷。
    ta.addEventListener("blur", () => paintCode(root));
    // 滚动同步（CSS 注释里那句「滚动同步」得有代码顶着）：
    // 两侧同字体同行高，一侧滚另一侧跟。用 guard 防互相触发成死循环
    // （scroll 事件在同步赋值时不会再触发，但异步布局下会，故显式设闸）。
    let syncing = false;
    const link = (from: HTMLElement, to: HTMLElement): void => {
      if (syncing) return;
      syncing = true;
      to.scrollTop = from.scrollTop;
      to.scrollLeft = from.scrollLeft;
      requestAnimationFrame(() => {
        syncing = false;
      });
    };
    ta.addEventListener("scroll", () => link(ta, pre));
    pre.addEventListener("scroll", () => link(pre, ta));
  }
  paintDirty(root);
  paintCode(root);
}

function paintCode(root: HTMLElement): void {
  const pre = root.querySelector<HTMLElement>("#sb-code");
  if (!pre || !editor) return;
  const lang = editor.viewer
    ? editor.viewer.language === "auto"
      ? detectLanguage(editor.rel)
      : editor.viewer.language
    : "plain";
  pre.innerHTML = highlight(editor.draft, lang);
}

function paintDirty(root: HTMLElement): void {
  const el = root.querySelector<HTMLElement>("#sb-dirty");
  if (!el || !editor) return;
  const dirty = editor.draft !== editor.original;
  el.textContent = dirty ? "未保存" : "";
  el.className = dirty ? "sb-dirty on" : "sb-dirty";
}

async function saveFile(root: HTMLElement): Promise<void> {
  if (!editor) return;
  const st = editor;
  if (st.draft === st.original) return;
  try {
    await ntInvoke("neobot_fs_write", { rel: st.rel, content: st.draft });
    st.original = st.draft;
    st.file = await ntInvoke<FileText>("neobot_fs_read", { rel: st.rel });
    toast(`已保存 ${st.rel}`, "ok");
    paintEditor(root);
  } catch (err) {
    toast(String(err), "err");
  }
}

// ---- 变动页（两个视角：本轮文件 / Git） ----

async function paintChanges(root: HTMLElement): Promise<void> {
  const body = root.querySelector<HTMLElement>("#sb-body");
  if (!body) return;
  body.innerHTML = '<div class="sb-empty">载入中…</div>';
  if (currentChange) {
    paintChangeDiff(root);
    return;
  }
  try {
    const recent = await ntInvoke<RecentPaths>("neobot_changes_recent_paths", { taskLimit: 1 });
    const tallies = recent.paths;
    const files = gitOk ? await ntInvoke<GitFile[]>("neobot_git_status", { rel: "" }).catch(() => [] as GitFile[]) : [];
    gitFiles = files;
    // 标题写清在看**哪一轮** —— 不然用户不知道「本轮」是哪轮。
    const roundLabel = recent.task_title
      ? `本轮文件 · ${recent.task_title.slice(0, 28)}`
      : "本轮文件";
    body.innerHTML = `
      <div class="sb-sec">${esc(roundLabel)}</div>
      ${tallies.length === 0 ? '<div class="sb-empty">最近这一轮模型还没碰过文件</div>' : tallies.map(tallyHtml).join("")}
      ${
        gitOk
          ? `<div class="sb-sec">Git 视角<button class="sb-mini primary" data-act="git-commit-open">提交</button></div>${
              files.length === 0 ? '<div class="sb-empty">工作区干净</div>' : files.map(gitRowHtml).join("")
            }`
          : '<div class="sb-note">未检测到 git，不显示 Git 视角</div>'
      }
    `;
  } catch (err) {
    body.innerHTML = `<div class="sb-empty">读变动账失败：${esc(String(err))}</div>`;
  }
}

async function openChange(root: HTMLElement, changeId: string): Promise<void> {
  if (!changeId) return;
  try {
    const view = await ntInvoke<ChangeView | null>("neobot_changes_get", { changeId });
    if (!view) {
      toast("这条改动已不在账里（可能已被清理）", "err");
      return;
    }
    currentChange = view;
    paintChangeDiff(root);
  } catch (err) {
    toast(String(err), "err");
  }
}

function paintChangeDiff(root: HTMLElement): void {
  const body = root.querySelector<HTMLElement>("#sb-body");
  if (!body || !currentChange) return;
  const ch = currentChange.change;
  body.innerHTML = `
    <div class="sb-editorhead">
      <button class="sb-mini" data-act="search-clear" title="返回列表">${icon("chevron", 14)}</button>
      <span class="sb-path mono">${esc(ch.path)}</span>
      <div class="grow"></div>
      <span class="sb-note sm">${esc(ch.kind)} · ${esc(fmtBytes(ch.bytes))}</span>
    </div>
    <div class="sb-diff" id="sb-diff">${changeDiffHtml()}</div>
  `;
}

function changeDiffHtml(): string {
  const view = currentChange;
  if (!view) return "";
  if (view.change.content_omitted) {
    // 界律 2：内容超限没存就说清楚，不给一段编出来的 diff。
    return `<div class="sb-empty">这一处的内容超过 ${fmtBytes(256 * 1024)} 上限，账里只存了元信息。<br/>要逐行对比请打开文件（工具写入时的原文不在库里）。</div>`;
  }
  const before = view.before ?? "";
  const after = view.after ?? "";
  if (before === "" && after === "") {
    return '<div class="sb-empty">只读了文件，没有内容变化（读不产生 diff）</div>';
  }
  const result = diffLines(before, after);
  return diffHtml(result, `${view.change.path}（本轮改动）`);
}

async function openGitDiff(root: HTMLElement, path: string, staged: boolean): Promise<void> {
  try {
    const got = await ntInvoke<GitDiff>("neobot_git_diff", { rel: "", path, staged });
    // Git diff 是**已渲染的 unified diff 文本**，不是前后两份内容。
    // 故不喂给 diffLines（那会把它当普通文本行），直接按 unified 语法着色。
    const body = root.querySelector<HTMLElement>("#sb-body");
    if (!body) return;
    body.innerHTML = `
      <div class="sb-editorhead">
        <button class="sb-mini" data-act="search-clear" title="返回列表">${icon("chevron", 14)}</button>
        <span class="sb-path mono">${esc(path)}</span>
        <div class="grow"></div>
        <span class="sb-note sm">${staged ? "已暂存" : "工作区"}${got.untracked_new ? " · 新增文件（下方为全文）" : ""}</span>
      </div>
      <div class="sb-diff">${unifiedHtml(got.text)}</div>
    `;
  } catch (err) {
    toast(String(err), "err");
  }
}

async function gitMutate(root: HTMLElement, what: string, path: string): Promise<void> {
  // 类型标注是**纯编译期**的，运行时不改：它把 `cmd` 钉成字面量联合，
  // 好让 `ntInvoke` 的 `K` 推出这三条命令各自的参数形状（三者同形 `{rel, path}`）。
  // 标注里若写错命令名，下面三个分支赋值会直接编译不过 —— 这行自带校验。
  const cmd: "neobot_git_stage" | "neobot_git_unstage" | "neobot_git_revert" =
    what === "stage" ? "neobot_git_stage" : what === "unstage" ? "neobot_git_unstage" : "neobot_git_revert";
  try {
    await ntInvoke(cmd, { rel: "", path });
    toast(`${what === "revert" ? "已还原" : what === "stage" ? "已暂存" : "已取消暂存"} ${path}`, "ok");
    await paintChanges(root);
  } catch (err) {
    toast(String(err), "err");
  }
}

async function gitCommit(root: HTMLElement): Promise<void> {
  const body = root.querySelector<HTMLElement>("#sb-body");
  if (!body) return;
  body.insertAdjacentHTML(
    "afterbegin",
    `<div class="sb-commit">
      <input id="sb-commit-msg" placeholder="提交信息（必填）" maxlength="400" />
      <label class="sb-verify" title="默认不跳过仓库 hook：点提交会执行 .git/hooks 里的代码，这是 git 的正常语义">
        <input type="checkbox" id="sb-commit-nov" /> 跳过 hook
      </label>
      <button class="sb-mini primary" data-act="git-commit">提交</button>
    </div>`,
  );
  body.querySelector<HTMLInputElement>("#sb-commit-msg")?.focus();
}

async function doCommit(root: HTMLElement): Promise<void> {
  const msg = root.querySelector<HTMLInputElement>("#sb-commit-msg")?.value ?? "";
  const noVerify = root.querySelector<HTMLInputElement>("#sb-commit-nov")?.checked ?? false;
  try {
    const got = await ntInvoke<{ short: string; subject: string }>("neobot_git_commit", {
      rel: "",
      message: msg,
      noVerify: !noVerify,
    });
    toast(`已提交 ${got.short} ${got.subject}`, "ok");
    await paintChanges(root);
  } catch (err) {
    toast(String(err), "err");
  }
}

// ---- 任务页 ----

async function paintTasks(root: HTMLElement): Promise<void> {
  const body = root.querySelector<HTMLElement>("#sb-body");
  if (!body) return;
  try {
    const [tasks, changes] = await Promise.all([
      ntInvoke<Array<{ id: string; title: string; status: string; conversation_id: string | null; attempts: number }>>(
        "neobot_tasks",
      ),
      ntInvoke<ChangeItem[]>("neobot_changes_list", { taskId: null, limit: 60 }),
    ]);
    const byTask = new Map<string, ChangeItem[]>();
    for (const ch of changes) {
      const list = byTask.get(ch.task_id) ?? [];
      list.push(ch);
      byTask.set(ch.task_id, list);
    }
    body.innerHTML = tasks
      .map((task) => {
        const items = byTask.get(task.id) ?? [];
        return `<details class="sb-task"${task.status === "running" ? " open" : ""}>
          <summary>
            <span class="sb-dot ${esc(task.status)}"></span>
            <span class="sb-name">${esc(task.title)}</span>
            <span class="sb-note sm">${esc(task.status)}${task.attempts > 1 ? ` · 第 ${task.attempts} 次` : ""}</span>
          </summary>
          ${
            items.length === 0
              ? '<div class="sb-sub">这一轮没碰文件</div>'
              : `<div class="sb-sub">${esc(
                  items.map((ch) => `${ch.kind === "read" ? "读" : ch.kind === "write" ? "写" : "改"} ${ch.path}`).join("、"),
                )}</div>`
          }
        </details>`;
      })
      .join("");
  } catch (err) {
    body.innerHTML = `<div class="sb-empty">读任务失败：${esc(String(err))}</div>`;
  }
}

// ---- 侧聊页 ----

async function paintChat(root: HTMLElement): Promise<void> {
  const body = root.querySelector<HTMLElement>("#sb-body");
  if (!body) return;
  if (!parentConvoId) {
    body.innerHTML = '<div class="sb-empty">先在左边选一个会话，侧聊会继承它的上下文</div>';
    return;
  }
  try {
    const threads = await ntInvoke<SideThread[]>("neobot_sidechat_list", { parentId: parentConvoId });
    if (!currentThread || !threads.some((t) => t.id === currentThread)) {
      currentThread = threads[0]?.id ?? "";
    }
    const active = threads.find((t) => t.id === currentThread) ?? null;
    body.innerHTML = `
      <div class="sb-sec">
        侧聊（继承本会话上下文、独立运行、不污染本会话）
        <button class="sb-mini primary" data-act="thread-new">新开</button>
      </div>
      <div class="sb-threads">
        ${
          threads.length === 0
            ? '<div class="sb-empty">还没有侧聊</div>'
            : threads
                .map(
                  (t) =>
                    `<div class="sb-row${t.id === currentThread ? " sel" : ""}" data-thread="${esc(t.id)}">
                      <span class="sb-name">${esc(t.title)}</span>
                      <span class="sb-note sm">${t.task_count} 轮</span>
                    </div>`,
                )
                .join("")
        }
      </div>
      ${active ? threadBodyHtml(active) : ""}
    `;
  } catch (err) {
    body.innerHTML = `<div class="sb-empty">读侧聊失败：${esc(String(err))}</div>`;
  }
}

function threadBodyHtml(thread: SideThread): string {
  return `<div class="sb-threadbody">
    <div class="sb-editorhead">
      <span class="sb-path">${esc(thread.title)}</span>
      <div class="grow"></div>
      <button class="sb-mini" data-act="thread-promote" title="升为顶层会话">升格</button>
    </div>
    <div class="sb-threadmsgs" id="sb-threadmsgs"><div class="sb-sub">上下文继承自母会话最近若干轮的摘要（不是逐字历史）。</div></div>
    <div class="sb-threadinput">
      <textarea id="sb-thread-input" rows="2" placeholder="就当前话题追问…"></textarea>
      <button class="sb-mini primary" data-act="thread-send" id="sb-thread-send" disabled>发送</button>
    </div>
  </div>`;
}

async function newThread(root: HTMLElement): Promise<void> {
  try {
    currentThread = await ntInvoke<string>("neobot_sidechat_open", { parentId: parentConvoId, title: "" });
    toast("已开侧聊", "ok");
    await paintChat(root);
  } catch (err) {
    toast(String(err), "err");
  }
}

async function promoteThread(root: HTMLElement): Promise<void> {
  if (!currentThread) return;
  try {
    await ntInvoke("neobot_sidechat_promote", { threadId: currentThread });
    toast("已升为顶层会话（去左边对话栏找它）", "ok");
    await paintChat(root);
  } catch (err) {
    toast(String(err), "err");
  }
}

function paintThreadSend(root: HTMLElement): void {
  const input = root.querySelector<HTMLTextAreaElement>("#sb-thread-input");
  const btn = root.querySelector<HTMLButtonElement>("#sb-thread-send");
  if (input && btn) btn.disabled = input.value.trim() === "";
}

async function sendThread(root: HTMLElement): Promise<void> {
  const input = root.querySelector<HTMLTextAreaElement>("#sb-thread-input");
  const text = input ? input.value.trim() : "";
  if (!text || !input || !currentThread || !opts) return;
  const msgs = root.querySelector<HTMLElement>("#sb-threadmsgs");
  if (msgs) {
    msgs.insertAdjacentHTML("beforeend", `<div class="sb-msg me">${esc(text)}</div>`);
  }
  input.value = "";
  paintThreadSend(root);
  // 继承摘要：母会话的最近几轮以摘要形式带上去（Rust 侧生成，抬头写明是摘要）。
  try {
    const ctx = await ntInvoke<string | null>("neobot_sidechat_context", { parentId: parentConvoId });
    const prompt = ctx ? `${ctx}\n\n用户：${text}` : text;
    if (msgs) msgs.insertAdjacentHTML("beforeend", '<div class="sb-sub">思考中…</div>');
    await opts.onRunTurn(prompt, currentThread);
    // 跑完了，把答案**画回这个侧聊的气泡里**。
    //
    // 早先的写法只插一句「已提交本轮（结果在主对话区）」—— 那等于侧聊只能发、
    // 不能读，回复还得去主对话区找，特性是半截的。这里从库里把本轮末次回复
    // 取回来直接显示（转录本体在 Rust 侧 `steps` 表，前端 localStorage 只有主对话）。
    const reply = await ntInvoke<string | null>("neobot_convo_last_reply", { convoId: currentThread });
    const thinking = msgs?.querySelector(".sb-sub");
    thinking?.remove();
    if (!msgs) return;
    if (reply && reply.trim()) {
      msgs.insertAdjacentHTML("beforeend", `<div class="sb-msg bot">${esc(reply)}</div>`);
    } else {
      // 取不到就**说取不到**，不留一句含糊的「已提交」。
      msgs.insertAdjacentHTML(
        "beforeend",
        '<div class="sb-sub warn">这一轮跑完了，但没取到正文（可能被策略网关挡住，或回复为空）</div>',
      );
    }
    msgs.scrollTop = msgs.scrollHeight;
  } catch (err) {
    msgs?.querySelector(".sb-sub")?.remove();
    toast(String(err), "err");
  }
}

// ---- 共享小件 ----

/** 人类可读字节数。 */
function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  return `${(n / 1024 / 1024).toFixed(1)} MiB`;
}

function searchShellHtml(): string {
  return `
    <div class="sb-searchrow">
      <input id="sb-search" value="${esc(searchQuery)}" placeholder="搜索文件名…" />
      <button class="sb-mini" data-act="search-clear">清除</button>
    </div>
    <div class="sb-list" id="sb-results"></div>
  `;
}

/** 模型 `sidebar_open` 的落点：解析后由 main.ts 调这里跳页。 */
export async function openFromModel(topic: string, path: string): Promise<void> {
  const body = document.getElementById("sb-body");
  const root = body?.closest<HTMLElement>("#sb-panel");
  if (!root) return;
  try {
    const target = await ntInvoke<{ topic: string; path: string; viewer: string | null }>(
      "neobot_sidebar_resolve",
      { topic, target: path },
    );
    activeTab = target.topic;
    const tabsEl = root.querySelector<HTMLElement>("#sb-tabs");
    if (tabsEl) tabsEl.innerHTML = tabsHtml();
    if (target.topic === "files" && target.path) {
      await revealPath(root, target.path);
      await openFile(root, target.path);
      return;
    }
    await refreshActive(root);
  } catch (err) {
    // 模型的提议被拒要**让用户看见**，不能静默吞掉。
    toast(`模型想打开 ${topic}，但没成功：${String(err)}`, "err");
  }
}
