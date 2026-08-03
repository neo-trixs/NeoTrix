import React, { useCallback, useEffect, useRef, useState } from "react";
import { readDir, readTextFile, writeTextFile, BaseDirectory, stat } from "@tauri-apps/plugin-fs";
import { open } from "@tauri-apps/plugin-shell";
import { invoke } from "@tauri-apps/api/core";
import styles from "./FileTreePanel.module.css";

interface TreeNode {
  name: string;
  path: string;
  isDirectory: boolean;
  children?: TreeNode[];
  expanded?: boolean;
  gitStatus?: string;
  size?: number;
  modified?: number;
}

const IGNORED = new Set(["node_modules", ".git", ".next", "dist", "target", "build", ".cache", "coverage", ".turbo", ".parcel-cache"]);

export function FileTreePanel({ projectRoot, onPick }: { projectRoot?: string; onPick?: (path: string) => void }) {
  const [root, setRoot] = useState<TreeNode | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [selected, setSelected] = useState<string | null>(null);
  const [preview, setPreview] = useState<string | null>(null);
  const [previewName, setPreviewName] = useState("");
  const [previewTruncated, setPreviewTruncated] = useState(false);
  const [editing, setEditing] = useState(false);
  const [editValue, setEditValue] = useState("");
  const [saving, setSaving] = useState(false);
  const [menu, setMenu] = useState<{ x: number; y: number; node: TreeNode } | null>(null);
  const [savedFlash, setSavedFlash] = useState(false);
  const [filter, setFilter] = useState("");
  const [gitStatusMap, setGitStatusMap] = useState<Record<string, string>>({});
  const [focusedNode, setFocusedNode] = useState<string | null>(null);
  const treeRef = useRef<HTMLDivElement>(null);
  const filterRef = useRef<HTMLInputElement>(null);

  const rootPath = projectRoot || ".";

  const loadDir = useCallback(async (dirPath: string, depth: number): Promise<TreeNode[]> => {
    if (depth > 4) return [];
    try {
      const entries = await readDir(dirPath);
      const dirs: TreeNode[] = [];
      const files: TreeNode[] = [];
      for (const e of entries) {
        if (IGNORED.has(e.name)) continue;
        const node: TreeNode = {
          name: e.name,
          path: (e as { path?: string }).path ?? (dirPath === "." ? e.name : `${dirPath.replace(/[\\/]+$/, "")}/${e.name}`),
          isDirectory: e.isDirectory,
        };
        if (e.isDirectory) {
          node.children = await loadDir(node.path, depth + 1);
          dirs.push(node);
        } else {
          files.push(node);
        }
      }
      return [...dirs, ...files];
    } catch {
      return [];
    }
  }, []);

  const loadGitStatus = useCallback(async () => {
    try {
      const statuses = await invoke<{ path: string; status: string }[]>("neocodex_git_file_status", { cwd: rootPath });
      const map: Record<string, string> = {};
      for (const s of statuses) {
        map[s.path] = s.status;
      }
      setGitStatusMap(map);
    } catch {
      /* ignore git status errors */
    }
  }, [rootPath]);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const children = await loadDir(rootPath, 0);
      try {
        await readDir(rootPath);
      } catch (e) {
        setError(`无法读取目录: ${e}`);
        setRoot(null);
        return;
      }
      const node: TreeNode = { name: rootPath, path: rootPath, isDirectory: true, children, expanded: true };
      setRoot(node);
      loadGitStatus();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [rootPath, loadDir, loadGitStatus]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const toggle = (node: TreeNode) => {
    setRoot((prev) => {
      if (!prev) return prev;
      const flip = (n: TreeNode): TreeNode => {
        if (n.path === node.path) return { ...n, expanded: !n.expanded };
        if (n.children) return { ...n, children: n.children.map(flip) };
        return n;
      };
      return flip(prev);
    });
  };

  const openFile = async (node: TreeNode) => {
    setSelected(node.path);
    setFocusedNode(node.path);
    setPreviewName(node.name);
    setPreview(null);
    setEditing(false);
    if (node.isDirectory) {
      toggle(node);
      return;
    }
    try {
      const sizeLimit = 256 * 1024;
      const text = await readTextFile(node.path);
      const truncated = text.length > sizeLimit;
      setPreview(truncated ? text.slice(0, sizeLimit) + "\n\n… (已截断)" : text);
      setPreviewTruncated(truncated);
      await invoke("neocodex_open_file", { path: node.path });
      window.dispatchEvent(new CustomEvent("neotrix:mention-file", { detail: node.path }));
    } catch (e) {
      setPreview(`[无法读取 ${node.name}]\n${e}`);
      setPreviewTruncated(false);
    }
  };

  const openExternal = async (path: string) => {
    try {
      await invoke("neocodex_open_external", { path });
    } catch {
      if (path) open(path).catch(() => {});
    }
  };

  const startEdit = () => {
    if (preview === null) return;
    if (previewTruncated) {
      if (selected) open(selected).catch(() => {});
      return;
    }
    setEditValue(preview);
    setEditing(true);
  };

  const saveFile = async () => {
    if (!selected) return;
    setSaving(true);
    try {
      await writeTextFile(selected, editValue);
      setPreview(editValue);
      setEditing(false);
      setSavedFlash(true);
      setTimeout(() => setSavedFlash(false), 1600);
    } catch (e) {
      setPreview(`[保存失败]\n${e}`);
      setEditing(false);
    } finally {
      setSaving(false);
    }
  };

  const handleContextMenu = (e: React.MouseEvent, node: TreeNode) => {
    e.preventDefault();
    setMenu({ x: e.clientX, y: e.clientY, node });
    setFocusedNode(node.path);
  };

  const menuAction = async (action: "open" | "reveal" | "copy" | "new_file" | "new_folder" | "delete" | "rename", renameValue?: string) => {
    if (!menu) return;
    const { node } = menu;
    try {
      switch (action) {
        case "open":
          await openFile(node);
          break;
        case "reveal":
          await open(node.path).catch(() => {});
          break;
        case "copy":
          await navigator.clipboard.writeText(node.path);
          break;
        case "new_file":
          await invoke("neocodex_file_operation", { op: "new_file", path: node.path + "/new.txt" });
          break;
        case "new_folder":
          await invoke("neocodex_file_operation", { op: "new_folder", path: node.path + "/NewFolder" });
          break;
        case "delete":
          if (window.confirm(`删除 "${node.name}"?`)) {
            await invoke("neocodex_file_operation", { op: "delete", path: node.path });
            if (selected === node.path) {
              setSelected(null);
              setPreview(null);
            }
          }
          break;
        case "rename":
          if (renameValue && renameValue !== node.name) {
            await invoke("neocodex_file_operation", { op: "rename", path: node.path, new_name: renameValue });
          }
          break;
      }
    } catch (e) {
      console.error("Menu action failed:", e);
    }
    setMenu(null);
    refresh();
  };

  const handleKeyDown = async (e: React.KeyboardEvent) => {
    if (!root || !treeRef.current) return;
    const nodes = getVisibleNodes(root);
    if (nodes.length === 0) return;
    const focusedIndex = focusedNode ? nodes.findIndex(n => n.path === focusedNode) : -1;
    let newIndex = focusedIndex;
    switch (e.key) {
      case "Enter":
        e.preventDefault();
        if (focusedNode) {
          const node = nodes.find(n => n.path === focusedNode);
          if (node) openFile(node);
        }
        break;
      case "ArrowDown":
        e.preventDefault();
        newIndex = Math.min(focusedIndex + 1, nodes.length - 1);
        break;
      case "ArrowUp":
        e.preventDefault();
        newIndex = Math.max(focusedIndex - 1, 0);
        break;
      case "ArrowRight":
        e.preventDefault();
        if (focusedNode) {
          const node = nodes.find(n => n.path === focusedNode);
          if (node && node.isDirectory && !node.expanded) toggle(node);
          else newIndex = Math.min(focusedIndex + 1, nodes.length - 1);
        }
        break;
      case "ArrowLeft":
        e.preventDefault();
        if (focusedNode) {
          const node = nodes.find(n => n.path === focusedNode);
          if (node && node.isDirectory && node.expanded) toggle(node);
          else {
            const parent = findParent(root, focusedNode);
            if (parent) newIndex = nodes.findIndex(n => n.path === parent.path);
          }
        }
        break;
      case "Delete":
        e.preventDefault();
        if (focusedNode) {
          const node = nodes.find(n => n.path === focusedNode);
          if (node && window.confirm(`删除 "${node.name}"?`)) {
            await invoke("neocodex_file_operation", { op: "delete", path: node.path });
            if (selected === node.path) {
              setSelected(null);
              setPreview(null);
            }
            refresh();
          }
        }
        break;
      case "F2":
        e.preventDefault();
        if (focusedNode) {
          const node = nodes.find(n => n.path === focusedNode);
          if (node) {
            const newName = window.prompt("重命名:", node.name);
            if (newName) await invoke("neocodex_file_operation", { op: "rename", path: node.path, new_name: newName });
            refresh();
          }
        }
        break;
      case "n":
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault();
          const target = focusedNode ? nodes.find(n => n.path === focusedNode) : root;
          if (target && target.isDirectory) {
            await invoke("neocodex_file_operation", { op: "new_file", path: target.path + "/new.txt" });
            refresh();
          }
        }
        break;
    }
    if (newIndex !== focusedIndex && newIndex >= 0) {
      setFocusedNode(nodes[newIndex].path);
      treeRef.current?.querySelector(`[data-path="${nodes[newIndex].path}"]`)?.scrollIntoView({ block: "nearest" });
    }
  };

  const getVisibleNodes = (node: TreeNode): TreeNode[] => {
    const result: TreeNode[] = [];
    const traverse = (n: TreeNode) => {
      if (filter && !n.name.toLowerCase().includes(filter.toLowerCase())) return;
      result.push(n);
      if (n.isDirectory && n.expanded && n.children) {
        for (const child of n.children) traverse(child);
      }
    };
    if (node.children) {
      for (const child of node.children) traverse(child);
    }
    return result;
  };

  const findParent = (node: TreeNode, targetPath: string, parent?: TreeNode): TreeNode | undefined => {
    if (!node.children) return undefined;
    for (const child of node.children) {
      if (child.path === targetPath) return parent || node;
      if (child.children) {
        const found = findParent(child, targetPath, child);
        if (found) return found;
      }
    }
    return undefined;
  };

  const filterNode = (node: TreeNode): boolean => {
    if (!filter) return true;
    if (node.name.toLowerCase().includes(filter.toLowerCase())) return true;
    if (node.children) return node.children.some(filterNode);
    return false;
  };

  const renderTree = (nodes: TreeNode[], depth: number): React.ReactNode => (
    nodes
      .filter(filterNode)
      .map((node) => {
        const isSel = selected === node.path;
        const isFocused = focusedNode === node.path;
        const gitStatus = gitStatusMap[node.path] || "";
        return (
          <div key={node.path}>
            <button
              type="button"
              ref={(el) => { if (isFocused && el) el.focus(); }}
              className={`${styles.row} ${isSel ? styles.selected : ""} ${isFocused ? styles.focused : ""}`}
              style={{ paddingLeft: 8 + depth * 14 }}
              onClick={() => openFile(node)}
              onDoubleClick={(e) => { e.stopPropagation(); if (!node.isDirectory) openFile(node); }}
              onContextMenu={(e) => handleContextMenu(e, node)}
              onKeyDown={handleKeyDown}
              data-path={node.path}
              tabIndex={0}
            >
              <span className={styles.icon}>{node.isDirectory ? (node.expanded ? "▾" : "▸") : "·"}</span>
              <span className={styles.name} title={node.path}>{node.name}</span>
              {gitStatus && <span className={`${styles.gitStatus} ${styles["git-" + gitStatus.trim().replace(" ", "-")]}`} title={gitStatus}>{gitStatus.trim()}</span>}
            </button>
            {node.isDirectory && node.expanded && node.children && renderTree(node.children, depth + 1)}
          </div>
        );
      })
  );

  const loadFileInfo = useCallback(async (path: string) => {
    try {
      const info = await stat(path);
      return { size: info.size, modified: info.mtime };
    } catch {
      return { size: 0, modified: 0 };
    }
  }, []);

  const [hoverInfo, setHoverInfo] = useState<{ path: string; size: number; modified: number } | null>(null);

  return (
    <div className={styles.panel} data-testid="file-tree-panel" onKeyDown={handleKeyDown}>
      <div className={styles.header}>
        <span className={styles.title}>文件</span>
        <div className={styles.headerActions}>
          <input
            ref={filterRef}
            type="text"
            className={styles.filterInput}
            placeholder="搜索文件..."
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            onKeyDown={(e) => e.stopPropagation()}
            aria-label="搜索文件"
          />
          <button type="button" className={styles.refresh} onClick={refresh} title="刷新" data-testid="file-tree-refresh">↻</button>
        </div>
      </div>
      <div className={styles.tree} ref={treeRef} tabIndex={0}>
        {loading && <div className={styles.muted}>加载中…</div>}
        {error && <div className={styles.error}>{error}</div>}
        {!loading && !error && root && root.children && root.children.length > 0 && renderTree(root.children, 0)}
        {!loading && !error && root && (!root.children || root.children.length === 0) && <div className={styles.muted}>无文件</div>}
        {!loading && !error && !root && <div className={styles.muted}>无文件</div>}
      </div>
      {preview !== null && (
        <div className={styles.preview}>
          <div className={styles.previewHeader}>
            <span className={styles.previewName}>{previewName}</span>
            <div className={styles.previewActions}>
              {savedFlash && <span className={styles.savedFlash}>已保存</span>}
              {!editing && previewName && (
                <>
                  <button type="button" className={styles.editBtn} onClick={startEdit}>编辑</button>
                  <button type="button" className={styles.externalBtn} onClick={() => selected && openExternal(selected)}>外部打开</button>
                </>
              )}
              {editing && (
                <>
                  <button type="button" className={styles.saveBtn} onClick={saveFile} disabled={saving}>
                    {saving ? "保存中…" : "保存"}
                  </button>
                  <button type="button" className={styles.editBtn} onClick={() => setEditing(false)}>取消</button>
                </>
              )}
              <button type="button" className={styles.close} onClick={() => setPreview(null)}>✕</button>
            </div>
          </div>
          {editing ? (
            <textarea
              className={styles.editor}
              value={editValue}
              onChange={(e) => setEditValue(e.target.value)}
              spellCheck={false}
            />
          ) : (
            <pre className={styles.pre}>{preview}</pre>
          )}
        </div>
      )}
      {onPick && selected && !editing && (
        <button type="button" className={styles.pickBtn} onClick={() => onPick(selected)}>插入到输入</button>
      )}
      {menu && (
        <>
          <div className={styles.menuOverlay} onClick={() => setMenu(null)} onContextMenu={(e) => { e.preventDefault(); setMenu(null); }} />
          <div className={styles.menu} style={{ top: menu.y, left: menu.x }}>
            <button type="button" className={styles.menuItem} onClick={() => menuAction("open")}>打开</button>
            <button type="button" className={styles.menuItem} onClick={() => menuAction("reveal")}>在系统中显示</button>
            <button type="button" className={styles.menuItem} onClick={() => menuAction("copy")}>复制路径</button>
            <hr className={styles.menuSep} />
            {menu.node.isDirectory && (
              <>
                <button type="button" className={styles.menuItem} onClick={() => menuAction("new_file")}>新建文件</button>
                <button type="button" className={styles.menuItem} onClick={() => menuAction("new_folder")}>新建文件夹</button>
                <hr className={styles.menuSep} />
              </>
            )}
            <button type="button" className={styles.menuItem} onClick={() => {
              const newName = window.prompt("重命名:", menu.node.name);
              if (newName) menuAction("rename", newName);
            }}>重命名</button>
            <button type="button" className={styles.menuItemDanger} onClick={() => menuAction("delete")}>删除</button>
          </div>
        </>
      )}
    </div>
  );
}