import React from "react";
import type { PermissionRequest } from "../types";

interface Props {
  permission: PermissionRequest;
  onApprove: () => void;
  onDeny: () => void;
}

const ACTION_ICONS: Record<string, string> = {
  file: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z",
  edit: "M17 3a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h10zm-7 11l-1 3 3-1 9-9-3-3-8 8z",
  shell: "M7 15l-5-5 5-5m10 10l5-5-5-5",
  network: "M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm-1 17.93A8 8 0 0 1 4 12a8 8 0 0 1 6-7.73zM11 12a1 1 0 1 1 2 0 1 1 0 0 1-2 0zm0-7a1 1 0 1 1 2 0 1 1 0 0 1-2 0zm7 7a1 1 0 1 1-2 0 1 1 0 0 1 2 0z",
  read: "M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2zM22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z",
  default: "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z",
};

function iconForAction(action: string): string {
  const lower = action.toLowerCase();
  if (lower.includes("edit") || lower.includes("write") || lower.includes("change") || lower.includes("modify"))
    return ACTION_ICONS.edit;
  if (lower.includes("shell") || lower.includes("bash") || lower.includes("exec") || lower.includes("run"))
    return ACTION_ICONS.shell;
  if (lower.includes("network") || lower.includes("http") || lower.includes("fetch") || lower.includes("curl") || lower.includes("api"))
    return ACTION_ICONS.network;
  if (lower.includes("read") || lower.includes("view") || lower.includes("list"))
    return ACTION_ICONS.read;
  if (lower.includes("file") || lower.includes("write_file"))
    return ACTION_ICONS.file;
  return ACTION_ICONS.default;
}

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: "fixed", top: 0, left: 0, right: 0, bottom: 0,
    background: "rgba(0,0,0,0.35)",
    backdropFilter: "blur(6px)",
    WebkitBackdropFilter: "blur(6px)",
    zIndex: 110,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    animation: "fadeIn 0.15s ease",
  },
  card: {
    width: 440,
    background: "rgba(255,255,255,0.92)",
    borderRadius: 12,
    border: "1px solid rgba(0,0,0,0.08)",
    boxShadow: "0 8px 32px rgba(0,0,0,0.12)",
    overflow: "hidden",
  },
  iconArea: {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    padding: "20px 20px 0",
  },
  iconCircle: {
    width: 48,
    height: 48,
    borderRadius: "50%",
    background: "rgba(0,122,255,0.1)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  },
  body: {
    padding: "16px 20px",
  },
  actionRow: {
    display: "flex",
    alignItems: "baseline",
    gap: 8,
    marginBottom: 8,
  },
  actionType: {
    fontSize: 12,
    fontWeight: 600,
    textTransform: "uppercase",
    letterSpacing: "0.5px",
    color: "#86868b",
  },
  actionValue: {
    fontSize: 20,
    fontWeight: 700,
    letterSpacing: "-0.5px",
    color: "#1d1d1f",
  },
  targetRow: {
    marginBottom: 12,
  },
  targetValue: {
    fontSize: 15,
    fontWeight: 500,
    color: "#007aff",
    fontFamily: "'SF Mono', Menlo, monospace",
    wordBreak: "break-all",
  },
  codeBlock: {
    background: "rgba(0,0,0,0.04)",
    borderRadius: 8,
    padding: "10px 12px",
    fontFamily: "'SF Mono', Menlo, monospace",
    fontSize: 12,
    lineHeight: 1.5,
    color: "#86868b",
    marginBottom: 16,
    maxHeight: 120,
    overflowY: "auto",
    whiteSpace: "pre-wrap",
    wordBreak: "break-word",
  },
  footer: {
    display: "flex",
    flexDirection: "column",
    gap: 12,
    padding: "0 20px 20px",
  },
  buttonRow: {
    display: "flex",
    gap: 10,
  },
  allowBtn: {
    flex: 1,
    padding: "10px 0",
    border: "none",
    borderRadius: 8,
    fontSize: 14,
    fontWeight: 600,
    cursor: "pointer",
    background: "#34c759",
    color: "#fff",
    transition: "opacity 0.15s ease",
  },
  denyBtn: {
    flex: 1,
    padding: "10px 0",
    border: "none",
    borderRadius: 8,
    fontSize: 14,
    fontWeight: 600,
    cursor: "pointer",
    background: "#ff3b30",
    color: "#fff",
    transition: "opacity 0.15s ease",
  },
  checkboxRow: {
    display: "flex",
    alignItems: "center",
    gap: 8,
    padding: "8px 0 0",
    borderTop: "1px solid rgba(0,0,0,0.06)",
  },
  checkbox: {
    width: 16,
    height: 16,
    accentColor: "#007aff",
    cursor: "pointer",
  },
  checkboxLabel: {
    fontSize: 12,
    color: "#86868b",
    cursor: "pointer",
  },
};

const PermissionCard: React.FC<Props> = ({ permission, onApprove, onDeny }) => {
  const iconPath = iconForAction(permission.action);

  return (
    <div style={styles.overlay} onMouseDown={(e) => { if (e.target === e.currentTarget) onDeny(); }}>
      <div style={styles.card} onMouseDown={(e) => e.stopPropagation()}>
        <div style={styles.iconArea}>
          <div style={styles.iconCircle}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#007aff" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
              <path d={iconPath} />
            </svg>
          </div>
        </div>

        <div style={styles.body}>
          <div style={styles.actionRow}>
            <span style={styles.actionType}>Action</span>
          </div>
          <div style={styles.actionValue}>{permission.action}</div>

          <div style={{ ...styles.targetRow, marginTop: 12 }}>
            <div style={{ ...styles.actionType, marginBottom: 4 }}>Target</div>
            <div style={styles.targetValue}>{permission.target}</div>
          </div>

          <div style={{ ...styles.actionType, marginBottom: 4 }}>Details</div>
          <div style={styles.codeBlock}>{permission.details}</div>
        </div>

        <div style={styles.footer}>
          <div style={styles.buttonRow}>
            <button
              style={styles.denyBtn}
              onClick={onDeny}
              onMouseEnter={(e) => (e.currentTarget.style.opacity = "0.85")}
              onMouseLeave={(e) => (e.currentTarget.style.opacity = "1")}
            >
              Deny
            </button>
            <button
              style={styles.allowBtn}
              onClick={onApprove}
              onMouseEnter={(e) => (e.currentTarget.style.opacity = "0.85")}
              onMouseLeave={(e) => (e.currentTarget.style.opacity = "1")}
            >
              Allow
            </button>
          </div>

          <div style={styles.checkboxRow}>
            <input type="checkbox" id="always-allow" style={styles.checkbox} />
            <label htmlFor="always-allow" style={styles.checkboxLabel}>
              Always allow for this session
            </label>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PermissionCard;
