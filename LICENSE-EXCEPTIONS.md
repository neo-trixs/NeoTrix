# LICENSE-EXCEPTIONS.md — Proprietary Module Exceptions

This repository is dual-licensed. The **root LICENSE** file grants the MIT
License to the public portions of the codebase. The modules listed below are
**proprietary** and are **excluded** from the MIT License. They may not be
copied, modified, distributed, or used in derivative works (including compiled
binaries) without explicit written permission from NeoTrix.

## Proprietary Modules

| Module | Path | Rationale |
|--------|------|-----------|
| **NT-SHIELD Security Layer** | `neotrix-core/src/neotrix/l1_body_impl/nt_shield*/`, `nt_shield_*.rs` | Stealth-net, proxy-pool, Tor client, fingerprint management — security-critical defensive tooling. Keeping this proprietary preserves the threat-model advantage of the toolkit and prevents misuse of evasion tooling. |
| **ConsciousnessTree Meta-Cognition** | `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind*` | The self-evolving reasoning core (SEAL pipeline, self-healing, evolution loop). This is NeoTrix's differentiating IP. |
| **VSA HyperCube Core** | `neotrix-core/src/core/nt_core_hcube*` | Vector Symbolic Architecture knowledge representation engine. Core research IP. |

## Boundary

- A module is "proprietary" only if listed above **and** its source file
  carries the SPDX marker `SPDX-License-Identifier: LicenseRef-NeoTrix-Proprietary`
  in its header.
- All other files (including CLI, GUI, frontend, docs, and integration code)
  remain MIT-licensed per the root LICENSE.
- If a listed module lacks the SPDX marker, the file is treated as MIT for
  practical compatibility; the table above is the authoritative intent list.

## Contact

For commercial licensing of the proprietary modules, contact:
- GitHub: https://github.com/neo-trixs/NeoTrix

---

© 2026 NeoTrix. All rights reserved for the proprietary modules listed above.