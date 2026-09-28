/**
 * NeoBot 独立前端 — 三栏布局（Rail / 会话栏 / Thread / 右 peek），浅色单主题。
 * 数据面仍是 neobot 本地 IPC（SQLite），无后端依赖。
 * store 键沿 ntos_*_v1 只增不改，parse 失败回默认绝不崩。
 */
import { convertFileSrc, Channel, invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { icon } from "./icons";
import { ntInvoke, ntInvokeStream } from "./invoke";
import { mountSidebar, openFromModel, refreshSidebarPanels, refreshSidebarTree } from "./sidebar";
import { avatarBg } from "./theme";
import { mountLiveBall, mountLiveRail, mountThumb, clearThumbs, setBallColor, setMood, gazeAt, moodForPresence } from "./moodball";
import {
  K, DEFAULT_STARTERS, STATUS_CN, load, save, esc, toast, $, thread, hero, input,
  sendBtn, shell, cache, currentView, AttRow,
  type TaskItem, type AuditItem, type CostActorRow, type ModelItem, type RoutineItem,
  type SkillItem, type PresenceItem, type MemberItem, type ConvoItem, type ModelSel,
  type ChatMsg, type AttInfo, type AgentMsg, type AgentTraceItem, type PageNode, type BoardView, type VNode,
  type TurnLabels, type ReplyMode,
  type View,
} from "./core";
import {
  bubbleText, mdLite, dayLabel, renderAtts, renderThread, taskNodeHtml, pushTaskNode,
  attachTaskToLast, allThreads, saveThreads, currentConvo, migrateThreads, threadOf, pushMsg, pushSys,
  agentTraceHtml, pushAgentMsg, setThreadActions,
} from "./thread";
import {
  card, btnRow, inspectorHtml, summaryHtml, boardNodeCount, boardPages, savePages,
  boardView, saveView, nodePos, collectNodes, renderBoardWorld, renderInfoHero,
  staleRunningIds,
} from "./board_render";
import {
  loginSites, saveLoginSites, loginState, loggedCount, validLoginUrl, siteNameOf,
  openLoginSite, verifyLoginSite, ensureLoginSite, saveLoginState, loginGoal,
  type LoginSite,
} from "./nt_login_kernel";
import { turnRanEngine, turnTaskToAttach } from "./turn_task";

// ─── 类型 ───



// ─── 极简图标注入（data-icon 占位 → 线条 SVG；badge 保留） ───
function paintIcons(): void {
  document.querySelectorAll<HTMLElement>("[data-icon]").forEach((el) => {
    const name = el.dataset["icon"] as "chat" | "clock" | "puzzle" | "chart" | "gear" | "x" | "plus" | "users";
    const badge = el.querySelector(".rail-badge");
    el.innerHTML = icon(name, 22);
    if (badge) el.appendChild(badge);
  });
  sendBtn.innerHTML = icon("arrow-up", 18);
  sendBtn.setAttribute("aria-label", "发送");
}

// ─── 新建：+ 空对话直发（发送时建归属）；群聊键进群组弹窗 ───
$("btn-new").addEventListener("click", () => {
  save(K.selConvo, "");
  input.value = "";
  persistDraft(); autogrow(); updateSendState();
  renderPanes();
  input.focus();
});
$("btn-group").addEventListener("click", () => {
  ncMode = "group";
  ($("nc-name") as HTMLInputElement).value = "";
  ($("nc-name") as HTMLInputElement).classList.remove("hidden");
  ($("nc-title") as HTMLElement).textContent = "发起群聊";
  renderNcMembers();
  ($("nc-yes") as HTMLButtonElement).style.display = "";
  $("newconvo").classList.remove("hidden");
  setTimeout(() => ($("nc-name") as HTMLInputElement).focus(), 50);
});
let pendingAtts: AttInfo[] = [];

// ─── 头像 / toast ───
// avatarBg 来自 theme.ts（主题单一源，此处无本地定义）。
function avatarHtml(name: string, size: number, presence?: string): string {
  const initial = (name.trim().charAt(0) || "?").toUpperCase();
  const dot = presence ? `<span class="dot" style="background:${presence === "active" ? "var(--avail)" : presence === "idle" ? "var(--working)" : "var(--resting)"}"></span>` : "";
  return `<div class="avatar" style="width:${size}px;height:${size}px;font-size:${Math.round(size * 0.38)}px;background:${avatarBg(name)}">${esc(initial)}${dot}</div>`;
}
/** 个人情绪球占位（渲染后 hydrateBalls 挂活球，失败留首字母渐变圆；state 供 CSS 交互态）。 */
function ballAvatar(name: string, size: number, presence?: string, state = ""): string {
  const initial = (name.trim().charAt(0) || "?").toUpperCase();
  const mood = presence ? moodForPresence(presence) : "idle";
  return `<div class="avatar ballmount" data-ballmood="${mood}" data-presence="${esc(presence ?? "")}" data-state="${state}" style="width:${size}px;height:${size}px;font-size:${Math.round(size * 0.38)}px;background:${avatarBg(name)}">${esc(initial)}</div>`;
}
/** 挂载所有占位球（含在线态表情 + 在线点；逐个 try，坏的不挡好）。 */
function hydrateBalls(root: ParentNode = document): void {
  root.querySelectorAll<HTMLElement>(".ballmount").forEach((el) => {
    if (el.querySelector("svg")) return;
    const presence = el.dataset["presence"] ?? "";
    el.innerHTML = "";
    const mood = (el.dataset["ballmood"] as "idle" | "sleep") || "idle";
    if (!mountThumb(el, mood)) {
      el.textContent = "?";
      return;
    }
    if (presence) {
      const dot = document.createElement("span");
      dot.className = "dot";
      dot.style.background = presence === "active" ? "var(--avail)" : presence === "idle" ? "var(--working)" : "var(--resting)";
      el.appendChild(dot);
    }
  });
}
/** 轻 markdown（顺序：转义 → 代码块占位 → 行内码 → 粗体 → 换行）。 */
/** 线程任务节点卡（画板任务信息统一整合进对话流；数据取实时 cache）。 */
async function onTaskNodeAct(act: string, taskId: string): Promise<void> {
  const me = load<string>(K.me, "neo");
  const t = cache.tasks.find((t) => t.id === taskId);
  try {
    if (act === "claim") {
      if (!t) return;
      if (t.claimed_by) await ntInvoke("neobot_task_release", { taskId, actorId: me });
      else await ntInvoke("neobot_task_claim", { taskId, actorId: me });
      pushSys(t.claimed_by ? `${me} 释放了「${t.title}」` : `${me} 认领了「${t.title}」`);
    } else if (act === "cancel") {
      await ntInvoke("neobot_task_cancel", { taskId });
      pushSys(`取消了「${t?.title ?? ""}」`);
    } else if (act === "retry") {
      await ntInvoke("neobot_task_retry", { taskId });
      pushSys(`重跑「${t?.title ?? ""}」`);
    }
    toast("已更新");
  } catch (e) { toast(String(e).slice(0, 90), "err"); }
  void refreshAll();
}
/** 消息悬停动作（仅改本地转录，不重跑任务）。 */
async function onMsgAct(convo: string, act: string, idx: number): Promise<void> {
  const msgs = threadOf(convo);
  const m = msgs[idx];
  if (!m || m.role === "sys") return;
  if (act === "copy") {
    // agent 消息正文与转录 text 同值，优先取结构化 output（证据行未抽取前的原文）。
    const body = m.agent?.output ?? m.text;
    try { await navigator.clipboard.writeText(body); toast("已复制"); }
    catch { toast("复制失败", "err"); }
  } else if (act === "quote") {
    // 引用：原文进作曲区（> 引用块），光标就绪。
    const src = (m.text || "").trim().split("\n").slice(0, 6).join("\n");
    input.value = (input.value ? input.value.replace(/\s+$/, "") + "\n" : "") + `> ${src.replace(/\n/g, "\n> ")}\n`;
    persistDraft(); autogrow(); updateSendState();
    input.focus();
    toast("已引用到输入框");
  } else if (act === "del") {
    askConfirm("删除这条本地消息？（任务不受影响）", () => {
      const all = allThreads();
      const cur = all[convo] ?? [];
      cur.splice(idx, 1);
      all[convo] = cur;
      saveThreads(all);
      renderThread();
      toast("已删除");
    });
  } else if (act === "rerun" && m.role === "assistant") {
    // 重跑语义：删掉本轮旧答，用其上一条用户原文再跑一轮，不双推用户消息。
    let qi = idx - 1;
    while (qi >= 0 && msgs[qi].role !== "user") qi--;
    const q = qi >= 0 ? msgs[qi].text : "";
    if (!q.trim()) { toast("找不到上一条用户消息", "err"); return; }
    if (shell.running) { toast("本轮还没跑完", "err"); return; }
    const all = allThreads();
    const cur = all[convo] ?? [];
    cur.splice(idx, 1);
    all[convo] = cur;
    saveThreads(all);
    renderThread();
    const me = load<string>(K.me, "neo");
    await runStreamTurn(convo, q.slice(0, 24), q, me, modelSel());
  } else if (act === "rerun" && m.role === "user") {
    // 用户消息重跑：复用同一 run 通道（`runStreamTurn` → `neobot_run_stream`），
    // 不双推用户消息（本行已在流内），直接以本条原文再跑一轮。
    const q = m.text;
    if (!q.trim()) { toast("空消息无法重跑", "err"); return; }
    if (shell.running) { toast("本轮还没跑完", "err"); return; }
    const me = load<string>(K.me, "neo");
    await runStreamTurn(convo, q.trim().slice(0, 24) || "重跑", q, me, modelSel());
  } else if (act === "edit" && m.role === "user") {    const row = thread.querySelector(`[data-mi="${idx}"] .bubble`);
    if (!row || row.querySelector("textarea")) return;
    const orig = m.text;
    row.innerHTML = `<textarea rows="3">${esc(orig)}</textarea><div style="display:flex;gap:6px;justify-content:flex-end;margin-top:6px"><button class="chip-btn" data-e="save">保存</button><button class="chip-btn" data-e="cancel">取消</button></div>`;
    const ta = row.querySelector("textarea");
    (ta as HTMLTextAreaElement | null)?.focus();
    row.querySelector('[data-e="save"]')?.addEventListener("click", () => {
      const v = ((ta as HTMLTextAreaElement | null)?.value ?? "").trim();
      if (!v) return;
      const all = allThreads();
      const cur = all[convo] ?? [];
      if (cur[idx]) { cur[idx].text = v; all[convo] = cur; saveThreads(all); }
      renderThread();
      toast("已修改（仅本地转录）");
    });
    row.querySelector('[data-e="cancel"]')?.addEventListener("click", () => renderThread());
  }
}
// ─── 按会话线程（IM 语义：一切转录按会话存；旧全局转录迁入 general） ───
/** 发送前确保会话：有选中即用；否则取默认个人对话（没有则建“我的私聊”）。 */
async function ensureSendConvo(): Promise<string> {
  const cur = currentConvo();
  if (cur) return cur;
  const me = load<string>(K.me, "neo");
  const id = await ntInvoke<string>("neobot_convo_ensure_default", { me });
  save(K.selConvo, id);
  return id;
}
/** 过程系统行（任务事件入流：认领/可见/取消/重跑/重命名/删除/接管）。 */

// ─── 作曲区（textarea 自增高；Enter 发送，Shift+Enter 换行） ───
function autogrow(): void {
  input.style.height = "auto";
  input.style.height = Math.min(input.scrollHeight, 120) + "px";
}
function allDrafts(): Record<string, string> {
  // 存量单字符串草稿（旧版）视为未落库新对话的草稿，读一次即迁走。
  const raw = load<unknown>(K.drafts, {});
  if (typeof raw === "string") return raw ? { "": raw } : {};
  if (raw && typeof raw === "object") return raw as Record<string, string>;
  return {};
}
function restoreDraft(): void {
  input.value = allDrafts()[currentConvo()] ?? "";
  autogrow();
}
function persistDraft(): void {
  const all = allDrafts();
  all[currentConvo()] = input.value;
  save(K.drafts, all);
}
/** 切会话统一入口：存旧（input 事件已实时存）、标已读、取新草稿、渲染。 */
function selectConvo(id: string): void {
  save(K.selConvo, id);
  if (id) void ntInvoke("neobot_convo_mark_read", { id }).catch(() => {});
  restoreDraft();
  renderThread();
  renderPanes();
}
// token 估算 + 发送键状态（对齐主 App InputArea：空禁发、估算、生成中占位）。
function updateSendState(): void {
  const has = input.value.trim().length > 0 || pendingAtts.length > 0;
  const est = $("tok-est");
  est.classList.toggle("hidden", !has);
  if (has) est.textContent = `≈${Math.max(1, Math.round(input.value.length / 4))} tok`;
  (sendBtn as HTMLButtonElement).toggleAttribute("disabled", !has && !shell.running);
  input.placeholder = shell.running ? "生成中仍可输入，下一条稍后发送…" : "输入消息…（Enter 发送，Shift+Enter 换行）";
}
input.addEventListener("input", () => { persistDraft(); autogrow(); updateSendState(); });
function setRunning(on: boolean): void {
  shell.running = on;
  // ↑ 发送 / ■ 停止（图标即状态，无需读字）。
  sendBtn.innerHTML = icon(on ? "stop" : "arrow-up", 18);
  sendBtn.classList.toggle("stop", on);
  sendBtn.title = on ? "停止（草稿保留）" : "发送";
  updateSendState();
  renderThread();
}
sendBtn.onclick = onSend;
input.addEventListener("keydown", (e) => {
  const cmdEnter = load<string>(PREF_ENTER, "enter") === "cmdenter";
  const want = e.key === "Enter" && (cmdEnter ? (e.metaKey || e.ctrlKey) : !e.shiftKey);
  if (want) { e.preventDefault(); void onSend(); }
});

// ─── 附件（作曲区待发 + 会话落库；图直显，文件/视频芯片） ───
$("btn-attach").addEventListener("click", () => void onAttach());
async function onAttach(): Promise<void> {
  const convo = currentConvo();
  let picked: string[] | null = null;
  try {
    const sel = await openDialog({ multiple: true, title: "添加附件" });
    if (!sel) return;
    picked = (Array.isArray(sel) ? sel : [sel]).filter((p): p is string => typeof p === "string");
  } catch (e) { toast(`选择失败：${String(e).slice(0, 60)}`, "err"); return; }
  for (const p of picked.slice(0, 5)) {
    try {
      const id = await ntInvoke<string>("neobot_attach_add", { convoId: convo, srcPath: p });
      const rows = await ntInvoke<AttRow[]>("neobot_attachments", { convoId: convo });
      const row = rows.find((r) => r.id === id);
      if (row) pendingAtts.push({ id: row.id, kind: row.kind, name: row.name, path: row.path });
    } catch (e) { toast(`附件失败：${String(e).slice(0, 60)}`, "err"); }
  }
  renderAttChips();
  updateSendState();
}
function renderAttChips(): void {
  const box = $("attchips");
  box.classList.toggle("hidden", pendingAtts.length === 0);
  box.innerHTML = "";
  for (const a of pendingAtts) {
    const chip = document.createElement("span");
    chip.className = "attchip";
    const thumb = a.kind === "image" ? `<img src="${convertFileSrc(a.path)}" alt="" />` : "";
    chip.innerHTML = `${thumb}<span>${esc(a.name)}</span><button data-unatt="${esc(a.id)}" title="移除">✕</button>`;
    box.appendChild(chip);
  }
  box.querySelectorAll<HTMLButtonElement>("[data-unatt]").forEach((b) => {
    b.onclick = async () => {
      const id = b.dataset["unatt"] ?? "";
      try { await ntInvoke("neobot_attach_remove", { id }); } catch { /* 行没了就行 */ }
      pendingAtts = pendingAtts.filter((a) => a.id !== id);
      renderAttChips();
      updateSendState();
    };
  });
}

// ─── 模型切换器（池子聚合；无回显选项——回显仅后端保底，前端不暴露） ───
function modelSel(): ModelSel | null { return load<ModelSel | null>(K.model, null); }
function updateModelBtn(): void {
  // 默认模型：neotrix 晶体核心；从未选过或选过回显（存 null）一律跟晶体（回显不进切换器）。
  let sel = modelSel();
  if (!sel) {
    sel = { provider: "neotrix", model: "neotrix-crystal" };
    save(K.model, sel);
  }
  $("btn-model").textContent = sel.model;
  $("btn-model").title = `${sel.provider} / ${sel.model}（点击切换）`;
}
/** 晶体面板（配对且在线时只暴露一个晶体模型；返回 true 表示已处理）。 */
async function paintCrystalPanel(
  panel: HTMLElement,
  sel: ModelSel | null,
): Promise<boolean> {
  let core: { paired: boolean; online: boolean; model: string; detail: string };
  try {
    core = await ntInvoke("neobot_core_status") as { paired: boolean; online: boolean; model: string; detail: string };
  } catch {
    return false;
  }
  if (!core.paired || !core.online) return false;
  const model = core.model;
  const onCrystal = sel !== null && sel.provider === "neotrix" && sel.model === model;
  panel.innerHTML =
    '<button class="mrow' + (onCrystal ? " on" : "") + '" data-mpick="neotrix" data-mid="' + esc(model) + '">'
    + '<span class="mid">' + esc(model) + "</span>"
    + '<span class="mown">neotrix crystal</span></button>';
  // 第三方池子模型（配对后也不隐藏；现拉一次，不依赖 cache 时机）。
  try {
    const pool = await ntInvoke<ModelItem[]>("neobot_models") as ModelItem[];
    const seen = new Set<string>();
    let phtml = "";
    for (const m of pool) {
      const key = m.source + "/" + m.id;
      if (seen.has(key)) continue;
      seen.add(key);
      const on = sel?.provider === m.source && sel?.model === m.id;
      phtml += `<button class="mrow${on ? " on" : ""}" data-mpick="${esc(m.source)}" data-mid="${esc(m.id)}"><span class="mid">${esc(m.id)}</span><span class="mown">${esc(m.source)} · ${esc(m.owner)}</span></button>`;
    }
    if (phtml) panel.innerHTML += `<div class="mp-group">第三方池子</div>` + phtml;
  } catch { /* 池子拉失败不挡晶体行 */ }
  panel.classList.remove("hidden");
  panel.querySelectorAll<HTMLButtonElement>("[data-mpick]").forEach((b) => {
    b.onclick = () => {
      const p = b.dataset["mpick"] ?? "";
      if (p === "neotrix") save(K.model, { provider: "neotrix", model });
      else save(K.model, { provider: p, model: b.dataset["mid"] ?? "" });
      panel.classList.add("hidden");
      updateModelBtn();
      toast(p === "neotrix" ? "crystal" : `已切 ${p}/${b.dataset["mid"]}`);
    };
  });
  return true;
}
$("btn-model").addEventListener("click", async () => {
  const panel = $("modelpanel");
  if (!panel.classList.contains("hidden")) { panel.classList.add("hidden"); return; }
  const sel = modelSel();
  // 晶体模式：只暴露一个 neotrix 晶体模型（池选型下沉到 neotrix 内部自主）。
  // 未配对时回落旧面板（池子 + 免费发现 + 配对入口）。
  if (await paintCrystalPanel(panel, sel)) return;
  const byProv = new Map<string, ModelItem[]>();
  for (const m of cache.models) {
    const list = byProv.get(m.source) ?? [];
    list.push(m);
    byProv.set(m.source, list);
  }
  let html = "";
  for (const [prov, models] of byProv) {
    html += `<div class="mp-group">${esc(prov)}</div>`;
    for (const m of models.slice(0, 12)) {
      const on = sel?.provider === prov && sel?.model === m.id;
      html += `<button class="mrow${on ? " on" : ""}" data-mpick="${esc(prov)}" data-mid="${esc(m.id)}"><span class="mid">${esc(m.id)}</span><span class="mown">${esc(m.owner)}</span></button>`;
    }
  }
  if (cache.models.length === 0) html += `<div class="empty-note">池子为空 — 下方免费发现可一键嵌入灵魂</div>`;
  panel.innerHTML = html;
  panel.classList.remove("hidden");
  panel.querySelectorAll<HTMLButtonElement>("[data-mpick]").forEach((b) => {
    b.onclick = () => {
      const p = b.dataset["mpick"] ?? "";
      save(K.model, { provider: p, model: b.dataset["mid"] ?? "" });
      panel.classList.add("hidden");
      updateModelBtn();
      toast(`已切 ${p}/${b.dataset["mid"]}`);
    };
  });
  // 免费发现（异步追加；缺 key 只展示、点则指路，配对成功即选中灵魂模型）。
  void (async () => {
    try {
      const free = await ntInvoke<Array<{ model_id: string; display: string; provider: string; key_env: string; key_ok: boolean; via: string }>>("neobot_core_free");
      if (panel.classList.contains("hidden") || free.length === 0) return;
      let fhtml = `<div class="mp-group">免费发现（配对即嵌入灵魂）</div>`;
      for (const f of free.slice(0, 15)) {
        const tag = f.via === "cli" ? "免key" : f.key_ok ? "key✓" : "缺 " + f.key_env;
        fhtml += `<button class="mrow" data-fpair="${esc(f.model_id)}"><span class="mid">${esc(f.model_id)}</span><span class="mown">${esc(f.provider)} · ${esc(tag)}</span></button>`;
      }
      panel.insertAdjacentHTML("beforeend", fhtml);
      panel.querySelectorAll<HTMLButtonElement>("[data-fpair]").forEach((b) => {
        b.onclick = () => {
          const mid = b.dataset["fpair"] ?? "";
          void (async () => {
            try {
              const msg = await ntInvoke<string>("neobot_core_pair_free", { modelId: mid });
              save(K.model, { provider: "neotrix", model: mid });
              panel.classList.add("hidden");
              updateModelBtn();
              toast(msg);
              void refreshAll();
            } catch (e) { toast(`配对失败：${String(e).slice(0, 90)}`, "err"); }
          })();
        };
      });
    } catch { /* 发现失败不挡面板 */ }
  })();
});
document.addEventListener("click", (e) => {
  const panel = $("modelpanel");
  if (!panel.classList.contains("hidden") && !(e.target as HTMLElement).closest?.("#modelpanel,#btn-model")) {
    panel.classList.add("hidden");
  }
});

// ─── Agent 运行最小 UI（⚡ 键在模型切换旁；当前输入为 goal，context 传空；不动发送流程） ───
interface AgentRunResult { status: string; output: string; trace: Array<{ kind: string; detail: string }>; model_used?: string; mode?: string; tools?: string[]; usage?: { prompt_tokens?: number; completion_tokens?: number; cost_usd?: number } | null; }
/** 跑轮返回（新口径 `{status, labels, task_id, cancelled}`；旧串口径兼容，缺标签即无 chips 不炸）。 */
interface RunResult { status: string; labels?: TurnLabels; task_id?: string; cancelled?: boolean; }
/** agent 透传结果 → 气泡标签（tools 空即由 trace 派生；用量缺省未计量）。 */
function labelsFromAgent(r: AgentRunResult): TurnLabels | undefined {
  const tools = Array.isArray(r.tools) && r.tools.length > 0
    ? r.tools.filter((t): t is string => typeof t === "string").map((t) => t.trim()).filter((t) => t.length > 0).slice(0, 20)
    : (Array.isArray(r.trace) ? [...new Set(r.trace.map((t) => String(t.kind ?? "").trim()).filter((k) => k.length > 0 && k !== "reply" && k !== "set_turn_status"))].slice(0, 20) : []);
  const mode: ReplyMode = r.mode === "direct" || r.mode === "fallback" || r.mode === "passthrough" ? r.mode : "passthrough";
  const model = typeof r.model_used === "string" && r.model_used.trim() ? r.model_used.trim() : "neotrix-crystal";
  const input = numOr(r.usage?.prompt_tokens, 0);
  const out = numOr(r.usage?.completion_tokens, 0);
  const costUsd = typeof r.usage?.cost_usd === "number" && Number.isFinite(r.usage.cost_usd) && r.usage.cost_usd > 0 ? r.usage.cost_usd : 0;
  return { model, mode, tools, usage: { input_tokens: input, output_tokens: out, cost_usd: costUsd, cost_micros: Math.round(costUsd * 1000000), measured: costUsd > 0 } };
}
/** 跑轮返回归一（对象新口径 / 字符串旧口径；失败回状态文本无标签）。 */
function asRunResult(v: unknown, fb: string): RunResult {
  if (typeof v === "string") return { status: v };
  if (v && typeof v === "object") {
    const o = v as { status?: unknown; labels?: unknown; task_id?: unknown; cancelled?: unknown };
    const status = typeof o.status === "string" && o.status ? o.status : fb;
    const labels = isLabels(o.labels) ? o.labels : undefined;
    // task_id 只有「非空串」才算数（空串 = 后端没认出本轮任务，如实当缺失）。
    const taskId = typeof o.task_id === "string" && o.task_id.trim() ? o.task_id.trim() : undefined;
    const cancelled = o.cancelled === true;
    return { status, labels, task_id: taskId, cancelled };
  }
  return { status: fb };
}
function isLabels(v: unknown): v is TurnLabels {
  if (!v || typeof v !== "object") return false;
  const o = v as Record<string, unknown>;
  return typeof o["model"] === "string" && typeof o["mode"] === "string" && Array.isArray(o["tools"]) && !!o["usage"];
}
function numOr(v: unknown, fb: number): number {
  return typeof v === "number" && Number.isFinite(v) && v >= 0 ? Math.floor(v) : fb;
}
let agentRunning = false;
async function onAgentRun(): Promise<void> {
  if (agentRunning) return;
  const goal = input.value.trim();
  if (!goal) { toast("先输入要让 Agent 做的目标", "err"); return; }
  let convo: string;
  try {
    convo = await ensureSendConvo();
  } catch (e) { toast(`建会话失败：${String(e).slice(0, 60)}`, "err"); return; }
  agentRunning = true;
  const btn = document.getElementById("btn-agent") as HTMLButtonElement | null;
  btn?.toggleAttribute("disabled", true);
  try {
    const r = await ntInvoke<AgentRunResult>("neobot_agent_run", { goal, context: "" });
    const trace: AgentTraceItem[] = Array.isArray(r.trace)
      ? r.trace.slice(0, 20).map((t) => ({ kind: String(t.kind ?? ""), detail: String(t.detail ?? "") }))
      : [];
    const out = `${r.output ?? ""}${r.status && r.status !== "done" ? `\n（status=${r.status}）` : ""}`.trim() || `（空输出，status=${r.status}）`;
    pushAgentMsg(convo, out, trace, typeof r.model_used === "string" ? r.model_used : undefined, labelsFromAgent(r));
    if (convo) void ntInvoke("neobot_convo_mark_read", { id: convo }).catch(() => {});
    toast("Agent 本轮完成");
  } catch (e) {
    toast(`Agent 运行失败：${String(e).slice(0, 90)}`, "err");
  } finally {
    agentRunning = false;
    btn?.toggleAttribute("disabled", false);
    void refreshAll();
  }
}

// ─── 对话 URL 闭环（网页任务最优解，中间过程优化点见注） ───
// 优化决策：
// ① 意图门（关键词/裸 URL）而非全量 URL 触发——纯提及只上板不烧 agent；
// ② 上板去重（同 URL 常驻一块板，多次帖不叠罗汉）；
// ③ 转流免双跑（agent 流即答案，不再走正常发送）；
// ④ verdict 首行契约（`已登录：` 前缀）→ 状态表可机读，看板点同步变 🟢/🔴；
// ⑤ 后继任务拼进同一 goal（一次 agent 跑完检查+办事，不断链）。
const URL_RE = /https?:\/\/[^\s)>"'\]]+/gi;
const LOGIN_INTENT = /登录|登陆|验证|打开|上板|看看|检查|逛|买|订|查/;
function boardLoginUrl(url: string): boolean {
  const pages = boardPages();
  if (pages.some((p) => p.url === url)) return false;
  const n = pages.length;
  pages.push({ id: `p${Date.now()}`, url, x: 120 + n * 36, y: 120 + n * 36, w: 360, h: 420 });
  savePages(pages);
  renderInfo();
  return true;
}
/** 返回 true = 已转流（调用方直接 return，免双跑）。 */
function onDialogUrl(convo: string, text: string): boolean {
  const urls = [...new Set((text.match(URL_RE) ?? [])
    .map((u) => u.replace(/[.,;!?]+$/, "")))].filter((u) => validLoginUrl(u));
  if (urls.length === 0) return false;
  const url = urls[0];
  const boarded = boardLoginUrl(url);
  const rest = text.replace(URL_RE, "").trim();
  if (!LOGIN_INTENT.test(text) && rest.length > 0) {
    if (boarded) toast("网页已上板（看板 +网页）");
    return false;
  }
  pushSys(`网页任务：已上板${boarded ? "" : "（已在板上）"}，转后端自动解析`);
  void onDialogUrlVerify(convo, url, rest);
  return true;
}
let urlVerifying = false;
async function onDialogUrlVerify(convo: string, url: string, followup: string): Promise<void> {
  if (urlVerifying) { toast("网页任务进行中，稍等", "err"); return; }
  urlVerifying = true;
  setRunning(true); shell.stopRequested = false;
  setMood("working");
  const site = ensureLoginSite(url);
  // 先转 🌀（失败由核内 finally 回 unknown，不卡死）。
  saveLoginState({ site_id: site.id, status: "checking", evidence: "", ts: Date.now() });
  renderInfo();
  try {
    const goal = followup
      ? `网页任务两步走：① 先按登录态模板检查 ${url}（首行只答：已登录：<证据> 或 未登录：<所见>）；② 然后完成：${followup}。两步结果都给出。`
      : loginGoal(url);
    const r = await ntInvoke<AgentRunResult>("neobot_agent_run", { goal, context: "" });
    const out = `${r.output ?? ""}`.trim() || `（空输出，status=${r.status}）`;
    const trace: AgentTraceItem[] = Array.isArray(r.trace)
      ? r.trace.slice(0, 20).map((t) => ({ kind: String(t.kind ?? ""), detail: String(t.detail ?? "") }))
      : [];
    pushAgentMsg(convo, out, trace, typeof r.model_used === "string" ? r.model_used : undefined, labelsFromAgent(r));
    saveLoginState({
      site_id: site.id,
      status: /^\s*已登录/.test(out) ? "logged" : "unlogged",
      evidence: out.slice(0, 120),
      ts: Date.now(),
    });
    setMood("done");
    if (convo) void ntInvoke("neobot_convo_mark_read", { id: convo }).catch(() => {});
    toast("网页任务完成");
  } catch (e) {
    pushMsg(convo, "assistant", `网页任务失败：${String(e).slice(0, 120)}`);
    setMood("error");
    toast("网页任务失败", "err");
  } finally {
    urlVerifying = false;
    setRunning(false);
    renderInfo();
    void refreshAll();
  }
}

async function onSend(): Promise<void> {
  // 停止键：真的去翻那一轮的 `StopToken`（此前只置 `shell.stopRequested` 让渲染
  // 跳过增量 —— 那一轮其实照跑照烧，见 `requestStop` 的注释）。
  if (shell.running) { void requestStop(); return; }
  const text = input.value.trim();
  const atts = pendingAtts.slice();
  if (!text && atts.length === 0) return;
  // 空对话直接发，发送时建归属（默认个人对话）。
  let convo: string;
  try {
    convo = await ensureSendConvo();
  } catch (e) { toast(`建会话失败：${String(e).slice(0, 60)}`, "err"); return; }
  const title = (text || atts[0]?.name || "附件").slice(0, 24);
  const me = load<string>(K.me, "neo");
  const sel = modelSel();
  const engineText = atts.length > 0 && text
    ? `${text}\n[附件：${atts.map((a) => a.name).join("、")}]`
    : text || `[附件：${atts.map((a) => a.name).join("、")}]`;
  pushMsg(convo, "user", text || `(附件 ${atts.length} 个)`, atts);
  input.value = ""; pendingAtts = []; renderAttChips(); persistDraft();
  { const all = allDrafts(); delete all[""]; save(K.drafts, all); }
  autogrow();
  // 对话 URL 闭环（网页任务最优解）：帖 URL 即上板 + 后端自动解析。
  // 有意图/裸 URL → 转 agent 流（免双跑）；纯提及 → 静默上板后走正常发送。
  if (text && onDialogUrl(convo, text)) return;
  await runStreamTurn(convo, title, engineText, me, sel);
}

/** 队内会话（group）才展示过程件（任务卡/耗时行）；个人 DM 只留最终答案 + 引用下标。 */
function isTeamConvo(convo: string): boolean {
  return cache.convos.some((c) => c.id === convo && c.kind === "group");
}

/**
 * 跑轮**起点**的任务 id 快照（一次便宜读）。
 *
 * 存在的理由：`neobot_tasks` 只给全局最近 50 条、且载荷里**没有** `created_at`
 * （`nt_cmd_tasks.rs:10` / `neobotTaskItem` 字段表），后端也没把本轮任务的
 * id 回给前端（`NeobotRunResult` 只有 status + labels，`nt_commands.rs:221`）。
 * 故「本轮新建了谁」只能靠**跑轮前后 id 集合之差**来认 —— 认不出就不挂卡。
 * 读失败记 null（fail-closed），绝不把「读不到」当成「没有任务」。
 */
async function snapshotTaskIds(): Promise<Set<string> | null> {
  try {
    return new Set((await ntInvoke<TaskItem[]>("neobot_tasks")).map((t) => t.id));
  } catch {
    return null;
  }
}

/**
 * 停止请求的状态机。
 *
 * `none` → `requested`（已乐观收起流式，cancel 在飞）→ 落到**两个终态之一**：
 * `accepted`（后端说令牌已翻）或 `failed`（后端说「没有可停的轮次」）。
 *
 * 为什么必须有两个终态而不能只有一个「已停」：**「以为停了其实没停」就是
 * 假回执**。用户会以为模型不听话了，而它还在烧 token —— 这比明说「停不了」
 * 坏得多。所以取消失败必须**可见**（进对话流，见 `requestStop`）。
 */
type StopState = "none" | "requested" | "accepted" | "failed";

/** 正在跑的那一轮（前端侧的账，与后端那张令牌登记表对同一件事）。 */
interface ActiveRun {
  /** 本轮所属会话 —— 取消请求的键（后端登记表按 `convo_id` 索引）。 */
  readonly convo: string;
  stop: StopState;
  /**
   * 那次 cancel 的 promise。**收尾前必须 await 它**：停止请求完全可能**晚于**
   * 跑轮返回（IPC 各自独立），不等就收尾等于「在一个还没到的取消结果之前
   * 先宣布结局」—— 那正是本文件要根除的假回执。
   */
  stopPromise: Promise<void> | null;
  /** 跑轮还在飞吗？取消失败时据此决定要不要把运行态还回去。 */
  inFlight: boolean;
  /** 回滚用：把流式气泡重新画出来（取消失败 ⇒ 「收起」是错的）。 */
  resume: (() => void) | null;
}

let activeRun: ActiveRun | null = null;

/**
 * 读这一轮的停止状态。
 *
 * 为什么要绕一道函数：TS 会把 `run.stop = "none"` 之后的 `run.stop` 窄化成
 * 字面量 `"none"`，于是「在途的 cancel 回调有没有已经把它改成 accepted」这条
 * 真问题被编译器判成不可能（TS2367）。它**确实**可能 —— 那次赋值发生在另一
 * 个执行流（await 之后的 IPC 回调）里。走函数读一次就拿回了联合类型。
 */
function stopOf(run: ActiveRun): StopState {
  return run.stop;
}

/**
 * 调停止键的后端命令。
 *
 * 曾为绕过文件所有权在这里就地声明一份参数形状（裸 `invoke`）——那是**临时的旁路**，
 * 现已并入 `invoke.ts` 的类型表，键名由**类型表 + `nt_ipc_keys.py` 两道**守。
 * 留一份旁路等于给类型安全开一个只属于这个命令的口子，所以并掉了。
 */
async function invokeRunCancel(convoId: string): Promise<unknown> {
  return ntInvoke("neobot_run_cancel", { convoId });
}

/** 停止键：**真的**去停那一轮。 */
async function requestStop(): Promise<void> {
  const run = activeRun;
  if (!run) {
    // 不可达（`shell.running` 与 `activeRun` 同生共死），但真发生了也不能
    // 留一个「以为停了」的界面给用户。
    setRunning(false);
    persistDraft();
    toast("没有正在跑的轮次可停", "err");
    return;
  }
  if (run.stop !== "none") return;
  // ① 乐观收起流式（保持即时手感）。**这是猜测，不是事实** —— 所以结果必须
  //    如实回填：② 失败就撤销它，③ 收尾时以跑轮的真实终态为准。
  shell.stopRequested = true;
  run.stop = "requested";
  setRunning(false);
  persistDraft();
  const convo = run.convo;
  run.stopPromise = (async () => {
    try {
      await invokeRunCancel(convo);
      run.stop = "accepted";
      setMood("idle");
    } catch (e) {
      run.stop = "failed";
      const why = String(e).replace(/^Error:\s*/, "").trim().slice(0, 160) || "后端没有给出原因";
      // ② 取消失败 = 「停掉了」不是事实 ⇒ 撤销乐观状态。
      //    这一轮要么还在跑、要么已经跑完（结果正在路上），两种都不该继续
      //    装作「已经停了」：还在跑就把运行态与流式气泡**还回去**。
      if (run.inFlight) {
        shell.stopRequested = false;
        setRunning(true);
        run.resume?.();
      }
      // ③ 可见反馈：进对话流（`toast` 会消失，而这一行会留在记录里）。
      // 显式带 convo：失败提示属于**那一轮所在的会话**，而用户此刻可能已经
      // 切到别的会话去了（`pushSys` 不带参数就落到当前会话）。
      pushSys(`停止失败，这一轮没有停住：${why}`, run.convo);
      toast("停止失败", "err");
    }
  })();
}

/** 流式跑轮（onSend 与消息重跑共用；调用方负责已 push 用户消息，免双推）。 */
async function runStreamTurn(convo: string, title: string, engineText: string, me: string, sel: ModelSel | null): Promise<void> {
  setRunning(true); shell.stopRequested = false;
  setMood("working");
  // 流式气泡：增量文本 + 工具步骤实时入流（工作流进对话流）。
  type StreamEv =
    | { kind: "delta"; text: string }
    | { kind: "step"; tool: string; ok: boolean; output: string }
    | { kind: "done"; status: string };
  let streamRaw = "";
  const live = document.createElement("div");
  live.className = "msg bot streaming-live";
  live.innerHTML = `<div class="bubble"><span class="stream-text"></span><div class="stream-steps"></div><div class="stream-meta muted"></div></div>`;
  thread.appendChild(live);
  thread.scrollTop = thread.scrollHeight;
  // Codex 式时间线：耗时 + 步数 + 当前工具（一眼知卡在哪）。
  const t0 = Date.now();
  let stepCount = 0;
  let lastTool = "思考";
  const elapsed = () => ((Date.now() - t0) / 1000).toFixed(0) + "s";
  const paintMeta = () => {
    const el = live.querySelector(".stream-meta");
    if (el) el.textContent = `工作中 · ${elapsed()} · ${stepCount}步 · ${lastTool}`;
  };
  const paintStream = () => {
    const el = live.querySelector(".stream-text");
    if (el) el.innerHTML = `${esc(streamRaw)}<span class="caret">▍</span>`;
    paintMeta();
    thread.scrollTop = thread.scrollHeight;
  };
  const channel = new Channel<StreamEv>();
  channel.onmessage = (ev) => {
    // 停止请求在飞：**画**停掉，但**收**不停。
    // 为什么还收：取消可能失败（那一轮其实还在跑或已经跑完），那时要把界面
    // 还原成「还在跑」。若这期间把增量丢了，还原出来的就是一份**缺页**的
    // 回复 —— 那比不还原更糟（用户看到的是模型没说完，而不是「我按了停」）。
    // 这期间**不做**的：画 step 行、跳转侧边栏（用户已经喊停了，别再替他做动作）。
    if (shell.stopRequested) {
      if (ev.kind === "delta") streamRaw += ev.text;
      else if (ev.kind === "step") { stepCount++; lastTool = ev.tool; }
      return;
    }
    if (ev.kind === "delta") {
      streamRaw += ev.text;
      paintStream();
    } else if (ev.kind === "step") {
      stepCount++;
      lastTool = ev.tool;
      // 模型的「打开侧边栏」提议：Rust 侧只成文（输出是一行 JSON），
      // 真正跳转在这里 —— 界面是唯一持有界面状态的一方。
      if (ev.tool === "sidebar_open" && ev.ok) {
        try {
          const at = JSON.parse(ev.output) as { topic: string; path: string };
          if (at.topic) void openFromModel(at.topic, at.path ?? "");
        } catch {
          // 解析失败不当错误：模型只是提议了个看不懂的落点，跳过即可。
        }
      }
      const box = live.querySelector(".stream-steps");
      if (box) {
        const row = document.createElement("div");
        row.className = "step-line";
        row.innerHTML = `<span class="${ev.ok ? "dec-allow" : "dec-deny"}">●</span> ${esc(ev.tool)} <span class="muted">+${elapsed()} · ${esc(ev.output.slice(0, 80))}</span>`;
        box.appendChild(row);
        paintMeta();
        thread.scrollTop = thread.scrollHeight;
      }
    }
  };
  const finishStream = (failed: boolean) => {
    live.remove();
    if (failed) renderThread();
  };
  // 起点快照：必须在跑轮**之前**取（跑轮会新建本轮任务）。收尾靠它认本轮任务。
  // （有了后端给的 `task_id` 之后它降级成**回落**判据，但快照仍要留着：后端
  //  认不出时（并发的别的跑轮让「恰好一个」不成立）只有它能救。）
  const taskIdsBefore = await snapshotTaskIds();
  const run: ActiveRun = { convo, stop: "none", stopPromise: null, inFlight: true, resume: null };
  activeRun = run;
  run.resume = () => paintStream();
  try {
    const raw = await ntInvokeStream<RunResult | string>("neobot_run_stream", {
      title, text: engineText, actorName: me, convoId: convo,
      modelProvider: sel?.provider ?? null, modelName: sel?.model ?? null,
      onEvent: channel,
    });
    // 停止请求可能**晚于**跑轮返回（两次 IPC 各自独立）。先等它一个准话，
    // 否则会在「还没到的取消结果」之前先宣布结局 —— 那是假回执的另一种形态。
    if (run.stopPromise) await run.stopPromise;
    const { status, labels, task_id, cancelled } = asRunResult(raw, "done");
    if (stopOf(run) !== "accepted") {
      finishStream(false);
      // 流式正文 + 任务节点卡（画板任务信息统一到对话流；标签 chips 随正文入库）。
      if (streamRaw.trim()) pushMsg(convo, "assistant", streamRaw.trim(), undefined, undefined, labels);
      // 个人对话：只留最终答案（过程件不上屏）；队内才并任务卡 + 耗时/消耗行。
      const team = isTeamConvo(convo);
      if (team) pushSys(`${status} · ${elapsed()} · ${stepCount}步`);
      // 本轮消耗（账本前后差值；有花费才入流，echo 零耗不打扰）。
      try {
        const after = await ntInvoke<CostActorRow[]>("neobot_cost_ledger_by_actor");
        const sum = (rows: CostActorRow[]) => rows.reduce((a, r) => a + r.in_tokens + r.out_tokens, 0);
        if (team && sum(after) - sum(cache.costs) > 0) {
          pushSys(`本轮消耗 ${sum(after) - sum(cache.costs)} tokens`);
        }
        cache.costs = after;
      } catch { /* 消耗统计失败不挡主流程 */ }
      try {
        // 任务卡只挂**真跑过轮**的那一条消息。桌面斜杠指令（`/help` `/stop` 等）
        // 在 Rust 侧跑轮之前就被本地回执掉了（`nt_cmd_run.rs` 的
        // `intercept_local_command`），一个任务都不建；此时若去取库里最新那条
        // 任务，挂上去的就是**上一轮真实对话**的任务卡。故先问语义
        // （`turnRanEngine`：status 是不是 TurnStatus），再问身份
        // （`turnTaskToAttach`：**优先**用后端给的 `task_id`，缺失时才回落到
        // 窗口内新建 + 归属本会话）。判据全不过就不挂 —— 不退回 `tasks[0]`。
        const ran = turnRanEngine(status);
        const after = ran ? await ntInvoke<TaskItem[]>("neobot_tasks") : null;
        const t = turnTaskToAttach(status, convo, taskIdsBefore, after, task_id);
        // 任务卡并入本轮回答（标准化输出块），个人对话不上卡。
        if (t && team) attachTaskToLast(convo, t.id);
        else if (!streamRaw.trim()) pushMsg(convo, "assistant", `本轮 ${status}。`, undefined, undefined, labels);
      } catch {
        pushMsg(convo, "assistant", `本轮 ${status}。`, undefined, undefined, labels);
      }
      setMood("done");
      if (convo) void ntInvoke("neobot_convo_mark_read", { id: convo }).catch(() => {});
      toast("本轮完成");
    } else if (cancelled) {
      // 停止信号递到了，而且这一轮**真的**以 `TaskStatus::Cancelled` 结束
      // （`cancelled` 是后端回库核对过的，不是「令牌被翻过」）。这才是「已停」。
      finishStream(true);
      pushSys(`已停（这一轮被叫停了，没有跑完 · ${elapsed()} · ${stepCount}步）`, convo);
      setMood("idle");
    } else {
      // 停止信号递到了，但这一轮**没被停掉** —— 它在信号到达前就跑完了。
      // 如实给回真实结果（不撤回正文、不假装已停）：那才是用户该看到的。
      finishStream(false);
      if (streamRaw.trim()) pushMsg(convo, "assistant", streamRaw.trim(), undefined, undefined, labels);
      pushSys("停止请求发出时这一轮已经跑完了（没停住）—— 上面是它真实的结果", convo);
      setMood("done");
      if (convo) void ntInvoke("neobot_convo_mark_read", { id: convo }).catch(() => {});
    }
  } catch (e) {
    const msg = String(e);
    // 池子不可达 → 自动回落本地回显再跑一次（默认模型不挡路）。
    if (sel && /transport|connection|refused|timed out|timedout|ECONN/i.test(msg)) {
      try {
        setMood("working");
        streamRaw = "";
        // 重试是**另一次**跑轮（`neobot_run`，它自己也登记一枚令牌 ⇒ 仍可被停）。
        // 之前那次 stop 的结论属于**第一次**跑轮，如实清掉，不拿它冒充这次的回执。
        run.stop = "none";
        run.stopPromise = null;
        const raw2 = await ntInvoke<RunResult | string>("neobot_run", {
          title, text: engineText, actorName: me, convoId: convo,
          modelProvider: null, modelName: null,
        });
        const { status: status2, labels: labels2 } = asRunResult(raw2, "done");
        if (stopOf(run) !== "accepted") {
          finishStream(false);
          pushMsg(convo, "assistant", `模型不可达，已用本地回显跑完（status=${status2}）。去设置·模型管理检查端点。`, undefined, undefined, labels2);
          setMood("done");
          if (convo) void ntInvoke("neobot_convo_mark_read", { id: convo }).catch(() => {});
          toast("已回落本地回显");
        } else { finishStream(true); setMood("idle"); }
      } catch (e2) {
        finishStream(true);
        pushMsg(convo, "assistant", `运行失败：${String(e2)}`);
        setMood("error");
        toast("运行失败", "err");
      }
    } else {
      finishStream(true);
      pushMsg(convo, "assistant", `运行失败：${msg}`);
      setMood("error");
      toast("运行失败", "err");
    }
  }
  finally {
    // 先把这一轮从「正在跑」里摘掉，再复位运行态 —— 顺序反了的话，
    // 在途的 cancel 失败回调会以为跑轮还活着而把界面**又**点亮。
    run.inFlight = false;
    if (activeRun === run) activeRun = null;
    setRunning(false);
    void refreshAll();
  }
}

// ─── 视图（当前只保留对话） ───
/** 展示 stale 的 running 任务 id 集（badge/忙点/hero 共用；只读展示，后端见 `nt_stale_guard`）。 */
function staleSet(): Set<string> {
  return new Set(staleRunningIds());
}
/** 对话列表“显示全部”开关（默认隐藏空/重名；只影响展示，不删数据）。 */
let showAllConvos = false;
function renderRail(): void {
  const v = currentView();
  document.querySelectorAll<HTMLButtonElement>(".rail-btn[data-view]").forEach((b) =>
    b.classList.toggle("active", b.dataset["view"] === v));
  const active = cache.tasks.filter((t) => (t.status === "pending" || t.status === "running") && !staleSet().has(t.id)).length;
  const badge = $("badge-convos");
  badge.classList.toggle("hidden", active === 0);
  badge.textContent = String(active);
  const me = load<string>(K.me, "neo");
  const meAv = $("me-avatar");
  // 左轨活球：只挂一次（重复挂载会泄漏引擎），之后只改 title。
  if (!meAv.querySelector("svg")) {
    meAv.innerHTML = "";
    (meAv as HTMLElement).style.background = "transparent";
    mountLiveRail(meAv);
  }
  (meAv as HTMLElement).title = me;
  // 头栏显示真实会话名（标题 + 成员行）。
  const sel = cache.convos.find((c) => c.id === currentConvo());
  if (sel) {
    $("head-title").textContent = sel.kind === "dm" ? peerOf(sel, me) : sel.title;
    const bits = [sel.kind === "dm" ? "个人对话" : "群组对话"];
    bits.push(sel.members.length <= 4 ? sel.members.join("、") : `${sel.members.length}人`);
    bits.push(`${sel.task_count}轮`);
    $("head-sub").textContent = bits.join(" · ");
  } else {
    $("head-title").textContent = "新对话";
    $("head-sub").textContent = "local-first · 数据只在 ~/.neobot";
  }
}

// ─── 顶栏搜索（pill 内搜本会话流，过滤＋计数，Esc 清除） ───
const headSearchBox = $("head-searchbox") as HTMLElement;
const headSearchInput = $("head-searchinput") as HTMLInputElement;
const headSearchCount = $("head-searchcount") as HTMLElement;
function clearHeadSearch(): void {
  headSearchInput.value = "";
  headSearchCount.textContent = "";
  document.querySelectorAll(".msg.search-hide").forEach((m) => m.classList.remove("search-hide"));
}
($("head-search") as HTMLButtonElement).onclick = (e) => {
  e.stopPropagation();
  const hidden = headSearchBox.classList.toggle("hidden");
  if (!hidden) { headSearchInput.focus(); headSearchInput.select(); } else { clearHeadSearch(); }
};
headSearchInput.addEventListener("input", () => {
  const q = headSearchInput.value.trim().toLowerCase();
  const rows = Array.from(document.querySelectorAll<HTMLElement>("#thread .msg[data-mi]"));
  if (!q) { rows.forEach((m) => m.classList.remove("search-hide")); headSearchCount.textContent = ""; return; }
  let hit = 0;
  rows.forEach((m) => {
    const ok = (m.textContent ?? "").toLowerCase().includes(q);
    m.classList.toggle("search-hide", !ok);
    if (ok) hit++;
  });
  headSearchCount.textContent = `${hit}/${rows.length}`;
});
headSearchInput.addEventListener("keydown", (e) => {
  if (e.key === "Escape") { clearHeadSearch(); headSearchBox.classList.add("hidden"); }
});

// ─── 左栏渲染 ───
function rowShell(sel: boolean): HTMLButtonElement {
  const b = document.createElement("button");
  b.className = "row" + (sel ? " selected" : "");
  return b;
}
function renderPanes(): void {
  clearThumbs(); // 旧缩略活球先销毁（整 pane 重建，防 rAF 泄漏）
  const box = $("convs-scroll"); box.innerHTML = "";
  renderConvosPane(box);
  renderInfo();
  renderRail();
  hydrateBalls(box);
}
($("convs-q") as HTMLInputElement).addEventListener("input", () => renderPanes());

function section(box: HTMLElement, title: string, count: number): void {
  const d = document.createElement("div");
  d.className = "sec-title";
  d.innerHTML = `<span>${esc(title)}</span><span class="count">${count}</span>`;
  box.appendChild(d);
}
function matchQ(...parts: string[]): boolean {
  const q = (($("convs-q") as HTMLInputElement).value || "").trim().toLowerCase();
  if (!q) return true;
  return parts.join(" ").toLowerCase().includes(q);
}

// ─── 对话列表：个人 DM / 群组（IM 语义；任务只活在画板） ───
function pinnedIds(): string[] { return load<string[]>(K.pinned, []); }
function peerOf(c: ConvoItem, me: string): string {
  return c.members.find((m) => m !== me) ?? c.title;
}
function renderConvosPane(box: HTMLElement): void {
  const me = load<string>(K.me, "neo");
  const sel = currentConvo();
  const pinned = new Set(pinnedIds());
  const q = (($("convs-q") as HTMLInputElement).value || "").trim().toLowerCase();
  // 全局搜索：标题/成员/转录文本（无分组标签，正常对话流）。
  const hitThread = (id: string) => {
    if (!q) return true;
    const msgs = threadOf(id);
    return msgs.some((m) => m.text.toLowerCase().includes(q));
  };
  const paintConvo = (c: ConvoItem) => {
    const b = rowShell(sel === c.id);
    const pin = pinned.has(c.id) ? `<span style="color:var(--gold-deep)"> ◆</span>` : "";
    const muteTag = c.muted ? `<span style="opacity:.55;font-size:11px" title="免打扰"> ·免打扰</span>` : "";
    // 未读角标 = 已读水位差；免打扰不亮。
    const badge = (!c.muted && c.unread > 0) ? c.unread : 0;
    // 果球交互态：未读泛珊瑚光、跑任务轻跳、免打扰降饱和、选中品牌光（CSS 消费）。
    // stale running 不计忙（展示层复位，后端见 `nt_stale_guard`）。
    const busy = cache.tasks.some((t) => t.conversation_id === c.id && (t.status === "pending" || t.status === "running") && !staleSet().has(t.id));
    const state = [
      badge > 0 ? "unread" : "",
      busy ? "busy" : "",
      c.muted ? "muted" : "",
      sel === c.id ? "sel" : "",
    ].filter(Boolean).join(" ");
    // 未读/免打扰行级 CSS 钩（纯展示：badge 计算逻辑不动）。
    if (badge > 0) b.classList.add("unread");
    if (c.muted) b.classList.add("muted");
    const peerPresence = c.kind === "dm"
      ? cache.presence.find((p) => p.id === peerOf(c, me))?.presence
      : undefined;
    const av = c.kind === "dm"
      ? ballAvatar(peerOf(c, me), 44, peerPresence, state)
      : avatarHtml(c.title, 44);
    // snippet 优先：草稿 ＞ 正在输入 ＞ 最近一条本地消息 ＞ 回落（IM 习惯）。
    // 草稿读本地 `ntos_drafts_v1`（只读不写）；typing 仅当 presence 真有该值才用，无则不动。
    const lastLocal = threadOf(c.id).filter((m) => m.role !== "sys").slice(-1)[0];
    const draftTxt = (allDrafts()[c.id] ?? "").trim();
    const typingActive = peerPresence === "typing";
    const subCls = draftTxt ? "pv is-draft" : typingActive ? "pv is-typing" : "pv";
    const sub = draftTxt
      ? `草稿：${esc(draftTxt.slice(0, 32))}`
      : typingActive
        ? `正在输入…`
        : lastLocal
          ? `${esc(lastLocal.text.slice(0, 32))}`
          : c.kind === "dm"
            ? peerPresence
              ? `${esc(peerPresence)} · ${c.task_count}轮`
              : `${c.task_count}轮`
            : `${c.members.length}人 · ${c.task_count}轮`;
    const localTs = lastLocal?.ts ?? 0;
    const remoteTs = new Date(c.last_active).getTime();
    const stamp = shortTime(new Date(Math.max(Number.isNaN(remoteTs) ? 0 : remoteTs, localTs)).toISOString());
    b.innerHTML = `${av}<div><div class="nm">${esc(c.kind === "dm" ? peerOf(c, me) : c.title)}${pin}${muteTag}</div><div class="${subCls}">${sub}</div></div><div><div class="tm">${esc(stamp)}</div>${badge > 0 ? `<span class="unread">${badge > 99 ? "99+" : badge}</span>` : ""}</div><span class="row-quick"><span data-q="pin" title="${pinned.has(c.id) ? "取消置顶" : "置顶"}">${pinned.has(c.id) ? "已顶" : "顶"}</span><span data-q="mute" title="${c.muted ? "取消免打扰" : "免打扰"}">${c.muted ? "已免" : "免"}</span></span>`;
    b.onclick = () => { selectConvo(c.id); expandRight(); };
    b.querySelectorAll<HTMLElement>("[data-q]").forEach((q) => {
      q.onclick = (e) => { e.stopPropagation(); void onConvoAct(q.dataset["q"] ?? "", c.id); };
    });
    b.oncontextmenu = (e) => { e.preventDefault(); openConvoMenu(e.clientX, e.clientY, c.id); };
    box.appendChild(b);
  };
  // 对话列表清理（只影响展示，不删数据；软隐藏优先）：
  // ① 空对话隐藏（任务数为 0 且本地无有效转录）；② 同名去重（DM 按对方名，群组按标题，
  // 置顶优先保留，否则留最新）；③ 按有效时间倒序（远端 last_active 与本地末条取大）。
  const effTs = (c: ConvoItem): number => {
    const remote = new Date(c.last_active).getTime();
    const local = threadOf(c.id).filter((m) => m.role !== "sys").slice(-1)[0]?.ts ?? 0;
    return Math.max(Number.isNaN(remote) ? 0 : remote, local);
  };
  const dispName = (c: ConvoItem): string =>
    (c.kind === "dm" ? peerOf(c, me) : c.title).trim().toLowerCase();
  const searched = cache.convos
    .filter((c) => matchQ(c.title, ...c.members) || hitThread(c.id));
  const sorted = searched
    .sort((a, b) => Number(pinned.has(b.id)) - Number(pinned.has(a.id)) || effTs(b) - effTs(a));
  const seenNames = new Set<string>();
  let hiddenConvos = 0;
  const list = sorted.filter((c) => {
    if (showAllConvos) return true;
    if (seenNames.has(dispName(c))) { hiddenConvos++; return false; }
    seenNames.add(dispName(c));
    const hasLocal = threadOf(c.id).some((m) => m.role !== "sys" && m.text.trim().length > 0);
    if (c.task_count === 0 && !hasLocal) { hiddenConvos++; return false; }
    return true;
  });
  if (list.length === 0) box.insertAdjacentHTML("beforeend", `<div class="empty-note">${q ? `无匹配「${esc(q)}」的对话` : "暂无对话 — 右上 + 发起"}</div>`);
  // 置顶分组线（纯展示分隔：排序/过滤逻辑不动，60 条上限内按原序切两段）。
  const visible = list.slice(0, 60);
  const pinnedVis = visible.filter((c) => pinned.has(c.id));
  const restVis = visible.filter((c) => !pinned.has(c.id));
  if (pinnedVis.length > 0 && restVis.length > 0) {
    section(box, "📌 置顶", pinnedVis.length);
    pinnedVis.forEach(paintConvo);
    section(box, "对话", restVis.length);
    restVis.forEach(paintConvo);
  } else {
    visible.forEach(paintConvo);
  }
  if (hiddenConvos > 0 && !showAllConvos) {
    const note = document.createElement("div");
    note.className = "empty-note";
    note.innerHTML = `已隐藏 ${hiddenConvos} 个空/重名对话 `;
    const btn = document.createElement("button");
    btn.className = "chip-btn";
    btn.textContent = "显示全部";
    btn.onclick = () => { showAllConvos = true; renderPanes(); };
    note.appendChild(btn);
    box.appendChild(note);
  } else if (showAllConvos) {
    const note = document.createElement("div");
    note.className = "empty-note";
    note.innerHTML = `已显示全部（含空/重名） `;
    const btn = document.createElement("button");
    btn.className = "chip-btn";
    btn.textContent = "仅看有效";
    btn.onclick = () => { showAllConvos = false; renderPanes(); };
    note.appendChild(btn);
    box.appendChild(note);
  }
  // 队友（点即开/进 DM，会话习惯；搜索同样命中名字）
  const pres = new Map(cache.presence.map((p) => [p.id, p]));
  const names = new Set<string>([...cache.members.map((m) => m.id), ...cache.presence.map((p) => p.id)]);
  const mates = [...names].filter((n) => n !== me && matchQ(n)).slice(0, 20);
  for (const n of mates) {
    const p = pres.get(n);
    const kind = cache.members.find((m) => m.id === n)?.kind ?? p?.kind ?? "human";
    const b = rowShell(false);
    b.innerHTML = `${ballAvatar(n, 44, p?.presence)}<div><div class="nm">${esc(n)}${cache.members.find((m) => m.id === n && m.owner) ? " ·主" : ""}</div><div class="pv">${esc(kind)}${p?.presence ? ` · ${esc(p.presence)}` : ""}</div></div><div></div>`;
    b.onclick = () => void openDm(n);
    box.appendChild(b);
  }
  hydrateBalls(box);
}
/** 短时间（今天 HH:MM / 昨天 / M-d），对话行右上。 */
function shortTime(rfc: string): string {
  const d = new Date(rfc);
  if (Number.isNaN(d.getTime())) return rfc.slice(5, 16).replace("T", " ");
  const now = new Date();
  const day = (x: Date) => `${x.getFullYear()}-${x.getMonth()}-${x.getDate()}`;
  const hm = `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
  if (day(d) === day(now)) return hm;
  const y = new Date(now.getTime() - 86400000);
  if (day(d) === day(y)) return `昨天 ${hm}`;
  return `${d.getMonth() + 1}-${d.getDate()} ${hm}`;
}

/** 与某人私聊（取或建 DM 并切过去）。 */
async function openDm(peer: string): Promise<void> {
  const me = load<string>(K.me, "neo");
  try {
    const id = await ntInvoke<string>("neobot_convo_dm", { me, peer });
    selectConvo(id);
    expandRight();
    await refreshAll();
    input.focus();
  } catch (e) { toast(String(e).slice(0, 90), "err"); }
}

// ─── 会话右键菜单：重命名 / 成员 / 删除 ───
function openConvoMenu(x: number, y: number, convoId: string): void {
  const c = cache.convos.find((c) => c.id === convoId);
  if (!c) return;
  const pinned = pinnedIds().includes(convoId);
  const items: Array<[string, string, boolean?]> = [
    ["rename", "重命名"],
    ["pin", pinned ? "取消置顶" : "置顶"],
    ["mute", c.muted ? "取消免打扰" : "免打扰"],
    ["members", c.kind === "dm" ? "查看成员" : "成员管理"],
    ["clear", "清空记录（仅本地转录）"],
    ["delete", "删除会话", false],
  ];
  const menu = $("ctxmenu");
  menu.innerHTML = items.map(([act, label, dis]) =>
    `<button data-ctx="${act}" ${dis ? "disabled" : ""} class="${act === "delete" ? "danger" : ""}">${esc(label)}</button>`).join("");
  menu.classList.remove("hidden");
  const mw = 168, mh = items.length * 33 + 8;
  menu.style.left = Math.min(x, window.innerWidth - mw - 8) + "px";
  menu.style.top = Math.min(y, window.innerHeight - mh - 8) + "px";
  menu.querySelectorAll<HTMLButtonElement>("[data-ctx]").forEach((b) => {
    b.onclick = () => { closeCtxMenu(); void onConvoAct(b.dataset["ctx"] ?? "", convoId); };
  });
}
async function onConvoAct(act: string, convoId: string): Promise<void> {
  const c = cache.convos.find((c) => c.id === convoId);
  try {
    if (act === "rename") {
      askInput("重命名会话", c?.title ?? "", async (v) => {
        await ntInvoke("neobot_convo_rename", { id: convoId, title: v });
        toast("已重命名");
        void refreshAll();
      });
    } else if (act === "pin") {
      const pins = pinnedIds();
      save(K.pinned, pins.includes(convoId) ? pins.filter((p) => p !== convoId) : [convoId, ...pins]);
      renderPanes();
    } else if (act === "mute") {
      await ntInvoke("neobot_convo_mute", { id: convoId, muted: !(c?.muted ?? false) });
      toast(c?.muted ? "已取消免打扰" : "已设免打扰");
      void refreshAll();
    } else if (act === "clear") {
      askConfirm("清空该会话的本地转录？（任务与账本保留）", async () => {
        const all = allThreads();
        delete all[convoId];
        saveThreads(all);
        if (currentConvo() === convoId) renderThread();
        toast("记录已清空");
      });
    } else if (act === "members") {
      openConvoMembers(convoId);
    } else if (act === "delete") {
      askConfirm("删除该会话？（含其中任务与附件，不可恢复）", async () => {
        await ntInvoke("neobot_convo_delete", { id: convoId });
        if (load<string>(K.selConvo, "") === convoId) selectConvo("general");
        toast("已删除");
        void refreshAll();
      });
    }
  } catch (e) { toast(String(e).slice(0, 90), "err"); }
}

// ─── 群组成员弹窗 ───
let ciConvo = "";
function openConvoMembers(convoId: string): void {
  const c = cache.convos.find((c) => c.id === convoId);
  if (!c) return;
  ciConvo = convoId;
  ($("ci-title") as HTMLElement).textContent = `${c.kind === "dm" ? "私聊" : "群组"} · ${c.title}`;
  renderConvoMembers();
  $("convoinfo").classList.remove("hidden");
}
function renderConvoMembers(): void {
  const c = cache.convos.find((c) => c.id === ciConvo);
  const box = $("ci-members");
  const pres = new Map(cache.presence.map((p) => [p.id, p]));
  box.innerHTML = "";
  for (const m of c?.members ?? []) {
    const li = document.createElement("li");
    li.innerHTML = `<b>${esc(m)}</b> <span class="muted">${esc(pres.get(m)?.presence ?? "off")}</span>${c?.kind === "group" ? ` <button class="chip-btn" data-rmm="${esc(m)}">移除</button>` : ""}`;
    box.appendChild(li);
  }
  box.querySelectorAll<HTMLButtonElement>("[data-rmm]").forEach((b) => {
    b.onclick = async () => {
      // `?? ""` 是纯类型收窄：`[data-rmm]` 选择器已保证该属性存在，运行时不改。
      // 少了它 `dataset[...]` 的 `string | undefined` 过不了 `member: String` 这道编译门。
      try { await ntInvoke("neobot_convo_rm_member", { id: ciConvo, member: b.dataset["rmm"] ?? "" }); }
      catch (e) { toast(String(e).slice(0, 80), "err"); }
      void refreshAll().then(renderConvoMembers);
    };
  });
}
$("ci-close").addEventListener("click", () => $("convoinfo").classList.add("hidden"));
$("ci-addbtn").addEventListener("click", async () => {
  const name = (($("ci-add") as HTMLInputElement).value || "").trim().slice(0, 8);
  if (!name || !ciConvo) return;
  try {
    await ntInvoke("neobot_convo_add_member", { id: ciConvo, member: name });
    ($("ci-add") as HTMLInputElement).value = "";
    toast("已添加");
  } catch (e) { toast(String(e).slice(0, 80), "err"); }
  void refreshAll().then(renderConvoMembers);
});

// ─── 新建会话（个人 DM / 发起群组，由 + 按钮经 openNewConvoPicker 进入） ───
let ncMode: "dm" | "group" = "group";
function renderNcMembers(): void {
  const me = load<string>(K.me, "neo");
  const box = $("nc-members");
  box.innerHTML = "";
  const candidates = cache.members.map((m) => m.id).filter((id) => id !== me);
  if (ncMode === "dm") {
    ($("nc-name") as HTMLInputElement).classList.add("hidden");
    ($("nc-title") as HTMLElement).textContent = "个人对话";
    if (candidates.length === 0) box.innerHTML = `<div class="empty-note">暂无其他成员 — 先去设置添加</div>`;
    for (const n of candidates) {
      const b = document.createElement("button");
      b.className = "chip-btn";
      b.style.cssText = "display:block;width:100%;text-align:left;margin-bottom:6px;padding:9px 12px";
      b.textContent = `@${n} 私聊`;
      b.onclick = () => { $("newconvo").classList.add("hidden"); void openDm(n); };
      box.appendChild(b);
    }
    ($("nc-yes") as HTMLButtonElement).style.display = "none";
  } else {
    ($("nc-name") as HTMLInputElement).classList.remove("hidden");
    ($("nc-title") as HTMLElement).textContent = "发起群组";
    ($("nc-yes") as HTMLButtonElement).style.display = "";
    if (candidates.length === 0) box.innerHTML = `<div class="empty-note">暂无其他成员 — 先去设置添加</div>`;
    for (const n of candidates) {
      const label = document.createElement("label");
      label.innerHTML = `<input type="checkbox" value="${esc(n)}" /> ${esc(n)}`;
      box.appendChild(label);
    }
  }
}
$("nc-no").addEventListener("click", () => $("newconvo").classList.add("hidden"));
$("nc-yes").addEventListener("click", async () => {
  if (ncMode !== "group") return;
  const title = (($("nc-name") as HTMLInputElement).value || "").trim().slice(0, 80) || "新群组";
  const picked = [...document.querySelectorAll<HTMLInputElement>("#nc-members input:checked")].map((i) => i.value);
  try {
    const id = await ntInvoke<string>("neobot_convo_group", { title, members: picked });
    selectConvo(id);
    $("newconvo").classList.add("hidden");
    toast("群组已建");
    await refreshAll();
    input.focus();
  } catch (e) { toast(String(e).slice(0, 90), "err"); }
});

// ─── 右键对话管理菜单 ───
function closeCtxMenu(): void { $("ctxmenu").classList.add("hidden"); }
document.addEventListener("click", (e) => {
  if (!(e.target as HTMLElement).closest?.("#ctxmenu")) closeCtxMenu();
});
document.addEventListener("keydown", (e) => { if (e.key === "Escape") closeCtxMenu(); });

// ─── 登录微内核看板视图 ───
function loginDot(s: string): string {
  const cls = s === "logged" ? "ldot-logged" : s === "checking" ? "ldot-checking"
    : s === "unlogged" ? "ldot-unlogged" : "ldot-unknown";
  return `<span class="ldot ${cls}"></span>`;
}
function loginBoardHtml(): string {
  const sites = loginSites();
  let html = `<div class="board-head"><div class="board-title">网页登录<div class="grow"></div>`
    + `<button class="chip-btn" data-login-add title="加登录站点">+ 站点</button></div>`
    + `<div class="empty-note">先打开登录，登完点验证；状态与对话框同步</div></div>`;
  if (sites.length === 0) html += `<div class="empty-note">暂无站点 — 点 + 站点加一个</div>`;
  for (const s of sites) {
    const st = loginState(s.id);
    const when = st.ts ? new Date(st.ts).toLocaleString() : "未验证";
    html += `<div class="login-row" data-nid="login-${esc(s.id)}">`
      + `<span title="${esc(st.evidence || "未验证")}">${loginDot(st.status)}</span> `
      + `<b>${esc(s.name)}</b> <span class="muted">${esc(s.url)}</span><br>`
      + `<span class="muted">${esc(st.evidence || when)}</span> `
      + `<button class="chip-btn" data-login-open="${esc(s.id)}">打开</button>`
      + `<button class="chip-btn" data-login-verify="${esc(s.id)}">验证</button>`
      + `<button class="chip-btn" data-login-rm="${esc(s.id)}">✕</button></div>`;
  }
  return html;
}
/** 看板验证：状态点先转 🌀，结论回状态表 + 进对话流（无选中对话即建默认 DM，消息不丢）。 */
async function onBoardLoginVerify(id: string): Promise<void> {
  const s = loginSites().find((x) => x.id === id);
  if (!s) return;
  renderInfo();
  try {
    const text = await verifyLoginSite(s);
    toast(text.slice(0, 60));
    const cur = await ensureSendConvo();
    if (cur) {
      pushAgentMsg(cur, `网页登录验证 · ${s.name}：${text}`, [], undefined);
      void ntInvoke("neobot_convo_mark_read", { id: cur }).catch(() => {});
      renderThread();
    }
  } catch (e) {
    toast(`验证失败：${String(e).slice(0, 80)}`, "err");
  }
  renderInfo();
}

function renderInfo(): void {
  const info = $("info");
  const f = load<string>(K.boardFilter, "summary");
  // —— 画板头：标题 + 计数 + 过滤器（汇总优先） ——
  const counts: Array<[string, string, number]> = [
    ["summary", "汇总", boardNodeCount("all")],
    ["login", "登录", loggedCount()],
    ["task", "任务", cache.tasks.length],
    ["routine", "例行", cache.routines.length],
    ["skill", "技能", cache.skills.length],
    ["ledger", "账本", cache.costs.length + cache.audits.filter((a) => a.decision === "deny").length],
  ];
  let html = `<div class="board-head"><div class="board-title">智能画板<div class="grow"></div><button class="chip-btn" data-page-add title="嵌入网页（可登录）">+ 网页</button><button class="chip-btn" data-collapse title="收起右栏">收起</button></div><div class="board-filters">${counts.map(([k, label, n]) => `<button class="chip-btn${f === k ? " on" : ""}" data-filter="${k}">${label} ${n}</button>`).join("")}</div><div class="board-zoom"><button data-zoom="out">−</button><input type="range" min="40" max="160" value="100" data-zoom-slider /><button data-zoom="in">+</button><button data-zoom="reset">1:1</button></div></div>`;
  // —— 选中巡检行（只读一行；动作钮统一在对话流任务卡） ——
  html += inspectorHtml();
  if (f === "summary") {
    // —— 汇总：聚合卡，需要时点卡下钻节点 ——
    html += summaryHtml();
  } else if (f === "login") {
    // —— 登录微内核视图（站点表 + 状态点 + 打开/验证；状态与对话框 🌐 同核同表） ——
    html += loginBoardHtml();
  } else {
    // —— 无限画布视口（平移/缩放/拖拽节点） ——
    const bv = boardView();
    html += `<div class="boardview" id="boardview"><div class="boardworld" id="boardworld" style="transform:translate(${bv.x}px,${bv.y}px) scale(${bv.k})">${renderBoardWorld(f)}</div></div>`;
  }
  info.innerHTML = html;
  info.querySelectorAll<HTMLButtonElement>("[data-filter]").forEach((b) => {
    b.onclick = () => { save(K.boardFilter, b.dataset["filter"]); renderInfo(); };
  });
  info.querySelectorAll<HTMLButtonElement>("[data-collapse]").forEach((b) => {
    b.onclick = () => collapseRight();
  });
  info.querySelectorAll<HTMLButtonElement>("[data-sumfilter]").forEach((b) => {
    b.onclick = () => { save(K.boardFilter, b.dataset["sumfilter"]); renderInfo(); };
  });
  info.querySelectorAll<HTMLButtonElement>("[data-sact]").forEach((b) => {
    b.onclick = () => void onSummaryAct(b.dataset["sact"] ?? "", b.dataset["sname"] ?? "");
  });
  // —— 登录微内核接线（打开/验证/加/删；验证是异步，点后先转 🌀） ——
  info.querySelectorAll<HTMLButtonElement>("[data-login-open]").forEach((b) => {
    b.onclick = async (e) => {
      e.stopPropagation();
      const s = loginSites().find((x) => x.id === b.dataset["loginOpen"]);
      if (!s || !validLoginUrl(s.url)) { toast("站点地址非法", "err"); return; }
      await openLoginSite(s.url);
      toast("已在系统浏览器打开，登录后点验证");
    };
  });
  info.querySelectorAll<HTMLButtonElement>("[data-login-verify]").forEach((b) => {
    b.onclick = (e) => {
      e.stopPropagation();
      void onBoardLoginVerify(b.dataset["loginVerify"] ?? "");
    };
  });
  info.querySelectorAll<HTMLButtonElement>("[data-login-rm]").forEach((b) => {
    b.onclick = (e) => {
      e.stopPropagation();
      const id = b.dataset["loginRm"] ?? "";
      saveLoginSites(loginSites().filter((x) => x.id !== id));
      renderInfo();
    };
  });
  info.querySelectorAll<HTMLButtonElement>("[data-login-add]").forEach((b) => {
    b.onclick = () => {
      askInput("加登录站点地址", "https://", (url) => {
        const u = url.trim();
        if (!validLoginUrl(u)) { toast("地址须 http(s):// 开头", "err"); return; }
        const sites = loginSites();
        sites.push({ id: `s${Date.now()}`, name: siteNameOf(u), url: u });
        saveLoginSites(sites);
        renderInfo();
      });
    };
  });
  info.querySelectorAll<HTMLButtonElement>("[data-page-add]").forEach((b) => {    b.onclick = () => {
      askInput("嵌入网页地址", "https://", (url) => {
        const u = url.trim();
        if (!/^https?:\/\//i.test(u)) { toast("地址须 http(s):// 开头", "err"); return; }
        const pages = boardPages();
        const n = pages.length;
        pages.push({ id: `p${Date.now()}`, url: u, x: 120 + n * 36, y: 120 + n * 36, w: 360, h: 420 });
        savePages(pages);
        renderInfo();
        toast("网页已上板（部分站点禁嵌套时用 ↗ 外开）");
      });
    };
  });
  info.querySelectorAll<HTMLButtonElement>("[data-pact]").forEach((b) => {
    b.onclick = async (e) => {
      e.stopPropagation();
      const id = b.closest("[data-nid]")?.getAttribute("data-nid")?.slice(5) ?? "";
      const pages = boardPages();
      const p = pages.find((x) => x.id === id);
      if (!p) return;
      const act = b.dataset["pact"];
      if (act === "close") {
        savePages(pages.filter((x) => x.id !== id));
        renderInfo();
      } else if (act === "reload") {
        const f = b.closest(".pagenode")?.querySelector("iframe");
        if (f) (f as HTMLIFrameElement).src = p.url;
      } else if (act === "open") {
        // F8（审计）：外部打开前白名单协议——CVE-2025-31477 类（plugin-shell 协议校验不当）。
        // p.url 手输可任意字符串，非 http(s) 一律拒（file:// / 自定义 scheme 直达系统 handler）。
        if (!/^https?:\/\//i.test(p.url.trim())) {
          toast("仅支持打开 http(s) 链接", "err");
          return;
        }
        try {
          const { open } = await import("@tauri-apps/plugin-shell");
          await open(p.url);
        } catch { window.open(p.url, "_blank"); }
      }
    };
  });
  info.querySelectorAll<HTMLButtonElement>("[data-zoom]").forEach((b) => {
    b.onclick = () => {
      const cur = boardView();
      const z = b.dataset["zoom"];
      const k = z === "reset" ? 1 : Math.min(1.6, Math.max(0.4, cur.k + (z === "in" ? 0.15 : -0.15)));
      saveView(z === "reset" ? { x: 0, y: 0, k: 1 } : { ...cur, k });
      renderInfo();
    };
  });
  info.querySelectorAll<HTMLInputElement>("[data-zoom-slider]").forEach((s) => {
    s.value = String(Math.round(boardView().k * 100));
    s.oninput = () => {
      const cur = boardView();
      saveView({ ...cur, k: Number(s.value) / 100 });
      const w = $("boardworld");
      if (w) w.style.transform = `translate(${cur.x}px,${cur.y}px) scale(${Number(s.value) / 100})`;
    };
    s.onchange = () => renderInfo();
  });
  wireBoardDrag();
  renderInfoHero();
  // stale hero 取消键（展示层复位：调已有 `neobot_task_cancel` 通道，不碰调度）。
  hero.querySelectorAll<HTMLButtonElement>("[data-hero]").forEach((b) => {
    b.onclick = async (e) => {
      e.stopPropagation();
      const tid = b.dataset["tid"] ?? "";
      if (b.dataset["hero"] !== "cancel" || !tid) return;
      try {
        await ntInvoke("neobot_task_cancel", { taskId: tid });
        toast("已取消卡死任务");
      } catch (err) { toast(String(err).slice(0, 90), "err"); }
      void refreshAll();
    };
  });
}

// —— 选中巡检行（只读一行；动作统一在对话流任务卡） ——

// —— 汇总卡（聚合呈现；点卡下钻节点） ——
async function onSummaryAct(act: string, name: string): Promise<void> {
  try {
    if (act === "fire" && name) {
      const r = await ntInvoke<string>("neobot_routine_fire", { name });
      toast(`firing 完成：${r}`);
    } else if (act === "sweep") {
      const rows = await ntInvoke<string[]>("neobot_routine_sweep");
      toast(rows.length === 0 ? "无到期例行" : `跑了 ${rows.length} 条`);
    }
  } catch (e) { toast(String(e).slice(0, 90), "err"); }
  void refreshAll();
}

// —— 画布交互：空白拖拽平移 / 滚轮缩放 / 节点头拖拽（点击阈值内仍算打开） ——
function wireBoardDrag(): void {
  const view = $("boardview");
  const world = $("boardworld");
  if (!view || !world) return;
  let pan: { sx: number; sy: number; ox: number; oy: number } | null = null;
  let drag: { key: string; sx: number; sy: number; ox: number; oy: number; moved: boolean; page: boolean; nx?: number; ny?: number } | null = null;
  const paintBg = () => {
    const v = boardView();
    view.style.backgroundPosition = `${v.x}px ${v.y}px`;
  };
  paintBg();
  view.addEventListener("pointerdown", (e) => {
    const t = e.target as HTMLElement;
    if (t.closest("button,input,iframe,a")) return;
    const node = t.closest<HTMLElement>("[data-nid]");
    const v = boardView();
    if (node) {
      // 页节点只认 header 拖拽；数据节点体可拖（按钮已在上面排除）。
      const key = node.dataset["nid"] ?? "";
      const isPage = key.startsWith("page:");
      if (isPage && !t.closest("[data-drag]")) return;
      const p = nodePos()[key];
      drag = { key, sx: e.clientX, sy: e.clientY, ox: (p?.x ?? node.offsetLeft), oy: (p?.y ?? node.offsetTop), moved: false, page: isPage };
    } else {
      pan = { sx: e.clientX, sy: e.clientY, ox: v.x, oy: v.y };
      view.setPointerCapture(e.pointerId);
    }
  });
  view.addEventListener("pointermove", (e) => {
    const v = boardView();
    if (pan) {
      saveView({ ...v, x: pan.ox + (e.clientX - pan.sx), y: pan.oy + (e.clientY - pan.sy) });
      world.style.transform = `translate(${pan.ox + (e.clientX - pan.sx)}px,${pan.oy + (e.clientY - pan.sy)}px) scale(${v.k})`;
      paintBg();
    } else if (drag) {
      const dx = (e.clientX - drag.sx) / v.k, dy = (e.clientY - drag.sy) / v.k;
      if (Math.abs(e.clientX - drag.sx) + Math.abs(e.clientY - drag.sy) > 4) drag.moved = true;
      const el = view.querySelector<HTMLElement>(`[data-nid="${CSS.escape(drag.key)}"]`);
      if (el) { el.style.left = `${drag.ox + dx}px`; el.style.top = `${drag.oy + dy}px`; }
      drag.nx = drag.ox + dx;
      drag.ny = drag.oy + dy;
    }
  });
  const end = (e: PointerEvent) => {
    void e;
    if (pan) { pan = null; }
    if (drag) {
      const d = drag; drag = null;
      if (!d.moved) {
        if (d.key.startsWith("page:")) return;
        void onNodeOpen(d.key);
      } else if (d.page) {
        const id = d.key.slice(5);
        const pages = boardPages();
        const p = pages.find((x) => x.id === id);
        if (p) {
          p.x = Math.round(d.nx ?? d.ox);
          p.y = Math.round(d.ny ?? d.oy);
          savePages(pages);
        }
      } else {
        const pos = nodePos();
        pos[d.key] = { x: Math.round(d.nx ?? d.ox), y: Math.round(d.ny ?? d.oy) };
        save(K.boardPos, pos);
      }
    }
  };
  view.addEventListener("pointerup", end);
  view.addEventListener("pointercancel", () => { pan = null; drag = null; });
  view.addEventListener("wheel", (e) => {
    e.preventDefault();
    const v = boardView();
    const k = Math.min(1.6, Math.max(0.4, v.k * (e.deltaY < 0 ? 1.1 : 1 / 1.1)));
    const r = view.getBoundingClientRect();
    const mx = e.clientX - r.left, my = e.clientY - r.top;
    const x = mx - ((mx - v.x) / v.k) * k;
    const y = my - ((my - v.y) / v.k) * k;
    saveView({ x, y, k });
    world.style.transform = `translate(${x}px,${y}px) scale(${k})`;
    paintBg();
    const s = view.parentElement?.querySelector<HTMLInputElement>("[data-zoom-slider]");
    if (s) s.value = String(Math.round(k * 100));
  }, { passive: false });
}


// —— 画板节点流：全部成果与节点 ——






async function onNodeOpen(spec: string): Promise<void> {
  const idx = spec.indexOf(":");
  const kind = idx < 0 ? spec : spec.slice(0, idx);
  const id = idx < 0 ? "" : spec.slice(idx + 1);
  if (kind === "task" && id) save(K.selTask, id);
  else if (kind === "routine" && id) save(K.selRoutine, id);
  else if (kind === "skill" && id) save(K.selSkill, id);
  else { toast("成本总览见中间账本页"); return; }
  expandRight();
  renderPanes();
}


// ─── 数据拉取 ───
async function safe<T>(fn: () => Promise<T>, fallback: T): Promise<T> {
  try { return await fn(); } catch { return fallback; }
}
async function refreshAll(): Promise<void> {
  // 正看着的会话先标已读（IM 习惯：眼前的不算未读），再拉列表。
  const cur = currentConvo();
  if (cur) await safe(() => ntInvoke("neobot_convo_mark_read", { id: cur }), undefined);
  cache.tasks = await safe(() => ntInvoke<TaskItem[]>("neobot_tasks"), []);
  cache.audits = await safe(() => ntInvoke<AuditItem[]>("neobot_audit"), []);
  cache.costs = await safe(() => ntInvoke<CostActorRow[]>("neobot_cost_ledger_by_actor"), []);
  cache.models = await safe(() => ntInvoke<ModelItem[]>("neobot_models"), []);
  cache.routines = await safe(() => ntInvoke<RoutineItem[]>("neobot_routine_list"), []);
  cache.skills = await safe(() => ntInvoke<SkillItem[]>("neobot_skills"), []);
  cache.presence = await safe(() => ntInvoke<PresenceItem[]>("neobot_roster_list"), []);
  cache.members = await safe(() => ntInvoke<MemberItem[]>("neobot_members"), []);
  cache.convos = await safe(() => ntInvoke<ConvoItem[]>("neobot_convos"), []);
  migrateThreads();
  updateModelBtn();
  renderPanes();
}
async function heartbeat(): Promise<void> {
  const me = load<string>(K.me, "neo");
  try { await ntInvoke("neobot_roster_heartbeat", { memberId: me, kind: "human" }); } catch { /* 忽略 */ }
  cache.presence = await safe(() => ntInvoke<PresenceItem[]>("neobot_roster_list"), []);
  if (currentView() === "convos") renderPanes();
}

// ─── 输入框（重命名用；确认框共用） ───
let inputFn: ((v: string) => void) | null = null;
function askInput(text: string, initial: string, fn: (v: string) => void): void {
  ($("confirm-text") as HTMLElement).textContent = text;
  const box = $("confirm-input") as HTMLInputElement;
  box.classList.remove("hidden");
  box.value = initial;
  confirmFn = null;
  inputFn = fn;
  $("confirm").classList.remove("hidden");
  box.focus();
  box.select();
}
($("confirm-input") as HTMLInputElement).addEventListener("keydown", (e) => {
  if (e.key === "Enter") ($("confirm-yes") as HTMLButtonElement).click();
});

// ─── 设置独立窗口 ───
$("btn-settings").addEventListener("click", () => {
  void ntInvoke("neobot_settings_window").catch((e) => toast(String(e).slice(0, 80), "err"));
});

// ─── 确认框（破坏性操作二次确认，对齐主 App ConfirmModal） ───
let confirmFn: (() => void) | null = null;
function hideConfirm(): void {
  confirmFn = null; inputFn = null;
  ($("confirm-input") as HTMLInputElement).classList.add("hidden");
  $("confirm").classList.add("hidden");
}
function askConfirm(text: string, fn: () => void): void {
  ($("confirm-text") as HTMLElement).textContent = text;
  ($("confirm-input") as HTMLInputElement).classList.add("hidden");
  confirmFn = fn;
  $("confirm").classList.remove("hidden");
}
$("confirm-no").addEventListener("click", hideConfirm);
$("confirm-yes").addEventListener("click", () => {
  const fn = confirmFn;
  const iv = inputFn;
  const v = (($("confirm-input") as HTMLInputElement).value || "").trim();
  hideConfirm();
  if (iv) { if (v) iv(v); return; }
  fn?.();
});

// ─── 外观偏好（localStorage + documentElement.dataset；设置窗口经事件同步） ───
const PREF_DENSITY = "ntos_pref_density";
const PREF_FONT = "ntos_pref_font";
const PREF_MOTION = "ntos_pref_motion";
const PREF_MSGWIDTH = "ntos_pref_msgwidth";
const PREF_FACE = "ntos_pref_face";
const PREF_BUBBLE = "ntos_pref_bubble";
const PREF_DARK = "ntos_pref_dark";
const PREF_ENTER = "ntos_pref_enter";
const PREF_NBANNER = "ntos_pref_nbanner";
const PREF_NVIEW = "ntos_pref_nview";
const PREF_NSOUND = "ntos_pref_nsound";
const PREF_READS = "ntos_pref_reads";
const PREF_AUTODL = "ntos_pref_autodl";
const PREF_WALL = "ntos_pref_wall";
function applyPrefs(): void {
  const root = document.documentElement;
  root.dataset["density"] = load<string>(PREF_DENSITY, "comfortable");
  root.dataset["fontsize"] = load<string>(PREF_FONT, "md");
  root.dataset["motion"] = load<string>(PREF_MOTION, "full");
  root.dataset["msgwidth"] = load<string>(PREF_MSGWIDTH, "normal");
  root.dataset["fontface"] = load<string>(PREF_FACE, "system");
  root.dataset["bubble"] = load<string>(PREF_BUBBLE, "snow");
  root.dataset["reads"] = load<string>(PREF_READS, "on");
  root.dataset["wall"] = load<string>(PREF_WALL, "snow");
  applyDark();
}
function applyDark(): void {
  const root = document.documentElement;
  const mode = load<string>(PREF_DARK, "light");
  const dark = mode === "dark" || (mode === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  root.dataset["dark"] = dark ? "dark" : "light";
}

// ─── 三栏列宽（拖拽 + 持久化；右栏双击收起/展开） ───
const W_CONVS = "ntos_w_convs", W_INFO = "ntos_w_info", INFO_OPEN = "ntos_info_open";
// 工作台（侧边栏右列）另有自己的宽度与开关；默认**折叠**，
// 不改变既有的三栏手感 —— 要用再点左轨那个按钮展开。
const W_SB = "ntos_w_sb", SB_OPEN = "ntos_sb_open";
function isInfoOpen(): boolean { return load<boolean>(INFO_OPEN, true); }
function isSbOpen(): boolean { return load<boolean>(SB_OPEN, false); }
function applyWidths(): void {
  const body = $("body");
  body.style.setProperty("--w-convs", load<number>(W_CONVS, 320) + "px");
  body.style.setProperty("--w-info", load<number>(W_INFO, 420) + "px");
  body.style.setProperty("--w-sb", load<number>(W_SB, 460) + "px");
  body.classList.toggle("no-right", !isInfoOpen());
  // 工作台展开时收起智能画板：两个右列并排会把聊天挤到放不下。
  const sb = isSbOpen();
  body.classList.toggle("sb-open", sb);
  $("sb-panel")?.classList.toggle("hidden", !sb);
  $("grip-sb")?.classList.toggle("hidden", !sb);
}
/** 把手 id → CSS 变量（新增把手时只改这张表）。 */
const WIDTH_VAR: Record<string, string> = {
  "grip-convs": "--w-convs",
  "grip-info": "--w-info",
  "grip-sb": "--w-sb",
};
function initGrip(id: string, key: string, min: number, max: number): void {
  const grip = $(id);
  grip.addEventListener("mousedown", (e) => {
    e.preventDefault();
    grip.classList.add("drag");
    const move = (ev: MouseEvent) => {
      const r = $("body").getBoundingClientRect();
      const v = id === "grip-convs" ? ev.clientX - r.left : r.right - ev.clientX;
      const w = Math.round(Math.min(max, Math.max(min, v)));
      save(key, w);
      $("body").style.setProperty(WIDTH_VAR[id] ?? "--w-convs", w + "px");
    };
    const up = () => {
      grip.classList.remove("drag");
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  });
}
/** 右栏自动展开（选中即展开，交互驱动）。 */
function expandRight(): void {
  if (!isInfoOpen()) { save(INFO_OPEN, true); applyWidths(); }
}
function collapseRight(): void {
  save(INFO_OPEN, false);
  applyWidths();
}

// ─── 主题（品牌色模板；设置窗口导入/选择后经事件同步，持久化） ───
interface ThemeState { name: string; tokens: Record<string, string>; }
function applyTheme(tokens: Record<string, string>): void {
  const root = document.documentElement;
  for (const [k, v] of Object.entries(tokens)) {
    root.style.setProperty(`--${k}`, v);
  }
  // 果球跟随品牌色（skype token 即球体色；启动时先于挂球执行）。
  if (typeof tokens["skype"] === "string") setBallColor(tokens["skype"]);
}
function currentTheme(): ThemeState {
  return load<ThemeState>(K.theme, { name: "默认", tokens: {} });
}
void listen<{ name: string; tokens: Record<string, string> }>("neobot:theme", (e) => {
  save(K.theme, { name: e.payload.name, tokens: e.payload.tokens });
  applyTheme(e.payload.tokens);
  toast(`主题：${e.payload.name}`);
});
void listen("neobot:theme-query", () => {
  void emit("neobot:theme-state", currentTheme());
});

// ─── 设置窗口事件委托（偏好/备份/清屏/改名，主窗口执行） ───
void listen<{ density?: string; fontsize?: string; motion?: string; msgwidth?: string; fontface?: string; bubble?: string; dark?: string; enterkey?: string; nbanner?: string; nview?: string; nsound?: string; reads?: string; autodl?: string; wall?: string }>("neobot:prefs", (e) => {
  if (e.payload.density) save(PREF_DENSITY, e.payload.density);
  if (e.payload.fontsize) save(PREF_FONT, e.payload.fontsize);
  if (e.payload.motion) save(PREF_MOTION, e.payload.motion);
  if (e.payload.msgwidth) save(PREF_MSGWIDTH, e.payload.msgwidth);
  if (e.payload.fontface) save(PREF_FACE, e.payload.fontface);
  if (e.payload.bubble) save(PREF_BUBBLE, e.payload.bubble);
  if (e.payload.dark) save(PREF_DARK, e.payload.dark);
  if (e.payload.enterkey) save(PREF_ENTER, e.payload.enterkey);
  if (e.payload.nbanner) save(PREF_NBANNER, e.payload.nbanner);
  if (e.payload.nview) save(PREF_NVIEW, e.payload.nview);
  if (e.payload.nsound) save(PREF_NSOUND, e.payload.nsound);
  if (e.payload.reads) save(PREF_READS, e.payload.reads);
  if (e.payload.autodl) save(PREF_AUTODL, e.payload.autodl);
  if (e.payload.wall) save(PREF_WALL, e.payload.wall);
  applyPrefs();
});
void listen("neobot:prefs-query", () => {
  void emit("neobot:prefs-state", {
    density: load<string>(PREF_DENSITY, "comfortable"),
    fontsize: load<string>(PREF_FONT, "md"),
    motion: load<string>(PREF_MOTION, "full"),
    msgwidth: load<string>(PREF_MSGWIDTH, "normal"),
    fontface: load<string>(PREF_FACE, "system"),
    bubble: load<string>(PREF_BUBBLE, "snow"),
    dark: load<string>(PREF_DARK, "light"),
    enterkey: load<string>(PREF_ENTER, "enter"),
    nbanner: load<string>(PREF_NBANNER, "on"),
    nview: load<string>(PREF_NVIEW, "full"),
    nsound: load<string>(PREF_NSOUND, "on"),
    reads: load<string>(PREF_READS, "on"),
    autodl: load<string>(PREF_AUTODL, "lazy"),
    wall: load<string>(PREF_WALL, "snow"),
  });
});
void listen("neobot:backup-export", () => {
  const data = JSON.stringify({ version: 3, app: "neobot", exportedAt: new Date().toISOString(), payload: { threads: allThreads(), mates: load(K.mates, []), starters: load(K.starters, DEFAULT_STARTERS), me: load(K.me, "neo") } });
  void emit("neobot:backup-data", data);
});
void listen<{ text: string }>("neobot:backup-import", (e) => {
  try {
    const j = JSON.parse(e.payload.text) as { app?: string; payload?: Record<string, unknown> };
    if (j.app !== "neobot" || !j.payload) { void emit("neobot:backup-done", "导入失败：非 neobot 备份"); return; }
    const d = j.payload;
    if (d["threads"]) save(K.threads, d["threads"]);
    else if (d["msgs"]) saveThreads({ [currentConvo()]: d["msgs"] as ChatMsg[] });
    if (d["mates"]) save(K.mates, d["mates"]);
    if (d["starters"]) save(K.starters, d["starters"]);
    renderThread();
    void emit("neobot:backup-done", "导入成功");
    toast("备份已导入");
    void refreshAll();
  } catch (err) { void emit("neobot:backup-done", `导入失败：${String(err).slice(0, 80)}`); }
});
void listen("neobot:msgs-clear", () => {
  const all = allThreads();
  all[currentConvo()] = [];
  saveThreads(all);
  renderThread();
  void emit("neobot:backup-done", "当前会话已清空");
  toast("会话已清空");
});
void listen<{ name: string }>("neobot:set-me", (e) => {
  const name = (e.payload.name || "").trim().slice(0, 8);
  if (!name) return;
  save(K.me, name);
  const mates = load<string[]>(K.mates, []);
  if (!mates.includes(name)) { mates.push(name); save(K.mates, mates); }
  renderRail(); void heartbeat();
});

// ─── 启动 ───
// 启动护栏（fail-visible）：同步启动任一步抛错都不留空白屏——顶栏报错条显错，// [重试] 重载，[清空视图缓存] 只清视图偏好（boardFilter/boardView/boardPos/theme/selConvo），
// 不动转录 threads 与业务库。
function showBootError(stage: string, e: unknown): void {
  try {
    let bar = document.getElementById("errorbar");
    if (!bar) {
      bar = document.createElement("div");
      bar.id = "errorbar";
      document.body.prepend(bar);
    }
    bar.innerHTML = `<b>启动卡住（${esc(stage)}）</b><span>${esc(String(e).slice(0, 220))}</span>`
      + `<button id="eb-retry">重试</button><button id="eb-reset">清空视图缓存</button>`;
    (document.getElementById("eb-retry") as HTMLButtonElement | null)?.addEventListener("click", () => location.reload());
    (document.getElementById("eb-reset") as HTMLButtonElement | null)?.addEventListener("click", () => {
      try {
        for (const k of [K.boardFilter, K.boardView, K.boardPos, K.theme, K.selConvo]) {
          try { localStorage.removeItem(k); } catch { /* 忽略 */ }
        }
      } finally { location.reload(); }
    });
  } catch { /* 护栏自身永不抛 */ }
}
window.addEventListener("error", (ev) => {
  showBootError("window.onerror", (ev as ErrorEvent).message || (ev as ErrorEvent).error);
});
paintIcons();
setThreadActions({
  onSuggest: (idx) => {
    const s = load(K.starters, DEFAULT_STARTERS)[idx];
    if (!s) return;
    input.value = s.prompt;
    input.focus();
    persistDraft();
    autogrow();
  },
  onMsg: (convo, act, idx) => void onMsgAct(convo, act, idx),
  onTask: (act, taskId) => void onTaskNodeAct(act, taskId),
  onCite: (url) => {
    // F8 白名单：引用原文外开，非 http(s) 一律拒。
    if (!/^https?:\/\//i.test(url.trim())) { toast("仅支持打开 http(s) 链接", "err"); return; }
    void (async () => {
      try {
        const { open } = await import("@tauri-apps/plugin-shell");
        await open(url);
      } catch { window.open(url, "_blank"); }
    })();
  },
});
applyPrefs();
applyTheme(currentTheme().tokens);
applyWidths();
initGrip("grip-convs", W_CONVS, 240, 520);
initGrip("grip-info", W_INFO, 300, 640);
initGrip("grip-sb", W_SB, 320, 720);
$("grip-info").addEventListener("dblclick", () => {
  if (isInfoOpen()) collapseRight(); else expandRight();
});
/** 工作台需要从主壳拿到的能力（单向注入，避免 sidebar 反向依赖内部状态）。 */
function sidebarOpts() {
  return {
    onInsertRef: (text: string) => {
      // `$` 无泛型（`core.ts` 的签名），`input` 在本模块是已导出的 textarea 常量。
      const ta = input;
      if (!ta) return;
      ta.value += text;
      ta.dispatchEvent(new Event("input"));
      ta.focus();
    },
    onGetParentConvoId: () => currentConvo(),
    onRunTurn: async (text: string, convoId: string) => {
      // 侧聊与主对话走**同一条**跑轮路径：同一引擎路由、同一策略网关、
      // 同一账本。差别只在 convoId —— 任务因此落在侧聊那个会话里，
      // 母会话的任务数与转录不受影响（「不污染母会话」是结构性保证，不是约定）。
      await runStreamTurn(
        convoId,
        "侧聊",
        text,
        load<string>(K.me, "neo"),
        modelSel(),
      );
    },
  };
}
// 工作台：左轨按钮开合。展开时收起智能画板（两个右列会把聊天挤扁）。
$("btn-workbench").addEventListener("click", () => {
  const next = !isSbOpen();
  save(SB_OPEN, next);
  if (next) save(INFO_OPEN, false);
  applyWidths();
  if (next) mountSidebar($("sb-panel"), sidebarOpts());
});
mountSidebar($("sb-panel"), sidebarOpts());
mountLiveBall($("moodball"));
// 作曲区保持干净：⚡/🌐 不挂载（Agent 走后端自动流，登录走看板微内核）。
window.addEventListener("pointermove", (e) => gazeAt($("moodball"), e.clientX, e.clientY));
try {
  renderThread(); restoreDraft(); renderRail(); updateSendState();
} catch (e) { showBootError("boot-render", e); }
void refreshAll();
void heartbeat();
setInterval(heartbeat, 30_000);
setInterval(() => void refreshAll(), 15_000);
// 工作台的展开目录跟着自动重列（better-sidebar 的 watch 等价物；只在文件页跑）。
setInterval(() => void refreshSidebarTree(), 4_000);
// 变动 / 任务 / 侧聊三页跟着主心跳刷新（否则数据停在打开那一刻）。
setInterval(() => void refreshSidebarPanels(), 15_000);
