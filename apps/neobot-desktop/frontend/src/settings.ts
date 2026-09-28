/**
 * 设置窗口（个人信息 / 通用 / 模型管理 / 关于）。
 * 数据面直调 neobot IPC（共享 SQLite）；外观偏好与备份经事件委托主窗口
 * （localStorage 按 webview 隔离，不可直读主窗口键）。
 */
import { emit, listen } from "@tauri-apps/api/event";
import { icon } from "./icons";
import { ntInvoke } from "./invoke";
import { avatarBg, BUILTIN_THEMES, FONT_FACES, type ThemeTokens } from "./theme";

const $ = (id: string) => document.getElementById(id) as HTMLElement;
function esc(s: string): string { return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;"); }
function toast(text: string, kind: "ok" | "err" = "ok"): void {
  const box = $("toasts");
  const el = document.createElement("div");
  el.className = `toast ${kind}`; el.textContent = text;
  box.appendChild(el);
  setTimeout(() => el.remove(), 2400);
}

// ─── 标签页 ───
document.querySelectorAll<HTMLButtonElement>("#setnav button").forEach((b) => {
  b.onclick = () => {
    document.querySelectorAll("#setnav button").forEach((x) => x.classList.remove("on"));
    b.classList.add("on");
    document.querySelectorAll<HTMLElement>("[data-pane]").forEach((p) => {
      p.classList.toggle("hidden", p.dataset["pane"] !== b.dataset["tab"]);
    });
  };
});

// ─── 确认框 ───
let confirmFn: (() => void) | null = null;
function askConfirm(text: string, fn: () => void): void {
  ($("confirm-text") as HTMLElement).textContent = text;
  confirmFn = fn;
  $("confirm").classList.remove("hidden");
}
$("confirm-no").addEventListener("click", () => { confirmFn = null; $("confirm").classList.add("hidden"); });
$("confirm-yes").addEventListener("click", () => {
  $("confirm").classList.add("hidden");
  const fn = confirmFn; confirmFn = null;
  fn?.();
});

// ─── 个人信息 ───
function paintHero(name: string, sub: string): void {
  $("s-hero-name").textContent = name;
  $("s-hero-sub").textContent = sub;
  const av = $("s-avatar");
  av.textContent = (name.trim().charAt(0) || "我").toUpperCase();
  (av as HTMLElement).style.background = avatarBg(name);
}
async function refreshMembers(): Promise<void> {
  try {
    const members = await ntInvoke<Array<{ id: string; kind: string; owner: boolean }>>("neobot_members");
    const presence = await ntInvoke<Array<{ id: string; presence: string }>>("neobot_roster_list").catch(() => [] as Array<{ id: string; presence: string }>);
    const pres = new Map(presence.map((p) => [p.id, p.presence]));
    const box = $("s-members");
    box.innerHTML = members.length === 0 ? `<li class="empty-note">暂无成员</li>` : "";
    const owner = members.find((m) => m.owner);
    for (const m of members) {
      const li = document.createElement("li");
      const dot = pres.get(m.id);
      const dotHtml = dot ? `<span class="status-dot st-${dot === "active" ? "done" : dot === "idle" ? "running" : "cancelled"}"></span>` : "";
      li.innerHTML = `<b>${esc(m.id)}</b> <span class="muted">${esc(m.kind)}${m.owner ? " ·主" : ""}${dot ? ` ·${dot}` : ""}</span> ${dotHtml}<button class="chip-btn" data-rm="${esc(m.id)}">移除</button>`;
      box.appendChild(li);
    }
    box.querySelectorAll<HTMLButtonElement>("[data-rm]").forEach((b) => {
      b.onclick = () => {
        const id = b.dataset["rm"] ?? "";
        askConfirm(`移除成员 ${id}？`, async () => {
          try { await ntInvoke("neobot_member_remove", { id }); toast("已移除"); }
          catch (e) { toast(`移除失败：${String(e).slice(0, 60)}`); }
          void refreshMembers();
        });
      };
    });
    if (owner) paintHero(owner.id, owner.kind === "human" ? "本机成员 · 主" : "agent · 主");
  } catch { /* 忽略 */ }
}
$("s-me-save").addEventListener("click", async () => {
  const name = (($("s-me") as HTMLInputElement).value || "").trim().slice(0, 8);
  if (!name) return;
  try {
    await ntInvoke("neobot_member_add", { id: name, kind: "human" });
    await emit("neobot:set-me", { name });
    paintHero(name, "本机成员");
    toast(`已设为 ${name}`);
  } catch (e) { toast(`失败：${String(e).slice(0, 60)}`); }
  void refreshMembers();
});
($("s-me") as HTMLInputElement).addEventListener("keydown", (e) => {
  if (e.key === "Enter") ($("s-me-save") as HTMLButtonElement).click();
});
$("s-member-add").addEventListener("click", async () => {
  const id = (($("s-member") as HTMLInputElement).value || "").trim().slice(0, 8);
  if (!id) return;
  try { await ntInvoke("neobot_member_add", { id, kind: "human" }); toast("已添加"); }
  catch (e) { toast(`失败：${String(e).slice(0, 60)}`); }
  ($("s-member") as HTMLInputElement).value = "";
  void refreshMembers();
});
async function refreshControl(): Promise<void> {
  try {
    const holder = await ntInvoke<string | null>("neobot_control_status");
    $("s-control").textContent = holder ? `被 ${holder} 接管` : "空闲";
  } catch { $("s-control").textContent = "未知"; }
}
$("s-take").addEventListener("click", async () => {
  const name = (($("s-me") as HTMLInputElement).value || "").trim() || "owner";
  try { await ntInvoke("neobot_control_take", { holder: name }); toast("已接管"); }
  catch (e) { toast(`失败：${String(e).slice(0, 60)}`); }
  void refreshControl();
});
$("s-release").addEventListener("click", async () => {
  const name = (($("s-me") as HTMLInputElement).value || "").trim() || "owner";
  try { await ntInvoke("neobot_control_release", { holder: name }); toast("已交棒"); }
  catch (e) { toast(`失败：${String(e).slice(0, 60)}`); }
  void refreshControl();
});

// ─── 主题（模板七选 + 七色微调 + 字体；定义见 theme.ts，主窗口实时生效） ───
const TOKEN_LABEL: Record<keyof ThemeTokens, string> = {
  skype: "主", deep: "深", ink: "墨", coral: "珊", gold: "金", whisper: "雾", avail: "活",
};
let curName = "默认";
let curTokens: ThemeTokens = { ...BUILTIN_THEMES["default"].tokens };
function emitTheme(): void {
  void emit("neobot:theme", { name: curName, tokens: { ...curTokens } });
}
function paintTemplates(): void {
  const box = $("s-themes");
  box.innerHTML = "";
  for (const [id, t] of Object.entries(BUILTIN_THEMES)) {
    const b = document.createElement("button");
    b.className = "btn" + (t.label === curName ? " primary" : "");
    b.dataset["theme"] = id;
    b.textContent = t.label;
    b.title = t.label;
    b.onclick = () => {
      curName = t.label;
      curTokens = { ...t.tokens };
      paintTemplates();
      paintColors();
      emitTheme();
      toast(`主题：${t.label}`);
    };
    box.append(b);
  }
}
function paintColors(): void {
  const box = $("s-colors");
  box.innerHTML = "";
  (Object.keys(TOKEN_LABEL) as Array<keyof ThemeTokens>).forEach((k) => {
    const label = document.createElement("span");
    label.className = "cinput";
    label.title = k;
    const dot = document.createElement("i");
    dot.textContent = TOKEN_LABEL[k];
    const input = document.createElement("input");
    input.type = "color";
    input.value = curTokens[k];
    input.oninput = () => {
      curTokens[k] = input.value;
      curName = "自定";
      paintTemplates();
      emitTheme();
    };
    label.append(dot, input);
    box.append(label);
  });
}
function isHex6(s: string): boolean { return /^#[0-9a-fA-F]{6}$/.test(s); }
paintTemplates();
paintColors();
let wantExport = false;
$("s-theme-export").addEventListener("click", () => { wantExport = true; void emit("neobot:theme-query", {}); });
void listen<{ name: string; tokens: ThemeTokens }>("neobot:theme-state", (e) => {
  if (!wantExport) return;
  wantExport = false;
  const blob = new Blob([JSON.stringify(e.payload, null, 2)], { type: "application/json" });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob); a.download = `neobot-theme-${e.payload.name}.json`; a.click();
});
// 启动时向主窗口要当前主题，种子化本地调节器（主窗口每次保存即真相，故可复用）。
let seeded = false;
void listen<{ name: string; tokens: Partial<ThemeTokens> }>("neobot:theme-state", (e) => {
  if (seeded) return;
  seeded = true;
  curName = e.payload.name || curName;
  for (const k of Object.keys(TOKEN_LABEL) as Array<keyof ThemeTokens>) {
    const v = e.payload.tokens[k];
    if (typeof v === "string" && isHex6(v)) curTokens[k] = v;
  }
  paintTemplates();
  paintColors();
});
void emit("neobot:theme-query", {});
$("s-theme-import").addEventListener("click", () => ($("s-theme-file") as HTMLInputElement).click());
($("s-theme-file") as HTMLInputElement).addEventListener("change", async (e) => {
  const f = (e.target as HTMLInputElement).files?.[0];
  if (!f) return;
  try {
    const j = JSON.parse(await f.text()) as { name?: string; tokens?: Record<string, string> };
    const tokens = j.tokens ?? {};
    for (const k of ["skype", "deep", "ink", "coral", "gold", "whisper", "avail"] as const) {
      if (!isHex6(tokens[k] ?? "")) { toast(`主题缺合法的 ${k}（#rrggbb）`); return; }
    }
    const name = (j.name || f.name.replace(/\.json$/i, "") || "自定").slice(0, 16);
    curName = name;
    for (const k of ["skype", "deep", "ink", "coral", "gold", "whisper", "avail"] as const) {
      curTokens[k] = tokens[k] as string;
    }
    paintTemplates();
    paintColors();
    emitTheme();
    toast(`主题已导入：${name}`);
  } catch { toast("主题解析失败", "err"); }
});
type Prefs = { density: string; fontsize: string; motion: string; msgwidth: string; fontface: string; bubble: string; dark: string; enterkey: string; nbanner: string; nview: string; nsound: string; reads: string; autodl: string; wall: string };
const prefs: Prefs = { density: "comfortable", fontsize: "md", motion: "full", msgwidth: "normal", fontface: "system", bubble: "snow", dark: "light", enterkey: "enter", nbanner: "on", nview: "full", nsound: "on", reads: "on", autodl: "lazy", wall: "snow" };
const CARD_KEY: Record<string, keyof Prefs> = {
  "s-density": "density",
  "s-font": "fontsize",
  "s-motion": "motion",
  "s-width": "msgwidth",
  "s-face": "fontface",
  "s-bubble": "bubble",
  "s-dark": "dark",
  "s-enter": "enterkey",
  "s-nbanner": "nbanner",
  "s-nview": "nview",
  "s-nsound": "nsound",
  "s-reads": "reads",
  "s-autodl": "autodl",
  "s-wall": "wall",
};
// 字体分段按 FONT_FACES 渲染（与模板同源，避免硬编码漂移）。
(function paintFaces() {
  const box = $("s-face");
  box.innerHTML = "";
  for (const f of FONT_FACES) {
    const b = document.createElement("button");
    b.dataset["v"] = f.id;
    b.textContent = f.label;
    box.append(b);
  }
})();
function paintCards(): void {
  for (const [card, key] of Object.entries(CARD_KEY)) {
    document.querySelectorAll<HTMLButtonElement>(`#${card} button`).forEach((b) => {
      b.classList.toggle("on", b.dataset["v"] === prefs[key]);
    });
  }
}
for (const [card, key] of Object.entries(CARD_KEY)) {
  document.querySelectorAll<HTMLButtonElement>(`#${card} button`).forEach((b) => {
    b.onclick = () => {
      prefs[key] = b.dataset["v"] ?? prefs[key];
      paintCards();
      void emit("neobot:prefs", { ...prefs });
    };
  });
}
void listen<Prefs>("neobot:prefs-state", (e) => {
  Object.assign(prefs, e.payload);
  paintCards();
});
void emit("neobot:prefs-query", {});
// 存储用量（本地估算只读展示）。
(function paintStorage() {
  const el = $("s-storage");
  const fmt = (n: number) => n > 1048576 ? `${(n / 1048576).toFixed(1)} MB` : `${Math.max(1, Math.round(n / 1024))} KB`;
  let ls = 0;
  try { for (let i = 0; i < localStorage.length; i++) { const k = localStorage.key(i); if (k) ls += (localStorage.getItem(k) ?? "").length * 2; } } catch { ls = 0; }
  if (navigator.storage?.estimate) {
    navigator.storage.estimate().then((e) => { el.textContent = `本地 ${fmt(ls)} · 浏览器配额 ${fmt(e.quota ?? 0)}（已用 ${fmt(e.usage ?? 0)}）`; }).catch(() => { el.textContent = `本地 ${fmt(ls)}`; });
  } else {
    el.textContent = `本地 ${fmt(ls)}`;
  }
})();
void listen<string>("neobot:backup-data", (e) => {
  $("s-preview").textContent = e.payload.slice(0, 2000);
  const blob = new Blob([e.payload], { type: "application/json" });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob); a.download = "neobot-backup.json"; a.click();
});
$("s-import").addEventListener("click", () => ($("s-file") as HTMLInputElement).click());
($("s-file") as HTMLInputElement).addEventListener("change", async (e) => {
  const f = (e.target as HTMLInputElement).files?.[0];
  if (!f) return;
  const text = await f.text();
  try {
    const j = JSON.parse(text) as { app?: string };
    if (j.app !== "neobot") { $("s-preview").textContent = "导入失败：非 neobot 备份"; return; }
  } catch { $("s-preview").textContent = "导入失败：JSON 解析失败"; return; }
  askConfirm("覆盖主窗口会话/队友/提示词库？", () => void emit("neobot:backup-import", { text }));
});
void listen<string>("neobot:backup-done", (e) => { ($("s-preview") as HTMLElement).textContent = e.payload; });
$("s-clear").addEventListener("click", () => {
  askConfirm("清空主窗口会话转录？（任务/审计不受影响）", () => void emit("neobot:msgs-clear", {}));
});

// ─── 模型管理 ───
// ─── 模型管理：第三方端点 + 引擎自检 + 模型池 ───
interface ProviderItem { name: string; base_url: string; key_env: string; model: string; enabled: boolean; }
async function refreshProviders(): Promise<void> {
  const box = $("s-prov-list");
  try {
    // 内置 neotrix 端点自动直连，不在此展示（池子自动聚合 + 自动刷新）。
    const rows = (await ntInvoke<ProviderItem[]>("neobot_providers")).filter((p) => p.name !== "neotrix");
    box.innerHTML = rows.length === 0 ? `<li class="empty-note">暂无自加端点 — 上面手动添加；neotrix 池自动在席</li>` : "";
    for (const p of rows) {
      const li = document.createElement("li");
      li.innerHTML = `<b>${esc(p.name)}</b> <span class="muted">${esc(p.base_url)}${p.model ? ` · ${esc(p.model)}` : ""}${p.key_env ? ` · key:$${esc(p.key_env)}` : ""}</span> <button class="chip-btn" data-tg="${esc(p.name)}">${p.enabled ? "停用" : "启用"}</button> <button class="chip-btn" data-rmprov="${esc(p.name)}">移除</button>`;
      box.appendChild(li);
    }
    box.querySelectorAll<HTMLButtonElement>("[data-tg]").forEach((b) => {
      b.onclick = async () => {
        const cur = rows.find((r) => r.name === b.dataset["tg"]);
        // `?? ""` 是纯类型收窄：`[data-tg]` 选择器已保证该属性存在，运行时不改。
        try { await ntInvoke("neobot_provider_toggle", { name: b.dataset["tg"] ?? "", enabled: !cur?.enabled }); toast("已切换"); }
        catch (e) { toast(`失败：${String(e).slice(0, 60)}`); }
        void refreshProviders();
      };
    });
    box.querySelectorAll<HTMLButtonElement>("[data-rmprov]").forEach((b) => {
      b.onclick = () => {
        const name = b.dataset["rmprov"] ?? "";
        askConfirm(`移除端点 ${name}？`, async () => {
          try { await ntInvoke("neobot_provider_remove", { name }); toast("已移除"); }
          catch (e) { toast(`失败：${String(e).slice(0, 60)}`); }
          void refreshProviders();
        });
      };
    });
  } catch { box.innerHTML = `<li class="empty-note">端点加载失败</li>`; }
}
$("s-prov-add").addEventListener("click", async () => {
  const name = (($("s-prov-name") as HTMLInputElement).value || "").trim();
  const baseUrl = (($("s-prov-base") as HTMLInputElement).value || "").trim();
  if (!name || !baseUrl) { toast("名字和地址必填"); return; }
  try {
    await ntInvoke("neobot_provider_add", {
      name,
      baseUrl,
      keyEnv: (($("s-prov-key") as HTMLInputElement).value || "").trim(),
      model: (($("s-prov-model") as HTMLInputElement).value || "").trim(),
    });
    toast("已保存");
  } catch (e) { toast(`失败：${String(e).slice(0, 80)}`); }
  void refreshProviders();
});
$("s-models").addEventListener("click", () => void refreshModels());
// 模型池自动刷新（模型页可见时 12s 一轮；串行 guard 防重叠）。
let modelsRefreshing = false;
async function autoRefreshModels(): Promise<void> {
  const pane = document.querySelector<HTMLElement>('[data-pane="models"]');
  if (!pane || pane.classList.contains("hidden") || modelsRefreshing) return;
  modelsRefreshing = true;
  try {
    await refreshModels();
    await refreshProviders();
  } finally {
    modelsRefreshing = false;
  }
}
setInterval(() => void autoRefreshModels(), 12_000);
async function refreshModels(): Promise<void> {
  const box = $("s-model-list");
  try {
    const rows = await ntInvoke<Array<{ id: string; owner: string; source: string }>>("neobot_models");
    box.innerHTML = rows.length === 0 ? `<li class="empty-note">模型池为空（端点不可达或未配置）</li>` : "";
    for (const m of rows.slice(0, 20)) {
      const li = document.createElement("li");
      li.innerHTML = `<b>${esc(m.id)}</b> <span class="muted">${esc(m.owner)} · ${esc(m.source)}</span>`;
      box.appendChild(li);
    }
  } catch { box.innerHTML = `<li class="empty-note">需 HTTP 引擎（NEOBOT_* env）</li>`; }
}

// ─── 启动：导航图标 + 数据 ───
void refreshProviders();
interface CoreStatus {
  paired: boolean; online: boolean; model: string; detail: string;
  base_url?: string; models?: number; latency_ms?: number;
  crystal_version?: string; tool_count?: number;
}
void (async () => {
  try {
    const s = await ntInvoke<CoreStatus>("neobot_core_status");
    if (!s.paired) {
      $("s-engine").textContent = "本地回显（未嵌入晶体核心）";
    } else {
      const parts = [s.model];
      if (typeof s.crystal_version === "string" && s.crystal_version) parts.push(`晶体 ${s.crystal_version}`);
      if (typeof s.tool_count === "number") parts.push(`工具 ${s.tool_count}`);
      if (s.detail) parts.push(s.detail);
      $("s-engine").textContent = parts.join(" · ");
    }
  } catch {
    try {
      const out = await ntInvoke<string>("neobot_doctor");
      const m = /engine=(\S(?:.*?\S)?) tasks=/.exec(out);
      if (m) $("s-engine").textContent = m[1];
    } catch { /* 忽略 */ }
  }
})();
document.querySelectorAll<HTMLButtonElement>("#setnav button").forEach((b) => {
  const name = b.dataset["icon"] as "user" | "sliders" | "chart" | "globe" | "info";
  b.insertAdjacentHTML("afterbegin", icon(name, 15));
});
void refreshMembers();
void refreshControl();
void (async () => { try { await ntInvoke("neobot_doctor"); } catch { /* 忽略 */ } })();

// ─── 晶体：重载池子 + capabilities 特性小字（调不到隐藏整行，无声降级） ───
document.getElementById("s-reload")?.addEventListener("click", async () => {
  try {
    const msg = await ntInvoke<string>("neobot_core_reload");
    toast(typeof msg === "string" && msg ? msg.slice(0, 90) : "池子已重载");
  } catch (e) { toast(`重载失败：${String(e).slice(0, 80)}`, "err"); }
});
void (async () => {
  try {
    const caps = await ntInvoke<{ features?: Record<string, boolean> | string[] }>("neobot_core_capabilities");
    const feats = caps.features;
    let text = "";
    if (Array.isArray(feats)) text = feats.join(" · ");
    else if (feats && typeof feats === "object") {
      text = Object.entries(feats).filter(([, v]) => v).map(([k]) => k).join(" · ");
    }
    const el = document.getElementById("s-caps");
    if (el) el.textContent = text || "—";
  } catch {
    document.getElementById("s-crystal-card")?.classList.add("hidden");
    document.getElementById("s-crystal-label")?.classList.add("hidden");
  }
})();


// ─── IM 渠道（吸收 dsh-im 的多渠道接入；凭证只存环境变量名） ───
//
// 三条界律：
// 1. **前端永远不碰 token 值**。这里只有 `token_env`（变量名）；
// 2. 访问模式的 fail-closed 要**说给人听** —— 「白名单 + 空名单 = 全拒」
//    反直觉，界面上必须写明，否则用户会以为机器人对所有人开放；
// 3. 「收一轮」是手动动作，不在后台偷偷轮询。

interface ChannelItem {
  id: string; title: string; enabled: boolean;
  access_mode: string; poll_secs: number; available: boolean;
}
interface ChannelBotItem {
  channel: string; bot_id: string; alias: string; token_env: string;
  conversation_id: string | null; model: string; allow_list: string[];
  access_mode: string; last_seen: string | null;
}

// 本文件是独立窗口入口，没从 core 引入 `$`；自带一个带泛型的取值器。
const $ch = <T extends HTMLElement = HTMLElement>(id: string): T | null =>
  document.getElementById(id) as T | null;
let chSelected = "";

const MODE_TEXT: Record<string, string> = {
  open: "开放 · 谁都能用",
  allow: "白名单 · 只放行名单内（名单为空 = 谁都不放行）",
  dm_only: "仅私聊 · 群消息一律不理",
};

function paintChannels(rows: ChannelItem[]): void {
  const list = $ch("ch-list");
  if (!list) return;
  if (!chSelected && rows.length) chSelected = rows[0].id;
  list.innerHTML = rows.length === 0
    ? '<li class="muted">没有渠道</li>'
    : rows.map((row) => `<li class="rowline">
        <span class="k">${esc(row.title)}</span>
        <span class="muted">${esc(MODE_TEXT[row.access_mode] ?? row.access_mode)}</span>
        <button class="btn" data-ch="${esc(row.id)}">${chSelected === row.id ? "● " : ""}选</button>
        <button class="btn" data-ch-toggle="${esc(row.id)}" data-on="${row.enabled ? "1" : "0"}">${row.enabled ? "停用" : "启用"}</button>
      </li>`).join("");
  const current = rows.find((row) => row.id === chSelected);
  const hint = $ch("ch-mode-hint");
  if (hint && current) hint.textContent = MODE_TEXT[current.access_mode] ?? current.access_mode;
}

async function refreshChannels(): Promise<void> {
  try {
    paintChannels(await ntInvoke<ChannelItem[]>("neobot_channels"));
    await refreshBots();
  } catch (e) {
    const list = $ch("ch-list");
    if (list) list.innerHTML = `<li class="muted">读取失败：${esc(String(e).slice(0, 80))}</li>`;
  }
}

async function refreshBots(): Promise<void> {
  const list = $ch("ch-bot-list");
  if (!list) return;
  if (!chSelected) { list.innerHTML = '<li class="muted">先选一个渠道</li>'; return; }
  try {
    const rows = await ntInvoke<ChannelBotItem[]>("neobot_channel_bots", { channel: chSelected });
    list.innerHTML = rows.length === 0
      ? '<li class="muted">这个渠道下还没有机器人</li>'
      : rows.map((b) => `<li class="rowline">
          <span class="k">${esc(b.alias || b.bot_id)}</span>
          <span class="mono muted">${esc(b.bot_id)}</span>
          <span class="muted">${esc(b.token_env || "（未设 token 变量）")}</span>
          <span class="muted">${b.allow_list.length ? `白名单 ${b.allow_list.length} 人` : "名单空"}</span>
          <span class="muted">${b.last_seen ? `最近 ${esc(b.last_seen.slice(0, 19))}` : "从未收发"}</span>
          <button class="btn" data-alias="${esc(b.bot_id)}">改别名</button>
          <button class="btn danger" data-rm="${esc(b.bot_id)}">摘掉</button>
        </li>`).join("");
  } catch (e) {
    list.innerHTML = `<li class="muted">读取失败：${esc(String(e).slice(0, 80))}</li>`;
  }
}

function wireChannels(): void {
  $ch("ch-refresh")?.addEventListener("click", () => void refreshChannels());
  $ch("ch-bot-add")?.addEventListener("click", async () => {
    if (!chSelected) { toast("先选一个渠道", "err"); return; }
    const botId = $ch<HTMLInputElement>("ch-bot-id")?.value.trim() ?? "";
    if (!botId) { toast("填一下 bot id", "err"); return; }
    const tokenEnv = $ch<HTMLInputElement>("ch-token-env")?.value.trim() ?? "";
    try {
      await ntInvoke("neobot_channel_bot_upsert", {
        channel: chSelected,
        botId,
        tokenEnv,
        model: $ch<HTMLInputElement>("ch-model")?.value.trim() ?? "",
        allowList: $ch<HTMLInputElement>("ch-allow")?.value.trim() ?? "",
        conversationId: null,
      });
      toast("已保存绑定");
      await refreshBots();
    } catch (e) { toast(String(e), "err"); }
  });
  $ch("ch-probe")?.addEventListener("click", async () => {
    if (!chSelected) { toast("先选一个渠道", "err"); return; }
    const status = $ch("ch-status");
    if (status) status.textContent = "探活中…";
    try {
      const h = await ntInvoke<{ ok: boolean; detail: string; info: string; failures: number }>(
        "neobot_channel_probe", { channel: chSelected },
      );
      if (status) {
        status.textContent = h.ok
          ? `在线 ${h.info}`
          : `不通：${h.detail.slice(0, 90)}${h.failures >= 3 ? `（连续失败 ${h.failures} 次，多半是 token 或网络）` : ""}`;
      }
    } catch (e) { if (status) status.textContent = String(e).slice(0, 90); }
  });
  $ch("ch-poll")?.addEventListener("click", async () => {
    if (!chSelected) { toast("先选一个渠道", "err"); return; }
    const status = $ch("ch-status");
    if (status) status.textContent = "收取中…（长轮询最多 25s）";
    try {
      const r = await ntInvoke<{ skipped: string | null; received: number; ignored: number; sent: number; deferred: number }>(
        "neobot_channel_poll_once", { channel: chSelected },
      );
      if (status) {
        status.textContent = r.skipped
          ? `跳过：${r.skipped}`
          : `收到 ${r.received}（忽略 ${r.ignored}）· 发出 ${r.sent} · 补发 ${r.deferred}`;
      }
      await refreshBots();
    } catch (e) { if (status) status.textContent = String(e).slice(0, 90); }
  });
  // 列表里的行内按钮（事件委托，因为列表是重绘的）
  $ch("ch-list")?.addEventListener("click", async (ev) => {
    const t = ev.target as HTMLElement;
    const sel = t.closest<HTMLElement>("[data-ch]");
    if (sel) { chSelected = sel.dataset["ch"] ?? ""; await refreshChannels(); return; }
    const tg = t.closest<HTMLElement>("[data-ch-toggle]");
    if (tg) {
      try {
        // `?? ""` 是纯类型收窄：`[data-ch-toggle]` 选择器已保证该属性存在，运行时不改。
        await ntInvoke("neobot_channel_toggle", { id: tg.dataset["chToggle"] ?? "", enabled: tg.dataset["on"] !== "1" });
        await refreshChannels();
      } catch (e) { toast(String(e), "err"); }
    }
  });
  $ch("ch-bot-list")?.addEventListener("click", async (ev) => {
    const t = ev.target as HTMLElement;
    const al = t.closest<HTMLElement>("[data-alias]");
    if (al) {
      const next = window.prompt("别名（留空 = 用平台原名）", "");
      if (next === null) return;
      try {
        await ntInvoke("neobot_channel_bot_alias", { channel: chSelected, botId: al.dataset["alias"] ?? "", alias: next });
        await refreshBots();
      } catch (e) { toast(String(e), "err"); }
      return;
    }
    const rm = t.closest<HTMLElement>("[data-rm]");
    if (rm) {
      if (!window.confirm("摘掉这个机器人？（它的会话与历史保留）")) return;
      try {
        await ntInvoke("neobot_channel_bot_remove", { channel: chSelected, botId: rm.dataset["rm"] ?? "" });
        await refreshBots();
      } catch (e) { toast(String(e), "err"); }
    }
  });
}

wireChannels();
void refreshChannels();
