import React, { useCallback, useEffect, useRef, useState } from "react";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { useStore } from "../../stores";
import type { Message } from "../../types";
import styles from "./ChatView.module.css";

const USER_AVATAR = `<svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="10" height="8" rx="1.5"/><circle cx="7" cy="7" r="1.5"/></svg>`;
const ASSISTANT_AVATAR = `<svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4l6 3-6 3V4z"/></svg>`;

function isHtmlContent(content: string): boolean {
  return /^\s*<(html|div|span|p|h[1-6]|table|ul|ol|section|article|header|footer|main|aside|nav|form|input|button|select|textarea|img|video|audio|canvas|svg|figure|figcaption|details|summary|dialog|data|time|mark|ruby|rt|rp|bdi|bdo|wbr|code|pre|blockquote|dl|dt|dd)[\s>]/i.test(content.trim());
}

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&").replace(/</g, "<").replace(/>/g, ">");
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
  return {
    html: DOMPurify.sanitize(html, {
      ALLOWED_TAGS: ["p", "br", "strong", "em", "code", "pre", "ul", "ol", "li", "a", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "hr", "table", "thead", "tbody", "tr", "th", "td", "span", "div", "img", "svg", "path", "circle", "rect", "line", "text", "button", "input", "textarea", "span"],
      ALLOWED_ATTR: ["href", "target", "rel", "src", "alt", "class", "style", "width", "height", "viewBox", "fill", "stroke", "strokeWidth", "d", "cx", "cy", "r", "x", "y", "rx", "ry", "xmlns", "textAnchor", "fontSize", "fontWeight", "onclick", "type", "checked", "disabled", "value", "placeholder", "rows", "class"],
      ALLOW_DATA_ATTR: false,
    }),
    codeBlocks,
  };
}

const roleClassMap: Record<string, string> = {
  user: styles.messageUser,
  assistant: styles.messageAssistant,
  system: styles.messageSystem,
  error: styles.messageSystem,
  tool: styles.messageTool,
};

const roleAvatarMap: Record<string, string> = {
  user: USER_AVATAR,
  assistant: ASSISTANT_AVATAR,
  system: ASSISTANT_AVATAR,
  error: ASSISTANT_AVATAR,
  tool: `<svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M5 3a2 2 0 11-2.8 2.8L1.5 6.5V5.5L3 4l1-1.5h1L5 3z"/><path d="M8 5l5-3-1 5-3 1-1 3-2-2-1 1"/></svg>`,
};

function formatTimestamp(ts?: number): string {
  if (!ts) return "";
  const date = new Date(ts > 1e12 ? ts : ts * 1000);
  const now = new Date();
  const sameDay = date.toDateString() === now.toDateString();
  const time = date.toTimeString().slice(0, 5);
  if (sameDay) return time;
  return `${date.getMonth() + 1}-${date.getDate()} ${time}`;
}

interface ChatViewProps {
  messages: Array<{ role: string; content: string; contentType?: "markdown" | "html" | "text"; timestamp?: number }>;
  streamingContent?: string;
  streamingRole?: "user" | "assistant";
  agentBusy: boolean;
  onSend: (content: string) => void;
}

export function ChatView({
  messages,
  streamingContent,
  streamingRole = "assistant",
  agentBusy,
  onSend,
}: ChatViewProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [input, setInput] = useState("");

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, streamingContent, scrollToBottom]);

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!input.trim() || agentBusy) return;
    const content = input.trim();
    setInput("");
    onSend(content);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (input.trim() && !agentBusy) {
        const form = (e.target as HTMLTextAreaElement).closest("form");
        form?.requestSubmit();
      }
    }
  };

  const hasMessages = messages.length > 0 || !!streamingContent;

  return (
    <div className={styles.container}>
      {/* Messages */}
      <main className={styles.messages}>
        {messages.map((msg, idx) => (
          <MessageBubble key={idx} message={msg} />
        ))}
        {streamingContent && (
          <MessageBubble
            message={{ role: streamingRole, content: streamingContent, contentType: "markdown" }}
            isStreaming
          />
        )}
        {!hasMessages && (
          <div className={styles.emptyState}>
            <div className={styles.emptyTitle}>NeoCodex</div>
            <div className={styles.emptyHint}>
              选择或新建一个会话，开始与自进化 Agent 对话。
              <br />
              <kbd>⌘N</kbd> 新建会话 · <kbd>⌘B</kbd> 收起侧栏 · <kbd>Ctrl+Tab</kbd> 切换会话
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </main>

      {/* Input */}
      <form onSubmit={handleSubmit} className={styles.inputArea}>
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={agentBusy ? "Waiting for response..." : "Enter 发送，Shift+Enter 换行"}
          disabled={agentBusy}
          rows={1}
          className={styles.textarea}
          style={{ height: "auto", minHeight: "44px" }}
        />
        <button type="submit" disabled={agentBusy || !input.trim()} className={styles.sendBtn}>
          <svg width="18" height="18" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M3 7l5-5 5 5M8 2v10" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
      </form>
    </div>
  );
}

function MessageBubble({
  message,
  isStreaming = false,
}: {
  message: { role: string; content: string; contentType?: "markdown" | "html" | "text"; timestamp?: number };
  isStreaming?: boolean;
}) {
  const { html, codeBlocks } = renderContent(message.content, message.contentType);
  const roleClass = roleClassMap[message.role] || styles.messageAssistant;
  const avatar = roleAvatarMap[message.role] || ASSISTANT_AVATAR;

  if (message.role === "tool") {
    return <ToolCard message={message} avatar={avatar} roleClass={roleClass} />;
  }

  return (
    <div className={`${styles.message} ${roleClass} ${isStreaming ? styles.streaming : ""}`}>
      <div className={styles.avatar} dangerouslySetInnerHTML={{ __html: avatar }} />
      <div className={styles.bubble}>
        <div className={styles.content} dangerouslySetInnerHTML={{ __html: html }} />
        {isStreaming && <span className={styles.streamCaret} aria-hidden="true" />}
        {codeBlocks > 0 && (
          <div className={styles.codeIndicator}>
            {codeBlocks} 代码块
          </div>
        )}
        {message.timestamp && <div className={styles.time}>{formatTimestamp(message.timestamp)}</div>}
        <button className={styles.copyBtn} onClick={() => copyMessage(message.content)} title="复制">
          <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <rect x="3" y="2" width="9" height="10" rx="1.5" strokeLinecap="round"/>
            <path d="M8 2v4h-2V2h-4v4H4v8h6V6h2v4a2 2 0 002 2h4a2 2 0 002-2V4a2 2 0 00-2-2h-4z" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
      </div>
    </div>
  );
}

async function copyMessage(content: string) {
  try {
    await navigator.clipboard.writeText(content);
  } catch {}
}

function ToolCard({ message, avatar, roleClass }: { message: { role: string; content: string; timestamp?: number }; avatar: string; roleClass: string }) {
  const [expanded, setExpanded] = useState(false);
  const m = /^\*\*([^*]+)\*\*(.*)$/s.exec(message.content);
  const toolName = m ? m[1] : "工具调用";
  const body = m ? m[2] : message.content;
  const { html: bodyHtml } = renderContent(body);

  return (
    <div className={`${styles.message} ${roleClass}`}>
      <div className={styles.avatar} dangerouslySetInnerHTML={{ __html: avatar }} />
      <div className={styles.toolCard}>
        <button className={styles.toolHeader} onClick={() => setExpanded((v) => !v)}>
          <span className={styles.toolName}>{toolName}</span>
          <span className={styles.toolTime}>{message.timestamp ? formatTimestamp(message.timestamp) : ""}</span>
          <svg
            width="12"
            height="12"
            viewBox="0 0 14 14"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            className={expanded ? styles.toolChevronOpen : ""}
          >
            <path d="M4 5l3 3 3-3" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
        {expanded && <div className={styles.toolBody} dangerouslySetInnerHTML={{ __html: bodyHtml }} />}
      </div>
    </div>
  );
}

export default ChatView;