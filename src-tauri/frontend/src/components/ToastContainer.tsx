import React from "react";
import { useStore, type Notification } from "../store";

const ICONS: Record<Notification["type"], string> = {
  success: "\u2713",
  error: "\u2715",
  warning: "\u26A0",
  info: "\u2139",
};

const ToastItem: React.FC<{ notif: Notification }> = ({ notif }) => {
  const removeNotification = useStore((s) => s.removeNotification);
  return (
    <div className={`toast toast-${notif.type}`} role="alert">
      <span className={`toast-icon ${notif.type}`}>{ICONS[notif.type]}</span>
      <span className="toast-message">{notif.message}</span>
      <button className="toast-close" onClick={() => removeNotification(notif.id)} aria-label="Dismiss">
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5">
          <path d="M2 2l6 6M8 2l-6 6" />
        </svg>
      </button>
    </div>
  );
};

const ToastContainer: React.FC = () => {
  const notifications = useStore((s) => s.notifications);
  if (notifications.length === 0) return null;
  return (
    <div className="toast-container" aria-live="polite">
      {notifications.map((n) => (
        <ToastItem key={n.id} notif={n} />
      ))}
    </div>
  );
};

export default ToastContainer;
