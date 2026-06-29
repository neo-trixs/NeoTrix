import { useState, useEffect, useRef } from 'react';
import { useAppCtx } from '../App';
import { chatStore, ConversationTab, ChatMessage } from '../core/chat-store';



const TAB_ICONS: Record<string, string> = { session: '⟐', agent: '⚡', person: '👤' };

export default function UniversalChat() {
  const { sideChatOpen, setSideChatOpen } = useAppCtx();
  const [tabs, setTabs] = useState<ConversationTab[]>([]);
  const [activeId, setActiveId] = useState('session');
  const [input, setInput] = useState('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const listRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (sideChatOpen) inputRef.current?.focus();
  }, [sideChatOpen]);

  useEffect(() => {
    const unsub = chatStore.subscribe(() => {
      setTabs([...chatStore.tabs]);
      setActiveId(chatStore.activeId);
      setMessages(chatStore.messages(chatStore.activeId));
    });
    if (chatStore.tabs.length > 0) {
      setTabs([...chatStore.tabs]);
      setActiveId(chatStore.activeId);
      setMessages(chatStore.messages(chatStore.activeId));
    }
    return unsub;
  }, []);

  useEffect(() => {
    if (listRef.current) listRef.current.scrollTop = listRef.current.scrollHeight;
  }, [messages]);

  const send = () => {
    if (!input.trim()) return;
    chatStore.send(input.trim());
    setInput('');
  };

  const timeStr = (ts: number) => {
    const d = new Date(ts * 1000);
    const now = new Date();
    const diff = (now.getTime() - d.getTime()) / 1000;
    if (diff < 60) return 'now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
    return d.toLocaleDateString();
  };

  if (!sideChatOpen) return null;

  return (
    <div className="side-chat-overlay" onClick={() => setSideChatOpen(false)}>
      <div className="uchat" onClick={e => e.stopPropagation()}>
        {/* Tab bar */}
        <div className="uchat-tabs" data-ai-role="chat-tabs">
          {tabs.map(t => (
            <div key={t.id}
              className={`uchat-tab ${t.id === activeId ? 'active' : ''}`}
              data-ai-role="chat-tab" data-ai-tab-id={t.id}
              onClick={() => chatStore.switchTab(t.id)}>
              <span className="uchat-tab-icon">{TAB_ICONS[t.type] ?? '⟐'}</span>
              <span className="uchat-tab-label">{t.label}</span>
              {t.unread > 0 && <span className="uchat-tab-badge">{t.unread}</span>}
              {t.id !== 'session' && (
                <button className="uchat-tab-close"
                  onClick={e => { e.stopPropagation(); chatStore.closeTab(t.id); }}>
                  <svg width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
                  </svg>
                </button>
              )}
            </div>
          ))}
          <button className="uchat-close" onClick={() => setSideChatOpen(false)}>
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        {/* Messages */}
        <div className="uchat-messages" data-ai-role="chat-messages" ref={listRef}>
          {messages.length === 0 && (
            <div className="uchat-empty">
              <div className="uchat-empty-text">No messages yet</div>
              <div className="uchat-empty-hint">Send a message to start the conversation</div>
            </div>
          )}
          {messages.map(m => (
            <div key={m.id} className={`uchat-msg ${m.isMine ? 'mine' : ''}`}>
              <div className="uchat-msg-avatar">
                {m.fromName.charAt(0).toUpperCase()}
              </div>
              <div className="uchat-msg-body">
                <div className="uchat-msg-name-row">
                  <span className="uchat-msg-name">{m.fromName}</span>
                  <span className="uchat-msg-time">{timeStr(m.ts)}</span>
                </div>
                <div className="uchat-msg-bubble">{m.content}</div>
              </div>
            </div>
          ))}
        </div>

        {/* Input */}
        <div className="uchat-input">
          <input ref={inputRef}
            className="uchat-input-field" data-ai-role="chat-input"
            placeholder={activeId !== 'session' ? `DM ${tabs.find(t => t.id === activeId)?.label}...` : 'Message session...'}
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyDown={e => { if (e.key === 'Enter') send(); if (e.key === 'Escape') setSideChatOpen(false); }}
          />
          <button className="uchat-send-btn" onClick={send} disabled={!input.trim()}>
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}
