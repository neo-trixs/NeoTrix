/**
 * VSA Plugin System — VSA 原生插件系统
 *
 * 非 MCP/ACP, 纯 VSA 向量通信。
 * Plugin = {name, hooks: {beforeTurn?, afterTurn?, onTool?, onRender?}, vsaTriggers[]}
 * 插件注册 → 意识循环关键点触发 → VSA 向量传递上下文
 */

export type PluginHookType = 'beforeTurn' | 'afterTurn' | 'onTool' | 'onRender' | 'onLayout' | 'onMemoryAccess';

export type PluginHook = (ctx: PluginContext) => PluginResult | Promise<PluginResult>;

export type PluginContext = {
  sessionId: string;
  turn?: number;
  toolName?: string;
  toolArgs?: Record<string, unknown>;
  vsaState: Uint8Array | null;       // 当前 VSA 状态向量
  layout: { mode: string; panes: number };
  memory: { accessType: string; key: string };
};

export type PluginResult = {
  modified?: boolean;
  sideEffects?: string[];
  vsaDelta?: Uint8Array | null;     // VSA 状态变更
  log?: string;
};

export type VsaPluginManifest = {
  name: string;
  version: string;
  description: string;
  hooks: PluginHookType[];
  vsaTriggers: string[];            // VSA 向量模式触发
  dependencies?: string[];
};

export type VsaPlugin = {
  manifest: VsaPluginManifest;
  hooks: Map<PluginHookType, PluginHook>;
  active: boolean;
  loadedAt: number;
};

/**
 * VSA 原生插件注册表
 */
export class PluginRegistry {
  private plugins: Map<string, VsaPlugin> = new Map();
  private hookMap: Map<PluginHookType, string[]> = new Map();

  register(manifest: VsaPluginManifest, hooks: Partial<Record<PluginHookType, PluginHook>>): void {
    const hookMap = new Map<PluginHookType, PluginHook>();
    for (const [type, fn] of Object.entries(hooks)) {
      hookMap.set(type as PluginHookType, fn);
    }
    this.plugins.set(manifest.name, {
      manifest,
      hooks: hookMap,
      active: true,
      loadedAt: Date.now(),
    });
    for (const h of manifest.hooks) {
      const list = this.hookMap.get(h) || [];
      list.push(manifest.name);
      this.hookMap.set(h, list);
    }
  }

  unregister(name: string): void {
    this.plugins.delete(name);
    for (const [, list] of this.hookMap) {
      const idx = list.indexOf(name);
      if (idx >= 0) list.splice(idx, 1);
    }
  }

  get(name: string): VsaPlugin | undefined {
    return this.plugins.get(name);
  }

  all(): VsaPlugin[] {
    return Array.from(this.plugins.values());
  }

  active(): VsaPlugin[] {
    return this.all().filter(p => p.active);
  }

  setActive(name: string, active: boolean): void {
    const p = this.plugins.get(name);
    if (p) p.active = active;
  }

  /**
   * 触发某类 hook, 返回汇总结果
   */
  async trigger(hookType: PluginHookType, ctx: PluginContext): Promise<PluginResult[]> {
    const names = this.hookMap.get(hookType) || [];
    const results: PluginResult[] = [];
    for (const name of names) {
      const p = this.plugins.get(name);
      if (!p || !p.active) continue;
      try {
        const r = await p.hooks.get(hookType)!(ctx);
        results.push({ ...r, log: `[${name}] ${r.log || ''}` });
      } catch (e) {
        results.push({ log: `[${name}] ERROR: ${e}` });
      }
    }
    return results;
  }

  /**
   * 内置插件注册
   */
  static seed(): PluginRegistry {
    const reg = new PluginRegistry();

    reg.register(
      { name: 'context-pruner', version: '1.0', description: '上下文智能裁剪 (Sinew + ACP 融合)', hooks: ['beforeTurn'], vsaTriggers: ['highEntropy', 'longContext'] },
      { beforeTurn: async (_ctx) => ({ modified: true, log: 'context pruned' }) }
    );

    reg.register(
      { name: 'memory-graph', version: '1.0', description: '知识图谱三视图 (total-agent-memory 融合)', hooks: ['onMemoryAccess', 'onRender'], vsaTriggers: ['knowledgeAccess'] },
      { onMemoryAccess: async (_ctx) => ({ log: 'memory access traced' }), onRender: async (_ctx) => ({ log: 'graph rendered' }) }
    );

    reg.register(
      { name: 'session-fork', version: '1.0', description: '会话分支树 (BB-Agent + muxd 融合)', hooks: ['afterTurn'], vsaTriggers: ['sessionCheckpoint'] },
      { afterTurn: async (_ctx) => ({ log: 'session checkpoint saved' }) }
    );

    reg.register(
      { name: 'layout-mgr', version: '1.0', description: '统一布局引擎 (TermHive + Ridge + HiveTerm 融合)', hooks: ['onLayout'], vsaTriggers: ['layoutChange'] },
      { onLayout: async (_ctx) => ({ log: 'layout synced' }) }
    );

    reg.register(
      { name: 'contact-hub', version: '1.0', description: '联系人式 Agent 列表 (NoWork + Project Moose 融合)', hooks: ['onRender'], vsaTriggers: ['agentSwitch'] },
      { onRender: async (_ctx) => ({ log: 'contact list rendered' }) }
    );

    return reg;
  }
}

export const pluginRegistry = PluginRegistry.seed();
