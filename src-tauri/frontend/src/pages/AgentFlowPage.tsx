import React, { useState } from "react";

/* ───────── inline SVG icons ───────── */

const IconPlus = () => (
  <svg viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M6 2v8M2 6h8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
  </svg>
);

const IconEmpty = () => (
  <svg viewBox="0 0 36 36" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="8" y="6" width="20" height="24" rx="3" stroke="currentColor" strokeWidth="1.5" />
    <path d="M14 14h8M14 18h6M14 22h4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
    <circle cx="26" cy="26" r="6" fill="var(--nt-accent)" stroke="none" opacity="0.2" />
    <path d="M26 23v6M23 26h6" stroke="var(--nt-accent)" strokeWidth="1.5" strokeLinecap="round" />
  </svg>
);

const IconChat = () => (
  <svg viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M2 2h8a1 1 0 011 1v5a1 1 0 01-1 1H5l-3 2V3a1 1 0 011-1z" stroke="currentColor" strokeWidth="1.2" />
  </svg>
);

const IconSection = () => (
  <svg viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="1" y="1" width="10" height="3" rx="1" stroke="currentColor" strokeWidth="1.2" />
    <rect x="1" y="8" width="10" height="3" rx="1" stroke="currentColor" strokeWidth="1.2" />
  </svg>
);

/* ───────── types ───────── */

interface CoworkSession {
  id: string;
  label: string;
  subtitle: string;
}

interface TaskItem {
  id: string;
  name: string;
  status: "pri" | "suc" | "des";
  meta: string;
}

interface TaskSection {
  id: string;
  title: string;
  tasks: TaskItem[];
}

interface AgentItem {
  id: string;
  name: string;
  status: "idle" | "busy" | "done" | "error";
}

/* ───────── mock data ───────── */

const SESSIONS: CoworkSession[] = [
  { id: "s1", label: "Architecture Review", subtitle: "4 tasks · 2 agents" },
  { id: "s2", label: "Bug Bash Week 28", subtitle: "8 tasks · 3 agents" },
  { id: "s3", label: "Frontend Polish", subtitle: "5 tasks · 1 agent" },
  { id: "s4", label: "API Migration", subtitle: "3 tasks · 2 agents" },
];

const TASK_SECTIONS: TaskSection[] = [
  {
    id: "ts1",
    title: "In Progress",
    tasks: [
      { id: "t1", name: "Refactor auth middleware", status: "pri", meta: "2h" },
      { id: "t2", name: "Optimize KB query path", status: "pri", meta: "30m" },
      { id: "t3", name: "Add playground endpoint", status: "des", meta: "1h" },
    ],
  },
  {
    id: "ts2",
    title: "Review",
    tasks: [
      { id: "t4", name: "Update connector docs", status: "suc", meta: "PR #214" },
      { id: "t5", name: "Fix rate-limit edge case", status: "suc", meta: "PR #213" },
    ],
  },
  {
    id: "ts3",
    title: "Done",
    tasks: [
      { id: "t6", name: "Migrate to GatewayV2", status: "suc", meta: "merged" },
      { id: "t7", name: "Benchmark E8 inference", status: "suc", meta: "23ms avg" },
    ],
  },
];

const AGENTS: AgentItem[] = [
  { id: "a1", name: "Coder", status: "busy" },
  { id: "a2", name: "Reviewer", status: "idle" },
  { id: "a3", name: "Tester", status: "done" },
  { id: "a4", name: "Docs", status: "idle" },
];

/* ───────── dot color helper ───────── */

function dotClass(status: TaskItem["status"]): string {
  return status === "pri" ? "dot" : status === "suc" ? "dot done" : "dot fail";
}

function dotBg(status: AgentItem["status"]): string {
  switch (status) {
    case "busy": return "var(--nt-primary)";
    case "done": return "var(--nt-success)";
    case "error": return "var(--nt-danger)";
    default: return "var(--nt-text-muted)";
  }
}

/* ───────── SessionSidebar ───────── */

interface SessionSidebarProps {
  sessions: CoworkSession[];
  activeId: string | null;
  onSelect: (id: string) => void;
  onAdd: () => void;
}

const SessionSidebar: React.FC<SessionSidebarProps> = ({ sessions, activeId, onSelect, onAdd }) => (
  <div className="cw-sidebar">
    <div className="cw-shead">
      <span>Cowork Sessions</span>
      <button className="cw-add" onClick={onAdd} title="New session" aria-label="New session">
        <IconPlus />
      </button>
    </div>
    <div className="cw-slist">
      {sessions.map((s) => (
        <div
          key={s.id}
          className={`cw-sitem${s.id === activeId ? " active" : ""}`}
          onClick={() => onSelect(s.id)}
        >
          {s.label}
          <span className="s">{s.subtitle}</span>
        </div>
      ))}
    </div>
  </div>
);

/* ───────── EmptyState ───────── */

const EmptyState: React.FC = () => (
  <div className="cw-empty">
    <IconEmpty />
    <p>No session selected</p>
    <span>Choose a session from the sidebar to view details</span>
  </div>
);

/* ───────── TaskBoard ───────── */

const TaskBoard: React.FC<{ sections: TaskSection[] }> = ({ sections }) => (
  <>
    {sections.map((sec) => (
      <div key={sec.id}>
        <div className="cw-section-title">
          <IconSection />
          {sec.title}
        </div>
        <div className="cw-tlist">
          {sec.tasks.map((t) => (
            <div key={t.id} className="cw-task">
              <span className={dotClass(t.status)} />
              <span className="tname">{t.name}</span>
              <span className="tstat">{t.meta}</span>
            </div>
          ))}
        </div>
      </div>
    ))}
  </>
);

/* ───────── AgentGrid ───────── */

const AgentGrid: React.FC<{ agents: AgentItem[] }> = ({ agents }) => (
  <div>
    <div className="cw-section-title">
      <IconChat />
      Agents
    </div>
    <div className="cw-agents">
      {agents.map((a) => (
        <div key={a.id} className="cw-agent">
          <span className="adot" style={{ background: dotBg(a.status) }} />
          {a.name}
        </div>
      ))}
    </div>
  </div>
);

/* ───────── SessionContent ───────── */

interface SessionContentProps {
  session: CoworkSession;
}

const SessionContent: React.FC<SessionContentProps> = ({ session }) => (
  <div className="cw-content">
    <div className="cw-header">
      <div>
        <div className="cw-title">{session.label}</div>
        <div className="cw-sub">{session.subtitle}</div>
      </div>
      <span className="cw-hbadge">active</span>
    </div>
    <TaskBoard sections={TASK_SECTIONS} />
    <AgentGrid agents={AGENTS} />
  </div>
);

/* ───────── Page ───────── */

const AgentFlowPage: React.FC = () => {
  const [activeId, setActiveId] = useState<string | null>(null);

  const activeSession = activeId ? SESSIONS.find((s) => s.id === activeId) ?? null : null;

  const handleSelect = (id: string) => setActiveId(id);
  const handleAdd = () => {
    /* placeholder — in production this would create a new session */
  };

  return (
    <div className="cw-layout">
      <SessionSidebar
        sessions={SESSIONS}
        activeId={activeId}
        onSelect={handleSelect}
        onAdd={handleAdd}
      />
      <div className="cw-main">
        {activeSession ? <SessionContent session={activeSession} /> : <EmptyState />}
      </div>
    </div>
  );
};

export default AgentFlowPage;
