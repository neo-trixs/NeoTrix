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
};

const roleAvatarMap: Record<string, string> = {
  user: USER_AVATAR,
  assistant: ASSISTANT_AVATAR,
  system: ASSISTANT_AVATAR,
  error: ASSISTANT_AVATAR,
};

interface ChatViewProps {
  messages: Array<{ role: string; content: string; contentType?: "markdown" | "html" | "text"; timestamp?: number }>;
  streamingContent?: string;
  streamingRole?: "user" | "assistant";
  agentBusy: boolean;
  onSend: (content: string) => void;
  onAddGoal: (desc: string, maxIter: number) => void;
}

export function ChatView({
  messages,
  streamingContent,
  streamingRole = "assistant",
  agentBusy,
  onSend,
  onAddGoal,
}: ChatViewProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [input, setInput] = useState("");
  const [showGoalDialog, setShowGoalDialog] = useState(false);
  const [goalDesc, setGoalDesc] = useState("");
  const [goalMaxIter, setGoalMaxIter] = useState(5);

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

  const handleGoalSubmit = () => {
    if (!goalDesc.trim()) return;
    onAddGoal(goalDesc.trim(), goalMaxIter);
    setShowGoalDialog(false);
    setGoalDesc("");
    setGoalMaxIter(5);
  };

  return (
    <div className={styles.container}>
      {/* Messages */}
      <main className={styles.messages} ref={messagesEndRef}>
        {messages.map((msg, idx) => (
          <MessageBubble key={idx} message={msg} />
        ))}
        {streamingContent && (
          <MessageBubble
            message={{ role: streamingRole, content: streamingContent, contentType: "markdown" }}
            isStreaming
          />
        )}
        <div ref={messagesEndRef} />
      </main>

      {/* Input */}
      <form onSubmit={handleSubmit} className={styles.inputArea}>
        <textarea
          ref={textareaRef}
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

      {/* Goal Dialog */}
      {showGoalDialog && (
        <div className={styles.dialogOverlay} onClick={() => setShowGoalDialog(false)}>
          <div className={styles.dialog} onClick={(e) => e.stopPropagation()}>
            <h3>添加进化目标</h3>
            <textarea
              value={goalDesc}
              onChange={(e) => setGoalDesc(e.target.value)}
              placeholder="描述目标，如：优化内存分配策略..."
              rows={3}
              className={styles.dialogInput}
            />
            <div className={styles.dialogRow}>
              <label>最大迭代数: </label>
              <input
                type="number"
                value={goalMaxIter}
                onChange={(e) => setGoalMaxIter(Number(e.target.value))}
                min={1}
                max={100}
                className={styles.dialogInput}
                style={{ width: "80px" }}
              />
            </div>
            <div className={styles.dialogActions}>
              <button className={styles.btnSecondary} onClick={() => setShowGoalDialog(false)}>取消</button>
              <button className={styles.btnPrimary} onClick={handleGoalSubmit}>添加</button>
            </div>
          </div>
        </div>
      )}
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

  return (
    <div className={`${styles.message} ${roleClass} ${isStreaming ? styles.streaming : ""}`}>
      <div className={styles.avatar} dangerouslySetInnerHTML={{ __html: avatar }} />
      <div className={styles.bubble}>
        <div className={styles.content} dangerouslySetInnerHTML={{ __html: html }} />
        {codeBlocks > 0 && (
          <div className={styles.codeIndicator}>
            {codeBlocks} 代码块
          </div>
        )}
      </div>
    </div>
  );
}

export default ChatView;