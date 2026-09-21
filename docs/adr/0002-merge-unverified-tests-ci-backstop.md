# ADR-0002: 未本地验证的单测随码合入，首绿由 CI 见证

---
status: accepted
date: 2026-09-21
decision-makers: [neo]
consulted: []
informed: [team]
sim-id: SIM-27
quality-attributes: [reliability, maintainability]
requirements: []
supersedes: []
superseded-by: []
---

## Context and Problem Statement

P1-05 Confidence 类型＋3 单测已写完（fmt-clean＋静态复核），但本机无法跑完验证：
单 crate 71 万行全量重编超 30 分钟／轮，文件锁被协作者构建长期持有，
本机内存曾 OOM Kill。无限重试＝烧轮次而无进展。

## Decision Drivers

- 代码是纯新增（L0 类型＋cfg(test) 单测），零行为变更，blast radius 为零。
- 静态证据齐全：rustfmt 解析通过、类型闭合复核、serde 路径与同文件先例一致。
- 拖延合入的代价（工作丢失风险、协作不可见）大于"CI 首绿"的形式完备性。

## Considered Options

- 继续蹲锁重试：每轮 30 分钟，上限未知，已耗三轮，无进展，否决。
- 拆小验证（如抽文件独立编译）：单 crate 结构不支持，伪验证，否决。
- 合入＋CI 首绿＋tripwire：采用（即本决策）。

## Decision Outcome

Chosen option: **合入，单测首绿由 CI 见证**，因为证据结构（静态全绿＋零行为面）
已达到合入标准，缺的仅是本机跑不完的执行确认。

### Consequences

- Good: P1-05 落地可见，SIM-09 链解冻，协作者可基于类型继续工作。
- Bad: 若 CI 首红，需要一次修复提交——已接受为计划内成本（tripwire）。
- Neutral: 不开"免验证合入"先例：仅适用于纯新增＋静态全绿＋CI 兜底三条件齐备。

## Verification

- [ ] CI `cargo test -p neotrix --lib nt_core_cross_layer` 首绿（3/3）
- [ ] fmt 门保持（仅历史漂移）
- [ ] tripwire：首红 24h 内修复（Owner：P1 owner）

## Links

- SIM: [SIM-27](../architecture/SIM-PROTOCOL.md)
- Code: [nt_core_cross_layer.rs](../../neotrix-core/src/l0_substrate/nt_core_cross_layer.rs) (Confidence § + 3 tests)
- Precedent: [ADR-0001](0001-precommit-bypass-oom-preexisting.md)（同类 override 程序）

## Resolution (2026-09-21, EQ-01)

本地实跑 3/3 绿（`cargo test --lib nt_core_cross_layer`，SIM-34），
pending-CI 转 closed。CI 首绿仍欢迎作为复核，但不再阻塞。本 ADR 关闭。
