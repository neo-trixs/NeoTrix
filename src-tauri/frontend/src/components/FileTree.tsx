import React, { useState, useEffect, useCallback, useRef } from "react";
import * as api from "../lib/api";
import type { FileNode } from "../types";
import { open } from "@tauri-apps/plugin-shell";

interface Props {
  rootPath: string;
  onClose: () => void;
  onStatusChange: (s: string) => void;
}

interface ContextMenu {
  x: number;
  y: number;
  path: string;
  name: string;
}

interface FilePreviewData {
  path: string;
  content: string;
}

function formatSize(bytes?: number): string {
  if (bytes === undefined || bytes === null) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const FileTree: React.FC<Props> = ({ rootPath, onClose, onStatusChange }) => {
  const [nodes, setNodes] = useState<FileNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [contextMenu, setContextMenu] = useState<ContextMenu | null>(null);
  const [preview, setPreview] = useState<FilePreviewData | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const result: FileNode[] = await api.readDirRecursive(rootPath, 3);
        setNodes(result);
      } catch (e) {
        onStatusChange(`加载失败: ${e}`);
      }
      setLoading(false);
    };
    load();
  }, [rootPath, onStatusChange]);

  useEffect(() => {
    const handleClick = () => setContextMenu(null);
    window.addEventListener("click", handleClick);
    return () => window.removeEventListener("click", handleClick);
  }, []);

  const toggleExpand = useCallback((path: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }, []);

  const handleDoubleClick = useCallback(async (node: FileNode) => {
    if (node.is_dir) return;
    setPreviewLoading(true);
    setPreview(null);
    try {
      const content = await api.readFile(node.path);
      setPreview({ path: node.path, content });
    } catch (e) {
      onStatusChange(`读取失败: ${e}`);
    }
    setPreviewLoading(false);
  }, [onStatusChange]);

  const handleContextMenu = useCallback((e: React.MouseEvent, node: FileNode) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ x: e.clientX, y: e.clientY, path: node.path, name: node.name });
  }, []);

  const handleCopyPath = useCallback(async (path: string) => {
    try {
      await navigator.clipboard.writeText(path);
      onStatusChange("路径已复制");
    } catch {
      onStatusChange("复制失败");
    }
    setContextMenu(null);
  }, [onStatusChange]);

  const handleOpenInEditor = useCallback(async (path: string) => {
    try {
      await open(path);
    } catch {
      onStatusChange("打开文件失败");
    }
    setContextMenu(null);
  }, [onStatusChange]);

  const renderNode = (node: FileNode, depth: number): React.ReactNode => {
    const indent = depth * 16;
    const isExpanded = expanded.has(node.path);

    return (
      <div key={node.path}>
        <div
          className={`file-node${node.is_dir ? " file-node-dir" : ""}`}
          style={{ paddingLeft: 12 + indent }}
          onClick={() => node.is_dir && toggleExpand(node.path)}
          onDoubleClick={() => handleDoubleClick(node)}
          onContextMenu={(e) => handleContextMenu(e, node)}
        >
          <span className="file-icon">
            {node.is_dir ? (
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" style={{ transform: isExpanded ? "rotate(90deg)" : "none", transition: "transform 0.15s ease" }}>
                <path d="M4 2l4 4-4 4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            ) : (
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                <path d="M7 1H3a1 1 0 00-1 1v8a1 1 0 001 1h6a1 1 0 001-1V4L7 1z" stroke="currentColor" strokeWidth="1.2" />
              </svg>
            )}
          </span>
          <span className="file-name">{node.name}</span>
          {!node.is_dir && node.size !== undefined && (
            <span className="file-size">{formatSize(node.size)}</span>
          )}
        </div>
        {node.is_dir && isExpanded && node.children?.map((child) => renderNode(child, depth + 1))}
      </div>
    );
  };

  return (
    <div className="file-panel glass-panel">
      <div className="file-panel-header">
        <span>文件</span>
        <button className="btn-icon" onClick={onClose}>
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
        </button>
      </div>
      <div className="file-panel-content">
        {loading ? (
          <div className="file-loading">加载中...</div>
        ) : nodes.length === 0 ? (
          <div className="file-empty">空目录</div>
        ) : (
          nodes.map((node) => renderNode(node, 0))
        )}
      </div>

      {preview && (
        <div className="file-preview glass-panel">
          <div className="file-preview-header">
            <span className="file-preview-name">{preview.path}</span>
            <button className="btn-icon" onClick={() => setPreview(null)}>
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              </svg>
            </button>
          </div>
          <pre className="file-preview-content">{preview.content}</pre>
        </div>
      )}

      {previewLoading && (
        <div className="file-preview-loading">加载中...</div>
      )}

      {contextMenu && (
        <div
          ref={menuRef}
          className="file-context-menu glass-panel"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          <div className="file-context-item" onClick={() => handleOpenInEditor(contextMenu.path)}>
            在编辑器中打开
          </div>
          <div className="file-context-item" onClick={() => handleCopyPath(contextMenu.path)}>
            复制路径
          </div>
        </div>
      )}
    </div>
  );
};

export default FileTree;
