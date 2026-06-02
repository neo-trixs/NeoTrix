# NeoTrix Evolution Roadmap — 完整路线图

**Sources**: MAPPA, BES, AdaCoM, agentmemory, Scrapling, V-SPLADE, Fastest-RAG, WeSight, GenAI Book, EFC
**Owner**: 多 session 协同
**核心约束**: 所有进化迭代必须在规则系统下执行

---

## 本次 Session (已执行)

| # | 任务 | 状态 |
|---|---|---|
| ✅ | 预存在测试错误修复 (engine_core 括号 — 1325-1335 孤儿代码) | ✅ 完成 |

---

## 统一重构 Session (另一 session 执行)

| Phase | 任务 | 优先级 |
|---|---|---|
| 0 | git 分支 + 映射清单 + cargo check 基线 | HIGH |
| 1 | NT-CORE 重命名 core/ → nt_core_* | HIGH |
| 2 | NT-MIND 重命名 reasoning_brain/ → nt_mind/ | HIGH |
| 3 | NT-MEMORY 重命名 knowledge_base/ → nt_memory_kb/ | HIGH |
| 4 | NT-SHIELD 重命名 security/ → nt_shield/ + CLI sandbox | MEDIUM |
| 5 | NT-ACT 重命名 crypto_agent/earn_agent/file_sync/social_media/spear/neogram/voice | MEDIUM |
| 6 | NT-WORLD 重命名 browser/crawler/world_model/jepa_world_model/sensory | MEDIUM |
| 7 | NT-IO 重命名 cli/server/entry/web_ui/notification + 前端 nt_ui_* | MEDIUM |
| 8 | crates/neotrix-types re-export + Cargo.toml path 更新 | MEDIUM |
| 9 | 测试修复 — #[path] + use 语句 | MEDIUM |
| 10 | 全量 cargo check + cargo test --lib + npm test 验证 | HIGH |

---

## 规则完善 (新 Phase — 重命名后、进化前执行)

**详见 `RULES_SYSTEM.md`**

| Phase | 任务 | 文件 | 阻断点 |
|---|---|---|---|
| R1 | 创建 `ShieldEnforcer` 统一入口 | `cli/shield_enforcer.rs` (新) | 所有命令派发 |
| R2 | 创建 `ProjectLaws` 代码化法则 | `cli/laws.rs` (新) | pre-commit + CI |
| R3 | Wire into CLI dispatch | `types.rs` execute() | 命令执行前 |
| R4 | Wire into file operations | `file_cmds.rs` | 文件写入前 |
| R5 | Wire into SEAL pipeline | `pipeline.rs` | 每 stage 执行前 |
| R6 | Wire into LLM/Provider | provider 调用链 | LLM 调用前 |
| R7 | Wire into MCP tools | `mcp_tools.rs` | 工具调用前 |
| R8 | CI pre-commit hook | 新脚本 | git commit 前 |
| R9 | E2E test: shield 阻断验证 | 新测试 | CI |

**规则**: 未通过的 ProjectLaws + ShieldEnforcer 不允许 P0 任何代码合并。

---

## 其他 Session 任务 (重命名 + 规则完善后执行)

| # | 任务 | 优先级 |
|---|---|---|
| 1 | 新模块测试套件 | MEDIUM |
| 2 | 跨模块集成测试设计 (12维同步) | MEDIUM |
| 3 | 文档同步 (AGENTS.md 11维架构章节) | MEDIUM |
| 4 | 守护进程集成 GeometrySync::cycle() | MEDIUM |
| 5 | clippy 警告清理 | MEDIUM |

---

## P0 进化任务 (规则完善后执行 — 在规则下执行)

### P0-A: MAPPA — Per-Action Process Rewards

| # | 任务 | 文件 | 检查点 |
|---|---|---|---|
| A1 | `CoachScore` struct | `coach.rs` | L009 (浮点约束) |
| A2 | `AICoach` trait | `coach.rs` | L001+L002 (命名+注册) |
| A3 | `LlmCoach` impl | `coach.rs` | R6 (LLM 调用走 Shield) |
| A4 | `NullCoach` impl | `coach.rs` | — |
| A5 | 注册 coach 模块 | `mod.rs` | L002 (必须注册) |
| A6 | `coach_scores` → `StageResult` | `pipeline.rs` | — |
| A7 | `coach` → `SelfIteratingBrain` | `loop_impl/core.rs` | — |
| A8 | `with_coach()` builder | `loop_impl/core.rs` | — |
| A9 | `BrainStage::process` 签名变更 | `pipeline.rs` trait | — |
| A10 | 更新 27 个 stage | 各 stage 文件 | — |
| A11 | pipeline.execute() 传递 coach | `pipeline.rs` | R5 (SEAL 走 Shield) |
| A12 | 每 stage 后 coach.score() | `pipeline.rs` | R6 (coach LLM 调用走 Shield) |
| A13 | REINFORCE++ 梯度 | `pipeline.rs` | — |
| A14 | baseline EMA | `pipeline.rs` | L009 (浮点约束) |
| A15 | E8 用 avg_coach_score | `e8_experiment.rs` | — |
| A16 | --coach-model CLI | CLI config | R3 (CLI 命令走 Shield) |
| A17-A19 | 3 个测试 | 对应文件 tests | R9 (Shield E2E) |

### P0-B: BES — Bidirectional Evolutionary Search

| # | 任务 | 文件 | 检查点 |
|---|---|---|---|
| B1 | `E8Trajectory` struct | `e8_reasoning.rs` | L001 (nt_ 前缀) |
| B2 | `crossover()` | `e8_reasoning.rs` | — |
| B3 | `mutate()` | `e8_reasoning.rs` | — |
| B4 | `population` + `generation` | `e8_experiment.rs` | — |
| B5 | `evolution_step()` | `e8_experiment.rs` | — |
| B6 | `backward_decompose()` | `e8_experiment.rs` | R6 (LLM 调用走 Shield) |
| B7 | `verify_subgoals()` | `e8_experiment.rs` | — |
| B8 | `core_review()` subgoal | `engine_core.rs` | R6 |
| B9 | BES 替代 best-of-N | `e8_experiment.rs` | — |
| B10-B13 | 3 测试 + 1 bench | 对应文件 | R9 |

---

## P1-P3 进化任务 (P0 完成后执行)

| Phase | 组 | 条目 | 前置 |
|---|---|---|---|
| P1-A | AdaCoM 上下文管理 | C1-C9 | P0 + R1-R9 |
| P1-B | agentmemory KB 后端 | D1-D6 | P0 + R1-R9 |
| P2-A | Scrapling 反机器人 | E1-E7 | P0 + R1-R9 |
| P2-B | V-SPLADE 稀疏检索 | F1-F4 | P0 + R1-R9 |
| P3 | 桌面 UI + GenAI 数学 | G1-G6 | P0 + R1-R9 |

---

## 执行时间线

```
重命名 (Phase 0-10)
  │
  ├──→ 规则完善 (Phase R1-R9) — 创建 ShieldEnforcer + ProjectLaws
  │       │
  │       ├──→ 其他 session 任务 (测试/文档/守护进程/clippy)
  │       │
  │       └──→ P0 进化: MAPPA (A1-A19) + BES (B1-B13)
  │               │
  │               └──→ P1-P3 进化
  │
  └── 每个阶段都经过 ShieldEnforcer 检查
```

**核心**: 在执行 `types.rs:120 execute()` 之前，先过 `ShieldEnforcer` 链。任何违反 ProjectLaws 的操作被阻断，报 `ExitCode::PermissionDenied(3)`。
