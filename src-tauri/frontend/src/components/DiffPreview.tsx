import { useState, useMemo } from "react";

interface DiffHunk {
  oldStart: number;
  newStart: number;
  lines: DiffLine[];
}

interface DiffLine {
  type: "add" | "del" | "ctx";
  content: string;
  oldLineNum: number | null;
  newLineNum: number | null;
}

interface DiffFile {
  oldPath: string;
  newPath: string;
  hunks: DiffHunk[];
}

function parseDiff(raw: string): DiffFile[] {
  const files: DiffFile[] = [];
  let currentFile: DiffFile | null = null;
  let currentHunk: DiffHunk | null = null;

  const lines = raw.split("\n");
  for (const line of lines) {
    const fileMatch = line.match(/^--- a\/(.+)/);
    const fileMatch2 = line.match(/^\+\+\+ b\/(.+)/);
    const hunkMatch = line.match(/^@@ -(\d+),?\d* \+(\d+),?\d* @@/);

    if (fileMatch) {
      if (currentFile) files.push(currentFile);
      currentFile = { oldPath: fileMatch[1], newPath: "", hunks: [] };
    } else if (fileMatch2 && currentFile) {
      currentFile.newPath = fileMatch2[1];
    } else if (hunkMatch) {
      if (currentHunk && currentFile) currentFile.hunks.push(currentHunk);
      currentHunk = {
        oldStart: parseInt(hunkMatch[1]),
        newStart: parseInt(hunkMatch[2]),
        lines: [],
      };
    } else if (currentHunk) {
      const type: DiffLine["type"] = line.startsWith("+") ? "add" : line.startsWith("-") ? "del" : "ctx";
      currentHunk.lines.push({
        type,
        content: line.slice(1),
        oldLineNum: type === "add" ? null : (currentHunk.oldStart + currentHunk.lines.filter(l => l.type !== "add").length),
        newLineNum: type === "del" ? null : (currentHunk.newStart + currentHunk.lines.filter(l => l.type !== "del").length),
      });
    }
  }
  if (currentHunk && currentFile) currentFile.hunks.push(currentHunk);
  if (currentFile) files.push(currentFile);
  return files;
}

function FileDiff({ file }: { file: DiffFile }) {
  const [collapsed, setCollapsed] = useState(false);
  const addCount = file.hunks.reduce((s, h) => s + h.lines.filter(l => l.type === "add").length, 0);
  const delCount = file.hunks.reduce((s, h) => s + h.lines.filter(l => l.type === "del").length, 0);

  const filename = file.newPath || file.oldPath;
  const status = file.oldPath === "/dev/null" ? "added" : file.newPath === "/dev/null" ? "deleted" : "modified";

  return (
    <div className="diff-file">
      <div className="diff-file-header" onClick={() => setCollapsed(!collapsed)}>
        <span className="diff-file-icon">{collapsed ? "▸" : "▾"}</span>
        <span className={`diff-file-status diff-status-${status}`}>
          {status === "added" ? "A" : status === "deleted" ? "D" : "M"}
        </span>
        <span className="diff-file-path">{filename}</span>
        <span className="diff-file-stats">
          <span className="diff-stat-add">+{addCount}</span>
          <span className="diff-stat-del">-{delCount}</span>
        </span>
      </div>
      {!collapsed && file.hunks.map((hunk, hi) => (
        <div key={hi} className="diff-hunk">
          <div className="diff-hunk-header">
            @@ -{hunk.oldStart},{hunk.newStart} @@
          </div>
          {hunk.lines.map((line, li) => (
            <div key={li} className={`diff-line diff-line-${line.type}`}>
              <span className="diff-line-num diff-line-old">{line.oldLineNum ?? ""}</span>
              <span className="diff-line-num diff-line-new">{line.newLineNum ?? ""}</span>
              <span className="diff-line-prefix">{line.type === "add" ? "+" : line.type === "del" ? "-" : " "}</span>
              <span className="diff-line-content">{line.content}</span>
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}

interface DiffPreviewProps {
  content: string;
  defaultCollapsed?: boolean;
}

export default function DiffPreview({ content, defaultCollapsed = false }: DiffPreviewProps) {
  const [collapsed, setCollapsed] = useState(defaultCollapsed);
  const files = useMemo(() => parseDiff(content), [content]);

  if (files.length === 0) return null;

  const totalAdd = files.reduce((s, f) => s + f.hunks.reduce((s2, h) => s2 + h.lines.filter(l => l.type === "add").length, 0), 0);
  const totalDel = files.reduce((s, f) => s + f.hunks.reduce((s2, h) => s2 + h.lines.filter(l => l.type === "del").length, 0), 0);

  return (
    <div className="diff-preview">
      <div className="diff-preview-header" onClick={() => setCollapsed(!collapsed)}>
        <span className="diff-preview-icon">{collapsed ? "▸" : "▾"}</span>
        <span className="diff-preview-label">Diff — {files.length} file{files.length > 1 ? "s" : ""}</span>
        <span className="diff-preview-stats">
          <span className="diff-stat-add">{totalAdd} additions</span>
          <span className="diff-stat-del">{totalDel} deletions</span>
        </span>
      </div>
      {!collapsed && (
        <div className="diff-files">
          {files.map((f, i) => <FileDiff key={i} file={f} />)}
        </div>
      )}
    </div>
  );
}
