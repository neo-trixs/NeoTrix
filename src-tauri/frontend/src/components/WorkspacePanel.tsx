import React, { useState, useEffect, useCallback } from "react";
import * as api from "../lib/api";
import type { FileNode, FileEntry } from "../types";

interface Props {
  rootPath: string;
  onClose: () => void;
  onStatusChange: (s: string) => void;
  onOpenFile: (path: string) => void;
}

export const WorkspacePanel: React.FC<Props> = ({ rootPath, onClose, onStatusChange, onOpenFile }) => {
  const [nodes, setNodes] = useState<FileNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<FileEntry[]>([]);
  const [searching, setSearching] = useState(false);
  const [gitBranch, setGitBranch] = useState("");
  const [gitModified, setGitModified] = useState(0);
  const [showSearch, setShowSearch] = useState(false);

  useEffect(() => {
    const load = async () => {
      try {
        const result: FileNode[] = await api.readDirRecursive(rootPath, 4);
        setNodes(result);
      } catch (e) {
        onStatusChange(`Load failed: ${e}`);
      }
      setLoading(false);
    };
    load();
    fetchGitInfo();
  }, [rootPath]);

  const fetchGitInfo = async () => {
    try {
      const branch = await api.execCommand("git rev-parse --abbrev-ref HEAD");
      if (branch) setGitBranch(branch.trim());
      const status = await api.execCommand("git status --porcelain | wc -l");
      if (status) setGitModified(parseInt(status.trim()) || 0);
    } catch {}
  };

  const handleSearch = useCallback(async (q: string) => {
    setSearchQuery(q);
    if (!q.trim()) { setSearchResults([]); return; }
    setSearching(true);
    const results = await api.searchFiles(q);
    setSearchResults(results.slice(0, 30));
    setSearching(false);
  }, []);

  const toggleExpand = useCallback(async (path: string) => {
    if (expanded.has(path)) {
      const next = new Set(expanded);
      next.delete(path);
      setExpanded(next);
      return;
    }
    setExpanded(prev => new Set(prev).add(path));
  }, [expanded]);

  const getIcon = (node: FileNode): string => {
    if (node.is_dir) return expanded.has(node.path) ? "📂" : "📁";
    const ext = node.name.split(".").pop()?.toLowerCase();
    if (["rs", "go", "py", "js", "ts", "tsx", "rb", "java"].includes(ext || "")) return "📄";
    if (["json", "yaml", "yml", "toml", "xml", "md", "css", "html"].includes(ext || "")) return "📝";
    if (["png", "jpg", "jpeg", "gif", "svg", "ico"].includes(ext || "")) return "🖼️";
    return "📄";
  };

  const renderNode = (node: FileNode, depth: number): React.ReactNode => {
    const isExpanded = expanded.has(node.path);
    const children = node.children || [];

    return (
      <div key={node.path}>
        <div
          className="workspace-file-row"
          style={{ paddingLeft: 12 + depth * 16 }}
          onClick={() => {
            if (node.is_dir) toggleExpand(node.path);
            else onOpenFile(node.path);
          }}
          onContextMenu={(e) => {
            e.preventDefault();
            onOpenFile(node.path);
          }}
        >
          <span className="workspace-file-icon">{getIcon(node)}</span>
          <span className="workspace-file-name">{node.name}</span>
          {!node.is_dir && node.size !== undefined && (
            <span className="workspace-file-size">{formatSize(node.size)}</span>
          )}
        </div>
        {node.is_dir && isExpanded && children.map((child) => renderNode(child, depth + 1))}
      </div>
    );
  };

  return (
    <div className="workspace-panel glass-panel">
      <div className="workspace-header">
        <div className="workspace-title-row">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.3">
            <path d="M1 4l6-3 6 3v7a1 1 0 01-1 1H2a1 1 0 01-1-1V4z" />
            <path d="M1 4l6 3 6-3M7 11V7" />
          </svg>
          <span className="workspace-title">Workspace</span>
          <div className="workspace-header-actions">
            <button className="workspace-icon-btn" onClick={() => setShowSearch(!showSearch)} title="Search files">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3">
                <circle cx="5" cy="5" r="3.5" /><path d="M8 8l3 3" />
              </svg>
            </button>
            <button className="workspace-icon-btn" onClick={onClose} title="Close panel">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3">
                <path d="M2 2l8 8M10 2l-8 8" />
              </svg>
            </button>
          </div>
        </div>
        {gitBranch && (
          <div className="workspace-git-row">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3">
              <path d="M3 1v8a2 2 0 002 2h4" /><path d="M7 8l3-3-3-3" />
            </svg>
            <span className="workspace-git-branch">{gitBranch}</span>
            {gitModified > 0 && (
              <span className="workspace-git-modified">{gitModified} modified</span>
            )}
          </div>
        )}
      </div>

      {showSearch && (
        <div className="workspace-search">
          <input
            type="text"
            className="workspace-search-input"
            placeholder="Search files..."
            value={searchQuery}
            onChange={(e) => handleSearch(e.target.value)}
            autoFocus
          />
          {searching && <div className="workspace-search-status">Searching...</div>}
          {searchResults.length > 0 && (
            <div className="workspace-search-results">
              {searchResults.map((r) => (
                <div key={r.path} className="workspace-search-item" onClick={() => onOpenFile(r.path)}>
                  {r.path}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      <div className="workspace-tree">
        {loading ? (
          <div className="workspace-loading">Loading...</div>
        ) : (
          nodes.map((node) => renderNode(node, 0))
        )}
      </div>
    </div>
  );
};

function formatSize(bytes?: number): string {
  if (bytes === undefined || bytes === null) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default WorkspacePanel;
