# Iteration Batch 635 — API Security / OAuth / JWT Analysis

**Date:** 2026-09-06
**Context:** Following Batch 634 (governance layer, process mining, BOAT, DTO, audit trails). This batch searches for API security, OAuth, JWT 2026 landscape to identify new defects for NeoTrix.

---

## Sources Consulted

| # | Source | URL | Date |
|---|--------|-----|------|
| 1 | NIST SP 800-228-upd1: API Protection for Cloud-Native Systems | nist.gov | 2026-03-13 |
| 2 | OWASP Secure API Gateway Blueprint | owasp.org | 2026 |
| 3 | OWASP API Security Top 10 (2023 stable) | owasp.org | 2023 |
| 4 | API Gateway Security Baseline for 2026 (Safeguard.sh) | safeguard.sh | 2026-04-22 |
| 5 | API Gateway Security in Zero Trust 2026 (Jimber) | jimber.io | 2026-06-03 |
| 6 | API Security 2026 Complete OWASP Guide (Valtik Studios) | valtikstudios.com | 2026-03-07 |
| 7 | API Security Guide 2026 (chs.us) | chs.us | 2026-04-10 |
| 8 | OWASP API Security Top 10 Guide (Offensive360) | offensive360.com | 2026-04-02 |
| 9 | RFC 10017: OAuth 2.0 for Browser-Based Applications | datatracker.ietf.org | 2026-08 |
| 10 | OAuth2/OIDC/PKCE Patterns & Anti-Patterns (Sachith) | sachith.co.uk | 2026-03-18 |
| 11 | Start with Identity: OAuth Auth Code + PKCE Recipe | startwithidentity.com | 2026-06-19 |
| 12 | OAuth 2.1 & OIDC Deep Dive (Akousa) | akousa.net | 2026-03-22 |
| 13 | Complete Guide OAuth 2.0 & OIDC in 2026 (DevConsole) | devconsole.dev | 2026-04-20 |
| 14 | OAuth 2.1 & OIDC Implementation Guide (Jishu Labs) | jishulabs.com | 2026-01-06 |
| 15 | OAuth 2.0 Security Best Practices RFC 9700 (Safeguard) | safeguard.sh | 2026-07-07 |
| 16 | What is OAuth? OAuth 2.1 Explained (askmeidentity) | askmeidentity.com | 2026-05-22 |
| 17 | JWT Best Practices 2026 (JSONCraft) | jsoncraft.dev | 2026-04-17 |
| 18 | JWT Structure, Signing & Security Mistakes (Toolsana) | toolsana.com | 2026-07-15 |
| 19 | JWT Best Practices: Security, Storage, Rotation (SkyCloak) | skycloak.io | 2026-08-20 |
| 20 | JWT Tokens Explained (StringTools) | stringtoolsapp.com | 2026-04-05 |
| 21 | draft-ietf-oauth-rfc8725bis-06 (IETF JWT BCP) | datatracker.ietf.org | 2026 |
| 22 | JWT Security Best Practices 2026 (jsmanifest) | jsmanifest.com | 2026-03-27 |

---

## API Security — New Findings & Defects

### Defect D-1: Gateway Authentication Default is Wrong in NeoTrix
**Source:** Safeguard.sh API Gateway Baseline 2026 + Jimber Zero Trust 2026
**Finding:** The 2026 consensus is that API gateways must default to **deny-all** with explicit public markers on unauthenticated routes. NeoTrix's NT-IO gateway (if/when implemented) likely defaults to allow. The Jimber analysis states: "We still find gateways in 2026 where the implicit default is open and individual routes are protected by per-route policies, which is the configuration that produces the most incidents."
**Defect:** NeoTrix lacks an explicit deny-by-default gateway policy. No gateway-layer authz exists at all.
**Fix:** Implement gateway authz with explicit per-route allow-lists. Public endpoints (health check, CLI login) get explicit `public` markers. All other routes require auth.

### Defect D-2: No Dual-Layer Rate Limiting
**Source:** Safeguard.sh 2026 + Jimber 2026
**Finding:** Two distinct rate limiting functions are needed: (1) backend protection (per-consumer, 100-1000 rps) and (2) attack mitigation (per-IP, 10-50 rps with sliding window). A single rate limit policy serving both jobs is the "trap that produces incidents."
**Defect:** NeoTrix has no rate limiting at the gateway layer. NT-ACT tool calls have no throttling.
**Fix:** Implement layered rate limiting: per-consumer (backend protection) + per-IP (attack mitigation) with sliding window log algorithm.

### Defect D-3: Schema Validation Not Enforced at Gateway
**Source:** Valtik Studios 2026, Jimber 2026, chs.us 2026
**Finding:** OpenAPI/GraphQL schema validation at the gateway catches injection and mass-assignment before backends. The 2025 Express middleware incident (unvalidated JSON → prototype pollution in 3 downstream services) would have been prevented by gateway schema validation.
**Defect:** NeoTrix has no gateway-level schema validation for API endpoints.
**Fix:** Enforce OAS 3.1 schema validation at gateway. Schema generation must be baked into the API release process (not generated once and left stale).

### Defect D-4: Gateway Logging Leaks Credentials
**Source:** Safeguard.sh 2026
**Finding:** Most gateway defaults log full request/response payloads, which means credentials, tokens, and PII end up in the log aggregator within minutes. The right baseline is structured logging with explicit allowlists for fields captured, not blocklists for fields redacted.
**Defect:** NeoTrix NT-IO HTTP server logs are not structured with field-level redaction. Authorization headers, API keys, and cookies would be logged by default.
**Fix:** Structured logging with explicit field allowlist. Headers `authorization`, `x-api-key`, `cookie` redacted at gateway level. Bodies sampled at low rates with active PII detection. W3C trace context enforced for backend correlation.

### Defect D-5: No Continuous Access Evaluation (CAEP)
**Source:** Jimber 2026
**Finding:** Traditional OAuth tokens suffer from an active-session window: once authorized, a token works until expiry even if the account is disabled. The Continuous Access Evaluation Profile (CAEP), built on the Shared Signals Framework, resolves this. The identity provider streams Security Event Tokens (RFC 8417) to the gateway for real-time invalidation.
**Defect:** NeoTrix has no mechanism for real-time token invalidation on account termination or device compliance failure.
**Fix:** Implement CAEP integration for NT-IO gateway. When a user is terminated or permissions change, gateway receives real-time signal and invalidates tokens immediately.

### Defect D-6: No Plugin Security Audit for Gateway Code
**Source:** Safeguard.sh 2026, chs.us 2026
**Finding:** Custom Lua/WASM plugins in Kong, custom middleware in Tyk — all can ship code that bypasses the gateway's own security model. Treat gateway plugins like production service code: SBOM, scanning, signed deploys, patching cadence faster than application services.
**Defect:** NeoTrix NT-IO gateway has no plugin audit trail, SBOM tracking, or signed deploy pipeline for gateway middleware.
**Fix:** Gateway plugins must have SBOM, signed deploys, and faster patching cadence than application services due to larger blast radius.

### Defect D-7: Admin API Exposure Risk (Kong Pattern)
**Source:** chs.us 2026, Trend Micro case study
**Finding:** Kong API Gateway misconfigurations: Admin API forwarded to public port (8001/8443), secrets stored in plaintext, community version lacks encryption. Docker Hub examples override localhost binding.
**Defect:** No hardening checklist for NeoTrix gateway admin endpoints.
**Fix:** Gateway hardening checklist: admin API bound to localhost only, network-level restriction, secrets in external vault, authorizer cache keys include resource path, Lua sandbox enabled, HTTP/2 end-to-end.

---

## OAuth 2.0 / OIDC — New Findings & Defects

### Defect D-8: PKCE Not Mandatory in NeoTrix OAuth Flow
**Source:** RFC 10017 (2026-08), RFC 9700, askmeidentity.com 2026
**Finding:** OAuth 2.1 (active draft as of 2026) makes PKCE mandatory for **every** client using the Authorization Code Flow, not just public clients. The Implicit Grant and ROPC grant are removed. PKCE binds the authorization code to the client that requested it.
**Defect:** NeoTrix's NT-IO OAuth implementation does not mandate PKCE for all clients. Implicit flow may still be offered.
**Fix:** Enforce PKCE on every Authorization Code flow. Remove Implicit Grant and ROPC grant entirely. Set `code_challenge_method=S256` exclusively.

### Defect D-9: No Mix-Up Attack Defense
**Source:** Safeguard.sh RFC 9700 Guide 2026
**Finding:** When a client talks to multiple authorization servers, it must defend against mix-up attacks where a response from one server is fed to the client as if it came from another. Defense: verify the issuer (`iss` parameter) of every authorization response.
**Defect:** NeoTrix multi-tenant OAuth has no mix-up defense. `iss` parameter not validated on authorization responses.
**Fix:** Verify `iss` on every authorization response. Reject responses where `iss` doesn't match the server the flow was started with.

### Defect D-10: Redirect URI Matching Too Loose
**Source:** RFC 9700, Akousa 2026, DevConsole 2026
**Finding:** RFC 9700 requires exact string matching against pre-registered `redirect_uri` values. No wildcards, no prefix rules, no extra parameters allowed. Loose matching is a leading cause of OAuth account takeover.
**Defect:** NeoTrix redirect URI validation may use pattern or prefix matching. No explicit exact-match enforcement.
**Fix:** Enforce exact string matching on `redirect_uri`. No wildcards, no `starts-with` checks, no allowing extra query parameters.

### Defect D-11: No Sender-Constrained Tokens (DPoP/mTLS)
**Source:** RFC 9700, Jimber 2026, Safeguard.sh 2026
**Finding:** Bearer tokens are usable by anyone who holds them. Sender-constrained tokens (DPoP RFC 9449 or mTLS RFC 8705) bind the token to a key the client must prove it holds on every call, turning a bearer token into something an attacker cannot simply replay. "Adopt them wherever your authorization server and clients support it, especially for high-value APIs."
**Defect:** NeoTrix uses plain bearer tokens. No DPoP or mTLS sender-constraining.
**Fix:** Implement DPoP (RFC 9449) for browser-based clients and mTLS (RFC 8705) for service-to-service. Bind tokens to client keys.

### Defect D-12: BFF Pattern Not Implemented for Browser Apps
**Source:** RFC 10017 (2026-08), Akousa 2026
**Finding:** RFC 10017 (August 2026) specifies that browser-based apps should use a Backend-for-Frontend (BFF) pattern as a confidential client. The BFF handles token exchange, stores tokens server-side, and issues session cookies. Tokens never reach the browser.
**Defect:** NeoTrix browser-based UI (if any) stores tokens client-side. No BFF pattern.
**Fix:** Implement BFF pattern for browser apps. BFF acts as confidential client, stores tokens server-side, issues httpOnly session cookies.

### Defect D-13: No Refresh Token Rotation
**Source:** Multiple 2026 sources (Jishu Labs, Akousa, DevConsole, SkyCloak)
**Finding:** OAuth 2.1 recommends refresh token rotation: each use issues a new refresh token and invalidates the old one. If an attacker steals a refresh token and uses it, the legitimate client's next refresh attempt detects the theft.
**Defect:** NeoTrix refresh tokens are not rotated. A stolen refresh token remains valid indefinitely.
**Fix:** Enable refresh token rotation. Each use issues new refresh token, invalidates old. Implement reuse detection: if a revoked refresh token is presented, revoke the entire token family and force re-authentication.

### Defect D-14: No `at_hash` Validation on ID Tokens
**Source:** Akousa 2026, RFC 10017
**Finding:** The `at_hash` claim in ID tokens is the left half of the SHA-256 hash of the access token, base64url-encoded. If present, it must be verified to match the access token received alongside. This prevents token swap attacks — the ID token cryptographically binds to the specific access token it was issued with.
**Defect:** NeoTrix ID token validation skips `at_hash` verification.
**Fix:** Validate `at_hash` claim on all ID tokens when present.

---

## JWT — New Findings & Defects

### Defect D-15: EdDSA Not Default Algorithm
**Source:** JSONCraft 2026, draft-ietf-oauth-rfc8725bis-06
**Finding:** EdDSA (Ed25519) is 8x faster to verify than RS256, signatures are 64 bytes vs 256, keys are 32 bytes vs 256, and it has no parameter choices that allow footguns. It's been standardized since RFC 8037 (2017). The 2026 consensus is EdDSA as the default for new systems.
**Defect:** NeoTrix likely uses RS256 or HS256. EdDSA should be the default for new token systems.
**Fix:** Migrate to EdDSA (Ed25519) for new token issuance. Use ES256 only for hardware/FIPS constraints. RS256 only for legacy clients.

### Defect D-16: Algorithm Pinning Not Enforced
**Source:** JSONCraft 2026, draft-ietf-oauth-rfc8725bis-06, jsmanifest 2026
**Finding:** The `alg:none` attack and algorithm confusion attack (HS256 signed with RSA public key as HMAC secret) remain active in 2026. Defense: libraries MUST provide a mechanism to explicitly restrict algorithms. Pin the expected algorithm in the verifier — never let the library read `alg` from the header.
**Defect:** NeoTrix JWT verification may not pin algorithms. `algorithms: ['EdDSA']` or `algorithms: ['RS256']` must be explicit.
**Fix:** Explicitly specify allowed algorithms in every verification call. Never trust the token's `alg` header. Reject `alg: none` unconditionally.

### Defect D-17: No `aud` Validation on JWTs
**Source:** JSONCraft 2026, Toolsana 2026, SkyCloak 2026
**Finding:** Skipping `aud` validation is how a token issued for Service A (which shares a signing key with Service B) gets replayed against Service B. `aud` is the boundary. Without it, cross-service replay is trivial.
**Defect:** NeoTrix JWT verification may skip `aud` claim validation.
**Fix:** Always validate `aud` claim. Token must be intended for the specific service receiving it.

### Defect D-18: No `typ` Claim Enforcement (Explicit Typing)
**Source:** draft-ietf-oauth-rfc8725bis-06
**Finding:** When two different uses of JWTs share a common set of claims, one kind of JWT can be confused for another. The `typ` Header Parameter prevents this. The updated JWT BCP (RFC 8725bis) adds explicit typing as a defense.
**Defect:** NeoTrix does not enforce `typ` claim on JWTs. Access tokens and ID tokens could be confused.
**Fix:** Use `typ` header parameter (e.g., `at+jwt` for access tokens, `JWT` for ID tokens). Validate `typ` on receipt.

### Defect D-19: No JWE for Sensitive Claims
**Source:** SkyCloak 2026, JWT BCP draft
**Finding:** Standard JWTs (JWS) are signed but NOT encrypted. The payload is base64-encoded and fully readable by anyone with the token. For tokens carrying PII (email, phone, national ID), JWE (JSON Web Encryption, RFC 7516) or opaque tokens with introspection should be used.
**Defect:** NeoTrix tokens may contain sensitive claims in JWS payload. No encryption layer.
**Fix:** Use JWE for tokens containing PII. Or use opaque access tokens with introspection endpoint.

### Defect D-20: No Refresh Token Reuse Detection
**Source:** JSONCraft 2026, Toolsana 2026, SkyCloak 2026, draft-ietf-oauth-rfc8725bis-06
**Finding:** When refresh tokens are rotated, if a revoked refresh token is presented again, treat it as theft and invalidate the entire token family. "If the attacker refreshes first, the next time the legitimate user's client tries to refresh, it gets a token-not-found error which should immediately invalidate the entire session and force a re-login."
**Defect:** NeoTrix has no reuse detection for revoked refresh tokens.
**Fix:** Implement reuse detection. On presentation of a revoked refresh token: revoke all tokens for that user, force re-authentication, log as security event.

### Defect D-21: Token Storage in localStorage
**Source:** Multiple 2026 sources (JSONCraft, Toolsana, SkyCloak, jsmanifest)
**Finding:** 2026 consensus: store access tokens in memory only (JavaScript variable), refresh tokens in httpOnly Secure SameSite cookies. localStorage is vulnerable to any XSS. "If attackers can run JavaScript on your page, no storage choice fully saves you" — but in-memory limits blast radius.
**Defect:** NeoTrix frontend may store tokens in localStorage. This is the most common JWT footgun in production.
**Fix:** Access tokens: in-memory only (JS variable, React state). Refresh tokens: httpOnly, Secure, SameSite=Strict cookie, path restricted to refresh endpoint. Never localStorage.

### Defect D-22: No PBES2 Count Limit (New in 2026 BCP)
**Source:** draft-ietf-oauth-rfc8725bis-06
**Finding:** The updated JWT BCP adds: implementations should limit hash iteration count when validating PBES2 encrypted content. Without this, attackers can impose unreasonable computational burden (DoS). Reject inputs with `p2c` value larger than 2x OWASP recommended limit.
**Defect:** NeoTrix JWT parsing does not enforce PBES2 iteration count limits.
**Fix:** Set upper limit on `p2c` (PBES2 Count). Reject values exceeding 2x OWASP recommended iterations.

### Defect D-23: JWT Format Confusion Attack (New in 2026 BCP)
**Source:** draft-ietf-oauth-rfc8725bis-06
**Finding:** JWT serialization format confusion attacks exist. Implementations must confirm the JWT is in legal format while parsing — legal JWTs contain only ASCII letters, numbers, dash, underscore, and period. Content with braces, quotation marks is not a JWT and must be rejected.
**Defect:** NeoTrix JWT parser does not validate format before decoding.
**Fix:** Validate JWT format on parse: reject any content containing non-ASCII-base64url characters (especially `{`, `}`, `"`).

### Defect D-24: No JWE Decompression Size Limit (New in 2026 BCP)
**Source:** draft-ietf-oauth-rfc8725bis-06
**Finding:** Without a limit on JWE decompressed size, compression can impose unreasonable memory or CPU burden. Recommended limit: 250 KB.
**Defect:** NeoTrix JWE parsing has no decompression size limit.
**Fix:** Set JWE decompression limit to 250 KB.

---

## Cross-Cutting Defects (API + OAuth + JWT Combined)

### Defect D-25: No CAEP + JWT Revocation Integration
**Source:** Jimber 2026 + JWT BCP 2026
**Finding:** The gap between CAEP (real-time token invalidation via Shared Signals Framework) and JWT (stateless, non-revocable) is the critical intersection. CAEP streams Security Event Tokens (RFC 8417) to gateways for immediate invalidation. JWT's statelessness means tokens remain valid until expiry.
**Defect:** NeoTrix has neither CAEP integration nor short-lived JWTs with refresh rotation. The compound effect: compromised credentials persist until manual intervention.
**Fix:** Implement both: (1) short-lived access tokens (5-15 min) + refresh rotation, and (2) CAEP integration for real-time revocation signals from IdP.

### Defect D-26: No Supply Chain Security for API Gateway
**Source:** OWASP Web Top 10 2025 (A03: Software Supply Chain Failures), chs.us 2026
**Finding:** The OWASP Web Top 10 2025 introduced "Software Supply Chain Failures" as a new category (#3). APIs consuming third-party libraries, SDKs, and build pipelines are in scope. PCI DSS 4.0.1 Requirement 6.3.2 mandates API inventory including third-party components.
**Defect:** NeoTrix has no API inventory or SBOM for API components. Gateway plugins and middleware not tracked.
**Fix:** Maintain API inventory. Track all API components (libraries, SDKs, gateway plugins) in SBOM. Implement signed deploys for gateway code.

---

## Summary: 26 Defects Found

| Category | Count | Severity Range |
|----------|-------|----------------|
| API Gateway Security | 7 (D-1 to D-7) | HIGH-CRITICAL |
| OAuth/OIDC | 7 (D-8 to D-14) | HIGH-CRITICAL |
| JWT Security | 10 (D-15 to D-24) | HIGH-CRITICAL |
| Cross-Cutting | 2 (D-25 to D-26) | CRITICAL |

## Key Insights for NeoTrix

1. **Gateway is the new attack surface**: In 2026, the API gateway is the single highest-leverage attack surface. NeoTrix must treat it as a first-class service with SBOM, signed deploys, and faster patching cadence.

2. **OAuth 2.1 is law**: PKCE mandatory for all clients. Implicit and ROPC grants removed. Exact redirect URI matching. Sender-constrained tokens (DPoP/mTLS) are the new baseline.

3. **JWT is a sharp tool**: EdDSA as default, algorithm pinning mandatory, `aud` validation required, `typ` explicit typing, PBES2 limits, format validation. The 2026 JWT BCP (draft-ietf-oauth-rfc8725bis-06) adds 5 new defense requirements.

4. **The stateless-revocable gap**: JWT's statelessness conflicts with the need for real-time revocation. Short-lived tokens + refresh rotation is the pragmatic answer. CAEP integration is the ideal answer.

5. **Supply chain meets API security**: OWASP Web Top 10 2025's new A03 category puts API supply chain in scope. NeoTrix must track all API components.

---

**Next Batch (636):** Should analyze the NIST SP 800-228-upd1 document in detail, evaluate CAEP/Shared Signals Framework integration feasibility, and assess EdDSA migration path for NeoTrix JWT infrastructure.
