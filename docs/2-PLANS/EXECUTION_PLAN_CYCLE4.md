# NeoTrix Cycle 4: 9支柱重构执行计划

> **时间**: 2026-07-01 | **范围**: 代码修复 + 架构重构 + 盲点补齐

---

## 阶段0: 代码缺陷修复（已执行）

| # | 操作 | 状态 |
|---|------|------|
| 1 | 删除 `nt_act_mcp.rs` (精确重复 `nt_agent_mcp_discovery.rs`) | ✅ |
| 2 | 验证 `nt_agent_protocol/` 目录存在 (5文件) | ✅ 存在 |
| 3 | 验证 `nt_agent_subagent/` 目录存在 (4文件) | ✅ 存在 |
| 4 | 架构文档: ARCHITECTURE_9PILLAR.md | ✅ |
| 5 | 经验树: EXPERIENCE_TREE_2026-07-01.md | ✅ |
| 6 | TODO清单: TODO.md | ✅ |

---

## 阶段1: 9支柱核心改造 (当前Cycle 4 Sprint)

### 并行执行计划

由于多数改造在独立模块中, 可以多任务并行:

```
Track A: NT-CORE 推理核 (P0级别)
  A1: Observer PRM头 (64→32→1)        ~80 LOC
  A2: Policy GRPO (G=4-8)             ~150 LOC
  A3: GWT竞争点火                     ~50 LOC
  A4: GWT 5层压缩                     ~300 LOC
  A5: E8→VSA嵌入                      ~120 LOC
  A6: SAE→E8推理流集成                 ~100 LOC

Track B: NT-AGENT 代理运行时 (P0级别)
  B1: MCP 4传→2传 (废弃WS+SSE)        ~150 LOC
  B2: Planner/Executor/Reflector分离   ~200 LOC

Track C: NT-PROVIDER LLM服务层
  C1: nt_io_provider→nt_provider迁移   ~重命名
  C2: 混合编排 (tiered routing)        ~300 LOC

Track D: NT-MIND 自我进化
  D1: DPOStage                        ~150 LOC
  D2: ConstitutionalSelfCritiqueStage  ~120 LOC
  D3: SafetyCheckStage                 ~80 LOC
  D4: ProceduralMemoryStage            ~150 LOC
```

### 执行顺序建议

```
Week 1: A1+A2+A3+B1 (可并行, 无依赖)
Week 2: A4+A5+B2 (依赖A2/A3的架构决策)
Week 3: A6+D1+D2 (依赖A1 PRM数据流)
Week 4: D3+D4+C1+C2 (依赖前期)
```

---

## 阶段2: 外部研究循环 (持续)

### Cycle 4 研究已完成
```
✓ 竞争项目分析: AutoGPT/LangGraph/CrewAI/smolagents/Letta/AutoGen/OpenCog/EIDOS/Lincoln/ExCortex
✓ 论文吸收: 12主题45+论文
✓ 代码缺陷发现: 5个 (已修2)
```

### Cycle 5 研究方向建议

| 方向 | 原因 | 预期收获 |
|------|------|---------|
| **C5-1: A2A协议** (Google/TensorLake) | MCP更新后需agent间协议 | agent间互操作标准 |
| **C5-2: 神经形态计算** (Loihi 2, Spiking NN) | VSA/谐振器网络在神经形态硬件上极高效 | 功耗降低1000×路径 |
| **C5-3: 形式化验证** (TLA+/Coq) | E8过渡矩阵安全性保障 | 数学证明保证推理正确性 |
| **C5-4: 差分隐私** (ε-DP) | 过程记忆含敏感数据 | 隐私保护记忆 |
| **C5-5: GPT-OSS MoE** (128专家路由) | HyperCube作为路由表验证 | 学习路由替代硬编码 |
| **C5-6: 多模态世界模型** (V-JEPA 2.1) | 感官输入+语言+代码多模态 | 统一表示空间 |

---

## 阶段3: 架构验证与质量门

每个PR/提交前必须通过:

```
1. cargo check                     → 0 error
2. cargo clippy                    → 0 warning
3. cargo test -p neotrix --lib     → 0 regression
4. cargo test -p neotrix-types     → 0 regression
5. npm test                        → 0 regression (frontend)
```

---

## 阶段4: 收尾 - 更新环

每次Sprint完成:

```
1. 更新 EXPERIENCE_TREE  → 记录新盲点发现
2. 更新 TODO.md          → 标记完成项 + 追加减速项
3. 更新 ARCHITECTURE_9PILLAR.md → 反映实际代码状态
4. 更新 AGENTS.md        → 同步9支柱命名约定
5. CHANGELOG.md          → 版本记录
```

---

## 关键里程碑

| 里程碑 | 目标日期 | 验收标准 |
|--------|---------|---------|
| M1: 核心修复 | Day 1 | nt_act_mcp删除 + 文档完成 |
| M2: PRM+GRPO | Week 1 | E8自适应搜索 ≤3步解决之前6步问题 |
| M3: GWT竞争点火 | Week 1 | 单个专家胜出率 >90% |
| M4: 5层压缩 | Week 2 | GWT上下文从∞→≤16K tokens |
| M5: ProceduralMemory | Week 3 | 成功E8序列→技能→再调用成功率>70% |
| M6: 混合编排 | Week 4 | 本地fallback + 成本感知路由 |
| M7: 编译绿线 | 持续 | cargo check 0 error |
