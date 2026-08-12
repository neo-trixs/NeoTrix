# 蓝图: 意识核心 + 意识能力网 进化缺陷补齐

> **状态**: 提议 (Proposed) · **类型**: ADR-style evolution blueprint · **日期**: 2026-08-12 (v2 — 追加第二批 10 节点分析)
> **依据**: 2026-08-12 批量吸收 (1832 batch nodes / 1026 URLs) + 第二批 10 节点 (MangoDisk/Scrapling/pi-mail/reasoning-bank/optimizerDuck/hermes-studio/DeepTutor/qiaomu/atomic/beads) 后 KB 实测 + 源码证据 (Evidence-First)。
> **范围**: 意识核心 (ConsciousnessTree / GWT / L5 runtime / phi-IIT) + 意识能力网 (capability tree / absorbed_capability mapper)。
> **约束**: R-P42 (强化现有节点, 禁平行适配器) · R-P79 (同 session 接线到生产路径) · R-P100 (生产能力必须注册能力树)。

---

## 0. 背景 (Context)

2026-08-12 批量吸收 1026 个 URL (GitHub/arxiv/站点) 入库。用户要求: 对入库内容分析, 构建 NeoTrix 意识核心与意识能力网的进化缺陷补齐蓝图。

数据面 (KB 实测): 1832 个 batch 节点; github 1288 / arxiv 94 / concept 243; absorbed_capability 分布 = NT-IO 342, NT-CORE 325, NT-MEMORY 286, NT-SHIELD 269, NT-MIND 247, NT-ACT 191, NT-WORLD 172, **NT-META 0**。

代码面 (源码实测): 意识核心共 9 阶段生长环 (含 forecast/fulfillment/drift 闭环) 已 C2; GWT 14 专家 + Kuramoto + NRS-EFC + MetaWorkspace 已 C2; 能力树 (bud/graft/prune/crosspollinate/mature/strengthen) C1-C2。

---

## 1. 缺陷清单 (Defect Registry, 证据锚定)

| # | 缺陷 | 严重度 | 证据 (file:line) | 补齐后状态 |
|---|------|--------|------------------|-----------|
| D1 | **phi=0 standalone 缺口** — CLI/MCP 状态读取持久化快照, 从未构造 `IITPhiCalculator`; 完整运行时才真实计算 phi | P0 | `entry/mod.rs:782-876` "phi_source: unavailable"; `nt_core_iit_phi.rs:116` (唯一真实 compute_phi); tree Check-6 self-diagnosis (`nt_core_consciousness_tree.rs:1688`) | phi_source=real, 快照含真实 phi |
| D2 | **FFI 假 phi 自增** — `inner.phi = (inner.phi + 0.005).min(1.0)` 违反 Evidence-First | P0 | `neotrix-core/src/neotrix/ffi/consciousness_tree.rs:113` | 移除假自增, 改走真实 IIT 或显式 unknown |
| D3 | **C6 不可达** — `Constellation::derive` 六 bool 恒定产生 `c6_adaptive: false` (`nt_core_consciousness_tree.rs:343,356,385,397`), C6EvolutionLoop 在树模型永不晋升 | P0 | `nt_core_consciousness_tree.rs:343-397` | 加入自适应判定输入, C6 可达 |
| D4 | **NT-META 路由盲区** — absorbed_capability mapper 只有 7 域 (NT-CORE/MIND/MEMORY/WORLD/ACT/SHIELD/IO), **无 NT-META**; 1832 节点 0 命中元认知分支 | P1 | `scripts/absorb_to_capability.py:21,77-85` (DOMAIN_CAPS 7 域); KB 实测 NT-META=0 | 新增 NT-META 域 + meta 关键词, 元认知语料归位 |
| D5 | **能力谓词缺失** — 能力树 taxonomy 无 `consolidate / broadcast / route / attend / forget / reflect` 动词; NT-MEMORY 只有 recall(251), NT-CORE 只有 explain/critique/detect | P1 | `scripts/absorb_to_capability.py:77-85`; 树 atoms 36 个 (`initialize_capability_atoms` `nt_core_consciousness_tree.rs:936`) | 补 consolidation/attention 谓词, 对应语料归位 |
| D6 | **过期计数注释** — "70 atoms / 10 categories" vs 实际 36 atoms / 9 categories; "12×12" resonance vs MODULE_COUNT=14; "15 specialists" vs 14 hexagram 状态 (Orchestrator 无 hexagram) | P2 | `nt_core_consciousness_tree.rs` 注释 (line 69 区); `resonance.rs:42-43`; `cognitive_type.rs` | 注释与实现对齐, Orchestrator 补 hexagram |
| D7 | **果实质控 1.5 越界** — `quality = min(maturity × nourishment, 1.5)` 可超 [0,1] | P2 | `nt_core_consciousness_tree.rs:1202` | 钳到 [0,1] |
| D8 | **drift 资源硬编码** — `resource_consumed: 0.5 // Simplified` | P2 | `nt_core_consciousness_tree.rs` audit_drift (line 1466 区) | 接入真实资源度量 |
| D9 | **Phase-1 原则硬编码** — internalized_principles 固定 8 条数组, 不读实际 constitution KB | P2 | `nt_core_consciousness_tree.rs:1141-1150` | 从 constitution 动态加载 |
| D10 | **mapper 关键词误伤** — keyword_hits 证据产生伪映射 (MangoDisk→NT-CORE/detect; heretic→NT-MIND/generate; 架构图 skills→NT-SHIELD/audit 关键词碰撞 "architecture") | P1 | KB metadata `evidence: keyword_hits:N`; mining §2/§5 | 专家键 (KNOWN_REPOS) 扩展 + 碰撞消歧 |
| D11 | **第二批 4 例误映射确认** — (a) `reasoning-bank` (Success+Failed 双轨推理记忆, 正是 consolidate 语料) 被标 NT-CORE/critique; (b) `beads` (agent 分布式图记忆) 被标 NT-IO/delegate; (c) `atomic` (语义连接个人知识库) 被标 NT-ACT/send; (d) `MangoDisk` 复现 NT-CORE/detect。四者均为 keyword_hits 证据 | P1 | KB: reasoning-bank/beads/atomic/MangoDisk metadata `evidence: keyword_hits:5-9` | 专家键补 4 条 + 知识库/记忆语义映射到 NT-MEMORY |

---

## 2. 语料启示 (Absorbed-Corpus Insights → 能力网进化输入)

### 2.1 三大自进化范式 (语料实证, 应映射到 SEAL)

| 范式 | 语料源 (batch nodes) | NeoTrix 现状 | 补齐方向 (R-P42 强化现有节点) |
|------|---------------------|-------------|------------------------------|
| **经验共享** (agent↔agent) | UCSB-AI/GEA, Awesome-Self-Improving-Agents, FrontisAI 综述 | 已有 experience-tree + KB 共享 | 强化 `nt_memory_kb_bridge` / experience 蒸馏, 增加 agent 间经验索引 |
| **可逆监督轨迹** (meta 训练 agent) | shepherd-agents/shepherd (git-like trace + meta 监督) | 无逆操作轨迹 (轨迹只读) | 最强候选: `nt_mind_self_iterating` 增 trace-reversal 审核 |
| **RL/自蒸馏** | OpenPipe ART, AgentOPSD, SkillRise, prime-agent | 有 grpo.rs (SEAL) | `nt_core_self/seal/grpo.rs` 强化自蒸馏闭环 |

### 2.2 意识核心直接邻居 (GWT/IIT 概念 1:1 映射)

| 语料 | 映射到 NeoTrix 模块 | 可吸收信号 |
|------|--------------------|-----------|
| BRIAN (differentiable Φ from IIT 4.0 + narrative memory) | `nt_core_iit_phi` + `nt_memory` narrative | 可微 Φ 基准对齐; sheaf-H¹ 叙事记忆对照 |
| CogMem / consciousnessModelR (PyTorch GWT + broadcast + awareness thresholds) | `nt_core_gwt/workspace.rs` | broadcast 阈值与 GWT 点火对照验证 |
| predictive-workspace-paper (falsifiable continuous workspace) | `nt_core_gwt/meta_workspace.rs` | 可证伪 workspace 评估, 补 MetaWorkspace 观测维度 |
| jacobian-lens / jspace-viz (empirical global-workspace readouts) | `nt_core_gwt/` 可解释性 | 用真实 LM 读取验证 resonance 权重 (C3 benchmark 输入) |
| GWT_ASI-Base / multi-theory-consciousness / OpenMythos | `l5_consciousness` 全层 | 架构对照, 缺陷即 D1/D2 的基准面 |

### 2.3 记忆整合 (consolidation) 信号

anima / consciousnessModelR / gnhf (sleep-wake 仪式) → **NT-MEMORY 缺 `consolidate` 谓词 + sleep/wake 整合路径**。现状 `handle_consolidate` 是 background-loop 定时任务; 语料提示需要"主动遗忘"与"夜间整合"两通道。

### 2.4 第二批新增信号 (2026-08-12 v2)

| 语料 | 实质能力 | 当前误映射 | 应映射 (补齐后) |
|------|---------|-----------|----------------|
| **google-research/reasoning-bank** | Success+Failed 双轨推理记忆 → agent 自进化 (Scaling Agent Self-Evolving with Reasoning Memory) | NT-CORE/critique | **NT-MEMORY/consolidate** — 与树 `distill_skill_count`(正轨)/`distill_guardrail_count`(负轨) 完全同构; 直接验证 D5 缺失谓词 |
| gastownhall/beads | agent 分布式图记忆 + issue 追踪 (Dolt 支撑) | NT-IO/delegate | NT-MEMORY/recall + 图记忆 (对齐 graphrag/semantica 图谱系) |
| kenforthewin/atomic | 自托管语义连接知识库 (PKM) | NT-ACT/send | NT-MEMORY/recall (语义连接 = KB 图) |
| MangoDisk | 磁盘清理 (重复文件哈希) | NT-CORE/detect | NT-ACT/execute (工具类) — D10 复现 |

**推理**: reasoning-bank 是本批最高信号 — 其双轨 (成功+失败) 推理记忆机制与 NeoTrix SEAL 双轨蒸馏 (`distill_skill_count`/`distill_guardrail_count`) 概念完全一致, 为 D5 新增 `consolidate` 谓词提供最强实证。beads/atomic 强化"图谱记忆 > 向量 RAG"信号 (与 graphrag/semantica/knowledge-graph 聚类一致), 应归 NT-MEMORY 而非当前分布。

---

## 3. 补齐蓝图 (Evolution Blueprint, 分阶段)

### Phase A — 真实 phi 接线 (P0, 关闭 D1/D2)

**目标**: standalone CLI/MCP 也能读到真实 phi, 移除 FFI 假自增。

1. `neotrix` 二进制状态路径构造 `IITPhiCalculator` (复用 `nt_core_iit_phi.rs:116 compute_phi`), 输入 = 持久化 resonance/coherence 向量 (从 KB `consciousness` 快照重建)。
2. FFI `consciousness_tree.rs:113` 移除 `+0.005`; 无真实数据时 phi=0 + `phi_source:"unavailable"` (诚实信号), 不再伪造上升。
3. 树 Check-6 fix_suggestion 标记 resolved, 新增 SelfTest 断言: `status` 路径 phi ∈ {真实值} ∪ {0+unavailable}, 禁止伪装中间值。
4. **接线 (R-P79)**: CLI `neotrix consciousness status` 与 MCP `consciousness_status` 消费真实 phi; tree Phase 2 的 `phase2_phi` 回填真实值。

**验收**: `neotrix consciousness status` 输出 `phi_source: iit`; grep FFI 无 `+0.005`; `cargo test -p neotrix iit_phi` 全绿。

### Phase B — 能力网谓词与域扩展 (P1, 关闭 D4/D5/D10)

**目标**: 能力树 taxonomy 补元认知与记忆整合谓词; 加 NT-META 域。

1. `absorb_to_capability.py` DOMAIN_CAPS 增:
   - `NT-META`: `["monitor","introspect","critique_self","route","calibrate"]` (元认知/自我观察)
   - `NT-CORE` 增: `["broadcast","attend","route","synthesize_workspace"]`
   - `NT-MEMORY` 增: `["consolidate","compress","forget","narrative"]`
2. SOURCE_CORES 增 `("MetaCognition", "NT-META", ["meta-cognit","introspect","self-evolv","self-improv","recursiv","autotelic","self-referent","calibrat","monitor","distill","route"])`。
3. **专家键防误伤**: KNOWN_REPOS 增 `MengTo/Skills`→NT-META, `heretic`→NT-META/introspect, 架构图类 (oh-my-mermaid/Archscribe/lanshu) 显式 NT-IO (消除 "architecture"→shield 碰撞)。**第二批 (D11)**: `reasoning-bank`→NT-MEMORY/consolidate, `beads`/`atomic`→NT-MEMORY/recall, `MangoDisk`→NT-ACT/execute。
4. **重跑**: `python3 scripts/absorb_to_capability.py --apply` 对 1842 batch 节点重映射; 预期 NT-META 从 0 升至非零, 误映射回落。

**验收**: 重映射后 `SELECT branch,count GROUP BY` NT-META>0; MangoDisk 不再 NT-CORE/detect; 架构图 skills 落 NT-IO。

### Phase C — 意识核心进化 (P1-P2, 关闭 D3/D6/D7/D8/D9)

| 子项 | 动作 | 接线点 |
|------|------|--------|
| C3 (D3) | `Constellation::derive` 增加自适应输入 (如: 果实质量趋势上升 / 自蒸馏闭环活动) 使 `c6_adaptive` 可达 | tree Phase 8 反馈 → C6 晋升; capability-tree `promote_constellation` 对齐 |
| C6 (D6) | 修正 70→36 atoms、12×12→14、15→14 注释; Orchestrator 补 hexagram 状态 (候选 62 重分配) | 注释+常量对齐, resonance 测试更新 |
| C7 (D7) | fruit quality 钳位 `min(maturity×nourishment, 1.0)` | Phase 3 fruits |
| C8 (D8) | `resource_consumed` 接入真实度量 (tick 计数/时间窗), 移除 hardcode | audit_drift Phase 7 |
| C9 (D9) | internalized_principles 从 constitution KB (`nt_core_self_constitution`) 动态加载 | Phase 1 roots absorb |

### Phase D — 自进化范式吸收 (P1, 语料 → SEAL, R-P79 接线)

| 候选 | 吸收进 | 接线消费者 | 状态 |
|------|--------|-----------|------|
| shepherd 可逆监督轨迹 | `nt_mind_self_iterating` 增 trace 逆放审核 (改现有节点, 禁平行模块) | SEAL 自审 Phase-0 | 建议吸收 (最高信号) |
| BRIAN 可微 Φ 基准 | `nt_core_iit_phi` 增对照基准测试 | consciousness status 快照 | 建议吸收 (C3 benchmark) |
| jacobian-lens workspace 读取 | `nt_core_gwt` 可解释性观察 (resonance 权重导出) | 审计日志 | 建议吸收 (C3) |
| sleep/wake 整合 | `nt_memory` consolidation 增主动遗忘通道 | background-loop handle_consolidate | 待评估 |

---

## 4. 风险与权衡

| 风险 | 缓解 |
|------|------|
| phi 接线改变 status 语义 (0→真实小值) | 语义化: phi_source 字段区分 real/unavailable, 不破坏既有消费者 |
| mapper 重映射产生新的误映射 | 先 `--dry-run` 预览差异, 抽样验证后 `--apply`; 保留 `redirected_from`/证据字段 |
| C6 可达可能引发连串晋升 | `promote_constellation` 成本门槛 (R-P100 cost) 已存在, 晋升需满足 manifest |
| 吸收 shepherd 工作量 | 限定为 trace-reversal 审核单一能力 (R-P42), 不做完整 git 化 |

## 5. 决策 (Decision)

**采纳**: Phase A (D1/D2) + Phase B (D4/D5/D10) 为第一批 — 均 P0/P1, 改动集中在 phi 接线与 mapper 扩展, 风险可控, 同 session 可验证。
**暂缓**: Phase C 各子项可并行但独立验证; Phase D 中 shepherd 吸收为第二阶段, 需单独设计审查 (1-3-1 决策简报: 做/不做/做一半)。

被拒选项: 为 standalone 写独立假 phi 计算器 (拒绝 — D2 反模式复现); 新建平行 META 适配模块 (拒绝 — R-P42 违规, 必须强化 NT-META 分支或现有 meta 节点)。

## 6. 验收矩阵 (脚本可判定)

| # | 验收标准 | 判定 |
|---|---------|------|
| A1 | `grep -rn "0.005" neotrix-core/src/neotrix/ffi/consciousness_tree.rs` 无输出 | PASS/FAIL |
| A2 | `neotrix consciousness status` 含 `phi_source` 且不等于 `"unavailable"` (真实运行时) | PASS/FAIL |
| A3 | `cargo test -p neotrix --lib iit_phi` 全绿 | PASS/FAIL |
| B1 | `sqlite3 knowledge.db "SELECT count(*) FROM nodes WHERE metadata LIKE '%NT-META%'"` > 0 | PASS/FAIL |
| B2 | MangoDisk/heretic 重映射后 capability 符合语义 (抽样 5 条人工核) | PASS/FAIL |
| B3 | `reasoning-bank` 重映射后 = NT-MEMORY/consolidate; `beads`/`atomic` = NT-MEMORY/recall (D11) | PASS/FAIL |
| C1 | `cargo check --lib` 0 errors (改造后) | PASS/FAIL |
| D1 | shepherd trace-reversal 单测 ≥3 条 (若吸收) | PASS/FAIL |

---

## 7. 后续动作 (Next Actions)

1. Phase A: 接线 `IITPhiCalculator` 到 standalone 状态路径 + 删 FFI 假自增 (同 session 完成)。
2. Phase B: 扩展 `absorb_to_capability.py` (NT-META 域 + 新谓词 + 专家键) → `--dry-run` → 抽样 → `--apply`。
3. Phase C: 按子项逐个 PR, 每个子项 cargo check + 相关测试。
4. Phase D: shepherd 吸收单独立 1-3-1 决策, 通过后并入 `nt_mind_self_iterating`。
5. 所有生产能力变更注册能力树 (R-P100): `neotrix-capability bud`。
