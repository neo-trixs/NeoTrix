# Security Policy

## Supported Versions

The following versions of NeoTrix are currently supported with security updates:

| Version | Supported |
|---------|-----------|
| 0.18.x (latest) | ✅ Active development — receives patches |
| < 0.18 | ❌ No longer supported |

Older versions may receive critical patches on a case-by-case basis. We strongly recommend always using the latest stable release.

## Reporting a Vulnerability

We take all security vulnerabilities seriously. Please report vulnerabilities to **security@neotrix.ai** — do **not** open public GitHub issues.

We aim to:
1. **Acknowledge receipt** within **48 hours**
2. **Provide an initial assessment** within **5 business days**
3. **Issue a patched release** within **90 days** of confirmation (disclosure policy)

### PGP Key

```
Key ID:     PENDING — generated before v0.19.0 release
Fingerprint: PENDING
```

> **Note:** PGP key will be generated and published before the v0.19.0 stable release. Until then, please report vulnerabilities via email to **security@neotrix.ai** (plain-text reports accepted).

To encrypt sensitive reports, use the fingerprint above to obtain the public key from a keyserver.

## Disclosure Policy

NeoTrix follows a **90-day disclosure deadline**. When a vulnerability is reported:

1. **Triage** — We confirm receipt and classify severity (Critical / High / Medium / Low).
2. **Investigation** — We reproduce the issue and determine impact scope.
3. **Fix** — A patch is developed and reviewed internally.
4. **Release** — A patched version is published, and a [security advisory](https://github.com/neotrix/neotrix/security/advisories) is issued.
5. **Disclosure** — Full details are published after the 90-day window or when a fix is available, whichever comes first.

If the vulnerability requires more than 90 days, we will communicate an extended timeline.

## Bug Bounty

NeoTrix does not currently operate a formal bug bounty program. However, we gratefully acknowledge security researchers who responsibly disclose vulnerabilities. Significant contributions may result in public recognition (with permission) and access to early releases.

## Security Practices

- **`#![forbid(unsafe_code)]`** — All core crates (`neotrix-core`, `neotrix-types`) forbid unsafe code at the compiler level.
- **`cargo-deny`** — Dependency advisory, license, and duplicate-version checking via `deny.toml`. Vulnerability advisories are denied at the CI level.
- **`cargo audit`** — Regular auditing against the [RustSec Advisory Database](https://rustsec.org) for known vulnerabilities in the dependency tree.
- **`overflow-checks = true`** — Enabled in all profiles (debug and release) to catch integer overflows at runtime.
- **LTO** — Thin LTO enabled in release builds.
- **Secret scanning** — Automated credential detection (API keys, tokens, private keys) integrated into the SEAL pipeline.
- **Dependency minimization** — Core layer (`core/`) has zero external dependencies; external crates are audited and pinned.

## What to Expect After Reporting

1. We confirm receipt and classify severity within 48 hours.
2. We reproduce the issue and determine the impact within 5 business days.
3. A patch is developed, reviewed, and released.
4. A security advisory is published via GitHub.
5. Details are disclosed after the 90-day window, giving users time to update.
