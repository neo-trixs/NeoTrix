import React, { useCallback, useEffect, useState } from "react";
import { readDir, readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { open } from "@tauri-apps/plugin-shell";
import styles from "./FileTreePanel.module.css";

interface TreeNode {
  name: string;
  path: string;
  isDirectory: boolean;
  children?: TreeNode[];
  expanded?: boolean;
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
  const [menu, setMenu] = useState<{ x: number; y: number; path: string; name: string } | null>(null);
  const [savedFlash, setSavedFlash] = useState(false);

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

  const refresh = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const children = await loadDir(rootPath, 0);
      // loadDir swallows per-entry errors; a failed root read must surface as
      // an error state, not a misleading "无文件" empty tree.
      try {
        await readDir(rootPath);
      } catch (e) {
        setError(`无法读取目录: ${e}`);
        setRoot(null);
        return;
      }
      const node: TreeNode = { name: rootPath, path: rootPath, isDirectory: true, children, expanded: true };
      setRoot(node);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [rootPath, loadDir]);

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
    } catch (e) {
      setPreview(`[无法读取 ${node.name}]\n${e}`);
      setPreviewTruncated(false);
    }
  };

  const startEdit = () => {
    if (preview === null) return;
    // Never edit truncated previews: writing them back would silently destroy
    // the tail of files >256 KB. Force open externally instead.
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
    if (node.isDirectory) return;
    setMenu({ x: e.clientX, y: e.clientY, path: node.path, name: node.name });
  };

  const menuAction = async (action: "open" | "reveal" | "copy") => {
    if (!menu) return;
    try {
      if (action === "copy") {
        await navigator.clipboard.writeText(menu.path);
      } else {
        await open(menu.path);
      }
    } catch {
      /* ignore */
    }
    setMenu(null);
  };

  const renderTree = (nodes: TreeNode[], depth: number): React.ReactNode => (
    nodes.map((node) => (
      <div key={node.path}>
        <button
          type="button"
          className={`${styles.row} ${selected === node.path ? styles.selected : ""}`}
          style={{ paddingLeft: 8 + depth * 14 }}
          onClick={() => openFile(node)}
          onContextMenu={(e) => handleContextMenu(e, node)}
        >
          <span className={styles.icon}>{node.isDirectory ? (node.expanded ? "▾" : "▸") : "·"}</span>
          <span className={styles.name} title={node.path}>{node.name}</span>
        </button>
        {node.isDirectory && node.expanded && node.children && renderTree(node.children, depth + 1)}
      </div>
    ))
  );

  return (
    <div className={styles.panel} data-testid="file-tree-panel">
      <div className={styles.header}>
        <span className={styles.title}>文件</span>
        <button type="button" className={styles.refresh} onClick={refresh} title="刷新" data-testid="file-tree-refresh">↻</button>
      </div>
      <div className={styles.tree}>
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
                <button type="button" className={styles.editBtn} onClick={startEdit}>编辑</button>
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
            <button type="button" className={styles.menuItem} onClick={() => menuAction("open")}>在外部编辑器打开</button>
            <button type="button" className={styles.menuItem} onClick={() => menuAction("reveal")}>在 Finder 中显示</button>
            <button type="button" className={styles.menuItem} onClick={() => menuAction("copy")}>复制路径</button>
          </div>
        </>
      )}
    </div>
  );
}
