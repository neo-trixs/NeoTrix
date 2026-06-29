import React, { useState, useEffect, useRef, useMemo } from "react";
import { scanProjectFiles, type FileEntry } from "../lib/api";

interface Props {
  query: string;
  projectPath?: string;
  onSelect: (file: FileEntry) => void;
  onClose: () => void;
}

const EXT_COLORS: Record<string, string> = {
  tsx: "#007aff",
  ts: "#007aff",
  jsx: "#f7df1e",
  js: "#f7df1e",
  rs: "#de5833",
  css: "#663399",
  scss: "#663399",
  sass: "#663399",
  json: "#34c759",
  md: "#86868b",
  html: "#e44d26",
  py: "#3776ab",
  toml: "#86868b",
  yaml: "#86868b",
  yml: "#86868b",
  svg: "#ffb347",
  xml: "#ff8c00",
  sh: "#34c759",
  bash: "#34c759",
  zsh: "#34c759",
  env: "#ff9500",
  gitignore: "#86868b",
  lock: "#86868b",
  mod: "#de5833",
};

function getFileColor(name: string): string {
  const ext = name.split(".").pop()?.toLowerCase() || "";
  return EXT_COLORS[ext] || "#86868b";
}

function getFileGlyph(name: string): string {
  const ext = name.split(".").pop()?.toLowerCase() || "";
  const codeExts = ["ts", "tsx", "js", "jsx", "rs", "py", "go", "java", "rb", "c", "cpp", "h", "hpp", "cs", "swift", "kt"];
  if (codeExts.includes(ext)) return "{ }";
  if (["css", "scss", "sass", "less"].includes(ext)) return "#";
  if (["json", "toml", "yaml", "yml", "xml", "env", "gitignore", "lock"].includes(ext)) return "<>";
  if (["md", "txt", "rtf"].includes(ext)) return "¶";
  if (["html", "htm"].includes(ext)) return "</>";
  if (["sh", "bash", "zsh", "fish"].includes(ext)) return ">";
  if (["png", "jpg", "jpeg", "gif", "svg", "ico", "webp"].includes(ext)) return "◷";
  return "·";
}

const FileMention: React.FC<Props> = ({ query, projectPath, onSelect, onClose }) => {
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scanProjectFiles(projectPath).then(setFiles).catch(() => setFiles([]));
  }, [projectPath]);

  const filtered = useMemo(() => {
    const q = query.toLowerCase();
    if (!q) return files.slice(0, 50);
    const terms = q.split(/\s+/).filter(Boolean);
    return files
      .filter((f) => {
        const name = f.name.toLowerCase();
        const path = f.path.toLowerCase();
        return terms.every((t) => name.includes(t) || path.includes(t));
      })
      .slice(0, 50);
  }, [files, query]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((i) => Math.min(i + 1, filtered.length - 1));
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((i) => Math.max(i - 1, 0));
      }
      if (e.key === "Enter" && filtered[selectedIndex]) {
        e.preventDefault();
        onSelect(filtered[selectedIndex]);
      }
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [filtered, selectedIndex, onSelect, onClose]);

  useEffect(() => {
    const el = listRef.current?.querySelector(".file-mention-item.selected") as HTMLElement;
    el?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  return (
    <div className="file-mention-dropdown">
      <div className="file-mention-header">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
          <path d="M2 3h4l2 2h4a1 1 0 011 1v6a1 1 0 01-1 1H2a1 1 0 01-1-1V4a1 1 0 011-1z" />
        </svg>
        <span>Search files...</span>
        <span className="file-mention-count">{filtered.length} files</span>
      </div>
      <div className="file-mention-list" ref={listRef}>
        {filtered.length === 0 ? (
          <div className="file-mention-empty">No matching files</div>
        ) : (
          filtered.map((f, i) => (
            <div
              key={f.path}
              className={`file-mention-item ${i === selectedIndex ? "selected" : ""}`}
              onMouseDown={(e) => {
                e.preventDefault();
                onSelect(f);
              }}
              onMouseEnter={() => setSelectedIndex(i)}
            >
              <span className="file-mention-glyph" style={{ color: getFileColor(f.name) }}>
                {getFileGlyph(f.name)}
              </span>
              <div className="file-mention-info">
                <span className="file-mention-name">{f.name}</span>
                <span className="file-mention-path">{f.path}</span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default FileMention;
