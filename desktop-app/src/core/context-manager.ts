/**
 * Context Manager — 上下文管理器
 *
 * 融合自:
 *   Sinew    — agent 驱动裁剪: agent 标记无用 tool result → 替换为占位符
 *   OpenCode — 系统规则自动 supersede: 10 策略 hash/queue/file/error 去重
 *
 * 在意识循环步 4-5 之间插入上下文裁剪:
 *   1. Pre-filter (系统规则): hash 去重 / file supersede / state query merge / error truncation
 *   2. Agent prune (agent 驱动): 标记无用 tool result → 替换为占位符
 *   3. Post-filter (全局优化): 完全相同的 tool+params 合并, 超长 tool result 截断
 *
 * 统计: 原始 token → 剪后 token → 节省率
 */

export interface ContextEntry {
  id: string;
  role: 'user' | 'assistant' | 'tool';
  content: string;
  toolName?: string;
  toolParams?: string;
  timestamp: number;
  pruned?: boolean;
  prunedBy?: 'system' | 'agent';
  placeholder?: string;
}

export type SupersedeStrategy = 'hash' | 'file' | 'todo' | 'url' | 'stateQuery' | 'snapshot' | 'retry' | 'filePart' | 'userCode' | 'error';

export interface PruneStats {
  originalTokens: number;
  afterSystemTokens: number;
  afterAgentTokens: number;
  afterFinalTokens: number;
  systemRuleHits: Record<SupersedeStrategy, number>;
  agentPruneCount: number;
  mergedPairs: number;
  truncatedLongResults: number;
}

/**
 * 模拟 token 计数 (约 4 chars per token)
 */
function estimateTokens(text: string): number {
  return Math.ceil(text.length / 4);
}

export class ContextManager {
  private history: ContextEntry[] = [];
  private hashCache: Set<string> = new Set();
  private lastFileWrite: Map<string, string> = new Map(); // path → content hash

  // ── 策略配置 ──
  private activeStrategies: Set<SupersedeStrategy> = new Set([
    'hash', 'file', 'url', 'stateQuery', 'error',
  ]);

  enableStrategy(s: SupersedeStrategy): void { this.activeStrategies.add(s); }
  disableStrategy(s: SupersedeStrategy): void { this.activeStrategies.delete(s); }
  isActive(s: SupersedeStrategy): boolean { return this.activeStrategies.has(s); }
  getAllStrategies(): SupersedeStrategy[] {
    return ['hash', 'file', 'todo', 'url', 'stateQuery', 'snapshot', 'retry', 'filePart', 'userCode', 'error'];
  }

  addEntry(entry: ContextEntry): void {
    this.history.push(entry);
  }

  getHistory(): ContextEntry[] {
    return [...this.history];
  }

  getActive(): ContextEntry[] {
    return this.history.filter(e => !e.pruned);
  }

  /**
   * 主入口: 在意识循环步 4.5 调用
   * 返回裁剪后的活跃上下文列表和统计
   */
  prune(): { entries: ContextEntry[]; stats: PruneStats } {
    const before = [...this.history];

    const stats: PruneStats = {
      originalTokens: this._totalTokens(before),
      afterSystemTokens: 0,
      afterAgentTokens: 0,
      afterFinalTokens: 0,
      systemRuleHits: { hash: 0, file: 0, todo: 0, url: 0, stateQuery: 0, snapshot: 0, retry: 0, filePart: 0, userCode: 0, error: 0 },
      agentPruneCount: 0,
      mergedPairs: 0,
      truncatedLongResults: 0,
    };

    // 阶段 1: 系统规则 supersede
    const afterSystem = this._systemSupersede(before, stats);

    // 阶段 2: agent 驱动裁剪 (模拟: 标记超长 tool result)
    const afterAgent = this._agentPrune(afterSystem, stats);

    // 阶段 3: 全局优化 — 合并相同 tool+params, 截断超长结果
    const afterFinal = this._finalOptimize(afterAgent, stats);

    this.history = afterFinal;

    stats.afterSystemTokens = this._totalTokens(afterSystem);
    stats.afterAgentTokens = this._totalTokens(afterAgent);
    stats.afterFinalTokens = this._totalTokens(afterFinal);

    return { entries: afterFinal, stats };
  }

  private _totalTokens(entries: ContextEntry[]): number {
    return entries.reduce((s, e) => s + estimateTokens(e.content), 0);
  }

  /**
   * 阶段 1: 系统规则 supersede
   */
  private _systemSupersede(entries: ContextEntry[], stats: PruneStats): ContextEntry[] {
    return entries.filter(e => {
      if (e.pruned) return false;

      // hash 去重: 相同内容仅保留第一条
      if (this.activeStrategies.has('hash')) {
        const h = this._simpleHash(e.content);
        if (this.hashCache.has(h) && e.role === 'tool') {
          stats.systemRuleHits.hash++;
          e.pruned = true;
          e.prunedBy = 'system';
          e.placeholder = '[Duplicate content removed]';
          return true; // 保留占位符而非完全删除
        }
        this.hashCache.add(h);
      }

      // file supersede: 写文件后旧的文件读写可裁
      if (this.activeStrategies.has('file') && e.toolName === 'write_file') {
        const path = this._extractPath(e.content);
        if (path) {
          this.lastFileWrite.set(path, this._simpleHash(e.content));
        }
      }
      // 如果 read_file 的内容与最近的 write_file 相同, 可裁
      if (this.activeStrategies.has('file') && e.toolName === 'read_file') {
        const path = this._extractPath(e.content);
        if (path && this.lastFileWrite.has(path)) {
          // keep it — read after write is intentional
        }
      }

      // url 去重: 相同 URL fetch 保留最新
      if (this.activeStrategies.has('url') && e.toolName === 'fetch_url') {
        const url = this._extractUrl(e.content);
        if (url) {
          const older = this.history.find(o =>
            o.id !== e.id && o.toolName === 'fetch_url' && this._extractUrl(o.content) === url
          );
          if (older) {
            stats.systemRuleHits.url++;
            older.pruned = true;
            older.prunedBy = 'system';
            older.placeholder = '[Superseded by newer fetch]';
          }
        }
      }

      // error 截断: 错误内容保留前 200 chars
      if (this.activeStrategies.has('error') && e.content.includes('Error:')) {
        if (e.content.length > 200) {
          e.content = e.content.slice(0, 200) + '… [truncated]';
          stats.systemRuleHits.error++;
        }
      }

      // stateQuery 合并: 同 type 的 state query 保留最新
      if (this.activeStrategies.has('stateQuery') && e.toolName === 'state_query') {
        const qtype = this._extractQueryType(e.content);
        if (qtype) {
          const older = this.history.find(o =>
            o.id !== e.id && o.toolName === 'state_query' && this._extractQueryType(o.content) === qtype
          );
          if (older && older.timestamp < e.timestamp - 5000) {
            stats.systemRuleHits.stateQuery++;
            older.pruned = true;
            older.prunedBy = 'system';
            older.placeholder = '[Superseded by newer state query]';
          }
        }
      }

      return !e.pruned;
    });
  }

  /**
   * 阶段 2: agent 驱动裁剪 (模拟 Sinew clean_context)
   */
  private _agentPrune(entries: ContextEntry[], stats: PruneStats): ContextEntry[] {
    return entries.map(e => {
      if (e.pruned) return e;

      // 模拟 agent 判断: 超长工具结果标记为可裁剪
      if (e.role === 'tool' && e.content.length > 2000) {
        // Agent 会判断是否对后续上下文有用
        // 这里简化: 长结果中若包含 "Search results" 等可能无用, 替换为 placeholder
        if (e.content.includes('Search results for') && e.content.length > 1500) {
          stats.agentPruneCount++;
          return {
            ...e,
            pruned: true,
            prunedBy: 'agent',
            placeholder: `[Tool result cleaned by you: irrelevant to future context. ${e.toolName || ''}]`,
            content: `[Tool result cleaned by you: irrelevant to future context. ${e.toolName || ''}]`,
          };
        }
      }

      return e;
    });
  }

  /**
   * 阶段 3: 全局优化
   */
  private _finalOptimize(entries: ContextEntry[], stats: PruneStats): ContextEntry[] {
    const result: ContextEntry[] = [];
    const seen = new Map<string, number>(); // tool+params → index in result

    for (const e of entries) {
      if (e.pruned) {
        result.push(e);
        continue;
      }

      // 完全相同的 tool+params 合并
      if (e.toolName && e.toolParams) {
        const key = `${e.toolName}|${e.toolParams}`;
        if (seen.has(key)) {
          const idx = seen.get(key)!;
          // 保留最新的
          result[idx] = e;
          stats.mergedPairs++;
          continue;
        }
        seen.set(key, result.length);
      }

      // 超长 tool result 截断 (保留前 500 chars)
      if (e.content.length > 2000) {
        e.content = e.content.slice(0, 500) + `\n… [truncated ${e.content.length} chars]`;
        stats.truncatedLongResults++;
      }

      result.push(e);
    }

    return result;
  }

  private _simpleHash(s: string): string {
    let h = 0;
    for (let i = 0; i < s.length; i++) {
      h = (h * 31 + s.charCodeAt(i)) | 0;
    }
    return h.toString(16);
  }

  private _extractPath(content: string): string | null {
    const m = content.match(/["']([^"']+\.\w+)["']/);
    return m ? m[1] : null;
  }

  private _extractUrl(content: string): string | null {
    const m = content.match(/https?:\/\/[^\s"']+/);
    return m ? m[0] : null;
  }

  private _extractQueryType(content: string): string | null {
    const m = content.match(/type["']:\s*["'](\w+)["']/);
    return m ? m[1] : null;
  }

  clear(): void {
    this.history = [];
    this.hashCache.clear();
    this.lastFileWrite.clear();
  }

  get len(): number { return this.history.length; }
}

export const contextManager = new ContextManager();
