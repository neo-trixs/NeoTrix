import { K, load, save } from "./core";
import { ntInvoke } from "./invoke";

// ─── nt_login_kernel — 网页登录极致微内核 ───
// 职责只三件：站点表（ presets + 自加，localStorage ）→ 系统浏览器打开 →
// 灵魂验证登录态（agent_run，结论回写状态表）。
// DOM-free（调用方渲染），Tauri-ok（open 经 plugin-shell，F8 白名单）。
// 对话框 🌐 面板与智能看板“登录” filter 共用同一核、同一状态表。

export interface LoginSite { id: string; name: string; url: string; }
export type LoginStatus = "unknown" | "checking" | "logged" | "unlogged";
export interface LoginState { site_id: string; status: LoginStatus; evidence: string; ts: number; }

const PRESETS: Array<[string, string]> = [
  ["Google", "https://accounts.google.com/"],
  ["GitHub", "https://github.com/login"],
];

export function loginSites(): LoginSite[] {
  const list = load<LoginSite[]>(K.loginSites, []);
  if (list.length > 0) return list;
  const seed = PRESETS.map(([name, url], i) => ({ id: `preset-${i}`, name, url }));
  try { save(K.loginSites, seed); } catch { /* 忽略 */ }
  return seed;
}
export function saveLoginSites(s: LoginSite[]): void { save(K.loginSites, s); }

function stateMap(): Record<string, LoginState> {
  return load<Record<string, LoginState>>(K.loginStates, {});
}
export function loginState(id: string): LoginState {
  return stateMap()[id] ?? { site_id: id, status: "unknown", evidence: "", ts: 0 };
}
export function saveLoginState(st: LoginState): void {
  const m = stateMap();
  m[st.site_id] = st;
  save(K.loginStates, m);
}
export function loggedCount(): number {
  const m = stateMap();
  return Object.values(m).filter((s) => s.status === "logged").length;
}

/** F8 白名单：只放 http(s)（file:// / 自定义 scheme 直达系统 handler，一律拒）。 */
export function validLoginUrl(url: string): boolean {
  return /^https?:\/\//i.test(url.trim());
}

export function siteNameOf(url: string): string {
  try { return new URL(url).hostname.replace(/^www\./, "") || url; }
  catch { return url.slice(0, 24); }
}

/** URL 归位站点表（按 url 去重；对话/看板两路共用，状态天然共享）。 */
export function ensureLoginSite(url: string): LoginSite {
  const sites = loginSites();
  const hit = sites.find((s) => s.url === url);
  if (hit) return hit;
  const site: LoginSite = { id: `s${Date.now()}`, name: siteNameOf(url), url };
  sites.push(site);
  saveLoginSites(sites);
  return site;
}

/** 系统浏览器打开登录页（调用方保证已 validLoginUrl）。 */
export async function openLoginSite(url: string): Promise<void> {
  try {
    const { open } = await import("@tauri-apps/plugin-shell");
    await open(url);
  } catch { window.open(url, "_blank"); }
}

export function loginGoal(url: string): string {
  return `检查网页登录态：用浏览工具打开 ${url}，判断当前是否已登录。`
    + `找账号名/头像/退出/SignOut字样为已登录证据，找登录框/密码框/登录按钮为未登录证据。`
    + `只回答一行：已登录：<证据> 或 未登录：<所见>。`;
}

export interface AgentRunResult { status: string; output: string; trace: Array<{ kind: string; detail: string }>; model_used?: string; }

/** 验证登录态：agent 跑完 → 状态表落盘 → 返回 verdict 行。抛错由调用方呈现。
 * 失败不清 🌀 为 unknown（审计 B1：抛错不留 checking 卡死态）。 */
export async function verifyLoginSite(site: LoginSite): Promise<string> {
  saveLoginState({ site_id: site.id, status: "checking", evidence: "", ts: Date.now() });
  try {
    const r = await ntInvoke<AgentRunResult>("neobot_agent_run", { goal: loginGoal(site.url), context: "" });
    const text = `${r.output ?? ""}`.trim() || `（空输出，status=${r.status}）`;
    const logged = /^\s*已登录/.test(text);
    saveLoginState({
      site_id: site.id,
      status: logged ? "logged" : "unlogged",
      evidence: text.slice(0, 120),
      ts: Date.now(),
    });
    return text;
  } catch (e) {
    saveLoginState({
      site_id: site.id,
      status: "unknown",
      evidence: `验证失败：${String(e).slice(0, 80)}`,
      ts: Date.now(),
    });
    throw e;
  }
}
