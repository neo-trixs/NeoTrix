import type { Session, ProviderConfig, KnowledgeEntry, AppSettings } from "../types";

const KEYS = {
  sessions: "neotrix_sessions",
  providerConfig: "neotrix_provider",
  knowledgeBase: "neotrix_knowledge",
  settings: "neotrix_settings",
} as const;

function safeGet<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function safeSet(key: string, value: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch (e) {
    console.warn("persistence: failed to save", key, e);
  }
}

export const persistence = {
  loadSessions(): Session[] {
    return safeGet<Session[]>(KEYS.sessions, []);
  },
  saveSessions(sessions: Session[]): void {
    safeSet(KEYS.sessions, sessions);
  },

  loadProviderConfig(): ProviderConfig | null {
    return safeGet<ProviderConfig | null>(KEYS.providerConfig, null);
  },
  saveProviderConfig(config: ProviderConfig): void {
    safeSet(KEYS.providerConfig, config);
  },

  loadKnowledgeBase(): KnowledgeEntry[] {
    return safeGet<KnowledgeEntry[]>(KEYS.knowledgeBase, []);
  },
  saveKnowledgeBase(entries: KnowledgeEntry[]): void {
    safeSet(KEYS.knowledgeBase, entries);
  },

  loadSettings(): AppSettings | null {
    return safeGet<AppSettings | null>(KEYS.settings, null);
  },
  saveSettings(settings: AppSettings): void {
    safeSet(KEYS.settings, settings);
  },
};
