# ADR-0003: 退役两个 theater 适应度守卫

---
status: accepted
date: 2026-09-21
decision-makers: [neo]
consulted: []
informed: [team]
sim-id: SIM-34
quality-attributes: [maintainability]
requirements: []
supersedes: []
superseded-by: []
---

## Context and Problem Statement

`LayerBoundaryFitness` 扫描 `src/neotrix/l1_body_impl`（不存在），
`CoreBoundaryFitness` 扫描 `src/core`（不存在）——两守卫恒为 Ok，
其 allowlist 引用的部分文件亦已搬迁或消失。恒绿的守卫比没有守卫更坏：
它提供虚假的安全感，且其单测（"allowlist 覆盖全仓"）空转通过。

## Decision Drivers

- ConfidenceLabelFitness（B2，已 14/14 实跑绿）覆盖真实 `src/l*_*/` 目录，
  功能为二者的超集（方向判定＋干净对回归）。
- `check-layer-deps.sh` 覆盖同范围（秒级，CI 可跑）。
- 修（重定向扫描根＋重写 allowlist）比重写还贵，且语义已由后继者承担。

## Considered Options

- 重定向＋重写 allowlist：工作量大，且与 B2/脚本三重覆盖，否决。
- 保留现状：虚假安全感持续，否决。
- 退役两者（删 struct＋注册＋专属单测），由 B2＋脚本承接：采用。

## Decision Outcome

Chosen option: **退役 LayerBoundaryFitness 与 CoreBoundaryFitness**，
因为后继覆盖已实跑验证，保留只剩误导价值。

### Consequences

- Good: 守卫注册表无 theater 项；`arch_fitness_tests` 调用方（self_test_integration、SEAL pipeline）行为不变（删的是恒 Ok 项）。
- Bad: 若未来需要 neotrix/ 旧树边界检查，需重写（YAGNI 前提下接受）。
- Neutral: 文件头"P0 八个守卫"清单同步改为六个有效＋两个退役注记。

## Verification

- [ ] 删除后 `cargo test --lib nt_core_arch_fitness` 全绿
- [ ] 注册表长度 8→6（5 旧有效＋B2＋...如数）
- [ ] 本 ADR＋SIM-34 为证

## Links

- SIM: [SIM-34](../architecture/SIM-PROTOCOL.md)
- Code: [nt_core_arch_fitness.rs](../../neotrix-core/src/l5_cognition/nt_core_arch_fitness.rs)
