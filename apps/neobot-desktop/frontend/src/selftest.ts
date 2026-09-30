/**
 * 新架构的自测 —— 与 `apps/neobot-desktop/frontend/src/selftest.ts` 同一约定
 * （纯逻辑、零 DOM、可 `node --experimental-strip-types` 直接跑）。
 *
 * 为什么必须可执行：这套东西是**契约**（插件注册、能力矩阵、宿主边界），
 * 契约出错不会编译报错，只会在某个宿主上「功能莫名不出现」。
 */

import { CapabilityRegistry } from "./plugin/contract.ts";
import { MemoryHost, defaultCapabilities } from "./host/host.ts";

export interface Failure { what: string; got?: unknown; want?: unknown; }
export async function runSelfTest(): Promise<Failure[]> {
const failures: Failure[] = [];
const ok = (cond: boolean, what: string): void => { if (!cond) failures.push({ what }); };
const eq = (a: unknown, b: unknown, what: string): void => {
  if (JSON.stringify(a) !== JSON.stringify(b)) failures.push({ what, got: a, want: b });
};

// ── ① 能力门控：未声明 ≠ 不支持，但两者都不可见且**必须给出理由** ──
{
  const r = new CapabilityRegistry();
  r.provide({ id: "chat", label: "对话", supported: true });
  r.provide({ id: "tray", label: "托盘", supported: false, reason: "浏览器无系统托盘" });

  const chat = r.verdict("chat");
  ok(chat.visible, "已声明且支持 ⇒ 可见");
  eq(chat.hint, undefined, "支持时不该有 hint");

  const tray = r.verdict("tray");
  ok(!tray.visible, "不支持 ⇒ 不可见");
  eq(tray.hint, "浏览器无系统托盘", "不支持必须给理由（否则用户以为是 bug）");

  const ghost = r.verdict("never-declared");
  ok(!ghost.visible, "未声明 ⇒ 不可见");
  ok((ghost.hint ?? "").includes("未声明"), "未声明要说清是「未声明」而不是「不支持」");
}

// ── ② 注册表自身的不变量 ──
{
  const r = new CapabilityRegistry();
  r.provide({ id: "a", label: "A", supported: true });
  // 支持却带 reason：自相矛盾，必须抛
  let threw = false;
  try { r.provide({ id: "a", label: "A", supported: true, reason: "不该有" }); } catch { threw = true; }
  ok(threw, "supported=true 却带 reason ⇒ 抛（自相矛盾不能进注册表）");

  // 结论相反的重复声明：抛
  threw = false;
  try { r.provide({ id: "a", label: "A", supported: false, reason: "x" }); } catch { threw = true; }
  ok(threw, "同一能力结论相反地重复声明 ⇒ 抛（不静默覆盖）");

  // 插件要了没人声明过的能力：抛
  threw = false;
  try {
    r.register({ self: { id: "p", name: "P", summary: "s", requires: { tauri: ["ghost"], browser: ["ghost"], node: ["ghost"] }, hosts: ["browser"] }, provides: [] });
  } catch { threw = true; }
  ok(threw, "插件要求未声明的能力 ⇒ 抛（否则界面出现幽灵开关）");

  threw = false;
  try {
    r.register({ self: { id: "p", name: "P", summary: "s", requires: { tauri: [], browser: [], node: [] }, hosts: ["browser"] }, provides: [] });
    r.register({ self: { id: "p", name: "P", summary: "s", requires: { tauri: [], browser: [], node: [] }, hosts: ["browser"] }, provides: [] });
  } catch { threw = true; }
  ok(threw, "插件重复注册 ⇒ 抛");
}

// ── ③ 插件可见性 + 缺能力时整体不出现（但内部仍逐项可查）──
{
  const r = new CapabilityRegistry();
  r.provide({ id: "chat", label: "对话", supported: true });
  r.provide({ id: "tray", label: "托盘", supported: true });
  r.provide({ id: "window-material", label: "窗口材质", supported: false, reason: "仅 macOS" });
  r.register({
    self: { id: "chat-panel", name: "对话", summary: "s", order: 10, hosts: ["browser", "tauri"], requires: { tauri: ["chat"], browser: ["chat"], node: ["chat"] } },
    provides: ["chat"],
  });
  r.register({
    self: { id: "window-panel", name: "窗口", summary: "s", order: 20, hosts: ["tauri"], requires: { tauri: ["window-material"], browser: ["window-material"], node: ["window-material"] } },
    provides: [],
  });
  r.register({
    self: { id: "off-panel", name: "关", summary: "s", order: 5, hosts: ["browser"], requires: { tauri: [], browser: [], node: [] } },
    provides: [],
    enabled: false,   // 关掉的插件：仍在注册表（可查），但不参与组装
  });

  eq(r.pluginsFor("browser").map((p) => p.id), ["chat-panel"], "browser 上只有启用的、声明支持 browser 的插件，且按 order 排");
  eq(r.pluginsFor("tauri").map((p) => p.id), ["chat-panel", "window-panel"], "tauri 上两个面板按 order 排");

  const pv = r.pluginVerdict("window-panel", "browser");
  ok(!pv.visible, "缺能力 ⇒ 面板整体不出现");
  eq(pv.missing, ["window-material"], "缺哪项要说清");
  ok((pv.hint ?? "").includes("窗口材质"), "面板级提示要带缺失能力名");
}

// ── ④ 宿主：能力矩阵按 kind 收窄，且状态可变可订阅 ──
{
  const browserCaps = defaultCapabilities("browser");
  const tauriCaps = defaultCapabilities("tauri");
  const g = (caps: { id: string; supported: boolean }[], id: string) => caps.find((c) => c.id === id)?.supported;
  eq(g(browserCaps, "local-model"), false, "浏览器无本地运行时");
  eq(g(tauriCaps, "local-model"), true, "Tauri 有");
  eq(g(browserCaps, "tray"), false, "浏览器无托盘");
  eq(g(tauriCaps, "tray"), true, "Tauri 有托盘");

  let notified = 0;
  const h = new MemoryHost({ kind: "tauri" }, tauriCaps);
  const off = h.subscribe(() => { notified += 1; });
  h.patch({ busy: true, localModel: true });
  eq(h.state().busy, true, "patch 生效");
  eq(notified, 1, "订阅者收到一次通知");
  off();
  h.patch({ busy: false });
  eq(notified, 1, "退订后不再收到");
}

// ── ⑤ 内存宿主能跑通「面板可见 + 动作可执行」这条完整链路 ──
{
  const h = new MemoryHost(
    { kind: "tauri", platform: "darwin", localModel: true },
    defaultCapabilities("tauri"),
  );
  h.caps.register({
    self: { id: "model-panel", name: "模型管理", summary: "管理端点", order: 10, hosts: ["tauri"], requires: { tauri: ["local-model"], browser: ["local-model"], node: ["local-model"] } },
    provides: ["local-model"],
  });
  eq(h.panels().map((p) => p.id), ["model-panel"], "本机且有本地模型 ⇒ 面板可见");
  await h.act("open-settings", { page: "models" });
  eq(h.acted(), ["open-settings"], "动作被记录（真实宿主在这里发 IPC）");
}

  // ── 2026-09-30 吸收 DSH-better-sidebar：注册契约的七条新纪律 ──
  console.log("  · 注册契约");
  {
    const r2 = new CapabilityRegistry();
    r2.provideAll(defaultCapabilities("tauri"));

    // ① 注销函数：作用域化且幂等
    const off = r2.register({
      self: { id: "p1", name: "一", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] } },
      provides: [],
    });
    eq(r2.pluginsFor("tauri").map((p) => p.id), ["p1"], "注册后可见");
    off();
    eq(r2.pluginsFor("tauri").map((p) => p.id), [], "注销后不可见");
    off();
    eq(r2.pluginsFor("tauri").map((p) => p.id), [], "重复注销幂等，不误删");

    // ② 旧注销不得删掉同名的新注册
    const offA = r2.register({
      self: { id: "p2", name: "二", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] } },
      provides: [],
    });
    offA();
    r2.register({
      self: { id: "p2", name: "二", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] } },
      provides: [],
    });
    offA();
    eq(r2.pluginsFor("tauri").map((p) => p.id), ["p2"], "旧注销不误删同名新注册");

    // ③ 崩溃隔离：谓词抛异常 ⇒ 不可用，且不掀翻调用方
    r2.register({
      self: {
        id: "boom", name: "炸", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] },
        available: () => { throw new Error("boom"); },
      },
      provides: [],
    });
    eq(r2.isAvailable("boom", "tauri"), false, "谓词抛异常按不可用处理（隔离生效）");
    eq(r2.menuFor("tauri").length >= 0, true, "一个坏插件不能让整份菜单崩掉");

    // ④ 谓词是活的：每次重新求值，不是快照
    let alive = false;
    r2.register({
      self: {
        id: "live", name: "活", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] },
        available: (ctx) => ctx.host === "tauri" && alive,
      },
      provides: [],
    });
    eq(r2.isAvailable("live", "tauri"), false, "alive=false 时不可用");
    alive = true;
    eq(r2.isAvailable("live", "tauri"), true, "alive=true 后立刻可用");

    // ⑤ hidden ≠ 不可用
    r2.register({
      self: { id: "hid", name: "隐", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] }, hidden: true },
      provides: [],
    });
    eq(r2.menuFor("tauri").map((p) => p.id).includes("hid"), false, "hidden 不进 + 菜单");
    eq(r2.isAvailable("hid", "tauri"), true, "hidden 仍可用（编辑器页签靠这个被文件唤起）");

    // ⑥ 缺席即启用（fail-open）
    eq(r2.isAvailable("p2", "tauri"), true, "enabled 缺席 = 启用");
    r2.register({
      self: { id: "off2", name: "关", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] } },
      provides: [], enabled: false,
    });
    eq(r2.isAvailable("off2", "tauri"), false, "显式 enabled:false 才禁用");

    // ⑦ single / dedupeKey
    r2.register({
      self: { id: "sing", name: "单", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] }, single: true },
      provides: [],
    });
    r2.trackTab({ type: "sing", id: "a", title: "A" });
    eq(r2.dedupeOf("sing", { type: "sing", id: "b", title: "B" })?.id, "a",
      "single=true：再开命中已有（聚焦而非复制）");
    r2.register({
      self: {
        id: "dk", name: "键", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] },
        dedupeKey: (t) => t.path,
      },
      provides: [],
    });
    r2.trackTab({ type: "dk", id: "x", title: "X", path: "/a.txt" });
    eq(r2.dedupeOf("dk", { type: "dk", id: "y", title: "Y", path: "/a.txt" })?.id, "x", "dedupeKey 按 path 去重");
    eq(r2.dedupeOf("dk", { type: "dk", id: "z", title: "Z", path: "/b.txt" }), undefined, "path 不同不去重");
    eq(r2.dedupeOf("dk", { type: "dk", id: "w", title: "W" }), undefined, "dedupeKey 返 undefined = 不去重");
  }

  // ── 2026-09-30 吸收 dsh-market：缺席 ≠ 空数组 ──
  console.log("  · 能力披露三态");
  {
    const r3 = new CapabilityRegistry();
    r3.register({
      self: { id: "p", name: "P", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] } },
      provides: [],
    });
    eq(r3.get("p")?.capabilities, undefined, "capabilities 缺席 = 未扫描");
    const r4 = new CapabilityRegistry();
    r4.register({
      self: { id: "q", name: "Q", summary: "s", hosts: ["tauri"], requires: { tauri: [], browser: [], node: [] } },
      provides: [], capabilities: [],
    });
    eq(r4.get("q")?.capabilities, [], "capabilities:[] = 扫过未检出（与缺席不同）");
  }

  return failures;
}
