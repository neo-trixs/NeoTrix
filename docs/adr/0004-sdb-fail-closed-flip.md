# ADR-0004: SDB-05 阻断翻转路径（分谓词三阶段）

---
status: proposed
date: 2026-09-22
decision-makers: [L5 待签]
consulted: [L1]
informed: [team]
sim-id: SIM-42
quality-attributes: [safety, reliability]
requirements: [P1-03]
supersedes: []
superseded-by: []
---

## Context and Problem Statement

SDB-05（dispatcher）现处 log-only：三 verifier 谓词 VP-1~3＋τ=0.65 已定，
评分首填 1/2/2/1/1（SDB-REGISTRY v0.6，SIM-42）。SDB 铁律要求 LLM→动作路径
最终有确定性 Verifier 把关；NTS V-3 规定阈值只许调严。问题：以何顺序、
何门槛把 log-only 翻成 fail-closed，而不把省预算的快路径误杀。

## Decision Drivers

- 三谓词皆确定性、无 LLM 参与，开销可忽略——翻转不引入新模型风险。
- VP-3（状态钳位）零语义风险；VP-2（τ 门）影响分发走向；VP-1（快路径门）
  直触成本节省路径，误拒＝多烧 LLM 预算。风险排序 VP-3 < VP-2 < VP-1。
- V-5 反自杀环：翻转后 propose→reject 循环必须有计数＋HITL 上限，否则阻断即 DoS on self。

## Considered Options

- 三谓词一次全翻：blast radius 不可控，否决。
- 永守 log-only：SDB 铁律落空（无 Verifier 路径长期 BLOCKER），否决。
- 分谓词三阶段＋浸泡（Stage 1 VP-3 → Stage 2 VP-2 → Stage 3 VP-1）：采用。

## Decision Outcome

Chosen option: **分阶段翻转**，每阶段准入三条件 pile（缺一即停）：
CI 全绿＋7 天干净日志＋L5 书面签；τ 调参钳位 [0.65, 1.0]（调松需另立 ADR＋安全签字）；
实现必须带单旗回滚（模式位切回 log-only，无需代码回退）＋V-5 循环计数（超 N=3 轮转 HITL，
audit 记 `SDB_LOOP_EXCEEDED`）。

### Consequences

- Good: 风险按 VP-3→VP-1 渐进暴露；每阶段可独立回滚。
- Bad: 全程走完需数周浸泡＋三次签字，慢。
- Neutral: log-only 日志格式不变（SDB v0.6 §评分首填），翻转只改执行位。

## Verification

- [ ] Stage 1：CI 绿＋7d 日志＋签字，VP-3 翻转，回滚旗实测有效
- [ ] Stage 2：同上，VP-2 翻转，τ 调参记录可审计
- [ ] Stage 3：同上，VP-1 翻转，快路径误拒率有数
- [ ] 本 ADR 状态 proposed→accepted（签字后改）

## Links

- SIM: [SIM-42](../architecture/SIM-PROTOCOL.md)
- Registry: [SDB-REGISTRY v0.6](../architecture/SDB-REGISTRY.md)
