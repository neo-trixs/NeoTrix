export interface Message {
  id?: number;
  role: "user" | "assistant" | "system" | "error" | "tool";
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
  notifyOnComplete: boolean;
  accent: string;
  permissionMode: PermissionMode;
}

export type NeoCodexMode = "Agent" | "Shell" | "Plan";

/** Claude Code Manual/AcceptEdits/Plan / Codex approval parity.
 *  auto: 自主执行（默认）；accept: 自动应用编辑但在 Diff 面板可审阅；
 *  manual: 每轮编辑后等待人工 per-file 确认；plan: 只读规划。 */
export type PermissionMode = "auto" | "accept" | "manual" | "plan";

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
  message_count?: number;
  messages: Message[];
  wire_path: string;
  created_at: number;
  updated_at: number;
}
