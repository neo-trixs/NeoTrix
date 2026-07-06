import type { ProviderConfig, AgentPreset } from "../types";
import { persistence } from "../lib/persistence";
import { savedProvider, DEFAULT_PROVIDER, loadCustomPresets, saveCustomPresets } from "./store-utils";

export interface ProviderSlice {
  providerConfig: ProviderConfig;
  customPresets: AgentPreset[];

  setProviderConfig: (config: ProviderConfig) => void;
  addCustomPreset: (preset: AgentPreset) => void;
  removeCustomPreset: (id: string) => void;
}

export const createProviderSlice = (set: any) => ({
  providerConfig: savedProvider ?? DEFAULT_PROVIDER,
  customPresets: loadCustomPresets(),

  setProviderConfig: (config: ProviderConfig) => {
    set({ providerConfig: config });
    persistence.saveProviderConfig(config);
  },

  addCustomPreset: (preset: AgentPreset) => set((state: any) => {
    const next = [...state.customPresets, preset];
    saveCustomPresets(next);
    return { customPresets: next };
  }),

  removeCustomPreset: (id: string) => set((state: any) => {
    const next = state.customPresets.filter((p: AgentPreset) => p.id !== id);
    saveCustomPresets(next);
    return { customPresets: next };
  }),
});
