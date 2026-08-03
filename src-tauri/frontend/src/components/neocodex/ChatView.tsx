import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../../stores";
import type { Attachment, Message } from "../../types";
import styles from "./ChatView.module.css";

const USER_AVATAR = `<svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="10" height="8" rx="1.5"/><circle cx="7" cy="7" r="1.5"/></svg>`;
const ASSISTANT_AVATAR = `<svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4l6 3-6 3V4z"/></svg>`;

function isHtmlContent(content: string): boolean {
  return /^\s*<(html|div|span|p|h[1-6]|table|ul|ol|section|article|header|footer|main|aside|nav|form|input|button|select|textarea|img|video|audio|canvas|svg|figure|figcaption|details|summary|dialog|data|time|mark|ruby|rt|rp|bdi|bdo|wbr|code|pre|blockquote|dl|dt|dd)[\s>]/i.test(content.trim());
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
  return {
    html: DOMPurify.sanitize(html, {
      ALLOWED_TAGS: ["p", "br", "strong", "em", "code", "pre", "ul", "ol", "li", "a", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "hr", "table", "thead", "tbody", "tr", "th", "td", "span", "div", "img", "svg", "path", "circle", "rect", "line", "text", "button", "input", "textarea", "span"],
      ALLOWED_ATTR: ["href", "target", "rel", "src", "alt", "class", "style", "width", "height", "viewBox", "fill", "stroke", "strokeWidth", "d", "cx", "cy", "r", "x", "y", "rx", "ry", "xmlns", "textAnchor", "fontSize", "fontWeight", "type", "checked", "disabled", "value", "placeholder", "rows", "data-copy-id"],
      ALLOW_DATA_ATTR: true,
    }),
    codeBlocks,
  };
}

// Injects a copy button into each <pre> block so users can copy a single
// code block (Claude Code / Codex parity) without copying the whole reply.
function decorateCodeBlocks(html: string, id: string): string {
  if (!html.includes("<pre")) return html;
  let idx = 0;
  return html.replace(/<pre[^>]*>([\s\S]*?)<\/pre>/g, (match, inner) => {
    const copyId = `${id}-cb-${idx++}`;
    return (
      `<div class="codeblock-wrap" style="position:relative">
         <button type="button" class="codeblock-copy" data-copy-id="${copyId}" title="复制代码" aria-label="复制代码" style="position:absolute;top:6px;right:6px;z-index:2;padding:3px 8px;font-size:11px;border-radius:4px;border:1px solid var(--border-primary);background:var(--bg-tertiary);color:var(--fg-secondary);cursor:pointer;opacity:0.6">复制</button>
         <pre data-copy-id="${copyId}" style="position:relative">${inner}</pre>
       </div>`
    );
  });
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

function formatRelative(ts?: number): string {
  if (!ts) return "";
  const t = ts > 1e12 ? ts : ts * 1000;
  const diff = Date.now() - t;
  if (diff < 3600000) return `${Math.max(1, Math.floor(diff / 60000))}分钟前`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`;
  return `${Math.floor(diff / 86400000)}天前`;
}

function formatFileSize(size: number): string {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / 1024 / 1024).toFixed(1)} MB`;
}

async function readFileAsBase64(file: File): Promise<string> {
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  const CHUNK = 0x8000;
  let binary = "";
  for (let i = 0; i < bytes.length; i += CHUNK) {
    binary += String.fromCharCode(...bytes.subarray(i, i + CHUNK));
  }
  return btoa(binary);
}

interface ChatViewProps {
  messages: Message[];
  streamingContent?: string;
  streamingRole?: "user" | "assistant";
  agentBusy: boolean;
  onSend: (content: string, attachments?: Attachment[]) => void;
  onEdit?: (id: number, content: string) => void;
  onRegenerate?: (id: number) => void;
  onDelete?: (id: number) => void;
  viewMode?: "verbose" | "normal" | "summary";
  contextUsage?: number;
  pendingPlanExecute?: boolean;
  onPlanApprove?: () => void;
  onStop?: () => void;
  recentSessions?: Array<{ id: string; name: string; mode: string; updated_at: number }>;
  onRecentSessionSelect?: (id: string) => void;
  onSlashAction?: (cmd: string, arg: string) => void;
  mode?: "Agent" | "Shell" | "Plan";
  onModeChange?: (mode: string) => void;
}

const SLASH_COMMANDS = [
  { cmd: "/compact", label: "压缩会话", hint: "释放上下文 token" },
  { cmd: "/goal <desc>", label: "设置目标", hint: "添加进化目标" },
  { cmd: "/plan", label: "计划模式", hint: "切换到 Plan 模式" },
  { cmd: "/status", label: "查看状态", hint: "模型·用量·会话信息" },
  { cmd: "/btw", label: "侧聊", hint: "打开侧边聊天" },
  { cmd: "/new", label: "新会话", hint: "创建新会话" },
  { cmd: "/model <name>", label: "切换模型", hint: "切换当前模型" },
  { cmd: "/init", label: "初始化项目", hint: "生成 AGENTS.md 与项目配置" },
  { cmd: "/feedback <text>", label: "反馈", hint: "给助手反馈" },
  { cmd: "/export", label: "导出会话", hint: "导出为 Markdown" },
  { cmd: "/rename <name>", label: "重命名会话", hint: "重命名当前会话" },
  { cmd: "/clear", label: "清除消息", hint: "清空当前会话消息" },
];

const QUICK_ACTIONS = [
  { icon: "🔍", label: "分析项目结构", desc: "梳理代码库模块与依赖", prompt: "请分析当前项目的整体结构，包括主要模块、入口点、依赖关系和数据流，给出架构概述。" },
  { icon: "🐛", label: "排查 Bug", desc: "定位并解释代码问题", prompt: "请帮我排查代码中的潜在 Bug。重点关注未处理的错误、竞态条件、边界情况和资源泄漏，指出问题位置并给出修复建议。" },
  { icon: "🧪", label: "生成单元测试", desc: "为关键逻辑补测试", prompt: "请为当前项目中的关键模块生成单元测试，遵循现有测试约定，覆盖正常路径与边界情况，并给出运行命令。" },
  { icon: "📐", label: "制定实施计划", desc: "拆解功能为可执行步骤", prompt: "请为当前任务制定一份实施计划，将其拆解为可执行步骤，包含前置检查、实现要点、测试与验证方式。" },
];

export function ChatView({
  messages,
  streamingContent,
  streamingRole = "assistant",
  agentBusy,
  onSend,
  onEdit,
  onRegenerate,
  onDelete,
  viewMode = "normal",
  contextUsage = 0,
  onStop,
  recentSessions,
  onRecentSessionSelect,
  pendingPlanExecute = false,
  onPlanApprove,
  onSlashAction,
  mode,
  onModeChange,
}: ChatViewProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const messagesScrollRef = useRef<HTMLElement>(null);
  const [showScrollDown, setShowScrollDown] = useState(false);
  const stickToBottomRef = useRef(true);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const dragCounter = useRef(0);
  const [input, setInput] = useState("");
  const [showSlash, setShowSlash] = useState(false);
  const [slashQuery, setSlashQuery] = useState("");
  const [slashHighlight, setSlashHighlight] = useState(0);
  const slashRef = useRef<HTMLDivElement>(null);
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  const [dragging, setDragging] = useState(false);
  const [mentionOpen, setMentionOpen] = useState(false);
  const [mentionQuery, setMentionQuery] = useState("");
  const [mentionFiles, setMentionFiles] = useState<string[]>([]);
  const [mentionDirs, setMentionDirs] = useState<string[]>([]);
  const [mentions, setMentions] = useState<string[]>([]);
  const [mentionHighlight, setMentionHighlight] = useState(0);
  const mentionRef = useRef<HTMLDivElement>(null);
  const addNotification = useStore((s) => s.addNotification);
  const permMode = useStore((s) => s.settings?.permissionMode) || "auto";
  const defaultModel = useStore((s) => s.settings?.defaultModel) || "GatewayV2";
  const storeMode = useStore((s) => s.neocodexMode);
  const activeMode = mode || storeMode;
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [lastInput, setLastInput] = useState("");
  const [queuedInputs, setQueuedInputs] = useState<Array<{ content: string; attachments: Attachment[] }>>([]);
  const queueRef = useRef<Array<{ content: string; attachments: Attachment[] }>>([]);
  const promptHistory = useRef<string[]>([]);

  const flushQueue = useCallback(() => {
    if (agentBusy) return;
    const next = queueRef.current.shift();
    if (next) {
      setQueuedInputs([...queueRef.current]);
      onSend(next.content, next.attachments.length > 0 ? next.attachments : undefined);
    }
  }, [agentBusy, onSend]);

  const filteredSlash = useMemo(() => {
    const q = slashQuery.trim().toLowerCase();
    if (!q) return SLASH_COMMANDS;
    return SLASH_COMMANDS.filter((c) => c.cmd.toLowerCase().includes(q) || c.label.toLowerCase().includes(q));
  }, [slashQuery]);

  const handleSlashSelect = (cmd: string) => {
    const prefix = cmd.split(" ")[0];
    // P2-4: slash commands that map to real backend operations are dispatched
    // immediately instead of merely being inserted as text. `/init`, `/export`,
    // `/clear`, `/feedback` previously did nothing when sent (they reached the
    // model as plain text or were no-ops), while the matching backend commands
    // existed but were never invoked by the UI.
    if (prefix === "/init" || prefix === "/export" || prefix === "/clear") {
      onSlashAction?.(prefix, "");
      setInput("");
      setShowSlash(false);
      setSlashHighlight(0);
      textareaRef.current?.focus();
      return;
    }
    if (prefix === "/feedback") {
      const rest = cmd.slice("/feedback".length).trim();
      if (rest) {
        onSlashAction?.("/feedback", rest);
        setInput("");
      } else {
        setInput("/feedback ");
      }
      setShowSlash(false);
      setSlashHighlight(0);
      textareaRef.current?.focus();
      return;
    }
    setInput((prev) => prev.replace(/\/\w*$/, "") + prefix + " ");
    setShowSlash(false);
    setSlashHighlight(0);
    textareaRef.current?.focus();
  };

  useEffect(() => {
    if (!mentionOpen) return;
    let cancelled = false;
    setMentionHighlight(0);
    (async () => {
      try {
        const q = mentionQuery.trim().toLowerCase();
        const entries = await invoke<string[]>("neocodex_search_files", { query: q });
        if (cancelled) return;
        const files = entries.filter((e) => !e.endsWith("/"));
        const dirs = entries.filter((e) => e.endsWith("/")).map((e) => e.slice(0, -1));
        setMentionFiles(files.slice(0, 40));
        setMentionDirs(dirs);
      } catch {
        if (!cancelled) {
          setMentionFiles([]);
          setMentionDirs([]);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [mentionOpen, mentionQuery]);

  useEffect(() => {
    if (!mentionOpen) return;
    const onDown = (ev: MouseEvent) => {
      const target = ev.target as HTMLElement;
      const inMenu = mentionRef.current?.contains(target);
      const inTextarea = textareaRef.current?.contains(target);
      if (!inMenu && !inTextarea) {
        setMentionOpen(false);
        setMentionQuery("");
      }
    };
    document.addEventListener("pointerdown", onDown);
    return () => document.removeEventListener("pointerdown", onDown);
  }, [mentionOpen]);

  useEffect(() => {
    if (!showSlash) return;
    const onDown = (ev: MouseEvent) => {
      const target = ev.target as HTMLElement;
      const inMenu = slashRef.current?.contains(target);
      const inTextarea = textareaRef.current?.contains(target);
      if (!inMenu && !inTextarea) {
        setShowSlash(false);
        setSlashQuery("");
      }
    };
    document.addEventListener("pointerdown", onDown);
    return () => document.removeEventListener("pointerdown", onDown);
  }, [showSlash]);

  useEffect(() => {
    const onKey = (ev: KeyboardEvent) => {
      if ((ev.metaKey || ev.ctrlKey) && ev.shiftKey && (ev.key === "C" || ev.key === "c")) {
        const lastAssistant = [...messages].reverse().find((m) => m.role === "assistant");
        if (lastAssistant && lastAssistant.content) {
          ev.preventDefault();
          copyMessage(lastAssistant.content);
          addNotification({ type: "success", message: "已复制最后一条回复", duration: 2000 });
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [messages, addNotification]);

  useEffect(() => {
    const onPick = (ev: Event) => {
      const path = (ev as CustomEvent<string>).detail;
      if (!path) return;
      setInput((prev) => prev + `@${path} `);
      setMentions((prev) => (prev.includes(path) ? prev : [...prev, path]));
      textareaRef.current?.focus();
      addNotification({ type: "info", message: `已引用 ${path}`, duration: 2000 });
    };
    window.addEventListener("neotrix:mention-file", onPick);
    return () => window.removeEventListener("neotrix:mention-file", onPick);
  }, [addNotification]);

  const handleMentionSelect = (name: string) => {
    setInput((prev) => prev.replace(/@\w*$/, "") + `@${name} `);
    setMentionOpen(false);
    setMentionQuery("");
    setMentionHighlight(0);
    setMentions((prev) => (prev.includes(name) ? prev : [...prev, name]));
    textareaRef.current?.focus();
  };

  const handleCompact = () => {
    setInput("/compact ");
    setShowSlash(false);
    setMentionOpen(false);
    textareaRef.current?.focus();
    addNotification({ type: "info", message: "已输入 /compact，回车发送", duration: 2000 });
  };

  const handleFiles = async (files: FileList | File[]) => {
    const incoming: Attachment[] = [];
    const MAX_ATTACHMENT_BYTES = 10 * 1024 * 1024;
    for (const file of Array.from(files)) {
      if (file.size > MAX_ATTACHMENT_BYTES) {
        addNotification({ type: "error", message: `「${file.name}」超过 10 MB 上限，未附加`, duration: 3000 });
        continue;
      }
      incoming.push({
        id: typeof crypto !== "undefined" && typeof crypto.randomUUID === "function" ? crypto.randomUUID() : `${Date.now()}-${Math.random().toString(36).slice(2)}`,
        name: file.name,
        size: file.size,
        mimeType: file.type || "application/octet-stream",
        data: await readFileAsBase64(file),
      });
    }
    setAttachments((prev) => [...prev, ...incoming]);
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) handleFiles(e.target.files);
    e.target.value = "";
  };

  const handleDragEnter = (e: React.DragEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (agentBusy) return;
    dragCounter.current++;
    setDragging(true);
  };

  const handleDragLeave = () => {
    dragCounter.current = Math.max(0, dragCounter.current - 1);
    if (dragCounter.current === 0) setDragging(false);
  };

  const handleDrop = (e: React.DragEvent<HTMLFormElement>) => {
    e.preventDefault();
    dragCounter.current = 0;
    setDragging(false);
    if (agentBusy) return;
    if (e.dataTransfer.files.length > 0) handleFiles(e.dataTransfer.files);
  };

  // Steer Mode (Codex v26): while the agent is busy, Enter/Tab queue the input
  // instead of dropping it. When idle, queued inputs flush in order.
  const queueCurrentInput = () => {
    const content = input.trim();
    if (!content) return;
    queueRef.current.push({ content, attachments });
    setQueuedInputs([...queueRef.current]);
    setInput("");
    setShowSlash(false);
    setMentionOpen(false);
    setMentionQuery("");
    setAttachments([]);
    textareaRef.current?.focus();
  };

  useEffect(() => {
    if (queuedInputs.length > 0 && !agentBusy) {
      const t = setTimeout(flushQueue, 50);
      return () => clearTimeout(t);
    }
  }, [queuedInputs, agentBusy, flushQueue]);

  const scrollToBottom = useCallback((smooth = false) => {
    const el = messagesScrollRef.current;
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior: smooth ? "smooth" : "auto" });
    stickToBottomRef.current = true;
    setShowScrollDown(false);
  }, []);

  useEffect(() => {
    const el = messagesScrollRef.current;
    if (!el) return;
    const onScroll = () => {
      const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 120;
      stickToBottomRef.current = nearBottom;
      setShowScrollDown(!nearBottom);
    };
    el.addEventListener("scroll", onScroll, { passive: true });
    return () => el.removeEventListener("scroll", onScroll);
  }, []);

  // Auto-stick to the newest content while streaming / on new messages,
  // unless the user has scrolled up to read earlier output.
  useEffect(() => {
    if (stickToBottomRef.current) {
      messagesEndRef.current?.scrollIntoView({ block: "end" });
    }
  }, [messages, streamingContent, viewMode]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (showSlash) {
      if (e.key === "ArrowDown") { e.preventDefault(); setSlashHighlight((h) => (h + 1) % Math.max(filteredSlash.length, 1)); return; }
      else if (e.key === "ArrowUp") { e.preventDefault(); setSlashHighlight((h) => (h - 1 + Math.max(filteredSlash.length, 1)) % Math.max(filteredSlash.length, 1)); return; }
      else if (e.key === "Enter" && filteredSlash.length > 0) { e.preventDefault(); handleSlashSelect(filteredSlash[Math.min(slashHighlight, filteredSlash.length - 1)].cmd); return; }
      else if (e.key === "Escape") { setShowSlash(false); setSlashHighlight(0); return; }
      return;
    }
    if (mentionOpen) {
      if (e.key === "Escape") { e.preventDefault(); setMentionOpen(false); setMentionQuery(""); return; }
      else if (e.key === "ArrowDown") { e.preventDefault(); setMentionHighlight((h) => (h + 1) % Math.max(mentionFiles.length, 1)); return; }
      else if (e.key === "ArrowUp") { e.preventDefault(); setMentionHighlight((h) => (h - 1 + Math.max(mentionFiles.length, 1)) % Math.max(mentionFiles.length, 1)); return; }
      else if (e.key === "Enter" && mentionFiles.length > 0) { e.preventDefault(); handleMentionSelect(mentionFiles[Math.min(mentionHighlight, mentionFiles.length - 1)]); return; }
    }
    // Steer mode: when busy, Enter and Tab queue input instead of sending.
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (input.trim()) {
        if (agentBusy) {
          queueCurrentInput();
          addNotification({ type: "info", message: "已排队，稍后自动发送", duration: 2000 });
        } else {
          const form = (e.target as HTMLTextAreaElement).closest("form");
          form?.requestSubmit();
        }
      }
      return;
    }
    if (e.key === "Tab" && agentBusy) {
      e.preventDefault();
      queueCurrentInput();
      addNotification({ type: "info", message: "已排队，稍后自动发送", duration: 2000 });
      return;
    }
    if (e.key === "ArrowUp" && !input) {
      e.preventDefault();
      if (historyIndex === -1) setLastInput("");
      if (promptHistory.current.length > 0) {
        const next = historyIndex + 1 < promptHistory.current.length ? historyIndex + 1 : historyIndex;
        if (next >= 0 && next < promptHistory.current.length) {
          setHistoryIndex(next);
          setInput(promptHistory.current[promptHistory.current.length - 1 - next]);
        }
      }
    } else if (e.key === "ArrowDown" && historyIndex >= 0) {
      e.preventDefault();
      const next = historyIndex - 1;
      if (next < 0) {
        setHistoryIndex(-1);
        setInput(lastInput);
      } else {
        setHistoryIndex(next);
        setInput(promptHistory.current[promptHistory.current.length - 1 - next]);
      }
    }
  };

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!input.trim() || agentBusy) return;
    const content = input.trim();
    setInput("");
    setShowSlash(false);
    setSlashHighlight(0);
    setMentionOpen(false);
    setMentionQuery("");
    if (attachments.length > 0) {
      addNotification({ type: "info", message: `已附加 ${attachments.length} 个文件`, duration: 2000 });
    }
    if (mentions.length > 0) {
      addNotification({ type: "info", message: `引用 ${mentions.length} 个文件`, duration: 2000 });
    }
    promptHistory.current.push(content);
    if (promptHistory.current.length > 100) promptHistory.current.shift();
    setHistoryIndex(-1);
    // P2-4: intercept first-tier slash commands at submit time instead of
    // shipping them to the model as plain text. `/plan /model /new /rename`
    // are real actions with backend/UI support; sending them as text makes the
    // model guess at intent. Insert-only hints like `/compact /status /btw
    // /goal` still pass through so the model can interpret them.
    const slashPrefix = content.split(/\s+/)[0];
    if (slashPrefix === "/plan" || slashPrefix === "/model" || slashPrefix === "/new" || slashPrefix === "/rename" || slashPrefix === "/export" || slashPrefix === "/clear") {
      onSlashAction?.(slashPrefix, content.slice(slashPrefix.length).trim());
      setAttachments([]);
      if (mentions.length > 0) setMentions([]);
      return;
    }
    onSend(content, attachments);
    setAttachments([]);
    if (mentions.length > 0) setMentions([]);
  };

  const { visibleMessages, visibleIndexToReal } = useMemo(() => {
    if (viewMode !== "summary") {
      return {
        visibleMessages: messages,
        visibleIndexToReal: messages.map((_, i) => i),
      };
    }
    const visible: typeof messages = [];
    const mapping: number[] = [];
    messages.forEach((m, i) => {
      if (m.role !== "tool") {
        visible.push(m);
        mapping.push(i);
      }
    });
    return { visibleMessages: visible, visibleIndexToReal: mapping };
  }, [messages, viewMode]);

  const hasMessages = visibleMessages.length > 0 || !!streamingContent;

  const hasStreamed = !!streamingContent;
  const showThinking = agentBusy && !hasStreamed && messages.length > 0;

  return (
    <div className={styles.container}>
      <div className={styles.messagesWrap}>
        {viewMode !== "normal" && (
          <span className={styles.viewModeBadge}>{viewMode === "verbose" ? "详细" : "摘要"}</span>
        )}
        {/* Messages */}
        <main className={styles.messages} aria-live="polite" aria-relevant="additions" ref={messagesScrollRef}>
          {contextUsage > 0.8 && (
            <div className={styles.compactBar}>
              <span>上下文接近满（{Math.round(contextUsage * 100)}%），建议压缩以释放空间</span>
              <button type="button" className={styles.compactBtn} onClick={handleCompact}>
                压缩 /compact
              </button>
            </div>
          )}
          {visibleMessages.map((msg, idx) => (
            <MessageBubble
              key={msg.id ?? idx}
              message={msg}
              index={visibleIndexToReal[idx]}
              allMessages={visibleMessages}
              isLastUser={idx === visibleMessages.length - 1 && msg.role === "user"}
              agentBusy={agentBusy}
              onSend={onSend}
              onEdit={onEdit}
              onRegenerate={onRegenerate}
              onDelete={onDelete}
              viewMode={viewMode}
            />
          ))}
        {showThinking && (
          <div className={styles.thinking}>
            <span className={styles.thinkingDot} />
            思考中…
          </div>
        )}
        {streamingContent && (
          <MessageBubble
            message={{ role: streamingRole, content: streamingContent, contentType: "markdown" }}
            isStreaming
          />
        )}
        {pendingPlanExecute && !agentBusy && (
          <div className={styles.planApprove} data-testid="plan-approve-banner" role="dialog" aria-label="计划审批">
            <div className={styles.planApproveTitle}>📋 计划已生成</div>
            <div className={styles.planApproveHint}>
              当前为 Plan 模式（只读）。批准后切换到 Agent 模式执行该计划。
            </div>
            <div className={styles.planApproveActions}>
              <button
                type="button"
                className={styles.planApproveBtn}
                data-testid="plan-approve-btn"
                onClick={onPlanApprove}
              >
                批准并执行
              </button>
            </div>
          </div>
        )}
        {!hasMessages && (
          <div className={styles.emptyState}>
            <div className={styles.emptyHint}>
              <kbd>⌘N</kbd> 新建会话 · <kbd>⌘B</kbd> 收起侧栏 · <kbd>Ctrl+Tab</kbd> 切换会话
            </div>
            <div className={styles.emptyTitle}>我们该做什么？</div>
            <div className={styles.quickActions} data-testid="quick-actions">
              {QUICK_ACTIONS.map((qa) => (
                <button
                  key={qa.label}
                  type="button"
                  className={styles.quickAction}
                  onClick={() => { setInput(qa.prompt); textareaRef.current?.focus(); }}
                  data-testid={`quick-action-${qa.label}`}
                >
                  <span className={styles.quickActionIcon}>{qa.icon}</span>
                  <span className={styles.quickActionLabel}>{qa.label}</span>
                  <span className={styles.quickActionDesc}>{qa.desc}</span>
                </button>
              ))}
            </div>
            {(recentSessions && recentSessions.length > 0) && (
              <div className={styles.recentBlock} data-testid="recent-sessions">
                <div className={styles.recentTitle}>最近会话</div>
                {recentSessions.slice(0, 5).map((s) => (
                  <button
                    key={s.id}
                    type="button"
                    className={styles.recentItem}
                    onClick={() => onRecentSessionSelect?.(s.id)}
                    data-testid={`recent-session-${s.id}`}
                  >
                    <span className={styles.recentName}>{s.name || "未命名会话"}</span>
                    <span className={styles.recentMeta}>{s.mode} · {formatRelative(s.updated_at)}</span>
                  </button>
                ))}
              </div>
            )}
            {contextUsage > 0 && (
              <div className={styles.emptyStats}>
                <span>上下文占用 {Math.round(contextUsage * 100)}%</span>
              </div>
            )}
          </div>
        )}
        <div ref={messagesEndRef} />
        </main>
        {showScrollDown && (
          <button
            type="button"
            className={styles.scrollDownBtn}
            onClick={() => scrollToBottom(true)}
            title="滚动到底部"
            aria-label="滚动到底部"
            data-testid="scroll-to-bottom"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.6">
              <path d="M7 2v10M3 8l4 4 4-4" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
        )}
      </div>

       {/* Input */}
       <form
         onSubmit={handleSubmit}
         className={`${styles.inputArea} ${dragging ? styles.dragging : ""}`}
         onDragEnter={handleDragEnter}
         onDragOver={(e) => e.preventDefault()}
         onDragLeave={handleDragLeave}
         onDrop={handleDrop}
       >
         <div className={styles.composerCard}>
         <button
           type="button"
           className={styles.attachBtn}
           disabled={agentBusy}
            onClick={() => fileInputRef.current?.click()}
            title="添加附件"
            aria-label="添加附件"
          >
           <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
             <path d="M5 11l5-5.5A1.8 1.8 0 007.5 3L2.5 8A3 3 0 006.5 12l5-5.5A4.2 4.2 0 008 2L3 7" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
         </button>
          <input ref={fileInputRef} type="file" multiple hidden onChange={handleFileChange} />
           <div className={styles.composerColumn}>
            {queuedInputs.length > 0 && (
              <div className={styles.queueBar}>
                <span className={styles.queueDot} />
                已排队 {queuedInputs.length} 条，完成后自动发送
                <button type="button" className={styles.queueCancel} onClick={() => { queueRef.current = []; setQueuedInputs([]); }} title="清空队列" aria-label="清空队列">
                  ✕
                </button>
              </div>
            )}
            {mentions.length > 0 && (
              <div className={styles.mentionsRow}>
                {mentions.map((m) => (
                  <span key={m} className={styles.mentionChip}>
                    <span className={styles.mentionGlyph}>@</span>
                    <span className={styles.attachmentName}>{m}</span>
                    <button
                      type="button"
                      className={styles.attachmentRemove}
                      title="移除"
                      onClick={() => setMentions((prev) => prev.filter((x) => x !== m))}
                    >
                      ✕
                    </button>
                  </span>
                ))}
              </div>
            )}
            {attachments.length > 0 && (
              <div className={styles.attachmentsRow}>
               {attachments.map((att) => (
                 <span key={att.id} className={styles.attachmentChip}>
                   <span className={styles.attachmentName}>{att.name}</span>
                   <span className={styles.attachmentSize}>{formatFileSize(att.size)}</span>
                   <button
                     type="button"
                     className={styles.attachmentRemove}
                     title="移除"
                     onClick={() => setAttachments((prev) => prev.filter((a) => a.id !== att.id))}
                   >
                     ✕
                   </button>
                 </span>
               ))}
             </div>
           )}
           <textarea
             ref={textareaRef}
             value={input}
              onChange={(e) => {
                const val = e.target.value;
                setInput(val);
                const lastWord = val.split(/\s/).pop() || "";
                if (lastWord.startsWith("/")) {
                  setSlashQuery(lastWord);
                  setShowSlash(true);
                } else {
                  setShowSlash(false);
                }
                if (lastWord.startsWith("@")) {
                  setMentionQuery(lastWord.slice(1));
                  setMentionOpen(true);
                } else {
                  setMentionOpen(false);
                }
              }}
             onKeyDown={handleKeyDown}
             placeholder={agentBusy ? "运行中… Enter/Tab 排队后续输入" : "Enter 发送，Shift+Enter 换行，/ 显示命令"}
             rows={1}
             className={styles.textarea}
             style={{ height: "auto", minHeight: "44px" }}
           />
          </div>
          {mentionOpen && (
            <div ref={mentionRef} className={styles.mentionMenu}>
              <div className={styles.mentionHint}>输入 @ 引用项目文件（递归搜索）</div>
              {mentionFiles.length === 0 && <div className={styles.mentionEmpty}>无匹配文件</div>}
              {mentionFiles.map((f, i) => (
                <button
                  key={f}
                  className={`${styles.mentionItem} ${i === mentionHighlight ? styles.mentionItemActive : ""}`}
                  onClick={() => handleMentionSelect(f)}
                >
                  <span className={styles.mentionIcon}>{mentionDirs.includes(f) ? "📁" : "📄"}</span>
                  <span className={styles.mentionName}>{f}</span>
                </button>
              ))}
            </div>
          )}
          {showSlash && (
            <div ref={slashRef} className={styles.slashMenu}>
             {filteredSlash.length === 0 && <div className={styles.slashEmpty}>无匹配命令</div>}
             {filteredSlash.map((c, i) => (
               <button
                 key={c.cmd}
                 className={`${styles.slashItem} ${i === slashHighlight ? styles.slashItemActive : ""}`}
                 onMouseEnter={() => setSlashHighlight(i)}
                 onClick={() => handleSlashSelect(c.cmd)}
               >
                 <span className={styles.slashCmd}>{c.cmd}</span>
                 <span className={styles.slashLabel}>{c.label}</span>
                 <span className={styles.slashHint}>{c.hint}</span>
               </button>
             ))}
           </div>
         )}
{agentBusy && onStop ? (
            <button type="button" className={styles.stopBtn} onClick={onStop} title="停止生成 (Esc)" aria-label="停止生成">
              <svg width="16" height="16" viewBox="0 0 14 14" fill="currentColor">
                <rect x="3" y="3" width="8" height="8" rx="1" />
              </svg>
            </button>
          ) : (
             <button type="submit" disabled={agentBusy || !input.trim()} className={styles.sendBtn} title="发送" aria-label="发送">
              <svg width="18" height="18" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M3 7l5-5 5 5M8 2v10" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
             </button>
          )}
          <div className={styles.composerFooter}>
            <span className={styles.composerLeft}>
              <span className={styles.modelChip} title="当前模型" data-testid="composer-model">
                <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                  <rect x="2" y="5" width="10" height="7" rx="1.5"/>
                  <path d="M4.5 5V3.5a1.5 1.5 0 013 0V5M9.5 5V3.5a1.5 1.5 0 01-3 0V5"/>
                </svg>
                {defaultModel}
              </span>
              <button
                type="button"
                className={`${styles.planToggle} ${activeMode === "Plan" ? styles.planToggleActive : ""}`}
                title={activeMode === "Plan" ? "Plan 模式（只读规划）已开启" : "切换到 Plan 模式"}
                data-testid="composer-plan-toggle"
                onClick={() => {
                  if (activeMode === "Plan") {
                    onModeChange?.("Agent");
                  } else {
                    onModeChange?.("Plan");
                  }
                }}
              >
                <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                  <rect x="1.5" y="3.5" width="11" height="8" rx="1.5"/>
                  <path d="M4 6.5h6M4 8.5h3"/>
                </svg>
                Plan
              </button>
              <span className={styles.contextCapsule} title="当前工作区上下文" data-testid="composer-context">
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M1.5 4.5h11M1.5 7h11M1.5 9.5h7"/>
                </svg>
                项目
                <span className={styles.contextCapsuleArrow}>▾</span>
              </span>
            </span>
            <button
              type="button"
              className={styles.permissionPill}
              title="审批模式"
              data-testid="composer-permission"
            >
              <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                <path d="M7 1.5L12 3v3.5c0 2.8-2 4.8-5 6-3-1.2-5-3.2-5-6V3L7 1.5z"/>
              </svg>
              {permMode === "auto" ? "自动" : permMode === "manual" ? "手动" : "接受"}
            </button>
          </div>
        </div>
       </form>
    </div>
  );
}

function MessageBubble({
  message,
  isStreaming = false,
  index = 0,
  allMessages,
  isLastUser = false,
  agentBusy = false,
  onSend,
  onEdit,
  onRegenerate,
  onDelete,
  viewMode = "normal",
}: {
  message: Message;
  isStreaming?: boolean;
  index?: number;
  allMessages?: Message[];
  isLastUser?: boolean;
  agentBusy?: boolean;
  onSend?: (content: string) => void;
  onEdit?: (id: number, content: string) => void;
  onRegenerate?: (id: number) => void;
  onDelete?: (id: number) => void;
  viewMode?: "verbose" | "normal" | "summary";
}) {
  const [feedback, setFeedback] = useState<"up" | "down" | null>(null);
  const [editing, setEditing] = useState(false);
  const [editVal, setEditVal] = useState(message.content);
  const addNotification = useStore((s) => s.addNotification);
  const { html: rawHtml, codeBlocks } = renderContent(message.content, message.contentType);
  const html = useMemo(() => decorateCodeBlocks(rawHtml, `m${message.id ?? Math.random().toString(36).slice(2, 8)}`), [rawHtml, message.id]);
  const diffStats = useMemo(() => {
    let added = 0;
    let removed = 0;
    const lines = message.content.split("\n");
    for (const line of lines) {
      if (line.startsWith("+")) added++;
      else if (line.startsWith("-")) removed++;
    }
    return added > 0 || removed > 0 ? { added, removed } : null;
  }, [message.content]);
  const roleClass = roleClassMap[message.role] || styles.messageAssistant;
  const avatar = roleAvatarMap[message.role] || ASSISTANT_AVATAR;
  const imageAtts = (message.attachments || []).filter((a) => a.mimeType.startsWith("image/") && a.data);

  const handleCopy = async (content: string) => {
    await copyMessage(content);
    addNotification({ type: "success", message: "已复制", duration: 2000 });
  };

  const handleEditSubmit = () => {
    if (editVal.trim()) {
      if (onEdit && message.id != null) {
        onEdit(message.id, editVal.trim());
      } else if (onSend) {
        onSend(editVal.trim());
      }
      setEditing(false);
    }
  };

  const handleRegenerate = () => {
    if (!onRegenerate) return;
    if (message.id != null) {
      onRegenerate(message.id);
    } else {
      const lastUser = [...(allMessages || [])].reverse().find((m) => m.role === "user");
      if (lastUser && onSend) onSend(lastUser.content);
    }
  };

  if (message.role === "tool") {
    return <ToolCard message={message} avatar={avatar} roleClass={roleClass} defaultExpanded={viewMode === "verbose"} />;
  }

  return (
    <div className={`${styles.message} ${roleClass} ${isStreaming ? styles.streaming : ""}`}>
      <div className={styles.avatar} dangerouslySetInnerHTML={{ __html: avatar }} />
      <div className={styles.bubble}>
        {imageAtts.length > 0 && (
          <div className={styles.imageRow}>
            {imageAtts.map((att, i) => (
              <img
                key={i}
                className={styles.attachmentImage}
                src={`data:${att.mimeType};base64,${att.data}`}
                alt={att.name}
                onClick={() => window.open(`data:${att.mimeType};base64,${att.data}`, "_blank")}
                title={`${att.name}（点击放大）`}
              />
            ))}
          </div>
        )}
        {editing ? (
          <div className={styles.editArea}>
            <textarea
              className={styles.editTextarea}
              value={editVal}
              onChange={(e) => setEditVal(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleEditSubmit(); }
                if (e.key === "Escape") { e.preventDefault(); setEditing(false); setEditVal(message.content); }
              }}
              autoFocus
              rows={3}
            />
            <div className={styles.editActions}>
              <button className={styles.actionIcon} onClick={handleEditSubmit} title="发送">
                <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <path d="M3 7l5-5 5 5M8 2v10" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              </button>
              <button className={styles.actionIcon} onClick={() => { setEditing(false); setEditVal(message.content); }} title="取消">
                <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <path d="M3 3l8 8M11 3l-8 8" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              </button>
            </div>
          </div>
        ) : (
        <>
        <div
          className={styles.content}
          dangerouslySetInnerHTML={{ __html: html }}
          onClick={async (e) => {
            const btn = (e.target as HTMLElement).closest?.("[data-copy-id]");
            const pre = (e.target as HTMLElement).closest?.("pre[data-copy-id]");
            const copyId = btn?.getAttribute("data-copy-id") || pre?.getAttribute("data-copy-id");
            if (!copyId) return;
            if (btn) e.stopPropagation();
            const codeEl = document.querySelector(`pre[data-copy-id="${copyId}"]`);
            if (!codeEl) return;
            await copyMessage((codeEl.textContent || "").trim());
            addNotification({ type: "success", message: "已复制代码块", duration: 2000 });
          }}
        />
        {isStreaming && <span className={styles.streamCaret} aria-hidden="true" />}
        {codeBlocks > 0 && (
          <div className={styles.codeIndicator}>
            {codeBlocks} 代码块
            {diffStats && (
              <span className={styles.diffStat}>
                <span className={styles.add}>+{diffStats.added}</span>
                <span className={styles.remove}>-{diffStats.removed}</span>
              </span>
            )}
          </div>
        )}
        {message.timestamp && <div className={styles.time}>{formatTimestamp(message.timestamp)}</div>}
        {!isStreaming && !editing && (
          <div className={styles.messageActions}>
            <button className={styles.actionIcon} onClick={() => handleCopy(message.content)} title="复制" aria-label="复制">
              <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <rect x="3" y="2" width="9" height="10" rx="1.5" strokeLinecap="round"/>
                <path d="M8 2v4h-2V2h-4v4H4v8h6V6h2v4a2 2 0 002 2h4a2 2 0 002-2V4a2 2 0 00-2-2h-4z" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </button>
            {message.role === "user" && (
              <>
                {message.id != null && (
                <button className={styles.actionIcon} onClick={() => { setEditVal(message.content); setEditing(true); }} title="编辑" aria-label="编辑">
                  <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                    <path d="M10 2l2 2-8 8H4v-2l6-6z" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                </button>
                )}
                {message.id != null && (
                <button className={styles.actionIcon} onClick={() => onDelete?.(message.id as number)} title="删除" aria-label="删除">
                  <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                    <path d="M2 4h10M5 4V2h4v2M4 4l1 8h4l1-8" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                </button>
                )}
              </>
            )}
            {message.role === "assistant" && !agentBusy && !isStreaming && onRegenerate && (
              <button className={styles.actionIcon} onClick={handleRegenerate} title="重新生成" aria-label="重新生成">
                <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <path d="M2 7a5 5 0 015-5 5 5 0 014 2M12 7a5 5 0 01-9 3M12 2v4H8" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              </button>
            )}
          </div>
        )}
        {!isStreaming && message.role === "assistant" && (
          <div className={styles.feedback}>
            <button className={styles.feedbackBtn} onClick={() => setFeedback("up")} title="有帮助" aria-label="有帮助">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M2 10l3-6h4l-1 6h3l-3 6H5l1-6H3z" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </button>
            <button className={styles.feedbackBtn} onClick={() => setFeedback("down")} title="无帮助" aria-label="无帮助">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M2 4l3 6h4l-1-6h3l3 6H9l1-6H5z" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </button>
          </div>
        )}
        </>
        )}
      </div>
    </div>
  );
}

async function copyMessage(content: string) {
  try {
    await navigator.clipboard.writeText(content);
  } catch {}
}

function ToolCard({ message, avatar, roleClass, defaultExpanded = false }: { message: Message; avatar: string; roleClass: string; defaultExpanded?: boolean }) {
  const [expanded, setExpanded] = useState(defaultExpanded);
  const m = /^\*\*([^*]+)\*\*(.*)$/s.exec(message.content);
  const toolName = m ? m[1] : "工具调用";
  const body = m ? m[2] : message.content;
  const failed = /\*\*.*\*\*\s*\(失败\)/.test(message.content);
  const { html: bodyHtml } = renderContent(body);

  return (
    <div className={`${styles.message} ${roleClass}`}>
      <div className={styles.avatar} dangerouslySetInnerHTML={{ __html: avatar }} />
      <div className={styles.toolCard}>
        <button className={styles.toolHeader} onClick={() => setExpanded((v) => !v)}>
          <span className={`${styles.toolStatus} ${failed ? styles.toolStatusFail : styles.toolStatusOk}`} title={failed ? "失败" : "成功"} />
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
