# ADR-0005: SEAL 威胁模型骨架（六件＋五问＋分级）

---
status: proposed
date: 2026-09-22
decision-makers: [L5 待签]
consulted: [L1]
informed: [team]
sim-id: SIM-42
quality-attributes: [safety]
requirements: [P1-03]
supersedes: []
superseded-by: []
---

## Context and Problem Statement

SEAL（红队对子）有 pair ledger 传统（NTS-E09，AgentHarm 先例）与候选阈值
（R10：0.35/0.85/0.6/0.7 未校准），缺威胁模型骨架：测什么、按何顺序、
何谓通过。本草案只定骨架，不定阈值（阈值校准是 P-task L5）。

## Decision Drivers

- ARES 六件（R11）：scope firewall／dry-run-before-traffic／单次 HMAC／
  taint 隔离／role gating／evidence redaction。
- trust-boundary 五问（R10）：谁触发／给什么工具／何处执行／读什么凭据／如何停止。
- deepteam Agentic 分类（R9）：目标窃取／递归劫持／越权／多智能体信道污染／漂移。
- ATR／asamm／AISVS（R12 全景）：规则与成熟度外标。

## Considered Options

- 一次建全模型＋全对子：blast radius 大，否决。
- 只要 ledger 不要模型：对子无骨架，分数不可比，否决。
- 骨架先行（本案）：六件为门、五问为表、分类为纲；对子逐批校准：采用。

## Decision Outcome

Chosen option: **三层骨架**。
L1 门（六件）：任一缺失即 BLOCKER，不进入评分。
L2 表（五问）：每个被测动作填表，无表不测。
L3 纲（Agentic 五类）：每类至少一对子；新增类别需 ADR。
校准纪律：阈值初值引用必注"未校准"（R9/R10 诚实口径）；校准用同一评分器重算（AERS）。

### Consequences

- Good: 对子可比、可审计； BLOCKER 前置，浪费减半。
- Bad: 首轮只有骨架无分数，安全感不增加（诚实）。
- Neutral: 与 ADR-0004 翻转门正交，各走各的签字。

## Verification

- [ ] L5 签字（本 ADR proposed→accepted）
- [ ] 首批对子落地（P-task，阈值注未校准）
- [ ] 本 ADR＋SIM-42 为证

## Links

- SIM: [SIM-42](../architecture/SIM-PROTOCOL.md)
- Round: [ROUND11](../architecture/ABSORPTION-ROUND11.md)（S-75/62/70）
