/**
 * NeoBot 前端共享核心 — 壳状态与基础构件（无业务渲染，不依赖任何 feature 模块）。
 * 主题色见 theme.ts；图标见 icons.ts；表情见 moodball.ts。
 */


export interface TaskItem { id: string; title: string; status: string; claimed_by?: string | null; visibility: string; attempts?: number; conversation_id?: string | null; }
export interface AuditItem { at: string; actor: string; tool: string; decision: string; rule?: string | null; }
export interface ModelItem { id: string; owner: string; source: string; }
export interface CostActorRow { engine: string; model: string; actor: string; in_tokens: number; out_tokens: number; cost_usd: number; }
export interface PresenceItem { id: string; kind: string; presence: string; last_seen_secs: number; }
export interface MemberItem { id: string; kind: string; owner: boolean; }
export interface RoutineItem { name: string; interval_secs: number; owner: string; failures: number; disabled: number; next_run_at: number; instruction: string; }
export interface SkillItem { name: string; description: string; }
export interface AgentTraceItem { kind: string; detail: string; }
/** 回复路由三态（后端 `ReplyMode` 小写原串直消）。 */
export type ReplyMode = "direct" | "passthrough" | "fallback";
/** 单轮用量（input/output tokens ＋ 美元/微美元双写；`measured=false` 即未计量）。 */
export interface TurnUsage { input_tokens: number; output_tokens: number; cost_usd: number; cost_micros: number; measured: boolean; }
/** 每轮回复标签（后端 `TurnLabels` serde 同形；全可选降级，老消息无标签不炸）。 */
export interface TurnLabels { model: string; mode: ReplyMode; tools: string[]; usage: TurnUsage; }
export interface AgentMsg { output: string; trace: AgentTraceItem[]; model_used?: string; labels?: TurnLabels; }
export interface ChatMsg { role: "user" | "assistant" | "sys"; text: string; ts: number; atts?: AttInfo[]; node?: { task: string }; agent?: AgentMsg; taskId?: string; labels?: TurnLabels }
export interface AttInfo { id: string; kind: string; name: string; path: string; }
export interface ConvoItem { id: string; kind: string; title: string; members: string[]; task_count: number; last_active: string; muted: boolean; unread: number; }
export interface ModelSel { provider: string; model: string; }
export type View = "convos";
export interface PageNode { id: string; url: string; x: number; y: number; w: number; h: number; }
export interface BoardView { x: number; y: number; k: number; }
export interface VNode { key: string; kind: string; color: string; title: string; sub: string; sel: boolean; x: number; y: number; }

export const K = {
  msgs: "ntos_msgs_v1",
  drafts: "ntos_drafts_v1",
  mates: "ntos_mates_v1",
  starters: "ntos_starters_v1",
  me: "ntos_me_v1",
  view: "ntos_view_v1",
  selTask: "ntos_seltask_v1",
  selRoutine: "ntos_selroutine_v1",
  selSkill: "ntos_selskill_v1",
  boardFilter: "ntos_boardfilter_v1",
  pinned: "ntos_pinned_v1",
  threads: "ntos_threads_v1",
  selConvo: "ntos_selconvo_v1",
  model: "ntos_model_v1",
  theme: "ntos_theme_v1",
  pages: "ntos_pages_v1",
  boardView: "ntos_boardview_v1",
  boardPos: "ntos_boardpos_v1",
  loginSites: "ntos_loginsites_v1",
  loginStates: "ntos_loginstates_v1",
};

export const DEFAULT_STARTERS = [
  { name: "规划", prompt: "帮我把下面目标拆成 3 步可执行计划：" },
  { name: "报告", prompt: "把下面内容整理成一份简报（含结论/依据/下一步）：" },
  { name: "周报", prompt: "把下面要点扩成周报（完成/阻塞/下周）：" },
  { name: "总结", prompt: "用三句话总结下面内容：" },
  { name: "翻译", prompt: "把下面翻译成英文，保持术语：" },
];

export const STATUS_CN: Record<string, string> = { pending: "排队", running: "运行中", done: "完成", failed: "失败", cancelled: "已取消", blocked: "被拒", waiting: "等人" };

export function load<T>(key: string, fallback: T): T {
  try { const raw = localStorage.getItem(key); if (!raw) return fallback; return JSON.parse(raw) as T; }
  catch { return fallback; }
}
export function save(key: string, value: unknown): void {
  try { localStorage.setItem(key, JSON.stringify(value)); } catch { /* 配额满则丢 */ }
}
export function esc(s: string): string {
  // P0 审计 F3：引号必须转义——输出常拼进属性（data-mid/iframe src），模型 id 来自远端池。
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}

// ─── DOM ───

export const $ = (id: string) => document.getElementById(id) as HTMLElement;
export const thread = $("thread"), hero = $("hero"), input = $("input") as HTMLTextAreaElement;
export const sendBtn = $("btn-send") as HTMLButtonElement;

// ─── 壳状态（唯一可变源；各模块经 shell/cache 读写，禁止裸 let 复制） ───
export const shell = { running: false, stopRequested: false };
export let cache = { tasks: [] as TaskItem[], audits: [] as AuditItem[], costs: [] as CostActorRow[], models: [] as ModelItem[], routines: [] as RoutineItem[], skills: [] as SkillItem[], presence: [] as PresenceItem[], members: [] as MemberItem[], convos: [] as ConvoItem[] };

export interface AttRow { id: string; kind: string; name: string; path: string; size: number; }

export function currentView(): View { return "convos"; }

export function toast(text: string, kind: "ok" | "err" = "ok"): void {
  // 通知偏好：横幅开关＋预览三态＋提示音（纯前端本地偏好）。
  if (load<string>("ntos_pref_nbanner", "on") === "off" && kind === "ok") return;
  const view = load<string>("ntos_pref_nview", "full");
  const show = view === "full" ? text : view === "name" ? text.slice(0, 2) + "…" : "有新通知";
  if (load<string>("ntos_pref_nsound", "on") === "on") {
    try {
      const AC = window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
      if (AC) {
        const ac = new AC();
        const o = ac.createOscillator();
        const g = ac.createGain();
        o.connect(g); g.connect(ac.destination);
        o.frequency.value = kind === "err" ? 330 : 660;
        g.gain.value = 0.06;
        o.start();
        o.stop(ac.currentTime + 0.09);
        setTimeout(() => void ac.close(), 300);
      }
    } catch { /* 无音频设备时静默 */ }
  }
  const box = $("toasts");
  const el = document.createElement("div");
  el.className = `toast ${kind}`; el.textContent = show;
  box.appendChild(el);
  setTimeout(() => el.remove(), 2600);
}

// ─── 会话转录体（精简：ticks/悬停编辑删除复制/md-lite/日期线） ───
