# NeoTrix 进化迭代缺陷补齐任务清单 — 2026-09-11

## 一、研究发现的核心缺口（按 P0/P1/P2 分级）

### P0 — 立即补齐（阻塞核心进化）

| # | 缺口 | 研究来源 | NeoTrix 目标 | 状态 |
|---|------|---------|-------------|------|
| **D1** | **3-Cadence Hook System** | Grok Build, harness-pi, OpenHarness | `nt_io/hooks.rs` — SessionStart/End, PreToolUse/PostToolUse, PreCompact/PostCompact | ❌ 未实现 |
| **D2** | **Cache-Aware Compaction Boundaries** | harness-pi, CacheRouter paper | `nt_io/llm/cache.rs` — 基于缓存命中率的智能 compaction | ❌ 未实现 |
| **D3** | **Pass-by-Reference Tool Results** | NVIDIA NOOA | `nt_act/reference_view.rs` — 活对象预览而非序列化 | ❌ 未实现 |
| **D4** | **Trinity Memory (向量+图+关系)** | TencentDB Agent Memory | `nt_memory/trinity.rs` — 统一向量+图+关系存储 | ❌ 仅部分实现 |
| **D5** | **Context-as-Filesystem** | OpenViking | `nt_memory/context_fs.rs` — viking:// URI 风格分层加载 | ❌ 未实现 |
| **D6** | **Typed I/O Contracts** | NVIDIA NOOA | `nt_core/agent/io_contract.rs` — 工具调用类型化契约 | ❌ 未实现 |
| **D7** | **Automated Harness Evolution** | NVlabs/SoL-Pi | `nt_mind/seal/harness_evolution.rs` — 可验证环境驱动优化 | ❌ 未实现 |
| **D8** | **ACP Protocol** | Grok Build | `nt_io/acp.rs` — JSON-RPC over stdin/stdout IDE 集成 | ❌ 未实现 |

### P1 — 高价值（增强进化能力）

| # | 缺口 | 研究来源 | NeoTrix 目标 | 状态 |
|---|------|---------|-------------|------|
| **D9** | **AEGIS 4-Stage Evolution** | HarnessX | `nt_mind/seal/aegis.rs` — Digester→Planner→Evolver→Critic | ❌ 未实现 |
| **D10** | **Decision Intelligence** | Semantica | `nt_governance/decision_graph.rs` — W3C PROV-O 审计链 | ❌ 未实现 |
| **D11** | **Trajectory-as-a-Skill** | TRIAGE paper | `nt_memory/trajectory_store.rs` — 轨迹存储为可复用技能 | ❌ 未实现 |
| **D12** | **Semantic Taint Tracking** | Semantica | `nt_shield/taint.rs` — 语义+因果溯源替代字符串匹配 | ❌ 未实现 |
| **D13** | **Subagent Worktrees** | Grok Build | `nt_act/worktree.rs` — 子代理隔离上下文窗口 | ❌ 未实现 |
| **D14** | **Agent-Initiated Context Mgmt** | ACM, ContextPilot | `nt_core/context/initiative.rs` — agent 主动管理上下文 | ❌ 未实现 |
| **D15** | **Continual Harness Refinement** | Prime Agent | `nt_mind/harness/refinement.rs` — `/refine` 能力 | ❌ 未实现 |
| **D16** | **Three-Type Memory** | Grok Build | `nt_memory/memory_types.rs` — Workflow/Subtask/Function 三类 | ❌ 未实现 |

### P2 — 中等价值（锦上添花）

| # | 缺口 | 研究来源 | NeoTrix 目标 | 状态 |
|---|------|---------|-------------|------|
| **D17** | **Cordis Plugin Composability** | DeepSeek Harness | `nt_core/plugin.rs` — 类型化事件+可逆效果 | ❌ 未实现 |
| **D18** | **Model-Callable Harness APIs** | NVIDIA NOOA | `nt_core/harness_api.rs` — 模型可检查/管理的上下文 API | ❌ 未实现 |
| **D19** | **RLM Recursive Delegation** | PrimeAgent | `nt_core/rlm.rs` — 上下文作为变量，子代理作为递归函数 | ❌ 未实现 |
| **D20** | **Dream Consolidation** | Grok Build | `nt_memory/dream.rs` — 睡眠期记忆整合 | ❌ 未实现 |

---

## 二、已完成模块验证

| 模块 | 文件 | 行数 | 编译 | 测试 | 待验证 |
|------|------|------|------|------|--------|
| AddressableStore | `nt_memory/addressable_store.rs` | ~200 | ✅ | ❌ | 需集成测试 |
| ContextSandbox | `nt_io/context_sandbox.rs` | ~200 | ✅ | ❌ | 需集成测试 |
| ProceduralGraph | `seal/procedural_graph.rs` | 980 | ✅ | ✅ 16/16 | 需与 SEAL 集成 |
| Distillation | `seal/distillation.rs` | 310 | ✅ | ✅ 4/4 | 需与 Crystallization 集成 |
| Crystallization | `seal/crystallization.rs` | ~300 | ✅ | ✅ 6/6 | 需与 ProceduralGraph 集成 |
| MCP Server | `nt_io/mcp_server.rs` | ~150 | ✅ | ✅ 3/3 | 需接入 DomainRegistry |
| Drift Detection | `gateway/drift.rs` | 334 | ✅ | ✅ 9/9 | 需接入 GatewayV2 |
| Context Assembly | `nt_core/context_assembly.rs` | 237 | ✅ | ❌ | 需接入 GWT |
| GWT Budget Routing | `attention_head.rs` | +100 | ✅ | ❌ | 需接入 GatewayV2 |
| MTRouter History | `attention_head.rs` | +80 | ✅ | ❌ | 需接入 GatewayV2 |

---

## 三、并行 Agent 路线图

### Wave 1: 基础补齐（4 agents 并行）

```
Agent A: Hook System + ACP Protocol
├── 创建 nt_io/hooks.rs (3-cadence hook system)
├── 创建 nt_io/acp.rs (ACP JSON-RPC protocol)
├── 修改 nt_io/mod.rs 添加模块
└── 验证编译

Agent B: Cache-Aware Compaction + Context-as-Filesystem
├── 创建 nt_io/llm/cache.rs (cache-aware compaction)
├── 创建 nt_memory/context_fs.rs (viking:// URI style)
├── 修改对应 mod.rs
└── 验证编译

Agent C: Typed I/O Contracts + Pass-by-Reference
├── 创建 nt_core/agent/io_contract.rs (typed contracts)
├── 创建 nt_act/reference_view.rs (live object preview)
├── 修改对应 mod.rs
└── 验证编译

Agent D: Harness Evolution (SoL-Pi pattern)
├── 创建 nt_mind/seal/harness_evolution.rs
├── 实现可验证环境驱动优化
├── 修改 seal/mod.rs
└── 验证编译
```

### Wave 2: 进化增强（4 agents 并行）

```
Agent E: AEGIS 4-Stage Evolution (HarnessX pattern)
├── 创建 nt_mind/seal/aegis.rs
├── Digester→Planner→Evolver→Critic pipeline
└── 验证编译

Agent F: Trinity Memory + Three-Type Memory
├── 创建 nt_memory/trinity.rs (vector+graph+relational)
├── 创建 nt_memory/memory_types.rs (Workflow/Subtask/Function)
└── 验证编译

Agent G: Decision Intelligence + Semantic Taint
├── 创建 nt_governance/decision_graph.rs (W3C PROV-O)
├── 创建 nt_shield/taint.rs (semantic taint tracking)
└── 验证编译

Agent H: Subagent Worktrees + Continual Harness
├── 创建 nt_act/worktree.rs (isolated context windows)
├── 创建 nt_mind/harness/refinement.rs (/refine command)
└── 验证编译
```

### Wave 3: 集成测试 + 清理（3 agents 并行）

```
Agent I: 全量编译验证
├── cargo check --lib 0 errors
├── cargo test --lib 全部通过
└── 修复所有回归

Agent J: 前端集成验证
├── npx vite build clean
├── Tauri 启动测试
└── DomainRegistry 完整性

Agent K: 冗余清理 + 文档
├── 删除重复代码
├── 更新 AGENTS.md
└── 更新 CONTEXT.md
```

---

## 四、执行优先级矩阵

```
┌─────────────────────────────────────────────────────────┐
│                    P0 缺口 (D1-D8)                       │
│  ┌──────────┬──────────┬──────────┬──────────┐          │
│  │ Agent A  │ Agent B  │ Agent C  │ Agent D  │          │
│  │ Hook+ACP │ Cache+FS │ IO+Ref   │ Harness  │          │
│  └──────────┴──────────┴──────────┴──────────┘          │
│                    Wave 1 (Day 1)                        │
├─────────────────────────────────────────────────────────┤
│                    P1 缺口 (D9-D16)                      │
│  ┌──────────┬──────────┬──────────┬──────────┐          │
│  │ Agent E  │ Agent F  │ Agent G  │ Agent H  │          │
│  │ AEGIS    │ Trinity  │ DecIDE   │ Worktree │          │
│  └──────────┴──────────┴──────────┴──────────┘          │
│                    Wave 2 (Day 2)                        │
├─────────────────────────────────────────────────────────┤
│                    集成 + 清理                            │
│  ┌──────────┬──────────┬──────────┐                     │
│  │ Agent I  │ Agent J  │ Agent K  │                     │
│  │ Compile  │ Frontend │ Cleanup  │                     │
│  └──────────┴──────────┴──────────┘                     │
│                    Wave 3 (Day 3)                        │
└─────────────────────────────────────────────────────────┘
```

---

## 五、关键约束

1. **UI 安全**: 所有 agent 限定文件范围，禁止触碰 UI 渲染组件
2. **版本管理**: 每个 Wave 完成后 git commit
3. **编译验证**: 每个 agent 完成后验证 `cargo check` 无新增错误
4. **模块隔离**: 每个 agent 只创建/修改指定文件，避免冲突
5. **向后兼容**: 新模块添加 `#[allow(dead_code)]`，不影响现有功能

---

## 六、预期成果

| 指标 | 当前 | Wave 1 后 | Wave 2 后 | Wave 3 后 |
|------|------|----------|----------|----------|
| 核心模块数 | 10 | 14 | 18 | 18 |
| 新增代码行 | ~2,700 | +1,200 | +1,500 | 0 |
| 编译错误 | 0 | 0 | 0 | 0 |
| 测试覆盖 | 部分 | +20 | +15 | 全量 |
| P0 缺口 | 8/8 未实现 | 4/8 已实现 | 8/8 已实现 | 8/8 已实现 |
| P1 缺口 | 8/8 未实现 | 8/8 未实现 | 4/8 已实现 | 8/8 已实现 |
