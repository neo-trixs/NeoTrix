/**
 * NeoBot 前端入口 —— 薄壳 + 能力门控。
 *
 * 分层（依赖单向，见 neobot/README.md）：
 *   tokens.css → shell/base/primitives/sheet.css → 本文件 → plugin/host
 *
 * 三条硬规则，都是被事故教出来的：
 *  ① 本文件**不得**直接 `invoke()`。所有宿主能力经 `host/host.ts` 的 Host 接口，
 *     真实 Tauri 实现在 `TauriHost`。理由：换宿主/单测时只改一处。
 *  ② 界面**不得**写 `if (isMac)` / `if (isTauri)` 判功能可不可用。
 *     一律问 `CapabilityRegistry` —— 这样「这个功能为什么不见了」有确定答案。
 *  ③ 侧栏面板**不得**写死 HTML。它由注册表 `menuFor()` 驱动，
 *     所以加一个能力就多一个面板，而不是去改这个文件。
 */

import { CapabilityRegistry, type PluginSelf } from "./plugin/contract.ts";
import { defaultCapabilities } from "./host/host.ts";
import type { HostKind } from "./plugin/contract.ts";
import { invoke, type Commands, type TraceRow } from "./ipc.ts";
import { renderList, type ListItem } from "./list.ts";
import { openSheet } from "./ui/sheet.ts";
import { appendBlock } from "./ui/blocks.ts";
import type { AnswerView } from "./ui/panel-view.ts";
import type { DecisionPanel } from "./ui/block-model.ts";
import type { DecisionPanelWire, EvidenceReport } from "./ipc.ts";
import { renderIsland } from "./ui/island.ts";
import { deriveIsland } from "./ui/island-model.ts";
import { hasPendingPanel } from "./ui/block-model.ts";
import type { Block, ToolStep } from "./ui/block-model.ts";

// ── 宿主判定：只问「有没有 Tauri 运行时」，不问平台 ──────────────
const w = window as unknown as { __TAURI_INTERNALS__?: unknown; __TAURI__?: unknown };
const HOST: HostKind = w.__TAURI_INTERNALS__ || w.__TAURI__ ? "tauri" : "browser";

const $ = <T extends HTMLElement = HTMLElement>(id: string): T =>
  document.getElementById(id) as T;

const thread = $("thread");
const input = $<HTMLTextAreaElement>("input");
const islandEl = $("island");

// ── 能力矩阵 ────────────────────────────────────────────────
const caps = new CapabilityRegistry();
caps.provideAll(defaultCapabilities(HOST));

/**
 * 注册侧栏面板。
 *
 * `requires` 里写的是**能力 id**而不是平台名：面板要的是「能聊天」「有本地推理」，
 * 不是「跑在 Tauri 上」。这样浏览器预览与桌面走同一套判定。
 */
caps.register({
  self: {
    id: "chat", name: "对话", summary: "会话与流式对话",
    icon: "chat", order: 10, hosts: ["tauri", "browser"],
    requires: { tauri: ["chat"], browser: ["chat"], node: ["chat"] },
  },
  provides: ["chat"],
});
caps.register({
  self: {
    id: "local", name: "本地推理", summary: "本地模型与引擎状态",
    icon: "sliders", order: 20, hosts: ["tauri"],
    requires: { tauri: ["chat", "local-model"], browser: ["chat"], node: ["chat"] },
    // ⛔ 不要把这个写成 `hosts: ["tauri"]` 就完事 —— 那样浏览器里
    //    会看到一个点了没反应的按钮。requires 让它**整块消失**并给出理由。
  },
  provides: ["local-model"],
});
caps.register({
  self: {
    id: "audit", name: "审计", summary: "工具调用与决策记录",
    icon: "layers", order: 30, hosts: ["tauri", "browser"],
    requires: { tauri: ["chat"], browser: ["chat"], node: ["chat"] },
  },
  provides: ["audit"],
});

// ── 侧栏面板由注册表生成 ──────────────────────────────────────
/**
 * 渲染侧栏面板按钮。
 *
 * 注意 `menuFor()` 而不是 `pluginsFor()`：前者只给**可用且非 hidden** 的，
 * 后者给全部注册的。区别在于后者会让「当前宿主用不了的面板」也冒出来。
 */
function renderPanels(active: string | null): void {
  const box = $("side-panels");
  box.replaceChildren();
  for (const p of caps.menuFor(HOST)) {
    const b = document.createElement("button");
    b.className = "side-panel-btn";
    b.type = "button";
    b.dataset["panel"] = p.id;
    b.setAttribute("aria-current", String(p.id === active));
    b.textContent = p.name;
    b.title = p.summary;
    b.addEventListener("click", () => selectPanel(p.id));
    box.appendChild(b);
  }
}
function selectPanel(id: string): void {
  const p = caps.get(id)?.self;
  if (!p) return;
  renderPanels(id);
  // unavailable 面板不该走到这里（菜单里根本不会出现），但仍做一次防御
  if (!caps.isAvailable(id, HOST)) {
    note("warn", caps.pluginVerdict(id, HOST).hint ?? "当前宿主不支持");
    return;
  }
  mark(`面板 · ${p.name}`);
}

// ── 会话流：一切都是块 ─────────────────────────────────────────
// 2026-09-30 读三个参考仓后做的结构升级。它们的 .tsx 全在 mobile（CopilotKit
// + React Native），故**组件不可移植，可移植的是模型**：
// openmuse 的 jev.ts（结构化决策面板）、DSH TUI 的 block-stream-writer（块序列 +
// tool-card）、douchat 的 CodeArtifact（产物块）—— 三条独立路径，同一结论：
// 扁平 text 管子装不下 agent 实际要表达的东西。
//
// 唯一代码可吸收的 openmuse 是 MIT；另两个是 AGPL / PolyForm Noncommercial，
// 只取思路不取码（见 ABSORPTION 记录）。

/** 线程内已渲染的块数。用作 blockKey 的稳定序号。 */
let blockSeq = 0;
/**
 * 已入流的块。活动岛**从它推导状态**，所以岛不会与线程脱节 ——
 * 一个只看 DOM 的岛，在块被重绘/折叠时就会失准。
 */
const blocks: Block[] = [];

/** 唯一的追加入口。界面**不得**再手写 thread.appendChild。 */
function pushBlock(b: Block): void {
  blocks.push(b);
  appendBlock(thread, b, blockSeq, { onAnswer: onPanelAnswer });
  blockSeq += 1;
  thread.scrollTop = thread.scrollHeight;
  refreshIsland();
}

/** 会话身份标记。独立于内容 —— 一条消息可承载多种块。 */
function mark(text: string): void {
  pushBlock({ kind: "mark", text });
}

function say(role: "user" | "bot", text: string, meta?: string): void {
  pushBlock({ kind: "text", role, text, meta });
}

/** 工具调用折叠卡。失败步在展开前就能看见（summarize 会带「N 步失败」）。 */
function pushTool(steps: ToolStep[]): void {
  pushBlock({ kind: "tool", steps });
}

/**
 * 决策面板的作答回流 —— 走**骨架侧**的校验命令。
 *
 * ⛔ 上一版用 `neobot_send` 把答案当普通文本发出去，等于**绕过了过期检测**：
 *     骨架收到的是一句「[选择] A（候选集 v1）」，它无从判断 v1 是不是过期。
 *     现在发 `neobot_panel_answer`，由 `nt_panel::validate_answer` 判定。
 *     前端那份校验仍在（要即时反馈），但它**只是体验，不是保护** ——
 *     真正的状态判定必须在骨架这侧。
 */
function onPanelAnswer(a: AnswerView): void {
  // ⛔ 只送 answer，**不送 panel**。基准由骨架侧的注册表持有 ——
  //    界面自带基准的话，任何校验都能被绕过（编一个同 id 的面板即可）。
  //    这与 ActorContext.resolvedBy 是同一条纪律。
  const res = call("neobot_panel_answer", {
    answer: {
      panel_id: a.panelId,
      option_id: a.optionId,
      candidate_set_version: a.candidateSetVersion,
    },
  });
  void res.then((r) => {
    if (!r.ok) { note("error", r.error, undefined, true); return; }
    const out = r.value;
    if ("accepted" in out) {
      const blk = blocks.find(
        (b): b is Extract<Block, { kind: "panel" }> =>
          b.kind === "panel" && b.panel.id === a.panelId,
      );
      if (blk) blk.panel.selectedId = a.optionId;
      setBusy(false);
      say("bot", `已记录你的选择：${a.label}`, `候选集 v${a.candidateSetVersion}`);
    } else {
      // 骨架拒收 ⇒ 不把选择标成已答（那是谎报状态）
      const rej = out.rejected;
      const why = "stale" in rej
        ? `候选集已更新（v${rej.stale.answered} → v${rej.stale.current}），请重新选择`
        : "no_such_option" in rej ? "该选项已不在候选集里，请重新选择"
        : "no_such_panel" in rej ? "该面板已失效（骨架未登记或已清理）"
        : `骨架拒绝了该面板：${rej.panel_invalid}`;
      note("warn", why);
    }
  });
}

/**
 * 订阅骨架下发的决策面板事件。
 *
 * ⛔ 这是面板进入界面的**唯一**通道。上一轮界面在启动时自己造了一个演示面板，
 *    那条路已删 —— 界面造面板 = 基准由被判定方提供，过期检测与能力门控同时失效。
 *    现在没有骨架就没有面板，界面上只出现「没有待答事项」，这是**正确**的。
 */
async function listenPanels(): Promise<void> {
  if (HOST !== "tauri") return;
  try {
    const { listen } = await import("@tauri-apps/api/event");
    await listen<DecisionPanelWire>("neobot:panel", (e) => {
      const w = e.payload;
      const panel: DecisionPanel = {
        id: w.id,
        threadId: w.thread_id,
        turnId: w.turn_id,
        candidateSetVersion: w.candidate_set_version,
        type: w.kind,
        title: w.title,
        mode: w.mode ?? "sample",
        options: w.options.map((o) => ({
          id: o.id,
          label: o.label,
          details: o.details ?? [],
          sources: (o.sources ?? []).map((x) => ({ title: x.title, url: x.url })),
        })),
      };
      // 同 id 再来一块 ⇒ 替换（骨架换批候选）。旧块从流里移除，
      // 留着两张同 id 的卡会让人以为有两件事在等他答。
      const at = blocks.findIndex((b) => b.kind === "panel" && b.panel.id === panel.id);
      if (at >= 0) blocks.splice(at, 1);
      thread.replaceChildren();
      blocks.forEach((b, i) => appendBlock(thread, b, i, { onAnswer: onPanelAnswer }));
      pushBlock({ kind: "panel", panel });
    });
  } catch (e) {
    note("warn", `无法订阅面板事件：${String(e).slice(0, 120)}`);
  }
}

/** 系统消息。`dataIntact` 显式声明数据是否还在 —— 宁可写死也不让用户猜。 */
function note(level: "error" | "warn" | "info", text: string, detail?: string, dataIntact?: boolean): void {
  pushBlock({ kind: "system", level, text, detail, dataIntact });
}

/** 是否正在跑。岛的状态由它 + 块流**推导**，不手工指定。 */
let busy = false;

/**
 * 重算活动岛。
 *
 * ⛔ 不接受「把岛设成某状态」这种接口 —— 那样状态就有两个来源
 *    （人手设一次、块流推一次），迟早不一致。现在它只从
 *    「忙碌标记 + 块流 + 能力矩阵」推，人改不了。
 */
function refreshIsland(): void {
  renderIsland(islandEl, deriveIsland(blocks, { caps, host: HOST, busy, seq: blockSeq }));
}

/** 忙碌标记。语义上属于「跑轮在飞」，故与块流分开但在同一处翻。 */
function setBusy(v: boolean): void {
  busy = v;
  refreshIsland();
}

// ── 宿主调用（规则 ①：只在这里碰 invoke） ──────────────────────
/**
 * 薄封装：把「是否 Tauri 宿主」这层判断收在这里，调用点只管业务。
 *
 * ⛔ 旧版是 `call<T>(cmd, args) { ... as T }` —— 返回类型由**调用方断言**，
 *    于是 `call<string>("neobot_evidence_summary")` 编译通过而运行时炸
 *    （后端返回的是对象）。
 *
 * ⚠️ 这里**必须**直接写 `<K extends keyof Commands>` 并用 `Commands[K][...]`。
 *    第一版写成 `call<K extends Parameters<typeof invoke>[0]>`，K 被拓宽成
 *    键的并集 ⇒ 按键的 args/ret 推导全部失效（tsc 报 5 处），
 *    等于把「推导」悄悄退回「union 后什么也推不出来」。
 */
const call = <K extends keyof Commands>(
  cmd: K,
  args: Commands[K]["args"],
): ReturnType<typeof invoke<K>> => invoke(cmd, args, HOST === "tauri");

// ── 能力入口 ────────────────────────────────────────────────
/** convo_group：建群（DSH sheet 形态：标题 + 多选成员） */
function newGroup(): void {
  void openSheet({
    title: "新建群组",
    subtitle: "多人共用一个会话上下文。",
    submit: "创建",
    fields: [
      { kind: "text", id: "title", label: "群名", hint: "留空则用成员名拼", placeholder: "例如：发布值班" },
      {
        kind: "pick", id: "members", label: "成员", multiple: true,
        options: MOCK_MEMBERS.map((m) => ({ id: m, label: m })),
      },
    ],
    onSubmit: async (v) => {
      const title = String(v["title"] ?? "").trim() || "新群";
      const members = (v["members"] as string[] | undefined) ?? [];
      const r = await call("neobot_convo_group", { title, members });
      if (!r.ok) return r.error;          // 失败留在弹层里，不静默关闭
      mark(`群组 · ${title}`);
      return undefined;
    },
  });
}

/** agent_run：手动跑一轮（类型表缺口：返回形状在此显式声明） */
function runOnce(): void {
  void openSheet({
    title: "手动跑一轮",
    subtitle: "给 agent 一个目标，结果会落进本会话。",
    submit: "跑",
    fields: [
      { kind: "text", id: "goal", label: "目标", required: true, multiline: true, placeholder: "例如：把 TODO 汇总成清单" },
      { kind: "text", id: "context", label: "上下文", multiline: true, hint: "可空。" },
    ],
    onSubmit: async (v) => {
      const goal = String(v["goal"] ?? "").trim();
      if (!goal) return "「目标」不能为空";
      const ctx = String(v["context"] ?? "").trim();
      setBusy(true);
      const r = await call("neobot_agent_run", ctx ? { goal, context: ctx } : { goal });
      setBusy(false);
      if (!r.ok) return r.error;
      say("bot", r.value.output || `（${r.value.status}，无输出）`, r.value.model_used);
      return undefined;
    },
  });
}

/** evidence_summary：一句话结论 */
function evidence(): void {
  const text = thread.textContent ?? "";
  if (!text.trim()) { note("warn", "这一轮还没有内容可审"); return; }
  // 演示骨架下发一块面板，验证「publish → 事件 → 界面 → answer → 注册表校验」全链。
// ⛔ 浏览器预览（HOST !== tauri）下无骨架 ⇒ 按钮不出现，避免让人以为它坏了。
if (HOST === "tauri") {
  $("#demo-panel")?.addEventListener("click", () => {
    void call("neobot_panel_demo_publish", {}).then((r) => {
      if (!r.ok) { note("error", `面板下发失败：${r.error}`, undefined, true); return; }
      note("info", `骨架已登记候选集 v${r.value}，面板正从 neobot:panel 事件过来。`);
    });
  });
}

void call("neobot_evidence_summary", { text }).then((r) => {
    if (!r.ok) { note("error", r.error, undefined, true); return; }
    const rep = r.value;
    const steps: ToolStep[] = rep.findings.map((f) => ({
      kind: f.kind === "overclaim" ? "过度断言" : f.kind === "unsourced" ? "断言无出处" : "数值不符",
      detail: f.excerpt,
      failed: true,
    }));
    steps.unshift(rep.clean
      ? { kind: "证据齐备", detail: rep.summary }
      : { kind: "证据不足", detail: rep.summary, failed: true });
    if (rep.sourced_ratio !== null) {
      steps.push({ kind: "有出处比例", detail: `${Math.round(rep.sourced_ratio * 100)}%` });
    }
    pushTool(steps);
    // 逐条明细进 Inspector：流里只是摘要，「哪一句有问题」得能展开看。
    showEvidence(rep);
  });
}

/** Inspector 显示证据明细。 */
function showEvidence(rep: EvidenceReport): void {
  $("app").dataset["inspector"] = "open";
  $("inspector").dataset["open"] = "true";
  $("inspector-title").textContent = "证据";
  const body = $("inspector-body");
  body.replaceChildren();

  const head = document.createElement("p");
  head.className = rep.clean ? "nb-item-title" : "nb-error";
  head.textContent = rep.summary;
  body.appendChild(head);

  const kv = (k: string, v: string) => {
    const row = document.createElement("div");
    row.className = "nb-item-row";
    const l = document.createElement("div");
    l.className = "nb-item-left";
    const ks = document.createElement("span");
    ks.className = "nb-item-sub";
    ks.textContent = k;
    l.appendChild(ks);
    const val = document.createElement("span");
    val.className = "nb-item-title";
    val.textContent = v;
    row.append(l, val);
    return row;
  };
  // ⛔ ratio 为 null 表示「没断言过」，显示「未检查」而不是 0%
  body.append(kv("有出处比例", rep.sourced_ratio === null
    ? "未检查（本轮无断言）"
    : `${Math.round(rep.sourced_ratio * 100)}%`));
  body.append(kv("问题数", String(rep.findings.length)));

  for (const f of rep.findings) {
    const card = document.createElement("div");
    card.className = "nb-item";
    const h = document.createElement("div");
    h.className = "nb-item-row";
    const l = document.createElement("div");
    l.className = "nb-item-left";
    const k = document.createElement("span");
    k.className = "nb-item-title";
    k.textContent = f.kind === "overclaim" ? "过度断言" : f.kind === "unsourced" ? "断言无出处" : "数值不符";
    l.appendChild(k);
    h.appendChild(l);
    card.appendChild(h);
    const ex = document.createElement("div");
    ex.className = "nb-item-sub";
    ex.textContent = f.excerpt;
    card.appendChild(ex);
    body.appendChild(card);
  }
}

// ── 演示数据（接真后端前先把界面跑起来；真数据到位后删） ──────────
// ⛔ 全是假数据时**不要**让界面看起来像真的：panel/rail 上标了「演示」，
//    空态文案也说明这一点。演示数据冒充真实是这个项目的原罪之一
//    （`nt_send` 那条链路至今还是 MOCK）。
const MOCK_MEMBERS = ["neo", "ada", "lin", "kiro"];
const MOCK_ITEMS: ListItem[] = [
  { id: "c1", title: "发布值班", sub: "已确认回滚脚本，等评审", tail: "14:02", unread: 2, when: "今天" },
  { id: "c4", title: "性能回归排查", sub: "火焰图已贴上来", tail: "13:20", when: "今天" },
  { id: "c5", title: "周会纪要", sub: "待你补第 3 节", tail: "11:05", when: "今天" },
  { id: "c2", title: "架构评审", sub: "三栏布局方案已定", tail: "昨天", pinned: true, when: "昨天" },
  { id: "c3", title: "与 ada", sub: "关于证据模块的接口", tail: "9-28", when: "更早" },
  { id: "c6", title: "长任务：全仓死代码清扫", sub: "已扫 279 个文件，212 真死", tail: "9-27", when: "更早" },
];

let selectedId: string | null = null;

/** 渲染侧栏列表。分组/条目解剖见 list.ts（交集来源）。 */
function renderConvos(): void {
  renderList($("convs"), {
    items: MOCK_ITEMS,
    selected: selectedId,
    query: ($("search") as HTMLInputElement).value,
    emptyText: "还没有会话 —— 上面点「群」建一个",
    onPick: (id) => {
      selectedId = id;
      const it = MOCK_ITEMS.find((x) => x.id === id);
      mark(`会话 · ${it?.title ?? id}`);
      renderConvos();
      openInspector(it);
    },
  });
}

// ── Inspector：详情开在右侧（交集：详情不内联） ───────────────
function openInspector(it: ListItem | undefined): void {
  const app = $("app");
  const insp = $("inspector");
  const body = $("inspector-body");
  if (!it) {
    app.dataset["inspector"] = "closed";
    insp.dataset["open"] = "false";
    return;
  }
  app.dataset["inspector"] = "open";
  insp.dataset["open"] = "true";
  $("inspector-title").textContent = it.title;
  body.replaceChildren();

  const kv = (k: string, v: string) => {
    const row = document.createElement("div");
    row.className = "nb-item-row";
    const l = document.createElement("div");
    l.className = "nb-item-left";
    l.appendChild(Object.assign(document.createElement("span"),
      { className: "nb-item-sub", textContent: k }));
    const val = document.createElement("span");
    val.className = "nb-item-title";
    val.textContent = v;
    row.append(l, val);
    return row;
  };

  body.append(kv("条目", it.id));
  body.append(kv("时间", it.tail ?? "—"));
  if (it.pinned) body.append(kv("置顶", "是"));
  if (it.unread) body.append(kv("未读", String(it.unread)));
  if (it.sub) {
    const p = document.createElement("p");
    p.className = "nb-item-sub";
    p.textContent = it.sub;
    body.append(p);
  }
  // ⛔ 明说这是演示数据。空态/详情都标出来，避免「看起来在工作」。
  const demo = document.createElement("p");
  demo.className = "nb-empty";
  demo.textContent = "演示数据：尚未接入真实会话列表";
  body.append(demo);
}

// ── Rail：能力门控的导航入口 ────────────────────────────────
/**
 * 渲染 Rail 入口。
 *
 * ⛔ **不可用的入口不渲染**（而不是渲染成灰的）。理由：四家参考仓都有
 *    「灰按钮」，但用户点它得不到任何反馈，只能猜。
 *    这里直接不出现 ⇒ 界面上不存在「点了没反应」的东西。
 *    若某能力重要到必须可见，改成 `rail-btn--blocked` + title 说明，
 *    而不是留一个哑按钮。
 */
function renderRail(active: string | null): void {
  const box = $("rail");
  for (const p of caps.menuFor(HOST)) {
    if (p.hidden) continue;
    const b = document.createElement("button");
    b.className = "rail-btn";
    b.type = "button";
    b.title = p.summary;
    b.setAttribute("aria-label", p.name);
    b.setAttribute("aria-current", String(p.id === active));
    b.textContent = railGlyph(p.id);
    b.addEventListener("click", () => {
      $("panel-title").textContent = p.name;
      renderRail(p.id);
      mark(`面板 · ${p.name}`);
    });
    box.appendChild(b);
  }
}

/** Rail 上的字形。用字符而非图标资源：省一次请求，且不依赖图标表。 */
function railGlyph(id: string): string {
  switch (id) {
    case "chat": return "◍";
    case "local": return "◉";
    case "audit": return "▤";
    default: return "◻";
  }
}

// ── 事件绑定 ────────────────────────────────────────────────
$("btn-new-group").addEventListener("click", () => newGroup());
$("btn-agent-run").addEventListener("click", () => runOnce());
$("btn-evidence").addEventListener("click", () => evidence());
$("search").addEventListener("input", () => renderConvos());
$("btn-panel-toggle").addEventListener("click", () => {
  const app = $("app");
  app.dataset["panel"] = app.dataset["panel"] === "open" ? "collapsed" : "open";
});
$("btn-inspector").addEventListener("click", () => {
  const app = $("app");
  app.dataset["inspector"] = app.dataset["inspector"] === "open" ? "closed" : "open";
  $("inspector").dataset["open"] = app.dataset["inspector"] === "open" ? "true" : "false";
});
$("btn-inspector-close").addEventListener("click", () => {
  $("app").dataset["inspector"] = "closed";
  $("inspector").dataset["open"] = "false";
});
$("btn-send").addEventListener("click", () => {
  const text = input.value.trim();
  if (!text) return;
  say("user", text);
  input.value = "";
  setBusy(true);
  void call("neobot_send", { text })
    .then((r) => {
      setBusy(false);
      if (!r.ok) { note("error", `发送失败：${r.error}`, undefined, true); return; }
      // 工具步骤若随结果一起回来，就落成折叠卡；没有就只显示回复。
      if (r.value.trace.length > 0) {
        pushTool(r.value.trace.map((t: TraceRow) => ({
          kind: t.kind,
          detail: t.detail,
          failed: t.failed,
        })));
      }
      say("bot", r.value.output || "（无输出）", r.value.model_used);
    });
});
input.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); $("btn-send").click(); }
});
// Enter 发送后焦点留在输入框（键盘流不该被打断）
$("btn-send").addEventListener("click", () => input.focus());

// ── 启动 ────────────────────────────────────────────────────
renderRail(null);
renderConvos();

// ── 演示内容 ──
// ⛔ **不再造决策面板**。面板只能来自骨架（`neobot:panel` 事件）；
//    界面自造面板会让过期检测与能力门控同时失去基准。
//    浏览器预览下没有骨架 ⇒ 没有面板 ⇒ 空态提示「暂无待答事项」，这是正确的。
{
  mark("演示");
  say("user", "把接口迁移方案定下来");
  say("bot", "我需要你定一个方向。", "本地模型 · local-model");
  pushTool([
    { kind: "读取", detail: "frontend/src/ipc.ts（165 行）" },
    { kind: "比对", detail: "3 处命令名不一致", failed: true },
  ]);
  note("info", "决策面板由骨架通过 neobot:panel 事件下发。浏览器预览下无骨架，故此处没有待答面板。");
}

void listenPanels();

// 侧栏面板可用性：让测试/调试能直接看到判定结果，不靠猜。
if (import.meta.env?.DEV) {
  (window as unknown as { __nb: unknown }).__nb = { caps, HOST, renderRail, renderConvos };
}
