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
import { answerToText } from "./ui/panel-view.ts";
import type { DecisionPanel } from "./ui/block-model.ts";
import type { EvidenceReport } from "./ipc.ts";
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
 * 决策面板的作答回流。
 *
 * ⛔ 真实实现应调 `neobot_panel_answer` 把 `candidateSetVersion` 一起送回，
 *    骨架据此判断这是不是**过期作答**。当前后端尚无该命令
 *    （见 FRONTEND-GAP），故先落成本地回显并**显式说明**，
 *    不假装骨架已经收到。
 */
function onPanelAnswer(a: { optionId: string; candidateSetVersion: number; label: string }): void {
  // ⚠️ 必须把 selectedId 写回块流，否则活动岛会一直说「等你选择」，
  //    用户以为没提交成功就会再点一次。
  const pend = hasPendingPanel(blocks);
  if (pend) {
    for (const b of blocks) {
      if (b.kind === "panel" && b.panel.id === pend.id) b.panel.selectedId = a.optionId;
    }
  }
  setBusy(true);
  void call("neobot_send", { text: answerToText(a) }).then((r) => {
    setBusy(false);
    if (r.ok) {
      say("bot", r.value.output || "（已记录你的选择）", r.value.model_used);
    } else {
      // 骨架还没实现该命令 ⇒ 明确说出来，不假装送达
      note("warn", `选择已记在本机，骨架尚未接收：${r.error}`);
      say("user", answerToText(a));
    }
  });
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

// ── 演示内容（让空态被真实形状替代；真数据到位后整段删） ──
// ⛔ 全部标 mode:"sample" / 显式文案，不让它看起来像真的产出。
{
  mark("演示 · 决策面板");
  say("user", "把接口迁移方案定下来");
  say("bot", "有两处不一致，需要你定一下方向。", "DeepSeek-Chat · fallback");
  pushTool([
    { kind: "读取", detail: "frontend/src/ipc.ts（118 行）" },
    { kind: "读取", detail: "crates/neotrix-neobot/src/nt_evidence.rs（329 行）" },
    { kind: "比对", detail: "发现 3 处命令名不一致", failed: true },
  ]);
  const panel: DecisionPanel = {
    id: "p-migrate-1",
    threadId: "c1",
    turnId: "tu-7",
    candidateSetVersion: 1,
    type: "comparison",
    title: "接口不一致怎么处理？",
    mode: "sample",
    options: [
      {
        id: "opt-rename",
        label: "改前端跟随后端",
        details: ["3 处都是前端写错", "后端不动，风险最低"],
        sources: [{ title: "ipc.ts 对照表", url: "https://example.com/nb/ipc" }],
      },
      {
        id: "opt-alias",
        label: "后端加别名兼容",
        details: ["两侧都不用改", "会留下长期别名债务"],
        sources: [
          { title: "对齐门", url: "https://example.com/nb/gate" },
          { title: "债务清单", url: "https://example.com/nb/debt" },
        ],
      },
      {
        id: "opt-gen",
        label: "从 Rust 生成前端类型",
        details: ["根治", "要引入 build 步骤，本轮不做"],
        sources: [{ title: "设计稿", url: "https://example.com/nb/codegen" }],
      },
    ],
  };
  pushBlock({ kind: "panel", panel });
}
// 能力门控把「当前宿主不支持」的部分整块隐藏，并给出可读理由。
/**
 * 宿主徽标：**说清「少了什么、为什么少」**。
 *
 * ⛔ 只写「部分能力不可用」是不够的 —— 用户看到 Rail 上少了入口，
 *    只会以为功能没做完。把被门控掉的面板与理由直接列出来。
 *    理由取自注册表，不在本文件编（否则「为什么这个没出现」会有两套答案）。
 */
{
  const badge = $("host-badge");
  const blocked = caps
    .pluginsFor(HOST)
    .filter((p) => !caps.isAvailable(p.id, HOST));
  if (blocked.length === 0) {
    badge.textContent = HOST === "tauri" ? "本地运行时 · 全部能力可用" : "浏览器预览";
    badge.removeAttribute("title");
  } else {
    const why = blocked
      .map((p) => `${p.name}（${caps.pluginVerdict(p.id, HOST).hint ?? "宿主不支持"}）`)
      .join("；");
    badge.textContent = `演示数据 · ${blocked.length} 个面板在本宿主不可用`;
    badge.title = why;
  }
}
refreshIsland();   // 启动也走推导路径，不手工置初值

// ── 交互自检（dev only）：决策面板必须真能选、能确认 ──
// ⛔ 面板是纯 UI，若「单选点不动」这类问题只有肉眼能发现，
//    那它就会活到用户手上。挂一个可断言的钩子给 e2e/手测用。
if (import.meta.env?.DEV) {
  (window as unknown as { __nbPanel: unknown }).__nbPanel = () => {
    const card = document.querySelector<HTMLElement>(".panel-card");
    if (!card) return { found: false };
    const input = card.querySelector<HTMLInputElement>("input[type=radio]");
    const btn = card.querySelector<HTMLButtonElement>(".panel-foot button");
    const before = btn?.disabled ?? null;
    input?.click();
    const after = btn?.disabled ?? null;
    return { found: true, optionCount: card.querySelectorAll("input[type=radio]").length,
             submitBefore: before, submitAfter: after,
             unlocked: before === true && after === false };
  };
}

// 侧栏面板可用性：让测试/调试能直接看到判定结果，不靠猜。
if (import.meta.env?.DEV) {
  (window as unknown as { __nb: unknown }).__nb = { caps, HOST, renderRail, renderConvos };
}
