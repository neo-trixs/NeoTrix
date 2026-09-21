# ADR-0001: Pre-commit P0 门单次 bypass（环境 OOM＋他人既有错误）

---
status: accepted
date: 2026-09-21
decision-makers: [neo]
consulted: []
informed: [team]
sim-id: SIM-20
quality-attributes: [reliability]
requirements: []
supersedes: []
superseded-by: []
---

## Context and Problem Statement

提交治理套件（33 文件：NT-STD/蓝图/SIM/脚本/模板/配置）与宪法接线代码时，
`.githooks/pre-commit` P0 门（`cargo check --tests -p neotrix`）失败。
SIM-20 查明失败与本次改动无关：4 个编译错误全在他人文件中，
另伴随 OOM Killer（`Killed: 9`，本机内存不足以跑完全量 check）。

## Decision Drivers

- 本次改动无可编译 Rust 增量：33 文件中唯一 `.rs` 是 docs 下的模板（crate 外，不参与编译）；
  宪法文件改动已 11/11 单测实跑全绿（SIM-16）。
- 门禁失败是环境性的（OOM）叠加既有红（他人 4 错），修门禁≠修本次改动。
- 用户已明确批复 bypass 路径（2026-09-21）。

## Considered Options

- 修 4 个他人错误再提交：越权改他人代码＋验证周期超本机能力（OOM 未解），否决。
- 只交非 `.rs` 部分：模板 `.rs` 照样触发门禁，问题延后不解决，否决。
- `--no-verify`＋书面 override＋CI 兜底：范围最小、证据最全，采用。

## Decision Outcome

Chosen option: **两次 `--no-verify` 提交（文档批＋宪法代码批），commit trailer 注明 override，
SIM-20＋本 ADR 为双证**，因为门禁失败与改动无关且有更强等价证据（11/11）。

### Consequences

- Good: 治理资产准时入库，不被无关红灯扣押；override 全程留痕可审计。
- Bad: 本地失去一次全量 check 背书——由 CI 同门补回（tripwire）。
- Neutral: 不开先例：下次 bypass 仍需独立 SIM＋批复。

## Verification

- [ ] Fitness/script gate: N/A（本次无门禁相关改动；模板 `.rs` 经 `bash -n` 语法位为空——纯模板无可执行位）
- [ ] Test: constitution 11/11（SIM-16 实跑）；`cargo check --tests` 由 CI 在 push 后执行
- [ ] Metric: CI 同门变绿前，本 ADR 状态保持 accepted-pending-CI（口头约定，见 tripwire）

## Links

- SIM: [SIM-20](../architecture/SIM-PROTOCOL.md)
- Blueprint: [D-05 八门](../architecture/NEOTRIX-MASTER-BLUEPRINT.md)（本次绕行的门）
- Code: [nt_core_self_constitution.rs](../../neotrix-core/src/l6_meta/nt_core_self_constitution.rs)（11/11 覆盖）
