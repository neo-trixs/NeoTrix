import type { AppSettings } from "../types";
import { persistence } from "../lib/persistence";
import { savedSettings, DEFAULT_SETTINGS } from "./store-utils";

export interface SessionSlice {
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;
}

export const createSessionSlice = (set: any) => ({
  // Merge saved settings over defaults so fields added after first launch
  // (e.g. notifyOnComplete) fall back gracefully for existing users.
  settings: savedSettings ? { ...DEFAULT_SETTINGS, ...savedSettings } : DEFAULT_SETTINGS,

  setSettings: (settings: AppSettings) => {
    set({ settings });
    persistence.saveSettings(settings);
  },
});
