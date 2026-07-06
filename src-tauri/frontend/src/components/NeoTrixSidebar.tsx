import React from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useStore } from "../stores";
import "./NeoTrixSidebar.css";

const TRAFFIC_ICONS = {
  chat: `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2.5H3a1.5 1.5 0 00-1.5 1.5v5A1.5 1.5 0 003 10.5h2.5l2 3 2-3H13a1.5 1.5 0 001.5-1.5V4A1.5 1.5 0 0013 2.5z"/></svg>`,
  cowork: `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="5" r="2"/><circle cx="11" cy="5" r="2"/><path d="M6 9c-2.5 0-4 1.5-4 3.5v1h8v-1c0-2-1.5-3.5-4-3.5z"/><path d="M11 9c2.5 0 4 1.5 4 3.5v1h-3"/></svg>`,
  code: `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M5.5 11L2 8l3.5-3M10.5 5L14 8l-3.5 3"/></svg>`,
  agent: `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><circle cx="8" cy="6" r="2.5"/><path d="M4 13.5c0-2.5 2-4.5 4-4.5s4 2 4 4.5"/><path d="M2 3L4 4M14 3L12 4M8 1v2"/></svg>`,
  explore: `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><circle cx="8" cy="8" r="6"/><path d="M8 5v3l2 2"/></svg>`,
  graph: `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><circle cx="5" cy="5" r="2"/><circle cx="11" cy="5" r="2"/><circle cx="8" cy="11" r="2"/><path d="M5 7v2a2 2 0 003 1.5M11 7v2a2 2 0 01-3 1.5M5 7l3-2M11 7l-3-2"/></svg>`,
};

const MODE_ICONS = {
  chat: `<svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 2H3a1 1 0 00-1 1v5a1 1 0 001 1h1.5l1.5 2 1.5-2H11a1 1 0 001-1V3a1 1 0 00-1-1z"/></svg>`,
  focus: `<svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"><circle cx="7" cy="5.5" r="3.5"/><path d="M7 1v1M7 12v1M1 7h1M12 7h1"/></svg>`,
  flow: `<svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 4h3l1.5 6L8 4h3"/></svg>`,
};

const NAV_ITEMS = [
  { id: "chat", label: "Chat", icon: TRAFFIC_ICONS.chat, route: "/" },
  { id: "cowork", label: "Cowork", icon: TRAFFIC_ICONS.cowork, route: "/agent-flow" },
  { id: "code", label: "Code", icon: TRAFFIC_ICONS.code, route: "/desktop" },
  { id: "agent", label: "Agent", icon: TRAFFIC_ICONS.agent, route: "/agents" },
  { id: "explore", label: "Explore", icon: TRAFFIC_ICONS.explore, route: "/explore" },
  { id: "graph", label: "Graph", icon: TRAFFIC_ICONS.graph, route: "/knowledge-graph" },
];

const MODE_ITEMS = [
  { id: "chat", icon: MODE_ICONS.chat, label: "Chat" },
  { id: "focus", icon: MODE_ICONS.focus, label: "Focus" },
  { id: "flow", icon: MODE_ICONS.flow, label: "Flow" },
];

const RECENT_SESSIONS = [
  { name: "API design review", id: "r1" },
  { name: "Debug memory leak", id: "r2" },
  { name: "Knowledge absorption", id: "r3", ghost: true },
  { name: "Code architecture audit", id: "r4" },
];

const NeoTrixSidebar: React.FC = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const sessions = useStore((s) => s.sessions);
  const activeSessionIndex = useStore((s) => s.activeSessionIndex);
  const setActiveSessionIndex = useStore((s) => s.setActiveSessionIndex);
  const addSession = useStore((s) => s.addSession);
  const consciousnessActive = useStore((s) => s.consciousnessActive);
  const setConsciousnessActive = useStore((s) => s.setConsciousnessActive);
  const setShowCommandPalette = useStore((s) => s.setShowCommandPalette);
  const setShowSearch = useStore((s) => s.setShowSearch);

  const currentRoute = location.pathname;
  const activeNav = NAV_ITEMS.find((n) => n.route === currentRoute)?.id
    || (currentRoute.startsWith("/agent") ? "agent" : "chat");

  return (
    <div className="nt-sidebar">
      <div className="nt-sidebar-top">
        {/* Traffic lights */}
        <div className="nt-traffic">
          <div className="nt-traffic-dot nt-traffic-r" />
          <div className="nt-traffic-dot nt-traffic-y" />
          <div className="nt-traffic-dot nt-traffic-g" />
        </div>

        {/* Header actions */}
        <div className="nt-sb-header-actions">
          <button className="nt-sb-btn" onClick={() => addSession()} title="New session">
            <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><path d="M7 2v10M2 7h10"/></svg>
          </button>
          <button className="nt-sb-btn" onClick={() => navigate("/chat")} title="Switch chat">
            <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M9 4L7 2 5 4M7 2v6"/><path d="M2 8v2.5a1.5 1.5 0 001.5 1.5h7A1.5 1.5 0 0012 10.5V8"/></svg>
          </button>
        </div>

        {/* Segmented mode tabs */}
        <div className="nt-seg">
          {MODE_ITEMS.map((mode) => (
            <button
              key={mode.id}
              className={`nt-segb ${consciousnessActive && mode.id === "chat" ? "on" : ""}`}
              onClick={() => setConsciousnessActive(mode.id === "focus" ? !consciousnessActive : true)}
              title={mode.label}
            >
              <span dangerouslySetInnerHTML={{ __html: mode.icon }} />
            </button>
          ))}
        </div>

        {/* Nav list */}
        <div className="nt-nav-list">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.id}
              className={`nt-nav-item ${activeNav === item.id ? "on" : ""}`}
              onClick={() => navigate(item.route)}
            >
              <span dangerouslySetInnerHTML={{ __html: item.icon }} />
              <span className="nt-nav-label">{item.label}</span>
            </button>
          ))}
        </div>

        {/* Recents */}
        <div className="nt-recents">
          <div className="nt-recents-header">Recents</div>
          {sessions.slice(-6).reverse().map((session, i) => (
            <div
              key={session.id}
              className={`nt-recent-item ${i >= 3 ? "ghost" : ""}`}
              onClick={() => {
                const idx = sessions.findIndex((s) => s.id === session.id);
                if (idx >= 0) setActiveSessionIndex(idx);
              }}
            >
              <div className="nt-recent-dot" />
              <span className="nt-recent-name">{session.name}</span>
            </div>
          ))}
          {sessions.length === 0 && (
            <div className="nt-recent-item ghost">
              <div className="nt-recent-circle" />
              <span className="nt-recent-name">No conversations</span>
            </div>
          )}
        </div>
      </div>

      {/* User bar at bottom */}
      <div className="nt-user-bar">
        <div className="nt-avatar">N</div>
        <div className="nt-user-info">
          <div className="nt-user-name">NeoTrix</div>
          <div className="nt-user-plan">Consciousness OS</div>
        </div>
        <div className="nt-user-actions">
          <button className="nt-sf-btn" onClick={() => setShowSearch(true)} title="Search (⌘F)">
            <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><circle cx="6.5" cy="6.5" r="3.5"/><path d="M9 9l3.5 3.5"/></svg>
          </button>
          <button className="nt-sf-btn" onClick={() => setShowCommandPalette(true)} title="Commands (⌘K)">
            <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><path d="M5 2v10M9 2v10M2 5h10M2 9h10"/></svg>
          </button>
          <button className="nt-sf-btn" onClick={() => navigate("/settings")} title="Settings (⌘,)">
            <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><circle cx="7" cy="7" r="2"/><path d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13M2.5 2.5l1 1M10.5 10.5l1 1M2.5 11.5l1-1M10.5 3.5l1-1"/></svg>
          </button>
        </div>
      </div>
    </div>
  );
};

export default NeoTrixSidebar;
