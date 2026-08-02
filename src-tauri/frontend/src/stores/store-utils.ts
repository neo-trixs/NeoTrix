import { persistence } from "../lib/persistence";
import type { AppSettings } from "../types";

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
  notifyOnComplete: true,
  accent: "default",
};

export const savedSettings = persistence.loadSettings();
