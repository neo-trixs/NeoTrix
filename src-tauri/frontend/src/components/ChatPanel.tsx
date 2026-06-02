import React, { useEffect, useRef } from "react";
import { marked } from "marked";
import DOMPurify from "dompurify";
import type { Message } from "../types";

interface Props {
  messages: Message[];
  agentBusy: boolean;
  streamingContent?: string;
  streamingContentType?: "markdown" | "html" | "text";
}

function isHtmlContent(content: string): boolean {
  return /^\s*<(html|div|span|p|h[1-6]|table|ul|ol|section|article|header|footer|main|aside|nav|form|input|button|select|textarea|img|video|audio|canvas|svg|figure|figcaption|details|summary|dialog|data|time|mark|ruby|rt|rp|bdi|bdo|wbr|code|pre|blockquote|dl|dt|dd|figure|figcaption|figure|figcaption)[\s>]/i.test(content.trim());
}

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function renderContent(content: string, contentType?: "markdown" | "html" | "text"): string {
  let html: string;
  if (contentType === "html" || isHtmlContent(content)) {
    html = content;
  } else if (contentType === "text") {
    html = `<pre style="white-space:pre-wrap">${escapeHtml(content)}</pre>`;
  } else {
    try {
      html = marked.parse(content, { breaks: true }) as string;
    } catch {
      html = `<pre>${escapeHtml(content)}</pre>`;
    }
  }
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ["p", "br", "strong", "em", "code", "pre", "ul", "ol", "li", "a", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "hr", "table", "thead", "tbody", "tr", "th", "td", "span", "div", "img", "svg", "path", "circle", "rect", "line", "text"],
    ALLOWED_ATTR: ["href", "target", "rel", "src", "alt", "class", "style", "width", "height", "viewBox", "fill", "stroke", "strokeWidth", "d", "cx", "cy", "r", "x", "y", "rx", "ry", "xmlns", "textAnchor", "fontSize", "fontWeight"],
    ALLOW_DATA_ATTR: false,
  });
}

const ChatPanel: React.FC<Props> = ({ messages, agentBusy, streamingContent, streamingContentType }) => {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingContent]);

  return (
    <div className="chat-panel glass-panel">
      <div className="chat-header">
        <span>{agentBusy ? "思考中..." : "对话"}</span>
      </div>
      <div className="chat-messages">
        {messages.length === 0 && !streamingContent && (
          <div className="chat-empty">
            <svg width="40" height="40" viewBox="0 0 40 40" fill="none" opacity="0.3">
              <circle cx="20" cy="20" r="18" stroke="currentColor" strokeWidth="1.5" />
              <path d="M14 18l6 6 6-6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
            <p>输入消息开始对话</p>
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} className={`message message-${msg.role}`}>
            <div className="message-role">{msg.role}</div>
            <div
              className="message-content"
              dangerouslySetInnerHTML={{ __html: renderContent(msg.content, msg.contentType) }}
            />
          </div>
        ))}
        {streamingContent && (
          <div className="message message-assistant message-streaming">
            <div className="message-role">assistant</div>
            <div
              className="message-content"
              dangerouslySetInnerHTML={{
                __html: renderContent(streamingContent, streamingContentType),
              }}
            />
            <span className="streaming-cursor">▊</span>
          </div>
        )}
        <div ref={bottomRef} />
      </div>
      <style>{`
        @keyframes blink { 0%, 100% { opacity: 1; } 50% { opacity: 0; } }
        .streaming-cursor { animation: blink 1s step-end infinite; margin-left: 2px; }
        .message-streaming { opacity: 0.85; border-left: 3px solid var(--mac-primary); }
      `}</style>
    </div>
  );
};

export default ChatPanel;
