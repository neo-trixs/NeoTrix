import type { DiffBlock, E8State, GWTResonance, SEALStatus } from "../types";
import { persistence } from "../lib/persistence";

export interface UiSlice {
  showOnboarding: boolean;
  showShortcuts: boolean;
  showSearch: boolean;
  showCommandPalette: boolean;
  searchQuery: string;
  splitViewActive: boolean;
  showAgentManager: boolean;
  showPrivacyFilter: boolean;
  showSandboxManager: boolean;
  showIdentityManager: boolean;

  // ── User ──
  userPopoverOpen: boolean;
  userDisplayName: string;

  // ── Consciousness ──
  e8State: E8State;
  gwtResonance: GWTResonance;
  sealStatus: SEALStatus;
  consciousnessActive: boolean;
  sidebarCollapsed: boolean;
  pendingDiff: { blocks: DiffBlock[]; filename?: string } | null;
  rightPanelWidth: number;

  setShowOnboarding: (show: boolean) => void;
  setShowShortcuts: (show: boolean) => void;
  setShowSearch: (show: boolean) => void;
  setShowCommandPalette: (show: boolean) => void;
  setSearchQuery: (query: string) => void;
  setSplitViewActive: (active: boolean) => void;
  setShowAgentManager: (show: boolean) => void;
  setShowPrivacyFilter: (show: boolean) => void;
  setShowSandboxManager: (show: boolean) => void;
  setShowIdentityManager: (show: boolean) => void;
  setUserPopoverOpen: (open: boolean) => void;
  setUserDisplayName: (name: string) => void;
  setE8State: (state: E8State) => void;
  setGWTResonance: (resonance: GWTResonance) => void;
  setSEALStatus: (status: SEALStatus) => void;
  setConsciousnessActive: (active: boolean) => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setPendingDiff: (diff: { blocks: DiffBlock[]; filename?: string } | null) => void;
  setRightPanelWidth: (width: number) => void;
}

let onboardingDone = false;
try { onboardingDone = localStorage.getItem("neotrix_onboarding_done") === "true"; } catch {};
const hasApiKey = !!(persistence.loadProviderConfig()?.apiKey);

const DEFAULT_E8: E8State = {
  hexagram: 0x01,
  hexagramName: "Grounding",
  confidence: 0.5,
  lines: [
    { value: 1, changing: false },
    { value: 1, changing: false },
    { value: 0, changing: false },
    { value: 0, changing: false },
    { value: 0, changing: false },
    { value: 1, changing: false },
  ],
  transitioning: false,
};

const DEFAULT_GWT: GWTResonance = {
  activeCount: 0,
  totalCount: 6,
  entropy: 0.5,
  experts: [
    { id: "lang", shortName: "语言", icon: "💬", resonance: 0.1, hue: 42, weight: 0.1 },
    { id: "code", shortName: "代码", icon: "💻", resonance: 0.1, hue: 38, weight: 0.1 },
    { id: "tool", shortName: "工具", icon: "🔧", resonance: 0.1, hue: 34, weight: 0.1 },
    { id: "search", shortName: "搜索", icon: "🔍", resonance: 0.1, hue: 30, weight: 0.1 },
    { id: "memory", shortName: "记忆", icon: "🧠", resonance: 0.1, hue: 26, weight: 0.1 },
    { id: "reflect", shortName: "反省", icon: "🪞", resonance: 0.1, hue: 22, weight: 0.1 },
  ],
};

const savedDisplayName = (() => { try { return localStorage.getItem("neotrix_display_name") || ""; } catch { return ""; } })();

export const createUiSlice = (set: any) => ({
  showOnboarding: !onboardingDone && !hasApiKey,
  showShortcuts: false,
  showSearch: false,
  showCommandPalette: false,
  searchQuery: "",
  splitViewActive: false,
  showAgentManager: false,
  showPrivacyFilter: false,
  showSandboxManager: false,
  showIdentityManager: false,

  userPopoverOpen: false,
  userDisplayName: savedDisplayName || "Neo",

  e8State: DEFAULT_E8,
  gwtResonance: DEFAULT_GWT,
  sealStatus: { maturityLevel: 3, currentEpoch: 7, stageName: "Gap Analysis", healthScore: 0.82 },
  consciousnessActive: false,
  sidebarCollapsed: false,
  pendingDiff: null,
  rightPanelWidth: 320,

  setShowOnboarding: (show: boolean) => set({ showOnboarding: show }),
  setShowShortcuts: (show: boolean) => set({ showShortcuts: show }),
  setShowSearch: (show: boolean) => set({ showSearch: show }),
  setShowCommandPalette: (show: boolean) => set({ showCommandPalette: show }),
  setSearchQuery: (query: string) => set({ searchQuery: query }),
  setSplitViewActive: (active: boolean) => set({ splitViewActive: active }),
  setShowAgentManager: (show: boolean) => set({ showAgentManager: show }),
  setShowPrivacyFilter: (show: boolean) => set({ showPrivacyFilter: show }),
  setShowSandboxManager: (show: boolean) => set({ showSandboxManager: show }),
  setShowIdentityManager: (show: boolean) => set({ showIdentityManager: show }),
  setUserPopoverOpen: (open: boolean) => set({ userPopoverOpen: open }),
  setUserDisplayName: (name: string) => {
    try { localStorage.setItem("neotrix_display_name", name); } catch {}
    set({ userDisplayName: name });
  },
  setE8State: (state: E8State) => set({ e8State: state }),
  setGWTResonance: (resonance: GWTResonance) => set({ gwtResonance: resonance }),
  setSEALStatus: (status: SEALStatus) => set({ sealStatus: status }),
  setConsciousnessActive: (active: boolean) => set({ consciousnessActive: active }),
  setSidebarCollapsed: (collapsed: boolean) => set({ sidebarCollapsed: collapsed }),
  setPendingDiff: (diff: { blocks: DiffBlock[]; filename?: string } | null) => set({ pendingDiff: diff }),
  setRightPanelWidth: (width: number) => set({ rightPanelWidth: width }),
});
