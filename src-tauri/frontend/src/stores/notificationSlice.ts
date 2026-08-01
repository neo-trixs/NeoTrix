export interface Notification {
  id: string;
  type: "success" | "error" | "warning" | "info";
  message: string;
  duration?: number;
  action?: { label: string; onClick: () => void };
}

export interface NotificationSlice {
  notifications: Notification[];
  addNotification: (n: Omit<Notification, "id">) => void;
  removeNotification: (id: string) => void;
}

export const createNotificationSlice = (set: any) => ({
  notifications: [],

  addNotification: (n: Omit<Notification, "id">) => {
    const id = `notif-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    const notification: Notification = { ...n, id };
    set((state: any) => ({ notifications: [...state.notifications, notification] }));
    const duration = n.duration ?? 5000;
    if (duration > 0) {
      setTimeout(() => {
        set((state: any) => ({ notifications: state.notifications.filter((x: Notification) => x.id !== id) }));
      }, duration);
    }
  },

  removeNotification: (id: string) => set((state: any) => ({ notifications: state.notifications.filter((x: Notification) => x.id !== id) })),
});
