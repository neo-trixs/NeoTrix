import { create } from "zustand";
import { createSessionSlice } from "./sessionSlice";
import { createUiSlice } from "./uiSlice";
import { createProviderSlice } from "./providerSlice";
import { createStreamingSlice } from "./streamingSlice";
import { createAgentSlice } from "./agentSlice";
import { createSyncSlice } from "./syncSlice";
import { createDesktopSlice } from "./desktopSlice";
import { createEditorSlice } from "./editorSlice";
import { createNotificationSlice } from "./notificationSlice";
import { createUpdateSlice } from "./updateSlice";
import type { SessionSlice } from "./sessionSlice";
import type { UiSlice } from "./uiSlice";
import type { ProviderSlice } from "./providerSlice";
import type { StreamingSlice } from "./streamingSlice";
import type { AgentSlice } from "./agentSlice";
import type { SyncSlice } from "./syncSlice";
import type { DesktopSlice } from "./desktopSlice";
import type { EditorSlice } from "./editorSlice";
import type { NotificationSlice } from "./notificationSlice";
import type { UpdateSlice } from "./updateSlice";

export type FullStore = SessionSlice & UiSlice & ProviderSlice & StreamingSlice & AgentSlice & SyncSlice & DesktopSlice & EditorSlice & NotificationSlice & UpdateSlice;

export type { Notification } from "./notificationSlice";
export type { E8State, GWTResonance, GWTExpert, SEALStatus } from "../types";

// @ts-expect-error — zustand v4 cannot infer spread-of-slices return type with create<FullStore>()
// Runtime behavior is correct; all slice fields and methods are available.
export const useStore = create<FullStore>()((set: any, get: any, _api: any) => ({
  ...createSessionSlice(set, get),
  ...createUiSlice(set),
  ...createProviderSlice(set),
  ...createStreamingSlice(set, get),
  ...createAgentSlice(set),
  ...createSyncSlice(set),
  ...createDesktopSlice(set),
  ...createEditorSlice(set),
  ...createNotificationSlice(set),
  ...createUpdateSlice(set),
}));
