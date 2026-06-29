/**
 * Social Events — 社交事件模型
 *
 * Nostr 启发的事件模型:
 *   kind 0  — 简档 (replaceable)
 *   kind 1  — 短文本笔记 (moment / feed 动态)
 *   kind 3  — 联系人列表
 *   kind 4  — 加密 DM
 *   kind 7  — 反应 (like)
 *   kind 31111 — Task 创建
 *   kind 31112 — Task 状态更新
 *   kind 31113 — Task 认领
 *
 * 每个事件: {id, pubkey, kind, content, tags, created_at, sig}
 */

import { socialIdentity } from './social-identity';

export type NostrKind = 0 | 1 | 3 | 4 | 7 | 31111 | 31112 | 31113;
export type EventTag = string[];

export interface NostrEvent {
  id: string;
  pubkey: string;
  kind: NostrKind;
  content: string;
  tags: EventTag[];
  created_at: number;
  sig: string;
  /** 元数据 (非协议字段, 仅客户端) */
  _sender?: string;   // 发送者别名
  _pending?: boolean;  // 等待发送
  _decrypted?: boolean; // DM 已解密
}

export type FeedFilter = 'all' | 'moments' | 'tasks' | 'mine';

export const KIND_LABELS: Record<NostrKind, string> = {
  0: 'Profile',
  1: 'Moment',
  3: 'Contacts',
  4: 'DM',
  7: 'Reaction',
  31111: 'Task Create',
  31112: 'Task Update',
  31113: 'Task Claim',
};

const KIND_ICONS: Record<NostrKind, string> = {
  0: '👤', 1: '📝', 3: '📇', 4: '💬', 7: '❤️',
  31111: '📋', 31112: '🔄', 31113: '✋',
};

export function kindIcon(k: NostrKind): string {
  return KIND_ICONS[k] ?? '📄';
}

/** 事件存储 — 本地 SQLite 替代 (localStorage 模拟) */
export class SocialEventStore {
  private _events: NostrEvent[] = [];

  get events(): NostrEvent[] { return [...this._events]; }

  init(): void {
    const saved = this._load();
    if (saved) {
      this._events = saved;
    } else {
      this._seedDemoEvents();
    }
  }

  /** 创建并存储事件 */
  createEvent(kind: NostrKind, content: string, tags: EventTag[] = []): NostrEvent {
    const pubkey = socialIdentity.pubkey;
    const created_at = Math.floor(Date.now() / 1000);
    const id = `${pubkey.slice(0, 8)}_${created_at}`;
    const sig = socialIdentity.sign(content);

    const ev: NostrEvent = {
      id, pubkey, kind, content, tags, created_at, sig,
      _sender: socialIdentity.profile?.name ?? 'You',
      _pending: false,
    };
    this._events.push(ev);
    this._save();
    return ev;
  }

  /** 接收外部事件 (从 relay 或 P2P 同步) */
  receiveEvent(ev: NostrEvent): void {
    const dup = this._events.find(e => e.id === ev.id);
    if (dup) return;
    const sender = socialIdentity.getContact(ev.pubkey);
    ev._sender = sender?.alias ?? ev.pubkey.slice(0, 12);
    this._events.push(ev);
    this._save();

    // DM → 未读计数
    if (ev.kind === 4 && ev.pubkey !== socialIdentity.pubkey) {
      socialIdentity.incrementUnread(ev.pubkey);
    }
  }

  /** 按条件查询事件 */
  query(opts: {
    kinds?: NostrKind[];
    authors?: string[];
    filter?: FeedFilter;
    limit?: number;
    since?: number;
  }): NostrEvent[] {
    let result = [...this._events];

    if (opts.kinds) result = result.filter(e => opts.kinds!.includes(e.kind));
    if (opts.authors) result = result.filter(e => opts.authors!.includes(e.pubkey));
    if (opts.since !== undefined) result = result.filter(e => e.created_at >= opts.since!);

    if (opts.filter === 'mine') result = result.filter(e => e.pubkey === socialIdentity.pubkey);
    if (opts.filter === 'moments') result = result.filter(e => e.kind === 1 || e.kind === 7);
    if (opts.filter === 'tasks') result = result.filter(e => e.kind === 31111 || e.kind === 31112 || e.kind === 31113);

    result.sort((a, b) => b.created_at - a.created_at);
    if (opts.limit) result = result.slice(0, opts.limit);
    return result;
  }

  /** 获取某个联系人的 DM 会话 */
  getDMs(pubkey: string): NostrEvent[] {
    const mine = socialIdentity.pubkey;
    return this._events.filter(e =>
      e.kind === 4 && ((e.pubkey === mine && e.tags.some(t => t[1] === pubkey)) || (e.pubkey === pubkey))
    ).sort((a, b) => a.created_at - b.created_at);
  }

  /** 获取最近的会话列表 */
  getConversations(): Array<{ pubkey: string; alias: string; lastMessage: string; lastTime: number; unread: number }> {
    const dms = this._events.filter(e => e.kind === 4);
    const groups = new Map<string, NostrEvent[]>();
    for (const dm of dms) {
      // 对方 pubkey
      const other = dm.pubkey === socialIdentity.pubkey
        ? dm.tags.find(t => t[0] === 'p')?.[1] ?? ''
        : dm.pubkey;
      if (!other) continue;
      if (!groups.has(other)) groups.set(other, []);
      groups.get(other)!.push(dm);
    }

    const result: Array<{ pubkey: string; alias: string; lastMessage: string; lastTime: number; unread: number }> = [];
    for (const [pubkey, msgs] of groups) {
      const contact = socialIdentity.getContact(pubkey);
      const sorted = msgs.sort((a, b) => b.created_at - a.created_at);
      result.push({
        pubkey,
        alias: contact?.alias ?? pubkey.slice(0, 12),
        lastMessage: sorted[0].content.slice(0, 60),
        lastTime: sorted[0].created_at,
        unread: contact?.unread ?? 0,
      });
    }
    result.sort((a, b) => b.lastTime - a.lastTime);
    return result;
  }

  /** 发送 DM */
  sendDM(toPubkey: string, content: string): NostrEvent {
    const ev = this.createEvent(4, content, [['p', toPubkey]]);
    return ev;
  }

  /** 发布动态 (kind 1) */
  postMoment(content: string): NostrEvent {
    return this.createEvent(1, content, [['t', 'moment']]);
  }

  /** 反应 (kind 7) */
  reactTo(targetId: string, emoji: string): NostrEvent {
    return this.createEvent(7, emoji, [['e', targetId]]);
  }

  private _seedDemoEvents(): void {
    const now = Math.floor(Date.now() / 1000);
    const demoContacts = socialIdentity.contacts;

    // 自己的动态
    this._events.push({
      id: 'self_m1', pubkey: socialIdentity.pubkey, kind: 1,
      content: '刚刚完成了布局引擎的 Grid 模式实现, 递归分屏树 + 拖拽交换 🚀',
      tags: [['t', 'moment']], created_at: now - 3600,
      sig: 'demo', _sender: 'You',
    });
    this._events.push({
      id: 'self_m2', pubkey: socialIdentity.pubkey, kind: 1,
      content: 'Context Prune 今日统计: 节省 58% token, 23 次系统规则命中',
      tags: [['t', 'moment']], created_at: now - 7200,
      sig: 'demo', _sender: 'You',
    });

    // 联系人动态
    if (demoContacts.length > 0) {
      const alice = demoContacts[0];
      this._events.push({
        id: 'alice_m1', pubkey: alice.pubkey, kind: 1,
        content: '探索了新的 P2P 协议, Hyperswarm NAT 打穿率 92% 📡',
        tags: [['t', 'moment']], created_at: now - 1800,
        sig: 'demo', _sender: alice.alias,
      });
      this._events.push({
        id: 'alice_m2', pubkey: alice.pubkey, kind: 1,
        content: '发布了 v0.3.0: 支持 E2EE 群聊和文件传输',
        tags: [['t', 'moment']], created_at: now - 5400,
        sig: 'demo', _sender: alice.alias,
      });
    }

    if (demoContacts.length > 1) {
      const bob = demoContacts[1];
      this._events.push({
        id: 'bob_m1', pubkey: bob.pubkey, kind: 1,
        content: '今天在研究 Nostr relay 实现, relay 端支持 NIP-11 和 NIP-42',
        tags: [['t', 'moment']], created_at: now - 900,
        sig: 'demo', _sender: bob.alias,
      });

      // DM 示例
      this._events.push({
        id: 'dm1', pubkey: bob.pubkey, kind: 4,
        content: 'Layout engine 的 Grid split 接口可以看看吗?',
        tags: [['p', socialIdentity.pubkey]], created_at: now - 600,
        sig: 'demo', _sender: bob.alias,
      });
      this._events.push({
        id: 'dm2', pubkey: socialIdentity.pubkey, kind: 4,
        content: '好的, 刚刚提交了 PR, 在 layout-engine.ts 里实现了递归分屏',
        tags: [['p', bob.pubkey]], created_at: now - 300,
        sig: 'demo', _sender: 'You',
      });
    }

    if (demoContacts.length > 2) {
      const carol = demoContacts[2];
      this._events.push({
        id: 'carol_m1', pubkey: carol.pubkey, kind: 1,
        content: 'Memory Graph 三视图终于渲染好了! Canvas 力导向真的比 three.js 轻量 ✨',
        tags: [['t', 'moment']], created_at: now - 150,
        sig: 'demo', _sender: carol.alias,
      });
    }

    // 任务事件
    this._events.push({
      id: 'task1', pubkey: socialIdentity.pubkey, kind: 31111,
      content: JSON.stringify({ title: '实现 SocialFeed 面板', description: 'Nostr 风格 kind 1 动态流, 支持 reactions', assignee: '' }),
      tags: [['t', 'task']], created_at: now - 7200,
      sig: 'demo', _sender: 'You',
    });
    this._events.push({
      id: 'task2', pubkey: socialIdentity.pubkey, kind: 31111,
      content: JSON.stringify({ title: 'P2P 中继连接', description: '接入 Nostr relay, 实现事件发布/订阅', assignee: '' }),
      tags: [['t', 'task']], created_at: now - 3600,
      sig: 'demo', _sender: 'You',
    });
  }

  private _storageKey = 'neotrix_social_events';

  private _save(): void {
    try { localStorage.setItem(this._storageKey, JSON.stringify(this._events)); } catch (e) { if (e instanceof Error) console.warn('[SocialEvents]', e.message); }
  }

  private _load(): NostrEvent[] | null {
    try {
      const raw = localStorage.getItem(this._storageKey);
      return raw ? JSON.parse(raw) : null;
    } catch (e) { if (e instanceof Error) console.warn('[SocialEvents]', e.message); return null; }
  }
}

export const socialEvents = new SocialEventStore();
