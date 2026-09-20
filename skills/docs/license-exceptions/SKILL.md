# License Exceptions

## Purpose
专有模块许可例外

## Trigger Words
- license
- proprietary
- license exceptions
- 许可证
- 专有模块

## Content
- 专有模块列表
- 许可边界
- SPDX 标记

## File
`LICENSE-EXCEPTIONS.md`

## Proprietary Modules
| Module | Path | Rationale |
|--------|------|-----------|
| NT-SHIELD Security Layer | `nt_shield*/` | Security-critical defensive tooling |
| ConsciousnessTree Meta-Cognition | `nt_mind*` | Self-evolving reasoning core (IP) |
| VSA HyperCube Core | `nt_core_hcube*` | Knowledge representation engine (IP) |

## Boundary
- Module is "proprietary" only if listed AND carries `SPDX-License-Identifier: LicenseRef-NeoTrix-Proprietary`
