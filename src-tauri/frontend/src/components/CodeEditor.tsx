import React, { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface CodeEditorProps {
  filePath: string;
  initialContent: string;
  language?: string;
  onSave?: (path: string, content: string) => void;
  onClose?: () => void;
}

interface Token {
  text: string;
  cssClass?: string;
}

const KEYWORDS: Record<string, RegExp> = {
  rust: /\b(?:fn|let|mut|const|pub|struct|enum|impl|match|if|else|for|while|loop|return|true|false|None|Some|Ok|Err|use|mod|as|where|trait|type|async|await|unsafe|ref|move|dyn|self|Self|super|crate|extern|static|break|continue|in|import|export|default|try|catch|panic|assert|print!|println!|format!|vec!|macro_rules!)\b/g,
  typescript: /\b(?:const|let|var|function|return|if|else|for|while|do|switch|case|break|continue|class|interface|type|enum|import|export|from|async|await|try|catch|throw|new|this|super|extends|implements|abstract|static|readonly|public|private|protected|keyof|typeof|instanceof|never|any|unknown|void|boolean|string|number|bigint|symbol|true|false|null|undefined|Record|Pick|Omit|Partial|Required|Readonly|NonNullable|Required|Promise|Array|Map|Set|WeakMap|WeakSet|Error)\b/g,
  javascript: /\b(?:const|let|var|function|return|if|else|for|while|do|switch|case|break|continue|class|import|export|from|async|await|try|catch|throw|new|this|super|typeof|instanceof|true|false|null|undefined|NaN|document|window|console|require|module|process|global|setTimeout|setInterval|clearTimeout|clearInterval|Promise|Array|Map|Set|Error|Symbol|BigInt|Infinity)\b/g,
  python: /\b(?:def|class|import|from|return|if|elif|else|for|while|try|except|finally|with|as|pass|break|continue|raise|yield|lambda|async|await|True|False|None|self|super|in|not|and|or|is|del|global|nonlocal|print|len|range|map|filter|zip|enumerate|type|isinstance|hasattr|open|list|dict|set|str|int|float|bool|tuple|object|Exception|ValueError|TypeError|KeyError|IndexError|AttributeError|ImportError)\b/g,
  html: /<\/?[\w\s="/.':;#-\/?]+>|&[\w#]+;|<!--[\s\S]*?-->/g,
  css: /\b(?:color|background|border|margin|padding|display|flex|grid|position|absolute|relative|fixed|sticky|top|left|right|bottom|width|height|min-width|max-width|min-height|max-height|font-size|font-weight|font-family|text-align|overflow|z-index|opacity|transition|transform|animation|box-shadow|text-shadow|linear-gradient|radial-gradient|calc|var|rgb|rgba|hsl|hsla|hwb|lab|lch|oklab|oklch|clamp|min|max|env|minmax|repeat|fit-content|auto-fit|auto-fill)\b|#[0-9a-fA-F]{3,8}|\.-?[\w-]+|#-?[\w-]+|\.[\w-]+(?=\s*\{)/g,
  json: /"(?:[^"\\]|\\.)*"|\b(?:true|false|null)\b|-?\d+\.?\d*(?:[eE][+-]?\d+)?/g,
  markdown: /#{1,6}\s|\*\*[^*]+\*\*|__[^_]+__|`[^`]+`|\[([^\]]+)\]\(([^)]+)\)|!\[([^\]]*)\]\(([^)]+)\)|-{3,}|={3,}|>+\s|(?:^|\s)[-*+]\s|\d+\.\s/g,
  yaml: /#.*|:\s|\b(?:true|false|yes|no|on|off|null)\b/gi,
  toml: /#.*|\[.*\]|:|=|\b(?:true|false)\b/gi,
  bash: /\b(?:if|then|else|elif|fi|for|while|do|done|case|esac|function|return|local|export|source|echo|exit|cd|ls|rm|mv|cp|mkdir|chmod|chown|grep|sed|awk|find|xargs|cat|less|more|head|tail|sort|uniq|wc|cut|tr|tee|read|test|eval|exec|trap|kill|ps|curl|wget|git|npm|yarn|pnpm|npx|docker|docker-compose|make|cmake|cargo|rustc|python|node|deno|bun)\b/g,
  go: /\b(?:func|return|if|else|for|range|switch|case|break|continue|default|defer|go|select|struct|interface|type|map|chan|var|const|package|import|true|false|nil|iota|new|make|append|len|cap|copy|delete|close|panic|recover|error|string|int|int8|int16|int32|int64|uint|uint8|uint16|uint32|uint64|float32|float64|complex64|complex128|byte|rune|bool|uintptr)\b/g,
};

const STRINGS = /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`)/g;
const COMMENTS = /(\/\/.*|\/\*[\s\S]*?\*\/|#.*|<!--[\s\S]*?-->)/g;
const NUMBERS = /\b(\d+\.?\d*(?:[eE][+-]?\d+)?|0[xX][0-9a-fA-F]+|0[bB][01]+|0[oO][0-7]+)\b/g;
const OPERATORS = /([+\-*/%=<>!&|^~?:]+)/g;

function tokenizeLine(line: string, language: string): Token[] {
  if (!language || !KEYWORDS[language]) return [{ text: line }];

  const tokens: Token[] = [];
  let lastIndex = 0;
  const kwRe = KEYWORDS[language];

  const combined = new RegExp(
    `(${STRINGS.source})|(${COMMENTS.source})|(${kwRe.source})|(${NUMBERS.source})`,
    "g"
  );

  let match: RegExpExecArray | null;
  while ((match = combined.exec(line)) !== null) {
    if (lastIndex < match.index) {
      const between = line.slice(lastIndex, match.index);
      tokenizeOperators(between, tokens);
    }
    if (match[1]) {
      tokens.push({ text: match[1], cssClass: "ce-string" });
    } else if (match[2]) {
      tokens.push({ text: match[2], cssClass: "ce-comment" });
    } else if (match[3]) {
      tokens.push({ text: match[3], cssClass: "ce-keyword" });
    } else if (match[4]) {
      tokens.push({ text: match[4], cssClass: "ce-number" });
    }
    lastIndex = combined.lastIndex;
  }

  if (lastIndex < line.length) {
    tokenizeOperators(line.slice(lastIndex), tokens);
  }

  return tokens;
}

function tokenizeOperators(text: string, tokens: Token[]): void {
  let idx = 0;
  let m: RegExpExecArray | null;
  const opRe = new RegExp(OPERATORS.source, "g");
  while ((m = opRe.exec(text)) !== null) {
    if (idx < m.index) {
      tokens.push({ text: text.slice(idx, m.index) });
    }
    tokens.push({ text: m[1], cssClass: "ce-operator" });
    idx = opRe.lastIndex;
  }
  if (idx < text.length) {
    tokens.push({ text: text.slice(idx) });
  }
}

function renderTokens(tokens: Token[], findQuery?: string, activeIdx?: number): React.ReactNode[] {
  if (!findQuery) {
    return tokens.map((t, i) =>
      t.cssClass
        ? <span key={i} className={t.cssClass}>{t.text}</span>
        : <span key={i}>{t.text}</span>
    );
  }

  const result: React.ReactNode[] = [];
  let globalKey = 0;

  for (const token of tokens) {
    if (!findQuery || token.cssClass === "ce-comment" || token.cssClass === "ce-string") {
      result.push(
        token.cssClass
          ? <span key={globalKey++} className={token.cssClass}>{token.text}</span>
          : <span key={globalKey++}>{token.text}</span>
      );
      continue;
    }

    const lower = token.text.toLowerCase();
    const queryLower = findQuery.toLowerCase();
    let start = 0;
    let idx: number;
    while ((idx = lower.indexOf(queryLower, start)) !== -1) {
      if (idx > start) {
        const preText = token.text.slice(start, idx);
        result.push(<span key={globalKey++}>{preText}</span>);
      }
      const matchText = token.text.slice(idx, idx + findQuery.length);
      const isActive = activeIdx === globalKey;
      result.push(
        <mark key={globalKey++} className={`ce-find-mark${isActive ? " ce-find-active" : ""}`}>
          {matchText}
        </mark>
      );
      start = idx + findQuery.length;
    }
    if (start < token.text.length) {
      result.push(<span key={globalKey++}>{token.text.slice(start)}</span>);
    }
  }

  return result;
}

function detectIndent(text: string): string {
  const lines = text.split("\n");
  const indents = lines
    .filter((l) => l.trim().length > 0)
    .map((l) => l.match(/^(\s+)/)?.[1])
    .filter(Boolean) as string[];
  if (indents.length === 0) return "  ";
  const tabCount = indents.filter((i) => i.startsWith("\t")).length;
  const spaceCount = indents.filter((i) => i.startsWith(" ")).length;
  return tabCount > spaceCount ? "\t" : "  ";
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function formatFilePath(path: string): { dir: string; name: string } {
  const parts = path.split("/");
  const name = parts.pop() || path;
  const dir = parts.join("/");
  return { dir, name };
}

function buildIndentGuides(lines: string[]): number[][] {
  return lines.map((line) => {
    const match = line.match(/^(\s*)/);
    if (!match) return [];
    const indent = match[1];
    const guides: number[] = [];
    const step = indent.startsWith("\t") ? 1 : 2;
    for (let i = step; i < indent.length; i += step) {
      guides.push(i);
    }
    return guides;
  });
}

const CodeEditor: React.FC<CodeEditorProps> = ({ filePath, initialContent, language, onSave, onClose }) => {
  const [content, setContent] = useState(initialContent);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lineCount, setLineCount] = useState(initialContent.split("\n").length);
  const [cursorLine, setCursorLine] = useState(1);
  const [cursorCol, setCursorCol] = useState(1);
  const [findOpen, setFindOpen] = useState(false);
  const [findQuery, setFindQuery] = useState("");
  const [findMatchCount, setFindMatchCount] = useState(0);
  const [findCurrentIdx, setFindCurrentIdx] = useState(0);
  const [autoSave, setAutoSave] = useState(true);

  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const gutterRef = useRef<HTMLDivElement>(null);
  const codeAreaRef = useRef<HTMLDivElement>(null);
  const findInputRef = useRef<HTMLInputElement>(null);
  const saveRef = useRef<() => void>(() => {});
  const autoSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const indentStr = useRef(detectIndent(initialContent));

  const lines = useRef(content.split("\n"));
  lines.current = content.split("\n");

  const { dir: fileDir, name: fileName } = formatFilePath(filePath);

  const handleSave = useCallback(async () => {
    setSaving(true);
    setError(null);
    try {
      await invoke("write_file", { path: filePath, content });
      setDirty(false);
      onSave?.(filePath, content);
    } catch (e) {
      setError(`保存失败: ${e}`);
    }
    setSaving(false);
  }, [filePath, content, onSave]);

  useEffect(() => {
    saveRef.current = handleSave;
  }, [handleSave]);

  useEffect(() => {
    if (!dirty || !autoSave) return;
    if (autoSaveTimerRef.current) clearTimeout(autoSaveTimerRef.current);
    autoSaveTimerRef.current = setTimeout(() => {
      saveRef.current();
    }, 3000);
    return () => {
      if (autoSaveTimerRef.current) clearTimeout(autoSaveTimerRef.current);
    };
  }, [dirty, content, autoSave]);

  useEffect(() => {
    if (findOpen) {
      requestAnimationFrame(() => findInputRef.current?.focus());
    }
  }, [findOpen]);

  useEffect(() => {
    if (!findQuery) {
      setFindMatchCount(0);
      setFindCurrentIdx(0);
      return;
    }
    try {
      const re = new RegExp(escapeRegex(findQuery), "gi");
      const matches = content.match(re);
      setFindMatchCount(matches ? matches.length : 0);
      if (findCurrentIdx > (matches ? matches.length : 0)) {
        setFindCurrentIdx(0);
      }
    } catch {
      setFindMatchCount(0);
      setFindCurrentIdx(0);
    }
  }, [findQuery, content, findCurrentIdx]);

  const updateCursor = useCallback(() => {
    const ta = textareaRef.current;
    if (!ta) return;
    const prefix = ta.value.slice(0, ta.selectionStart);
    const lineParts = prefix.split("\n");
    setCursorLine(lineParts.length);
    setCursorCol(lineParts[lineParts.length - 1].length + 1);
  }, []);

  const syncGutterScroll = useCallback(() => {
    const ta = textareaRef.current;
    const gutter = gutterRef.current;
    if (ta && gutter) {
      gutter.scrollTop = ta.scrollTop;
    }
  }, []);

  const handleChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const val = e.target.value;
    setContent(val);
    setDirty(true);
    setLineCount(val.split("\n").length);
    setError(null);
    requestAnimationFrame(syncGutterScroll);
  }, [syncGutterScroll]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    const meta = e.metaKey || e.ctrlKey;

    if (meta && e.key === "s") {
      e.preventDefault();
      saveRef.current();
      return;
    }

    if (meta && e.key === "w") {
      e.preventDefault();
      onClose?.();
      return;
    }

    if (meta && e.key === "f") {
      e.preventDefault();
      setFindOpen(true);
      return;
    }

    if (e.key === "Escape" && findOpen) {
      e.preventDefault();
      setFindOpen(false);
      requestAnimationFrame(() => textareaRef.current?.focus());
      return;
    }

    if (meta && e.key === "Enter" && findOpen) {
      e.preventDefault();
      findNext();
      return;
    }

    if (meta && e.shiftKey && e.key === "Enter" && findOpen) {
      e.preventDefault();
      findPrev();
      return;
    }

    if (e.key === "Tab") {
      e.preventDefault();
      const ta = textareaRef.current;
      if (!ta) return;
      const start = ta.selectionStart;
      const end = ta.selectionEnd;
      const newVal = content.slice(0, start) + indentStr.current + content.slice(end);
      setContent(newVal);
      setDirty(true);
      setLineCount(newVal.split("\n").length);
      requestAnimationFrame(() => {
        ta.selectionStart = ta.selectionEnd = start + indentStr.current.length;
      });
    }

    if (meta && e.key === "d") {
      e.preventDefault();
      const ta = textareaRef.current;
      if (!ta) return;
      const start = ta.selectionStart;
      const end = ta.selectionEnd;
      if (start === end) {
        const lineStart = content.lastIndexOf("\n", start - 1) + 1;
        let lineEnd = content.indexOf("\n", start);
        if (lineEnd === -1) lineEnd = content.length;
        const line = content.slice(lineStart, lineEnd);
        const newVal = content.slice(0, lineEnd) + "\n" + line + content.slice(lineEnd);
        setContent(newVal);
        setDirty(true);
        setLineCount(newVal.split("\n").length);
        requestAnimationFrame(() => {
          ta.selectionStart = ta.selectionEnd = lineEnd + 1 + (start - lineStart);
        });
      }
    }

    if (meta && e.shiftKey && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
      e.preventDefault();
      const ta = textareaRef.current;
      if (!ta) return;
      const lines_arr = content.split("\n");
      const cursorPos = ta.selectionStart;
      let lineIdx = 0;
      let pos = 0;
      for (let i = 0; i < lines_arr.length; i++) {
        if (pos + lines_arr[i].length + 1 > cursorPos) { lineIdx = i; break; }
        pos += lines_arr[i].length + 1;
      }
      const targetIdx = e.key === "ArrowDown" ? lineIdx + 1 : lineIdx - 1;
      if (targetIdx < 0 || targetIdx >= lines_arr.length) return;
      const tmp = lines_arr[lineIdx];
      lines_arr[lineIdx] = lines_arr[targetIdx];
      lines_arr[targetIdx] = tmp;
      const newVal = lines_arr.join("\n");
      setContent(newVal);
      setDirty(true);
      setLineCount(newVal.split("\n").length);
    }
  }, [content, findOpen, onClose]);

  const findNext = useCallback(() => {
    if (findMatchCount === 0 || !findQuery) return;
    const ta = textareaRef.current;
    if (!ta) return;
    const lower = content.toLowerCase();
    const qLower = findQuery.toLowerCase();
    const cursorPos = ta.selectionStart;
    const nextIdx = lower.indexOf(qLower, cursorPos + 1);
    if (nextIdx !== -1) {
      ta.selectionStart = nextIdx;
      ta.selectionEnd = nextIdx + findQuery.length;
      ta.focus();
      setFindCurrentIdx((prev) => Math.min(prev + 1, findMatchCount));
    } else {
      const firstIdx = lower.indexOf(qLower);
      if (firstIdx !== -1) {
        ta.selectionStart = firstIdx;
        ta.selectionEnd = firstIdx + findQuery.length;
        ta.focus();
        setFindCurrentIdx(1);
      }
    }
    updateCursor();
  }, [content, findQuery, findMatchCount, updateCursor]);

  const findPrev = useCallback(() => {
    if (findMatchCount === 0 || !findQuery) return;
    const ta = textareaRef.current;
    if (!ta) return;
    const lower = content.toLowerCase();
    const qLower = findQuery.toLowerCase();
    const cursorPos = ta.selectionStart;
    let prevIdx = -1;
    let idx = 0;
    while ((idx = lower.indexOf(qLower, idx)) !== -1) {
      if (idx >= cursorPos) break;
      prevIdx = idx;
      idx += 1;
    }
    if (prevIdx !== -1) {
      ta.selectionStart = prevIdx;
      ta.selectionEnd = prevIdx + findQuery.length;
      ta.focus();
      setFindCurrentIdx((prev) => Math.max(prev - 1, 1));
    } else {
      const lastIdx = lower.lastIndexOf(qLower);
      if (lastIdx !== -1) {
        ta.selectionStart = lastIdx;
        ta.selectionEnd = lastIdx + findQuery.length;
        ta.focus();
        setFindCurrentIdx(findMatchCount);
      }
    }
    updateCursor();
  }, [content, findQuery, findMatchCount, updateCursor]);

  const handleScroll = useCallback(() => {
    updateCursor();
    syncGutterScroll();
  }, [updateCursor, syncGutterScroll]);

  const handleFindKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      if (e.shiftKey) findPrev();
      else findNext();
    }
    if (e.key === "Escape") {
      setFindOpen(false);
      requestAnimationFrame(() => textareaRef.current?.focus());
    }
  }, [findNext, findPrev]);

  const indentGuides = useRef<number[][]>([]);
  indentGuides.current = buildIndentGuides(lines.current);

  const renderedLines = useCallback(() => {
    const q = findOpen ? findQuery : "";
    const currentFindIdx = findCurrentIdx;
    let matchCounter = 0;

    return lines.current.map((line, i) => {
      const tokens = tokenizeLine(line, language || "");
      const guides = indentGuides.current[i] || [];
      const tokensWithFind = q
        ? renderTokensWithFind(tokens, q, () => ++matchCounter, currentFindIdx)
        : renderTokens(tokens);

      return (
        <div key={i} className="ce-highlight-line">
          {guides.length > 0 && (
            <span className="ce-indent-guides" aria-hidden="true">
              {guides.map((col) => (
                <span key={col} style={{ left: `${col}ch` }} className="ce-indent-guide" />
              ))}
            </span>
          )}
          {tokensWithFind}
        </div>
      );
    });
  }, [language, findOpen, findQuery, findCurrentIdx]);

  return (
    <div className="code-editor glass-panel">
      <div className="ce-header">
        <div className="ce-file-info">
          <span className="ce-file-icon">{"</>"}</span>
          <span className="ce-file-name" title={filePath}>
            {dirty && <span className="ce-dirty">* </span>}
            {fileDir ? (
              <>
                <span className="ce-file-dir">{fileDir}/</span>
                <span className="ce-file-base">{fileName}</span>
              </>
            ) : (
              fileName
            )}
          </span>
          {language && <span className="ce-lang-badge">{language}</span>}
        </div>
        <div className="ce-actions">
          <span
            className={`ce-auto-save-indicator${autoSave ? " active" : ""}`}
            onClick={() => setAutoSave(!autoSave)}
            title={autoSave ? "Auto-save on" : "Auto-save off"}
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <circle cx="6" cy="6" r="4" />
              {autoSave && <circle cx="6" cy="6" r="1.5" fill="currentColor" />}
            </svg>
          </span>
          <button
            className="ce-btn ce-btn-save"
            onClick={handleSave}
            disabled={!dirty || saving}
            title="Save (Cmd+S)"
          >
            {saving ? "Saving..." : "Save"}
          </button>
          {onClose && (
            <button className="ce-btn ce-btn-close" onClick={onClose} title="Close (Cmd+W)">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                <line x1="3" y1="3" x2="11" y2="11" /><line x1="11" y1="3" x2="3" y2="11" />
              </svg>
            </button>
          )}
        </div>
      </div>

      {error && <div className="ce-error">{error}</div>}

      {findOpen && (
        <div className="ce-find-bar">
          <div className="ce-find-input-wrap">
            <svg className="ce-find-icon" width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <circle cx="5" cy="5" r="3.5" /><line x1="7.5" y1="7.5" x2="10.5" y2="10.5" />
            </svg>
            <input
              ref={findInputRef}
              className="ce-find-input"
              type="text"
              value={findQuery}
              onChange={(e) => setFindQuery(e.target.value)}
              onKeyDown={handleFindKeyDown}
              placeholder="Find..."
              spellCheck={false}
              autoComplete="off"
            />
          </div>
          <span className="ce-find-count">
            {findQuery ? `${findCurrentIdx}/${findMatchCount}` : ""}
          </span>
          <div className="ce-find-actions">
            <button className="ce-find-btn" onClick={findPrev} disabled={!findQuery || findMatchCount === 0} title="Previous (Shift+Enter)">
              <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor"><path d="M5 8L0 2h10z" /></svg>
            </button>
            <button className="ce-find-btn" onClick={findNext} disabled={!findQuery || findMatchCount === 0} title="Next (Enter)">
              <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor"><path d="M5 2l5 6H0z" /></svg>
            </button>
            <button className="ce-find-btn ce-find-close" onClick={() => { setFindOpen(false); textareaRef.current?.focus(); }} title="Close (Esc)">
              <svg width="10" height="10" viewBox="0 0 10 10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                <line x1="2" y1="2" x2="8" y2="8" /><line x1="8" y1="2" x2="2" y2="8" />
              </svg>
            </button>
          </div>
        </div>
      )}

      <div className="ce-body">
        <div className="ce-gutter" ref={gutterRef}>
          {lines.current.map((_, i) => (
            <div
              key={i}
              className={`ce-gutter-line${i + 1 === cursorLine ? " ce-gutter-active" : ""}`}
            >
              {i + 1}
            </div>
          ))}
        </div>
        <div className="ce-code-area" ref={codeAreaRef}>
          <textarea
            ref={textareaRef}
            className="ce-textarea"
            value={content}
            onChange={handleChange}
            onKeyDown={handleKeyDown}
            onScroll={handleScroll}
            onClick={handleScroll}
            onKeyUp={handleScroll}
            spellCheck={false}
            autoComplete="off"
            autoCorrect="off"
            autoCapitalize="off"
            wrap="off"
          />
          <pre className="ce-highlight-layer" aria-hidden="true">
            {renderedLines()}
          </pre>
        </div>
      </div>

      <div className="ce-status-bar">
        <span className="ce-status-item">Ln {cursorLine}, Col {cursorCol}</span>
        <span className="ce-status-item">Lines: {lineCount}</span>
        <span className="ce-status-item">{language || "plain"}</span>
        <span className="ce-status-item ce-status-right">
          {dirty ? (autoSave ? "Unsaved (auto)" : "Modified") : "Saved"}
        </span>
      </div>
    </div>
  );
};

function renderTokensWithFind(
  tokens: Token[],
  query: string,
  getMatchIdx: () => number,
  currentMatchIdx: number
): React.ReactNode[] {
  const result: React.ReactNode[] = [];
  let globalKey = 0;

  for (const token of tokens) {
    if (!query || token.cssClass === "ce-comment") {
      result.push(
        token.cssClass
          ? <span key={globalKey++} className={token.cssClass}>{token.text}</span>
          : <span key={globalKey++}>{token.text}</span>
      );
      continue;
    }

    const lower = token.text.toLowerCase();
    const qLower = query.toLowerCase();
    let start = 0;
    let idx: number;
    while ((idx = lower.indexOf(qLower, start)) !== -1) {
      if (idx > start) {
        result.push(<span key={globalKey++}>{token.text.slice(start, idx)}</span>);
      }
      const matchText = token.text.slice(idx, idx + query.length);
      const matchIdx = getMatchIdx();
      const isActive = matchIdx === currentMatchIdx;
      result.push(
        <mark key={globalKey++} className={`ce-find-mark${isActive ? " ce-find-active" : ""}`}>
          {matchText}
        </mark>
      );
      start = idx + query.length;
    }
    if (start < token.text.length) {
      result.push(<span key={globalKey++}>{token.text.slice(start)}</span>);
    }
  }

  return result;
}

export default CodeEditor;
