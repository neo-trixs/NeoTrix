import { useState, useCallback, useRef, useEffect, createContext, useContext } from 'react';
import { useApp } from './hooks/useApp';
import { useKeyboard } from './hooks/useKeyboard';

import './App.css';
import ReasoningView from './components/ReasoningView';
import ExperienceTreeView from './components/ExperienceTreeView';
import SystemDashboard from './components/SystemDashboard';
import CommandPalette from './components/CommandPalette';
import SessionSidebar from './components/SessionSidebar';
import SpaceView from './components/SpaceView';
import CodeMapView from './components/CodeMapView';
import UniversalChat from './components/UniversalChat';
import DockPanel from './components/DockPanel';
import CausalGraph from './components/CausalGraph';
import PlanMode from './components/PlanMode';
import WorkflowCanvas from './components/WorkflowCanvas';
import UnifiedContactHub from './components/UnifiedContactHub';
import ContextInspector from './components/ContextInspector';
import SessionTree from './components/SessionTree';
import MemoryGraph from './components/MemoryGraph';
import { getSessions, getSession } from './core/session-manager';
import { layoutEngine } from './core/layout-engine';
import { socialIdentity } from './core/social-identity';

type SideContext = 'sessions' | 'knowledge' | 'system' | 'spaces' | 'codemap' | 'plan' | 'workflow' | 'contacts' | 'context';
type ViewMode = 'verbose' | 'normal' | 'summary';

interface AppCtx {
  sideContext: SideContext;
  setSideContext: (v: SideContext) => void;
  panelOpen: boolean;
  setPanelOpen: (v: boolean) => void;
  rightPanelOpen: boolean;
  setRightPanelOpen: (v: boolean) => void;
  isGenerating: boolean;
  setIsGenerating: (v: boolean) => void;
  activeSessionId: string;
  setActiveSessionId: (v: string) => void;
  viewMode: ViewMode;
  setViewMode: (v: ViewMode) => void;
  sideChatOpen: boolean;
  setSideChatOpen: (v: boolean) => void;
  toggleSidebar: () => void;
  toggleRightPanel: () => void;
  setLayout: (layout: 'default' | 'compact' | 'wide') => void;
  setCurrentSession: (session: string) => void;
}
export const AppContext = createContext<AppCtx>({
  sideContext: 'sessions', setSideContext: () => {},
  panelOpen: false, setPanelOpen: () => {},
  rightPanelOpen: false, setRightPanelOpen: () => {},
  isGenerating: false, setIsGenerating: () => {},
  activeSessionId: '', setActiveSessionId: () => {},
  viewMode: 'normal', setViewMode: () => {},
  sideChatOpen: false, setSideChatOpen: () => {},
  toggleSidebar: () => {},
  toggleRightPanel: () => {},
  setLayout: () => {},
  setCurrentSession: () => {},
});
export const useAppCtx = () => useContext(AppContext);

export default function App() {
  const { state, setTheme, toggleSidebar, toggleRightPanel, setLayout, setCurrentSession } = useApp();
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [panelOpen, setPanelOpen] = useState(false);
  const [rightPanelOpen, setRightPanelOpen] = useState(false);
  const [sideContext, setSideContext] = useState<SideContext>('sessions');
  const [isGenerating, setIsGenerating] = useState(false);
  const [activeSessionId, setActiveSessionId] = useState(() => getSessions()[0]?.id || 'ses_0001');
  const [viewMode, setViewMode] = useState<ViewMode>('normal');
  const [sideChatOpen, setSideChatOpen] = useState(false);
  const [sessionView, setSessionView] = useState<'list' | 'tree'>('list');
  const [socialOnline, setSocialOnline] = useState(0);
  const [socialUnread, setSocialUnread] = useState(0);
  const panelRef = useRef<HTMLDivElement>(null);

  // close panel on backdrop click
  const handleBackdropClick = useCallback(() => {
    setPanelOpen(false);
  }, []);

  // Track social metrics
  useEffect(() => {
    const update = () => {
      setSocialOnline(socialIdentity.contacts.filter(c => c.online).length);
      const totalUnread = socialIdentity.contacts.reduce((s, c) => s + (c.unread ?? 0), 0);
      setSocialUnread(totalUnread);
    };
    update();
    const iv = setInterval(update, 5000);
    return () => clearInterval(iv);
  }, []);

  // Sync sessions → contacts layout engine
  useEffect(() => {
    layoutEngine.syncFromSessions(getSessions().map(s => ({
      id: s.id, label: s.label, status: s.status,
    })));
  }, []);

  // click outside → close
  useEffect(() => {
    if (!panelOpen) return;
    const onDocKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setPanelOpen(false);
    };
    document.addEventListener('keydown', onDocKey);
    return () => document.removeEventListener('keydown', onDocKey);
  }, [panelOpen]);

  useKeyboard([
    { key: 'k', metaKey: true, handler: () => setPaletteOpen(v => !v) },
    { key: '1', metaKey: true, handler: () => { setSideContext('sessions'); setPanelOpen(true); } },
    { key: '2', metaKey: true, handler: () => { setSideContext('knowledge'); setPanelOpen(true); } },
    { key: '3', metaKey: true, handler: () => { setSideContext('system'); setPanelOpen(true); } },
    { key: '4', metaKey: true, handler: () => { setSideContext('spaces'); setPanelOpen(true); } },
    { key: '5', metaKey: true, handler: () => { setSideContext('codemap'); setPanelOpen(true); } },
    { key: '6', metaKey: true, handler: () => { setSideContext('plan'); setPanelOpen(true); } },
    { key: '7', metaKey: true, handler: () => { setSideContext('workflow'); setPanelOpen(true); } },
    { key: '8', metaKey: true, handler: () => { setSideContext('contacts'); setPanelOpen(true); } },
    { key: '9', metaKey: true, handler: () => { setSideContext('context'); setPanelOpen(true); } },
    { key: 'b', metaKey: true, handler: () => setPanelOpen(v => !v) },
    { key: ';', metaKey: true, handler: () => setSideChatOpen(v => !v) },
  ]);

  const contextLabel = sideContext === 'sessions' ? 'Sessions'
    : sideContext === 'knowledge' ? 'Knowledge'
    : sideContext === 'system' ? 'System'
    : sideContext === 'spaces' ? 'Spaces'
    : sideContext === 'codemap' ? 'Code Map'
    : sideContext === 'plan' ? 'Plan'
    : sideContext === 'workflow' ? 'Flow'
    : sideContext === 'contacts' ? 'Contacts'
    : sideContext === 'context' ? 'Context'
    : 'Panel';

  const paletteCommands = [
    { id: 'sessions', label: 'Sessions', shortcut: '⌘1',
      icon: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/></svg>,
      action: () => { setSideContext('sessions'); setPanelOpen(true); } },
    { id: 'knowledge', label: 'Knowledge', shortcut: '⌘2',
      icon: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>,
      action: () => { setSideContext('knowledge'); setPanelOpen(true); } },
    { id: 'system', label: 'System', shortcut: '⌘3',
      icon: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>,
      action: () => { setSideContext('system'); setPanelOpen(true); } },
    { id: 'spaces', label: 'Spaces', shortcut: '⌘4',
      icon: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="21" x2="9" y2="9"/></svg>,
      action: () => { setSideContext('spaces'); setPanelOpen(true); } },
    { id: 'codemap', label: 'Code Map', shortcut: '⌘5',
      icon: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>,
      action: () => { setSideContext('codemap'); setPanelOpen(true); } },
    { id: 'plan', label: 'Plan Mode', shortcut: '⌘6',
      icon: <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>,
      action: () => { setSideContext('plan'); setPanelOpen(true); } },
    {
      id: 'toggle-theme', label: state.currentTheme === 'dark' ? 'Light Mode' : 'Dark Mode',
      icon: state.currentTheme === 'dark'
        ? <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/></svg>
        : <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>,
      action: () => { setTheme(state.currentTheme === 'dark' ? 'light' : 'dark'); },
    },
  ];

  return (
      <AppContext.Provider value={{ sideContext, setSideContext, panelOpen, setPanelOpen, rightPanelOpen, setRightPanelOpen, isGenerating, setIsGenerating, activeSessionId, setActiveSessionId, viewMode, setViewMode, sideChatOpen, setSideChatOpen, toggleSidebar: () => { toggleSidebar(); setPanelOpen(v => !v); }, toggleRightPanel: () => { toggleRightPanel(); setRightPanelOpen(v => !v); }, setLayout, setCurrentSession: (id: string) => { setCurrentSession(id); setActiveSessionId(id); } }}>
      <div className="app-container">

        {/* ===== Top Bar (ultra minimal) ===== */}
        <header className="top-bar">
          <div className="top-brand">
            <span className="top-brand-dot" />
            NeoTrix
          </div>
          <div className="top-spacer" />
          <div className="top-view-modes">
            {(['normal', 'verbose', 'summary'] as const).map(v => (
              <button
                key={v}
                className={`top-view-btn ${viewMode === v ? 'active' : ''}`}
                onClick={() => setViewMode(v)}
                title={v.charAt(0).toUpperCase() + v.slice(1)}
              >
                {v === 'normal' ? 'N' : v === 'verbose' ? 'V' : 'S'}
              </button>
            ))}
          </div>
          <button className="top-action" onClick={() => setPanelOpen(v => !v)} title="Panel (⌘B)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="7" height="7" rx="1"/>
              <rect x="14" y="3" width="7" height="7" rx="1"/>
              <rect x="14" y="14" width="7" height="7" rx="1"/>
              <rect x="3" y="14" width="7" height="7" rx="1"/>
            </svg>
          </button>
          <button className="top-action" onClick={() => setRightPanelOpen(v => !v)} title="Right Panel">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="18" height="18" rx="2"/>
              <line x1="15" y1="3" x2="15" y2="21"/>
            </svg>
          </button>
          <button className="top-action" onClick={() => setPaletteOpen(v => !v)} title="Commands (⌘K)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
            </svg>
          </button>
        </header>

        {/* ===== Main ===== */}
        <div className="main-area">

          {/* Floating panel overlay */}
          <div className={`panel-overlay ${panelOpen ? 'active' : ''}`}>
            <div className="panel-backdrop" onClick={handleBackdropClick} />
            <div className="panel-container" ref={panelRef}>
              <div className="panel-header">
                <span className="panel-title">{contextLabel}</span>
                <div className="panel-close" onClick={() => setPanelOpen(false)}>
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
                  </svg>
                </div>
              </div>
              <div className="panel-body">
                {sideContext === 'sessions' && (
                  <>
                    <div className="panel-tabs">
                      <button className={`panel-tab ${sessionView === 'list' ? 'active' : ''}`}
                        onClick={() => setSessionView('list')}>List</button>
                      <button className={`panel-tab ${sessionView === 'tree' ? 'active' : ''}`}
                        onClick={() => setSessionView('tree')}>Tree</button>
                    </div>
                    {sessionView === 'list' ? (
                      <SessionSidebar
                        activeId={activeSessionId}
                        onSelect={(id) => { setActiveSessionId(id); }}
                      />
                    ) : (
                      <SessionTree />
                    )}
                  </>
                )}
                {sideContext === 'knowledge' && <ExperienceTreeView compact />}
                {sideContext === 'system' && <SystemDashboard compact />}
                {sideContext === 'spaces' && (
                  <SpaceView
                    activeSessionId={activeSessionId}
                    onSelect={(id) => { setActiveSessionId(id); setSideContext('sessions'); }}
                  />
                )}
                {sideContext === 'codemap' && <CodeMapView />}
                {sideContext === 'plan' && (
                  <PlanMode
                    sessionId={activeSessionId}
                    onClose={() => setPanelOpen(false)}
                  />
                )}
                {sideContext === 'workflow' && <WorkflowCanvas />}
                {sideContext === 'contacts' && <UnifiedContactHub />}
                {sideContext === 'context' && <ContextInspector />}
              </div>
            </div>
          </div>

          {/* Main content — reasoning + dock panels */}
          <ReasoningView />
          <DockPanel
            side="right"
            items={[
              { id: 'graph', label: 'Graph', defaultOpen: true,
                icon: <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><circle cx="19" cy="5" r="2"/><circle cx="5" cy="19" r="2"/><line x1="12" y1="12" x2="19" y2="5"/><line x1="12" y1="12" x2="5" y2="19"/></svg>,
                panel: <CausalGraph sessionId={activeSessionId} />,
              },
              { id: 'memory', label: 'Memory', defaultOpen: false,
                icon: <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><circle cx="19" cy="5" r="2"/><circle cx="5" cy="19" r="2"/><line x1="12" y1="12" x2="19" y2="5"/><line x1="12" y1="12" x2="5" y2="19"/></svg>,
                panel: <MemoryGraph />,
              },
            ]}
          />
          <UniversalChat />
        </div>

        {/* ===== Status Bar ===== */}
        <footer className="status-bar">
          <span className="status-item">
            <span className="status-dot" style={{ background: '#5e8aff' }} />
            C(t) <span className="status-val">68%</span>
          </span>
          <span className="status-sep" />
          <span className="status-item">
            <span className="status-dot" style={{ background: '#ff6cb4' }} />
            Φ <span className="status-val">3.42</span>
          </span>
          <span className="status-sep" />
          <span className="status-item">
            <span className="status-dot" style={{ background: '#48b87a' }} />
            Coh <span className="status-val">72%</span>
          </span>
          <span className="status-sep" />
          <span className="status-item">
            <span className="status-dot" style={{ background: '#c86cff' }} />
            VSA <span className="status-val">4.2K/s</span>
          </span>
          <span className="status-sep" />
          <span className="status-item" title="Context usage">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ opacity: 0.4 }}>
              <circle cx="12" cy="12" r="10" />
              <circle cx="12" cy="12" r="5" />
              <line x1="12" y1="2" x2="12" y2="6" />
              <line x1="12" y1="18" x2="12" y2="22" />
              <line x1="4.93" y1="4.93" x2="7.76" y2="7.76" />
              <line x1="16.24" y1="16.24" x2="19.07" y2="19.07" />
            </svg>
            <span className="status-val" style={{ fontSize: 9 }}>62%</span>
          </span>
          <span className="status-sep" />
          <span className="status-item" title="Contacts online">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ opacity: 0.4 }}>
              <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/>
              <path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>
            </svg>
            <span className="status-val" style={{ color: socialOnline > 0 ? '#48b87a' : undefined }}>
              {socialOnline}
            </span>
          </span>
          <span className="status-sep" />
          <span className="status-item" title="Unread DMs" style={socialUnread > 0 ? { cursor: 'pointer' } : {}} onClick={() => { setPanelOpen(true); setSideContext('contacts'); }}>
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ opacity: 0.4 }}>
              <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
            </svg>
            {socialUnread > 0 ? (
              <span className="status-val" style={{ color: '#e04e4e' }}>{socialUnread}</span>
            ) : (
              <span className="status-val">0</span>
            )}
          </span>
          <span className="status-sep" />
          <span className="status-item" title="Social identity">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ opacity: 0.4 }}>
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>
            </svg>
            <span className="status-val" style={{ fontSize: 8, fontFamily: 'var(--font-mono)', opacity: 0.6 }}>
              {socialIdentity.shortId.slice(0, 8)}
            </span>
          </span>
          <span className="status-spacer" />
          {activeSessionId && (() => {
            const s = getSession(activeSessionId);
            if (!s) return null;
            return (
              <>
                <span className="status-item">
                  <span className="status-dot" style={{
                    background: s.status === 'running' ? '#48b87a' : s.status === 'paused' ? '#e8a030' : 'transparent',
                  }} />
                  {s.label}
                </span>
                {s.status === 'running' && (
                  <>
                    <span className="status-sep" />
                    <span className="status-item">
                      step <span className="status-val">{s.currentStep}</span>
                    </span>
                  </>
                )}
              </>
            );
          })()}
          <span className="status-sep" />
          <span className="status-item">
            {contextLabel}
          </span>
        </footer>

        <CommandPalette
          open={paletteOpen}
          onClose={() => setPaletteOpen(false)}
          commands={paletteCommands}
        />
      </div>
    </AppContext.Provider>
  );
}
