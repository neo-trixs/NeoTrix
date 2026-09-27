# NeoTrix TODO 列表
> 智能同步生成，最后更新：2026-13-02T01:58:00

> **2026-09-27 卡死/内存专项 + 长尾分诊（收口）**：全量 `--lib` 串行实跑
> **11493 绿 / 51 红**（起点 10113/125）。8 条卡死/内存根因全部除根并加三道闸；
> 21+25+27 = **73 处生产缺陷**已修（安全洞 5、解析/数据 11、逻辑 25、stub 补实现 9、
> 环境依赖去抖动 6、契约对齐 12、并发/治理 5）。详见
> `sessions/handoff-disease-list-20260927.md`（含 125 条分诊全表与 file:line）。
>
> **待办（按性质分组，非按模块）**
> - 🔴 **B 类·需设计裁决**（不可一行改，先定方向）：
>   1. `l0_substrate/nt_core_kb_primitives.rs:188` — `nodes.id` 单列 PK 使双时态版本化
>      不可能 → 需 PK→`UNIQUE(id,transaction_time)` + edges FK 重构 + 真实库迁移（2 测试）
>   2. `l4_emotion/nt_memory/cascade/cascade.rs:216,126` — `length_score = len/200` 过小，
>      且 `tick()` 永久丢弃未达阈样本（属设计缺陷，需重定晋升/丢弃策略）
>   3. `neotrix-types` 的 `Severity`/`FlagSeverity` 判别序"越严重越小"是**承重约定**
>      （`l2_perception/nt_world/osint/sweep.rs:225` 依赖它做 `min_severity` 过滤）→ 不要盲翻 `Ord`
> - 🟡 **C 类·未接线 stub**（实现或显式 `#[ignore]`，禁止改松断言凑绿）：
>   `nt_core_embed::TextEmbedder`（字节位置袋，任意文本相似度≈0.83）、
>   `nt_shield_ztnet/crypto/noise_handshake`（缺 `_create_message3`，握手无法完成）、
>   `publish_gateway`（YouTube 上传）、`nt_codegen::parse_yaml`（误用 serde_json，需引 serde_yaml）、
>   `nt_memory_kb::nt_memory_distill`（测试 teacher 与 student 恒等，`after<before` 不可满足）
> - 🟡 **环境依赖**（改确定性断言或 `#[ignore]`）：~~`l6_meta::runtime_monitor::test_get_health`（探针挂起）~~
>   ~~`nt_feel::writing_style`（2）~~ —— **已由 cycle `audit0927b` 修掉，非环境问题**：
>   runtime_monitor 是**真死锁**（monitor 持 metrics 守卫调 check_thresholds，后者再
>   锁同一把，std Mutex 不可重入）；writing_style 是该 1,241 LOC 模块**从未被 mod 声明**，
>   从未编译，接上即 0 error 0 warning。详见 `sessions/handoff-disease-list-20260927.md` §10。
>   仍待处理：`l6_meta::nt_core_aware`（4）、`nt_core_observer_error`（2）、
>   `nt_feel::cognitive_bridge::feedback`（2）
> - ⚪ **结构性债务（PARK，有归属前置）**：L0 `CapabilityRegistry` ×4 + `SemanticRouter` ×2
>   正典收敛（异构，需专窗迁移）｜`proxy_pool.rs` 1757 行拆分（他人在途 1039+/24-）｜
>   剩余 God-file（`pdf.rs` 2142 / `nt_crystal_serve.rs` 2002 / gateway 1678 / hex 1530）｜
>   5 个内容型 worktree 裁决｜前端 `apps/neobot-desktop/frontend/{src,dist}` 归属与提交

> **Batch3 吸收执行 (47 源)**: 四波 21 任务 20/20 闭环 · **交接 Wave 4: 10 任务待做** (🔴P0×3 越层修复/e8_state 合成值/测试抖动加固 · 🟡P1×3 情报工具接线/SEAL C0→C2/补全排序 · ⚪P2×4) → `docs/absorption-knowledge-base/batch3-2026-08-26-unified-evolution-todo.md` Wave 4 段 + 根 `HANDOFF.md` (2026-08-26 版)

### 🔄 task-2: parent

**状态**: in_progress
**子代理**: ses_1787882307_4
**依赖**: task-1
**更新**: 2026-13-02T01:58:00
**效率分数**: 32.0

### ⬜ task-1: blockable

**状态**: in_progress
**更新**: 2026-13-02T01:58:00
**效率分数**: 30.0

### ⬜ task-3: child

**状态**: pending
**更新**: 2026-13-02T01:58:00
**效率分数**: 30.0

### ⬜ task-4: test

**状态**: pending
**更新**: 2026-13-02T01:58:00
**效率分数**: 30.0

### ⬜ task-5: move_test

**状态**: pending
**更新**: 2026-13-02T01:58:00
**效率分数**: 30.0

### ⬜ task-6: json_test

**状态**: pending
**更新**: 2026-13-02T01:58:00
**效率分数**: 30.0

---

## 🔴 Agent Guardrail 架构升级 (2026-09-20)
> 解决 AI 对话过程中违反规则的问题
> 方案文件: `docs/plans/2026-09-20-agent-guardrail-architecture.md`
> 参考: AEGIS (336★), GuardRail (172★), AgentJail (85★), AgentGuard, LITMUS

### ⬜ ag-1: 实现 Pre-Execution Firewall 核心框架
**优先级**: 🔴 High | **状态**: pending
文件: `neotrix-core/src/l2_perception/nt_shield/pre_execution_firewall.rs` (新建)
操作: 创建 5-stage pipeline (Classify → Anomaly → Evaluate → Match DSL → Decide)
参考: AEGIS architecture

### ⬜ ag-2: 实现 5 个核心 Guards
**优先级**: 🔴 High | **状态**: pending | **依赖**: ag-1
Guards:
- main_push_guard (禁止 push 到 protected branches)
- force_push_guard (禁止 force push)
- destructive_path_guard (禁止 rm -rf 系统路径)
- secret_leak_guard (检测 API key 泄漏)
- sql_injection_guard (检测 SQL 注入)
参考: GuardRail 18 guards

### ⬜ ag-3: 实现 Policy DSL 解析器
**优先级**: 🔴 High | **状态**: pending
文件: `neotrix-core/src/l2_perception/nt_shield/policy_engine.rs` (新建)
操作: 解析 nt-policies.yaml，执行策略匹配
参考: AEGIS Policy DSL, AgentJail OPA Rego

### ⬜ ag-4: 创建 nt-policies.yaml 策略文件
**优先级**: 🟡 Medium | **状态**: pending | **依赖**: ag-3
文件: `nt-policies.yaml` (根目录)
内容: 定义核心策略规则

### ⬜ ag-5: 实现 Audit Log (SHA-256 hash chain)
**优先级**: 🟡 Medium | **状态**: pending
文件: `neotrix-core/src/l2_perception/nt_shield/audit_log.rs` (新建)
操作: 不可篡改审计日志，每次决策记录 hash chain

### ⬜ ag-6: 实现 Human Approval Queue
**优先级**: 🟡 Medium | **状态**: pending | **依赖**: ag-3
文件: `neotrix-core/src/l2_perception/nt_shield/approval_queue.rs` (新建)
操作: 高风险操作需要人工审批

### ⬜ ag-7: 集成到 Agent 执行循环
**优先级**: 🔴 High | **状态**: pending | **依赖**: ag-1, ag-3
操作: 在 Agent 执行 tool call 前调用 firewall.check()
文件: Agent 执行循环相关文件

### ⬜ ag-8: Physical-Layer Verification 原型
**优先级**: 🟢 Low | **状态**: pending
操作: 验证实际系统状态，检测 Execution Hallucination
参考: LITMUS semantic-physical dual verification

---

## 🔴 Bend 语言吸收 — NT-LAWS 设计 (2026-09-20)
> 吸收自 https://github.com/bend-lang/bend (22K★)，核心启发：LAWS.bend + 数学证明机制
> 方案文件: `docs/plans/2026-09-20-bend-absorption-analysis.md`

### ⬜ bend-1: 实现 NT-LAWS.nt parser 原型
**优先级**: 🔴 High | **状态**: pending
文件: `crates/nt-lang/src/` 
操作: 添加 `law` / `proof` 语法解析，生成 IR
参考: Bend LAWS.bend 语法 + 现有 test_parser.rs 模式

**Phase 0 第一步 (今天可做)**:
1. 在 test_parser.rs 中添加 `law` 关键字识别
2. 生成 `LawIR { name, for_vars, ensures }` 结构
3. codegen 输出 `const_assert!(...)` 宏调用

### ⬜ bend-2: 选择 2-3 个关键不变量用 NT-LAWS 表达
**优先级**: 🔴 High | **状态**: pending | **依赖**: bend-1
候选不变量:
- memory_never_leaks (R-P161 对抗管线)
- cost_stays_below (R-P190 成本追踪)
- handoff_includes_context (R-P186 交接协议)

### ⬜ bend-3: Agent 并行声明式标记原型
**优先级**: 🟡 Medium | **状态**: pending
文件: `crates/neotrix-multi-agent/src/parallel.rs`
操作: 添加 `#[nt_parallel]` 属性宏，自动映射到 rayon

### ⬜ bend-4: 依赖类型 trait bound 代码生成
**优先级**: 🟡 Medium | **状态**: pending | **依赖**: bend-1
操作: nt-lang codegen 生成带能力约束的 Rust trait bound

---

## 🔴 统一数据源架构 — 后续任务 (2026-09-20)
> `cargo check -p neotrix --lib` 已通过 0 errors。以下任务待其他会话完成。

### ⬜ ds-1: cargo check --all-targets 验证测试编译
**优先级**: 🔴 High | **状态**: pending
```bash
cargo check --all-targets -p neotrix 2>&1 | head -100
```
修复模式: 仅在 `#[cfg(test)]` 中使用的 import → 移入 `mod tests` 内部

### ⬜ ds-2: 检查 UnifiedEngine 是否集成 crawl_bridge 和 subscription_source
**优先级**: 🔴 High | **状态**: pending | **依赖**: ds-1
文件: `unified_engine.rs` / `crawl_bridge.rs` / `subscription_source.rs`
操作: 如缺失 `with_crawl_sources()` / `with_subscription_sources()`，补充到 UnifiedEngine

### ⬜ ds-3: 审查各桥接层函数命名一致性
**优先级**: 🟡 Medium | **状态**: pending | **依赖**: ds-2
统一为 `bridge_all_*()` 或 `create_*_bridges()`，当前命名不统一

### ⬜ ds-4: 补充 UnifiedEngine 测试覆盖
**优先级**: 🟡 Medium | **状态**: pending | **依赖**: ds-1
补充: `test_search_by_domains` / `test_health_check` / `test_engine_stats` / `test_registry_find`

### ⬜ ds-5: 更新架构文档模块数量和桥接状态
**优先级**: 🟢 Low | **状态**: pending | **依赖**: ds-2, ds-3
更新: `docs/architecture/NEOTRIX-FULL-ARCHITECTURE.md` 等

---

## 🔴 Fusion Cleanup — 架构清理 (2026-09-20)
> MemoryEntry 重复类型已清理（11→1 canonical + 9 renamed）。lib 编译 0 errors。
> 经验已写入 `~/.neotrix/pending-absorb.json`，等待后台循环吸收。

### ⬜ fc-1: 删除 unused import 修复 lib 编译
**优先级**: 🔴 High | **状态**: pending
```bash
# 修复这两个文件
neotrix-core/src/l0_substrate/nt_core_platform/agent_registry.rs:14
  → 删除: use super::health::HealthChecker;
neotrix-core/src/l2_perception/nt_world/crawl/classifier.rs:5
  → 删除: use super::discover::DiscoveryExtractor;
```

### ⬜ fc-2: cargo check --all-targets 修复剩余测试错误
**优先级**: 🔴 High | **状态**: pending | **依赖**: fc-1
```bash
cargo check -p neotrix --all-targets 2>&1 | grep "error\["
```
已修复的测试错误（供参考，无需重复）：
- orchestrator.rs: +import AttackStrategy
- chain_executor.rs: +import ChainConfig
- first_person_ref.rs: vsa_tag 路径→nt_consciousness
- nt_core_capability/mod.rs: +import std::sync::Arc
- mem.rs: +impl Default for ReasoningMemory
- nt_emotion_reasoning_bridge.rs: +EmotionEngine::new(), +SystemEvent variants
- nt_core_kernel.rs: 删除 output.trace 断言
- agent_capability/tests.rs: SearchResult→WebSearchResult
- integration.rs: SelfModel→MetaSelfModel
- diagnostic_chain/mod.rs: HealthSignal::new +timestamp
- resilience.rs: value 类型 + 类型标注
- whois_module.rs: test 函数重命名
- vector_store.rs: search_batch 参数类型
- cert_transparency.rs / harvest_engine.rs: 删 proxy() 断言
- subscription_source.rs: +chrono::Datelike
- recon_engine.rs: "v".into()→"v"

### ⬜ fc-3: GraphEdge 重复类型清理（8→1）
**优先级**: 🟡 Medium | **状态**: pending
canonical 在 L0 `nt_core_capability_types.rs:496`。
7 个非 canonical 定义需 rename:
- crystal_core/knowledge_graph.rs → CrystalGraphEdge
- nt_core_graph.rs → ActionGraphEdge
- nt_mind/evolution/deliberation.rs → DeliberationGraphEdge
- nt_mind/graph_types.rs → MindGraphEdge
- nt_core/nt_state_graph.rs → StateGraphEdge
- nt_memory_openknowledge.rs → KnowledgeGraphEdge
- nt_memory_leann_store.rs → LeannGraphEdge

### ⬜ fc-4: SearchResult 重复类型清理（13→trait impl）
**优先级**: 🟡 Medium | **状态**: pending
`SearchResultTrait` 已在 L0。各模块保留自己的 SearchResult struct，实现 trait。

### ⬜ fc-5: L5 拆分分析（nt_mind 388 文件占 59%）
**优先级**: 🟢 Low | **状态**: pending
候选: nt_game(61)→独立 crate, mind_modules(25)→L4, cross_domain(5)→L6

### ⬜ fc-6: nt_memory/cascade 路径更新
**优先级**: 🟢 Low | **状态**: pending
已移到 `l4_emotion/nt_memory/cascade/mod.rs`，路线图路径过时。

### ⬜ fc-7: 最终验证
**优先级**: 🔴 High | **状态**: pending | **依赖**: fc-1, fc-2
```bash
cargo check -p neotrix --lib          # 0 errors
cargo check -p neotrix --all-targets  # 0 errors
cargo test -p neotrix --lib           # pass
```

---

## 🔴 编译修复 — 阻塞所有后续工作 (2026-09-20)
> `cargo check -p neotrix --lib` 有 4 个 error，必须先修复

### ⬜ cf-1: 删除 unused import `HealthChecker`
**优先级**: 🔴 High | **状态**: pending
文件: `neotrix-core/src/l0_substrate/nt_core_platform/agent_registry.rs:14`
修复: 删除 `use super::health::HealthChecker;` 或改为 `use super::health::HealthChecker as _;`

### ⬜ cf-2: 删除 unused import `DiscoveryExtractor`
**优先级**: 🔴 High | **状态**: pending
文件: `neotrix-core/src/l2_perception/nt_world/crawl/classifier.rs:5`
修复: 删除 `use super::discover::DiscoveryExtractor;`

### ⬜ cf-3: 修复 `PersonalitySnapshot` 缺少 `tick` 字段
**优先级**: 🔴 High | **状态**: pending
文件: 搜索 `PersonalitySnapshot` struct 定义
修复: 删除 struct 初始化中的 `tick` 字段，或在 struct 中添加 `pub tick: u64`

### ⬜ cf-4: 修复 unused variable `base`
**优先级**: 🔴 High | **状态**: pending
修复: 改为 `_base` 或删除

### ⬜ cf-5: 测试编译验证
**优先级**: 🔴 High | **状态**: pending | **依赖**: cf-1~cf-4
```bash
cargo check -p neotrix --lib        # 0 errors
cargo test -p neotrix --lib         # 单元测试通过
cargo test -p neotrix --tests       # 集成测试通过
```

---

## 🟡 能力树 C1→C2 推进 (2026-09-20)
> 41 个模块已在 C1 (UnitTest)，需推进到 C2 (IntegrationTest)

### ⬜ ct-1: nt_memory 域 C2 推进
**优先级**: 🟡 Medium | **状态**: pending
模块: entity_linking, consolidation, distillation, add_only_writes, hybrid_retrieval, admission_control, decay_forgetting
验证: 集成测试 `nt_memory_integration.rs` 全部通过

### ⬜ ct-2: nt_shield 域 C2 推进
**优先级**: 🟡 Medium | **状态**: pending
模块: llm_scanner, red_team, secret_scanner, container_scan, compliance, agent_guardrails
验证: 集成测试 `nt_shield_integration.rs` 全部通过

### ⬜ ct-3: nt_meta/nt_core 域 C2 推进
**优先级**: 🟡 Medium | **状态**: pending
模块: prompt_manager, context_router, coordinator, graph_orch, decision_engine, skill_chain
验证: 集成测试 `nt_meta_integration.rs` 全部通过

### ⬜ ct-4: healing/governance 域 C2 推进
**优先级**: 🟡 Medium | **状态**: pending
模块: self_healing, predictive_maintenance, diagnostic_chain, enforcement
验证: 集成测试 `healing_integration.rs` 全部通过

### ⬜ ct-5: 更新能力树 roadmap.rs
**优先级**: 🟡 Medium | **状态**: pending | **依赖**: ct-1~ct-4
文件: `nt_core_capability_tree/src/roadmap.rs`
操作: 将 41 个模块的 maturity 从 C1UnitTest 改为 C2IntegrationTest

---

## 🔴 P0: 紧急修复 — 编译验证 (2026-09-20)

### ⬜ TODO-001: 验证编译状态
**优先级**: 🔴 P0 | **状态**: pending
描述: 运行 cargo clean && cargo check 验证当前编译状态
命令:
```bash
cargo clean -p neotrix
cargo check -p neotrix 2>&1 | tail -50
```
验收标准: 编译通过或错误数明确
耗时: ~5 分钟

---

## 🟡 P1: 高优先级 — 本周任务 (2026-09-20)

### ⬜ TODO-007: 修复 SelfIteratingBrain FIXME
**优先级**: 🟡 P1 | **状态**: pending
描述: select_operator / selective_state 字段被注释
涉及文件:
- `l5_cognition/nt_mind/seal_core/self_iterating/pipeline.rs:1015`
- `l5_cognition/nt_mind/seal_core/self_iterating/loop_impl/seal_loop.rs:1211`
耗时: ~30 分钟

### ⬜ TODO-010: 提升 L2 测试覆盖率
**优先级**: 🟡 P1 | **状态**: pending
描述: 从 62% 提升到 70%+
涉及目录: `neotrix-core/src/l2_perception/`
步骤:
1. 找出没有测试的文件
2. 为每个文件添加基本单元测试
3. 运行 cargo test 验证
耗时: ~2 小时

### ⬜ TODO-011: 修复剩余编译错误
**优先级**: 🟡 P1 | **状态**: pending
描述: 164 个预存错误需要修复
重点区域:
- `l2_perception/nt_world/` (导入错误)
- `l4_emotion/nt_memory/` (类型错误)
- `l5_cognition/nt_mind/` (路径错误)
耗时: ~4 小时

### ⬜ TODO-012: 清理 neotrix-core/src/neotrix/ 命名空间
**优先级**: 🟡 P1 | **状态**: pending
描述: 110 个文件在遗留命名空间中
涉及目录: `neotrix-core/src/neotrix/`
耗时: ~4 小时

---

## ⚪ P2: 中优先级 — 本月任务 (2026-09-20)

### ⬜ TODO-013: 解决 fusion-plan-215 TODO
**优先级**: ⚪ P2 | **状态**: pending
描述: 合并 EchoPrmBridge 和 MethodRegistry
涉及文件:
- `echo_terminal.rs:406`
- `reasoning_core.rs:66`
耗时: ~2 小时

### ⬜ TODO-014: 删除 neotrix-consciousness/src/legacy.rs
**优先级**: ⚪ P2 | **状态**: pending
描述: 所有消费者迁移到新路径后删除
前提: 所有使用旧路径的代码已更新
耗时: ~1 小时

### ⬜ TODO-015: 添加文档注释
**优先级**: ⚪ P2 | **状态**: pending
描述: 78.5% 公共项未文档化
涉及目录: 整个 neotrix-core/src/
耗时: ~8 小时

### ⬜ TODO-016: 修复 L4→L5 层次违规
**优先级**: ⚪ P2 | **状态**: pending
描述: nt_feel/emotion_engine.rs 导入 l5_cognition::l1_facade::emotion_state
涉及文件:
- `l4_emotion/nt_feel/emotion_engine.rs`
- `l4_emotion/nt_emotion_facade.rs`
- `l4_emotion/nt_memory_kb/mod.rs`
耗时: ~1 小时

---

## ⚪ P3: 低优先级 — 积压任务 (2026-09-20)

### ⬜ TODO-017: 解决 29 个 TODO 注释
**优先级**: ⚪ P3 | **状态**: pending
描述: 生产代码中的 TODO 注释
耗时: ~4 小时

### ⬜ TODO-018: 修复 nt_codegen.rs:165
**优先级**: ⚪ P3 | **状态**: pending
描述: 生成 `// TODO: implement system logic` 作为代码输出
涉及文件: `nt_codegen.rs:165`
耗时: ~30 分钟

### ⬜ TODO-019: 实现 McpRegistry.gateway()
**优先级**: ⚪ P3 | **状态**: pending
描述: 2 个 FIXME 关于此功能
涉及文件: `agent_cmds.rs:408,520`
耗时: ~2 小时

### ⬜ TODO-020: 整理 nt_core_capability_tree
**优先级**: ⚪ P3 | **状态**: pending
描述: 嵌入式路径 crate 需要评估
涉及目录: `neotrix-core/src/neotrix/nt_core_capability_tree/`
耗时: ~1 小时

---

## ✅ 文档标准建立 (2026-09-20) — 已完成

### ✅ doc-1: 创建 DOCUMENTATION-MAP.md
**优先级**: 🔴 High | **状态**: ✅ completed
文件: `/Users/neo/Downloads/neotrix/DOCUMENTATION-MAP.md`
内容: 文档分类、存放位置、命名规范、禁止行为、生命周期

### ✅ doc-2: 添加文档管理规则 R-P212~R-P220
**优先级**: 🔴 High | **状态**: ✅ completed
文件: `docs/dev-rules.md`
内容: 9 条文档管理规则

### ✅ doc-3: 清理 neotrix-core/docs/ 研究笔记
**优先级**: 🟡 Medium | **状态**: ✅ completed
操作: 删除/移动 20+ 研究文件

### ✅ doc-4: 清理根目录临时文件
**优先级**: 🟡 Medium | **状态**: ✅ completed
操作: 删除空 README.md、.specstory/、notes/

### ✅ doc-5: 更新 AGENTS.md 引用文档标准
**优先级**: 🟡 Medium | **状态**: ✅ completed
操作: 添加文档管理规则章节

