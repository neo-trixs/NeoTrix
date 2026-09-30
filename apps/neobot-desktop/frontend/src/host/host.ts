/**
 * 宿主适配层 —— 吸收自 DSH 的 `adapter` / `bridge` 做法。
 *
 * # 这一层存在的理由
 *
 * 组件一旦直接摸 `navigator.userAgent` 或直接 `invoke()`，就没法在浏览器里
 * 跑、没法单测、换宿主要逐个组件改。DSH 的做法是所有平台差异收进
 * `NextSettingsAdapter`，组件只认 `adapter.command({type})`。
 *
 * 我把它做成**接口 + 内存实现**两个文件：
 *   · `Host` 接口 —— 界面唯一能看见的世界
 *   · `MemoryHost` —— 纯内存实现，selftest 与浏览器预览都用它
 *
 * 真实宿主（Tauri）另有一个实现，但那是**接线活**，不是架构活 ——
 * 有了这个接口之后，接上去只影响一个文件。
 */

import { CapabilityRegistry, type Capability, type HostKind, type PluginSelf } from "../plugin/contract.ts";

/** 宿主状态快照。界面读它渲染，不再自己探测环境。 */
export interface HostState {
  readonly kind: HostKind;
  readonly platform: "darwin" | "win32" | "linux" | "browser";
  /** 骨架（NeoTrix）是否在线。离线时大部分能力应当降级。 */
  readonly coreOnline: boolean;
  /** 是否有本地模型在跑。 */
  readonly localModel: boolean;
  /** 窄窗（侧栏与正文要折叠）。 */
  readonly narrow: boolean;
  /** 宿主认为「现在忙」，设置里的写操作应禁用。 */
  readonly busy: boolean;
}

/** 宿主对界面开放的动作。 */
export interface Host {
  readonly kind: HostKind;
  /** 当前状态快照。 */
  state(): HostState;
  /** 订阅状态变化，返回退订函数。 */
  subscribe(fn: (s: HostState) => void): () => void;
  /** 宿主声明的能力（经由注册表）。 */
  readonly caps: CapabilityRegistry;
  /** 在该宿主上可见的面板。 */
  panels(): readonly PluginSelf[];
  /** 执行宿主动作。未知动作必须**抛**，不能静默 no-op。 */
  act(type: string, payload?: unknown): Promise<void>;
}

/** 默认能力矩阵。真实宿主可覆盖其中任意项。 */
export function defaultCapabilities(kind: HostKind): Capability[] {
  const tauri = kind === "tauri";
  /**
   * 理由**只在不支持时**才有值。这不是洁癖：注册表有一条不变量
   * 「supported=true 却带 reason ⇒ 抛」，而第一版正是这么写的
   * （`supported: tauri, reason: "浏览器无系统托盘"`），被 selftest 当场抓到。
   * 改用构造器以后，这个矛盾在**类型层就写不出来**。
   */
  const cap = (id: string, label: string, supported: boolean, reason: string): Capability =>
    ({ id, label, supported, reason: supported ? undefined : reason });
  return [
    { id: "chat", label: "对话", supported: true },
    cap("local-model", "本地推理", tauri, "浏览器里没有本地运行时"),
    cap("window-material", "窗口材质", false, "仅原生窗口支持"),
    cap("tray", "托盘", tauri, "浏览器无系统托盘"),
    cap("background-run", "后台运行", tauri, "浏览器关闭即终止"),
    { id: "plugins", label: "插件", supported: true },
    cap("remote-control", "远程控制", false, "尚未启用"),
  ];
}

/** 内存宿主：selftest 与浏览器预览共用。纯数据，零 IO。 */
export class MemoryHost implements Host {
  readonly caps: CapabilityRegistry;
  private s: HostState;
  private readonly listeners = new Set<(s: HostState) => void>();
  private readonly actions: string[] = [];

  constructor(state?: Partial<HostState>, caps?: Capability[]) {
    this.s = {
      kind: "browser",
      platform: "browser",
      coreOnline: true,
      localModel: false,
      narrow: false,
      busy: false,
      ...state,
    };
    this.caps = new CapabilityRegistry();
    this.caps.provideAll(caps ?? defaultCapabilities(this.s.kind));
  }

  get kind(): HostKind { return this.s.kind; }
  state(): HostState { return this.s; }
  subscribe(fn: (s: HostState) => void): () => void {
    this.listeners.add(fn);
    return () => { this.listeners.delete(fn); };
  }
  panels(): readonly PluginSelf[] { return this.caps.pluginsFor(this.s.kind); }
  act(type: string, payload?: unknown): Promise<void> {
    this.actions.push(type);
    void payload;
    return Promise.resolve();
  }
  /** 自测用：已执行过的动作序列。 */
  acted(): readonly string[] { return [...this.actions]; }
  /** 改状态并通知。 */
  patch(delta: Partial<HostState>): void {
    this.s = { ...this.s, ...delta };
    for (const fn of this.listeners) fn(this.s);
  }
}
