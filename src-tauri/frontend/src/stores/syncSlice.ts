import type { ProxyStatus } from "../types";

export interface SyncSlice {
  syncVisible: boolean;
  proxyVisible: boolean;
  proxyStatus: ProxyStatus;

  setSyncVisible: (show: boolean) => void;
  setProxyVisible: (show: boolean) => void;
  setProxyStatus: (status: ProxyStatus) => void;
}

export const createSyncSlice = (set: any) => ({
  syncVisible: false,
  proxyVisible: false,
  proxyStatus: { running: false, mode: "off", pid: 0, port: 11080, uptime_secs: 0, active_count: 0, idle_secs: 0 },

  setSyncVisible: (show: boolean) => set({ syncVisible: show }),
  setProxyVisible: (show: boolean) => set({ proxyVisible: show }),
  setProxyStatus: (status: ProxyStatus) => set({ proxyStatus: status }),
});
