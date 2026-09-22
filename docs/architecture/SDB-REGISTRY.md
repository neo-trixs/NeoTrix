# SDB 登记表 (Stochastic-Deterministic Boundary Registry)

> **版本**: v0.6 (EQ-09/10 落地) | **日期**: 2026-09-22 | **对应**: ROADMAP P1-03 / BLUEPRINT D-04
> **变更记**: v0.6 dispatcher 三 verifier 谓词＋阈值＋log-only 方案（SIM-42）；
> SDB-06/07 接线对象确定；评分列首填（V-1~V-5 vs SDB-05）。
> v0.5 V 构件定级 (SIM-29)：SDB-06/07 均为真 verifier（接线待定）。
> v0.3 实点登记 (SIM-27)：SDB-04 退役（误报）＋ dispatcher 3 实点＋ loops 待追。
> v0.2 新增 §Verifier 评分标准（源自 Reflect/PSR/NeurIPS25，见 ABSORPTION-ROUND2 §2.2）。
> **规则**: 无 Verifier 的 LLM→动作路径一律 BLOCKER。本表是全部登记的唯一事实源。
> **状态（诚实口径 v0.4）**: 活 P 点 1 组（dispatcher 442/890/1001）＋ 脚手架 3
> （loops 确定性模板，无 LLM 调用，暂不适用）＋ 退役 1（SDB-04）＋ V/C 复用构件 2 组；
> **V/C/R 实现 0，verifier 一行未编码**。规则是目标态，不是现状——现状见本表，目标见 §Verifier。
> 收敛结论（SIM-28）：SDB 首战对象只有 dispatcher 一处，loops 接线时重登记。

四格定义：P=Proposer(LLM) / V=Verifier(确定性) / C=Commit(持久) / R=Reject(类型化回执)。
状态：OK=四件套齐全 / PARTIAL=缺格(注明缺哪格) / UNKNOWN=待审计 / RETIRED=误报退役 / SCAFFOLD=无LLM调用暂不适用。

---

## 候选站点登记

| ID | 站点 (代码路径) | P | V | C | R | 状态 | 备注 |
|----|----------------|---|---|---|---|------|------|
| SDB-01 | l1_action/nt_act/agent_loop/ (manager/executor/planner) | 无（execute_step 系 format! 拼接） | checker/auditor 系断言检查器（非 LLM verifier） | — | — | SCAFFOLD | 确定性模板规划；接 LLM 时重登记为新 SDB 号 |
| SDB-02 | l1_action/nt_io/nt_io_hive_agent_loop.rs | 无（仅 mock 注释） | — | outbox 引用存（C 候选） | — | SCAFFOLD | 接 LLM 时重登记；outbox 可复用 |
| SDB-03 | l1_action/nt_act/nt_act_autonomy/ (oracle_gate) | 无（确定性 Gap 枚举） | oracle_gate 系 gap 推理（非 LLM） | — | — | SCAFFOLD | 接 LLM 时重登记 |
| SDB-04 | ~~nt_act_trade (TradeEngineRegistry)~~ | — | — | — | — | RETIRED | 误报：TradeEngine 系交易引擎注册表，非 LLM 调用点 (SIM-27 实测退役) |
| SDB-05 | l1_action/nt_core_task_dispatcher.rs | `.complete()`×3 (L442/890/1001) ＋ CoTGenerator 接线 | VP-1 快路径门：class==Deterministic ∧ kernel.is_some() ∧ pred_confidence≥0.65（代码 L700 实证）；VP-2 置信阈值 τ=0.65（E8 predict_next）；VP-3 状态钳位 hexagram&0x3f | dispatch 结构化日志（log-only；outbox 推广 P-task） | TaskDispatchError 类型化回执（已有） | PARTIAL (V 谓词已定分未打；C log-only) | 首个实名 P 点；三谓词 log-only 先行，阻断待 P1-03（SIM-42/EQ-09） |
| SDB-06 | l5_cognition/nt_core_prm/ (verifier.rs) | — (评估器，非动作路径) | V-CONFIRMED：6 维 MCTS 步评估（ModeConsistency/TransitionPattern/RewardHistory/DirectionChange/OscillationCheck/StepPosition） | — | — | V-READY (已接线：SDB-05 decompose_task 步评估) | 接线对象确定＝SDB-05 分解步（SIM-42/EQ-10）；log-only 起步 |
| SDB-07 | l5_cognition/nt_goal/ (behavioral_verifier/rl_feedback) | — (门控对象为代码修改) | V-CONFIRMED：编译＋测试＋属性三重门＋run_bounded 超时 kill＋RL 奖励信号 | — | — | V-READY (已接线：未来代码修改类动作 SDB-08+；dispatcher 暂不适用) | 接线对象确定＝代码修改动作（SIM-42/EQ-10）；新 SDB 号登记时生效 |

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

## §评分首填 (EQ-09；SIM-42；基线口径，P1-03 复测)

> 方法：同一评分器重算、不自报（AERS/R9 信条）。低分是基线，不是事故。

| 标准 | SDB-05 现状 | 首填分 (1–5) |
|------|-------------|--------------|
| V-1 评分制 | 谓词已定（VP-1~3），无 Likert 评分器 | 1 |
| V-2 风险分级 | Fast/高风险未分级，直通单路径 | 2 |
| V-3 误放厌恶 | log-only 未阻断；阈值收紧方向已定（只许调严） | 2 |
| V-4 经验有界 | 经验记忆未接 experience-tree | 1 |
| V-5 反自杀环 | 无循环计数/HITL 升级 | 1 |

log-only 记录格式（每 dispatch 一条）：`{pred_confidence, τ=0.65, class, kernel_present, vp1_pass, vp2_pass, vp3_pass, decision}`；
阻断翻转需 P1-03＋ADR（V-3 只许调严）。

---

*维护：新增 LLM→动作点必须先登记再编码，登记号顺序分配 (SDB-08 起)。*
