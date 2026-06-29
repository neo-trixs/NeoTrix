const CYCLE_STEPS = [
  'GATHER', 'REFLECT', 'REASON', 'PLAN',
  'ACT', 'RECORD', 'METRIC', 'EVOLVE',
  'SLEEP', 'META', 'VETO', 'AWAKEN',
] as const;

export type CycleStep = typeof CYCLE_STEPS[number];

// ===== Spaces (Devin Desktop-inspired) =====
export interface Space {
  id: string;
  label: string;
  sessionIds: string[];
  prUrls: string[];
  files: string[];
  createdAt: number;
}

export interface SessionState {
  id: string;
  label: string;
  status: 'idle' | 'running' | 'paused' | 'completed' | 'error';
  currentStep: CycleStep;
  progress: number;
  memoryUsage: number;
  startedAt: number;
  subsystemCount: number;
  activeSubsystems: number;
  spaceId?: string;
  plan?: PlanStep[];
}

export interface PlanStep {
  id: string;
  action: string;
  detail: string;
  status: 'pending' | 'approved' | 'executing' | 'done' | 'rejected';
}

let nextId = 1;
let nextSpaceId = 1;

function createNewSession(label: string): SessionState {
  const id = `ses_${String(nextId++).padStart(4, '0')}`;
  return {
    id,
    label,
    status: 'idle',
    currentStep: 'GATHER',
    progress: 0,
    memoryUsage: 0,
    startedAt: Date.now(),
    subsystemCount: 56,
    activeSubsystems: 56,
  };
}

const sessions: SessionState[] = [
  createNewSession('Main Cycle'),
  createNewSession('Research'),
  createNewSession('Self-Evolution'),
  createNewSession('Memory Consolidation'),
];

const spaces: Space[] = [
  { id: 'spc_001', label: 'Core Architecture', sessionIds: [sessions[0].id, sessions[2].id], prUrls: ['#42', '#43'], files: [], createdAt: Date.now() - 3600000 },
  { id: 'spc_002', label: 'Knowledge Pipeline', sessionIds: [sessions[1].id, sessions[3].id], prUrls: ['#41'], files: [], createdAt: Date.now() - 1800000 },
];

// Assign sessions to spaces
sessions[0].spaceId = 'spc_001';
sessions[2].spaceId = 'spc_001';
sessions[1].spaceId = 'spc_002';
sessions[3].spaceId = 'spc_002';

const stepColors: Record<CycleStep, string> = {
  GATHER: '#6c8cff',
  REFLECT: '#8a6cff',
  REASON: '#ff6cb4',
  PLAN: '#ffb86c',
  ACT: '#6cffaa',
  RECORD: '#6ccfff',
  METRIC: '#ff6c6c',
  EVOLVE: '#c86cff',
  SLEEP: '#6c6cff',
  META: '#ff9a6c',
  VETO: '#ff6c8a',
  AWAKEN: '#6cffd4',
};

export function getSessions(): SessionState[] {
  return sessions;
}

export function getSession(id: string): SessionState | undefined {
  return sessions.find(s => s.id === id);
}

export function createSession(label: string): SessionState {
  const s = createNewSession(label);
  sessions.push(s);
  return s;
}

// ===== Space Management =====
export function getSpaces(): Space[] {
  return spaces;
}

export function getSpace(id: string): Space | undefined {
  return spaces.find(s => s.id === id);
}

export function createSpace(label: string, sessionIds: string[] = []): Space {
  const id = `spc_${String(nextSpaceId++).padStart(3, '0')}`;
  const space: Space = { id, label, sessionIds, prUrls: [], files: [], createdAt: Date.now() };
  spaces.push(space);
  for (const sid of sessionIds) {
    const s = sessions.find(ss => ss.id === sid);
    if (s) s.spaceId = id;
  }
  return space;
}

export function addSessionToSpace(spaceId: string, sessionId: string): void {
  const space = spaces.find(s => s.id === spaceId);
  const session = sessions.find(s => s.id === sessionId);
  if (space && session && !space.sessionIds.includes(sessionId)) {
    space.sessionIds.push(sessionId);
    session.spaceId = spaceId;
  }
}

export function removeSessionFromSpace(sessionId: string): void {
  const session = sessions.find(s => s.id === sessionId);
  if (session?.spaceId) {
    const space = spaces.find(s => s.id === session.spaceId);
    if (space) space.sessionIds = space.sessionIds.filter(id => id !== sessionId);
    session.spaceId = undefined;
  }
}

// ===== Plan Mode (Windsurf-inspired) =====
export function generatePlan(sessionId: string, task: string): PlanStep[] {
  const plan: PlanStep[] = [
    { id: `${sessionId}_p1`, action: 'Analyze', detail: `Scan codebase for "${task}"`, status: 'pending' },
    { id: `${sessionId}_p2`, action: 'Design', detail: 'Determine approach and identify affected files', status: 'pending' },
    { id: `${sessionId}_p3`, action: 'Execute', detail: 'Implement changes across identified files', status: 'pending' },
    { id: `${sessionId}_p4`, action: 'Verify', detail: 'Run tests and validate output', status: 'pending' },
  ];
  const session = sessions.find(s => s.id === sessionId);
  if (session) session.plan = plan;
  return plan;
}

export function approvePlanStep(sessionId: string, stepId: string): void {
  const session = sessions.find(s => s.id === sessionId);
  const step = session?.plan?.find(p => p.id === stepId);
  if (step && step.status === 'pending') step.status = 'approved';
}

export function rejectPlanStep(sessionId: string, stepId: string): void {
  const session = sessions.find(s => s.id === sessionId);
  const step = session?.plan?.find(p => p.id === stepId);
  if (step && step.status === 'pending') step.status = 'rejected';
}

export function tickSession(id: string): void {
  const session = sessions.find(s => s.id === id);
  if (!session || session.status !== 'running') return;

  const idx = CYCLE_STEPS.indexOf(session.currentStep);
  const next = (idx + 1) % CYCLE_STEPS.length;
  session.currentStep = CYCLE_STEPS[next];
  session.progress = Math.min(100, session.progress + 8.33);

  if (next === 0) {
    session.progress = 100;
    session.status = 'completed';
  }
}

export function getActiveStep(id: string): string {
  const session = sessions.find(s => s.id === id);
  return session?.currentStep || 'GATHER';
}

export function getSessionStatus(id: string): string {
  const session = sessions.find(s => s.id === id);
  return session?.status || 'idle';
}

export function startSession(id: string): void {
  const session = sessions.find(s => s.id === id);
  if (session) {
    session.status = 'running';
    session.startedAt = Date.now();
  }
}

export function pauseSession(id: string): void {
  const session = sessions.find(s => s.id === id);
  if (session && session.status === 'running') {
    session.status = 'paused';
  }
}

export function stopSession(id: string): void {
  const session = sessions.find(s => s.id === id);
  if (session) {
    session.status = 'idle';
    session.currentStep = 'GATHER';
    session.progress = 0;
  }
}

export { CYCLE_STEPS, stepColors };
