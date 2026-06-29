/**
 * 统一聊天存储 — 融合 session 上下文 / Agent DM / 社交 DM
 *
 * 设计: UniversalChat 的唯一数据源
 * - tabs: 打开的所有对话 (session + DMs)
 * - addTab(contactId) 从任何入口 (ContactHub / SideChat 按钮) 打开新对话
 * - 消息统一来自 chatStore.messages(tabId)
 * - 外部输入: 键盘消息 → chatStore.send()
 */
import { socialIdentity } from './social-identity';
import { socialEvents } from './social-events';

export type ConversationType = 'session' | 'agent' | 'person';

export interface ConversationTab {
  id: string;
  type: ConversationType;
  label: string;
  unread: number;
}

export interface ChatMessage {
  id: string;
  from: string;
  fromName: string;
  content: string;
  ts: number;
  isMine: boolean;
}

type Listener = () => void;

class ChatStore {
  private _tabs: ConversationTab[] = [];
  private _activeId = 'session';
  private _listeners = new Set<Listener>();

  constructor() {
    this._tabs = [{ id: 'session', type: 'session', label: 'Session', unread: 0 }];
  }

  get tabs(): ConversationTab[] { return this._tabs; }
  get activeId(): string { return this._activeId; }
  get activeTab(): ConversationTab | undefined { return this._tabs.find(t => t.id === this._activeId); }

  subscribe(fn: Listener): () => void { this._listeners.add(fn); return () => { this._listeners.delete(fn); }; }
  private _notify() { this._listeners.forEach(fn => fn()); }

  switchTab(id: string) {
    if (this._tabs.find(t => t.id === id)) {
      this._activeId = id;
      const tab = this._tabs.find(t => t.id === id);
      if (tab) tab.unread = 0;
      this._notify();
    }
  }

  /** 打开或切换到某个 contact 的 DM */
  openDM(contactId: string, label: string, type: 'agent' | 'person') {
    const existing = this._tabs.find(t => t.id === `dm:${contactId}`);
    if (existing) {
      this._activeId = existing.id;
      existing.unread = 0;
    } else {
      this._tabs.push({ id: `dm:${contactId}`, type, label, unread: 0 });
      this._activeId = `dm:${contactId}`;
    }
    this._notify();
  }

  closeTab(id: string) {
    if (id === 'session') return; // 不能关闭 session
    this._tabs = this._tabs.filter(t => t.id !== id);
    if (this._activeId === id) this._activeId = 'session';
    this._notify();
  }

  /** 获取某个 tab 的消息 */
  messages(tabId: string): ChatMessage[] {
    const tab = this._tabs.find(t => t.id === tabId);
    if (!tab) return [];

    if (tab.type === 'session') {
      // session 消息: 从模拟 session 步骤派生
      return this._sessionMessages();
    }

    // DM 消息: 从 socialEvents 读取
    const contactId = tabId.replace(/^dm:/, '');
    return socialEvents.getDMs(contactId).map(e => ({
      id: e.id,
      from: e.pubkey,
      fromName: socialIdentity.getContact(e.pubkey)?.alias ?? socialIdentity.profile?.name ?? 'Unknown',
      content: e.content,
      ts: e.created_at,
      isMine: e.pubkey === socialIdentity.pubkey,
    }));
  }

  /** 发送消息到当前活跃 tab */
  send(content: string) {
    const tab = this._tabs.find(t => t.id === this._activeId);
    if (!tab || !content.trim()) return;

    if (tab.type === 'session') {
      // session: 存入模拟步骤
      this._addSessionMsg(content);
    } else {
      const contactId = this._activeId.replace(/^dm:/, '');
      socialEvents.sendDM(contactId, content.trim());
    }
    this._notify();
  }

  // --- session 消息模拟 ---
  private _sessionLog: ChatMessage[] = [];

  private _sessionMessages(): ChatMessage[] {
    return [
      { id: 's1', from: 'system', fromName: 'NeoTrix', content: 'Session started', ts: Date.now() / 1000 - 120, isMine: false },
      { id: 's2', from: 'user', fromName: 'You', content: 'Analyze the architecture', ts: Date.now() / 1000 - 90, isMine: true },
      { id: 's3', from: 'system', fromName: 'NeoTrix', content: 'Running GATHER phase... scanning codebase', ts: Date.now() / 1000 - 60, isMine: false },
      { id: 's4', from: 'system', fromName: 'NeoTrix', content: 'REASON complete — 3 dependencies found', ts: Date.now() / 1000 - 30, isMine: false },
      ...this._sessionLog,
    ];
  }

  private _addSessionMsg(content: string) {
    this._sessionLog.push({
      id: `session_${Date.now()}`,
      from: 'user',
      fromName: 'You',
      content,
      ts: Date.now() / 1000,
      isMine: true,
    });
  }
}

export const chatStore = new ChatStore();
