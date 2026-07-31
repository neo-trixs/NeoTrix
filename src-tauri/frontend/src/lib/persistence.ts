import type { AppSettings } from "../types";

const KEYS = {
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
  loadSettings(): AppSettings | null {
    return safeGet<AppSettings | null>(KEYS.settings, null);
  },
  saveSettings(settings: AppSettings): void {
    safeSet(KEYS.settings, settings);
  },
};
