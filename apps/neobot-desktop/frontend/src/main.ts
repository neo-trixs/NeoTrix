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
import { openSheet } from "./ui/sheet.ts";
import { appendBlock } from "./ui/blocks.ts";
import { renderIsland } from "./ui/island.ts";
import { deriveIsland } from "./ui/island-model.ts";
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
  appendBlock(thread, b, blockSeq);
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
type Result<T> = { ok: true; value: T } | { ok: false; error: string };

/** Tauri 存在时走 IPC，否则返回一个**可读**的失败原因而不是崩。 */
async function call<T>(cmd: string, args: Record<string, unknown> = {}): Promise<Result<T>> {
  if (HOST !== "tauri") {
    return { ok: false, error: `浏览器预览下无法调用 ${cmd}（没有 Tauri 运行时）` };
  }
  try {
    const core = await import("@tauri-apps/api/core");
    return { ok: true, value: (await core.invoke(cmd, args)) as T };
  } catch (e) {
    return { ok: false, error: String(e instanceof Error ? e.message : e).slice(0, 200) };
  }
}

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
      const r = await call<string>("neobot_convo_group", { title, members });
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
      const r = await call<{ status: string; output: string; model_used?: string }>(
        "neobot_agent_run", ctx ? { goal, context: ctx } : { goal });
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
  void call<string>("neobot_evidence_summary", { text }).then((r) => {
    if (!r.ok) { note("error", r.error, undefined, true); return; }
    const ok = r.value.includes("齐备");
    // 证据是「对整轮的判断」，不是一句回复 ⇒ 走 tool 卡（可折叠、可追溯步骤），
    // 而不是伪装成一条消息。
    pushTool([{ kind: ok ? "证据齐备" : "证据不足", detail: r.value, failed: !ok }]);
  });
}

// ── 演示数据（接真后端前先把界面跑起来；真数据到位后删） ──────────
const MOCK_MEMBERS = ["neo", "ada", "lin", "kiro"];
const MOCK_CONVOS = [
  { id: "c1", title: "发布值班", time: "14:02" },
  { id: "c2", title: "架构评审", time: "昨天" },
  { id: "c3", title: "与 ada", time: "9-28" },
];

function renderConvos(q: string): void {
  const box = $("convs");
  box.replaceChildren();
  const hits = MOCK_CONVOS.filter((c) => c.title.toLowerCase().includes(q.toLowerCase()));
  if (hits.length === 0) {
    const e = document.createElement("div");
    e.className = "nb-empty";
    e.textContent = q ? `没有匹配「${q}」的会话` : "还没有会话";
    box.appendChild(e);
    return;
  }
  for (const c of hits) {
    const row = document.createElement("div");
    row.className = "convo";
    row.setAttribute("role", "option");
    const txt = document.createElement("div");
    txt.className = "convo-txt";
    const t = document.createElement("div");
    t.className = "convo-title";
    t.textContent = c.title;
    txt.appendChild(t);
    const time = document.createElement("span");
    time.className = "convo-time";
    time.textContent = c.time;
    row.append(txt, time);
    row.addEventListener("click", () => mark(`会话 · ${c.title}`));
    box.appendChild(row);
  }
}

// ── 事件绑定 ────────────────────────────────────────────────
$("btn-new-group").addEventListener("click", () => newGroup());
$("btn-agent-run").addEventListener("click", () => runOnce());
$("btn-evidence").addEventListener("click", () => evidence());
$("search").addEventListener("input", (e) => {
  renderConvos((e.target as HTMLInputElement).value);
});
$("btn-send").addEventListener("click", () => {
  const text = input.value.trim();
  if (!text) return;
  say("user", text);
  input.value = "";
  setBusy(true);
  void call<{ output?: string; status?: string; trace?: unknown[] }>("neobot_send", { text })
    .then((r) => {
      setBusy(false);
      if (!r.ok) { note("error", `发送失败：${r.error}`, undefined, true); return; }
      // 工具步骤若随结果一起回来，就落成折叠卡；没有就只显示回复。
      const steps = Array.isArray(r.value.trace) ? r.value.trace : [];
      if (steps.length > 0) {
        pushTool(steps.map((t) => {
          const o = t as { kind?: string; detail?: string; failed?: boolean };
          return { kind: o.kind ?? "tool", detail: o.detail ?? "", failed: o.failed };
        }));
      }
      say("bot", r.value.output ?? "（无输出）");
    });
});
input.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); $("btn-send").click(); }
});
// Enter 发送后焦点留在输入框（键盘流不该被打断）
$("btn-send").addEventListener("click", () => input.focus());

// ── 启动 ────────────────────────────────────────────────────
renderPanels(null);
renderConvos("");
// 能力门控把「当前宿主不支持」的部分整块隐藏，并给出可读理由。
$("host-badge").textContent =
  HOST === "tauri" ? "本地运行时" : "浏览器预览（部分能力不可用）";
refreshIsland();   // 启动也走推导路径，不手工置初值

// 侧栏面板可用性：让测试/调试能直接看到判定结果，不靠猜。
if (import.meta.env?.DEV) {
  (window as unknown as { __nb: unknown }).__nb = { caps, HOST, renderPanels };
}
