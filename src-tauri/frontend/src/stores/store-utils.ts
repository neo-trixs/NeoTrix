import { persistence } from "../lib/persistence";
import type { Session, ProviderConfig, AppSettings, AgentPreset } from "../types";

export const PERSIST_DEBOUNCE = 500;

let persistTimer: ReturnType<typeof setTimeout> | null = null;

export function schedulePersist(sessions: Session[]) {
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    persistence.saveSessions(sessions);
  }, PERSIST_DEBOUNCE);
}

export const DEFAULT_PROVIDER: ProviderConfig = {
  id: "anthropic",
  name: "Anthropic Claude",
  model: "claude-sonnet-4-20250514",
  apiKey: "",
  learningRate: 0.05,
};

export const DEFAULT_SETTINGS: AppSettings = {
  theme: "light",
  fontSize: 13,
  autoSave: true,
  language: "zh-CN",
  terminalPath: "",
  maxSessions: 20,
  voiceInput: true,
  voiceLang: "zh-CN",
  voiceAutoSend: false,
  privacyStoreMessages: true,
  privacyTelemetry: false,
  privacyLocalFirst: true,
  privacyPreflightCheck: true,
  defaultModel: "GatewayV2",
  temperature: 0.7,
  maxTokens: 8192,
};

export const savedProvider = persistence.loadProviderConfig();
export const savedSettings = persistence.loadSettings();
export const savedKnowledge = persistence.loadKnowledgeBase();
export const savedSessions = persistence.loadSessions();

export function loadCustomPresets(): AgentPreset[] {
  try {
    const raw = localStorage.getItem("neotrix_presets");
    return raw ? JSON.parse(raw) : [];
  } catch { return []; }
}

export function saveCustomPresets(presets: AgentPreset[]) {
  try { localStorage.setItem("neotrix_presets", JSON.stringify(presets)); } catch {}
}
