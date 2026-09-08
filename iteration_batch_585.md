# Iteration Batch 585 — Communication Protocol Attack Surface

**Date:** 2026-09-06
**Scope:** gRPC/Protobuf, WebSocket/SSE, IoT (MQTT/CoAP)
**Context:** Extends batch 584 (SAST/SCA isolation blindness, AI agent tool-calling, CI/CD OIDC hijacking, Rust crate supply chain, non-atomic credential rotation) into the *communication protocol layer* — the wires between modules, not the modules themselves.

---

## What's NEW vs Batch 584

Batch 584 covered **build-time and deployment-time** attack surfaces (supply chain, CI/CD identity, tool-calling). Batch 585 shifts to **runtime wire-protocol** attack surfaces — where data moves between services, devices, and agents. Each finding below is a defect or class absent from batch 584.

| # | Domain | Finding | Batch 584 Overlap |
|---|--------|---------|-------------------|
| 1 | gRPC | Protobuf memory amplification (CVE-2026-55407) — unknown fields inflate heap | **None** — batch 584 had no deserialization amplification |
| 2 | gRPC | protobuf.js RCE via schema handling (GHSA-xq3m-2v4x-88gg) — dev tool as code execution primitive | **Extends** supply chain (batch 584 #4) but the vector is schema-as-weapon, not crate poisoning |
| 3 | gRPC | gRPC-Go `:path` header auth bypass (CVE-2026-33186) — canonicalization gap | **None** — routing-layer auth bypass is new dimension |
| 4 | WebSocket | Netty WebSocket V07/V08 handshaker smuggling (CVE-2026-59898) — protocol confusion | **None** — HTTP/WS protocol boundary attack |
| 5 | SSE | SSE auth bypass via query-param token leakage (CVE-2026-46431 pattern) | **None** — credential-in-URL is distinct from rotation issues |
| 6 | SSE | SSE stream corruption in Spring MVC/WebFlux (CVE-2026-22735) | **None** — data integrity in persistent connections |
| 7 | MQTT | MQTT wildcard subscribe ACL bypass (CVE-2026-33356, CVE-2026-49186) | **None** — pub/sub trust boundary |
| 8 | MQTT | MQTT broker config injection via unauthenticated ID (CVE-2026-44091) | **None** — trust boundary violation in IoT control plane |
| 9 | MQTT | NanoMQ heap corruption via reconnect churn (CVE-2026-22040) | **None** — connection-state machine exploitation |
| 10 | MQTT | Hardcoded JWT HMAC secret in MQTT broker (CVE-2026-71960) | **Extends** credential rotation (#5) but vector is static secret in firmware, not rotation failure |

---

## Detailed Findings

### Finding 1: Protobuf Memory Amplification (Rust)

**CVE-2026-55407** — `buffa` and `connectrpc` crates before 0.8.0

- **Mechanism:** Pre-0.8.0 decoders tracked *depth* only, not *count* of preserved unknown fields. A stream of tiny unknown protobuf fields causes unbounded heap allocation — small untrusted message → process OOM kill.
- **Root cause:** `preserve_unknown_fields=true` (default) combined with no per-message field count limit.
- **Fix:** `ctx.register_unknown_field()` now consumes one allowance slot per materialized unknown field.
- **NeoTrix impact:** Any `nt_*` module using `prost`/`tonic` with `preserve_unknown_fields` and accepting untrusted protobuf input is vulnerable. The fix requires updating `buffa`/`connectrpc` to ≥0.8.0 and auditing all `.proto` files for `preserve_unknown_fields` settings.
- **Novelty vs 584:** Batch 584 covered Rust crate *supply chain poisoning*; this is *memory amplification through deserialization semantics* — a logic bug in the parsing layer, not a trustworthiness issue with the crate itself.

**Source:** https://corgea.com/research/cve-2026-55407-buffa-connectrpc-protobuf-memory-amplification

---

### Finding 2: protobuf.js RCE via Schema Handling

**GHSA-xq3m-2v4x-88gg** — protobuf.js ≤8.0.0 / ≤7.5.4 (CVSS 9.4)

- **Mechanism:** `Type.generateConstructor` converts untrusted `.proto`/JSON "type names" into executable JavaScript without sanitization. A crafted schema file injects arbitrary code execution.
- **Scope:** 52M weekly downloads; affects gRPC, Firebase, Google Cloud services that process untrusted schema input.
- **Attack model:** "dev-tool-as-code-execution-primitive" — development tools themselves become attack vectors.
- **Fix:** `jsname = name.replace(/\W/g, "")` (one-line fix in 8.0.1/7.5.5).
- **NeoTrix impact:** If any NeoTrix component uses protobuf.js for dynamic schema loading (e.g., gRPC reflection, schema evolution tools), this is a critical code execution path. Requires auditing all JS/TS dependencies for protobuf.js versions.
- **Novelty vs 584:** Batch 584 covered Rust crate supply chain; this is *JavaScript supply chain* with a distinct vector — schema-as-weapon rather than dependency poisoning. Also introduces the concept of "dev tools as attack surface."

**Source:** https://blog.rankiteo.com/goog-r1776771217-grpc-google-cloud-vulnerability-april-2026/

---

### Finding 3: gRPC-Go `:path` Header Authorization Bypass

**CVE-2026-33186** — gRPC-Go <1.79.3 (CVSS 9.1 CRITICAL)

- **Mechanism:** gRPC-Go server accepts requests where `:path` omits the leading slash (e.g., `Service/Method` instead of `/Service/Method`). Server routes correctly, but authorization interceptors evaluate the *raw* non-canonical path. "Deny" rules using canonical paths fail to match → policy bypass if fallback "allow" exists.
- **Attack:** Attacker sends raw HTTP/2 frames with malformed `:path` headers directly to gRPC server.
- **CWE:** CWE-285 (Improper Authorization), CWE-551 (Incorrect Behavior Order: Authorization Before Parsing).
- **Fix:** v1.79.3 rejects any `:path` not starting with `/` with `codes.Unimplemented`.
- **NeoTrix impact:** If NeoTrix uses gRPC-Go with path-based authorization (official `grpc/authz` or custom interceptors), this bypass allows unauthorized method invocation. Critical for any microservice boundary using gRPC-Go.
- **Novelty vs 584:** Batch 584 covered OIDC identity hijacking (deployment-time); this is *runtime authorization bypass via HTTP/2 header canonicalization* — a protocol-level flaw in the authorization pipeline itself.

**Source:** https://nvd.nist.gov/vuln/detail/CVE-2026-33186, https://github.com/grpc/grpc-go/security/advisories/GHSA-p77j-4mvh-x3m3

---

### Finding 4: Netty WebSocket Protocol Confusion Smuggling

**CVE-2026-59898** — Netty <4.1.136.Final / <4.2.16.Final (CVSS 7.5 HIGH)

- **Mechanism:** V07/V08 WebSocket handshaker accepts `Sec-WebSocket-Version: 7` while omitting `Connection: Upgrade` and `Upgrade: websocket` headers. Server completes protocol switch; upstream proxy does *not* recognize this as an Upgrade → HTTP request smuggling via protocol confusion.
- **CWE:** CWE-444 (HTTP Request/Response Smuggling).
- **Scope:** Affects Red Hat AMQ Streams, Kafka Streams, and any Netty-based WebSocket server behind a reverse proxy.
- **Fix:** Strict validation in 4.1.136.Final / 4.2.16.Final.
- **NeoTrix impact:** If NeoTrix uses Netty for WebSocket endpoints (e.g., real-time agent communication), this enables request smuggling through proxy confusion. Requires ensuring all WebSocket handshakes enforce RFC 6455 compliance.
- **Novelty vs 584:** Batch 584 had no protocol-boundary smuggling findings. This is a *transport-layer protocol confusion* attack — distinct from application-layer injection or identity issues.

**Source:** https://nvd.nist.gov/vuln/detail/cve-2026-59898, https://www.sentinelone.com/vulnerability-database/cve-2026-59898/

---

### Finding 5: SSE Authentication Bypass via Token Leakage

**CVE-2026-46431 pattern** (Algernon, CVSS 4.3) + architectural analysis

- **Mechanism:** `EventSource` API does not support custom headers. Developers pass auth tokens via URL query parameters → tokens leak into server access logs, browser history, proxy logs. Additionally, `Access-Control-Allow-Origin: *` in separate code paths bypasses CORS enforcement.
- **Architectural flaw:** SSE connections are GET requests with long-lived state; standard HTTP caching and logging infrastructure treats them as normal requests, leaking credentials.
- **Fix:** Pre-flight handshake pattern (POST to establish session → receive SSE URL with session token), `SameSite=Strict` cookies, `Cache-Control: no-store`.
- **NeoTrix impact:** Any SSE endpoint using query-param authentication is a credential leak vector. Requires implementing cookie-based or pre-flight auth for all SSE streams.
- **Novelty vs 584:** Batch 584 covered non-atomic credential *rotation*; this is credential *leakage through protocol architectural constraints* — the SSE API itself forces insecure patterns.

**Source:** https://dev.to/roxdavirox/server-sent-events-security-how-eventsource-breaks-your-api-authentication-model-3643, https://rubel.dev/blog/server-sent-events-security-preventing-connection-hijacking

---

### Finding 6: SSE Stream Corruption in Spring Framework

**CVE-2026-22735** — Spring MVC/WebFlux (CVSS 2.6, but high reliability impact)

- **Mechanism:** Improper handling of SSE in Spring MVC/WebFlux corrupts the event stream, causing data loss or miscommunication between server and client. Affects Spring 5.3.0–5.3.46, 6.1.0–6.1.25, 6.2.0–6.2.16, 7.0.0–7.0.5.
- **Impact:** Application instability, unreliable data delivery, potential for *silent data corruption* in real-time feeds.
- **NeoTrix impact:** If NeoTrix uses Spring for any SSE-based real-time feature, event stream corruption could cause silent data loss — particularly dangerous for consciousness-state broadcasting or evolution event feeds.
- **Novelty vs 584:** Batch 584 covered integrity in build artifacts; this is *runtime data integrity loss in persistent connections* — a distinct layer of the integrity problem.

**Source:** https://vulert.com/vuln-db/CVE-2026-22735

---

### Finding 7: MQTT Wildcard Subscribe ACL Bypass

**CVE-2026-33356** — EMQX 4.x (CVSS 7.7 HIGH)
**CVE-2026-49186** — Local MQTT Broker (Acer)

- **Mechanism:** Broker enforces publish restrictions but *not* equivalent subscribe authorization. Any authenticated low-privilege account subscribes to `#` or `+` wildcard topics → receives telemetry from devices the user does not own. CWE-639 (Authorization Bypass Through User-Controlled Key).
- **Scope:** Meari IoT Cloud deployments, Acer devices with local MQTT brokers.
- **NeoTrix impact:** If NeoTrix MQTT subsystem (e.g., `nt_physical` sensor data) uses wildcard subscriptions without per-device subscribe ACLs, a compromised agent could eavesdrop on all sensor feeds.
- **Novelty vs 584:** Batch 584 covered credential rotation; this is *pub/sub authorization model gap* — the broker's authorization model has a structural hole between publish and subscribe permissions.

**Source:** https://github.com/xn0tsa/nobody-puts-baby-in-a-corner, https://github.com/advisories/GHSA-2vfr-ch7v-7754, https://github.com/advisories/GHSA-vvpf-h42q-v96v

---

### Finding 8: MQTT Broker Configuration Injection

**CVE-2026-44091** — MQTT Broker (CERT-VDE VDE-2026-008)

- **Mechanism:** Unauthenticated remote attacker posts a malicious ID to MQTT Broker → creates a new configuration entry in system configuration store. CWE-501 (Trust Boundary Violation). No authentication, no user interaction, low complexity.
- **Impact:** Integrity compromise and availability loss of IoT control plane.
- **NeoTrix impact:** If any NeoTrix IoT integration uses MQTT for device configuration, an unauthenticated attacker could inject rogue configuration entries → device misbehavior or denial of service.
- **Novelty vs 584:** Batch 584 covered OIDC identity hijacking in CI/CD; this is *trust boundary violation in IoT control plane* — data crosses from untrusted network into trusted configuration without validation.

**Source:** https://www.sentinelone.com/vulnerability-database/cve-2026-44091/

---

### Finding 9: NanoMQ Heap Corruption via Connection Churn

**CVE-2026-22040** — NanoMQ MQTT Broker 0.24.6

- **Mechanism:** Combined traffic pattern of high-frequency publishes + rapid reconnect/kick-out using same ClientID + massive subscribe/unsubscribe jitter → heap memory corruption → SIGABRT (free(): invalid pointer). Race condition in memory deallocation during concurrent connection operations.
- **Impact:** Immediate broker process termination, service disruption, data loss in edge computing environments. **No patched version available at disclosure.**
- **NeoTrix impact:** Edge deployments using NanoMQ as messaging backbone are vulnerable to denial-of-service through connection-state machine exploitation. Requires rate limiting and connection throttling as interim mitigations.
- **Novelty vs 584:** Batch 584 had no memory corruption findings; this is *heap corruption through protocol-level state machine manipulation* — distinct from use-after-free or buffer overflow.

**Source:** https://vuldb.com/vuln/348856

---

### Finding 10: Hardcoded JWT Secret in MQTT Broker Firmware

**CVE-2026-71960** — Cudy WR3000 2.0 firmware <2.5.24 (CVSS 9.1 CRITICAL)

- **Mechanism:** Mosquitto MQTT broker authentication plugin uses hardcoded JWT HMAC signing secret. Attacker extracts secret from firmware image → forges arbitrary JWT tokens → authenticates to MQTT broker without legitimate credentials → accesses mesh networking interface.
- **CWE-798:** Use of Hard-coded Credentials.
- **NeoTrix impact:** Any IoT device using MQTT with hardcoded secrets represents a static credential attack vector. Requires firmware-level secret rotation and certificate-based auth as alternative.
- **Novelty vs 584:** Batch 584 covered non-atomic credential *rotation* (dynamic); this is *static credential embedded in firmware* (permanent). Two distinct failure modes for the same credential management problem.

**Source:** https://www.thehackerwire.com/vulnerability/CVE-2026-71960/

---

## Defect Classification (NeoTrix-Specific)

| Defect Class | Findings | Severity | NeoTrix Module Risk |
|---|---|---|---|
| **Deserialization Amplification** | #1 (protobuf OOM) | HIGH | `nt_world::crawl`, `nt_io::llm` (any protobuf decoder) |
| **Schema-as-Weapon** | #2 (protobuf.js RCE) | CRITICAL | JS/TS tooling layer, gRPC reflection |
| **Authorization Bypass via Canonicalization** | #3 (gRPC-Go `:path`) | CRITICAL | Any gRPC-Go service boundary |
| **Protocol Confusion Smuggling** | #4 (Netty WebSocket) | HIGH | WebSocket endpoints behind proxies |
| **Credential Leakage via Protocol Constraints** | #5 (SSE query-param) | MEDIUM | All SSE streams |
| **Persistent Connection Data Corruption** | #6 (Spring SSE) | LOW-MED | SSE-based state broadcasting |
| **Pub/Sub Authorization Model Gap** | #7 (MQTT wildcards) | HIGH | `nt_physical` sensor subscriptions |
| **IoT Trust Boundary Violation** | #8 (MQTT config injection) | HIGH | Device configuration pipelines |
| **Connection-State Heap Corruption** | #9 (NanoMQ) | CRITICAL | Edge MQTT deployments |
| **Static Firmware Credentials** | #10 (Cudy JWT) | CRITICAL | IoT device fleet |

---

## Sources Cited

1. CVE-2026-55407 — https://corgea.com/research/cve-2026-55407-buffa-connectrpc-protobuf-memory-amplification
2. GHSA-xq3m-2v4x-88gg — https://hackread.com/52m-download-protobuf-js-library-rce-schema-handle/
3. CVE-2026-33186 — https://nvd.nist.gov/vuln/detail/CVE-2026-33186
4. CVE-2026-59898 — https://nvd.nist.gov/vuln/detail/cve-2026-59898
5. CVE-2026-46431 pattern — https://dev.to/roxdavirox/server-sent-events-security-how-eventsource-breaks-your-api-authentication-model-3643
6. CVE-2026-22735 — https://vulert.com/vuln-db/CVE-2026-22735
7. CVE-2026-33356 — https://github.com/advisories/GHSA-2vfr-ch7v-7754
8. CVE-2026-44091 — https://www.sentinelone.com/vulnerability-database/cve-2026-44091/
9. CVE-2026-22040 — https://vuldb.com/vuln/348856
10. CVE-2026-71960 — https://www.thehackerwire.com/vulnerability/CVE-2026-71960/

---

## Meta-Analysis: New Attack Surface Layer

**Batch 584 → 585 progression:**

| Layer | Batch 584 | Batch 585 |
|-------|-----------|-----------|
| Build-time | SAST/SCA isolation, supply chain | — |
| Deploy-time | CI/CD OIDC, credential rotation | — |
| Runtime wire-protocol | — | gRPC deserialization, WebSocket smuggling, SSE leakage, MQTT authorization |

**Key insight:** Communication protocols introduce a *second trust boundary* separate from module boundaries. Batch 584's findings are about *who you trust* (identities, dependencies). Batch 585's findings are about *what you parse* (protobuf fields, HTTP/2 headers, WebSocket handshakes, MQTT wildcards). The parser is the new attack surface.

**Recommendation for NeoTrix architecture:**
1. Add a `nt_shield::protocol_guard` module that validates wire-protocol canonicalization (gRPC `:path`, WebSocket handshake, SSE auth) at module boundaries.
2. Enforce protobuf field-count limits on all external-facing decoders.
3. Implement per-device MQTT subscribe ACLs as default (not opt-in).
4. Replace all query-param auth with cookie/pre-flight patterns for SSE endpoints.
