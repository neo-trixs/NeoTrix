import React from "react";
import { useStore, type Notification } from "../stores";
import styles from "./NotificationToast.module.css";

const ICONS: Record<Notification["type"], string> = {
  success: "✓",
  error: "✕",
  warning: "⚠",
  info: "ℹ",
};

const typeClassMap: Record<Notification["type"], string> = {
  success: styles.success,
  error: styles.error,
  warning: styles.warning,
  info: styles.info,
};

const NotificationItem: React.FC<{ notif: Notification }> = ({ notif }) => {
  const removeNotification = useStore((s) => s.removeNotification);
  return (
    <div className={`${styles.toast} ${typeClassMap[notif.type]}`} role="alert">
      <span className={styles.icon}>{ICONS[notif.type]}</span>
      <span className={styles.message}>{notif.message}</span>
      {notif.action && (
        <button
          className={styles.action}
          onClick={() => {
            notif.action!.onClick();
            removeNotification(notif.id);
          }}
        >
          {notif.action.label}
        </button>
      )}
      <button
        className={styles.close}
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
    <div className={styles.container} aria-live="polite">
      {notifications.map((n) => (
        <NotificationItem key={n.id} notif={n} />
      ))}
    </div>
  );
};

export default NotificationToast;
