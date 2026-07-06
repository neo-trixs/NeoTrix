import React, { useState, Suspense } from "react";

const VirtualOS = React.lazy(() => import("../components/VirtualOS"));

function Lazy({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<div className="panel-loading" />}>{children}</Suspense>;
}

interface TreeNode {
  name: string;
  kind: "file" | "folder";
  children?: TreeNode[];
}

interface EditorTab {
  name: string;
  active: boolean;
}

const FILE_TREE: TreeNode[] = [
  {
    name: "src",
    kind: "folder",
    children: [
      { name: "App.tsx", kind: "file" },
      { name: "main.tsx", kind: "file" },
      { name: "vite-env.d.ts", kind: "file" },
    ],
  },
  {
    name: "components",
    kind: "folder",
    children: [
      { name: "ChatPanel.tsx", kind: "file" },
      { name: "FileTree.tsx", kind: "file" },
      { name: "StatusBar.tsx", kind: "file" },
    ],
  },
  {
    name: "pages",
    kind: "folder",
    children: [
      { name: "HomePage.tsx", kind: "file" },
      { name: "SettingsPage.tsx", kind: "file" },
      { name: "TerminalPage.tsx", kind: "file" },
    ],
  },
  {
    name: "styles",
    kind: "folder",
    children: [
      { name: "global.css", kind: "file" },
      { name: "design-tokens.css", kind: "file" },
    ],
  },
  {
    name: "index.html",
    kind: "file",
  },
  {
    name: "package.json",
    kind: "file",
  },
  {
    name: "tsconfig.json",
    kind: "file",
  },
];

function sp(cls: string, text: string): string {
  return '<span class="' + cls + '">' + text + "</span>";
}
const kw = (t: string) => sp("kw", t);
const fn = (t: string) => sp("fn", t);
const hl = (t: string) => sp("hl", t);
const cm = (t: string) => sp("cm", t);

const CODE_LINES: string[] = [
  "import " + kw("React") + ', { ' + fn("useState") + ", " + fn("useEffect") + ' } from ' + hl('"react"') + ";",
  "import { " + fn("useStore") + " } from " + hl('"../stores"') + ";",
  "import { " + fn("ChatPanel") + " } from " + hl('"../components/ChatPanel"') + ";",
  cm("// NeoTrix \u2014 AI-native developer toolkit"),
  "",
  kw("interface") + " " + fn("AppProps") + " {",
  "  projectPath?: " + kw("string") + ";",
  "  onNavigate?: (path: string) => " + kw("void") + ";",
  "}",
  "",
  kw("const") + " " + fn("App") + ": React.FC<AppProps> = ({ projectPath, onNavigate }) => {",
  "  const [activeSession, setActiveSession] = " + fn("useState") + "<" + kw("string") + " | " + kw("null") + ">(" + hl("null") + ");",
  "  const sessions = useStore((s) => s.sessions);",
  "  const messages = useStore((s) => s.messages);",
  "",
  "  useEffect(() => {",
  "    if (projectPath) {",
  "      console." + fn("log") + "(" + hl("`Mounted at ${projectPath}`") + ");",
  "    }",
  "  }, [projectPath]);",
  "",
  "  const handleSend = async (prompt: string) => {",
  "    " + fn("setLoading") + "(" + hl("true") + ");",
  "    try {",
  "      const response = await api.complete(prompt);",
  "      addMessage(response);",
  "    } catch (err) {",
  "      console.error(err);",
  "    } finally {",
  "      " + fn("setLoading") + "(" + hl("false") + ");",
  "    }",
  "  };",
  "",
  "  return (",
  '    <div className="app-layout">',
  "      <SidebarNav sessions={sessions} />",
  '      <main className="app-main">',
  '        <ChatPanel messages={messages} onSend={handleSend} />',
  "      </main>",
  "      <RightPanel />",
  "    </div>",
  "  );",
  "};",
  "",
  kw("export default") + " " + fn("App") + ";",
];

const FolderChevron: React.FC<{ open: boolean }> = ({ open }) => (
  <svg
    width="10"
    height="10"
    viewBox="0 0 10 10"
    fill="none"
    style={{
      transform: open ? "rotate(90deg)" : undefined,
      transition: "transform 0.15s ease",
    }}
  >
    <path d="M3 1.5l4 3.5-4 3.5" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
);

const FileIcon: React.FC = () => (
  <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
    <path d="M7 1H3a1 1 0 00-1 1v8a1 1 0 001 1h6a1 1 0 001-1V4L7 1z" stroke="currentColor" strokeWidth="1.2" />
  </svg>
);

const FolderIcon: React.FC = () => (
  <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
    <path d="M1 4.5a1 1 0 011-1h2.5l1.5-1.5H10a1 1 0 011 1v5.5a1 1 0 01-1 1H2a1 1 0 01-1-1v-5z" stroke="currentColor" strokeWidth="1.2" />
  </svg>
);

const TreeItem: React.FC<{
  node: TreeNode;
  depth: number;
  expanded: Set<string>;
  onToggle: (name: string) => void;
}> = ({ node, depth, expanded, onToggle }) => {
  const isFolder = node.kind === "folder";
  const isExpanded = expanded.has(node.name);

  return (
    <>
      <div
        className="cd-tree-item"
        style={{ paddingLeft: 12 + depth * 14 }}
        onClick={() => isFolder && onToggle(node.name)}
      >
        <span className="cd-tree-icon">
          {isFolder ? (
            <FolderChevron open={isExpanded} />
          ) : (
            <FileIcon />
          )}
        </span>
        <span className="cd-tree-label">{node.name}</span>
      </div>
      {isFolder && isExpanded && node.children?.map((child) => (
        <TreeItem
          key={child.name}
          node={child}
          depth={depth + 1}
          expanded={expanded}
          onToggle={onToggle}
        />
      ))}
    </>
  );
};

const PlusIcon: React.FC = () => (
  <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
    <path d="M6 2v8M2 6h8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
  </svg>
);

const UndoIcon: React.FC = () => (
  <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
    <path d="M4 7h6.5A2.5 2.5 0 0113 9.5v0A2.5 2.5 0 0110.5 12H7" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
    <path d="M6.5 9.5L4 7l2.5-2.5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
);

const RedoIcon: React.FC = () => (
  <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
    <path d="M12 7H5.5A2.5 2.5 0 003 9.5v0A2.5 2.5 0 005.5 12H9" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
    <path d="M9.5 9.5L12 7l-2.5-2.5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
);

const SaveIcon: React.FC = () => (
  <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
    <path d="M13 4.5V13a1 1 0 01-1 1H4a1 1 0 01-1-1V3a1 1 0 011-1h8.5L13 4.5z" stroke="currentColor" strokeWidth="1.3" />
    <path d="M5 14V9h6v5" stroke="currentColor" strokeWidth="1.3" />
  </svg>
);

const CutIcon: React.FC = () => (
  <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
    <circle cx="4.5" cy="4.5" r="2" stroke="currentColor" strokeWidth="1.3" />
    <circle cx="4.5" cy="11.5" r="2" stroke="currentColor" strokeWidth="1.3" />
    <path d="M6 5l6 7M12 4l-6 7" stroke="currentColor" strokeWidth="1.3" />
  </svg>
);

const CopyIcon: React.FC = () => (
  <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
    <rect x="5" y="5" width="9" height="10" rx="1" stroke="currentColor" strokeWidth="1.3" />
    <path d="M2 11V3a1 1 0 011-1h8" stroke="currentColor" strokeWidth="1.3" />
  </svg>
);

const PasteIcon: React.FC = () => (
  <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
    <path d="M5 3h6a1 1 0 011 1v10a1 1 0 01-1 1H5a1 1 0 01-1-1V4a1 1 0 011-1z" stroke="currentColor" strokeWidth="1.3" />
    <path d="M5.5 2V1a1 1 0 011-1h3a1 1 0 011 1v1" stroke="currentColor" strokeWidth="1.3" />
  </svg>
);

const IndentIcon: React.FC = () => (
  <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
    <path d="M3 4h10M3 8h6M3 12h10" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
    <path d="M7 6L9 8l-2 2" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
);

const OutdentIcon: React.FC = () => (
  <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
    <path d="M3 4h10M3 8h6M3 12h10" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
    <path d="M9 6L7 8l2 2" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
);

const SearchIcon: React.FC = () => (
  <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
    <circle cx="7" cy="7" r="4.5" stroke="currentColor" strokeWidth="1.3" />
    <path d="M10.5 10.5L14 14" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
  </svg>
);

const CloseIcon: React.FC = () => (
  <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
    <path d="M2 2l6 6M8 2l-6 6" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
  </svg>
);

function CodeView() {
  return (
    <div className="cd-view">
      <div className="cd-ln">
        {CODE_LINES.map((_, i) => (
          <div key={i}>{i + 1}</div>
        ))}
      </div>
      <div>
        {CODE_LINES.map((line, i) => (
          <div key={i} dangerouslySetInnerHTML={{ __html: line || "&nbsp;" }} />
        ))}
      </div>
    </div>
  );
}

const VirtualDesktopPage: React.FC = () => {
  const [mode, setMode] = useState<"desktop" | "code">("code");
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set(["src", "components"]));
  const [activeTab, setActiveTab] = useState<string>("App.tsx");

  const toggleFolder = (name: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  };

  if (mode === "desktop") {
    return (
      <div style={{ display: "flex", flex: 1, flexDirection: "column" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "4px 12px", borderBottom: "1px solid var(--nt-border)", fontSize: "var(--nt-font-size-caption)", color: "var(--nt-text-tertiary)" }}>
          <button
            onClick={() => setMode("code")}
            style={{ background: "none", border: "none", color: "var(--nt-primary)", cursor: "pointer", fontSize: "inherit", padding: 0 }}
          >
            Code View
          </button>
          <span>/</span>
          <span>Virtual Desktop</span>
        </div>
        <div style={{ flex: 1, display: "flex" }}>
          <Lazy><VirtualOS /></Lazy>
        </div>
      </div>
    );
  }

  const tabs: EditorTab[] = [
    { name: "App.tsx", active: activeTab === "App.tsx" },
    { name: "styles.css", active: activeTab === "styles.css" },
    { name: "utils.ts", active: activeTab === "utils.ts" },
  ];

  return (
    <div className="vw-code" style={{ display: "flex", flex: 1, overflow: "hidden" }}>
      <div style={{ display: "flex", flexDirection: "column", flex: 1 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "3px 12px", borderBottom: "1px solid var(--nt-border)", fontSize: "var(--nt-font-size-caption)", color: "var(--nt-text-tertiary)" }}>
          <span>Virtual Desktop</span>
          <span style={{ color: "var(--nt-text-muted)" }}>/</span>
          <button
            onClick={() => setMode("desktop")}
            style={{ background: "none", border: "none", color: "var(--nt-primary)", cursor: "pointer", fontSize: "inherit", padding: 0 }}
          >
            Desktop
          </button>
        </div>
        <div className="cd-layout">
          {/* ── File Tree ── */}
          <div className="cd-tree">
            <div className="cd-thead">
              <span>EXPLORER</span>
              <button className="cd-tbtn" title="New File">
                <PlusIcon />
              </button>
            </div>
            {FILE_TREE.map((node) => (
              <TreeItem
                key={node.name}
                node={node}
                depth={0}
                expanded={expandedFolders}
                onToggle={toggleFolder}
              />
            ))}
          </div>

          {/* ── Editor ── */}
          <div className="cd-editor">
            <div className="cd-tabs">
              {tabs.map((tab) => (
                <div
                  key={tab.name}
                  className={`cd-tab${tab.active ? " on" : ""}`}
                  onClick={() => setActiveTab(tab.name)}
                >
                  {tab.name}
                </div>
              ))}
            </div>

            <div className="cd-toolbar">
              <button className="cd-tb-btn" title="Undo"><UndoIcon /></button>
              <button className="cd-tb-btn" title="Redo"><RedoIcon /></button>
              <span style={{ width: 1, height: 16, background: "var(--nt-border)", margin: "0 2px" }} />
              <button className="cd-tb-btn" title="Save"><SaveIcon /></button>
              <button className="cd-tb-btn" title="Cut"><CutIcon /></button>
              <button className="cd-tb-btn" title="Copy"><CopyIcon /></button>
              <button className="cd-tb-btn" title="Paste"><PasteIcon /></button>
              <span style={{ width: 1, height: 16, background: "var(--nt-border)", margin: "0 2px" }} />
              <button className="cd-tb-btn" title="Indent"><IndentIcon /></button>
              <button className="cd-tb-btn" title="Outdent"><OutdentIcon /></button>
              <button className="cd-tb-btn" title="Search"><SearchIcon /></button>
              <span className="cd-lang">TypeScript</span>
            </div>

            <CodeView />
          </div>
        </div>
      </div>
    </div>
  );
};

export default VirtualDesktopPage;
