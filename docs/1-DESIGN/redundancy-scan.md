# neotrix-core/src/ 冗余代码扫描报告

**扫描日期**: 2026-09-11
**扫描范围**: `neotrix-core/src/` 全量 `.rs` 文件
**总行数**: ~620,426 行

---

## 一、严重重复类型定义 (Top 5)

### 1. SearchResult — 13 处定义

| # | 文件 | 层级 |
|---|------|------|
| 1 | `core/nt_core_answer_engine.rs:21` | L5 |
| 2 | `core/nt_core_vector_store/types.rs:45` | L5 |
| 3 | `l2_perception/nt_world/nt_world_search.rs:60` | L2 |
| 4 | `l2_perception/nt_world/source/types.rs:86` | L2 |
| 5 | `l1_action/traits.rs:239` | L1 |
| 6 | `l1_action/nt_act/types.rs:162` | L1 |
| 7 | `l1_action/nt_memory/nt_memory_kb/nt_memory_types.rs:83` | L1 |
| 8 | `l1_action/nt_memory/nt_memory_kb/kb_cognition.rs:25` | L1 |
| 9 | `l1_action/nt_memory/nt_memory_kb/ntx/vec_segment.rs:305` | L1 |
| 10 | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/search_skill_stage.rs:68` | L5 |
| 11 | `l5_cognition/nt_mind/nt_mind/evolution/casebase.rs:521` | L5 |
| 12 | `l3_embodiment/nt_shield/nt_shield_stealth_net/crawler_core.rs:100` | L3 |
| 13 | `unified_archive/.../core/types.rs:86` | 归档 |

**建议**: 统一为 `nt_core_graph_types::SearchResult`，各模块引用。

### 2. RiskLevel — 13 处定义

| # | 文件 | 层级 |
|---|------|------|
| 1 | `core/nt_core_meta/planner.rs:220` | L5 |
| 2 | `core/nt_core_self/human_approval.rs:24` | L5 |
| 3 | `l2_perception/nt_world/explore/system_scanner.rs:19` | L2 |
| 4 | `l1_action/nt_act/nt_act_cleanup/shared.rs:15` | L1 |
| 5 | `l1_action/nt_act/nt_act_trade/trade_core.rs:385` | L1 |
| 6 | `l1_action/nt_act/nt_act_trade/finance_compliance.rs:114` | L1 |
| 7 | `l1_action/nt_act/nt_act_trade/full_cycle.rs:509` | L1 |
| 8 | `l1_action/nt_act/nt_act_trade/production_logistics.rs:324` | L1 |
| 9 | `l1_action/nt_act/actions/disk_guard.rs:24` | L1 |
| 10 | `l6_meta/evolution/nt_act_human_approval.rs:66` | L6 |
| 11 | `l5_cognition/nt_mind/foundation/cleanup_engine.rs:639` | L5 |
| 12 | `l5_cognition/nt_mind/foundation/repair.rs:77` | L5 |
| 13 | `l3_embodiment/nt_shield/nt_shield/redaction.rs:19` | L3 |

**建议**: 至少合并为 3 个语义层级: `GuardRiskLevel`(安全审批) / `CleanupRiskLevel`(清理) / `TradeRiskLevel`(交易)。

### 3. GraphNode/GraphEdge — 8 处定义

| # | 文件 |
|---|------|
| 1 | `core/nt_core_graph_types.rs` (事实源) |
| 2 | `core/nt_core_capability/mod.rs:577` |
| 3 | `neotrix/nt_unified_api/mod.rs:192` |
| 4 | `l1_action/nt_memory/nt_memory_openknowledge.rs:97` |
| 5 | `l1_action/nt_memory/nt_memory_kb/ntx/graph_segment.rs:20` |
| 6 | `l1_action/nt_memory/nt_memory_kb/knowledge_storage.rs:313` |
| 7 | `l1_action/nt_memory/nt_memory_leann_store.rs:9` |
| 8 | `l5_cognition/nt_mind/nt_mind/graph_types.rs:56` |

**建议**: 统一引用 `core::nt_core_graph_types::{GraphNode, GraphEdge}`。

### 4. Domain enum — 6+ 处定义

| # | 文件 | 变体数 |
|---|------|--------|
| 1 | `core/nt_core_capability/mod.rs:48` | 7 域 |
| 2 | `core/l7_capability/nt_core_grounded_gate.rs:172` | — |
| 3 | `l2_perception/nt_world/sense/nt_world_model_types.rs:144` | — |
| 4 | `neotrix/nt_core_capability_tree/src/node.rs:9` | — |
| 5 | `l1_action/nt_io/nt_io_provider/generation_classifier.rs:72` | — |
| 6 | `l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs:62` | — |

**建议**: 定义 `core::nt_core_domain::Domain` 为唯一事实源。

### 5. TaskType / TaskStatus — 8/8 处定义

**TaskType**: `core/nt_core_knowledge/types.rs`, `l2_perception/nt_world/nt_world_model.rs`, `l1_action/nt_act/nt_act_orchestrator/planner.rs`, `l1_action/nt_act/nt_act_orchestrator/critic.rs`, `l1_action/nt_act/actions/production_pipeline.rs:32`, `l1_action/nt_io/nt_io_provider/generation_classifier.rs:16`, `l5_cognition/nt_core/nt_consciousness_core/resource_router.rs:116`, `l5_cognition/traits.rs:21`

**TaskStatus**: `core/nt_core_capability/discovery.rs:291`, `core/nt_core_self/cuda_agent.rs:24`, `core/l7_capability/nt_core_orch_agent.rs:8`, `l1_action/nt_act/parallel_task.rs:15`, `l1_action/nt_act/actions/nt_act_ai_assistant.rs:103`, `l1_action/nt_act/actions/production_pipeline.rs:15`, `l5_cognition/nt_core/reasoning/nt_core_planning.rs:86`, `l3_embodiment/nt_shield/nt_shield_impl/nt_shield_swarm.rs:80`

**建议**: 分层定义——L1 `nt_act::task_types`、L5 `nt_core::task_types`，跨层引用。

### 6. SourceType — 7 处定义

`core/nt_core_context/context_budget.rs:12`, `core/nt_core_accessor.rs:5`, `core/nt_core_answer_engine.rs:12`, `l2_perception/nt_world/osint/self_curriculum/mod.rs:8`, `l1_action/nt_act/nt_act_crypto/opportunity.rs:83`, `l5_cognition/nt_mind/nt_mind/knowledge/knowledge_engine/types.rs:6`, `l3_embodiment/nt_shield/nt_shield_osint.rs:60`

---

## 二、大文件 Top 20 (>1500行)

| 行数 | 文件 | 风险 |
|------|------|------|
| 4118 | `bin/experience.rs` | 🔴 超大二进制 |
| 3349 | `core/nt_core_consciousness_core.rs` | 🔴 需拆分 |
| 3219 | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | 🔴 |
| 3192 | `entry/mod.rs` | 🔴 入口过重 |
| 3091 | `l1_action/nt_memory/nt_memory_kb/mod.rs` | 🔴 |
| 2628 | `l5_cognition/nt_mind/nt_mind_skill_engine.rs` | 🟡 |
| 2509 | `l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs` | 🟡 |
| 2327 | `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 🟡 |
| 2270 | `l2_perception/nt_world/nt_world_video_pipeline.rs` | 🟡 |
| 2228 | `l1_action/nt_memory/nt_memory_kb/nt_memory_graphrag/mod.rs` | 🟡 |
| 2178 | `core/nt_core_mcp.rs` | 🟡 |
| 2149 | `core/nt_core_gate/mod.rs` | 🟡 |
| 2142 | `l6_meta/healing/nt_mind_eval_harness.rs` | 🟡 |
| 2140 | `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | 🟡 |
| 2134 | `l1_action/nt_memory/nt_memory_kb/nt_memory_search.rs` | 🟡 |
| 2055 | `l1_action/nt_memory/nt_memory_kb/nt_memory_geo.rs` | 🟡 |
| 2029 | `core/nt_core_e8/nt_core_community_ingester.rs` | 🟡 |
| 2004 | `neotrix/nt_file_ability.rs` | 🟡 |
| 1995 | `core/nt_core_deploy.rs` | 🟡 |
| 1970 | `l5_cognition/nt_mind/foundation/cleanup_engine.rs` | 🟡 |

**阈值**: >3000 行建议立即拆分；>2000 行建议拆分。

---

## 三、可能空/死模块 (<20行)

| 文件 | 行数 | 判断 |
|------|------|------|
| `core/energy_core/seal_pipeline.rs` | 6 | 🔴 疑似残留 |
| `core/nt_core_prm/mod.rs` | 12 | 🔴 疑似残留 |
| `core/l2_perception/mod.rs` | 11 | 🟡 可能仅 re-export |
| `core/nt_core_aura/mod.rs` | 11 | 🔴 疑似残留 |
| `core/nt_core_consciousness_tree/selftest.rs` | 10 | 🟡 可能仅 re-export |
| `server/mod.rs` | 5 | 🔴 疑似残留 |
| `cli/tui/app/mod.rs` | 8 | 🟡 |
| `l2_perception/nt_world/source/video/mod.rs` | 3 | 🔴 疑似残留 |
| `l2_perception/nt_world/source/text/lyrics/mod.rs` | 3 | 🔴 疑似残留 |
| `l2_perception/nt_world/source/text/document/mod.rs` | 2 | 🔴 疑似残留 |
| `l2_perception/nt_world/source/text/social/mod.rs` | 4 | 🔴 疑似残留 |
| `l2_perception/nt_world/source/text/book/mod.rs` | 2 | 🔴 疑似残留 |
| `bin/uniffi-bindgen.rs` | 7 | 🟡 生成代码 |

---

## 四、unified_archive vs 活跃路径重复

**统计**: unified_archive 含 **98 个 .rs 文件** (4,769 行)，与活跃路径存在大量同名重复:

| 归档路径 | 活跃路径 |
|----------|----------|
| `unified_archive/.../book/annas_archive.rs` | `l2_perception/nt_world/source/text/book/annas_archive.rs` |
| `unified_archive/.../document/arxiv.rs` | `l2_perception/nt_world/source/text/document/arxiv.rs` |
| `unified_archive/.../audio/bandcamp.rs` | `l2_perception/nt_world/source/audio/bandcamp.rs` |
| `unified_archive/.../video/bilibili.rs` | `l2_perception/nt_world/source/video/bilibili.rs` |
| `unified_archive/.../core/cache_warmer.rs` | `l2_perception/nt_world/source/cache_warmer.rs` |
| `unified_archive/.../core/crypto.rs` | `l2_perception/nt_world/source/crypto.rs` |
| `unified_archive/.../audio/deezer.rs` | `l2_perception/nt_world/source/audio/deezer.rs` |
| `unified_archive/.../core/evolution_bridge.rs` | `l2_perception/nt_world/source/evolution_bridge.rs` |
| `unified_archive/.../core/evolution_decision.rs` | `l2_perception/nt_world/source/evolution_decision.rs` |
| `unified_archive/.../core/evolution_metrics.rs` | `l2_perception/nt_world/source/evolution_metrics.rs` |
| `unified_archive/.../lyrics/genius.rs` | `l2_perception/nt_world/source/text/lyrics/genius.rs` |

**结论**: unified_archive 是旧路径的完整快照，4,769 行可安全清理。

---

## 五、架构冗余模式总结

| 模式 | 影响 | 优先级 |
|------|------|--------|
| 类型碎片化 (SearchResult×13, RiskLevel×13, GraphNode×8) | 编译慢、维护成本高、跨域转换开销 | P0 |
| unified_archive 残留 | 98 文件 4,769 行死代码 | P1 |
| 超大文件 (>3000行×4) | 认知负担、合并冲突 | P1 |
| 空/死模块 (<20行×13) | 编译噪音 | P2 |
| 同名枚举跨层重复 (Domain×6, TaskType×8, TaskStatus×8, SourceType×7) | 类型转换 boilerplate | P0 |

---

## 六、推荐治理路径

1. **P0 类型统一**: 创建 `core::nt_core_types` 事实源，定义 SearchResult/Domain/GraphNode/GraphEdge，各模块 `use` 引用
2. **P1 归档清理**: 删除 `unified_archive/` 目录 (4,769 行)
3. **P1 大文件拆分**: 对 >3000 行文件按职责域拆分
4. **P2 死模块清理**: 对 <20 行模块逐一审查，删除无消费者的残留
