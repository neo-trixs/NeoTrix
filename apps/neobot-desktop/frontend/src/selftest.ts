/**
 * 新架构的自测 —— 与 `apps/neobot-desktop/frontend/src/selftest.ts` 同一约定
 * （纯逻辑、零 DOM、可 `node --experimental-strip-types` 直接跑）。
 *
 * 为什么必须可执行：这套东西是**契约**（插件注册、能力矩阵、宿主边界），
 * 契约出错不会编译报错，只会在某个宿主上「功能莫名不出现」。
 */

import { CapabilityRegistry, validateActor, type ActorContext } from "./plugin/contract.ts";
import { MemoryHost, defaultCapabilities } from "./host/host.ts";
import { deriveIsland, islandLabel, islandIsLive } from "./ui/island-model.ts";
import { blockKey, summarizeTool, validatePanel, validateAnswer, hasPendingPanel,
  type DecisionPanel, type Block } from "./ui/block-model.ts";

/** 决策面板基线夹具：合法。任何不变量测试都从它的**变体**出发，
 *  而不是从空白出发 —— 空对象几乎总能通过，测不出约束。 */
function basePanel(): DecisionPanel {
  return {
    id: "p1", threadId: "t1", turnId: "tu1",
    candidateSetVersion: 1, type: "clarification",
    title: "选哪个", mode: "live",
    options: [{ id: "clar-a", label: "方案 A", details: ["更快"], sources: [{ title: "doc", url: "https://example.com/a" }] }],
  };
}

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


  // ── 2026-09-30 会话流块模型 ──────────────────────────────
  console.log("  · 块模型与决策面板不变量");
  {
    // 块 key 必须稳定且按 kind 区分
    const k1 = blockKey({ kind: "mark", text: "A" }, 0);
    eq(k1, "m:0:A", "mark 的 key 含内容与序号");
    eq(blockKey({ kind: "text", role: "bot", text: "x" }, 3), "t:3:bot", "text 的 key 含 role");
    eq(
      blockKey({ kind: "panel", panel: basePanel() }, 9),
      `p:${basePanel().id}:1`,
      "panel 的 key 用 id+版本（不看序号），版本变了 key 就变 ⇒ DOM 换新",
    );

    // 工具卡摘要
    eq(summarizeTool([]), "本轮没有工具调用", "空步骤给明确说法，不渲染空卡");
    eq(summarizeTool([{ kind: "a", detail: "" }, { kind: "b", detail: "" }]), "本轮工具 2 步",
      "全成功不带失败数");
    eq(
      summarizeTool([{ kind: "a", detail: "" }, { kind: "b", detail: "", failed: true }]),
      "本轮工具 2 步（1 步失败）",
      "有失败必须出现在摘要里（不展开也能看见）",
    );

    // 决策面板：合法基线必须通过
    eq(validatePanel(basePanel()), null, "合法面板通过校验");

    // 五条不变量逐条
    eq(
      validatePanel({ ...basePanel(), candidateSetVersion: 0 }),
      "candidateSetVersion 必须 ≥ 1",
      "版本号必须 ≥1",
    );
    eq(
      validatePanel({ ...basePanel(), options: [] }),
      "决策面板至少要一个选项",
      "空选项被拒",
    );
    const dup = basePanel();
    eq(
      validatePanel({ ...dup, options: [dup.options[0], dup.options[0]] }),
      "选项 id 重复：clar-a",
      "重复选项 id 被拒",
    );
    const noSrc = basePanel();
    eq(
      validatePanel({
        ...noSrc,
        type: "comparison",
        options: [{ id: "x", label: "X", details: [], sources: [] }],
      }),
      "比较型面板的选项「X」必须给出处",
      "比较型的选项必须给出处（否则「比较」只是在比没有依据的断言）",
    );
    eq(
      validatePanel({ ...basePanel(), type: "comparison", options: [
        { id: "1", label: "1", details: [], sources: [{ title: "s", url: "https://a" }] },
        { id: "2", label: "2", details: [], sources: [{ title: "s", url: "https://a" }] },
        { id: "3", label: "3", details: [], sources: [{ title: "s", url: "https://a" }] },
        { id: "4", label: "4", details: [], sources: [{ title: "s", url: "https://a" }] },
      ] }),
      "比较型面板最多 3 个选项（超过就失去「比较」的意义）",
      "比较型最多 3 项",
    );
    // ⛔ 非 http(s) 的「出处」不是出处 —— 拿到点击位上就是注入点
    eq(
      validatePanel({
        ...basePanel(),
        options: [{ id: "x", label: "X", details: [], sources: [{ title: "t", url: "javascript:alert(1)" }] }],
      }),
      "选项「X」的出处必须是 http(s)：javascript:alert(1)",
      "非 http(s) 出处被硬拒（这是注入点，不是可降级的警告）",
    );
    eq(
      validatePanel({ ...basePanel(), selectedId: "不存在" }),
      "selectedId 不在选项里：不存在",
      "selectedId 必须落在选项内",
    );

    // 过期作答：版本对不上必须拦住
    eq(validateAnswer(basePanel(), { optionId: "clar-a", candidateSetVersion: 1 }), null, "同版本作答通过");
    const stale = validateAnswer(basePanel(), { optionId: "clar-a", candidateSetVersion: 0 });
    eq(
      stale,
      "选项已更新（v0 → v1），请重新选择",
      "候选集版本对不上 ⇒ 拦下（否则骨架把旧选择当新选择，界面看不出异常）",
    );
    eq(
      validateAnswer(basePanel(), { optionId: "nope", candidateSetVersion: 1 }),
      "选项 nope 不在当前候选集里",
      "作答指向不存在的选项被拦",
    );

    // 非法面板**不能**被当作待答
    eq(
      hasPendingPanel([{ kind: "panel", panel: { ...basePanel(), options: [] } }]),
      undefined,
      "非法面板不算待答（界面得暴露问题，而不是渲染一张看着能用的坏卡）",
    );
    eq(
      hasPendingPanel([{ kind: "text", role: "bot", text: "hi" }, { kind: "panel", panel: basePanel() }])
        ?.id,
      basePanel().id,
      "合法面板被识别为待答",
    );
    eq(
      hasPendingPanel([{ kind: "text", role: "bot", text: "hi" }]),
      undefined,
      "没有面板时返回 undefined",
    );
  }


  // ── 2026-09-30 吸收 workdsh：主体身份必须由可信边界解析 ──
  console.log("  · 主体身份（防自报提权）");
  {
    const good: ActorContext = {
      principalId: "neo", requestId: "r1", resolvedBy: "tauri-host",
      sessionId: "s1", delegatedByPrincipalId: "root",
    };
    eq(validateActor(good), null, "宿主在可信边界解析的身份通过");

    // ⛔ 这是整组最重要的一条：resolvedBy 为空 = 身份未核实 = 自报。
    eq(
      validateActor({ ...good, resolvedBy: "" }),
      "resolvedBy 不能为空：身份必须由宿主身份提供方解析，不能自报",
      "resolvedBy 为空被硬拒（模型/前端自报身份的入口就在这里）",
    );
    eq(
      validateActor({ ...good, resolvedBy: "   " }),
      "resolvedBy 不能为空：身份必须由宿主身份提供方解析，不能自报",
      "空白串也视为未解析（trim 后判定，防 ' ' 绕过）",
    );
    eq(validateActor({ ...good, principalId: "" }), "principalId 不能为空", "空主体被拒");
    eq(validateActor({ ...good, requestId: "" }), "requestId 不能为空", "空 requestId 被拒（无法追溯）");
    eq(
      validateActor({ ...good, delegatedByPrincipalId: "neo" }),
      "delegatedByPrincipalId 不能等于 principalId（自委派没有意义）",
      "自委派被拒",
    );

    // 注册表：未设主体时必须仍是「未解析」，而不是编一个
    const rr = new CapabilityRegistry();
    eq(rr.currentActor().resolvedBy, "", "未设主体时 resolvedBy 为空（未解析），不是占位身份");
    eq(validateActor(rr.currentActor())?.startsWith("principalId"), true, "未设主体的 actor 不可用");

    // setActor：非法主体要被拒，且**不改现状**
    eq(rr.setActor({ principalId: "neo", requestId: "r", resolvedBy: "" }) !== null, true,
      "setActor 拒绝未解析身份");
    eq(rr.currentActor().resolvedBy, "", "被拒后现状不变（不会留下半套身份）");
    eq(rr.setActor(good), null, "setActor 接受合法身份");
    eq(rr.currentActor().resolvedBy, "tauri-host", "setActor 后可读回");
    eq(rr.currentActor().delegatedByPrincipalId, "root", "委派链被保留（审计要能区分代人与亲为）");

    // 谓词拿到的 actor 与注册表持有的是同一份
    let seen: ActorContext | undefined;
    rr.register({
      self: {
        id: "who", name: "谁", summary: "s", hosts: ["tauri"],
        requires: { tauri: [], browser: [], node: [] },
        available: (ctx) => { seen = ctx.actor; return true; },
      },
      provides: [],
    });
    rr.isAvailable("who", "tauri");
    eq(seen?.resolvedBy, "tauri-host", "谓词读到的是注册表持有的同一份 actor（不是另造一个）");
    eq(seen?.delegatedByPrincipalId, "root", "委派链对谓词可见");
  }


  // ── 2026-09-30 吸收 trycua/cua：reasoning 是一等块 + 工具输出形状统一 ──
  console.log("  · reasoning 块与工具产出物");
  {
    // ⛔ 这组是补测：上一版把 ReasoningBlock 加进联合类型却**没写任何断言**，
    //    变异验证时「把 reasoning 从联合里去掉」竟然 0 失败 —— 类型改了、
    //    行为没被任何东西钉住。补上。
    const r: Block = { kind: "reasoning", text: "先看目录再决定", tokens: 128 };
    eq(r.kind, "reasoning", "reasoning 是一等块（与 text/tool 平级，不是正文里的一段）");
    eq(blockKey(r, 2), "r:2:128", "reasoning 的 key 含 token 量级");
    eq(blockKey({ kind: "reasoning", text: "x" }, 0), "r:0:0", "无 token 时 key 仍稳定");

    // 产出物：形状统一 ⇒ 界面只有一套渲染器（cua: always a screenshot）
    const withOut: Block = {
      kind: "tool",
      steps: [{ kind: "screenshot", detail: "读取屏幕", outputs: { screenshot: "shot-1.png" } }],
    };
    eq(withOut.kind, "tool", "带产出物的 tool 块构造成功");
    const b = withOut as Extract<Block, { kind: "tool" }>;
    eq(Object.keys(b.steps[0].outputs ?? {}), ["screenshot"], "产出物是名字→值的记录（不认识具体类型）");
    // 无产出物时不应崩（渲染器会兜到空对象）
    const noOut: Block = { kind: "tool", steps: [{ kind: "a", detail: "b" }] };
    eq((noOut as Extract<Block, { kind: "tool" }>).steps[0].outputs, undefined, "无产出物时为 undefined 而非 {}");
    eq(
      summarizeTool((noOut as Extract<Block, { kind: "tool" }>).steps),
      "本轮工具 1 步",
      "无产出物不影响摘要",
    );
  }


  // ── 2026-09-30 活动岛：状态只能被推导，不能被手设 ──
  console.log("  · 活动岛状态推导");
  {
    const rc = new CapabilityRegistry();
    rc.provideAll(defaultCapabilities("tauri"));
    const O = { caps: rc, host: "tauri" as const, busy: false, seq: 1 };
    const panel = basePanel();

    eq(deriveIsland([], O).kind, "idle", "无块且不忙 ⇒ 空闲");

    // ⛔ 优先级：等决策 > 出错 > 运行中 > 空闲。
    //    球在**用户**那边时显示「运行中」，用户会以为系统还在算而继续等 ——
    //    那是最坏的错，因为他等不到任何后续。
    eq(
      deriveIsland([{ kind: "panel", panel }], { ...O, busy: true }).kind,
      "awaiting",
      "有决策面板时，即使忙也显示 awaiting（优先级高于 running）",
    );
    eq(
      deriveIsland([{ kind: "panel", panel }, { kind: "system", level: "error", text: "炸了" }], O).kind,
      "awaiting",
      "awaiting 高于 error（先处理球在我这边的）",
    );
    eq(
      deriveIsland([{ kind: "system", level: "error", text: "炸了" }], { ...O, busy: true }).kind,
      "error",
      "error 高于 running（出错时不该显示运行中）",
    );
    eq(deriveIsland([], { ...O, busy: true }).kind, "running", "仅忙 ⇒ running");

    // 非法面板不算待答 ⇒ 岛不会因为一个坏面板就宣称「等你选择」
    eq(
      deriveIsland([{ kind: "panel", panel: { ...panel, options: [] } }], O).kind,
      "idle",
      "非法面板不算待答（界面不能宣称在等一个不存在的选择）",
    );

    // 能力门控：缺能力 ⇒ unavailable 且带**注册表给的**理由
    const rc2 = new CapabilityRegistry();
    rc2.provideAll(defaultCapabilities("browser"));   // browser 无 local-model，但有 chat
    // ⛔ 不能先 provideAll(defaultCapabilities("tauri")) 再把 chat 声明为不支持：
    //    注册表会抛「被重复声明且结论相反」—— 那是它自己的不变量在正常工作，
    //    是**这个夹具**写错了。正确做法是从空表只声明需要的那一条。
    const noChat = new CapabilityRegistry();
    noChat.provide({ id: "chat", label: "对话", supported: false, reason: "宿主禁用了对话" });
    eq(
      deriveIsland([], { caps: noChat, host: "tauri", busy: true, seq: 1 }),
      { kind: "unavailable", label: "运行一轮", reason: "宿主禁用了对话" },
      "缺能力 ⇒ unavailable，理由取自注册表而不是本文件编的",
    );

    // 文案与 a11y 播报
    eq(islandLabel({ kind: "idle" }), "空闲", "idle 文案");
    eq(islandLabel({ kind: "awaiting", panel, seq: 1 }), "等你选择", "awaiting 文案点明是用户在等");
    eq(islandLabel({ kind: "unavailable", label: "X", reason: "Y" }), "X · 不可用", "unavailable 文案");
    eq(islandIsLive({ kind: "error", label: "e" }), "alert", "出错要 alert");
    eq(islandIsLive({ kind: "awaiting", panel, seq: 1 }), "status", "等决策要 status");
    eq(islandIsLive({ kind: "running", label: "r", seq: 1 }), "none",
      "运行中**不播报**（否则读屏每帧念一次）");
  }


  // ── 决策面板「已回答」不算待办（实测发现的 bug） ──
  console.log("  · 面板已回答后不算待办");
  {
    const p0 = basePanel();
    eq(hasPendingPanel([{ kind: "panel", panel: p0 }])?.id, p0.id, "未回答的面板算待办");
    // ⚠️ 已回答的面板**不算**待办：否则活动岛会一直说「等你选择」，
    //    用户以为没提交成功就会再点一次。
    const answered: Block = { kind: "panel", panel: { ...p0, selectedId: "clar-a" } };
    eq(hasPendingPanel([answered]), undefined, "已回答的面板不算待办");
    // 多个面板：只有**最后一个未答的**算待办
    const p1 = { ...p0, id: "p2" };
    eq(
      hasPendingPanel([answered, { kind: "panel", panel: p1 }])?.id,
      "p2",
      "多个面板时取最后一个未答的",
    );
    // 非法面板仍然不算待办（覆盖在优先级上）
    eq(
      hasPendingPanel([{ kind: "panel", panel: p1 }, { kind: "panel", panel: { ...p0, options: [] } }]),
      undefined,
      "流末尾是非法面板 ⇒ 不当作待答（暴露问题优先）",
    );
    // 活动岛随之变化
    const rc2 = new CapabilityRegistry();
    rc2.provideAll(defaultCapabilities("tauri"));
    const O2 = { caps: rc2, host: "tauri" as const, busy: false, seq: 1 };
    eq(deriveIsland([answered], O2).kind, "idle", "回答后面板 ⇒ 岛回到空闲（不是等你选择）");
    eq(deriveIsland([{ kind: "panel", panel: p1 }], O2).kind, "awaiting", "还有未答面板 ⇒ 岛显示等你选择");
  }

  return failures;
}
