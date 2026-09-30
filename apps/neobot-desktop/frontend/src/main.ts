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

// ── 宿主判定：只问「有没有 Tauri 运行时」，不问平台 ──────────────
const w = window as unknown as { __TAURI_INTERNALS__?: unknown; __TAURI__?: unknown };
const HOST: HostKind = w.__TAURI_INTERNALS__ || w.__TAURI__ ? "tauri" : "browser";

const $ = <T extends HTMLElement = HTMLElement>(id: string): T =>
  document.getElementById(id) as T;

const thread = $("thread");
const input = $<HTMLTextAreaElement>("input");
const statusEl = $("status");

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
    toast(caps.pluginVerdict(id, HOST).hint ?? "当前宿主不支持", "err");
    return;
  }
  mark(`面板 · ${p.name}`);
}

// ── 消息渲染 ────────────────────────────────────────────────
interface Line { role: "user" | "bot"; text: string; meta?: string; }

function mark(text: string): void {
  const el = document.createElement("div");
  el.className = "session-mark";
  el.textContent = text;
  thread.appendChild(el);
}
function pushLine(role: "user" | "bot", text: string, meta?: string): void {
  const wrap = document.createElement("div");
  wrap.className = `msg msg--${role === "user" ? "user" : "bot"}`;
  const bubble = document.createElement("div");
  bubble.className = "msg-bubble";
  bubble.textContent = text;               // 永不用 innerHTML 拼用户内容
  wrap.appendChild(bubble);
  if (meta) {
    const m = document.createElement("div");
    m.className = "msg-meta";
    m.textContent = meta;
    wrap.appendChild(m);
  }
  thread.appendChild(wrap);
  thread.scrollTop = thread.scrollHeight;
}
function setStatus(text: string, state: "idle" | "running" | "ok" | "error"): void {
  statusEl.textContent = text;
  statusEl.dataset["state"] = state;
}
function toast(text: string, kind: "ok" | "err" = "ok"): void {
  const box = document.createElement("div");
  box.className = kind === "err" ? "nb-error" : "nb-empty";
  box.setAttribute("role", kind === "err" ? "alert" : "status");
  box.textContent = text;
  thread.appendChild(box);
  box.scrollIntoView({ block: "nearest" });
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
      setStatus("运行中", "running");
      const r = await call<{ status: string; output: string; model_used?: string }>(
        "neobot_agent_run", ctx ? { goal, context: ctx } : { goal });
      setStatus("空闲", r.ok ? "ok" : "error");
      if (!r.ok) return r.error;
      pushLine("bot", r.value.output || `（${r.value.status}，无输出）`, r.value.model_used);
      return undefined;
    },
  });
}

/** evidence_summary：一句话结论 */
function evidence(): void {
  const text = thread.textContent ?? "";
  if (!text.trim()) { toast("这一轮还没有内容可审", "err"); return; }
  void call<string>("neobot_evidence_summary", { text }).then((r) => {
    if (!r.ok) { toast(r.error, "err"); return; }
    const ok = r.value.includes("齐备");
    const box = document.createElement("div");
    box.className = ok ? "nb-item" : "nb-error";
    box.setAttribute("role", "status");
    const t = document.createElement("div");
    t.className = "nb-item-title";
    t.textContent = `证据：${r.value}`;
    box.appendChild(t);
    thread.appendChild(box);
    box.scrollIntoView({ block: "nearest" });
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
  pushLine("user", text);
  input.value = "";
  setStatus("运行中", "running");
  void call<{ output?: string; status?: string }>("neobot_send", { text })
    .then((r) => {
      setStatus("空闲", r.ok ? "ok" : "error");
      pushLine("bot", r.ok ? (r.value.output ?? "（无输出）") : `发送失败：${r.error}`);
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
setStatus(HOST === "tauri" ? "空闲" : "预览模式", HOST === "tauri" ? "idle" : "error");

// 侧栏面板可用性：让测试/调试能直接看到判定结果，不靠猜。
if (import.meta.env?.DEV) {
  (window as unknown as { __nb: unknown }).__nb = { caps, HOST, renderPanels };
}
