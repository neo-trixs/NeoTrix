/**
 * API 面板 —— **后端的可视化交互**。
 *
 * # 这块面板存在的理由
 *
 * 契约（`src/api.rs` SPECS）是后端的声明；面板是它的**可视化**。
 * 有了它，「后端到底有什么、哪些能用、哪些决定不做」不再需要读 Rust 源码，
 * 而是打开界面就能看到、能点、能试。
 *
 * 这直接对上用户那句「前端是后端的可视化交互」——
 * 不是「前端调后端」，是**后端的形状本身长在界面上**。
 *
 * # 三态必须用不同颜色，不能只用文字
 *
 * 「本仓不做」（Stub）与「还没做」（Planned）**行为上都是不可用**，
 * 但含义完全相反：一个是决定，一个是欠账。混在一起 ⇒ 路线图失真。
 * 只用文字标签时，扫视一圈下来分不出「这 44 个是不需要」还是「这 44 个没排期」。
 * ⇒ Stub 灰（这是结论）、Planned 琥珀（这是欠账）、Implemented 绿（这是能力）。
 */

import { invokeCmd as invoke } from "./ipc";
import { t } from "./i18n";
import { el, span } from "./dom.ts";

type Status = "implemented" | "stub" | "planned";

interface ApiSpec {
  name: string;
  category: string;
  params: string[];
  ret: string;
  status: Status;
  note: string;
}

interface ApiSummary {
  total: number;
  implemented: number;
  stub: number;
  planned: number;
  upstream_unlisted: number;
  upstream_total: number;
}

interface ApiCatalog {
  summary: ApiSummary;
  specs: ApiSpec[];
  upstream_unlisted: string[];
}

/**
 * 状态标签。⛔ 刻意做成**函数**而非模块级常量：
 * 若写成 `Record<Status, string> = { implemented: t('api.status.implemented'), ... }`，
 * 它在**模块加载时求值一次** ⇒ 之后切语言**永不更新**。
 * —— 与我此前在 neobot-root 犯的错同类（模块级 `t` 不订阅语言变化）。
 */
const statusText = (st: Status): string =>
  st === "implemented" ? t("api.statusImplemented")
  : st === "planned" ? t("api.statusPlanned")
  : t("api.statusStub");

/** 拉契约。失败要**说清原因**，不能静默空面板。 */
export async function loadApiPanel(host: HTMLElement): Promise<void> {
  host.replaceChildren(el("div", "api-loading", t("api.loading")));
  let cat: ApiCatalog;
  try {
    cat = await invoke<ApiCatalog>("neobot_api_specs");
  } catch (e) {
    // ⛔ 契约都读不到 ⇒ 这个面板没有存在意义了，必须显式说出来。
    //    静默留空白会被读成「后端没有接口」。
    const box = el("div", "api-error");
    box.appendChild(el("div", "api-error-title", t("api.loadFailed")));
    box.appendChild(el("div", "api-error-msg", String(e).slice(0, 200)));
    host.replaceChildren(box);
    return;
  }
  renderApiPanel(host, cat);
}

function renderApiPanel(host: HTMLElement, cat: ApiCatalog): void {
  host.replaceChildren();

  // ── 顶部：数字说话 ──
  const s = cat.summary;
  const sum = el("div", "api-summary");
  const stat = (n: number, label: string, cls: string) => {
    const d = el("div", `api-stat api-stat--${cls}`);
    d.appendChild(el("div", "api-stat-n", String(n)));
    d.appendChild(el("div", "api-stat-l", label));
    return d;
  };
  sum.appendChild(stat(s.implemented, t("api.statusImplemented"), "ok"));
  sum.appendChild(stat(s.planned, t("api.statusPlanned"), "todo"));
  sum.appendChild(stat(s.stub, t("api.statusStub"), "skip"));
  sum.appendChild(stat(s.upstream_unlisted, t("api.upstreamUnlisted"), "dim"));
  host.appendChild(sum);

  // ── 一句话结论：把「为什么有这么多不做」讲在前面 ──
  const lead = el("div", "api-lead");
  lead.appendChild(
    el(
      "div",
      "api-lead-text",
      t("api.lead", { total: s.upstream_total }),
    ),
  );
  host.appendChild(lead);

  // ── 按分类分组 ──
  const byCat = new Map<string, ApiSpec[]>();
  for (const sp of cat.specs) {
    if (!byCat.has(sp.category)) byCat.set(sp.category, []);
    byCat.get(sp.category)!.push(sp);
  }
  for (const [catName, items] of byCat) {
    const g = el("div", "api-group");
    const head = el("div", "api-group-head");
    head.appendChild(el("span", "api-group-name", catName));
    const counts = el("span", "api-group-counts");
    for (const st of ["implemented", "planned", "stub"] as Status[]) {
      const n = items.filter((x) => x.status === st).length;
      if (n > 0) {
        const c = span(`api-pill api-pill--${st}`, `${statusText(st)} ${n}`);
        counts.appendChild(c);
      }
    }
    head.appendChild(counts);
    g.appendChild(head);

    for (const sp of items) {
      g.appendChild(apiRow(sp));
    }
    host.appendChild(g);
  }

  // ── 上游未展开的清单：折叠，但**必须可展开** ──
  if (cat.upstream_unlisted.length > 0) {
    const d = el("details", "api-unlisted");
    d.appendChild(el("summary", "api-unlisted-sum", t("api.unlistedSummary", { n: cat.upstream_unlisted.length })));
    const ul = el("div", "api-unlisted-list");
    for (const n of cat.upstream_unlisted) ul.appendChild(span("api-unlisted-item", n));
    d.appendChild(ul);
    host.appendChild(d);
  }
}

function apiRow(sp: ApiSpec): HTMLElement {
  const row = el("div", "api-row");
  row.appendChild(span(`api-dot api-dot--${sp.status}`, ""));

  const body = el("div", "api-row-body");
  const top = el("div", "api-row-top");
  top.appendChild(el("code", "api-name", sp.name));
  top.appendChild(span(`api-status api-status--${sp.status}`, statusText(sp.status)));
  body.appendChild(top);

  const sig = el("div", "api-sig");
  sig.appendChild(span("api-sig-part", `(${sp.params.join(", ") || "—"})`));
  sig.appendChild(span("api-sig-arrow", "→"));
  sig.appendChild(span("api-sig-part", sp.ret));
  body.appendChild(sig);

  // 理由是**必读**的：没有理由的 Stub 会被后人「顺手补上」。
  if (sp.note) body.appendChild(el("div", "api-note", sp.note));

  row.appendChild(body);

  // 点一下试调。⛔ 未实现的不发请求 —— 直接显示理由，
  // 免得每次点都打一次注定失败的网络往返。
  if (sp.status !== "implemented") {
    row.addEventListener("click", () => {
      row.classList.toggle("api-row--open");
    });
    row.setAttribute("role", "button");
    row.setAttribute("tabindex", "0");
  } else {
    row.addEventListener("click", () => {
      void invoke("neobot_api_call", { name: sp.name, args: null })
        .then((r) => {
          const d = el("div", "api-call-out", JSON.stringify(r).slice(0, 200));
          row.appendChild(d);
        })
        .catch((e) => row.appendChild(el("div", "api-call-out api-call-out--err", String(e).slice(0, 200))));
    });
    row.setAttribute("role", "button");
    row.setAttribute("tabindex", "0");
  }
  return row;
}
