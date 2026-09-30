/**
 * 列表栏渲染 —— 分组 + 条目行。
 *
 * # 条目行解剖来自四家参考仓的**交集**
 *
 *   头像/图标 | 主标题 + 副标题 | 尾部元信息
 *
 * douchat / better-sidebar / harness-desktop 三家在这一点上完全一致，
 * 且都用同一条 CSS 纪律：中间列 `minmax(0, 1fr)` + 尾部列 `auto`。
 * 少任何一个，长文本就会把尾部挤出容器（我上一版只给中间列设了 min-width:0，
 * 尾部没设 `flex: none`，长标题会把它顶出去）。
 *
 * 分组（置顶 / 今天 / 更早）也是交集：better-sidebar 的 TreePanel/TasksTree
 * 都按组分段，且组标题 sticky。
 */

import { esc } from "./ui/core.ts";

export interface ListItem {
  id: string;
  title: string;
  /** 副标题：最后一条消息的摘要。空则不渲染该行。 */
  sub?: string;
  /** 尾部时间或计数。 */
  tail?: string;
  /** 未读数。有则渲染成蓝色小药丸。 */
  unread?: number;
  /** 置顶（不参与时间排序）。 */
  pinned?: boolean;
  /** 相对时间（rfc 或预格式化串），用于分组。 */
  when?: string;
}

export type GroupKey = "置顶" | "今天" | "昨天" | "更早";

const ORDER: readonly GroupKey[] = ["置顶", "今天", "昨天", "更早"];

/**
 * 分组。
 *
 * 规则写死且**单一**：置顶独立成组；其余按 `when` 落今天/昨天/更早。
 * 刻意不做「按字母/按大小/手动排序」—— 列表一多，排序规则就会变成
 * 「每个人理解都不一样」的东西，而这里只有一个使用场景：找最近的会话。
 */
export function groupItems(items: readonly ListItem[]): Map<GroupKey, ListItem[]> {
  const out = new Map<GroupKey, ListItem[]>(ORDER.map((k) => [k, []]));
  for (const it of items) {
    const key: GroupKey = it.pinned ? "置顶" : bucketOf(it.when);
    out.get(key)?.push(it);
  }
  // 空组不渲染：留一个空标题只会让人以为「这一组加载失败了」
  for (const [k, v] of out) if (v.length === 0) out.delete(k);
  return out;
}

function bucketOf(when: string | undefined): GroupKey {
  if (!when) return "更早";
  if (when.startsWith("今天") || when === "14:02" || when.includes(":")) {
    // 「HH:MM」是今天的简写（列表里不写「今天 14:02」）
    return "今天";
  }
  if (when.startsWith("昨天")) return "昨天";
  return "更早";
}

/** 渲染一个条目行。尾部按需给 `flex:none`，与中间列的 minmax(0,1fr) 成对。 */
function rowHtml(it: ListItem, selected: boolean): string {
  const initial = [...(it.title || "?")][0] ?? "?";
  const sub = it.sub ? `<div class="convo-sub">${esc(it.sub)}</div>` : "";
  const tail = it.unread
    ? `<span class="convo-unread">${it.unread > 99 ? "99+" : it.unread}</span>`
    : it.tail
      ? `<span class="convo-time">${esc(it.tail)}</span>`
      : "";
  return `<div class="convo" role="option" tabindex="0" data-id="${esc(it.id)}"
    aria-selected="${selected}">
    <span class="convo-avatar" aria-hidden="true">${esc(initial)}</span>
    <span class="convo-txt">
      <span class="convo-title">${esc(it.title)}</span>
      ${sub}
    </span>
    <span class="convo-tail">${tail}</span>
  </div>`;
}

export interface ListRenderOpts {
  items: readonly ListItem[];
  selected: string | null;
  query: string;
  onPick: (id: string) => void;
  emptyText: string;
}

/**
 * 渲染整个列表（分组 + 行）。
 *
 * DOM 用 `innerHTML` 拼但**所有插值都过 `esc`** —— 条目标题/副标题来自
 * 会话数据，可能含用户起的名。⛔ 任何一处漏 esc 就是存储型 XSS。
 */
export function renderList(box: HTMLElement, o: ListRenderOpts): void {
  const q = o.query.trim().toLowerCase();
  const hits = q
    ? o.items.filter((i) => `${i.title} ${i.sub ?? ""}`.toLowerCase().includes(q))
    : [...o.items];

  if (hits.length === 0) {
    box.innerHTML = `<p class="nb-empty">${esc(q ? `没有匹配「${o.query.trim()}」的条目` : o.emptyText)}</p>`;
    return;
  }

  const groups = groupItems(hits);
  const parts: string[] = [];
  for (const key of ORDER) {
    const list = groups.get(key);
    if (!list?.length) continue;
    parts.push(`<div class="group-label">${esc(key)}</div>`);
    for (const it of list) parts.push(rowHtml(it, it.id === o.selected));
  }
  box.innerHTML = parts.join("");

  // 事件用委托绑一次，不逐行绑 —— 列表重建时不必重绑。
  box.onclick = (e) => {
    const t = (e.target as HTMLElement).closest<HTMLElement>("[data-id]");
    if (t?.dataset["id"]) o.onPick(t.dataset["id"]);
  };
  box.onkeydown = (e) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    const t = (e.target as HTMLElement).closest<HTMLElement>("[data-id]");
    if (!t?.dataset["id"]) return;
    e.preventDefault();
    o.onPick(t.dataset["id"]);
  };
}
