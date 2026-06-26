import { useEffect, useState, useCallback } from "react";
import * as api from "../lib/api";
import type { ImAdapterStatus, ImAdapterConfig } from "../types";

const ADAPTER_META: Record<string, { icon: string; label: string; fields: { key: string; label: string; placeholder: string; secret: boolean }[] }> = {
  telegram: {
    icon: "✈",
    label: "Telegram Bot",
    fields: [
      { key: "bot_token", label: "Bot Token", placeholder: "1234567890:ABCdefGHIjklmNOPqrstUVwxyz", secret: true },
    ],
  },
  whatsapp: {
    icon: "💬",
    label: "WhatsApp Business API",
    fields: [
      { key: "phone_number_id", label: "Phone Number ID", placeholder: "123456789012345", secret: false },
      { key: "access_token", label: "Access Token", placeholder: "EAAC...", secret: true },
      { key: "verify_token", label: "Verify Token", placeholder: "my_verify_token", secret: false },
      { key: "webhook_url", label: "Webhook URL (optional)", placeholder: "https://your-domain.com/webhook", secret: false },
    ],
  },
};

function StatusDot({ status }: { status: ImAdapterStatus }) {
  const color = status.running ? "#34C759" : status.error ? "#FF453A" : status.enabled ? "#FFCC00" : "#aeaeb2";
  const label = status.running ? "Running" : status.error ? "Error" : status.enabled ? "Idle" : "Disabled";
  return (
    <span style={{ display: "inline-flex", alignItems: "center", gap: 4, fontSize: 12 }}>
      <span style={{ width: 8, height: 8, borderRadius: "50%", background: color, display: "inline-block" }} />
      {label}
    </span>
  );
}

export default function ImSettings() {
  const [adapters, setAdapters] = useState<ImAdapterStatus[]>([]);
  const [editing, setEditing] = useState<string | null>(null);
  const [editForm, setEditForm] = useState<Record<string, string>>({});
  const [connecting, setConnecting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const list = await api.imListAdapters();
      setAdapters(list);
    } catch (e) {
      setError(`Failed to load adapters: ${e}`);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleEdit = useCallback(async (name: string) => {
    try {
      const config = await api.imGetAdapter(name);
      if (config) {
        setEditForm({
          bot_token: config.bot_token || "",
          phone_number_id: config.phone_number_id || "",
          access_token: config.access_token || "",
          webhook_url: config.webhook_url || "",
          verify_token: config.verify_token || "",
        });
      } else {
        setEditForm({ bot_token: "", phone_number_id: "", access_token: "", webhook_url: "", verify_token: "" });
      }
      setEditing(name);
    } catch (e) {
      setError(`Failed to load config: ${e}`);
    }
  }, []);

  const handleSave = useCallback(async () => {
    if (!editing) return;
    setError(null);
    try {
      await api.imSaveAdapter(
        editing,
        editing,
        editForm.bot_token || null,
        editForm.phone_number_id || null,
        editForm.access_token || null,
        editForm.webhook_url || null,
        editForm.verify_token || null,
      );
      await load();
      setEditing(null);
    } catch (e) {
      setError(`Save failed: ${e}`);
    }
  }, [editing, editForm, load]);

  const handleConnect = useCallback(async (name: string) => {
    setConnecting(name);
    setError(null);
    try {
      await api.imConnectAdapter(name);
      await load();
    } catch (e) {
      setError(`Connection failed: ${e}`);
    }
    setConnecting(null);
  }, [load]);

  const handleDisconnect = useCallback(async (name: string) => {
    try {
      await api.imDisconnectAdapter(name);
      await load();
    } catch (e) {
      setError(`Disconnect failed: ${e}`);
    }
  }, [load]);

  return (
    <div className="im-settings">
      <div className="settings-group-header">
        <h3>IM 通信频道</h3>
        <p style={{ fontSize: 12, opacity: 0.5, margin: 0 }}>连接 WhatsApp / Telegram 机器人，让 NeoTrix 通过 IM 与你对话</p>
      </div>

      {error && (
        <div className="im-error">
          <span>⚠</span> {error}
          <button className="btn-icon" onClick={() => setError(null)} style={{ marginLeft: "auto", background: "none", border: "none", cursor: "pointer", opacity: 0.5 }}>
            ✕
          </button>
        </div>
      )}

      <div className="im-adapter-list">
        {adapters.map((adapter) => {
          const meta = ADAPTER_META[adapter.adapter_type!];
          if (!meta) return null;
          const isEditing = editing === adapter.name;

          return (
            <div key={adapter.name} className={`im-adapter-card ${adapter.running ? "im-running" : ""} ${adapter.error ? "im-error-card" : ""}`}>
              <div className="im-adapter-header">
                <span className="im-adapter-icon">{meta.icon}</span>
                <div className="im-adapter-info">
                  <strong>{meta.label}</strong>
                  <StatusDot status={adapter} />
                </div>
                <div className="im-adapter-actions">
                  {adapter.running ? (
                    <button className="btn-xs btn-secondary" onClick={() => handleDisconnect(adapter.name!)}>Disconnect</button>
                  ) : adapter.enabled ? (
                    <button className="btn-xs btn-primary" onClick={() => handleConnect(adapter.name!)} disabled={connecting === adapter.name}>
                      {connecting === adapter.name ? "Connecting..." : "Connect"}
                    </button>
                  ) : null}
                  <button className="btn-xs btn-ghost" onClick={() => handleEdit(adapter.name!)}>
                    {isEditing ? "Cancel" : "Configure"}
                  </button>
                </div>
              </div>

              {adapter.error && (
                <div className="im-adapter-error">{adapter.error}</div>
              )}

              {isEditing && (
                <div className="im-adapter-form">
                  {meta.fields.map((field: { key: string; label: string; placeholder: string; secret: boolean }) => (
                    <div key={field.key} className="settings-group">
                      <label>{field.label}</label>
                      <input
                        type={field.secret ? "password" : "text"}
                        value={editForm[field.key] || ""}
                        onChange={(e) => setEditForm({ ...editForm, [field.key]: e.target.value })}
                        placeholder={field.placeholder}
                      />
                    </div>
                  ))}
                  <div className="im-form-actions">
                    <button className="btn-primary" onClick={handleSave}>Save & Close</button>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>

      <style>{`
        .im-settings { display: flex; flex-direction: column; gap: 16px; }
        .settings-group-header { margin-bottom: 8px; }
        .settings-group-header h3 { margin: 0 0 4px; font-size: 15px; }

        .im-error {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px 12px;
          background: rgba(255,69,58,0.08);
          border: 1px solid rgba(255,69,58,0.2);
          border-radius: 6px;
          font-size: 13px;
          color: #FF453A;
        }

        .im-adapter-list { display: flex; flex-direction: column; gap: 12px; }

        .im-adapter-card {
          border: 1px solid var(--border-color, rgba(0,0,0,0.08));
          border-radius: 10px;
          overflow: hidden;
          transition: box-shadow 0.2s;
        }
        .im-adapter-card:hover { box-shadow: 0 1px 8px rgba(0,0,0,0.04); }
        .im-adapter-card.im-running { border-left: 3px solid #34C759; }
        .im-adapter-card.im-error-card { border-left: 3px solid #FF453A; }

        .im-adapter-header {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 12px;
        }

        .im-adapter-icon {
          width: 36px;
          height: 36px;
          border-radius: 8px;
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 18px;
          background: rgba(0,122,255,0.08);
        }

        .im-adapter-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
        .im-adapter-info strong { font-size: 14px; }

        .im-adapter-actions { display: flex; gap: 6px; align-items: center; }

        .im-adapter-error {
          padding: 6px 12px 10px 60px;
          font-size: 12px;
          color: #FF453A;
          opacity: 0.8;
        }

        .im-adapter-form {
          padding: 0 12px 12px 60px;
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .im-form-actions { margin-top: 4px; }

        .btn-xs {
          padding: 4px 10px;
          font-size: 11px;
          border-radius: 5px;
          border: 1px solid var(--border-color, rgba(0,0,0,0.1));
          cursor: pointer;
          font-weight: 500;
          transition: all 0.15s;
        }
        .btn-xs:disabled { opacity: 0.5; cursor: not-allowed; }
        .btn-ghost { background: transparent; }
        .btn-ghost:hover { background: rgba(0,0,0,0.04); }
      `}</style>
    </div>
  );
}
