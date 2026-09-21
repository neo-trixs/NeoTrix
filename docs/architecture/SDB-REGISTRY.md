# SDB 登记表 (Stochastic-Deterministic Boundary Registry)

> **版本**: v0.2 (评分制) | **日期**: 2026-09-21 | **对应**: ROADMAP P1-03 / BLUEPRINT D-04
> **变更记**: v0.2 新增 §Verifier 评分标准（源自 Reflect/PSR/NeurIPS25，见 ABSORPTION-ROUND2 §2.2）。
> **规则**: 无 Verifier 的 LLM→动作路径一律 BLOCKER。本表是全部登记的唯一事实源。
> **状态**: 登记中 — 下表为 2026-09-21 代码实测枚举的候选站点，四格多为待确认。
> SDB 覆盖率当前为 0% (已登记 0 / 候选 7)，目标 P1 末登记 100%。

四格定义：P=Proposer(LLM) / V=Verifier(确定性) / C=Commit(持久) / R=Reject(类型化回执)。
状态：OK=四件套齐全 / PARTIAL=缺格(注明缺哪格) / UNKNOWN=待审计。

---

## 候选站点登记

| ID | 站点 (代码路径) | P | V | C | R | 状态 | 备注 |
|----|----------------|---|---|---|---|------|------|
| SDB-01 | l1_action/nt_act/agent_loop/ (manager/executor/planner) | LLM planning? | checker_executor + auditor? | UNKNOWN | UNKNOWN | UNKNOWN | 需审计：planner 提议→checker 是否为确定性 verifier |
| SDB-02 | l1_action/nt_io/nt_io_hive_agent_loop.rs | hive LLM? | UNKNOWN | outbox 有引用 | UNKNOWN | UNKNOWN | outbox 存在是 C 的起点，需确认是否为持久提交 |
| SDB-03 | l1_action/nt_act/nt_act_autonomy/ (oracle_gate) | per_agent? | oracle_gate? | UNKNOWN | UNKNOWN | UNKNOWN | gate 命名像 verifier，需确认谓词是否确定性 |
| SDB-04 | l1_action/nt_act/nt_act_trade/ (engine_traits/mod) | EngineAdapter run? | UNKNOWN | UNKNOWN | UNKNOWN | UNKNOWN | 引擎适配层，需确认 run_tier 调用点 |
| SDB-05 | l1_action/nt_core_task_dispatcher.rs | CoTGenerator? | E8Policy? | UNKNOWN | UNKNOWN | UNKNOWN | CoT+policy 同处一文件，需拆分 P/V 边界 |
| SDB-06 | l5_cognition/nt_core_prm/ (verifier.rs) | UNKNOWN | verifier.rs | UNKNOWN | UNKNOWN | UNKNOWN | PRM verifier 可能是可复用的 V 构件 |
| SDB-07 | l5_cognition/nt_goal/ (behavioral_verifier/rl_feedback) | UNKNOWN | behavioral_verifier? | UNKNOWN | UNKNOWN | UNKNOWN | 行为 verifier，需确认是否 gate 动作 |

---

## 已确认的 V 构件 (可复用)

| 构件 | 路径 | 类型 | 说明 |
|------|------|------|------|
| checker_executor | nt_act/agent_loop/checker_executor.rs | 候选 V | 待确认确定性 |
| auditor | nt_act/agent_loop/auditor.rs | 候选 V/审计 | 待确认是否阻断 |
| oracle_gate | nt_act_autonomy/oracle_gate.rs | 候选 V | 待确认谓词 |
| prm verifier | nt_core_prm/verifier.rs | 候选 V | 待确认 |
| behavioral_verifier | nt_goal/behavioral_verifier.rs | 候选 V | 待确认 |

## 已确认的 C 通道 (需推广到全系统)

| 通道 | 路径 | 说明 |
|------|------|------|
| outbox | neotrix-gateway/src/hive.rs, neotrix-multi-agent/src/hive.rs | 仅 hive 域有，L5 全域缺统一 commit 通道 |

---

## 登记方法 (审计一站点走一遍)

1. rg 找 LLM 调用点 (run_tier/EngineAdapter/chat/complete) → 填 P。
2. 沿调用链找确定性检查 (schema/策略/状态机谓词/人工门) → 填 V，无则标缺 V。
3. 找持久写 (outbox/状态/审计三件) → 填 C，无则标缺 C。
4. 找失败回执 (类型化错误码+重试指引) → 填 R，无则标缺 R。
5. 四格齐 → OK；缺格 → PARTIAL + 开补齐任务；找不到 → UNKNOWN 保持。

## 验收

- 登记 100%：候选站点无 UNKNOWN。
- 测试 100%：每个 OK/PARTIAL 站点有 SDB contract test (verifier 拒绝坏提议 + commit 可查 + reject 可读)。

---

---

## §Verifier 评分标准 (v0.2 新增)

> 源自 Reflect（宪法逐条 Likert 评分 + 阈值门）、PSR（按输入风险自适应反思深度）、
> NeurIPS25 自验证理论（误拒优于误放）、Reflexion（经验记忆有界 1–3）。
> 详见 ABSORPTION-ROUND2.md §2.2。P1-03 补齐时逐站点填写评分列。

### V-1 评分制（替代纯 pass/fail）

Verifier 对每条策略输出 Likert 1–5 分；任一条 < 3 即触发 critique+revision；
全过才 commit。分数写入 Reject/Commit 回执（可审计、可调阈值）。

### V-2 风险分级深度（Triage 联动）

- 低风险（triage  Fast 档）：快 verifier（schema + 策略表），单次通过。
- 高风险（不可逆/越权/外联）：全 critique+revision，最多 N=3 轮 + 终止信号。
- 无"无反思直通"：任何 LLM→动作路径至少一次 V 评分。

### V-3 误放厌恶（fail-closed 理论依据）

固定算力下，误放（accept bad）代价高于误拒（reject good）。
因此阈值调参只允许向严方向；放宽阈值需 ADR + 安全签字。

### V-4 经验记忆有界

Reflexion 式 revise 经验注入 experience-tree 时有界（默认 ≤3 条/任务）且带评分；
低分经验自动淘汰，不污染跨会话记忆。

### V-5 反自杀环

propose→reject 无界循环视为 DoS on self：超 N 轮自动升级人工（HITL），
audit 记 `SDB_LOOP_EXCEEDED`。

---

*维护：新增 LLM→动作点必须先登记再编码，登记号顺序分配 (SDB-08 起)。*
