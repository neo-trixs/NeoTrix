# Iteration Batch 749 — Email / Notification / Communication Defects

**Date:** 2026-09-07  
**Sources:** smtp.com, tiagoscarvalho.com (Exchange Online 2026), smtpedia.com, emailshield.co, mailcop.net, aaronmccarthy.com, getmailbird.com, systemdesignhandbook.com, nvecta.com, appbot.co, knowledgelib.io, singhajit.com, codelit.io, mirrorfly.com, minhmannh2001.github.io, rocket.chat

---

## Defect E-01: SMTP AUTH Basic Auth Retirement — No Migration Path

**Severity:** CRITICAL  
**Source:** getmailbird.com (Nov 2025), tiagoscarvalho.com (Jun 2026)

Microsoft 365 SMTP AUTH Basic Auth is being disabled **100% by April 30, 2026**. NeoTrix's `EmailProvider` in `nt_io_messaging.rs:207-266` stores raw `username`/`password` fields with no OAuth 2.0 flow. Post-April 2026, any Microsoft 365-hosted email will fail silently — the `send()` stub returns `Ok(msg_id)` without connecting.

**Defect:** `EmailProvider` has no OAuth2 token refresh, no per-mailbox SMTP AUTH control, no migration path to Microsoft Graph `sendMail` API or HVE (High Volume Email, GA March 2026).  
**Fix:** Add `OAuth2TokenProvider` trait, implement Microsoft Graph `sendMail` adapter, add HVE connector for internal LOB sending. The `EmailProvider` must support both SMTP+OAuth and Graph API transport.

---

## Defect E-02: No Email Validation at Entry Point — 22-28% Annual List Decay

**Severity:** HIGH  
**Source:** mailcop.net (Jun 2026), prospeo.io

Email lists decay 22-28% per year. NeoTrix has zero email validation — no syntax check, no MX lookup, no SMTP verification, no catch-all detection, no disposable email filtering. The `MessagingRouter::send()` in `nt_io_messaging.rs:351-355` passes any string as `to:` address.

**Defect:** No `EmailValidator` service. Regex-only validation is insufficient (misses typos like `gmial.com`, catch-all domains, temporary addresses).  
**Fix:** Implement 3-layer validation: (1) RFC 5322 syntax, (2) DNS/MX resolution, (3) async SMTP probe with catch-all detection. Wire into `MessagingRegistry` as pre-send guard.

---

## Defect E-03: No DMARC/DKIM/SPF Enforcement — Deliverability Blind

**Severity:** HIGH  
**Source:** tiagoscarvalho.com (Jun 2026)

DMARC enforcement is now baseline for 2026. Domains with no DMARC record or permissive DMARC receive lower trust from major receivers. NeoTrix has zero DMARC/DKIM/SPF handling in `EmailProvider`.

**Defect:** `EmailProvider` sends with no outbound authentication posture. No DKIM signing, no SPF alignment check, no DMARC policy enforcement. Emails from NeoTrix will increasingly land in spam.  
**Fix:** Add DKIM key management, SPF record validation pre-send, DMARC policy checker. Consider Microsoft Graph API path which inherits tenant's existing DKIM/SPF/DMARC posture (aaronmccarthy.com: "emails are sent as part of your Microsoft 365 environment").

---

## Defect N-01: Notification System Has No Channel Routing or Fallback

**Severity:** HIGH  
**Source:** systemdesignhandbook.com, nvecta.com (Jun 2026)

Production notification systems use channel fallback: if push fails → SMS backup → email fallback. NeoTrix's `nt_io_notify.rs` is a bare OS-notification wrapper (osascript/notify-send/PowerShell) with no:
- Channel preference per notification type
- Fallback chain (push → in-app → email)
- Delivery confirmation tracking
- Urgency-based routing

**Defect:** Single hardcoded OS notification path. No multi-channel orchestration.  
**Fix:** Implement `NotificationRouter` with channel priority chains, delivery status tracking, and fallback logic. Integrate with `MessagingRouter` from `nt_io_messaging.rs`.

---

## Defect N-02: No Notification Preference Service — GDPR/CAN-SPAM Risk

**Severity:** HIGH  
**Source:** knowledgelib.io (Feb 2026), appbot.co (Jan 2026)

2026 systems require per-user, per-channel notification preferences. A single "allow notifications?" toggle is unacceptable UX. NeoTrix has no preference model — `nt_io_notify.rs:21-23` sends unconditionally.

**Defect:** No user preference storage, no opt-in/opt-out tracking, no frequency caps, no quiet hours. Violates GDPR Article 6(1)(a) for notification consent.  
**Fix:** Create `NotificationPreferenceStore` with per-user, per-channel, per-type granularity. Add quiet hours, frequency limits, and consent audit trail.

---

## Defect N-03: No AI-Driven Notification Timing — 2026 Baseline Missing

**Severity:** MEDIUM  
**Source:** nvecta.com (Jun 2026), appbot.co (Jan 2026)

Personalized push notifications get ~4x CTR vs generic broadcasts (nvecta.com). AI-driven timing (send when user is most active) is now standard. NeoTrix's notification is synchronous fire-and-forget.

**Defect:** No user behavior analysis, no optimal send-time prediction, no engagement tracking per notification.  
**Fix:** Add `NotificationIntelligence` module that tracks user activity patterns and recommends optimal delivery windows. Wire into `NotificationRouter`.

---

## Defect C-01: No WebSocket/Real-Time Communication Layer

**Severity:** HIGH  
**Source:** minhmannh2001.github.io (Jan 2026), mirrorfly.com (Aug 2026)

Real-time chat requires WebSocket connections with <500ms delivery. NeoTrix has `neotrix-dialogue` with HTTP-based Anthropic API proxy but no WebSocket server for bidirectional real-time communication.

**Defect:** No WebSocket endpoint for real-time agent↔user or agent↔agent messaging. The `server.rs` in neotrix-dialogue is pure HTTP request-response.  
**Fix:** Add WebSocket upgrade handler in `nt_io_web/server.rs`. Implement connection manager with heartbeat, reconnection, and presence tracking.

---

## Defect C-02: No End-to-End Encryption for Inter-Agent Communication

**Severity:** MEDIUM  
**Source:** minhmannh2001.github.io (Jan 2026)

E2EE is non-negotiable for multi-agent communication trust. NeoTrix agents communicate via plaintext EventBus messages. No key exchange, no session keys, no forward secrecy.

**Defect:** Agent-to-agent messages (cross-domain coordination) travel unencrypted through internal channels. If a domain is compromised, all inter-agent traffic is readable.  
**Fix:** Implement per-session key exchange (X25519), encrypt EventBus payloads between domains, add forward secrecy via ephemeral session keys.

---

## Defect C-03: No Message Ordering Guarantees Across Domains

**Severity:** MEDIUM  
**Source:** minhmannh2001.github.io (Jan 2026)

Multi-device and cross-domain messaging requires consistent message ordering. NeoTrix's EventBus has no sequence numbering — messages can arrive out-of-order across domains.

**Defect:** `EventBus::emit` has no monotonic sequence ID. Messages from NT-CORE→NT-MIND and NT-CORE→NT-ACT may arrive in different order, causing state divergence.  
**Fix:** Add per-source sequence counter to EventBus messages. Receiver applies causal ordering (vector clocks or Lamport timestamps).

---

## Defect C-04: No Offline Message Queue with Delivery Guarantees

**Severity:** MEDIUM  
**Source:** minhmannh2001.github.io (Jan 2026)

Messages cannot be lost during network failures or server outages. NeoTrix has no persistent message queue — if a domain is unreachable, messages are dropped.

**Defect:** EventBus emit is fire-and-forget. No DLQ (Dead Letter Queue), no retry with exponential backoff, no delivery acknowledgment.  
**Fix:** Add `PersistentMessageQueue` backed by SQLite. Implement retry policy (exponential backoff + jitter), DLQ for poison messages, delivery ACK.

---

## Defect C-05: No Multi-Device Sync for Agent Sessions

**Severity:** LOW  
**Source:** minhmannh2001.github.io (Jan 2026)

Users expect seamless experience across phone, tablet, desktop. NeoTrix sessions are local-only — no cross-device message sync.

**Defect:** Session state in `neotrix-dialogue` is SQLite-local. No sync protocol for multi-device agent interaction.  
**Fix:** Add CRDT-based session sync or conflict-free merge protocol for cross-device message history.

---

## Summary

| ID | Domain | Severity | Category |
|-----|--------|----------|----------|
| E-01 | Email | CRITICAL | SMTP AUTH retirement — no OAuth2 migration |
| E-02 | Email | HIGH | No email validation (22-28% annual decay) |
| E-03 | Email | HIGH | No DMARC/DKIM/SPF enforcement |
| N-01 | Notification | HIGH | No channel routing or fallback |
| N-02 | Notification | HIGH | No preference service (GDPR risk) |
| N-03 | Notification | MEDIUM | No AI-driven timing |
| C-01 | Communication | HIGH | No WebSocket real-time layer |
| C-02 | Communication | MEDIUM | No E2EE for inter-agent |
| C-03 | Communication | MEDIUM | No message ordering guarantees |
| C-04 | Communication | MEDIUM | No offline message queue |
| C-05 | Communication | LOW | No multi-device sync |

**Total: 11 new defects** (1 CRITICAL, 5 HIGH, 4 MEDIUM, 1 LOW)
