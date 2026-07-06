import type { EvolutionState, BrainHealth, BrainEvent } from "../types";

export interface AgentSlice {
  agentBusy: boolean;
  agentMakerActive: boolean;
  evolutionVisible: boolean;
  evolutionState: EvolutionState;
  brainHealth: BrainHealth;
  brainEvents: BrainEvent[];

  setAgentBusy: (busy: boolean) => void;
  setAgentMakerActive: (active: boolean) => void;
  setEvolutionVisible: (show: boolean) => void;
  setEvolutionState: (state: EvolutionState) => void;
  setBrainHealth: (health: BrainHealth) => void;
  setBrainEvents: (events: BrainEvent[]) => void;
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

  setAgentBusy: (busy: boolean) => set({ agentBusy: busy }),
  setAgentMakerActive: (active: boolean) => set({ agentMakerActive: active }),
  setEvolutionVisible: (show: boolean) => set({ evolutionVisible: show }),
  setEvolutionState: (state: EvolutionState) => set({ evolutionState: state }),
  setBrainHealth: (health: BrainHealth) => set({ brainHealth: health }),
  setBrainEvents: (events: BrainEvent[]) => set({ brainEvents: events }),
});
