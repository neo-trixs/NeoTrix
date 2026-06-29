import React, { useState, useEffect, useCallback, useRef } from "react";
import { useStore } from "../store";
import * as api from "../lib/api";
import type { FileNode, VirtualApp, DesktopWindow } from "../types";

const VIRTUAL_APPS: VirtualApp[] = [
  { id: "terminal", name: "Terminal", icon: "💻", description: "Command line interface", action: "terminal" },
  { id: "files", name: "Files", icon: "📁", description: "Browse project files", action: "files" },
  { id: "settings", name: "Settings", icon: "⚙️", description: "Configure NeoTrix", action: "settings" },
  { id: "splitview", name: "Split View", icon: "🔄", description: "Compare model responses", action: "splitview" },
  { id: "agentmaker", name: "Agent Maker", icon: "🤖", description: "Create custom agents", action: "agentmaker" },
  { id: "agentflow", name: "Agent Flow", icon: "🔀", description: "Visual agent workflow", action: "agentflow" },
];

const DESKTOP_ICONS = [
  { id: "myfiles", name: "My Files", icon: "📁" },
  { id: "applications", name: "Applications", icon: "🧩" },
  { id: "trash", name: "Trash", icon: "🗑️" },
];

const DOCK_ITEMS = [
  { id: "files", icon: "📁", label: "Files", action: "files" as const },
  { id: "terminal", icon: "💻", label: "Terminal", action: "terminal" as const },
  { id: "settings", icon: "⚙️", label: "Settings", action: "settings" as const },
];

interface FileBrowserState {
  path: string;
  nodes: FileNode[];
  loading: boolean;
}

interface DragState {
  winId: string;
  startX: number;
  startY: number;
  origX: number;
  origY: number;
}

function formatSize(bytes?: number): string {
  if (bytes === undefined || bytes === null) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const DesktopIcon: React.FC<{
  name: string;
  icon: string;
  selected: boolean;
  onSelect: () => void;
  onOpen: () => void;
}> = ({ name, icon, selected, onSelect, onOpen }) => (
  <div
    className={`desktop-icon${selected ? " selected" : ""}`}
    onClick={onSelect}
    onDoubleClick={onOpen}
  >
    <div className="desktop-icon-image">{icon}</div>
    <div className="desktop-icon-label">{name}</div>
  </div>
);

const FilePreviewOverlay: React.FC<{
  path: string;
  content: string;
  onClose: () => void;
}> = ({ path, content, onClose }) => (
  <div className="file-preview-overlay" onClick={onClose}>
    <div className="file-preview-modal glass-panel" onClick={(e) => e.stopPropagation()}>
      <div className="file-preview-modal-header">
        <span className="file-preview-name">{path}</span>
        <button className="btn-icon" onClick={onClose}>
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
        </button>
      </div>
      <pre className="file-preview-content">{content}</pre>
    </div>
  </div>
);

const VirtualOS: React.FC = () => {
  const [desktopView, setDesktopView] = useState<"desktop" | "applications">("desktop");
  const [selectedIcon, setSelectedIcon] = useState<string | null>(null);
  const [nextZ, setNextZ] = useState(1);
  const [previewData, setPreviewData] = useState<{ path: string; content: string } | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [fileBrowsers, setFileBrowsers] = useState<Record<string, FileBrowserState>>({});

  const dragRef = useRef<DragState | null>(null);

  const windows = useStore((s) => s.desktopWindows);
  const openWindow = useStore((s) => s.openWindow);
  const closeWindow = useStore((s) => s.closeWindow);
  const focusWindow = useStore((s) => s.focusWindow);
  const moveWindow = useStore((s) => s.moveWindow);
  const toggleMinimizeWindow = useStore((s) => s.toggleMinimizeWindow);
  const setShowTerminal = useStore((s) => s.setShowTerminal);
  const setShowSettings = useStore((s) => s.setShowSettings);
  const setSplitViewActive = useStore((s) => s.setSplitViewActive);
  const setVirtualOSActive = useStore((s) => s.setVirtualOSActive);
  const setAgentMakerActive = useStore((s) => s.setAgentMakerActive);
  const setAgentFlowActive = useStore((s) => s.setAgentFlowActive);
  const projectPath = useStore((s) => s.projectPath);

  useEffect(() => {
    if (!dragRef.current) return;
    const handleMouseMove = (e: MouseEvent) => {
      const d = dragRef.current!;
      const dx = e.clientX - d.startX;
      const dy = e.clientY - d.startY;
      moveWindow(d.winId, d.origX + dx, d.origY + dy);
    };
    const handleMouseUp = () => { dragRef.current = null; };
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [moveWindow]);

  const handleTitleMouseDown = useCallback((e: React.MouseEvent, winId: string, win: DesktopWindow) => {
    e.preventDefault();
    focusWindow(winId);
    dragRef.current = {
      winId,
      startX: e.clientX,
      startY: e.clientY,
      origX: win.x,
      origY: win.y,
    };
  }, [focusWindow]);

  const loadFileBrowser = useCallback(async (winId: string, dirPath: string) => {
    setFileBrowsers((prev) => ({ ...prev, [winId]: { path: dirPath, nodes: [], loading: true } }));
    try {
      const nodes = await api.readDirRecursive(dirPath, 1);
      setFileBrowsers((prev) => ({ ...prev, [winId]: { path: dirPath, nodes, loading: false } }));
    } catch {
      setFileBrowsers((prev) => ({ ...prev, [winId]: { path: dirPath, nodes: [], loading: false } }));
    }
  }, []);

  const openAppAction = useCallback((action: VirtualApp["action"]) => {
    if (action === "terminal") { setShowTerminal(true); return true; }
    if (action === "settings") { setShowSettings(true); return true; }
    if (action === "splitview") { setSplitViewActive(true); return true; }
    if (action === "agentmaker") { setAgentMakerActive(true); return true; }
    if (action === "agentflow") { setAgentFlowActive(true); return true; }
    return false;
  }, [setShowTerminal, setShowSettings, setSplitViewActive, setAgentMakerActive, setAgentFlowActive]);

  const openAppWindow = useCallback((app: VirtualApp) => {
    if (openAppAction(app.action)) return;
    if (app.action !== "files") return;
    const id = `win-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    const z = nextZ;
    setNextZ(z + 1);
    const win: DesktopWindow = {
      id,
      title: app.name,
      appId: app.id,
      x: 60 + (windows.length % 5) * 30,
      y: 40 + (windows.length % 5) * 30,
      width: 420,
      height: 380,
      zIndex: z,
      minimized: false,
    };
    openWindow(win);
    loadFileBrowser(id, projectPath || ".");
  }, [nextZ, windows.length, openWindow, loadFileBrowser, projectPath, openAppAction]);

  const handleDesktopIconOpen = useCallback((id: string) => {
    if (id === "applications") { setDesktopView("applications"); return; }
    if (id === "myfiles") {
      const app = VIRTUAL_APPS.find((a) => a.id === "files")!;
      openAppWindow(app);
      return;
    }
    if (id === "trash") {
      const z = nextZ;
      setNextZ(z + 1);
      const win: DesktopWindow = {
        id: `win-trash-${Date.now()}`,
        title: "Trash",
        appId: "trash",
        x: 100,
        y: 80,
        width: 320,
        height: 240,
        zIndex: z,
        minimized: false,
      };
      openWindow(win);
    }
  }, [nextZ, openWindow, openAppWindow]);

  const handleFileDoubleClick = useCallback(async (winId: string, node: FileNode) => {
    if (node.is_dir) {
      loadFileBrowser(winId, node.path);
      return;
    }
    setPreviewLoading(true);
    try {
      const content = await api.readFile(node.path);
      setPreviewData({ path: node.path, content });
    } catch {
      setPreviewData({ path: node.path, content: "// Error reading file" });
    }
    setPreviewLoading(false);
  }, [loadFileBrowser]);

  const handleDockClick = useCallback((action: string) => {
    if (action === "terminal") { setShowTerminal(true); return; }
    if (action === "settings") { setShowSettings(true); return; }
    if (action === "files") {
      const app = VIRTUAL_APPS.find((a) => a.id === "files")!;
      openAppWindow(app);
    }
  }, [setShowTerminal, setShowSettings, openAppWindow]);

  const renderWindowContent = (win: DesktopWindow) => {
    if (win.appId === "trash") {
      return (
        <div className="window-content-empty">
          <span style={{ fontSize: 32 }}>🗑️</span>
          <p>Trash is empty</p>
        </div>
      );
    }
    if (win.appId === "agentmaker") {
      return (
        <div className="window-content-empty">
          <span style={{ fontSize: 32 }}>🤖</span>
          <p>Agent Maker</p>
        </div>
      );
    }
    if (win.appId === "files") {
      const fb = fileBrowsers[win.id];
      return (
        <div className="file-browser">
          <div className="file-browser-path">{fb ? fb.path : "Loading..."}</div>
          <div className="file-browser-list">
            {!fb ? (
              <div className="file-loading">Loading...</div>
            ) : fb.loading ? (
              <div className="file-loading">Loading...</div>
            ) : fb.nodes.length === 0 ? (
              <div className="file-empty">Empty directory</div>
            ) : (
              fb.nodes.map((node) => (
                <div
                  key={node.path}
                  className="file-browser-item"
                  onDoubleClick={() => handleFileDoubleClick(win.id, node)}
                >
                  <span className="file-browser-item-icon">
                    {node.is_dir ? "📁" : "📄"}
                  </span>
                  <span className="file-browser-item-name">{node.name}</span>
                  {!node.is_dir && node.size !== undefined && (
                    <span className="file-size">{formatSize(node.size)}</span>
                  )}
                </div>
              ))
            )}
          </div>
        </div>
      );
    }
    return (
      <div className="window-content-empty">
        <p>{win.title}</p>
      </div>
    );
  };

  return (
    <div className="virtual-os">
      <div className="desktop-wallpaper">
        <div className="desktop-icons">
          {DESKTOP_ICONS.map((ic) => (
            <DesktopIcon
              key={ic.id}
              name={ic.name}
              icon={ic.icon}
              selected={selectedIcon === ic.id}
              onSelect={() => setSelectedIcon(ic.id)}
              onOpen={() => handleDesktopIconOpen(ic.id)}
            />
          ))}
        </div>

        {desktopView === "applications" && (
          <div className="app-grid-overlay" onClick={() => setDesktopView("desktop")}>
            <div className="app-grid" onClick={(e) => e.stopPropagation()}>
              <div className="app-grid-header">
                <h2>Applications</h2>
                <button className="btn-icon" onClick={() => setDesktopView("desktop")}>
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
                  </svg>
                </button>
              </div>
              <div className="app-grid-body">
                {VIRTUAL_APPS.map((app) => (
                  <div
                    key={app.id}
                    className="app-grid-item"
                    onClick={() => { openAppWindow(app); setDesktopView("desktop"); }}
                  >
                    <div className="app-grid-item-icon">{app.icon}</div>
                    <div className="app-grid-item-name">{app.name}</div>
                    <div className="app-grid-item-desc">{app.description}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        <div className="desktop-windows">
          {windows.map((win) => !win.minimized && (
            <div
              key={win.id}
              className="desktop-window glass-panel"
              style={{
                left: win.x,
                top: win.y,
                width: win.width,
                height: win.height,
                zIndex: win.zIndex,
              }}
              onMouseDown={() => focusWindow(win.id)}
            >
              <div
                className="desktop-window-titlebar"
                onMouseDown={(e) => handleTitleMouseDown(e, win.id, win)}
              >
                <div className="desktop-window-title">{win.title}</div>
                <div className="desktop-window-controls">
                  <button
                    className="desktop-window-btn desktop-window-btn-minimize"
                    onClick={(e) => { e.stopPropagation(); toggleMinimizeWindow(win.id); }}
                  />
                  <button
                    className="desktop-window-btn desktop-window-btn-maximize"
                    onClick={(e) => { e.stopPropagation(); }}
                  />
                  <button
                    className="desktop-window-btn desktop-window-btn-close"
                    onClick={(e) => { e.stopPropagation(); closeWindow(win.id); }}
                  />
                </div>
              </div>
              <div className="desktop-window-content">
                {renderWindowContent(win)}
              </div>
            </div>
          ))}
        </div>

        {previewLoading && (
          <div className="file-preview-loading" style={{ position: "fixed", bottom: 80, left: "50%", transform: "translateX(-50%)", zIndex: 999, background: "rgba(0,0,0,0.7)", color: "#fff", padding: "8px 16px", borderRadius: 8, fontSize: 12 }}>
            Loading preview...
          </div>
        )}

        {previewData && (
          <FilePreviewOverlay
            path={previewData.path}
            content={previewData.content}
            onClose={() => setPreviewData(null)}
          />
        )}

        <div className="dock">
          {DOCK_ITEMS.map((item) => (
            <div
              key={item.id}
              className="dock-item"
              onClick={() => handleDockClick(item.action)}
              title={item.label}
            >
              <span className="dock-item-icon">{item.icon}</span>
              <span className="dock-item-label">{item.label}</span>
            </div>
          ))}
        </div>

        <div className="sandbox-badge">Sandbox Mode</div>
        <button
          className="virtual-os-exit"
          onClick={() => setVirtualOSActive(false)}
          title="Exit Virtual OS"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
            <path d="M10 1h2a1 1 0 011 1v10a1 1 0 01-1 1h-2M5 10l4-4-4-4M9 7H1" />
          </svg>
        </button>
      </div>
    </div>
  );
};

export default VirtualOS;
