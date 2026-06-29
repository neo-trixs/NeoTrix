/**
 * Social Tasks — 社交任务同步模型
 *
 * OpenPact + Seabay 启发:
 *   - Task 生命周期: open → claimed → in_progress → completed / cancelled
 *   - 认领带 TTL 自动释放 (避免死锁)
 *   - 风险等级 R0-R3 控制 human confirmation
 *   - 事件驱动: 每个状态变化对应 Nostr event
 */

import { socialEvents, NostrEvent } from './social-events';
import { socialIdentity } from './social-identity';

export type TaskStatus = 'open' | 'claimed' | 'in_progress' | 'completed' | 'cancelled' | 'expired';
export type RiskLevel = 'R0' | 'R1' | 'R2' | 'R3';

export interface SocialTask {
  id: string;                // 对应 event id
  title: string;
  description: string;
  creatorPubkey: string;
  assigneePubkey: string;
  status: TaskStatus;
  riskLevel: RiskLevel;
  parentTaskId: string;
  claimExpiresAt: number;    // unix ts
  completedAt: number;
  createdAt: number;
}

const RISK_LABELS: Record<RiskLevel, string> = {
  R0: 'Read-only',
  R1: 'Editable',
  R2: 'Destructive',
  R3: 'Irreversible',
};

const RISK_COLORS: Record<RiskLevel, string> = {
  R0: '#6c8cff',
  R1: '#ffb86c',
  R2: '#ff6cb4',
  R3: '#e04e4e',
};

export function riskLabel(r: RiskLevel): string { return RISK_LABELS[r]; }
export function riskColor(r: RiskLevel): string { return RISK_COLORS[r]; }

export class SocialTaskManager {
  private _tasks: SocialTask[] = [];

  init(): void {
    const saved = this._load();
    if (saved) {
      this._tasks = saved;
    } else {
      this._seedFromEvents();
    }
  }

  get tasks(): SocialTask[] { return [...this._tasks]; }

  /** 从事件存储重建任务 */
  private _seedFromEvents(): void {
    const taskEvents = socialEvents.query({ kinds: [31111, 31112, 31113] });
    for (const ev of taskEvents) {
      if (ev.kind === 31111) this._fromCreateEvent(ev);
      else if (ev.kind === 31112) this._fromUpdateEvent(ev);
      else if (ev.kind === 31113) this._fromClaimEvent(ev);
    }
    this._save();
  }

  private _fromCreateEvent(ev: NostrEvent): SocialTask {
    let payload: { title: string; description: string; riskLevel?: RiskLevel; assignee?: string; parentTaskId?: string };
    try { payload = JSON.parse(ev.content); } catch (e) {
      if (e instanceof Error) console.warn('[SocialTasks]', e.message);
      payload = { title: ev.content.slice(0, 40), description: '' };
    }
    const task: SocialTask = {
      id: ev.id,
      title: payload.title,
      description: payload.description ?? '',
      creatorPubkey: ev.pubkey,
      assigneePubkey: payload.assignee ?? '',
      status: 'open',
      riskLevel: payload.riskLevel ?? 'R0',
      parentTaskId: payload.parentTaskId ?? '',
      claimExpiresAt: 0,
      completedAt: 0,
      createdAt: ev.created_at,
    };
    const existing = this._tasks.find(t => t.id === task.id);
    if (!existing) this._tasks.push(task);
    return task;
  }

  private _fromUpdateEvent(ev: NostrEvent): void {
    const payload = JSON.parse(ev.content);
    const task = this._tasks.find(t => t.id === ev.tags.find(t => t[0] === 'e')?.[1]);
    if (task) {
      task.status = payload.status ?? task.status;
      if (payload.status === 'completed') task.completedAt = Date.now();
    }
  }

  private _fromClaimEvent(ev: NostrEvent): void {
    const taskId = ev.tags.find(t => t[0] === 'e')?.[1];
    const task = this._tasks.find(t => t.id === taskId);
    if (task && task.status === 'open') {
      task.status = 'claimed';
      task.assigneePubkey = ev.pubkey;
      task.claimExpiresAt = ev.created_at + 86400; // 24h TTL
    }
  }

  /** 创建任务 (发布 kind 31111 事件) */
  createTask(title: string, description: string, riskLevel: RiskLevel = 'R0'): SocialTask {
    const content = JSON.stringify({ title, description, riskLevel });
    const ev = socialEvents.createEvent(31111, content, [['t', 'task']]);
    const task: SocialTask = {
      id: ev.id, title, description,
      creatorPubkey: socialIdentity.pubkey,
      assigneePubkey: '', status: 'open',
      riskLevel, parentTaskId: '', claimExpiresAt: 0, completedAt: 0,
      createdAt: ev.created_at,
    };
    this._tasks.push(task);
    this._save();
    return task;
  }

  /** 认领任务 */
  claimTask(taskId: string): boolean {
    const task = this._tasks.find(t => t.id === taskId);
    if (!task || task.status !== 'open') return false;
    task.status = 'claimed';
    task.assigneePubkey = socialIdentity.pubkey;
    task.claimExpiresAt = Math.floor(Date.now() / 1000) + 86400;
    socialEvents.createEvent(31113, '', [['e', taskId]]);
    this._save();
    return true;
  }

  /** 完成任务 */
  completeTask(taskId: string): boolean {
    const task = this._tasks.find(t => t.id === taskId);
    if (!task || (task.status !== 'claimed' && task.status !== 'in_progress')) return false;
    task.status = 'completed';
    task.completedAt = Date.now();
    socialEvents.createEvent(31112, JSON.stringify({ status: 'completed' }), [['e', taskId]]);
    this._save();
    return true;
  }

  /** 检查过期认领 */
  checkExpired(): number {
    const now = Math.floor(Date.now() / 1000);
    let count = 0;
    for (const t of this._tasks) {
      if (t.status === 'claimed' && t.claimExpiresAt > 0 && t.claimExpiresAt < now) {
        t.status = 'expired';
        t.assigneePubkey = '';
        count++;
      }
    }
    if (count > 0) this._save();
    return count;
  }

  /** 获取某联系人相关的任务 */
  tasksForContact(pubkey: string): SocialTask[] {
    return this._tasks.filter(t => t.creatorPubkey === pubkey || t.assigneePubkey === pubkey);
  }

  /** 获取公开可认领任务 */
  getOpenTasks(): SocialTask[] {
    return this._tasks.filter(t => t.status === 'open');
  }

  private _storageKey = 'neotrix_social_tasks';

  private _save(): void {
    try { localStorage.setItem(this._storageKey, JSON.stringify(this._tasks)); } catch (e) { if (e instanceof Error) console.warn('[SocialTasks]', e.message); }
  }

  private _load(): SocialTask[] | null {
    try {
      const raw = localStorage.getItem(this._storageKey);
      return raw ? JSON.parse(raw) : null;
    } catch (e) { if (e instanceof Error) console.warn('[SocialTasks]', e.message); return null; }
  }
}

export const socialTasks = new SocialTaskManager();
