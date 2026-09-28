/**
 * NeoBot 画板纯渲染（ storage helpers + html builders + 世界视口）。
 * 只依赖 core；事件装配（wireBoardDrag）与行级动作留在 main.ts。
 */

import {
  K, STATUS_CN, cache, hero, esc, load, save, currentView, shell,
  type PageNode, type BoardView, type VNode,
} from "./core";
import { icon } from "./icons";

export function card(title: string, inner: string): string {
  return `<div class="card"><h4>${esc(title)}</h4>${inner}</div>`;
}

export function btnRow(btns: Array<[string, string, boolean?]>): string {
  return `<div style="display:flex;gap:6px;flex-wrap:wrap;margin-top:8px">${btns.map(([id, label, dis]) => `<button class="chip-btn" data-act="${id}" ${dis ? "disabled" : ""}>${esc(label)}</button>`).join("")}</div>`;
}

export function inspectorHtml(): string {
  const t = cache.tasks.find((t) => t.id === load<string>(K.selTask, ""));
  if (t) {
    return `<div class="audit-line"><span class="trace-ic">${icon("search", 12)}</span><b>${esc(t.title)}</b> · ${esc(STATUS_CN[t.status] ?? t.status)}${t.claimed_by ? ` · ${esc(t.claimed_by)}` : ""} <span class="muted">（操作去对话流任务卡）</span></div>`;
  }
  const r = cache.routines.find((r) => r.name === load<string>(K.selRoutine, ""));
  if (r) return `<div class="audit-line">🔍 <b>${esc(r.name)}</b> · ${r.disabled ? "已自停" : `败${r.failures}`}</div>`;
  const s = cache.skills.find((s) => s.name === load<string>(K.selSkill, ""));
  if (s) return `<div class="audit-line">🔍 <b>${esc(s.name)}</b></div>`;
  return "";
}

export function summaryHtml(): string {
  const by = (st: string) => cache.tasks.filter((t) => t.status === st).length;
  const totIn = cache.costs.reduce((a, r) => a + r.in_tokens, 0);
  const totOut = cache.costs.reduce((a, r) => a + r.out_tokens, 0);
  const totC = cache.costs.reduce((a, r) => a + r.cost_usd, 0);
  const denies = cache.audits.filter((a) => a.decision === "deny");
  const sick = cache.routines.filter((r) => r.disabled || r.failures > 0);
  let html = "";
  html += `<div class="card sumcard" data-sumfilter="task"><h4>任务 · ${cache.tasks.length}</h4><div class="sumdots"><span><i class="status-dot st-done"></i>${by("done")}成</span><span><i class="status-dot st-running"></i>${by("running") + by("pending")}活</span><span><i class="status-dot st-failed"></i>${by("failed")}败</span><span><i class="status-dot st-cancelled"></i>${by("cancelled")}取</span></div><div class="muted">点卡看节点 ↓</div></div>`;
  html += `<div class="card sumcard" data-sumfilter="ledger"><h4>成本 · $${totC.toFixed(4)}</h4><div class="kv"><span class="k">in/out</span><span class="mono">${totIn}/${totOut}</span></div><div class="kv"><span class="k">拒绝</span><span class="mono">${denies.length}</span></div><div class="muted">点卡看节点 ↓</div></div>`;
  html += `<div class="card sumcard"><h4>例行 · ${cache.routines.filter((r) => !r.disabled).length}/${cache.routines.length}启用</h4>`;
  if (sick.length === 0) html += `<div class="empty-note">全部健康</div>`;
  for (const r of sick.slice(0, 5)) {
    html += `<div class="audit-line"><b>${esc(r.name)}</b> ${r.disabled ? "已自停" : `败${r.failures}`} <button class="chip-btn" data-sact="fire" data-sname="${esc(r.name)}" ${r.disabled ? "disabled" : ""}>跑一次</button></div>`;
  }
  html += `<div class="btnrow"><button class="chip-btn" data-sact="sweep">扫到期</button><button class="chip-btn" data-sumfilter="routine">全部例行 ↓</button></div></div>`;
  html += `<div class="card sumcard" data-sumfilter="skill"><h4>技能 · ${cache.skills.length}</h4><div class="muted">${cache.skills.slice(0, 6).map((s) => esc(s.name)).join("、") || "暂无"}</div><div class="muted">点卡看节点 ↓</div></div>`;
  return html;
}

export function boardNodeCount(f: string): number {
  if (f === "task") return cache.tasks.length;
  if (f === "routine") return cache.routines.length;
  if (f === "skill") return cache.skills.length;
  if (f === "ledger") return cache.costs.length + cache.audits.filter((a) => a.decision === "deny").length;
  return cache.tasks.length + cache.routines.length + cache.skills.length + cache.costs.length;
}

export function boardPages(): PageNode[] { return load<PageNode[]>(K.pages, []); }

export function savePages(p: PageNode[]): void { save(K.pages, p); }

export function boardView(): BoardView { return load<BoardView>(K.boardView, { x: 0, y: 0, k: 1 }); }

export function saveView(v: BoardView): void { save(K.boardView, v); }

export function nodePos(): Record<string, { x: number; y: number }> { return load(K.boardPos, {}); }

export function collectNodes(f: string): VNode[] {
  const out: VNode[] = [];
  const show = (k: string) => f === "all" || f === k;
  const lanes: Record<string, number> = { task: 0, routine: 270, skill: 540, ledger: 810 };
  const cursor: Record<string, number> = { task: 0, routine: 0, skill: 0, ledger: 0 };
  const push = (kind: string, key: string, color: string, title: string, sub: string, sel: boolean) => {
    const over = nodePos()[`${kind}:${key}`];
    const x = over?.x ?? lanes[kind] ?? 0;
    const y = over?.y ?? cursor[kind] ?? 0;
    if (over === undefined) cursor[kind] = (cursor[kind] ?? 0) + 104;
    out.push({ key: `${kind}:${key}`, kind, color, title, sub, sel, x, y });
  };
  if (show("task")) {
    const sel = load<string>(K.selTask, "");
    for (const t of cache.tasks.slice(0, 30)) {
      const color = t.status === "done" ? "var(--avail)" : t.status === "failed" || t.status === "cancelled" ? "var(--coral)" : t.status === "running" ? "var(--working)" : "var(--skype)";
      push("task", t.id, color, t.title, `${esc(STATUS_CN[t.status] ?? t.status)}${t.claimed_by ? ` · ${esc(t.claimed_by)}` : ""}`, sel === t.id);
    }
  }
  if (show("routine")) {
    const sel = load<string>(K.selRoutine, "");
    for (const r of cache.routines) {
      push("routine", r.name, r.disabled ? "var(--resting)" : "var(--working)", r.name, r.disabled ? "已自停" : `每${Math.round(r.interval_secs / 60)}min · 败${r.failures}`, sel === r.name);
    }
  }
  if (show("skill")) {
    const sel = load<string>(K.selSkill, "");
    for (const s of cache.skills) {
      push("skill", s.name, "var(--whisper)", s.name, esc(s.description), sel === s.name);
    }
  }
  if (show("ledger")) {
    if (cache.costs.length > 0) {
      const totC = cache.costs.reduce((a, r) => a + r.cost_usd, 0);
      const actors = cache.costs.map((r) => r.actor).filter((v, i, a) => a.indexOf(v) === i).join("、");
      push("cost", "-", "var(--skype)", `成本 $${totC.toFixed(4)}`, `${cache.costs.length} 条 · ${esc(actors)}`, false);
    }
    cache.audits.filter((a) => a.decision === "deny").slice(0, 5).forEach((a, i) => {
      push("deny", String(i), "var(--coral)", `${a.tool} 被拒`, `${esc(a.rule ?? "")} · ${esc(a.actor)}`, false);
    });
  }
  return out;
}

export function renderBoardWorld(f: string): string {
  const nodes = collectNodes(f);
  const pos = nodePos();
  const nodeHtmlAbs = (n: VNode) => {
    const p = pos[n.key];
    const x = p?.x ?? n.x, y = p?.y ?? n.y;
    return `<div class="node${n.sel ? " sel" : ""}" data-node="${esc(n.key)}" data-nid="${esc(n.key)}" style="left:${x}px;top:${y}px;width:240px;--nd:${n.color}"><div class="nm">${esc(n.title)}</div><div class="pv">${n.sub}</div></div>`;
  };
  let html = nodes.map(nodeHtmlAbs).join("");
  const showPages = f === "all" || f === "ledger";
  if (showPages) {
    for (const p of boardPages()) {
      const host = (() => { try { return new URL(p.url).host; } catch { return p.url; } })();
      html += `<div class="pagenode" data-nid="page:${esc(p.id)}" style="left:${p.x}px;top:${p.y}px;width:${p.w}px"><div class="pagehead" data-drag="page:${esc(p.id)}"><span class="pv">${esc(host)}</span><span class="pageacts"><button data-pact="reload" title="刷新">↻</button><button data-pact="open" title="外部打开">↗</button><button data-pact="close" title="移除">×</button></span></div><iframe src="${esc(p.url)}" style="width:100%;height:${p.h}px;border:none;border-radius:0 0 11px 11px;background:#fff" sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-modals" loading="lazy"></iframe></div>`;
    }
  }
  if (nodes.length === 0 && (!showPages || boardPages().length === 0)) {
    html += `<div class="node" style="left:0;top:0;--nd:var(--ink-200)"><div class="pv">暂无节点 — 右上 + 加网页</div></div>`;
  }
  return html;
}

/** stale running 防御性复位（展示层；调度语义不动，后端见 `nt_stale_guard`）。
 *  同一 running 任务在无本地流（`shell.running`）的情况下持续超过上限，
 *  即判展示 stale：hero 降级为“运行超时” + 取消键，不再常亮“运行中”。
 *  上限与后端租约 `nt_agent::LEASE_SECS` / `STALE_RUNNING_SECS` 同值（600s）。 */
export const STALE_RUNNING_MS = 600_000;
const runningSeen = new Map<string, number>();
const staleRunning = new Set<string>();
/** 当前展示 stale 的 running 任务 id（badge/忙点据此剔除，不改库）。 */
export function staleRunningIds(): string[] {
  return [...staleRunning];
}
export function renderInfoHero(): void {
  // 剪枝：已不在 running 的 id 清掉（任务结束/取消即忘，不泄漏）。
  const runningIds = new Set(cache.tasks.filter((t) => t.status === "running").map((t) => t.id));
  for (const id of [...runningSeen.keys()]) {
    if (!runningIds.has(id)) { runningSeen.delete(id); staleRunning.delete(id); }
  }
  // hero 卡：最新 running 任务独占卡槽
  const hot = cache.tasks.find((t) => t.status === "running");
  if (currentView() === "convos" && hot) {
    const now = Date.now();
    if (!runningSeen.has(hot.id)) runningSeen.set(hot.id, now);
    // 本地流进行中永不判 stale（本轮正在跑是合法态）。
    const stale = !shell.running && now - (runningSeen.get(hot.id) ?? now) > STALE_RUNNING_MS;
    if (stale) staleRunning.add(hot.id);
    else staleRunning.delete(hot.id);
    hero.classList.remove("hidden");
    if (stale) {
      hero.innerHTML = `<b>运行超时 · ${esc(hot.title)}</b><span class="muted">（疑似卡死，已超10分钟）</span><button class="chip-btn" data-hero="cancel" data-tid="${esc(hot.id)}" title="取消该卡死任务">取消</button>`;
    } else {
      hero.innerHTML = `<b>运行中 · ${esc(hot.title)}</b>${hot.claimed_by ? ` · ${esc(hot.claimed_by)}认领中` : ""}`;
    }
  } else hero.classList.add("hidden");
}
