import React, { useState } from "react";
import * as api from "../lib/api";

interface Props {
  url: string;
  domain: string;
  reason: string;
  onClose: () => void;
}

const LoginDialog: React.FC<Props> = ({ url, domain, reason, onClose }) => {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);

  const handleSubmit = async () => {
    if (!username || !password) {
      setError("请输入用户名和密码");
      return;
    }
    setSaving(true);
    setError("");

    try {
      await api.browserCredentialStore(domain, username, password);
      setSuccess(true);
      setTimeout(onClose, 1500);
    } catch (e: any) {
      setError(e?.toString() || "保存失败");
    } finally {
      setSaving(false);
    }
  };

  if (success) {
    return (
      <div className="settings-overlay" onClick={onClose}>
        <div className="settings-panel glass-panel" onClick={(e) => e.stopPropagation()} style={{ maxWidth: 420 }}>
          <div className="settings-header"><h2>✅ 已保存</h2></div>
          <div className="settings-section">
            <p style={{ opacity: 0.7, fontSize: 13 }}>
              凭据已保存。请在对话中告诉 AI 重试登录。
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel glass-panel" onClick={(e) => e.stopPropagation()} style={{ maxWidth: 420 }}>
        <div className="settings-header">
          <h2>🔑 需要登录</h2>
          <button className="btn-icon" onClick={onClose}>✕</button>
        </div>

        <div className="settings-section">
          <p style={{ opacity: 0.7, fontSize: 13, marginBottom: 16, lineHeight: 1.5 }}>
            {reason}
          </p>

          <div style={{ marginBottom: 12, fontSize: 12, opacity: 0.5 }}>
            站点: {domain}
          </div>

          {error && (
            <div style={{ color: "#FF3B30", fontSize: 12, marginBottom: 12 }}>{error}</div>
          )}

          <div className="input-group" style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 12, opacity: 0.6, display: "block", marginBottom: 4 }}>
              用户名 / 邮箱
            </label>
            <input
              type="text"
              className="input-field"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="输入用户名或邮箱"
              autoFocus
            />
          </div>

          <div className="input-group" style={{ marginBottom: 16 }}>
            <label style={{ fontSize: 12, opacity: 0.6, display: "block", marginBottom: 4 }}>
              密码
            </label>
            <input
              type="password"
              className="input-field"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="输入密码"
              onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
            />
          </div>

          <div style={{ fontSize: 12, opacity: 0.5, marginBottom: 16 }}>
            保存后请在对话中告诉 AI 重试登录操作。
          </div>

          <div className="settings-footer" style={{ borderTop: "none", padding: 0 }}>
            <button className="btn-secondary" onClick={onClose}>取消</button>
            <button className="btn-primary" onClick={handleSubmit} disabled={saving}>
              {saving ? "保存中..." : "保存凭据"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LoginDialog;
