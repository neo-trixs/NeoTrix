import React, { Suspense, useCallback, useEffect, useRef, useState } from "react";
import { useStore } from "../stores";

const FileTree = React.lazy(() => import("./FileTree"));
const CodeEditor = React.lazy(() => import("./CodeEditor"));
const DiffViewer = React.lazy(() => import("./DiffViewer"));
const EvolutionPanel = React.lazy(() => import("./EvolutionPanel"));

type PanelTab = "files" | "editor" | "diff" | "evolution" | "artifact" | "tools";

interface PanelTabConfig {
  id: PanelTab;
  label: string;
  icon: string;
  condition: () => boolean;
}

const TABS: PanelTabConfig[] = [
  { id: "files", label: "Files", icon: "📁", condition: () => useStore.getState().showFileTree && !!useStore.getState().projectPath },
  { id: "editor", label: "Editor", icon: "✎", condition: () => useStore.getState().editorState.open },
  { id: "diff", label: "Diff", icon: "⇄", condition: () => !!useStore.getState().pendingDiff },
  { id: "evolution", label: "Evolution", icon: "📈", condition: () => useStore.getState().evolutionVisible },
  { id: "artifact", label: "Artifact", icon: "🖼", condition: () => true },
  { id: "tools", label: "Tools", icon: "🔧", condition: () => true },
];

const MIN_PANEL_WIDTH = 240;
const MAX_PANEL_WIDTH = 600;

const RightPanel: React.FC = () => {
  const [activeTab, setActiveTab] = useState<PanelTab>("files");

  const showFileTree = useStore((s) => s.showFileTree);
  const projectPath = useStore((s) => s.projectPath);
  const editorState = useStore((s) => s.editorState);
  const evolutionVisible = useStore((s) => s.evolutionVisible);
  const closeEditor = useStore((s) => s.closeEditor);
  const pendingDiff = useStore((s) => s.pendingDiff);
  const setPendingDiff = useStore((s) => s.setPendingDiff);
  const rightPanelWidth = useStore((s) => s.rightPanelWidth);
  const setRightPanelWidth = useStore((s) => s.setRightPanelWidth);

  const dragRef = useRef(false);
  const startXRef = useRef(0);
  const startWidthRef = useRef(0);

  const visibleTabs = TABS.filter((t) => t.condition());
  if (!visibleTabs.find((t) => t.id === activeTab) && visibleTabs.length > 0) {
    setActiveTab(visibleTabs[0].id);
  }

  const onMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    dragRef.current = true;
    startXRef.current = e.clientX;
    startWidthRef.current = rightPanelWidth;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }, [rightPanelWidth]);

  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      if (!dragRef.current) return;
      const newWidth = startWidthRef.current - (e.clientX - startXRef.current);
      setRightPanelWidth(Math.max(MIN_PANEL_WIDTH, Math.min(MAX_PANEL_WIDTH, newWidth)));
    };
    const onMouseUp = () => {
      if (!dragRef.current) return;
      dragRef.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
    return () => {
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };
  }, [setRightPanelWidth]);

  if (visibleTabs.length === 0) return null;

  return (
    <div className="right-panel" style={{ width: rightPanelWidth }}>
      <div className="right-panel-resize" onMouseDown={onMouseDown} />

      <div className="right-panel-tabs">
        {visibleTabs.map((tab) => (
          <button
            key={tab.id}
            className={`right-panel-tab ${activeTab === tab.id ? "active" : ""}`}
            onClick={() => setActiveTab(tab.id)}
          >
            <span>{tab.icon}</span>
            <span>{tab.label}</span>
          </button>
        ))}
        {(activeTab === "editor" || activeTab === "diff") && (
          <button
            className="right-panel-close"
            onClick={() => {
              if (activeTab === "editor") closeEditor();
              if (activeTab === "diff") setPendingDiff(null);
            }}
          >✕</button>
        )}
      </div>

      <div className="right-panel-content">
        <Suspense fallback={<div className="right-panel-loading" />}>
          {activeTab === "files" && showFileTree && projectPath && (
            <FileTree rootPath={projectPath} onClose={() => {}} onStatusChange={() => {}} />
          )}
          {activeTab === "editor" && editorState.open && (
            <CodeEditor
              filePath={editorState.filePath}
              initialContent={editorState.initialContent}
              language={editorState.language}
              onClose={closeEditor}
              onSave={() => {}}
            />
          )}
          {activeTab === "diff" && pendingDiff && (
            <DiffViewer
              diffBlocks={pendingDiff.blocks}
              filename={pendingDiff.filename}
              onApply={() => { setPendingDiff(null); }}
              onReject={() => { setPendingDiff(null); }}
            />
          )}
          {activeTab === "evolution" && evolutionVisible && <EvolutionPanel />}
          {activeTab === "artifact" && <ArtifactView />}
          {activeTab === "tools" && <ToolsView />}
        </Suspense>
      </div>
    </div>
  );
};

function ArtifactView() {
  const [view, setView] = useState<"code" | "render">("code");
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      <div style={{ display: "flex", gap: 4, alignItems: "center" }}>
        <span style={{ fontSize: 11, fontWeight: 600, flex: 1 }}>Preview</span>
        <button
          onClick={() => setView("code")}
          style={{
            padding: "2px 6px", fontSize: 9, border: "none", borderRadius: 3,
            background: view === "code" ? "var(--nt-active, rgba(0,122,255,0.12))" : "none",
            color: "var(--nt-text)", cursor: "pointer", fontFamily: "inherit",
          }}
        >Code</button>
        <button
          onClick={() => setView("render")}
          style={{
            padding: "2px 6px", fontSize: 9, border: "none", borderRadius: 3,
            background: view === "render" ? "var(--nt-active, rgba(0,122,255,0.12))" : "none",
            color: "var(--nt-text)", cursor: "pointer", fontFamily: "inherit",
          }}
        >Render</button>
      </div>
      <div style={{
        background: "var(--nt-canvas, #FAF8F4)", borderRadius: 6, padding: 10,
        fontSize: 10, fontFamily: "'SF Mono', Menlo, monospace", lineHeight: 1.5,
        overflowX: "auto", minHeight: 100,
      }}>
        {view === "code" ? (
          <pre style={{ margin: 0 }}>{`// session.rs
fn process() {
    let ctx = E8::ground();
    ctx.reason("hello");
}`}</pre>
        ) : (
          <div style={{ color: "var(--nt-text)" }}>
            <strong>Rendered:</strong> &ldquo;NeoTrix consciousness engaged&rdquo;
          </div>
        )}
      </div>
    </div>
  );
}

function ToolsView() {
  const tools = [
    { name: "filesystem", status: "connected" as const, count: 3 },
    { name: "brave-search", status: "disconnected" as const, count: 1 },
    { name: "github", status: "disconnected" as const, count: 5 },
  ];
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
      <div style={{ fontSize: 11, fontWeight: 600, marginBottom: 4 }}>MCP Tools</div>
      {tools.map((t) => (
        <div key={t.name} style={{
          display: "flex", alignItems: "center", gap: 6,
          padding: "4px 6px", borderRadius: 4,
          background: "var(--nt-hover, rgba(0,0,0,0.04))",
        }}>
          <span style={{
            width: 6, height: 6, borderRadius: "50%", flexShrink: 0,
            background: t.status === "connected" ? "var(--nt-success, #34c759)" : "var(--nt-text-muted, #aeaeb2)",
          }} />
          <span style={{ flex: 1, fontSize: 11 }}>{t.name}</span>
          <span style={{ fontSize: 9, color: "var(--nt-text-muted)" }}>{t.count} tools</span>
        </div>
      ))}
    </div>
  );
}

export default RightPanel;