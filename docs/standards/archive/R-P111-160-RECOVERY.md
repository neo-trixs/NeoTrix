# R-P111–R-P160 Recovery Annex (SIM-23 打捞)

> **Date**: 2026-09-21 | **Method**: 全仓 `R-P1[1-5][0-9]` 打捞，正文以模块文档注释为准。
> **Result**: 20 IDs RECOVERED (正文完整) / 1 AMBIGUOUS (R-P128 三义) / 31 STILL-MISSING.
> NTS 映射列为提案（proposal），正式映射待 NT-STD-1.1 追认。

## RECOVERED (19 + 1 ambiguous)

| ID | 正文 (harvested) | 首源 | NTS 提案映射 |
|----|-----------------|------|-------------|
| R-P110 | Internal dispatch, not CLI（内部调度禁 CLI 化） | nt_core_consciousness_core.rs:842 | NTS-B02 (facade 纪律) |
| R-P116 | 3-Tier memory（常驻 T1 / 向量归档 T2 / 回忆 T3） | tiered_memory/mod.rs:17 | NTS-F03 |
| R-P117 | ADD-only writes（只追加不 mutate） | add_only_writes/mod.rs:6 | NTS-F03 |
| R-P118 | Hybrid multi-signal retrieval（混合多信号检索） | hybrid_retrieval/mod.rs:3 | NTS-F03 |
| R-P119 | 吸收周期（外部源提取规则→解冲突→应用） | evolution_loop/absorber.rs:3 | NTS-G04 |
| R-P120 | Temporal fact windows（valid_from/valid_to） | add_only_writes/mod.rs:8 | NTS-F03 |
| R-P121 | Decay curves / pruner / salience（衰减遗忘三件套） | decay_forgetting/ | NTS-F03 |
| R-P122 | Localized maintenance（supersession 唯一写路径） | add_only_writes/mod.rs:11 | NTS-F03 |
| R-P123 | Decompose by domain（按认知域拆分） | skill_chain/mod.rs:9 | NTS-B09 |
| R-P124 | Centralized config + Default trait（配置集中） | chain_config.rs:4 | NTS-C12 |
| R-P125 | Typed agent interfaces（类型化智能体接口） | graph_orch/mod.rs:1 | NTS-B02 |
| R-P126 | Graph-based scheduling（图调度＋可观测） | graph_orch/scheduler.rs:4 | NTS-B08 |
| R-P127 | State graph checkpointing（状态图检查点） | nt_state_graph.rs:1 | NTS-E03 (audit 亲缘) |
| R-P129 | HITL gate for high-stakes（高风险人工门） | agent_guardrails/mod.rs:5 | NTS-E08 |
| R-P130 | Session replay logger（会话回放） | context_router/replay.rs:7 | NTS-E03 |
| R-P132 | Guardrails non-optional in production（生产护栏强制） | agent_guardrails/mod.rs:3 | NTS-E08 |
| R-P139 | forbid(unsafe_code)，无外部依赖，仅 std（自愈模块） | self_healing/mod.rs:9 | NTS-A01 |
| R-P141 | Governance enforcement engine（治理执行） | governance/enforcement/ | NTS-E03 |
| R-P142 | Audit logging for governance（治理审计） | governance/enforcement/audit.rs | NTS-E03 |

## AMBIGUOUS (1)

| ID | 三义并存 | 消歧决议 (文档层，不碰他人注释) |
|----|---------|-------------------------------|
| R-P128 | (a) cost-aware routing（salience.rs:4＋models.rs:10，2 处，域一致） | **R-P128a = cost-aware routing**（正典义，NTS-F01 亲缘） |
| | (b) no raw pointers（mcp_protocol/mod.rs:5，1 处） | 疑似误标 → tripwire 转模块 owner 确认/改号 |
| | (c) config-driven, not hardcoded（models.rs:4，1 处） | 实为 R-P124 复述 → tripwire 转模块 owner 改引 R-P124 |

## STILL-MISSING (31, 保持 SUSPENDED)

R-P111, 112, 113, 114, 115（治理域，5 个——仍是最大黑洞）；
R-P131, 133, 134, 135, 136, 137, 138, 140；
R-P143–160（18 个， prior wave 后段）。

## Tripwires

- R-P128b/c 改号确认（Owner：对应模块 owner，日期：M1 评审日）。
- STILL-MISSING 31 个：locate-or-reratify 延续（Owner：Architect，日期：NT-STD-1.1 规划日，
  承接 NTS-G08）。
- NTS 映射列追认（Owner：Architect，日期：NT-STD-1.1）。

---

*End of Recovery Annex v1.0 (SIM-23).*
