import React, { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { marked } from "marked";
import DOMPurify from "dompurify";
import hljs from "highlight.js";
import "highlight.js/styles/github.css";
import * as api from "../lib/api";
import PetBar from "../components/PetBar";
import type { Attachment } from "../types";
import styles from "./ChatPage.module.css";

interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  status: "pending" | "streaming" | "complete" | "error";
  errorMessage?: string;
  errorType?: "auth" | "rate_limit" | "overloaded" | "network" | "unknown";
  attachments?: Attachment[];
  createdAt: number;
}

interface Conversation {
  id: string;
  title: string;
  messages: ChatMessage[];
  model: string;
  createdAt: number;
}

interface ArtifactItem {
  id: string;
  lang: string;
  code: string;
  messageId: string;
}

const SUGGESTIONS = [
  { icon: "📝", title: "Write a blog post", text: "Draft a tech blog about Rust async patterns" },
  { icon: "💻", title: "Explain code", text: "Explain how monads work in functional programming" },
  { icon: "🔍", title: "Debug my code", text: "Help me find a bug in my React useEffect cleanup" },
  { icon: "🧠", title: "Brainstorm ideas", text: "Brainstorm 5 SaaS ideas for indie developers" },
];

const MODELS = [
  "claude-sonnet-4-20250514",
  "claude-sonnet-4-20250514-thinking",
  "claude-3-5-haiku-20241022",
  "claude-3-opus-20240229",
];

const ACCEPTED_TYPES = [
  ".txt", ".md", ".pdf", ".json", ".yaml", ".yml", ".toml", ".csv",
  ".rs", ".py", ".js", ".ts", ".tsx", ".jsx", ".html", ".css", ".scss", ".sh", ".sql",
  ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".ico",
];

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / 1048576).toFixed(1)}MB`;
}

function fileToAttachment(file: File): Promise<Attachment> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve({
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      name: file.name, size: file.size, mimeType: file.type, data: reader.result as string,
    });
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

function parseErrorType(msg: string): ChatMessage["errorType"] {
  if (msg.includes("401") || msg.includes("api key") || msg.includes("unauthorized")) return "auth";
  if (msg.includes("429") || msg.includes("rate limit")) return "rate_limit";
  if (msg.includes("529") || msg.includes("overloaded")) return "overloaded";
  if (msg.includes("network") || msg.includes("timeout") || msg.includes("dns")) return "network";
  return "unknown";
}

const ERROR_UI: Record<string, { title: string; desc: string }> = {
  auth: { title: "Invalid API Key", desc: "Go to Settings to update your API key." },
  rate_limit: { title: "Rate Limited", desc: "Too many requests. Please wait a moment and try again." },
  overloaded: { title: "Service Overloaded", desc: "Anthropic's API is temporarily overloaded. Try again shortly." },
  network: { title: "Network Error", desc: "Check your internet connection and try again." },
  unknown: { title: "Error", desc: "An unexpected error occurred." },
};

const STORAGE_KEY = "novachat-conversations";
const INDEX_KEY = "novachat-active-idx";

function loadConversations(): Conversation[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Conversation[];
      if (Array.isArray(parsed) && parsed.length > 0) return parsed;
    }
  } catch { /* corrupted or missing */ }
  return [];
}

function saveConversations(convs: Conversation[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(convs));
  } catch { /* storage full — silently ignore */ }
}

let codeBlockId = 0;

const COLLAPSE_LINE_LIMIT = 16;
const COLLAPSE_CHAR_LIMIT = 2000;

function shouldCollapse(code: string): boolean {
  const lines = code.split("\n").length;
  return lines > COLLAPSE_LINE_LIMIT || code.length > COLLAPSE_CHAR_LIMIT;
}

function renderMarkdown(content: string, onOpenArtifact?: (lang: string, code: string) => void): string {
  try {
    const tokens = marked.lexer(content);
    const html = marked.parser(tokens, { breaks: true });
    const withCode = html.replace(
      /<pre><code class="language-(\w+)">([\s\S]*?)<\/code><\/pre>/g,
      (_, lang, code) => {
        const cbId = `cb-${++codeBlockId}`;
        const decoded = code.replace(/&amp;/g, "&").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"');
        let highlighted: string;
        try {
          highlighted = hljs.highlight(decoded, { language: lang, ignoreIllegals: true }).value;
        } catch {
          highlighted = decoded.replace(/</g, "&lt;").replace(/>/g, "&gt;");
        }
        const openAttr = onOpenArtifact
          ? `<button class="code-preview" onclick="window.__openArtifact('${lang.replace(/'/g, "\\'")}','${cbId}')">Preview</button>`
          : "";
        const collapse = shouldCollapse(decoded);
        const collapsePreClass = collapse ? ' class="code-collapsed"' : "";
        const toggleBtn = collapse
          ? `<button class="code-toggle" onclick="window.__toggleCodeBlock('${cbId}')">Show more</button>`
          : "";
        return `<div class="code-block${collapse ? " is-collapsible" : ""}"><div class="code-header"><span class="code-lang">${lang}</span><div class="code-actions">${openAttr}<button class="code-copy" onclick="navigator.clipboard.writeText(document.getElementById('${cbId}').textContent)">Copy</button></div></div><pre${collapsePreClass}><code id="${cbId}" class="hljs language-${lang}">${highlighted}</code></pre>${toggleBtn}</div>`;
      }
    );
    return DOMPurify.sanitize(withCode, {
      ALLOWED_TAGS: ["p","br","strong","em","code","pre","ul","ol","li","a","h1","h2","h3","h4","h5","h6","blockquote","hr","table","thead","tbody","tr","th","td","span","div","img","svg","path","circle","rect","line","text","button","input"],
      ALLOWED_ATTR: ["href","target","rel","src","alt","class","width","height","viewBox","fill","stroke","d","xmlns","onclick","style","id"],
      ALLOWED_URI_REGEXP: /^(?:(?:https?|mailto|data):|[^a-z]|[a-z+.-]+(?:[^a-z+.-:]|$))/i,
    });
  } catch {
    const e = content.replace(/&/g, "&amp;").replace(/</g, "&lt;");
    return `<pre style="white-space:pre-wrap">${e}</pre>`;
  }
}

function AttachmentChip({ attachment, onRemove }: { attachment: Attachment; onRemove: (id: string) => void }) {
  const isImage = attachment.mimeType.startsWith("image/");
  return (
    <div className={styles.chip}>
      {isImage && <img src={attachment.data} alt="" className={styles.chipThumb} />}
      <div className={styles.chipInfo}>
        <span className={styles.chipName}>{attachment.name}</span>
        <span className={styles.chipSize}>{formatSize(attachment.size)}</span>
      </div>
      <button className={styles.chipRemove} onClick={() => onRemove(attachment.id)}>
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none"><path d="M2 2l6 6M8 2l-6 6" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/></svg>
      </button>
    </div>
  );
}

function ChatPage() {
  const navigate = useNavigate();
  const [convs, setConvs] = useState<Conversation[]>(() => {
    const saved = loadConversations();
    if (saved.length > 0) return saved;
    return [{
      id: `conv-${Date.now()}`, title: "New conversation", messages: [], model: MODELS[0], createdAt: Date.now(),
    }];
  });
  const [activeIdx, setActiveIdx] = useState(() => {
    try {
      const saved = localStorage.getItem(INDEX_KEY);
      if (saved) {
        const idx = parseInt(saved, 10);
        const convsState = loadConversations();
        if (!isNaN(idx) && idx >= 0 && idx < convsState.length) return idx;
      }
    } catch { /* ignore */ }
    return 0;
  });
  const [streamingId, setStreamingId] = useState<string | null>(null);
  const [hasKey, setHasKey] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showShortcuts, setShowShortcuts] = useState(false);
  const [showSidebar, setShowSidebar] = useState(true);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [apiKeyStatus, setApiKeyStatus] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [editingTitle, setEditingTitle] = useState(false);
  const [titleDraft, setTitleDraft] = useState("");
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  const [dragOver, setDragOver] = useState(false);
  const [scrollAuto, setScrollAuto] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [inputText, setInputText] = useState("");
  const [showMenu, setShowMenu] = useState(false);
  const [artifact, setArtifact] = useState<ArtifactItem | null>(null);
  const [smallScreen, setSmallScreen] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const titleInputRef = useRef<HTMLInputElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const messagesAreaRef = useRef<HTMLDivElement>(null);
  const streamBufRef = useRef<Map<string, string>>(new Map());
  const streamingConvIdRef = useRef<string | null>(null);
  const streamingIdRef = useRef<string | null>(null);
  const streamGenRef = useRef(0);
  const rAFRef = useRef<number>(0);
  const menuRef = useRef<HTMLDivElement>(null);

  const conv = convs[activeIdx];
  const messages = conv?.messages ?? [];
  const filteredConvs = searchQuery
    ? convs.filter((c) => c.title.toLowerCase().includes(searchQuery.toLowerCase()))
    : convs;

  // Responsive sidebar
  useEffect(() => {
    const mq = window.matchMedia("(max-width: 900px)");
    setSmallScreen(mq.matches);
    setShowSidebar(!mq.matches);
    const handler = (e: MediaQueryListEvent) => {
      setSmallScreen(e.matches);
      if (e.matches) setShowSidebar(false);
      else setShowSidebar(true);
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);

  // Global artifact opener + code block toggler
  useEffect(() => {
    (window as any).__openArtifact = (lang: string, codeId: string) => {
      const el = document.getElementById(codeId);
      if (el) setArtifact({ id: `art-${Date.now()}`, lang, code: el.textContent || "", messageId: "" });
    };
    (window as any).__toggleCodeBlock = (codeId: string) => {
      const el = document.getElementById(codeId);
      if (!el) return;
      const pre = el.closest("pre");
      const block = el.closest(".code-block");
      if (!pre || !block) return;
      const isCollapsed = pre.classList.contains("code-collapsed");
      const btn = block.querySelector(".code-toggle");
      if (isCollapsed) {
        pre.classList.remove("code-collapsed");
        if (btn) btn.textContent = "Show less";
      } else {
        pre.classList.add("code-collapsed");
        if (btn) btn.textContent = "Show more";
      }
    };
    return () => { delete (window as any).__openArtifact; delete (window as any).__toggleCodeBlock; };
  }, []);

  useEffect(() => {
    streamingIdRef.current = streamingId;
  }, [streamingId]);

  useEffect(() => {
    saveConversations(convs);
  }, [convs]);

  useEffect(() => {
    try { localStorage.setItem(INDEX_KEY, String(activeIdx)); } catch { /* ignore */ }
  }, [activeIdx]);

  useEffect(() => {
    api.hasApiKey().then(setHasKey);
  }, []);

  useEffect(() => {
    if (scrollAuto) messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, scrollAuto]);

  useEffect(() => {
    if (editingTitle && titleInputRef.current) { titleInputRef.current.focus(); titleInputRef.current.select(); }
  }, [editingTitle]);

  useEffect(() => {
    if (!inputText && textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
  }, [inputText]);

  // Close menu on outside click
  useEffect(() => {
    if (!showMenu) return;
    const handler = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) setShowMenu(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [showMenu]);

  const flushStreamBuffer = useCallback(() => {
    const buf = streamBufRef.current;
    if (buf.size === 0) { rAFRef.current = 0; return; }
    const convId = streamingConvIdRef.current;
    setConvs((prev) => {
      const convIdx = prev.findIndex((c) => c.id === convId);
      if (convIdx === -1) return prev;
      const next = [...prev];
      const msgs = [...next[convIdx].messages];
      let changed = false;
      for (const [id, delta] of buf) {
        const idx = msgs.findIndex((m) => m.id === id);
        if (idx !== -1) { msgs[idx] = { ...msgs[idx], content: msgs[idx].content + delta }; changed = true; }
      }
      if (changed) next[convIdx] = { ...next[convIdx], messages: msgs };
      return next;
    });
    buf.clear();
    rAFRef.current = requestAnimationFrame(flushStreamBuffer);
  }, []);

  function flushStreamBufferNow() {
    const buf = streamBufRef.current;
    if (buf.size === 0) return;
    const convId = streamingConvIdRef.current;
    setConvs((prev) => {
      const convIdx = prev.findIndex((c) => c.id === convId);
      if (convIdx === -1) return prev;
      const next = [...prev];
      const msgs = [...next[convIdx].messages];
      for (const [id, delta] of buf) {
        const idx = msgs.findIndex((m) => m.id === id);
        if (idx !== -1) msgs[idx] = { ...msgs[idx], content: msgs[idx].content + delta };
      }
      next[convIdx] = { ...next[convIdx], messages: msgs };
      return next;
    });
    buf.clear();
  }

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const meta = e.metaKey || e.ctrlKey;
      if (meta && e.key === "n") { e.preventDefault(); newConversation(); }
      if (meta && e.key === "k") { e.preventDefault(); setShowSidebar((s) => !s); }
      if (e.key === "Escape") { setShowSettings(false); setShowShortcuts(false); setEditingTitle(false); setShowMenu(false); setArtifact(null); }
      if (e.key === "?" && !meta) { e.preventDefault(); setShowShortcuts((s) => !s); }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [convs.length]);

  const handleScroll = useCallback(() => {
    const el = messagesAreaRef.current;
    if (!el) return;
    setScrollAuto(el.scrollHeight - el.scrollTop - el.clientHeight < 80);
  }, []);

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    setScrollAuto(true);
  }, []);

  const newConversation = useCallback(() => {
    const id = `conv-${Date.now()}`;
    setConvs((prev) => {
      setActiveIdx(prev.length);
      return [...prev, { id, title: "New conversation", messages: [], model: MODELS[0], createdAt: Date.now() }];
    });
    setAttachments([]);
    setArtifact(null);
  }, []);

  const switchConversation = useCallback((idx: number) => {
    setActiveIdx(idx);
    if (streamingIdRef.current) { api.stopGeneration(); streamingIdRef.current = null; setStreamingId(null); }
    setAttachments([]);
    setArtifact(null);
    if (smallScreen) setShowSidebar(false);
  }, [smallScreen]);

  const deleteConversation = useCallback((idx: number) => {
    setConvs((prev) => {
      const next = prev.filter((_, i) => i !== idx);
      if (next.length === 0) {
        setActiveIdx(0);
        return [{ id: `conv-${Date.now()}`, title: "New conversation", messages: [], model: MODELS[0], createdAt: Date.now() }];
      }
      setActiveIdx((active) => {
        if (active >= next.length) return next.length - 1;
        if (active > idx) return active - 1;
        return active;
      });
      return next;
    });
  }, []);

  const handleStreaming = useCallback(async (content: string, atch?: Attachment[]) => {
    const convId = conv.id;
    const gen = ++streamGenRef.current;
    streamingConvIdRef.current = convId;
    const userMsg: ChatMessage = {
      id: `msg-${Date.now()}`, role: "user", content, status: "complete",
      attachments: atch, createdAt: Date.now(),
    };
    const assistId = `msg-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    const assistMsg: ChatMessage = {
      id: assistId, role: "assistant", content: "", status: "streaming", createdAt: Date.now(),
    };

    setConvs((prev) => {
      const convIdx = prev.findIndex((c) => c.id === convId);
      if (convIdx === -1) return prev;
      const next = [...prev];
      next[convIdx] = {
        ...next[convIdx],
        messages: [...next[convIdx].messages, userMsg, assistMsg],
      };
      if (next[convIdx].title === "New conversation") {
        next[convIdx].title = content.length > 50 ? content.slice(0, 50) + "..." : content;
      }
      return next;
    });
    setStreamingId(assistId);
    setScrollAuto(true);

    if (!rAFRef.current) rAFRef.current = requestAnimationFrame(flushStreamBuffer);

    const genCheck = () => gen === streamGenRef.current;

    const unlistenChunk = await listen<{ messageId: string; delta: string }>("stream-chunk", (event) => {
      if (!genCheck()) return;
      if (event.payload.messageId === assistId) {
        streamBufRef.current.set(assistId, (streamBufRef.current.get(assistId) || "") + event.payload.delta);
      }
    });

    const unlistenDone = await listen<{ messageId: string; success: boolean }>("stream-done", (event) => {
      if (!genCheck()) return;
      if (event.payload.messageId === assistId) {
        flushStreamBufferNow();
        const cId = streamingConvIdRef.current;
        setConvs((prev) => {
          const convIdx = prev.findIndex((c) => c.id === cId);
          if (convIdx === -1) return prev;
          const next = [...prev];
          const msgs = [...next[convIdx].messages];
          const idx = msgs.findIndex((m) => m.id === assistId);
          if (idx !== -1) msgs[idx] = { ...msgs[idx], status: event.payload.success ? "complete" : "error" };
          next[convIdx] = { ...next[convIdx], messages: msgs };
          return next;
        });
        setStreamingId(null);
      }
    });

    const unlistenError = await listen<{ messageId: string; error: string }>("stream-error", (event) => {
      if (!genCheck()) return;
      if (event.payload.messageId === assistId) {
        flushStreamBufferNow();
        const errType = parseErrorType(event.payload.error);
        const cId = streamingConvIdRef.current;
        setConvs((prev) => {
          const convIdx = prev.findIndex((c) => c.id === cId);
          if (convIdx === -1) return prev;
          const next = [...prev];
          const msgs = [...next[convIdx].messages];
          const idx = msgs.findIndex((m) => m.id === assistId);
          if (idx !== -1) msgs[idx] = { ...msgs[idx], status: "error", errorMessage: event.payload.error, errorType: errType };
          next[convIdx] = { ...next[convIdx], messages: msgs };
          return next;
        });
        setStreamingId(null);
      }
    });

    try {
      await api.sendMessage(convId, content, conv.model);
    } catch (e: unknown) {
      flushStreamBufferNow();
      const errType = parseErrorType(String(e));
      const cId = streamingConvIdRef.current;
      setConvs((prev) => {
        const convIdx = prev.findIndex((c) => c.id === cId);
        if (convIdx === -1) return prev;
        const next = [...prev];
        const msgs = [...next[convIdx].messages];
        const idx = msgs.findIndex((m) => m.id === assistId);
        if (idx !== -1) msgs[idx] = { ...msgs[idx], status: "error", errorMessage: String(e), errorType: errType };
        next[convIdx] = { ...next[convIdx], messages: msgs };
        return next;
      });
      setStreamingId(null);
    } finally {
      unlistenChunk(); unlistenDone(); unlistenError();
    }
  }, [conv, flushStreamBuffer]);

  const handleSubmit = useCallback(async () => {
    if (!inputText.trim() || streamingId) return;
    const atch = attachments.length > 0 ? [...attachments] : undefined;
    setInputText("");
    setAttachments([]);
    await handleStreaming(inputText.trim(), atch);
  }, [inputText, streamingId, attachments, handleStreaming]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleSubmit(); }
  }, [handleSubmit]);

  const handleSaveApiKey = useCallback(async () => {
    if (!apiKeyInput.trim()) return;
    setApiKeyStatus("saving");
    try {
      await api.saveApiKey(apiKeyInput.trim());
      setApiKeyStatus("saved"); setHasKey(true);
      setTimeout(() => { setShowSettings(false); setApiKeyStatus("idle"); }, 1000);
    } catch { setApiKeyStatus("error"); }
  }, [apiKeyInput]);

  const handleFilePick = useCallback(async (files: FileList | null) => {
    if (!files) return;
    const newAt: Attachment[] = [];
    for (let i = 0; i < files.length; i++) newAt.push(await fileToAttachment(files[i]));
    setAttachments((prev) => [...prev, ...newAt]);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    if (e.dataTransfer.files.length > 0) handleFilePick(e.dataTransfer.files);
  }, [handleFilePick]);

  const autoResizeTextarea = useCallback(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 240) + "px";
  }, []);

  const copyMessage = useCallback((content: string) => navigator.clipboard.writeText(content), []);

  const regenerateMessage = useCallback(async () => {
    const convId = conv.id;
    const msgs = conv.messages;
    const lastUser = [...msgs].reverse().find((m) => m.role === "user");
    if (lastUser) {
      setConvs((prev) => {
        const ci = prev.findIndex((c) => c.id === convId);
        if (ci === -1) return prev;
        const n = [...prev];
        n[ci] = { ...n[ci], messages: msgs.slice(0, -1) };
        return n;
      });
      await handleStreaming(lastUser.content);
    }
  }, [conv, handleStreaming]);

  const exportConversation = useCallback(async (format: "markdown" | "json") => {
    const msgs = conv.messages;
    if (msgs.length === 0) return;
    if (format === "markdown") {
      const md = msgs.map((m) => `## ${m.role === "user" ? "User" : "Assistant"}\n\n${m.content}`).join("\n\n---\n\n");
      await api.saveFileDialog(md, `${conv.title.replace(/[^a-zA-Z0-9]/g, "_")}.md`);
    } else {
      const data = JSON.stringify({ title: conv.title, model: conv.model, messages: msgs.map(({ id, role, content, attachments }) => ({ id, role, content, attachments })) }, null, 2);
      await api.saveFileDialog(data, `${conv.title.replace(/[^a-zA-Z0-9]/g, "_")}.json`);
    }
    setShowMenu(false);
  }, [conv]);

  const openArtifact = useCallback((lang: string, code: string) => {
    setArtifact({ id: `art-${Date.now()}`, lang, code, messageId: "" });
  }, []);

  if (!hasKey) {
    return (
      <div className={styles.welcomeContainer}>
        <div className={styles.welcomeCard}>
          <div className={styles.welcomeIcon}>
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
            </svg>
          </div>
          <h1 className={styles.welcomeTitle}>Welcome to NovaChat</h1>
          <p className={styles.welcomeDesc}>Enter your Anthropic API key. Stored securely in your system keychain.</p>
          <div className={styles.apiKeyForm}>
            <input type="password" className={styles.apiKeyInput} placeholder="sk-ant-..." value={apiKeyInput}
              onChange={(e) => setApiKeyInput(e.target.value)} onKeyDown={(e) => e.key === "Enter" && handleSaveApiKey()} />
            <button className={styles.apiKeyButton} onClick={handleSaveApiKey} disabled={apiKeyStatus === "saving"}>
              {apiKeyStatus === "saving" ? "Saving..." : apiKeyStatus === "saved" ? "Saved!" : "Save Key"}
            </button>
          </div>
          {apiKeyStatus === "error" && <p className={styles.errorText}>Failed to save API key.</p>}
          <p className={styles.apiKeyHint}>Stored in macOS Keychain / Windows Credential Manager.</p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.chatContainer}
      onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
      onDragLeave={() => setDragOver(false)}
      onDrop={handleDrop}>
      {/* Sidebar */}
      <div className={`${styles.sidebar} ${showSidebar ? styles.sidebarVisible : styles.sidebarHidden}`}>
        <div className={styles.sidebarHeader}>
          <span className={styles.sidebarTitle}>Conversations</span>
          <button className={styles.sidebarNewBtn} onClick={newConversation} title="New (Cmd+N)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 5v14M5 12h14" /></svg>
          </button>
        </div>
        <div className={styles.sidebarSearch}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className={styles.searchIcon}><circle cx="11" cy="11" r="8" /><path d="M21 21l-4.35-4.35" /></svg>
          <input className={styles.searchInput} placeholder="Search conversations..." value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)} />
        </div>
        <div className={styles.sidebarList}>
          {filteredConvs.map((c) => {
            const realIdx = convs.findIndex((x) => x.id === c.id);
            return (
            <div key={c.id} className={`${styles.sidebarItem} ${realIdx === activeIdx ? styles.sidebarItemActive : ""}`}
              onClick={() => switchConversation(realIdx)}>
              <div className={styles.sidebarItemTitle}>{c.title}</div>
              <div className={styles.sidebarItemMeta}>{c.messages.length} msgs</div>
              {realIdx === activeIdx && (
                <button className={styles.sidebarItemDel} onClick={(e) => { e.stopPropagation(); deleteConversation(realIdx); }}>✕</button>
              )}
            </div>
            );
          })}
        </div>
      </div>

      {/* Sidebar overlay for small screens */}
      {showSidebar && smallScreen && (
        <div className={styles.sidebarOverlay} onClick={() => setShowSidebar(false)} />
      )}

      <div className={styles.mainArea}>
        {/* TopBar */}
        <div className={styles.topBar}>
          <button className={styles.topBarBtn} onClick={() => setShowSidebar((s) => !s)} title={smallScreen ? "Menu" : "Toggle (Cmd+K)"}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M3 12h18M3 6h18M3 18h18" /></svg>
          </button>
          {editingTitle ? (
            <input ref={titleInputRef} className={styles.titleInput} value={titleDraft}
              onChange={(e) => setTitleDraft(e.target.value)}
              onBlur={() => { setConvs((prev) => { const n = [...prev]; n[activeIdx] = { ...n[activeIdx], title: titleDraft }; return n; }); setEditingTitle(false); }}
              onKeyDown={(e) => { if (e.key === "Enter") { setConvs((prev) => { const n = [...prev]; n[activeIdx] = { ...n[activeIdx], title: titleDraft }; return n; }); setEditingTitle(false); } }} />
          ) : (
            <span className={styles.topBarTitle} onClick={() => { setTitleDraft(conv.title); setEditingTitle(true); }}>{conv.title}</span>
          )}
          <div className={styles.topBarActions}>
            <select className={styles.modelSelect} value={conv.model}
              onChange={(e) => setConvs((prev) => { const n = [...prev]; n[activeIdx] = { ...n[activeIdx], model: e.target.value }; return n; })}>
              {MODELS.map((m) => <option key={m} value={m}>{m.replace("claude-", "").replace(/-/g, " ")}</option>)}
            </select>
            <div className={styles.menuWrapper} ref={menuRef}>
              <button className={styles.topBarBtn} onClick={() => setShowMenu((s) => !s)} title="More">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="5" r="1" /><circle cx="12" cy="12" r="1" /><circle cx="12" cy="19" r="1" /></svg>
              </button>
              {showMenu && (
                <div className={styles.dropdown}>
                  <button className={styles.dropdownItem} onClick={() => exportConversation("markdown")} disabled={conv.messages.length === 0}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" /></svg>
                    Export as Markdown
                  </button>
                  <button className={styles.dropdownItem} onClick={() => exportConversation("json")} disabled={conv.messages.length === 0}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" /></svg>
                    Export as JSON
                  </button>
                  <button className={styles.dropdownItem} onClick={() => { setShowMenu(false); setShowSettings(true); }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3" /><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" /></svg>
                    Settings
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Messages */}
        <div className={styles.messagesArea} ref={messagesAreaRef} onScroll={handleScroll}>
          {messages.length === 0 ? (
            <div className={styles.emptyState}>
              <div className={styles.emptyIcon}>
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" /></svg>
              </div>
              <h2 className={styles.emptyTitle}>What can I help with?</h2>
              <p className={styles.emptyDesc}>Send a message or choose a suggestion below.</p>
              <div className={styles.suggestionGrid}>
                {SUGGESTIONS.map((s) => (
                  <button key={s.title} className={styles.suggestionCard} onClick={() => { setInputText(s.text); textareaRef.current?.focus(); }}>
                    <span className={styles.suggestionIcon}>{s.icon}</span>
                    <span className={styles.suggestionTitle}>{s.title}</span>
                    <span className={styles.suggestionText}>{s.text}</span>
                  </button>
                ))}
              </div>
            </div>
          ) : (
            messages.map((msg) => (
              <div key={msg.id} className={`${styles.messageRow} ${msg.role === "user" ? styles.userRow : styles.assistantRow}`}
                onMouseEnter={(e) => { const el = e.currentTarget.querySelector(`.${styles.msgActions}`); if (el) (el as HTMLElement).style.opacity = "1"; }}
                onMouseLeave={(e) => { const el = e.currentTarget.querySelector(`.${styles.msgActions}`); if (el) (el as HTMLElement).style.opacity = "0"; }}>
                {msg.role === "assistant" && (
                  <div className={styles.avatarIcon}>
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" /></svg>
                  </div>
                )}
                <div className={styles.bubbleWrap}>
                  <div className={`${styles.bubble} ${msg.role === "user" ? styles.userBubble : styles.assistantBubble}`}>
                    {msg.attachments && msg.attachments.length > 0 && (
                      <div className={styles.msgAttachments}>
                        {msg.attachments.map((a) => (
                          a.mimeType.startsWith("image/")
                            ? <img key={a.id} src={a.data} alt="" className={styles.msgImage} />
                            : <div key={a.id} className={styles.msgFile}><span className={styles.msgFileName}>{a.name}</span><span className={styles.msgFileSize}>{formatSize(a.size)}</span></div>
                        ))}
                      </div>
                    )}
                    {msg.status === "streaming" ? (
                      <div className={styles.mdContent} dangerouslySetInnerHTML={{
                        __html: renderMarkdown(msg.content, openArtifact) + '<span class="streaming-cursor">▊</span>'
                      }} />
                    ) : msg.status === "error" ? (
                      <div className={styles.errorBubble}>
                        <div className={styles.errorTitle}>{ERROR_UI[msg.errorType || "unknown"].title}</div>
                        <p className={styles.errorDesc}>{msg.errorMessage || ERROR_UI[msg.errorType || "unknown"].desc}</p>
                        <button className={styles.retryBtn} onClick={() => {
                          setConvs((prev) => { const n = [...prev]; n[activeIdx] = { ...n[activeIdx], messages: prev[activeIdx].messages.slice(0, -1) }; return n; });
                          handleStreaming(msg.content, msg.attachments);
                        }}>Retry</button>
                        {msg.errorType === "auth" && (
                          <button className={styles.retryBtn} style={{ marginLeft: 8 }} onClick={() => setShowSettings(true)}>Settings</button>
                        )}
                      </div>
                    ) : (
                      <div className={styles.mdContent} dangerouslySetInnerHTML={{ __html: renderMarkdown(msg.content, openArtifact) }} />
                    )}
                  </div>
                  {msg.status === "complete" && (
                    <div className={styles.msgActions} style={{ opacity: 0 }}>
                      <button className={styles.msgActionBtn} onClick={() => copyMessage(msg.content)} title="Copy">
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="9" y="9" width="13" height="13" rx="2" /><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" /></svg>
                      </button>
                      {msg.role === "user" && (
                        <button className={styles.msgActionBtn} onClick={() => { setInputText(msg.content); textareaRef.current?.focus(); }} title="Edit">
                          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" /><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" /></svg>
                        </button>
                      )}
                      {msg.role === "assistant" && (
                        <button className={styles.msgActionBtn} onClick={regenerateMessage} title="Regenerate">
                          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 2v6h-6M3 12a9 9 0 0115.364-6.364L21 8M3 22v-6h6M21 12a9 9 0 01-15.364 6.364L3 16" /></svg>
                        </button>
                      )}
                    </div>
                  )}
                </div>
              </div>
            ))
          )}
          {!scrollAuto && messages.length > 0 && (
            <button className={styles.scrollToBottom} onClick={scrollToBottom}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 5v14M19 12l-7 7-7-7" /></svg>
              <span>Back to bottom</span>
            </button>
          )}
          <div ref={messagesEndRef} />
        </div>

        {/* Input */}
        <div className={styles.inputArea}>
          {attachments.length > 0 && (
            <div className={styles.chipRow}>
              {attachments.map((a) => <AttachmentChip key={a.id} attachment={a} onRemove={(id) => setAttachments((prev) => prev.filter((x) => x.id !== id))} />)}
            </div>
          )}
          <div className={styles.inputWrapper}>
            <button className={styles.attachBtn} onClick={() => fileInputRef.current?.click()} title="Attach file">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21.44 11.05l-9.19 9.19a6 6 0 01-8.49-8.49l9.19-9.19a4 4 0 015.66 5.66l-9.2 9.19a2 2 0 01-2.83-2.83l8.49-8.48" /></svg>
            </button>
            <input ref={fileInputRef} type="file" multiple accept={ACCEPTED_TYPES.join(",")} style={{ display: "none" }}
              onChange={(e) => handleFilePick(e.target.files)} />
            <textarea ref={textareaRef} className={styles.textarea} placeholder="Send a message..."
              value={inputText} onChange={(e) => setInputText(e.target.value)}
              onKeyDown={handleKeyDown} onInput={autoResizeTextarea}
              rows={1} disabled={!!streamingId} />
            <button className={styles.sendBtn} onClick={streamingId
              ? async () => { await api.stopGeneration(); setStreamingId(null); }
              : handleSubmit} disabled={!streamingId && !inputText.trim()} title={streamingId ? "Stop" : "Send"}>
              {streamingId ? (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="2" /></svg>
              ) : (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z" /></svg>
              )}
            </button>
          </div>
        </div>

        <PetBar />
      </div>

      {/* Artifact Panel */}
      {artifact && (
        <div className={styles.artifactPanel}>
          <div className={styles.artifactHeader}>
            <span className={styles.artifactLang}>{artifact.lang}</span>
            <div className={styles.artifactActions}>
              <button className={styles.artifactActionBtn} onClick={() => navigator.clipboard.writeText(artifact.code)} title="Copy code">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="9" y="9" width="13" height="13" rx="2" /><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" /></svg>
              </button>
              <button className={styles.artifactActionBtn} onClick={() => setArtifact(null)} title="Close">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 6L6 18M6 6l12 12" /></svg>
              </button>
            </div>
          </div>
          <div className={styles.artifactContent}>
            <pre><code className="hljs" dangerouslySetInnerHTML={{
              __html: (() => {
                try { return hljs.highlight(artifact.code, { language: artifact.lang, ignoreIllegals: true }).value; }
                catch { return artifact.code.replace(/</g, "&lt;").replace(/>/g, "&gt;"); }
              })()
            }} /></pre>
          </div>
        </div>
      )}

      {/* Drag overlay */}
      {dragOver && <div className={styles.dragOverlay}><div className={styles.dragOverlayText}>Drop files here</div></div>}

      {/* Shortcuts modal */}
      {showShortcuts && (
        <div className={styles.modalOverlay} onClick={() => setShowShortcuts(false)}>
          <div className={`${styles.modal} ${styles.shortcutsModal}`} onClick={(e) => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <h2>Keyboard Shortcuts</h2>
              <button className={styles.modalClose} onClick={() => setShowShortcuts(false)}>✕</button>
            </div>
            <div className={styles.modalBody}>
              {[
                { keys: "Cmd + N", action: "New conversation" },
                { keys: "Cmd + K", action: "Toggle sidebar" },
                { keys: "?", action: "Show shortcuts" },
                { keys: "Enter", action: "Send message" },
                { keys: "Shift + Enter", action: "New line" },
                { keys: "Esc", action: "Close modals / Cancel edit" },
              ].map((s) => (
                <div key={s.keys} className={styles.shortcutRow}>
                  <kbd className={styles.shortcutKeys}>{s.keys}</kbd>
                  <span className={styles.shortcutAction}>{s.action}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Settings modal */}
      {showSettings && (
        <div className={styles.modalOverlay} onClick={() => setShowSettings(false)}>
          <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
            <div className={styles.modalHeader}><h2>Settings</h2><button className={styles.modalClose} onClick={() => setShowSettings(false)}>✕</button></div>
            <div className={styles.modalBody}>
              <label className={styles.modalLabel}>Anthropic API Key</label>
              <div className={styles.apiKeyForm}>
                <input type="password" className={styles.apiKeyInput} placeholder="sk-ant-..." value={apiKeyInput}
                  onChange={(e) => setApiKeyInput(e.target.value)} />
                <button className={styles.apiKeyButton} onClick={handleSaveApiKey}>{apiKeyStatus === "saving" ? "Saving..." : apiKeyStatus === "saved" ? "Saved!" : "Update"}</button>
              </div>
              <button className={styles.deleteKeyBtn} onClick={async () => { await api.deleteApiKey(); setHasKey(false); setShowSettings(false); }}>Remove API Key</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default ChatPage;
