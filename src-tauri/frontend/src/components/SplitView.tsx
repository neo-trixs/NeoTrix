import React, { useState, useRef, useCallback } from "react";
import { marked } from "marked";
import DOMPurify from "dompurify";
import * as api from "../lib/api";

const MODEL_OPTIONS = [
  { value: "claude-sonnet-4-20250514", label: "Claude Sonnet 4" },
  { value: "gpt-4o", label: "GPT-4o" },
  { value: "gemini-2.5-pro", label: "Gemini 2.5 Pro" },
  { value: "deepseek-coder-v2", label: "DeepSeek Coder V2" },
  { value: "qwen2.5:32b", label: "Qwen 2.5 (Ollama)" },
];

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function renderMarkdown(content: string): string {
  let html: string;
  try {
    html = marked.parse(content, { breaks: true }) as string;
  } catch {
    html = `<pre style="white-space:pre-wrap">${escapeHtml(content)}</pre>`;
  }
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ["p", "br", "strong", "em", "code", "pre", "ul", "ol", "li", "a", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "hr", "table", "thead", "tbody", "tr", "th", "td", "span", "div", "img", "svg", "path", "circle", "rect", "line", "text"],
    ALLOWED_ATTR: ["href", "target", "rel", "src", "alt", "class", "style", "width", "height", "viewBox", "fill", "stroke", "strokeWidth", "d", "cx", "cy", "r", "x", "y", "rx", "ry", "xmlns", "textAnchor", "fontSize", "fontWeight"],
    ALLOW_DATA_ATTR: false,
  });
}

const SplitView: React.FC = () => {
  const [leftModel, setLeftModel] = useState(MODEL_OPTIONS[0].value);
  const [rightModel, setRightModel] = useState(MODEL_OPTIONS[1].value);
  const [leftPrompt, setLeftPrompt] = useState("");
  const [rightPrompt, setRightPrompt] = useState("");
  const [leftResponse, setLeftResponse] = useState("");
  const [rightResponse, setRightResponse] = useState("");
  const [leftLoading, setLeftLoading] = useState(false);
  const [rightLoading, setRightLoading] = useState(false);
  const [leftError, setLeftError] = useState("");
  const [rightError, setRightError] = useState("");
  const [splitPercent, setSplitPercent] = useState(50);
  const draggingRef = useRef(false);

  const handleCompare = useCallback(async () => {
    const prompt = leftPrompt || rightPrompt;
    if (!prompt.trim()) return;

    setLeftLoading(true);
    setRightLoading(true);
    setLeftResponse("");
    setRightResponse("");
    setLeftError("");
    setRightError("");

    const [leftResult, rightResult] = await Promise.all([
      api.agentReason(prompt),
      api.agentReason(prompt),
    ]);

    if (leftResult.success) {
      setLeftResponse(leftResult.output);
    } else {
      setLeftError(leftResult.output || "左面板请求失败");
    }

    if (rightResult.success) {
      setRightResponse(rightResult.output);
    } else {
      setRightError(rightResult.output || "右面板请求失败");
    }

    setLeftLoading(false);
    setRightLoading(false);
  }, [leftPrompt, rightPrompt]);

  const handleLeftSend = useCallback(async () => {
    if (!leftPrompt.trim()) return;
    setLeftLoading(true);
    setLeftResponse("");
    setLeftError("");
    const result = await api.agentReason(leftPrompt);
    if (result.success) {
      setLeftResponse(result.output);
    } else {
      setLeftError(result.output || "请求失败");
    }
    setLeftLoading(false);
  }, [leftPrompt]);

  const handleRightSend = useCallback(async () => {
    if (!rightPrompt.trim()) return;
    setRightLoading(true);
    setRightResponse("");
    setRightError("");
    const result = await api.agentReason(rightPrompt);
    if (result.success) {
      setRightResponse(result.output);
    } else {
      setRightError(result.output || "请求失败");
    }
    setRightLoading(false);
  }, [rightPrompt]);

  const handleDividerMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    draggingRef.current = true;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";

    const handleMouseMove = (ev: MouseEvent) => {
      if (!draggingRef.current) return;
      const container = document.querySelector(".split-view");
      if (!container) return;
      const rect = container.getBoundingClientRect();
      const pct = ((ev.clientX - rect.left) / rect.width) * 100;
      setSplitPercent(Math.min(Math.max(pct, 15), 85));
    };

    const handleMouseUp = () => {
      draggingRef.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  }, []);

  return (
    <div className="split-view glass-panel">
      <div className="split-view-toolbar">
        <button
          className="btn-primary"
          onClick={handleCompare}
          disabled={leftLoading || rightLoading}
        >
          {leftLoading || rightLoading ? "比较中..." : "比较"}
        </button>
        <span className="split-toolbar-hint">将相同提示词发送至两个模型</span>
      </div>
      <div className="split-view-body">
        <div className="split-panel" style={{ width: `${splitPercent}%` }}>
          <div className="split-panel-header">
            <select
              className="split-model-select"
              value={leftModel}
              onChange={(e) => setLeftModel(e.target.value)}
            >
              {MODEL_OPTIONS.map((m) => (
                <option key={m.value} value={m.value}>{m.label}</option>
              ))}
            </select>
            <button
              className="btn-send split-send-btn"
              onClick={handleLeftSend}
              disabled={leftLoading || !leftPrompt.trim()}
            >
              发送
            </button>
          </div>
          <textarea
            className="split-prompt-input"
            placeholder="左侧提示词..."
            value={leftPrompt}
            onChange={(e) => setLeftPrompt(e.target.value)}
            rows={2}
          />
          <div className="split-response">
            {leftLoading && (
              <div className="split-loading">
                <span className="split-spinner" />
                <span>思考中...</span>
              </div>
            )}
            {leftError && !leftLoading && (
              <div className="split-response-error">{leftError}</div>
            )}
            {leftResponse && !leftLoading && (
              <div
                className="message-content"
                dangerouslySetInnerHTML={{ __html: renderMarkdown(leftResponse) }}
              />
            )}
            {!leftResponse && !leftLoading && !leftError && (
              <div className="split-response-empty">输入提示词并发送</div>
            )}
          </div>
        </div>

        <div
          className="split-divider"
          onMouseDown={handleDividerMouseDown}
        >
          <div className="split-divider-grip" />
        </div>

        <div className="split-panel" style={{ width: `${100 - splitPercent}%` }}>
          <div className="split-panel-header">
            <select
              className="split-model-select"
              value={rightModel}
              onChange={(e) => setRightModel(e.target.value)}
            >
              {MODEL_OPTIONS.map((m) => (
                <option key={m.value} value={m.value}>{m.label}</option>
              ))}
            </select>
            <button
              className="btn-send split-send-btn"
              onClick={handleRightSend}
              disabled={rightLoading || !rightPrompt.trim()}
            >
              发送
            </button>
          </div>
          <textarea
            className="split-prompt-input"
            placeholder="右侧提示词..."
            value={rightPrompt}
            onChange={(e) => setRightPrompt(e.target.value)}
            rows={2}
          />
          <div className="split-response">
            {rightLoading && (
              <div className="split-loading">
                <span className="split-spinner" />
                <span>思考中...</span>
              </div>
            )}
            {rightError && !rightLoading && (
              <div className="split-response-error">{rightError}</div>
            )}
            {rightResponse && !rightLoading && (
              <div
                className="message-content"
                dangerouslySetInnerHTML={{ __html: renderMarkdown(rightResponse) }}
              />
            )}
            {!rightResponse && !rightLoading && !rightError && (
              <div className="split-response-empty">输入提示词并发送</div>
            )}
          </div>
        </div>
      </div>
      <style>{`
        @keyframes split-spin {
          to { transform: rotate(360deg); }
        }
        .split-spinner {
          display: inline-block;
          width: 14px; height: 14px;
          border: 2px solid var(--mac-border);
          border-top-color: var(--mac-primary);
          border-radius: 50%;
          animation: split-spin 0.6s linear infinite;
        }
      `}</style>
    </div>
  );
};

export default SplitView;
