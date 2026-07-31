import { create } from "zustand";
import { createSessionSlice } from "./sessionSlice";
import { createAgentSlice } from "./agentSlice";
import { createNotificationSlice } from "./notificationSlice";
import { createUpdateSlice } from "./updateSlice";
import type { SessionSlice } from "./sessionSlice";
import type { AgentSlice } from "./agentSlice";
import type { NotificationSlice } from "./notificationSlice";
import type { UpdateSlice } from "./updateSlice";

export type FullStore = SessionSlice & AgentSlice & NotificationSlice & UpdateSlice;

export type { Notification } from "./notificationSlice";

// @ts-expect-error — zustand v4 cannot infer spread-of-slices return type with create<FullStore>()
// Runtime behavior is correct; all slice fields and methods are available.
export const useStore = create<FullStore>()((set: any, get: any, _api: any) => ({
  ...createSessionSlice(set),
  ...createAgentSlice(set),
  ...createNotificationSlice(set),
  ...createUpdateSlice(set),
}));
