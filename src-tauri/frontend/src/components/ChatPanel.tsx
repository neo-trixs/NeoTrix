import React, { useEffect, useRef, useState, useCallback } from "react";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { invoke } from "@tauri-apps/api/core";
import type { Message } from "../types";
import DiffView from "./DiffView";
import ConsciousnessPipeline from "./ConsciousnessPipeline";

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

function renderToolCallArgs(args: Record<string, string>): string {
  const lines: string[] = [];
  for (const [k, v] of Object.entries(args)) {
    lines.push(`${escapeHtml(k)}: ${escapeHtml(v)}`);
  }
  return lines.join("\n");
}

const ChatPanel: React.FC<Props> = ({ messages, agentBusy, streamingContent, streamingContentType }) => {
  const bottomRef = useRef<HTMLDivElement>(null);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [diffStatus, setDiffStatus] = useState<Record<string, "accepted" | "rejected">>({});
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingContent]);

  const toggleExpand = useCallback((key: string) => {
    setExpanded(prev => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
  }, []);

  function renderSteps(steps: { label: string; status: "done" | "running" | "pending" }[]) {
    return (
      <ol className="step-list">
        {steps.map((s, j) => (
          <li key={j} className={`step-item step-status-${s.status}`}>
            <span className="step-indicator">
              {s.status === "done" ? "✅" : s.status === "running" ? "⏳" : "⏺️"}
            </span>
            <span className="step-label">{s.label}</span>
          </li>
        ))}
      </ol>
    );
  }

  function renderDiffs(diffs: { file: string; added: number; removed: number; diff: string }[], _msgIdx: number) {
    return (
      <div className="diff-blocks">
        {diffs.map((d, j) => {
          const key = `${_msgIdx}-${j}`;
          const status = diffStatus[key];
          return (
            <DiffView
              key={j}
              file={d.file}
              added={d.added}
              removed={d.removed}
              diff={d.diff}
              status={status}
              onAccept={status ? undefined : async () => {
                try {
                  await invoke("apply_diff", { file: d.file, content: d.diff });
                  setDiffStatus((prev) => ({ ...prev, [key]: "accepted" }));
                } catch {
                  setDiffStatus((prev) => ({ ...prev, [key]: "rejected" }));
                }
              }}
              onReject={status ? undefined : () => {
                setDiffStatus((prev) => ({ ...prev, [key]: "rejected" }));
              }}
            />
          );
        })}
      </div>
    );
  }

  return (
    <div className="chat-panel glass-panel">
      <div className="chat-header">
        <span>{agentBusy ? "Active" : "对话"}</span>
      </div>
      <ConsciousnessPipeline />
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
        {messages.map((msg, i) => {
          const ek = `m-${i}`;
          const isExpanded = expanded.has(ek);

          if (msg.role === "tool_call") {
            return (
              <div key={i} className="message message-tool_call">
                <div className="tool-call-compact" onClick={() => toggleExpand(ek)}>
                  <span className="tool-call-icon">🔧</span>
                  <span className="tool-call-name">{msg.toolCall?.tool ?? msg.content}</span>
                  <span className="tool-call-args-preview">
                    ({msg.toolCall ? Object.keys(msg.toolCall.args).join(", ") : ""})
                  </span>
                  <span className="tool-call-toggle">{isExpanded ? "▲" : "▼"}</span>
                </div>
                {isExpanded && (
                  <div className="tool-call-expanded">
                    <pre className="tool-call-args">{msg.toolCall ? renderToolCallArgs(msg.toolCall.args) : escapeHtml(msg.content)}</pre>
                    {msg.toolCall?.result !== undefined && (
                      <>
                        <div className="tool-call-result-header">Result:</div>
                        <pre className="tool-call-result">{escapeHtml(msg.toolCall.result)}</pre>
                      </>
                    )}
                  </div>
                )}
              </div>
            );
          }

          if (msg.role === "tool_result") {
            return (
              <div key={i} className="message message-tool_result">
                <div className="tool-result-compact" onClick={() => toggleExpand(ek)}>
                  <span className="tool-result-icon">📋</span>
                  <span className="tool-result-label">Result</span>
                  {msg.toolCall?.duration_ms !== undefined && (
                    <span className="tool-result-duration">{msg.toolCall.duration_ms}ms</span>
                  )}
                  <span className="tool-call-toggle">{isExpanded ? "▲" : "▼"}</span>
                </div>
                {isExpanded && (
                  <div className="tool-call-expanded">
                    <pre className="tool-call-result">{escapeHtml(msg.content)}</pre>
                  </div>
                )}
              </div>
            );
          }

          const mainContent = msg.collapsible && !isExpanded ? (
            <div>
              <div
                className="message-content"
                dangerouslySetInnerHTML={{ __html: renderContent(msg.content, msg.contentType) }}
              />
            </div>
          ) : (
            <div
              className="message-content"
              dangerouslySetInnerHTML={{ __html: renderContent(msg.content, msg.contentType) }}
            />
          );

          return (
            <div key={i} className={`message message-${msg.role}`}>
              <div className="message-role">{msg.role}</div>

              {msg.collapsible ? (
                <div className="collapsible-wrapper">
                  {isExpanded ? (
                    <>
                      <div
                        className="message-content"
                        dangerouslySetInnerHTML={{ __html: renderContent(msg.content, msg.contentType) }}
                      />
                      <span className="collapsible-toggle" onClick={() => toggleExpand(ek)}>Hide details ▲</span>
                    </>
                  ) : (
                    <span className="collapsible-toggle" onClick={() => toggleExpand(ek)}>Show details... ▼</span>
                  )}
                </div>
              ) : (
                <div
                  className="message-content"
                  dangerouslySetInnerHTML={{ __html: renderContent(msg.content, msg.contentType) }}
                />
              )}

              {msg.toolCall && (() => {
                const tck = `tc-${i}`;
                const tcOpen = expanded.has(tck);
                return (
                  <div className="tool-call-sub-block">
                    <div className="tool-call-compact sub" onClick={() => toggleExpand(tck)}>
                      <span className="tool-call-icon">🔧</span>
                      <span className="tool-call-name">{msg.toolCall.tool}</span>
                      <span className="tool-call-args-preview">
                        ({Object.keys(msg.toolCall.args).join(", ")})
                      </span>
                      <span className="tool-call-toggle">{tcOpen ? "▲" : "▼"}</span>
                    </div>
                    {tcOpen && (
                      <div className="tool-call-expanded">
                        <pre className="tool-call-args">{renderToolCallArgs(msg.toolCall.args)}</pre>
                        {msg.toolCall.result !== undefined && (
                          <>
                            <div className="tool-call-result-header">Result:</div>
                            <pre className="tool-call-result">{escapeHtml(msg.toolCall.result)}</pre>
                          </>
                        )}
                      </div>
                    )}
                  </div>
                );
              })()}

              {msg.steps && renderSteps(msg.steps)}

              {msg.diffs && renderDiffs(msg.diffs, i)}
            </div>
          );
        })}
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

        .tool-call-compact, .tool-result-compact {
          cursor: pointer; display: flex; align-items: center; gap: 6px;
          padding: 4px 8px; border-radius: 6px; font-size: 0.85em;
          background: var(--mac-surface-2, #1e1e2e); user-select: none;
        }
        .tool-call-compact:hover, .tool-result-compact:hover {
          background: var(--mac-surface-3, #2a2a3e);
        }
        .tool-call-compact.sub {
          margin-top: 6px; font-size: 0.8em;
        }
        .tool-call-icon, .tool-result-icon { flex-shrink: 0; }
        .tool-call-name { font-weight: 600; color: var(--mac-primary, #7c5cfc); }
        .tool-call-args-preview { color: var(--mac-text-secondary, #888); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        .tool-call-toggle, .tool-result-toggle { margin-left: auto; font-size: 0.75em; color: var(--mac-text-secondary, #888); }
        .tool-result-label { font-weight: 600; }
        .tool-result-duration { color: var(--mac-text-secondary, #888); font-size: 0.85em; }

        .tool-call-expanded {
          padding: 8px; margin-top: 4px; border-radius: 6px;
          background: var(--mac-surface-1, #141420);
        }
        .tool-call-args, .tool-call-result {
          margin: 0; font-size: 0.8em; white-space: pre-wrap; word-break: break-all;
          color: var(--mac-text, #cdd6f4);
        }
        .tool-call-result-header {
          font-size: 0.8em; font-weight: 600; margin-top: 8px; margin-bottom: 4px;
          color: var(--mac-text-secondary, #888);
        }

        .step-list { margin: 6px 0 0 0; padding: 0 0 0 20px; list-style: none; }
        .step-item { display: flex; align-items: center; gap: 6px; font-size: 0.8em; padding: 2px 0; }
        .step-indicator { flex-shrink: 0; font-size: 0.9em; }
        .step-label { color: var(--mac-text, #cdd6f4); }
        .step-status-done { opacity: 0.7; }
        .step-status-running { opacity: 1; }
        .step-status-pending { opacity: 0.4; }

        .collapsible-wrapper { display: flex; flex-direction: column; }
        .collapsible-toggle {
          cursor: pointer; font-size: 0.8em; color: var(--mac-primary, #7c5cfc);
          user-select: none; margin-top: 4px;
        }
        .collapsible-toggle:hover { text-decoration: underline; }
      `}</style>
    </div>
  );
};

export default ChatPanel;
