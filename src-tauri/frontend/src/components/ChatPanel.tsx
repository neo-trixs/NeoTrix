import React, { useCallback, useEffect, useRef, useState } from "react";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { useStore } from "../stores";
import type { Attachment, Message } from "../types";
import styles from "./ChatPanel.module.css";

interface Props {
  messages: Message[];
  agentBusy: boolean;
  streamingContent?: string;
  streamingContentType?: "markdown" | "html" | "text";
}

const USER_AVATAR = `<svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="10" height="8" rx="1.5"/><circle cx="7" cy="7" r="1.5"/></svg>`;
const ASSISTANT_AVATAR = `<svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4l6 3-6 3V4z"/></svg>`;

function isHtmlContent(content: string): boolean {
  return /^\s*<(html|div|span|p|h[1-6]|table|ul|ol|section|article|header|footer|main|aside|nav|form|input|button|select|textarea|img|video|audio|canvas|svg|figure|figcaption|details|summary|dialog|data|time|mark|ruby|rt|rp|bdi|bdo|wbr|code|pre|blockquote|dl|dt|dd|figure|figcaption|figure|figcaption)[\s>]/i.test(content.trim());
}

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function renderContent(content: string, contentType?: "markdown" | "html" | "text"): { html: string; codeBlocks: number } {
  let html: string;
  let codeBlocks = 0;
  if (contentType === "html" || isHtmlContent(content)) {
    html = content;
    codeBlocks = (content.match(/<pre/g) || []).length;
  } else if (contentType === "text") {
    html = `<pre style="white-space:pre-wrap">${escapeHtml(content)}</pre>`;
  } else {
    try {
      const tokens = marked.lexer(content);
      codeBlocks = tokens.filter((t) => t.type === "code").length;
      html = marked.parser(tokens, { breaks: true }) as string;
    } catch {
      html = `<pre>${escapeHtml(content)}</pre>`;
    }
  }
  html = html.replace(
    /<pre><code class="language-(\w+)">/g,
    '<div class="code-block-header"><span class="code-lang">$1</span><button class="code-copy-btn" onclick="navigator.clipboard.writeText(this.parentElement.nextElementSibling.textContent)">复制</button></div><pre><code class="language-$1">'
  );
  return { html: DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ["p", "br", "strong", "em", "code", "pre", "ul", "ol", "li", "a", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "hr", "table", "thead", "tbody", "tr", "th", "td", "span", "div", "img", "svg", "path", "circle", "rect", "line", "text", "button", "input", "textarea"],
    ALLOWED_ATTR: ["href", "target", "rel", "src", "alt", "class", "style", "width", "height", "viewBox", "fill", "stroke", "strokeWidth", "d", "cx", "cy", "r", "x", "y", "rx", "ry", "xmlns", "textAnchor", "fontSize", "fontWeight", "onclick", "type", "checked", "disabled", "value", "placeholder", "rows"],
    ALLOW_DATA_ATTR: false,
  }), codeBlocks };
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / 1048576).toFixed(1)}MB`;
}

function AttachmentChip({ attachment }: { attachment: Attachment }) {
  const ext = attachment.name.split(".").pop()?.toLowerCase() || "";
  const isImage = attachment.mimeType.startsWith("image/");
  return (
    <div className={styles.attachmentChip}>
      {isImage && <img src={attachment.data} alt="" className={styles.attachmentThumb} />}
      <svg className={styles.attachmentIcon} width="12" height="12" viewBox="0 0 14 14" fill="none"><path d="M7 3v5a2 2 0 004 0V4a3.5 3.5 0 00-7 0v5a4.5 4.5 0 009 0V3" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round"/></svg>
      <div className={styles.attachmentInfo}>
        <span className={styles.attachmentName}>{attachment.name}</span>
        <span className={styles.attachmentSize}>{formatSize(attachment.size)}</span>
      </div>
    </div>
  );
}

const roleClassMap: Record<string, string> = {
  user: styles.messageUser,
  assistant: styles.messageAssistant,
  system: styles.messageSystem,
  error: styles.messageSystem,
};

const bubbleClassMap: Record<string, string> = {
  user: styles.bubbleUser,
  assistant: styles.bubbleAssistant,
  system: styles.bubbleSystem,
  error: styles.bubbleError,
};

const avatarClassMap: Record<string, string> = {
  user: styles.avatarUser,
  assistant: styles.avatarAssistant,
};

function ConsciousnessFingerprint() {
  const resonance = useStore((s) => s.gwtResonance);
  const topExperts = [...resonance.experts]
    .sort((a, b) => b.weight - a.weight)
    .slice(0, 3)
    .filter((e) => e.weight > 0.05);

  if (topExperts.length === 0) return null;

  return (
    <div className="consciousness-fingerprint">
      {topExperts.map((expert) => (
        <span key={expert.id} className="fingerprint-expert">
          {expert.icon} {Math.round(expert.weight * 100)}%
        </span>
      ))}
      <span className="fingerprint-label">contributors</span>
    </div>
  );
}

function MessageActionBar({ messageIdx, onCopy, onRegenerate, onEdit }: { messageIdx: number; onCopy: (idx: number) => void; onRegenerate: (idx: number) => void; onEdit: (idx: number) => void }) {
  const [liked, setLiked] = useState<boolean | null>(null);
  return (
    <div className={styles.actionBar}>
      <button className={styles.actionBtn} onClick={() => onCopy(messageIdx)} title="Copy">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none"><path d="M4 3V1h9v9h-2M1 4h9v9H1V4z" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round"/></svg>
      </button>
      <button className={styles.actionBtn} onClick={() => onEdit(messageIdx)} title="Edit">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none"><path d="M10 1l3 3-8 8H2v-3l8-8z" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round"/></svg>
      </button>
      <button className={styles.actionBtn} onClick={() => onRegenerate(messageIdx)} title="Regenerate">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none"><path d="M1 7a6 6 0 0111.3-3M13 7a6 6 0 01-11.3 3" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/><path d="M1 1v4h4M13 13V9H9" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round"/></svg>
      </button>
      <button
        className={`${styles.actionBtn} ${liked === true ? styles.actionActive : ""}`}
        onClick={() => setLiked(liked === true ? null : true)}
        title="Like"
      >
        <svg width="12" height="12" viewBox="0 0 14 14" fill={liked === true ? "currentColor" : "none"}><path d="M4 8L2 6v6h2M4 8l2 3h5a1 1 0 001-1V8a1 1 0 00-1-1H9l.5-2.5A1.5 1.5 0 008 3H7L5 7" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round"/></svg>
      </button>
      <button
        className={`${styles.actionBtn} ${liked === false ? styles.actionActive : ""}`}
        onClick={() => setLiked(liked === false ? null : false)}
        title="Dislike"
      >
        <svg width="12" height="12" viewBox="0 0 14 14" fill={liked === false ? "currentColor" : "none"}><path d="M10 6l2-2v6h-2M10 6l-2-3H3a1 1 0 00-1 1v3a1 1 0 001 1h4l-.5 2.5A1.5 1.5 0 007 12h1l2-4" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round"/></svg>
      </button>
    </div>
  );
}

function LoadingDots() {
  return (
    <div className={styles.loadingDots}>
      <span></span><span></span><span></span>
    </div>
  );
}

const ChatPanel: React.FC<Props> = ({ messages, agentBusy, streamingContent, streamingContentType }) => {
  const bottomRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const [hoveredIdx, setHoveredIdx] = useState<number | null>(null);
  const [showScrollBtn, setShowScrollBtn] = useState(false);
  const [editingIdx, setEditingIdx] = useState<number | null>(null);
  const [editValue, setEditValue] = useState("");
  const [showLoadingDots, setShowLoadingDots] = useState(false);
  const loadingTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pushMessage = useStore((s) => s.pushMessage);
  const updateMessage = useStore((s) => s.updateMessage);
  const activeSessionIndex = useStore((s) => s.activeSessionIndex);

  // Scroll to bottom on new messages / streaming
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Streaming throttle: batch re-renders to 32ms intervals
  const [displayedStream, setDisplayedStream] = useState("");
  const streamTimerRef = useRef<ReturnType<typeof requestAnimationFrame> | null>(null);
  useEffect(() => {
    if (streamingContent !== undefined) {
      if (streamTimerRef.current) cancelAnimationFrame(streamTimerRef.current);
      streamTimerRef.current = requestAnimationFrame(() => {
        setDisplayedStream(streamingContent || "");
      });
    }
    return () => {
      if (streamTimerRef.current) cancelAnimationFrame(streamTimerRef.current);
    };
  }, [streamingContent]);

  // Loading dots: show when agent is busy but no streaming content yet
  useEffect(() => {
    if (agentBusy && !streamingContent) {
      loadingTimerRef.current = setTimeout(() => setShowLoadingDots(true), 200);
    } else {
      if (loadingTimerRef.current) clearTimeout(loadingTimerRef.current);
      setShowLoadingDots(false);
    }
    return () => {
      if (loadingTimerRef.current) clearTimeout(loadingTimerRef.current);
    };
  }, [agentBusy, streamingContent]);

  // Scroll-to-bottom button: show when scrolled up >150px
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const handler = () => {
      const dist = el.scrollHeight - el.scrollTop - el.clientHeight;
      setShowScrollBtn(dist > 150);
    };
    el.addEventListener("scroll", handler);
    return () => el.removeEventListener("scroll", handler);
  }, []);

  const scrollToBottom = () => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  const handleCopy = useCallback(async (idx: number) => {
    const msg = messages[idx];
    if (!msg) return;
    try {
      await navigator.clipboard.writeText(msg.content);
    } catch {}
  }, [messages]);

  const handleRegenerate = useCallback((idx: number) => {
    const msg = messages[idx];
    if (!msg) return;
    pushMessage("system", `Regenerate #${idx + 1}`);
  }, [pushMessage]);

  const handleEdit = useCallback((idx: number) => {
    const msg = messages[idx];
    if (!msg) return;
    setEditingIdx(idx);
    setEditValue(msg.content);
  }, [messages]);

  const commitEdit = useCallback((idx: number) => {
    if (editValue.trim()) {
      updateMessage(activeSessionIndex, idx, editValue.trim());
    }
    setEditingIdx(null);
  }, [editValue, activeSessionIndex, updateMessage]);

  return (
    <div className={`${styles.panel} glass-panel`}>
      <div className={styles.header}>
        <span aria-label={agentBusy ? "Thinking" : "Conversation"}>{agentBusy ? "Thinking..." : "Conversation"}</span>
      </div>
      <div className={styles.messages} ref={scrollRef} role="log" aria-label="Conversation" aria-live="polite" aria-relevant="additions">
        {messages.length === 0 && !streamingContent && !agentBusy && (
          <div className={styles.empty} aria-label="No messages">
            <svg width="40" height="40" viewBox="0 0 40 40" fill="none" opacity="0.3">
              <circle cx="20" cy="20" r="18" stroke="currentColor" strokeWidth="1.5" />
              <path d="M14 18l6 6 6-6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
            <p>Send a message to start</p>
          </div>
        )}
        {messages.map((msg, i) => {
          const isUser = msg.role === "user";
          const { html } = renderContent(msg.content, msg.contentType);
          return (
            <div
              key={i}
              className={`${styles.message} ${roleClassMap[msg.role] || ""}`}
              role="article"
              onMouseEnter={() => setHoveredIdx(i)}
              onMouseLeave={() => setHoveredIdx(null)}
            >
              <div className={`${styles.avatar} ${avatarClassMap[msg.role] || styles.avatarAssistant}`}>
                <span dangerouslySetInnerHTML={{ __html: isUser ? USER_AVATAR : ASSISTANT_AVATAR }} />
              </div>
              <div className={`${styles.bubble} ${bubbleClassMap[msg.role] || styles.bubbleAssistant}`}>
                {editingIdx === i ? (
                  <div className={styles.editArea}>
                    <textarea
                      className={styles.editInput}
                      value={editValue}
                      onChange={(e) => setEditValue(e.target.value)}
                      rows={4}
                    />
                    <div className={styles.editActions}>
                      <button className="nt-btn-sm" onClick={() => commitEdit(i)}>Save</button>
                      <button className="nt-btn-sm nt-btn-secondary" onClick={() => setEditingIdx(null)}>Cancel</button>
                    </div>
                  </div>
                ) : (
                  <div
                    className={styles.bubbleContent}
                    dangerouslySetInnerHTML={{ __html: html }}
                  />
                )}
                {msg.attachments && msg.attachments.length > 0 && (
                  <div className={styles.attachmentRow}>
                    {msg.attachments.map((a) => <AttachmentChip key={a.id} attachment={a} />)}
                  </div>
                )}
                {msg.role === "assistant" && <ConsciousnessFingerprint />}
                {hoveredIdx === i && editingIdx !== i && (
                  <MessageActionBar messageIdx={i} onCopy={handleCopy} onRegenerate={handleRegenerate} onEdit={handleEdit} />
                )}
                <div className={styles.timestamp}>just now</div>
              </div>
            </div>
          );
        })}
        {showLoadingDots && (
          <div className={`${styles.message} ${styles.messageAssistant}`}>
            <div className={`${styles.avatar} ${styles.avatarAssistant}`}>
              <span dangerouslySetInnerHTML={{ __html: ASSISTANT_AVATAR }} />
            </div>
            <div className={`${styles.bubble} ${styles.bubbleAssistant} ${styles.streaming}`}>
              <div className={styles.typing}>
                <span /><span /><span />
              </div>
            </div>
          </div>
        )}
        {streamingContent && !showLoadingDots && (
          <div className={`${styles.message} ${styles.messageAssistant}`}>
            <div className={`${styles.avatar} ${styles.avatarAssistant}`}>
              <span dangerouslySetInnerHTML={{ __html: ASSISTANT_AVATAR }} />
            </div>
            <div className={`${styles.bubble} ${styles.bubbleAssistant} ${styles.streaming}`}>
              <div
                className={styles.bubbleContent}
                dangerouslySetInnerHTML={{
                  __html: renderContent(displayedStream || streamingContent, streamingContentType).html,
                }}
              />
              <span className={styles.streamingCursor} />
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>
      {showScrollBtn && (
        <button className={styles.scrollToBottom} onClick={scrollToBottom}>
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 4l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/></svg>
          Scroll to bottom
        </button>
      )}
    </div>
  );
};

export default ChatPanel;