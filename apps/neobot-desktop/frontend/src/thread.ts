/**
 * NeoBot 对话流 thread（转录存取 + 渲染 + 任务/agent 节点卡）。
 * 只依赖 core（壳状态）与 tauri invoke；不依赖 main.ts（无循环）。
 */

import { convertFileSrc } from "@tauri-apps/api/core";
import { icon } from "./icons";import {
  K, DEFAULT_STARTERS, STATUS_CN, cache, shell, thread, esc, toast, load, save,
  type AttInfo, type ChatMsg, type TaskItem, type AgentTraceItem,
  type ReplyMode, type TurnLabels,
} from "./core";

/** thread 内按钮的行为注入（main.ts 在启动时注册；保持依赖 main→thread 单向）。 */
export interface ThreadActions {
  /** 建议按钮：按序号填入作曲区。 */
  onSuggest(idx: number): void;
  /** 消息动作（convo, act, idx）。 */
  onMsg(convo: string, act: string, idx: number): void;
  /** 任务卡动作（act, taskId）。 */
  onTask(act: string, taskId: string): void;
  /** 引用原文（url；主窗口白名单外开）。 */
  onCite(url: string): void;
}
let actions: ThreadActions | null = null;
export function setThreadActions(a: ThreadActions): void {
  actions = a;
}
export function needActions(): ThreadActions | null {
  if (!actions) console.warn("[thread] actions not registered");
  return actions;
}

export interface EvidenceRef { n: string; title: string; url: string; desc: string; }

/** 证据行抽取：`[n] 标题 — url`（+ 紧随的非空描述行）→ 引用卡；正文不再裸贴长 URL。
 *  行内 `[n]` 上标 chip 由 mdLite 负责，独立生效。 */
export function splitEvidence(text: string): { body: string; evs: EvidenceRef[] } {
  const lines = text.split("\n");
  const out: string[] = [];
  const evs: EvidenceRef[] = [];
  const head = /^\s*\[(\d{1,3})\]\s+(.+?)\s+[—–-]\s+(https?:\/\/\S+)\s*$/;
  for (let i = 0; i < lines.length; i++) {
    const m = head.exec(lines[i]);
    if (!m) { out.push(lines[i]); continue; }
    const ev: EvidenceRef = { n: m[1], title: m[2].trim(), url: m[3].replace(/[.,;!?]+$/, ""), desc: "" };
    const nxt = (lines[i + 1] ?? "").trim();
    if (nxt && !head.test(lines[i + 1])) { ev.desc = nxt.slice(0, 160); i++; }
    evs.push(ev);
  }
  return { body: out.join("\n"), evs };
}

export function evidenceHtml(evs: EvidenceRef[]): string {
  if (evs.length === 0) return "";
  const rows = evs.slice(0, 10).map((e) =>
    `<div class="evrow"><sup class="cite">[${esc(e.n)}]</sup>`
    + `<span class="ev-title">${esc(e.title)}</span>`
    + `<button class="chip-btn ev-open" data-cite="${esc(e.url)}" title="系统浏览器打开原文">原文</button>`
    + (e.desc ? `<div class="ev-desc muted">${esc(e.desc)}</div>` : "")
    + `</div>`).join("");
  return `<div class="evlist">${rows}</div>`;
}

export function bubbleText(text: string): string {
  if (text.length <= 400) return mdLite(text);
  const head = text.slice(0, 400);
  const cut = head.lastIndexOf("\n");
  const preview = cut > 100 ? head.slice(0, cut) : head;
  return `<span class="preview-text">${mdLite(preview)}…</span><details><summary>展开全文（${text.length}字）</summary><span class="preview-text">${mdLite(text)}</span></details>`;
}

export function mdLite(raw: string): string {
  const text = esc(raw);
  const blocks: string[] = [];
  const noBlocks = text.replace(/```([\s\S]*?)```/g, (_m, code) => {
    blocks.push(`<pre class="codeblock">${code.replace(/^\n/, "")}</pre>`);
    return `\u0000${blocks.length - 1}\u0000`;
  });
  const inline = noBlocks
    .replace(/`([^`\n]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    // 人类对话风：# 标题不显示 # 号，转正文标题行（样式见 .htitle）。
    .replace(/^#{1,6}\s+(.+)$/gm, `<div class="htitle">$1</div>`)
    .replace(/\n/g, "<br>");
  const restored = inline.replace(/\u0000(\d+)\u0000/g, (_m, i) => blocks[Number(i)] ?? "");
  // 引用下标标签（[1][2] → 上标 chip；<pre> 代码块内不动，保持原文）。
  // 视频直嵌（mp4 直播卡；抖音链走 data-cite 系统打开）：同样跳过代码块。
  return restored
    .split(/(<pre class="codeblock">[\s\S]*?<\/pre>)/g)
    .map((seg, i) => i % 2 === 1 ? seg
      : seg
        .replace(/(https?:\/\/[^\s<>"']+\.mp4(?:\?[^\s<>"']*)?)/g,
          '<video class="attvid" src="$1" controls preload="metadata"></video>')
        .replace(/(https?:\/\/(?:www\.|v\.)?douyin\.com[^\s<>"']*)/g,
          '<button class="chip-btn vplay" data-cite="$1" title="系统浏览器打开">▶ 抖音视频 · 点击播放</button>')
        .replace(/\[(\d{1,3})\]/g, '<sup class="cite">[$1]</sup>'))
    .join("");
}

/** HH:MM（标签行时间；与 dayline 配合，细粒度只到分）。 */
export function msgTime(ts: number): string {
  const t = new Date(ts);
  return `${String(t.getHours()).padStart(2, "0")}:${String(t.getMinutes()).padStart(2, "0")}`;
}

export function dayLabel(ts: number): string {  const d = new Date(ts);
  const now = new Date();
  const day = (x: Date) => `${x.getFullYear()}-${x.getMonth()}-${x.getDate()}`;
  if (day(d) === day(now)) return "今天";
  const y = new Date(now.getTime() - 86400000);
  if (day(d) === day(y)) return "昨天";
  return `${d.getMonth() + 1}月${d.getDate()}日`;
}

export function renderAtts(atts?: AttInfo[]): string {
  if (!atts || atts.length === 0) return "";
  return atts.map((a) => {
    // 图片加载：原图直载 vs 滚动懒载（设置·数据·图片加载）。
    if (a.kind === "image") return `<img class="attimg" src="${convertFileSrc(a.path)}" alt="${esc(a.name)}" loading="${load<string>("ntos_pref_autodl", "lazy") === "full" ? "eager" : "lazy"}" />`;
    if (a.kind === "video") return `<video class="attvid" src="${convertFileSrc(a.path)}" controls preload="metadata"></video>`;
    if (a.kind === "audio") return `<audio class="attaudio" src="${convertFileSrc(a.path)}" controls preload="metadata"></audio>`;
    return `<div class="attfile">${esc(a.kind === "text" ? "文" : "件")} · ${esc(a.name)}</div>`;
  }).join("");
}

export function renderThread(): void {
  const convo = currentConvo();
  const msgs = threadOf(convo);
  let html: string;
  if (msgs.length === 0) {
    // 空态极简：一行问候（建议按钮已下线，2026-09-27）。
    html = `<div class="empty-hero minimal"><div class="empty-title">有什么可以帮你？</div></div>`;
  } else {
    const parts: string[] = [];
    let lastDay = "";
    msgs.forEach((m, i) => {
      const day = dayLabel(m.ts);
      if (day !== lastDay) { lastDay = day; parts.push(`<div class="dayline"><span>${day}</span></div>`); }
      if (m.role === "sys") {
        parts.push(`<div class="sysline"><span>${esc(m.text)}</span></div>`);
      } else if (m.role === "user") {
        parts.push(`<div class="msg user" data-mi="${i}"><div class="bubble">${renderAtts(m.atts)}${esc(m.text)}<span class="ticks" title="已保存到本地">${icon("check", 11)}${icon("check", 11)}</span></div><div class="msg-meta muted">${msgTime(m.ts)}</div><div class="msg-actions"><button data-mact="copy">复制</button><button data-mact="rerun">重跑</button><button data-mact="quote">引用</button><button data-mact="edit">编辑</button><button data-mact="del">删除</button></div></div>`);
      } else if (m.agent) {
        // 标准化输出块：标签 chips（气泡顶部）→ 正文（证据行已抽为引用卡，无裸 URL）→ 执行轨迹 → 任务卡 → 时间行。
        // 精简：模型名只在顶部 chips 出现一次，meta 行只留时间（`agent.model_used` 内联前缀已删）。
        const spl = splitEvidence(m.agent.output);
        const au = `<div class="msg-meta muted">${msgTime(m.ts)}</div>`;
        const taskCard = m.taskId ? taskNodeHtml(m.taskId) : "";
        parts.push(`<div class="msg bot" data-mi="${i}"><div class="bubble">${replyTagsHtml(m.agent.labels, m.agent.model_used)}${bubbleText(spl.body)}${evidenceHtml(spl.evs)}${agentTraceHtml(m.agent.trace)}${taskCard}${au}</div><div class="msg-actions"><button data-mact="copy">复制</button><button data-mact="quote">引用</button><button data-mact="rerun">重跑</button><button data-mact="del">删除</button></div></div>`);
      } else {
        const taskCard = m.taskId ? taskNodeHtml(m.taskId) : "";
        const spl = m.node ? { body: m.text, evs: [] } : splitEvidence(m.text);
        parts.push(m.node
          ? `<div class="msg bot" data-mi="${i}">${taskNodeHtml(m.node.task)}</div>`
          : `<div class="msg bot" data-mi="${i}"><div class="bubble">${replyTagsHtml(m.labels)}${bubbleText(spl.body)}${evidenceHtml(spl.evs)}${taskCard}</div><div class="msg-meta muted">${msgTime(m.ts)}</div><div class="msg-actions"><button data-mact="copy">复制</button><button data-mact="quote">引用</button><button data-mact="rerun">重跑</button><button data-mact="del">删除</button></div></div>`);
      }
    });
    html = parts.join("");
  }
  if (shell.running && !thread.querySelector(".streaming-live")) html += `<div class="msg bot typing"><div class="bubble"><span></span><span></span><span></span></div></div>`;
  thread.innerHTML = html;
  thread.querySelectorAll<HTMLButtonElement>("[data-suggest]").forEach((b) => {
    b.onclick = () => {
      needActions()?.onSuggest(Number(b.dataset["suggest"]));
    };
  });
  thread.querySelectorAll<HTMLButtonElement>("[data-mact]").forEach((b) => {
    b.onclick = (e) => {
      e.stopPropagation();
      const row = b.closest("[data-mi]");
      const idx = Number(row?.getAttribute("data-mi"));
      needActions()?.onMsg(currentConvo(), b.dataset["mact"] ?? "", idx);
    };
  });
  thread.querySelectorAll<HTMLButtonElement>("[data-tact]").forEach((b) => {
    b.onclick = (e) => {
      e.stopPropagation();
      needActions()?.onTask(b.dataset["tact"] ?? "", b.dataset["tid"] ?? "");
    };
  });
  thread.querySelectorAll<HTMLButtonElement>("[data-cite]").forEach((b) => {
    b.onclick = (e) => {
      e.stopPropagation();
      needActions()?.onCite(b.dataset["cite"] ?? "");
    };
  });
  thread.scrollTop = thread.scrollHeight;
}

export function taskNodeHtml(taskId: string): string {
  const t = cache.tasks.find((t) => t.id === taskId);
  if (!t) return `<div class="bubble">（任务已删除）</div>`;
  const st = STATUS_CN[t.status] ?? t.status;
  return `<div class="bubble tasknode"><div class="tn-head"><span class="status-dot st-${esc(t.status)}"></span><b>${esc(t.title)}</b></div><div class="tn-meta">${esc(st)} · ${esc(t.claimed_by ? `${t.claimed_by}认领中` : "未认领")}</div><div class="tn-acts"><button class="chip-btn" data-tact="claim" data-tid="${esc(t.id)}">${t.claimed_by ? "释放" : "认领"}</button><button class="chip-btn" data-tact="cancel" data-tid="${esc(t.id)}" ${t.status !== "pending" && t.status !== "running" ? "disabled" : ""}>取消</button><button class="chip-btn" data-tact="retry" data-tid="${esc(t.id)}" ${t.status !== "failed" && t.status !== "cancelled" ? "disabled" : ""}>重跑</button></div></div>`;
}

export function pushTaskNode(convo: string, taskId: string): void {
  pushMsg(convo, "assistant", "", undefined, { task: taskId });
}

/** 任务卡并入本轮回答（标准化输出块；无回答消息时才退回独立节点卡）。 */
export function attachTaskToLast(convo: string, taskId: string): void {
  const all = allThreads();
  const msgs = all[convo] ?? [];
  for (let i = msgs.length - 1; i >= 0; i--) {
    if (msgs[i].role === "assistant") {
      msgs[i].taskId = taskId;
      all[convo] = msgs;
      saveThreads(all);
      renderThread();
      return;
    }
  }
  pushTaskNode(convo, taskId);
}

export function allThreads(): Record<string, ChatMsg[]> {
  return load<Record<string, ChatMsg[]>>(K.threads, {});
}

export function saveThreads(t: Record<string, ChatMsg[]>): void { save(K.threads, t); }

export function currentConvo(): string {
  // 空 = 新对话（未落库）：thread 显示空态，发送时再建。
  const id = load<string>(K.selConvo, "");
  if (id && cache.convos.some((c) => c.id === id)) return id;
  return "";
}

export function migrateThreads(): void {
  try {
    if (localStorage.getItem(K.threads)) return;
  } catch { return; }
  const old = load<ChatMsg[]>(K.msgs, []);
  if (old.length === 0) { saveThreads({}); return; }
  const target = cache.convos[0]?.id ?? "general";
  saveThreads({ [target]: old });
}

export function threadOf(convo: string): ChatMsg[] { return allThreads()[convo] ?? []; }

export function pushMsg(convo: string, role: ChatMsg["role"], text: string, atts?: AttInfo[], node?: { task: string }, labels?: TurnLabels): void {
  const all = allThreads();
  const msgs = all[convo] ?? [];
  msgs.push({ role, text, ts: Date.now(), atts, node, labels });
  all[convo] = msgs.slice(-300);
  saveThreads(all);
  renderThread();
}

export function pushSys(text: string, convo?: string): void { pushMsg(convo ?? currentConvo(), "sys", text); }

/** Mac 风工具图标：kind 关键字 → 极简线条 icon（16 网格单色，与图标系统同语言）。 */
function traceIcon(kind: string): string {
  const k = kind.toLowerCase();
  const name = /search|recall|grep|find|query/.test(k) ? "search"
    : /run|exec|shell|bash|command|test/.test(k) ? "play"
    : /edit|write|patch|apply|create|save/.test(k) ? "edit"
    : /browse|fetch|web|http|url|crawl/.test(k) ? "globe"
    : /skill|agent|plan|task/.test(k) ? "puzzle"
    : /read|cat|load/.test(k) ? "doc"
    : "info";
  return `<span class="trace-ic">${icon(name, 12)}</span>`;
}

export function agentTraceHtml(trace: AgentTraceItem[]): string {
  if (!trace || trace.length === 0) return "";
  const rows = trace.slice(0, 20).map((t) =>
    `<div class="audit-line"><span>${traceIcon(t.kind)}</span> <b>${esc(t.kind)}</b> · ${esc(t.detail.slice(0, 160))}</div>`).join("");
  return `<details><summary>思考过程（${trace.length}步）</summary>${rows}</details>`;
}

export function pushAgentMsg(convo: string, output: string, trace: AgentTraceItem[], modelUsed?: string, labels?: TurnLabels): void {
  const all = allThreads();
  const msgs = all[convo] ?? [];
  msgs.push({ role: "assistant", text: output, ts: Date.now(), agent: { output, trace, model_used: modelUsed, labels } });
  all[convo] = msgs.slice(-300);
  saveThreads(all);
  renderThread();
}

/** 回复标签（极简：只留一个引擎名 chip；详情进 title 悬停）。
 *  无标签回空（老消息不炸）。 */
export function replyTagsHtml(labels?: TurnLabels, fallbackModel?: string): string {
  const norm = normalizeLabels(labels, fallbackModel);
  if (!norm) return "";
  const u = norm.usage;
  const tip = `${norm.model} · ${norm.mode} · 工具${norm.tools.length} · in ${u.input_tokens}/out ${u.output_tokens}`;
  return `<div class="reply-tags" aria-label="回复标签">`
    + `<span class="reply-tag model" title="${esc(tip)}">${esc(norm.model)}</span>`
    + `</div>`;
}

/** 标签归一（后端缺省降级：model 空回 fallbackModel/echo，mode 非法回 direct）。 */
function normalizeLabels(labels?: TurnLabels, fallbackModel?: string): TurnLabels | null {
  const model = (labels?.model ?? "").trim() || (fallbackModel ?? "").trim();
  if (!labels && !model) return null;
  const mode: ReplyMode = labels?.mode === "passthrough" || labels?.mode === "fallback" || labels?.mode === "direct"
    ? labels.mode
    : "direct";
  const tools = Array.isArray(labels?.tools)
    ? labels.tools.filter((t): t is string => typeof t === "string").map((t) => t.trim()).filter((t) => t.length > 0).slice(0, 20)
    : [];
  const u = labels?.usage;
  const input = numOr(u?.input_tokens, 0);
  const out = numOr(u?.output_tokens, 0);
  const micros = numOr(u?.cost_micros, 0);
  const costUsd = typeof u?.cost_usd === "number" && Number.isFinite(u.cost_usd) && u.cost_usd > 0 ? u.cost_usd : 0;
  return {
    model: model || "echo",
    mode,
    tools,
    usage: {
      input_tokens: input,
      output_tokens: out,
      cost_usd: costUsd,
      cost_micros: micros > 0 ? micros : Math.round(costUsd * 1000000),
      measured: u?.measured === true,
    },
  };
}

function numOr(v: unknown, fb: number): number {
  return typeof v === "number" && Number.isFinite(v) && v >= 0 ? Math.floor(v) : fb;
}
