import type { EvolutionState, BrainHealth, BrainEvent, NeoCodexHealthReport, NeoCodexMode, NeoCodexEvolutionState, NeoCodexProviderConfig } from "../types";

export interface AgentSlice {
  agentBusy: boolean;
  agentMakerActive: boolean;
  evolutionVisible: boolean;
  evolutionState: EvolutionState;
  brainHealth: BrainHealth;
  brainEvents: BrainEvent[];

  // NeoCodex Desktop state
  neocodexMode: NeoCodexMode;
  neocodexHealth: NeoCodexHealthReport | null;
  neocodexEvolution: NeoCodexEvolutionState | null;
  neocodexProviderConfig: NeoCodexProviderConfig | null;
  neocodexMessages: Array<{ role: string; content: string; contentType?: "markdown" | "html" | "text"; timestamp?: number }>;
  neocodexStreaming: { content: string; role: "user" | "assistant" } | null;
  neocodexSessions: Array<{ id: string; name: string; mode: string; messages: any[]; wire_path: string; created_at: number; updated_at: number }>;
  neocodexActiveSessionId: string | null;

  setAgentBusy: (busy: boolean) => void;
  setAgentMakerActive: (active: boolean) => void;
  setEvolutionVisible: (show: boolean) => void;
  setEvolutionState: (state: EvolutionState) => void;
  setBrainHealth: (health: BrainHealth) => void;
  setBrainEvents: (events: BrainEvent[]) => void;

  // NeoCodex actions
  setNeoCodexMode: (mode: NeoCodexMode) => void;
  setNeoCodexHealth: (health: NeoCodexHealthReport) => void;
  setNeoCodexEvolution: (evolution: NeoCodexEvolutionState) => void;
  setNeoCodexProviderConfig: (config: NeoCodexProviderConfig) => void;
  setNeoCodexMessages: (messages: Array<{ role: string; content: string; contentType?: "markdown" | "html" | "text"; timestamp?: number }>) => void;
  addNeoCodexMessage: (message: { role: string; content: string; contentType?: "markdown" | "html" | "text"; timestamp?: number }) => void;
  setNeoCodexStreaming: (streaming: { content: string; role: "user" | "assistant" } | null) => void;
  setNeoCodexSessions: (sessions: Array<{ id: string; name: string; mode: string; messages: any[]; wire_path: string; created_at: number; updated_at: number }>) => void;
  setNeoCodexActiveSession: (sessionId: string | null) => void;
}

export const createAgentSlice = (set: any) => ({
  agentBusy: false,
  agentMakerActive: false,
  evolutionVisible: false,
  evolutionState: {
    iteration: 0, strategy: 'Direct', contextUsage: 0,
    intrinsicReward: 0.5, confidence: 0.5, errorRate: 0, noveltyScore: 0,
    shouldExplore: true, stabilityScore: 0.5, flagsCount: 0, repairsCount: 0,
    archiveSnapshots: 0, selfRepairs: 0,
  },
  brainHealth: { health_score: 85, degradation: "full", cognitive_load: "balanced", iteration: 0, curiosity_bonus: 0.5 },
  brainEvents: [],

  // NeoCodex initial state
  neocodexMode: "Agent",
  neocodexHealth: null,
  neocodexEvolution: null,
  neocodexProviderConfig: null,
  neocodexMessages: [],
  neocodexStreaming: null,
  neocodexSessions: [],
  neocodexActiveSessionId: null,

  setAgentBusy: (busy: boolean) => set({ agentBusy: busy }),
  setAgentMakerActive: (active: boolean) => set({ agentMakerActive: active }),
  setEvolutionVisible: (show: boolean) => set({ evolutionVisible: show }),
  setEvolutionState: (state: EvolutionState) => set({ evolutionState: state }),
  setBrainHealth: (health: BrainHealth) => set({ brainHealth: health }),
  setBrainEvents: (events: BrainEvent[]) => set({ brainEvents: events }),

  // NeoCodex actions
  setNeoCodexMode: (mode: NeoCodexMode) => set({ neocodexMode: mode }),
  setNeoCodexHealth: (health: NeoCodexHealthReport) => set({ neocodexHealth: health }),
  setNeoCodexEvolution: (evolution: NeoCodexEvolutionState) => set({ neocodexEvolution: evolution }),
  setNeoCodexProviderConfig: (config: NeoCodexProviderConfig) => set({ neocodexProviderConfig: config }),
  setNeoCodexMessages: (messages: Array<{ role: string; content: string; contentType?: "markdown" | "html" | "text"; timestamp?: number }>) => set({ neocodexMessages: messages }),
  addNeoCodexMessage: (message: { role: string; content: string; contentType?: "markdown" | "html" | "text"; timestamp?: number }) => set((state: any) => ({ neocodexMessages: [...state.neocodexMessages, message] })),
  setNeoCodexStreaming: (streaming: { content: string; role: "user" | "assistant" } | null) => set({ neocodexStreaming: streaming }),
  setNeoCodexSessions: (sessions: Array<{ id: string; name: string; mode: string; messages: any[]; wire_path: string; created_at: number; updated_at: number }>) => set({ neocodexSessions: sessions }),
  setNeoCodexActiveSession: (sessionId: string | null) => set({ neocodexActiveSessionId: sessionId }),
});
