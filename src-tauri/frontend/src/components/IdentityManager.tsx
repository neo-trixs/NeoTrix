import React, { useState } from "react";
import GlassPanel from "./GlassPanel";

interface AccessKey {
  id: string;
  label: string;
  prefix: string;
  scope: "full" | "agent-only" | "read-only";
  status: "active" | "revoked" | "expired";
  createdAt: string;
  lastUsed: string;
}

const MOCK_KEYS: AccessKey[] = [
  { id: "key-1", label: "Claude Code Desktop", prefix: "osk-v1-a3f8...", scope: "full", status: "active", createdAt: "2026-06-28", lastUsed: "2m ago" },
  { id: "key-2", label: "CI Pipeline", prefix: "osk-v1-b7c2...", scope: "agent-only", status: "active", createdAt: "2026-06-25", lastUsed: "1h ago" },
  { id: "key-3", label: "Old Laptop", prefix: "osk-v1-d4e9...", scope: "full", status: "revoked", createdAt: "2026-06-20", lastUsed: "3d ago" },
];

const SCOPE_OPTIONS: Array<{ id: AccessKey["scope"]; label: string }> = [
  { id: "full", label: "full access" },
  { id: "agent-only", label: "agent only" },
  { id: "read-only", label: "read only" },
];

const STATUS_ICONS: Record<string, string> = {
  active: "\uD83D\uDFE2",
  revoked: "\uD83D\uDD34",
  expired: "\uD83D\uDFE1",
};

const TRUST_CHAIN = [
  {
    title: "Master Key (iCloud Keychain)",
    key: "0x8f3a...b7c2",
    badge: "\uD83D\uDFE2 Hardware-backed",
  },
  {
    title: "Device Key (This Mac)",
    key: "0xd4e9...a1b2",
    badge: "Enclave",
  },
  {
    title: "Agent Key (NeoTrix E8)",
    key: "0xf1c2...8e3d",
    badge: "Scoped: agent-only",
  },
];

function generateKey(label: string, scope: AccessKey["scope"]): AccessKey {
  return {
    id: `key-${Date.now()}`,
    label,
    prefix: `osk-v1-${Math.random().toString(16).slice(2, 10)}...`,
    scope,
    status: "active",
    createdAt: new Date().toISOString().split("T")[0],
    lastUsed: "just now",
  };
}

const IdentityManager: React.FC = () => {
  const [identityName, setIdentityName] = useState("NeoTrix User");
  const [keys, setKeys] = useState<AccessKey[]>(MOCK_KEYS);
  const [showNewKey, setShowNewKey] = useState(false);
  const [newKeyLabel, setNewKeyLabel] = useState("");
  const [newKeyScope, setNewKeyScope] = useState<AccessKey["scope"]>("agent-only");

  const revokeKey = (id: string) => {
    setKeys((prev) => prev.map((k) => k.id === id ? { ...k, status: "revoked" as const } : k));
  };

  const createKey = () => {
    if (!newKeyLabel.trim()) return;
    setKeys((prev) => [generateKey(newKeyLabel, newKeyScope), ...prev]);
    setNewKeyLabel("");
    setShowNewKey(false);
  };

  const activeCount = keys.filter((k) => k.status === "active").length;

  return (
    <div className="lg-split-panel">
      <GlassPanel
        variant="strong"
        header={<><span>\uD83D\uDD11 Identity & Keys</span><span className="lg-badge">{activeCount} active keys</span></>}
        className="lg-flex-col"
      >
        <div className="lg-card lg-card-padded identity-header-card">
          <div className="lg-label">Your Identity (secp256k1)</div>
          <div className="identity-header-row">
            <div className="identity-avatar">{identityName[0]}</div>
            <div className="identity-meta-col">
              <input
                className="lg-input identity-name-input"
                value={identityName}
                onChange={(e) => setIdentityName(e.target.value)}
              />
              <div className="identity-address">
                0x8f3a...b7c2 \u00B7 ed. 4
              </div>
            </div>
            <span className="lg-badge lg-badge-success identity-verified-badge">Verified</span>
          </div>
        </div>

        <div className="identity-section-header">
          <span className="lg-label">Access Keys</span>
          <button className="lg-btn lg-btn-primary" onClick={() => setShowNewKey(!showNewKey)}>
            {showNewKey ? "\u2715" : "\uff0b New Key"}
          </button>
        </div>

        {showNewKey && (
          <div className="lg-glass new-key-form">
            <div className="new-key-fields">
              <input
                className="lg-input"
                value={newKeyLabel}
                onChange={(e) => setNewKeyLabel(e.target.value)}
                placeholder="Key label (e.g. 'CI Server')"
              />
              <div className="new-key-scope-row">
                {SCOPE_OPTIONS.map((opt) => (
                  <button
                    key={opt.id}
                    className={`lg-btn ${newKeyScope === opt.id ? "lg-btn-primary" : ""}`}
                    onClick={() => setNewKeyScope(opt.id)}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
              <div className="new-key-actions">
                <button className="lg-btn lg-btn-primary" onClick={createKey}>Generate & Copy</button>
                <button className="lg-btn" onClick={() => setShowNewKey(false)}>Cancel</button>
              </div>
            </div>
          </div>
        )}

        <div className="lg-list lg-flex-1 lg-scroll-auto">
          {keys.map((key) => (
            <div key={key.id} className="lg-list-item">
              <div className="lg-list-item-content">
                <span>{STATUS_ICONS[key.status] || "\uD83D\uDFE2"}</span>
                <div className="key-meta-col">
                  <div className="key-label">{key.label}</div>
                  <div className="key-prefix">{key.prefix}</div>
                </div>
              </div>
              <div className="key-list-actions">
                <span className="lg-badge key-scope-badge">{key.scope}</span>
                <span className="lg-badge key-lastused">{key.lastUsed}</span>
                {key.status === "active" && (
                  <button className="lg-btn lg-btn-icon key-revoke-btn" onClick={() => revokeKey(key.id)} title="Revoke key">
                    \u2715
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      </GlassPanel>

      <GlassPanel variant="clear" header="Trust Chain" className="lg-side-panel">
        <div className="trust-chain-column">
          {TRUST_CHAIN.map((node, i) => (
            <React.Fragment key={i}>
              {i > 0 && (
                <div className="trust-connector">
                  <span>\u2503</span>
                  <span className="trust-connector-label">{i === 1 ? "\u25BC signs" : "\u25BC authorizes"}</span>
                  <span>\u2503</span>
                </div>
              )}
              <div className="lg-card lg-card-padded">
                <div className="lg-label">{node.title}</div>
                <div className="trust-key-text">{node.key}</div>
                <span className={`${node.badge.includes("Hardware") ? "lg-badge-success" : ""} lg-badge`}>{node.badge}</span>
              </div>
            </React.Fragment>
          ))}
          <div className="lg-divider" />
          <div className="lg-label">Secure Channel</div>
          <div className="secure-channel-stats">
            <div className="channel-row">
              <span>Status</span>
              <span className="lg-badge lg-badge-success">\u25CF Connected</span>
            </div>
            <div className="channel-row">
              <span>Handshake</span>
              <span className="channel-mono-text">X25519 + ChaCha20</span>
            </div>
            <div className="channel-row">
              <span>Peer</span>
              <span className="channel-mono-text">0xa3f8...d2e1</span>
            </div>
          </div>
          <button className="lg-btn pair-device-btn">
            \uD83D\uDD17 Pair New Device
          </button>
        </div>
      </GlassPanel>
    </div>
  );
};

IdentityManager.displayName = "IdentityManager";

export default IdentityManager;
