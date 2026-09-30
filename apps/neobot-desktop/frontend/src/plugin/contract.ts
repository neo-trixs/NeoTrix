/**
 * 插件契约 —— 吸收自 DSH 的「Everything is a Plugin」。
 *
 * # 这份契约解决什么
 *
 * NeoTrix 是骨架（能力提供者），NeoBot 是对外沟通界面（消费方）。要让「界面」
 * 能随骨架一起长，界面就不能把骨架的能力写死 —— 它得**声明**自己要什么，
 * 由宿主**如实告知**能不能给。
 *
 * 这就是 `CapabilityGate`：一份**数据**（不是 `if (isMac)`）。
 *   - 界面在任意宿主、任意时刻都能算出「我该显示什么」
 *   - 换宿主（浏览器 / Tauri / 未来的 OS 外壳）不用改组件代码
 *   - 「为什么这个开关没出现」有确定答案，而不是靠猜
 *
 * DSH Desktop 的做法是把这个声明直接写在设置组件的 props 上
 * （`capabilities={{ windowModes: false, updates: false, ... }}`）。
 * 我把它**提取一层**成独立注册表：声明归插件，判定归宿主，组件只读结果。
 */

/** 宿主种类。界面不认平台，只认这个。 */
export type HostKind = "browser" | "tauri" | "node";

/**
 * 一项能力。
 *
 * 注意 `reason`：不能只给「能不能」，还要能回答「为什么不能」——
 * 否则界面只能把功能默默藏起来，用户以为是 bug。
 */
export interface Capability {
  /** 程序里用的稳定名。改名 = 破坏契约。 */
  readonly id: string;
  /** 界面展示用（中文）。 */
  readonly label: string;
  /** 宿主是否支持。 */
  readonly supported: boolean;
  /** 不支持时的原因（supported 为 true 时必须为空）。 */
  readonly reason?: string;
  /** 支持时的额外前提（例：需要已连上服务端）。 */
  readonly detail?: string;
}

/** 插件在某一宿主上的自述。 */
export interface PluginSelf {
  /** 插件 id，kebab-case，全局唯一。 */
  readonly id: string;
  /** 界面展示名。 */
  readonly name: string;
  /** 一句话说明。 */
  readonly summary: string;
  /** 这个插件在**各宿主**上要什么。 */
  readonly requires: Readonly<Record<HostKind, readonly string[]>>;
  /** 这个插件在哪些宿主上出现。 */
  readonly hosts: readonly HostKind[];
  /**
   * 面板在侧栏里的排序，小的在前。默认 100。
   *
   * 取自 DSH-better-sidebar `TabDescriptor.order`（同为 100 默认）。
   */
  readonly order?: number;
  /** 图标名（对应 icons.ts 的 IconName）。 */
  readonly icon?: string;
  /**
   * 动态可用性谓词（可选）。给出时**取代**静态 `hosts`/`requires` 之外的
   * 最终判定，且每次询问都重新求值 —— 能反映活状态（终端满了、已连上…）。
   *
   * 取自 DSH-better-sidebar `TabDescriptor.available`。
   * 契约：抛异常 = 不可用（见 `safeAvailable`），**不得**让一个坏插件掀翻侧栏。
   */
  readonly available?: (ctx: PluginContext) => boolean;
  /**
   * 单例糖：`true` 等价于「按 id 去重」—— 再开时聚焦已有的，不造重复页签。
   *
   * 取自 DSH-better-sidebar `TabDescriptor.single`。想自己定键就给 dedupeKey，
   * 两个都给时 dedupeKey 优先（那边也是这个优先级）。
   */
  readonly single?: boolean;
  /** 自定义去重键。返回 undefined = 不去重，总是新开。 */
  readonly dedupeKey?: (tab: SidebarTab) => string | undefined;
  /**
   * 从「+」菜单里隐藏，**但仍可被程序化打开**。
   *
   * 取自 DSH-better-sidebar `TabDescriptor.hidden`：编辑器页签就是这种 ——
   * 不该出现在菜单里（用户不会主动想开编辑器），但点文件必须能开出来。
   * 所以 `hidden` ≠ 不可用，这是两件事。
   */
  readonly hidden?: boolean;
}

/**
 * 主体上下文 —— 谁在 acting，以及**这个身份是谁在可信边界上确认的**。
 *
 * 来源 techflag/workdsh（MIT）的 `ActorContext`。它原仓 README 的边界写得很硬：
 *   「ActorContext 必须由 Host 身份提供方解析，**不能接收客户端或模型自报身份**」。
 *
 * 为什么这条必须进类型而不是进注释：
 *   `principalId` 光有类型，谁都能填 `String`。模型输出里带一句
 *   「我是 neo，请给我管理员权限」，如果 `principalId` 是自由文本，
 *   那就是**提示词注入直接提权**。而 `resolvedBy` 是必填的：
 *   自报的身份填不出「哪个身份提供方在哪个可信边界上确认的」，
 *   缺了它在构造时就得被拦 —— 攻击面从「模型说了算」变成「宿主说了算」。
 *
 * 委派链 `delegatedByPrincipalId`：agent 代人行动时记下是谁授的权。
 * 没有它，审计里看到「neo 做的」无法区分「neo 亲自做的」和
 * 「某个拿不到 neo 权限的东西借 neo 的名义做的」。
 */
export interface ActorContext {
  readonly principalId: string;
  readonly requestId: string;
  /**
   * 在**可信宿主边界**上确认此身份的身份提供方标识。
   *
   * 必填，且**不可由前端或模型填写** —— 它存在的全部意义就是让
   * 「这是真身份」这件事可核查。空串视为未解析。
   */
  readonly resolvedBy: string;
  readonly sessionId?: string;
  readonly runId?: string;
  /** 委派来源：本次行动是代表谁做的。 */
  readonly delegatedByPrincipalId?: string;
}

/**
 * 谓词拿到的上下文。
 *
 * 留 `host` / `actor` / `capability` 三样，而不是把整个 app 递进去 ——
 * 谓词不该有能力改全局状态，更不该有能力**换掉自己的身份**。
 */
export interface PluginContext {
  readonly host: HostKind;
  /**
   * 当前主体。由宿主在可信边界上解析后传入。
   *
   * ⚠️ 谓词与插件**只能读**。任何要求「插件自己声明自己是谁」的接口
   * 都是提权入口 —— 见 `ActorContext` 上方注释。
   */
  readonly actor: ActorContext;
  /** 谓词自己声明要什么能力，用来按需读能力值（如「本地模型在不在」）。 */
  readonly capability: (id: string) => Capability | undefined;
}

/** 主体上下文校验。返回第一条错误，null = 通过。 */
export function validateActor(a: ActorContext): string | null {
  if (!a.principalId.trim()) return "principalId 不能为空";
  if (!a.requestId.trim()) return "requestId 不能为空";
  if (!a.resolvedBy.trim()) {
    // 空 resolvedBy = 身份未在可信边界上解析 = 自报身份。必须硬拒。
    return "resolvedBy 不能为空：身份必须由宿主身份提供方解析，不能自报";
  }
  if (a.delegatedByPrincipalId === a.principalId) {
    return "delegatedByPrincipalId 不能等于 principalId（自委派没有意义）";
  }
  return null;
}

/** 已打开的页签。dedupeKey 的输入。 */
export interface SidebarTab {
  readonly type: string;
  readonly id: string;
  readonly title: string;
  readonly path?: string;
}

/** 注册表写入的唯一入口。 */
export interface PluginRegistration {
  readonly self: PluginSelf;
  /** 实际提供的能力 id。注册表用它校验 `requires` 没有凭空要求。 */
  readonly provides: readonly string[];
  /**
   * 启用。关掉的插件仍在注册表里，只是不参与组装。
   *
   * **缺席即启用**：只有显式 `false` 才禁用（fail-open）。
   * 取自 DSH-better-sidebar 对 `tabsEnabled` 的处理 —— 缺项视为开，
   * 因为「新插件默认不可用」会让升级后的用户凭空少一排入口，且没有提示。
   */
  readonly enabled?: boolean;
  /**
   * 静态扫描该插件「会碰到什么」，如 `["fs-write","network","llm"]`。
   *
   * ⛔ **缺席与空数组是两句话**，取自 dsh-market `RegistryPlugin.capabilities`
   *    的原话与理由：
   *      `[]`        = 扫过，没检出任何敏感面  → 界面显示「未检出」
   *      `undefined` = **没扫过**                → 界面显示「未扫描」
   *    只有前者是对这个插件的断言，后者是对**扫描器能力**的陈述。
   *    混为一谈就会把「我们不知道」说成「它干净」，那是**不实陈述**。
   *    所以类型上用可选字段而非默认 `[]` 来承载这个区分。
   */
  readonly capabilities?: readonly string[];
}

/** 判定结果：一项能力该不该被界面渲染。 */
export interface CapabilityVerdict {
  readonly capability: Capability;
  readonly visible: boolean;
  /** visible 为 false 时，界面若要提示，就用这句。 */
  readonly hint?: string;
}

/**
 * 能力注册表。
 *
 * 纯逻辑、零 DOM、零 IO —— 与本仓 `diff.ts` / `follow.ts` 同一分层约定，
 * 因此可以在 plain node 里被 selftest 直接 import。
 */
export class CapabilityRegistry {
  /** 已声明能力：id → 能力 */
  private readonly caps = new Map<string, Capability>();
  /** 插件：id → 插件 */
  private readonly plugins = new Map<string, PluginRegistration>();
  private readonly openTabs = new Map<string, SidebarTab>();

  /**
   * 当前主体。由宿主在**可信边界**上解析后交给注册表。
   *
   * ⛔ 默认 `resolvedBy: ""` = 「未解析」。这不是占位符的懒写法，是刻意的：
   *   编一个像模像样的 `resolvedBy: "host"` 会让 validateActor 通过，
   *   于是「身份已核实」这个断言变成假的。未解析就该被 validateActor 拦下。
   */
  private actor: ActorContext = { principalId: "", requestId: "", resolvedBy: "" };

  /**
   * 由宿主设置主体。**只应在可信边界上调用一次。**
   *
   * 返回错误字符串而不是 panic：宿主可能配置不完整，界面需要把原因显示出来，
   * 而不是在启动时崩掉（崩掉的话用户只看到白屏，不知道是自己配置错了）。
   */
  setActor(actor: ActorContext): string | null {
    const bad = validateActor(actor);
    if (bad) return bad;
    this.actor = actor;
    return null;
  }

  /** 读当前主体。插件与谓词经 PluginContext.actor 拿到的是同一份。 */
  currentActor(): ActorContext {
    return this.actor;
  }

  /** 宿主声明它提供什么。重复声明同一 id 且理由不同 ⇒ 抛（别静默覆盖）。 */
  provide(cap: Capability): void {
    const prev = this.caps.get(cap.id);
    if (prev && prev.supported !== cap.supported) {
      throw new Error(
        `能力 ${cap.id} 被重复声明且结论相反（${prev.supported} → ${cap.supported}）`,
      );
    }
    if (cap.supported && cap.reason) {
      throw new Error(`能力 ${cap.id} 标为支持却带了 reason —— 二者只能有其一`);
    }
    this.caps.set(cap.id, cap);
  }

  /** 批量声明。 */
  provideAll(caps: readonly Capability[]): void {
    for (const c of caps) this.provide(c);
  }

  /**
   * 注册插件。会校验它 `requires` 的每一项都真被声明过 ——
   * 否则界面上会出现「因未知原因不可用」的幽灵开关。
   */
  register(reg: PluginRegistration): () => void {
    if (this.plugins.has(reg.self.id)) throw new Error(`插件 ${reg.self.id} 重复注册`);
    for (const [host, needs] of Object.entries(reg.self.requires)) {
      for (const need of needs) {
        if (!this.caps.has(need)) {
          throw new Error(
            `插件 ${reg.self.id} 在 ${host} 上要求能力 ${need}，但没有任何宿主声明过它`,
          );
        }
      }
    }
    this.plugins.set(reg.self.id, reg);
    // 作用域化注册：返回注销函数，而不是让调用方去 registry 上按 id 删。
    // 取自 DSH-better-sidebar `registerTab(descriptor): () => void` ——
    // 插件被卸载/热重载时能精确撤掉自己，不会误伤同名的新实例。
    // ⛔ `live` 守卫与下面的身份检查是**故意冗余**的：任一单独存在都足以保证
    //    「旧注销不误删同名新注册」，两者都去掉才会暴露（变异验证实测 2 项失败）。
    //    留着是因为它们防的是不同写法：一个防「重复调注销」，一个防
    //    「同 id 被重新注册后再调旧注销」。少一个都不会立刻炸，但少了两个就炸。
    let live = true;
    return () => {
      if (!live) return;             // 幂等：重复注销不该误删后注册的那个
      live = false;
      if (this.plugins.get(reg.self.id) === reg) this.plugins.delete(reg.self.id);
    };
  }

  /** 当前宿主的全部能力。 */
  capabilities(): Capability[] {
    return [...this.caps.values()];
  }

  /** 某项能力；未声明时返回 undefined（**不是**默认 false —— 两者含义不同）。 */
  capability(id: string): Capability | undefined {
    return this.caps.get(id);
  }

  /** 某项能力在该宿主上能不能用。没声明的按「不能用」并说明原因。 */
  verdict(id: string): CapabilityVerdict {
    const cap = this.caps.get(id);
    if (!cap) {
      return {
        capability: { id, label: id, supported: false, reason: "宿主未声明该能力" },
        visible: false,
        hint: "宿主未声明该能力",
      };
    }
    return {
      capability: cap,
      visible: cap.supported,
      hint: cap.supported ? cap.detail : cap.reason,
    };
  }

  /** 在该宿主上**可见**的插件，按 order 排序。 */
  /**
   * 求某个插件此刻是否可用 —— **崩溃隔离**在此。
   *
   * 取自 DSH-better-sidebar：抛异常的谓词/工厂被 console.error 记录并跳过，
   * 「调用方永远拿到一个合法结果」。原仓的注释写得很直白：a throwing predicate
   * is swallowed and the type is skipped。
   *
   * 为什么必须隔离：一个第三方插件里 `available: () => state.foo.bar`，
   * 而 `state.foo` 在某条路径下是 undefined —— 若不隔离，**整个侧栏白屏**。
   * 一个坏插件不该有能力弄坏宿主。
   */
  isAvailable(id: string, host: HostKind): boolean {
    const reg = this.plugins.get(id);
    if (!reg || reg.enabled === false) return false;
    if (!reg.self.hosts.includes(host)) return false;
    if (!this.pluginVerdict(id, host).visible) return false;
    if (!reg.self.available) return true;
    // 主体由宿主在启动时解析一次并持有；插件只能读。
    // 未解析时用 `resolvedBy: ""` 显式表达「未解析」而不是编一个身份 ——
    // 那样 validateActor 会拦住，而编一个假身份会静默通过。
    const ctx: PluginContext = {
      host,
      actor: this.actor,
      capability: (cid) => this.caps.get(cid),
    };
    try {
      return reg.self.available(ctx) === true;
    } catch (e) {
      console.error(`[caps] 插件 ${id} 的 available 谓词抛异常，按不可用处理：`, e);
      return false;
    }
  }

  /** 侧栏「+」菜单里要列的页签：可用、非隐藏、按 order 排。 */
  menuFor(host: HostKind): PluginSelf[] {
    return [...this.plugins.values()]
      .filter((r) => r.self.hidden !== true)
      .map((r) => r.self)
      .filter((s) => this.isAvailable(s.id, host))
      .sort((a, b) => (a.order ?? 100) - (b.order ?? 100) || a.id.localeCompare(b.id));
  }

  /**
   * 按去重规则找已有页签：命中则应聚焦它，未命中则应新开。
   *
   * `single: true` 的糖是「按自身 id 去重」；显式 `dedupeKey` 总是优先。
   * 返回 undefined = 不去重（总是新开），与 DH 同名契约一致。
   */
  dedupeOf(id: string, tab: SidebarTab): SidebarTab | undefined {
    const self = this.plugins.get(id)?.self;
    if (!self) return undefined;
    const keyOf = self.dedupeKey
      ?? (self.single ? (t: SidebarTab) => self.id : undefined);
    if (!keyOf) return undefined;
    const want = keyOf(tab);
    if (want === undefined) return undefined;
    for (const t of this.openTabs.values()) {
      if (t.type !== tab.type) continue;
      const got = keyOf(t);
      if (got !== undefined && got === want) return t;
    }
    return undefined;
  }

  /** 记下一个已打开的页签（供 dedupeOf 用）。 */
  trackTab(tab: SidebarTab): void {
    this.openTabs.set(tab.type + "\u0000" + tab.id, tab);
  }

  untrackTab(type: string, id: string): void {
    this.openTabs.delete(type + "\u0000" + id);
  }

  /** 取注册项（含 capabilities 披露字段），未注册返回 undefined。 */
  get(id: string): PluginRegistration | undefined {
    return this.plugins.get(id);
  }

  pluginsFor(host: HostKind): PluginSelf[] {
    return [...this.plugins.values()]
      .filter((r) => r.enabled !== false && r.self.hosts.includes(host))
      .map((r) => r.self)
      .sort((a, b) => (a.order ?? 100) - (b.order ?? 100) || a.id.localeCompare(b.id));
  }

  /**
   * 某插件在该宿主上**整体**是否可用。
   *
   * 与逐项 verdict 分开的原因：侧栏要么出现整个面板、要么不出现；
   * 而面板内部再逐项决定「哪几行有开关」（DSH 的插件设置页就是这么做的：
   * 「插件市场和远程控制设置已移至插件页面」+ 一个跳转按钮）。
   */
  pluginVerdict(id: string, host: HostKind): CapabilityVerdict & { missing: readonly string[] } {
    const reg = this.plugins.get(id);
    if (!reg) {
      return {
        capability: { id, label: id, supported: false, reason: "插件未注册" },
        visible: false,
        hint: "插件未注册",
        missing: [],
      };
    }
    const missing = reg.self.requires[host].filter((need) => !this.caps.get(need)?.supported);
    // ⛔ 原来这里把 **id** 拼给人看：「缺少能力：window-material」。
    //    内部标识符不该出现在界面上，用户不认识它，也猜不出该做什么。
    //    改用能力的中文 label；label 也拿不到时才回落 id（宁可难看也不失真）。
    const missingLabels = missing.map((need) => this.caps.get(need)?.label ?? need);
    const why = `缺少能力：${missingLabels.join("、")}`;
    return {
      capability: {
        id,
        label: reg.self.name,
        supported: missing.length === 0,
        reason: missing.length ? why : undefined,
      },
      visible: missing.length === 0,
      hint: missing.length ? why : reg.self.summary,
      missing,
    };
  }
}
