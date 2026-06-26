import React from "react";
import { useStore, type Notification } from "../store";

const ICONS: Record<Notification["type"], string> = {
  success: "✓",
  error: "✕",
  warning: "⚠",
  info: "ℹ",
};

const NotificationItem: React.FC<{ notif: Notification }> = ({ notif }) => {
  const removeNotification = useStore((s) => s.removeNotification);
  return (
    <div className={`notification-toast notification-${notif.type}`} role="alert">
      <span className="notification-icon">{ICONS[notif.type]}</span>
      <span className="notification-message">{notif.message}</span>
      <button
        className="notification-close"
        onClick={() => removeNotification(notif.id)}
        aria-label="Dismiss"
      >
        ✕
      </button>
    </div>
  );
};

const NotificationToast: React.FC = () => {
  const notifications = useStore((s) => s.notifications);
  if (notifications.length === 0) return null;
  return (
    <div className="notification-container" aria-live="polite">
      {notifications.map((n) => (
        <NotificationItem key={n.id} notif={n} />
      ))}
    </div>
  );
};

export default NotificationToast;
