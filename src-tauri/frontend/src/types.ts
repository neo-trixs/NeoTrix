export interface Message {
  role: "user" | "assistant" | "system" | "error";
  content: string;
  contentType?: "markdown" | "html" | "text";
  timestamp?: number;
  attachments?: Attachment[];
}

export interface Attachment {
  id: string;
  name: string;
  size: number;
  mimeType: string;
  data: string; // base64
}

export interface AppSettings {
  theme: "light" | "dark" | "system";
  fontSize: number;
  autoSave: boolean;
  language: "zh-CN" | "en-US";
  terminalPath: string;
  maxSessions: number;
  voiceInput: boolean;
  voiceLang: string;
  voiceAutoSend: boolean;
  privacyStoreMessages: boolean;
  privacyTelemetry: boolean;
  privacyLocalFirst: boolean;
  privacyPreflightCheck: boolean;
  defaultModel: string;
  temperature: number;
  maxTokens: number;
}

export type NeoCodexMode = "Agent" | "Shell" | "Plan";

export interface NeoCodexProviderConfig {
  provider_count: number;
  resolvable: boolean;
  active_model: string;
  providers: NeoCodexProviderEntry[];
}

export interface NeoCodexProviderEntry {
  name: string;
  model: string;
  resolvable: boolean;
}

export interface NeoCodexSession {
  id: string;
  name: string;
  mode: NeoCodexMode;
  messages: Message[];
  wire_path: string;
  created_at: number;
  updated_at: number;
}
